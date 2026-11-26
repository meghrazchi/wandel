use crate::formats::{AudioFormat, ImageFormat, VideoFormat, VideoTarget};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum ResizeMode {
    #[default]
    Original,
    MaxDimensions {
        width: Option<u32>,
        height: Option<u32>,
    },
    Exact {
        width: u32,
        height: u32,
        keep_aspect_ratio: bool,
    },
    Percentage {
        percent: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AspectCrop {
    Square1x1,
    Landscape16x9,
    Portrait9x16,
    Standard4x3,
    Photo3x2,
}

impl AspectCrop {
    pub fn ratio(self) -> (u32, u32) {
        match self {
            AspectCrop::Square1x1 => (1, 1),
            AspectCrop::Landscape16x9 => (16, 9),
            AspectCrop::Portrait9x16 => (9, 16),
            AspectCrop::Standard4x3 => (4, 3),
            AspectCrop::Photo3x2 => (3, 2),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedCrop {
    /// 0.0..=1.0
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameGradient {
    #[default]
    None,
    Indigo,
    Sunset,
    Ocean,
    Emerald,
    Midnight,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageFilterPreset {
    #[default]
    None,
    Sepia,
    Warm,
    Cool,
    Vintage,
    HighContrastBw,
    Invert,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoPlaybackMode {
    #[default]
    Normal,
    Reverse,
    Boomerang,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoAspectReframe {
    #[default]
    None,
    Portrait9x16,
    Portrait4x5,
    Square1x1,
    Landscape16x9,
}

impl VideoAspectReframe {
    pub fn target_dimensions(self) -> Option<(u32, u32)> {
        match self {
            VideoAspectReframe::None => None,
            VideoAspectReframe::Portrait9x16 => Some((1080, 1920)),
            VideoAspectReframe::Portrait4x5 => Some((1080, 1350)),
            VideoAspectReframe::Square1x1 => Some((1080, 1080)),
            VideoAspectReframe::Landscape16x9 => Some((1920, 1080)),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WatermarkPosition {
    #[default]
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
    Center,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatermarkSettings {
    pub image_path: PathBuf,
    /// 0.1..=1.0
    pub opacity: f32,
    /// Watermark width as percentage of base image width (5..=60)
    pub scale_percent: f32,
    pub margin_px: u32,
    pub position: WatermarkPosition,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfPageFit {
    #[default]
    FitToImage,
    A4,
    Letter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextWatermarkSettings {
    pub text: String,
    /// Font size as percentage of image width (2..=12)
    pub font_size_percent: f32,
    /// Hex color e.g. "#ffffff"
    pub color: String,
    /// 0.1..=1.0
    pub opacity: f32,
    pub margin_px: u32,
    pub position: WatermarkPosition,
    /// Tile watermark diagonally across the entire image canvas
    #[serde(default)]
    pub tiled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactSheetOptions {
    /// Columns (0 = auto sqrt)
    #[serde(default)]
    pub columns: u32,
    /// Max cell width/height in pixels
    #[serde(default = "default_cell_size")]
    pub cell_size: u32,
    /// Gap between cells in pixels
    #[serde(default = "default_gap")]
    pub gap_px: u32,
    /// Background hex color
    #[serde(default = "default_sheet_bg")]
    pub background_hex: String,
}

fn default_cell_size() -> u32 {
    400
}

fn default_gap() -> u32 {
    16
}

fn default_sheet_bg() -> String {
    "#18181b".into()
}

impl Default for ContactSheetOptions {
    fn default() -> Self {
        Self {
            columns: 0,
            cell_size: default_cell_size(),
            gap_px: default_gap(),
            background_hex: default_sheet_bg(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfSettings {
    /// Rasterization DPI (72, 150, 300)
    #[serde(default = "default_pdf_dpi")]
    pub dpi: u16,
    /// Optional 1-based page range e.g. "1-5,8" (empty = all pages)
    #[serde(default)]
    pub page_range: String,
}

fn default_pdf_dpi() -> u16 {
    150
}

impl Default for PdfSettings {
    fn default() -> Self {
        Self {
            dpi: 150,
            page_range: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageSettings {
    pub format: ImageFormat,
    /// 1..=100
    pub quality: u8,
    pub lossless: bool,
    pub resize: ResizeMode,
    pub never_upscale: bool,
    #[serde(default = "default_true")]
    pub strip_metadata: bool,
    /// Optional target max file size in KB (binary searches quality)
    #[serde(default)]
    pub target_size_kb: Option<u32>,
    /// Clockwise rotation in degrees: 0, 90, 180, 270
    #[serde(default)]
    pub rotate: u16,
    #[serde(default)]
    pub flip_horizontal: bool,
    #[serde(default)]
    pub flip_vertical: bool,
    #[serde(default)]
    pub crop_aspect: Option<AspectCrop>,
    /// Optional visual custom crop rectangle (normalized 0.0..=1.0)
    #[serde(default)]
    pub custom_crop: Option<NormalizedCrop>,
    /// Pad image into target aspect ratio without cropping
    #[serde(default)]
    pub pad_aspect: Option<AspectCrop>,
    /// Auto-trim excess transparent or solid-color margins before resizing/framing
    #[serde(default)]
    pub auto_trim_border: bool,
    /// Per-channel tolerance (0..=64) for auto-trim border detection
    #[serde(default = "default_trim_tolerance")]
    pub auto_trim_tolerance: u8,
    /// Rounded corner radius as % of min(width, height) (0..=50)
    #[serde(default)]
    pub corner_radius_percent: f32,
    /// Outer screenshot frame padding in pixels (0..=120)
    #[serde(default)]
    pub frame_padding_px: u32,
    /// Gradient backdrop for screenshot frame
    #[serde(default)]
    pub frame_gradient: FrameGradient,
    /// Soft drop shadow behind image when frame padding > 0
    #[serde(default)]
    pub drop_shadow: bool,
    #[serde(default)]
    pub grayscale: bool,
    #[serde(default)]
    pub sharpen: bool,
    /// Automatic 1%-99% luminance histogram contrast stretch
    #[serde(default)]
    pub auto_levels: bool,
    /// One-click color grading / filter preset
    #[serde(default)]
    pub filter_preset: ImageFilterPreset,
    /// Gaussian blur sigma (0.0..=20.0)
    #[serde(default)]
    pub blur_sigma: f32,
    /// Pixelate / mosaic block size in px (0 = off, 2..=32)
    #[serde(default)]
    pub pixelate_block: u32,
    /// Radial vignette strength (0.0..=1.0)
    #[serde(default)]
    pub vignette_strength: f32,
    /// -100..=100
    #[serde(default)]
    pub brightness: i32,
    /// -100.0..=100.0
    #[serde(default)]
    pub contrast: f32,
    /// -100.0..=100.0
    #[serde(default)]
    pub saturation: f32,
    /// Fill color for transparent pixels when encoding to JPG/BMP (default "#ffffff")
    #[serde(default = "default_bg_hex")]
    pub background_hex: Option<String>,
    #[serde(default)]
    pub watermark: Option<WatermarkSettings>,
    #[serde(default)]
    pub text_watermark: Option<TextWatermarkSettings>,
}

fn default_true() -> bool {
    true
}

fn default_bg_hex() -> Option<String> {
    Some("#ffffff".into())
}

fn default_trim_tolerance() -> u8 {
    12
}

impl Default for ImageSettings {
    fn default() -> Self {
        Self {
            format: ImageFormat::Webp,
            quality: 82,
            lossless: false,
            resize: ResizeMode::Original,
            never_upscale: true,
            strip_metadata: true,
            target_size_kb: None,
            rotate: 0,
            flip_horizontal: false,
            flip_vertical: false,
            crop_aspect: None,
            custom_crop: None,
            pad_aspect: None,
            auto_trim_border: false,
            auto_trim_tolerance: default_trim_tolerance(),
            corner_radius_percent: 0.0,
            frame_padding_px: 0,
            frame_gradient: FrameGradient::None,
            drop_shadow: false,
            grayscale: false,
            sharpen: false,
            auto_levels: false,
            filter_preset: ImageFilterPreset::None,
            blur_sigma: 0.0,
            pixelate_block: 0,
            vignette_strength: 0.0,
            brightness: 0,
            contrast: 0.0,
            saturation: 0.0,
            background_hex: default_bg_hex(),
            watermark: None,
            text_watermark: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoSettings {
    pub target: VideoTarget,
    /// 1..=100 (mapped to CRF/CQ internally)
    pub quality: u8,
    pub use_hardware_accel: bool,
    pub resize: ResizeMode,
    pub never_upscale: bool,
    pub gif_fps: u8,
    pub audio_bitrate_kbps: u16,
    #[serde(default = "default_true")]
    pub strip_metadata: bool,
    /// Optional target max file size in MB (calculates bitrate from duration)
    #[serde(default)]
    pub target_size_mb: Option<f32>,
    #[serde(default)]
    pub trim_start_secs: Option<f64>,
    #[serde(default)]
    pub trim_end_secs: Option<f64>,
    #[serde(default)]
    pub mute_audio: bool,
    #[serde(default)]
    pub max_fps: Option<u8>,
    /// Playback speed multiplier (0.25..=4.0, default 1.0)
    #[serde(default = "default_speed")]
    pub speed: f32,
    /// Clockwise rotation in degrees: 0, 90, 180, 270
    #[serde(default)]
    pub rotate: u16,
    #[serde(default)]
    pub flip_horizontal: bool,
    /// EBU R128 loudness normalization (-af loudnorm)
    #[serde(default)]
    pub normalize_loudness: bool,
    /// Optional audio sample rate (44100, 48000)
    #[serde(default)]
    pub sample_rate_hz: Option<u32>,
    /// Downmix audio to mono (-ac 1)
    #[serde(default)]
    pub mono: bool,
    /// Optional .srt / .vtt subtitle file to burn into video
    #[serde(default)]
    pub subtitle_path: Option<PathBuf>,
    /// Optional external audio file to replace / add soundtrack
    #[serde(default)]
    pub replace_audio_path: Option<PathBuf>,
    /// Optional PNG/SVG logo watermark overlay for videos
    #[serde(default)]
    pub watermark: Option<WatermarkSettings>,
    /// Optional audio fade-in duration in seconds
    #[serde(default)]
    pub fade_in_secs: Option<f32>,
    /// Optional audio fade-out duration in seconds
    #[serde(default)]
    pub fade_out_secs: Option<f32>,
    /// Auto-trim leading and trailing silence (-45 dB threshold)
    #[serde(default)]
    pub trim_silence: bool,
    /// Playback mode: normal, reverse, or boomerang loop
    #[serde(default)]
    pub playback_mode: VideoPlaybackMode,
    /// Social aspect-ratio reframing with softly blurred background (9:16, 4:5, 1:1, 16:9)
    #[serde(default)]
    pub aspect_reframe: VideoAspectReframe,
    /// Fast lossless container remux (-c:v copy -c:a copy) with zero re-encoding
    #[serde(default)]
    pub remux_only: bool,
    /// FFT broadband voice noise reduction (highpass + lowpass + afftdn)
    #[serde(default)]
    pub denoise_audio: bool,
    /// Deinterlace interlaced footage (yadif)
    #[serde(default)]
    pub deinterlace: bool,
    /// Subtle unsharp mask for crisper video
    #[serde(default)]
    pub video_sharpen: bool,
    /// Audio volume gain adjustment in dB (-20.0..=20.0, 0.0 = off)
    #[serde(default)]
    pub audio_gain_db: f32,
}

fn default_speed() -> f32 {
    1.0
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            target: VideoTarget::Video(VideoFormat::Mp4H264),
            quality: 75,
            use_hardware_accel: true,
            resize: ResizeMode::Original,
            never_upscale: true,
            gif_fps: 15,
            audio_bitrate_kbps: 192,
            strip_metadata: true,
            target_size_mb: None,
            trim_start_secs: None,
            trim_end_secs: None,
            mute_audio: false,
            max_fps: None,
            speed: 1.0,
            rotate: 0,
            flip_horizontal: false,
            normalize_loudness: false,
            sample_rate_hz: None,
            mono: false,
            subtitle_path: None,
            replace_audio_path: None,
            watermark: None,
            fade_in_secs: None,
            fade_out_secs: None,
            trim_silence: false,
            playback_mode: VideoPlaybackMode::Normal,
            aspect_reframe: VideoAspectReframe::None,
            remux_only: false,
            denoise_audio: false,
            deinterlace: false,
            video_sharpen: false,
            audio_gain_db: 0.0,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputSettings {
    /// None = same folder as source file
    pub output_dir: Option<PathBuf>,
    /// Optional suffix appended before extension
    pub suffix: String,
    /// Optional rename template supporting {name}, {index}, {index:03}, {width}, {height}, {format}
    #[serde(default)]
    pub rename_template: String,
    /// Mirror dropped folder hierarchy inside custom output_dir
    #[serde(default)]
    pub preserve_folder_structure: bool,
    /// Never overwrite by default
    pub overwrite: bool,
    /// Safely move original file to OS Trash after successful conversion
    #[serde(default)]
    pub trash_originals: bool,
    /// Bundle all converted batch outputs into a single ZIP archive
    #[serde(default)]
    pub bundle_zip: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionSettings {
    pub image: ImageSettings,
    pub video: VideoSettings,
    #[serde(default)]
    pub pdf: PdfSettings,
    pub output: OutputSettings,
    /// None = use all logical CPU cores
    pub max_cpu_threads: Option<usize>,
    /// None = adaptive concurrency based on CPU/GPU
    pub max_concurrent_videos: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Preset {
    WebOptimized,
    HighQuality,
    SmallSize,
}

impl Preset {
    pub fn to_settings(self) -> ConversionSettings {
        match self {
            Preset::WebOptimized => ConversionSettings {
                image: ImageSettings {
                    format: ImageFormat::Webp,
                    quality: 82,
                    lossless: false,
                    resize: ResizeMode::MaxDimensions {
                        width: Some(1920),
                        height: None,
                    },
                    never_upscale: true,
                    strip_metadata: true,
                    ..Default::default()
                },
                video: VideoSettings {
                    target: VideoTarget::Video(VideoFormat::Mp4H264),
                    quality: 75,
                    use_hardware_accel: true,
                    resize: ResizeMode::MaxDimensions {
                        width: Some(1920),
                        height: Some(1080),
                    },
                    never_upscale: true,
                    gif_fps: 15,
                    audio_bitrate_kbps: 192,
                    strip_metadata: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            Preset::HighQuality => ConversionSettings {
                image: ImageSettings {
                    format: ImageFormat::Png,
                    quality: 95,
                    lossless: true,
                    resize: ResizeMode::Original,
                    never_upscale: true,
                    strip_metadata: false,
                    ..Default::default()
                },
                video: VideoSettings {
                    target: VideoTarget::Video(VideoFormat::Mp4H265),
                    quality: 90,
                    use_hardware_accel: true,
                    resize: ResizeMode::Original,
                    never_upscale: true,
                    gif_fps: 20,
                    audio_bitrate_kbps: 320,
                    strip_metadata: false,
                    ..Default::default()
                },
                ..Default::default()
            },
            Preset::SmallSize => ConversionSettings {
                image: ImageSettings {
                    format: ImageFormat::Avif,
                    quality: 65,
                    lossless: false,
                    resize: ResizeMode::MaxDimensions {
                        width: Some(1280),
                        height: None,
                    },
                    never_upscale: true,
                    strip_metadata: true,
                    ..Default::default()
                },
                video: VideoSettings {
                    target: VideoTarget::Video(VideoFormat::Mp4H265),
                    quality: 55,
                    use_hardware_accel: true,
                    resize: ResizeMode::MaxDimensions {
                        width: Some(1280),
                        height: Some(720),
                    },
                    never_upscale: true,
                    gif_fps: 12,
                    audio_bitrate_kbps: 128,
                    strip_metadata: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}

/// Computes target `(width, height)` for an image/video given its source size and `ResizeMode`.
/// Returns `None` if no resizing is needed.
pub fn compute_target_dimensions(
    src_w: u32,
    src_h: u32,
    mode: &ResizeMode,
    never_upscale: bool,
) -> Option<(u32, u32)> {
    if src_w == 0 || src_h == 0 {
        return None;
    }

    let (mut dst_w, mut dst_h) = match *mode {
        ResizeMode::Original => return None,
        ResizeMode::Percentage { percent } => {
            let factor = (percent / 100.0).clamp(0.01, 10.0);
            let w = ((src_w as f32) * factor).round().max(1.0) as u32;
            let h = ((src_h as f32) * factor).round().max(1.0) as u32;
            (w, h)
        }
        ResizeMode::MaxDimensions { width, height } => match (width, height) {
            (None, None) => return None,
            (Some(max_w), None) => {
                if never_upscale && src_w <= max_w {
                    return None;
                }
                let ratio = max_w as f64 / src_w as f64;
                (max_w.max(1), ((src_h as f64) * ratio).round().max(1.0) as u32)
            }
            (None, Some(max_h)) => {
                if never_upscale && src_h <= max_h {
                    return None;
                }
                let ratio = max_h as f64 / src_h as f64;
                (((src_w as f64) * ratio).round().max(1.0) as u32, max_h.max(1))
            }
            (Some(max_w), Some(max_h)) => {
                let ratio_w = max_w as f64 / src_w as f64;
                let ratio_h = max_h as f64 / src_h as f64;
                let ratio = ratio_w.min(ratio_h);
                if never_upscale && ratio >= 1.0 {
                    return None;
                }
                (
                    ((src_w as f64) * ratio).round().max(1.0) as u32,
                    ((src_h as f64) * ratio).round().max(1.0) as u32,
                )
            }
        },
        ResizeMode::Exact {
            width,
            height,
            keep_aspect_ratio,
        } => {
            let target_w = width.max(1);
            let target_h = height.max(1);
            if keep_aspect_ratio {
                let ratio = (target_w as f64 / src_w as f64).min(target_h as f64 / src_h as f64);
                (
                    ((src_w as f64) * ratio).round().max(1.0) as u32,
                    ((src_h as f64) * ratio).round().max(1.0) as u32,
                )
            } else {
                (target_w, target_h)
            }
        }
    };

    if never_upscale && dst_w > src_w && dst_h > src_h {
        return None;
    }

    dst_w = dst_w.max(1);
    dst_h = dst_h.max(1);

    if dst_w == src_w && dst_h == src_h {
        None
    } else {
        Some((dst_w, dst_h))
    }
}

pub fn default_audio_target() -> AudioFormat {
    AudioFormat::Mp3
}
