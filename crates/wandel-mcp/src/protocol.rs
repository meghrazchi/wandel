use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const PROTOCOL_VERSION: &str = "2025-03-26";
pub const SERVER_NAME: &str = "wandel-mcp";
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    Number(i64),
    String(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: Option<String>,
    pub id: Option<RequestId>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    pub id: RequestId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    pub fn ok(id: RequestId, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn err(id: RequestId, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

/// Returns the 6 cohesive, batch-first MCP tool definitions with strict JSON Schemas and MCP annotations.
pub fn tool_definitions() -> Value {
    json!([
        {
            "name": "wandel_info",
            "description": "Return cached host hardware capabilities (CPU cores, FFmpeg/FFprobe availability, GPU hardware encoders) and all supported image, video, audio, and PDF formats. Instant and read-only.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            },
            "annotations": {
                "title": "Wandel Hardware & Format Info",
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            }
        },
        {
            "name": "wandel_inspect",
            "description": "Inspect media files or entire folders in parallel: returns dimensions, duration, file size, media kind, PDF page count, and optionally detailed EXIF/codec metadata & GPS privacy warnings. Omit thumbnails by default to keep responses compact.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "inputs": {
                        "type": "array",
                        "items": { "type": "string" },
                        "minItems": 1,
                        "description": "Absolute file or directory paths to inspect."
                    },
                    "detailed": {
                        "type": "boolean",
                        "default": false,
                        "description": "Include detailed EXIF, GPS privacy warnings, and codec metadata entries."
                    },
                    "includeThumbnails": {
                        "type": "boolean",
                        "default": false,
                        "description": "Include base64 preview thumbnails (false by default to save context tokens)."
                    }
                },
                "required": ["inputs"],
                "additionalProperties": false
            },
            "annotations": {
                "title": "Inspect Media Files",
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            }
        },
        {
            "name": "wandel_convert",
            "description": "Batch-convert images, videos, audio, and PDFs in parallel across all CPU cores and GPU hardware encoders. Accepts local file/folder paths via 'inputs', OR inline chat-attached files via 'inputBase64' (raw base64 or data URL) with optional 'inputName' and 'returnBase64: true' so chat attachments can be converted without disk access.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "inputs": {
                        "type": "array",
                        "items": { "type": "string" },
                        "minItems": 1,
                        "description": "Absolute file or folder paths to convert (optional if inputBase64 is provided)."
                    },
                    "inputBase64": {
                        "type": "string",
                        "description": "Optional base64-encoded file bytes or data URL (e.g. 'data:image/png;base64,...') for converting chat-attached images or files without a disk path."
                    },
                    "inputName": {
                        "type": "string",
                        "description": "Optional filename with extension when using inputBase64 (e.g. 'screenshot.png'). Inferred from data URL MIME type if omitted."
                    },
                    "returnBase64": {
                        "type": "boolean",
                        "default": false,
                        "description": "Include base64 and dataUrl of converted outputs in the response (auto-enabled when using inputBase64 without outputDir)."
                    },
                    "to": {
                        "type": "string",
                        "description": "Target format: webp, jpg, png, avif, jxl, heic, gif, tiff, bmp, ico, mp4, h265, webm, av1, mov, mkv, avi, animated_webp, mp3, wav, aac, flac, opus."
                    },
                    "preset": {
                        "type": "string",
                        "enum": ["web", "high", "small"],
                        "description": "Quick preset applied before individual overrides."
                    },
                    "quality": { "type": "integer", "minimum": 1, "maximum": 100 },
                    "lossless": { "type": "boolean", "default": false },
                    "maxWidth": { "type": "integer", "minimum": 1 },
                    "maxHeight": { "type": "integer", "minimum": 1 },
                    "exactWidth": { "type": "integer", "minimum": 1, "description": "Exact target width in pixels (use with exactHeight)." },
                    "exactHeight": { "type": "integer", "minimum": 1, "description": "Exact target height in pixels (use with exactWidth)." },
                    "keepAspectRatio": { "type": "boolean", "default": true, "description": "Preserve aspect ratio when using exactWidth/exactHeight." },
                    "neverUpscale": { "type": "boolean", "default": true, "description": "Prevent enlarging media beyond its original dimensions." },
                    "percent": { "type": "number", "minimum": 1, "maximum": 400 },
                    "outputDir": { "type": "string", "description": "Optional output directory (defaults to same folder as each source file)." },
                    "suffix": { "type": "string", "default": "" },
                    "namingTemplate": { "type": "string", "description": "Optional filename template with tokens like {name}, {width}, {height}, {ext}." },
                    "preserveStructure": { "type": "boolean", "default": false, "description": "Preserve nested subfolder structure when outputDir is set." },
                    "overwrite": { "type": "boolean", "default": false },
                    "keepMetadata": { "type": "boolean", "default": false },
                    "noHw": { "type": "boolean", "default": false },
                    "threads": { "type": "integer", "minimum": 1 },
                    "targetKb": { "type": "integer", "minimum": 1, "description": "Target max image size in KB (binary searches quality)." },
                    "targetMb": { "type": "number", "minimum": 0.1, "description": "Target max video size in MB." },
                    "rotate": { "type": "integer", "enum": [0, 90, 180, 270] },
                    "flipH": { "type": "boolean", "default": false },
                    "flipV": { "type": "boolean", "default": false },
                    "crop": { "type": "string", "enum": ["1:1", "16:9", "9:16", "4:3", "3:2"] },
                    "padAspect": { "type": "string", "enum": ["1:1", "16:9", "9:16", "4:3", "3:2"], "description": "Pad image into aspect ratio without cropping." },
                    "customCrop": {
                        "type": "object",
                        "description": "Normalized custom crop rectangle (0.0..=1.0).",
                        "properties": {
                            "x": { "type": "number", "minimum": 0, "maximum": 1 },
                            "y": { "type": "number", "minimum": 0, "maximum": 1 },
                            "width": { "type": "number", "minimum": 0, "maximum": 1 },
                            "height": { "type": "number", "minimum": 0, "maximum": 1 }
                        },
                        "required": ["x", "y", "width", "height"]
                    },
                    "autoTrim": { "type": "boolean", "default": false },
                    "trimTolerance": { "type": "integer", "minimum": 0, "maximum": 64, "default": 12 },
                    "cornerRadiusPercent": { "type": "number", "minimum": 0, "maximum": 50, "default": 0 },
                    "grayscale": { "type": "boolean", "default": false },
                    "sharpen": { "type": "boolean", "default": false },
                    "autoLevels": { "type": "boolean", "default": false },
                    "blurSigma": { "type": "number", "minimum": 0, "maximum": 20, "default": 0 },
                    "pixelateBlock": { "type": "integer", "minimum": 0, "maximum": 32, "default": 0 },
                    "vignetteStrength": { "type": "number", "minimum": 0, "maximum": 1, "default": 0 },
                    "brightness": { "type": "integer", "minimum": -100, "maximum": 100, "default": 0 },
                    "contrast": { "type": "number", "minimum": -100, "maximum": 100, "default": 0 },
                    "saturation": { "type": "number", "minimum": -100, "maximum": 100, "default": 0 },
                    "backgroundHex": { "type": "string", "description": "Hex fill color for transparent pixels when encoding to JPG/BMP (default #ffffff)." },
                    "filter": { "type": "string", "enum": ["none", "sepia", "warm", "cool", "vintage", "bw", "invert"] },
                    "framePaddingPx": { "type": "integer", "minimum": 0, "maximum": 160, "default": 0 },
                    "frameGradient": { "type": "string", "enum": ["none", "indigo", "sunset", "ocean", "emerald", "midnight"] },
                    "dropShadow": { "type": "boolean", "default": false },
                    "watermark": {
                        "type": "object",
                        "description": "Optional PNG/SVG logo watermark overlay for images.",
                        "properties": {
                            "imagePath": { "type": "string" },
                            "opacity": { "type": "number", "minimum": 0.1, "maximum": 1.0, "default": 0.8 },
                            "scalePercent": { "type": "number", "minimum": 5, "maximum": 60, "default": 18 },
                            "marginPx": { "type": "integer", "minimum": 0, "default": 24 },
                            "position": { "type": "string", "enum": ["bottom_right", "bottom_left", "top_right", "top_left", "center"], "default": "bottom_right" }
                        },
                        "required": ["imagePath"]
                    },
                    "mute": { "type": "boolean", "default": false },
                    "fps": { "type": "integer", "minimum": 1, "maximum": 120 },
                    "gifFps": { "type": "integer", "minimum": 1, "maximum": 30, "default": 15 },
                    "audioBitrateKbps": { "type": "integer", "minimum": 32, "maximum": 512, "default": 192 },
                    "sampleRateHz": { "type": "integer", "minimum": 8000, "maximum": 96000 },
                    "mono": { "type": "boolean", "default": false },
                    "subtitlePath": { "type": "string", "description": "Optional .srt or .vtt subtitle file path to burn into video." },
                    "replaceAudioPath": { "type": "string", "description": "Optional external audio file path to replace or add as soundtrack." },
                    "fadeInSecs": { "type": "number", "minimum": 0, "maximum": 30 },
                    "fadeOutSecs": { "type": "number", "minimum": 0, "maximum": 30 },
                    "videoWatermark": {
                        "type": "object",
                        "description": "Optional PNG/SVG logo watermark overlay for videos.",
                        "properties": {
                            "imagePath": { "type": "string" },
                            "opacity": { "type": "number", "minimum": 0.1, "maximum": 1.0, "default": 0.8 },
                            "scalePercent": { "type": "number", "minimum": 5, "maximum": 60, "default": 18 },
                            "marginPx": { "type": "integer", "minimum": 0, "default": 24 },
                            "position": { "type": "string", "enum": ["bottom_right", "bottom_left", "top_right", "top_left", "center"], "default": "bottom_right" }
                        },
                        "required": ["imagePath"]
                    },
                    "trimStart": { "type": "number", "minimum": 0 },
                    "trimEnd": { "type": "number", "minimum": 0 },
                    "speed": { "type": "number", "minimum": 0.25, "maximum": 4.0, "default": 1.0 },
                    "gainDb": { "type": "number", "minimum": -20.0, "maximum": 20.0, "default": 0.0 },
                    "videoReframe": { "type": "string", "enum": ["none", "9:16", "4:5", "1:1", "16:9"] },
                    "playbackMode": { "type": "string", "enum": ["normal", "reverse", "boomerang"] },
                    "remux": { "type": "boolean", "default": false, "description": "Lossless container stream-copy (-c copy) with zero re-encoding." },
                    "denoiseAudio": { "type": "boolean", "default": false },
                    "normalizeAudio": { "type": "boolean", "default": false, "description": "Normalize audio loudness to EBU R128 (-16 LUFS)." },
                    "trimSilence": { "type": "boolean", "default": false, "description": "Trim leading and trailing silence on audio exports." },
                    "deinterlace": { "type": "boolean", "default": false },
                    "videoSharpen": { "type": "boolean", "default": false },
                    "pdfDpi": { "type": "integer", "minimum": 72, "maximum": 300, "default": 150, "description": "Rasterization DPI when converting PDF pages to images." },
                    "pdfPageRange": { "type": "string", "description": "1-based page range when converting PDF pages to images (e.g. '1-3,5')." },
                    "bundleZip": { "type": "boolean", "default": false },
                    "trashOriginals": { "type": "boolean", "default": false },
                    "textWatermark": { "type": "string", "description": "Optional text watermark overlay for images." },
                    "textWatermarkColor": { "type": "string", "default": "#ffffff" },
                    "textWatermarkOpacity": { "type": "number", "minimum": 0.1, "maximum": 1.0, "default": 0.75 },
                    "textWatermarkSizePercent": { "type": "number", "minimum": 2, "maximum": 12, "default": 4.5 },
                    "textWatermarkPosition": { "type": "string", "enum": ["bottom_right", "bottom_left", "top_right", "top_left", "center"], "default": "bottom_right" },
                    "tiledWatermark": { "type": "boolean", "default": false, "description": "Tile textWatermark diagonally across the entire image." },
                    "removeBgColor": { "type": "string", "description": "Remove solid background color ('auto' samples 4 corners, or '#RRGGBB')." },
                    "removeBgTolerance": { "type": "integer", "minimum": 0, "maximum": 120, "default": 24 },
                    "alsoFormats": {
                        "type": "array",
                        "description": "Additional image formats to export simultaneously from a single decode (e.g. ['avif', 'jpg']).",
                        "items": { "type": "string" }
                    },
                    "caption": { "type": "string", "description": "Optional bottom caption bar text rendered below images." },
                    "duotone": {
                        "type": "array",
                        "minItems": 2,
                        "maxItems": 2,
                        "items": { "type": "string" },
                        "description": "Optional duotone color grading pair [shadowHex, highlightHex]."
                    },
                    "blurRegions": {
                        "type": "array",
                        "description": "Optional normalized privacy redaction blur boxes [{x, y, width, height}] (0.0..=1.0).",
                        "items": {
                            "type": "object",
                            "properties": {
                                "x": { "type": "number", "minimum": 0, "maximum": 1 },
                                "y": { "type": "number", "minimum": 0, "maximum": 1 },
                                "width": { "type": "number", "minimum": 0, "maximum": 1 },
                                "height": { "type": "number", "minimum": 0, "maximum": 1 }
                            },
                            "required": ["x", "y", "width", "height"]
                        }
                    },
                    "dryRun": { "type": "boolean", "default": false, "description": "Preview planned outputs without writing or modifying any files." }
                },
                "additionalProperties": false
            },
            "annotations": {
                "title": "Parallel Batch Media Convert",
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": false
            }
        },
        {
            "name": "wandel_pdf_ops",
            "description": "100% pure-Rust offline PDF toolkit: merge images to PDF, concatenate PDFs, split/extract pages, compress, rotate, reorder, stamp watermark text, add page numbers, extract embedded images, or scrub identifying metadata.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": [
                            "merge_images",
                            "concat",
                            "split",
                            "compress",
                            "rotate",
                            "reorder",
                            "stamp",
                            "number",
                            "extract_images",
                            "scrub_metadata",
                            "extract_text",
                            "dark_mode",
                            "extract_markdown",
                            "two_up",
                            "set_metadata",
                            "remove_blank_pages",
                            "search",
                            "interleave",
                            "organize"
                        ]
                    },
                    "input": { "type": "string", "description": "Single input PDF path (for split, compress, rotate, reorder, organize, stamp, number, extract_images, scrub_metadata, search, remove_blank_pages, set_metadata)." },
                    "inputs": { "type": "array", "items": { "type": "string" }, "description": "Ordered input paths (for merge_images, concat, or interleave [odd, even])." },
                    "output": { "type": "string", "description": "Output PDF file path (or output directory for extract_images; omitted for search)." },
                    "pages": { "type": "string", "description": "1-based page range or ordered list, e.g. '1-3,5' or '3,1-2'." },
                    "plan": {
                        "type": "array",
                        "description": "Ordered page operations for action='organize' (reorder, rotate, duplicate, or omit pages in one lossless pass).",
                        "items": {
                            "type": "object",
                            "properties": {
                                "sourcePage": { "type": "integer", "minimum": 1 },
                                "rotate": { "type": "integer", "enum": [0, 90, 180, 270], "default": 0 }
                            },
                            "required": ["sourcePage"]
                        }
                    },
                    "degrees": { "type": "integer", "enum": [90, 180, 270], "default": 90 },
                    "dpi": { "type": "integer", "minimum": 72, "maximum": 300, "default": 120 },
                    "quality": { "type": "integer", "minimum": 25, "maximum": 95, "default": 70 },
                    "pageFit": { "type": "string", "enum": ["fit", "a4", "letter"], "default": "fit" },
                    "text": { "type": "string", "description": "Watermark stamp text (for stamp), page number format (for number), or search query (for search)." },
                    "title": { "type": "string", "description": "Document /Title metadata (for set_metadata)." },
                    "author": { "type": "string", "description": "Document /Author metadata (for set_metadata)." },
                    "subject": { "type": "string", "description": "Document /Subject metadata (for set_metadata)." },
                    "keywords": { "type": "string", "description": "Document /Keywords metadata (for set_metadata)." },
                    "threshold": { "type": "number", "minimum": 0.5, "maximum": 1.0, "default": 0.995, "description": "Blank-page near-white pixel ratio threshold for remove_blank_pages." },
                    "caseSensitive": { "type": "boolean", "default": false, "description": "Case-sensitive matching for action='search'." },
                    "reverseEven": { "type": "boolean", "default": false, "description": "Reverse the even PDF pages when interleaving a duplex scan." }
                },
                "required": ["action"],
                "additionalProperties": false
            },
            "annotations": {
                "title": "Pure-Rust PDF Operations",
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            }
        },
        {
            "name": "wandel_image_tools",
            "description": "Creative and developer image utilities: generate PWA/favicon icon packs, contact sheet grids, Before/After comparison collages, responsive srcset bundles, N×M grid slices, sprite sheets + CSS maps, dominant color palettes, perceptual dHash deduplication, QR codes, or Base64 Data URLs.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": [
                            "icon_pack",
                            "contact_sheet",
                            "compare",
                            "srcset",
                            "slice",
                            "sprite_sheet",
                            "palette",
                            "dedupe",
                            "qr",
                            "data_url",
                            "social_kit",
                            "make_gif",
                            "gif_frames",
                            "stitch",
                            "lqip",
                            "diff"
                        ]
                    },
                    "input": { "type": "string" },
                    "inputs": { "type": "array", "items": { "type": "string" } },
                    "output": { "type": "string", "description": "Output file or directory path depending on action." },
                    "format": { "type": "string", "default": "png" },
                    "quality": { "type": "integer", "minimum": 1, "maximum": 100, "default": 82 },
                    "cols": { "type": "integer", "minimum": 0, "default": 3 },
                    "rows": { "type": "integer", "minimum": 1, "default": 1 },
                    "cellSize": { "type": "integer", "minimum": 32, "default": 400 },
                    "gapPx": { "type": "integer", "minimum": 0, "default": 12 },
                    "vertical": { "type": "boolean", "default": false },
                    "leftLabel": { "type": "string" },
                    "rightLabel": { "type": "string" },
                    "count": { "type": "integer", "minimum": 1, "maximum": 16, "default": 6 },
                    "threshold": { "type": "integer", "minimum": 0, "maximum": 32, "default": 5 },
                    "content": { "type": "string", "description": "Text or URL for QR code generation." },
                    "darkHex": { "type": "string", "default": "#000000" },
                    "lightHex": { "type": "string", "default": "#ffffff" },
                    "sizePx": { "type": "integer", "minimum": 16, "maximum": 2048, "default": 512 },
                    "delayMs": { "type": "integer", "minimum": 20, "maximum": 10000, "default": 300, "description": "Frame delay in milliseconds for make_gif." },
                    "maxWidth": { "type": "integer", "minimum": 16, "description": "Optional max width for make_gif." },
                    "direction": { "type": "string", "enum": ["vertical", "horizontal"], "default": "vertical", "description": "Stitch direction for action='stitch'." },
                    "padding": { "type": "integer", "minimum": 0, "default": 0, "description": "Gap in pixels between stitched images." }
                },
                "required": ["action"],
                "additionalProperties": false
            },
            "annotations": {
                "title": "Image Dev & Creative Utilities",
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            }
        },
        {
            "name": "wandel_media_tools",
            "description": "FFmpeg-powered video and audio utilities: merge videos, merge audio tracks, generate 3×3 video storyboards, extract frame sequences at target FPS, render audio waveform PNGs, build image-to-video slideshows, create Cover+Audio audiogram MP4s, or split media losslessly into equal-duration chunks.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": [
                            "merge_videos",
                            "merge_audio",
                            "storyboard",
                            "extract_frames",
                            "waveform",
                            "slideshow",
                            "audiogram",
                            "split_segments",
                            "poster"
                        ]
                    },
                    "input": { "type": "string" },
                    "inputs": { "type": "array", "items": { "type": "string" } },
                    "cover": { "type": "string", "description": "Cover image path for audiogram." },
                    "audio": { "type": "string", "description": "Optional soundtrack path for slideshow." },
                    "output": { "type": "string", "description": "Output file or directory path." },
                    "fps": { "type": "number", "minimum": 0.1, "maximum": 60.0, "default": 1.0 },
                    "format": { "type": "string", "default": "png" },
                    "frames": { "type": "integer", "minimum": 1, "maximum": 36, "default": 9 },
                    "cols": { "type": "integer", "minimum": 1, "maximum": 6, "default": 3 },
                    "seconds": { "type": "number", "minimum": 0.5, "default": 3.0 },
                    "width": { "type": "integer", "minimum": 160, "default": 1280 },
                    "height": { "type": "integer", "minimum": 80, "default": 720 },
                    "colorHex": { "type": "string", "default": "#6366f1" },
                    "bgHex": { "type": "string", "default": "#18181b" }
                },
                "required": ["action", "output"],
                "additionalProperties": false
            },
            "annotations": {
                "title": "Video & Audio Media Utilities",
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            }
        },
        {
            "name": "wandel_pipeline",
            "description": "Execute a multi-step dependent media pipeline in a single call with zero intermediate LLM round-trips. Reference previous step outputs using '$prev' or '$step.0', '$step.1', etc. inside 'input' or 'inputs'.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "steps": {
                        "type": "array",
                        "minItems": 1,
                        "items": {
                            "type": "object",
                            "properties": {
                                "tool": {
                                    "type": "string",
                                    "enum": ["wandel_convert", "wandel_pdf_ops", "wandel_image_tools", "wandel_media_tools"]
                                },
                                "arguments": { "type": "object" }
                            },
                            "required": ["tool", "arguments"]
                        }
                    }
                },
                "required": ["steps"],
                "additionalProperties": false
            },
            "annotations": {
                "title": "Multi-Step Composable Media Pipeline",
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": false
            }
        }
    ])
}

pub fn resource_definitions() -> Value {
    json!([
        {
            "uri": "wandel://capabilities",
            "name": "Wandel Capabilities & Hardware",
            "description": "Detected CPU cores, GPU hardware encoders, FFmpeg status, supported formats, and built-in presets.",
            "mimeType": "application/json"
        }
    ])
}

pub fn prompt_definitions() -> Value {
    json!([
        {
            "name": "optimize-for-web",
            "description": "Optimize a folder of images into lightweight WebP/AVIF assets and responsive srcset bundles.",
            "arguments": [
                { "name": "path", "description": "File or folder path to optimize", "required": true }
            ]
        },
        {
            "name": "prepare-social-video",
            "description": "Reframe a landscape video to 9:16 with a softly blurred background and denoised audio.",
            "arguments": [
                { "name": "path", "description": "Input video file path", "required": true }
            ]
        },
        {
            "name": "clean-pdf-for-sharing",
            "description": "Scrub identifying metadata, add page numbers, and optionally compress a PDF before external sharing.",
            "arguments": [
                { "name": "path", "description": "Input PDF path", "required": true }
            ]
        }
    ])
}

pub fn get_prompt(name: &str, args: &Value) -> Option<Value> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .unwrap_or("<path>");
    match name {
        "optimize-for-web" => Some(json!({
            "description": "Optimize media assets for fast web delivery using Wandel.",
            "messages": [{
                "role": "user",
                "content": {
                    "type": "text",
                    "text": format!(
                        "Inspect '{}' with wandel_inspect, then batch-convert images in parallel using wandel_convert with preset='web', to='webp', quality=82, maxWidth=1920, and summarize total bytes saved.",
                        path
                    )
                }
            }]
        })),
        "prepare-social-video" => Some(json!({
            "description": "Convert a landscape clip into a 9:16 vertical social video with blurred background.",
            "messages": [{
                "role": "user",
                "content": {
                    "type": "text",
                    "text": format!(
                        "Convert '{}' using wandel_convert with to='mp4', videoReframe='9:16', denoiseAudio=true, and report the output path and size.",
                        path
                    )
                }
            }]
        })),
        "clean-pdf-for-sharing" => Some(json!({
            "description": "Strip metadata and add page numbers to a PDF.",
            "messages": [{
                "role": "user",
                "content": {
                    "type": "text",
                    "text": format!(
                        "Use wandel_pdf_ops with action='scrub_metadata' on '{}', then add page numbers with action='number' and text='Page {{n}} of {{total}}'.",
                        path
                    )
                }
            }]
        })),
        _ => None,
    }
}
