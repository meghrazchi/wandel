# Wandel

**Convert, compress, and edit images, videos, audio, and PDFs in seconds. 100% private, offline, and free on Mac, Windows, and Linux.**

[![CI](https://github.com/meghrazchi/wandel/actions/workflows/ci.yml/badge.svg)](https://github.com/meghrazchi/wandel/actions/workflows/ci.yml)
[![Release](https://github.com/meghrazchi/wandel/actions/workflows/release.yml/badge.svg)](https://github.com/meghrazchi/wandel/releases)
![Platforms](https://img.shields.io/badge/platforms-macOS%20%7C%20Windows%20%7C%20Linux-indigo)
![License](https://img.shields.io/badge/license-MIT-emerald)

Stop uploading private photos, screen recordings, and confidential PDFs to slow, ad-filled websites. **Wandel** puts every media tool you need right on your computer as a lightweight **Desktop App**, a scriptable **CLI**, and a built-in **AI Agent MCP Server**.

---

## Why Wandel?

- **100% Private & Offline** — Your files never leave your machine. No accounts, no watermarks, no file-size limits, no subscriptions.
- **Built for Speed** — Uses every CPU core in parallel and automatically taps your Mac, NVIDIA, Intel, or AMD GPU for fast video conversion.
- **One App for Everything** — Images, Videos, Audio, and PDFs live in a single clean interface with one prominent **Wandel** button.
- **Safe by Default** — Saves converted files right next to your originals without ever overwriting them unless you ask.

---

## Features at a Glance

| Category | What You Can Do |
| :--- | :--- |
| **Images** | Convert & compress **PNG, JPG, WebP, AVIF, JPEG XL, HEIC, SVG, GIF, TIFF, BMP, ICO** · **Multi-Format Fan-Out** (`--also avif,jpg`) · Target max size (`≤ KB`) · Resize, rotate, flip, visual crop, aspect pad, **Auto-Trim**, **Auto-Levels** · **Privacy Blur Redaction** & **Smart Background Removal** · Color presets, blur, pixelate, vignette, rounded corners, **Screenshot Beautifier** · Logo, text & tiled watermarks · **Social Media Kit (5 sizes)**, **Pure-Rust Animated GIF Maker & Frame Extractor**, Contact Sheets, Comparisons, Grid Slicer, Sprite Sheets + CSS, Responsive `srcset`, PWA Icon Packs, Color Palettes, QR Codes, Base64 Data URLs, and Duplicate Finder |
| **Video & Audio** | Convert **MP4 (H.264 / H.265), WebM (VP9 / AV1), MOV, MKV, AVI, Animated GIF, Animated WebP** and **MP3, AAC, OGG (Opus), WAV, FLAC** · Instant lossless **Fast Remux** · **Social 9:16 / 1:1 / 4:5 Blur-Pad Reframing** · Visual trim, Reverse & Boomerang loops, speed (`0.5×–4×`), subtitle burn-in, audio track replace, video watermark, deinterlace & sharpen · **Voice Denoise (FFT)**, **EBU R128 loudness normalization**, volume gain (dB), **auto-trim silence**, fade in/out · Merge videos/audio, lossless segment splitter, 3×3 storyboards, **Poster Frame extractor**, frame sequences, waveform PNGs, Slideshows, and Cover + Audio MP4 audiograms |
| **PDF Toolkit** | **100% offline & pure-Rust**: **Visual Page Organizer** (drag-to-reorder, per-page rotate/duplicate/delete), **Dark-Mode PDF converter**, **Offline PDF Text Extraction (.txt)**, merge images into PDF, rasterize PDF to images, merge PDFs, split/extract pages, compress PDFs, rotate/reorder pages, stamp diagonal text, add page numbers, extract embedded images, and scrub identifying metadata |
| **Workflow** | Drag-and-drop files or folders · Clipboard paste (`⌘V`) · **Desktop Composable Pipeline Runner** · **Watch Folders** & Menu Bar / System Tray mode · Responsive mobile/narrow tab layout · Smart rename templates & folder preservation · ZIP bundle export & safe Trash originals · Custom presets · Queue search/sort & **History CSV export** · Command Palette (`⌘K`) · English & German UI |
| **CLI & AI (MCP)** | Full headless CLI (`wandel`) and built-in **Model Context Protocol (MCP)** server over **stdio** or **Local HTTP** (`wandel mcp --http 127.0.0.1:3847 --token ...`) with parallel execution, directory sandboxing (`--allowed-dir`), dry-run previews, and multi-step composable pipelines (`wandel_pipeline`) |

---

## Quick Start

### 1. Download Wandel Desktop
Grab the latest installer for your system from the [**GitHub Releases**](https://github.com/meghrazchi/wandel/releases) page:

- **macOS (Apple Silicon — M1 / M2 / M3 / M4):** `Wandel_*_aarch64.dmg`
- **macOS (Intel):** `Wandel_*_x64.dmg`
- **Windows (64-bit):** `Wandel_*_x64-setup.exe` or `.msi`
- **Linux (64-bit):** `.AppImage`, `.deb`, or `.rpm`

### 2. macOS Setup & Configuration
1. Open the downloaded `.dmg` and drag **Wandel** into your **Applications** folder.
2. **First Launch (Unsigned Open-Source Build):** If macOS Gatekeeper warns that the app is from an unidentified developer, run once in Terminal:
   ```bash
   xattr -cr /Applications/Wandel.app
   ```
   Then launch Wandel normally (or right-click → **Open**).
3. **Hardware Acceleration:** Apple Silicon and Intel Macs automatically enable **Apple VideoToolbox** (`h264_videotoolbox` / `hevc_videotoolbox`) for fast, low-power video encoding — no setup needed.
4. **Menu Bar Background Mode:** Enable **Minimize to Tray** in Settings to keep Watch Folders converting files silently in the background from your macOS Menu Bar.

### 3. Quick Start: Headless CLI & MCP Server
Prefer the terminal or want to connect Wandel to **Claude Desktop, Cursor, Windsurf, or Antigravity**? Install the standalone `wandel` CLI/MCP binary in one line (macOS & Linux, with SHA-256 verification) or grab the Windows archive from [Releases](https://github.com/meghrazchi/wandel/releases):

```bash
# One-line installer (macOS & Linux)
curl -fsSL https://raw.githubusercontent.com/meghrazchi/wandel/main/packaging/install.sh | bash

# Batch-convert a folder of photos to WebP (max 1920px wide)
wandel convert ./photos --to webp --quality 82 --max-width 1920

# Master podcast audio (EBU R128 loudness + silence trim + voice denoise)
wandel convert episode.wav --to mp3 --normalize-audio --trim-silence --denoise-audio

# Start the headless MCP server (stdio or local HTTP)
wandel mcp --allowed-dir ~/Documents/media
wandel mcp --http 127.0.0.1:3847 --token my-secret --allowed-dir ~/Documents/media
```

> **Package Manager Manifests:** Ready-to-use Homebrew Cask ([packaging/homebrew/wandel.rb](packaging/homebrew/wandel.rb)) and Windows Scoop manifest ([packaging/scoop/wandel.json](packaging/scoop/wandel.json)) are included in the repository.

Add to your MCP client config (or click **MCP** inside Wandel Desktop to copy your pre-filled config):

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
| `Esc` | Cancel conversion / close modal |

---

## Documentation

- [**Architecture & Performance Guide (`docs/ARCHITECTURE.md`)**](docs/ARCHITECTURE.md) — Crate layout, SIMD + `rayon` + GPU pipeline design, pure-Rust PDF stack, and build/test commands.
- [**MCP Server & Pipeline Guide (`docs/MCP.md`)**](docs/MCP.md) — Headless & UI MCP setup, `--allowed-dir` sandboxing, 7-tool reference, and `wandel_pipeline` JSON recipes.
- [**Roadmap & Version History (`docs/PLANS.md`)**](docs/PLANS.md) — Release history from `v1.0.0` to `v2.2.0` and upcoming plans.

---

## Building from Source

```bash
# 1. Stage local FFmpeg/FFprobe sidecars (macOS/Linux)
./scripts/fetch-ffmpeg.sh

# 2. Install frontend dependencies & run tests
npm install
npm run test:coverage
npm run build

# 3. Run Rust lints & full test suite
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# 4. Launch Desktop app in dev mode or build production bundle
npm run tauri dev
npm run tauri build
```

## License

MIT — free for personal and commercial use.
