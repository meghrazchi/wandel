use image::{DynamicImage, ImageBuffer, Rgba};
use serde_json::{json, Value};
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use wandel_mcp::McpServer;

struct SharedBuf(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for SharedBuf {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn mcp_handshake_and_parallel_tool_calls() {
    let dir = tempdir().unwrap();
    let img_a = dir.path().join("a.png");
    let img_b = dir.path().join("b.png");
    DynamicImage::ImageRgba8(ImageBuffer::from_pixel(64, 48, Rgba([100, 150, 220, 255])))
        .save(&img_a)
        .unwrap();
    DynamicImage::ImageRgba8(ImageBuffer::from_pixel(64, 48, Rgba([220, 100, 120, 255])))
        .save(&img_b)
        .unwrap();
    let out_dir = dir.path().join("out");
    let pdf_out = dir.path().join("merged.pdf");
    let qr_out = dir.path().join("qr.svg");

    let requests = vec![
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26"}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        json!({"jsonrpc":"2.0","id":3,"method":"resources/read","params":{"uri":"wandel://capabilities"}}),
        json!({"jsonrpc":"2.0","id":4,"method":"prompts/list"}),
        // Burst of 5 concurrent tool calls in parallel:
        json!({"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"wandel_info","arguments":{}}}),
        json!({"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"wandel_inspect","arguments":{"inputs":[img_a.to_string_lossy(), img_b.to_string_lossy()]}}}),
        json!({"jsonrpc":"2.0","id":12,"method":"tools/call","params":{"name":"wandel_convert","arguments":{"inputs":[img_a.to_string_lossy(), img_b.to_string_lossy()],"to":"webp","outputDir":out_dir.to_string_lossy()},"_meta":{"progressToken":"tok-1"}}}),
        json!({"jsonrpc":"2.0","id":13,"method":"tools/call","params":{"name":"wandel_pdf_ops","arguments":{"action":"merge_images","inputs":[img_a.to_string_lossy(), img_b.to_string_lossy()],"output":pdf_out.to_string_lossy()}}}),
        json!({"jsonrpc":"2.0","id":14,"method":"tools/call","params":{"name":"wandel_image_tools","arguments":{"action":"qr","content":"https://wandel.app","output":qr_out.to_string_lossy()}}}),
        // Structured error check for unknown tool
        json!({"jsonrpc":"2.0","id":15,"method":"tools/call","params":{"name":"nonexistent_tool","arguments":{}}}),
    ];

    let input_stream = requests
        .into_iter()
        .map(|v| serde_json::to_string(&v).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    let out_buf = Arc::new(Mutex::new(Vec::new()));
    let server = McpServer::new(None, None);
    server
        .run_io(Cursor::new(input_stream.into_bytes()), SharedBuf(Arc::clone(&out_buf)))
        .unwrap();

    let raw_out = String::from_utf8(out_buf.lock().unwrap().clone()).unwrap();
    let messages: Vec<Value> = raw_out
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("Every MCP stdout line must be valid JSON"))
        .collect();

    let by_id = |id: i64| -> Value {
        messages
            .iter()
            .find(|m| m.get("id").and_then(|v| v.as_i64()) == Some(id))
            .cloned()
            .unwrap_or_else(|| panic!("Missing MCP response for id={}", id))
    };

    // Handshake & discovery
    assert_eq!(by_id(1)["result"]["serverInfo"]["name"], "wandel-mcp");
    assert_eq!(by_id(2)["result"]["tools"].as_array().unwrap().len(), 7);
    assert!(by_id(3)["result"]["contents"][0]["text"]
        .as_str()
        .unwrap()
        .contains("pdfPureRust"));
    assert_eq!(by_id(4)["result"]["prompts"].as_array().unwrap().len(), 3);

    // Parallel tool call outputs
    assert_eq!(by_id(10)["result"]["isError"], false);
    assert_eq!(by_id(11)["result"]["structuredContent"]["count"], 2);
    assert_eq!(by_id(12)["result"]["isError"], false);
    assert_eq!(
        by_id(12)["result"]["structuredContent"]["summary"]["completed"],
        2
    );
    assert_eq!(by_id(13)["result"]["isError"], false);
    assert!(pdf_out.exists());
    assert_eq!(by_id(14)["result"]["isError"], false);
    assert!(qr_out.exists());

    // Progress notifications emitted for progressToken "tok-1"
    assert!(messages
        .iter()
        .any(|m| m["method"] == "notifications/progress" && m["params"]["progressToken"] == "tok-1"));

    // Structured error response
    assert_eq!(by_id(15)["result"]["isError"], true);
}

#[test]
fn mcp_pipeline_sandbox_and_dry_run() {
    let allowed = tempdir().unwrap();
    let outside = tempdir().unwrap();

    let img_a = allowed.path().join("one.png");
    let img_b = allowed.path().join("two.png");
    let forbidden = outside.path().join("secret.png");

    DynamicImage::ImageRgba8(ImageBuffer::from_pixel(80, 60, Rgba([40, 120, 220, 255])))
        .save(&img_a)
        .unwrap();
    DynamicImage::ImageRgba8(ImageBuffer::from_pixel(80, 60, Rgba([220, 90, 60, 255])))
        .save(&img_b)
        .unwrap();
    DynamicImage::ImageRgba8(ImageBuffer::from_pixel(32, 32, Rgba([10, 10, 10, 255])))
        .save(&forbidden)
        .unwrap();

    let conv_dir = allowed.path().join("conv");
    let merged_pdf = allowed.path().join("album.pdf");
    let stamped_pdf = allowed.path().join("album_stamped.pdf");
    let dry_dir = allowed.path().join("dry_out");

    let requests = vec![
        // 1. Dry-run convert: should plan outputs without creating dry_dir
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "wandel_convert",
                "arguments": {
                    "inputs": [img_a.to_string_lossy()],
                    "to": "webp",
                    "outputDir": dry_dir.to_string_lossy(),
                    "dryRun": true
                }
            }
        }),
        // 2. Sandbox block: reading outside allowed_dir must fail
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "wandel_inspect",
                "arguments": {
                    "inputs": [forbidden.to_string_lossy()]
                }
            }
        }),
        // 3. 3-step wandel_pipeline using $prev chaining inside allowed_dir
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "wandel_pipeline",
                "arguments": {
                    "steps": [
                        {
                            "tool": "wandel_convert",
                            "arguments": {
                                "inputs": [img_a.to_string_lossy(), img_b.to_string_lossy()],
                                "to": "png",
                                "autoLevels": true,
                                "textWatermark": "DRAFT",
                                "tiledWatermark": true,
                                "outputDir": conv_dir.to_string_lossy()
                            }
                        },
                        {
                            "tool": "wandel_pdf_ops",
                            "arguments": {
                                "action": "merge_images",
                                "inputs": "$prev",
                                "output": merged_pdf.to_string_lossy(),
                                "pageFit": "a4"
                            }
                        },
                        {
                            "tool": "wandel_pdf_ops",
                            "arguments": {
                                "action": "stamp",
                                "input": "$prev",
                                "output": stamped_pdf.to_string_lossy(),
                                "text": "CONFIDENTIAL"
                            }
                        }
                    ]
                }
            }
        }),
    ];

    let input_stream = requests
        .into_iter()
        .map(|v| serde_json::to_string(&v).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    let out_buf = Arc::new(Mutex::new(Vec::new()));
    let server = McpServer::with_allowed_dirs(None, None, vec![allowed.path().to_path_buf()]);
    server
        .run_io(
            Cursor::new(input_stream.into_bytes()),
            SharedBuf(Arc::clone(&out_buf)),
        )
        .unwrap();

    let raw_out = String::from_utf8(out_buf.lock().unwrap().clone()).unwrap();
    let messages: Vec<Value> = raw_out
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let by_id = |id: i64| -> Value {
        messages
            .iter()
            .find(|m| m.get("id").and_then(|v| v.as_i64()) == Some(id))
            .cloned()
            .unwrap()
    };

    // Dry-run succeeded with 0 files written
    assert_eq!(by_id(1)["result"]["isError"], false);
    assert_eq!(by_id(1)["result"]["structuredContent"]["dryRun"], true);
    assert_eq!(by_id(1)["result"]["structuredContent"]["count"], 1);
    assert!(!dry_dir.exists(), "dryRun must not write output files");

    // Sandbox blocked forbidden path
    assert_eq!(by_id(2)["result"]["isError"], true);
    assert!(by_id(2)["result"]["structuredContent"]["error"]
        .as_str()
        .unwrap()
        .contains("Sandbox violation"));

    // 3-step pipeline succeeded and produced final stamped PDF
    assert_eq!(by_id(3)["result"]["isError"], false);
    assert_eq!(
        by_id(3)["result"]["structuredContent"]["completedSteps"],
        3
    );
    assert!(stamped_pdf.exists());
}
