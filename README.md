# Wandel: Offline Image, Video, Audio & PDF Converter (Desktop, CLI & MCP Server)

**Convert, compress, and edit images, videos, audio, and PDFs locally in seconds. 100% private, offline, and free on macOS, Windows, and Linux.**

[![CI](https://github.com/meghrazchi/wandel/actions/workflows/ci.yml/badge.svg)](https://github.com/meghrazchi/wandel/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/meghrazchi/wandel?color=4f46e5)](https://github.com/meghrazchi/wandel/releases)
![Platforms](https://img.shields.io/badge/platforms-macOS%20%7C%20Windows%20%7C%20Linux-4f46e5)
![Stack](https://img.shields.io/badge/stack-Rust%20%7C%20Tauri%20v2%20%7C%20React%2019-4f46e5)
![MCP](https://img.shields.io/badge/MCP-Claude%20%7C%20Codex%20%7C%20Antigravity-4f46e5)
![License](https://img.shields.io/badge/license-MIT-4f46e5)

Stop uploading private photos, screen recordings, and confidential PDFs to slow, ad-filled websites. **Wandel** combines a fast **Desktop App**, a scriptable **CLI**, and a headless **Model Context Protocol (MCP) Server** powered by one shared Rust media engine.

---

## Table of Contents
- [Why Wandel](#why-wandel)
- [Capabilities at a Glance](#capabilities-at-a-glance)
- [Download & Install](#download--install)
- [Connect Wandel MCP to AI Agents](#connect-wandel-mcp-to-ai-agents)
- [Quick CLI Examples](#quick-cli-examples)
- [Keyboard Shortcuts](#keyboard-shortcuts)
- [Documentation, Architecture & Specs](#documentation-architecture--specs)
- [Building from Source](#building-from-source)
- [License](#license)

---

## Why Wandel

- **100% Private & Offline:** Every conversion runs locally on your machine. No accounts, no cloud uploads, no file-size limits, no watermarks, and no subscriptions.
- **Parallel CPU + GPU Speed:** Uses Rust `rayon` parallelism + SIMD image resizing across all CPU cores, plus automatic hardware video encoding (Apple VideoToolbox, NVIDIA NVENC, Intel QSV, AMD VAAPI).
- **One App for Every Media Format:** Batch-convert Images, Videos, Audio, and PDFs in a single distraction-free queue with one **Wandel** button.
- **Built for Developers & AI Agents:** Ships with a standalone headless `wandel` CLI and a 7-tool MCP server (`stdio` and Local HTTP) with 100% Rust engine parity, directory sandboxing, and composable multi-step pipelines.
- **Safe by Default:** Outputs save right next to your original files with smart non-destructive naming unless you opt in to overwrite.

---

## Capabilities at a Glance

| Domain | Supported Formats & Tools |
| :--- | :--- |
| **Images** | **PNG, JPG, WebP, AVIF, JPEG XL, HEIC, SVG, GIF, TIFF, BMP, ICO** · Single-decode **Multi-Format Fan-Out** (`WebP + AVIF + JPG`) · Target max file size (`<= KB`) · Resize, rotate, flip, visual crop, aspect pad, Auto-Trim, Auto-Levels · **Privacy Blur Redaction**, **Smart Background Removal**, **Duotone Grading**, **Visual Pixel Diff Highlighter** · Color presets, blur, pixelate, vignette, rounded corners, Screenshot Beautifier, Caption Footer Bar · Logo, text & diagonal tiled watermarks · **Social Media Kit (5 platforms)**, Vertical/Horizontal Stitcher, Pure-Rust Animated GIF Maker & Frame Extractor, Contact Sheets, Before/After Comparisons, Grid Slicer, Sprite Sheets + CSS, Responsive `srcset`, Web LQIP placeholders, PWA Icon Packs, Color Palettes, QR Codes, Base64 Data URLs, and dHash Duplicate Finder |
| **Video & Audio** | **MP4 (H.264 / H.265), WebM (VP9 / AV1), MOV, MKV, AVI, Animated GIF, Animated WebP** and **MP3, AAC, OGG (Opus), WAV, FLAC** · Instant lossless **Fast Remux** (`-c copy`) · **Social 9:16 / 1:1 / 4:5 Blur-Pad Reframing** · Visual trim, Reverse & Boomerang loops, speed (`0.25x` to `4x`), subtitle burn-in, external audio track replace, video watermark, deinterlace & sharpen · **FFT Voice Denoise**, **EBU R128 loudness normalization**, volume gain (dB), **auto-trim silence**, fade in/out · Merge videos/audio, lossless segment splitter, 3x3 storyboards, Poster Frame extractor, frame sequences, waveform PNGs, Slideshows, and Cover + Audio MP4 audiograms |
| **Pure-Rust PDF Toolkit** | **100% offline, zero external PDF tools:** **Visual Page Organizer** (reorder, per-page rotate, duplicate, delete), **Lossless Blank Page Remover**, **Lossless Duplex PDF Interleaver**, **Offline PDF Text Search (`pdf-grep`)**, **PDF to Markdown (`.md`)** for AI/RAG, **PDF to Text (`.txt`)**, **2-Up Side-by-Side PDF**, **Dark-Mode PDF**, **Lossless Metadata Editor**, merge images to PDF, rasterize PDF to images, merge PDFs, split/extract pages, compress PDFs, rotate/reorder pages, diagonal text stamp, page numbering, embedded image extractor, and metadata privacy scrubber |
| **Desktop Workflow** | Drag-and-drop files or folders · Clipboard paste (`⌘V`) · **Composable Pipeline Runner** · **Watch Folders** & Menu Bar / System Tray mode · Responsive tablet/narrow tab layout · Smart rename templates & folder preservation · ZIP bundle export & safe Trash originals · Custom presets · Queue search/sort & History CSV export · Command Palette (`⌘K`) · English & German UI |

---

## Download & Install

### Option 1: Desktop Installers (Includes Bundled Headless CLI)

Download the latest installer from [**GitHub Releases**](https://github.com/meghrazchi/wandel/releases):

| Platform | Installer File | Notes |
| :--- | :--- | :--- |
| **macOS (Apple Silicon: M1, M2, M3, M4)** | `Wandel_*_aarch64.dmg` | Bundled `wandel-cli` + FFmpeg + VideoToolbox GPU |
| **macOS (Intel)** | `Wandel_*_x64.dmg` | Bundled `wandel-cli` + FFmpeg + VideoToolbox GPU |
| **Windows (64-bit)** | `Wandel_*_x64-setup.exe` or `.msi` | Native NSIS / MSI installer with bundled CLI + FFmpeg |
| **Linux (64-bit)** | `.AppImage`, `.deb`, or `.rpm` | Bundled `wandel-cli` + FFmpeg sidecars |

#### macOS Setup
1. Open the downloaded `.dmg` and drag **Wandel** into **Applications**.
2. Open **Wandel** once. On macOS, Wandel automatically symlinks `~/.local/bin/wandel` to the bundled headless `wandel-cli` inside `Wandel.app` so Terminal and MCP clients can call `wandel` immediately without ever opening the UI.
3. If installing an unsigned dev build, clear Gatekeeper quarantine once in Terminal:
   ```bash
   xattr -cr /Applications/Wandel.app
   ```

#### Windows & Linux Setup
- **Windows:** Run the `.exe` or `.msi` installer. If SmartScreen prompts on first run, click **More info** then **Run anyway**.
- **Linux:** For `.AppImage`, run `chmod +x Wandel_*.AppImage && ./Wandel_*.AppImage`. For `.deb` / `.rpm`, install with `sudo apt install ./Wandel_*.deb` or `sudo rpm -i Wandel_*.rpm`.

### Option 2: Standalone Headless CLI & MCP Server

Install only the headless `wandel` binary (no GUI required):

```bash
# macOS & Linux one-line installer (SHA-256 verified)
curl -fsSL https://raw.githubusercontent.com/meghrazchi/wandel/main/packaging/install.sh | bash
```

Package manager manifests:
- **Homebrew Cask (macOS Desktop + CLI):** [packaging/homebrew/wandel.rb](packaging/homebrew/wandel.rb)
- **Scoop (Windows CLI & MCP):** [packaging/scoop/wandel.json](packaging/scoop/wandel.json)

---

## Connect Wandel MCP to AI Agents

Wandel's MCP server runs strictly through the headless `wandel` CLI (never opens the Desktop UI), supports both disk paths and inline base64 chat attachments (`inputBase64`), and exposes 7 batch-first tools with 100% Rust engine parity.

Follow the step-by-step setup guide for your AI client:

| AI Agent / Client | Setup Guide | Transport |
| :--- | :--- | :--- |
| **Claude Code & Claude Desktop** | [**Claude Code & Claude Desktop Guide (`docs/mcp/claude-code.md`)**](docs/mcp/claude-code.md) | `stdio` (`claude mcp add` / `claude_desktop_config.json`) |
| **OpenAI Codex** | [**OpenAI Codex Guide (`docs/mcp/codex.md`)**](docs/mcp/codex.md) | `stdio` (`codex mcp add` / `config.toml`) or Local HTTP |
| **Google Antigravity** | [**Google Antigravity Guide (`docs/mcp/antigravity.md`)**](docs/mcp/antigravity.md) | `stdio` or Local HTTP (`mcp_config.json`) |
| **Full MCP Spec & Pipelines** | [**MCP Protocol & Pipeline Reference (`docs/MCP.md`)**](docs/MCP.md) | All 7 tools, `--allowed-dir` sandboxing, `wandel_pipeline` |

---

## Quick CLI Examples

```bash
# Export a folder of photos into WebP, AVIF, and JPG in a single parallel pass
wandel convert ./photos --to webp --also avif,jpg --quality 82 --max-width 1920

# Apply duotone grading + bottom caption bar
wandel convert hero.png --to webp --duotone "#1e1b4b,#c084fc" --caption "Wandel Studio"

# Visual pixel diff between two UI screenshots
wandel diff before.png after.png -o diff.png --threshold 10

# Master podcast audio (EBU R128 loudness + silence trim + FFT voice denoise)
wandel convert episode.wav --to mp3 --normalize-audio --trim-silence --denoise-audio

# Pure-Rust PDF tools: Markdown export, offline grep, deblank, duplex interleave
wandel pdf-md paper.pdf -o paper.md
wandel pdf-grep "invoice" ./docs/*.pdf
wandel pdf-deblank scan.pdf -o clean.pdf
wandel pdf-interleave odd.pdf even.pdf --reverse-even -o full.pdf

# Start headless MCP server (stdio or local HTTP with bearer auth)
wandel mcp --allowed-dir ~/Documents/media
wandel mcp --http 127.0.0.1:3847 --token my-secret --allowed-dir ~/Documents/media
```

---

## Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| `⌘ / Ctrl + Enter` | Start **Wandel** conversion |
| `⌘ / Ctrl + K` | Open Command Palette |
| `⌘ / Ctrl + V` | Paste image from clipboard |
| `?` | Show Shortcuts & About |
| `Esc` | Cancel conversion or close modal |

---

## Documentation, Architecture & Specs

- [**Architecture & Performance Guide (`docs/ARCHITECTURE.md`)**](docs/ARCHITECTURE.md): Workspace crate layout, `rayon` + SIMD + GPU pipeline design, pure-Rust PDF stack, and testing strategy.
- [**MCP Server & Pipeline Spec (`docs/MCP.md`)**](docs/MCP.md): Headless `stdio` and Local HTTP MCP specification, `--allowed-dir` sandboxing, 7-tool JSON schemas, and `wandel_pipeline` recipes.
- [**macOS Code Signing & Notarization (`docs/MACOS_SIGNING.md`)**](docs/MACOS_SIGNING.md): `.p12` export, Base64 `APPLE_CERTIFICATE` setup, local `.env` signed builds, and GitHub Actions secrets.
- [**Roadmap & Version History (`docs/PLANS.md`)**](docs/PLANS.md): Complete milestone history from `v1.0.0` through `v3.5.x`.

---

## Building from Source

```bash
# 1. Stage local FFmpeg/FFprobe sidecars (macOS/Linux)
./scripts/fetch-ffmpeg.sh

# 2. Build & stage the bundled headless wandel-cli sidecar
npm run stage:cli

# 3. Install frontend dependencies & run tests
npm install
npm run test:coverage
npm run build

# 4. Run Rust lints & full workspace test suite
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# 5. Launch Desktop app in dev mode or build production installer
npm run tauri dev
npm run tauri build
```

---

## License

MIT: free for personal and commercial use.
