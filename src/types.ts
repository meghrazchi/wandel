export type MediaKind = "image" | "video" | "audio" | "pdf";

export type ImageFormat =
  | "png"
  | "jpg"
  | "webp"
  | "avif"
  | "gif"
  | "bmp"
  | "tiff"
  | "ico"
  | "heic"
  | "jxl"
  | "svg";

export type VideoFormat =
  | "mp4_h264"
  | "mp4_h265"
  | "webm_vp9"
  | "webm_av1"
  | "mov"
  | "mkv"
  | "avi"
  | "gif"
  | "animated_webp";

export type AspectCrop =
  | "square1x1"
  | "landscape16x9"
  | "portrait9x16"
  | "standard4x3"
  | "photo3x2";

export type AudioFormat = "mp3" | "wav" | "aac" | "flac" | "opus";

export type VideoTarget =
  | { kind: "video"; format: VideoFormat }
  | { kind: "audio"; format: AudioFormat };

export type ResizeMode =
  | { mode: "original" }
  | { mode: "max_dimensions"; width: number | null; height: number | null }
  | { mode: "exact"; width: number; height: number; keep_aspect_ratio: boolean }
  | { mode: "percentage"; percent: number };

export type FrameGradient =
  | "none"
  | "indigo"
  | "sunset"
  | "ocean"
  | "emerald"
  | "midnight";

export type ImageFilterPreset =
  | "none"
  | "sepia"
  | "warm"
  | "cool"
  | "vintage"
  | "high_contrast_bw"
  | "invert";

export type VideoPlaybackMode = "normal" | "reverse" | "boomerang";

export type VideoAspectReframe =
  | "none"
  | "portrait9x16"
  | "portrait4x5"
  | "square1x1"
  | "landscape16x9";

export interface ResponsiveSetResult {
  files: string[];
  htmlSnippet: string;
}

export interface SpriteSheetResult {
  imagePath: string;
  cssPath: string;
  cssSnippet: string;
  width: number;
  height: number;
  count: number;
}

export type WatermarkPosition =
  | "bottom_right"
  | "bottom_left"
  | "top_right"
  | "top_left"
  | "center";

export interface WatermarkSettings {
  imagePath: string;
  opacity: number;
  scalePercent: number;
  marginPx: number;
  position: WatermarkPosition;
}

export type PdfPageFit = "fit_to_image" | "a4" | "letter";

export interface TextWatermarkSettings {
  text: string;
  fontSizePercent: number;
  color: string;
  opacity: number;
  marginPx: number;
  position: WatermarkPosition;
  tiled?: boolean;
}

export interface PdfSettings {
  dpi: number;
  pageRange: string;
}

export interface NormalizedCrop {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface DuplicateGroup {
  keeper: string;
  duplicates: string[];
}

export interface ImageSettings {
  format: ImageFormat;
  quality: number;
  lossless: boolean;
  resize: ResizeMode;
  neverUpscale: boolean;
  stripMetadata: boolean;
  targetSizeKb: number | null;
  rotate: number;
  flipHorizontal: boolean;
  flipVertical: boolean;
  cropAspect: AspectCrop | null;
  customCrop: NormalizedCrop | null;
  padAspect: AspectCrop | null;
  autoTrimBorder: boolean;
  autoTrimTolerance: number;
  cornerRadiusPercent: number;
  framePaddingPx: number;
  frameGradient: FrameGradient;
  dropShadow: boolean;
  grayscale: boolean;
  sharpen: boolean;
  autoLevels: boolean;
  filterPreset: ImageFilterPreset;
  blurSigma: number;
  pixelateBlock: number;
  vignetteStrength: number;
  brightness: number;
  contrast: number;
  saturation: number;
  backgroundHex: string | null;
  watermark: WatermarkSettings | null;
  textWatermark: TextWatermarkSettings | null;
  blurRegions?: NormalizedCrop[];
  removeBgColor?: string | null;
  removeBgTolerance?: number;
  extraFormats?: ImageFormat[];
  captionText?: string | null;
}

export interface VideoSettings {
  target: VideoTarget;
  quality: number;
  useHardwareAccel: boolean;
  resize: ResizeMode;
  neverUpscale: boolean;
  gifFps: number;
  audioBitrateKbps: number;
  stripMetadata: boolean;
  targetSizeMb: number | null;
  trimStartSecs: number | null;
  trimEndSecs: number | null;
  muteAudio: boolean;
  maxFps: number | null;
  speed: number;
  rotate: number;
  flipHorizontal: boolean;
  normalizeLoudness: boolean;
  sampleRateHz: number | null;
  mono: boolean;
  subtitlePath: string | null;
  replaceAudioPath: string | null;
  watermark: WatermarkSettings | null;
  fadeInSecs: number | null;
  fadeOutSecs: number | null;
  trimSilence: boolean;
  playbackMode: VideoPlaybackMode;
  aspectReframe: VideoAspectReframe;
  remuxOnly: boolean;
  denoiseAudio: boolean;
  deinterlace: boolean;
  videoSharpen: boolean;
  audioGainDb: number;
}

export interface DataUrlExport {
  dataUrl: string;
  mime: string;
  bytes: number;
  width: number;
  height: number;
}

export interface ColorSwatch {
  hex: string;
  r: number;
  g: number;
  b: number;
  percent: number;
}

export interface ImagePreview {
  originalDataUrl: string;
  convertedDataUrl: string;
  originalBytes: number;
  estimatedBytes: number;
  originalWidth: number;
  originalHeight: number;
  outputWidth: number;
  outputHeight: number;
}

export interface CustomPreset {
  id: string;
  name: string;
  settings: ConversionSettings;
}

export interface OutputSettings {
  outputDir: string | null;
  suffix: string;
  renameTemplate: string;
  preserveFolderStructure: boolean;
  overwrite: boolean;
  trashOriginals: boolean;
  bundleZip: boolean;
}

export interface ConversionSettings {
  image: ImageSettings;
  video: VideoSettings;
  pdf: PdfSettings;
  output: OutputSettings;
  maxCpuThreads: number | null;
  maxConcurrentVideos: number | null;
}

export type Preset = "web_optimized" | "high_quality" | "small_size";

export interface FileMetadata {
  id: string;
  path: string;
  fileName: string;
  sizeBytes: number;
  mediaKind: MediaKind;
  sourceFormat: string;
  width: number | null;
  height: number | null;
  durationSecs: number | null;
  thumbnailDataUrl: string | null;
  relativeDir: string | null;
  pageCount: number | null;
}

export interface HardwareCapabilities {
  ffmpegAvailable: boolean;
  ffprobeAvailable: boolean;
  h264HwEncoder: string | null;
  h265HwEncoder: string | null;
  av1HwEncoder: string | null;
  gpuBackendName: string | null;
  cpuCores: number;
}

export type ItemStatus =
  | "pending"
  | "converting"
  | "completed"
  | "failed"
  | "cancelled";

export interface QueueItem extends FileMetadata {
  status: ItemStatus;
  progress: number;
  outputPath?: string;
  outputBytes?: number;
  durationMs?: number;
  error?: string;
  overrideImageFormat?: ImageFormat;
  overrideVideoTarget?: VideoTarget;
}

export interface WatchFolderInfo {
  folderPath: string;
  convertedCount: number;
}

export interface WatchConvertedPayload {
  folderPath: string;
  fileName: string;
  outputPath: string;
  inputBytes: number;
  outputBytes: number;
}

export interface HistoryEntry {
  id: string;
  fileName: string;
  outputPath: string;
  inputBytes: number;
  outputBytes: number;
  timestamp: number;
  source: "manual" | "watch";
}

export interface LifetimeStats {
  totalFiles: number;
  totalInputBytes: number;
  totalOutputBytes: number;
}

export interface MetadataEntry {
  label: string;
  value: string;
}

export interface DetailedMetadata {
  fileName: string;
  path: string;
  mediaKind: MediaKind;
  sizeBytes: number;
  hasExif: boolean;
  hasGpsWarning: boolean;
  entries: MetadataEntry[];
}

export type Locale = "en" | "de";

export type ProgressEvent =
  | { event: "batchStarted"; data: { total: number } }
  | { event: "fileStarted"; data: { id: string; outputPath: string } }
  | { event: "fileProgress"; data: { id: string; percent: number } }
  | {
      event: "fileCompleted";
      data: {
        id: string;
        outputPath: string;
        outputBytes: number;
        durationMs: number;
      };
    }
  | { event: "fileFailed"; data: { id: string; error: string } }
  | { event: "fileCancelled"; data: { id: string } }
  | {
      event: "batchFinished";
      data: {
        completed: number;
        failed: number;
        cancelled: number;
        totalDurationMs: number;
      };
    };

export const IMAGE_OUTPUT_FORMATS: {
  value: ImageFormat;
  label: string;
  supportsQuality: boolean;
  supportsLossless: boolean;
}[] = [
  { value: "webp", label: "WebP", supportsQuality: true, supportsLossless: true },
  { value: "jpg", label: "JPG", supportsQuality: true, supportsLossless: false },
  { value: "png", label: "PNG", supportsQuality: false, supportsLossless: true },
  { value: "avif", label: "AVIF", supportsQuality: true, supportsLossless: false },
  { value: "jxl", label: "JPEG XL", supportsQuality: true, supportsLossless: true },
  { value: "heic", label: "HEIC", supportsQuality: true, supportsLossless: false },
  { value: "gif", label: "GIF", supportsQuality: false, supportsLossless: false },
  { value: "tiff", label: "TIFF", supportsQuality: false, supportsLossless: true },
  { value: "bmp", label: "BMP", supportsQuality: false, supportsLossless: true },
  { value: "ico", label: "ICO", supportsQuality: false, supportsLossless: false },
];

export const VIDEO_OUTPUT_FORMATS: { value: VideoFormat; label: string }[] = [
  { value: "mp4_h264", label: "MP4 (H.264)" },
  { value: "mp4_h265", label: "MP4 (H.265 / HEVC)" },
  { value: "webm_vp9", label: "WebM (VP9)" },
  { value: "webm_av1", label: "WebM (AV1)" },
  { value: "mov", label: "MOV" },
  { value: "mkv", label: "MKV" },
  { value: "avi", label: "AVI" },
  { value: "gif", label: "Animated GIF" },
  { value: "animated_webp", label: "Animated WebP" },
];

export const AUDIO_OUTPUT_FORMATS: { value: AudioFormat; label: string }[] = [
  { value: "mp3", label: "MP3 Audio" },
  { value: "aac", label: "AAC (M4A)" },
  { value: "opus", label: "OGG (Opus)" },
  { value: "wav", label: "WAV (Lossless)" },
  { value: "flac", label: "FLAC (Lossless)" },
];

export const DEFAULT_SETTINGS: ConversionSettings = {
  image: {
    format: "webp",
    quality: 82,
    lossless: false,
    resize: { mode: "original" },
    neverUpscale: true,
    stripMetadata: true,
    targetSizeKb: null,
    rotate: 0,
    flipHorizontal: false,
    flipVertical: false,
    cropAspect: null,
    customCrop: null,
    padAspect: null,
    autoTrimBorder: false,
    autoTrimTolerance: 12,
    cornerRadiusPercent: 0,
    framePaddingPx: 0,
    frameGradient: "none",
    dropShadow: false,
    grayscale: false,
    sharpen: false,
    autoLevels: false,
    filterPreset: "none",
    blurSigma: 0,
    pixelateBlock: 0,
    vignetteStrength: 0,
    brightness: 0,
    contrast: 0,
    saturation: 0,
    backgroundHex: "#ffffff",
    watermark: null,
    textWatermark: null,
    blurRegions: [],
    removeBgColor: null,
    removeBgTolerance: 24,
    extraFormats: [],
    captionText: null,
  },
  video: {
    target: { kind: "video", format: "mp4_h264" },
    quality: 75,
    useHardwareAccel: true,
    resize: { mode: "original" },
    neverUpscale: true,
    gifFps: 15,
    audioBitrateKbps: 192,
    stripMetadata: true,
    targetSizeMb: null,
    trimStartSecs: null,
    trimEndSecs: null,
    muteAudio: false,
    maxFps: null,
    speed: 1.0,
    rotate: 0,
    flipHorizontal: false,
    normalizeLoudness: false,
    sampleRateHz: null,
    mono: false,
    subtitlePath: null,
    replaceAudioPath: null,
    watermark: null,
    fadeInSecs: null,
    fadeOutSecs: null,
    trimSilence: false,
    playbackMode: "normal",
    aspectReframe: "none",
    remuxOnly: false,
    denoiseAudio: false,
    deinterlace: false,
    videoSharpen: false,
    audioGainDb: 0,
  },
  pdf: {
    dpi: 150,
    pageRange: "",
  },
  output: {
    outputDir: null,
    suffix: "",
    renameTemplate: "",
    preserveFolderStructure: false,
    overwrite: false,
    trashOriginals: false,
    bundleZip: false,
  },
  maxCpuThreads: null,
  maxConcurrentVideos: null,
};

export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const val = bytes / Math.pow(1024, i);
  return `${val >= 100 || i === 0 ? val.toFixed(0) : val.toFixed(1)} ${units[i]}`;
}

export function formatDuration(secs: number | null): string {
  if (secs == null || !Number.isFinite(secs)) return "";
  const total = Math.round(secs);
  const mins = Math.floor(total / 60);
  const rem = total % 60;
  return `${mins}:${rem.toString().padStart(2, "0")}`;
}

export function targetLabelForItem(
  item: QueueItem,
  settings: ConversionSettings
): string {
  if (item.mediaKind === "image" || item.mediaKind === "pdf") {
    const fmt = item.overrideImageFormat ?? settings.image.format;
    return (
      IMAGE_OUTPUT_FORMATS.find((f) => f.value === fmt)?.label ??
      fmt.toUpperCase()
    );
  }
  const target = item.overrideVideoTarget ?? settings.video.target;
  if (target.kind === "video") {
    return (
      VIDEO_OUTPUT_FORMATS.find((f) => f.value === target.format)?.label ??
      "VIDEO"
    );
  }
  return (
    AUDIO_OUTPUT_FORMATS.find((f) => f.value === target.format)?.label ??
    "AUDIO"
  );
}
