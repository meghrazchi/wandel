use crate::error::{Result, WandelError};
use crate::formats::{AudioFormat, VideoFormat, VideoTarget};
use crate::settings::{
    ResizeMode, VideoAspectReframe, VideoPlaybackMode, VideoSettings, WatermarkPosition,
};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareCapabilities {
    pub ffmpeg_available: bool,
    pub ffprobe_available: bool,
    pub h264_hw_encoder: Option<String>,
    pub h265_hw_encoder: Option<String>,
    pub av1_hw_encoder: Option<String>,
    pub gpu_backend_name: Option<String>,
    pub cpu_cores: usize,
}

impl HardwareCapabilities {
    pub fn has_any_gpu_encoder(&self) -> bool {
        self.h264_hw_encoder.is_some()
            || self.h265_hw_encoder.is_some()
            || self.av1_hw_encoder.is_some()
    }
}

/// Probes available FFmpeg/FFprobe binaries and verifies working GPU hardware encoders.
pub fn probe_hardware(
    ffmpeg_path: Option<&Path>,
    ffprobe_path: Option<&Path>,
) -> HardwareCapabilities {
    let cpu_cores = num_cpus::get();
    let ffmpeg_available = ffmpeg_path
        .map(|p| {
            Command::new(p)
                .arg("-version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        })
        .unwrap_or(false);

    let ffprobe_available = ffprobe_path
        .map(|p| {
            Command::new(p)
                .arg("-version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        })
        .unwrap_or(false);

    let mut caps = HardwareCapabilities {
        ffmpeg_available,
        ffprobe_available,
        cpu_cores,
        ..Default::default()
    };

    let Some(ffmpeg) = ffmpeg_path.filter(|_| ffmpeg_available) else {
        return caps;
    };

    let encoders_out = Command::new(ffmpeg)
        .args(["-hide_banner", "-encoders"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let h264_candidates: &[(&str, &str)] = if cfg!(target_os = "macos") {
        &[("h264_videotoolbox", "Apple VideoToolbox")]
    } else if cfg!(target_os = "windows") {
        &[
            ("h264_nvenc", "NVIDIA NVENC"),
            ("h264_qsv", "Intel QuickSync"),
            ("h264_amf", "AMD AMF"),
        ]
    } else {
        &[
            ("h264_nvenc", "NVIDIA NVENC"),
            ("h264_vaapi", "VAAPI"),
            ("h264_qsv", "Intel QuickSync"),
        ]
    };

    for (enc, label) in h264_candidates {
        if encoders_out.contains(enc) && test_encoder(ffmpeg, enc) {
            caps.h264_hw_encoder = Some((*enc).to_string());
            caps.gpu_backend_name = Some((*label).to_string());
            break;
        }
    }

    let h265_candidates: &[(&str, &str)] = if cfg!(target_os = "macos") {
        &[("hevc_videotoolbox", "Apple VideoToolbox")]
    } else if cfg!(target_os = "windows") {
        &[
            ("hevc_nvenc", "NVIDIA NVENC"),
            ("hevc_qsv", "Intel QuickSync"),
            ("hevc_amf", "AMD AMF"),
        ]
    } else {
        &[
            ("hevc_nvenc", "NVIDIA NVENC"),
            ("hevc_vaapi", "VAAPI"),
            ("hevc_qsv", "Intel QuickSync"),
        ]
    };

    for (enc, label) in h265_candidates {
        if encoders_out.contains(enc) && test_encoder(ffmpeg, enc) {
            caps.h265_hw_encoder = Some((*enc).to_string());
            if caps.gpu_backend_name.is_none() {
                caps.gpu_backend_name = Some((*label).to_string());
            }
            break;
        }
    }

    let av1_candidates: &[&str] = &["av1_nvenc", "av1_qsv", "av1_vaapi", "av1_amf"];
    for enc in av1_candidates {
        if encoders_out.contains(enc) && test_encoder(ffmpeg, enc) {
            caps.av1_hw_encoder = Some((*enc).to_string());
            break;
        }
    }

    caps
}

fn test_encoder(ffmpeg: &Path, encoder: &str) -> bool {
    Command::new(ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=c=black:s=64x64:d=0.1",
            "-frames:v",
            "1",
            "-c:v",
            encoder,
            "-f",
            "null",
            "-",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Inspects video/audio dimensions, duration, and generates a thumbnail for videos.
pub fn inspect_video(
    path: &Path,
    ffprobe_path: Option<&Path>,
    ffmpeg_path: Option<&Path>,
    is_audio_only: bool,
) -> (Option<u32>, Option<u32>, Option<f64>, Option<String>) {
    let mut width = None;
    let mut height = None;
    let mut duration = None;

    if let Some(ffprobe) = ffprobe_path {
        if let Ok(out) = Command::new(ffprobe)
            .args([
                "-v",
                "quiet",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
            ])
            .arg(path)
            .output()
        {
            if out.status.success() {
                if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                    if let Some(dur_str) = json["format"]["duration"].as_str() {
                        duration = dur_str.parse::<f64>().ok();
                    }
                    if let Some(streams) = json["streams"].as_array() {
                        for stream in streams {
                            if stream["codec_type"].as_str() == Some("video") {
                                width = stream["width"].as_u64().map(|v| v as u32);
                                height = stream["height"].as_u64().map(|v| v as u32);
                                if duration.is_none() {
                                    duration = stream["duration"]
                                        .as_str()
                                        .and_then(|s| s.parse::<f64>().ok());
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    let thumb = if !is_audio_only {
        ffmpeg_path.and_then(|ffmpeg| extract_video_thumbnail(path, ffmpeg))
    } else {
        None
    };

    (width, height, duration, thumb)
}

fn extract_video_thumbnail(path: &Path, ffmpeg: &Path) -> Option<String> {
    let out = Command::new(ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-ss", "0.2", "-i"])
        .arg(path)
        .args([
            "-frames:v",
            "1",
            "-vf",
            "scale=96:-2",
            "-f",
            "image2pipe",
            "-vcodec",
            "mjpeg",
            "-q:v",
            "6",
            "pipe:1",
        ])
        .output()
        .ok()?;

    if out.status.success() && !out.stdout.is_empty() {
        let b64 = base64::engine::general_purpose::STANDARD.encode(&out.stdout);
        Some(format!("data:image/jpeg;base64,{}", b64))
    } else {
        None
    }
}

/// Converts a video or extracts audio using FFmpeg, streaming live progress `0.0..=100.0`.
#[allow(clippy::too_many_arguments)]
pub fn convert_video<F>(
    input: &Path,
    output: &Path,
    settings: &VideoSettings,
    duration_secs: Option<f64>,
    hw: &HardwareCapabilities,
    ffmpeg_path: &Path,
    threads: usize,
    cancel_flag: &AtomicBool,
    mut on_progress: F,
) -> Result<u64>
where
    F: FnMut(f32),
{
    if cancel_flag.load(Ordering::Relaxed) {
        return Err(WandelError::Cancelled);
    }

    let effective_duration = compute_effective_duration(
        duration_secs,
        settings.trim_start_secs,
        settings.trim_end_secs,
    );

    if matches!(settings.target, VideoTarget::Video(VideoFormat::AnimatedWebp)) {
        return convert_video_to_animated_webp(
            input,
            output,
            settings,
            ffmpeg_path,
            cancel_flag,
            on_progress,
        );
    }

    let want_hw = settings.use_hardware_accel
        && !settings.remux_only
        && settings.target_size_mb.is_none()
        && target_supports_hw(&settings.target, hw);
    let args = build_ffmpeg_args(
        input,
        output,
        settings,
        effective_duration,
        hw,
        want_hw,
        threads,
    );

    let first_attempt = run_ffmpeg_with_progress(
        ffmpeg_path,
        &args,
        effective_duration,
        cancel_flag,
        &mut on_progress,
    );

    match first_attempt {
        Ok(()) => {
            let size = std::fs::metadata(output)?.len();
            Ok(size)
        }
        Err(WandelError::Cancelled) => {
            let _ = std::fs::remove_file(output);
            Err(WandelError::Cancelled)
        }
        Err(hw_err) if want_hw => {
            // Graceful automatic fallback to CPU software encoder
            let _ = std::fs::remove_file(output);
            let sw_args = build_ffmpeg_args(
                input,
                output,
                settings,
                effective_duration,
                hw,
                false,
                threads,
            );
            run_ffmpeg_with_progress(
                ffmpeg_path,
                &sw_args,
                effective_duration,
                cancel_flag,
                &mut on_progress,
            )
            .map_err(|sw_err| {
                let _ = std::fs::remove_file(output);
                WandelError::FfmpegFailed(format!(
                    "Hardware encode failed ({}); CPU fallback also failed ({})",
                    hw_err, sw_err
                ))
            })?;
            let size = std::fs::metadata(output)?.len();
            Ok(size)
        }
        Err(err) => {
            let _ = std::fs::remove_file(output);
            Err(err)
        }
    }
}

fn compute_effective_duration(
    duration_secs: Option<f64>,
    trim_start: Option<f64>,
    trim_end: Option<f64>,
) -> Option<f64> {
    let start = trim_start.unwrap_or(0.0).max(0.0);
    match (duration_secs, trim_end) {
        (_, Some(end)) if end > start => Some(end - start),
        (Some(dur), None) if dur > start => Some(dur - start),
        (d, _) => d,
    }
}

fn target_supports_hw(target: &VideoTarget, hw: &HardwareCapabilities) -> bool {
    match target {
        VideoTarget::Video(VideoFormat::Mp4H264 | VideoFormat::Mov | VideoFormat::Mkv) => {
            hw.h264_hw_encoder.is_some()
        }
        VideoTarget::Video(VideoFormat::Mp4H265) => hw.h265_hw_encoder.is_some(),
        VideoTarget::Video(VideoFormat::WebmAv1) => hw.av1_hw_encoder.is_some(),
        VideoTarget::Video(
            VideoFormat::WebmVp9 | VideoFormat::Avi | VideoFormat::Gif | VideoFormat::AnimatedWebp,
        )
        | VideoTarget::Audio(_) => false,
    }
}

fn build_scale_filter(mode: &ResizeMode, never_upscale: bool) -> Option<String> {
    match *mode {
        ResizeMode::Original => None,
        ResizeMode::Percentage { percent } => {
            let f = (percent / 100.0).clamp(0.05, 4.0);
            if never_upscale && f >= 1.0 {
                return None;
            }
            Some(format!(
                "scale=trunc(iw*{:.4}/2)*2:trunc(ih*{:.4}/2)*2:flags=lanczos",
                f, f
            ))
        }
        ResizeMode::MaxDimensions { width, height } => match (width, height) {
            (None, None) => None,
            (Some(w), None) => {
                if never_upscale {
                    Some(format!(
                        "scale='min({},iw)':-2:flags=lanczos",
                        w.max(2)
                    ))
                } else {
                    Some(format!("scale={}:-2:flags=lanczos", (w / 2 * 2).max(2)))
                }
            }
            (None, Some(h)) => {
                if never_upscale {
                    Some(format!(
                        "scale=-2:'min({},ih)':flags=lanczos",
                        h.max(2)
                    ))
                } else {
                    Some(format!("scale=-2:{}:flags=lanczos", (h / 2 * 2).max(2)))
                }
            }
            (Some(w), Some(h)) => {
                let suffix =
                    ":force_original_aspect_ratio=decrease,scale=trunc(iw/2)*2:trunc(ih/2)*2";
                if never_upscale {
                    Some(format!(
                        "scale='min({},iw)':'min({},ih)'{}",
                        w.max(2),
                        h.max(2),
                        suffix
                    ))
                } else {
                    Some(format!("scale={}:{}{}", w.max(2), h.max(2), suffix))
                }
            }
        },
        ResizeMode::Exact {
            width,
            height,
            keep_aspect_ratio,
        } => {
            let w = (width / 2 * 2).max(2);
            let h = (height / 2 * 2).max(2);
            if keep_aspect_ratio {
                Some(format!(
                    "scale={}:{}:force_original_aspect_ratio=decrease:flags=lanczos,scale=trunc(iw/2)*2:trunc(ih/2)*2",
                    w, h
                ))
            } else {
                Some(format!("scale={}:{}:flags=lanczos", w, h))
            }
        }
    }
}

fn build_ffmpeg_args(
    input: &Path,
    output: &Path,
    settings: &VideoSettings,
    effective_duration: Option<f64>,
    hw: &HardwareCapabilities,
    use_hw: bool,
    threads: usize,
) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "-y".into(),
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-progress".into(),
        "pipe:1".into(),
        "-nostats".into(),
    ];

    if let Some(start) = settings.trim_start_secs.filter(|&s| s > 0.0) {
        args.extend(["-ss".into(), format!("{:.3}", start)]);
    }

    args.extend(["-i".into(), input.to_string_lossy().into_owned()]);

    let has_replace_audio = settings
        .replace_audio_path
        .as_ref()
        .is_some_and(|p| p.is_file());
    let has_watermark = matches!(settings.target, VideoTarget::Video(_))
        && settings
            .watermark
            .as_ref()
            .is_some_and(|w| w.image_path.is_file());

    let mut next_input_idx = 1usize;
    let mut replace_audio_idx = None;
    let mut wm_input_idx = None;

    if let Some(audio_path) = settings
        .replace_audio_path
        .as_ref()
        .filter(|p| p.is_file())
    {
        args.extend(["-i".into(), audio_path.to_string_lossy().into_owned()]);
        replace_audio_idx = Some(next_input_idx);
        next_input_idx += 1;
    }

    if has_watermark {
        if let Some(wm) = &settings.watermark {
            args.extend(["-i".into(), wm.image_path.to_string_lossy().into_owned()]);
            wm_input_idx = Some(next_input_idx);
        }
    }

    let uses_filter_complex = matches!(settings.target, VideoTarget::Video(fmt) if !matches!(fmt, VideoFormat::Gif | VideoFormat::AnimatedWebp))
        && (wm_input_idx.is_some()
            || settings.aspect_reframe != VideoAspectReframe::None
            || settings.playback_mode == VideoPlaybackMode::Boomerang);

    if let Some(audio_idx) = replace_audio_idx {
        if !uses_filter_complex {
            args.extend([
                "-map".into(),
                "0:v:0".into(),
                "-map".into(),
                format!("{}:a:0", audio_idx),
                "-shortest".into(),
            ]);
        }
    }

    if let Some(end) = settings.trim_end_secs {
        let start = settings.trim_start_secs.unwrap_or(0.0).max(0.0);
        if end > start {
            args.extend(["-t".into(), format!("{:.3}", end - start)]);
        }
    }

    if threads > 0 {
        args.push("-threads".into());
        args.push(threads.to_string());
    }

    if settings.strip_metadata {
        args.extend(["-map_metadata".into(), "-1".into()]);
    }

    let q = settings.quality.clamp(1, 100);
    // Map quality 100..1 to CRF 16..38
    let crf = (38.0 - (q as f32 / 100.0) * 22.0).round() as u32;
    let audio_br_kbps = settings.audio_bitrate_kbps.clamp(64, 320);
    let audio_br = format!("{}k", audio_br_kbps);

    // Optional target file size bitrate calculation (kbps)
    let target_video_kbps: Option<u32> = settings.target_size_mb.and_then(|mb| {
        if mb <= 0.0 {
            return None;
        }
        let dur = effective_duration.unwrap_or(10.0).max(0.5);
        // Reserve ~5% for container overhead
        let total_kbps = (f64::from(mb) * 8192.0 * 0.95) / dur;
        let audio_reserve = if settings.mute_audio {
            0.0
        } else {
            f64::from(audio_br_kbps)
        };
        let video_kbps = (total_kbps - audio_reserve).clamp(150.0, 50_000.0);
        Some(video_kbps.round() as u32)
    });

    match settings.target {
        VideoTarget::Audio(audio_fmt) => {
            args.push("-vn".into());
            let mut afilters = Vec::new();
            if settings.denoise_audio {
                afilters.push("highpass=f=80,lowpass=f=12000,afftdn=nf=-25".to_string());
            }
            let speed = settings.speed.clamp(0.25, 4.0);
            if (speed - 1.0).abs() > 0.01 {
                let mut rem = speed;
                while rem > 2.0 {
                    afilters.push("atempo=2.0".to_string());
                    rem /= 2.0;
                }
                while rem < 0.5 {
                    afilters.push("atempo=0.5".to_string());
                    rem *= 2.0;
                }
                afilters.push(format!("atempo={:.3}", rem));
            }
            if settings.audio_gain_db.abs() > 0.1 {
                afilters.push(format!(
                    "volume={:.1}dB",
                    settings.audio_gain_db.clamp(-20.0, 20.0)
                ));
            }
            if let Some(fi) = settings.fade_in_secs.filter(|&v| v > 0.0) {
                afilters.push(format!("afade=t=in:st=0:d={:.2}", fi));
            }
            if let (Some(fo), Some(dur)) = (
                settings.fade_out_secs.filter(|&v| v > 0.0),
                effective_duration,
            ) {
                let st = (dur - f64::from(fo)).max(0.0);
                afilters.push(format!("afade=t=out:st={:.2}:d={:.2}", st, fo));
            }
            if settings.trim_silence {
                afilters.push(
                    "silenceremove=start_periods=1:start_threshold=-45dB:stop_periods=-1:stop_threshold=-45dB"
                        .to_string(),
                );
            }
            if settings.normalize_loudness {
                afilters.push("loudnorm".to_string());
            }
            if !afilters.is_empty() {
                args.extend(["-af".into(), afilters.join(",")]);
            }
            if let Some(sr) = settings.sample_rate_hz {
                args.extend(["-ar".into(), sr.to_string()]);
            }
            if settings.mono {
                args.extend(["-ac".into(), "1".into()]);
            }
            match audio_fmt {
                AudioFormat::Mp3 => {
                    args.extend(["-c:a".into(), "libmp3lame".into(), "-b:a".into(), audio_br]);
                }
                AudioFormat::Wav => {
                    args.extend(["-c:a".into(), "pcm_s16le".into()]);
                }
                AudioFormat::Aac => {
                    args.extend(["-c:a".into(), "aac".into(), "-b:a".into(), audio_br]);
                }
                AudioFormat::Flac => {
                    args.extend(["-c:a".into(), "flac".into()]);
                }
                AudioFormat::Opus => {
                    args.extend(["-c:a".into(), "libopus".into(), "-b:a".into(), audio_br]);
                }
            }
        }
        VideoTarget::Video(VideoFormat::Gif) => {
            let fps = settings.max_fps.unwrap_or(settings.gif_fps).clamp(5, 30);
            let scale = build_scale_filter(&settings.resize, settings.never_upscale)
                .unwrap_or_else(|| "scale='min(720,iw)':-1:flags=lanczos".into());
            let filter = format!(
                "fps={},{},split[s0][s1];[s0]palettegen=stats_mode=diff[p];[s1][p]paletteuse=dither=bayer:bayer_scale=3",
                fps, scale
            );
            args.extend(["-vf".into(), filter, "-loop".into(), "0".into()]);
        }
        VideoTarget::Video(VideoFormat::AnimatedWebp) => {
            let fps = settings.max_fps.unwrap_or(settings.gif_fps).clamp(5, 30);
            let mut filters = vec![format!("fps={}", fps)];
            if let Some(scale) = build_scale_filter(&settings.resize, settings.never_upscale) {
                filters.push(scale);
            } else {
                filters.push("scale='min(960,iw)':-2:flags=lanczos".into());
            }
            args.extend([
                "-vf".into(),
                filters.join(","),
                "-c:v".into(),
                "libwebp".into(),
                "-lossless".into(),
                "0".into(),
                "-q:v".into(),
                q.to_string(),
                "-loop".into(),
                "0".into(),
                "-preset".into(),
                "default".into(),
                "-an".into(),
            ]);
        }
        VideoTarget::Video(video_fmt) => {
            let mut vfilters = Vec::new();
            if settings.deinterlace {
                vfilters.push("yadif=mode=0:parity=-1:deint=1".into());
            }
            let speed = settings.speed.clamp(0.25, 4.0);
            if (speed - 1.0).abs() > 0.01 {
                vfilters.push(format!("setpts={:.4}*PTS", 1.0 / speed));
            }
            match settings.rotate % 360 {
                90 => vfilters.push("transpose=1".into()),
                180 => vfilters.push("transpose=2,transpose=2".into()),
                270 => vfilters.push("transpose=2".into()),
                _ => {}
            }
            if settings.flip_horizontal {
                vfilters.push("hflip".into());
            }
            if let Some(fps) = settings.max_fps.filter(|&f| f > 0) {
                vfilters.push(format!("fps={}", fps));
            }
            if settings.aspect_reframe == VideoAspectReframe::None {
                if let Some(vf) = build_scale_filter(&settings.resize, settings.never_upscale) {
                    vfilters.push(vf);
                }
            }
            if settings.video_sharpen {
                vfilters.push("unsharp=5:5:0.8:5:5:0.0".into());
            }
            if let Some(sub_path) = settings.subtitle_path.as_ref().filter(|p| p.is_file()) {
                // Escape colons and backslashes for FFmpeg filtergraph syntax
                let escaped = sub_path
                    .to_string_lossy()
                    .replace('\\', "\\\\")
                    .replace(':', "\\:")
                    .replace('\'', "\\'");
                vfilters.push(format!("subtitles='{}'", escaped));
            }
            if settings.playback_mode == VideoPlaybackMode::Reverse {
                vfilters.push("reverse".into());
            }

            let has_reframe = settings.aspect_reframe != VideoAspectReframe::None;
            let has_boomerang = settings.playback_mode == VideoPlaybackMode::Boomerang;

            if has_reframe || has_boomerang || wm_input_idx.is_some() {
                let mut stages: Vec<String> = Vec::new();
                let mut cur = "[0:v]".to_string();

                if !vfilters.is_empty() {
                    stages.push(format!("{}{}[vpre]", cur, vfilters.join(",")));
                    cur = "[vpre]".to_string();
                }

                if let Some((mut rw, mut rh)) = settings.aspect_reframe.target_dimensions() {
                    if let ResizeMode::MaxDimensions { width, height } = settings.resize {
                        let mut scale = 1.0f64;
                        if let Some(mw) = width {
                            scale = scale.min(f64::from(mw) / f64::from(rw));
                        }
                        if let Some(mh) = height {
                            scale = scale.min(f64::from(mh) / f64::from(rh));
                        }
                        if scale < 1.0 {
                            rw = (((f64::from(rw) * scale) / 2.0).round() as u32 * 2).max(2);
                            rh = (((f64::from(rh) * scale) / 2.0).round() as u32 * 2).max(2);
                        }
                    }
                    stages.push(format!(
                        "{cur}split[rf_bg][rf_fg];[rf_bg]scale={rw}:{rh}:force_original_aspect_ratio=increase,crop={rw}:{rh},gblur=sigma=24[rf_bg];[rf_fg]scale={rw}:{rh}:force_original_aspect_ratio=decrease[rf_fg];[rf_bg][rf_fg]overlay=(W-w)/2:(H-h)/2,setsar=1[vrf]"
                    ));
                    cur = "[vrf]".to_string();
                }

                if let Some(wm_idx) = wm_input_idx {
                    let wm = settings.watermark.as_ref().unwrap();
                    let margin = wm.margin_px;
                    let (ox, oy) = match wm.position {
                        WatermarkPosition::TopLeft => (format!("{}", margin), format!("{}", margin)),
                        WatermarkPosition::TopRight => {
                            (format!("W-w-{}", margin), format!("{}", margin))
                        }
                        WatermarkPosition::BottomLeft => {
                            (format!("{}", margin), format!("H-h-{}", margin))
                        }
                        WatermarkPosition::BottomRight => {
                            (format!("W-w-{}", margin), format!("H-h-{}", margin))
                        }
                        WatermarkPosition::Center => ("(W-w)/2".into(), "(H-h)/2".into()),
                    };
                    let scale_frac = (wm.scale_percent.clamp(5.0, 50.0) / 100.0).max(0.05);
                    let opacity = wm.opacity.clamp(0.1, 1.0);
                    stages.push(format!(
                        "[{wm}:v]format=rgba,colorchannelmixer=aa={op:.2}[wma];[wma]{cur}scale2ref=w=iw*{sf:.3}:h=-1[wmr][bref];[bref][wmr]overlay={ox}:{oy}[vwm]",
                        wm = wm_idx,
                        op = opacity,
                        cur = cur,
                        sf = scale_frac,
                        ox = ox,
                        oy = oy,
                    ));
                    cur = "[vwm]".to_string();
                }

                if has_boomerang {
                    stages.push(format!(
                        "{cur}split[bm_f][bm_r];[bm_r]reverse[bm_rv];[bm_f][bm_rv]concat=n=2:v=1:a=0[vbm]"
                    ));
                    cur = "[vbm]".to_string();
                }

                args.extend([
                    "-filter_complex".into(),
                    stages.join(";"),
                    "-map".into(),
                    cur,
                ]);
                if !has_boomerang {
                    if let Some(audio_idx) = replace_audio_idx {
                        args.extend([
                            "-map".into(),
                            format!("{}:a:0", audio_idx),
                            "-shortest".into(),
                        ]);
                    } else if !settings.mute_audio {
                        args.extend(["-map".into(), "0:a?".into()]);
                    }
                }
            } else if !vfilters.is_empty() {
                args.extend(["-vf".into(), vfilters.join(",")]);
            }

            let push_audio = |args: &mut Vec<String>, codec: &str| {
                if (settings.mute_audio && !has_replace_audio)
                    || settings.playback_mode == VideoPlaybackMode::Boomerang
                {
                    args.push("-an".into());
                } else {
                    let mut afilters = Vec::new();
                    if settings.denoise_audio {
                        afilters.push("highpass=f=80,lowpass=f=12000,afftdn=nf=-25".to_string());
                    }
                    if settings.playback_mode == VideoPlaybackMode::Reverse {
                        afilters.push("areverse".to_string());
                    }
                    if (speed - 1.0).abs() > 0.01 {
                        // Chain atempo filters since each atempo accepts 0.5..=2.0
                        let mut rem = speed;
                        while rem > 2.0 {
                            afilters.push("atempo=2.0".to_string());
                            rem /= 2.0;
                        }
                        while rem < 0.5 {
                            afilters.push("atempo=0.5".to_string());
                            rem *= 2.0;
                        }
                        afilters.push(format!("atempo={:.3}", rem));
                    }
                    if settings.normalize_loudness {
                        afilters.push("loudnorm".to_string());
                    }
                    if settings.audio_gain_db.abs() > 0.1 {
                        afilters.push(format!(
                            "volume={:.1}dB",
                            settings.audio_gain_db.clamp(-20.0, 20.0)
                        ));
                    }
                    if !afilters.is_empty() {
                        args.extend(["-af".into(), afilters.join(",")]);
                    }
                    if let Some(sr) = settings.sample_rate_hz {
                        args.extend(["-ar".into(), sr.to_string()]);
                    }
                    if settings.mono {
                        args.extend(["-ac".into(), "1".into()]);
                    }
                    args.extend([
                        "-c:a".into(),
                        codec.into(),
                        "-b:a".into(),
                        audio_br.clone(),
                    ]);
                }
            };

            let can_remux = settings.remux_only
                && vfilters.is_empty()
                && !has_reframe
                && !has_boomerang
                && wm_input_idx.is_none()
                && matches!(
                    video_fmt,
                    VideoFormat::Mp4H264
                        | VideoFormat::Mp4H265
                        | VideoFormat::Mov
                        | VideoFormat::Mkv
                        | VideoFormat::Avi
                );

            if can_remux {
                args.extend(["-c:v".into(), "copy".into()]);
                if settings.mute_audio && !has_replace_audio {
                    args.push("-an".into());
                } else {
                    args.extend(["-c:a".into(), "copy".into()]);
                }
                if matches!(
                    video_fmt,
                    VideoFormat::Mp4H264 | VideoFormat::Mp4H265 | VideoFormat::Mov
                ) {
                    args.extend(["-movflags".into(), "+faststart".into()]);
                }
            } else {
            match video_fmt {
                VideoFormat::Mp4H264 | VideoFormat::Mov | VideoFormat::Mkv => {
                    if let Some(kbps) = target_video_kbps {
                        args.extend([
                            "-c:v".into(),
                            "libx264".into(),
                            "-b:v".into(),
                            format!("{}k", kbps),
                            "-maxrate".into(),
                            format!("{}k", (kbps as f32 * 1.4).round() as u32),
                            "-bufsize".into(),
                            format!("{}k", kbps * 2),
                            "-preset".into(),
                            "medium".into(),
                        ]);
                    } else if use_hw {
                        if let Some(enc) = &hw.h264_hw_encoder {
                            push_hw_video_codec(&mut args, enc, q, crf);
                        } else {
                            args.extend([
                                "-c:v".into(),
                                "libx264".into(),
                                "-crf".into(),
                                crf.to_string(),
                                "-preset".into(),
                                "medium".into(),
                            ]);
                        }
                    } else {
                        args.extend([
                            "-c:v".into(),
                            "libx264".into(),
                            "-crf".into(),
                            crf.to_string(),
                            "-preset".into(),
                            "medium".into(),
                        ]);
                    }
                    args.extend(["-pix_fmt".into(), "yuv420p".into()]);
                    push_audio(&mut args, "aac");
                    if matches!(video_fmt, VideoFormat::Mp4H264 | VideoFormat::Mov) {
                        args.extend(["-movflags".into(), "+faststart".into()]);
                    }
                }
                VideoFormat::Mp4H265 => {
                    if let Some(kbps) = target_video_kbps {
                        args.extend([
                            "-c:v".into(),
                            "libx265".into(),
                            "-b:v".into(),
                            format!("{}k", kbps),
                            "-preset".into(),
                            "medium".into(),
                            "-tag:v".into(),
                            "hvc1".into(),
                        ]);
                    } else if use_hw {
                        if let Some(enc) = &hw.h265_hw_encoder {
                            push_hw_video_codec(&mut args, enc, q, crf);
                            if enc.contains("videotoolbox") {
                                args.extend(["-tag:v".into(), "hvc1".into()]);
                            }
                        } else {
                            args.extend([
                                "-c:v".into(),
                                "libx265".into(),
                                "-crf".into(),
                                (crf + 2).to_string(),
                                "-preset".into(),
                                "medium".into(),
                                "-tag:v".into(),
                                "hvc1".into(),
                            ]);
                        }
                    } else {
                        args.extend([
                            "-c:v".into(),
                            "libx265".into(),
                            "-crf".into(),
                            (crf + 2).to_string(),
                            "-preset".into(),
                            "medium".into(),
                            "-tag:v".into(),
                            "hvc1".into(),
                        ]);
                    }
                    args.extend(["-pix_fmt".into(), "yuv420p".into()]);
                    push_audio(&mut args, "aac");
                    args.extend(["-movflags".into(), "+faststart".into()]);
                }
                VideoFormat::WebmVp9 => {
                    let vp9_crf = (42.0 - (q as f32 / 100.0) * 24.0).round() as u32;
                    args.extend(["-c:v".into(), "libvpx-vp9".into()]);
                    if let Some(kbps) = target_video_kbps {
                        args.extend(["-b:v".into(), format!("{}k", kbps)]);
                    } else {
                        args.extend(["-crf".into(), vp9_crf.to_string(), "-b:v".into(), "0".into()]);
                    }
                    args.extend([
                        "-row-mt".into(),
                        "1".into(),
                        "-deadline".into(),
                        "good".into(),
                        "-cpu-used".into(),
                        "2".into(),
                    ]);
                    push_audio(&mut args, "libopus");
                }
                VideoFormat::WebmAv1 => {
                    let av1_crf = (44.0 - (q as f32 / 100.0) * 24.0).round() as u32;
                    if let Some(kbps) = target_video_kbps {
                        args.extend([
                            "-c:v".into(),
                            "libsvtav1".into(),
                            "-b:v".into(),
                            format!("{}k", kbps),
                            "-preset".into(),
                            "6".into(),
                        ]);
                    } else if use_hw {
                        if let Some(enc) = &hw.av1_hw_encoder {
                            push_hw_video_codec(&mut args, enc, q, av1_crf);
                        } else {
                            args.extend([
                                "-c:v".into(),
                                "libsvtav1".into(),
                                "-crf".into(),
                                av1_crf.to_string(),
                                "-preset".into(),
                                "6".into(),
                            ]);
                        }
                    } else {
                        args.extend([
                            "-c:v".into(),
                            "libsvtav1".into(),
                            "-crf".into(),
                            av1_crf.to_string(),
                            "-preset".into(),
                            "6".into(),
                        ]);
                    }
                    push_audio(&mut args, "libopus");
                }
                VideoFormat::Avi => {
                    let qscale =
                        (12.0 - (q as f32 / 100.0) * 10.0).round().clamp(2.0, 12.0) as u32;
                    args.extend([
                        "-c:v".into(),
                        "mpeg4".into(),
                        "-q:v".into(),
                        qscale.to_string(),
                    ]);
                    push_audio(&mut args, "libmp3lame");
                }
                VideoFormat::Gif | VideoFormat::AnimatedWebp => unreachable!(),
            }
            }
        }
    }

    args.push(output.to_string_lossy().into_owned());
    args
}

fn push_hw_video_codec(args: &mut Vec<String>, enc: &str, quality: u8, crf: u32) {
    args.extend(["-c:v".into(), enc.into()]);
    if enc.contains("videotoolbox") {
        // VideoToolbox uses -q:v 1..100 (higher = better quality)
        let vt_q = quality.clamp(20, 95);
        args.extend(["-q:v".into(), vt_q.to_string()]);
    } else if enc.contains("nvenc") {
        args.extend([
            "-rc".into(),
            "vbr".into(),
            "-cq".into(),
            crf.to_string(),
            "-preset".into(),
            "p4".into(),
        ]);
    } else if enc.contains("qsv") {
        args.extend(["-global_quality".into(), crf.to_string()]);
    } else {
        args.extend(["-qp".into(), crf.to_string()]);
    }
}

fn run_ffmpeg_with_progress<F>(
    ffmpeg_path: &Path,
    args: &[String],
    duration_secs: Option<f64>,
    cancel_flag: &AtomicBool,
    on_progress: &mut F,
) -> Result<()>
where
    F: FnMut(f32),
{
    let mut child = Command::new(ffmpeg_path)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| WandelError::FfmpegFailed(format!("Failed to spawn ffmpeg: {}", e)))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    // Drain stderr on a tiny helper thread so pipe buffers never deadlock
    let stderr_handle = std::thread::spawn(move || {
        let mut buf = String::new();
        if let Some(err_stream) = stderr {
            let mut reader = BufReader::new(err_stream);
            let _ = std::io::Read::read_to_string(&mut reader, &mut buf);
        }
        buf
    });

    if let Some(out_stream) = stdout {
        let reader = BufReader::new(out_stream);
        for line in reader.lines().map_while(std::result::Result::ok) {
            if cancel_flag.load(Ordering::Relaxed) {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stderr_handle.join();
                return Err(WandelError::Cancelled);
            }

            if let Some(us_str) = line.strip_prefix("out_time_us=") {
                if let (Ok(us), Some(dur)) = (us_str.trim().parse::<f64>(), duration_secs) {
                    if dur > 0.0 {
                        let pct = ((us / 1_000_000.0) / dur * 100.0).clamp(0.0, 99.5) as f32;
                        on_progress(pct);
                    }
                }
            } else if line.trim() == "progress=end" {
                on_progress(100.0);
            }
        }
    }

    let status = child.wait()?;
    let stderr_text = stderr_handle.join().unwrap_or_default();

    if cancel_flag.load(Ordering::Relaxed) {
        return Err(WandelError::Cancelled);
    }

    if !status.success() {
        let trimmed = stderr_text.trim();
        if trimmed.contains("does not contain any stream") {
            return Err(WandelError::FfmpegFailed(
                "This file has no audio track to extract.".into(),
            ));
        }
        return Err(WandelError::FfmpegFailed(trimmed.to_string()));
    }

    Ok(())
}

/// Converts a video into an Animated WebP using FFmpeg for frame extraction
/// and statically linked `webp::AnimEncoder` (works even if FFmpeg lacks `libwebp`).
fn convert_video_to_animated_webp<F>(
    input: &Path,
    output: &Path,
    settings: &VideoSettings,
    ffmpeg_path: &Path,
    cancel_flag: &AtomicBool,
    mut on_progress: F,
) -> Result<u64>
where
    F: FnMut(f32),
{
    let fps = settings.max_fps.unwrap_or(settings.gif_fps).clamp(5, 30);
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let frame_dir = std::env::temp_dir().join(format!(
        "wandel_awebp_{}_{}",
        std::process::id(),
        unique
    ));
    std::fs::create_dir_all(&frame_dir)?;

    let cleanup = || {
        let _ = std::fs::remove_dir_all(&frame_dir);
    };

    let mut args: Vec<String> = vec!["-y".into(), "-hide_banner".into(), "-loglevel".into(), "error".into()];
    if let Some(start) = settings.trim_start_secs.filter(|&s| s > 0.0) {
        args.extend(["-ss".into(), format!("{:.3}", start)]);
    }
    args.extend(["-i".into(), input.to_string_lossy().into_owned()]);
    if let Some(end) = settings.trim_end_secs {
        let start = settings.trim_start_secs.unwrap_or(0.0).max(0.0);
        if end > start {
            args.extend(["-t".into(), format!("{:.3}", end - start)]);
        }
    }

    let scale = build_scale_filter(&settings.resize, settings.never_upscale)
        .unwrap_or_else(|| "scale='min(720,iw)':-2:flags=lanczos".into());
    args.extend([
        "-vf".into(),
        format!("fps={},{}", fps, scale),
        frame_dir
            .join("frame_%05d.png")
            .to_string_lossy()
            .into_owned(),
    ]);

    on_progress(15.0);
    let out = Command::new(ffmpeg_path)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()?;

    if cancel_flag.load(Ordering::Relaxed) {
        cleanup();
        return Err(WandelError::Cancelled);
    }

    if !out.status.success() {
        cleanup();
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(WandelError::FfmpegFailed(stderr.trim().to_string()));
    }

    let mut frame_paths: Vec<_> = std::fs::read_dir(&frame_dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("png"))
        .collect();
    frame_paths.sort();

    if frame_paths.is_empty() {
        cleanup();
        return Err(WandelError::FfmpegFailed(
            "No video frames could be extracted for Animated WebP".into(),
        ));
    }

    on_progress(45.0);

    // Decode frames into RGBA buffers
    let mut decoded_frames = Vec::with_capacity(frame_paths.len());
    let mut width = 0u32;
    let mut height = 0u32;

    for (idx, fp) in frame_paths.iter().enumerate() {
        if cancel_flag.load(Ordering::Relaxed) {
            cleanup();
            return Err(WandelError::Cancelled);
        }
        let rgba = image::open(fp)
            .map_err(|e| WandelError::ImageDecode(e.to_string()))?
            .to_rgba8();
        if idx == 0 {
            width = rgba.width();
            height = rgba.height();
        }
        decoded_frames.push(rgba.into_raw());
    }

    on_progress(75.0);

    let mut config = webp::WebPConfig::new()
        .map_err(|_| WandelError::ImageEncode("Failed to initialize WebPConfig".into()))?;
    config.quality = settings.quality.clamp(10, 100) as f32;
    config.lossless = 0;

    let mut encoder = webp::AnimEncoder::new(width, height, &config);
    encoder.set_loop_count(0);

    let frame_step_ms = (1000.0 / f64::from(fps)).round() as i32;
    for (idx, raw) in decoded_frames.iter().enumerate() {
        let ts = (idx as i32) * frame_step_ms;
        encoder.add_frame(webp::AnimFrame::from_rgba(raw, width, height, ts));
    }

    let webp_mem = encoder
        .try_encode()
        .map_err(|e| WandelError::ImageEncode(format!("Animated WebP encode failed: {:?}", e)))?;

    std::fs::write(output, &*webp_mem)?;
    cleanup();
    on_progress(100.0);

    let size = std::fs::metadata(output)?.len();
    Ok(size)
}

/// Concatenates multiple video clips in order into a single video file,
/// automatically normalizing resolution and pixel format across mixed inputs.
pub fn merge_videos(
    inputs: &[PathBuf],
    output: &Path,
    settings: &VideoSettings,
    hw: &HardwareCapabilities,
    ffmpeg_path: &Path,
) -> Result<u64> {
    if inputs.len() < 2 {
        return Err(WandelError::InvalidSettings(
            "At least 2 videos are required to merge".into(),
        ));
    }

    let mut args: Vec<String> = vec![
        "-y".into(),
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
    ];

    for p in inputs {
        args.extend(["-i".into(), p.to_string_lossy().into_owned()]);
    }

    // Normalize all clips to 1280x720 (or fit) with setsar=1 so mixed resolutions concat cleanly
    let mut fc = String::new();
    for idx in 0..inputs.len() {
        fc.push_str(&format!(
            "[{i}:v]scale=1280:720:force_original_aspect_ratio=decrease,pad=1280:720:(ow-iw)/2:(oh-ih)/2,setsar=1,fps=30,format=yuv420p[v{i}];",
            i = idx
        ));
    }
    for idx in 0..inputs.len() {
        fc.push_str(&format!("[v{}]", idx));
    }
    fc.push_str(&format!("concat=n={}:v=1:a=0[vout]", inputs.len()));

    args.extend(["-filter_complex".into(), fc, "-map".into(), "[vout]".into()]);

    let q = settings.quality.clamp(1, 100);
    let crf = (38.0 - (q as f32 / 100.0) * 22.0).round() as u32;
    if settings.use_hardware_accel {
        if let Some(enc) = &hw.h264_hw_encoder {
            push_hw_video_codec(&mut args, enc, q, crf);
        } else {
            args.extend([
                "-c:v".into(),
                "libx264".into(),
                "-crf".into(),
                crf.to_string(),
                "-preset".into(),
                "medium".into(),
            ]);
        }
    } else {
        args.extend([
            "-c:v".into(),
            "libx264".into(),
            "-crf".into(),
            crf.to_string(),
            "-preset".into(),
            "medium".into(),
        ]);
    }
    args.extend([
        "-pix_fmt".into(),
        "yuv420p".into(),
        "-movflags".into(),
        "+faststart".into(),
        output.to_string_lossy().into_owned(),
    ]);

    let out = Command::new(ffmpeg_path)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(WandelError::FfmpegFailed(stderr.trim().to_string()));
    }

    let size = std::fs::metadata(output)?.len();
    Ok(size)
}

/// Concatenates multiple audio files in order into a single output file (MP3/AAC/OGG/WAV/FLAC).
pub fn merge_audio(
    inputs: &[PathBuf],
    output: &Path,
    settings: &VideoSettings,
    ffmpeg_path: &Path,
) -> Result<u64> {
    if inputs.len() < 2 {
        return Err(WandelError::InvalidSettings(
            "At least 2 audio files are required to merge".into(),
        ));
    }

    let mut args: Vec<String> = vec![
        "-y".into(),
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
    ];
    for p in inputs {
        args.extend(["-i".into(), p.to_string_lossy().into_owned()]);
    }

    let mut fc = String::new();
    for idx in 0..inputs.len() {
        fc.push_str(&format!("[{}:a:0]", idx));
    }
    fc.push_str(&format!("concat=n={}:v=0:a=1[aout]", inputs.len()));
    args.extend(["-filter_complex".into(), fc, "-map".into(), "[aout]".into()]);

    let audio_br = format!("{}k", settings.audio_bitrate_kbps.clamp(64, 320));
    let ext = output
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mp3")
        .to_ascii_lowercase();
    match ext.as_str() {
        "wav" => args.extend(["-c:a".into(), "pcm_s16le".into()]),
        "flac" => args.extend(["-c:a".into(), "flac".into()]),
        "m4a" | "aac" => args.extend(["-c:a".into(), "aac".into(), "-b:a".into(), audio_br]),
        "ogg" | "opus" => args.extend(["-c:a".into(), "libopus".into(), "-b:a".into(), audio_br]),
        _ => args.extend(["-c:a".into(), "libmp3lame".into(), "-b:a".into(), audio_br]),
    }

    args.push(output.to_string_lossy().into_owned());

    let out = Command::new(ffmpeg_path)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(WandelError::FfmpegFailed(stderr.trim().to_string()));
    }

    let size = std::fs::metadata(output)?.len();
    Ok(size)
}

/// Extracts `frame_count` evenly spaced frames across a video and stitches them into a
/// crisp Contact Sheet / Storyboard image (default 3x3 = 9 frames).
pub fn create_video_storyboard(
    input: &Path,
    output: &Path,
    frame_count: u32,
    columns: u32,
    ffmpeg_path: &Path,
    ffprobe_path: Option<&Path>,
) -> Result<u64> {
    let count = frame_count.clamp(4, 25);
    let cols = if columns > 0 {
        columns
    } else {
        (count as f64).sqrt().ceil() as u32
    };

    let (_, _, duration_opt, _) = inspect_video(input, ffprobe_path, Some(ffmpeg_path), false);
    let duration = duration_opt.unwrap_or(5.0).max(0.5);

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let frame_dir = std::env::temp_dir().join(format!(
        "wandel_storyboard_{}_{}",
        std::process::id(),
        unique
    ));
    std::fs::create_dir_all(&frame_dir)?;

    let mut frame_paths = Vec::with_capacity(count as usize);
    for idx in 0..count {
        let ts = if count > 1 {
            (duration * (idx as f64 + 0.5)) / f64::from(count)
        } else {
            duration * 0.5
        };
        let fp = frame_dir.join(format!("sb_{:02}.png", idx));
        let st = Command::new(ffmpeg_path)
            .args([
                "-y",
                "-hide_banner",
                "-loglevel",
                "error",
                "-ss",
                &format!("{:.3}", ts),
                "-i",
            ])
            .arg(input)
            .args(["-frames:v", "1", "-vf", "scale=480:-2"])
            .arg(&fp)
            .status()?;
        if st.success() && fp.is_file() {
            frame_paths.push(fp);
        }
    }

    if frame_paths.is_empty() {
        let _ = std::fs::remove_dir_all(&frame_dir);
        return Err(WandelError::FfmpegFailed(
            "Could not extract any frames for storyboard".into(),
        ));
    }

    let ext = output
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");
    let fmt = crate::formats::ImageFormat::from_extension(ext)
        .unwrap_or(crate::formats::ImageFormat::Png);

    let opts = crate::settings::ContactSheetOptions {
        columns: cols,
        cell_size: 400,
        gap_px: 12,
        background_hex: "#18181b".into(),
    };
    let img_settings = crate::settings::ImageSettings {
        format: fmt,
        quality: 85,
        ..Default::default()
    };

    let res = crate::image_engine::create_contact_sheet(
        &frame_paths,
        output,
        &opts,
        &img_settings,
        Some(ffmpeg_path),
    );
    let _ = std::fs::remove_dir_all(&frame_dir);
    res
}

/// Extracts a frame sequence from a video at `fps` frames per second into `output_dir`.
/// Supported extensions: `png`, `jpg`, `webp`.
pub fn extract_video_frames(
    input: &Path,
    output_dir: &Path,
    fps: f32,
    format_ext: &str,
    ffmpeg_path: &Path,
) -> Result<Vec<String>> {
    use rayon::prelude::*;

    let clean_fps = fps.clamp(0.1, 60.0);
    let ext = match format_ext.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" => "jpg",
        "webp" => "webp",
        _ => "png",
    };

    std::fs::create_dir_all(output_dir)?;
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("video");

    // Extract as PNG first if target is WebP (avoids FFmpeg builds without libwebp)
    let extract_ext = if ext == "webp" { "png" } else { ext };
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let stage_dir = if ext == "webp" {
        let d = std::env::temp_dir().join(format!("wandel_frames_{}_{}", std::process::id(), unique));
        std::fs::create_dir_all(&d)?;
        d
    } else {
        output_dir.to_path_buf()
    };

    let pattern = stage_dir.join(format!("{}_frame_%04d.{}", stem, extract_ext));
    let out = Command::new(ffmpeg_path)
        .args(["-y", "-hide_banner", "-loglevel", "error", "-i"])
        .arg(input)
        .args(["-vf", &format!("fps={:.3}", clean_fps)])
        .arg(&pattern)
        .output()?;

    if !out.status.success() {
        if ext == "webp" {
            let _ = std::fs::remove_dir_all(&stage_dir);
        }
        return Err(WandelError::FfmpegFailed(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }

    let prefix = format!("{}_frame_", stem);
    let mut extracted: Vec<PathBuf> = std::fs::read_dir(&stage_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(&prefix) && n.ends_with(extract_ext))
        })
        .collect();
    extracted.sort();

    if extracted.is_empty() {
        if ext == "webp" {
            let _ = std::fs::remove_dir_all(&stage_dir);
        }
        return Err(WandelError::FfmpegFailed(
            "No frames were extracted from this video".into(),
        ));
    }

    if ext == "webp" {
        let img_settings = crate::settings::ImageSettings {
            format: crate::formats::ImageFormat::Webp,
            quality: 85,
            ..Default::default()
        };
        let converted: Result<Vec<String>> = extracted
            .par_iter()
            .map(|src_png| {
                let fname = src_png
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("frame");
                let dest = output_dir.join(format!("{}.webp", fname));
                crate::image_engine::convert_image(src_png, &dest, &img_settings, None)?;
                Ok(dest.to_string_lossy().into_owned())
            })
            .collect();
        let _ = std::fs::remove_dir_all(&stage_dir);
        let mut files = converted?;
        files.sort();
        Ok(files)
    } else {
        Ok(extracted
            .into_iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect())
    }
}

/// Renders an audio file's waveform as a high-resolution PNG graphic using FFmpeg `showwavespic`.
pub fn create_audio_waveform(
    input: &Path,
    output: &Path,
    width: u32,
    height: u32,
    color_hex: &str,
    bg_hex: &str,
    ffmpeg_path: &Path,
) -> Result<u64> {
    let w = width.clamp(320, 4096);
    let h = height.clamp(80, 2048);
    let clean_hex = |s: &str, fallback: &str| -> String {
        let trimmed = s.trim().trim_start_matches('#');
        if trimmed.len() == 6 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            format!("0x{}", trimmed)
        } else {
            fallback.to_string()
        }
    };
    let fg = clean_hex(color_hex, "0x6366f1");
    let bg = clean_hex(bg_hex, "0x18181b");

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let filter = format!(
        "color=c={bg}:s={w}x{h}:d=1[bg];[0:a]showwavespic=s={w}x{h}:colors={fg}[fg];[bg][fg]overlay=format=auto"
    );

    let out = Command::new(ffmpeg_path)
        .args(["-y", "-hide_banner", "-loglevel", "error", "-i"])
        .arg(input)
        .args(["-filter_complex", &filter, "-frames:v", "1"])
        .arg(output)
        .output()?;

    if !out.status.success() {
        return Err(WandelError::FfmpegFailed(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }

    let size = std::fs::metadata(output)?.len();
    Ok(size)
}

/// Turns an ordered list of images into an MP4 slideshow video with uniform canvas scaling
/// and optional background soundtrack.
#[allow(clippy::too_many_arguments)]
pub fn create_slideshow_video(
    inputs: &[PathBuf],
    output: &Path,
    secs_per_slide: f32,
    width: u32,
    height: u32,
    audio_path: Option<&Path>,
    ffmpeg_path: &Path,
) -> Result<u64> {
    if inputs.len() < 2 {
        return Err(WandelError::InvalidSettings(
            "At least 2 images are required to create a slideshow".into(),
        ));
    }

    let dur = secs_per_slide.clamp(0.5, 30.0);
    let w = (width / 2 * 2).clamp(160, 3840);
    let h = (height / 2 * 2).clamp(160, 2160);

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut args: Vec<String> = vec![
        "-y".into(),
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
    ];

    for path in inputs {
        args.extend([
            "-loop".into(),
            "1".into(),
            "-t".into(),
            format!("{:.2}", dur),
            "-i".into(),
            path.to_string_lossy().into_owned(),
        ]);
    }

    let audio_idx = audio_path.filter(|p| p.is_file()).map(|p| {
        args.extend(["-i".into(), p.to_string_lossy().into_owned()]);
        inputs.len()
    });

    let mut filter_parts = Vec::with_capacity(inputs.len() + 1);
    let mut concat_inputs = String::new();
    for i in 0..inputs.len() {
        filter_parts.push(format!(
            "[{i}:v]scale={w}:{h}:force_original_aspect_ratio=decrease,pad={w}:{h}:(ow-iw)/2:(oh-ih)/2:color=#18181b,setsar=1,fps=30,format=yuv420p[v{i}]"
        ));
        concat_inputs.push_str(&format!("[v{i}]"));
    }
    filter_parts.push(format!(
        "{}concat=n={}:v=1:a=0[vout]",
        concat_inputs,
        inputs.len()
    ));

    args.extend([
        "-filter_complex".into(),
        filter_parts.join(";"),
        "-map".into(),
        "[vout]".into(),
    ]);

    if let Some(aidx) = audio_idx {
        args.extend([
            "-map".into(),
            format!("{}:a:0", aidx),
            "-c:a".into(),
            "aac".into(),
            "-b:a".into(),
            "192k".into(),
            "-shortest".into(),
        ]);
    } else {
        args.push("-an".into());
    }

    args.extend([
        "-c:v".into(),
        "libx264".into(),
        "-crf".into(),
        "22".into(),
        "-preset".into(),
        "medium".into(),
        "-pix_fmt".into(),
        "yuv420p".into(),
        "-movflags".into(),
        "+faststart".into(),
        output.to_string_lossy().into_owned(),
    ]);

    let out = Command::new(ffmpeg_path).args(&args).output()?;
    if !out.status.success() {
        return Err(WandelError::FfmpegFailed(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }

    let size = std::fs::metadata(output)?.len();
    Ok(size)
}

/// Combines a static cover image and an audio track into a shareable MP4 video (audiogram/podcast video).
pub fn create_podcast_video(
    audio_path: &Path,
    cover_image: &Path,
    output: &Path,
    width: u32,
    height: u32,
    ffmpeg_path: &Path,
) -> Result<u64> {
    let w = (width / 2 * 2).clamp(320, 3840);
    let h = (height / 2 * 2).clamp(240, 2160);

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let vf = format!(
        "scale={w}:{h}:force_original_aspect_ratio=decrease,pad={w}:{h}:(ow-iw)/2:(oh-ih)/2:color=#18181b,setsar=1,format=yuv420p"
    );

    let out = Command::new(ffmpeg_path)
        .args(["-y", "-hide_banner", "-loglevel", "error", "-loop", "1", "-i"])
        .arg(cover_image)
        .arg("-i")
        .arg(audio_path)
        .args([
            "-vf",
            &vf,
            "-c:v",
            "libx264",
            "-tune",
            "stillimage",
            "-crf",
            "22",
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-pix_fmt",
            "yuv420p",
            "-shortest",
            "-movflags",
            "+faststart",
        ])
        .arg(output)
        .output()?;

    if !out.status.success() {
        return Err(WandelError::FfmpegFailed(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }

    let size = std::fs::metadata(output)?.len();
    Ok(size)
}

/// Losslessly splits a video or audio file into equal-duration chunks using FFmpeg's segment muxer.
pub fn split_media_segments(
    input: &Path,
    output_dir: &Path,
    segment_secs: f32,
    ffmpeg_path: &Path,
) -> Result<Vec<String>> {
    let secs = segment_secs.clamp(0.5, 36_000.0);
    std::fs::create_dir_all(output_dir)?;

    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("media");
    let ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mp4");

    let pattern = output_dir.join(format!("{}_part_%03d.{}", stem, ext));
    let out = Command::new(ffmpeg_path)
        .args(["-y", "-hide_banner", "-loglevel", "error", "-i"])
        .arg(input)
        .args([
            "-c",
            "copy",
            "-map",
            "0",
            "-f",
            "segment",
            "-segment_time",
            &format!("{:.2}", secs),
            "-reset_timestamps",
            "1",
        ])
        .arg(&pattern)
        .output()?;

    if !out.status.success() {
        return Err(WandelError::FfmpegFailed(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }

    let prefix = format!("{}_part_", stem);
    let mut parts: Vec<String> = std::fs::read_dir(output_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(&prefix) && n.ends_with(ext))
        })
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    parts.sort();

    if parts.is_empty() {
        return Err(WandelError::FfmpegFailed(
            "No segments were produced from this file".into(),
        ));
    }

    Ok(parts)
}

/// Extracts a single crisp poster/cover frame from a video at `timestamp_secs` (fast `-ss` seek).
pub fn extract_video_poster(
    input: &Path,
    output: &Path,
    timestamp_secs: f32,
    ffmpeg_path: &Path,
) -> Result<u64> {
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let ts = timestamp_secs.max(0.0);
    let out = Command::new(ffmpeg_path)
        .args([
            "-y",
            "-hide_banner",
            "-loglevel",
            "error",
            "-ss",
            &format!("{:.3}", ts),
            "-i",
        ])
        .arg(input)
        .args(["-frames:v", "1", "-q:v", "2"])
        .arg(output)
        .output()?;

    if !out.status.success() {
        return Err(WandelError::FfmpegFailed(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }
    let size = std::fs::metadata(output)?.len();
    if size == 0 {
        return Err(WandelError::FfmpegFailed(
            "Poster frame extraction produced an empty file".into(),
        ));
    }
    Ok(size)
}
