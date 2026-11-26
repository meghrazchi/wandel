use crate::state::AppState;
use crate::watcher::WatchFolderInfo;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::ipc::Channel;
use tauri::{AppHandle, State};
use wandel_core::{
    ColorSwatch, ComparisonOptions, ContactSheetOptions, ConversionSettings, DataUrlExport,
    DetailedMetadata, DuplicateGroup, FileMetadata, HardwareCapabilities, ImagePreview,
    ImageSettings, JobItem, PdfPageFit, Preset, ProgressEvent, ResponsiveSetResult,
    SpriteSheetResult, VideoSettings,
};

#[tauri::command]
pub async fn preview_image(
    path: String,
    settings: ImageSettings,
    state: State<'_, AppState>,
) -> Result<ImagePreview, String> {
    let ffmpeg = state.ffmpeg_path.clone();
    let input = PathBuf::from(path);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::generate_image_preview(&input, &settings, ffmpeg.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_hardware_info(state: State<'_, AppState>) -> HardwareCapabilities {
    state.hardware.clone()
}

#[tauri::command]
pub fn get_preset_settings(preset: Preset) -> ConversionSettings {
    preset.to_settings()
}

#[tauri::command]
pub async fn inspect_paths(
    paths: Vec<String>,
    state: State<'_, AppState>,
) -> Result<Vec<FileMetadata>, String> {
    let ffmpeg = state.ffmpeg_path.clone();
    let ffprobe = state.ffprobe_path.clone();
    let path_bufs: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();

    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::inspect_paths(&path_bufs, ffmpeg.as_deref(), ffprobe.as_deref())
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_clipboard_image(
    bytes: Vec<u8>,
    ext: String,
    state: State<'_, AppState>,
) -> Result<Vec<FileMetadata>, String> {
    let clean_ext = match ext.to_ascii_lowercase().as_str() {
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "tiff" => ext.to_ascii_lowercase(),
        _ => "png".to_string(),
    };
    let clip_dir = std::env::temp_dir().join("wandel-clipboard");
    std::fs::create_dir_all(&clip_dir).map_err(|e| e.to_string())?;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let file_path = clip_dir.join(format!("clipboard_{}.{}", ts, clean_ext));
    std::fs::write(&file_path, bytes).map_err(|e| e.to_string())?;

    let ffmpeg = state.ffmpeg_path.clone();
    let ffprobe = state.ffprobe_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::inspect_paths(&[file_path], ffmpeg.as_deref(), ffprobe.as_deref())
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_wandel(
    jobs: Vec<JobItem>,
    settings: ConversionSettings,
    on_event: Channel<ProgressEvent>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut guard = state.active_cancel.lock().map_err(|e| e.to_string())?;
        if let Some(prev) = guard.replace(Arc::clone(&cancel_flag)) {
            prev.store(true, Ordering::Relaxed);
        }
    }

    let hw = state.hardware.clone();
    let ffmpeg = state.ffmpeg_path.clone();

    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::run_batch(
            jobs,
            &settings,
            &hw,
            ffmpeg.as_deref(),
            cancel_flag,
            move |event| {
                let _ = on_event.send(event);
            },
        );
    })
    .await
    .map_err(|e| e.to_string())?;

    if let Ok(mut guard) = state.active_cancel.lock() {
        *guard = None;
    }

    Ok(())
}

#[tauri::command]
pub fn cancel_wandel(state: State<'_, AppState>) -> Result<(), String> {
    let guard = state.active_cancel.lock().map_err(|e| e.to_string())?;
    if let Some(flag) = guard.as_ref() {
        flag.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[tauri::command]
pub fn list_watch_folders(state: State<'_, AppState>) -> Vec<WatchFolderInfo> {
    state.watch_manager.list()
}

#[tauri::command]
pub fn start_watch_folder(
    folder_path: String,
    settings: ConversionSettings,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<WatchFolderInfo, String> {
    state.watch_manager.start(
        PathBuf::from(folder_path),
        settings,
        state.hardware.clone(),
        state.ffmpeg_path.clone(),
        state.ffprobe_path.clone(),
        app_handle,
    )
}

#[tauri::command]
pub fn stop_watch_folder(folder_path: String, state: State<'_, AppState>) {
    state.watch_manager.stop(&PathBuf::from(folder_path));
}

#[tauri::command]
pub fn set_minimize_to_tray(enabled: bool, state: State<'_, AppState>) {
    state.minimize_to_tray.store(enabled, Ordering::Relaxed);
}

#[tauri::command]
pub async fn merge_images_to_pdf(
    paths: Vec<String>,
    output_path: String,
    settings: ImageSettings,
    page_fit: PdfPageFit,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let ffmpeg = state.ffmpeg_path.clone();
    let inputs: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let out = PathBuf::from(output_path);

    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::merge_images_to_pdf(&inputs, &out, &settings, page_fit, ffmpeg.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn capture_video_frame(
    path: String,
    timestamp_secs: f64,
    output_path: String,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let ffmpeg = state
        .ffmpeg_path
        .clone()
        .ok_or_else(|| "FFmpeg binary not found".to_string())?;

    tauri::async_runtime::spawn_blocking(move || {
        let out = std::process::Command::new(&ffmpeg)
            .args([
                "-y",
                "-hide_banner",
                "-loglevel",
                "error",
                "-ss",
                &format!("{:.3}", timestamp_secs.max(0.0)),
                "-i",
                &path,
                "-frames:v",
                "1",
                &output_path,
            ])
            .output()
            .map_err(|e| e.to_string())?;

        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
        }
        std::fs::metadata(&output_path)
            .map(|m| m.len())
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn inspect_detailed_metadata(
    path: String,
    state: State<'_, AppState>,
) -> Result<DetailedMetadata, String> {
    let ffprobe = state.ffprobe_path.clone();
    let p = PathBuf::from(path);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::inspect_detailed_metadata(&p, ffprobe.as_deref())
            .ok_or_else(|| "Could not inspect file metadata".to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn generate_icon_pack(
    path: String,
    output_dir: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let ffmpeg = state.ffmpeg_path.clone();
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_dir);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::generate_icon_pack(&input, &out, ffmpeg.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_contact_sheet(
    paths: Vec<String>,
    output_path: String,
    options: ContactSheetOptions,
    settings: ImageSettings,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let ffmpeg = state.ffmpeg_path.clone();
    let inputs: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let out = PathBuf::from(output_path);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::create_contact_sheet(&inputs, &out, &options, &settings, ffmpeg.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn merge_videos(
    paths: Vec<String>,
    output_path: String,
    settings: VideoSettings,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let ffmpeg = state
        .ffmpeg_path
        .clone()
        .ok_or_else(|| "FFmpeg binary not found".to_string())?;
    let hw = state.hardware.clone();
    let inputs: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let out = PathBuf::from(output_path);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::merge_videos(&inputs, &out, &settings, &hw, &ffmpeg)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn merge_pdfs(paths: Vec<String>, output_path: String) -> Result<u64, String> {
    let inputs: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let out = PathBuf::from(output_path);
    tauri::async_runtime::spawn_blocking(move || wandel_core::merge_pdfs(&inputs, &out))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn split_pdf(
    path: String,
    page_range: String,
    output_path: String,
) -> Result<u64, String> {
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_path);
    tauri::async_runtime::spawn_blocking(move || wandel_core::split_pdf(&input, &page_range, &out))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn find_duplicate_images(
    paths: Vec<String>,
    max_hamming: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<DuplicateGroup>, String> {
    let ffmpeg = state.ffmpeg_path.clone();
    let inputs: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let threshold = max_hamming.unwrap_or(5);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::find_duplicate_images(&inputs, threshold, ffmpeg.as_deref())
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn compress_pdf(
    path: String,
    output_path: String,
    dpi: u16,
    quality: u8,
) -> Result<u64, String> {
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_path);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::compress_pdf(&input, &out, dpi, quality)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn extract_color_palette(
    path: String,
    count: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<ColorSwatch>, String> {
    let ffmpeg = state.ffmpeg_path.clone();
    let input = PathBuf::from(path);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::extract_color_palette(&input, count.unwrap_or(6), ffmpeg.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn generate_qr_svg(content: String, dark_hex: String, light_hex: String) -> Result<String, String> {
    wandel_core::generate_qr_svg(&content, &dark_hex, &light_hex).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_qr_code(
    content: String,
    dark_hex: String,
    light_hex: String,
    size_px: u32,
    output_path: String,
) -> Result<u64, String> {
    let out = PathBuf::from(output_path);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::save_qr_code(&content, &dark_hex, &light_hex, size_px, &out)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn merge_audio(
    paths: Vec<String>,
    output_path: String,
    settings: VideoSettings,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let ffmpeg = state
        .ffmpeg_path
        .clone()
        .ok_or_else(|| "FFmpeg binary not found".to_string())?;
    let inputs: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let out = PathBuf::from(output_path);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::merge_audio(&inputs, &out, &settings, &ffmpeg)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_image_data_url(
    path: String,
    settings: ImageSettings,
    state: State<'_, AppState>,
) -> Result<DataUrlExport, String> {
    let ffmpeg = state.ffmpeg_path.clone();
    let input = PathBuf::from(path);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::export_image_data_url(&input, &settings, ffmpeg.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_video_storyboard(
    path: String,
    output_path: String,
    frame_count: Option<u32>,
    columns: Option<u32>,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let ffmpeg = state
        .ffmpeg_path
        .clone()
        .ok_or_else(|| "FFmpeg binary not found".to_string())?;
    let ffprobe = state.ffprobe_path.clone();
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_path);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::create_video_storyboard(
            &input,
            &out,
            frame_count.unwrap_or(9),
            columns.unwrap_or(3),
            &ffmpeg,
            ffprobe.as_deref(),
        )
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_responsive_set(
    path: String,
    output_dir: String,
    settings: ImageSettings,
    state: State<'_, AppState>,
) -> Result<ResponsiveSetResult, String> {
    let ffmpeg = state.ffmpeg_path.clone();
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_dir);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::generate_responsive_set(&input, &out, &settings, ffmpeg.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rotate_pdf_pages(
    path: String,
    output_path: String,
    degrees: u16,
    page_spec: Option<String>,
) -> Result<u64, String> {
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_path);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::rotate_pdf_pages(&input, &out, degrees, page_spec.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reorder_pdf_pages(
    path: String,
    output_path: String,
    page_spec: String,
) -> Result<u64, String> {
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_path);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::reorder_pdf_pages(&input, &out, &page_spec)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_comparison_image(
    first_path: String,
    second_path: String,
    output_path: String,
    options: ComparisonOptions,
    settings: ImageSettings,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let ffmpeg = state.ffmpeg_path.clone();
    let p1 = PathBuf::from(first_path);
    let p2 = PathBuf::from(second_path);
    let out = PathBuf::from(output_path);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::create_comparison_image(&p1, &p2, &out, &options, &settings, ffmpeg.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn extract_video_frames(
    path: String,
    output_dir: String,
    fps: f32,
    format_ext: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let ffmpeg = state
        .ffmpeg_path
        .clone()
        .ok_or_else(|| "FFmpeg binary not found".to_string())?;
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_dir);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::extract_video_frames(&input, &out, fps, &format_ext, &ffmpeg)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_audio_waveform(
    path: String,
    output_path: String,
    width: Option<u32>,
    height: Option<u32>,
    color_hex: Option<String>,
    bg_hex: Option<String>,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let ffmpeg = state
        .ffmpeg_path
        .clone()
        .ok_or_else(|| "FFmpeg binary not found".to_string())?;
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_path);
    let w = width.unwrap_or(1200);
    let h = height.unwrap_or(320);
    let color = color_hex.unwrap_or_else(|| "#6366f1".into());
    let bg = bg_hex.unwrap_or_else(|| "#18181b".into());
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::create_audio_waveform(&input, &out, w, h, &color, &bg, &ffmpeg)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_slideshow_video(
    paths: Vec<String>,
    output_path: String,
    secs_per_slide: f32,
    width: Option<u32>,
    height: Option<u32>,
    audio_path: Option<String>,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let ffmpeg = state
        .ffmpeg_path
        .clone()
        .ok_or_else(|| "FFmpeg binary not found".to_string())?;
    let inputs: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let out = PathBuf::from(output_path);
    let audio = audio_path.map(PathBuf::from);
    let w = width.unwrap_or(1920);
    let h = height.unwrap_or(1080);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::create_slideshow_video(
            &inputs,
            &out,
            secs_per_slide,
            w,
            h,
            audio.as_deref(),
            &ffmpeg,
        )
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stamp_pdf_text(
    path: String,
    output_path: String,
    text: String,
) -> Result<u64, String> {
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_path);
    tauri::async_runtime::spawn_blocking(move || wandel_core::stamp_pdf_text(&input, &out, &text))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn split_image_grid(
    path: String,
    output_dir: String,
    cols: u32,
    rows: u32,
    settings: ImageSettings,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let ffmpeg = state.ffmpeg_path.clone();
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_dir);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::split_image_grid(&input, &out, cols, rows, &settings, ffmpeg.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn number_pdf_pages(
    path: String,
    output_path: String,
    format_template: String,
) -> Result<u64, String> {
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_path);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::number_pdf_pages(&input, &out, &format_template)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_sprite_sheet(
    paths: Vec<String>,
    output_path: String,
    padding_px: Option<u32>,
    settings: ImageSettings,
    state: State<'_, AppState>,
) -> Result<SpriteSheetResult, String> {
    let ffmpeg = state.ffmpeg_path.clone();
    let inputs: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let out = PathBuf::from(output_path);
    let pad = padding_px.unwrap_or(4);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::create_sprite_sheet(&inputs, &out, pad, &settings, ffmpeg.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn extract_pdf_images(
    path: String,
    output_dir: String,
) -> Result<Vec<String>, String> {
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_dir);
    tauri::async_runtime::spawn_blocking(move || wandel_core::extract_pdf_images(&input, &out))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_podcast_video(
    audio_path: String,
    cover_image: String,
    output_path: String,
    width: Option<u32>,
    height: Option<u32>,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let ffmpeg = state
        .ffmpeg_path
        .clone()
        .ok_or_else(|| "FFmpeg binary not found".to_string())?;
    let audio = PathBuf::from(audio_path);
    let cover = PathBuf::from(cover_image);
    let out = PathBuf::from(output_path);
    let w = width.unwrap_or(1280);
    let h = height.unwrap_or(720);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::create_podcast_video(&audio, &cover, &out, w, h, &ffmpeg)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn scrub_pdf_metadata(path: String, output_path: String) -> Result<u64, String> {
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_path);
    tauri::async_runtime::spawn_blocking(move || wandel_core::scrub_pdf_metadata(&input, &out))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn split_media_segments(
    path: String,
    output_dir: String,
    segment_secs: f32,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let ffmpeg = state
        .ffmpeg_path
        .clone()
        .ok_or_else(|| "FFmpeg binary not found".to_string())?;
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_dir);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::split_media_segments(&input, &out, segment_secs, &ffmpeg)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_mcp_status(state: State<'_, AppState>) -> serde_json::Value {
    let server = wandel_mcp::McpServer::new(state.ffmpeg_path.clone(), state.ffprobe_path.clone());
    let mut info = server.health_check();
    let cli_command = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|dir| dir.join(if cfg!(windows) { "wandel.exe" } else { "wandel" })))
        .filter(|p| p.is_file())
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "wandel".to_string());
    if let Some(obj) = info.as_object_mut() {
        obj.insert("cliCommand".into(), serde_json::Value::String(cli_command));
    }
    info
}

#[tauri::command]
pub async fn render_pdf_thumbnails(
    path: String,
    max_edge_px: Option<u32>,
) -> Result<Vec<wandel_core::PdfPageThumb>, String> {
    let input = PathBuf::from(path);
    let edge = max_edge_px.unwrap_or(240);
    tauri::async_runtime::spawn_blocking(move || wandel_core::render_pdf_thumbnails(&input, edge))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn organize_pdf_pages(
    path: String,
    output_path: String,
    plan: Vec<wandel_core::PdfPagePlan>,
) -> Result<u64, String> {
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_path);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::organize_pdf_pages(&input, &out, &plan)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn run_pipeline_recipe(
    recipe_json: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let recipe: serde_json::Value =
        serde_json::from_str(&recipe_json).map_err(|e| format!("Invalid recipe JSON: {}", e))?;
    let ffmpeg = state.ffmpeg_path.clone();
    let ffprobe = state.ffprobe_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let server = wandel_mcp::McpServer::new(ffmpeg, ffprobe);
        let (payload, is_err) = server.run_pipeline_recipe(&recipe);
        if is_err {
            Err(payload
                .get("error")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| payload.to_string()))
        } else {
            Ok(payload)
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn export_pdf_text_file(
    path: String,
    output_path: String,
) -> Result<(usize, u64), String> {
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_path);
    tauri::async_runtime::spawn_blocking(move || wandel_core::export_pdf_text_file(&input, &out))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_social_kit(
    path: String,
    output_dir: String,
    format: wandel_core::ImageFormat,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let ffmpeg = state.ffmpeg_path.clone();
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_dir);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::generate_social_kit(&input, &out, format, ffmpeg.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn convert_pdf_dark_mode(
    path: String,
    output_path: String,
    dpi: Option<u16>,
) -> Result<u64, String> {
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_path);
    let d = dpi.unwrap_or(150);
    tauri::async_runtime::spawn_blocking(move || wandel_core::convert_pdf_dark_mode(&input, &out, d))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn extract_video_poster(
    path: String,
    output_path: String,
    timestamp_secs: Option<f32>,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let ffmpeg = state
        .ffmpeg_path
        .clone()
        .ok_or_else(|| "FFmpeg binary not found".to_string())?;
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_path);
    let ts = timestamp_secs.unwrap_or(1.0);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::extract_video_poster(&input, &out, ts, &ffmpeg)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_animated_gif(
    paths: Vec<String>,
    output_path: String,
    delay_ms: Option<u32>,
    max_width: Option<u32>,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let ffmpeg = state.ffmpeg_path.clone();
    let inputs: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let out = PathBuf::from(output_path);
    let delay = delay_ms.unwrap_or(300);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::create_animated_gif(&inputs, &out, delay, max_width, ffmpeg.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn extract_gif_frames(
    path: String,
    output_dir: String,
    format: wandel_core::ImageFormat,
) -> Result<Vec<String>, String> {
    let input = PathBuf::from(path);
    let out = PathBuf::from(output_dir);
    tauri::async_runtime::spawn_blocking(move || {
        wandel_core::extract_gif_frames(&input, &out, format)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}
