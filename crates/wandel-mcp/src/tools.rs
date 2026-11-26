use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use wandel_core::{
    compress_pdf, create_audio_waveform, create_comparison_image, create_contact_sheet,
    create_podcast_video, create_slideshow_video, create_sprite_sheet, create_video_storyboard,
    export_image_data_url, extract_color_palette, extract_pdf_images, extract_video_frames,
    find_duplicate_images, generate_icon_pack, generate_responsive_set, inspect_detailed_metadata,
    inspect_paths, merge_audio, merge_images_to_pdf, merge_pdfs, merge_videos, number_pdf_pages,
    probe_hardware, reorder_pdf_pages, rotate_pdf_pages, run_batch, save_qr_code,
    scrub_pdf_metadata, split_image_grid, split_media_segments, split_pdf, stamp_pdf_text,
    AspectCrop, AudioFormat, ComparisonOptions, ContactSheetOptions, ConversionSettings,
    FrameGradient, HardwareCapabilities, ImageFilterPreset, ImageFormat, ImageSettings, JobItem,
    MediaKind, OutputSettings, PdfPageFit, Preset, ProgressEvent, ResizeMode,
    TextWatermarkSettings, VideoAspectReframe, VideoFormat, VideoPlaybackMode, VideoSettings,
    VideoTarget, WatermarkPosition,
};

/// Shared read-only server context initialized once at startup (avoids per-call FFmpeg probing).
pub struct McpContext {
    pub ffmpeg_path: Option<PathBuf>,
    pub ffprobe_path: Option<PathBuf>,
    pub hardware: HardwareCapabilities,
    pub allowed_dirs: Vec<PathBuf>,
}

impl McpContext {
    pub fn new(ffmpeg_path: Option<PathBuf>, ffprobe_path: Option<PathBuf>) -> Self {
        Self::with_allowed_dirs(ffmpeg_path, ffprobe_path, Vec::new())
    }

    pub fn with_allowed_dirs(
        ffmpeg_path: Option<PathBuf>,
        ffprobe_path: Option<PathBuf>,
        allowed_dirs: Vec<PathBuf>,
    ) -> Self {
        let hardware = probe_hardware(ffmpeg_path.as_deref(), ffprobe_path.as_deref());
        let canonical_dirs = allowed_dirs
            .into_iter()
            .map(|d| std::fs::canonicalize(&d).unwrap_or(d))
            .collect();
        Self {
            ffmpeg_path,
            ffprobe_path,
            hardware,
            allowed_dirs: canonical_dirs,
        }
    }

    /// Enforces `--allowed-dir` sandbox roots when configured.
    pub fn validate_path(&self, path: &Path) -> Result<(), String> {
        if self.allowed_dirs.is_empty() {
            return Ok(());
        }
        let resolved = resolve_for_sandbox(path);
        if self.allowed_dirs.iter().any(|root| resolved.starts_with(root)) {
            Ok(())
        } else {
            Err(format!(
                "Sandbox violation: path '{}' is outside allowed directories",
                path.display()
            ))
        }
    }

    pub fn validate_paths(&self, paths: &[PathBuf]) -> Result<(), String> {
        for p in paths {
            self.validate_path(p)?;
        }
        Ok(())
    }

    pub fn capabilities_json(&self) -> Value {
        json!({
            "version": env!("CARGO_PKG_VERSION"),
            "hardware": self.hardware,
            "sandboxed": !self.allowed_dirs.is_empty(),
            "allowedDirs": self.allowed_dirs,
            "presets": ["web", "high", "small"],
            "imageFormats": ["webp", "jpg", "png", "avif", "jxl", "heic", "gif", "tiff", "bmp", "ico"],
            "videoFormats": ["mp4", "h265", "webm", "av1", "mov", "mkv", "avi", "gif", "animated_webp"],
            "audioFormats": ["mp3", "aac", "opus", "wav", "flac"],
            "pdfPureRust": true
        })
    }
}

fn resolve_for_sandbox(path: &Path) -> PathBuf {
    if let Ok(canon) = std::fs::canonicalize(path) {
        return canon;
    }
    let mut tail = Vec::new();
    let mut cur = path;
    while let Some(name) = cur.file_name() {
        tail.push(name.to_os_string());
        if let Some(parent) = cur.parent() {
            if let Ok(canon_parent) = std::fs::canonicalize(parent) {
                let mut out = canon_parent;
                for seg in tail.into_iter().rev() {
                    out.push(seg);
                }
                return out;
            }
            cur = parent;
        } else {
            break;
        }
    }
    path.to_path_buf()
}

pub struct ToolCallOutcome {
    pub payload: Value,
    pub is_error: bool,
}

impl ToolCallOutcome {
    pub fn ok(payload: Value) -> Self {
        Self {
            payload,
            is_error: false,
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            payload: json!({ "error": msg.into() }),
            is_error: true,
        }
    }

    pub fn to_mcp_result(&self) -> Value {
        let text = serde_json::to_string_pretty(&self.payload)
            .unwrap_or_else(|_| self.payload.to_string());
        json!({
            "content": [{ "type": "text", "text": text }],
            "structuredContent": self.payload,
            "isError": self.is_error
        })
    }
}

pub fn execute_tool<F>(
    ctx: &McpContext,
    name: &str,
    args: &Value,
    cancel: Arc<AtomicBool>,
    on_progress: F,
) -> ToolCallOutcome
where
    F: Fn(f64, f64, &str) + Send + Sync,
{
    if name == "wandel_pipeline" {
        return tool_pipeline(ctx, args, cancel, &on_progress);
    }
    execute_single_tool(ctx, name, args, cancel, &on_progress)
}

fn execute_single_tool(
    ctx: &McpContext,
    name: &str,
    args: &Value,
    cancel: Arc<AtomicBool>,
    on_progress: &(dyn Fn(f64, f64, &str) + Send + Sync),
) -> ToolCallOutcome {
    match name {
        "wandel_info" => ToolCallOutcome::ok(ctx.capabilities_json()),
        "wandel_inspect" => tool_inspect(ctx, args),
        "wandel_convert" => tool_convert(ctx, args, cancel, |p, t, m| on_progress(p, t, m)),
        "wandel_pdf_ops" => tool_pdf_ops(ctx, args),
        "wandel_image_tools" => tool_image_tools(ctx, args),
        "wandel_media_tools" => tool_media_tools(ctx, args),
        other => ToolCallOutcome::err(format!("Unknown Wandel MCP tool: '{}'", other)),
    }
}

fn str_arg<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(|v| v.as_str()).filter(|s| !s.is_empty())
}

fn path_list_arg(args: &Value, key: &str) -> Vec<PathBuf> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
                .collect()
        })
        .unwrap_or_default()
}

fn tool_inspect(ctx: &McpContext, args: &Value) -> ToolCallOutcome {
    let inputs = path_list_arg(args, "inputs");
    if inputs.is_empty() {
        return ToolCallOutcome::err("'inputs' must contain at least one file or directory path");
    }
    if let Err(e) = ctx.validate_paths(&inputs) {
        return ToolCallOutcome::err(e);
    }
    let detailed = args.get("detailed").and_then(|v| v.as_bool()).unwrap_or(false);
    let include_thumbs = args
        .get("includeThumbnails")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut files = inspect_paths(&inputs, ctx.ffmpeg_path.as_deref(), ctx.ffprobe_path.as_deref());
    if !include_thumbs {
        for f in &mut files {
            f.thumbnail_data_url = None;
        }
    }

    if detailed {
        let detailed_list: Vec<Value> = files
            .iter()
            .map(|f| {
                let meta = inspect_detailed_metadata(
                    &PathBuf::from(&f.path),
                    ctx.ffprobe_path.as_deref(),
                );
                json!({
                    "file": f,
                    "detailed": meta
                })
            })
            .collect();
        ToolCallOutcome::ok(json!({
            "count": detailed_list.len(),
            "files": detailed_list
        }))
    } else {
        ToolCallOutcome::ok(json!({
            "count": files.len(),
            "files": files
        }))
    }
}

fn apply_target_format(settings: &mut ConversionSettings, to: &str) -> Result<(), String> {
    let lower = to.to_ascii_lowercase();
    if let Some(img_fmt) = ImageFormat::from_extension(&lower) {
        if !img_fmt.can_encode() {
            return Err(format!("Cannot encode to {}", img_fmt.display_name()));
        }
        settings.image.format = img_fmt;
        if lower == "gif" {
            settings.video.target = VideoTarget::Video(VideoFormat::Gif);
        }
        return Ok(());
    }

    match lower.as_str() {
        "mp4" | "h264" | "mp4_h264" => {
            settings.video.target = VideoTarget::Video(VideoFormat::Mp4H264)
        }
        "h265" | "hevc" | "mp4_h265" => {
            settings.video.target = VideoTarget::Video(VideoFormat::Mp4H265)
        }
        "webm" | "vp9" | "webm_vp9" => {
            settings.video.target = VideoTarget::Video(VideoFormat::WebmVp9)
        }
        "av1" | "webm_av1" => settings.video.target = VideoTarget::Video(VideoFormat::WebmAv1),
        "mov" => settings.video.target = VideoTarget::Video(VideoFormat::Mov),
        "mkv" => settings.video.target = VideoTarget::Video(VideoFormat::Mkv),
        "avi" => settings.video.target = VideoTarget::Video(VideoFormat::Avi),
        "awebp" | "animated_webp" => {
            settings.video.target = VideoTarget::Video(VideoFormat::AnimatedWebp)
        }
        "mp3" => settings.video.target = VideoTarget::Audio(AudioFormat::Mp3),
        "wav" => settings.video.target = VideoTarget::Audio(AudioFormat::Wav),
        "aac" | "m4a" => settings.video.target = VideoTarget::Audio(AudioFormat::Aac),
        "flac" => settings.video.target = VideoTarget::Audio(AudioFormat::Flac),
        "ogg" | "opus" => settings.video.target = VideoTarget::Audio(AudioFormat::Opus),
        _ => return Err(format!("Unknown target format: '{}'", to)),
    }
    Ok(())
}

fn tool_convert<F>(
    ctx: &McpContext,
    args: &Value,
    cancel: Arc<AtomicBool>,
    on_progress: F,
) -> ToolCallOutcome
where
    F: Fn(f64, f64, &str) + Send + Sync,
{
    let inputs = path_list_arg(args, "inputs");
    if inputs.is_empty() {
        return ToolCallOutcome::err("'inputs' must contain at least one file or directory path");
    }
    if let Err(e) = ctx.validate_paths(&inputs) {
        return ToolCallOutcome::err(e);
    }
    if let Some(out_dir) = str_arg(args, "outputDir") {
        if let Err(e) = ctx.validate_path(Path::new(out_dir)) {
            return ToolCallOutcome::err(e);
        }
    }

    let mut settings = match str_arg(args, "preset") {
        Some("web" | "web_optimized") => Preset::WebOptimized.to_settings(),
        Some("high" | "high_quality") => Preset::HighQuality.to_settings(),
        Some("small" | "small_size") => Preset::SmallSize.to_settings(),
        Some(other) => {
            return ToolCallOutcome::err(format!("Unknown preset '{}'. Use web, high, or small.", other));
        }
        None => ConversionSettings::default(),
    };

    if let Some(to) = str_arg(args, "to") {
        if let Err(e) = apply_target_format(&mut settings, to) {
            return ToolCallOutcome::err(e);
        }
    }

    if let Some(q) = args.get("quality").and_then(|v| v.as_u64()) {
        let clamped = (q as u8).clamp(1, 100);
        settings.image.quality = clamped;
        settings.video.quality = clamped;
    }
    if let Some(v) = args.get("lossless").and_then(|v| v.as_bool()) {
        settings.image.lossless = v;
    }

    let max_w = args.get("maxWidth").and_then(|v| v.as_u64()).map(|v| v as u32);
    let max_h = args.get("maxHeight").and_then(|v| v.as_u64()).map(|v| v as u32);
    if let Some(pct) = args.get("percent").and_then(|v| v.as_f64()) {
        let mode = ResizeMode::Percentage { percent: pct as f32 };
        settings.image.resize = mode.clone();
        settings.video.resize = mode;
    } else if max_w.is_some() || max_h.is_some() {
        let mode = ResizeMode::MaxDimensions {
            width: max_w,
            height: max_h,
        };
        settings.image.resize = mode.clone();
        settings.video.resize = mode;
    }

    if let Some(keep) = args.get("keepMetadata").and_then(|v| v.as_bool()) {
        settings.image.strip_metadata = !keep;
        settings.video.strip_metadata = !keep;
    }
    if let Some(no_hw) = args.get("noHw").and_then(|v| v.as_bool()) {
        settings.video.use_hardware_accel = !no_hw;
    }
    if let Some(t) = args.get("threads").and_then(|v| v.as_u64()) {
        settings.max_cpu_threads = Some(t as usize);
    }
    settings.image.target_size_kb = args.get("targetKb").and_then(|v| v.as_u64()).map(|v| v as u32);
    settings.video.target_size_mb = args.get("targetMb").and_then(|v| v.as_f64()).map(|v| v as f32);
    if let Some(rot) = args.get("rotate").and_then(|v| v.as_u64()) {
        settings.image.rotate = rot as u16;
        settings.video.rotate = rot as u16;
    }
    if let Some(v) = args.get("flipH").and_then(|v| v.as_bool()) {
        settings.image.flip_horizontal = v;
        settings.video.flip_horizontal = v;
    }
    if let Some(v) = args.get("flipV").and_then(|v| v.as_bool()) {
        settings.image.flip_vertical = v;
    }
    if let Some(c) = str_arg(args, "crop") {
        settings.image.crop_aspect = match c {
            "1:1" => Some(AspectCrop::Square1x1),
            "16:9" => Some(AspectCrop::Landscape16x9),
            "9:16" => Some(AspectCrop::Portrait9x16),
            "4:3" => Some(AspectCrop::Standard4x3),
            "3:2" => Some(AspectCrop::Photo3x2),
            other => return ToolCallOutcome::err(format!("Unknown crop ratio '{}'", other)),
        };
    }
    if let Some(v) = args.get("autoTrim").and_then(|v| v.as_bool()) {
        settings.image.auto_trim_border = v;
    }
    if let Some(tol) = args.get("trimTolerance").and_then(|v| v.as_u64()) {
        settings.image.auto_trim_tolerance = (tol as u8).min(64);
    }
    if let Some(v) = args.get("autoLevels").and_then(|v| v.as_bool()) {
        settings.image.auto_levels = v;
    }
    if let Some(flt) = str_arg(args, "filter") {
        settings.image.filter_preset = match flt {
            "none" => ImageFilterPreset::None,
            "sepia" => ImageFilterPreset::Sepia,
            "warm" => ImageFilterPreset::Warm,
            "cool" => ImageFilterPreset::Cool,
            "vintage" => ImageFilterPreset::Vintage,
            "bw" => ImageFilterPreset::HighContrastBw,
            "invert" => ImageFilterPreset::Invert,
            other => return ToolCallOutcome::err(format!("Unknown filter '{}'", other)),
        };
    }
    if let Some(pad) = args.get("framePaddingPx").and_then(|v| v.as_u64()) {
        settings.image.frame_padding_px = pad as u32;
    }
    if let Some(g) = str_arg(args, "frameGradient") {
        settings.image.frame_gradient = match g {
            "none" => FrameGradient::None,
            "indigo" => FrameGradient::Indigo,
            "sunset" => FrameGradient::Sunset,
            "ocean" => FrameGradient::Ocean,
            "emerald" => FrameGradient::Emerald,
            "midnight" => FrameGradient::Midnight,
            other => return ToolCallOutcome::err(format!("Unknown frameGradient '{}'", other)),
        };
    }
    if let Some(v) = args.get("dropShadow").and_then(|v| v.as_bool()) {
        settings.image.drop_shadow = v;
    }
    if let Some(v) = args.get("mute").and_then(|v| v.as_bool()) {
        settings.video.mute_audio = v;
    }
    if let Some(fps) = args.get("fps").and_then(|v| v.as_u64()) {
        settings.video.max_fps = Some(fps as u8);
    }
    settings.video.trim_start_secs = args.get("trimStart").and_then(|v| v.as_f64());
    settings.video.trim_end_secs = args.get("trimEnd").and_then(|v| v.as_f64());
    if let Some(sp) = args.get("speed").and_then(|v| v.as_f64()) {
        settings.video.speed = sp as f32;
    }
    if let Some(db) = args.get("gainDb").and_then(|v| v.as_f64()) {
        settings.video.audio_gain_db = db as f32;
    }
    if let Some(rf) = str_arg(args, "videoReframe") {
        settings.video.aspect_reframe = match rf {
            "none" => VideoAspectReframe::None,
            "9:16" => VideoAspectReframe::Portrait9x16,
            "4:5" => VideoAspectReframe::Portrait4x5,
            "1:1" => VideoAspectReframe::Square1x1,
            "16:9" => VideoAspectReframe::Landscape16x9,
            other => return ToolCallOutcome::err(format!("Unknown videoReframe '{}'", other)),
        };
    }
    if let Some(pm) = str_arg(args, "playbackMode") {
        settings.video.playback_mode = match pm {
            "normal" => VideoPlaybackMode::Normal,
            "reverse" => VideoPlaybackMode::Reverse,
            "boomerang" => VideoPlaybackMode::Boomerang,
            other => return ToolCallOutcome::err(format!("Unknown playbackMode '{}'", other)),
        };
    }
    if let Some(v) = args.get("remux").and_then(|v| v.as_bool()) {
        settings.video.remux_only = v;
    }
    if let Some(v) = args.get("denoiseAudio").and_then(|v| v.as_bool()) {
        settings.video.denoise_audio = v;
    }
    if let Some(v) = args.get("deinterlace").and_then(|v| v.as_bool()) {
        settings.video.deinterlace = v;
    }
    if let Some(v) = args.get("videoSharpen").and_then(|v| v.as_bool()) {
        settings.video.video_sharpen = v;
    }
    if let Some(twm_text) = str_arg(args, "textWatermark") {
        settings.image.text_watermark = Some(TextWatermarkSettings {
            text: twm_text.to_string(),
            font_size_percent: 4.5,
            color: "#ffffff".into(),
            opacity: 0.75,
            margin_px: 24,
            position: WatermarkPosition::BottomRight,
            tiled: args
                .get("tiledWatermark")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        });
    }

    settings.output = OutputSettings {
        output_dir: str_arg(args, "outputDir").map(PathBuf::from),
        suffix: str_arg(args, "suffix").unwrap_or("").to_string(),
        overwrite: args.get("overwrite").and_then(|v| v.as_bool()).unwrap_or(false),
        bundle_zip: args.get("bundleZip").and_then(|v| v.as_bool()).unwrap_or(false),
        trash_originals: args
            .get("trashOriginals")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        ..Default::default()
    };

    let inspected = inspect_paths(&inputs, ctx.ffmpeg_path.as_deref(), ctx.ffprobe_path.as_deref());
    if inspected.is_empty() {
        return ToolCallOutcome::err("No supported media files found in 'inputs'");
    }

    let dry_run = args.get("dryRun").and_then(|v| v.as_bool()).unwrap_or(false);
    if dry_run {
        let planned: Vec<Value> = inspected
            .iter()
            .map(|m| {
                let ext = match m.media_kind {
                    MediaKind::Image | MediaKind::Pdf => settings.image.format.extension(),
                    MediaKind::Video | MediaKind::Audio => settings.video.target.extension(),
                };
                let src = PathBuf::from(&m.path);
                let out_dir = settings
                    .output
                    .output_dir
                    .clone()
                    .unwrap_or_else(|| src.parent().unwrap_or(Path::new(".")).to_path_buf());
                let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
                let planned_path =
                    out_dir.join(format!("{}{}.{}", stem, settings.output.suffix, ext));
                json!({
                    "inputPath": m.path,
                    "mediaKind": m.media_kind,
                    "inputBytes": m.size_bytes,
                    "plannedOutputPath": planned_path.to_string_lossy()
                })
            })
            .collect();
        return ToolCallOutcome::ok(json!({
            "dryRun": true,
            "count": planned.len(),
            "planned": planned
        }));
    }

    let total = inspected.len() as f64;
    let input_sizes: std::collections::HashMap<String, u64> = inspected
        .iter()
        .map(|m| (m.id.clone(), m.size_bytes))
        .collect();

    let jobs: Vec<JobItem> = inspected
        .into_iter()
        .map(|m| JobItem {
            id: m.id,
            path: PathBuf::from(m.path),
            media_kind: m.media_kind,
            duration_secs: m.duration_secs,
            width: m.width,
            height: m.height,
            relative_dir: m.relative_dir,
            image_override: None,
            video_override: None,
        })
        .collect();

    let completed_items = Arc::new(Mutex::new(Vec::<Value>::new()));
    let failed_items = Arc::new(Mutex::new(Vec::<Value>::new()));
    let summary = Arc::new(Mutex::new(json!({})));
    let done_counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let comp_ref = Arc::clone(&completed_items);
    let fail_ref = Arc::clone(&failed_items);
    let sum_ref = Arc::clone(&summary);
    let done_ref = Arc::clone(&done_counter);

    run_batch(
        jobs,
        &settings,
        &ctx.hardware,
        ctx.ffmpeg_path.as_deref(),
        cancel,
        move |ev| match ev {
            ProgressEvent::FileCompleted {
                id,
                output_path,
                output_bytes,
                duration_ms,
            } => {
                let in_bytes = input_sizes.get(&id).copied().unwrap_or(0);
                let saved_pct = if in_bytes > 0 {
                    ((1.0 - (output_bytes as f64 / in_bytes as f64)) * 100.0).round() as i64
                } else {
                    0
                };
                if let Ok(mut guard) = comp_ref.lock() {
                    guard.push(json!({
                        "inputPath": id,
                        "outputPath": output_path,
                        "inputBytes": in_bytes,
                        "outputBytes": output_bytes,
                        "savedPercent": saved_pct,
                        "durationMs": duration_ms
                    }));
                }
                let n = done_ref.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                on_progress(n as f64, total, "Converted file");
            }
            ProgressEvent::FileFailed { id, error } => {
                if let Ok(mut guard) = fail_ref.lock() {
                    guard.push(json!({ "inputPath": id, "error": error }));
                }
                let n = done_ref.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                on_progress(n as f64, total, "File failed");
            }
            ProgressEvent::BatchFinished {
                completed,
                failed,
                cancelled,
                total_duration_ms,
            } => {
                if let Ok(mut guard) = sum_ref.lock() {
                    *guard = json!({
                        "completed": completed,
                        "failed": failed,
                        "cancelled": cancelled,
                        "totalDurationMs": total_duration_ms
                    });
                }
            }
            _ => {}
        },
    );

    let comp = completed_items.lock().map(|g| g.clone()).unwrap_or_default();
    let fail = failed_items.lock().map(|g| g.clone()).unwrap_or_default();
    let sum = summary.lock().map(|g| g.clone()).unwrap_or_else(|_| json!({}));

    let is_error = comp.is_empty() && !fail.is_empty();
    ToolCallOutcome {
        payload: json!({
            "summary": sum,
            "completed": comp,
            "failed": fail
        }),
        is_error,
    }
}

fn tool_pdf_ops(ctx: &McpContext, args: &Value) -> ToolCallOutcome {
    let Some(action) = str_arg(args, "action") else {
        return ToolCallOutcome::err("Missing required 'action'");
    };
    let Some(output_str) = str_arg(args, "output") else {
        return ToolCallOutcome::err("Missing required 'output' path");
    };
    let output = PathBuf::from(output_str);
    if let Err(e) = ctx.validate_path(&output) {
        return ToolCallOutcome::err(e);
    }
    if let Some(inp) = str_arg(args, "input") {
        if let Err(e) = ctx.validate_path(Path::new(inp)) {
            return ToolCallOutcome::err(e);
        }
    }
    if let Err(e) = ctx.validate_paths(&path_list_arg(args, "inputs")) {
        return ToolCallOutcome::err(e);
    }

    let res: Result<Value, String> = match action {
        "merge_images" => {
            let inputs = path_list_arg(args, "inputs");
            let fit = match str_arg(args, "pageFit").unwrap_or("fit") {
                "a4" => PdfPageFit::A4,
                "letter" => PdfPageFit::Letter,
                _ => PdfPageFit::FitToImage,
            };
            merge_images_to_pdf(
                &inputs,
                &output,
                &ImageSettings::default(),
                fit,
                ctx.ffmpeg_path.as_deref(),
            )
            .map(|bytes| json!({ "output": output_str, "bytes": bytes, "pages": inputs.len() }))
            .map_err(|e| e.to_string())
        }
        "concat" => {
            let inputs = path_list_arg(args, "inputs");
            merge_pdfs(&inputs, &output)
                .map(|bytes| json!({ "output": output_str, "bytes": bytes, "inputs": inputs.len() }))
                .map_err(|e| e.to_string())
        }
        "split" => {
            let Some(input) = str_arg(args, "input").map(PathBuf::from) else {
                return ToolCallOutcome::err("'input' is required for split");
            };
            let pages = str_arg(args, "pages").unwrap_or("1");
            split_pdf(&input, pages, &output)
                .map(|bytes| json!({ "output": output_str, "bytes": bytes, "pages": pages }))
                .map_err(|e| e.to_string())
        }
        "compress" => {
            let Some(input) = str_arg(args, "input").map(PathBuf::from) else {
                return ToolCallOutcome::err("'input' is required for compress");
            };
            let dpi = args.get("dpi").and_then(|v| v.as_u64()).unwrap_or(120) as u16;
            let q = args.get("quality").and_then(|v| v.as_u64()).unwrap_or(70) as u8;
            compress_pdf(&input, &output, dpi, q)
                .map(|bytes| json!({ "output": output_str, "bytes": bytes, "dpi": dpi, "quality": q }))
                .map_err(|e| e.to_string())
        }
        "rotate" => {
            let Some(input) = str_arg(args, "input").map(PathBuf::from) else {
                return ToolCallOutcome::err("'input' is required for rotate");
            };
            let deg = args.get("degrees").and_then(|v| v.as_u64()).unwrap_or(90) as u16;
            let pages = str_arg(args, "pages");
            rotate_pdf_pages(&input, &output, deg, pages)
                .map(|bytes| json!({ "output": output_str, "bytes": bytes, "degrees": deg }))
                .map_err(|e| e.to_string())
        }
        "reorder" => {
            let Some(input) = str_arg(args, "input").map(PathBuf::from) else {
                return ToolCallOutcome::err("'input' is required for reorder");
            };
            let Some(pages) = str_arg(args, "pages") else {
                return ToolCallOutcome::err("'pages' spec (e.g. '3,1-2') is required for reorder");
            };
            reorder_pdf_pages(&input, &output, pages)
                .map(|bytes| json!({ "output": output_str, "bytes": bytes, "pages": pages }))
                .map_err(|e| e.to_string())
        }
        "stamp" => {
            let Some(input) = str_arg(args, "input").map(PathBuf::from) else {
                return ToolCallOutcome::err("'input' is required for stamp");
            };
            let text = str_arg(args, "text").unwrap_or("CONFIDENTIAL");
            stamp_pdf_text(&input, &output, text)
                .map(|bytes| json!({ "output": output_str, "bytes": bytes, "text": text }))
                .map_err(|e| e.to_string())
        }
        "number" => {
            let Some(input) = str_arg(args, "input").map(PathBuf::from) else {
                return ToolCallOutcome::err("'input' is required for number");
            };
            let fmt = str_arg(args, "text").unwrap_or("{n} / {total}");
            number_pdf_pages(&input, &output, fmt)
                .map(|bytes| json!({ "output": output_str, "bytes": bytes, "format": fmt }))
                .map_err(|e| e.to_string())
        }
        "extract_images" => {
            let Some(input) = str_arg(args, "input").map(PathBuf::from) else {
                return ToolCallOutcome::err("'input' is required for extract_images");
            };
            extract_pdf_images(&input, &output)
                .map(|files| json!({ "outputDir": output_str, "count": files.len(), "files": files }))
                .map_err(|e| e.to_string())
        }
        "scrub_metadata" => {
            let Some(input) = str_arg(args, "input").map(PathBuf::from) else {
                return ToolCallOutcome::err("'input' is required for scrub_metadata");
            };
            scrub_pdf_metadata(&input, &output)
                .map(|bytes| json!({ "output": output_str, "bytes": bytes }))
                .map_err(|e| e.to_string())
        }
        other => Err(format!("Unknown PDF action '{}'", other)),
    };

    match res {
        Ok(v) => ToolCallOutcome::ok(v),
        Err(e) => ToolCallOutcome::err(e),
    }
}

fn tool_image_tools(ctx: &McpContext, args: &Value) -> ToolCallOutcome {
    let Some(action) = str_arg(args, "action") else {
        return ToolCallOutcome::err("Missing required 'action'");
    };
    if let Some(inp) = str_arg(args, "input") {
        if let Err(e) = ctx.validate_path(Path::new(inp)) {
            return ToolCallOutcome::err(e);
        }
    }
    if let Some(out) = str_arg(args, "output") {
        if let Err(e) = ctx.validate_path(Path::new(out)) {
            return ToolCallOutcome::err(e);
        }
    }
    if let Err(e) = ctx.validate_paths(&path_list_arg(args, "inputs")) {
        return ToolCallOutcome::err(e);
    }

    let fmt = ImageFormat::from_extension(str_arg(args, "format").unwrap_or("png"))
        .unwrap_or(ImageFormat::Png);
    let quality = args.get("quality").and_then(|v| v.as_u64()).unwrap_or(82) as u8;
    let img_settings = ImageSettings {
        format: fmt,
        quality: quality.clamp(1, 100),
        ..Default::default()
    };

    let res: Result<Value, String> = match action {
        "icon_pack" => {
            let Some(input) = str_arg(args, "input").map(PathBuf::from) else {
                return ToolCallOutcome::err("'input' is required");
            };
            let Some(out) = str_arg(args, "output").map(PathBuf::from) else {
                return ToolCallOutcome::err("'output' directory is required");
            };
            generate_icon_pack(&input, &out, ctx.ffmpeg_path.as_deref())
                .map(|files| json!({ "count": files.len(), "files": files }))
                .map_err(|e| e.to_string())
        }
        "contact_sheet" => {
            let inputs = path_list_arg(args, "inputs");
            let Some(out) = str_arg(args, "output").map(PathBuf::from) else {
                return ToolCallOutcome::err("'output' file is required");
            };
            let opts = ContactSheetOptions {
                columns: args.get("cols").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                cell_size: args.get("cellSize").and_then(|v| v.as_u64()).unwrap_or(400) as u32,
                gap_px: args.get("gapPx").and_then(|v| v.as_u64()).unwrap_or(16) as u32,
                ..Default::default()
            };
            create_contact_sheet(&inputs, &out, &opts, &img_settings, ctx.ffmpeg_path.as_deref())
                .map(|bytes| json!({ "output": out, "bytes": bytes, "tiles": inputs.len() }))
                .map_err(|e| e.to_string())
        }
        "compare" => {
            let inputs = path_list_arg(args, "inputs");
            if inputs.len() < 2 {
                return ToolCallOutcome::err("'inputs' requires 2 image paths for compare");
            }
            let Some(out) = str_arg(args, "output").map(PathBuf::from) else {
                return ToolCallOutcome::err("'output' file is required");
            };
            let opts = ComparisonOptions {
                vertical: args.get("vertical").and_then(|v| v.as_bool()).unwrap_or(false),
                gap_px: args.get("gapPx").and_then(|v| v.as_u64()).unwrap_or(8) as u32,
                left_label: str_arg(args, "leftLabel").map(String::from),
                right_label: str_arg(args, "rightLabel").map(String::from),
                ..Default::default()
            };
            create_comparison_image(
                &inputs[0],
                &inputs[1],
                &out,
                &opts,
                &img_settings,
                ctx.ffmpeg_path.as_deref(),
            )
            .map(|bytes| json!({ "output": out, "bytes": bytes }))
            .map_err(|e| e.to_string())
        }
        "srcset" => {
            let Some(input) = str_arg(args, "input").map(PathBuf::from) else {
                return ToolCallOutcome::err("'input' is required");
            };
            let Some(out) = str_arg(args, "output").map(PathBuf::from) else {
                return ToolCallOutcome::err("'output' directory is required");
            };
            generate_responsive_set(&input, &out, &img_settings, ctx.ffmpeg_path.as_deref())
                .map(|r| serde_json::to_value(r).unwrap_or_default())
                .map_err(|e| e.to_string())
        }
        "slice" => {
            let Some(input) = str_arg(args, "input").map(PathBuf::from) else {
                return ToolCallOutcome::err("'input' is required");
            };
            let Some(out) = str_arg(args, "output").map(PathBuf::from) else {
                return ToolCallOutcome::err("'output' directory is required");
            };
            let cols = args.get("cols").and_then(|v| v.as_u64()).unwrap_or(3) as u32;
            let rows = args.get("rows").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
            split_image_grid(&input, &out, cols, rows, &img_settings, ctx.ffmpeg_path.as_deref())
                .map(|files| json!({ "count": files.len(), "files": files }))
                .map_err(|e| e.to_string())
        }
        "sprite_sheet" => {
            let inputs = path_list_arg(args, "inputs");
            let Some(out) = str_arg(args, "output").map(PathBuf::from) else {
                return ToolCallOutcome::err("'output' file is required");
            };
            let pad = args.get("gapPx").and_then(|v| v.as_u64()).unwrap_or(4) as u32;
            create_sprite_sheet(&inputs, &out, pad, &img_settings, ctx.ffmpeg_path.as_deref())
                .map(|r| serde_json::to_value(r).unwrap_or_default())
                .map_err(|e| e.to_string())
        }
        "palette" => {
            let Some(input) = str_arg(args, "input").map(PathBuf::from) else {
                return ToolCallOutcome::err("'input' is required");
            };
            let count = args.get("count").and_then(|v| v.as_u64()).unwrap_or(6) as usize;
            extract_color_palette(&input, count, ctx.ffmpeg_path.as_deref())
                .map(|swatches| json!({ "swatches": swatches }))
                .map_err(|e| e.to_string())
        }
        "dedupe" => {
            let inputs = path_list_arg(args, "inputs");
            let threshold = args.get("threshold").and_then(|v| v.as_u64()).unwrap_or(5) as u32;
            let inspected = inspect_paths(&inputs, ctx.ffmpeg_path.as_deref(), ctx.ffprobe_path.as_deref());
            let img_paths: Vec<PathBuf> = inspected
                .into_iter()
                .filter(|m| m.media_kind == MediaKind::Image)
                .map(|m| PathBuf::from(m.path))
                .collect();
            let groups = find_duplicate_images(&img_paths, threshold, ctx.ffmpeg_path.as_deref());
            Ok(json!({ "scanned": img_paths.len(), "duplicateGroups": groups }))
        }
        "qr" => {
            let Some(content) = str_arg(args, "content") else {
                return ToolCallOutcome::err("'content' is required for qr");
            };
            let Some(out) = str_arg(args, "output").map(PathBuf::from) else {
                return ToolCallOutcome::err("'output' file (.svg or .png) is required");
            };
            let dark = str_arg(args, "darkHex").unwrap_or("#000000");
            let light = str_arg(args, "lightHex").unwrap_or("#ffffff");
            let size = args.get("sizePx").and_then(|v| v.as_u64()).unwrap_or(512) as u32;
            save_qr_code(content, dark, light, size, &out)
                .map(|bytes| json!({ "output": out, "bytes": bytes }))
                .map_err(|e| e.to_string())
        }
        "data_url" => {
            let Some(input) = str_arg(args, "input").map(PathBuf::from) else {
                return ToolCallOutcome::err("'input' is required");
            };
            export_image_data_url(&input, &img_settings, ctx.ffmpeg_path.as_deref())
                .map(|exp| serde_json::to_value(exp).unwrap_or_default())
                .map_err(|e| e.to_string())
        }
        other => Err(format!("Unknown image action '{}'", other)),
    };

    match res {
        Ok(v) => ToolCallOutcome::ok(v),
        Err(e) => ToolCallOutcome::err(e),
    }
}

fn tool_media_tools(ctx: &McpContext, args: &Value) -> ToolCallOutcome {
    let Some(ffmpeg) = ctx.ffmpeg_path.as_deref() else {
        return ToolCallOutcome::err("FFmpeg binary not found on host");
    };
    let Some(action) = str_arg(args, "action") else {
        return ToolCallOutcome::err("Missing required 'action'");
    };
    let Some(output_str) = str_arg(args, "output") else {
        return ToolCallOutcome::err("Missing required 'output' path");
    };
    let output = PathBuf::from(output_str);
    if let Err(e) = ctx.validate_path(&output) {
        return ToolCallOutcome::err(e);
    }
    for key in ["input", "cover", "audio"] {
        if let Some(p) = str_arg(args, key) {
            if let Err(e) = ctx.validate_path(Path::new(p)) {
                return ToolCallOutcome::err(e);
            }
        }
    }
    if let Err(e) = ctx.validate_paths(&path_list_arg(args, "inputs")) {
        return ToolCallOutcome::err(e);
    }

    let res: Result<Value, String> = match action {
        "merge_videos" => {
            let inputs = path_list_arg(args, "inputs");
            merge_videos(&inputs, &output, &VideoSettings::default(), &ctx.hardware, ffmpeg)
                .map(|bytes| json!({ "output": output_str, "bytes": bytes, "count": inputs.len() }))
                .map_err(|e| e.to_string())
        }
        "merge_audio" => {
            let inputs = path_list_arg(args, "inputs");
            merge_audio(&inputs, &output, &VideoSettings::default(), ffmpeg)
                .map(|bytes| json!({ "output": output_str, "bytes": bytes, "count": inputs.len() }))
                .map_err(|e| e.to_string())
        }
        "storyboard" => {
            let Some(input) = str_arg(args, "input").map(PathBuf::from) else {
                return ToolCallOutcome::err("'input' video path is required");
            };
            let frames = args.get("frames").and_then(|v| v.as_u64()).unwrap_or(9) as u32;
            let cols = args.get("cols").and_then(|v| v.as_u64()).unwrap_or(3) as u32;
            create_video_storyboard(&input, &output, frames, cols, ffmpeg, ctx.ffprobe_path.as_deref())
                .map(|bytes| json!({ "output": output_str, "bytes": bytes, "frames": frames }))
                .map_err(|e| e.to_string())
        }
        "extract_frames" => {
            let Some(input) = str_arg(args, "input").map(PathBuf::from) else {
                return ToolCallOutcome::err("'input' video path is required");
            };
            let fps = args.get("fps").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
            let fmt = str_arg(args, "format").unwrap_or("png");
            extract_video_frames(&input, &output, fps, fmt, ffmpeg)
                .map(|files| json!({ "outputDir": output_str, "count": files.len(), "files": files }))
                .map_err(|e| e.to_string())
        }
        "waveform" => {
            let Some(input) = str_arg(args, "input").map(PathBuf::from) else {
                return ToolCallOutcome::err("'input' audio path is required");
            };
            let w = args.get("width").and_then(|v| v.as_u64()).unwrap_or(1200) as u32;
            let h = args.get("height").and_then(|v| v.as_u64()).unwrap_or(320) as u32;
            let color = str_arg(args, "colorHex").unwrap_or("#6366f1");
            let bg = str_arg(args, "bgHex").unwrap_or("#18181b");
            create_audio_waveform(&input, &output, w, h, color, bg, ffmpeg)
                .map(|bytes| json!({ "output": output_str, "bytes": bytes, "width": w, "height": h }))
                .map_err(|e| e.to_string())
        }
        "slideshow" => {
            let inputs = path_list_arg(args, "inputs");
            let secs = args.get("seconds").and_then(|v| v.as_f64()).unwrap_or(3.0) as f32;
            let w = args.get("width").and_then(|v| v.as_u64()).unwrap_or(1920) as u32;
            let h = args.get("height").and_then(|v| v.as_u64()).unwrap_or(1080) as u32;
            let audio = str_arg(args, "audio").map(PathBuf::from);
            create_slideshow_video(&inputs, &output, secs, w, h, audio.as_deref(), ffmpeg)
                .map(|bytes| json!({ "output": output_str, "bytes": bytes, "slides": inputs.len() }))
                .map_err(|e| e.to_string())
        }
        "audiogram" => {
            let Some(input) = str_arg(args, "input").map(PathBuf::from) else {
                return ToolCallOutcome::err("'input' audio path is required");
            };
            let Some(cover) = str_arg(args, "cover").map(PathBuf::from) else {
                return ToolCallOutcome::err("'cover' image path is required");
            };
            let w = args.get("width").and_then(|v| v.as_u64()).unwrap_or(1280) as u32;
            let h = args.get("height").and_then(|v| v.as_u64()).unwrap_or(720) as u32;
            create_podcast_video(&input, &cover, &output, w, h, ffmpeg)
                .map(|bytes| json!({ "output": output_str, "bytes": bytes }))
                .map_err(|e| e.to_string())
        }
        "split_segments" => {
            let Some(input) = str_arg(args, "input").map(PathBuf::from) else {
                return ToolCallOutcome::err("'input' media path is required");
            };
            let secs = args.get("seconds").and_then(|v| v.as_f64()).unwrap_or(60.0) as f32;
            split_media_segments(&input, &output, secs, ffmpeg)
                .map(|parts| json!({ "outputDir": output_str, "count": parts.len(), "parts": parts }))
                .map_err(|e| e.to_string())
        }
        other => Err(format!("Unknown media action '{}'", other)),
    };

    match res {
        Ok(v) => ToolCallOutcome::ok(v),
        Err(e) => ToolCallOutcome::err(e),
    }
}

fn extract_step_outputs(payload: &Value) -> Vec<String> {
    if let Some(comp) = payload.get("completed").and_then(|v| v.as_array()) {
        let list: Vec<String> = comp
            .iter()
            .filter_map(|item| item.get("outputPath").and_then(|v| v.as_str()).map(String::from))
            .collect();
        if !list.is_empty() {
            return list;
        }
    }
    for arr_key in ["files", "parts"] {
        if let Some(arr) = payload.get(arr_key).and_then(|v| v.as_array()) {
            let list: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            if !list.is_empty() {
                return list;
            }
        }
    }
    for single_key in ["output", "imagePath"] {
        if let Some(s) = payload.get(single_key).and_then(|v| v.as_str()) {
            return vec![s.to_string()];
        }
    }
    Vec::new()
}

fn resolve_placeholder(val: &Value, step_outputs: &[Vec<String>]) -> Value {
    let lookup = |token: &str| -> Option<&Vec<String>> {
        if token == "$prev" {
            step_outputs.last()
        } else if let Some(idx_s) = token.strip_prefix("$step.") {
            idx_s.parse::<usize>().ok().and_then(|i| step_outputs.get(i))
        } else {
            None
        }
    };

    match val {
        Value::String(s) => {
            if let Some(outs) = lookup(s) {
                if let Some(first) = outs.first() {
                    return Value::String(first.clone());
                }
            }
            val.clone()
        }
        Value::Array(arr) => {
            let mut expanded = Vec::new();
            for item in arr {
                if let Some(s) = item.as_str() {
                    if let Some(outs) = lookup(s) {
                        for o in outs {
                            expanded.push(Value::String(o.clone()));
                        }
                        continue;
                    }
                }
                expanded.push(resolve_placeholder(item, step_outputs));
            }
            Value::Array(expanded)
        }
        Value::Object(map) => {
            let mut next = serde_json::Map::new();
            for (k, v) in map {
                if k == "inputs" {
                    if let Some(s) = v.as_str() {
                        if let Some(outs) = lookup(s) {
                            next.insert(
                                k.clone(),
                                Value::Array(outs.iter().cloned().map(Value::String).collect()),
                            );
                            continue;
                        }
                    }
                }
                next.insert(k.clone(), resolve_placeholder(v, step_outputs));
            }
            Value::Object(next)
        }
        other => other.clone(),
    }
}

fn tool_pipeline(
    ctx: &McpContext,
    args: &Value,
    cancel: Arc<AtomicBool>,
    on_progress: &(dyn Fn(f64, f64, &str) + Send + Sync),
) -> ToolCallOutcome {
    let Some(steps) = args.get("steps").and_then(|v| v.as_array()) else {
        return ToolCallOutcome::err("'steps' must be a non-empty array");
    };
    if steps.is_empty() {
        return ToolCallOutcome::err("'steps' must contain at least one step");
    }

    let total = steps.len() as f64;
    let mut step_outputs: Vec<Vec<String>> = Vec::with_capacity(steps.len());
    let mut step_reports: Vec<Value> = Vec::with_capacity(steps.len());
    let t0 = std::time::Instant::now();
    let noop_progress = |_: f64, _: f64, _: &str| {};

    for (idx, step) in steps.iter().enumerate() {
        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            return ToolCallOutcome::err("Pipeline cancelled");
        }
        let Some(tool_name) = step.get("tool").and_then(|v| v.as_str()) else {
            return ToolCallOutcome::err(format!("Step {} missing 'tool'", idx));
        };
        if tool_name == "wandel_pipeline" {
            return ToolCallOutcome::err("Nested wandel_pipeline calls are not allowed");
        }
        let raw_args = step
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let resolved_args = resolve_placeholder(&raw_args, &step_outputs);

        let st = std::time::Instant::now();
        let outcome =
            execute_single_tool(ctx, tool_name, &resolved_args, Arc::clone(&cancel), &noop_progress);
        let elapsed_ms = st.elapsed().as_millis() as u64;

        let outs = extract_step_outputs(&outcome.payload);
        step_reports.push(json!({
            "step": idx,
            "tool": tool_name,
            "durationMs": elapsed_ms,
            "isError": outcome.is_error,
            "outputs": outs,
            "result": outcome.payload
        }));

        if outcome.is_error {
            return ToolCallOutcome {
                payload: json!({
                    "failedAtStep": idx,
                    "totalDurationMs": t0.elapsed().as_millis() as u64,
                    "steps": step_reports
                }),
                is_error: true,
            };
        }

        step_outputs.push(outs);
        on_progress((idx + 1) as f64, total, &format!("Completed step {}", idx + 1));
    }

    let final_outputs = step_outputs.last().cloned().unwrap_or_default();
    ToolCallOutcome::ok(json!({
        "completedSteps": step_reports.len(),
        "totalDurationMs": t0.elapsed().as_millis() as u64,
        "finalOutputs": final_outputs,
        "steps": step_reports
    }))
}
