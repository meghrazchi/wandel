use crate::settings::OutputSettings;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone, Default)]
pub struct NamingContext<'a> {
    pub index: usize,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub relative_dir: Option<&'a str>,
}

/// Thread-safe output path allocator that guarantees no two parallel jobs
/// collide and existing files are never overwritten unless `overwrite` is true.
#[derive(Default)]
pub struct OutputPathAllocator {
    reserved: Mutex<HashSet<PathBuf>>,
}

impl OutputPathAllocator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allocate(
        &self,
        input: &Path,
        target_ext: &str,
        output: &OutputSettings,
    ) -> std::io::Result<PathBuf> {
        self.allocate_with_context(input, target_ext, output, &NamingContext::default())
    }

    pub fn allocate_with_context(
        &self,
        input: &Path,
        target_ext: &str,
        output: &OutputSettings,
        ctx: &NamingContext<'_>,
    ) -> std::io::Result<PathBuf> {
        let mut dir = match &output.output_dir {
            Some(custom) => custom.clone(),
            None => input
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from(".")),
        };

        if output.preserve_folder_structure && output.output_dir.is_some() {
            if let Some(rel) = ctx.relative_dir.filter(|r| !r.is_empty()) {
                dir = dir.join(rel);
            }
        }
        std::fs::create_dir_all(&dir)?;

        let stem = input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        let rendered_stem = if !output.rename_template.trim().is_empty() {
            render_template(&output.rename_template, stem, target_ext, ctx)
        } else {
            format!("{}{}", stem, output.suffix)
        };

        let mut reserved = self.reserved.lock().expect("allocator mutex poisoned");

        let is_taken = |candidate: &Path, reserved: &HashSet<PathBuf>| -> bool {
            if reserved.contains(candidate) {
                return true;
            }
            if candidate == input {
                return true;
            }
            if !output.overwrite && candidate.exists() {
                return true;
            }
            false
        };

        let first = dir.join(format!("{}.{}", rendered_stem, target_ext));
        if !is_taken(&first, &reserved) {
            reserved.insert(first.clone());
            return Ok(first);
        }

        let wandel_candidate = dir.join(format!("{}_wandel.{}", rendered_stem, target_ext));
        if !is_taken(&wandel_candidate, &reserved) {
            reserved.insert(wandel_candidate.clone());
            return Ok(wandel_candidate);
        }

        for idx in 1..10_000u32 {
            let candidate = dir.join(format!(
                "{}_wandel ({}).{}",
                rendered_stem, idx, target_ext
            ));
            if !is_taken(&candidate, &reserved) {
                reserved.insert(candidate.clone());
                return Ok(candidate);
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Could not find a unique output filename",
        ))
    }
}

fn render_template(template: &str, stem: &str, target_ext: &str, ctx: &NamingContext<'_>) -> String {
    let idx_1 = ctx.index + 1;
    let w = ctx.width.map(|v| v.to_string()).unwrap_or_default();
    let h = ctx.height.map(|v| v.to_string()).unwrap_or_default();

    let result = template
        .replace("{name}", stem)
        .replace("{index:03}", &format!("{:03}", idx_1))
        .replace("{index:02}", &format!("{:02}", idx_1))
        .replace("{index}", &idx_1.to_string())
        .replace("{width}", &w)
        .replace("{height}", &h)
        .replace("{format}", target_ext);

    // Sanitize path separators to avoid directory traversal
    let sanitized: String = result
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect();

    if sanitized.trim().is_empty() {
        stem.to_string()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn allocates_same_folder_and_avoids_overwrite() {
        let dir = tempdir().unwrap();
        let input = dir.path().join("photo.png");
        std::fs::write(&input, b"fake").unwrap();

        let alloc = OutputPathAllocator::new();
        let settings = OutputSettings::default();

        let p1 = alloc.allocate(&input, "webp", &settings).unwrap();
        assert_eq!(p1, dir.path().join("photo.webp"));

        let p2 = alloc.allocate(&input, "webp", &settings).unwrap();
        assert_eq!(p2, dir.path().join("photo_wandel.webp"));

        let p3 = alloc.allocate(&input, "webp", &settings).unwrap();
        assert_eq!(p3, dir.path().join("photo_wandel (1).webp"));

        let p4 = alloc.allocate(&input, "png", &settings).unwrap();
        assert_eq!(p4, dir.path().join("photo_wandel.png"));
    }

    #[test]
    fn renders_rename_template_and_preserves_folder_structure() {
        let dir = tempdir().unwrap();
        let out_root = dir.path().join("out");
        let input = dir.path().join("vacation").join("beach.jpg");
        std::fs::create_dir_all(input.parent().unwrap()).unwrap();
        std::fs::write(&input, b"img").unwrap();

        let alloc = OutputPathAllocator::new();
        let settings = OutputSettings {
            output_dir: Some(out_root.clone()),
            rename_template: "{index:03}_{name}_{width}x{height}".into(),
            preserve_folder_structure: true,
            ..Default::default()
        };
        let ctx = NamingContext {
            index: 4,
            width: Some(1920),
            height: Some(1080),
            relative_dir: Some("vacation"),
        };

        let out = alloc
            .allocate_with_context(&input, "webp", &settings, &ctx)
            .unwrap();
        assert_eq!(out, out_root.join("vacation").join("005_beach_1920x1080.webp"));
    }
}
