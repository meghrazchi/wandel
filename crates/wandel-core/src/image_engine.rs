use crate::error::{Result, WandelError};
use crate::formats::ImageFormat;
use crate::settings::{
    compute_target_dimensions, AspectCrop, ContactSheetOptions, FrameGradient, ImageFilterPreset,
    ImageSettings, ResizeMode, TextWatermarkSettings, WatermarkPosition, WatermarkSettings,
};
use base64::Engine as _;
use fast_image_resize::images::Image as FirImage;
use fast_image_resize::{FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};
use image::metadata::Orientation;
use image::{DynamicImage, GenericImageView, ImageDecoder, ImageReader};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePreview {
    pub original_data_url: String,
    pub converted_data_url: String,
    pub original_bytes: u64,
    pub estimated_bytes: u64,
    pub original_width: u32,
    pub original_height: u32,
    pub output_width: u32,
    pub output_height: u32,
}

/// Fast inspection of an image file: returns `(width, height, optional_thumbnail_data_url)`.
pub fn inspect_image(
    path: &Path,
    ffmpeg_path: Option<&Path>,
) -> (Option<u32>, Option<u32>, Option<String>) {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default();
    let fmt = ImageFormat::from_extension(ext);

    // Try fast decode for thumbnail + exact dimensions
    if let Ok(img) = decode_image(path, &ResizeMode::Original, true, ffmpeg_path) {
        let (w, h) = img.dimensions();
        let thumb = make_thumbnail_data_url(&img);
        return (Some(w), Some(h), thumb);
    }

    // Fallback to header-only dimensions if full decode failed
    if matches!(fmt, Some(ImageFormat::Svg)) {
        if let Ok(data) = std::fs::read(path) {
            if let Ok(tree) = resvg::usvg::Tree::from_data(&data, &resvg::usvg::Options::default())
            {
                let size = tree.size().to_int_size();
                return (Some(size.width()), Some(size.height()), None);
            }
        }
    }

    if let Ok((w, h)) = image::image_dimensions(path) {
        return (Some(w), Some(h), None);
    }

    (None, None, None)
}

pub(crate) fn make_thumbnail_data_url(img: &DynamicImage) -> Option<String> {
    let thumb = resize_image(
        img.clone(),
        &ResizeMode::MaxDimensions {
            width: Some(96),
            height: Some(96),
        },
        true,
    )
    .unwrap_or_else(|_| img.thumbnail(96, 96));

    let rgba = thumb.to_rgba8();
    let encoder = webp::Encoder::from_rgba(&rgba, rgba.width(), rgba.height());
    let mem = encoder.encode(70.0);
    let b64 = base64::engine::general_purpose::STANDARD.encode(&*mem);
    Some(format!("data:image/webp;base64,{}", b64))
}

/// Decodes any supported image format into a `DynamicImage`.
pub fn decode_image(
    path: &Path,
    resize_hint: &ResizeMode,
    never_upscale: bool,
    ffmpeg_path: Option<&Path>,
) -> Result<DynamicImage> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default();
    let fmt = ImageFormat::from_extension(ext);

    match fmt {
        Some(ImageFormat::Svg) => decode_svg(path, resize_hint, never_upscale),
        Some(ImageFormat::Jxl) => decode_jxl(path, ffmpeg_path),
        Some(ImageFormat::Heic) | Some(ImageFormat::Avif) => {
            decode_via_ffmpeg(path, ffmpeg_path)
        }
        _ => {
            let reader = ImageReader::open(path)
                .map_err(|e| WandelError::ImageDecode(format!("{}: {}", path.display(), e)))?
                .with_guessed_format()
                .map_err(|e| WandelError::ImageDecode(format!("{}: {}", path.display(), e)))?;
            let decoded = reader.into_decoder().and_then(|mut decoder| {
                let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
                let mut img = DynamicImage::from_decoder(decoder)?;
                if orientation != Orientation::NoTransforms {
                    img.apply_orientation(orientation);
                }
                Ok(img)
            });
            match decoded {
                Ok(img) => Ok(img),
                Err(primary_err) => {
                    // Fallback to FFmpeg if available (e.g., unusual pixel format or misnamed file)
                    if ffmpeg_path.is_some() {
                        decode_via_ffmpeg(path, ffmpeg_path)
                            .map_err(|_| WandelError::ImageDecode(primary_err.to_string()))
                    } else {
                        Err(WandelError::ImageDecode(primary_err.to_string()))
                    }
                }
            }
        }
    }
}

fn decode_svg(
    path: &Path,
    resize_hint: &ResizeMode,
    never_upscale: bool,
) -> Result<DynamicImage> {
    let data = std::fs::read(path)?;
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(&data, &opt)
        .map_err(|e| WandelError::ImageDecode(format!("SVG parse error: {}", e)))?;

    let orig_size = tree.size().to_int_size();
    let (target_w, target_h) = compute_target_dimensions(
        orig_size.width(),
        orig_size.height(),
        resize_hint,
        // SVGs are vector, so allow upscaling cleanly unless user explicitly locked original
        matches!(resize_hint, ResizeMode::Original) && never_upscale,
    )
    .unwrap_or((orig_size.width().max(1), orig_size.height().max(1)));

    let mut pixmap = resvg::tiny_skia::Pixmap::new(target_w, target_h)
        .ok_or_else(|| WandelError::ImageDecode("Failed to allocate SVG pixmap".into()))?;

    let sx = target_w as f32 / tree.size().width().max(1.0);
    let sy = target_h as f32 / tree.size().height().max(1.0);
    let transform = resvg::tiny_skia::Transform::from_scale(sx, sy);

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Convert premultiplied RGBA to straight RGBA for DynamicImage
    let mut rgba = pixmap.take();
    for px in rgba.chunks_exact_mut(4) {
        let a = px[3];
        if a > 0 && a < 255 {
            let af = a as f32 / 255.0;
            px[0] = ((px[0] as f32 / af).min(255.0)) as u8;
            px[1] = ((px[1] as f32 / af).min(255.0)) as u8;
            px[2] = ((px[2] as f32 / af).min(255.0)) as u8;
        }
    }

    let buf = image::RgbaImage::from_raw(target_w, target_h, rgba)
        .ok_or_else(|| WandelError::ImageDecode("Invalid SVG RGBA buffer".into()))?;
    Ok(DynamicImage::ImageRgba8(buf))
}

fn decode_jxl(path: &Path, ffmpeg_path: Option<&Path>) -> Result<DynamicImage> {
    let jxl_result = (|| -> Result<DynamicImage> {
        let image = jxl_oxide::JxlImage::builder()
            .open(path)
            .map_err(|e| WandelError::ImageDecode(format!("JXL open error: {}", e)))?;
        let render = image
            .render_frame(0)
            .map_err(|e| WandelError::ImageDecode(format!("JXL render error: {}", e)))?;
        let fb = render.image_all_channels();
        let width = fb.width() as u32;
        let height = fb.height() as u32;
        let channels = fb.channels();
        let buf = fb.buf();

        let to_u8 = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);

        match channels {
            1 => {
                for &g in buf {
                    let v = to_u8(g);
                    rgba.extend_from_slice(&[v, v, v, 255]);
                }
            }
            2 => {
                for chunk in buf.chunks_exact(2) {
                    let v = to_u8(chunk[0]);
                    let a = to_u8(chunk[1]);
                    rgba.extend_from_slice(&[v, v, v, a]);
                }
            }
            3 => {
                for chunk in buf.chunks_exact(3) {
                    rgba.extend_from_slice(&[
                        to_u8(chunk[0]),
                        to_u8(chunk[1]),
                        to_u8(chunk[2]),
                        255,
                    ]);
                }
            }
            _ => {
                for chunk in buf.chunks_exact(channels) {
                    rgba.extend_from_slice(&[
                        to_u8(chunk[0]),
                        to_u8(chunk[1]),
                        to_u8(chunk[2]),
                        to_u8(chunk[3]),
                    ]);
                }
            }
        }

        let img = image::RgbaImage::from_raw(width, height, rgba)
            .ok_or_else(|| WandelError::ImageDecode("JXL buffer size mismatch".into()))?;
        Ok(DynamicImage::ImageRgba8(img))
    })();

    if jxl_result.is_ok() {
        jxl_result
    } else if ffmpeg_path.is_some() {
        decode_via_ffmpeg(path, ffmpeg_path)
    } else {
        jxl_result
    }
}

fn decode_via_ffmpeg(path: &Path, ffmpeg_path: Option<&Path>) -> Result<DynamicImage> {
    let ffmpeg = ffmpeg_path.ok_or_else(|| {
        WandelError::FfmpegNotFound("FFmpeg is required to decode HEIC/AVIF/JXL images".into())
    })?;

    let output = Command::new(ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(path)
        .args(["-frames:v", "1", "-f", "image2pipe", "-vcodec", "png", "pipe:1"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(WandelError::ImageDecode(format!(
            "FFmpeg decode failed: {}",
            stderr.trim()
        )));
    }

    image::load_from_memory(&output.stdout)
        .map_err(|e| WandelError::ImageDecode(format!("Failed to parse FFmpeg PNG pipe: {}", e)))
}

/// SIMD-accelerated high-quality Lanczos3 image resizing via `fast_image_resize`.
pub fn resize_image(
    img: DynamicImage,
    mode: &ResizeMode,
    never_upscale: bool,
) -> Result<DynamicImage> {
    let (src_w, src_h) = img.dimensions();
    let Some((dst_w, dst_h)) = compute_target_dimensions(src_w, src_h, mode, never_upscale) else {
        return Ok(img);
    };

    let rgba = img.to_rgba8();
    let src_image = FirImage::from_vec_u8(src_w, src_h, rgba.into_raw(), PixelType::U8x4)
        .map_err(|e| WandelError::Resize(e.to_string()))?;

    let mut dst_image = FirImage::new(dst_w, dst_h, src_image.pixel_type());
    let mut resizer = Resizer::new();
    let options =
        ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::Lanczos3));

    resizer
        .resize(&src_image, &mut dst_image, Some(&options))
        .map_err(|e| WandelError::Resize(e.to_string()))?;

    let out_buf = image::RgbaImage::from_raw(dst_w, dst_h, dst_image.into_vec())
        .ok_or_else(|| WandelError::Resize("Failed to construct resized RGBA image".into()))?;
    Ok(DynamicImage::ImageRgba8(out_buf))
}

/// Applies rotation, horizontal/vertical flips, and center aspect-ratio cropping.
pub fn apply_image_transforms(mut img: DynamicImage, settings: &ImageSettings) -> DynamicImage {
    match settings.rotate % 360 {
        90 => img = img.rotate90(),
        180 => img = img.rotate180(),
        270 => img = img.rotate270(),
        _ => {}
    }
    if settings.flip_horizontal {
        img = img.fliph();
    }
    if settings.flip_vertical {
        img = img.flipv();
    }
    if let Some(crop) = settings.crop_aspect {
        img = center_crop_aspect(img, crop);
    }
    if let Some(cc) = settings.custom_crop {
        let (w, h) = img.dimensions();
        if w > 1 && h > 1 {
            let cx = ((cc.x.clamp(0.0, 0.99)) * w as f32).round() as u32;
            let cy = ((cc.y.clamp(0.0, 0.99)) * h as f32).round() as u32;
            let cw = ((cc.width.clamp(0.01, 1.0)) * w as f32)
                .round()
                .max(1.0) as u32;
            let ch = ((cc.height.clamp(0.01, 1.0)) * h as f32)
                .round()
                .max(1.0) as u32;
            let clamped_w = cw.min(w.saturating_sub(cx)).max(1);
            let clamped_h = ch.min(h.saturating_sub(cy)).max(1);
            img = img.crop_imm(cx, cy, clamped_w, clamped_h);
        }
    }
    if settings.auto_trim_border {
        img = auto_trim_image(img, settings.auto_trim_tolerance);
    }
    if let Some(pad) = settings.pad_aspect {
        img = pad_to_aspect(img, pad, settings.background_hex.as_deref());
    }
    img
}

/// Automatically trims uniform transparent or solid-color outer borders from `img`.
pub fn auto_trim_image(img: DynamicImage, tolerance: u8) -> DynamicImage {
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    if w <= 2 || h <= 2 {
        return img;
    }

    let tol = i16::from(tolerance.min(64));
    let corners = [
        *rgba.get_pixel(0, 0),
        *rgba.get_pixel(w - 1, 0),
        *rgba.get_pixel(0, h - 1),
        *rgba.get_pixel(w - 1, h - 1),
    ];

    let trim_transparent = corners.iter().all(|p| i16::from(p[3]) <= tol);
    let bg = corners[0];

    let is_border = |px: &image::Rgba<u8>| -> bool {
        if trim_transparent {
            i16::from(px[3]) <= tol
        } else {
            (i16::from(px[0]) - i16::from(bg[0])).abs() <= tol
                && (i16::from(px[1]) - i16::from(bg[1])).abs() <= tol
                && (i16::from(px[2]) - i16::from(bg[2])).abs() <= tol
                && (i16::from(px[3]) - i16::from(bg[3])).abs() <= tol
        }
    };

    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0u32;
    let mut max_y = 0u32;
    let mut found = false;

    for (x, y, px) in rgba.enumerate_pixels() {
        if !is_border(px) {
            found = true;
            if x < min_x {
                min_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if x > max_x {
                max_x = x;
            }
            if y > max_y {
                max_y = y;
            }
        }
    }

    if !found {
        return img;
    }

    let crop_w = (max_x - min_x + 1).max(1);
    let crop_h = (max_y - min_y + 1).max(1);
    if crop_w == w && crop_h == h {
        return img;
    }

    img.crop_imm(min_x, min_y, crop_w, crop_h)
}

fn pad_to_aspect(img: DynamicImage, aspect: AspectCrop, bg_hex: Option<&str>) -> DynamicImage {
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return img;
    }
    let (ar_w, ar_h) = aspect.ratio();
    let target_ratio = ar_w as f64 / ar_h as f64;
    let current_ratio = w as f64 / h as f64;
    if (current_ratio - target_ratio).abs() < 0.005 {
        return img;
    }

    let (canvas_w, canvas_h) = if current_ratio > target_ratio {
        // Too wide: add top/bottom padding
        let new_h = ((w as f64) / target_ratio).round().max(h as f64) as u32;
        (w, new_h)
    } else {
        // Too tall: add left/right padding
        let new_w = ((h as f64) * target_ratio).round().max(w as f64) as u32;
        (new_w, h)
    };

    let [r, g, b] = parse_hex_rgb(bg_hex.unwrap_or("#ffffff"));
    let mut canvas = image::RgbaImage::from_pixel(canvas_w, canvas_h, image::Rgba([r, g, b, 255]));
    let ox = (canvas_w - w) / 2;
    let oy = (canvas_h - h) / 2;
    let src = img.to_rgba8();

    for (x, y, px) in src.enumerate_pixels() {
        let a = px[3] as f32 / 255.0;
        let inv = 1.0 - a;
        let out_r = (px[0] as f32 * a + r as f32 * inv).round().clamp(0.0, 255.0) as u8;
        let out_g = (px[1] as f32 * a + g as f32 * inv).round().clamp(0.0, 255.0) as u8;
        let out_b = (px[2] as f32 * a + b as f32 * inv).round().clamp(0.0, 255.0) as u8;
        canvas.put_pixel(ox + x, oy + y, image::Rgba([out_r, out_g, out_b, 255]));
    }

    DynamicImage::ImageRgba8(canvas)
}

/// Applies smooth anti-aliased rounded corners (radius as % of min(width, height), 0..=50).
pub fn apply_rounded_corners(img: DynamicImage, radius_percent: f32) -> DynamicImage {
    let pct = radius_percent.clamp(0.0, 50.0);
    if pct <= 0.1 {
        return img;
    }
    let mut rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let r = (w.min(h) as f32) * (pct / 100.0);
    if r < 1.0 {
        return DynamicImage::ImageRgba8(rgba);
    }

    let fw = w as f32;
    let fh = h as f32;
    for y in 0..h {
        let py = y as f32 + 0.5;
        let dy = if py < r {
            r - py
        } else if py > fh - r {
            py - (fh - r)
        } else {
            0.0
        };
        if dy == 0.0 {
            continue;
        }
        for x in 0..w {
            let px = x as f32 + 0.5;
            let dx = if px < r {
                r - px
            } else if px > fw - r {
                px - (fw - r)
            } else {
                0.0
            };
            if dx == 0.0 {
                continue;
            }
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > r + 0.5 {
                rgba.get_pixel_mut(x, y)[3] = 0;
            } else if dist > r - 0.5 {
                let coverage = (r + 0.5 - dist).clamp(0.0, 1.0);
                let pixel = rgba.get_pixel_mut(x, y);
                pixel[3] = ((pixel[3] as f32) * coverage).round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    DynamicImage::ImageRgba8(rgba)
}

fn center_crop_aspect(img: DynamicImage, aspect: AspectCrop) -> DynamicImage {
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return img;
    }
    let (ar_w, ar_h) = aspect.ratio();
    let target_ratio = ar_w as f64 / ar_h as f64;
    let current_ratio = w as f64 / h as f64;

    if (current_ratio - target_ratio).abs() < 0.005 {
        return img;
    }

    let (crop_w, crop_h) = if current_ratio > target_ratio {
        // Image is wider than target: crop width
        let new_w = ((h as f64) * target_ratio).round().max(1.0) as u32;
        (new_w.min(w), h)
    } else {
        // Image is taller than target: crop height
        let new_h = ((w as f64) / target_ratio).round().max(1.0) as u32;
        (w, new_h.min(h))
    };

    let x = (w - crop_w) / 2;
    let y = (h - crop_h) / 2;
    img.crop_imm(x, y, crop_w, crop_h)
}

/// Composites a scaled logo watermark onto `base` at the configured corner and opacity.
pub fn apply_watermark(
    base: DynamicImage,
    wm: &WatermarkSettings,
    ffmpeg_path: Option<&Path>,
) -> Result<DynamicImage> {
    if !wm.image_path.is_file() {
        return Err(WandelError::ImageDecode(format!(
            "Watermark file not found: {}",
            wm.image_path.display()
        )));
    }

    let mut base_rgba = base.to_rgba8();
    let (base_w, base_h) = base_rgba.dimensions();
    let target_wm_w = ((base_w as f32) * (wm.scale_percent.clamp(5.0, 60.0) / 100.0))
        .round()
        .max(8.0) as u32;

    let wm_decoded = decode_image(
        &wm.image_path,
        &ResizeMode::MaxDimensions {
            width: Some(target_wm_w),
            height: None,
        },
        false,
        ffmpeg_path,
    )?;
    let wm_resized = resize_image(
        wm_decoded,
        &ResizeMode::MaxDimensions {
            width: Some(target_wm_w),
            height: Some(base_h.max(8)),
        },
        false,
    )?
    .to_rgba8();

    let (wm_w, wm_h) = wm_resized.dimensions();
    let margin = wm.margin_px.min(base_w / 4).min(base_h / 4);

    let (offset_x, offset_y) = match wm.position {
        WatermarkPosition::TopLeft => (margin, margin),
        WatermarkPosition::TopRight => (base_w.saturating_sub(wm_w + margin), margin),
        WatermarkPosition::BottomLeft => (margin, base_h.saturating_sub(wm_h + margin)),
        WatermarkPosition::BottomRight => (
            base_w.saturating_sub(wm_w + margin),
            base_h.saturating_sub(wm_h + margin),
        ),
        WatermarkPosition::Center => (
            base_w.saturating_sub(wm_w) / 2,
            base_h.saturating_sub(wm_h) / 2,
        ),
    };

    let opacity = wm.opacity.clamp(0.05, 1.0);
    for y in 0..wm_h {
        let by = offset_y + y;
        if by >= base_h {
            continue;
        }
        for x in 0..wm_w {
            let bx = offset_x + x;
            if bx >= base_w {
                continue;
            }
            let src = wm_resized.get_pixel(x, y);
            let src_a = (src[3] as f32 / 255.0) * opacity;
            if src_a <= 0.001 {
                continue;
            }
            let dst = base_rgba.get_pixel_mut(bx, by);
            let dst_a = dst[3] as f32 / 255.0;
            let out_a = src_a + dst_a * (1.0 - src_a);
            if out_a > 0.0 {
                for c in 0..3 {
                    let s = src[c] as f32;
                    let d = dst[c] as f32;
                    dst[c] = ((s * src_a + d * dst_a * (1.0 - src_a)) / out_a)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                }
                dst[3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    Ok(DynamicImage::ImageRgba8(base_rgba))
}

/// Runs decode, transforms, resize, color adjustments, grayscale/sharpen, and watermarks.
pub fn process_image_pipeline(
    input: &Path,
    settings: &ImageSettings,
    ffmpeg_path: Option<&Path>,
) -> Result<DynamicImage> {
    let is_svg = input
        .extension()
        .and_then(|e| e.to_str())
        .and_then(ImageFormat::from_extension)
        == Some(ImageFormat::Svg);

    let decoded = decode_image(input, &settings.resize, settings.never_upscale, ffmpeg_path)?;
    apply_post_decode_pipeline(decoded, settings, is_svg, ffmpeg_path)
}

/// Shared post-decode processing pipeline (used by both single images and PDF pages).
pub fn apply_post_decode_pipeline(
    decoded: DynamicImage,
    settings: &ImageSettings,
    is_svg: bool,
    ffmpeg_path: Option<&Path>,
) -> Result<DynamicImage> {
    let transformed = apply_image_transforms(decoded, settings);

    // SVGs are already rasterized at target dimensions unless cropped/rotated
    let mut processed = if is_svg && settings.crop_aspect.is_none() && settings.rotate == 0 {
        transformed
    } else {
        resize_image(transformed, &settings.resize, settings.never_upscale)?
    };

    if settings.auto_levels {
        processed = apply_auto_levels(processed);
    }
    if settings.brightness != 0 {
        processed = processed.brighten(settings.brightness.clamp(-100, 100));
    }
    if settings.contrast.abs() > 0.1 {
        processed = processed.adjust_contrast(settings.contrast.clamp(-100.0, 100.0));
    }
    if settings.saturation.abs() > 0.1 {
        processed = adjust_saturation(processed, settings.saturation.clamp(-100.0, 100.0));
    }
    if settings.grayscale {
        processed = processed.grayscale();
    }
    if settings.sharpen {
        processed = processed.unsharpen(1.0, 4);
    }
    if settings.filter_preset != ImageFilterPreset::None {
        processed = apply_filter_preset(processed, settings.filter_preset);
    }
    if settings.pixelate_block >= 2 {
        processed = apply_pixelate(processed, settings.pixelate_block.clamp(2, 64));
    }
    if settings.blur_sigma > 0.1 {
        processed = processed.blur(settings.blur_sigma.clamp(0.1, 20.0));
    }
    if settings.vignette_strength > 0.01 {
        processed = apply_vignette(processed, settings.vignette_strength.clamp(0.0, 1.0));
    }
    if settings.corner_radius_percent > 0.1 {
        processed = apply_rounded_corners(processed, settings.corner_radius_percent);
    }
    if settings.frame_padding_px > 0 {
        processed = apply_frame_and_shadow(
            processed,
            settings.frame_padding_px,
            settings.frame_gradient,
            settings.drop_shadow,
            settings.background_hex.as_deref(),
        );
    }

    if let Some(wm) = &settings.watermark {
        processed = apply_watermark(processed, wm, ffmpeg_path)?;
    }

    if let Some(twm) = &settings.text_watermark {
        if !twm.text.trim().is_empty() {
            processed = apply_text_watermark(processed, twm)?;
        }
    }

    Ok(processed)
}

fn apply_frame_and_shadow(
    img: DynamicImage,
    padding_px: u32,
    gradient: FrameGradient,
    drop_shadow: bool,
    bg_hex: Option<&str>,
) -> DynamicImage {
    let pad = padding_px.clamp(4, 160);
    let fg = img.to_rgba8();
    let (fw, fh) = fg.dimensions();
    let canvas_w = fw + pad * 2;
    let canvas_h = fh + pad * 2;

    let (start_rgb, end_rgb) = match gradient {
        FrameGradient::None => {
            let c = parse_hex_rgb(bg_hex.unwrap_or("#f4f4f5"));
            (c, c)
        }
        FrameGradient::Indigo => ([79, 70, 229], [147, 51, 234]),
        FrameGradient::Sunset => ([249, 115, 22], [236, 72, 153]),
        FrameGradient::Ocean => ([6, 182, 212], [59, 130, 246]),
        FrameGradient::Emerald => ([16, 185, 129], [20, 184, 166]),
        FrameGradient::Midnight => ([15, 23, 42], [49, 46, 129]),
    };

    let mut canvas = image::RgbaImage::new(canvas_w, canvas_h);
    let denom = ((canvas_w + canvas_h).max(1)) as f32;
    for (x, y, px) in canvas.enumerate_pixels_mut() {
        let t = ((x + y) as f32 / denom).clamp(0.0, 1.0);
        let r = (start_rgb[0] as f32 * (1.0 - t) + end_rgb[0] as f32 * t).round() as u8;
        let g = (start_rgb[1] as f32 * (1.0 - t) + end_rgb[1] as f32 * t).round() as u8;
        let b = (start_rgb[2] as f32 * (1.0 - t) + end_rgb[2] as f32 * t).round() as u8;
        *px = image::Rgba([r, g, b, 255]);
    }

    if drop_shadow {
        let mut shadow_mask =
            image::RgbaImage::from_pixel(canvas_w, canvas_h, image::Rgba([0, 0, 0, 0]));
        let shadow_offset_y = (pad / 4).clamp(4, 18);
        for (x, y, px) in fg.enumerate_pixels() {
            if px[3] > 0 {
                let sy = (pad + y + shadow_offset_y).min(canvas_h - 1);
                let sx = pad + x;
                let a = ((px[3] as f32) * 0.42).round() as u8;
                shadow_mask.put_pixel(sx, sy, image::Rgba([0, 0, 0, a]));
            }
        }
        let blur_sigma = (pad as f32 * 0.25).clamp(4.0, 16.0);
        let blurred = DynamicImage::ImageRgba8(shadow_mask)
            .blur(blur_sigma)
            .to_rgba8();
        image::imageops::overlay(&mut canvas, &blurred, 0, 0);
    }

    image::imageops::overlay(&mut canvas, &fg, i64::from(pad), i64::from(pad));
    DynamicImage::ImageRgba8(canvas)
}

fn apply_auto_levels(img: DynamicImage) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    let total_px = u64::from(rgba.width()) * u64::from(rgba.height());
    if total_px == 0 {
        return DynamicImage::ImageRgba8(rgba);
    }

    let mut hist = [0u64; 256];
    for px in rgba.pixels() {
        let luma = (0.299 * f32::from(px[0]) + 0.587 * f32::from(px[1]) + 0.114 * f32::from(px[2]))
            .round()
            .clamp(0.0, 255.0) as usize;
        hist[luma] += 1;
    }

    let low_thresh = total_px / 100;
    let high_thresh = total_px.saturating_sub(low_thresh);

    let mut acc = 0u64;
    let mut low = 0usize;
    for (i, &c) in hist.iter().enumerate() {
        acc += c;
        if acc >= low_thresh {
            low = i;
            break;
        }
    }

    acc = 0;
    let mut high = 255usize;
    for (i, &c) in hist.iter().enumerate() {
        acc += c;
        if acc >= high_thresh {
            high = i;
            break;
        }
    }

    if high <= low + 4 {
        return DynamicImage::ImageRgba8(rgba);
    }

    let lo_f = low as f32;
    let scale = 255.0 / (high - low) as f32;
    for px in rgba.pixels_mut() {
        px[0] = ((f32::from(px[0]) - lo_f) * scale).round().clamp(0.0, 255.0) as u8;
        px[1] = ((f32::from(px[1]) - lo_f) * scale).round().clamp(0.0, 255.0) as u8;
        px[2] = ((f32::from(px[2]) - lo_f) * scale).round().clamp(0.0, 255.0) as u8;
    }

    DynamicImage::ImageRgba8(rgba)
}

fn apply_filter_preset(img: DynamicImage, preset: ImageFilterPreset) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    for px in rgba.pixels_mut() {
        let r = px[0] as f32;
        let g = px[1] as f32;
        let b = px[2] as f32;
        let (nr, ng, nb) = match preset {
            ImageFilterPreset::None => (r, g, b),
            ImageFilterPreset::Sepia => (
                0.393 * r + 0.769 * g + 0.189 * b,
                0.349 * r + 0.686 * g + 0.168 * b,
                0.272 * r + 0.534 * g + 0.131 * b,
            ),
            ImageFilterPreset::Warm => (r * 1.12 + 8.0, g * 1.03, b * 0.88),
            ImageFilterPreset::Cool => (r * 0.88, g * 1.02, b * 1.15 + 6.0),
            ImageFilterPreset::Vintage => (
                0.9 * r + 0.15 * g + 14.0,
                0.08 * r + 0.85 * g + 10.0,
                0.06 * r + 0.12 * g + 0.72 * b + 18.0,
            ),
            ImageFilterPreset::HighContrastBw => {
                let luma = 0.299 * r + 0.587 * g + 0.114 * b;
                let contrasted = (luma - 128.0) * 1.45 + 128.0;
                (contrasted, contrasted, contrasted)
            }
            ImageFilterPreset::Invert => (255.0 - r, 255.0 - g, 255.0 - b),
        };
        px[0] = nr.round().clamp(0.0, 255.0) as u8;
        px[1] = ng.round().clamp(0.0, 255.0) as u8;
        px[2] = nb.round().clamp(0.0, 255.0) as u8;
    }
    DynamicImage::ImageRgba8(rgba)
}

fn apply_pixelate(img: DynamicImage, block: u32) -> DynamicImage {
    let (w, h) = img.dimensions();
    let small_w = (w / block).max(1);
    let small_h = (h / block).max(1);
    let down = img.resize_exact(small_w, small_h, image::imageops::FilterType::Triangle);
    down.resize_exact(w, h, image::imageops::FilterType::Nearest)
}

fn apply_vignette(img: DynamicImage, strength: f32) -> DynamicImage {
    let mut rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let max_dist = (cx * cx + cy * cy).sqrt().max(1.0);

    for (x, y, px) in rgba.enumerate_pixels_mut() {
        let dx = x as f32 - cx;
        let dy = y as f32 - cy;
        let norm = (dx * dx + dy * dy).sqrt() / max_dist;
        // Smooth radial darkening toward corners
        let factor = (1.0 - strength * norm * norm).clamp(0.0, 1.0);
        px[0] = ((px[0] as f32) * factor).round() as u8;
        px[1] = ((px[1] as f32) * factor).round() as u8;
        px[2] = ((px[2] as f32) * factor).round() as u8;
    }
    DynamicImage::ImageRgba8(rgba)
}

fn adjust_saturation(img: DynamicImage, delta_percent: f32) -> DynamicImage {
    let factor = (1.0 + delta_percent / 100.0).max(0.0);
    let mut rgba = img.to_rgba8();
    for px in rgba.pixels_mut() {
        let r = px[0] as f32;
        let g = px[1] as f32;
        let b = px[2] as f32;
        let luma = 0.299 * r + 0.587 * g + 0.114 * b;
        px[0] = (luma + (r - luma) * factor).round().clamp(0.0, 255.0) as u8;
        px[1] = (luma + (g - luma) * factor).round().clamp(0.0, 255.0) as u8;
        px[2] = (luma + (b - luma) * factor).round().clamp(0.0, 255.0) as u8;
    }
    DynamicImage::ImageRgba8(rgba)
}

fn parse_hex_rgb(hex: &str) -> [u8; 3] {
    let h = hex.trim().trim_start_matches('#');
    if h.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&h[0..2], 16),
            u8::from_str_radix(&h[2..4], 16),
            u8::from_str_radix(&h[4..6], 16),
        ) {
            return [r, g, b];
        }
    }
    [255, 255, 255]
}

fn flatten_alpha_to_rgb(img: &DynamicImage, bg_hex: Option<&str>) -> image::RgbImage {
    let Some(hex) = bg_hex else {
        return img.to_rgb8();
    };
    let [bg_r, bg_g, bg_b] = parse_hex_rgb(hex);
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut out = image::RgbImage::new(w, h);
    for (x, y, px) in rgba.enumerate_pixels() {
        let a = px[3] as f32 / 255.0;
        let inv = 1.0 - a;
        let r = (px[0] as f32 * a + bg_r as f32 * inv).round().clamp(0.0, 255.0) as u8;
        let g = (px[1] as f32 * a + bg_g as f32 * inv).round().clamp(0.0, 255.0) as u8;
        let b = (px[2] as f32 * a + bg_b as f32 * inv).round().clamp(0.0, 255.0) as u8;
        out.put_pixel(x, y, image::Rgb([r, g, b]));
    }
    out
}

/// Renders a crisp anti-aliased text watermark via `resvg` and composites it onto `base`.
pub fn apply_text_watermark(
    base: DynamicImage,
    twm: &TextWatermarkSettings,
) -> Result<DynamicImage> {
    let mut base_rgba = base.to_rgba8();
    let (base_w, base_h) = base_rgba.dimensions();
    let font_px = ((base_w as f32) * (twm.font_size_percent.clamp(2.0, 12.0) / 100.0))
        .round()
        .max(12.0);
    let margin = twm.margin_px.min(base_w / 4).min(base_h / 4) as f32;

    let (x_str, anchor) = match twm.position {
        WatermarkPosition::TopLeft | WatermarkPosition::BottomLeft => {
            (format!("{:.1}", margin), "start")
        }
        WatermarkPosition::TopRight | WatermarkPosition::BottomRight => {
            (format!("{:.1}", base_w as f32 - margin), "end")
        }
        WatermarkPosition::Center => (format!("{:.1}", base_w as f32 / 2.0), "middle"),
    };

    let y_val = match twm.position {
        WatermarkPosition::TopLeft | WatermarkPosition::TopRight => margin + font_px,
        WatermarkPosition::BottomLeft | WatermarkPosition::BottomRight => {
            (base_h as f32 - margin).max(font_px)
        }
        WatermarkPosition::Center => (base_h as f32 + font_px * 0.7) / 2.0,
    };

    let escaped = twm
        .text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");

    let color = if twm.color.starts_with('#') {
        twm.color.as_str()
    } else {
        "#ffffff"
    };
    let opacity = twm.opacity.clamp(0.05, 1.0);

    let svg = if twm.tiled {
        let step_x = ((escaped.chars().count().max(4) as f32) * font_px * 0.78 + font_px * 2.5)
            .max(font_px * 4.5);
        let step_y = (font_px * 3.8).max(48.0);
        let diag = ((base_w as f32).hypot(base_h as f32)) * 1.25;
        let mut tiles = String::new();
        let mut y = -diag * 0.4;
        while y < diag {
            let mut x = -diag * 0.4;
            while x < diag {
                tiles.push_str(&format!(
                    r##"<text x="{x:.1}" y="{y:.1}" font-family="sans-serif" font-weight="700" font-size="{fs:.1}" fill="{color}" fill-opacity="{op:.2}">{text}</text>"##,
                    x = x,
                    y = y,
                    fs = font_px,
                    color = color,
                    op = (opacity * 0.55).clamp(0.05, 0.8),
                    text = escaped,
                ));
                x += step_x;
            }
            y += step_y;
        }
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}"><g transform="rotate(-30 {cx:.1} {cy:.1})">{tiles}</g></svg>"##,
            w = base_w,
            h = base_h,
            cx = base_w as f32 / 2.0,
            cy = base_h as f32 / 2.0,
            tiles = tiles,
        )
    } else {
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}">
            <text x="{x}" y="{y_sh:.1}" text-anchor="{anchor}" font-family="sans-serif" font-weight="700" font-size="{fs:.1}" fill="#000000" fill-opacity="{sh_op:.2}">{text}</text>
            <text x="{x}" y="{y:.1}" text-anchor="{anchor}" font-family="sans-serif" font-weight="700" font-size="{fs:.1}" fill="{color}" fill-opacity="{op:.2}">{text}</text>
        </svg>"##,
            w = base_w,
            h = base_h,
            x = x_str,
            y = y_val,
            y_sh = y_val + (font_px * 0.06).max(1.0),
            anchor = anchor,
            fs = font_px,
            color = color,
            op = opacity,
            sh_op = (opacity * 0.55).min(0.6),
            text = escaped,
        )
    };

    let mut opt = resvg::usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();

    let tree = resvg::usvg::Tree::from_str(&svg, &opt)
        .map_err(|e| WandelError::ImageEncode(format!("Text watermark SVG error: {}", e)))?;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(base_w, base_h)
        .ok_or_else(|| WandelError::ImageEncode("Failed to allocate watermark pixmap".into()))?;
    resvg::render(&tree, resvg::tiny_skia::Transform::identity(), &mut pixmap.as_mut());

    let wm_data = pixmap.data();
    for (idx, dst) in base_rgba.pixels_mut().enumerate() {
        let off = idx * 4;
        let sa = wm_data[off + 3] as f32 / 255.0;
        if sa <= 0.001 {
            continue;
        }
        // pixmap is premultiplied RGBA
        let sr = wm_data[off] as f32;
        let sg = wm_data[off + 1] as f32;
        let sb = wm_data[off + 2] as f32;
        let da = dst[3] as f32 / 255.0;
        let out_a = sa + da * (1.0 - sa);
        if out_a > 0.0 {
            dst[0] = ((sr + (dst[0] as f32) * da * (1.0 - sa)) / out_a)
                .round()
                .clamp(0.0, 255.0) as u8;
            dst[1] = ((sg + (dst[1] as f32) * da * (1.0 - sa)) / out_a)
                .round()
                .clamp(0.0, 255.0) as u8;
            dst[2] = ((sb + (dst[2] as f32) * da * (1.0 - sa)) / out_a)
                .round()
                .clamp(0.0, 255.0) as u8;
            dst[3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }

    Ok(DynamicImage::ImageRgba8(base_rgba))
}

/// Converts and writes a single image from `input` to `output` according to `settings`.
pub fn convert_image(
    input: &Path,
    output: &Path,
    settings: &ImageSettings,
    ffmpeg_path: Option<&Path>,
) -> Result<u64> {
    if !settings.format.can_encode() {
        return Err(WandelError::UnsupportedFormat(format!(
            "Cannot encode to {}",
            settings.format.display_name()
        )));
    }

    let processed = process_image_pipeline(input, settings, ffmpeg_path)?;
    encode_image_with_target(&processed, settings, output, ffmpeg_path)
}

/// Encodes an already-processed `DynamicImage` to `output`, applying target-size binary search if set.
pub fn encode_image_with_target(
    processed: &DynamicImage,
    settings: &ImageSettings,
    output: &Path,
    ffmpeg_path: Option<&Path>,
) -> Result<u64> {
    if let Some(target_kb) = settings.target_size_kb {
        if target_kb > 0 && settings.format.supports_quality() && !settings.lossless {
            encode_image_to_target_size(
                processed,
                settings,
                output,
                u64::from(target_kb) * 1024,
                ffmpeg_path,
            )?;
            let size = std::fs::metadata(output)?.len();
            return Ok(size);
        }
    }

    encode_image(processed, settings, output, ffmpeg_path)?;

    let size = std::fs::metadata(output)?.len();
    Ok(size)
}

/// Binary searches quality (10..=95) to find the highest visual quality whose encoded
/// output fits within `target_bytes`.
fn encode_image_to_target_size(
    img: &DynamicImage,
    base_settings: &ImageSettings,
    dest: &Path,
    target_bytes: u64,
    ffmpeg_path: Option<&Path>,
) -> Result<()> {
    let mut low: u8 = 10;
    let mut high: u8 = 95;
    let mut best_quality: u8 = low;
    let mut found_under = false;

    for _ in 0..6 {
        if low > high {
            break;
        }
        let mid = low + (high - low) / 2;
        let mut trial = base_settings.clone();
        trial.quality = mid;
        encode_image(img, &trial, dest, ffmpeg_path)?;
        let size = std::fs::metadata(dest)?.len();

        if size <= target_bytes {
            best_quality = mid;
            found_under = true;
            low = mid.saturating_add(1);
        } else {
            if mid <= 10 {
                break;
            }
            high = mid.saturating_sub(1);
        }
    }

    let final_q = if found_under { best_quality } else { 10 };
    let mut final_settings = base_settings.clone();
    final_settings.quality = final_q;
    encode_image(img, &final_settings, dest, ffmpeg_path)
}

pub fn encode_image(
    img: &DynamicImage,
    settings: &ImageSettings,
    dest: &Path,
    ffmpeg_path: Option<&Path>,
) -> Result<()> {
    let quality = settings.quality.clamp(1, 100);

    match settings.format {
        ImageFormat::Jpg => {
            let rgb = flatten_alpha_to_rgb(img, settings.background_hex.as_deref());
            let mut buf = Vec::new();
            let mut encoder = jpeg_encoder::Encoder::new(&mut buf, quality);
            encoder.set_progressive(true);
            encoder
                .encode(
                    rgb.as_raw(),
                    rgb.width() as u16,
                    rgb.height() as u16,
                    jpeg_encoder::ColorType::Rgb,
                )
                .map_err(|e| WandelError::ImageEncode(format!("JPEG encode error: {}", e)))?;
            std::fs::write(dest, buf)?;
            Ok(())
        }
        ImageFormat::Png => {
            let mut raw_png = Vec::new();
            img.write_to(&mut Cursor::new(&mut raw_png), image::ImageFormat::Png)
                .map_err(|e| WandelError::ImageEncode(format!("PNG encode error: {}", e)))?;

            // Fast lossless PNG optimization with oxipng
            let preset = if quality >= 90 { 2 } else { 3 };
            let mut opts = oxipng::Options::from_preset(preset);
            if settings.strip_metadata {
                opts.strip = oxipng::StripChunks::Safe;
            }
            let optimized = oxipng::optimize_from_memory(&raw_png, &opts).unwrap_or(raw_png);
            std::fs::write(dest, optimized)?;
            Ok(())
        }
        ImageFormat::Webp => {
            let rgba = img.to_rgba8();
            let encoder = webp::Encoder::from_rgba(&rgba, rgba.width(), rgba.height());
            let mem = if settings.lossless {
                encoder.encode_lossless()
            } else {
                encoder.encode(quality as f32)
            };
            std::fs::write(dest, &*mem)?;
            Ok(())
        }
        ImageFormat::Avif => {
            let rgba = img.to_rgba8();
            let pixels: Vec<rgb::RGBA8> = rgba
                .pixels()
                .map(|p| rgb::RGBA8::new(p[0], p[1], p[2], p[3]))
                .collect();
            let img_ref =
                ravif::Img::new(pixels.as_slice(), rgba.width() as usize, rgba.height() as usize);
            let encoded = ravif::Encoder::new()
                .with_quality(quality as f32)
                .with_speed(6)
                .encode_rgba(img_ref)
                .map_err(|e| WandelError::ImageEncode(format!("AVIF encode error: {}", e)))?;
            std::fs::write(dest, encoded.avif_file)?;
            Ok(())
        }
        ImageFormat::Gif => {
            img.save_with_format(dest, image::ImageFormat::Gif)
                .map_err(|e| WandelError::ImageEncode(format!("GIF encode error: {}", e)))?;
            Ok(())
        }
        ImageFormat::Bmp => {
            flatten_alpha_to_rgb(img, settings.background_hex.as_deref())
                .save_with_format(dest, image::ImageFormat::Bmp)
                .map_err(|e| WandelError::ImageEncode(format!("BMP encode error: {}", e)))?;
            Ok(())
        }
        ImageFormat::Tiff => {
            img.save_with_format(dest, image::ImageFormat::Tiff)
                .map_err(|e| WandelError::ImageEncode(format!("TIFF encode error: {}", e)))?;
            Ok(())
        }
        ImageFormat::Ico => {
            // ICO format requires dimensions <= 256x256
            let ico_img = if img.width() > 256 || img.height() > 256 {
                resize_image(
                    img.clone(),
                    &ResizeMode::MaxDimensions {
                        width: Some(256),
                        height: Some(256),
                    },
                    true,
                )?
            } else {
                img.clone()
            };
            ico_img
                .save_with_format(dest, image::ImageFormat::Ico)
                .map_err(|e| WandelError::ImageEncode(format!("ICO encode error: {}", e)))?;
            Ok(())
        }
        ImageFormat::Jxl => encode_via_ffmpeg(img, dest, "libjxl", quality, settings.lossless, ffmpeg_path),
        ImageFormat::Heic => encode_via_ffmpeg(img, dest, "libx265", quality, settings.lossless, ffmpeg_path),
        ImageFormat::Svg => Err(WandelError::UnsupportedFormat(
            "Encoding to SVG is not supported".into(),
        )),
    }
}

fn encode_via_ffmpeg(
    img: &DynamicImage,
    dest: &Path,
    codec: &str,
    quality: u8,
    lossless: bool,
    ffmpeg_path: Option<&Path>,
) -> Result<()> {
    let ffmpeg = ffmpeg_path.ok_or_else(|| {
        WandelError::FfmpegNotFound(format!("FFmpeg is required to encode with {}", codec))
    })?;

    let mut raw_png = Vec::new();
    img.write_to(&mut Cursor::new(&mut raw_png), image::ImageFormat::Png)
        .map_err(|e| WandelError::ImageEncode(e.to_string()))?;

    let mut cmd = Command::new(ffmpeg);
    cmd.args(["-y", "-hide_banner", "-loglevel", "error", "-f", "image2pipe", "-vcodec", "png", "-i", "pipe:0"]);

    if codec == "libjxl" {
        cmd.args(["-c:v", "libjxl"]);
        if lossless {
            cmd.args(["-distance", "0"]);
        } else {
            // Map 1..100 quality to JXL distance ~15.0..0.1
            let dist = ((100.0 - quality as f32) / 100.0 * 12.0).clamp(0.1, 15.0);
            cmd.args(["-distance", &format!("{:.2}", dist)]);
        }
    } else {
        // HEIC/HEVC single frame
        let crf = ((100 - quality) as f32 * 0.45).round() as u32;
        cmd.args(["-c:v", "libx265", "-crf", &crf.to_string(), "-pix_fmt", "yuv420p", "-frames:v", "1"]);
    }

    cmd.arg(dest);
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(&raw_png)?;
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(WandelError::ImageEncode(format!(
            "FFmpeg {} encode failed: {}",
            codec,
            stderr.trim()
        )));
    }
    Ok(())
}

/// Generates a real-time Before/After visual preview and exact estimated output size
/// by running the configured conversion pipeline in a temporary file.
pub fn generate_image_preview(
    input: &Path,
    settings: &ImageSettings,
    ffmpeg_path: Option<&Path>,
) -> Result<ImagePreview> {
    let original_bytes = std::fs::metadata(input)?.len();
    let original_img = decode_image(input, &ResizeMode::Original, true, ffmpeg_path)?;
    let (original_width, original_height) = original_img.dimensions();

    // Encode to a unique temp file using the exact real conversion pipeline
    let ext = settings.format.extension();
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_path = std::env::temp_dir().join(format!(
        "wandel_preview_{}_{}.{}",
        std::process::id(),
        unique,
        ext
    ));

    let convert_res = convert_image(input, &temp_path, settings, ffmpeg_path);
    let estimated_bytes = match convert_res {
        Ok(bytes) => bytes,
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            return Err(e);
        }
    };

    let converted_img = decode_image(&temp_path, &ResizeMode::Original, true, ffmpeg_path);
    let _ = std::fs::remove_file(&temp_path);
    let converted_img = converted_img?;
    let (output_width, output_height) = converted_img.dimensions();

    // Clamp display data URLs to 1200px max for smooth 60fps slider rendering in the webview
    let preview_resize = ResizeMode::MaxDimensions {
        width: Some(1200),
        height: Some(1200),
    };
    let orig_display = resize_image(original_img, &preview_resize, true)?;
    let conv_display = resize_image(converted_img, &preview_resize, true)?;

    let to_webp_data_url = |img: &DynamicImage| -> String {
        let rgba = img.to_rgba8();
        let enc = webp::Encoder::from_rgba(&rgba, rgba.width(), rgba.height());
        let mem = enc.encode_lossless();
        let b64 = base64::engine::general_purpose::STANDARD.encode(&*mem);
        format!("data:image/webp;base64,{}", b64)
    };

    Ok(ImagePreview {
        original_data_url: to_webp_data_url(&orig_display),
        converted_data_url: to_webp_data_url(&conv_display),
        original_bytes,
        estimated_bytes,
        original_width,
        original_height,
        output_width,
        output_height,
    })
}

/// Generates a complete Web & App Favicon / PWA Icon Pack into `output_dir`:
/// `favicon.ico`, `favicon-16x16.png`, `favicon-32x32.png`, `apple-touch-icon.png` (180x180),
/// `icon-192.png`, `icon-512.png`, and `site.webmanifest`.
pub fn generate_icon_pack(
    input: &Path,
    output_dir: &Path,
    ffmpeg_path: Option<&Path>,
) -> Result<Vec<String>> {
    std::fs::create_dir_all(output_dir)?;

    let base = decode_image(
        input,
        &ResizeMode::Exact {
            width: 512,
            height: 512,
            keep_aspect_ratio: true,
        },
        false,
        ffmpeg_path,
    )?;
    // Pad to 1:1 square transparently if non-square
    let (w, h) = base.dimensions();
    let square = if w != h {
        let side = w.max(h);
        let mut canvas = image::RgbaImage::from_pixel(side, side, image::Rgba([0, 0, 0, 0]));
        let ox = (side - w) / 2;
        let oy = (side - h) / 2;
        image::imageops::overlay(&mut canvas, &base.to_rgba8(), i64::from(ox), i64::from(oy));
        DynamicImage::ImageRgba8(canvas)
    } else {
        base
    };

    let targets: &[(&str, u32)] = &[
        ("favicon-16x16.png", 16),
        ("favicon-32x32.png", 32),
        ("apple-touch-icon.png", 180),
        ("icon-192.png", 192),
        ("icon-512.png", 512),
    ];

    let mut created = Vec::with_capacity(targets.len() + 2);
    for &(fname, size) in targets {
        let resized = resize_image(
            square.clone(),
            &ResizeMode::Exact {
                width: size,
                height: size,
                keep_aspect_ratio: false,
            },
            false,
        )?;
        let dest = output_dir.join(fname);
        resized
            .save_with_format(&dest, image::ImageFormat::Png)
            .map_err(|e| WandelError::ImageEncode(format!("Icon {} error: {}", fname, e)))?;
        created.push(dest.to_string_lossy().into_owned());
    }

    // 48x48 favicon.ico
    let ico_img = resize_image(
        square,
        &ResizeMode::Exact {
            width: 48,
            height: 48,
            keep_aspect_ratio: false,
        },
        false,
    )?;
    let ico_path = output_dir.join("favicon.ico");
    ico_img
        .save_with_format(&ico_path, image::ImageFormat::Ico)
        .map_err(|e| WandelError::ImageEncode(format!("favicon.ico error: {}", e)))?;
    created.push(ico_path.to_string_lossy().into_owned());

    // Ready-to-use site.webmanifest
    let manifest_path = output_dir.join("site.webmanifest");
    let manifest = r##"{
  "name": "My App",
  "short_name": "App",
  "icons": [
    { "src": "/icon-192.png", "sizes": "192x192", "type": "image/png" },
    { "src": "/icon-512.png", "sizes": "512x512", "type": "image/png" }
  ],
  "theme_color": "#4f46e5",
  "background_color": "#ffffff",
  "display": "standalone"
}
"##;
    std::fs::write(&manifest_path, manifest)?;
    created.push(manifest_path.to_string_lossy().into_owned());

    Ok(created)
}

/// Stitches an ordered list of images into an N-column Contact Sheet / Grid collage
/// in parallel across all CPU cores.
pub fn create_contact_sheet(
    inputs: &[PathBuf],
    output: &Path,
    opts: &ContactSheetOptions,
    settings: &ImageSettings,
    ffmpeg_path: Option<&Path>,
) -> Result<u64> {
    use rayon::prelude::*;

    if inputs.is_empty() {
        return Err(WandelError::InvalidSettings(
            "No images provided for contact sheet".into(),
        ));
    }

    let cell = opts.cell_size.clamp(64, 2000);
    let gap = opts.gap_px.min(200);
    let count = inputs.len() as u32;
    let cols = if opts.columns > 0 {
        opts.columns.min(count)
    } else {
        (count as f64).sqrt().ceil() as u32
    }
    .max(1);
    let rows = count.div_ceil(cols);

    let fit_mode = ResizeMode::MaxDimensions {
        width: Some(cell),
        height: Some(cell),
    };

    // Decode and resize all tiles in parallel
    let tiles: Result<Vec<image::RgbaImage>> = inputs
        .par_iter()
        .map(|p| {
            let decoded = decode_image(p, &fit_mode, false, ffmpeg_path)?;
            let resized = resize_image(decoded, &fit_mode, false)?;
            Ok(resized.to_rgba8())
        })
        .collect();
    let tiles = tiles?;

    let canvas_w = cols * cell + (cols + 1) * gap;
    let canvas_h = rows * cell + (rows + 1) * gap;
    let [r, g, b] = parse_hex_rgb(&opts.background_hex);
    let mut canvas = image::RgbaImage::from_pixel(canvas_w, canvas_h, image::Rgba([r, g, b, 255]));

    for (idx, tile) in tiles.iter().enumerate() {
        let col = (idx as u32) % cols;
        let row = (idx as u32) / cols;
        let cell_x = gap + col * (cell + gap);
        let cell_y = gap + row * (cell + gap);
        let ox = cell_x + (cell.saturating_sub(tile.width())) / 2;
        let oy = cell_y + (cell.saturating_sub(tile.height())) / 2;
        image::imageops::overlay(&mut canvas, tile, i64::from(ox), i64::from(oy));
    }

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let final_img = DynamicImage::ImageRgba8(canvas);
    encode_image_with_target(&final_img, settings, output, ffmpeg_path)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroup {
    pub keeper: String,
    pub duplicates: Vec<String>,
}

/// Computes a fast 64-bit perceptual difference hash (dHash) for an image.
fn compute_dhash(path: &Path, ffmpeg_path: Option<&Path>) -> Option<u64> {
    let img = decode_image(path, &ResizeMode::Original, true, ffmpeg_path).ok()?;
    let small = img
        .resize_exact(9, 8, image::imageops::FilterType::Triangle)
        .to_luma8();
    let mut hash = 0u64;
    let mut bit = 0u32;
    for y in 0..8 {
        for x in 0..8 {
            let left = small.get_pixel(x, y)[0];
            let right = small.get_pixel(x + 1, y)[0];
            if left > right {
                hash |= 1u64 << bit;
            }
            bit += 1;
        }
    }
    Some(hash)
}

/// Scans an image list in parallel via Rayon and groups exact or visually near-duplicate
/// images (Hamming distance <= `max_hamming`, recommended default 5).
pub fn find_duplicate_images(
    paths: &[PathBuf],
    max_hamming: u32,
    ffmpeg_path: Option<&Path>,
) -> Vec<DuplicateGroup> {
    use rayon::prelude::*;

    let hashed: Vec<(String, u64)> = paths
        .par_iter()
        .filter_map(|p| {
            let h = compute_dhash(p, ffmpeg_path)?;
            Some((p.to_string_lossy().into_owned(), h))
        })
        .collect();

    let mut visited = vec![false; hashed.len()];
    let mut groups = Vec::new();

    for i in 0..hashed.len() {
        if visited[i] {
            continue;
        }
        visited[i] = true;
        let (ref keeper_path, keeper_hash) = hashed[i];
        let mut dups = Vec::new();

        for j in (i + 1)..hashed.len() {
            if visited[j] {
                continue;
            }
            let dist = (keeper_hash ^ hashed[j].1).count_ones();
            if dist <= max_hamming {
                visited[j] = true;
                dups.push(hashed[j].0.clone());
            }
        }

        if !dups.is_empty() {
            groups.push(DuplicateGroup {
                keeper: keeper_path.clone(),
                duplicates: dups,
            });
        }
    }

    groups
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorSwatch {
    pub hex: String,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub percent: f32,
}

/// Extracts the top `count` dominant colors from an image using fast deterministic k-means
/// quantization on a 64x64 downsampled copy (<10ms).
pub fn extract_color_palette(
    input: &Path,
    count: usize,
    ffmpeg_path: Option<&Path>,
) -> Result<Vec<ColorSwatch>> {
    let k = count.clamp(2, 12);
    let img = decode_image(input, &ResizeMode::Original, true, ffmpeg_path)?;
    let small = img
        .resize_exact(64, 64, image::imageops::FilterType::Triangle)
        .to_rgba8();

    let pixels: Vec<[f32; 3]> = small
        .pixels()
        .filter(|p| p[3] >= 32)
        .map(|p| [p[0] as f32, p[1] as f32, p[2] as f32])
        .collect();

    if pixels.is_empty() {
        return Ok(vec![ColorSwatch {
            hex: "#ffffff".into(),
            r: 255,
            g: 255,
            b: 255,
            percent: 100.0,
        }]);
    }

    // Deterministic evenly-spaced initial centroids
    let step = (pixels.len() / k).max(1);
    let mut centroids: Vec<[f32; 3]> = (0..k)
        .map(|i| pixels[(i * step).min(pixels.len() - 1)])
        .collect();
    let mut counts = vec![0usize; k];

    for _ in 0..12 {
        let mut sums = vec![[0.0f32; 3]; k];
        counts.fill(0);

        for px in &pixels {
            let mut best_idx = 0;
            let mut best_dist = f32::MAX;
            for (ci, c) in centroids.iter().enumerate() {
                let dr = px[0] - c[0];
                let dg = px[1] - c[1];
                let db = px[2] - c[2];
                let d = dr * dr + dg * dg + db * db;
                if d < best_dist {
                    best_dist = d;
                    best_idx = ci;
                }
            }
            sums[best_idx][0] += px[0];
            sums[best_idx][1] += px[1];
            sums[best_idx][2] += px[2];
            counts[best_idx] += 1;
        }

        for ci in 0..k {
            if counts[ci] > 0 {
                let n = counts[ci] as f32;
                centroids[ci] = [sums[ci][0] / n, sums[ci][1] / n, sums[ci][2] / n];
            }
        }
    }

    let total = pixels.len() as f32;
    let mut swatches: Vec<ColorSwatch> = centroids
        .into_iter()
        .zip(counts)
        .filter(|(_, c)| *c > 0)
        .map(|(c, cnt)| {
            let r = c[0].round().clamp(0.0, 255.0) as u8;
            let g = c[1].round().clamp(0.0, 255.0) as u8;
            let b = c[2].round().clamp(0.0, 255.0) as u8;
            ColorSwatch {
                hex: format!("#{:02x}{:02x}{:02x}", r, g, b),
                r,
                g,
                b,
                percent: ((cnt as f32 / total) * 1000.0).round() / 10.0,
            }
        })
        .collect();

    swatches.sort_by(|a, b| {
        b.percent
            .partial_cmp(&a.percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(swatches)
}

/// Generates a crisp vector QR code SVG string from `content`.
pub fn generate_qr_svg(content: &str, dark_hex: &str, light_hex: &str) -> Result<String> {
    if content.trim().is_empty() {
        return Err(WandelError::InvalidSettings(
            "QR code content cannot be empty".into(),
        ));
    }
    let code = qrcode::QrCode::new(content.as_bytes())
        .map_err(|e| WandelError::ImageEncode(format!("QR code encode error: {}", e)))?;
    let dark = if dark_hex.starts_with('#') {
        dark_hex
    } else {
        "#000000"
    };
    let light = if light_hex.starts_with('#') {
        light_hex
    } else {
        "#ffffff"
    };
    let svg = code
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(512, 512)
        .dark_color(qrcode::render::svg::Color(dark))
        .light_color(qrcode::render::svg::Color(light))
        .build();
    Ok(svg)
}

/// Generates a QR code and saves it as either `.svg` or rasterized `.png` (via `resvg`).
pub fn save_qr_code(
    content: &str,
    dark_hex: &str,
    light_hex: &str,
    size_px: u32,
    output: &Path,
) -> Result<u64> {
    let svg = generate_qr_svg(content, dark_hex, light_hex)?;
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let is_svg = output
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("svg"));

    if is_svg {
        std::fs::write(output, svg.as_bytes())?;
        return Ok(svg.len() as u64);
    }

    let tree = resvg::usvg::Tree::from_str(&svg, &resvg::usvg::Options::default())
        .map_err(|e| WandelError::ImageEncode(format!("QR SVG raster error: {}", e)))?;
    let dim = size_px.clamp(128, 2048);
    let mut pixmap = resvg::tiny_skia::Pixmap::new(dim, dim)
        .ok_or_else(|| WandelError::ImageEncode("Failed to allocate QR pixmap".into()))?;
    let sx = dim as f32 / tree.size().width().max(1.0);
    let sy = dim as f32 / tree.size().height().max(1.0);
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(sx, sy),
        &mut pixmap.as_mut(),
    );
    let img = image::RgbaImage::from_raw(dim, dim, pixmap.take())
        .ok_or_else(|| WandelError::ImageEncode("Invalid QR RGBA buffer".into()))?;
    DynamicImage::ImageRgba8(img)
        .save_with_format(output, image::ImageFormat::Png)
        .map_err(|e| WandelError::ImageEncode(format!("QR PNG save error: {}", e)))?;
    Ok(std::fs::metadata(output)?.len())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataUrlExport {
    pub data_url: String,
    pub mime: String,
    pub bytes: u64,
    pub width: u32,
    pub height: u32,
}

/// Converts an image according to `settings` and returns a ready-to-paste Base64 Data URL.
pub fn export_image_data_url(
    input: &Path,
    settings: &ImageSettings,
    ffmpeg_path: Option<&Path>,
) -> Result<DataUrlExport> {
    let processed = process_image_pipeline(input, settings, ffmpeg_path)?;
    let (width, height) = processed.dimensions();

    let ext = settings.format.extension();
    let mime = match settings.format {
        ImageFormat::Png => "image/png",
        ImageFormat::Jpg => "image/jpeg",
        ImageFormat::Webp => "image/webp",
        ImageFormat::Avif => "image/avif",
        ImageFormat::Gif => "image/gif",
        ImageFormat::Bmp => "image/bmp",
        ImageFormat::Tiff => "image/tiff",
        ImageFormat::Ico => "image/x-icon",
        ImageFormat::Heic => "image/heic",
        ImageFormat::Jxl => "image/jxl",
        ImageFormat::Svg => "image/svg+xml",
    }
    .to_string();

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_path = std::env::temp_dir().join(format!(
        "wandel_dataurl_{}_{}.{}",
        std::process::id(),
        unique,
        ext
    ));

    let bytes = encode_image_with_target(&processed, settings, &temp_path, ffmpeg_path)?;
    let raw = std::fs::read(&temp_path)?;
    let _ = std::fs::remove_file(&temp_path);

    let b64 = base64::engine::general_purpose::STANDARD.encode(&raw);
    Ok(DataUrlExport {
        data_url: format!("data:{};base64,{}", mime, b64),
        mime,
        bytes,
        width,
        height,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponsiveSetResult {
    pub files: Vec<String>,
    pub html_snippet: String,
}

/// Generates a Responsive Web `srcset` bundle (320w, 640w, 1024w, 1920w — never upscaling)
/// in parallel across CPU cores and writes a ready-to-paste `snippet.html`.
pub fn generate_responsive_set(
    input: &Path,
    output_dir: &Path,
    settings: &ImageSettings,
    ffmpeg_path: Option<&Path>,
) -> Result<ResponsiveSetResult> {
    use rayon::prelude::*;

    std::fs::create_dir_all(output_dir)?;
    let base = decode_image(input, &ResizeMode::Original, true, ffmpeg_path)?;
    let orig_w = base.width().max(1);
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");
    let ext = if settings.format.can_encode() {
        settings.format.extension()
    } else {
        "webp"
    };
    let target_fmt = if settings.format.can_encode() {
        settings.format
    } else {
        ImageFormat::Webp
    };

    let mut widths: Vec<u32> = [320, 640, 1024, 1920]
        .into_iter()
        .filter(|&w| w < orig_w)
        .collect();
    widths.push(orig_w);

    let generated: Result<Vec<(u32, String, PathBuf)>> = widths
        .par_iter()
        .map(|&w| {
            let mut variant_settings = settings.clone();
            variant_settings.format = target_fmt;
            variant_settings.resize = ResizeMode::MaxDimensions {
                width: Some(w),
                height: None,
            };
            variant_settings.never_upscale = true;

            let fname = format!("{}-{}w.{}", stem, w, ext);
            let dest = output_dir.join(&fname);
            let processed =
                apply_post_decode_pipeline(base.clone(), &variant_settings, false, ffmpeg_path)?;
            let actual_w = processed.width();
            encode_image_with_target(&processed, &variant_settings, &dest, ffmpeg_path)?;
            Ok((actual_w, fname, dest))
        })
        .collect();

    let mut generated = generated?;
    generated.sort_by_key(|(w, _, _)| *w);

    let srcset_entries: Vec<String> = generated
        .iter()
        .map(|(w, fname, _)| format!("{} {}w", fname, w))
        .collect();
    let default_src = generated
        .last()
        .map(|(_, fname, _)| fname.as_str())
        .unwrap_or("");
    let html_snippet = format!(
        "<img\n  src=\"{}\"\n  srcset=\"{}\"\n  sizes=\"(max-width: 768px) 100vw, 1024px\"\n  alt=\"{}\"\n  loading=\"lazy\"\n/>",
        default_src,
        srcset_entries.join(", "),
        stem
    );

    let snippet_path = output_dir.join(format!("{}-srcset.html", stem));
    std::fs::write(&snippet_path, &html_snippet)?;

    let mut files: Vec<String> = generated
        .into_iter()
        .map(|(_, _, p)| p.to_string_lossy().into_owned())
        .collect();
    files.push(snippet_path.to_string_lossy().into_owned());

    Ok(ResponsiveSetResult {
        files,
        html_snippet,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonOptions {
    #[serde(default)]
    pub vertical: bool,
    #[serde(default = "default_compare_gap")]
    pub gap_px: u32,
    #[serde(default = "default_compare_bg")]
    pub background_hex: String,
    #[serde(default)]
    pub left_label: Option<String>,
    #[serde(default)]
    pub right_label: Option<String>,
}

fn default_compare_gap() -> u32 {
    8
}

fn default_compare_bg() -> String {
    "#18181b".into()
}

impl Default for ComparisonOptions {
    fn default() -> Self {
        Self {
            vertical: false,
            gap_px: default_compare_gap(),
            background_hex: default_compare_bg(),
            left_label: Some("Before".into()),
            right_label: Some("After".into()),
        }
    }
}

/// Generates a side-by-side or stacked Before/After comparison collage from two images.
pub fn create_comparison_image(
    first: &Path,
    second: &Path,
    output: &Path,
    opts: &ComparisonOptions,
    settings: &ImageSettings,
    ffmpeg_path: Option<&Path>,
) -> Result<u64> {
    let img1 = decode_image(first, &ResizeMode::Original, true, ffmpeg_path)?;
    let img2_raw = decode_image(second, &ResizeMode::Original, true, ffmpeg_path)?;

    let (w1, h1) = img1.dimensions();
    let (w2_raw, h2_raw) = img2_raw.dimensions();
    let gap = opts.gap_px.min(64);

    let img2 = if opts.vertical {
        if w2_raw != w1 && w2_raw > 0 {
            let scaled_h = ((f64::from(h2_raw) * f64::from(w1)) / f64::from(w2_raw))
                .round()
                .max(1.0) as u32;
            resize_image(
                img2_raw,
                &ResizeMode::Exact {
                    width: w1,
                    height: scaled_h,
                    keep_aspect_ratio: false,
                },
                false,
            )?
        } else {
            img2_raw
        }
    } else if h2_raw != h1 && h2_raw > 0 {
        let scaled_w = ((f64::from(w2_raw) * f64::from(h1)) / f64::from(h2_raw))
            .round()
            .max(1.0) as u32;
        resize_image(
            img2_raw,
            &ResizeMode::Exact {
                width: scaled_w,
                height: h1,
                keep_aspect_ratio: false,
            },
            false,
        )?
    } else {
        img2_raw
    };

    let (w2, h2) = img2.dimensions();
    let (canvas_w, canvas_h, x2, y2) = if opts.vertical {
        (w1.max(w2), h1 + gap + h2, 0u32, h1 + gap)
    } else {
        (w1 + gap + w2, h1.max(h2), w1 + gap, 0u32)
    };

    let [bg_r, bg_g, bg_b] = parse_hex_rgb(&opts.background_hex);
    let mut canvas =
        image::RgbaImage::from_pixel(canvas_w, canvas_h, image::Rgba([bg_r, bg_g, bg_b, 255]));
    image::imageops::overlay(&mut canvas, &img1.to_rgba8(), 0, 0);
    image::imageops::overlay(&mut canvas, &img2.to_rgba8(), i64::from(x2), i64::from(y2));

    let draw_badge = |canvas: &mut image::RgbaImage, text: &str, bx: u32, by: u32| {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }
        let escaped = trimmed
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;");
        let font_px = ((canvas_w.min(canvas_h) as f32) * 0.045).clamp(14.0, 32.0);
        let pad_x = font_px * 0.65;
        let pad_y = font_px * 0.35;
        let pill_w = ((trimmed.chars().count() as f32) * font_px * 0.62 + pad_x * 2.0)
            .ceil()
            .min((canvas_w / 2).max(60) as f32) as u32;
        let pill_h = (font_px + pad_y * 2.0).ceil() as u32;
        let svg = format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}">
                <rect x="0" y="0" width="{w}" height="{h}" rx="{rx:.1}" fill="#09090b" fill-opacity="0.78"/>
                <text x="{tx:.1}" y="{ty:.1}" text-anchor="middle" font-family="sans-serif" font-weight="700" font-size="{fs:.1}" fill="#ffffff">{text}</text>
            </svg>"##,
            w = pill_w,
            h = pill_h,
            rx = (pill_h as f32) * 0.35,
            tx = pill_w as f32 / 2.0,
            ty = pad_y + font_px * 0.82,
            fs = font_px,
            text = escaped,
        );
        let mut opt = resvg::usvg::Options::default();
        opt.fontdb_mut().load_system_fonts();
        if let Ok(tree) = resvg::usvg::Tree::from_str(&svg, &opt) {
            if let Some(mut pixmap) = resvg::tiny_skia::Pixmap::new(pill_w, pill_h) {
                resvg::render(&tree, resvg::tiny_skia::Transform::identity(), &mut pixmap.as_mut());
                let raw = pixmap.take();
                if let Some(badge) = image::RgbaImage::from_raw(pill_w, pill_h, raw) {
                    image::imageops::overlay(canvas, &badge, i64::from(bx), i64::from(by));
                }
            }
        }
    };

    let inset = (canvas_w.min(canvas_h) / 36).clamp(8, 24);
    if let Some(ref lbl) = opts.left_label {
        draw_badge(&mut canvas, lbl, inset, inset);
    }
    if let Some(ref lbl) = opts.right_label {
        draw_badge(&mut canvas, lbl, x2 + inset, y2 + inset);
    }

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let final_img = DynamicImage::ImageRgba8(canvas);
    encode_image_with_target(&final_img, settings, output, ffmpeg_path)
}

/// Slices an image into an equal `cols x rows` grid of tiles (e.g. 3x1 carousel or 3x3 grid)
/// in parallel across CPU cores.
pub fn split_image_grid(
    input: &Path,
    output_dir: &Path,
    cols: u32,
    rows: u32,
    settings: &ImageSettings,
    ffmpeg_path: Option<&Path>,
) -> Result<Vec<String>> {
    use rayon::prelude::*;

    let c = cols.clamp(1, 12);
    let r = rows.clamp(1, 12);
    if c == 1 && r == 1 {
        return Err(WandelError::InvalidSettings(
            "Grid slice requires at least 2 tiles (cols > 1 or rows > 1)".into(),
        ));
    }

    let processed = process_image_pipeline(input, settings, ffmpeg_path)?;
    let (w, h) = processed.dimensions();
    let tile_w = (w / c).max(1);
    let tile_h = (h / r).max(1);

    std::fs::create_dir_all(output_dir)?;
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("tile");
    let target_fmt = if settings.format.can_encode() {
        settings.format
    } else {
        ImageFormat::Png
    };
    let ext = target_fmt.extension();
    let mut tile_settings = settings.clone();
    tile_settings.format = target_fmt;

    let coords: Vec<(u32, u32)> = (0..r)
        .flat_map(|row| (0..c).map(move |col| (row, col)))
        .collect();

    let results: Result<Vec<String>> = coords
        .par_iter()
        .map(|&(row, col)| {
            let x = col * tile_w;
            let y = row * tile_h;
            let cw = if col == c - 1 { w - x } else { tile_w };
            let ch = if row == r - 1 { h - y } else { tile_h };
            let tile = processed.crop_imm(x, y, cw.max(1), ch.max(1));
            let dest = output_dir.join(format!("{}_r{}_c{}.{}", stem, row + 1, col + 1, ext));
            encode_image_with_target(&tile, &tile_settings, &dest, ffmpeg_path)?;
            Ok(dest.to_string_lossy().into_owned())
        })
        .collect();

    let mut files = results?;
    files.sort();
    Ok(files)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteSheetResult {
    pub image_path: String,
    pub css_path: String,
    pub css_snippet: String,
    pub width: u32,
    pub height: u32,
    pub count: usize,
}

/// Packs 2+ images into a compact shelf-packed transparent sprite sheet and writes a matching `.css` map.
pub fn create_sprite_sheet(
    inputs: &[PathBuf],
    output: &Path,
    padding_px: u32,
    settings: &ImageSettings,
    ffmpeg_path: Option<&Path>,
) -> Result<SpriteSheetResult> {
    use rayon::prelude::*;

    if inputs.len() < 2 {
        return Err(WandelError::InvalidSettings(
            "At least 2 images are required to create a sprite sheet".into(),
        ));
    }

    let pad = padding_px.min(32);

    let decoded: Result<Vec<(String, image::RgbaImage)>> = inputs
        .par_iter()
        .map(|p| {
            let img = decode_image(p, &ResizeMode::Original, true, ffmpeg_path)?.to_rgba8();
            let raw_name = p
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("sprite");
            let clean: String = raw_name
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
                .collect();
            Ok((clean.trim_matches('-').to_string(), img))
        })
        .collect();
    let mut items = decoded?;
    // Sort tallest first for compact shelf packing
    items.sort_by_key(|b| std::cmp::Reverse(b.1.height()));

    let total_area: u64 = items
        .iter()
        .map(|(_, im)| u64::from(im.width() + pad) * u64::from(im.height() + pad))
        .sum();
    let max_item_w = items.iter().map(|(_, im)| im.width()).max().unwrap_or(1);
    let target_row_w = ((total_area as f64 * 1.15).sqrt().ceil() as u32).max(max_item_w);

    let mut placements: Vec<(String, u32, u32, u32, u32, usize)> = Vec::with_capacity(items.len());
    let mut cur_x = 0u32;
    let mut cur_y = 0u32;
    let mut shelf_h = 0u32;
    let mut canvas_w = 0u32;

    for (idx, (name, im)) in items.iter().enumerate() {
        let (w, h) = im.dimensions();
        if cur_x > 0 && cur_x + w > target_row_w {
            cur_x = 0;
            cur_y += shelf_h + pad;
            shelf_h = 0;
        }
        placements.push((name.clone(), cur_x, cur_y, w, h, idx));
        cur_x += w + pad;
        shelf_h = shelf_h.max(h);
        canvas_w = canvas_w.max(cur_x.saturating_sub(pad));
    }
    let canvas_h = (cur_y + shelf_h).max(1);
    canvas_w = canvas_w.max(1);

    let mut canvas = image::RgbaImage::from_pixel(canvas_w, canvas_h, image::Rgba([0, 0, 0, 0]));
    let file_name = output
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("sprites.png");

    let mut css = format!(
        ".sprite {{\n  background-image: url('{}');\n  background-repeat: no-repeat;\n  display: inline-block;\n}}\n",
        file_name
    );

    for (i, (name, x, y, w, h, idx)) in placements.iter().enumerate() {
        image::imageops::overlay(&mut canvas, &items[*idx].1, i64::from(*x), i64::from(*y));
        let cls = if name.is_empty() {
            format!("item-{}", i + 1)
        } else {
            name.clone()
        };
        css.push_str(&format!(
            ".sprite-{} {{ width: {}px; height: {}px; background-position: -{}px -{}px; }}\n",
            cls, w, h, x, y
        ));
    }

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut out_settings = settings.clone();
    if !out_settings.format.can_encode() {
        out_settings.format = ImageFormat::Png;
    }
    let sheet_img = DynamicImage::ImageRgba8(canvas);
    encode_image_with_target(&sheet_img, &out_settings, output, ffmpeg_path)?;

    let css_path = output.with_extension("css");
    std::fs::write(&css_path, &css)?;

    Ok(SpriteSheetResult {
        image_path: output.to_string_lossy().into_owned(),
        css_path: css_path.to_string_lossy().into_owned(),
        css_snippet: css,
        width: canvas_w,
        height: canvas_h,
        count: items.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};
    use tempfile::tempdir;

    #[test]
    fn converts_and_resizes_png_to_webp_jpg_avif() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("sample.png");

        // Create a 120x80 gradient PNG
        let img = ImageBuffer::from_fn(120, 80, |x, y| {
            Rgba([(x * 2) as u8, (y * 3) as u8, 180, 255])
        });
        DynamicImage::ImageRgba8(img).save(&src).unwrap();

        for (fmt, ext) in [
            (ImageFormat::Webp, "webp"),
            (ImageFormat::Jpg, "jpg"),
            (ImageFormat::Avif, "avif"),
            (ImageFormat::Png, "out.png"),
        ] {
            let dst = dir.path().join(format!("out.{}", ext));
            let settings = ImageSettings {
                format: fmt,
                quality: 80,
                lossless: false,
                resize: ResizeMode::MaxDimensions {
                    width: Some(60),
                    height: None,
                },
                never_upscale: true,
                strip_metadata: true,
                ..Default::default()
            };
            let bytes = convert_image(&src, &dst, &settings, None).unwrap();
            assert!(bytes > 0);
            assert!(dst.exists());
            if fmt != ImageFormat::Avif {
                let (w, h) = image::image_dimensions(&dst).unwrap();
                assert_eq!((w, h), (60, 40));
            }
        }
    }
}
