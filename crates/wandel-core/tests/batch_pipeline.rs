use image::{DynamicImage, ImageBuffer, Rgba};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use wandel_core::{
    inspect_paths, probe_hardware, run_batch, AudioFormat, ConversionSettings, ImageFormat,
    ImageSettings, JobItem, MediaKind, ProgressEvent, ResizeMode, VideoFormat, VideoSettings,
    VideoTarget,
};

fn find_bin(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let p = dir.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

#[test]
fn parallel_image_batch_and_svg_rasterization() {
    let dir = tempdir().unwrap();
    let mut input_paths = Vec::new();

    // Generate 12 PNGs (320x240)
    for idx in 0..12 {
        let p = dir.path().join(format!("photo_{:02}.png", idx));
        let img = ImageBuffer::from_fn(320, 240, |x, y| {
            Rgba([
                ((x + idx * 17) % 255) as u8,
                ((y + idx * 23) % 255) as u8,
                160,
                255,
            ])
        });
        DynamicImage::ImageRgba8(img).save(&p).unwrap();
        input_paths.push(p);
    }

    // Also create a test SVG
    let svg_path = dir.path().join("logo.svg");
    std::fs::write(
        &svg_path,
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect width="100" height="100" fill="#4f46e5"/>
            <circle cx="50" cy="50" r="30" fill="#ffffff"/>
        </svg>"##,
    )
    .unwrap();
    input_paths.push(svg_path);

    let inspected = inspect_paths(&input_paths, None, None);
    assert_eq!(inspected.len(), 13);
    assert!(inspected.iter().all(|m| m.media_kind == MediaKind::Image));
    assert!(inspected.iter().all(|m| m.thumbnail_data_url.is_some()));

    let jobs: Vec<JobItem> = inspected
        .iter()
        .map(|m| JobItem {
            id: m.id.clone(),
            path: PathBuf::from(&m.path),
            media_kind: m.media_kind,
            duration_secs: m.duration_secs,
            width: m.width,
            height: m.height,
            relative_dir: m.relative_dir.clone(),
            image_override: None,
            video_override: None,
        })
        .collect();

    let settings = ConversionSettings {
        image: ImageSettings {
            format: ImageFormat::Webp,
            quality: 80,
            lossless: false,
            resize: ResizeMode::MaxDimensions {
                width: Some(160),
                height: None,
            },
            never_upscale: true,
            strip_metadata: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let events = Arc::new(Mutex::new(Vec::new()));
    let events_clone = Arc::clone(&events);
    let hw = probe_hardware(None, None);

    run_batch(
        jobs,
        &settings,
        &hw,
        None,
        Arc::new(AtomicBool::new(false)),
        move |ev| {
            events_clone.lock().unwrap().push(ev);
        },
    );

    let recorded = events.lock().unwrap();
    let finished = recorded.iter().find_map(|e| match e {
        ProgressEvent::BatchFinished {
            completed,
            failed,
            cancelled,
            total_duration_ms,
        } => Some((*completed, *failed, *cancelled, *total_duration_ms)),
        _ => None,
    });

    let (completed, failed, cancelled, total_ms) = finished.expect("BatchFinished event missing");
    println!(
        "Parallel batch of 13 images (12 PNG + 1 SVG -> WebP) finished in {} ms",
        total_ms
    );
    assert_eq!(completed, 13);
    assert_eq!(failed, 0);
    assert_eq!(cancelled, 0);

    // Check resized dimensions on one converted WebP
    let out_webp = dir.path().join("photo_00.webp");
    assert!(out_webp.exists());
    let (w, h) = image::image_dimensions(&out_webp).unwrap();
    assert_eq!((w, h), (160, 120));
}

#[test]
fn video_conversion_and_audio_extraction_pipeline() {
    let Some(ffmpeg) = find_bin("ffmpeg") else {
        eprintln!("Skipping video test: ffmpeg not in PATH");
        return;
    };
    let ffprobe = find_bin("ffprobe");

    let hw = probe_hardware(Some(&ffmpeg), ffprobe.as_deref());
    assert!(hw.ffmpeg_available);
    println!(
        "Hardware probe: cores={}, gpu={:?}, h264={:?}, h265={:?}",
        hw.cpu_cores, hw.gpu_backend_name, hw.h264_hw_encoder, hw.h265_hw_encoder
    );

    let dir = tempdir().unwrap();
    let src_mp4 = dir.path().join("clip.mp4");

    // Generate a 1-second 320x240 test video with sine audio tone
    let status = Command::new(&ffmpeg)
        .args([
            "-y",
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "testsrc=duration=1:size=320x240:rate=24",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=1",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
        ])
        .arg(&src_mp4)
        .status()
        .unwrap();
    assert!(status.success());

    let inspected = inspect_paths(std::slice::from_ref(&src_mp4), Some(&ffmpeg), ffprobe.as_deref());
    assert_eq!(inspected.len(), 1);
    assert_eq!(inspected[0].media_kind, MediaKind::Video);
    assert_eq!(inspected[0].width, Some(320));
    assert_eq!(inspected[0].height, Some(240));
    assert!(inspected[0].duration_secs.unwrap_or(0.0) > 0.5);

    // 1. Convert MP4 -> resized MP4 (H.264 with hardware accel if available)
    run_video_target(
        &inspected[0],
        VideoTarget::Video(VideoFormat::Mp4H264),
        ResizeMode::MaxDimensions {
            width: Some(160),
            height: None,
        },
        &hw,
        &ffmpeg,
        dir.path(),
    );

    // 2. Convert MP4 -> Animated GIF
    run_video_target(
        &inspected[0],
        VideoTarget::Video(VideoFormat::Gif),
        ResizeMode::Original,
        &hw,
        &ffmpeg,
        dir.path(),
    );

    // 3. Extract MP3 audio
    run_video_target(
        &inspected[0],
        VideoTarget::Audio(AudioFormat::Mp3),
        ResizeMode::Original,
        &hw,
        &ffmpeg,
        dir.path(),
    );

    // 4. Convert MP4 -> Animated WebP (v3.0)
    run_video_target(
        &inspected[0],
        VideoTarget::Video(VideoFormat::AnimatedWebp),
        ResizeMode::Original,
        &hw,
        &ffmpeg,
        dir.path(),
    );
}

#[test]
fn v3_image_transforms_and_target_size() {
    use wandel_core::image_engine::convert_image;
    use wandel_core::AspectCrop;

    let dir = tempdir().unwrap();
    let src = dir.path().join("wide.png");
    let img = ImageBuffer::from_fn(200, 100, |x, y| {
        Rgba([(x % 255) as u8, ((y * 2) % 255) as u8, 120, 255])
    });
    DynamicImage::ImageRgba8(img).save(&src).unwrap();

    // Rotate 90° (100x200) + 1:1 center crop (100x100) + target <= 8 KB
    let dst = dir.path().join("transformed.jpg");
    let settings = ImageSettings {
        format: ImageFormat::Jpg,
        quality: 90,
        rotate: 90,
        flip_horizontal: true,
        crop_aspect: Some(AspectCrop::Square1x1),
        target_size_kb: Some(8),
        ..Default::default()
    };
    let bytes = convert_image(&src, &dst, &settings, None).unwrap();
    assert!(bytes > 0 && bytes <= 8 * 1024);
    let (w, h) = image::image_dimensions(&dst).unwrap();
    assert_eq!((w, h), (100, 100));
}

#[test]
fn v4_watermark_and_per_file_override() {
    use wandel_core::{WatermarkPosition, WatermarkSettings};

    let dir = tempdir().unwrap();
    let src = dir.path().join("base.png");
    let logo = dir.path().join("logo.png");

    let base_img = ImageBuffer::from_fn(200, 200, |_, _| Rgba([30, 30, 40, 255]));
    DynamicImage::ImageRgba8(base_img).save(&src).unwrap();

    let wm_img = ImageBuffer::from_fn(40, 40, |_, _| Rgba([255, 255, 255, 200]));
    DynamicImage::ImageRgba8(wm_img).save(&logo).unwrap();

    let override_settings = ImageSettings {
        format: ImageFormat::Png,
        watermark: Some(WatermarkSettings {
            image_path: logo,
            opacity: 0.8,
            scale_percent: 25.0,
            margin_px: 10,
            position: WatermarkPosition::BottomRight,
        }),
        ..Default::default()
    };

    let jobs = vec![JobItem {
        id: "override-job".into(),
        path: src.clone(),
        media_kind: MediaKind::Image,
        duration_secs: None,
        width: Some(200),
        height: Some(200),
        relative_dir: None,
        image_override: Some(override_settings),
        video_override: None,
    }];

    // Global setting is JPG, but per-file override forces PNG + watermark
    let global = ConversionSettings {
        image: ImageSettings {
            format: ImageFormat::Jpg,
            ..Default::default()
        },
        ..Default::default()
    };

    let out_path = Arc::new(Mutex::new(None::<String>));
    let out_clone = Arc::clone(&out_path);
    let hw = probe_hardware(None, None);

    run_batch(
        jobs,
        &global,
        &hw,
        None,
        Arc::new(AtomicBool::new(false)),
        move |ev| {
            if let ProgressEvent::FileCompleted { output_path, .. } = ev {
                *out_clone.lock().unwrap() = Some(output_path);
            }
        },
    );

    let out = out_path.lock().unwrap().clone().expect("Expected FileCompleted");
    assert!(out.ends_with(".png"), "Per-file override should produce PNG, got {}", out);
    // Verify bottom-right watermark blended onto dark base
    let result_img = image::open(&out).unwrap().to_rgba8();
    let px = result_img.get_pixel(170, 170);
    assert!(px[0] > 100, "Expected brightened watermark pixel, got {:?}", px);
}

#[test]
fn v5_merge_images_to_pdf_and_text_watermark() {
    use wandel_core::{merge_images_to_pdf, PdfPageFit, TextWatermarkSettings, WatermarkPosition};

    let dir = tempdir().unwrap();
    let p1 = dir.path().join("page1.png");
    let p2 = dir.path().join("page2.png");

    let img1 = ImageBuffer::from_fn(240, 160, |x, y| Rgba([(x % 255) as u8, (y % 255) as u8, 200, 255]));
    let img2 = ImageBuffer::from_fn(160, 240, |x, y| Rgba([200, (x % 255) as u8, (y % 255) as u8, 255]));
    DynamicImage::ImageRgba8(img1).save(&p1).unwrap();
    DynamicImage::ImageRgba8(img2).save(&p2).unwrap();

    let out_pdf = dir.path().join("bundle.pdf");
    let settings = ImageSettings {
        quality: 80,
        grayscale: true,
        sharpen: true,
        text_watermark: Some(TextWatermarkSettings {
            text: "Wandel v5".into(),
            font_size_percent: 6.0,
            color: "#ffffff".into(),
            opacity: 0.8,
            margin_px: 12,
            position: WatermarkPosition::BottomRight,
            tiled: false,
        }),
        ..Default::default()
    };

    let bytes = merge_images_to_pdf(&[p1, p2], &out_pdf, &settings, PdfPageFit::A4, None).unwrap();
    assert!(bytes > 500, "Expected non-trivial PDF output, got {} bytes", bytes);
    let raw = std::fs::read(&out_pdf).unwrap();
    assert!(raw.starts_with(b"%PDF-"), "Output should have valid PDF header");
}

#[test]
fn v6_pdf_to_images_and_transparent_bg_fill() {
    use wandel_core::image_engine::convert_image;
    use wandel_core::{merge_images_to_pdf, PdfPageFit, PdfSettings};

    let dir = tempdir().unwrap();

    // 1. Create a 2-page PDF, inspect it, and extract both pages to WebP via pure-Rust hayro
    let p1 = dir.path().join("src1.png");
    let p2 = dir.path().join("src2.png");
    DynamicImage::ImageRgba8(ImageBuffer::from_fn(200, 150, |_, _| Rgba([40, 100, 220, 255])))
        .save(&p1)
        .unwrap();
    DynamicImage::ImageRgba8(ImageBuffer::from_fn(200, 150, |_, _| Rgba([220, 80, 60, 255])))
        .save(&p2)
        .unwrap();

    let pdf_path = dir.path().join("sample.pdf");
    merge_images_to_pdf(
        &[p1, p2],
        &pdf_path,
        &ImageSettings::default(),
        PdfPageFit::FitToImage,
        None,
    )
    .unwrap();

    let inspected = inspect_paths(std::slice::from_ref(&pdf_path), None, None);
    assert_eq!(inspected.len(), 1);
    assert_eq!(inspected[0].media_kind, MediaKind::Pdf);
    assert_eq!(inspected[0].page_count, Some(2));
    assert!(inspected[0].thumbnail_data_url.is_some());

    let jobs = vec![JobItem {
        id: inspected[0].id.clone(),
        path: pdf_path.clone(),
        media_kind: MediaKind::Pdf,
        duration_secs: None,
        width: inspected[0].width,
        height: inspected[0].height,
        relative_dir: None,
        image_override: None,
        video_override: None,
    }];

    let settings = ConversionSettings {
        image: ImageSettings {
            format: ImageFormat::Webp,
            quality: 85,
            ..Default::default()
        },
        pdf: PdfSettings {
            dpi: 150,
            page_range: "1-2".into(),
        },
        ..Default::default()
    };

    let hw = probe_hardware(None, None);
    let completed = Arc::new(Mutex::new(0usize));
    let completed_ref = Arc::clone(&completed);
    run_batch(
        jobs,
        &settings,
        &hw,
        None,
        Arc::new(AtomicBool::new(false)),
        move |ev| {
            if let ProgressEvent::FileCompleted { .. } = ev {
                *completed_ref.lock().unwrap() += 1;
            }
            if let ProgressEvent::FileFailed { error, .. } = ev {
                panic!("PDF-to-Images failed: {}", error);
            }
        },
    );
    assert_eq!(*completed.lock().unwrap(), 1);
    assert!(dir.path().join("sample_page_001.webp").exists());
    assert!(dir.path().join("sample_page_002.webp").exists());

    // 2. Test transparent PNG -> JPG with white #ffffff background fill + color adjustments
    let trans_png = dir.path().join("transparent.png");
    let out_jpg = dir.path().join("flattened.jpg");
    DynamicImage::ImageRgba8(ImageBuffer::from_fn(40, 40, |_, _| Rgba([0, 0, 0, 0])))
        .save(&trans_png)
        .unwrap();

    let jpg_settings = ImageSettings {
        format: ImageFormat::Jpg,
        quality: 95,
        background_hex: Some("#ffffff".into()),
        brightness: 5,
        contrast: 5.0,
        saturation: 10.0,
        ..Default::default()
    };
    convert_image(&trans_png, &out_jpg, &jpg_settings, None).unwrap();
    let flattened = image::open(&out_jpg).unwrap().to_rgb8();
    let px = flattened.get_pixel(20, 20);
    assert!(px[0] > 240 && px[1] > 240 && px[2] > 240, "Expected white fill, got {:?}", px);
}

#[test]
fn v7_rounded_corners_padding_icon_pack_and_metadata() {
    use wandel_core::image_engine::convert_image;
    use wandel_core::{generate_icon_pack, inspect_detailed_metadata, AspectCrop};

    let dir = tempdir().unwrap();
    let src = dir.path().join("rect.png");
    DynamicImage::ImageRgba8(ImageBuffer::from_fn(200, 100, |_, _| Rgba([60, 120, 240, 255])))
        .save(&src)
        .unwrap();

    // 1. Pad 200x100 -> 1:1 (200x200) + 50% rounded corners (circle)
    let out_png = dir.path().join("padded_circle.png");
    let settings = ImageSettings {
        format: ImageFormat::Png,
        pad_aspect: Some(AspectCrop::Square1x1),
        corner_radius_percent: 50.0,
        background_hex: Some("#ffffff".into()),
        ..Default::default()
    };
    convert_image(&src, &out_png, &settings, None).unwrap();
    let out_img = image::open(&out_png).unwrap().to_rgba8();
    assert_eq!(out_img.dimensions(), (200, 200));
    // Corner pixel (0,0) must be transparent due to 50% corner radius
    assert_eq!(out_img.get_pixel(0, 0)[3], 0);
    // Center pixel must be opaque
    assert_eq!(out_img.get_pixel(100, 100)[3], 255);

    // 2. Icon Pack generator
    let pack_dir = dir.path().join("icons");
    let files = generate_icon_pack(&src, &pack_dir, None).unwrap();
    assert_eq!(files.len(), 7);
    assert!(pack_dir.join("favicon.ico").exists());
    assert!(pack_dir.join("apple-touch-icon.png").exists());
    assert!(pack_dir.join("icon-512.png").exists());
    assert!(pack_dir.join("site.webmanifest").exists());

    // 3. Detailed Metadata Inspector
    let meta = inspect_detailed_metadata(&src, None).unwrap();
    assert_eq!(meta.file_name, "rect.png");
    assert!(!meta.has_gps_warning);
    assert!(meta.entries.iter().any(|e| e.label == "Dimensions" && e.value == "200 × 100 px"));
}

#[test]
fn v8_contact_sheet_zip_bundle_and_video_merge() {
    use wandel_core::{
        bundle_files_to_zip, create_contact_sheet, merge_videos, ContactSheetOptions,
    };

    let dir = tempdir().unwrap();

    // 1. Contact Sheet Grid (4 tiles -> 2x2 grid)
    let mut tile_paths = Vec::new();
    for i in 0..4 {
        let p = dir.path().join(format!("tile_{}.png", i));
        DynamicImage::ImageRgba8(ImageBuffer::from_fn(120, 90, |_, _| {
            Rgba([(i * 60) as u8, 120, 220, 255])
        }))
        .save(&p)
        .unwrap();
        tile_paths.push(p);
    }

    let grid_out = dir.path().join("contact_sheet.png");
    let opts = ContactSheetOptions {
        columns: 2,
        cell_size: 100,
        gap_px: 10,
        background_hex: "#18181b".into(),
    };
    let grid_bytes = create_contact_sheet(
        &tile_paths,
        &grid_out,
        &opts,
        &ImageSettings {
            format: ImageFormat::Png,
            ..Default::default()
        },
        None,
    )
    .unwrap();
    assert!(grid_bytes > 0);
    // 2 cols * 100 + 3 gaps * 10 = 230
    assert_eq!(image::image_dimensions(&grid_out).unwrap(), (230, 230));

    // 2. Pure-Rust ZIP archive bundle
    let zip_out = dir.path().join("bundle.zip");
    let zip_bytes = bundle_files_to_zip(&tile_paths, &zip_out).unwrap();
    assert!(zip_bytes > 100);
    let zip_header = std::fs::read(&zip_out).unwrap();
    assert_eq!(&zip_header[0..2], b"PK");

    // 3. Video Merge (if ffmpeg available)
    if let Some(ffmpeg) = find_bin("ffmpeg") {
        let ffprobe = find_bin("ffprobe");
        let hw = probe_hardware(Some(&ffmpeg), ffprobe.as_deref());
        let v1 = dir.path().join("v1.mp4");
        let v2 = dir.path().join("v2.mp4");
        for (vp, color) in [(&v1, "blue"), (&v2, "red")] {
            let st = Command::new(&ffmpeg)
                .args([
                    "-y",
                    "-hide_banner",
                    "-loglevel",
                    "error",
                    "-f",
                    "lavfi",
                    "-i",
                    &format!("color=c={}:s=320x240:d=0.5", color),
                    "-c:v",
                    "libx264",
                    "-pix_fmt",
                    "yuv420p",
                ])
                .arg(vp)
                .status()
                .unwrap();
            assert!(st.success());
        }
        let merged = dir.path().join("merged.mp4");
        let merged_bytes =
            merge_videos(&[v1, v2], &merged, &VideoSettings::default(), &hw, &ffmpeg).unwrap();
        assert!(merged_bytes > 1000);
    }
}

#[test]
fn v9_pdf_merge_split_dhash_dedupe_and_custom_crop() {
    use wandel_core::image_engine::convert_image;
    use wandel_core::{
        find_duplicate_images, inspect_pdf, merge_images_to_pdf, merge_pdfs, split_pdf,
        NormalizedCrop, PdfPageFit,
    };

    let dir = tempdir().unwrap();

    // 1. Custom visual normalized crop (middle 50% of 200x100 -> 100x50)
    let img_a = dir.path().join("a.png");
    let img_a_dup = dir.path().join("a_copy.png");
    let img_b = dir.path().join("b.png");

    let pattern_a = ImageBuffer::from_fn(200, 100, |x, y| {
        Rgba([(x % 255) as u8, ((y * 2) % 255) as u8, 120, 255])
    });
    let pattern_b = ImageBuffer::from_fn(200, 100, |x, y| {
        let v = if (x / 20 + y / 20) % 2 == 0 { 250 } else { 10 };
        Rgba([v, v, v, 255])
    });
    DynamicImage::ImageRgba8(pattern_a.clone()).save(&img_a).unwrap();
    DynamicImage::ImageRgba8(pattern_a).save(&img_a_dup).unwrap();
    DynamicImage::ImageRgba8(pattern_b).save(&img_b).unwrap();

    let cropped = dir.path().join("cropped.png");
    convert_image(
        &img_a,
        &cropped,
        &ImageSettings {
            format: ImageFormat::Png,
            custom_crop: Some(NormalizedCrop {
                x: 0.25,
                y: 0.25,
                width: 0.5,
                height: 0.5,
            }),
            ..Default::default()
        },
        None,
    )
    .unwrap();
    assert_eq!(image::image_dimensions(&cropped).unwrap(), (100, 50));

    // 2. Perceptual dHash duplicate finder
    let groups = find_duplicate_images(&[img_a.clone(), img_a_dup.clone(), img_b.clone()], 5, None);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].duplicates.len(), 1);

    // 3. Pure-Rust PDF merge & split
    let pdf1 = dir.path().join("doc1.pdf");
    let pdf2 = dir.path().join("doc2.pdf");
    merge_images_to_pdf(
        std::slice::from_ref(&img_a),
        &pdf1,
        &ImageSettings::default(),
        PdfPageFit::A4,
        None,
    )
    .unwrap();
    merge_images_to_pdf(
        &[img_a, img_b],
        &pdf2,
        &ImageSettings::default(),
        PdfPageFit::A4,
        None,
    )
    .unwrap();

    let merged_pdf = dir.path().join("merged.pdf");
    merge_pdfs(&[pdf1, pdf2], &merged_pdf).unwrap();
    let (merged_pages, _, _, _) = inspect_pdf(&merged_pdf);
    assert_eq!(merged_pages, Some(3));

    let split_out = dir.path().join("split.pdf");
    split_pdf(&merged_pdf, "1,3", &split_out).unwrap();
    let (split_pages, _, _, _) = inspect_pdf(&split_out);
    assert_eq!(split_pages, Some(2));
}

#[test]
fn v10_pdf_compress_palette_qr_and_audio_merge() {
    use wandel_core::{
        compress_pdf, extract_color_palette, generate_qr_svg, inspect_pdf, merge_audio,
        merge_images_to_pdf, save_qr_code, PdfPageFit,
    };

    let dir = tempdir().unwrap();

    // 1. Dominant Color Palette Extractor
    let img_path = dir.path().join("brand.png");
    DynamicImage::ImageRgba8(ImageBuffer::from_fn(120, 120, |x, _| {
        if x < 60 {
            Rgba([79, 70, 229, 255]) // Indigo
        } else {
            Rgba([16, 185, 129, 255]) // Emerald
        }
    }))
    .save(&img_path)
    .unwrap();

    let swatches = extract_color_palette(&img_path, 4, None).unwrap();
    assert!(!swatches.is_empty());
    assert!(swatches[0].hex.starts_with('#'));

    // 2. Pure-Rust QR Code SVG & PNG
    let svg = generate_qr_svg("https://wandel.app", "#000000", "#ffffff").unwrap();
    assert!(svg.contains("<svg"));
    let qr_png = dir.path().join("qr.png");
    let qr_bytes = save_qr_code("https://wandel.app", "#4f46e5", "#ffffff", 256, &qr_png).unwrap();
    assert!(qr_bytes > 200);
    assert_eq!(image::image_dimensions(&qr_png).unwrap(), (256, 256));

    // 3. Pure-Rust PDF Compress
    let src_pdf = dir.path().join("src.pdf");
    merge_images_to_pdf(
        &[img_path],
        &src_pdf,
        &ImageSettings::default(),
        PdfPageFit::A4,
        None,
    )
    .unwrap();
    let comp_pdf = dir.path().join("compressed.pdf");
    let comp_bytes = compress_pdf(&src_pdf, &comp_pdf, 100, 65).unwrap();
    assert!(comp_bytes > 200);
    assert_eq!(inspect_pdf(&comp_pdf).0, Some(1));

    // 4. Audio Merge (if ffmpeg available)
    if let Some(ffmpeg) = find_bin("ffmpeg") {
        let a1 = dir.path().join("a1.wav");
        let a2 = dir.path().join("a2.wav");
        for (ap, freq) in [(&a1, 440), (&a2, 660)] {
            let st = Command::new(&ffmpeg)
                .args([
                    "-y",
                    "-hide_banner",
                    "-loglevel",
                    "error",
                    "-f",
                    "lavfi",
                    "-i",
                    &format!("sine=frequency={}:duration=0.4", freq),
                ])
                .arg(ap)
                .status()
                .unwrap();
            assert!(st.success());
        }
        let merged_mp3 = dir.path().join("merged.mp3");
        let bytes = merge_audio(&[a1, a2], &merged_mp3, &VideoSettings::default(), &ffmpeg).unwrap();
        assert!(bytes > 1000);
    }
}

#[test]
fn v11_filters_data_url_and_video_storyboard() {
    use wandel_core::{create_video_storyboard, export_image_data_url};

    let dir = tempdir().unwrap();

    // 1. Blur + Pixelate + Vignette + Base64 Data URL export
    let src = dir.path().join("photo.png");
    DynamicImage::ImageRgba8(ImageBuffer::from_fn(120, 120, |x, y| {
        Rgba([(x * 2) as u8, (y * 2) as u8, 200, 255])
    }))
    .save(&src)
    .unwrap();

    let settings = ImageSettings {
        format: ImageFormat::Webp,
        quality: 80,
        blur_sigma: 1.5,
        pixelate_block: 4,
        vignette_strength: 0.5,
        ..Default::default()
    };
    let exported = export_image_data_url(&src, &settings, None).unwrap();
    assert!(exported.data_url.starts_with("data:image/webp;base64,"));
    assert_eq!((exported.width, exported.height), (120, 120));
    assert!(exported.bytes > 100);

    // 2. Video Storyboard (3x3 contact sheet from a 1s synthetic clip)
    if let Some(ffmpeg) = find_bin("ffmpeg") {
        let ffprobe = find_bin("ffprobe");
        let clip = dir.path().join("clip.mp4");
        let st = Command::new(&ffmpeg)
            .args([
                "-y",
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc=duration=1:size=320x240:rate=24",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
            ])
            .arg(&clip)
            .status()
            .unwrap();
        assert!(st.success());

        let sb = dir.path().join("storyboard.png");
        let sb_bytes =
            create_video_storyboard(&clip, &sb, 4, 2, &ffmpeg, ffprobe.as_deref()).unwrap();
        assert!(sb_bytes > 500);
        assert!(sb.exists());
    }
}

#[test]
fn v12_screenshot_frame_shadow_srcset_and_boomerang() {
    use wandel_core::image_engine::convert_image;
    use wandel_core::{generate_responsive_set, FrameGradient, VideoPlaybackMode};

    let dir = tempdir().unwrap();

    // 1. Screenshot Gradient Frame + Soft Drop Shadow + Rounded Corners
    let src = dir.path().join("shot.png");
    DynamicImage::ImageRgba8(ImageBuffer::from_fn(800, 500, |x, y| {
        Rgba([(x % 255) as u8, (y % 255) as u8, 220, 255])
    }))
    .save(&src)
    .unwrap();

    let framed = dir.path().join("framed.png");
    convert_image(
        &src,
        &framed,
        &ImageSettings {
            format: ImageFormat::Png,
            corner_radius_percent: 6.0,
            frame_padding_px: 40,
            frame_gradient: FrameGradient::Indigo,
            drop_shadow: true,
            ..Default::default()
        },
        None,
    )
    .unwrap();
    // 800 + 2*40 = 880, 500 + 2*40 = 580
    assert_eq!(image::image_dimensions(&framed).unwrap(), (880, 580));

    // 2. Responsive Web srcset generator (320w, 640w, 800w + snippet.html)
    let srcset_dir = dir.path().join("srcset");
    let res = generate_responsive_set(
        &src,
        &srcset_dir,
        &ImageSettings {
            format: ImageFormat::Webp,
            quality: 80,
            ..Default::default()
        },
        None,
    )
    .unwrap();
    assert_eq!(res.files.len(), 4); // 320w, 640w, 800w, snippet.html
    assert!(res.html_snippet.contains("srcset="));
    assert!(res.html_snippet.contains("320w"));

    // 3. Video Boomerang loop (if ffmpeg available)
    if let Some(ffmpeg) = find_bin("ffmpeg") {
        let ffprobe = find_bin("ffprobe");
        let hw = probe_hardware(Some(&ffmpeg), ffprobe.as_deref());
        let clip = dir.path().join("clip.mp4");
        let st = Command::new(&ffmpeg)
            .args([
                "-y",
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc=duration=0.5:size=320x240:rate=24",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
            ])
            .arg(&clip)
            .status()
            .unwrap();
        assert!(st.success());

        let boom_out = dir.path().join("boomerang.mp4");
        let settings = VideoSettings {
            playback_mode: VideoPlaybackMode::Boomerang,
            use_hardware_accel: false,
            ..Default::default()
        };
        let bytes = wandel_core::video_engine::convert_video(
            &clip,
            &boom_out,
            &settings,
            Some(0.5),
            &hw,
            &ffmpeg,
            2,
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        assert!(bytes > 1000);
    }
}

#[test]
fn v13_auto_trim_pdf_rotate_reorder_and_video_reframe() {
    use wandel_core::image_engine::convert_image;
    use wandel_core::{
        inspect_pdf, merge_images_to_pdf, reorder_pdf_pages, rotate_pdf_pages, PdfPageFit,
        VideoAspectReframe,
    };

    let dir = tempdir().unwrap();

    // 1. Auto-Trim Solid White Border (200x160 with 30px white margin -> 140x100)
    let bordered_solid = dir.path().join("solid_border.png");
    DynamicImage::ImageRgba8(ImageBuffer::from_fn(200, 160, |x, y| {
        if (30..170).contains(&x) && (30..130).contains(&y) {
            Rgba([40, 120, 240, 255])
        } else {
            Rgba([255, 255, 255, 255])
        }
    }))
    .save(&bordered_solid)
    .unwrap();

    let trimmed_solid = dir.path().join("trimmed_solid.png");
    convert_image(
        &bordered_solid,
        &trimmed_solid,
        &ImageSettings {
            format: ImageFormat::Png,
            auto_trim_border: true,
            auto_trim_tolerance: 12,
            ..Default::default()
        },
        None,
    )
    .unwrap();
    assert_eq!(image::image_dimensions(&trimmed_solid).unwrap(), (140, 100));

    // 2. Auto-Trim Transparent Border (120x120 with 20px transparent margin -> 80x80)
    let bordered_alpha = dir.path().join("alpha_border.png");
    DynamicImage::ImageRgba8(ImageBuffer::from_fn(120, 120, |x, y| {
        if (20..100).contains(&x) && (20..100).contains(&y) {
            Rgba([220, 60, 90, 255])
        } else {
            Rgba([0, 0, 0, 0])
        }
    }))
    .save(&bordered_alpha)
    .unwrap();

    let trimmed_alpha = dir.path().join("trimmed_alpha.png");
    convert_image(
        &bordered_alpha,
        &trimmed_alpha,
        &ImageSettings {
            format: ImageFormat::Png,
            auto_trim_border: true,
            auto_trim_tolerance: 8,
            ..Default::default()
        },
        None,
    )
    .unwrap();
    assert_eq!(image::image_dimensions(&trimmed_alpha).unwrap(), (80, 80));

    // 3. Lossless PDF Rotate & Reorder / Extract
    let p1 = dir.path().join("p1.png");
    let p2 = dir.path().join("p2.png");
    let p3 = dir.path().join("p3.png");
    DynamicImage::ImageRgba8(ImageBuffer::from_pixel(100, 140, Rgba([200, 40, 40, 255])))
        .save(&p1)
        .unwrap();
    DynamicImage::ImageRgba8(ImageBuffer::from_pixel(140, 100, Rgba([40, 200, 40, 255])))
        .save(&p2)
        .unwrap();
    DynamicImage::ImageRgba8(ImageBuffer::from_pixel(120, 120, Rgba([40, 40, 200, 255])))
        .save(&p3)
        .unwrap();

    let base_pdf = dir.path().join("three_pages.pdf");
    merge_images_to_pdf(
        &[p1, p2, p3],
        &base_pdf,
        &ImageSettings::default(),
        PdfPageFit::FitToImage,
        None,
    )
    .unwrap();
    assert_eq!(inspect_pdf(&base_pdf).0, Some(3));

    let rotated_pdf = dir.path().join("rotated.pdf");
    rotate_pdf_pages(&base_pdf, &rotated_pdf, 90, Some("1,3")).unwrap();
    assert_eq!(inspect_pdf(&rotated_pdf).0, Some(3));
    // Verify /Rotate = 90 on page 1 and 0/absent on page 2 via lopdf
    let rot_doc = lopdf::Document::load(&rotated_pdf).unwrap();
    let rot_pages = rot_doc.get_pages();
    let p1_rot = rot_doc
        .get_object(*rot_pages.get(&1).unwrap())
        .unwrap()
        .as_dict()
        .unwrap()
        .get(b"Rotate")
        .unwrap()
        .as_i64()
        .unwrap();
    assert_eq!(p1_rot, 90);
    assert!(rot_doc
        .get_object(*rot_pages.get(&2).unwrap())
        .unwrap()
        .as_dict()
        .unwrap()
        .get(b"Rotate")
        .is_err());

    let reordered_pdf = dir.path().join("reordered.pdf");
    reorder_pdf_pages(&base_pdf, &reordered_pdf, "3,1").unwrap();
    assert_eq!(inspect_pdf(&reordered_pdf).0, Some(2));
    // First page in reordered PDF should be original page 3 (120x120)
    let (_, w_first, h_first, _) = inspect_pdf(&reordered_pdf);
    assert_eq!((w_first, h_first), (Some(120), Some(120)));

    // 4. Social Video Blur-Pad Aspect Reframing (16:9 -> 9:16 with max height 480 -> 270x480)
    if let Some(ffmpeg) = find_bin("ffmpeg") {
        let ffprobe = find_bin("ffprobe");
        let hw = probe_hardware(Some(&ffmpeg), ffprobe.as_deref());
        let clip = dir.path().join("landscape.mp4");
        let st = Command::new(&ffmpeg)
            .args([
                "-y",
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc=duration=0.4:size=320x180:rate=24",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
            ])
            .arg(&clip)
            .status()
            .unwrap();
        assert!(st.success());

        let vertical_out = dir.path().join("vertical_9x16.mp4");
        let settings = VideoSettings {
            aspect_reframe: VideoAspectReframe::Portrait9x16,
            resize: ResizeMode::MaxDimensions {
                width: None,
                height: Some(480),
            },
            use_hardware_accel: false,
            ..Default::default()
        };
        let bytes = wandel_core::video_engine::convert_video(
            &clip,
            &vertical_out,
            &settings,
            Some(0.4),
            &hw,
            &ffmpeg,
            2,
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        assert!(bytes > 1000);

        if let Some(ffp) = ffprobe.as_deref() {
            let (vw, vh, _, _) =
                wandel_core::video_engine::inspect_video(&vertical_out, Some(ffp), None, true);
            assert_eq!((vw, vh), (Some(270), Some(480)));
        }
    }
}

#[test]
fn v14_compare_collage_video_frames_and_audio_waveform() {
    use wandel_core::{
        create_audio_waveform, create_comparison_image, extract_video_frames, ComparisonOptions,
    };

    let dir = tempdir().unwrap();

    // 1. Horizontal & Vertical Before/After Comparison Collage
    let img_a = dir.path().join("before.png");
    let img_b = dir.path().join("after.png");
    DynamicImage::ImageRgba8(ImageBuffer::from_pixel(200, 100, Rgba([60, 120, 220, 255])))
        .save(&img_a)
        .unwrap();
    // Different height (200px) -> should scale to height 100 (width 100) for side-by-side
    DynamicImage::ImageRgba8(ImageBuffer::from_pixel(200, 200, Rgba([220, 80, 100, 255])))
        .save(&img_b)
        .unwrap();

    let cmp_h = dir.path().join("compare_h.png");
    let bytes_h = create_comparison_image(
        &img_a,
        &img_b,
        &cmp_h,
        &ComparisonOptions {
            vertical: false,
            gap_px: 10,
            ..Default::default()
        },
        &ImageSettings {
            format: ImageFormat::Png,
            ..Default::default()
        },
        None,
    )
    .unwrap();
    assert!(bytes_h > 500);
    // 200 + 10 + 100 = 310, height = 100
    assert_eq!(image::image_dimensions(&cmp_h).unwrap(), (310, 100));

    let cmp_v = dir.path().join("compare_v.png");
    create_comparison_image(
        &img_a,
        &img_b,
        &cmp_v,
        &ComparisonOptions {
            vertical: true,
            gap_px: 10,
            ..Default::default()
        },
        &ImageSettings {
            format: ImageFormat::Png,
            ..Default::default()
        },
        None,
    )
    .unwrap();
    // width = 200, height = 100 + 10 + 200 = 310
    assert_eq!(image::image_dimensions(&cmp_v).unwrap(), (200, 310));

    // 2. Video Frame Sequence Extraction & Audio Waveform PNG (if ffmpeg available)
    if let Some(ffmpeg) = find_bin("ffmpeg") {
        let clip = dir.path().join("seq.mp4");
        let st = Command::new(&ffmpeg)
            .args([
                "-y",
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc=duration=1.0:size=160x120:rate=24",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
            ])
            .arg(&clip)
            .status()
            .unwrap();
        assert!(st.success());

        let frames_dir = dir.path().join("frames");
        let frames = extract_video_frames(&clip, &frames_dir, 2.0, "webp", &ffmpeg).unwrap();
        assert!(frames.len() >= 2, "Expected >= 2 extracted frames, got {}", frames.len());
        assert!(Path::new(&frames[0]).exists());

        let audio = dir.path().join("tone.wav");
        let st_a = Command::new(&ffmpeg)
            .args([
                "-y",
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=1.0",
            ])
            .arg(&audio)
            .status()
            .unwrap();
        assert!(st_a.success());

        let wave_png = dir.path().join("waveform.png");
        let wave_bytes = create_audio_waveform(
            &audio,
            &wave_png,
            640,
            180,
            "#6366f1",
            "#18181b",
            &ffmpeg,
        )
        .unwrap();
        assert!(wave_bytes > 500);
        assert_eq!(image::image_dimensions(&wave_png).unwrap(), (640, 180));
    }
}

#[test]
fn v15_filter_presets_pdf_stamp_slideshow_and_remux() {
    use wandel_core::image_engine::convert_image;
    use wandel_core::{
        create_slideshow_video, inspect_pdf, merge_images_to_pdf, stamp_pdf_text,
        ImageFilterPreset, PdfPageFit,
    };

    let dir = tempdir().unwrap();

    // 1. One-click Color Filter Preset (Invert & Sepia)
    let src = dir.path().join("color.png");
    DynamicImage::ImageRgba8(ImageBuffer::from_pixel(64, 64, Rgba([20, 80, 200, 255])))
        .save(&src)
        .unwrap();

    let inverted = dir.path().join("inverted.png");
    convert_image(
        &src,
        &inverted,
        &ImageSettings {
            format: ImageFormat::Png,
            filter_preset: ImageFilterPreset::Invert,
            ..Default::default()
        },
        None,
    )
    .unwrap();
    let inv_px = image::open(&inverted).unwrap().to_rgba8().get_pixel(0, 0).0;
    assert_eq!(inv_px, [235, 175, 55, 255]);

    let sepia = dir.path().join("sepia.png");
    convert_image(
        &src,
        &sepia,
        &ImageSettings {
            format: ImageFormat::Png,
            filter_preset: ImageFilterPreset::Sepia,
            ..Default::default()
        },
        None,
    )
    .unwrap();
    let sep_px = image::open(&sepia).unwrap().to_rgba8().get_pixel(0, 0).0;
    assert!(sep_px[0] >= sep_px[1] && sep_px[1] >= sep_px[2]);

    // 2. Lossless Pure-Rust PDF Watermark Stamp
    let base_pdf = dir.path().join("doc.pdf");
    merge_images_to_pdf(
        &[src.clone(), inverted.clone()],
        &base_pdf,
        &ImageSettings::default(),
        PdfPageFit::A4,
        None,
    )
    .unwrap();
    let stamped_pdf = dir.path().join("stamped.pdf");
    let stamped_bytes = stamp_pdf_text(&base_pdf, &stamped_pdf, "CONFIDENTIAL").unwrap();
    assert!(stamped_bytes > 500);
    assert_eq!(inspect_pdf(&stamped_pdf).0, Some(2));
    let stamped_raw = std::fs::read(&stamped_pdf).unwrap();
    assert!(
        String::from_utf8_lossy(&stamped_raw).contains("WandelStampFont"),
        "Expected WandelStampFont resource in stamped PDF"
    );

    // 3. Image-to-Video Slideshow & Instant Lossless Remux (if ffmpeg available)
    if let Some(ffmpeg) = find_bin("ffmpeg") {
        let ffprobe = find_bin("ffprobe");
        let hw = probe_hardware(Some(&ffmpeg), ffprobe.as_deref());

        let slide_mp4 = dir.path().join("slideshow.mp4");
        let slide_bytes = create_slideshow_video(
            &[src, inverted],
            &slide_mp4,
            0.5,
            320,
            240,
            None,
            &ffmpeg,
        )
        .unwrap();
        assert!(slide_bytes > 1000);

        // Lossless remux MP4 -> MKV (-c:v copy)
        let remux_mkv = dir.path().join("remuxed.mkv");
        let remux_settings = VideoSettings {
            target: VideoTarget::Video(VideoFormat::Mkv),
            remux_only: true,
            ..Default::default()
        };
        let remux_bytes = wandel_core::video_engine::convert_video(
            &slide_mp4,
            &remux_mkv,
            &remux_settings,
            Some(1.0),
            &hw,
            &ffmpeg,
            2,
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        assert!(remux_bytes > 1000);
    }
}

#[test]
fn v16_image_slice_pdf_numbering_denoise_and_video_restore() {
    use wandel_core::{
        inspect_pdf, merge_images_to_pdf, number_pdf_pages, split_image_grid, AudioFormat,
        PdfPageFit,
    };

    let dir = tempdir().unwrap();

    // 1. 3x2 Image Grid Slicer (300x200 -> 6 tiles of 100x100)
    let pano = dir.path().join("pano.png");
    DynamicImage::ImageRgba8(ImageBuffer::from_fn(300, 200, |x, y| {
        Rgba([(x % 255) as u8, (y % 255) as u8, 180, 255])
    }))
    .save(&pano)
    .unwrap();

    let tiles_dir = dir.path().join("tiles");
    let tiles = split_image_grid(
        &pano,
        &tiles_dir,
        3,
        2,
        &ImageSettings {
            format: ImageFormat::Png,
            ..Default::default()
        },
        None,
    )
    .unwrap();
    assert_eq!(tiles.len(), 6);
    assert_eq!(image::image_dimensions(&tiles[0]).unwrap(), (100, 100));

    // 2. Lossless Pure-Rust PDF Page Numbering
    let base_pdf = dir.path().join("book.pdf");
    merge_images_to_pdf(
        &[PathBuf::from(&tiles[0]), PathBuf::from(&tiles[1])],
        &base_pdf,
        &ImageSettings::default(),
        PdfPageFit::A4,
        None,
    )
    .unwrap();
    let numbered_pdf = dir.path().join("numbered.pdf");
    let num_bytes = number_pdf_pages(&base_pdf, &numbered_pdf, "Page {n} of {total}").unwrap();
    assert!(num_bytes > 500);
    assert_eq!(inspect_pdf(&numbered_pdf).0, Some(2));
    let raw = std::fs::read(&numbered_pdf).unwrap();
    assert!(
        String::from_utf8_lossy(&raw).contains("WandelPageNumFont"),
        "Expected WandelPageNumFont resource in numbered PDF"
    );

    // 3. Audio FFT Denoise & Video Deinterlace + Sharpen (if ffmpeg available)
    if let Some(ffmpeg) = find_bin("ffmpeg") {
        let ffprobe = find_bin("ffprobe");
        let hw = probe_hardware(Some(&ffmpeg), ffprobe.as_deref());

        let noisy_wav = dir.path().join("voice.wav");
        let st_a = Command::new(&ffmpeg)
            .args([
                "-y",
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=300:duration=0.6",
            ])
            .arg(&noisy_wav)
            .status()
            .unwrap();
        assert!(st_a.success());

        let clean_mp3 = dir.path().join("clean.mp3");
        let audio_settings = VideoSettings {
            target: VideoTarget::Audio(AudioFormat::Mp3),
            denoise_audio: true,
            ..Default::default()
        };
        let audio_bytes = wandel_core::video_engine::convert_video(
            &noisy_wav,
            &clean_mp3,
            &audio_settings,
            Some(0.6),
            &hw,
            &ffmpeg,
            2,
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        assert!(audio_bytes > 500);

        let clip = dir.path().join("raw.mp4");
        let st_v = Command::new(&ffmpeg)
            .args([
                "-y",
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc=duration=0.4:size=240x160:rate=24",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
            ])
            .arg(&clip)
            .status()
            .unwrap();
        assert!(st_v.success());

        let restored = dir.path().join("restored.mp4");
        let restore_settings = VideoSettings {
            deinterlace: true,
            video_sharpen: true,
            use_hardware_accel: false,
            ..Default::default()
        };
        let vid_bytes = wandel_core::video_engine::convert_video(
            &clip,
            &restored,
            &restore_settings,
            Some(0.4),
            &hw,
            &ffmpeg,
            2,
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        assert!(vid_bytes > 1000);
    }
}

#[test]
fn v17_sprite_sheet_pdf_image_extract_audio_gain_and_audiogram() {
    use wandel_core::{
        create_podcast_video, create_sprite_sheet, extract_pdf_images, merge_images_to_pdf,
        AudioFormat, PdfPageFit,
    };

    let dir = tempdir().unwrap();

    // 1. Sprite Sheet + CSS coordinate map
    let icon_a = dir.path().join("icon_home.png");
    let icon_b = dir.path().join("icon_star.png");
    let icon_c = dir.path().join("icon_bell.png");
    DynamicImage::ImageRgba8(ImageBuffer::from_pixel(32, 32, Rgba([80, 120, 250, 255])))
        .save(&icon_a)
        .unwrap();
    DynamicImage::ImageRgba8(ImageBuffer::from_pixel(48, 48, Rgba([250, 180, 40, 255])))
        .save(&icon_b)
        .unwrap();
    DynamicImage::ImageRgba8(ImageBuffer::from_pixel(24, 24, Rgba([50, 200, 130, 255])))
        .save(&icon_c)
        .unwrap();

    let sheet_path = dir.path().join("sprites.png");
    let sprite_res = create_sprite_sheet(
        &[icon_a.clone(), icon_b.clone(), icon_c],
        &sheet_path,
        4,
        &ImageSettings {
            format: ImageFormat::Png,
            ..Default::default()
        },
        None,
    )
    .unwrap();
    assert_eq!(sprite_res.count, 3);
    assert!(Path::new(&sprite_res.image_path).exists());
    assert!(Path::new(&sprite_res.css_path).exists());
    assert!(sprite_res.css_snippet.contains(".sprite-icon-home"));
    assert!(sprite_res.css_snippet.contains(".sprite-icon-star"));

    // 2. Pure-Rust PDF Embedded Image Extractor
    let pdf_path = dir.path().join("embedded.pdf");
    merge_images_to_pdf(
        &[icon_a, icon_b.clone()],
        &pdf_path,
        &ImageSettings::default(),
        PdfPageFit::FitToImage,
        None,
    )
    .unwrap();
    let img_out_dir = dir.path().join("pdf_imgs");
    let extracted = extract_pdf_images(&pdf_path, &img_out_dir).unwrap();
    assert_eq!(extracted.len(), 2);
    assert!(extracted[0].ends_with(".jpg"));
    assert!(Path::new(&extracted[0]).exists());

    // 3. Audio Gain + Speed & Cover + Audio -> MP4 (if ffmpeg available)
    if let Some(ffmpeg) = find_bin("ffmpeg") {
        let ffprobe = find_bin("ffprobe");
        let hw = probe_hardware(Some(&ffmpeg), ffprobe.as_deref());

        let tone = dir.path().join("tone.wav");
        let st = Command::new(&ffmpeg)
            .args([
                "-y",
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=0.8",
            ])
            .arg(&tone)
            .status()
            .unwrap();
        assert!(st.success());

        let boosted = dir.path().join("boosted.mp3");
        let audio_settings = VideoSettings {
            target: VideoTarget::Audio(AudioFormat::Mp3),
            audio_gain_db: 4.0,
            speed: 1.5,
            ..Default::default()
        };
        let boosted_bytes = wandel_core::video_engine::convert_video(
            &tone,
            &boosted,
            &audio_settings,
            Some(0.8),
            &hw,
            &ffmpeg,
            2,
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        assert!(boosted_bytes > 500);

        let podcast_mp4 = dir.path().join("podcast.mp4");
        let pod_bytes =
            create_podcast_video(&tone, &icon_b, &podcast_mp4, 320, 240, &ffmpeg).unwrap();
        assert!(pod_bytes > 1000);
    }
}

#[test]
fn v18_auto_levels_pdf_scrub_and_media_split() {
    use wandel_core::image_engine::convert_image;
    use wandel_core::{
        inspect_pdf, merge_images_to_pdf, scrub_pdf_metadata, split_media_segments, PdfPageFit,
    };

    let dir = tempdir().unwrap();

    // 1. Auto-Levels Contrast Stretch on a washed-out (90..150) image
    let flat_img = dir.path().join("flat.png");
    DynamicImage::ImageRgba8(ImageBuffer::from_fn(100, 100, |x, _| {
        let v = (90 + (x * 60 / 99)) as u8;
        Rgba([v, v, v, 255])
    }))
    .save(&flat_img)
    .unwrap();

    let stretched = dir.path().join("stretched.png");
    convert_image(
        &flat_img,
        &stretched,
        &ImageSettings {
            format: ImageFormat::Png,
            auto_levels: true,
            ..Default::default()
        },
        None,
    )
    .unwrap();
    let out_rgba = image::open(&stretched).unwrap().to_rgba8();
    let dark_px = out_rgba.get_pixel(0, 0).0[0];
    let bright_px = out_rgba.get_pixel(99, 0).0[0];
    assert!(dark_px < 15, "Expected dark end stretched near 0, got {}", dark_px);
    assert!(
        bright_px > 240,
        "Expected bright end stretched near 255, got {}",
        bright_px
    );

    // 2. Lossless PDF Metadata Scrubber
    let pdf_path = dir.path().join("with_meta.pdf");
    merge_images_to_pdf(
        &[flat_img],
        &pdf_path,
        &ImageSettings::default(),
        PdfPageFit::A4,
        None,
    )
    .unwrap();
    // Inject /Info author metadata via lopdf
    let mut doc = lopdf::Document::load(&pdf_path).unwrap();
    let mut info = lopdf::Dictionary::new();
    info.set("Author", lopdf::Object::string_literal("Secret Author"));
    info.set("Title", lopdf::Object::string_literal("Confidential Report"));
    let info_id = doc.add_object(lopdf::Object::Dictionary(info));
    doc.trailer.set("Info", lopdf::Object::Reference(info_id));
    doc.save(&pdf_path).unwrap();

    let clean_pdf = dir.path().join("clean.pdf");
    let clean_bytes = scrub_pdf_metadata(&pdf_path, &clean_pdf).unwrap();
    assert!(clean_bytes > 500);
    assert_eq!(inspect_pdf(&clean_pdf).0, Some(1));
    let clean_doc = lopdf::Document::load(&clean_pdf).unwrap();
    assert!(clean_doc.trailer.get(b"Info").is_err());

    // 3. Lossless Media Segment Splitter (2.4s video -> multiple chunks at 1.0s)
    if let Some(ffmpeg) = find_bin("ffmpeg") {
        let clip = dir.path().join("long.mp4");
        let st = Command::new(&ffmpeg)
            .args([
                "-y",
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc=duration=2.4:size=160x120:rate=24",
                "-c:v",
                "libx264",
                "-g",
                "12",
                "-pix_fmt",
                "yuv420p",
            ])
            .arg(&clip)
            .status()
            .unwrap();
        assert!(st.success());

        let chunks_dir = dir.path().join("chunks");
        let parts = split_media_segments(&clip, &chunks_dir, 1.0, &ffmpeg).unwrap();
        assert!(parts.len() >= 2, "Expected >= 2 chunks, got {}", parts.len());
        assert!(Path::new(&parts[0]).exists());
    }
}

#[test]
fn v20_tiled_text_watermark() {
    use wandel_core::image_engine::convert_image;
    use wandel_core::{TextWatermarkSettings, WatermarkPosition};

    let dir = tempdir().unwrap();
    let src = dir.path().join("proof.png");
    DynamicImage::ImageRgba8(ImageBuffer::from_pixel(320, 200, Rgba([20, 30, 50, 255])))
        .save(&src)
        .unwrap();

    let out = dir.path().join("proof_tiled.png");
    let bytes = convert_image(
        &src,
        &out,
        &ImageSettings {
            format: ImageFormat::Png,
            text_watermark: Some(TextWatermarkSettings {
                text: "WANDEL PROOF".into(),
                font_size_percent: 5.0,
                color: "#ffffff".into(),
                opacity: 0.7,
                margin_px: 16,
                position: WatermarkPosition::Center,
                tiled: true,
            }),
            ..Default::default()
        },
        None,
    )
    .unwrap();
    assert!(bytes > 500);
    assert_eq!(image::image_dimensions(&out).unwrap(), (320, 200));
}

#[test]
fn v22_perf_cached_fonts_and_parallel_filters() {
    use wandel_core::image_engine::convert_image;
    use wandel_core::{ImageFilterPreset, TextWatermarkSettings, WatermarkPosition};

    let dir = tempdir().unwrap();
    let src = dir.path().join("hd.png");
    DynamicImage::ImageRgba8(ImageBuffer::from_fn(640, 360, |x, y| {
        Rgba([
            (80 + (x % 80)) as u8,
            (90 + (y % 70)) as u8,
            130,
            255,
        ])
    }))
    .save(&src)
    .unwrap();

    let settings = ImageSettings {
        format: ImageFormat::Webp,
        quality: 80,
        auto_levels: true,
        filter_preset: ImageFilterPreset::Warm,
        saturation: 15.0,
        vignette_strength: 0.3,
        corner_radius_percent: 4.0,
        text_watermark: Some(TextWatermarkSettings {
            text: "WANDEL FAST".into(),
            font_size_percent: 4.0,
            color: "#ffffff".into(),
            opacity: 0.7,
            margin_px: 12,
            position: WatermarkPosition::BottomRight,
            tiled: true,
        }),
        ..Default::default()
    };

    // First call warms OnceLock fontdb; subsequent calls reuse cached fontdb
    let out1 = dir.path().join("out1.webp");
    let out2 = dir.path().join("out2.webp");
    assert!(convert_image(&src, &out1, &settings, None).unwrap() > 500);
    let t0 = std::time::Instant::now();
    assert!(convert_image(&src, &out2, &settings, None).unwrap() > 500);
    let elapsed = t0.elapsed().as_millis();
    println!("Cached-font + parallel-filter 640x360 WebP took {} ms", elapsed);
}

fn run_video_target(
    meta: &wandel_core::FileMetadata,
    target: VideoTarget,
    resize: ResizeMode,
    hw: &wandel_core::HardwareCapabilities,
    ffmpeg: &Path,
    _dir: &Path,
) {
    let settings = ConversionSettings {
        video: VideoSettings {
            target,
            quality: 75,
            use_hardware_accel: true,
            resize,
            never_upscale: true,
            gif_fps: 12,
            audio_bitrate_kbps: 128,
            strip_metadata: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let jobs = vec![JobItem {
        id: meta.id.clone(),
        path: PathBuf::from(&meta.path),
        media_kind: meta.media_kind,
        duration_secs: meta.duration_secs,
        width: meta.width,
        height: meta.height,
        relative_dir: meta.relative_dir.clone(),
        image_override: None,
        video_override: None,
    }];

    let completed_path = Arc::new(Mutex::new(None::<String>));
    let completed_clone = Arc::clone(&completed_path);

    run_batch(
        jobs,
        &settings,
        hw,
        Some(ffmpeg),
        Arc::new(AtomicBool::new(false)),
        move |ev| match ev {
            ProgressEvent::FileCompleted { output_path, .. } => {
                *completed_clone.lock().unwrap() = Some(output_path);
            }
            ProgressEvent::FileFailed { error, .. } => {
                panic!("Video target {:?} failed: {}", target, error);
            }
            _ => {}
        },
    );

    let out = completed_path
        .lock()
        .unwrap()
        .clone()
        .expect("Expected FileCompleted event");
    assert!(Path::new(&out).exists(), "Output file missing: {}", out);
    assert!(std::fs::metadata(&out).unwrap().len() > 0);
}
