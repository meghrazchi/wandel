use crate::protocol::{JsonRpcRequest, PROTOCOL_VERSION, SERVER_NAME, SERVER_VERSION};
use crate::server::McpServer;
use serde_json::json;
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

const MAX_BODY_BYTES: usize = 16 * 1024 * 1024;

/// Binds a local HTTP/1.1 MCP JSON-RPC server on `bind_addr` (e.g. `"127.0.0.1:3847"`).
/// Optionally enforces `Authorization: Bearer <token>` on `POST /mcp`.
pub fn serve_http(
    server: Arc<McpServer>,
    bind_addr: &str,
    bearer_token: Option<String>,
) -> io::Result<()> {
    let listener = TcpListener::bind(bind_addr)?;
    let actual = listener.local_addr()?;
    eprintln!(
        "Wandel MCP HTTP server listening on http://{} (POST /mcp, GET /health){}",
        actual,
        if bearer_token.is_some() {
            " [bearer auth enabled]"
        } else {
            ""
        }
    );
    serve_http_on_listener(server, listener, bearer_token, Arc::new(AtomicBool::new(false)))
}

/// Runs the HTTP MCP server on a pre-bound `TcpListener` until `shutdown` becomes `true`.
pub fn serve_http_on_listener(
    server: Arc<McpServer>,
    listener: TcpListener,
    bearer_token: Option<String>,
    shutdown: Arc<AtomicBool>,
) -> io::Result<()> {
    listener.set_nonblocking(true)?;
    let token = bearer_token.map(Arc::new);

    while !shutdown.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _)) => {
                let srv = Arc::clone(&server);
                let tok = token.clone();
                std::thread::spawn(move || {
                    let _ = handle_connection(stream, &srv, tok.as_deref().map(String::as_str));
                });
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(15));
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn handle_connection(
    stream: TcpStream,
    server: &McpServer,
    expected_token: Option<&str>,
) -> io::Result<()> {
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream;

    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(());
    }
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return write_http_response(
            &mut writer,
            400,
            "Bad Request",
            "application/json",
            br#"{"error":"Malformed HTTP request line"}"#,
        );
    }
    let method = parts[0];
    let path = parts[1].split('?').next().unwrap_or(parts[1]);

    // Read headers
    let mut headers: HashMap<String, String> = HashMap::new();
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
        }
    }

    if method == "OPTIONS" {
        return write_http_response(&mut writer, 204, "No Content", "text/plain", b"");
    }

    if method == "GET" && path == "/health" {
        let body = serde_json::to_vec(&json!({
            "status": "ok",
            "serverName": SERVER_NAME,
            "version": SERVER_VERSION,
            "protocolVersion": PROTOCOL_VERSION,
            "transport": "http"
        }))
        .unwrap_or_default();
        return write_http_response(&mut writer, 200, "OK", "application/json", &body);
    }

    if method != "POST" || path != "/mcp" {
        return write_http_response(
            &mut writer,
            404,
            "Not Found",
            "application/json",
            br#"{"error":"Not found. Use POST /mcp or GET /health"}"#,
        );
    }

    // Optional Bearer token authentication
    if let Some(expected) = expected_token {
        let authorized = headers
            .get("authorization")
            .and_then(|v| v.strip_prefix("Bearer ").or_else(|| v.strip_prefix("bearer ")))
            .map(|provided| provided == expected)
            .unwrap_or(false);
        if !authorized {
            return write_http_response(
                &mut writer,
                401,
                "Unauthorized",
                "application/json",
                br#"{"error":"Missing or invalid Authorization Bearer token"}"#,
            );
        }
    }

    let content_length: usize = headers
        .get("content-length")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if content_length > MAX_BODY_BYTES {
        return write_http_response(
            &mut writer,
            413,
            "Payload Too Large",
            "application/json",
            br#"{"error":"Request body exceeds 16 MB limit"}"#,
        );
    }

    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body)?;

    let req: JsonRpcRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(err) => {
            let err_json = serde_json::to_vec(&json!({
                "jsonrpc": "2.0",
                "id": serde_json::Value::Null,
                "error": { "code": -32700, "message": format!("Parse error: {}", err) }
            }))
            .unwrap_or_default();
            return write_http_response(&mut writer, 200, "OK", "application/json", &err_json);
        }
    };

    match server.handle_single_request(req) {
        Some(resp) => {
            let resp_bytes = serde_json::to_vec(&resp).unwrap_or_default();
            write_http_response(&mut writer, 200, "OK", "application/json", &resp_bytes)
        }
        None => write_http_response(&mut writer, 202, "Accepted", "text/plain", b""),
    }
}

fn write_http_response(
    writer: &mut TcpStream,
    status_code: u16,
    reason: &str,
    content_type: &str,
    body: &[u8],
) -> io::Result<()> {
    let header = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: Content-Type, Authorization\r\nAccess-Control-Allow-Methods: POST, GET, OPTIONS\r\nConnection: close\r\n\r\n",
        status_code,
        reason,
        content_type,
        body.len()
    );
    writer.write_all(header.as_bytes())?;
    writer.write_all(body)?;
    writer.flush()
}
