# Wandel

**Convert, compress, and edit images, videos, audio, and PDFs in seconds. 100% private, offline, and free on Mac, Windows, and Linux.**

[![CI](https://github.com/meghrazchi/wandel/actions/workflows/ci.yml/badge.svg)](https://github.com/meghrazchi/wandel/actions/workflows/ci.yml)
[![Release](https://github.com/meghrazchi/wandel/releases)](https://github.com/meghrazchi/wandel/releases)
![Platforms](https://img.shields.io/badge/platforms-macOS%20%7C%20Windows%20%7C%20Linux-indigo)
![License](https://img.shields.io/badge/license-MIT-emerald)

Stop uploading private photos, screen recordings, and confidential PDFs to slow, ad-filled websites. **Wandel** puts every media tool you need right on your computer as a fast **Desktop App**, a scriptable **CLI**, and a built-in **AI Agent MCP Server**.

---

## Why People Switch to Wandel

- **100% Private and Offline:** Your files never leave your computer. No accounts, no uploads, no file-size caps, no watermarks, and no subscriptions.
- **Seriously Fast:** Processes entire folders in parallel across all CPU cores and automatically uses your Mac, NVIDIA, Intel, or AMD GPU for hardware-accelerated video encoding.
- **Everything in One Clean App:** Images, Videos, Audio, and PDFs live together in one distraction-free workspace powered by a single **Wandel** button.
- **Safe by Default:** Converted files save right next to your originals without overwriting them unless you explicitly choose to.

---

## What You Can Do with Wandel

| Category | Highlights |
| :--- | :--- |
| **Images** | Convert and compress **PNG, JPG, WebP, AVIF, JPEG XL, HEIC, SVG, GIF, TIFF, BMP, ICO** · **Multi-Format Export** (`WebP + AVIF + JPG` in one click) · Target max size (`≤ KB`) · Resize, rotate, flip, visual crop, aspect pad, Auto-Trim, Auto-Levels · **Privacy Blur Redaction** & **Smart Background Removal** · Color looks, blur, pixelate, vignette, rounded corners, **Screenshot Beautifier**, **Caption Footer Bar** · Logo, text & diagonal tiled watermarks · **Social Media Kit (5 sizes)**, **Vertical/Horizontal Stitcher**, **Pure-Rust Animated GIF Maker & Frame Extractor**, Contact Sheets, Comparisons, Grid Slicer, Sprite Sheets + CSS, Responsive `srcset`, Web LQIP placeholders, PWA Icon Packs, Color Palettes, QR Codes, Base64 Data URLs, and Duplicate Finder |
| **Video & Audio** | Convert **MP4 (H.264 / H.265), WebM (VP9 / AV1), MOV, MKV, AVI, Animated GIF, Animated WebP** and **MP3, AAC, OGG (Opus), WAV, FLAC** · Instant lossless **Fast Remux** · **Social 9:16 / 1:1 / 4:5 Blur-Pad Reframing** · Visual trim, Reverse & Boomerang loops, speed (`0.5×` to `4×`), subtitle burn-in, audio track replace, video watermark, deinterlace & sharpen · **Voice Denoise (FFT)**, **EBU R128 loudness normalization**, volume gain (dB), **auto-trim silence**, fade in/out · Merge videos/audio, lossless segment splitter, 3×3 storyboards, **Poster Frame extractor**, frame sequences, waveform PNGs, Slideshows, and Cover + Audio MP4 audiograms |
| **PDF Toolkit** | **100% offline and pure-Rust:** **Visual Page Organizer** (drag-to-reorder, per-page rotate, duplicate, delete), **PDF to Markdown (.md)** for AI/RAG, **PDF to Text (.txt)**, **2-Up Side-by-Side PDF**, **Dark-Mode PDF**, **Lossless Metadata Editor** (Title, Author, Subject, Keywords), merge images into PDF, rasterize PDF to images, merge PDFs, split/extract pages, compress PDFs, rotate/reorder pages, stamp diagonal text, add page numbers, extract embedded images, and scrub identifying metadata |
| **Workflow** | Drag-and-drop files or folders · Clipboard paste (`⌘V`) · **Desktop Composable Pipeline Runner** · **Watch Folders** & Menu Bar / System Tray mode · Responsive narrow/tablet tab layout · Smart rename templates & folder preservation · ZIP bundle export & safe Trash originals · Custom presets · Queue search/sort & **History CSV export** · Command Palette (`⌘K`) · English & German UI |
| **CLI & AI (MCP)** | Headless `wandel` CLI and built-in **Model Context Protocol (MCP)** server over **stdio** or **Local HTTP** (`wandel mcp --http 127.0.0.1:3847 --token ...`) with parallel execution, directory sandboxing (`--allowed-dir`), dry-run previews, and multi-step composable pipelines (`wandel_pipeline`) |

---

## Download & Setup

### Option 1: Desktop Installers (Recommended)

Download the latest release for your OS from [**GitHub Releases**](https://github.com/meghrazchi/wandel/releases):

| Platform | Installer File | Notes |
| :--- | :--- | :--- |
| **macOS (Apple Silicon: M1, M2, M3, M4)** | `Wandel_*_aarch64.dmg` | Automatic Apple VideoToolbox GPU acceleration |
| **macOS (Intel)** | `Wandel_*_x64.dmg` | Automatic Apple VideoToolbox GPU acceleration |
| **Windows (64-bit)** | `Wandel_*_x64-setup.exe` or `.msi` | Native NSIS / MSI installer |
| **Linux (64-bit)** | `.AppImage`, `.deb`, or `.rpm` | Bundled FFmpeg sidecar included |

#### macOS Setup Guide
1. Open the downloaded `.dmg` and drag **Wandel** into **Applications**.
2. **First Launch (Unsigned Open-Source Build):** Because Wandel is a free open-source build without a paid Apple notarization certificate, macOS Gatekeeper may show a warning on first launch. Run this once in Terminal:
   ```bash
   xattr -cr /Applications/Wandel.app
   ```
   Then open **Wandel** normally (or right-click the app and choose **Open**).
3. **Optional Background Mode:** Enable **Minimize to Tray** in Settings to keep Watch Folders converting files quietly from your macOS Menu Bar.

#### Windows & Linux Setup
- **Windows:** Run the downloaded `.exe` or `.msi` installer. If Windows SmartScreen prompts on first run, click **More info** then **Run anyway**.
- **Linux:** For `.AppImage`, mark executable (`chmod +x Wandel_*.AppImage`) and run it. For `.deb` / `.rpm`, install with `sudo apt install ./Wandel_*.deb` or `sudo rpm -i Wandel_*.rpm`.

---

### Option 2: Headless CLI & MCP Server

Install the standalone `wandel` binary (includes both the CLI and the MCP server for Claude Desktop, Cursor, Windsurf, and Antigravity):

```bash
# macOS & Linux one-line installer (with SHA-256 verification)
curl -fsSL https://raw.githubusercontent.com/meghrazchi/wandel/main/packaging/install.sh | bash
```

Package manager manifests are also included in the repository:
- **Homebrew Cask (macOS Desktop):** [packaging/homebrew/wandel.rb](packaging/homebrew/wandel.rb)
- **Scoop (Windows CLI & MCP):** [packaging/scoop/wandel.json](packaging/scoop/wandel.json)

#### Quick CLI & MCP Examples

```bash
# Export a folder of photos into WebP, AVIF, and JPG in one pass
wandel convert ./photos --to webp --also avif,jpg --quality 82 --max-width 1920

# Master podcast audio (EBU R128 loudness + silence trim + voice denoise)
wandel convert episode.wav --to mp3 --normalize-audio --trim-silence --denoise-audio

# Convert a PDF to clean Markdown for AI / RAG workflows
wandel pdf-md paper.pdf -o paper.md

# Start the headless MCP server (stdio or local HTTP with optional bearer token)
wandel mcp --allowed-dir ~/Documents/media
wandel mcp --http 127.0.0.1:3847 --token my-secret --allowed-dir ~/Documents/media
```

Add to your MCP client config (or click **MCP** inside Wandel Desktop to copy a pre-filled JSON snippet):

```json
{
  "mcpServers": {
    "wandel": {
      "command": "wandel",
      "args": ["mcp"]
    }
  }
}
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

## Technical Architecture, Specs & Roadmap

For engineering details, MCP protocol specs, and full version history:

- [**Architecture & Performance Guide (`docs/ARCHITECTURE.md`)**](docs/ARCHITECTURE.md): Workspace crate layout, SIMD + `rayon` + GPU pipeline design, pure-Rust PDF stack, and build/test commands.
- [**MCP Server & Pipeline Specification (`docs/MCP.md`)**](docs/MCP.md): Stdio and Local HTTP MCP setup, `--allowed-dir` sandboxing, 7-tool JSON schema reference, and composable `wandel_pipeline` recipes.
- [**Roadmap & Version History (`docs/PLANS.md`)**](docs/PLANS.md): Complete milestone history from `v1.0.0` through `v3.2.0` and upcoming backlog.

---

## Building from Source

```bash
# 1. Stage local FFmpeg/FFprobe sidecars (macOS/Linux)
./scripts/fetch-ffmpeg.sh

# 2. Install frontend dependencies & run tests
npm install
npm run test:coverage
npm run build

# 3. Run Rust lints & full workspace test suite
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# 4. Launch Desktop app in dev mode or build production installer
npm run tauri dev
npm run tauri build
```

## License

MIT: free for personal and commercial use.
