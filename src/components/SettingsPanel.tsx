import { useState } from "react";
import {
  ChevronDown,
  ChevronRight,
  Cpu,
  Crop,
  Download,
  FileText,
  Film,
  FlipHorizontal2,
  FlipVertical2,
  Folder,
  Image as ImageIcon,
  Plus,
  RotateCcw,
  RotateCw,
  Scissors,
  ShieldCheck,
  Sliders,
  Sparkles,
  Trash2,
  Upload,
  Volume2,
  VolumeX,
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
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [savingPreset, setSavingPreset] = useState(false);
  const [newPresetName, setNewPresetName] = useState("");

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

  return (
    <div className="flex flex-1 flex-col gap-5 overflow-y-auto pr-1">
      {/* Quick Presets */}
      <div>
        <div className="mb-2 flex items-center justify-between">
          <div className="flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider text-zinc-500 dark:text-zinc-400">
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
          Video & Audio {videoCount > 0 ? `(${videoCount})` : ""}
        </button>
      </div>

      {/* Format Selection */}
      {activeTab === "image" ? (
        <div className="space-y-4">
          {/* PDF → Images Options (shown when PDFs are in queue) */}
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
                    settings.image.format === fmt.value
                      ? "bg-indigo-600 text-white shadow-sm"
                      : "border border-zinc-200 bg-white text-zinc-700 hover:border-indigo-400 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                  }`}
                >
                  {fmt.label}
                </button>
              ))}
            </div>
          </div>

          {/* Quality / Compression */}
          {imgFormatMeta?.supportsQuality && (
            <div>
              <div className="mb-1.5 flex items-center justify-between text-xs">
                <span className="font-medium text-zinc-600 dark:text-zinc-400">
                  Quality / Compression
                </span>
                <span className="font-semibold text-indigo-600 dark:text-indigo-400">
                  {settings.image.lossless ? "Lossless" : `${settings.image.quality}%`}
                </span>
              </div>
              <input
                type="range"
                min={10}
                max={100}
                value={settings.image.quality}
                disabled={disabled || settings.image.lossless}
                onChange={(e) =>
                  updateImage({ quality: Number(e.target.value) })
                }
                className="w-full accent-indigo-600 disabled:opacity-40"
              />
              <div className="mt-1 flex justify-between text-[10px] text-zinc-400">
                <span>Smaller file</span>
                <span>Balanced</span>
                <span>Best quality</span>
              </div>
            </div>
          )}

          {imgFormatMeta?.supportsLossless &&
            settings.image.format !== "png" &&
            settings.image.format !== "bmp" &&
            settings.image.format !== "tiff" && (
              <label className="flex items-center justify-between rounded-xl border border-zinc-200 bg-white px-3 py-2 text-xs font-medium text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 cursor-pointer">
                <span>Lossless compression</span>
                <input
                  type="checkbox"
                  checked={settings.image.lossless}
                  disabled={disabled}
                  onChange={(e) => updateImage({ lossless: e.target.checked })}
                  className="h-4 w-4 accent-indigo-600"
                />
              </label>
            )}

          {/* Target Max File Size (KB) */}
          {imgFormatMeta?.supportsQuality && !settings.image.lossless && (
            <div>
              <label className="mb-1 block text-xs font-medium text-zinc-600 dark:text-zinc-400">
                Target Max Size (KB, optional)
              </label>
              <input
                type="number"
                min={10}
                placeholder="e.g. 250 (auto-tunes quality)"
                disabled={disabled}
                value={settings.image.targetSizeKb ?? ""}
                onChange={(e) =>
                  updateImage({
                    targetSizeKb: e.target.value ? Number(e.target.value) : null,
                  })
                }
                className="w-full rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs text-zinc-900 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
              />
            </div>
          )}

          {/* Image Transforms: Rotate, Flip, Aspect Crop */}
          <div className="space-y-2 border-t border-zinc-200 pt-3 dark:border-zinc-800">
            <div className="flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider text-zinc-500 dark:text-zinc-400">
              <Crop className="h-3.5 w-3.5" />
              Transform & Crop
            </div>
            <div className="flex items-center gap-1.5">
              <button
                type="button"
                disabled={disabled}
                onClick={() =>
                  updateImage({ rotate: (settings.image.rotate + 90) % 360 })
                }
                className={`flex flex-1 items-center justify-center gap-1 rounded-lg border py-1.5 text-[11px] font-semibold transition cursor-pointer ${
                  settings.image.rotate > 0
                    ? "border-indigo-500 bg-indigo-600 text-white"
                    : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                }`}
                title="Rotate 90° clockwise"
              >
                <RotateCw className="h-3.5 w-3.5" />
                {settings.image.rotate}°
              </button>
              <button
                type="button"
                disabled={disabled}
                onClick={() =>
                  updateImage({ flipHorizontal: !settings.image.flipHorizontal })
                }
                className={`flex flex-1 items-center justify-center gap-1 rounded-lg border py-1.5 text-[11px] font-semibold transition cursor-pointer ${
                  settings.image.flipHorizontal
                    ? "border-indigo-500 bg-indigo-600 text-white"
                    : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                }`}
                title="Flip horizontally"
              >
                <FlipHorizontal2 className="h-3.5 w-3.5" />
                Flip H
              </button>
              <button
                type="button"
                disabled={disabled}
                onClick={() =>
                  updateImage({ flipVertical: !settings.image.flipVertical })
                }
                className={`flex flex-1 items-center justify-center gap-1 rounded-lg border py-1.5 text-[11px] font-semibold transition cursor-pointer ${
                  settings.image.flipVertical
                    ? "border-indigo-500 bg-indigo-600 text-white"
                    : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                }`}
                title="Flip vertically"
              >
                <FlipVertical2 className="h-3.5 w-3.5" />
                Flip V
              </button>
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
                  className={`rounded-md py-1 text-[10px] font-semibold transition cursor-pointer ${
                    settings.image.cropAspect === c.id
                      ? "bg-indigo-600 text-white"
                      : "border border-zinc-200 bg-white text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-400"
                  }`}
                >
                  {c.label}
                </button>
              ))}
            </div>
            <div className="grid grid-cols-3 gap-1.5 pt-1">
              <button
                type="button"
                disabled={disabled}
                onClick={() => updateImage({ grayscale: !settings.image.grayscale })}
                className={`rounded-lg border py-1.5 text-[10px] font-semibold transition cursor-pointer ${
                  settings.image.grayscale
                    ? "border-indigo-500 bg-indigo-600 text-white"
                    : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                }`}
              >
                Grayscale
              </button>
              <button
                type="button"
                disabled={disabled}
                onClick={() => updateImage({ sharpen: !settings.image.sharpen })}
                className={`rounded-lg border py-1.5 text-[10px] font-semibold transition cursor-pointer ${
                  settings.image.sharpen
                    ? "border-indigo-500 bg-indigo-600 text-white"
                    : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                }`}
              >
                Sharpen
              </button>
              <button
                type="button"
                disabled={disabled}
                onClick={() => updateImage({ autoLevels: !settings.image.autoLevels })}
                className={`rounded-lg border py-1.5 text-[10px] font-semibold transition cursor-pointer ${
                  settings.image.autoLevels
                    ? "border-indigo-500 bg-indigo-600 text-white"
                    : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                }`}
                title="Automatic 1%-99% luminance histogram contrast stretch"
              >
                Auto-Levels
              </button>
            </div>

            {/* Auto-Trim Borders */}
            <div className="space-y-1.5 rounded-xl border border-zinc-200 bg-white px-3 py-2 dark:border-zinc-800 dark:bg-zinc-900">
              <label className="flex items-center justify-between text-xs font-medium text-zinc-700 dark:text-zinc-300 cursor-pointer">
                <span>Auto-Trim Borders</span>
                <input
                  type="checkbox"
                  checked={settings.image.autoTrimBorder}
                  disabled={disabled}
                  onChange={(e) =>
                    updateImage({ autoTrimBorder: e.target.checked })
                  }
                  className="h-4 w-4 accent-indigo-600"
                />
              </label>
              {settings.image.autoTrimBorder && (
                <div className="flex items-center gap-2 text-[10px] text-zinc-500">
                  <span className="w-16">Tolerance</span>
                  <input
                    type="range"
                    min={0}
                    max={64}
                    disabled={disabled}
                    value={settings.image.autoTrimTolerance}
                    onChange={(e) =>
                      updateImage({ autoTrimTolerance: Number(e.target.value) })
                    }
                    className="flex-1 accent-indigo-600"
                  />
                  <span className="w-6 text-right">
                    {settings.image.autoTrimTolerance}
                  </span>
                </div>
              )}
            </div>

            {/* Pad to Aspect Ratio (No-Crop Social Fit) & Rounded Corners */}
            <div className="space-y-2 rounded-xl border border-zinc-200 bg-zinc-50/70 p-2.5 dark:border-zinc-800 dark:bg-zinc-900/50">
              <div className="text-[10px] font-semibold text-zinc-500">
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
                    className={`rounded-md py-1 text-[10px] font-semibold transition cursor-pointer ${
                      settings.image.padAspect === c.id
                        ? "bg-indigo-600 text-white"
                        : "border border-zinc-200 bg-white text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-400"
                    }`}
                  >
                    {c.label}
                  </button>
                ))}
              </div>
              <div className="flex items-center gap-2 pt-1 text-[10px] text-zinc-500">
                <span className="w-20">Corners</span>
                <input
                  type="range"
                  min={0}
                  max={50}
                  disabled={disabled}
                  value={settings.image.cornerRadiusPercent}
                  onChange={(e) =>
                    updateImage({ cornerRadiusPercent: Number(e.target.value) })
                  }
                  className="flex-1 accent-indigo-600"
                />
                <span className="w-8 text-right">
                  {settings.image.cornerRadiusPercent}%
                </span>
              </div>

              {/* Screenshot Frame & Soft Drop Shadow */}
              <div className="flex items-center gap-2 pt-1 text-[10px] text-zinc-500">
                <span className="w-20">Frame Pad</span>
                <input
                  type="range"
                  min={0}
                  max={80}
                  step={4}
                  disabled={disabled}
                  value={settings.image.framePaddingPx}
                  onChange={(e) =>
                    updateImage({ framePaddingPx: Number(e.target.value) })
                  }
                  className="flex-1 accent-indigo-600"
                />
                <span className="w-8 text-right">
                  {settings.image.framePaddingPx}px
                </span>
              </div>

              {settings.image.framePaddingPx > 0 && (
                <div className="space-y-1.5 pt-1">
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
                          settings.image.frameGradient === g.id
                            ? "ring-2 ring-indigo-500 ring-offset-1 dark:ring-offset-zinc-900"
                            : "opacity-75 hover:opacity-100"
                        }`}
                      />
                    ))}
                  </div>
                  <label className="flex items-center gap-2 text-[11px] text-zinc-600 dark:text-zinc-400 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={settings.image.dropShadow}
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
            </div>

            {/* Color Adjustments: Brightness, Contrast, Saturation */}
            <div className="space-y-2 rounded-xl border border-zinc-200 bg-zinc-50/70 p-2.5 dark:border-zinc-800 dark:bg-zinc-900/50">
              <div className="text-[10px] font-semibold text-zinc-500">
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
                      settings.image.filterPreset === fp.id
                        ? "bg-indigo-600 text-white"
                        : "border border-zinc-200 bg-white text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-400"
                    }`}
                  >
                    {fp.label}
                  </button>
                ))}
              </div>
            </div>

            <div className="space-y-2 rounded-xl border border-zinc-200 bg-zinc-50/70 p-2.5 dark:border-zinc-800 dark:bg-zinc-900/50">
              <div className="flex items-center justify-between text-[11px] font-semibold text-zinc-500">
                <span className="flex items-center gap-1">
                  <Sliders className="h-3 w-3" />
                  Color Adjustments
                </span>
                {(settings.image.brightness !== 0 ||
                  settings.image.contrast !== 0 ||
                  settings.image.saturation !== 0) && (
                  <button
                    type="button"
                    onClick={() =>
                      updateImage({ brightness: 0, contrast: 0, saturation: 0 })
                    }
                    className="text-[10px] text-indigo-500 hover:underline cursor-pointer"
                  >
                    Reset
                  </button>
                )}
              </div>
              <div className="space-y-1.5">
                <div className="flex items-center gap-2 text-[10px] text-zinc-500">
                  <span className="w-16">Brightness</span>
                  <input
                    type="range"
                    min={-50}
                    max={50}
                    disabled={disabled}
                    value={settings.image.brightness}
                    onChange={(e) =>
                      updateImage({ brightness: Number(e.target.value) })
                    }
                    className="flex-1 accent-indigo-600"
                  />
                  <span className="w-7 text-right">{settings.image.brightness}</span>
                </div>
                <div className="flex items-center gap-2 text-[10px] text-zinc-500">
                  <span className="w-16">Contrast</span>
                  <input
                    type="range"
                    min={-50}
                    max={50}
                    disabled={disabled}
                    value={settings.image.contrast}
                    onChange={(e) =>
                      updateImage({ contrast: Number(e.target.value) })
                    }
                    className="flex-1 accent-indigo-600"
                  />
                  <span className="w-7 text-right">{settings.image.contrast}</span>
                </div>
                <div className="flex items-center gap-2 text-[10px] text-zinc-500">
                  <span className="w-16">Saturation</span>
                  <input
                    type="range"
                    min={-100}
                    max={100}
                    disabled={disabled}
                    value={settings.image.saturation}
                    onChange={(e) =>
                      updateImage({ saturation: Number(e.target.value) })
                    }
                    className="flex-1 accent-indigo-600"
                  />
                  <span className="w-7 text-right">{settings.image.saturation}</span>
                </div>
                <div className="flex items-center gap-2 text-[10px] text-zinc-500">
                  <span className="w-16">Blur</span>
                  <input
                    type="range"
                    min={0}
                    max={20}
                    step={0.5}
                    disabled={disabled}
                    value={settings.image.blurSigma}
                    onChange={(e) =>
                      updateImage({ blurSigma: Number(e.target.value) })
                    }
                    className="flex-1 accent-indigo-600"
                  />
                  <span className="w-7 text-right">{settings.image.blurSigma}</span>
                </div>
                <div className="flex items-center gap-2 text-[10px] text-zinc-500">
                  <span className="w-16">Pixelate</span>
                  <input
                    type="range"
                    min={0}
                    max={32}
                    step={2}
                    disabled={disabled}
                    value={settings.image.pixelateBlock}
                    onChange={(e) =>
                      updateImage({ pixelateBlock: Number(e.target.value) })
                    }
                    className="flex-1 accent-indigo-600"
                  />
                  <span className="w-7 text-right">
                    {settings.image.pixelateBlock}
                  </span>
                </div>
                <div className="flex items-center gap-2 text-[10px] text-zinc-500">
                  <span className="w-16">Vignette</span>
                  <input
                    type="range"
                    min={0}
                    max={100}
                    disabled={disabled}
                    value={Math.round(settings.image.vignetteStrength * 100)}
                    onChange={(e) =>
                      updateImage({
                        vignetteStrength: Number(e.target.value) / 100,
                      })
                    }
                    className="flex-1 accent-indigo-600"
                  />
                  <span className="w-7 text-right">
                    {Math.round(settings.image.vignetteStrength * 100)}%
                  </span>
                </div>
              </div>
            </div>

            {/* Background Fill for JPG/BMP */}
            {(settings.image.format === "jpg" ||
              settings.image.format === "bmp") && (
              <div className="flex items-center justify-between rounded-xl border border-zinc-200 bg-white px-3 py-2 text-xs dark:border-zinc-800 dark:bg-zinc-900">
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
                        settings.image.backgroundHex === hex
                          ? "ring-2 ring-indigo-500"
                          : "border-zinc-300 dark:border-zinc-700"
                      }`}
                      style={{ backgroundColor: hex }}
                      title={hex}
                    />
                  ))}
                  <input
                    type="color"
                    disabled={disabled}
                    value={settings.image.backgroundHex ?? "#ffffff"}
                    onChange={(e) =>
                      updateImage({ backgroundHex: e.target.value })
                    }
                    className="h-5 w-6 cursor-pointer rounded border-0 bg-transparent p-0"
                  />
                </div>
              </div>
            )}
          </div>

          {/* Batch Logo & Text Watermark */}
          <div className="space-y-2 border-t border-zinc-200 pt-3 dark:border-zinc-800">
            <div className="flex items-center justify-between">
              <span className="text-xs font-semibold uppercase tracking-wider text-zinc-500 dark:text-zinc-400">
                Watermark
              </span>
              {settings.image.watermark && (
                <button
                  type="button"
                  disabled={disabled}
                  onClick={() => updateImage({ watermark: null })}
                  className="text-[11px] font-medium text-red-500 hover:underline cursor-pointer"
                >
                  Remove
                </button>
              )}
            </div>
            <button
              type="button"
              disabled={disabled}
              onClick={onPickWatermark}
              className="flex w-full items-center gap-2 rounded-xl border border-zinc-200 bg-white px-3 py-2 text-left text-xs text-zinc-700 transition hover:border-indigo-400 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 cursor-pointer"
            >
              <ImageIcon className="h-3.5 w-3.5 shrink-0 text-indigo-500" />
              <span className="truncate">
                {settings.image.watermark
                  ? settings.image.watermark.imagePath.split(/[\\/]/).pop()
                  : "Add PNG / SVG logo watermark..."}
              </span>
            </button>
            {settings.image.watermark && (
              <div className="space-y-2 rounded-xl border border-zinc-200 bg-zinc-50/70 p-2.5 dark:border-zinc-800 dark:bg-zinc-900/60">
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
                          watermark: {
                            ...settings.image.watermark!,
                            position: pos.id,
                          },
                        })
                      }
                      className={`rounded py-1 text-[10px] font-semibold cursor-pointer ${
                        settings.image.watermark?.position === pos.id
                          ? "bg-indigo-600 text-white"
                          : "bg-white text-zinc-600 dark:bg-zinc-800 dark:text-zinc-400"
                      }`}
                    >
                      {pos.label}
                    </button>
                  ))}
                </div>
                <div className="grid grid-cols-2 gap-2">
                  <div>
                    <div className="flex justify-between text-[10px] text-zinc-500">
                      <span>Opacity</span>
                      <span>
                        {Math.round(settings.image.watermark.opacity * 100)}%
                      </span>
                    </div>
                    <input
                      type="range"
                      min={10}
                      max={100}
                      value={Math.round(settings.image.watermark.opacity * 100)}
                      onChange={(e) =>
                        updateImage({
                          watermark: {
                            ...settings.image.watermark!,
                            opacity: Number(e.target.value) / 100,
                          },
                        })
                      }
                      className="w-full accent-indigo-600"
                    />
                  </div>
                  <div>
                    <div className="flex justify-between text-[10px] text-zinc-500">
                      <span>Scale</span>
                      <span>{settings.image.watermark.scalePercent}%</span>
                    </div>
                    <input
                      type="range"
                      min={5}
                      max={50}
                      value={settings.image.watermark.scalePercent}
                      onChange={(e) =>
                        updateImage({
                          watermark: {
                            ...settings.image.watermark!,
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

            {/* Text Watermark Input */}
            <input
              type="text"
              placeholder="Or add text watermark (e.g. © 2026 Wandel)..."
              disabled={disabled}
              value={settings.image.textWatermark?.text ?? ""}
              onChange={(e) => {
                const val = e.target.value;
                if (!val) {
                  updateImage({ textWatermark: null });
                } else {
                  updateImage({
                    textWatermark: {
                      text: val,
                      fontSizePercent:
                        settings.image.textWatermark?.fontSizePercent ?? 4,
                      color: settings.image.textWatermark?.color ?? "#ffffff",
                      opacity: settings.image.textWatermark?.opacity ?? 0.75,
                      marginPx: settings.image.textWatermark?.marginPx ?? 20,
                      position:
                        settings.image.textWatermark?.position ??
                        "bottom_right",
                      tiled: settings.image.textWatermark?.tiled ?? false,
                    },
                  });
                }
              }}
              className="w-full rounded-xl border border-zinc-200 bg-white px-3 py-2 text-xs text-zinc-900 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
            />
            {settings.image.textWatermark && (
              <label className="flex items-center gap-2 px-1 text-[11px] text-zinc-600 dark:text-zinc-400 cursor-pointer">
                <input
                  type="checkbox"
                  checked={Boolean(settings.image.textWatermark.tiled)}
                  disabled={disabled}
                  onChange={(e) =>
                    updateImage({
                      textWatermark: {
                        ...settings.image.textWatermark!,
                        tiled: e.target.checked,
                      },
                    })
                  }
                  className="h-3.5 w-3.5 accent-indigo-600"
                />
                Tile watermark diagonally across entire image
              </label>
            )}
          </div>
        </div>
      ) : (
        <div className="space-y-4">
          {/* Video / Audio Format */}
          <div>
            <label className="mb-1.5 block text-xs font-medium text-zinc-600 dark:text-zinc-400">
              Video Format
            </label>
            <div className="grid grid-cols-2 gap-1.5">
              {VIDEO_OUTPUT_FORMATS.map((fmt) => {
                const selected =
                  settings.video.target.kind === "video" &&
                  settings.video.target.format === fmt.value;
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
                    className={`rounded-lg px-2 py-1.5 text-left text-[11px] font-semibold transition cursor-pointer disabled:opacity-50 ${
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

            <label className="mb-1.5 mt-3 block text-xs font-medium text-zinc-600 dark:text-zinc-400">
              Extract / Convert Audio
            </label>
            <div className="grid grid-cols-5 gap-1">
              {AUDIO_OUTPUT_FORMATS.map((fmt) => {
                const selected =
                  settings.video.target.kind === "audio" &&
                  settings.video.target.format === fmt.value;
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
                    className={`rounded-lg py-1.5 text-[11px] font-semibold transition cursor-pointer disabled:opacity-50 ${
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

          {/* Video Quality */}
          {settings.video.target.kind === "video" &&
            settings.video.target.format !== "gif" && (
              <div>
                <div className="mb-1.5 flex items-center justify-between text-xs">
                  <span className="font-medium text-zinc-600 dark:text-zinc-400">
                    Video Quality
                  </span>
                  <span className="font-semibold text-indigo-600 dark:text-indigo-400">
                    {settings.video.quality}%
                  </span>
                </div>
                <input
                  type="range"
                  min={20}
                  max={100}
                  value={settings.video.quality}
                  disabled={disabled}
                  onChange={(e) =>
                    updateVideo({ quality: Number(e.target.value) })
                  }
                  className="w-full accent-indigo-600"
                />
              </div>
            )}

          {/* GIF FPS */}
          {settings.video.target.kind === "video" &&
            settings.video.target.format === "gif" && (
              <div>
                <div className="mb-1.5 flex items-center justify-between text-xs">
                  <span className="font-medium text-zinc-600 dark:text-zinc-400">
                    GIF Frame Rate
                  </span>
                  <span className="font-semibold text-indigo-600 dark:text-indigo-400">
                    {settings.video.gifFps} FPS
                  </span>
                </div>
                <input
                  type="range"
                  min={8}
                  max={30}
                  value={settings.video.gifFps}
                  disabled={disabled}
                  onChange={(e) =>
                    updateVideo({ gifFps: Number(e.target.value) })
                  }
                  className="w-full accent-indigo-600"
                />
              </div>
            )}

          {/* Audio Bitrate & Suite Controls */}
          {settings.video.target.kind === "audio" && (
            <div className="space-y-3">
              {(settings.video.target.format === "mp3" ||
                settings.video.target.format === "aac" ||
                settings.video.target.format === "opus") && (
                <div>
                  <label className="mb-1.5 block text-xs font-medium text-zinc-600 dark:text-zinc-400">
                    Audio Bitrate
                  </label>
                  <div className="grid grid-cols-4 gap-1.5">
                    {[128, 192, 256, 320].map((br) => (
                      <button
                        key={br}
                        type="button"
                        disabled={disabled}
                        onClick={() => updateVideo({ audioBitrateKbps: br })}
                        className={`rounded-lg py-1.5 text-[11px] font-semibold transition cursor-pointer ${
                          settings.video.audioBitrateKbps === br
                            ? "bg-indigo-600 text-white"
                            : "border border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                        }`}
                      >
                        {br}k
                      </button>
                    ))}
                  </div>
                </div>
              )}

              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="mb-1 block text-[11px] text-zinc-500">
                    Sample Rate
                  </label>
                  <select
                    disabled={disabled}
                    value={settings.video.sampleRateHz ?? ""}
                    onChange={(e) =>
                      updateVideo({
                        sampleRateHz: e.target.value
                          ? Number(e.target.value)
                          : null,
                      })
                    }
                    className="w-full rounded-lg border border-zinc-200 bg-white px-2 py-1.5 text-xs text-zinc-900 outline-none dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
                  >
                    <option value="">Original</option>
                    <option value={44100}>44.1 kHz</option>
                    <option value={48000}>48 kHz</option>
                  </select>
                </div>
                <div className="flex items-end">
                  <label className="flex w-full items-center justify-between rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 cursor-pointer">
                    <span>Mono</span>
                    <input
                      type="checkbox"
                      checked={settings.video.mono}
                      disabled={disabled}
                      onChange={(e) => updateVideo({ mono: e.target.checked })}
                      className="h-3.5 w-3.5 accent-indigo-600"
                    />
                  </label>
                </div>
              </div>

              <label className="flex items-center justify-between rounded-xl border border-zinc-200 bg-white px-3 py-2 text-xs font-medium text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 cursor-pointer">
                <span className="flex items-center gap-1.5">
                  <Volume2 className="h-3.5 w-3.5 text-indigo-500" />
                  Normalize Loudness (EBU R128)
                </span>
                <input
                  type="checkbox"
                  checked={settings.video.normalizeLoudness}
                  disabled={disabled}
                  onChange={(e) =>
                    updateVideo({ normalizeLoudness: e.target.checked })
                  }
                  className="h-4 w-4 accent-indigo-600"
                />
              </label>

              <label className="flex items-center justify-between rounded-xl border border-zinc-200 bg-white px-3 py-2 text-xs font-medium text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 cursor-pointer">
                <span>Trim Leading &amp; Trailing Silence</span>
                <input
                  type="checkbox"
                  checked={settings.video.trimSilence}
                  disabled={disabled}
                  onChange={(e) =>
                    updateVideo({ trimSilence: e.target.checked })
                  }
                  className="h-4 w-4 accent-indigo-600"
                />
              </label>

              <label className="flex items-center justify-between rounded-xl border border-zinc-200 bg-white px-3 py-2 text-xs font-medium text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 cursor-pointer">
                <span>Voice Denoise (FFT Noise Reduction)</span>
                <input
                  type="checkbox"
                  checked={settings.video.denoiseAudio}
                  disabled={disabled}
                  onChange={(e) =>
                    updateVideo({ denoiseAudio: e.target.checked })
                  }
                  className="h-4 w-4 accent-indigo-600"
                />
              </label>

              {/* Audio Volume Gain (dB) & Speed */}
              <div className="space-y-2 rounded-xl border border-zinc-200 bg-zinc-50/70 p-2.5 dark:border-zinc-800 dark:bg-zinc-900/50">
                <div className="flex items-center gap-2 text-[10px] text-zinc-500">
                  <span className="w-16">Gain (dB)</span>
                  <input
                    type="range"
                    min={-12}
                    max={12}
                    step={1}
                    disabled={disabled}
                    value={settings.video.audioGainDb}
                    onChange={(e) =>
                      updateVideo({ audioGainDb: Number(e.target.value) })
                    }
                    className="flex-1 accent-indigo-600"
                  />
                  <span className="w-10 text-right">
                    {settings.video.audioGainDb > 0
                      ? `+${settings.video.audioGainDb}`
                      : settings.video.audioGainDb}
                    dB
                  </span>
                </div>
                <div className="grid grid-cols-5 gap-1">
                  {[0.75, 1, 1.25, 1.5, 2].map((sp) => (
                    <button
                      key={sp}
                      type="button"
                      disabled={disabled}
                      onClick={() => updateVideo({ speed: sp })}
                      className={`rounded-md py-1 text-[10px] font-semibold cursor-pointer ${
                        settings.video.speed === sp
                          ? "bg-indigo-600 text-white"
                          : "border border-zinc-200 bg-white text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-400"
                      }`}
                    >
                      {sp}×
                    </button>
                  ))}
                </div>
              </div>

              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="mb-1 block text-[11px] text-zinc-500">
                    Fade In (s)
                  </label>
                  <input
                    type="number"
                    min={0}
                    step={0.5}
                    placeholder="0"
                    disabled={disabled}
                    value={settings.video.fadeInSecs ?? ""}
                    onChange={(e) =>
                      updateVideo({
                        fadeInSecs: e.target.value
                          ? Number(e.target.value)
                          : null,
                      })
                    }
                    className="w-full rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs text-zinc-900 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
                  />
                </div>
                <div>
                  <label className="mb-1 block text-[11px] text-zinc-500">
                    Fade Out (s)
                  </label>
                  <input
                    type="number"
                    min={0}
                    step={0.5}
                    placeholder="0"
                    disabled={disabled}
                    value={settings.video.fadeOutSecs ?? ""}
                    onChange={(e) =>
                      updateVideo({
                        fadeOutSecs: e.target.value
                          ? Number(e.target.value)
                          : null,
                      })
                    }
                    className="w-full rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs text-zinc-900 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
                  />
                </div>
              </div>
            </div>
          )}

          {/* Hardware GPU Acceleration */}
          {settings.video.target.kind === "video" && (
            <>
              <label className="flex items-center justify-between rounded-xl border border-zinc-200 bg-white px-3 py-2 text-xs dark:border-zinc-800 dark:bg-zinc-900 cursor-pointer">
                <div>
                  <div className="font-semibold text-zinc-800 dark:text-zinc-200">
                    Fast Remux (Lossless Copy)
                  </div>
                  <div className="text-[10px] text-zinc-500">
                    Instant container swap without re-encoding
                  </div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.video.remuxOnly}
                  disabled={disabled}
                  onChange={(e) =>
                    updateVideo({ remuxOnly: e.target.checked })
                  }
                  className="h-4 w-4 accent-indigo-600"
                />
              </label>

              <label className="flex items-center justify-between rounded-xl border border-zinc-200 bg-white px-3 py-2.5 text-xs dark:border-zinc-800 dark:bg-zinc-900 cursor-pointer">
                <div className="flex items-center gap-2">
                  <Zap className="h-4 w-4 text-amber-500" />
                  <div>
                    <div className="font-semibold text-zinc-800 dark:text-zinc-200">
                      GPU Acceleration
                    </div>
                    <div className="text-[10px] text-zinc-500">
                      {hardware?.gpuBackendName ?? "CPU fallback if unavailable"}
                    </div>
                  </div>
                </div>
                <input
                  type="checkbox"
                  checked={settings.video.useHardwareAccel}
                  disabled={disabled || settings.video.remuxOnly}
                  onChange={(e) =>
                    updateVideo({ useHardwareAccel: e.target.checked })
                  }
                  className="h-4 w-4 accent-indigo-600"
                />
              </label>
            </>
          )}

          {/* Video Target Size, Trim, FPS, Mute */}
          {settings.video.target.kind === "video" && (
            <div className="space-y-2.5 border-t border-zinc-200 pt-3 dark:border-zinc-800">
              <div className="flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider text-zinc-500 dark:text-zinc-400">
                <Scissors className="h-3.5 w-3.5" />
                Trim, FPS & Size Limit
              </div>

              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="mb-1 block text-[11px] text-zinc-500">
                    Trim Start (s)
                  </label>
                  <input
                    type="number"
                    min={0}
                    step={0.5}
                    placeholder="0"
                    disabled={disabled}
                    value={settings.video.trimStartSecs ?? ""}
                    onChange={(e) =>
                      updateVideo({
                        trimStartSecs: e.target.value
                          ? Number(e.target.value)
                          : null,
                      })
                    }
                    className="w-full rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs text-zinc-900 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
                  />
                </div>
                <div>
                  <label className="mb-1 block text-[11px] text-zinc-500">
                    Trim End (s)
                  </label>
                  <input
                    type="number"
                    min={0}
                    step={0.5}
                    placeholder="Full"
                    disabled={disabled}
                    value={settings.video.trimEndSecs ?? ""}
                    onChange={(e) =>
                      updateVideo({
                        trimEndSecs: e.target.value
                          ? Number(e.target.value)
                          : null,
                      })
                    }
                    className="w-full rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs text-zinc-900 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="mb-1 block text-[11px] text-zinc-500">
                    Max Size (MB)
                  </label>
                  <input
                    type="number"
                    min={0.5}
                    step={0.5}
                    placeholder="e.g. 10"
                    disabled={disabled}
                    value={settings.video.targetSizeMb ?? ""}
                    onChange={(e) =>
                      updateVideo({
                        targetSizeMb: e.target.value
                          ? Number(e.target.value)
                          : null,
                      })
                    }
                    className="w-full rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs text-zinc-900 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
                  />
                </div>
                <div>
                  <label className="mb-1 block text-[11px] text-zinc-500">
                    Max FPS
                  </label>
                  <input
                    type="number"
                    min={5}
                    max={120}
                    placeholder="Original"
                    disabled={disabled}
                    value={settings.video.maxFps ?? ""}
                    onChange={(e) =>
                      updateVideo({
                        maxFps: e.target.value ? Number(e.target.value) : null,
                      })
                    }
                    className="w-full rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs text-zinc-900 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
                  />
                </div>
              </div>

              {settings.video.target.format !== "gif" &&
                settings.video.target.format !== "animated_webp" && (
                  <>
                    {/* Social Blur-Pad Aspect Reframe */}
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
                              settings.video.aspectReframe === rf.id
                                ? "bg-indigo-600 text-white"
                                : "border border-zinc-200 bg-white text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-400"
                            }`}
                          >
                            {rf.label}
                          </button>
                        ))}
                      </div>
                    </div>

                    {/* Speed & Playback Mode */}
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
                            settings.video.playbackMode === pm.id
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
                            settings.video.speed === sp
                              ? "bg-indigo-600 text-white"
                              : "border border-zinc-200 bg-white text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-400"
                          }`}
                        >
                          {sp}×
                        </button>
                      ))}
                    </div>

                    <div className="flex items-center gap-1.5">
                      <button
                        type="button"
                        disabled={disabled}
                        onClick={() =>
                          updateVideo({
                            rotate: (settings.video.rotate + 90) % 360,
                          })
                        }
                        className={`flex flex-1 items-center justify-center gap-1 rounded-lg border py-1.5 text-[11px] font-semibold cursor-pointer ${
                          settings.video.rotate > 0
                            ? "border-indigo-500 bg-indigo-600 text-white"
                            : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                        }`}
                      >
                        <RotateCw className="h-3.5 w-3.5" />
                        Rotate {settings.video.rotate}°
                      </button>
                      <button
                        type="button"
                        disabled={disabled}
                        onClick={() =>
                          updateVideo({
                            flipHorizontal: !settings.video.flipHorizontal,
                          })
                        }
                        className={`flex flex-1 items-center justify-center gap-1 rounded-lg border py-1.5 text-[11px] font-semibold cursor-pointer ${
                          settings.video.flipHorizontal
                            ? "border-indigo-500 bg-indigo-600 text-white"
                            : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                        }`}
                      >
                        <FlipHorizontal2 className="h-3.5 w-3.5" />
                        Flip H
                      </button>
                    </div>

                    <div className="grid grid-cols-3 gap-1.5">
                      <button
                        type="button"
                        disabled={disabled}
                        onClick={() =>
                          updateVideo({ deinterlace: !settings.video.deinterlace })
                        }
                        className={`rounded-lg border py-1.5 text-[10px] font-semibold cursor-pointer ${
                          settings.video.deinterlace
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
                          updateVideo({ videoSharpen: !settings.video.videoSharpen })
                        }
                        className={`rounded-lg border py-1.5 text-[10px] font-semibold cursor-pointer ${
                          settings.video.videoSharpen
                            ? "border-indigo-500 bg-indigo-600 text-white"
                            : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                        }`}
                      >
                        Sharpen
                      </button>
                      <button
                        type="button"
                        disabled={disabled}
                        onClick={() =>
                          updateVideo({ denoiseAudio: !settings.video.denoiseAudio })
                        }
                        className={`rounded-lg border py-1.5 text-[10px] font-semibold cursor-pointer ${
                          settings.video.denoiseAudio
                            ? "border-indigo-500 bg-indigo-600 text-white"
                            : "border-zinc-200 bg-white text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300"
                        }`}
                      >
                        Denoise Mic
                      </button>
                    </div>

                    <label className="flex items-center justify-between rounded-xl border border-zinc-200 bg-white px-3 py-2 text-xs font-medium text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 cursor-pointer">
                      <span className="flex items-center gap-1.5">
                        <VolumeX className="h-3.5 w-3.5 text-zinc-400" />
                        Mute / Remove audio track
                      </span>
                      <input
                        type="checkbox"
                        checked={settings.video.muteAudio}
                        disabled={disabled}
                        onChange={(e) =>
                          updateVideo({ muteAudio: e.target.checked })
                        }
                        className="h-4 w-4 accent-indigo-600"
                      />
                    </label>

                    {/* Subtitle Burn-In & Replace Audio Track */}
                    <div className="space-y-1.5">
                      <div className="flex items-center gap-1.5">
                        <button
                          type="button"
                          disabled={disabled}
                          onClick={onPickSubtitle}
                          className="flex min-w-0 flex-1 items-center gap-2 rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-left text-[11px] text-zinc-700 transition hover:border-indigo-400 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 cursor-pointer"
                        >
                          <span className="truncate">
                            {settings.video.subtitlePath
                              ? `Sub: ${settings.video.subtitlePath.split(/[\\/]/).pop()}`
                              : "Burn-in subtitles (.srt / .vtt)..."}
                          </span>
                        </button>
                        {settings.video.subtitlePath && (
                          <button
                            type="button"
                            onClick={() => updateVideo({ subtitlePath: null })}
                            className="rounded-lg p-1.5 text-zinc-400 hover:text-red-500 cursor-pointer"
                          >
                            <X className="h-3.5 w-3.5" />
                          </button>
                        )}
                      </div>

                      <div className="flex items-center gap-1.5">
                        <button
                          type="button"
                          disabled={disabled}
                          onClick={onPickReplaceAudio}
                          className="flex min-w-0 flex-1 items-center gap-2 rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-left text-[11px] text-zinc-700 transition hover:border-indigo-400 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 cursor-pointer"
                        >
                          <span className="truncate">
                            {settings.video.replaceAudioPath
                              ? `Audio: ${settings.video.replaceAudioPath.split(/[\\/]/).pop()}`
                              : "Replace / add audio track..."}
                          </span>
                        </button>
                        {settings.video.replaceAudioPath && (
                          <button
                            type="button"
                            onClick={() =>
                              updateVideo({ replaceAudioPath: null })
                            }
                            className="rounded-lg p-1.5 text-zinc-400 hover:text-red-500 cursor-pointer"
                          >
                            <X className="h-3.5 w-3.5" />
                          </button>
                        )}
                      </div>

                      <div className="flex items-center gap-1.5">
                        <button
                          type="button"
                          disabled={disabled}
                          onClick={onPickVideoWatermark}
                          className="flex min-w-0 flex-1 items-center gap-2 rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-left text-[11px] text-zinc-700 transition hover:border-indigo-400 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 cursor-pointer"
                        >
                          <span className="truncate">
                            {settings.video.watermark
                              ? `Logo: ${settings.video.watermark.imagePath.split(/[\\/]/).pop()}`
                              : "Add video logo watermark..."}
                          </span>
                        </button>
                        {settings.video.watermark && (
                          <button
                            type="button"
                            onClick={() => updateVideo({ watermark: null })}
                            className="rounded-lg p-1.5 text-zinc-400 hover:text-red-500 cursor-pointer"
                          >
                            <X className="h-3.5 w-3.5" />
                          </button>
                        )}
                      </div>
                    </div>
                  </>
                )}
            </div>
          )}
        </div>
      )}

      {/* Resize Section (hidden for Audio extraction) */}
      {!(
        activeTab === "video" && settings.video.target.kind === "audio"
      ) && (
        <div className="space-y-2.5 border-t border-zinc-200 pt-4 dark:border-zinc-800">
          <div className="text-xs font-semibold uppercase tracking-wider text-zinc-500 dark:text-zinc-400">
            Resize
          </div>

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
                className={`rounded-lg py-1 text-[11px] font-semibold transition cursor-pointer ${
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
              <div>
                <label className="mb-1 block text-[11px] text-zinc-500">
                  Max Width (px)
                </label>
                <input
                  type="number"
                  min={16}
                  placeholder="Auto"
                  disabled={disabled}
                  value={currentResize.width ?? ""}
                  onChange={(e) =>
                    setResize({
                      ...currentResize,
                      width: e.target.value ? Number(e.target.value) : null,
                    })
                  }
                  className="w-full rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs text-zinc-900 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
                />
              </div>
              <div>
                <label className="mb-1 block text-[11px] text-zinc-500">
                  Max Height (px)
                </label>
                <input
                  type="number"
                  min={16}
                  placeholder="Auto"
                  disabled={disabled}
                  value={currentResize.height ?? ""}
                  onChange={(e) =>
                    setResize({
                      ...currentResize,
                      height: e.target.value ? Number(e.target.value) : null,
                    })
                  }
                  className="w-full rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs text-zinc-900 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
                />
              </div>
            </div>
          )}

          {currentResize.mode === "exact" && (
            <div className="space-y-2">
              <div className="grid grid-cols-2 gap-2">
                <div>
                  <label className="mb-1 block text-[11px] text-zinc-500">
                    Width (px)
                  </label>
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
                    className="w-full rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs text-zinc-900 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
                  />
                </div>
                <div>
                  <label className="mb-1 block text-[11px] text-zinc-500">
                    Height (px)
                  </label>
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
                    className="w-full rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs text-zinc-900 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
                  />
                </div>
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
        </div>
      )}

      {/* Privacy / Metadata Stripping */}
      <label className="flex items-center justify-between rounded-xl border border-zinc-200 bg-white px-3 py-2.5 text-xs dark:border-zinc-800 dark:bg-zinc-900 cursor-pointer">
        <div className="flex items-center gap-2">
          <ShieldCheck className="h-4 w-4 text-emerald-500" />
          <div>
            <div className="font-semibold text-zinc-800 dark:text-zinc-200">
              Strip EXIF & Metadata
            </div>
            <div className="text-[10px] text-zinc-500">
              Removes GPS, camera & container tags
            </div>
          </div>
        </div>
        <input
          type="checkbox"
          checked={
            activeTab === "image"
              ? settings.image.stripMetadata
              : settings.video.stripMetadata
          }
          disabled={disabled}
          onChange={(e) => {
            if (activeTab === "image") {
              updateImage({ stripMetadata: e.target.checked });
            } else {
              updateVideo({ stripMetadata: e.target.checked });
            }
          }}
          className="h-4 w-4 accent-indigo-600"
        />
      </label>

      {/* Output Folder */}
      <div className="space-y-2 border-t border-zinc-200 pt-4 dark:border-zinc-800">
        <div className="text-xs font-semibold uppercase tracking-wider text-zinc-500 dark:text-zinc-400">
          Output Folder
        </div>
        <div className="flex items-center gap-1.5">
          <button
            type="button"
            disabled={disabled}
            onClick={onPickOutputDir}
            className="flex min-w-0 flex-1 items-center gap-2 rounded-xl border border-zinc-200 bg-white px-3 py-2 text-left text-xs text-zinc-700 transition hover:border-indigo-400 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 cursor-pointer disabled:opacity-50"
            title={settings.output.outputDir ?? "Same folder as source"}
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
              className="rounded-xl border border-zinc-200 p-2 text-zinc-500 transition hover:bg-zinc-100 dark:border-zinc-800 dark:hover:bg-zinc-800 cursor-pointer"
              title="Reset to same folder as source"
            >
              <RotateCcw className="h-3.5 w-3.5" />
            </button>
          )}
        </div>

        <label className="flex items-center justify-between rounded-xl border border-zinc-200 bg-white px-3 py-2 text-xs text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 cursor-pointer">
          <span className="flex items-center gap-1.5 font-medium">
            <Trash2 className="h-3.5 w-3.5 text-amber-500" />
            Move originals to Trash after success
          </span>
          <input
            type="checkbox"
            checked={settings.output.trashOriginals}
            disabled={disabled}
            onChange={(e) => updateOutput({ trashOriginals: e.target.checked })}
            className="h-4 w-4 accent-indigo-600"
          />
        </label>

        <label className="flex items-center justify-between rounded-xl border border-zinc-200 bg-white px-3 py-2 text-xs text-zinc-700 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 cursor-pointer">
          <span className="font-medium">Bundle outputs into single .zip</span>
          <input
            type="checkbox"
            checked={settings.output.bundleZip}
            disabled={disabled}
            onChange={(e) => updateOutput({ bundleZip: e.target.checked })}
            className="h-4 w-4 accent-indigo-600"
          />
        </label>
      </div>

      {/* Advanced Performance Tuning */}
      <div className="border-t border-zinc-200 pt-3 dark:border-zinc-800">
        <button
          type="button"
          onClick={() => setShowAdvanced((v) => !v)}
          className="flex w-full items-center justify-between text-xs font-medium text-zinc-500 hover:text-zinc-800 dark:text-zinc-400 dark:hover:text-zinc-200 cursor-pointer"
        >
          <span className="flex items-center gap-1.5">
            <Cpu className="h-3.5 w-3.5" />
            Performance & Naming
          </span>
          {showAdvanced ? (
            <ChevronDown className="h-3.5 w-3.5" />
          ) : (
            <ChevronRight className="h-3.5 w-3.5" />
          )}
        </button>

        {showAdvanced && (
          <div className="mt-3 space-y-3">
            <div>
              <label className="mb-1 block text-[11px] text-zinc-500">
                Rename Template (optional)
              </label>
              <input
                type="text"
                placeholder="{name}_{width}x{height} or {index:03}_{name}"
                disabled={disabled}
                value={settings.output.renameTemplate}
                onChange={(e) =>
                  updateOutput({ renameTemplate: e.target.value })
                }
                className="w-full rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs text-zinc-900 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
              />
            </div>
            <div>
              <label className="mb-1 block text-[11px] text-zinc-500">
                Filename Suffix (optional)
              </label>
              <input
                type="text"
                placeholder="e.g. _min"
                disabled={disabled}
                value={settings.output.suffix}
                onChange={(e) => updateOutput({ suffix: e.target.value })}
                className="w-full rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs text-zinc-900 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
              />
            </div>
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
                Preserve dropped subfolder structure
              </label>
            )}
            <div>
              <div className="mb-1 flex justify-between text-[11px] text-zinc-500">
                <span>CPU Threads</span>
                <span>
                  {settings.maxCpuThreads
                    ? `${settings.maxCpuThreads} cores`
                    : `All (${hardware?.cpuCores ?? "Auto"} cores)`}
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
              Keep running in Menu Bar / Tray on close
            </label>
          </div>
        )}
      </div>
    </div>
  );
}
