use crate::formats::{detect_media_kind, is_supported_file, FileMetadata, MediaKind};
use crate::image_engine::inspect_image;
use crate::pdf_engine::inspect_pdf;
use crate::video_engine::inspect_video;
use image::{ImageDecoder, ImageReader};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataEntry {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailedMetadata {
    pub file_name: String,
    pub path: String,
    pub media_kind: MediaKind,
    pub size_bytes: u64,
    pub has_exif: bool,
    pub has_gps_warning: bool,
    pub entries: Vec<MetadataEntry>,
}

/// Deep inspection for the Metadata Inspector modal: reads EXIF/GPS presence for images,
/// full stream/codec details via `ffprobe` for videos/audio, and page geometry for PDFs.
pub fn inspect_detailed_metadata(
    path: &Path,
    ffprobe_path: Option<&Path>,
) -> Option<DetailedMetadata> {
    let media_kind = detect_media_kind(path)?;
    let size_bytes = std::fs::metadata(path).ok()?.len();
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut entries = Vec::new();
    let mut has_exif = false;
    let mut has_gps_warning = false;

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_uppercase();
    entries.push(MetadataEntry {
        label: "Format".into(),
        value: ext,
    });

    match media_kind {
        MediaKind::Image => {
            if let Ok(reader) = ImageReader::open(path).and_then(|r| r.with_guessed_format()) {
                if let Ok(mut decoder) = reader.into_decoder() {
                    let (w, h) = decoder.dimensions();
                    entries.push(MetadataEntry {
                        label: "Dimensions".into(),
                        value: format!("{} × {} px", w, h),
                    });
                    entries.push(MetadataEntry {
                        label: "Color Type".into(),
                        value: format!("{:?}", decoder.color_type()),
                    });
                    if let Ok(Some(exif_bytes)) = decoder.exif_metadata() {
                        has_exif = !exif_bytes.is_empty();
                        // Detect GPS IFD pointer tag (0x8825) in big-endian or little-endian TIFF EXIF
                        has_gps_warning = exif_bytes
                            .windows(2)
                            .any(|w| w == [0x88, 0x25] || w == [0x25, 0x88]);
                        entries.push(MetadataEntry {
                            label: "EXIF Metadata".into(),
                            value: format!("Present ({} bytes)", exif_bytes.len()),
                        });
                    } else {
                        entries.push(MetadataEntry {
                            label: "EXIF Metadata".into(),
                            value: "None detected".into(),
                        });
                    }
                }
            }
        }
        MediaKind::Video | MediaKind::Audio => {
            if let Some(ffprobe) = ffprobe_path {
                if let Ok(out) = Command::new(ffprobe)
                    .args([
                        "-v",
                        "quiet",
                        "-print_format",
                        "json",
                        "-show_format",
                        "-show_streams",
                    ])
                    .arg(path)
                    .output()
                {
                    if out.status.success() {
                        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                            if let Some(dur) = json["format"]["duration"]
                                .as_str()
                                .and_then(|s| s.parse::<f64>().ok())
                            {
                                entries.push(MetadataEntry {
                                    label: "Duration".into(),
                                    value: format!("{:.2}s", dur),
                                });
                            }
                            if let Some(br) = json["format"]["bit_rate"]
                                .as_str()
                                .and_then(|s| s.parse::<u64>().ok())
                            {
                                entries.push(MetadataEntry {
                                    label: "Overall Bitrate".into(),
                                    value: format!("{} kbps", br / 1000),
                                });
                            }
                            if let Some(tags) = json["format"]["tags"].as_object() {
                                if !tags.is_empty() {
                                    has_exif = true;
                                    if tags.keys().any(|k| {
                                        let lk = k.to_ascii_lowercase();
                                        lk.contains("location") || lk.contains("gps")
                                    }) {
                                        has_gps_warning = true;
                                    }
                                    entries.push(MetadataEntry {
                                        label: "Container Tags".into(),
                                        value: format!("{} tag(s)", tags.len()),
                                    });
                                }
                            }
                            if let Some(streams) = json["streams"].as_array() {
                                for s in streams {
                                    match s["codec_type"].as_str() {
                                        Some("video") => {
                                            if let Some(codec) = s["codec_name"].as_str() {
                                                entries.push(MetadataEntry {
                                                    label: "Video Codec".into(),
                                                    value: codec.to_ascii_uppercase(),
                                                });
                                            }
                                            if let (Some(w), Some(h)) =
                                                (s["width"].as_u64(), s["height"].as_u64())
                                            {
                                                entries.push(MetadataEntry {
                                                    label: "Resolution".into(),
                                                    value: format!("{} × {}", w, h),
                                                });
                                            }
                                            if let Some(fps) = s["r_frame_rate"].as_str() {
                                                entries.push(MetadataEntry {
                                                    label: "Frame Rate".into(),
                                                    value: fps.to_string(),
                                                });
                                            }
                                        }
                                        Some("audio") => {
                                            if let Some(codec) = s["codec_name"].as_str() {
                                                entries.push(MetadataEntry {
                                                    label: "Audio Codec".into(),
                                                    value: codec.to_ascii_uppercase(),
                                                });
                                            }
                                            if let Some(sr) = s["sample_rate"].as_str() {
                                                entries.push(MetadataEntry {
                                                    label: "Sample Rate".into(),
                                                    value: format!("{} Hz", sr),
                                                });
                                            }
                                            if let Some(ch) = s["channels"].as_u64() {
                                                entries.push(MetadataEntry {
                                                    label: "Channels".into(),
                                                    value: if ch == 1 {
                                                        "Mono".into()
                                                    } else if ch == 2 {
                                                        "Stereo".into()
                                                    } else {
                                                        format!("{} ch", ch)
                                                    },
                                                });
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        MediaKind::Pdf => {
            let (pages, w, h, _) = inspect_pdf(path);
            if let Some(p) = pages {
                entries.push(MetadataEntry {
                    label: "Pages".into(),
                    value: p.to_string(),
                });
            }
            if let (Some(pw), Some(ph)) = (w, h) {
                entries.push(MetadataEntry {
                    label: "Page 1 Size".into(),
                    value: format!(
                        "{} × {} pt ({:.0} × {:.0} mm)",
                        pw,
                        ph,
                        pw as f64 * 0.3528,
                        ph as f64 * 0.3528
                    ),
                });
            }
        }
    }

    Some(DetailedMetadata {
        file_name,
        path: path.to_string_lossy().into_owned(),
        media_kind,
        size_bytes,
        has_exif,
        has_gps_warning,
        entries,
    })
}

/// Expands files and folders recursively, deduplicates supported media paths,
/// and inspects all metadata + thumbnails in parallel across all CPU cores.
pub fn inspect_paths(
    paths: &[PathBuf],
    ffmpeg_path: Option<&Path>,
    ffprobe_path: Option<&Path>,
) -> Vec<FileMetadata> {
    let mut unique_paths = HashSet::new();
    let mut ordered_paths: Vec<(PathBuf, Option<String>)> = Vec::new();

    for path in paths {
        if path.is_dir() {
            let root_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(ToString::to_string);
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                let p = entry.path().to_path_buf();
                if is_supported_file(&p) && unique_paths.insert(p.clone()) {
                    let rel = p
                        .parent()
                        .and_then(|parent| parent.strip_prefix(path).ok())
                        .map(|sub| {
                            let base = root_name.as_deref().unwrap_or("");
                            let sub_str = sub.to_string_lossy();
                            if sub_str.is_empty() {
                                base.to_string()
                            } else if base.is_empty() {
                                sub_str.into_owned()
                            } else {
                                format!("{}/{}", base, sub_str)
                            }
                        });
                    ordered_paths.push((p, rel));
                }
            }
        } else if is_supported_file(path) && unique_paths.insert(path.clone()) {
            ordered_paths.push((path.clone(), None));
        }
    }

    ordered_paths
        .par_iter()
        .filter_map(|(path, rel)| {
            inspect_single_file(path, rel.clone(), ffmpeg_path, ffprobe_path)
        })
        .collect()
}

fn inspect_single_file(
    path: &Path,
    relative_dir: Option<String>,
    ffmpeg_path: Option<&Path>,
    ffprobe_path: Option<&Path>,
) -> Option<FileMetadata> {
    let media_kind = detect_media_kind(path)?;
    let meta = std::fs::metadata(path).ok()?;
    let size_bytes = meta.len();
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    let source_format = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_uppercase();
    let path_str = path.to_string_lossy().into_owned();

    let (width, height, duration_secs, thumbnail_data_url, page_count) = match media_kind {
        MediaKind::Image => {
            let (w, h, thumb) = inspect_image(path, ffmpeg_path);
            (w, h, None, thumb, None)
        }
        MediaKind::Video => {
            let (w, h, d, thumb) = inspect_video(path, ffprobe_path, ffmpeg_path, false);
            (w, h, d, thumb, None)
        }
        MediaKind::Audio => {
            let (w, h, d, thumb) = inspect_video(path, ffprobe_path, ffmpeg_path, true);
            (w, h, d, thumb, None)
        }
        MediaKind::Pdf => {
            let (pages, w, h, thumb) = inspect_pdf(path);
            (w, h, None, thumb, pages)
        }
    };

    Some(FileMetadata {
        id: path_str.clone(),
        path: path_str,
        file_name,
        size_bytes,
        media_kind,
        source_format,
        width,
        height,
        duration_secs,
        thumbnail_data_url,
        relative_dir,
        page_count,
    })
}
