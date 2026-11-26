use crate::protocol::{
    get_prompt, prompt_definitions, resource_definitions, tool_definitions, JsonRpcRequest,
    JsonRpcResponse, RequestId, PROTOCOL_VERSION, SERVER_NAME, SERVER_VERSION,
};
use crate::tools::{execute_tool, McpContext};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};

/// Lightweight bounded concurrency semaphore for parallel JSON-RPC tool execution.
struct Semaphore {
    state: Mutex<usize>,
    cvar: Condvar,
    limit: usize,
}

impl Semaphore {
    fn new(limit: usize) -> Self {
        Self {
            state: Mutex::new(0),
            cvar: Condvar::new(),
            limit: limit.max(1),
        }
    }

    fn acquire(&self) {
        let mut active = self.state.lock().unwrap();
        while *active >= self.limit {
            active = self.cvar.wait(active).unwrap();
        }
        *active += 1;
    }

    fn release(&self) {
        let mut active = self.state.lock().unwrap();
        *active = active.saturating_sub(1);
        self.cvar.notify_one();
    }
}

/// High-performance, concurrent MCP server for Wandel.
/// Works headless over stdio (`wandel mcp`) and provides in-process health checks for the Desktop UI.
pub struct McpServer {
    ctx: Arc<McpContext>,
    max_concurrent_rpcs: usize,
}

impl McpServer {
    pub fn new(ffmpeg_path: Option<PathBuf>, ffprobe_path: Option<PathBuf>) -> Self {
        Self::with_allowed_dirs(ffmpeg_path, ffprobe_path, Vec::new())
    }

    pub fn with_allowed_dirs(
        ffmpeg_path: Option<PathBuf>,
        ffprobe_path: Option<PathBuf>,
        allowed_dirs: Vec<PathBuf>,
    ) -> Self {
        let ctx = Arc::new(McpContext::with_allowed_dirs(
            ffmpeg_path,
            ffprobe_path,
            allowed_dirs,
        ));
        let max_concurrent_rpcs = ctx.hardware.cpu_cores.clamp(4, 16);
        Self {
            ctx,
            max_concurrent_rpcs,
        }
    }

    /// Executes a `wandel_pipeline` JSON recipe directly (used by `wandel pipeline <recipe.json>`).
    pub fn run_pipeline_recipe(&self, recipe: &Value) -> (Value, bool) {
        let outcome = execute_tool(
            &self.ctx,
            "wandel_pipeline",
            recipe,
            Arc::new(AtomicBool::new(false)),
            |_, _, _| {},
        );
        (outcome.payload, outcome.is_error)
    }

    /// Returns an in-process health & discovery snapshot for the Wandel Desktop UI.
    pub fn health_check(&self) -> Value {
        json!({
            "serverName": SERVER_NAME,
            "serverVersion": SERVER_VERSION,
            "protocolVersion": PROTOCOL_VERSION,
            "maxConcurrentRpcs": self.max_concurrent_rpcs,
            "capabilities": self.ctx.capabilities_json(),
            "tools": tool_definitions()
        })
    }

    /// Runs the MCP server over standard input/output (newline-delimited JSON-RPC 2.0).
    pub fn run_stdio(&self) -> io::Result<()> {
        let stdin = io::stdin();
        let stdout = io::stdout();
        self.run_io(stdin.lock(), stdout)
    }

    /// Runs the MCP server over any `BufRead` / `Write` pair (used by both stdio and integration tests).
    /// Dispatches `tools/call` concurrently up to `max_concurrent_rpcs` while serializing output frames.
    pub fn run_io<R: BufRead, W: Write + Send + 'static>(
        &self,
        reader: R,
        writer: W,
    ) -> io::Result<()> {
        let writer = Arc::new(Mutex::new(writer));
        let cancels: Arc<Mutex<HashMap<RequestId, Arc<AtomicBool>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let sem = Arc::new(Semaphore::new(self.max_concurrent_rpcs));
        let mut handles = Vec::new();

        for line_res in reader.lines() {
            let line = line_res?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let req: JsonRpcRequest = match serde_json::from_str(trimmed) {
                Ok(r) => r,
                Err(err) => {
                    // JSON-RPC Parse error (-32700)
                    let msg = json!({
                        "jsonrpc": "2.0",
                        "id": Value::Null,
                        "error": { "code": -32700, "message": format!("Parse error: {}", err) }
                    });
                    let _ = write_json_line(&writer, &msg);
                    continue;
                }
            };

            // Handle notifications (no id) immediately
            if req.id.is_none() {
                if req.method == "notifications/cancelled" {
                    if let Some(id_val) = req.params.get("requestId") {
                        if let Ok(req_id) = serde_json::from_value::<RequestId>(id_val.clone()) {
                            if let Ok(guard) = cancels.lock() {
                                if let Some(flag) = guard.get(&req_id) {
                                    flag.store(true, Ordering::Relaxed);
                                }
                            }
                        }
                    }
                }
                continue;
            }

            let req_id = req.id.unwrap();

            // Fast-path synchronous metadata/handshake RPCs directly on the reader thread
            match req.method.as_str() {
                "initialize" => {
                    let resp = JsonRpcResponse::ok(
                        req_id,
                        json!({
                            "protocolVersion": PROTOCOL_VERSION,
                            "capabilities": {
                                "tools": { "listChanged": false },
                                "resources": { "subscribe": false, "listChanged": false },
                                "prompts": { "listChanged": false }
                            },
                            "serverInfo": {
                                "name": SERVER_NAME,
                                "version": SERVER_VERSION
                            },
                            "instructions": "Wandel is a fast parallel media converter (Images, Video, Audio, PDF). Prefer batching multiple files/folders into a single wandel_convert or wandel_inspect call for maximum CPU+GPU parallelism."
                        }),
                    );
                    let _ = write_json_line(&writer, &resp);
                }
                "ping" => {
                    let _ = write_json_line(&writer, &JsonRpcResponse::ok(req_id, json!({})));
                }
                "tools/list" => {
                    let _ = write_json_line(
                        &writer,
                        &JsonRpcResponse::ok(req_id, json!({ "tools": tool_definitions() })),
                    );
                }
                "resources/list" => {
                    let _ = write_json_line(
                        &writer,
                        &JsonRpcResponse::ok(
                            req_id,
                            json!({ "resources": resource_definitions() }),
                        ),
                    );
                }
                "resources/read" => {
                    let uri = req
                        .params
                        .get("uri")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let resp = if uri == "wandel://capabilities" {
                        let text = serde_json::to_string_pretty(&self.ctx.capabilities_json())
                            .unwrap_or_default();
                        JsonRpcResponse::ok(
                            req_id,
                            json!({
                                "contents": [{
                                    "uri": uri,
                                    "mimeType": "application/json",
                                    "text": text
                                }]
                            }),
                        )
                    } else {
                        JsonRpcResponse::err(req_id, -32602, format!("Unknown resource: {}", uri))
                    };
                    let _ = write_json_line(&writer, &resp);
                }
                "prompts/list" => {
                    let _ = write_json_line(
                        &writer,
                        &JsonRpcResponse::ok(req_id, json!({ "prompts": prompt_definitions() })),
                    );
                }
                "prompts/get" => {
                    let name = req
                        .params
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let args = req.params.get("arguments").cloned().unwrap_or(Value::Null);
                    let resp = match get_prompt(name, &args) {
                        Some(p) => JsonRpcResponse::ok(req_id, p),
                        None => {
                            JsonRpcResponse::err(req_id, -32602, format!("Unknown prompt: {}", name))
                        }
                    };
                    let _ = write_json_line(&writer, &resp);
                }
                "tools/call" => {
                    let ctx = Arc::clone(&self.ctx);
                    let writer_ref = Arc::clone(&writer);
                    let cancels_ref = Arc::clone(&cancels);
                    let sem_ref = Arc::clone(&sem);

                    let cancel_flag = Arc::new(AtomicBool::new(false));
                    if let Ok(mut guard) = cancels.lock() {
                        guard.insert(req_id.clone(), Arc::clone(&cancel_flag));
                    }

                    let progress_token = req
                        .params
                        .get("_meta")
                        .and_then(|m| m.get("progressToken"))
                        .cloned();
                    let tool_name = req
                        .params
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let tool_args = req
                        .params
                        .get("arguments")
                        .cloned()
                        .unwrap_or_else(|| json!({}));

                    // Dispatch concurrently on a bounded worker thread so parallel LLM tool calls run simultaneously
                    sem_ref.acquire();
                    let handle = std::thread::spawn(move || {
                        let writer_prog = Arc::clone(&writer_ref);
                        let outcome = execute_tool(
                            &ctx,
                            &tool_name,
                            &tool_args,
                            cancel_flag,
                            move |progress, total, message| {
                                if let Some(ref token) = progress_token {
                                    let notif = json!({
                                        "jsonrpc": "2.0",
                                        "method": "notifications/progress",
                                        "params": {
                                            "progressToken": token,
                                            "progress": progress,
                                            "total": total,
                                            "message": message
                                        }
                                    });
                                    let _ = write_json_line(&writer_prog, &notif);
                                }
                            },
                        );

                        if let Ok(mut guard) = cancels_ref.lock() {
                            guard.remove(&req_id);
                        }

                        let resp = JsonRpcResponse::ok(req_id, outcome.to_mcp_result());
                        let _ = write_json_line(&writer_ref, &resp);
                        sem_ref.release();
                    });
                    handles.push(handle);
                }
                other => {
                    let _ = write_json_line(
                        &writer,
                        &JsonRpcResponse::err(req_id, -32601, format!("Method not found: {}", other)),
                    );
                }
            }
        }

        // Wait cleanly for all in-flight parallel tool calls to finish before exiting
        for h in handles {
            let _ = h.join();
        }
        Ok(())
    }
}

fn write_json_line<W: Write, T: serde::Serialize>(
    writer: &Arc<Mutex<W>>,
    payload: &T,
) -> io::Result<()> {
    let serialized = serde_json::to_string(payload)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let mut guard = writer
        .lock()
        .map_err(|_| io::Error::other("MCP stdout mutex poisoned"))?;
    writeln!(guard, "{}", serialized)?;
    guard.flush()
}
