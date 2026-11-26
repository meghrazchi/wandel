use thiserror::Error;

#[derive(Debug, Error)]
pub enum WandelError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Failed to decode image: {0}")]
    ImageDecode(String),

    #[error("Failed to encode image: {0}")]
    ImageEncode(String),

    #[error("Failed to resize image: {0}")]
    Resize(String),

    #[error("FFmpeg binary not found: {0}")]
    FfmpegNotFound(String),

    #[error("FFmpeg conversion failed: {0}")]
    FfmpegFailed(String),

    #[error("FFprobe inspection failed: {0}")]
    FfprobeFailed(String),

    #[error("Conversion cancelled by user")]
    Cancelled,

    #[error("Invalid settings: {0}")]
    InvalidSettings(String),
}

pub type Result<T> = std::result::Result<T, WandelError>;
