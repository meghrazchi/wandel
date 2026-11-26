use image::{DynamicImage, ImageBuffer, Rgba};
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::tempdir;

fn wandel_bin() -> &'static str {
    env!("CARGO_BIN_EXE_wandel")
}

#[test]
fn cli_end_to_end_subcommands_and_mcp_stdio() {
    let dir = tempdir().unwrap();

    let img_a = dir.path().join("a.png");
    let img_b = dir.path().join("b.png");
    DynamicImage::ImageRgba8(ImageBuffer::from_pixel(120, 80, Rgba([60, 120, 220, 255])))
        .save(&img_a)
        .unwrap();
    DynamicImage::ImageRgba8(ImageBuffer::from_pixel(120, 80, Rgba([220, 90, 60, 255])))
        .save(&img_b)
        .unwrap();

    // 1. info & inspect
    let info = Command::new(wandel_bin()).arg("info").output().unwrap();
    assert!(
        info.status.success(),
        "status: {:?}\nstderr: {}",
        info.status,
        String::from_utf8_lossy(&info.stderr)
    );
    assert!(String::from_utf8_lossy(&info.stdout).contains("Wandel CLI"));

    let insp = Command::new(wandel_bin())
        .args(["inspect", img_a.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(insp.status.success());

    // 2. convert (batch + flags)
    let out_conv = dir.path().join("conv");
    let conv = Command::new(wandel_bin())
        .args([
            "convert",
            img_a.to_str().unwrap(),
            img_b.to_str().unwrap(),
            "--to",
            "webp",
            "--quality",
            "80",
            "--auto-levels",
            "--filter",
            "warm",
            "-o",
            out_conv.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(conv.status.success(), "{}", String::from_utf8_lossy(&conv.stderr));
    assert!(out_conv.join("a.webp").exists());
    assert!(out_conv.join("b.webp").exists());

    // 3. PDF subcommands: pdf-merge -> pdf-rotate -> pdf-reorder -> pdf-stamp -> pdf-number -> pdf-clean -> pdf-images -> pdf-compress -> pdf-concat
    let pdf1 = dir.path().join("doc1.pdf");
    assert!(Command::new(wandel_bin())
        .args(["pdf-merge", img_a.to_str().unwrap(), img_b.to_str().unwrap(), "-o", pdf1.to_str().unwrap(), "--fit", "a4"])
        .status()
        .unwrap()
        .success());
    assert!(pdf1.exists());

    let pdf_rot = dir.path().join("rot.pdf");
    assert!(Command::new(wandel_bin())
        .args(["pdf-rotate", pdf1.to_str().unwrap(), "-o", pdf_rot.to_str().unwrap(), "--degrees", "90"])
        .status()
        .unwrap()
        .success());

    let pdf_reord = dir.path().join("reord.pdf");
    assert!(Command::new(wandel_bin())
        .args(["pdf-reorder", pdf1.to_str().unwrap(), "-o", pdf_reord.to_str().unwrap(), "--pages", "2,1"])
        .status()
        .unwrap()
        .success());

    let pdf_stamp = dir.path().join("stamp.pdf");
    assert!(Command::new(wandel_bin())
        .args(["pdf-stamp", pdf1.to_str().unwrap(), "-o", pdf_stamp.to_str().unwrap(), "--text", "DRAFT"])
        .status()
        .unwrap()
        .success());

    let pdf_num = dir.path().join("num.pdf");
    assert!(Command::new(wandel_bin())
        .args(["pdf-number", pdf1.to_str().unwrap(), "-o", pdf_num.to_str().unwrap()])
        .status()
        .unwrap()
        .success());

    let pdf_clean = dir.path().join("clean.pdf");
    assert!(Command::new(wandel_bin())
        .args(["pdf-clean", pdf1.to_str().unwrap(), "-o", pdf_clean.to_str().unwrap()])
        .status()
        .unwrap()
        .success());

    let pdf_imgs = dir.path().join("pdf_imgs");
    assert!(Command::new(wandel_bin())
        .args(["pdf-images", pdf1.to_str().unwrap(), "-o", pdf_imgs.to_str().unwrap()])
        .status()
        .unwrap()
        .success());

    let pdf_comp = dir.path().join("comp.pdf");
    assert!(Command::new(wandel_bin())
        .args(["pdf-compress", pdf1.to_str().unwrap(), "-o", pdf_comp.to_str().unwrap(), "--dpi", "72", "-q", "60"])
        .status()
        .unwrap()
        .success());

    let pdf_cat = dir.path().join("cat.pdf");
    assert!(Command::new(wandel_bin())
        .args(["pdf-concat", pdf1.to_str().unwrap(), pdf_clean.to_str().unwrap(), "-o", pdf_cat.to_str().unwrap()])
        .status()
        .unwrap()
        .success());

    // 4. Image tools: icon-pack, grid, compare, slice, sprite, palette, dedupe, qr, data-url, srcset
    let icons_dir = dir.path().join("icons");
    assert!(Command::new(wandel_bin())
        .args(["icon-pack", img_a.to_str().unwrap(), "-o", icons_dir.to_str().unwrap()])
        .status()
        .unwrap()
        .success());

    let grid_out = dir.path().join("grid.png");
    assert!(Command::new(wandel_bin())
        .args(["grid", img_a.to_str().unwrap(), img_b.to_str().unwrap(), "-o", grid_out.to_str().unwrap(), "--cols", "2"])
        .status()
        .unwrap()
        .success());

    let cmp_out = dir.path().join("cmp.png");
    assert!(Command::new(wandel_bin())
        .args(["compare", img_a.to_str().unwrap(), img_b.to_str().unwrap(), "-o", cmp_out.to_str().unwrap()])
        .status()
        .unwrap()
        .success());

    let slice_dir = dir.path().join("slices");
    assert!(Command::new(wandel_bin())
        .args(["slice", img_a.to_str().unwrap(), "-o", slice_dir.to_str().unwrap(), "--cols", "2", "--rows", "1"])
        .status()
        .unwrap()
        .success());

    let sprite_out = dir.path().join("sprites.png");
    assert!(Command::new(wandel_bin())
        .args(["sprite", img_a.to_str().unwrap(), img_b.to_str().unwrap(), "-o", sprite_out.to_str().unwrap()])
        .status()
        .unwrap()
        .success());

    assert!(Command::new(wandel_bin())
        .args(["palette", img_a.to_str().unwrap(), "-n", "3"])
        .status()
        .unwrap()
        .success());

    assert!(Command::new(wandel_bin())
        .args(["dedupe", img_a.to_str().unwrap(), img_b.to_str().unwrap()])
        .status()
        .unwrap()
        .success());

    let qr_out = dir.path().join("qr.svg");
    assert!(Command::new(wandel_bin())
        .args(["qr", "https://wandel.app", "-o", qr_out.to_str().unwrap()])
        .status()
        .unwrap()
        .success());

    let du = Command::new(wandel_bin())
        .args(["data-url", img_a.to_str().unwrap(), "-t", "webp"])
        .output()
        .unwrap();
    assert!(du.status.success());
    assert!(String::from_utf8_lossy(&du.stdout).starts_with("data:image/webp;base64,"));

    let srcset_dir = dir.path().join("srcset");
    assert!(Command::new(wandel_bin())
        .args(["srcset", img_a.to_str().unwrap(), "-o", srcset_dir.to_str().unwrap()])
        .status()
        .unwrap()
        .success());

    // 5. Pipeline recipe execution
    let recipe_path = dir.path().join("recipe.json");
    let piped_pdf = dir.path().join("piped.pdf");
    std::fs::write(
        &recipe_path,
        serde_json::json!({
            "steps": [
                {
                    "tool": "wandel_pdf_ops",
                    "arguments": {
                        "action": "merge_images",
                        "inputs": [img_a.to_string_lossy(), img_b.to_string_lossy()],
                        "output": piped_pdf.to_string_lossy()
                    }
                }
            ]
        })
        .to_string(),
    )
    .unwrap();
    assert!(Command::new(wandel_bin())
        .args(["pipeline", recipe_path.to_str().unwrap()])
        .status()
        .unwrap()
        .success());
    assert!(piped_pdf.exists());

    // 6. Headless MCP stdio handshake via real binary
    let mut child = Command::new(wandel_bin())
        .args(["mcp", "--allowed-dir", dir.path().to_str().unwrap()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(
                br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26"}}
{"jsonrpc":"2.0","id":2,"method":"ping"}
"#,
            )
            .unwrap();
    }
    let mcp_out = child.wait_with_output().unwrap();
    assert!(mcp_out.status.success());
    let stdout = String::from_utf8_lossy(&mcp_out.stdout);
    assert!(stdout.contains("wandel-mcp"));
}
