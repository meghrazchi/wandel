# Connect Wandel MCP to Google Antigravity

Wandel works natively with **Google Antigravity** over `stdio` or Local HTTP (`--http`) as a headless, parallel Rust MCP server. Connecting Wandel gives Antigravity full offline control over images, videos, audio, and PDFs without opening the Wandel desktop UI.

---

## 1. Install the Headless `wandel` CLI

- **From the Desktop installer (macOS `.dmg` / Windows / Linux):** Install and launch **Wandel** once; it bundles `wandel-cli` and automatically links `~/.local/bin/wandel`.
- **Standalone one-line installer (macOS & Linux):**
  ```bash
  curl -fsSL https://raw.githubusercontent.com/meghrazchi/wandel/main/packaging/install.sh | bash
  ```
- **Verify:**
  ```bash
  which wandel && wandel info
  ```

---

## 2. Connect to Antigravity

### Option A: Stdio Transport (Recommended for Local Mac/Linux/Windows)

Open Antigravity's MCP configuration file (typically `~/.gemini/antigravity/mcp_config.json`, or open **MCP Servers -> Manage MCP Servers -> View raw config** in Antigravity):

```json
{
  "mcpServers": {
    "wandel": {
      "command": "/Users/YOUR_USERNAME/.local/bin/wandel",
      "args": ["mcp"]
    }
  }
}
```

To restrict file access to specific project directories, add `--allowed-dir`:

```json
{
  "mcpServers": {
    "wandel": {
      "command": "/Users/YOUR_USERNAME/.local/bin/wandel",
      "args": [
        "mcp",
        "--allowed-dir",
        "/Users/YOUR_USERNAME/Documents/projects"
      ]
    }
  }
}
```

### Option B: Local HTTP Transport

Start the Wandel HTTP MCP server:

```bash
wandel mcp --http 127.0.0.1:3847 --token my-secret-token --allowed-dir "$HOME/Documents/projects"
```

Add the HTTP server entry to `mcp_config.json`:

```json
{
  "mcpServers": {
    "wandel": {
      "serverUrl": "http://127.0.0.1:3847/mcp",
      "headers": {
        "Authorization": "Bearer my-secret-token"
      }
    }
  }
}
```

---

## 3. Verify & Use in Antigravity

Refresh MCP servers in Antigravity (or restart the session) and verify the 7 Wandel tools appear:
- `wandel_info`
- `wandel_inspect`
- `wandel_convert`
- `wandel_pdf_ops`
- `wandel_image_tools`
- `wandel_media_tools`
- `wandel_pipeline`

### Example Prompts
- *"Use Wandel to generate a PWA icon pack and Web LQIP placeholders for `./public/logo.svg`."*
- *"Create a visual pixel diff between `baseline.png` and `current.png` with Wandel."*
- *"Use `wandel_pdf_ops` to remove blank pages, interleave odd/even duplex scans, and update the PDF metadata."*

See [docs/MCP.md](../MCP.md) for the complete 7-tool schema and pipeline reference.
