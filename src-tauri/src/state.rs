use crate::watcher::WatchManager;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use wandel_core::HardwareCapabilities;

pub struct AppState {
    pub ffmpeg_path: Option<PathBuf>,
    pub ffprobe_path: Option<PathBuf>,
    pub hardware: HardwareCapabilities,
    pub active_cancel: Mutex<Option<Arc<AtomicBool>>>,
    pub watch_manager: WatchManager,
    pub minimize_to_tray: AtomicBool,
}

impl AppState {
    pub fn new(ffmpeg_path: Option<PathBuf>, ffprobe_path: Option<PathBuf>) -> Self {
        let hardware =
            wandel_core::probe_hardware(ffmpeg_path.as_deref(), ffprobe_path.as_deref());
        Self {
            ffmpeg_path,
            ffprobe_path,
            hardware,
            active_cancel: Mutex::new(None),
            watch_manager: WatchManager::new(),
            minimize_to_tray: AtomicBool::new(false),
        }
    }
}

/// Resolves bundled Tauri sidecar binaries first, falling back to system PATH.
pub fn resolve_sidecar_binary(name: &str) -> Option<PathBuf> {
    let exe_ext = if cfg!(windows) { ".exe" } else { "" };

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Bundled sidecar next to the executable (Tauri v2 convention)
            let direct = exe_dir.join(format!("{}{}", name, exe_ext));
            if direct.is_file() {
                return Some(direct);
            }

            // Dev mode: check src-tauri/binaries/<name>-<target>
            let dev_dir = exe_dir
                .parent()
                .and_then(|p| p.parent())
                .map(|p| p.join("binaries"));
            if let Some(bin_dir) = dev_dir {
                if let Ok(entries) = std::fs::read_dir(bin_dir) {
                    for entry in entries.flatten() {
                        let fname = entry.file_name().to_string_lossy().to_string();
                        if fname.starts_with(&format!("{}-", name)) && entry.path().is_file() {
                            return Some(entry.path());
                        }
                    }
                }
            }
        }
    }

    // Fallback to PATH
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(format!("{}{}", name, exe_ext));
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}
