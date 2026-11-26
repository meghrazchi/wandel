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
| **Images** | Convert & compress **PNG, JPG, WebP, AVIF, JPEG XL, HEIC, SVG, GIF, TIFF, BMP, ICO** · Target max size (`≤ KB`) · Resize, rotate, flip, visual crop, aspect pad, **Auto-Trim borders**, **Auto-Levels** · Color presets (Sepia, Warm, Cool, Vintage, B&W+, Invert) · Blur, pixelate, vignette, rounded corners · **Screenshot Beautifier** (gradient frame + drop shadow) · Logo, text & diagonal tiled watermarks · Contact Sheet grids, Before/After comparisons, Image Grid Slicer (`3×1` / `3×3`), Sprite Sheets + CSS maps, Responsive `srcset` bundles, PWA Icon Packs, Dominant Color Palette, QR Codes, Base64 Data URLs, and Duplicate Finder |
| **Video & Audio** | Convert **MP4 (H.264 / H.265), WebM (VP9 / AV1), MOV, MKV, AVI, Animated GIF, Animated WebP** and **MP3, AAC, OGG (Opus), WAV, FLAC** · Instant lossless **Fast Remux** (zero re-encoding) · **Social 9:16 / 1:1 / 4:5 Blur-Pad Reframing** · Visual trim, Reverse & Boomerang loops, speed (`0.5×–4×`), subtitle burn-in, audio track replace, video watermark, deinterlace & sharpen · **Voice Denoise (FFT)**, EBU R128 loudness normalization, volume gain (dB), silence trim, fade in/out · Merge videos/audio, lossless segment splitter, 3×3 storyboards, frame extractor, audio waveform PNGs, Image Slideshows, and Cover + Audio MP4 audiograms |
| **PDF Toolkit** | **100% offline & lossless**: Merge images into PDF (Fit / A4 / Letter), rasterize PDF to images (72 / 150 / 300 DPI), merge PDFs, split/extract pages, compress PDFs, rotate pages, reorder pages, stamp diagonal text (`CONFIDENTIAL`), add page numbers, extract embedded images, and scrub identifying metadata |
| **Workflow** | Drag-and-drop files or folders · Clipboard paste (`⌘V`) · **Watch Folders** (Auto-Wandel) & Menu Bar / System Tray mode · Smart rename templates & folder preservation · ZIP bundle export & safe Trash originals · Custom presets (JSON import/export) · Queue search/sort & CSV batch reports · Command Palette (`⌘K`) · English & German UI |
| **CLI & AI (MCP)** | Full headless CLI (`wandel`) and built-in **Model Context Protocol (MCP)** server (`wandel mcp`) with parallel tool execution, directory sandboxing (`--allowed-dir`), dry-run previews, and multi-step composable pipelines (`wandel_pipeline`) |

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
Prefer the terminal or want to connect Wandel to **Claude Desktop, Cursor, Windsurf, or Antigravity**? Download the standalone `wandel-cli` archive from [Releases](https://github.com/meghrazchi/wandel/releases) (or build from source):

```bash
# Batch-convert a folder of photos to WebP (max 1920px wide)
wandel convert ./photos --to webp --quality 82 --max-width 1920

# Losslessly scrub metadata & number a PDF
wandel pdf-clean report.pdf -o clean.pdf
wandel pdf-number clean.pdf -o ready.pdf --format "Page {n} of {total}"

# Start the headless MCP server (optional --allowed-dir sandbox)
wandel mcp --allowed-dir ~/Documents/media
```

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
