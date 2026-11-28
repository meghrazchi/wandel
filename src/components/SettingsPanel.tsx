import { useState } from "react";
import {
  Crop,
  Download,
  FileText,
  Film,
  FlipHorizontal2,
  FlipVertical2,
  Folder,
  Frame,
  Image as ImageIcon,
  Maximize2,
  Plus,
  RotateCcw,
  RotateCw,
  Scissors,
  ShieldCheck,
  Sliders,
  Sparkles,
  Stamp,
  Trash2,
  Upload,
  Volume2,
  VolumeX,
  Wand2,
  X,
  Zap,
} from "lucide-react";
import {
  AUDIO_OUTPUT_FORMATS,
  AspectCrop,
  AudioFormat,
  ConversionSettings,
  CustomPreset,
  FrameGradient,
  HardwareCapabilities,
  IMAGE_OUTPUT_FORMATS,
  ImageFilterPreset,
  ImageFormat,
  Preset,
  ResizeMode,
  VIDEO_OUTPUT_FORMATS,
  VideoAspectReframe,
  VideoFormat,
  VideoPlaybackMode,
  WatermarkPosition,
} from "../types";
import { SectionAccordion } from "./SectionAccordion";

interface SettingsPanelProps {
  settings: ConversionSettings;
  activePreset: string | null;
  customPresets: CustomPreset[];
  hardware: HardwareCapabilities | null;
  imageCount: number;
  pdfCount: number;
  videoCount: number;
  disabled: boolean;
  minimizeToTray: boolean;
  onToggleMinimizeToTray: (enabled: boolean) => void;
  onChange: (next: ConversionSettings) => void;
  onApplyPreset: (preset: Preset) => void;
  onApplyCustomPreset: (preset: CustomPreset) => void;
  onSaveCustomPreset: (name: string) => void;
  onDeleteCustomPreset: (id: string) => void;
  onExportPresets: () => void;
  onImportPresets: () => void;
  onResetSettings: () => void;
  onPickOutputDir: () => void;
  onPickWatermark: () => void;
  onPickVideoWatermark: () => void;
  onPickSubtitle: () => void;
  onPickReplaceAudio: () => void;
}

type SectionId =
  | "img_transform"
  | "img_color"
  | "img_frame"
  | "img_watermark"
  | "vid_trim"
  | "vid_reframe"
  | "vid_audio"
  | "resize"
  | "output";

export function SettingsPanel({
  settings,
  activePreset,
  customPresets,
  hardware,
  imageCount,
  pdfCount,
  videoCount,
  disabled,
  minimizeToTray,
  onToggleMinimizeToTray,
  onChange,
  onApplyPreset,
  onApplyCustomPreset,
  onSaveCustomPreset,
  onDeleteCustomPreset,
  onExportPresets,
  onImportPresets,
  onResetSettings,
  onPickOutputDir,
  onPickWatermark,
  onPickVideoWatermark,
  onPickSubtitle,
  onPickReplaceAudio,
}: SettingsPanelProps) {
  const [manualTab, setManualTab] = useState<"image" | "video">("image");
  const [openSections, setOpenSections] = useState<Record<SectionId, boolean>>({
    img_transform: false,
    img_color: false,
    img_frame: false,
    img_watermark: false,
    vid_trim: false,
    vid_reframe: false,
    vid_audio: false,
    resize: false,
    output: false,
  });
  const [savingPreset, setSavingPreset] = useState(false);
  const [newPresetName, setNewPresetName] = useState("");

  const toggleSection = (id: SectionId) =>
    setOpenSections((prev) => ({ ...prev, [id]: !prev[id] }));

  const activeTab: "image" | "video" =
    imageCount > 0 && videoCount === 0
      ? "image"
      : videoCount > 0 && imageCount === 0
      ? "video"
      : manualTab;

  const imgFormatMeta = IMAGE_OUTPUT_FORMATS.find(
    (f) => f.value === settings.image.format
  );

  const updateImage = (patch: Partial<ConversionSettings["image"]>) =>
    onChange({ ...settings, image: { ...settings.image, ...patch } });

  const updateVideo = (patch: Partial<ConversionSettings["video"]>) =>
    onChange({ ...settings, video: { ...settings.video, ...patch } });

  const updateOutput = (patch: Partial<ConversionSettings["output"]>) =>
    onChange({ ...settings, output: { ...settings.output, ...patch } });

  const currentResize =
    activeTab === "image" ? settings.image.resize : settings.video.resize;
  const currentNeverUpscale =
    activeTab === "image"
      ? settings.image.neverUpscale
      : settings.video.neverUpscale;

  const setResize = (resize: ResizeMode) => {
    if (activeTab === "image") updateImage({ resize });
    else updateVideo({ resize });
  };

  const setNeverUpscale = (neverUpscale: boolean) => {
    if (activeTab === "image") updateImage({ neverUpscale });
    else updateVideo({ neverUpscale });
  };

  // Compute active modifier chips & per-section counts so users always see what's enabled
  const img = settings.image;
  const vid = settings.video;

  const imgTransformCount =
    (img.rotate > 0 ? 1 : 0) +
    (img.flipHorizontal ? 1 : 0) +
    (img.flipVertical ? 1 : 0) +
    (img.cropAspect ? 1 : 0) +
    (img.autoTrimBorder ? 1 : 0) +
    (img.padAspect ? 1 : 0);

  const imgColorCount =
    (img.filterPreset !== "none" ? 1 : 0) +
    (img.grayscale ? 1 : 0) +
    (img.sharpen ? 1 : 0) +
    (img.autoLevels ? 1 : 0) +
    (img.brightness !== 0 ? 1 : 0) +
    (img.contrast !== 0 ? 1 : 0) +
    (img.saturation !== 0 ? 1 : 0) +
    (img.blurSigma > 0 ? 1 : 0) +
    (img.pixelateBlock > 0 ? 1 : 0) +
    (img.vignetteStrength > 0 ? 1 : 0) +
    (img.removeBgColor ? 1 : 0) +
    (img.duotone ? 1 : 0);

  const imgFrameCount =
    (img.cornerRadiusPercent > 0 ? 1 : 0) +
    (img.framePaddingPx > 0 ? 1 : 0);

  const imgWatermarkCount =
    (img.watermark ? 1 : 0) + (img.textWatermark ? 1 : 0);

  const vidTrimCount =
    (vid.trimStartSecs != null ? 1 : 0) +
    (vid.trimEndSecs != null ? 1 : 0) +
    (vid.targetSizeMb != null ? 1 : 0) +
    (vid.maxFps != null ? 1 : 0) +
    (vid.playbackMode !== "normal" ? 1 : 0) +
    (vid.speed !== 1 ? 1 : 0);

  const vidReframeCount =
    (vid.aspectReframe !== "none" ? 1 : 0) +
    (vid.rotate > 0 ? 1 : 0) +
    (vid.flipHorizontal ? 1 : 0) +
    (vid.deinterlace ? 1 : 0) +
    (vid.videoSharpen ? 1 : 0);

  const vidAudioCount =
    (vid.muteAudio ? 1 : 0) +
    (vid.denoiseAudio ? 1 : 0) +
    (vid.normalizeLoudness ? 1 : 0) +
    (vid.trimSilence ? 1 : 0) +
    (vid.audioGainDb !== 0 ? 1 : 0) +
    (vid.fadeInSecs != null ? 1 : 0) +
    (vid.fadeOutSecs != null ? 1 : 0) +
    (vid.subtitlePath ? 1 : 0) +
    (vid.replaceAudioPath ? 1 : 0) +
    (vid.watermark ? 1 : 0);

  const outputCount =
    (settings.output.outputDir ? 1 : 0) +
    (settings.output.trashOriginals ? 1 : 0) +
    (settings.output.bundleZip ? 1 : 0) +
    (settings.output.renameTemplate ? 1 : 0) +
    (settings.output.suffix ? 1 : 0);

  const activeChips: { label: string; clear: () => void }[] =
    activeTab === "image"
      ? [
          ...(img.rotate > 0
            ? [{ label: `Rotate ${img.rotate}°`, clear: () => updateImage({ rotate: 0 }) }]
            : []),
          ...(img.cropAspect
            ? [{ label: `Crop ${img.cropAspect}`, clear: () => updateImage({ cropAspect: null }) }]
            : []),
          ...(img.autoTrimBorder
            ? [{ label: "Auto-Trim", clear: () => updateImage({ autoTrimBorder: false }) }]
            : []),
          ...(img.autoLevels
            ? [{ label: "Auto-Levels", clear: () => updateImage({ autoLevels: false }) }]
            : []),
          ...(img.filterPreset !== "none"
            ? [{ label: `Look: ${img.filterPreset}`, clear: () => updateImage({ filterPreset: "none" }) }]
            : []),
          ...(img.framePaddingPx > 0
            ? [{ label: `Frame ${img.framePaddingPx}px`, clear: () => updateImage({ framePaddingPx: 0 }) }]
            : []),
          ...(img.textWatermark
            ? [{ label: img.textWatermark.tiled ? "Tiled Watermark" : "Text Watermark", clear: () => updateImage({ textWatermark: null }) }]
            : []),
          ...(img.watermark
            ? [{ label: "Logo Watermark", clear: () => updateImage({ watermark: null }) }]
            : []),
          ...((img.blurRegions?.length ?? 0) > 0
            ? [{ label: `${img.blurRegions!.length} Redact Blur`, clear: () => updateImage({ blurRegions: [] }) }]
            : []),
          ...(img.removeBgColor
            ? [{ label: `Remove BG (${img.removeBgColor})`, clear: () => updateImage({ removeBgColor: null }) }]
            : []),
          ...((img.extraFormats?.length ?? 0) > 0
            ? [{ label: `+${img.extraFormats!.length} Extra Format${img.extraFormats!.length === 1 ? "" : "s"}`, clear: () => updateImage({ extraFormats: [] }) }]
            : []),
          ...(img.captionText?.trim()
            ? [{ label: "Caption Bar", clear: () => updateImage({ captionText: null }) }]
            : []),
          ...(img.duotone
            ? [{ label: "Duotone", clear: () => updateImage({ duotone: null }) }]
            : []),
        ]
      : [
          ...(vid.remuxOnly
            ? [{ label: "Fast Remux", clear: () => updateVideo({ remuxOnly: false }) }]
            : []),
          ...(vid.aspectReframe !== "none"
            ? [{ label: `Reframe ${vid.aspectReframe}`, clear: () => updateVideo({ aspectReframe: "none" }) }]
            : []),
          ...(vid.playbackMode !== "normal"
            ? [{ label: vid.playbackMode, clear: () => updateVideo({ playbackMode: "normal" }) }]
            : []),
          ...(vid.speed !== 1
            ? [{ label: `${vid.speed}× Speed`, clear: () => updateVideo({ speed: 1 }) }]
            : []),
          ...(vid.denoiseAudio
            ? [{ label: "Voice Denoise", clear: () => updateVideo({ denoiseAudio: false }) }]
            : []),
          ...(vid.normalizeLoudness
            ? [{ label: "EBU R128", clear: () => updateVideo({ normalizeLoudness: false }) }]
            : []),
          ...(vid.trimSilence
            ? [{ label: "Trim Silence", clear: () => updateVideo({ trimSilence: false }) }]
            : []),
          ...(vid.audioGainDb !== 0
            ? [{ label: `${vid.audioGainDb > 0 ? "+" : ""}${vid.audioGainDb}dB`, clear: () => updateVideo({ audioGainDb: 0 }) }]
            : []),
          ...(vid.muteAudio
            ? [{ label: "Muted", clear: () => updateVideo({ muteAudio: false }) }]
            : []),
        ];

  return (
    <div className="flex flex-1 flex-col gap-4 overflow-y-auto pr-1">
      {/* Quick Presets */}
      <div>
        <div className="mb-2 flex items-center justify-between">
          <div className="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider text-zinc-500 dark:text-zinc-400">
            <Sparkles className="h-3.5 w-3.5 text-indigo-500" />
            Presets
          </div>
          <div className="flex items-center gap-1">
            {customPresets.length > 0 && (
              <button
                type="button"
                disabled={disabled}
                onClick={onExportPresets}
                className="rounded-md p-1 text-zinc-400 transition hover:bg-zinc-200/70 hover:text-zinc-800 dark:hover:bg-zinc-800 dark:hover:text-zinc-200 cursor-pointer"
                title="Export custom presets to JSON"
              >
                <Download className="h-3 w-3" />
              </button>
            )}
            <button
              type="button"
              disabled={disabled}
              onClick={onImportPresets}
              className="rounded-md p-1 text-zinc-400 transition hover:bg-zinc-200/70 hover:text-zinc-800 dark:hover:bg-zinc-800 dark:hover:text-zinc-200 cursor-pointer"
              title="Import custom presets from JSON"
            >
              <Upload className="h-3 w-3" />
            </button>
            <button
              type="button"
              disabled={disabled}
              onClick={onResetSettings}
              className="rounded-md p-1 text-zinc-400 transition hover:bg-zinc-200/70 hover:text-zinc-800 dark:hover:bg-zinc-800 dark:hover:text-zinc-200 cursor-pointer"
              title="Reset all conversion settings to defaults"
            >
              <RotateCcw className="h-3 w-3" />
            </button>
            <button
              type="button"
              disabled={disabled}
              onClick={() => setSavingPreset((v) => !v)}
              className="inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-[11px] font-medium text-indigo-600 transition hover:bg-indigo-500/10 dark:text-indigo-400 cursor-pointer"
              title="Save current settings as a custom preset"
            >
              <Plus className="h-3 w-3" />
              Save
            </button>
          </div>
        </div>

        {savingPreset && (
          <form
            className="mb-2 flex gap-1.5"
            onSubmit={(e) => {
              e.preventDefault();
              const trimmed = newPresetName.trim();
              if (!trimmed) return;
              onSaveCustomPreset(trimmed);
              setNewPresetName("");
              setSavingPreset(false);
            }}
          >
            <input
              type="text"
              placeholder="Preset name..."
              value={newPresetName}
              onChange={(e) => setNewPresetName(e.target.value)}
              autoFocus
              className="min-w-0 flex-1 rounded-lg border border-indigo-500/50 bg-white px-2.5 py-1 text-xs text-zinc-900 outline-none dark:bg-zinc-900 dark:text-zinc-100"
            />
            <button
              type="submit"
              className="rounded-lg bg-indigo-600 px-2.5 py-1 text-xs font-semibold text-white hover:bg-indigo-500 cursor-pointer"
            >
              Add
            </button>
          </form>
        )}

        <div className="grid grid-cols-3 gap-1.5">
          {(
            [
              { id: "web_optimized", label: "Web" },
              { id: "high_quality", label: "High Quality" },
              { id: "small_size", label: "Small Size" },
            ] as { id: Preset; label: string }[]
          ).map((p) => (
            <button
              key={p.id}
              type="button"
              disabled={disabled}
              onClick={() => onApplyPreset(p.id)}
              className={`rounded-xl px-2.5 py-2 text-xs font-medium transition cursor-pointer disabled:opacity-50 ${
                activePreset === p.id
                  ? "bg-indigo-600 text-white shadow-sm shadow-indigo-500/25"
                  : "border border-zinc-200 bg-white text-zinc-700 hover:bg-zinc-100 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 dark:hover:bg-zinc-800"
              }`}
            >
              {p.label}
            </button>
          ))}
        </div>

        {customPresets.length > 0 && (
          <div className="mt-1.5 flex flex-wrap gap-1.5">
            {customPresets.map((cp) => (
              <div
                key={cp.id}
                className={`inline-flex items-center gap-1 rounded-lg border pl-2 pr-1 py-1 text-[11px] font-medium transition ${
                  activePreset === cp.id
                    ? "border-indigo-500 bg-indigo-600 text-white"
                    : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                }`}
              >
                <button
                  type="button"
                  disabled={disabled}
                  onClick={() => onApplyCustomPreset(cp)}
                  className="cursor-pointer"
                >
                  {cp.name}
                </button>
                <button
                  type="button"
                  disabled={disabled}
                  onClick={() => onDeleteCustomPreset(cp.id)}
                  className="rounded p-0.5 opacity-60 hover:opacity-100 cursor-pointer"
                  title="Delete preset"
                >
                  <X className="h-2.5 w-2.5" />
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Media Type Switcher */}
      <div className="grid grid-cols-2 rounded-xl bg-zinc-200/70 p-1 dark:bg-zinc-900">
        <button
          type="button"
          onClick={() => setManualTab("image")}
          className={`flex items-center justify-center gap-1.5 rounded-lg py-1.5 text-xs font-semibold transition cursor-pointer ${
            activeTab === "image"
              ? "bg-white text-zinc-900 shadow-sm dark:bg-zinc-800 dark:text-zinc-100"
              : "text-zinc-500 hover:text-zinc-800 dark:text-zinc-400 dark:hover:text-zinc-200"
          }`}
        >
          <ImageIcon className="h-3.5 w-3.5" />
          Images {imageCount > 0 ? `(${imageCount})` : ""}
        </button>
        <button
          type="button"
          onClick={() => setManualTab("video")}
          className={`flex items-center justify-center gap-1.5 rounded-lg py-1.5 text-xs font-semibold transition cursor-pointer ${
            activeTab === "video"
              ? "bg-white text-zinc-900 shadow-sm dark:bg-zinc-800 dark:text-zinc-100"
              : "text-zinc-500 hover:text-zinc-800 dark:text-zinc-400 dark:hover:text-zinc-200"
          }`}
        >
          <Film className="h-3.5 w-3.5" />
          Video &amp; Audio {videoCount > 0 ? `(${videoCount})` : ""}
        </button>
      </div>

      {/* Active Modifiers Strip (prevents hidden forgotten filters) */}
      {activeChips.length > 0 && (
        <div className="flex flex-wrap items-center gap-1 rounded-xl border border-indigo-500/20 bg-indigo-500/5 px-2.5 py-2">
          {activeChips.map((chip) => (
            <button
              key={chip.label}
              type="button"
              disabled={disabled}
              onClick={chip.clear}
              className="inline-flex items-center gap-1 rounded-md bg-indigo-500/15 px-1.5 py-0.5 text-[10px] font-semibold text-indigo-700 transition hover:bg-indigo-500/25 dark:text-indigo-300 cursor-pointer"
              title="Click to turn off"
            >
              {chip.label}
              <X className="h-2.5 w-2.5" />
            </button>
          ))}
        </div>
      )}

      {/* Essentials & Progressive-Disclosure Accordions */}
      {activeTab === "image" ? (
        <div className="space-y-3">
          {pdfCount > 0 && (
            <div className="space-y-2 rounded-xl border border-rose-500/25 bg-rose-500/5 p-3">
              <div className="flex items-center gap-1.5 text-xs font-semibold text-rose-600 dark:text-rose-400">
                <FileText className="h-3.5 w-3.5" />
                PDF → Images ({pdfCount})
              </div>
              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="mb-1 block text-[10px] text-zinc-500">
                    Render DPI
                  </label>
                  <select
                    disabled={disabled}
                    value={settings.pdf.dpi}
                    onChange={(e) =>
                      onChange({
                        ...settings,
                        pdf: { ...settings.pdf, dpi: Number(e.target.value) },
                      })
                    }
                    className="w-full rounded-lg border border-zinc-200 bg-white px-2 py-1.5 text-xs text-zinc-900 outline-none dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
                  >
                    <option value={72}>72 DPI (Screen)</option>
                    <option value={150}>150 DPI (Standard)</option>
                    <option value={300}>300 DPI (Print HD)</option>
                  </select>
                </div>
                <div>
                  <label className="mb-1 block text-[10px] text-zinc-500">
                    Pages (optional)
                  </label>
                  <input
                    type="text"
                    placeholder="All (e.g. 1-5)"
                    disabled={disabled}
                    value={settings.pdf.pageRange}
                    onChange={(e) =>
                      onChange({
                        ...settings,
                        pdf: { ...settings.pdf, pageRange: e.target.value },
                      })
                    }
                    className="w-full rounded-lg border border-zinc-200 bg-white px-2 py-1.5 text-xs text-zinc-900 outline-none dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
                  />
                </div>
              </div>
            </div>
          )}

          {/* Output Format */}
          <div>
            <label className="mb-1.5 block text-xs font-medium text-zinc-600 dark:text-zinc-400">
              Output Format
            </label>
            <div className="grid grid-cols-5 gap-1.5">
              {IMAGE_OUTPUT_FORMATS.map((fmt) => (
                <button
                  key={fmt.value}
                  type="button"
                  disabled={disabled}
                  onClick={() =>
                    updateImage({ format: fmt.value as ImageFormat })
                  }
                  className={`rounded-lg py-1.5 text-[11px] font-semibold transition cursor-pointer disabled:opacity-50 ${
                    img.format === fmt.value
                      ? "bg-indigo-600 text-white shadow-sm"
                      : "border border-zinc-200 bg-white text-zinc-700 hover:border-indigo-400 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                  }`}
                >
                  {fmt.label}
                </button>
              ))}
            </div>

            {/* Also Export As (Single-Decode Multi-Format Fan-Out) */}
            <div className="mt-2 flex flex-wrap items-center gap-1">
              <span className="text-[10px] font-medium text-zinc-500">
                Also export:
              </span>
              {(["webp", "avif", "jpg", "png"] as ImageFormat[])
                .filter((f) => f !== img.format)
                .map((extraFmt) => {
                  const active = (img.extraFormats ?? []).includes(extraFmt);
                  return (
                    <button
                      key={extraFmt}
                      type="button"
                      disabled={disabled}
                      onClick={() => {
                        const prev = img.extraFormats ?? [];
                        const next = active
                          ? prev.filter((f) => f !== extraFmt)
                          : [...prev, extraFmt];
                        updateImage({ extraFormats: next });
                      }}
                      className={`rounded-md px-1.5 py-0.5 text-[10px] font-semibold uppercase transition cursor-pointer disabled:opacity-50 ${
                        active
                          ? "bg-emerald-600 text-white"
                          : "border border-zinc-200 bg-white text-zinc-500 hover:border-emerald-400 hover:text-emerald-600 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-400"
                      }`}
                    >
                      +{extraFmt}
                    </button>
                  );
                })}
            </div>
          </div>

          {/* Quality / Lossless / Target Size */}
          {imgFormatMeta?.supportsQuality && (
            <div className="space-y-2 rounded-xl border border-zinc-200/80 bg-white/70 p-3 dark:border-zinc-800/80 dark:bg-zinc-900/40">
              <div className="flex items-center justify-between text-xs">
                <span className="font-medium text-zinc-600 dark:text-zinc-400">
                  Quality
                </span>
                <span className="font-semibold text-indigo-600 dark:text-indigo-400">
                  {img.lossless ? "Lossless" : `${img.quality}%`}
                </span>
              </div>
              <input
                type="range"
                min={10}
                max={100}
                value={img.quality}
                disabled={disabled || img.lossless}
                onChange={(e) =>
                  updateImage({ quality: Number(e.target.value) })
                }
                className="w-full accent-indigo-600 disabled:opacity-40"
              />
              {imgFormatMeta.supportsLossless &&
                img.format !== "png" &&
                img.format !== "bmp" &&
                img.format !== "tiff" && (
                  <label className="flex items-center justify-between pt-0.5 text-[11px] font-medium text-zinc-600 dark:text-zinc-400 cursor-pointer">
                    <span>Lossless compression</span>
                    <input
                      type="checkbox"
                      checked={img.lossless}
                      disabled={disabled}
                      onChange={(e) =>
                        updateImage({ lossless: e.target.checked })
                      }
                      className="h-3.5 w-3.5 accent-indigo-600"
                    />
                  </label>
                )}
              {!img.lossless && (
                <div className="flex items-center gap-2 pt-1">
                  <span className="text-[11px] text-zinc-500">Max KB</span>
                  <input
                    type="number"
                    min={10}
                    placeholder="Auto"
                    disabled={disabled}
                    value={img.targetSizeKb ?? ""}
                    onChange={(e) =>
                      updateImage({
                        targetSizeKb: e.target.value
                          ? Number(e.target.value)
                          : null,
                      })
                    }
                    className="w-full rounded-lg border border-zinc-200 bg-white px-2 py-1 text-xs text-zinc-900 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
                  />
                </div>
              )}
            </div>
          )}

          {/* Accordion 1: Transform & Crop */}
          <SectionAccordion
            title="Transform & Crop"
            icon={<Crop className="h-3.5 w-3.5" />}
            open={openSections.img_transform}
            onToggle={() => toggleSection("img_transform")}
            activeCount={imgTransformCount}
          >
            <div className="flex items-center gap-1.5">
              <button
                type="button"
                disabled={disabled}
                onClick={() => updateImage({ rotate: (img.rotate + 90) % 360 })}
                className={`flex flex-1 items-center justify-center gap-1 rounded-lg border py-1.5 text-[11px] font-semibold cursor-pointer ${
                  img.rotate > 0
                    ? "border-indigo-500 bg-indigo-600 text-white"
                    : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                }`}
              >
                <RotateCw className="h-3.5 w-3.5" />
                {img.rotate}°
              </button>
              <button
                type="button"
                disabled={disabled}
                onClick={() =>
                  updateImage({ flipHorizontal: !img.flipHorizontal })
                }
                className={`flex flex-1 items-center justify-center gap-1 rounded-lg border py-1.5 text-[11px] font-semibold cursor-pointer ${
                  img.flipHorizontal
                    ? "border-indigo-500 bg-indigo-600 text-white"
                    : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                }`}
              >
                <FlipHorizontal2 className="h-3.5 w-3.5" />
                Flip H
              </button>
              <button
                type="button"
                disabled={disabled}
                onClick={() => updateImage({ flipVertical: !img.flipVertical })}
                className={`flex flex-1 items-center justify-center gap-1 rounded-lg border py-1.5 text-[11px] font-semibold cursor-pointer ${
                  img.flipVertical
                    ? "border-indigo-500 bg-indigo-600 text-white"
                    : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                }`}
              >
                <FlipVertical2 className="h-3.5 w-3.5" />
                Flip V
              </button>
            </div>

            <div>
              <div className="mb-1 text-[10px] font-semibold text-zinc-500">
                Center Crop Aspect
              </div>
              <div className="grid grid-cols-6 gap-1">
                {(
                  [
                    { id: null, label: "Off" },
                    { id: "square1x1", label: "1:1" },
                    { id: "landscape16x9", label: "16:9" },
                    { id: "portrait9x16", label: "9:16" },
                    { id: "standard4x3", label: "4:3" },
                    { id: "photo3x2", label: "3:2" },
                  ] as { id: AspectCrop | null; label: string }[]
                ).map((c) => (
                  <button
                    key={c.label}
                    type="button"
                    disabled={disabled}
                    onClick={() => updateImage({ cropAspect: c.id })}
                    className={`rounded-md py-1 text-[10px] font-semibold cursor-pointer ${
                      img.cropAspect === c.id
                        ? "bg-indigo-600 text-white"
                        : "border border-zinc-200 bg-white text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-400"
                    }`}
                  >
                    {c.label}
                  </button>
                ))}
              </div>
            </div>

            <div>
              <div className="mb-1 text-[10px] font-semibold text-zinc-500">
                Pad to Aspect (No-Crop)
              </div>
              <div className="grid grid-cols-6 gap-1">
                {(
                  [
                    { id: null, label: "Off" },
                    { id: "square1x1", label: "1:1" },
                    { id: "landscape16x9", label: "16:9" },
                    { id: "portrait9x16", label: "9:16" },
                    { id: "standard4x3", label: "4:3" },
                    { id: "photo3x2", label: "3:2" },
                  ] as { id: AspectCrop | null; label: string }[]
                ).map((c) => (
                  <button
                    key={c.label}
                    type="button"
                    disabled={disabled}
                    onClick={() => updateImage({ padAspect: c.id })}
                    className={`rounded-md py-1 text-[10px] font-semibold cursor-pointer ${
                      img.padAspect === c.id
                        ? "bg-indigo-600 text-white"
                        : "border border-zinc-200 bg-white text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-400"
                    }`}
                  >
                    {c.label}
                  </button>
                ))}
              </div>
            </div>

            <div className="space-y-1.5 rounded-lg border border-zinc-200/80 bg-zinc-50/70 px-2.5 py-2 dark:border-zinc-800 dark:bg-zinc-950/40">
              <label className="flex items-center justify-between text-xs font-medium text-zinc-700 dark:text-zinc-300 cursor-pointer">
                <span>Auto-Trim Borders</span>
                <input
                  type="checkbox"
                  checked={img.autoTrimBorder}
                  disabled={disabled}
                  onChange={(e) =>
                    updateImage({ autoTrimBorder: e.target.checked })
                  }
                  className="h-3.5 w-3.5 accent-indigo-600"
                />
              </label>
              {img.autoTrimBorder && (
                <div className="flex items-center gap-2 text-[10px] text-zinc-500">
                  <span className="w-16">Tolerance</span>
                  <input
                    type="range"
                    min={0}
                    max={64}
                    disabled={disabled}
                    value={img.autoTrimTolerance}
                    onChange={(e) =>
                      updateImage({ autoTrimTolerance: Number(e.target.value) })
                    }
                    className="flex-1 accent-indigo-600"
                  />
                  <span className="w-6 text-right">{img.autoTrimTolerance}</span>
                </div>
              )}
            </div>
          </SectionAccordion>

          {/* Accordion 2: Color & Effects */}
          <SectionAccordion
            title="Color & Effects"
            icon={<Sliders className="h-3.5 w-3.5" />}
            open={openSections.img_color}
            onToggle={() => toggleSection("img_color")}
            activeCount={imgColorCount}
          >
            <div className="grid grid-cols-3 gap-1.5">
              <button
                type="button"
                disabled={disabled}
                onClick={() => updateImage({ grayscale: !img.grayscale })}
                className={`rounded-lg border py-1.5 text-[10px] font-semibold cursor-pointer ${
                  img.grayscale
                    ? "border-indigo-500 bg-indigo-600 text-white"
                    : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                }`}
              >
                Grayscale
              </button>
              <button
                type="button"
                disabled={disabled}
                onClick={() => updateImage({ sharpen: !img.sharpen })}
                className={`rounded-lg border py-1.5 text-[10px] font-semibold cursor-pointer ${
                  img.sharpen
                    ? "border-indigo-500 bg-indigo-600 text-white"
                    : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                }`}
              >
                Sharpen
              </button>
              <button
                type="button"
                disabled={disabled}
                onClick={() => updateImage({ autoLevels: !img.autoLevels })}
                className={`rounded-lg border py-1.5 text-[10px] font-semibold cursor-pointer ${
                  img.autoLevels
                    ? "border-indigo-500 bg-indigo-600 text-white"
                    : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                }`}
              >
                Auto-Levels
              </button>
            </div>

            {/* Smart Solid / Auto Background Removal */}
            <div className="space-y-2 rounded-lg border border-zinc-200/80 bg-zinc-50/70 p-2.5 dark:border-zinc-800 dark:bg-zinc-900/50">
              <div className="flex items-center justify-between text-[11px] font-medium text-zinc-700 dark:text-zinc-300">
                <span>Remove Solid Background</span>
                <div className="flex items-center gap-1">
                  {(
                    [
                      { id: null, label: "Off" },
                      { id: "auto", label: "Auto" },
                      { id: "#ffffff", label: "White" },
                    ] as const
                  ).map((opt) => (
                    <button
                      key={opt.label}
                      type="button"
                      disabled={disabled}
                      onClick={() => updateImage({ removeBgColor: opt.id })}
                      className={`rounded px-1.5 py-0.5 text-[10px] font-semibold cursor-pointer ${
                        img.removeBgColor === opt.id
                          ? "bg-indigo-600 text-white"
                          : "bg-white text-zinc-600 hover:bg-zinc-200/70 dark:bg-zinc-800 dark:text-zinc-300"
                      }`}
                    >
                      {opt.label}
                    </button>
                  ))}
                </div>
              </div>
              {img.removeBgColor && (
                <label className="flex items-center justify-between gap-2 text-[10px] text-zinc-500">
                  <span>Tolerance ({img.removeBgTolerance ?? 24})</span>
                  <input
                    type="range"
                    min={4}
                    max={90}
                    disabled={disabled}
                    value={img.removeBgTolerance ?? 24}
                    onChange={(e) =>
                      updateImage({ removeBgTolerance: Number(e.target.value) })
                    }
                    className="w-28 accent-indigo-600"
                  />
                </label>
              )}
            </div>

            <div>
              <div className="mb-1 text-[10px] font-semibold text-zinc-500">
                Duotone Grading
              </div>
              <div className="mb-2.5 grid grid-cols-5 gap-1">
                {(
                  [
                    { label: "Off", pair: null },
                    { label: "Indigo", pair: ["#1e1b4b", "#c084fc"] as [string, string] },
                    { label: "Sunset", pair: ["#431407", "#fb923c"] as [string, string] },
                    { label: "Mint", pair: ["#064e3b", "#6ee7b7"] as [string, string] },
                    { label: "Cyber", pair: ["#0f172a", "#38bdf8"] as [string, string] },
                  ] as const
                ).map((d) => {
                  const isActive =
                    (d.pair === null && !img.duotone) ||
                    (d.pair !== null &&
                      img.duotone?.[0] === d.pair[0] &&
                      img.duotone?.[1] === d.pair[1]);
                  return (
                    <button
                      key={d.label}
                      type="button"
                      disabled={disabled}
                      onClick={() => updateImage({ duotone: d.pair })}
                      className={`rounded-md py-1 text-[10px] font-semibold transition cursor-pointer ${
                        isActive
                          ? "bg-indigo-600 text-white"
                          : "border border-zinc-200 bg-white text-zinc-600 hover:border-indigo-400 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                      }`}
                    >
                      {d.label}
                    </button>
                  );
                })}
              </div>

              <div className="mb-1 text-[10px] font-semibold text-zinc-500">
                Color Look Preset
              </div>
              <div className="grid grid-cols-4 gap-1">
                {(
                  [
                    { id: "none", label: "Off" },
                    { id: "sepia", label: "Sepia" },
                    { id: "warm", label: "Warm" },
                    { id: "cool", label: "Cool" },
                    { id: "vintage", label: "Vintage" },
                    { id: "high_contrast_bw", label: "B&W+" },
                    { id: "invert", label: "Invert" },
                  ] as { id: ImageFilterPreset; label: string }[]
                ).map((fp) => (
                  <button
                    key={fp.id}
                    type="button"
                    disabled={disabled}
                    onClick={() => updateImage({ filterPreset: fp.id })}
                    className={`rounded-md py-1 text-[10px] font-semibold cursor-pointer ${
                      img.filterPreset === fp.id
                        ? "bg-indigo-600 text-white"
                        : "border border-zinc-200 bg-white text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-400"
                    }`}
                  >
                    {fp.label}
                  </button>
                ))}
              </div>
            </div>

            <div className="space-y-1.5">
              {[
                { label: "Brightness", min: -50, max: 50, step: 1, val: img.brightness, set: (v: number) => updateImage({ brightness: v }), disp: `${img.brightness}` },
                { label: "Contrast", min: -50, max: 50, step: 1, val: img.contrast, set: (v: number) => updateImage({ contrast: v }), disp: `${img.contrast}` },
                { label: "Saturation", min: -100, max: 100, step: 1, val: img.saturation, set: (v: number) => updateImage({ saturation: v }), disp: `${img.saturation}` },
                { label: "Blur", min: 0, max: 20, step: 0.5, val: img.blurSigma, set: (v: number) => updateImage({ blurSigma: v }), disp: `${img.blurSigma}` },
                { label: "Pixelate", min: 0, max: 32, step: 2, val: img.pixelateBlock, set: (v: number) => updateImage({ pixelateBlock: v }), disp: `${img.pixelateBlock}` },
                { label: "Vignette", min: 0, max: 100, step: 1, val: Math.round(img.vignetteStrength * 100), set: (v: number) => updateImage({ vignetteStrength: v / 100 }), disp: `${Math.round(img.vignetteStrength * 100)}%` },
              ].map((s) => (
                <div key={s.label} className="flex items-center gap-2 text-[10px] text-zinc-500">
                  <span className="w-16">{s.label}</span>
                  <input
                    type="range"
                    min={s.min}
                    max={s.max}
                    step={s.step}
                    disabled={disabled}
                    value={s.val}
                    onChange={(e) => s.set(Number(e.target.value))}
                    className="flex-1 accent-indigo-600"
                  />
                  <span className="w-8 text-right">{s.disp}</span>
                </div>
              ))}
            </div>

            {(img.format === "jpg" || img.format === "bmp") && (
              <div className="flex items-center justify-between pt-1 text-xs">
                <span className="text-zinc-600 dark:text-zinc-400">
                  Transparency Fill
                </span>
                <div className="flex items-center gap-1.5">
                  {["#ffffff", "#000000"].map((hex) => (
                    <button
                      key={hex}
                      type="button"
                      disabled={disabled}
                      onClick={() => updateImage({ backgroundHex: hex })}
                      className={`h-5 w-5 rounded-md border cursor-pointer ${
                        img.backgroundHex === hex
                          ? "ring-2 ring-indigo-500"
                          : "border-zinc-300 dark:border-zinc-700"
                      }`}
                      style={{ backgroundColor: hex }}
                    />
                  ))}
                  <input
                    type="color"
                    disabled={disabled}
                    value={img.backgroundHex ?? "#ffffff"}
                    onChange={(e) =>
                      updateImage({ backgroundHex: e.target.value })
                    }
                    className="h-5 w-6 cursor-pointer rounded border-0 bg-transparent p-0"
                  />
                </div>
              </div>
            )}
          </SectionAccordion>

          {/* Accordion 3: Beautify & Frame */}
          <SectionAccordion
            title="Beautify & Frame"
            icon={<Frame className="h-3.5 w-3.5" />}
            open={openSections.img_frame}
            onToggle={() => toggleSection("img_frame")}
            activeCount={imgFrameCount}
          >
            <div className="flex items-center gap-2 text-[10px] text-zinc-500">
              <span className="w-20">Corners</span>
              <input
                type="range"
                min={0}
                max={50}
                disabled={disabled}
                value={img.cornerRadiusPercent}
                onChange={(e) =>
                  updateImage({ cornerRadiusPercent: Number(e.target.value) })
                }
                className="flex-1 accent-indigo-600"
              />
              <span className="w-8 text-right">{img.cornerRadiusPercent}%</span>
            </div>
            <div className="flex items-center gap-2 text-[10px] text-zinc-500">
              <span className="w-20">Frame Pad</span>
              <input
                type="range"
                min={0}
                max={80}
                step={4}
                disabled={disabled}
                value={img.framePaddingPx}
                onChange={(e) =>
                  updateImage({ framePaddingPx: Number(e.target.value) })
                }
                className="flex-1 accent-indigo-600"
              />
              <span className="w-8 text-right">{img.framePaddingPx}px</span>
            </div>
            <div className="space-y-1">
              <label className="block text-[10px] font-medium text-zinc-500">
                Caption Footer Bar (optional)
              </label>
              <input
                type="text"
                disabled={disabled}
                value={img.captionText ?? ""}
                onChange={(e) =>
                  updateImage({ captionText: e.target.value || null })
                }
                placeholder="e.g. Figure 1: Architecture Overview"
                className="w-full rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-[11px] text-zinc-800 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-200"
              />
            </div>
            {img.framePaddingPx > 0 && (
              <div className="space-y-2">
                <div className="grid grid-cols-6 gap-1">
                  {(
                    [
                      { id: "none", label: "Solid", cls: "bg-zinc-300 dark:bg-zinc-700" },
                      { id: "indigo", label: "Indigo", cls: "bg-gradient-to-br from-indigo-500 to-purple-600" },
                      { id: "sunset", label: "Sunset", cls: "bg-gradient-to-br from-orange-500 to-pink-500" },
                      { id: "ocean", label: "Ocean", cls: "bg-gradient-to-br from-cyan-500 to-blue-500" },
                      { id: "emerald", label: "Mint", cls: "bg-gradient-to-br from-emerald-500 to-teal-500" },
                      { id: "midnight", label: "Night", cls: "bg-gradient-to-br from-slate-900 to-indigo-900" },
                    ] as { id: FrameGradient; label: string; cls: string }[]
                  ).map((g) => (
                    <button
                      key={g.id}
                      type="button"
                      disabled={disabled}
                      onClick={() => updateImage({ frameGradient: g.id })}
                      title={g.label}
                      className={`h-5 rounded-md ${g.cls} cursor-pointer ${
                        img.frameGradient === g.id
                          ? "ring-2 ring-indigo-500 ring-offset-1 dark:ring-offset-zinc-900"
                          : "opacity-75 hover:opacity-100"
                      }`}
                    />
                  ))}
                </div>
                <label className="flex items-center gap-2 text-[11px] text-zinc-600 dark:text-zinc-400 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={img.dropShadow}
                    disabled={disabled}
                    onChange={(e) =>
                      updateImage({ dropShadow: e.target.checked })
                    }
                    className="h-3.5 w-3.5 accent-indigo-600"
                  />
                  Soft drop shadow
                </label>
              </div>
            )}
          </SectionAccordion>

          {/* Accordion 4: Watermark */}
          <SectionAccordion
            title="Watermark"
            icon={<Stamp className="h-3.5 w-3.5" />}
            open={openSections.img_watermark}
            onToggle={() => toggleSection("img_watermark")}
            activeCount={imgWatermarkCount}
          >
            <div className="flex items-center gap-1.5">
              <button
                type="button"
                disabled={disabled}
                onClick={onPickWatermark}
                className="flex min-w-0 flex-1 items-center gap-2 rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-left text-xs text-zinc-700 transition hover:border-indigo-400 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 cursor-pointer"
              >
                <ImageIcon className="h-3.5 w-3.5 shrink-0 text-indigo-500" />
                <span className="truncate">
                  {img.watermark
                    ? img.watermark.imagePath.split(/[\\/]/).pop()
                    : "Add PNG / SVG logo watermark..."}
                </span>
              </button>
              {img.watermark && (
                <button
                  type="button"
                  disabled={disabled}
                  onClick={() => updateImage({ watermark: null })}
                  className="rounded-lg p-1.5 text-zinc-400 hover:text-red-500 cursor-pointer"
                >
                  <X className="h-3.5 w-3.5" />
                </button>
              )}
            </div>
            {img.watermark && (
              <div className="space-y-2 rounded-lg border border-zinc-200/80 bg-zinc-50/70 p-2 dark:border-zinc-800 dark:bg-zinc-950/40">
                <div className="grid grid-cols-5 gap-1">
                  {(
                    [
                      { id: "top_left", label: "TL" },
                      { id: "top_right", label: "TR" },
                      { id: "center", label: "Mid" },
                      { id: "bottom_left", label: "BL" },
                      { id: "bottom_right", label: "BR" },
                    ] as { id: WatermarkPosition; label: string }[]
                  ).map((pos) => (
                    <button
                      key={pos.id}
                      type="button"
                      disabled={disabled}
                      onClick={() =>
                        updateImage({
                          watermark: { ...img.watermark!, position: pos.id },
                        })
                      }
                      className={`rounded py-1 text-[10px] font-semibold cursor-pointer ${
                        img.watermark?.position === pos.id
                          ? "bg-indigo-600 text-white"
                          : "bg-white text-zinc-600 dark:bg-zinc-800 dark:text-zinc-400"
                      }`}
                    >
                      {pos.label}
                    </button>
                  ))}
                </div>
                <div className="grid grid-cols-2 gap-2 text-[10px] text-zinc-500">
                  <div>
                    <div className="flex justify-between">
                      <span>Opacity</span>
                      <span>{Math.round(img.watermark.opacity * 100)}%</span>
                    </div>
                    <input
                      type="range"
                      min={10}
                      max={100}
                      value={Math.round(img.watermark.opacity * 100)}
                      onChange={(e) =>
                        updateImage({
                          watermark: {
                            ...img.watermark!,
                            opacity: Number(e.target.value) / 100,
                          },
                        })
                      }
                      className="w-full accent-indigo-600"
                    />
                  </div>
                  <div>
                    <div className="flex justify-between">
                      <span>Scale</span>
                      <span>{img.watermark.scalePercent}%</span>
                    </div>
                    <input
                      type="range"
                      min={5}
                      max={50}
                      value={img.watermark.scalePercent}
                      onChange={(e) =>
                        updateImage({
                          watermark: {
                            ...img.watermark!,
                            scalePercent: Number(e.target.value),
                          },
                        })
                      }
                      className="w-full accent-indigo-600"
                    />
                  </div>
                </div>
              </div>
            )}

            <input
              type="text"
              placeholder="Or add text watermark (e.g. © 2026 Wandel)..."
              disabled={disabled}
              value={img.textWatermark?.text ?? ""}
              onChange={(e) => {
                const val = e.target.value;
                if (!val) {
                  updateImage({ textWatermark: null });
                } else {
                  updateImage({
                    textWatermark: {
                      text: val,
                      fontSizePercent: img.textWatermark?.fontSizePercent ?? 4,
                      color: img.textWatermark?.color ?? "#ffffff",
                      opacity: img.textWatermark?.opacity ?? 0.75,
                      marginPx: img.textWatermark?.marginPx ?? 20,
                      position: img.textWatermark?.position ?? "bottom_right",
                      tiled: img.textWatermark?.tiled ?? false,
                    },
                  });
                }
              }}
              className="w-full rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs text-zinc-900 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
            />
            {img.textWatermark && (
              <label className="flex items-center gap-2 text-[11px] text-zinc-600 dark:text-zinc-400 cursor-pointer">
                <input
                  type="checkbox"
                  checked={Boolean(img.textWatermark.tiled)}
                  disabled={disabled}
                  onChange={(e) =>
                    updateImage({
                      textWatermark: {
                        ...img.textWatermark!,
                        tiled: e.target.checked,
                      },
                    })
                  }
                  className="h-3.5 w-3.5 accent-indigo-600"
                />
                Tile watermark diagonally across entire image
              </label>
            )}
          </SectionAccordion>
        </div>
      ) : (
        <div className="space-y-3">
          {/* Video Format */}
          <div>
            <label className="mb-1.5 block text-xs font-medium text-zinc-600 dark:text-zinc-400">
              Video Format
            </label>
            <div className="grid grid-cols-2 gap-1.5">
              {VIDEO_OUTPUT_FORMATS.map((fmt) => {
                const selected =
                  vid.target.kind === "video" && vid.target.format === fmt.value;
                return (
                  <button
                    key={fmt.value}
                    type="button"
                    disabled={disabled}
                    onClick={() =>
                      updateVideo({
                        target: { kind: "video", format: fmt.value as VideoFormat },
                      })
                    }
                    className={`rounded-lg px-2 py-1.5 text-[11px] font-semibold transition cursor-pointer disabled:opacity-50 ${
                      selected
                        ? "bg-indigo-600 text-white shadow-sm"
                        : "border border-zinc-200 bg-white text-zinc-700 hover:border-indigo-400 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                    }`}
                  >
                    {fmt.label}
                  </button>
                );
              })}
            </div>
          </div>

          {/* Extract Audio */}
          <div>
            <label className="mb-1.5 block text-xs font-medium text-zinc-600 dark:text-zinc-400">
              Extract / Convert Audio
            </label>
            <div className="grid grid-cols-3 gap-1.5">
              {AUDIO_OUTPUT_FORMATS.map((fmt) => {
                const selected =
                  vid.target.kind === "audio" && vid.target.format === fmt.value;
                return (
                  <button
                    key={fmt.value}
                    type="button"
                    disabled={disabled}
                    onClick={() =>
                      updateVideo({
                        target: { kind: "audio", format: fmt.value as AudioFormat },
                      })
                    }
                    className={`rounded-lg px-2 py-1.5 text-[11px] font-semibold transition cursor-pointer disabled:opacity-50 ${
                      selected
                        ? "bg-violet-600 text-white shadow-sm"
                        : "border border-zinc-200 bg-white text-zinc-700 hover:border-violet-400 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                    }`}
                  >
                    {fmt.value.toUpperCase()}
                  </button>
                );
              })}
            </div>
          </div>

          {/* Quality + Fast Remux + GPU */}
          {vid.target.kind === "video" && (
            <div className="space-y-2 rounded-xl border border-zinc-200/80 bg-white/70 p-3 dark:border-zinc-800/80 dark:bg-zinc-900/40">
              <div className="flex items-center justify-between text-xs">
                <span className="font-medium text-zinc-600 dark:text-zinc-400">
                  Video Quality
                </span>
                <span className="font-semibold text-indigo-600 dark:text-indigo-400">
                  {vid.remuxOnly ? "Lossless Copy" : `${vid.quality}%`}
                </span>
              </div>
              <input
                type="range"
                min={10}
                max={100}
                value={vid.quality}
                disabled={disabled || vid.remuxOnly}
                onChange={(e) => updateVideo({ quality: Number(e.target.value) })}
                className="w-full accent-indigo-600 disabled:opacity-40"
              />
              <label className="flex items-center justify-between pt-0.5 text-[11px] font-medium text-zinc-700 dark:text-zinc-300 cursor-pointer">
                <span>Fast Remux (No Re-encode)</span>
                <input
                  type="checkbox"
                  checked={vid.remuxOnly}
                  disabled={disabled}
                  onChange={(e) => updateVideo({ remuxOnly: e.target.checked })}
                  className="h-3.5 w-3.5 accent-indigo-600"
                />
              </label>
              <label className="flex items-center justify-between text-[11px] font-medium text-zinc-700 dark:text-zinc-300 cursor-pointer">
                <span className="flex items-center gap-1">
                  <Zap className="h-3 w-3 text-amber-500" />
                  GPU ({hardware?.gpuBackendName ?? "Auto"})
                </span>
                <input
                  type="checkbox"
                  checked={vid.useHardwareAccel}
                  disabled={disabled || vid.remuxOnly}
                  onChange={(e) =>
                    updateVideo({ useHardwareAccel: e.target.checked })
                  }
                  className="h-3.5 w-3.5 accent-indigo-600"
                />
              </label>
            </div>
          )}

          {/* Video Accordion 1: Trim, Speed & Loop */}
          {vid.target.kind === "video" && (
            <SectionAccordion
              title="Trim, Speed & Loop"
              icon={<Scissors className="h-3.5 w-3.5" />}
              open={openSections.vid_trim}
              onToggle={() => toggleSection("vid_trim")}
              activeCount={vidTrimCount}
            >
              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="mb-1 block text-[10px] text-zinc-500">
                    Trim Start (s)
                  </label>
                  <input
                    type="number"
                    min={0}
                    step={0.5}
                    placeholder="0"
                    disabled={disabled}
                    value={vid.trimStartSecs ?? ""}
                    onChange={(e) =>
                      updateVideo({
                        trimStartSecs: e.target.value
                          ? Number(e.target.value)
                          : null,
                      })
                    }
                    className="w-full rounded-lg border border-zinc-200 bg-white px-2 py-1 text-xs dark:border-zinc-800 dark:bg-zinc-900"
                  />
                </div>
                <div>
                  <label className="mb-1 block text-[10px] text-zinc-500">
                    Trim End (s)
                  </label>
                  <input
                    type="number"
                    min={0}
                    step={0.5}
                    placeholder="Full"
                    disabled={disabled}
                    value={vid.trimEndSecs ?? ""}
                    onChange={(e) =>
                      updateVideo({
                        trimEndSecs: e.target.value
                          ? Number(e.target.value)
                          : null,
                      })
                    }
                    className="w-full rounded-lg border border-zinc-200 bg-white px-2 py-1 text-xs dark:border-zinc-800 dark:bg-zinc-900"
                  />
                </div>
                <div>
                  <label className="mb-1 block text-[10px] text-zinc-500">
                    Max Size (MB)
                  </label>
                  <input
                    type="number"
                    min={0.5}
                    step={0.5}
                    placeholder="Auto"
                    disabled={disabled}
                    value={vid.targetSizeMb ?? ""}
                    onChange={(e) =>
                      updateVideo({
                        targetSizeMb: e.target.value
                          ? Number(e.target.value)
                          : null,
                      })
                    }
                    className="w-full rounded-lg border border-zinc-200 bg-white px-2 py-1 text-xs dark:border-zinc-800 dark:bg-zinc-900"
                  />
                </div>
                <div>
                  <label className="mb-1 block text-[10px] text-zinc-500">
                    Max FPS
                  </label>
                  <input
                    type="number"
                    min={5}
                    max={120}
                    placeholder="Original"
                    disabled={disabled}
                    value={vid.maxFps ?? ""}
                    onChange={(e) =>
                      updateVideo({
                        maxFps: e.target.value ? Number(e.target.value) : null,
                      })
                    }
                    className="w-full rounded-lg border border-zinc-200 bg-white px-2 py-1 text-xs dark:border-zinc-800 dark:bg-zinc-900"
                  />
                </div>
              </div>

              <div className="grid grid-cols-3 gap-1">
                {(
                  [
                    { id: "normal", label: "Normal" },
                    { id: "reverse", label: "Reverse" },
                    { id: "boomerang", label: "Boomerang" },
                  ] as { id: VideoPlaybackMode; label: string }[]
                ).map((pm) => (
                  <button
                    key={pm.id}
                    type="button"
                    disabled={disabled}
                    onClick={() => updateVideo({ playbackMode: pm.id })}
                    className={`rounded-md py-1 text-[10px] font-semibold cursor-pointer ${
                      vid.playbackMode === pm.id
                        ? "bg-indigo-600 text-white"
                        : "border border-zinc-200 bg-white text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-400"
                    }`}
                  >
                    {pm.label}
                  </button>
                ))}
              </div>

              <div className="grid grid-cols-5 gap-1">
                {[0.5, 1, 1.5, 2, 4].map((sp) => (
                  <button
                    key={sp}
                    type="button"
                    disabled={disabled}
                    onClick={() => updateVideo({ speed: sp })}
                    className={`rounded-md py-1 text-[10px] font-semibold cursor-pointer ${
                      vid.speed === sp
                        ? "bg-indigo-600 text-white"
                        : "border border-zinc-200 bg-white text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-400"
                    }`}
                  >
                    {sp}×
                  </button>
                ))}
              </div>
            </SectionAccordion>
          )}

          {/* Video Accordion 2: Reframe & Restore */}
          {vid.target.kind === "video" &&
            vid.target.format !== "gif" &&
            vid.target.format !== "animated_webp" && (
              <SectionAccordion
                title="Reframe & Restore"
                icon={<Wand2 className="h-3.5 w-3.5" />}
                open={openSections.vid_reframe}
                onToggle={() => toggleSection("vid_reframe")}
                activeCount={vidReframeCount}
              >
                <div>
                  <div className="mb-1 text-[10px] font-semibold text-zinc-500">
                    Blur-Pad Aspect Reframe
                  </div>
                  <div className="grid grid-cols-5 gap-1">
                    {(
                      [
                        { id: "none", label: "Off" },
                        { id: "portrait9x16", label: "9:16" },
                        { id: "portrait4x5", label: "4:5" },
                        { id: "square1x1", label: "1:1" },
                        { id: "landscape16x9", label: "16:9" },
                      ] as { id: VideoAspectReframe; label: string }[]
                    ).map((rf) => (
                      <button
                        key={rf.id}
                        type="button"
                        disabled={disabled}
                        onClick={() => updateVideo({ aspectReframe: rf.id })}
                        className={`rounded-md py-1 text-[10px] font-semibold cursor-pointer ${
                          vid.aspectReframe === rf.id
                            ? "bg-indigo-600 text-white"
                            : "border border-zinc-200 bg-white text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-400"
                        }`}
                      >
                        {rf.label}
                      </button>
                    ))}
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-1.5">
                  <button
                    type="button"
                    disabled={disabled}
                    onClick={() =>
                      updateVideo({ rotate: (vid.rotate + 90) % 360 })
                    }
                    className={`flex items-center justify-center gap-1 rounded-lg border py-1.5 text-[11px] font-semibold cursor-pointer ${
                      vid.rotate > 0
                        ? "border-indigo-500 bg-indigo-600 text-white"
                        : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                    }`}
                  >
                    <RotateCw className="h-3 w-3" />
                    Rotate {vid.rotate}°
                  </button>
                  <button
                    type="button"
                    disabled={disabled}
                    onClick={() =>
                      updateVideo({ flipHorizontal: !vid.flipHorizontal })
                    }
                    className={`flex items-center justify-center gap-1 rounded-lg border py-1.5 text-[11px] font-semibold cursor-pointer ${
                      vid.flipHorizontal
                        ? "border-indigo-500 bg-indigo-600 text-white"
                        : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                    }`}
                  >
                    <FlipHorizontal2 className="h-3 w-3" />
                    Flip H
                  </button>
                  <button
                    type="button"
                    disabled={disabled}
                    onClick={() =>
                      updateVideo({ deinterlace: !vid.deinterlace })
                    }
                    className={`rounded-lg border py-1.5 text-[11px] font-semibold cursor-pointer ${
                      vid.deinterlace
                        ? "border-indigo-500 bg-indigo-600 text-white"
                        : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                    }`}
                  >
                    Deinterlace
                  </button>
                  <button
                    type="button"
                    disabled={disabled}
                    onClick={() =>
                      updateVideo({ videoSharpen: !vid.videoSharpen })
                    }
                    className={`rounded-lg border py-1.5 text-[11px] font-semibold cursor-pointer ${
                      vid.videoSharpen
                        ? "border-indigo-500 bg-indigo-600 text-white"
                        : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                    }`}
                  >
                    Sharpen
                  </button>
                </div>
              </SectionAccordion>
            )}

          {/* Video/Audio Accordion 3: Audio & Subtitles */}
          <SectionAccordion
            title="Audio, Subtitles & Overlays"
            icon={<Volume2 className="h-3.5 w-3.5" />}
            open={openSections.vid_audio}
            onToggle={() => toggleSection("vid_audio")}
            activeCount={vidAudioCount}
          >
            {vid.target.kind === "audio" && (
              <div className="space-y-2">
                {vid.target.format !== "wav" && vid.target.format !== "flac" && (
                  <div className="grid grid-cols-4 gap-1">
                    {[128, 192, 256, 320].map((br) => (
                      <button
                        key={br}
                        type="button"
                        disabled={disabled}
                        onClick={() => updateVideo({ audioBitrateKbps: br })}
                        className={`rounded-md py-1 text-[10px] font-semibold cursor-pointer ${
                          vid.audioBitrateKbps === br
                            ? "bg-indigo-600 text-white"
                            : "border border-zinc-200 bg-white text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-400"
                        }`}
                      >
                        {br}k
                      </button>
                    ))}
                  </div>
                )}
                <div className="grid grid-cols-2 gap-2">
                  <select
                    disabled={disabled}
                    value={vid.sampleRateHz ?? ""}
                    onChange={(e) =>
                      updateVideo({
                        sampleRateHz: e.target.value
                          ? Number(e.target.value)
                          : null,
                      })
                    }
                    className="rounded-lg border border-zinc-200 bg-white px-2 py-1 text-xs dark:border-zinc-800 dark:bg-zinc-900"
                  >
                    <option value="">Original Hz</option>
                    <option value={44100}>44.1 kHz</option>
                    <option value={48000}>48 kHz</option>
                  </select>
                  <label className="flex items-center justify-between rounded-lg border border-zinc-200 bg-white px-2.5 py-1 text-xs dark:border-zinc-800 dark:bg-zinc-900 cursor-pointer">
                    <span>Mono</span>
                    <input
                      type="checkbox"
                      checked={vid.mono}
                      disabled={disabled}
                      onChange={(e) => updateVideo({ mono: e.target.checked })}
                      className="h-3.5 w-3.5 accent-indigo-600"
                    />
                  </label>
                </div>
                <div className="grid grid-cols-5 gap-1">
                  {[0.75, 1, 1.25, 1.5, 2].map((sp) => (
                    <button
                      key={sp}
                      type="button"
                      disabled={disabled}
                      onClick={() => updateVideo({ speed: sp })}
                      className={`rounded-md py-1 text-[10px] font-semibold cursor-pointer ${
                        vid.speed === sp
                          ? "bg-indigo-600 text-white"
                          : "border border-zinc-200 bg-white text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-400"
                      }`}
                    >
                      {sp}×
                    </button>
                  ))}
                </div>
                <div className="grid grid-cols-2 gap-2">
                  <input
                    type="number"
                    min={0}
                    step={0.5}
                    placeholder="Fade In (s)"
                    disabled={disabled}
                    value={vid.fadeInSecs ?? ""}
                    onChange={(e) =>
                      updateVideo({
                        fadeInSecs: e.target.value
                          ? Number(e.target.value)
                          : null,
                      })
                    }
                    className="rounded-lg border border-zinc-200 bg-white px-2 py-1 text-xs dark:border-zinc-800 dark:bg-zinc-900"
                  />
                  <input
                    type="number"
                    min={0}
                    step={0.5}
                    placeholder="Fade Out (s)"
                    disabled={disabled}
                    value={vid.fadeOutSecs ?? ""}
                    onChange={(e) =>
                      updateVideo({
                        fadeOutSecs: e.target.value
                          ? Number(e.target.value)
                          : null,
                      })
                    }
                    className="rounded-lg border border-zinc-200 bg-white px-2 py-1 text-xs dark:border-zinc-800 dark:bg-zinc-900"
                  />
                </div>
              </div>
            )}

            <div className="flex items-center gap-2 text-[10px] text-zinc-500">
              <span className="w-16">Gain (dB)</span>
              <input
                type="range"
                min={-12}
                max={12}
                step={1}
                disabled={disabled}
                value={vid.audioGainDb}
                onChange={(e) =>
                  updateVideo({ audioGainDb: Number(e.target.value) })
                }
                className="flex-1 accent-indigo-600"
              />
              <span className="w-10 text-right">
                {vid.audioGainDb > 0 ? `+${vid.audioGainDb}` : vid.audioGainDb}
                dB
              </span>
            </div>

            <div className="space-y-1.5 text-xs">
              <label className="flex items-center justify-between text-zinc-700 dark:text-zinc-300 cursor-pointer">
                <span>Voice Denoise (FFT)</span>
                <input
                  type="checkbox"
                  checked={vid.denoiseAudio}
                  disabled={disabled}
                  onChange={(e) =>
                    updateVideo({ denoiseAudio: e.target.checked })
                  }
                  className="h-3.5 w-3.5 accent-indigo-600"
                />
              </label>
              <label className="flex items-center justify-between text-zinc-700 dark:text-zinc-300 cursor-pointer">
                <span>Normalize Loudness (EBU R128)</span>
                <input
                  type="checkbox"
                  checked={vid.normalizeLoudness}
                  disabled={disabled}
                  onChange={(e) =>
                    updateVideo({ normalizeLoudness: e.target.checked })
                  }
                  className="h-3.5 w-3.5 accent-indigo-600"
                />
              </label>
              {vid.target.kind === "audio" && (
                <label className="flex items-center justify-between text-zinc-700 dark:text-zinc-300 cursor-pointer">
                  <span>Trim Silence</span>
                  <input
                    type="checkbox"
                    checked={vid.trimSilence}
                    disabled={disabled}
                    onChange={(e) =>
                      updateVideo({ trimSilence: e.target.checked })
                    }
                    className="h-3.5 w-3.5 accent-indigo-600"
                  />
                </label>
              )}
              {vid.target.kind === "video" && (
                <>
                  <label className="flex items-center justify-between text-zinc-700 dark:text-zinc-300 cursor-pointer">
                    <span className="flex items-center gap-1.5">
                      <VolumeX className="h-3.5 w-3.5 text-zinc-400" />
                      Mute / Remove Audio
                    </span>
                    <input
                      type="checkbox"
                      checked={vid.muteAudio}
                      disabled={disabled}
                      onChange={(e) =>
                        updateVideo({ muteAudio: e.target.checked })
                      }
                      className="h-3.5 w-3.5 accent-indigo-600"
                    />
                  </label>
                  <div className="space-y-1.5 pt-1">
                    <div className="flex items-center gap-1">
                      <button
                        type="button"
                        disabled={disabled}
                        onClick={onPickSubtitle}
                        className="flex min-w-0 flex-1 rounded-lg border border-zinc-200 bg-white px-2 py-1 text-left text-[11px] dark:border-zinc-800 dark:bg-zinc-900 cursor-pointer"
                      >
                        <span className="truncate">
                          {vid.subtitlePath
                            ? `Sub: ${vid.subtitlePath.split(/[\\/]/).pop()}`
                            : "Burn-in subtitles (.srt / .vtt)..."}
                        </span>
                      </button>
                      {vid.subtitlePath && (
                        <button
                          type="button"
                          onClick={() => updateVideo({ subtitlePath: null })}
                          className="p-1 text-zinc-400 hover:text-red-500 cursor-pointer"
                        >
                          <X className="h-3 w-3" />
                        </button>
                      )}
                    </div>
                    <div className="flex items-center gap-1">
                      <button
                        type="button"
                        disabled={disabled}
                        onClick={onPickReplaceAudio}
                        className="flex min-w-0 flex-1 rounded-lg border border-zinc-200 bg-white px-2 py-1 text-left text-[11px] dark:border-zinc-800 dark:bg-zinc-900 cursor-pointer"
                      >
                        <span className="truncate">
                          {vid.replaceAudioPath
                            ? `Audio: ${vid.replaceAudioPath.split(/[\\/]/).pop()}`
                            : "Replace / add audio track..."}
                        </span>
                      </button>
                      {vid.replaceAudioPath && (
                        <button
                          type="button"
                          onClick={() => updateVideo({ replaceAudioPath: null })}
                          className="p-1 text-zinc-400 hover:text-red-500 cursor-pointer"
                        >
                          <X className="h-3 w-3" />
                        </button>
                      )}
                    </div>
                    <div className="flex items-center gap-1">
                      <button
                        type="button"
                        disabled={disabled}
                        onClick={onPickVideoWatermark}
                        className="flex min-w-0 flex-1 rounded-lg border border-zinc-200 bg-white px-2 py-1 text-left text-[11px] dark:border-zinc-800 dark:bg-zinc-900 cursor-pointer"
                      >
                        <span className="truncate">
                          {vid.watermark
                            ? `Logo: ${vid.watermark.imagePath.split(/[\\/]/).pop()}`
                            : "Add video logo watermark..."}
                        </span>
                      </button>
                      {vid.watermark && (
                        <button
                          type="button"
                          onClick={() => updateVideo({ watermark: null })}
                          className="p-1 text-zinc-400 hover:text-red-500 cursor-pointer"
                        >
                          <X className="h-3 w-3" />
                        </button>
                      )}
                    </div>
                  </div>
                </>
              )}
            </div>
          </SectionAccordion>
        </div>
      )}

      {/* Shared Accordion: Resize */}
      {!(activeTab === "video" && vid.target.kind === "audio") && (
        <SectionAccordion
          title="Resize"
          icon={<Maximize2 className="h-3.5 w-3.5" />}
          open={openSections.resize}
          onToggle={() => toggleSection("resize")}
          activeCount={currentResize.mode !== "original" ? 1 : 0}
        >
          <div className="grid grid-cols-4 gap-1 rounded-xl bg-zinc-200/70 p-1 dark:bg-zinc-900">
            {(
              [
                { id: "original", label: "Original" },
                { id: "max_dimensions", label: "Max" },
                { id: "exact", label: "Exact" },
                { id: "percentage", label: "%" },
              ] as const
            ).map((opt) => (
              <button
                key={opt.id}
                type="button"
                disabled={disabled}
                onClick={() => {
                  if (opt.id === "original") setResize({ mode: "original" });
                  else if (opt.id === "max_dimensions")
                    setResize({
                      mode: "max_dimensions",
                      width: 1920,
                      height: null,
                    });
                  else if (opt.id === "exact")
                    setResize({
                      mode: "exact",
                      width: 1280,
                      height: 720,
                      keep_aspect_ratio: true,
                    });
                  else setResize({ mode: "percentage", percent: 50 });
                }}
                className={`rounded-lg py-1 text-[11px] font-semibold cursor-pointer ${
                  currentResize.mode === opt.id
                    ? "bg-white text-zinc-900 shadow-sm dark:bg-zinc-800 dark:text-zinc-100"
                    : "text-zinc-500 dark:text-zinc-400"
                }`}
              >
                {opt.label}
              </button>
            ))}
          </div>

          {currentResize.mode === "max_dimensions" && (
            <div className="grid grid-cols-2 gap-2">
              <input
                type="number"
                min={16}
                placeholder="Max Width"
                disabled={disabled}
                value={currentResize.width ?? ""}
                onChange={(e) =>
                  setResize({
                    ...currentResize,
                    width: e.target.value ? Number(e.target.value) : null,
                  })
                }
                className="rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs dark:border-zinc-800 dark:bg-zinc-900"
              />
              <input
                type="number"
                min={16}
                placeholder="Max Height"
                disabled={disabled}
                value={currentResize.height ?? ""}
                onChange={(e) =>
                  setResize({
                    ...currentResize,
                    height: e.target.value ? Number(e.target.value) : null,
                  })
                }
                className="rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs dark:border-zinc-800 dark:bg-zinc-900"
              />
            </div>
          )}

          {currentResize.mode === "exact" && (
            <div className="space-y-2">
              <div className="grid grid-cols-2 gap-2">
                <input
                  type="number"
                  min={16}
                  disabled={disabled}
                  value={currentResize.width}
                  onChange={(e) =>
                    setResize({
                      ...currentResize,
                      width: Math.max(16, Number(e.target.value) || 16),
                    })
                  }
                  className="rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs dark:border-zinc-800 dark:bg-zinc-900"
                />
                <input
                  type="number"
                  min={16}
                  disabled={disabled}
                  value={currentResize.height}
                  onChange={(e) =>
                    setResize({
                      ...currentResize,
                      height: Math.max(16, Number(e.target.value) || 16),
                    })
                  }
                  className="rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs dark:border-zinc-800 dark:bg-zinc-900"
                />
              </div>
              <label className="flex items-center gap-2 text-xs text-zinc-600 dark:text-zinc-400 cursor-pointer">
                <input
                  type="checkbox"
                  checked={currentResize.keep_aspect_ratio}
                  disabled={disabled}
                  onChange={(e) =>
                    setResize({
                      ...currentResize,
                      keep_aspect_ratio: e.target.checked,
                    })
                  }
                  className="h-3.5 w-3.5 accent-indigo-600"
                />
                Lock aspect ratio
              </label>
            </div>
          )}

          {currentResize.mode === "percentage" && (
            <div>
              <div className="mb-1 flex justify-between text-xs">
                <span className="text-zinc-500">Scale</span>
                <span className="font-semibold text-indigo-500">
                  {currentResize.percent}%
                </span>
              </div>
              <input
                type="range"
                min={10}
                max={200}
                step={5}
                disabled={disabled}
                value={currentResize.percent}
                onChange={(e) =>
                  setResize({
                    mode: "percentage",
                    percent: Number(e.target.value),
                  })
                }
                className="w-full accent-indigo-600"
              />
            </div>
          )}

          {currentResize.mode !== "original" && (
            <label className="flex items-center gap-2 text-xs text-zinc-600 dark:text-zinc-400 cursor-pointer">
              <input
                type="checkbox"
                checked={currentNeverUpscale}
                disabled={disabled}
                onChange={(e) => setNeverUpscale(e.target.checked)}
                className="h-3.5 w-3.5 accent-indigo-600"
              />
              Don't upscale smaller files
            </label>
          )}
        </SectionAccordion>
      )}

      {/* Shared Accordion: Output, Privacy & Automation */}
      <SectionAccordion
        title="Output & Automation"
        icon={<Folder className="h-3.5 w-3.5" />}
        open={openSections.output}
        onToggle={() => toggleSection("output")}
        activeCount={outputCount}
      >
        <div className="flex items-center gap-1.5">
          <button
            type="button"
            disabled={disabled}
            onClick={onPickOutputDir}
            className="flex min-w-0 flex-1 items-center gap-2 rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-left text-xs text-zinc-700 transition hover:border-indigo-400 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 cursor-pointer"
          >
            <Folder className="h-3.5 w-3.5 shrink-0 text-indigo-500" />
            <span className="truncate">
              {settings.output.outputDir ?? "Same folder as source"}
            </span>
          </button>
          {settings.output.outputDir && (
            <button
              type="button"
              disabled={disabled}
              onClick={() => updateOutput({ outputDir: null })}
              className="rounded-lg border border-zinc-200 p-1.5 text-zinc-500 hover:bg-zinc-100 dark:border-zinc-800 cursor-pointer"
            >
              <RotateCcw className="h-3.5 w-3.5" />
            </button>
          )}
        </div>

        <label className="flex items-center justify-between text-xs text-zinc-700 dark:text-zinc-300 cursor-pointer">
          <span className="flex items-center gap-1.5">
            <ShieldCheck className="h-3.5 w-3.5 text-emerald-500" />
            Strip EXIF &amp; Metadata
          </span>
          <input
            type="checkbox"
            checked={
              activeTab === "image" ? img.stripMetadata : vid.stripMetadata
            }
            disabled={disabled}
            onChange={(e) => {
              if (activeTab === "image")
                updateImage({ stripMetadata: e.target.checked });
              else updateVideo({ stripMetadata: e.target.checked });
            }}
            className="h-3.5 w-3.5 accent-indigo-600"
          />
        </label>

        <label className="flex items-center justify-between text-xs text-zinc-700 dark:text-zinc-300 cursor-pointer">
          <span>Bundle outputs into .zip</span>
          <input
            type="checkbox"
            checked={settings.output.bundleZip}
            disabled={disabled}
            onChange={(e) => updateOutput({ bundleZip: e.target.checked })}
            className="h-3.5 w-3.5 accent-indigo-600"
          />
        </label>

        <label className="flex items-center justify-between text-xs text-zinc-700 dark:text-zinc-300 cursor-pointer">
          <span className="flex items-center gap-1.5">
            <Trash2 className="h-3.5 w-3.5 text-amber-500" />
            Move originals to Trash
          </span>
          <input
            type="checkbox"
            checked={settings.output.trashOriginals}
            disabled={disabled}
            onChange={(e) => updateOutput({ trashOriginals: e.target.checked })}
            className="h-3.5 w-3.5 accent-indigo-600"
          />
        </label>

        <input
          type="text"
          placeholder="Rename template: {name}_{width}x{height}"
          disabled={disabled}
          value={settings.output.renameTemplate}
          onChange={(e) => updateOutput({ renameTemplate: e.target.value })}
          className="w-full rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs dark:border-zinc-800 dark:bg-zinc-900"
        />

        <input
          type="text"
          placeholder="Filename suffix (e.g. _min)"
          disabled={disabled}
          value={settings.output.suffix}
          onChange={(e) => updateOutput({ suffix: e.target.value })}
          className="w-full rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs dark:border-zinc-800 dark:bg-zinc-900"
        />

        {settings.output.outputDir && (
          <label className="flex items-center gap-2 text-xs text-zinc-600 dark:text-zinc-400 cursor-pointer">
            <input
              type="checkbox"
              checked={settings.output.preserveFolderStructure}
              disabled={disabled}
              onChange={(e) =>
                updateOutput({ preserveFolderStructure: e.target.checked })
              }
              className="h-3.5 w-3.5 accent-indigo-600"
            />
            Preserve subfolder structure
          </label>
        )}

        <div>
          <div className="mb-1 flex justify-between text-[11px] text-zinc-500">
            <span>CPU Threads</span>
            <span>
              {settings.maxCpuThreads
                ? `${settings.maxCpuThreads} cores`
                : `All (${hardware?.cpuCores ?? "Auto"})`}
            </span>
          </div>
          <input
            type="range"
            min={1}
            max={hardware?.cpuCores ?? 16}
            disabled={disabled}
            value={settings.maxCpuThreads ?? hardware?.cpuCores ?? 8}
            onChange={(e) => {
              const val = Number(e.target.value);
              onChange({
                ...settings,
                maxCpuThreads:
                  hardware && val >= hardware.cpuCores ? null : val,
              });
            }}
            className="w-full accent-indigo-600"
          />
        </div>

        <label className="flex items-center gap-2 text-xs text-zinc-600 dark:text-zinc-400 cursor-pointer">
          <input
            type="checkbox"
            checked={minimizeToTray}
            onChange={(e) => onToggleMinimizeToTray(e.target.checked)}
            className="h-3.5 w-3.5 accent-indigo-600"
          />
          Keep running in Menu Bar / Tray
        </label>
      </SectionAccordion>
    </div>
  );
}
