use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use wandel_core::{ConversionSettings, HardwareCapabilities, JobItem, ProgressEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchFolderInfo {
    pub folder_path: String,
    pub converted_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchConvertedPayload {
    pub folder_path: String,
    pub file_name: String,
    pub output_path: String,
    pub input_bytes: u64,
    pub output_bytes: u64,
}

struct ActiveWatcher {
    _watcher: RecommendedWatcher,
    converted_count: Arc<AtomicUsize>,
}

#[derive(Default)]
pub struct WatchManager {
    watchers: Mutex<HashMap<PathBuf, ActiveWatcher>>,
}

impl WatchManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn list(&self) -> Vec<WatchFolderInfo> {
        let guard = self.watchers.lock().expect("watch mutex poisoned");
        guard
            .iter()
            .map(|(path, w)| WatchFolderInfo {
                folder_path: path.to_string_lossy().into_owned(),
                converted_count: w.converted_count.load(Ordering::Relaxed),
            })
            .collect()
    }

    pub fn stop(&self, folder: &PathBuf) {
        let mut guard = self.watchers.lock().expect("watch mutex poisoned");
        guard.remove(folder);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn start(
        &self,
        folder: PathBuf,
        settings: ConversionSettings,
        hw: HardwareCapabilities,
        ffmpeg_path: Option<PathBuf>,
        ffprobe_path: Option<PathBuf>,
        app_handle: AppHandle,
    ) -> Result<WatchFolderInfo, String> {
        if !folder.is_dir() {
            return Err(format!("Not a directory: {}", folder.display()));
        }

        let converted_count = Arc::new(AtomicUsize::new(0));
        let count_ref = Arc::clone(&converted_count);
        let ignored_outputs: Arc<Mutex<HashSet<PathBuf>>> = Arc::new(Mutex::new(HashSet::new()));
        let recent_inputs: Arc<Mutex<HashMap<PathBuf, Instant>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let folder_str = folder.to_string_lossy().into_owned();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            let Ok(event) = res else {
                return;
            };
            if !matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                return;
            }

            for path in event.paths {
                if !wandel_core::formats::is_supported_file(&path) {
                    continue;
                }

                // Skip files created by Wandel itself
                if let Ok(ignored) = ignored_outputs.lock() {
                    if ignored.contains(&path) {
                        continue;
                    }
                }

                // Debounce duplicate FS events for the same file within 3 seconds
                {
                    let mut recent = recent_inputs.lock().expect("recent mutex poisoned");
                    let now = Instant::now();
                    recent.retain(|_, t| now.duration_since(*t) < Duration::from_secs(10));
                    if let Some(prev) = recent.get(&path) {
                        if now.duration_since(*prev) < Duration::from_secs(3) {
                            continue;
                        }
                    }
                    recent.insert(path.clone(), now);
                }

                let settings = settings.clone();
                let hw = hw.clone();
                let ffmpeg = ffmpeg_path.clone();
                let ffprobe = ffprobe_path.clone();
                let ignored_ref = Arc::clone(&ignored_outputs);
                let count_clone = Arc::clone(&count_ref);
                let app = app_handle.clone();
                let folder_label = folder_str.clone();

                std::thread::spawn(move || {
                    // Brief settle delay so OS finishes copying/writing the dropped file
                    std::thread::sleep(Duration::from_millis(450));
                    let inspected = wandel_core::inspect_paths(
                        std::slice::from_ref(&path),
                        ffmpeg.as_deref(),
                        ffprobe.as_deref(),
                    );
                    let Some(meta) = inspected.into_iter().next() else {
                        return;
                    };

                    let input_bytes = meta.size_bytes;
                    let file_name = meta.file_name.clone();
                    let job = JobItem {
                        id: meta.id,
                        path: PathBuf::from(meta.path),
                        media_kind: meta.media_kind,
                        duration_secs: meta.duration_secs,
                        width: meta.width,
                        height: meta.height,
                        relative_dir: None,
                        image_override: None,
                        video_override: None,
                    };

                    wandel_core::run_batch(
                        vec![job],
                        &settings,
                        &hw,
                        ffmpeg.as_deref(),
                        Arc::new(AtomicBool::new(false)),
                        |ev| match ev {
                            ProgressEvent::FileStarted { output_path, .. } => {
                                if let Ok(mut guard) = ignored_ref.lock() {
                                    guard.insert(PathBuf::from(output_path));
                                }
                            }
                            ProgressEvent::FileCompleted {
                                output_path,
                                output_bytes,
                                ..
                            } => {
                                count_clone.fetch_add(1, Ordering::Relaxed);
                                let _ = app.emit(
                                    "wandel://watch-converted",
                                    WatchConvertedPayload {
                                        folder_path: folder_label.clone(),
                                        file_name: file_name.clone(),
                                        output_path,
                                        input_bytes,
                                        output_bytes,
                                    },
                                );
                            }
                            _ => {}
                        },
                    );
                });
            }
        })
        .map_err(|e| format!("Failed to start watcher: {}", e))?;

        watcher
            .watch(&folder, RecursiveMode::NonRecursive)
            .map_err(|e| format!("Failed to watch folder: {}", e))?;

        let info = WatchFolderInfo {
            folder_path: folder.to_string_lossy().into_owned(),
            converted_count: 0,
        };

        let mut guard = self.watchers.lock().expect("watch mutex poisoned");
        guard.insert(
            folder,
            ActiveWatcher {
                _watcher: watcher,
                converted_count,
            },
        );

        Ok(info)
    }
}
