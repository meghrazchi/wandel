//! Wandel Core — High-performance, cross-platform image and video conversion engine.

pub mod error;
pub mod formats;
pub mod image_engine;
pub mod inspect;
pub mod naming;
pub mod pdf_engine;
pub mod scheduler;
pub mod settings;
pub mod video_engine;

pub use error::{Result, WandelError};
pub use formats::{AudioFormat, FileMetadata, ImageFormat, MediaKind, VideoFormat, VideoTarget};
pub use image_engine::{
    create_comparison_image, create_contact_sheet, create_sprite_sheet, export_image_data_url,
    extract_color_palette, find_duplicate_images, generate_icon_pack, generate_image_preview,
    generate_qr_svg, generate_responsive_set, save_qr_code, split_image_grid, ColorSwatch,
    ComparisonOptions, DataUrlExport, DuplicateGroup, ImagePreview, ResponsiveSetResult,
    SpriteSheetResult,
};
pub use inspect::{inspect_detailed_metadata, inspect_paths, DetailedMetadata, MetadataEntry};
pub use pdf_engine::{
    compress_pdf, convert_pdf_to_images, extract_pdf_images, inspect_pdf, merge_images_to_pdf,
    merge_pdfs, number_pdf_pages, organize_pdf_pages, render_pdf_thumbnails, reorder_pdf_pages,
    rotate_pdf_pages, scrub_pdf_metadata, split_pdf, stamp_pdf_text, PdfPagePlan, PdfPageThumb,
};
pub use scheduler::{bundle_files_to_zip, run_batch, JobItem, ProgressEvent};
pub use settings::{
    AspectCrop, ContactSheetOptions, ConversionSettings, FrameGradient, ImageFilterPreset,
    ImageSettings, NormalizedCrop, OutputSettings, PdfPageFit, PdfSettings, Preset, ResizeMode,
    TextWatermarkSettings, VideoAspectReframe, VideoPlaybackMode, VideoSettings,
    WatermarkPosition, WatermarkSettings,
};
pub use video_engine::{
    create_audio_waveform, create_podcast_video, create_slideshow_video, create_video_storyboard,
    extract_video_frames, merge_audio, merge_videos, probe_hardware, split_media_segments,
    HardwareCapabilities,
};
