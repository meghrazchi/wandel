//! Wandel Core: High-performance, cross-platform image and video conversion engine.

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
    create_animated_gif, create_comparison_image, create_contact_sheet, create_pixel_diff,
    create_sprite_sheet, export_image_data_url, extract_color_palette, extract_gif_frames,
    find_duplicate_images, generate_icon_pack, generate_image_preview, generate_lqip,
    generate_qr_svg, generate_responsive_set, generate_social_kit, save_qr_code, split_image_grid,
    stitch_images, ColorSwatch, ComparisonOptions, DataUrlExport, DuplicateGroup, ImagePreview,
    PixelDiffResult, ResponsiveSetResult, SpriteSheetResult,
};
pub use inspect::{inspect_detailed_metadata, inspect_paths, DetailedMetadata, MetadataEntry};
pub use pdf_engine::{
    compress_pdf, convert_pdf_dark_mode, convert_pdf_to_images, convert_pdf_two_up,
    export_pdf_markdown_file, export_pdf_text_file, extract_pdf_images, extract_pdf_text,
    inspect_pdf, interleave_pdfs, merge_images_to_pdf, merge_pdfs, number_pdf_pages,
    organize_pdf_pages, remove_blank_pdf_pages, render_pdf_thumbnails, reorder_pdf_pages,
    rotate_pdf_pages, scrub_pdf_metadata, search_pdf_text, set_pdf_metadata, split_pdf,
    stamp_pdf_text, PdfPagePlan, PdfPageThumb, PdfSearchMatch,
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
    extract_video_frames, extract_video_poster, merge_audio, merge_videos, probe_hardware,
    split_media_segments, HardwareCapabilities,
};
