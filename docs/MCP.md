# Wandel MCP Server Guide

Wandel includes a native, zero-bloat **Model Context Protocol (MCP)** server (`crates/wandel-mcp`) that works **completely headless** over `stdio` (no GUI or display required) and integrates directly with the Wandel Desktop UI.

---

## Running Headless

```bash
# Unrestricted local access (stdio)
wandel mcp

# Production sandbox: restrict all reads & writes to approved folders (repeatable)
wandel mcp --allowed-dir /path/to/workspace --allowed-dir /tmp/exports

# Local HTTP JSON-RPC transport (for Docker, n8n, scripts, or HTTP MCP clients)
wandel mcp --http 127.0.0.1:3847 --token my-secret-token --allowed-dir /path/to/workspace
```

### Local HTTP Endpoints (`--http`)

When started with `--http <addr>` (recommended: loopback `127.0.0.1:<port>`):
- **`GET /health`** — returns `{ "status": "ok", "serverName": "wandel-mcp", "version": "...", "transport": "http" }`.
- **`POST /mcp`** — accepts standard JSON-RPC 2.0 MCP requests (`initialize`, `tools/list`, `tools/call`, `resources/*`, `prompts/*`).
- **`--token <secret>`** — optional bearer auth requiring `Authorization: Bearer <secret>` on every `POST /mcp` call.

### Client Configuration (Antigravity, Claude Desktop, Cursor, Windsurf)

```json
{
  "mcpServers": {
    "wandel": {
      "command": "wandel",
      "args": ["mcp", "--allowed-dir", "/path/to/workspace"]
    }
  }
}
```

> **Tip:** Open Wandel Desktop and click **MCP** in the top header (or press `⌘K → Open MCP Server Integration`) to verify server health, pick a sandbox folder, and copy the exact JSON config for your system.

---

## Exposed Tools (7 Cohesive, Batch-First Tools)

| Tool | Read-Only | Purpose |
| :--- | :---: | :--- |
| `wandel_info` | Yes | Instant cached CPU cores, GPU encoders, FFmpeg status, sandbox roots, and supported formats. |
| `wandel_inspect` | Yes | Parallel inspection of files or folders (dimensions, duration, codecs, EXIF/GPS warnings, PDF page count). Thumbnails omitted by default to keep LLM context compact. |
| `wandel_convert` | No | Parallel batch converter for images, videos, audio, and PDFs. Supports `preset`, `to`, `quality`, resize, `autoTrim`, `autoLevels`, `filter`, `framePaddingPx`, `textWatermark`/`tiledWatermark`, `videoReframe`, `playbackMode`, `remux`, `denoiseAudio`, `deinterlace`, `videoSharpen`, `gainDb`, `speed`, and `dryRun: true`. |
| `wandel_pdf_ops` | No | Pure-Rust PDF toolkit (`action`: `merge_images`, `concat`, `split`, `compress`, `rotate`, `reorder`, `stamp`, `number`, `extract_images`, `scrub_metadata`). |
| `wandel_image_tools` | No | Dev & creative image utilities (`action`: `icon_pack`, `contact_sheet`, `compare`, `srcset`, `slice`, `sprite_sheet`, `palette`, `dedupe`, `qr`, `data_url`). |
| `wandel_media_tools` | No | Video & audio utilities (`action`: `merge_videos`, `merge_audio`, `storyboard`, `extract_frames`, `waveform`, `slideshow`, `audiogram`, `split_segments`). |
| `wandel_pipeline` | No | Multi-step composable pipeline executing dependent stages in one call with `"$prev"` and `"$step.N"` output piping. |

### Resources & Prompts
- **Resource:** `wandel://capabilities` (`application/json`)
- **Prompts:** `optimize-for-web`, `prepare-social-video`, `clean-pdf-for-sharing`

---

## Composable Pipeline Example (`wandel_pipeline` & `wandel pipeline`)

Save as `recipe.json` and run via `wandel pipeline recipe.json` (or pass directly to `wandel_pipeline` via MCP):

```json
{
  "steps": [
    {
      "tool": "wandel_convert",
      "arguments": {
        "inputs": ["./photos"],
        "to": "png",
        "autoTrim": true,
        "autoLevels": true,
        "textWatermark": "DRAFT",
        "tiledWatermark": true,
        "outputDir": "./processed"
      }
    },
    {
      "tool": "wandel_pdf_ops",
      "arguments": {
        "action": "merge_images",
        "inputs": "$prev",
        "output": "./proof_book.pdf",
        "pageFit": "a4"
      }
    },
    {
      "tool": "wandel_pdf_ops",
      "arguments": {
        "action": "number",
        "input": "$prev",
        "output": "./proof_book_final.pdf",
        "text": "Page {n} of {total}"
      }
    }
  ]
}
```
