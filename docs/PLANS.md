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
| **v2.5 (`v25`)** | EBU R128 loudness normalization & auto-trim silence across CLI, MCP, and UI Active Modifiers; Desktop Composable Pipeline Runner (`PipelineModal` with starter templates + dry-run preview); and distribution packaging (`packaging/install.sh`, Homebrew Cask, Scoop manifest). |
| **v2.6 (`v26`)** | Interactive Privacy Blur Redaction regions in `PreviewModal` (click-and-drag redaction boxes + `--blur-region` CLI + `blurRegions` MCP) and pure-Rust offline PDF text extraction (`wandel pdf-text`, `wandel_pdf_ops` `extract_text`, and Desktop `.txt` export). |
| **v2.7 (`v27`)** | Parallel 5-platform Social Media Kit generator (`generate_social_kit` across Core/UI/CLI/MCP), Video Poster Frame Extractor (`wandel poster` / MCP `poster`), and sidebar Active Modifiers chip for privacy blur redactions. |
| **v2.8 (`v28`)** | Pure-Rust Smart Solid/Auto Background Removal with soft edge feathering (`--remove-bg` / MCP `removeBgColor` / UI controls), Parallel Dark-Mode PDF Converter (`wandel pdf-dark` / MCP `dark_mode` / UI), and Desktop Video Poster Frame menu action. |
| **v2.9 (`v29`)** | Pure-Rust Images → Animated GIF Maker (`wandel make-gif` / MCP `make_gif` / UI), Parallel Animated GIF Frame Extractor (`wandel gif-frames` / MCP `gif_frames` / UI), `#[cfg(desktop)]` mobile-ready Tauri guards, and responsive Queue/Settings mobile tab layout. |
| **v3.0 (`v30`)** | Single-Decode Multi-Format Fan-Out Export (`--also avif,jpg` / MCP `alsoFormats` / UI **Also Export As** pills), History CSV export in `HistoryDrawer`, and v3.0.0 milestone docs & package manager polish. |
| **v3.1 (`v31`)** | Pure-Rust PDF → Markdown Converter with YAML frontmatter (`wandel pdf-md` / MCP `extract_markdown` / UI), Parallel 2-Up Side-by-Side PDF Imposition (`wandel pdf-2up` / MCP `two_up` / UI), and Image Caption Footer Bar (`--caption` / MCP `caption` / UI). |
| **v3.2 (`v32`)** | Full-Resolution Vertical & Horizontal Image Stitcher (`wandel stitch` / MCP `stitch` / UI), Lossless PDF Metadata Editor (`wandel pdf-meta` / MCP `set_metadata` / UI), and Web LQIP Tiny Placeholder Generator (`wandel lqip` / MCP `lqip`). |
| **v3.3 (`v33`)** | Unified Wandel indigo palette for Technical Stack & Capabilities badges/pills, Pure-Rust Lossless Blank PDF Page Remover (`wandel pdf-deblank` / MCP `remove_blank_pages` / UI), and Parallel Image Duotone Grading (`--duotone` / MCP `duotone` / UI). |
| **v3.4 (`v34`)** | Visual Pixel Diff Highlighter (`wandel diff` / MCP `diff` / UI), Offline PDF Text Search (`wandel pdf-grep` / MCP `search`), and Lossless Duplex PDF Interleaver (`wandel pdf-interleave` / MCP `interleave` / UI). |
| **v3.5 (`v35`)** | Headless CLI-first MCP architecture (never opens Desktop UI, early `mcp` safety guard in `wandel-desktop`, auto-discovery of bundled/Homebrew FFmpeg/FFprobe under Claude Desktop's minimal PATH, and absolute CLI path detection in `McpModal`) plus 100% Rust engine & UI option parity across `wandel-mcp` and `wandel-cli`. |

---

## Future Ideas (Backlog)
- Signed & notarized macOS Universal builds + Windows code signing.
