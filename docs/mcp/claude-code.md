# Connect Wandel MCP to Claude Code & Claude Desktop

Wandel connects to **Claude Code** and **Claude Desktop** as a pure-Rust, **headless** MCP server (`wandel mcp`). It runs directly through the CLI, never opens the Wandel desktop window, and automatically uses your CPU cores and GPU encoders (Apple VideoToolbox, NVENC, QSV, VAAPI).

---

## 1. Install the Headless `wandel` CLI

Choose either option:
- **From the Wandel Desktop installer (macOS `.dmg` / Windows / Linux):** Drag `Wandel.app` to Applications and open it once. Wandel bundles `wandel-cli` inside the app and automatically symlinks `~/.local/bin/wandel`.
- **Standalone one-line installer (macOS & Linux):**
  ```bash
  curl -fsSL https://raw.githubusercontent.com/meghrazchi/wandel/main/packaging/install.sh | bash
  ```
- **Verify installation and get your exact absolute path:**
  ```bash
  which wandel && wandel info
  ```
  On macOS/Linux this is typically `/Users/<your-username>/.local/bin/wandel`.

---

## 2. Connect to Claude Code (CLI)

### Option A: One-Command Setup (Recommended)

Add Wandel to your user-level Claude Code configuration:

```bash
claude mcp add --transport stdio --scope user wandel -- "$(which wandel)" mcp
```

To restrict Wandel to specific folders (sandbox mode):

```bash
claude mcp add --transport stdio --scope user wandel -- "$(which wandel)" mcp --allowed-dir "$HOME/Documents" --allowed-dir "$HOME/Downloads"
```

### Option B: Project-Scoped `.mcp.json`

Create or update `.mcp.json` in your repository root so your whole team can share the setup:

```json
{
  "mcpServers": {
    "wandel": {
      "command": "wandel",
      "args": ["mcp", "--allowed-dir", "."]
    }
  }
}
```

### Verify in Claude Code

```bash
claude mcp list
```
Or inside a Claude Code session, run `/mcp` and confirm `wandel` shows as **connected** with 7 tools (`wandel_info`, `wandel_inspect`, `wandel_convert`, `wandel_pdf_ops`, `wandel_image_tools`, `wandel_media_tools`, `wandel_pipeline`).

---

## 3. Connect to Claude Desktop

Open your Claude Desktop config file:
- **macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

Add the `wandel` entry using the **absolute path** to `wandel` (recommended because Claude Desktop launches with a minimal system `PATH`):

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

> **Tip:** You can also open Wandel Desktop, click **MCP** in the header, and copy the pre-filled JSON with your detected absolute CLI path.

Restart **Claude Desktop** completely (`⌘Q` then reopen).

---

## 4. Example Prompts to Try

- *"Convert all PNGs in `~/Downloads/screenshots` to WebP and AVIF at quality 82, max width 1920."*
- *"Search all PDFs in `~/Documents/invoices` for `Q4 2026` using Wandel."*
- *"Create a visual pixel diff between `before.png` and `after.png` and tell me what percentage of pixels changed."*
- *"Run a Wandel pipeline: auto-trim and convert `./raw` images to PNG, merge them into an A4 PDF, remove blank pages, and stamp page numbers."*

See [docs/MCP.md](../MCP.md) for the complete 7-tool schema and pipeline reference.
