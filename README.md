# Wandel

**Wandel** is a fast, lightweight, cross-platform **Image, Video, Audio & PDF converter** built in **Rust** and **Tauri v2**, engineered to saturate all CPU cores and GPU hardware encoders while staying tiny and easy to use.

## Highlights

- **Parallel Multi-Core & GPU Engine:** Uses `rayon` + SIMD `fast_image_resize` (NEON / AVX2) for batch images and adaptive FFmpeg concurrency with automatic GPU hardware acceleration (**Apple VideoToolbox**, **NVIDIA NVENC**, **Intel QSV**, **VAAPI**) and seamless CPU fallback.
- **Images:** Convert, compress, and resize **PNG (`oxipng`)**, **JPG**, **WebP**, **AVIF (`ravif`)**, **JPEG XL**, **HEIC**, **SVG**, **GIF**, **TIFF**, **BMP**, and **ICO**. Includes target max size (`≤ KB`), EXIF auto-orientation & privacy stripping, Before/After visual split preview, visual custom crop, no-crop aspect padding, rounded corners, brightness/contrast/saturation, grayscale, auto-sharpen, logo & text watermarks, Contact Sheet grids, Favicon/PWA Icon Pack generator, and perceptual duplicate image detection (dHash).
- **Video & Audio:** Convert **MP4 (H.264 / H.265)**, **WebM (VP9 / AV1)**, **MOV**, **MKV**, **AVI**, **Animated GIF**, **Animated WebP**, and extract/convert **MP3, AAC, OGG (Opus), WAV, FLAC**. Includes visual video/audio trim scrubbers, frame snapshots, target max size (`≤ MB`), playback speed (`0.5×–4×`), rotate/flip, subtitle burn-in (`.srt`/`.vtt`), audio track replacement, video logo watermark, video concatenation, and EBU R128 loudness normalization.
- **PDF Suite (100% Pure Rust):** Merge images → multi-page PDF (`pdf-writer`), rasterize PDF → images at 72/150/300 DPI (`hayro`), merge multiple PDFs losslessly (`lopdf`), and split/extract PDF page ranges.
- **Workflow & Automation:** Native drag-and-drop (files & folders), Clipboard Paste (`⌘V`), Watch Folders (Auto-Wandel) with System Tray / Menu Bar background mode, smart rename templates (`{name}_{width}x{height}`, `{index:03}`), folder hierarchy preservation, ZIP archive bundling, per-file format overrides, custom presets (with JSON import/export), conversion history & space-saved stats, `⌘K` Command Palette, and EN/DE localization.

## Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| `⌘ / Ctrl + Enter` | Start **Wandel** conversion |
| `⌘ / Ctrl + K` | Open Command Palette |
| `⌘ / Ctrl + V` | Paste image from clipboard |
| `Esc` | Cancel active conversion / close modal |

## CLI (`wandel-cli`)

```bash
# Hardware & CPU/GPU info
cargo run -p wandel-cli -- info

# Batch convert a folder of photos to WebP, max 1920px wide
cargo run -p wandel-cli -- convert ./photos --to webp --quality 82 --max-width 1920

# Inspect EXIF / GPS / codec / PDF metadata
cargo run -p wandel-cli -- inspect ./photo.jpg

# Find duplicate images
cargo run -p wandel-cli -- dedupe ./photos

# Merge images to PDF, merge PDFs, generate Favicon Icon Pack, create Contact Sheet, or merge videos
cargo run -p wandel-cli -- pdf-merge a.png b.png -o album.pdf --fit a4
cargo run -p wandel-cli -- pdf-concat doc1.pdf doc2.pdf -o combined.pdf
cargo run -p wandel-cli -- icon-pack logo.svg -o ./icons
cargo run -p wandel-cli -- grid *.png -o sheet.png --cols 3
cargo run -p wandel-cli -- video-merge clip1.mp4 clip2.mp4 -o merged.mp4

# Run headless MCP (Model Context Protocol) server over stdio
cargo run -p wandel-cli -- mcp
```

## MCP Server (Headless & Desktop UI)

Wandel ships a high-performance **Model Context Protocol (MCP)** server (`crates/wandel-mcp`) that works **completely headless** (`wandel mcp`, no display/UI required) and integrates with the Wandel Desktop UI (click **MCP** in the header or `⌘K → Open MCP Server Integration`).

- **Concurrent multi-tool execution:** Parallel `tools/call` requests from AI agents are dispatched concurrently across a bounded worker pool while serializing JSON-RPC frames safely.
- **One-time startup caching:** Hardware & FFmpeg/GPU capabilities are probed once on startup.
- **Batch-first 6-tool surface:** `wandel_info`, `wandel_inspect`, `wandel_convert`, `wandel_pdf_ops`, `wandel_image_tools`, and `wandel_media_tools`, plus `wandel://capabilities` resource, prompt templates, progress notifications, and `notifications/cancelled` support.

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

## Development & Packaging

```bash
# 1. Stage local FFmpeg/FFprobe sidecars (macOS/Linux)
./scripts/fetch-ffmpeg.sh

# 2. Install frontend dependencies
npm install

# 3. Launch Wandel in development mode
npm run tauri dev

# 4. Run full test suite & lints
npm run build
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# 5. Build production bundle (.dmg / .app / .msi / .exe / .deb / .AppImage)
npm run tauri build
```
