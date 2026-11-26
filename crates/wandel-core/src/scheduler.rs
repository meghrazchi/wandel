use crate::error::WandelError;
use crate::formats::MediaKind;
use crate::image_engine::convert_image;
use crate::naming::{NamingContext, OutputPathAllocator};
use crate::pdf_engine::convert_pdf_to_images;
use crate::settings::{ConversionSettings, ImageSettings, VideoSettings};
use crate::video_engine::{convert_video, HardwareCapabilities};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Pure-Rust ZIP archive bundler for batch outputs.
pub fn bundle_files_to_zip(files: &[PathBuf], zip_path: &Path) -> crate::error::Result<u64> {
    use std::io::Write;
    if let Some(parent) = zip_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(zip_path)?;
    let mut zip_writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for path in files {
        if !path.is_file() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");
        zip_writer
            .start_file(name, options)
            .map_err(|e| WandelError::Io(std::io::Error::other(e.to_string())))?;
        let data = std::fs::read(path)?;
        zip_writer.write_all(&data)?;
    }

    zip_writer
        .finish()
        .map_err(|e| WandelError::Io(std::io::Error::other(e.to_string())))?;
    let size = std::fs::metadata(zip_path)?.len();
    Ok(size)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobItem {
    pub id: String,
    pub path: PathBuf,
    pub media_kind: MediaKind,
    pub duration_secs: Option<f64>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub relative_dir: Option<String>,
    #[serde(default)]
    pub image_override: Option<ImageSettings>,
    #[serde(default)]
    pub video_override: Option<VideoSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "camelCase")]
pub enum ProgressEvent {
    BatchStarted {
        total: usize,
    },
    FileStarted {
        id: String,
        #[serde(rename = "outputPath")]
        output_path: String,
    },
    FileProgress {
        id: String,
        percent: f32,
    },
    FileCompleted {
        id: String,
        #[serde(rename = "outputPath")]
        output_path: String,
        #[serde(rename = "outputBytes")]
        output_bytes: u64,
        #[serde(rename = "durationMs")]
        duration_ms: u64,
    },
    FileFailed {
        id: String,
        error: String,
    },
    FileCancelled {
        id: String,
    },
    BatchFinished {
        completed: usize,
        failed: usize,
        cancelled: usize,
        #[serde(rename = "totalDurationMs")]
        total_duration_ms: u64,
    },
}

/// Orchestrates parallel image conversion (all CPU cores via Rayon) and adaptive
/// parallel video/audio conversion (throttled FFmpeg workers + GPU hardware accel).
#[allow(clippy::too_many_arguments)]
pub fn run_batch<F>(
    jobs: Vec<JobItem>,
    settings: &ConversionSettings,
    hw: &HardwareCapabilities,
    ffmpeg_path: Option<&Path>,
    cancel_flag: Arc<AtomicBool>,
    emit: F,
) where
    F: Fn(ProgressEvent) + Send + Sync,
{
    let batch_start = Instant::now();
    let total = jobs.len();
    emit(ProgressEvent::BatchStarted { total });

    let allocator = Arc::new(OutputPathAllocator::new());
    let completed = Arc::new(AtomicUsize::new(0));
    let failed = Arc::new(AtomicUsize::new(0));
    let cancelled = Arc::new(AtomicUsize::new(0));
    let produced_outputs: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::new()));

    type IndexedJobs = Vec<(usize, JobItem)>;
    let mut image_jobs: IndexedJobs = Vec::new();
    let mut pdf_jobs: IndexedJobs = Vec::new();
    let mut media_jobs: IndexedJobs = Vec::new();
    for (idx, job) in jobs.into_iter().enumerate() {
        match job.media_kind {
            MediaKind::Image => image_jobs.push((idx, job)),
            MediaKind::Pdf => pdf_jobs.push((idx, job)),
            MediaKind::Video | MediaKind::Audio => media_jobs.push((idx, job)),
        }
    }

    let total_cores = num_cpus::get().max(1);
    let img_threads = settings.max_cpu_threads.unwrap_or(total_cores).clamp(1, 128);

    // Adaptive concurrency for video/audio jobs:
    // GPU mode: max 2 concurrent encodes to respect hardware encoder session limits.
    // CPU mode: 1..=4 concurrent encodes with explicit per-process thread budgets.
    let using_gpu = settings.video.use_hardware_accel && hw.has_any_gpu_encoder();
    let vid_concurrency = settings
        .max_concurrent_videos
        .unwrap_or_else(|| {
            if using_gpu {
                2.min(total_cores)
            } else {
                (total_cores / 4).clamp(1, 4)
            }
        })
        .max(1);
    let ffmpeg_threads_per_job = (total_cores / vid_concurrency).max(1);

    std::thread::scope(|s| {
        if !image_jobs.is_empty() {
            let alloc = Arc::clone(&allocator);
            let cancel = Arc::clone(&cancel_flag);
            let done_cnt = Arc::clone(&completed);
            let fail_cnt = Arc::clone(&failed);
            let cancel_cnt = Arc::clone(&cancelled);
            let outputs_ref = Arc::clone(&produced_outputs);
            let emit_ref = &emit;

            s.spawn(move || {
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(img_threads)
                    .thread_name(|i| format!("wandel-img-{}", i))
                    .build()
                    .expect("failed to build image thread pool");

                pool.install(|| {
                    image_jobs.par_iter().for_each(|(job_idx, job)| {
                        if cancel.load(Ordering::Relaxed) {
                            cancel_cnt.fetch_add(1, Ordering::Relaxed);
                            emit_ref(ProgressEvent::FileCancelled { id: job.id.clone() });
                            return;
                        }

                        let start = Instant::now();
                        let img_settings = job.image_override.as_ref().unwrap_or(&settings.image);
                        let ext = img_settings.format.extension();
                        let naming_ctx = NamingContext {
                            index: *job_idx,
                            width: job.width,
                            height: job.height,
                            relative_dir: job.relative_dir.as_deref(),
                        };
                        let out_path = match alloc.allocate_with_context(
                            &job.path,
                            ext,
                            &settings.output,
                            &naming_ctx,
                        ) {
                            Ok(p) => p,
                            Err(e) => {
                                fail_cnt.fetch_add(1, Ordering::Relaxed);
                                emit_ref(ProgressEvent::FileFailed {
                                    id: job.id.clone(),
                                    error: e.to_string(),
                                });
                                return;
                            }
                        };

                        emit_ref(ProgressEvent::FileStarted {
                            id: job.id.clone(),
                            output_path: out_path.to_string_lossy().into_owned(),
                        });
                        emit_ref(ProgressEvent::FileProgress {
                            id: job.id.clone(),
                            percent: 25.0,
                        });

                        match convert_image(&job.path, &out_path, img_settings, ffmpeg_path) {
                            Ok(bytes) => {
                                if cancel.load(Ordering::Relaxed) {
                                    let _ = std::fs::remove_file(&out_path);
                                    cancel_cnt.fetch_add(1, Ordering::Relaxed);
                                    emit_ref(ProgressEvent::FileCancelled { id: job.id.clone() });
                                    return;
                                }
                                if settings.output.trash_originals && out_path != job.path {
                                    let _ = trash::delete(&job.path);
                                }
                                if settings.output.bundle_zip {
                                    if let Ok(mut guard) = outputs_ref.lock() {
                                        guard.push(out_path.clone());
                                    }
                                }
                                done_cnt.fetch_add(1, Ordering::Relaxed);
                                emit_ref(ProgressEvent::FileCompleted {
                                    id: job.id.clone(),
                                    output_path: out_path.to_string_lossy().into_owned(),
                                    output_bytes: bytes,
                                    duration_ms: start.elapsed().as_millis() as u64,
                                });
                            }
                            Err(e) => {
                                let _ = std::fs::remove_file(&out_path);
                                fail_cnt.fetch_add(1, Ordering::Relaxed);
                                emit_ref(ProgressEvent::FileFailed {
                                    id: job.id.clone(),
                                    error: e.to_string(),
                                });
                            }
                        }
                    });
                });
            });
        }

        if !pdf_jobs.is_empty() {
            let alloc = Arc::clone(&allocator);
            let cancel = Arc::clone(&cancel_flag);
            let done_cnt = Arc::clone(&completed);
            let fail_cnt = Arc::clone(&failed);
            let cancel_cnt = Arc::clone(&cancelled);
            let emit_ref = &emit;

            s.spawn(move || {
                for (_, job) in &pdf_jobs {
                    if cancel.load(Ordering::Relaxed) {
                        cancel_cnt.fetch_add(1, Ordering::Relaxed);
                        emit_ref(ProgressEvent::FileCancelled { id: job.id.clone() });
                        continue;
                    }

                    let start = Instant::now();
                    let img_settings = job.image_override.as_ref().unwrap_or(&settings.image);

                    emit_ref(ProgressEvent::FileStarted {
                        id: job.id.clone(),
                        output_path: job.path.to_string_lossy().into_owned(),
                    });

                    let job_id = job.id.clone();
                    match convert_pdf_to_images(
                        &job.path,
                        &settings.pdf,
                        img_settings,
                        &settings.output,
                        &alloc,
                        job.relative_dir.as_deref(),
                        ffmpeg_path,
                        &cancel,
                        |pct| {
                            emit_ref(ProgressEvent::FileProgress {
                                id: job_id.clone(),
                                percent: pct,
                            });
                        },
                    ) {
                        Ok((first_out, total_bytes)) => {
                            if settings.output.trash_originals {
                                let _ = trash::delete(&job.path);
                            }
                            done_cnt.fetch_add(1, Ordering::Relaxed);
                            emit_ref(ProgressEvent::FileCompleted {
                                id: job.id.clone(),
                                output_path: first_out.to_string_lossy().into_owned(),
                                output_bytes: total_bytes,
                                duration_ms: start.elapsed().as_millis() as u64,
                            });
                        }
                        Err(WandelError::Cancelled) => {
                            cancel_cnt.fetch_add(1, Ordering::Relaxed);
                            emit_ref(ProgressEvent::FileCancelled { id: job.id.clone() });
                        }
                        Err(e) => {
                            fail_cnt.fetch_add(1, Ordering::Relaxed);
                            emit_ref(ProgressEvent::FileFailed {
                                id: job.id.clone(),
                                error: e.to_string(),
                            });
                        }
                    }
                }
            });
        }

        if !media_jobs.is_empty() {
            let alloc = Arc::clone(&allocator);
            let cancel = Arc::clone(&cancel_flag);
            let done_cnt = Arc::clone(&completed);
            let fail_cnt = Arc::clone(&failed);
            let cancel_cnt = Arc::clone(&cancelled);
            let outputs_ref = Arc::clone(&produced_outputs);
            let emit_ref = &emit;

            s.spawn(move || {
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(vid_concurrency)
                    .thread_name(|i| format!("wandel-vid-{}", i))
                    .build()
                    .expect("failed to build video thread pool");

                pool.install(|| {
                    media_jobs.par_iter().for_each(|(job_idx, job)| {
                        if cancel.load(Ordering::Relaxed) {
                            cancel_cnt.fetch_add(1, Ordering::Relaxed);
                            emit_ref(ProgressEvent::FileCancelled { id: job.id.clone() });
                            return;
                        }

                        let Some(ffmpeg) = ffmpeg_path else {
                            fail_cnt.fetch_add(1, Ordering::Relaxed);
                            emit_ref(ProgressEvent::FileFailed {
                                id: job.id.clone(),
                                error: "Bundled FFmpeg binary was not found".into(),
                            });
                            return;
                        };

                        let start = Instant::now();
                        let vid_settings = job.video_override.as_ref().unwrap_or(&settings.video);
                        let ext = vid_settings.target.extension();
                        let naming_ctx = NamingContext {
                            index: *job_idx,
                            width: job.width,
                            height: job.height,
                            relative_dir: job.relative_dir.as_deref(),
                        };
                        let out_path = match alloc.allocate_with_context(
                            &job.path,
                            ext,
                            &settings.output,
                            &naming_ctx,
                        ) {
                            Ok(p) => p,
                            Err(e) => {
                                fail_cnt.fetch_add(1, Ordering::Relaxed);
                                emit_ref(ProgressEvent::FileFailed {
                                    id: job.id.clone(),
                                    error: e.to_string(),
                                });
                                return;
                            }
                        };

                        emit_ref(ProgressEvent::FileStarted {
                            id: job.id.clone(),
                            output_path: out_path.to_string_lossy().into_owned(),
                        });

                        let mut last_emit = Instant::now();
                        let res = convert_video(
                            &job.path,
                            &out_path,
                            vid_settings,
                            job.duration_secs,
                            hw,
                            ffmpeg,
                            ffmpeg_threads_per_job,
                            &cancel,
                            |pct| {
                                if last_emit.elapsed().as_millis() >= 60 || pct >= 99.0 {
                                    last_emit = Instant::now();
                                    emit_ref(ProgressEvent::FileProgress {
                                        id: job.id.clone(),
                                        percent: pct,
                                    });
                                }
                            },
                        );

                        match res {
                            Ok(bytes) => {
                                if settings.output.trash_originals && out_path != job.path {
                                    let _ = trash::delete(&job.path);
                                }
                                if settings.output.bundle_zip {
                                    if let Ok(mut guard) = outputs_ref.lock() {
                                        guard.push(out_path.clone());
                                    }
                                }
                                done_cnt.fetch_add(1, Ordering::Relaxed);
                                emit_ref(ProgressEvent::FileCompleted {
                                    id: job.id.clone(),
                                    output_path: out_path.to_string_lossy().into_owned(),
                                    output_bytes: bytes,
                                    duration_ms: start.elapsed().as_millis() as u64,
                                });
                            }
                            Err(WandelError::Cancelled) => {
                                cancel_cnt.fetch_add(1, Ordering::Relaxed);
                                emit_ref(ProgressEvent::FileCancelled { id: job.id.clone() });
                            }
                            Err(e) => {
                                fail_cnt.fetch_add(1, Ordering::Relaxed);
                                emit_ref(ProgressEvent::FileFailed {
                                    id: job.id.clone(),
                                    error: e.to_string(),
                                });
                            }
                        }
                    });
                });
            });
        }
    });

    if settings.output.bundle_zip && !cancel_flag.load(Ordering::Relaxed) {
        let outputs = produced_outputs.lock().expect("outputs mutex poisoned");
        if !outputs.is_empty() {
            let base_dir = settings
                .output
                .output_dir
                .clone()
                .or_else(|| outputs[0].parent().map(Path::to_path_buf))
                .unwrap_or_else(|| PathBuf::from("."));
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let zip_path = base_dir.join(format!("wandel_bundle_{}.zip", ts));
            if bundle_files_to_zip(&outputs, &zip_path).is_ok() {
                for f in outputs.iter() {
                    let _ = std::fs::remove_file(f);
                }
            }
        }
    }

    emit(ProgressEvent::BatchFinished {
        completed: completed.load(Ordering::Relaxed),
        failed: failed.load(Ordering::Relaxed),
        cancelled: cancelled.load(Ordering::Relaxed),
        total_duration_ms: batch_start.elapsed().as_millis() as u64,
    });
}
