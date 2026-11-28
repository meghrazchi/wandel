# Connect Wandel MCP to OpenAI Codex

Wandel integrates with **OpenAI Codex CLI** (and Codex IDE extensions) as a fast, offline, headless **Model Context Protocol (MCP)** server (`wandel mcp`). All image, video, audio, and PDF processing runs locally on your machine with zero uploads and never opens the Wandel UI.

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

## 2. Connect to Codex CLI

### Option A: `codex mcp add` Command (Recommended)

```bash
codex mcp add wandel -- "$(which wandel)" mcp
```

With optional directory sandboxing (`--allowed-dir` is repeatable):

```bash
codex mcp add wandel -- "$(which wandel)" mcp --allowed-dir "$HOME/Documents" --allowed-dir "$HOME/Downloads"
```

### Option B: Manual `~/.codex/config.toml`

Open `~/.codex/config.toml` and add:

```toml
[mcp_servers.wandel]
command = "/Users/YOUR_USERNAME/.local/bin/wandel"
args = ["mcp"]
```

Or with directory sandboxing:

```toml
[mcp_servers.wandel]
command = "/Users/YOUR_USERNAME/.local/bin/wandel"
args = ["mcp", "--allowed-dir", "/Users/YOUR_USERNAME/projects"]
```

### Option C: Local HTTP MCP Transport (Docker / Remote Agents)

Start Wandel's local HTTP MCP server in a terminal or container:

```bash
wandel mcp --http 127.0.0.1:3847 --token my-secret-token --allowed-dir "$HOME/projects"
```

Then configure `~/.codex/config.toml`:

```toml
[mcp_servers.wandel]
url = "http://127.0.0.1:3847/mcp"
bearer_token_env_var = "WANDEL_MCP_TOKEN"
```

---

## 3. Verify in Codex

```bash
codex mcp list
```

Confirm `wandel` is listed and enabled. In a Codex session, ask:
- *"Use `wandel_info` to check available CPU cores and GPU encoders."*
- *"Convert all images in `./assets` to WebP + AVIF and generate a responsive `srcset` bundle."*
- *"Extract `./spec.pdf` to clean Markdown with YAML frontmatter."*

See [docs/MCP.md](../MCP.md) for the complete 7-tool schema and composable pipeline guide.
