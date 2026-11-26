use crate::error::{Result, WandelError};
use crate::image_engine::{
    apply_post_decode_pipeline, encode_image_with_target, make_thumbnail_data_url,
    process_image_pipeline,
};
use crate::naming::{NamingContext, OutputPathAllocator};
use crate::settings::{ImageSettings, OutputSettings, PdfPageFit, PdfSettings};
use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_syntax::Pdf as HayroPdf;
use hayro::vello_cpu::color::palette::css::WHITE;
use hayro::{PixmapSettings, RenderCache, RenderSettings};
use image::{DynamicImage, RgbaImage};
use pdf_writer::{Content, Filter, Finish, Name, Pdf, Rect, Ref};
use rayon::prelude::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

struct PreparedPdfPage {
    jpeg_bytes: Vec<u8>,
    width_px: u32,
    height_px: u32,
}

/// Merges an ordered list of images into a single multi-page PDF document in parallel.
/// Each image goes through the configured transforms, resize, grayscale/sharpen, and watermarks.
pub fn merge_images_to_pdf(
    inputs: &[PathBuf],
    output: &Path,
    settings: &ImageSettings,
    page_fit: PdfPageFit,
    ffmpeg_path: Option<&Path>,
) -> Result<u64> {
    if inputs.is_empty() {
        return Err(WandelError::InvalidSettings(
            "No images provided for PDF merge".into(),
        ));
    }

    let quality = settings.quality.clamp(20, 100);

    // Decode, transform, resize, and JPEG-encode all pages in parallel across all CPU cores
    let pages: Result<Vec<PreparedPdfPage>> = inputs
        .par_iter()
        .map(|path| {
            let processed = process_image_pipeline(path, settings, ffmpeg_path)?;
            let rgb = processed.to_rgb8();
            let (w, h) = (rgb.width(), rgb.height());
            let mut jpeg_bytes = Vec::new();
            let mut enc = jpeg_encoder::Encoder::new(&mut jpeg_bytes, quality);
            enc.set_progressive(false); // Baseline JPEG is universally supported in PDF viewers
            enc.encode(
                rgb.as_raw(),
                w as u16,
                h as u16,
                jpeg_encoder::ColorType::Rgb,
            )
            .map_err(|e| WandelError::ImageEncode(format!("PDF JPEG encode error: {}", e)))?;
            Ok(PreparedPdfPage {
                jpeg_bytes,
                width_px: w,
                height_px: h,
            })
        })
        .collect();

    let pages = pages?;

    let mut pdf = Pdf::new();
    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);

    let mut next_id = 3i32;
    let mut page_ids = Vec::with_capacity(pages.len());

    // Allocate 3 object IDs per page: Page, Image XObject, Content Stream
    let mut page_triples = Vec::with_capacity(pages.len());
    for _ in &pages {
        let p_id = Ref::new(next_id);
        let img_id = Ref::new(next_id + 1);
        let content_id = Ref::new(next_id + 2);
        next_id += 3;
        page_ids.push(p_id);
        page_triples.push((p_id, img_id, content_id));
    }

    pdf.catalog(catalog_id).pages(page_tree_id);
    pdf.pages(page_tree_id)
        .kids(page_ids.iter().copied())
        .count(page_ids.len() as i32);

    for (page_data, (page_id, img_id, content_id)) in pages.iter().zip(page_triples) {
        let (page_w, page_h, draw_x, draw_y, draw_w, draw_h) = match page_fit {
            PdfPageFit::FitToImage => {
                // 72 PDF points per inch; clamp to comfortable display size
                let w = page_data.width_px as f32;
                let h = page_data.height_px as f32;
                (w, h, 0.0, 0.0, w, h)
            }
            PdfPageFit::A4 => fit_into_box(page_data.width_px, page_data.height_px, 595.28, 841.89, 24.0),
            PdfPageFit::Letter => fit_into_box(page_data.width_px, page_data.height_px, 612.0, 792.0, 24.0),
        };

        let mut page = pdf.page(page_id);
        page.media_box(Rect::new(0.0, 0.0, page_w, page_h));
        page.parent(page_tree_id);
        page.contents(content_id);
        page.resources().x_objects().pair(Name(b"Im0"), img_id);
        page.finish();

        let mut img_xobj = pdf.image_xobject(img_id, &page_data.jpeg_bytes);
        img_xobj.filter(Filter::DctDecode);
        img_xobj.width(page_data.width_px as i32);
        img_xobj.height(page_data.height_px as i32);
        img_xobj.color_space().device_rgb();
        img_xobj.bits_per_component(8);
        img_xobj.finish();

        let mut content = Content::new();
        content.save_state();
        content.transform([draw_w, 0.0, 0.0, draw_h, draw_x, draw_y]);
        content.x_object(Name(b"Im0"));
        content.restore_state();
        pdf.stream(content_id, &content.finish());
    }

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let pdf_bytes = pdf.finish();
    std::fs::write(output, &pdf_bytes)?;
    Ok(pdf_bytes.len() as u64)
}

fn fit_into_box(
    img_w: u32,
    img_h: u32,
    page_w: f32,
    page_h: f32,
    margin: f32,
) -> (f32, f32, f32, f32, f32, f32) {
    let avail_w = (page_w - margin * 2.0).max(1.0);
    let avail_h = (page_h - margin * 2.0).max(1.0);
    let scale = (avail_w / img_w.max(1) as f32).min(avail_h / img_h.max(1) as f32);
    let draw_w = img_w as f32 * scale;
    let draw_h = img_h as f32 * scale;
    let draw_x = (page_w - draw_w) / 2.0;
    let draw_y = (page_h - draw_h) / 2.0;
    (page_w, page_h, draw_x, draw_y, draw_w, draw_h)
}

/// Parses a 1-based page range spec like `"1-3,5,7-9"` into sorted, deduplicated 0-based indices.
/// Empty or invalid specs default to all pages (`0..total_pages`).
pub fn parse_page_range(spec: &str, total_pages: usize) -> Vec<usize> {
    if total_pages == 0 {
        return Vec::new();
    }
    let trimmed = spec.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("all") {
        return (0..total_pages).collect();
    }

    let mut set = BTreeSet::new();
    for part in trimmed.split(',') {
        let p = part.trim();
        if let Some((start_s, end_s)) = p.split_once('-') {
            if let (Ok(s), Ok(e)) = (start_s.trim().parse::<usize>(), end_s.trim().parse::<usize>())
            {
                let lo = s.max(1).min(total_pages);
                let hi = e.max(lo).min(total_pages);
                for idx in lo..=hi {
                    set.insert(idx - 1);
                }
            }
        } else if let Ok(page_num) = p.parse::<usize>() {
            if page_num >= 1 && page_num <= total_pages {
                set.insert(page_num - 1);
            }
        }
    }

    if set.is_empty() {
        (0..total_pages).collect()
    } else {
        set.into_iter().collect()
    }
}

fn pixmap_to_dynamic_image(pixmap: hayro::vello_cpu::Pixmap) -> Result<DynamicImage> {
    let w = u32::from(pixmap.width());
    let h = u32::from(pixmap.height());
    // Rendered onto opaque WHITE background, so premultiplied RGBA == straight RGBA
    let raw = pixmap.data_as_u8_slice().to_vec();
    let img = RgbaImage::from_raw(w, h, raw)
        .ok_or_else(|| WandelError::ImageDecode("Invalid PDF rendered pixmap dimensions".into()))?;
    Ok(DynamicImage::ImageRgba8(img))
}

/// Fast PDF inspection: returns `(page_count, first_page_w, first_page_h, thumbnail_data_url)`.
pub fn inspect_pdf(path: &Path) -> (Option<u32>, Option<u32>, Option<u32>, Option<String>) {
    let Ok(bytes) = std::fs::read(path) else {
        return (None, None, None, None);
    };
    let Ok(pdf) = HayroPdf::new(Arc::new(bytes)) else {
        return (None, None, None, None);
    };

    let pages = pdf.pages();
    let page_count = pages.len() as u32;
    let Some(first) = pages.first() else {
        return (Some(0), None, None, None);
    };

    let (pw, ph) = first.render_dimensions();
    let thumb_scale = (96.0 / pw.max(ph).max(1.0)).clamp(0.05, 1.0);
    let cache = RenderCache::new();
    let pixmap = hayro::render(
        first,
        &cache,
        &InterpreterSettings::default(),
        &RenderSettings::default(),
        &PixmapSettings {
            x_scale: thumb_scale,
            y_scale: thumb_scale,
            bg_color: WHITE,
        },
    );
    let thumb = pixmap_to_dynamic_image(pixmap)
        .ok()
        .and_then(|img| make_thumbnail_data_url(&img));

    (
        Some(page_count),
        Some(pw.round() as u32),
        Some(ph.round() as u32),
        thumb,
    )
}

/// Rasterizes selected pages of a PDF in parallel using pure-Rust `hayro` and encodes
/// each page using Wandel's full `ImageSettings` pipeline (PNG, JPG, WebP, AVIF, etc.).
/// Returns `(first_output_path, total_output_bytes)`.
#[allow(clippy::too_many_arguments)]
pub fn convert_pdf_to_images<F>(
    input: &Path,
    pdf_settings: &PdfSettings,
    image_settings: &ImageSettings,
    output_settings: &OutputSettings,
    allocator: &OutputPathAllocator,
    relative_dir: Option<&str>,
    ffmpeg_path: Option<&Path>,
    cancel_flag: &AtomicBool,
    on_progress: F,
) -> Result<(PathBuf, u64)>
where
    F: Fn(f32) + Send + Sync,
{
    if !image_settings.format.can_encode() {
        return Err(WandelError::UnsupportedFormat(format!(
            "Cannot encode PDF pages to {}",
            image_settings.format.display_name()
        )));
    }

    let bytes = Arc::new(std::fs::read(input)?);
    let pdf = HayroPdf::new(Arc::clone(&bytes))
        .map_err(|e| WandelError::ImageDecode(format!("Failed to parse PDF: {:?}", e)))?;

    let total_pages = pdf.pages().len();
    if total_pages == 0 {
        return Err(WandelError::ImageDecode("PDF contains no pages".into()));
    }

    let page_indices = parse_page_range(&pdf_settings.page_range, total_pages);
    let selected_count = page_indices.len();
    let scale = (f32::from(pdf_settings.dpi.clamp(72, 600))) / 72.0;
    let ext = image_settings.format.extension();
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("document");

    let completed_pages = AtomicUsize::new(0);

    // Render and encode each page in parallel across CPU cores
    let results: Result<Vec<(PathBuf, u64)>> = page_indices
        .par_iter()
        .map(|&page_idx| {
            if cancel_flag.load(Ordering::Relaxed) {
                return Err(WandelError::Cancelled);
            }

            // Parse lightweight view per worker thread (zero-copy Arc<Vec<u8>>)
            let thread_pdf = HayroPdf::new(Arc::clone(&bytes))
                .map_err(|e| WandelError::ImageDecode(format!("PDF parse error: {:?}", e)))?;
            let page = thread_pdf
                .pages()
                .get(page_idx)
                .ok_or_else(|| WandelError::ImageDecode("PDF page index out of bounds".into()))?;

            let cache = RenderCache::new();
            let pixmap = hayro::render(
                page,
                &cache,
                &InterpreterSettings::default(),
                &RenderSettings::default(),
                &PixmapSettings {
                    x_scale: scale,
                    y_scale: scale,
                    bg_color: WHITE,
                },
            );

            let rendered_img = pixmap_to_dynamic_image(pixmap)?;
            let processed =
                apply_post_decode_pipeline(rendered_img, image_settings, false, ffmpeg_path)?;

            let mut page_output_settings = output_settings.clone();
            if page_output_settings.rename_template.trim().is_empty() {
                page_output_settings.rename_template =
                    format!("{}_page_{:03}", stem, page_idx + 1);
            }
            let ctx = NamingContext {
                index: page_idx,
                width: Some(processed.width()),
                height: Some(processed.height()),
                relative_dir,
            };
            let dest = allocator.allocate_with_context(input, ext, &page_output_settings, &ctx)?;

            let out_bytes = encode_image_with_target(&processed, image_settings, &dest, ffmpeg_path)?;

            let done = completed_pages.fetch_add(1, Ordering::Relaxed) + 1;
            on_progress((done as f32 / selected_count as f32) * 100.0);

            Ok((dest, out_bytes))
        })
        .collect();

    let outputs = results?;
    let first_path = outputs
        .first()
        .map(|(p, _)| p.clone())
        .unwrap_or_else(|| input.to_path_buf());
    let total_bytes: u64 = outputs.iter().map(|(_, b)| *b).sum();
    Ok((first_path, total_bytes))
}

/// Merges multiple existing PDF files losslessly into a single PDF document (pure Rust via `lopdf`).
pub fn merge_pdfs(inputs: &[PathBuf], output: &Path) -> Result<u64> {
    use lopdf::{Document, Object, ObjectId};
    use std::collections::BTreeMap;

    if inputs.len() < 2 {
        return Err(WandelError::InvalidSettings(
            "At least 2 PDF files are required to merge".into(),
        ));
    }

    let mut max_id = 1u32;
    let mut documents_pages: BTreeMap<ObjectId, Object> = BTreeMap::new();
    let mut documents_objects: BTreeMap<ObjectId, Object> = BTreeMap::new();
    let mut merged = Document::with_version("1.7");

    for path in inputs {
        let mut doc = Document::load(path)
            .map_err(|e| WandelError::ImageDecode(format!("Failed to load PDF {}: {}", path.display(), e)))?;
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;

        for (_, object_id) in doc.get_pages() {
            if let Ok(obj) = doc.get_object(object_id) {
                documents_pages.insert(object_id, obj.to_owned());
            }
        }
        documents_objects.extend(doc.objects);
    }

    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    for (object_id, object) in documents_objects {
        match object.type_name().unwrap_or(b"") {
            b"Catalog" => {
                if catalog_object.is_none() {
                    catalog_object = Some((object_id, object));
                }
            }
            b"Pages" => {
                if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();
                    if let Some((_, ref existing)) = pages_object {
                        if let Ok(old_dict) = existing.as_dict() {
                            dictionary.extend(old_dict);
                        }
                    }
                    pages_object = Some((
                        pages_object.map(|(id, _)| id).unwrap_or(object_id),
                        Object::Dictionary(dictionary),
                    ));
                }
            }
            b"Page" | b"Outlines" | b"Outline" => {}
            _ => {
                merged.objects.insert(object_id, object);
            }
        }
    }

    let (pages_id, pages_obj) = pages_object
        .ok_or_else(|| WandelError::ImageDecode("No Pages root found in input PDFs".into()))?;
    let (catalog_id, catalog_obj) = catalog_object
        .ok_or_else(|| WandelError::ImageDecode("No Catalog root found in input PDFs".into()))?;

    for (page_id, mut page_obj) in documents_pages.clone() {
        if let Ok(dict) = page_obj.as_dict_mut() {
            dict.set("Parent", pages_id);
        }
        merged.objects.insert(page_id, page_obj);
    }

    if let Ok(mut pages_dict) = pages_obj.as_dict().cloned() {
        pages_dict.set("Count", documents_pages.len() as u32);
        pages_dict.set(
            "Kids",
            documents_pages
                .keys()
                .map(|&id| Object::Reference(id))
                .collect::<Vec<_>>(),
        );
        merged.objects.insert(pages_id, Object::Dictionary(pages_dict));
    }

    if let Ok(mut catalog_dict) = catalog_obj.as_dict().cloned() {
        catalog_dict.set("Pages", pages_id);
        catalog_dict.remove(b"Outlines");
        merged
            .objects
            .insert(catalog_id, Object::Dictionary(catalog_dict));
    }

    merged.trailer.set("Root", catalog_id);
    merged.max_id = merged.objects.len() as u32;
    merged.renumber_objects();
    merged.adjust_zero_pages();

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    merged
        .save(output)
        .map_err(|e| WandelError::ImageEncode(format!("Failed to save merged PDF: {}", e)))?;
    let size = std::fs::metadata(output)?.len();
    Ok(size)
}

/// Extracts a 1-based page range (e.g. `"1-3,5"`) from `input` into a standalone PDF without rasterizing.
pub fn split_pdf(input: &Path, page_range: &str, output: &Path) -> Result<u64> {
    let mut doc = lopdf::Document::load(input)
        .map_err(|e| WandelError::ImageDecode(format!("Failed to load PDF: {}", e)))?;
    let total_pages = doc.get_pages().len();
    if total_pages == 0 {
        return Err(WandelError::ImageDecode("PDF has no pages".into()));
    }

    let keep_zero_based: BTreeSet<usize> = parse_page_range(page_range, total_pages)
        .into_iter()
        .collect();
    let delete_one_based: Vec<u32> = (1..=total_pages)
        .filter(|p| !keep_zero_based.contains(&(p - 1)))
        .map(|p| p as u32)
        .collect();

    if !delete_one_based.is_empty() {
        doc.delete_pages(&delete_one_based);
    }
    doc.prune_objects();

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    doc.save(output)
        .map_err(|e| WandelError::ImageEncode(format!("Failed to save split PDF: {}", e)))?;
    let size = std::fs::metadata(output)?.len();
    Ok(size)
}

/// Compresses a bloated/scanned PDF by rasterizing pages in parallel via `hayro` at `dpi`
/// and re-embedding them as compact baseline JPEGs at `quality` (1..=100) via `pdf-writer`.
pub fn compress_pdf(input: &Path, output: &Path, dpi: u16, quality: u8) -> Result<u64> {
    let bytes = Arc::new(std::fs::read(input)?);
    let pdf = HayroPdf::new(Arc::clone(&bytes))
        .map_err(|e| WandelError::ImageDecode(format!("Failed to parse PDF: {:?}", e)))?;
    let total_pages = pdf.pages().len();
    if total_pages == 0 {
        return Err(WandelError::ImageDecode("PDF contains no pages".into()));
    }

    let scale = f32::from(dpi.clamp(72, 300)) / 72.0;
    let q = quality.clamp(25, 95);

    let pages: Result<Vec<PreparedPdfPage>> = (0..total_pages)
        .into_par_iter()
        .map(|idx| {
            let thread_pdf = HayroPdf::new(Arc::clone(&bytes))
                .map_err(|e| WandelError::ImageDecode(format!("PDF parse error: {:?}", e)))?;
            let page = thread_pdf
                .pages()
                .get(idx)
                .ok_or_else(|| WandelError::ImageDecode("Page index out of bounds".into()))?;
            let cache = RenderCache::new();
            let pixmap = hayro::render(
                page,
                &cache,
                &InterpreterSettings::default(),
                &RenderSettings::default(),
                &PixmapSettings {
                    x_scale: scale,
                    y_scale: scale,
                    bg_color: WHITE,
                },
            );
            let img = pixmap_to_dynamic_image(pixmap)?;
            let rgb = img.to_rgb8();
            let (w, h) = (rgb.width(), rgb.height());
            let mut jpeg_bytes = Vec::new();
            let mut enc = jpeg_encoder::Encoder::new(&mut jpeg_bytes, q);
            enc.set_progressive(false);
            enc.encode(
                rgb.as_raw(),
                w as u16,
                h as u16,
                jpeg_encoder::ColorType::Rgb,
            )
            .map_err(|e| WandelError::ImageEncode(format!("PDF compress JPEG error: {}", e)))?;
            Ok(PreparedPdfPage {
                jpeg_bytes,
                width_px: w,
                height_px: h,
            })
        })
        .collect();

    let pages = pages?;

    let mut out_pdf = Pdf::new();
    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);
    let mut next_id = 3i32;
    let mut page_ids = Vec::with_capacity(pages.len());
    let mut page_triples = Vec::with_capacity(pages.len());

    for _ in &pages {
        let p_id = Ref::new(next_id);
        let img_id = Ref::new(next_id + 1);
        let content_id = Ref::new(next_id + 2);
        next_id += 3;
        page_ids.push(p_id);
        page_triples.push((p_id, img_id, content_id));
    }

    out_pdf.catalog(catalog_id).pages(page_tree_id);
    out_pdf
        .pages(page_tree_id)
        .kids(page_ids.iter().copied())
        .count(page_ids.len() as i32);

    for (page_data, (page_id, img_id, content_id)) in pages.iter().zip(page_triples) {
        let w = (page_data.width_px as f32) / scale;
        let h = (page_data.height_px as f32) / scale;
        let mut page = out_pdf.page(page_id);
        page.media_box(Rect::new(0.0, 0.0, w, h));
        page.parent(page_tree_id);
        page.contents(content_id);
        page.resources().x_objects().pair(Name(b"Im0"), img_id);
        page.finish();

        let mut img_xobj = out_pdf.image_xobject(img_id, &page_data.jpeg_bytes);
        img_xobj.filter(Filter::DctDecode);
        img_xobj.width(page_data.width_px as i32);
        img_xobj.height(page_data.height_px as i32);
        img_xobj.color_space().device_rgb();
        img_xobj.bits_per_component(8);
        img_xobj.finish();

        let mut content = Content::new();
        content.save_state();
        content.transform([w, 0.0, 0.0, h, 0.0, 0.0]);
        content.x_object(Name(b"Im0"));
        content.restore_state();
        out_pdf.stream(content_id, &content.finish());
    }

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let final_bytes = out_pdf.finish();
    std::fs::write(output, &final_bytes)?;
    Ok(final_bytes.len() as u64)
}

/// Parses an ordered 1-based page specification like `"3,1-2,5"` into deduplicated 0-based indices
/// preserving the exact order requested by the user.
pub fn parse_ordered_page_spec(spec: &str, total_pages: usize) -> Vec<usize> {
    if total_pages == 0 {
        return Vec::new();
    }
    let trimmed = spec.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("all") {
        return (0..total_pages).collect();
    }

    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    let mut push_page = |one_based: usize| {
        if one_based >= 1 && one_based <= total_pages {
            let idx = one_based - 1;
            if seen.insert(idx) {
                out.push(idx);
            }
        }
    };

    for part in trimmed.split(',') {
        let p = part.trim();
        if let Some((start_s, end_s)) = p.split_once('-') {
            if let (Ok(s), Ok(e)) = (start_s.trim().parse::<usize>(), end_s.trim().parse::<usize>())
            {
                let s_clamped = s.clamp(1, total_pages);
                let e_clamped = e.clamp(1, total_pages);
                if s_clamped <= e_clamped {
                    for pg in s_clamped..=e_clamped {
                        push_page(pg);
                    }
                } else {
                    for pg in (e_clamped..=s_clamped).rev() {
                        push_page(pg);
                    }
                }
            }
        } else if let Ok(pg) = p.parse::<usize>() {
            push_page(pg);
        }
    }

    out
}

/// Losslessly rotates pages of an existing PDF clockwise by `degrees` (90, 180, or 270).
/// If `page_spec` is `None` or empty, rotates all pages.
pub fn rotate_pdf_pages(
    input: &Path,
    output: &Path,
    degrees: u16,
    page_spec: Option<&str>,
) -> Result<u64> {
    let delta = i64::from(degrees % 360);
    if !matches!(delta, 0 | 90 | 180 | 270) {
        return Err(WandelError::InvalidSettings(
            "PDF rotation degrees must be a multiple of 90 (90, 180, or 270)".into(),
        ));
    }

    let mut doc = lopdf::Document::load(input)
        .map_err(|e| WandelError::ImageDecode(format!("Failed to load PDF: {}", e)))?;
    let pages = doc.get_pages();
    let total_pages = pages.len();
    if total_pages == 0 {
        return Err(WandelError::ImageDecode("PDF has no pages".into()));
    }

    let target_indices = match page_spec {
        Some(spec) if !spec.trim().is_empty() => parse_page_range(spec, total_pages),
        _ => (0..total_pages).collect(),
    };

    for idx in target_indices {
        let page_num = (idx + 1) as u32;
        if let Some(&obj_id) = pages.get(&page_num) {
            if let Ok(obj) = doc.get_object_mut(obj_id) {
                if let Ok(dict) = obj.as_dict_mut() {
                    let current = dict
                        .get(b"Rotate")
                        .ok()
                        .and_then(|o| o.as_i64().ok())
                        .unwrap_or(0);
                    let next_rot = (current + delta).rem_euclid(360);
                    dict.set("Rotate", lopdf::Object::Integer(next_rot));
                }
            }
        }
    }

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    doc.save(output)
        .map_err(|e| WandelError::ImageEncode(format!("Failed to save rotated PDF: {}", e)))?;
    let size = std::fs::metadata(output)?.len();
    Ok(size)
}

/// Losslessly reorders or extracts pages of a PDF according to a 1-based ordered spec (e.g. `"3,1-2"`).
pub fn reorder_pdf_pages(input: &Path, output: &Path, page_spec: &str) -> Result<u64> {
    use lopdf::Object;

    let mut doc = lopdf::Document::load(input)
        .map_err(|e| WandelError::ImageDecode(format!("Failed to load PDF: {}", e)))?;
    let pages = doc.get_pages();
    let total_pages = pages.len();
    if total_pages == 0 {
        return Err(WandelError::ImageDecode("PDF has no pages".into()));
    }

    let ordered_zero_based = parse_ordered_page_spec(page_spec, total_pages);
    if ordered_zero_based.is_empty() {
        return Err(WandelError::InvalidSettings(format!(
            "Page specification '{}' matched no valid pages (1..={})",
            page_spec, total_pages
        )));
    }

    let kept_ids: Vec<lopdf::ObjectId> = ordered_zero_based
        .iter()
        .filter_map(|&idx| pages.get(&((idx + 1) as u32)).copied())
        .collect();

    // Locate the root /Pages dictionary from the catalog
    let root_pages_id = doc
        .trailer
        .get(b"Root")
        .ok()
        .and_then(|o| o.as_reference().ok())
        .and_then(|cat_id| doc.get_object(cat_id).ok())
        .and_then(|cat| cat.as_dict().ok())
        .and_then(|cat_dict| cat_dict.get(b"Pages").ok())
        .and_then(|o| o.as_reference().ok())
        .ok_or_else(|| WandelError::ImageDecode("PDF catalog missing /Pages reference".into()))?;

    for &page_id in &kept_ids {
        if let Ok(obj) = doc.get_object_mut(page_id) {
            if let Ok(dict) = obj.as_dict_mut() {
                dict.set("Parent", Object::Reference(root_pages_id));
            }
        }
    }

    if let Ok(pages_obj) = doc.get_object_mut(root_pages_id) {
        if let Ok(pages_dict) = pages_obj.as_dict_mut() {
            pages_dict.set("Count", Object::Integer(kept_ids.len() as i64));
            pages_dict.set(
                "Kids",
                Object::Array(kept_ids.iter().map(|&id| Object::Reference(id)).collect()),
            );
        }
    }

    doc.prune_objects();

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    doc.save(output)
        .map_err(|e| WandelError::ImageEncode(format!("Failed to save reordered PDF: {}", e)))?;
    let size = std::fs::metadata(output)?.len();
    Ok(size)
}

/// Losslessly stamps diagonal text (e.g. `"CONFIDENTIAL"`, `"DRAFT"`) onto every page of a PDF
/// using pure Rust `lopdf` without rasterizing pages.
pub fn stamp_pdf_text(input: &Path, output: &Path, text: &str) -> Result<u64> {
    use lopdf::{Dictionary, Object, Stream};

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(WandelError::InvalidSettings(
            "PDF stamp text cannot be empty".into(),
        ));
    }

    let mut doc = lopdf::Document::load(input)
        .map_err(|e| WandelError::ImageDecode(format!("Failed to load PDF: {}", e)))?;
    let pages = doc.get_pages();
    if pages.is_empty() {
        return Err(WandelError::ImageDecode("PDF has no pages".into()));
    }

    // Add a standard Type1 Helvetica font resource
    let mut font_dict = Dictionary::new();
    font_dict.set("Type", Object::Name(b"Font".to_vec()));
    font_dict.set("Subtype", Object::Name(b"Type1".to_vec()));
    font_dict.set("BaseFont", Object::Name(b"Helvetica-Bold".to_vec()));
    let font_id = doc.add_object(Object::Dictionary(font_dict));

    let escaped = trimmed
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)");

    for &page_id in pages.values() {
        // Read page dimensions from /MediaBox (fallback A4)
        let (pw, ph) = doc
            .get_object(page_id)
            .ok()
            .and_then(|o| o.as_dict().ok())
            .and_then(|d| d.get(b"MediaBox").ok())
            .and_then(|mb| mb.as_array().ok())
            .and_then(|arr| {
                if arr.len() >= 4 {
                    let x0 = arr[0].as_float().or_else(|_| arr[0].as_i64().map(|v| v as f32)).ok()?;
                    let y0 = arr[1].as_float().or_else(|_| arr[1].as_i64().map(|v| v as f32)).ok()?;
                    let x1 = arr[2].as_float().or_else(|_| arr[2].as_i64().map(|v| v as f32)).ok()?;
                    let y1 = arr[3].as_float().or_else(|_| arr[3].as_i64().map(|v| v as f32)).ok()?;
                    Some(((x1 - x0).abs().max(72.0), (y1 - y0).abs().max(72.0)))
                } else {
                    None
                }
            })
            .unwrap_or((595.28, 841.89));

        let font_size = (pw.min(ph) * 0.095).clamp(18.0, 72.0);
        let approx_width = (trimmed.chars().count() as f32) * font_size * 0.56;
        let offset = (approx_width * std::f32::consts::FRAC_1_SQRT_2) / 2.0;
        let cx = (pw * 0.5 - offset).max(12.0);
        let cy = (ph * 0.5 - offset).max(12.0);

        let ops = format!(
            "q\n0.65 0.18 0.18 rg\nBT\n/WandelStampFont {:.1} Tf\n0.7071 0.7071 -0.7071 0.7071 {:.1} {:.1} Tm\n({}) Tj\nET\nQ\n",
            font_size, cx, cy, escaped
        );
        let stamp_stream_id = doc.add_object(Object::Stream(Stream::new(
            Dictionary::new(),
            ops.into_bytes(),
        )));

        // Resolve or create /Resources and /Font on the page
        let existing_res_ref = doc
            .get_object(page_id)
            .ok()
            .and_then(|o| o.as_dict().ok())
            .and_then(|d| d.get(b"Resources").ok())
            .and_then(|r| r.as_reference().ok());

        if let Some(res_id) = existing_res_ref {
            let font_ref = doc
                .get_object(res_id)
                .ok()
                .and_then(|o| o.as_dict().ok())
                .and_then(|d| d.get(b"Font").ok())
                .and_then(|f| f.as_reference().ok());
            if let Some(fid) = font_ref {
                if let Ok(Ok(fdict)) = doc.get_object_mut(fid).map(|o| o.as_dict_mut()) {
                    fdict.set("WandelStampFont", Object::Reference(font_id));
                }
            } else if let Ok(Ok(rdict)) = doc.get_object_mut(res_id).map(|o| o.as_dict_mut()) {
                let mut fdict = rdict
                    .get(b"Font")
                    .ok()
                    .and_then(|f| f.as_dict().ok())
                    .cloned()
                    .unwrap_or_default();
                fdict.set("WandelStampFont", Object::Reference(font_id));
                rdict.set("Font", Object::Dictionary(fdict));
            }
        } else if let Ok(Ok(pdict)) = doc.get_object_mut(page_id).map(|o| o.as_dict_mut()) {
            let mut rdict = pdict
                .get(b"Resources")
                .ok()
                .and_then(|r| r.as_dict().ok())
                .cloned()
                .unwrap_or_default();
            let mut fdict = rdict
                .get(b"Font")
                .ok()
                .and_then(|f| f.as_dict().ok())
                .cloned()
                .unwrap_or_default();
            fdict.set("WandelStampFont", Object::Reference(font_id));
            rdict.set("Font", Object::Dictionary(fdict));
            pdict.set("Resources", Object::Dictionary(rdict));
        }

        // Append stamp_stream_id to /Contents
        if let Ok(Ok(pdict)) = doc.get_object_mut(page_id).map(|o| o.as_dict_mut()) {
            let new_contents = match pdict.get(b"Contents").ok().cloned() {
                Some(Object::Array(mut arr)) => {
                    arr.push(Object::Reference(stamp_stream_id));
                    Object::Array(arr)
                }
                Some(other) => Object::Array(vec![other, Object::Reference(stamp_stream_id)]),
                None => Object::Reference(stamp_stream_id),
            };
            pdict.set("Contents", new_contents);
        }
    }

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    doc.save(output)
        .map_err(|e| WandelError::ImageEncode(format!("Failed to save stamped PDF: {}", e)))?;
    let size = std::fs::metadata(output)?.len();
    Ok(size)
}

/// Losslessly adds centered bottom-margin page numbers to every page of a PDF using `lopdf`.
/// `format_template` supports `{n}` (1-based page number) and `{total}` (total pages),
/// defaulting to `"{n} / {total}"` if empty.
pub fn number_pdf_pages(input: &Path, output: &Path, format_template: &str) -> Result<u64> {
    use lopdf::{Dictionary, Object, Stream};

    let template = if format_template.trim().is_empty() {
        "{n} / {total}"
    } else {
        format_template.trim()
    };

    let mut doc = lopdf::Document::load(input)
        .map_err(|e| WandelError::ImageDecode(format!("Failed to load PDF: {}", e)))?;
    let pages = doc.get_pages();
    let total = pages.len();
    if total == 0 {
        return Err(WandelError::ImageDecode("PDF has no pages".into()));
    }

    let mut font_dict = Dictionary::new();
    font_dict.set("Type", Object::Name(b"Font".to_vec()));
    font_dict.set("Subtype", Object::Name(b"Type1".to_vec()));
    font_dict.set("BaseFont", Object::Name(b"Helvetica".to_vec()));
    let font_id = doc.add_object(Object::Dictionary(font_dict));

    for (page_num, &page_id) in &pages {
        let pw = doc
            .get_object(page_id)
            .ok()
            .and_then(|o| o.as_dict().ok())
            .and_then(|d| d.get(b"MediaBox").ok())
            .and_then(|mb| mb.as_array().ok())
            .and_then(|arr| {
                if arr.len() >= 4 {
                    let x0 = arr[0].as_float().or_else(|_| arr[0].as_i64().map(|v| v as f32)).ok()?;
                    let x1 = arr[2].as_float().or_else(|_| arr[2].as_i64().map(|v| v as f32)).ok()?;
                    Some((x1 - x0).abs().max(72.0))
                } else {
                    None
                }
            })
            .unwrap_or(595.28);

        let label = template
            .replace("{n}", &page_num.to_string())
            .replace("{total}", &total.to_string());
        let escaped = label
            .replace('\\', "\\\\")
            .replace('(', "\\(")
            .replace(')', "\\)");

        let font_size = 10.5f32;
        let approx_w = (label.chars().count() as f32) * font_size * 0.52;
        let x = ((pw - approx_w) / 2.0).max(12.0);
        let y = 18.0f32;

        let ops = format!(
            "q\n0.35 0.35 0.35 rg\nBT\n/WandelPageNumFont {:.1} Tf\n1 0 0 1 {:.1} {:.1} Tm\n({}) Tj\nET\nQ\n",
            font_size, x, y, escaped
        );
        let num_stream_id = doc.add_object(Object::Stream(Stream::new(
            Dictionary::new(),
            ops.into_bytes(),
        )));

        let existing_res_ref = doc
            .get_object(page_id)
            .ok()
            .and_then(|o| o.as_dict().ok())
            .and_then(|d| d.get(b"Resources").ok())
            .and_then(|r| r.as_reference().ok());

        if let Some(res_id) = existing_res_ref {
            let font_ref = doc
                .get_object(res_id)
                .ok()
                .and_then(|o| o.as_dict().ok())
                .and_then(|d| d.get(b"Font").ok())
                .and_then(|f| f.as_reference().ok());
            if let Some(fid) = font_ref {
                if let Ok(Ok(fdict)) = doc.get_object_mut(fid).map(|o| o.as_dict_mut()) {
                    fdict.set("WandelPageNumFont", Object::Reference(font_id));
                }
            } else if let Ok(Ok(rdict)) = doc.get_object_mut(res_id).map(|o| o.as_dict_mut()) {
                let mut fdict = rdict
                    .get(b"Font")
                    .ok()
                    .and_then(|f| f.as_dict().ok())
                    .cloned()
                    .unwrap_or_default();
                fdict.set("WandelPageNumFont", Object::Reference(font_id));
                rdict.set("Font", Object::Dictionary(fdict));
            }
        } else if let Ok(Ok(pdict)) = doc.get_object_mut(page_id).map(|o| o.as_dict_mut()) {
            let mut rdict = pdict
                .get(b"Resources")
                .ok()
                .and_then(|r| r.as_dict().ok())
                .cloned()
                .unwrap_or_default();
            let mut fdict = rdict
                .get(b"Font")
                .ok()
                .and_then(|f| f.as_dict().ok())
                .cloned()
                .unwrap_or_default();
            fdict.set("WandelPageNumFont", Object::Reference(font_id));
            rdict.set("Font", Object::Dictionary(fdict));
            pdict.set("Resources", Object::Dictionary(rdict));
        }

        if let Ok(Ok(pdict)) = doc.get_object_mut(page_id).map(|o| o.as_dict_mut()) {
            let new_contents = match pdict.get(b"Contents").ok().cloned() {
                Some(Object::Array(mut arr)) => {
                    arr.push(Object::Reference(num_stream_id));
                    Object::Array(arr)
                }
                Some(other) => Object::Array(vec![other, Object::Reference(num_stream_id)]),
                None => Object::Reference(num_stream_id),
            };
            pdict.set("Contents", new_contents);
        }
    }

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    doc.save(output)
        .map_err(|e| WandelError::ImageEncode(format!("Failed to save numbered PDF: {}", e)))?;
    let size = std::fs::metadata(output)?.len();
    Ok(size)
}

/// Losslessly extracts embedded raster images (JPEG `/DCTDecode` and RGB/Gray `/FlateDecode`)
/// from an existing PDF document into `output_dir` using pure Rust `lopdf`.
pub fn extract_pdf_images(input: &Path, output_dir: &Path) -> Result<Vec<String>> {
    use lopdf::Object;

    let doc = lopdf::Document::load(input)
        .map_err(|e| WandelError::ImageDecode(format!("Failed to load PDF: {}", e)))?;

    std::fs::create_dir_all(output_dir)?;
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("pdf");

    let mut extracted = Vec::new();
    let mut counter = 0usize;

    for obj in doc.objects.values() {
        let Ok(stream) = obj.as_stream() else {
            continue;
        };
        let is_image = stream
            .dict
            .get(b"Subtype")
            .ok()
            .and_then(|o| o.as_name().ok())
            .is_some_and(|n| n == b"Image");
        if !is_image {
            continue;
        }

        let is_jpeg = match stream.dict.get(b"Filter").ok() {
            Some(Object::Name(n)) => n == b"DCTDecode",
            Some(Object::Array(arr)) => arr
                .iter()
                .any(|f| matches!(f, Object::Name(n) if n == b"DCTDecode")),
            _ => false,
        };

        if is_jpeg && !stream.content.is_empty() {
            counter += 1;
            let dest = output_dir.join(format!("{}_img_{:03}.jpg", stem, counter));
            std::fs::write(&dest, &stream.content)?;
            extracted.push(dest.to_string_lossy().into_owned());
            continue;
        }

        let w = stream
            .dict
            .get(b"Width")
            .ok()
            .and_then(|o| o.as_i64().ok())
            .unwrap_or(0) as u32;
        let h = stream
            .dict
            .get(b"Height")
            .ok()
            .and_then(|o| o.as_i64().ok())
            .unwrap_or(0) as u32;
        if w == 0 || h == 0 {
            continue;
        }

        let raw = stream
            .decompressed_content()
            .unwrap_or_else(|_| stream.content.clone());
        let expected_rgb = (w as usize) * (h as usize) * 3;
        let expected_gray = (w as usize) * (h as usize);

        if raw.len() == expected_rgb {
            if let Some(rgb) = image::RgbImage::from_raw(w, h, raw) {
                counter += 1;
                let dest = output_dir.join(format!("{}_img_{:03}.png", stem, counter));
                DynamicImage::ImageRgb8(rgb)
                    .save(&dest)
                    .map_err(|e| WandelError::ImageEncode(e.to_string()))?;
                extracted.push(dest.to_string_lossy().into_owned());
            }
        } else if raw.len() == expected_gray {
            if let Some(gray) = image::GrayImage::from_raw(w, h, raw) {
                counter += 1;
                let dest = output_dir.join(format!("{}_img_{:03}.png", stem, counter));
                DynamicImage::ImageLuma8(gray)
                    .save(&dest)
                    .map_err(|e| WandelError::ImageEncode(e.to_string()))?;
                extracted.push(dest.to_string_lossy().into_owned());
            }
        }
    }

    if extracted.is_empty() {
        return Err(WandelError::ImageDecode(
            "No extractable embedded raster images found in this PDF".into(),
        ));
    }

    Ok(extracted)
}

/// Losslessly strips identifying metadata (`/Info` trailer dictionary and `/Metadata` XMP stream)
/// from a PDF without altering any page content.
pub fn scrub_pdf_metadata(input: &Path, output: &Path) -> Result<u64> {
    let mut doc = lopdf::Document::load(input)
        .map_err(|e| WandelError::ImageDecode(format!("Failed to load PDF: {}", e)))?;

    doc.trailer.remove(b"Info");

    if let Some(cat_id) = doc
        .trailer
        .get(b"Root")
        .ok()
        .and_then(|o| o.as_reference().ok())
    {
        if let Ok(Ok(cat_dict)) = doc.get_object_mut(cat_id).map(|o| o.as_dict_mut()) {
            cat_dict.remove(b"Metadata");
        }
    }

    doc.prune_objects();

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    doc.save(output)
        .map_err(|e| WandelError::ImageEncode(format!("Failed to save cleaned PDF: {}", e)))?;
    let size = std::fs::metadata(output)?.len();
    Ok(size)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPageThumb {
    pub page: usize,
    pub width: u32,
    pub height: u32,
    pub thumbnail_data_url: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPagePlan {
    /// 1-based source page number.
    pub page: usize,
    /// Additional clockwise rotation in degrees (multiple of 90).
    #[serde(default)]
    pub rotation_delta: i32,
}

/// Renders lightweight JPEG data-URL thumbnails for every page of a PDF in parallel via `hayro` + `rayon`.
pub fn render_pdf_thumbnails(input: &Path, max_edge_px: u32) -> Result<Vec<PdfPageThumb>> {
    use base64::Engine;

    let bytes = Arc::new(std::fs::read(input)?);
    let pdf = HayroPdf::new(Arc::clone(&bytes))
        .map_err(|e| WandelError::ImageDecode(format!("Failed to parse PDF: {:?}", e)))?;
    let total_pages = pdf.pages().len();
    if total_pages == 0 {
        return Err(WandelError::ImageDecode("PDF contains no pages".into()));
    }

    let max_edge = max_edge_px.clamp(80, 512) as f32;

    (0..total_pages)
        .into_par_iter()
        .map(|idx| {
            let thread_pdf = HayroPdf::new(Arc::clone(&bytes))
                .map_err(|e| WandelError::ImageDecode(format!("PDF parse error: {:?}", e)))?;
            let page = thread_pdf
                .pages()
                .get(idx)
                .ok_or_else(|| WandelError::ImageDecode("PDF page index out of bounds".into()))?;
            let (pw, ph) = page.render_dimensions();
            let scale = (max_edge / pw.max(ph).max(1.0)).clamp(0.1, 2.0);
            let cache = RenderCache::new();
            let pixmap = hayro::render(
                page,
                &cache,
                &InterpreterSettings::default(),
                &RenderSettings::default(),
                &PixmapSettings {
                    x_scale: scale,
                    y_scale: scale,
                    bg_color: WHITE,
                },
            );
            let img = pixmap_to_dynamic_image(pixmap)?;
            let rgb = img.to_rgb8();
            let (w, h) = (rgb.width(), rgb.height());
            let mut jpeg_bytes = Vec::new();
            let mut enc = jpeg_encoder::Encoder::new(&mut jpeg_bytes, 78);
            enc.set_progressive(false);
            enc.encode(
                rgb.as_raw(),
                w as u16,
                h as u16,
                jpeg_encoder::ColorType::Rgb,
            )
            .map_err(|e| WandelError::ImageEncode(format!("PDF thumb JPEG error: {}", e)))?;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&jpeg_bytes);
            Ok(PdfPageThumb {
                page: idx + 1,
                width: pw.round() as u32,
                height: ph.round() as u32,
                thumbnail_data_url: format!("data:image/jpeg;base64,{}", b64),
            })
        })
        .collect()
}

/// Losslessly reorders, duplicates, deletes, and rotates individual pages of a PDF in a single `lopdf` pass.
pub fn organize_pdf_pages(input: &Path, output: &Path, plan: &[PdfPagePlan]) -> Result<u64> {
    use lopdf::Object;

    if plan.is_empty() {
        return Err(WandelError::InvalidSettings(
            "At least one page must remain in the organized PDF".into(),
        ));
    }

    let mut doc = lopdf::Document::load(input)
        .map_err(|e| WandelError::ImageDecode(format!("Failed to load PDF: {}", e)))?;
    let pages = doc.get_pages();
    let total_pages = pages.len();
    if total_pages == 0 {
        return Err(WandelError::ImageDecode("PDF has no pages".into()));
    }

    for item in plan {
        if item.page == 0 || item.page > total_pages {
            return Err(WandelError::InvalidSettings(format!(
                "Page {} is out of range (1..={})",
                item.page, total_pages
            )));
        }
        if item.rotation_delta.rem_euclid(90) != 0 {
            return Err(WandelError::InvalidSettings(format!(
                "Rotation delta {} must be a multiple of 90",
                item.rotation_delta
            )));
        }
    }

    let root_pages_id = doc
        .trailer
        .get(b"Root")
        .ok()
        .and_then(|o| o.as_reference().ok())
        .and_then(|cat_id| doc.get_object(cat_id).ok())
        .and_then(|cat| cat.as_dict().ok())
        .and_then(|cat_dict| cat_dict.get(b"Pages").ok())
        .and_then(|o| o.as_reference().ok())
        .ok_or_else(|| WandelError::ImageDecode("PDF catalog missing /Pages reference".into()))?;

    let mut used_ids: BTreeSet<lopdf::ObjectId> = BTreeSet::new();
    let mut final_ids: Vec<lopdf::ObjectId> = Vec::with_capacity(plan.len());

    for item in plan {
        let src_id = *pages
            .get(&(item.page as u32))
            .ok_or_else(|| WandelError::ImageDecode(format!("Missing page {}", item.page)))?;
        let target_id = if used_ids.insert(src_id) {
            src_id
        } else {
            let cloned = doc
                .get_object(src_id)
                .map_err(|e| WandelError::ImageDecode(format!("Failed to clone page: {}", e)))?
                .clone();
            doc.add_object(cloned)
        };

        if let Ok(obj) = doc.get_object_mut(target_id) {
            if let Ok(dict) = obj.as_dict_mut() {
                dict.set("Parent", Object::Reference(root_pages_id));
                let delta = i64::from(item.rotation_delta.rem_euclid(360));
                if delta != 0 {
                    let current = dict
                        .get(b"Rotate")
                        .ok()
                        .and_then(|o| o.as_i64().ok())
                        .unwrap_or(0);
                    let next_rot = (current + delta).rem_euclid(360);
                    dict.set("Rotate", Object::Integer(next_rot));
                }
            }
        }
        final_ids.push(target_id);
    }

    if let Ok(pages_obj) = doc.get_object_mut(root_pages_id) {
        if let Ok(pages_dict) = pages_obj.as_dict_mut() {
            pages_dict.set("Count", Object::Integer(final_ids.len() as i64));
            pages_dict.set(
                "Kids",
                Object::Array(final_ids.iter().map(|&id| Object::Reference(id)).collect()),
            );
        }
    }

    doc.prune_objects();

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    doc.save(output)
        .map_err(|e| WandelError::ImageEncode(format!("Failed to save organized PDF: {}", e)))?;
    let size = std::fs::metadata(output)?.len();
    Ok(size)
}

/// Extracts plain text page-by-page from a PDF completely offline using `lopdf`.
pub fn extract_pdf_text(input: &Path) -> Result<Vec<(u32, String)>> {
    let doc = lopdf::Document::load(input)
        .map_err(|e| WandelError::ImageDecode(format!("Failed to load PDF: {}", e)))?;
    let pages = doc.get_pages();
    if pages.is_empty() {
        return Err(WandelError::ImageDecode("PDF has no pages".into()));
    }
    let mut out = Vec::with_capacity(pages.len());
    for &page_num in pages.keys() {
        let text = doc
            .extract_text(&[page_num])
            .unwrap_or_default()
            .trim()
            .to_string();
        out.push((page_num, text));
    }
    Ok(out)
}

/// Extracts all page text from a PDF and saves it as a clean page-delimited UTF-8 `.txt` file.
/// Returns `(page_count, output_bytes)`.
pub fn export_pdf_text_file(input: &Path, output: &Path) -> Result<(usize, u64)> {
    let pages = extract_pdf_text(input)?;
    let page_count = pages.len();
    let mut content = String::new();
    for (idx, (page_num, text)) in pages.iter().enumerate() {
        if idx > 0 {
            content.push_str("\n\n");
        }
        content.push_str(&format!("--- Page {} ---\n{}", page_num, text));
    }
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output, content.as_bytes())?;
    Ok((page_count, content.len() as u64))
}

/// Converts a light-background PDF into an eye-friendly dark-mode PDF in parallel via `hayro` + `rayon`
/// (maps pure white `#ffffff` -> `#18181b` and black `#000000` -> `#e4e4e7`) and repacks via `pdf-writer`.
pub fn convert_pdf_dark_mode(input: &Path, output: &Path, dpi: u16) -> Result<u64> {
    let bytes = Arc::new(std::fs::read(input)?);
    let pdf = HayroPdf::new(Arc::clone(&bytes))
        .map_err(|e| WandelError::ImageDecode(format!("Failed to parse PDF: {:?}", e)))?;
    let total_pages = pdf.pages().len();
    if total_pages == 0 {
        return Err(WandelError::ImageDecode("PDF contains no pages".into()));
    }

    let scale = f32::from(dpi.clamp(72, 300)) / 72.0;

    let pages: Result<Vec<PreparedPdfPage>> = (0..total_pages)
        .into_par_iter()
        .map(|idx| {
            let thread_pdf = HayroPdf::new(Arc::clone(&bytes))
                .map_err(|e| WandelError::ImageDecode(format!("PDF parse error: {:?}", e)))?;
            let page = thread_pdf
                .pages()
                .get(idx)
                .ok_or_else(|| WandelError::ImageDecode("Page index out of bounds".into()))?;
            let cache = RenderCache::new();
            let pixmap = hayro::render(
                page,
                &cache,
                &InterpreterSettings::default(),
                &RenderSettings::default(),
                &PixmapSettings {
                    x_scale: scale,
                    y_scale: scale,
                    bg_color: WHITE,
                },
            );
            let img = pixmap_to_dynamic_image(pixmap)?;
            let mut rgb = img.to_rgb8();
            // Smooth dark-mode palette inversion: white (255) -> zinc-900 (24,24,27), black (0) -> zinc-200 (228,228,231)
            for px in rgb.as_mut().as_chunks_mut::<3>().0 {
                let r = 255.0 - f32::from(px[0]);
                let g = 255.0 - f32::from(px[1]);
                let b = 255.0 - f32::from(px[2]);
                px[0] = (24.0 + r * (204.0 / 255.0)).round().clamp(0.0, 255.0) as u8;
                px[1] = (24.0 + g * (204.0 / 255.0)).round().clamp(0.0, 255.0) as u8;
                px[2] = (27.0 + b * (204.0 / 255.0)).round().clamp(0.0, 255.0) as u8;
            }
            let (w, h) = (rgb.width(), rgb.height());
            let mut jpeg_bytes = Vec::new();
            let mut enc = jpeg_encoder::Encoder::new(&mut jpeg_bytes, 84);
            enc.set_progressive(false);
            enc.encode(
                rgb.as_raw(),
                w as u16,
                h as u16,
                jpeg_encoder::ColorType::Rgb,
            )
            .map_err(|e| WandelError::ImageEncode(format!("Dark PDF JPEG error: {}", e)))?;
            Ok(PreparedPdfPage {
                jpeg_bytes,
                width_px: w,
                height_px: h,
            })
        })
        .collect();

    let pages = pages?;

    let mut out_pdf = Pdf::new();
    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);
    let mut next_id = 3i32;
    let mut page_ids = Vec::with_capacity(pages.len());
    let mut page_triples = Vec::with_capacity(pages.len());

    for _ in &pages {
        let p_id = Ref::new(next_id);
        let img_id = Ref::new(next_id + 1);
        let content_id = Ref::new(next_id + 2);
        next_id += 3;
        page_ids.push(p_id);
        page_triples.push((p_id, img_id, content_id));
    }

    out_pdf.catalog(catalog_id).pages(page_tree_id);
    out_pdf
        .pages(page_tree_id)
        .kids(page_ids.iter().copied())
        .count(page_ids.len() as i32);

    for (page_data, (page_id, img_id, content_id)) in pages.iter().zip(page_triples) {
        let w = (page_data.width_px as f32) / scale;
        let h = (page_data.height_px as f32) / scale;
        let mut page = out_pdf.page(page_id);
        page.media_box(Rect::new(0.0, 0.0, w, h));
        page.parent(page_tree_id);
        page.contents(content_id);
        page.resources().x_objects().pair(Name(b"Im0"), img_id);
        page.finish();

        let mut img_xobj = out_pdf.image_xobject(img_id, &page_data.jpeg_bytes);
        img_xobj.filter(Filter::DctDecode);
        img_xobj.width(page_data.width_px as i32);
        img_xobj.height(page_data.height_px as i32);
        img_xobj.color_space().device_rgb();
        img_xobj.bits_per_component(8);
        img_xobj.finish();

        let mut content = Content::new();
        content.save_state();
        content.transform([w, 0.0, 0.0, h, 0.0, 0.0]);
        content.x_object(Name(b"Im0"));
        content.restore_state();
        out_pdf.stream(content_id, &content.finish());
    }

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let final_bytes = out_pdf.finish();
    std::fs::write(output, &final_bytes)?;
    Ok(final_bytes.len() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_page_ranges_and_ordered_specs() {
        assert_eq!(parse_page_range("", 4), vec![0, 1, 2, 3]);
        assert_eq!(parse_page_range("1-2,4", 5), vec![0, 1, 3]);
        assert_eq!(parse_page_range("all", 3), vec![0, 1, 2]);
        assert_eq!(parse_page_range("1", 0), Vec::<usize>::new());

        assert_eq!(parse_ordered_page_spec("3,1-2", 4), vec![2, 0, 1]);
        assert_eq!(parse_ordered_page_spec("3-1,2", 4), vec![2, 1, 0]);
        assert_eq!(parse_ordered_page_spec("all", 2), vec![0, 1]);
        assert_eq!(parse_ordered_page_spec("1", 0), Vec::<usize>::new());
    }
}
