# Wandel Roadmap & Version History

This document tracks Wandel's completed milestones from `v1.0.0` through `v2.2.0` and outlines future ideas.

---

## Completed Milestones

| Version | Highlights |
| :--- | :--- |
| **v1.0 (`v1`–`v10`)** | Rust + Tauri v2 foundation, `rayon` + SIMD `fast_image_resize` parallel image engine, adaptive GPU video/audio engine (VideoToolbox/NVENC/QSV/VAAPI), pure-Rust PDF suite (`pdf-writer`, `hayro`, `lopdf`), visual crop & trim previews, Watch Folders, System Tray, EXIF/GPS inspector, Icon Packs, Contact Sheets, dHash Dedupe, Color Palettes, QR Generator, Command Palette (`⌘K`), EN/DE i18n, and `wandel-cli`. |
| **v1.1 (`v11`)** | Image Blur / Pixelate / Vignette filters, Base64 Data URL export, 3×3 Video Storyboard contact sheets, Re-queue Outputs chaining, and Keyboard Shortcuts modal. |
| **v1.2 (`v12`)** | Screenshot Beautifier (gradient frame + soft drop shadow), Responsive Web `srcset` bundle generator (`wandel srcset`), Video Reverse & Boomerang loop modes, and persisted settings + Reset. |
| **v1.3 (`v13`)** | Smart Auto-Trim image borders, Social Video Blur-Pad reframing (`9:16`, `4:5`, `1:1`, `16:9`), lossless PDF page rotate & reorder/extract, and custom presets in Command Palette. |
| **v1.4 (`v14`)** | Before/After Comparison Collage (`wandel compare`), Video Frame Sequence Extractor (`wandel frames`), Audio Waveform PNG Generator (`wandel waveform`), and Queue Search & Sort. |
| **v1.5 (`v15`)** | Image-to-Video Slideshow Maker (`wandel slideshow`), one-click Color Look presets (Sepia/Warm/Cool/Vintage/B&W+/Invert), lossless PDF diagonal text stamping (`wandel pdf-stamp`), and instant lossless Video Remux (`-c copy`). |
| **v1.6 (`v16`)** | Image Grid Slicer (`wandel slice`), FFT Audio Voice Denoise (`--denoise-audio`), lossless PDF page numbering (`wandel pdf-number`), and Video Deinterlace + Sharpen restoration. |
| **v1.7 (`v17`)** | Image Sprite Sheet + CSS Map generator (`wandel sprite`), pure-Rust PDF embedded image extractor (`wandel pdf-images`), Audio Volume Gain (dB) + Speed (`atempo`), and Cover + Audio → MP4 Audiograms (`wandel audiogram`). |
| **v1.8 (`v18`)** | Pure-Rust Auto-Levels histogram contrast stretch (`--auto-levels`), lossless Video/Audio segment splitter (`wandel split-media`), lossless PDF metadata privacy scrubber (`wandel pdf-clean`), and Batch CSV report export. |
| **v1.9 (`v19`)** | Dedicated `crates/wandel-mcp` concurrent stdio MCP server (headless `wandel mcp` + Desktop UI `McpModal`), startup hardware caching, 6 batch-first tools, resources, prompts, progress & cancellation. |
| **v2.0 (`v20`)** | Composable multi-step `wandel_pipeline` (`$prev` / `$step.N` piping + `wandel pipeline <recipe.json>`), MCP directory sandboxing (`--allowed-dir`), zero-write `dryRun` preview, and diagonal repeating tiled text watermarks. |
| **v2.1 (`v21`)** | 100% frontend utility coverage (`Vitest` + `v8`), full `wandel-cli` E2E test suite, expanded `wandel-mcp` & `wandel-core` tests, strict GitHub Actions CI with `cargo-llvm-cov`, and automated macOS (arm64/x64), Linux (x64), and Windows (x64) Desktop + CLI/MCP releases. |
| **v2.2 (`v22`)** | Cached `OnceLock` system `fontdb`, `rayon` parallel RGBA pixel passes + zero-copy `into_rgba8()`, memoized queue rendering, SEO/marketing `README.md` with macOS setup guide, and structured `docs/` (`ARCHITECTURE.md`, `MCP.md`, `PLANS.md`). |
| **v2.3 (`v23`)** | Progressive disclosure UX (`SectionAccordion` collapsible drawers, live Active Modifiers chip strip, decluttered `Batch Tools ▾` & per-row `⋯` menus, floating dismissible toast), Wandel brand SVG icon, and cross-OS CI/Release hardening (`nasm`, `fonts-dejavu-core`, 8 MB Windows CLI stack). |
| **v2.4 (`v24`)** | Interactive Visual PDF Page Organizer (`PdfOrganizerModal` with parallel `hayro` thumbnails, drag-to-reorder, per-page rotate/duplicate/remove, single-pass lossless `lopdf` export) and Local HTTP MCP Transport (`wandel mcp --http` with optional `--token` bearer auth). |

---

## Future Ideas (Backlog)
- Signed & notarized macOS Universal builds + Windows code signing.
- Optional Homebrew Cask (`brew install --cask wandel`) and Winget / Scoop manifests.
