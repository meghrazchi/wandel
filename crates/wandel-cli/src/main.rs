use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use wandel_core::{
    compress_pdf, create_audio_waveform, create_comparison_image, create_contact_sheet,
    create_podcast_video, create_slideshow_video, create_sprite_sheet, create_video_storyboard,
    export_image_data_url, extract_color_palette, extract_pdf_images, extract_video_frames,
    find_duplicate_images, generate_icon_pack, generate_responsive_set, inspect_detailed_metadata,
    inspect_paths, merge_audio, merge_images_to_pdf, merge_pdfs, merge_videos, number_pdf_pages,
    probe_hardware, reorder_pdf_pages, rotate_pdf_pages, run_batch, save_qr_code,
    scrub_pdf_metadata, split_image_grid, split_media_segments, stamp_pdf_text, AspectCrop, AudioFormat, ComparisonOptions, ContactSheetOptions,
    ConversionSettings, ImageFilterPreset, ImageFormat, ImageSettings, JobItem, OutputSettings,
    PdfPageFit, Preset, ProgressEvent, ResizeMode, VideoAspectReframe, VideoFormat, VideoSettings,
    VideoTarget,
};

#[derive(Parser)]
#[command(
    name = "wandel",
    version,
    about = "Wandel: Fast, parallel image, video, audio & PDF converter"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
enum Commands {
    /// Convert single files, multiple files, or entire folders in parallel
    Convert {
        /// Input files or folders
        #[arg(required = true)]
        inputs: Vec<PathBuf>,

        /// Target format (e.g. webp, jpg, png, avif, mp4, h265, webm, mov, mkv, gif, mp3, wav, aac, flac)
        #[arg(short = 't', long)]
        to: Option<String>,

        /// Quick preset (web, high, small)
        #[arg(short = 'p', long)]
        preset: Option<String>,

        /// Quality 1-100
        #[arg(short = 'q', long)]
        quality: Option<u8>,

        /// Enable lossless compression (for PNG/WebP/JXL)
        #[arg(long)]
        lossless: bool,

        /// Maximum output width in pixels
        #[arg(long)]
        max_width: Option<u32>,

        /// Maximum output height in pixels
        #[arg(long)]
        max_height: Option<u32>,

        /// Scale by percentage (e.g. 50 for 50%)
        #[arg(long)]
        percent: Option<f32>,

        /// Output directory (defaults to same folder as each source file)
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Optional filename suffix (e.g. _min)
        #[arg(long, default_value = "")]
        suffix: String,

        /// Overwrite existing output files
        #[arg(long)]
        overwrite: bool,

        /// Keep EXIF / container metadata instead of stripping it
        #[arg(long)]
        keep_metadata: bool,

        /// Disable GPU hardware acceleration for video
        #[arg(long)]
        no_hw: bool,

        /// Limit CPU worker threads (defaults to all cores)
        #[arg(long)]
        threads: Option<usize>,

        /// Target max image file size in KB
        #[arg(long)]
        target_kb: Option<u32>,

        /// Target max video file size in MB
        #[arg(long)]
        target_mb: Option<f32>,

        /// Rotate images clockwise (90, 180, 270)
        #[arg(long, default_value_t = 0)]
        rotate: u16,

        /// Flip images horizontally
        #[arg(long)]
        flip_h: bool,

        /// Flip images vertically
        #[arg(long)]
        flip_v: bool,

        /// Center-crop images to aspect ratio (1:1, 16:9, 9:16, 4:3, 3:2)
        #[arg(long)]
        crop: Option<String>,

        /// Mute / remove audio track from videos
        #[arg(long)]
        mute: bool,

        /// Cap video frame rate (FPS)
        #[arg(long)]
        fps: Option<u8>,

        /// Trim video start time in seconds
        #[arg(long)]
        trim_start: Option<f64>,

        /// Trim video end time in seconds
        #[arg(long)]
        trim_end: Option<f64>,

        /// Move original files to OS Trash after successful conversion
        #[arg(long)]
        trash_originals: bool,

        /// Auto-trim excess transparent or solid-color image borders
        #[arg(long)]
        auto_trim: bool,

        /// Per-channel tolerance (0-64) for --auto-trim
        #[arg(long, default_value_t = 12)]
        trim_tolerance: u8,

        /// Social blur-pad aspect reframing for video (9:16, 4:5, 1:1, 16:9)
        #[arg(long)]
        video_reframe: Option<String>,

        /// One-click image color filter preset (sepia, warm, cool, vintage, bw, invert)
        #[arg(long)]
        filter: Option<String>,

        /// Instant lossless video container remux (-c copy, zero re-encoding)
        #[arg(long)]
        remux: bool,

        /// FFT broadband voice noise reduction for audio/video
        #[arg(long)]
        denoise_audio: bool,

        /// Normalize audio loudness to EBU R128 (-16 LUFS)
        #[arg(long)]
        normalize_audio: bool,

        /// Automatically trim leading and trailing silence on audio exports
        #[arg(long)]
        trim_silence: bool,

        /// Deinterlace interlaced video footage (yadif)
        #[arg(long)]
        deinterlace: bool,

        /// Apply subtle unsharp mask to video
        #[arg(long)]
        video_sharpen: bool,

        /// Audio volume gain in dB (-20.0..=20.0)
        #[arg(long, default_value_t = 0.0)]
        gain_db: f32,

        /// Playback speed multiplier for audio/video (0.5..=4.0)
        #[arg(long, default_value_t = 1.0)]
        speed: f32,

        /// Automatic 1%-99% luminance histogram contrast stretch for images
        #[arg(long)]
        auto_levels: bool,

        /// Privacy blur redaction rectangle as normalized x,y,w,h (0.0..=1.0, repeatable)
        #[arg(long = "blur-region")]
        blur_regions: Vec<String>,

        /// Remove solid background color ("auto" samples corners, or "#RRGGBB")
        #[arg(long = "remove-bg")]
        remove_bg: Option<String>,

        /// Color distance tolerance (0..=120) for --remove-bg
        #[arg(long = "remove-bg-tolerance", default_value_t = 24)]
        remove_bg_tolerance: u8,

        /// Additional comma-separated image formats to export simultaneously (e.g. avif,jpg)
        #[arg(long)]
        also: Option<String>,

        /// Optional bottom caption bar text rendered below images
        #[arg(long)]
        caption: Option<String>,
    },

    /// Display detected CPU cores, FFmpeg status, and GPU hardware encoders
    Info,

    /// Inspect EXIF, GPS, video/audio codec, or PDF metadata for a file
    Inspect {
        /// File to inspect
        file: PathBuf,
    },

    /// Merge multiple images into a single multi-page PDF
    PdfMerge {
        /// Ordered input image files
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
        /// Output PDF file path
        #[arg(short = 'o', long)]
        output: PathBuf,
        /// Page fit: fit, a4, letter
        #[arg(long, default_value = "fit")]
        fit: String,
    },

    /// Generate a complete Web & PWA Favicon / App Icon Pack from a logo
    IconPack {
        /// Source logo image (PNG/SVG/WebP)
        input: PathBuf,
        /// Output directory
        #[arg(short = 'o', long)]
        output_dir: PathBuf,
    },

    /// Stitch multiple images into a Contact Sheet / Grid collage
    Grid {
        /// Ordered input image files
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
        /// Output image file path (e.g. sheet.png or sheet.webp)
        #[arg(short = 'o', long)]
        output: PathBuf,
        /// Number of columns (0 = auto)
        #[arg(long, default_value_t = 0)]
        cols: u32,
        /// Cell width/height in pixels
        #[arg(long, default_value_t = 400)]
        cell_size: u32,
        /// Gap between cells in pixels
        #[arg(long, default_value_t = 16)]
        gap: u32,
    },

    /// Concatenate multiple videos into a single MP4
    VideoMerge {
        /// Ordered input video files
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
        /// Output MP4 file path
        #[arg(short = 'o', long)]
        output: PathBuf,
    },

    /// Merge multiple PDF files losslessly into a single PDF
    PdfConcat {
        /// Ordered input PDF files
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
        /// Output PDF file path
        #[arg(short = 'o', long)]
        output: PathBuf,
    },

    /// Find exact and visually near-duplicate images using perceptual dHash
    Dedupe {
        /// Input image files or folders
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
        /// Max Hamming distance (0 = exact perceptual match, default 5)
        #[arg(long, default_value_t = 5)]
        threshold: u32,
    },

    /// Compress / optimize a PDF file
    PdfCompress {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
        #[arg(long, default_value_t = 120)]
        dpi: u16,
        #[arg(short = 'q', long, default_value_t = 70)]
        quality: u8,
    },

    /// Extract dominant brand color palette from an image
    Palette {
        input: PathBuf,
        #[arg(short = 'n', long, default_value_t = 6)]
        count: usize,
    },

    /// Generate a QR code as SVG or PNG
    Qr {
        /// Text or URL to encode
        content: String,
        #[arg(short = 'o', long)]
        output: PathBuf,
        #[arg(long, default_value = "#000000")]
        dark: String,
        #[arg(long, default_value = "#ffffff")]
        light: String,
        #[arg(long, default_value_t = 512)]
        size: u32,
    },

    /// Concatenate multiple audio tracks into a single audio file
    AudioMerge {
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
        #[arg(short = 'o', long)]
        output: PathBuf,
    },

    /// Create a thumbnail storyboard grid from a video file
    Storyboard {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
        #[arg(long, default_value_t = 9)]
        frames: u32,
        #[arg(long, default_value_t = 3)]
        cols: u32,
    },

    /// Convert an image and print its Base64 Data URL to stdout
    DataUrl {
        input: PathBuf,
        #[arg(short = 't', long, default_value = "webp")]
        to: String,
        #[arg(short = 'q', long, default_value_t = 82)]
        quality: u8,
    },

    /// Generate a Responsive Web srcset image bundle (320w..1920w) + HTML snippet
    Srcset {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output_dir: PathBuf,
        #[arg(short = 't', long, default_value = "webp")]
        to: String,
        #[arg(short = 'q', long, default_value_t = 82)]
        quality: u8,
    },

    /// Losslessly rotate pages of a PDF clockwise (90, 180, or 270 degrees)
    PdfRotate {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
        #[arg(short = 'd', long, default_value_t = 90)]
        degrees: u16,
        /// Optional 1-based page range (e.g. "1,3-5"; omitted = all pages)
        #[arg(long)]
        pages: Option<String>,
    },

    /// Losslessly reorder or extract pages of a PDF (e.g. --pages "3,1-2")
    PdfReorder {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
        /// Ordered 1-based page specification (e.g. "3,1-2,5")
        #[arg(long)]
        pages: String,
    },

    /// Create a Before/After side-by-side or stacked comparison collage from two images
    Compare {
        first: PathBuf,
        second: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
        /// Stack top-to-bottom instead of side-by-side
        #[arg(long)]
        vertical: bool,
        #[arg(long, default_value = "Before")]
        left_label: String,
        #[arg(long, default_value = "After")]
        right_label: String,
    },

    /// Extract a frame sequence from a video at a given FPS
    Frames {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output_dir: PathBuf,
        #[arg(long, default_value_t = 1.0)]
        fps: f32,
        #[arg(short = 't', long, default_value = "png")]
        to: String,
    },

    /// Render an audio track's waveform as a high-resolution PNG graphic
    Waveform {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
        #[arg(long, default_value_t = 1200)]
        width: u32,
        #[arg(long, default_value_t = 320)]
        height: u32,
        #[arg(long, default_value = "#6366f1")]
        color: String,
        #[arg(long, default_value = "#18181b")]
        bg: String,
    },

    /// Turn multiple images into an MP4 slideshow video
    Slideshow {
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
        #[arg(short = 'o', long)]
        output: PathBuf,
        #[arg(long, default_value_t = 3.0)]
        seconds: f32,
        #[arg(long, default_value_t = 1920)]
        width: u32,
        #[arg(long, default_value_t = 1080)]
        height: u32,
        #[arg(long)]
        audio: Option<PathBuf>,
    },

    /// Losslessly stamp diagonal watermark text onto every page of a PDF
    PdfStamp {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
        #[arg(long)]
        text: String,
    },

    /// Slice an image into an equal cols x rows grid of tiles (e.g. 3x1 carousel)
    Slice {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output_dir: PathBuf,
        #[arg(long, default_value_t = 3)]
        cols: u32,
        #[arg(long, default_value_t = 1)]
        rows: u32,
        #[arg(short = 't', long, default_value = "png")]
        to: String,
    },

    /// Losslessly add centered page numbers to every page of a PDF
    PdfNumber {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
        #[arg(long, default_value = "{n} / {total}")]
        format: String,
    },

    /// Pack multiple images into a transparent sprite sheet + matching CSS coordinate map
    Sprite {
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
        #[arg(short = 'o', long)]
        output: PathBuf,
        #[arg(long, default_value_t = 4)]
        padding: u32,
    },

    /// Extract embedded raster images directly out of a PDF file (pure Rust)
    PdfImages {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output_dir: PathBuf,
    },

    /// Combine a cover image and an audio track into a shareable MP4 video
    Audiogram {
        input: PathBuf,
        #[arg(long)]
        cover: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
        #[arg(long, default_value_t = 1280)]
        width: u32,
        #[arg(long, default_value_t = 720)]
        height: u32,
    },

    /// Losslessly split a video or audio file into equal-duration chunks
    SplitMedia {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output_dir: PathBuf,
        #[arg(long, default_value_t = 60.0)]
        seconds: f32,
    },

    /// Losslessly strip identifying metadata (/Info & XMP) from a PDF file
    PdfClean {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
    },

    /// Extract plain text page-by-page from a PDF offline (prints to stdout or saves to -o)
    PdfText {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },

    /// Generate a 5-platform Social Media Kit (OG, YouTube, IG Square, IG Story, X Header)
    SocialKit {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output_dir: PathBuf,
        #[arg(short = 't', long, default_value = "jpg")]
        to: String,
    },

    /// Extract a single crisp poster/cover frame from a video at a timestamp
    Poster {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
        #[arg(long, default_value_t = 1.0)]
        at: f32,
    },

    /// Convert a light-background PDF into an eye-friendly dark-mode PDF
    PdfDark {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
        #[arg(long, default_value_t = 150)]
        dpi: u16,
    },

    /// Create a looping animated GIF from 2+ images in pure Rust (zero FFmpeg)
    MakeGif {
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
        #[arg(short = 'o', long)]
        output: PathBuf,
        #[arg(long, default_value_t = 300)]
        delay_ms: u32,
        #[arg(long)]
        max_width: Option<u32>,
    },

    /// Extract every frame of an animated GIF into numbered images in pure Rust
    GifFrames {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output_dir: PathBuf,
        #[arg(short = 't', long, default_value = "png")]
        to: String,
    },

    /// Export a PDF as clean RAG-ready Markdown (.md) with YAML frontmatter
    PdfMd {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },

    /// Impose pairs of PDF pages side-by-side (2-Up) onto landscape sheets
    Pdf2up {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
        #[arg(long, default_value_t = 150)]
        dpi: u16,
    },

    /// Stitch 2+ images vertically or horizontally at full resolution
    Stitch {
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
        #[arg(short = 'o', long)]
        output: PathBuf,
        #[arg(long, default_value = "vertical")]
        direction: String,
        #[arg(long, default_value_t = 0)]
        gap: u32,
        #[arg(long, default_value = "#ffffff")]
        bg: String,
    },

    /// Losslessly set PDF document metadata (/Title, /Author, /Subject, /Keywords)
    PdfMeta {
        input: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        author: Option<String>,
        #[arg(long)]
        subject: Option<String>,
        #[arg(long)]
        keywords: Option<String>,
    },

    /// Generate a tiny (< 1 KB) blurred inline Base64 WebP LQIP placeholder
    Lqip {
        input: PathBuf,
        #[arg(long, default_value_t = 24)]
        width: u32,
    },

    /// Start the headless Wandel MCP (Model Context Protocol) server over stdio or local HTTP
    Mcp {
        /// Optional sandbox root directory (repeatable; restricts all MCP file access)
        #[arg(long = "allowed-dir")]
        allowed_dirs: Vec<PathBuf>,

        /// Optional local HTTP bind address (e.g. 127.0.0.1:3847). If omitted, runs over stdio.
        #[arg(long)]
        http: Option<String>,

        /// Optional Bearer token required for Authorization header when --http is enabled
        #[arg(long)]
        token: Option<String>,
    },

    /// Execute a multi-step composable JSON media pipeline recipe headlessly
    Pipeline {
        /// Path to JSON recipe file containing { "steps": [...] }
        recipe: PathBuf,
    },
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    let exe_ext = if cfg!(windows) { ".exe" } else { "" };
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(format!("{}{}", name, exe_ext));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
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

fn main() {
    // Windows defaults to a 1 MB main-thread stack, which is too small for the
    // debug-mode clap parser + 25-subcommand dispatch frame. Run on an 8 MB stack.
    let handle = std::thread::Builder::new()
        .name("wandel-main".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(run)
        .expect("Failed to spawn main worker thread");
    if let Err(err) = handle.join() {
        std::panic::resume_unwind(err);
    }
}

fn run() {
    let cli = Cli::parse();
    let ffmpeg = find_in_path("ffmpeg");
    let ffprobe = find_in_path("ffprobe");

    match cli.command {
        Commands::Info => {
            let hw = probe_hardware(ffmpeg.as_deref(), ffprobe.as_deref());
            println!("Wandel CLI v{}", env!("CARGO_PKG_VERSION"));
            println!("  CPU Cores:       {}", hw.cpu_cores);
            println!(
                "  FFmpeg:          {}",
                if hw.ffmpeg_available {
                    ffmpeg
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "available".into())
                } else {
                    "not found".into()
                }
            );
            println!(
                "  GPU Backend:     {}",
                hw.gpu_backend_name.as_deref().unwrap_or("CPU only")
            );
            println!(
                "  H.264 HW:        {}",
                hw.h264_hw_encoder.as_deref().unwrap_or("software")
            );
            println!(
                "  H.265 HW:        {}",
                hw.h265_hw_encoder.as_deref().unwrap_or("software")
            );
        }
        Commands::Inspect { file } => {
            match inspect_detailed_metadata(&file, ffprobe.as_deref()) {
                Some(meta) => {
                    println!("{} ({} bytes)", meta.file_name, meta.size_bytes);
                    if meta.has_gps_warning {
                        println!("  [WARN] GPS / location metadata detected!");
                    }
                    for entry in meta.entries {
                        println!("  {:<18} {}", format!("{}:", entry.label), entry.value);
                    }
                }
                None => {
                    eprintln!("Unsupported or unreadable file: {}", file.display());
                    std::process::exit(1);
                }
            }
        }
        Commands::PdfMerge {
            inputs,
            output,
            fit,
        } => {
            let page_fit = match fit.to_ascii_lowercase().as_str() {
                "a4" => PdfPageFit::A4,
                "letter" => PdfPageFit::Letter,
                _ => PdfPageFit::FitToImage,
            };
            match merge_images_to_pdf(
                &inputs,
                &output,
                &ImageSettings::default(),
                page_fit,
                ffmpeg.as_deref(),
            ) {
                Ok(bytes) => println!(
                    "Merged {} image(s) -> {} ({:.1} KB)",
                    inputs.len(),
                    output.display(),
                    bytes as f64 / 1024.0
                ),
                Err(e) => {
                    eprintln!("PDF merge failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::IconPack { input, output_dir } => {
            match generate_icon_pack(&input, &output_dir, ffmpeg.as_deref()) {
                Ok(files) => println!(
                    "Generated {} icon pack files in {}",
                    files.len(),
                    output_dir.display()
                ),
                Err(e) => {
                    eprintln!("Icon pack failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Grid {
            inputs,
            output,
            cols,
            cell_size,
            gap,
        } => {
            let ext = output
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png");
            let fmt = ImageFormat::from_extension(ext).unwrap_or(ImageFormat::Png);
            let opts = ContactSheetOptions {
                columns: cols,
                cell_size,
                gap_px: gap,
                ..Default::default()
            };
            let img_settings = ImageSettings {
                format: fmt,
                ..Default::default()
            };
            match create_contact_sheet(&inputs, &output, &opts, &img_settings, ffmpeg.as_deref()) {
                Ok(bytes) => println!(
                    "Created contact sheet ({} tiles) -> {} ({:.1} KB)",
                    inputs.len(),
                    output.display(),
                    bytes as f64 / 1024.0
                ),
                Err(e) => {
                    eprintln!("Grid failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::VideoMerge { inputs, output } => {
            let Some(ffmpeg_bin) = ffmpeg.as_deref() else {
                eprintln!("FFmpeg not found in PATH");
                std::process::exit(1);
            };
            let hw = probe_hardware(Some(ffmpeg_bin), ffprobe.as_deref());
            match merge_videos(&inputs, &output, &VideoSettings::default(), &hw, ffmpeg_bin) {
                Ok(bytes) => println!(
                    "Merged {} video(s) -> {} ({:.1} KB)",
                    inputs.len(),
                    output.display(),
                    bytes as f64 / 1024.0
                ),
                Err(e) => {
                    eprintln!("Video merge failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::PdfConcat { inputs, output } => match merge_pdfs(&inputs, &output) {
            Ok(bytes) => println!(
                "Merged {} PDF(s) -> {} ({:.1} KB)",
                inputs.len(),
                output.display(),
                bytes as f64 / 1024.0
            ),
            Err(e) => {
                eprintln!("PDF concat failed: {}", e);
                std::process::exit(1);
            }
        },
        Commands::Dedupe { inputs, threshold } => {
            let inspected = inspect_paths(&inputs, ffmpeg.as_deref(), ffprobe.as_deref());
            let img_paths: Vec<PathBuf> = inspected
                .into_iter()
                .filter(|m| m.media_kind == wandel_core::MediaKind::Image)
                .map(|m| PathBuf::from(m.path))
                .collect();
            let groups = find_duplicate_images(&img_paths, threshold, ffmpeg.as_deref());
            if groups.is_empty() {
                println!("No duplicates found across {} image(s).", img_paths.len());
            } else {
                println!("Found {} duplicate group(s):", groups.len());
                for (i, g) in groups.iter().enumerate() {
                    println!("  Group {} | Keep: {}", i + 1, g.keeper);
                    for d in &g.duplicates {
                        println!("    Duplicate: {}", d);
                    }
                }
            }
        }
        Commands::PdfCompress {
            input,
            output,
            dpi,
            quality,
        } => match compress_pdf(&input, &output, dpi, quality) {
            Ok(bytes) => println!(
                "Compressed PDF -> {} ({:.1} KB)",
                output.display(),
                bytes as f64 / 1024.0
            ),
            Err(e) => {
                eprintln!("PDF compress failed: {}", e);
                std::process::exit(1);
            }
        },
        Commands::Palette { input, count } => {
            match extract_color_palette(&input, count, ffmpeg.as_deref()) {
                Ok(swatches) => {
                    for s in swatches {
                        println!(
                            "  {}  rgb({:3}, {:3}, {:3})  ({:.1}%)",
                            s.hex, s.r, s.g, s.b, s.percent
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Palette extraction failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Qr {
            content,
            output,
            dark,
            light,
            size,
        } => match save_qr_code(&content, &dark, &light, size, &output) {
            Ok(bytes) => println!(
                "Generated QR code -> {} ({:.1} KB)",
                output.display(),
                bytes as f64 / 1024.0
            ),
            Err(e) => {
                eprintln!("QR generation failed: {}", e);
                std::process::exit(1);
            }
        },
        Commands::AudioMerge { inputs, output } => {
            let Some(ffmpeg_bin) = ffmpeg.as_deref() else {
                eprintln!("FFmpeg not found in PATH");
                std::process::exit(1);
            };
            match merge_audio(&inputs, &output, &VideoSettings::default(), ffmpeg_bin) {
                Ok(bytes) => println!(
                    "Merged {} audio file(s) -> {} ({:.1} KB)",
                    inputs.len(),
                    output.display(),
                    bytes as f64 / 1024.0
                ),
                Err(e) => {
                    eprintln!("Audio merge failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Storyboard {
            input,
            output,
            frames,
            cols,
        } => {
            let Some(ffmpeg_bin) = ffmpeg.as_deref() else {
                eprintln!("FFmpeg not found in PATH");
                std::process::exit(1);
            };
            match create_video_storyboard(
                &input,
                &output,
                frames,
                cols,
                ffmpeg_bin,
                ffprobe.as_deref(),
            ) {
                Ok(bytes) => println!(
                    "Created storyboard ({} frames) -> {} ({:.1} KB)",
                    frames,
                    output.display(),
                    bytes as f64 / 1024.0
                ),
                Err(e) => {
                    eprintln!("Storyboard failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::DataUrl { input, to, quality } => {
            let fmt = ImageFormat::from_extension(&to).unwrap_or(ImageFormat::Webp);
            let settings = ImageSettings {
                format: fmt,
                quality,
                ..Default::default()
            };
            match export_image_data_url(&input, &settings, ffmpeg.as_deref()) {
                Ok(exp) => println!("{}", exp.data_url),
                Err(e) => {
                    eprintln!("Data URL export failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Srcset {
            input,
            output_dir,
            to,
            quality,
        } => {
            let fmt = ImageFormat::from_extension(&to).unwrap_or(ImageFormat::Webp);
            let settings = ImageSettings {
                format: fmt,
                quality,
                ..Default::default()
            };
            match generate_responsive_set(&input, &output_dir, &settings, ffmpeg.as_deref()) {
                Ok(res) => {
                    println!(
                        "Generated {} responsive files in {}\n\n{}",
                        res.files.len(),
                        output_dir.display(),
                        res.html_snippet
                    );
                }
                Err(e) => {
                    eprintln!("Srcset generation failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::PdfRotate {
            input,
            output,
            degrees,
            pages,
        } => match rotate_pdf_pages(&input, &output, degrees, pages.as_deref()) {
            Ok(bytes) => println!(
                "Rotated PDF ({}°) -> {} ({:.1} KB)",
                degrees % 360,
                output.display(),
                bytes as f64 / 1024.0
            ),
            Err(e) => {
                eprintln!("PDF rotate failed: {}", e);
                std::process::exit(1);
            }
        },
        Commands::PdfReorder {
            input,
            output,
            pages,
        } => match reorder_pdf_pages(&input, &output, &pages) {
            Ok(bytes) => println!(
                "Reordered PDF (pages {}) -> {} ({:.1} KB)",
                pages,
                output.display(),
                bytes as f64 / 1024.0
            ),
            Err(e) => {
                eprintln!("PDF reorder failed: {}", e);
                std::process::exit(1);
            }
        },
        Commands::Compare {
            first,
            second,
            output,
            vertical,
            left_label,
            right_label,
        } => {
            let ext = output
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png");
            let fmt = ImageFormat::from_extension(ext).unwrap_or(ImageFormat::Png);
            let opts = ComparisonOptions {
                vertical,
                left_label: if left_label.is_empty() {
                    None
                } else {
                    Some(left_label)
                },
                right_label: if right_label.is_empty() {
                    None
                } else {
                    Some(right_label)
                },
                ..Default::default()
            };
            let img_settings = ImageSettings {
                format: fmt,
                ..Default::default()
            };
            match create_comparison_image(
                &first,
                &second,
                &output,
                &opts,
                &img_settings,
                ffmpeg.as_deref(),
            ) {
                Ok(bytes) => println!(
                    "Created comparison collage -> {} ({:.1} KB)",
                    output.display(),
                    bytes as f64 / 1024.0
                ),
                Err(e) => {
                    eprintln!("Compare failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Frames {
            input,
            output_dir,
            fps,
            to,
        } => {
            let Some(ffmpeg_bin) = ffmpeg.as_deref() else {
                eprintln!("FFmpeg not found in PATH");
                std::process::exit(1);
            };
            match extract_video_frames(&input, &output_dir, fps, &to, ffmpeg_bin) {
                Ok(files) => println!(
                    "Extracted {} frame(s) at {:.2} fps -> {}",
                    files.len(),
                    fps,
                    output_dir.display()
                ),
                Err(e) => {
                    eprintln!("Frame extraction failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Waveform {
            input,
            output,
            width,
            height,
            color,
            bg,
        } => {
            let Some(ffmpeg_bin) = ffmpeg.as_deref() else {
                eprintln!("FFmpeg not found in PATH");
                std::process::exit(1);
            };
            match create_audio_waveform(&input, &output, width, height, &color, &bg, ffmpeg_bin) {
                Ok(bytes) => println!(
                    "Generated audio waveform -> {} ({:.1} KB)",
                    output.display(),
                    bytes as f64 / 1024.0
                ),
                Err(e) => {
                    eprintln!("Waveform generation failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Slideshow {
            inputs,
            output,
            seconds,
            width,
            height,
            audio,
        } => {
            let Some(ffmpeg_bin) = ffmpeg.as_deref() else {
                eprintln!("FFmpeg not found in PATH");
                std::process::exit(1);
            };
            match create_slideshow_video(
                &inputs,
                &output,
                seconds,
                width,
                height,
                audio.as_deref(),
                ffmpeg_bin,
            ) {
                Ok(bytes) => println!(
                    "Created slideshow ({} slides) -> {} ({:.1} KB)",
                    inputs.len(),
                    output.display(),
                    bytes as f64 / 1024.0
                ),
                Err(e) => {
                    eprintln!("Slideshow failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::PdfStamp {
            input,
            output,
            text,
        } => match stamp_pdf_text(&input, &output, &text) {
            Ok(bytes) => println!(
                "Stamped PDF ('{}') -> {} ({:.1} KB)",
                text,
                output.display(),
                bytes as f64 / 1024.0
            ),
            Err(e) => {
                eprintln!("PDF stamp failed: {}", e);
                std::process::exit(1);
            }
        },
        Commands::Slice {
            input,
            output_dir,
            cols,
            rows,
            to,
        } => {
            let fmt = ImageFormat::from_extension(&to).unwrap_or(ImageFormat::Png);
            let img_settings = ImageSettings {
                format: fmt,
                ..Default::default()
            };
            match split_image_grid(&input, &output_dir, cols, rows, &img_settings, ffmpeg.as_deref())
            {
                Ok(files) => println!(
                    "Sliced image into {} tile(s) ({}x{}) -> {}",
                    files.len(),
                    cols,
                    rows,
                    output_dir.display()
                ),
                Err(e) => {
                    eprintln!("Image slice failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::PdfNumber {
            input,
            output,
            format,
        } => match number_pdf_pages(&input, &output, &format) {
            Ok(bytes) => println!(
                "Numbered PDF -> {} ({:.1} KB)",
                output.display(),
                bytes as f64 / 1024.0
            ),
            Err(e) => {
                eprintln!("PDF numbering failed: {}", e);
                std::process::exit(1);
            }
        },
        Commands::Sprite {
            inputs,
            output,
            padding,
        } => {
            let ext = output
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png");
            let fmt = ImageFormat::from_extension(ext).unwrap_or(ImageFormat::Png);
            let img_settings = ImageSettings {
                format: fmt,
                ..Default::default()
            };
            match create_sprite_sheet(&inputs, &output, padding, &img_settings, ffmpeg.as_deref()) {
                Ok(res) => println!(
                    "Packed {} sprites ({}x{}) -> {} + {}",
                    res.count, res.width, res.height, res.image_path, res.css_path
                ),
                Err(e) => {
                    eprintln!("Sprite sheet failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::PdfImages { input, output_dir } => {
            match extract_pdf_images(&input, &output_dir) {
                Ok(files) => println!(
                    "Extracted {} embedded image(s) -> {}",
                    files.len(),
                    output_dir.display()
                ),
                Err(e) => {
                    eprintln!("PDF image extraction failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Audiogram {
            input,
            cover,
            output,
            width,
            height,
        } => {
            let Some(ffmpeg_bin) = ffmpeg.as_deref() else {
                eprintln!("FFmpeg not found in PATH");
                std::process::exit(1);
            };
            match create_podcast_video(&input, &cover, &output, width, height, ffmpeg_bin) {
                Ok(bytes) => println!(
                    "Created audiogram MP4 -> {} ({:.1} KB)",
                    output.display(),
                    bytes as f64 / 1024.0
                ),
                Err(e) => {
                    eprintln!("Audiogram failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::SplitMedia {
            input,
            output_dir,
            seconds,
        } => {
            let Some(ffmpeg_bin) = ffmpeg.as_deref() else {
                eprintln!("FFmpeg not found in PATH");
                std::process::exit(1);
            };
            match split_media_segments(&input, &output_dir, seconds, ffmpeg_bin) {
                Ok(parts) => println!(
                    "Split into {} segment(s) ({:.1}s each) -> {}",
                    parts.len(),
                    seconds,
                    output_dir.display()
                ),
                Err(e) => {
                    eprintln!("Media split failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::PdfClean { input, output } => match scrub_pdf_metadata(&input, &output) {
            Ok(bytes) => println!(
                "Scrubbed PDF metadata -> {} ({:.1} KB)",
                output.display(),
                bytes as f64 / 1024.0
            ),
            Err(e) => {
                eprintln!("PDF clean failed: {}", e);
                std::process::exit(1);
            }
        },
        Commands::PdfText { input, output } => {
            if let Some(out) = output {
                match wandel_core::export_pdf_text_file(&input, &out) {
                    Ok((pages, bytes)) => println!(
                        "Extracted text from {} page(s) -> {} ({:.1} KB)",
                        pages,
                        out.display(),
                        bytes as f64 / 1024.0
                    ),
                    Err(e) => {
                        eprintln!("PDF text extraction failed: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                match wandel_core::extract_pdf_text(&input) {
                    Ok(pages) => {
                        for (num, text) in pages {
                            println!("--- Page {} ---\n{}\n", num, text);
                        }
                    }
                    Err(e) => {
                        eprintln!("PDF text extraction failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::SocialKit {
            input,
            output_dir,
            to,
        } => {
            let fmt = ImageFormat::from_extension(&to.to_ascii_lowercase())
                .filter(|f| f.can_encode())
                .unwrap_or(ImageFormat::Jpg);
            match wandel_core::generate_social_kit(&input, &output_dir, fmt, ffmpeg.as_deref()) {
                Ok(files) => println!(
                    "Generated {} social media assets in {}",
                    files.len(),
                    output_dir.display()
                ),
                Err(e) => {
                    eprintln!("Social kit generation failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Poster { input, output, at } => {
            let Some(ffmpeg_bin) = ffmpeg.as_deref() else {
                eprintln!("FFmpeg not found in PATH");
                std::process::exit(1);
            };
            match wandel_core::extract_video_poster(&input, &output, at, ffmpeg_bin) {
                Ok(bytes) => println!(
                    "Extracted video poster at {:.2}s -> {} ({:.1} KB)",
                    at,
                    output.display(),
                    bytes as f64 / 1024.0
                ),
                Err(e) => {
                    eprintln!("Poster extraction failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::PdfDark { input, output, dpi } => {
            match wandel_core::convert_pdf_dark_mode(&input, &output, dpi) {
                Ok(bytes) => println!(
                    "Converted PDF to dark mode ({} DPI) -> {} ({:.1} KB)",
                    dpi,
                    output.display(),
                    bytes as f64 / 1024.0
                ),
                Err(e) => {
                    eprintln!("Dark-mode PDF conversion failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::MakeGif {
            inputs,
            output,
            delay_ms,
            max_width,
        } => {
            match wandel_core::create_animated_gif(
                &inputs,
                &output,
                delay_ms,
                max_width,
                ffmpeg.as_deref(),
            ) {
                Ok(bytes) => println!(
                    "Created animated GIF ({} frames, {}ms) -> {} ({:.1} KB)",
                    inputs.len(),
                    delay_ms,
                    output.display(),
                    bytes as f64 / 1024.0
                ),
                Err(e) => {
                    eprintln!("Animated GIF creation failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::GifFrames {
            input,
            output_dir,
            to,
        } => {
            let fmt = ImageFormat::from_extension(&to.to_ascii_lowercase())
                .filter(|f| f.can_encode())
                .unwrap_or(ImageFormat::Png);
            match wandel_core::extract_gif_frames(&input, &output_dir, fmt) {
                Ok(files) => println!(
                    "Extracted {} GIF frame(s) -> {}",
                    files.len(),
                    output_dir.display()
                ),
                Err(e) => {
                    eprintln!("GIF frame extraction failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::PdfMd { input, output } => {
            if let Some(out) = output {
                match wandel_core::export_pdf_markdown_file(&input, &out) {
                    Ok((pages, bytes)) => println!(
                        "Exported {} PDF page(s) to Markdown -> {} ({:.1} KB)",
                        pages,
                        out.display(),
                        bytes as f64 / 1024.0
                    ),
                    Err(e) => {
                        eprintln!("PDF Markdown export failed: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                match wandel_core::extract_pdf_text(&input) {
                    Ok(pages) => {
                        let title = input
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Document");
                        println!("# {}\n", title);
                        for (num, text) in pages {
                            println!("## Page {}\n\n{}\n", num, text);
                        }
                    }
                    Err(e) => {
                        eprintln!("PDF Markdown export failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Pdf2up { input, output, dpi } => {
            match wandel_core::convert_pdf_two_up(&input, &output, dpi) {
                Ok((sheets, bytes)) => println!(
                    "Created 2-Up PDF ({} sheets, {} DPI) -> {} ({:.1} KB)",
                    sheets,
                    dpi,
                    output.display(),
                    bytes as f64 / 1024.0
                ),
                Err(e) => {
                    eprintln!("2-Up PDF imposition failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Stitch {
            inputs,
            output,
            direction,
            gap,
            bg,
        } => {
            let vertical = !direction.eq_ignore_ascii_case("horizontal");
            let ext = output
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png")
                .to_ascii_lowercase();
            let fmt = ImageFormat::from_extension(&ext)
                .filter(|f| f.can_encode())
                .unwrap_or(ImageFormat::Png);
            let settings = ImageSettings {
                format: fmt,
                quality: 88,
                ..Default::default()
            };
            match wandel_core::stitch_images(
                &inputs,
                &output,
                vertical,
                gap,
                &bg,
                &settings,
                ffmpeg.as_deref(),
            ) {
                Ok(bytes) => println!(
                    "Stitched {} images ({}) -> {} ({:.1} KB)",
                    inputs.len(),
                    if vertical { "vertical" } else { "horizontal" },
                    output.display(),
                    bytes as f64 / 1024.0
                ),
                Err(e) => {
                    eprintln!("Image stitch failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::PdfMeta {
            input,
            output,
            title,
            author,
            subject,
            keywords,
        } => match wandel_core::set_pdf_metadata(
            &input,
            &output,
            title.as_deref(),
            author.as_deref(),
            subject.as_deref(),
            keywords.as_deref(),
        ) {
            Ok(bytes) => println!(
                "Updated PDF metadata -> {} ({:.1} KB)",
                output.display(),
                bytes as f64 / 1024.0
            ),
            Err(e) => {
                eprintln!("PDF metadata update failed: {}", e);
                std::process::exit(1);
            }
        },
        Commands::Lqip { input, width } => {
            match wandel_core::generate_lqip(&input, width, ffmpeg.as_deref()) {
                Ok(exp) => {
                    println!("{}", exp.data_url);
                    eprintln!(
                        "LQIP generated ({}x{}, {} bytes)",
                        exp.width, exp.height, exp.bytes
                    );
                }
                Err(e) => {
                    eprintln!("LQIP generation failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Mcp {
            allowed_dirs,
            http,
            token,
        } => {
            let server = std::sync::Arc::new(wandel_mcp::McpServer::with_allowed_dirs(
                ffmpeg,
                ffprobe,
                allowed_dirs,
            ));
            let res = if let Some(addr) = http {
                wandel_mcp::serve_http(server, &addr, token)
            } else {
                server.run_stdio()
            };
            if let Err(e) = res {
                eprintln!("Wandel MCP server error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Pipeline { recipe } => {
            let raw = match std::fs::read_to_string(&recipe) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to read pipeline recipe {}: {}", recipe.display(), e);
                    std::process::exit(1);
                }
            };
            let json_val: serde_json::Value = match serde_json::from_str(&raw) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Invalid JSON in {}: {}", recipe.display(), e);
                    std::process::exit(1);
                }
            };
            let server = wandel_mcp::McpServer::new(ffmpeg, ffprobe);
            let (payload, is_err) = server.run_pipeline_recipe(&json_val);
            println!(
                "{}",
                serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string())
            );
            if is_err {
                std::process::exit(1);
            }
        }
        Commands::Convert {
            inputs,
            to,
            preset,
            quality,
            lossless,
            max_width,
            max_height,
            percent,
            output,
            suffix,
            overwrite,
            keep_metadata,
            no_hw,
            threads,
            target_kb,
            target_mb,
            rotate,
            flip_h,
            flip_v,
            crop,
            mute,
            fps,
            trim_start,
            trim_end,
            trash_originals,
            auto_trim,
            trim_tolerance,
            video_reframe,
            filter,
            remux,
            denoise_audio,
            normalize_audio,
            trim_silence,
            deinterlace,
            video_sharpen,
            gain_db,
            speed,
            auto_levels,
            blur_regions,
            remove_bg,
            remove_bg_tolerance,
            also,
            caption,
        } => {
            let mut settings = match preset.as_deref() {
                Some("web" | "web_optimized") => Preset::WebOptimized.to_settings(),
                Some("high" | "high_quality") => Preset::HighQuality.to_settings(),
                Some("small" | "small_size") => Preset::SmallSize.to_settings(),
                Some(other) => {
                    eprintln!("Unknown preset '{}'. Use: web, high, or small.", other);
                    std::process::exit(1);
                }
                None => ConversionSettings::default(),
            };

            if let Some(target) = to {
                if let Err(err) = apply_target_format(&mut settings, &target) {
                    eprintln!("Error: {}", err);
                    std::process::exit(1);
                }
            }

            if let Some(q) = quality {
                settings.image.quality = q.clamp(1, 100);
                settings.video.quality = q.clamp(1, 100);
            }
            if lossless {
                settings.image.lossless = true;
            }
            if let Some(pct) = percent {
                let mode = ResizeMode::Percentage { percent: pct };
                settings.image.resize = mode.clone();
                settings.video.resize = mode;
            } else if max_width.is_some() || max_height.is_some() {
                let mode = ResizeMode::MaxDimensions {
                    width: max_width,
                    height: max_height,
                };
                settings.image.resize = mode.clone();
                settings.video.resize = mode;
            }

            settings.image.strip_metadata = !keep_metadata;
            settings.image.target_size_kb = target_kb;
            settings.image.rotate = rotate;
            settings.image.flip_horizontal = flip_h;
            settings.image.flip_vertical = flip_v;
            settings.image.auto_trim_border = auto_trim;
            settings.image.auto_trim_tolerance = trim_tolerance.min(64);
            settings.image.auto_levels = auto_levels;
            settings.image.remove_bg_color = remove_bg;
            settings.image.remove_bg_tolerance = remove_bg_tolerance;
            settings.image.caption_text = caption;
            if let Some(also_str) = also {
                for tok in also_str.split(',') {
                    let clean = tok.trim().to_ascii_lowercase();
                    if let Some(fmt) = ImageFormat::from_extension(&clean) {
                        if fmt.can_encode() && !settings.image.extra_formats.contains(&fmt) {
                            settings.image.extra_formats.push(fmt);
                        }
                    }
                }
            }
            for spec in blur_regions {
                let parts: Vec<f32> = spec
                    .split(',')
                    .filter_map(|s| s.trim().parse::<f32>().ok())
                    .collect();
                if parts.len() == 4 {
                    settings.image.blur_regions.push(wandel_core::NormalizedCrop {
                        x: parts[0],
                        y: parts[1],
                        width: parts[2],
                        height: parts[3],
                    });
                } else {
                    eprintln!("Invalid --blur-region '{}'. Expected x,y,w,h (0.0..=1.0).", spec);
                    std::process::exit(1);
                }
            }
            if let Some(c) = crop {
                settings.image.crop_aspect = match c.as_str() {
                    "1:1" | "square" => Some(AspectCrop::Square1x1),
                    "16:9" => Some(AspectCrop::Landscape16x9),
                    "9:16" => Some(AspectCrop::Portrait9x16),
                    "4:3" => Some(AspectCrop::Standard4x3),
                    "3:2" => Some(AspectCrop::Photo3x2),
                    other => {
                        eprintln!("Unknown crop ratio '{}'. Use 1:1, 16:9, 9:16, 4:3, or 3:2.", other);
                        std::process::exit(1);
                    }
                };
            }
            if let Some(rf) = video_reframe {
                settings.video.aspect_reframe = match rf.as_str() {
                    "9:16" => VideoAspectReframe::Portrait9x16,
                    "4:5" => VideoAspectReframe::Portrait4x5,
                    "1:1" | "square" => VideoAspectReframe::Square1x1,
                    "16:9" => VideoAspectReframe::Landscape16x9,
                    other => {
                        eprintln!("Unknown video reframe '{}'. Use 9:16, 4:5, 1:1, or 16:9.", other);
                        std::process::exit(1);
                    }
                };
            }
            if let Some(flt) = filter {
                settings.image.filter_preset = match flt.to_ascii_lowercase().as_str() {
                    "sepia" => ImageFilterPreset::Sepia,
                    "warm" => ImageFilterPreset::Warm,
                    "cool" => ImageFilterPreset::Cool,
                    "vintage" => ImageFilterPreset::Vintage,
                    "bw" | "high_contrast_bw" => ImageFilterPreset::HighContrastBw,
                    "invert" => ImageFilterPreset::Invert,
                    other => {
                        eprintln!(
                            "Unknown filter '{}'. Use sepia, warm, cool, vintage, bw, or invert.",
                            other
                        );
                        std::process::exit(1);
                    }
                };
            }
            settings.video.remux_only = remux;
            settings.video.denoise_audio = denoise_audio;
            settings.video.normalize_loudness = normalize_audio;
            settings.video.trim_silence = trim_silence;
            settings.video.deinterlace = deinterlace;
            settings.video.video_sharpen = video_sharpen;
            settings.video.audio_gain_db = gain_db;
            settings.video.speed = speed;
            settings.video.strip_metadata = !keep_metadata;
            settings.video.use_hardware_accel = !no_hw;
            settings.video.target_size_mb = target_mb;
            settings.video.mute_audio = mute;
            settings.video.max_fps = fps;
            settings.video.trim_start_secs = trim_start;
            settings.video.trim_end_secs = trim_end;
            settings.max_cpu_threads = threads;
            settings.output = OutputSettings {
                output_dir: output,
                suffix,
                overwrite,
                trash_originals,
                ..Default::default()
            };

            let inspected = inspect_paths(&inputs, ffmpeg.as_deref(), ffprobe.as_deref());
            if inspected.is_empty() {
                eprintln!("No supported media files found in the given paths.");
                std::process::exit(1);
            }

            println!("Wandeling {} file(s)...", inspected.len());

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

            let hw = probe_hardware(ffmpeg.as_deref(), ffprobe.as_deref());
            let cancel = Arc::new(AtomicBool::new(false));

            run_batch(
                jobs,
                &settings,
                &hw,
                ffmpeg.as_deref(),
                cancel,
                |ev| match ev {
                    ProgressEvent::FileCompleted {
                        output_path,
                        output_bytes,
                        duration_ms,
                        ..
                    } => {
                        println!(
                            "  [OK] {} ({:.1} KB, {:.2}s)",
                            output_path,
                            output_bytes as f64 / 1024.0,
                            duration_ms as f64 / 1000.0
                        );
                    }
                    ProgressEvent::FileFailed { id, error } => {
                        eprintln!("  [ERR] {}: {}", id, error);
                    }
                    ProgressEvent::BatchFinished {
                        completed,
                        failed,
                        cancelled,
                        total_duration_ms,
                    } => {
                        println!(
                            "Done: {} converted, {} failed, {} cancelled in {:.2}s",
                            completed,
                            failed,
                            cancelled,
                            total_duration_ms as f64 / 1000.0
                        );
                        if failed > 0 {
                            std::process::exit(1);
                        }
                    }
                    _ => {}
                },
            );
        }
    }
}
