mod commands;
mod state;
mod watcher;

use state::{resolve_sidecar_binary, AppState};
#[cfg(desktop)]
use std::sync::atomic::Ordering;
#[cfg(desktop)]
use tauri::menu::{Menu, MenuItem};
#[cfg(desktop)]
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
#[cfg(desktop)]
use tauri::{Manager, WindowEvent};

/// Headless safety guard: if the desktop binary is invoked with `mcp` as its first argument,
/// run the pure-Rust MCP server over stdio or HTTP and return `true` before Tauri initializes
/// any GUI window, WebView, or system tray.
pub fn try_run_headless_mcp() -> bool {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(|s| s.as_str()) != Some("mcp") {
        return false;
    }

    let mut allowed_dirs: Vec<std::path::PathBuf> = Vec::new();
    let mut http_addr: Option<String> = None;
    let mut token: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--allowed-dir" if i + 1 < args.len() => {
                allowed_dirs.push(std::path::PathBuf::from(&args[i + 1]));
                i += 2;
            }
            "--http" if i + 1 < args.len() => {
                http_addr = Some(args[i + 1].clone());
                i += 2;
            }
            "--token" if i + 1 < args.len() => {
                token = Some(args[i + 1].clone());
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    let ffmpeg_path = resolve_sidecar_binary("ffmpeg");
    let ffprobe_path = resolve_sidecar_binary("ffprobe");
    let server = std::sync::Arc::new(wandel_mcp::McpServer::with_allowed_dirs(
        ffmpeg_path,
        ffprobe_path,
        allowed_dirs,
    ));

    let res = if let Some(addr) = http_addr {
        wandel_mcp::serve_http(server, &addr, token)
    } else {
        server.run_stdio()
    };

    if let Err(e) = res {
        eprintln!("Wandel MCP server error: {}", e);
        std::process::exit(1);
    }
    true
}

#[cfg(all(desktop, unix))]
fn ensure_cli_symlink() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let Some(dir) = exe.parent() else {
        return;
    };
    let bundled_cli = dir.join("wandel-cli");
    if !bundled_cli.is_file() {
        return;
    }
    if let (Ok(cur), Ok(cli)) = (std::fs::canonicalize(&exe), std::fs::canonicalize(&bundled_cli)) {
        if cur == cli {
            return;
        }
    }
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    let local_bin = std::path::PathBuf::from(home).join(".local").join("bin");
    if std::fs::create_dir_all(&local_bin).is_err() {
        return;
    }
    let link_path = local_bin.join("wandel");
    if let Ok(meta) = std::fs::symlink_metadata(&link_path) {
        if meta.file_type().is_symlink() {
            let should_replace = !link_path.exists()
                || std::fs::read_link(&link_path)
                    .ok()
                    .map(|p| p.to_string_lossy().contains("Wandel.app"))
                    .unwrap_or(false);
            if should_replace {
                let _ = std::fs::remove_file(&link_path);
                let _ = std::os::unix::fs::symlink(&bundled_cli, &link_path);
            }
        }
        return;
    }
    let _ = std::os::unix::fs::symlink(&bundled_cli, &link_path);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let ffmpeg_path = resolve_sidecar_binary("ffmpeg");
    let ffprobe_path = resolve_sidecar_binary("ffprobe");
    let app_state = AppState::new(ffmpeg_path, ffprobe_path);

    let builder = tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init());

    #[cfg(desktop)]
    let builder = builder
        .setup(|app| {
            #[cfg(unix)]
            ensure_cli_symlink();

            let show_item = MenuItem::with_id(app, "show", "Open Wandel", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit Wandel", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Wandel: Parallel Media Converter")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                });

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }
            let _ = tray_builder.build(app);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<AppState>();
                if state.minimize_to_tray.load(Ordering::Relaxed) {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        });

    builder
        .invoke_handler(tauri::generate_handler![
            commands::get_hardware_info,
            commands::get_preset_settings,
            commands::inspect_paths,
            commands::import_clipboard_image,
            commands::preview_image,
            commands::start_wandel,
            commands::cancel_wandel,
            commands::list_watch_folders,
            commands::start_watch_folder,
            commands::stop_watch_folder,
            commands::set_minimize_to_tray,
            commands::merge_images_to_pdf,
            commands::capture_video_frame,
            commands::inspect_detailed_metadata,
            commands::generate_icon_pack,
            commands::create_contact_sheet,
            commands::merge_videos,
            commands::merge_pdfs,
            commands::split_pdf,
            commands::find_duplicate_images,
            commands::compress_pdf,
            commands::extract_color_palette,
            commands::generate_qr_svg,
            commands::save_qr_code,
            commands::merge_audio,
            commands::export_image_data_url,
            commands::create_video_storyboard,
            commands::generate_responsive_set,
            commands::rotate_pdf_pages,
            commands::reorder_pdf_pages,
            commands::create_comparison_image,
            commands::extract_video_frames,
            commands::create_audio_waveform,
            commands::create_slideshow_video,
            commands::stamp_pdf_text,
            commands::split_image_grid,
            commands::number_pdf_pages,
            commands::create_sprite_sheet,
            commands::extract_pdf_images,
            commands::create_podcast_video,
            commands::scrub_pdf_metadata,
            commands::split_media_segments,
            commands::get_mcp_status,
            commands::render_pdf_thumbnails,
            commands::organize_pdf_pages,
            commands::run_pipeline_recipe,
            commands::export_pdf_text_file,
            commands::generate_social_kit,
            commands::convert_pdf_dark_mode,
            commands::extract_video_poster,
            commands::create_animated_gif,
            commands::extract_gif_frames,
            commands::export_pdf_markdown_file,
            commands::convert_pdf_two_up,
            commands::stitch_images,
            commands::set_pdf_metadata,
            commands::remove_blank_pdf_pages,
            commands::create_pixel_diff,
            commands::interleave_pdfs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Wandel");
}
