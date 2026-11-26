use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    Image,
    Video,
    Audio,
    Pdf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageFormat {
    Png,
    Jpg,
    Webp,
    Avif,
    Gif,
    Bmp,
    Tiff,
    Ico,
    Heic,
    Jxl,
    Svg,
}

impl ImageFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpg => "jpg",
            ImageFormat::Webp => "webp",
            ImageFormat::Avif => "avif",
            ImageFormat::Gif => "gif",
            ImageFormat::Bmp => "bmp",
            ImageFormat::Tiff => "tiff",
            ImageFormat::Ico => "ico",
            ImageFormat::Heic => "heic",
            ImageFormat::Jxl => "jxl",
            ImageFormat::Svg => "svg",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ImageFormat::Png => "PNG",
            ImageFormat::Jpg => "JPG",
            ImageFormat::Webp => "WebP",
            ImageFormat::Avif => "AVIF",
            ImageFormat::Gif => "GIF",
            ImageFormat::Bmp => "BMP",
            ImageFormat::Tiff => "TIFF",
            ImageFormat::Ico => "ICO",
            ImageFormat::Heic => "HEIC",
            ImageFormat::Jxl => "JPEG XL",
            ImageFormat::Svg => "SVG",
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "png" => Some(ImageFormat::Png),
            "jpg" | "jpeg" | "jpe" => Some(ImageFormat::Jpg),
            "webp" => Some(ImageFormat::Webp),
            "avif" => Some(ImageFormat::Avif),
            "gif" => Some(ImageFormat::Gif),
            "bmp" => Some(ImageFormat::Bmp),
            "tif" | "tiff" => Some(ImageFormat::Tiff),
            "ico" => Some(ImageFormat::Ico),
            "heic" | "heif" => Some(ImageFormat::Heic),
            "jxl" => Some(ImageFormat::Jxl),
            "svg" => Some(ImageFormat::Svg),
            _ => None,
        }
    }

    pub fn can_encode(&self) -> bool {
        !matches!(self, ImageFormat::Svg)
    }

    pub fn supports_quality(&self) -> bool {
        matches!(
            self,
            ImageFormat::Jpg
                | ImageFormat::Webp
                | ImageFormat::Avif
                | ImageFormat::Jxl
                | ImageFormat::Heic
        )
    }

    pub fn supports_lossless(&self) -> bool {
        matches!(
            self,
            ImageFormat::Png
                | ImageFormat::Webp
                | ImageFormat::Jxl
                | ImageFormat::Bmp
                | ImageFormat::Tiff
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoFormat {
    Mp4H264,
    Mp4H265,
    WebmVp9,
    WebmAv1,
    Mov,
    Mkv,
    Avi,
    Gif,
    AnimatedWebp,
}

impl VideoFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            VideoFormat::Mp4H264 | VideoFormat::Mp4H265 => "mp4",
            VideoFormat::WebmVp9 | VideoFormat::WebmAv1 => "webm",
            VideoFormat::Mov => "mov",
            VideoFormat::Mkv => "mkv",
            VideoFormat::Avi => "avi",
            VideoFormat::Gif => "gif",
            VideoFormat::AnimatedWebp => "webp",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            VideoFormat::Mp4H264 => "MP4 (H.264)",
            VideoFormat::Mp4H265 => "MP4 (H.265 / HEVC)",
            VideoFormat::WebmVp9 => "WebM (VP9)",
            VideoFormat::WebmAv1 => "WebM (AV1)",
            VideoFormat::Mov => "MOV",
            VideoFormat::Mkv => "MKV",
            VideoFormat::Avi => "AVI",
            VideoFormat::Gif => "Animated GIF",
            VideoFormat::AnimatedWebp => "Animated WebP",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioFormat {
    Mp3,
    Wav,
    Aac,
    Flac,
    Opus,
}

impl AudioFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Wav => "wav",
            AudioFormat::Aac => "m4a",
            AudioFormat::Flac => "flac",
            AudioFormat::Opus => "ogg",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "MP3",
            AudioFormat::Wav => "WAV",
            AudioFormat::Aac => "AAC (M4A)",
            AudioFormat::Flac => "FLAC",
            AudioFormat::Opus => "OGG (Opus)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "format", rename_all = "snake_case")]
pub enum VideoTarget {
    Video(VideoFormat),
    Audio(AudioFormat),
}

impl VideoTarget {
    pub fn extension(&self) -> &'static str {
        match self {
            VideoTarget::Video(v) => v.extension(),
            VideoTarget::Audio(a) => a.extension(),
        }
    }
}

pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "m4v", "webm", "mov", "mkv", "avi", "flv", "wmv", "mpeg", "mpg", "ts", "3gp", "ogv",
];

pub const AUDIO_EXTENSIONS: &[&str] = &["mp3", "wav", "aac", "m4a", "flac", "ogg", "opus", "wma"];

pub fn detect_media_kind(path: &Path) -> Option<MediaKind> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    if ext == "pdf" {
        return Some(MediaKind::Pdf);
    }
    if ImageFormat::from_extension(&ext).is_some() {
        return Some(MediaKind::Image);
    }
    if VIDEO_EXTENSIONS.contains(&ext.as_str()) {
        return Some(MediaKind::Video);
    }
    if AUDIO_EXTENSIONS.contains(&ext.as_str()) {
        return Some(MediaKind::Audio);
    }
    None
}

pub fn is_supported_file(path: &Path) -> bool {
    path.is_file() && detect_media_kind(path).is_some()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileMetadata {
    pub id: String,
    pub path: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub media_kind: MediaKind,
    pub source_format: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_secs: Option<f64>,
    pub thumbnail_data_url: Option<String>,
    #[serde(default)]
    pub relative_dir: Option<String>,
    #[serde(default)]
    pub page_count: Option<u32>,
}
