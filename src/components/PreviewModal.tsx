import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Check, Crop, Loader2, MoveHorizontal, RotateCcw, X } from "lucide-react";
import {
  ImagePreview,
  ImageSettings,
  NormalizedCrop,
  QueueItem,
  formatBytes,
} from "../types";

interface PreviewModalProps {
  item: QueueItem;
  settings: ImageSettings;
  onApplyCustomCrop: (crop: NormalizedCrop | null) => void;
  onClose: () => void;
}

export function PreviewModal({
  item,
  settings,
  onApplyCustomCrop,
  onClose,
}: PreviewModalProps) {
  const [preview, setPreview] = useState<ImagePreview | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [splitPercent, setSplitPercent] = useState(50);
  const [cropMode, setCropMode] = useState(false);
  const [crop, setCrop] = useState<NormalizedCrop>(
    settings.customCrop ?? { x: 0.1, y: 0.1, width: 0.8, height: 0.8 }
  );

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);

    invoke<ImagePreview>("preview_image", {
      path: item.path,
      settings,
    })
      .then((res) => {
        if (!cancelled) {
          setPreview(res);
          setLoading(false);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(String(err));
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [item.path, settings]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const savedPercent =
    preview && preview.originalBytes > 0
      ? Math.round((1 - preview.estimatedBytes / preview.originalBytes) * 100)
      : null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-6 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="flex max-h-[88vh] w-full max-w-4xl flex-col overflow-hidden rounded-2xl border border-zinc-200 bg-white shadow-2xl dark:border-zinc-800 dark:bg-zinc-950"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between border-b border-zinc-200 px-5 py-3.5 dark:border-zinc-800">
          <div className="min-w-0">
            <h3 className="truncate text-sm font-semibold text-zinc-900 dark:text-zinc-100">
              Before / After Preview — {item.fileName}
            </h3>
            {preview && (
              <div className="mt-0.5 flex flex-wrap items-center gap-2 text-xs text-zinc-500 dark:text-zinc-400">
                <span>
                  {item.sourceFormat} · {preview.originalWidth}×{preview.originalHeight} ·{" "}
                  {formatBytes(preview.originalBytes)}
                </span>
                <ArrowRight className="h-3 w-3" />
                <span className="font-semibold text-indigo-600 dark:text-indigo-400">
                  {settings.format.toUpperCase()} · {preview.outputWidth}×{preview.outputHeight} ·{" "}
                  {formatBytes(preview.estimatedBytes)}
                </span>
                {savedPercent !== null && (
                  <span
                    className={`rounded-full px-2 py-0.5 text-[11px] font-semibold ${
                      savedPercent >= 0
                        ? "bg-emerald-500/15 text-emerald-600 dark:text-emerald-400"
                        : "bg-amber-500/15 text-amber-600 dark:text-amber-400"
                    }`}
                  >
                    {savedPercent >= 0
                      ? `-${savedPercent}% smaller`
                      : `+${Math.abs(savedPercent)}% larger`}
                  </span>
                )}
              </div>
            )}
          </div>
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={() => setCropMode((v) => !v)}
              className={`inline-flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs font-semibold transition cursor-pointer ${
                cropMode
                  ? "bg-indigo-600 text-white"
                  : "border border-zinc-200 bg-white text-zinc-700 hover:bg-zinc-100 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-200"
              }`}
            >
              <Crop className="h-3.5 w-3.5" />
              {cropMode ? "Back to Compare" : "Visual Crop"}
            </button>
            <button
              type="button"
              onClick={onClose}
              className="rounded-lg p-1.5 text-zinc-400 transition hover:bg-zinc-100 hover:text-zinc-800 dark:hover:bg-zinc-800 dark:hover:text-zinc-100 cursor-pointer"
            >
              <X className="h-4.5 w-4.5" />
            </button>
          </div>
        </div>

        {/* Comparison / Visual Crop Canvas */}
        <div className="relative flex min-h-[380px] flex-1 items-center justify-center overflow-hidden bg-zinc-100 p-4 dark:bg-zinc-900">
          {loading ? (
            <div className="flex flex-col items-center gap-2 text-sm text-zinc-500">
              <Loader2 className="h-7 w-7 animate-spin text-indigo-500" />
              <span>Encoding live preview...</span>
            </div>
          ) : error ? (
            <p className="max-w-md text-center text-sm text-red-500">{error}</p>
          ) : preview && cropMode ? (
            <div className="relative inline-block max-h-[60vh] max-w-full select-none overflow-hidden rounded-xl border border-zinc-300 shadow-lg dark:border-zinc-700">
              <img
                src={preview.originalDataUrl}
                alt="Crop target"
                className="block max-h-[60vh] max-w-full object-contain opacity-45"
                draggable={false}
              />
              <div
                className="pointer-events-none absolute border-2 border-indigo-400 shadow-[0_0_0_9999px_rgba(0,0,0,0.45)]"
                style={{
                  left: `${crop.x * 100}%`,
                  top: `${crop.y * 100}%`,
                  width: `${crop.width * 100}%`,
                  height: `${crop.height * 100}%`,
                }}
              />
            </div>
          ) : (
            preview && (
              <div className="relative inline-block max-h-[64vh] max-w-full select-none overflow-hidden rounded-xl border border-zinc-300 shadow-lg dark:border-zinc-700">
                {/* Base layer: Converted (After) */}
                <img
                  src={preview.convertedDataUrl}
                  alt="Converted preview"
                  className="block max-h-[64vh] max-w-full object-contain"
                  draggable={false}
                />

                {/* Overlay layer: Original (Before) clipped to splitPercent */}
                <div
                  className="absolute inset-0 overflow-hidden"
                  style={{ clipPath: `inset(0 ${100 - splitPercent}% 0 0)` }}
                >
                  <img
                    src={preview.originalDataUrl}
                    alt="Original preview"
                    className="h-full w-full object-contain"
                    draggable={false}
                  />
                </div>

                {/* Split Divider Line */}
                <div
                  className="pointer-events-none absolute inset-y-0 w-0.5 bg-white shadow-[0_0_8px_rgba(0,0,0,0.6)]"
                  style={{ left: `${splitPercent}%` }}
                >
                  <div className="absolute top-1/2 -translate-x-1/2 -translate-y-1/2 flex h-8 w-8 items-center justify-center rounded-full bg-white text-zinc-900 shadow-md">
                    <MoveHorizontal className="h-4 w-4" />
                  </div>
                </div>

                {/* Corner Labels */}
                <span className="pointer-events-none absolute left-3 top-3 rounded-md bg-black/60 px-2 py-0.5 text-[11px] font-semibold text-white backdrop-blur-sm">
                  Original ({formatBytes(preview.originalBytes)})
                </span>
                <span className="pointer-events-none absolute right-3 top-3 rounded-md bg-indigo-600/85 px-2 py-0.5 text-[11px] font-semibold text-white backdrop-blur-sm">
                  {settings.format.toUpperCase()} ({formatBytes(preview.estimatedBytes)})
                </span>

                {/* Invisible Range Input for smooth drag */}
                <input
                  type="range"
                  min={0}
                  max={100}
                  value={splitPercent}
                  onChange={(e) => setSplitPercent(Number(e.target.value))}
                  aria-label="Before and after comparison slider"
                  className="absolute inset-0 h-full w-full cursor-ew-resize opacity-0"
                />
              </div>
            )
          )}
        </div>

        {/* Footer / Visual Crop Controls */}
        {cropMode ? (
          <div className="space-y-2.5 border-t border-zinc-200 bg-zinc-50 px-5 py-3 dark:border-zinc-800 dark:bg-zinc-900/80">
            <div className="grid grid-cols-2 gap-3 text-[11px] text-zinc-500 sm:grid-cols-4">
              <label className="flex flex-col gap-1">
                <span>Left ({Math.round(crop.x * 100)}%)</span>
                <input
                  type="range"
                  min={0}
                  max={85}
                  value={Math.round(crop.x * 100)}
                  onChange={(e) => {
                    const x = Number(e.target.value) / 100;
                    setCrop((c) => ({ ...c, x, width: Math.min(c.width, 1 - x) }));
                  }}
                  className="accent-indigo-600"
                />
              </label>
              <label className="flex flex-col gap-1">
                <span>Top ({Math.round(crop.y * 100)}%)</span>
                <input
                  type="range"
                  min={0}
                  max={85}
                  value={Math.round(crop.y * 100)}
                  onChange={(e) => {
                    const y = Number(e.target.value) / 100;
                    setCrop((c) => ({ ...c, y, height: Math.min(c.height, 1 - y) }));
                  }}
                  className="accent-indigo-600"
                />
              </label>
              <label className="flex flex-col gap-1">
                <span>Width ({Math.round(crop.width * 100)}%)</span>
                <input
                  type="range"
                  min={10}
                  max={Math.round((1 - crop.x) * 100)}
                  value={Math.round(crop.width * 100)}
                  onChange={(e) =>
                    setCrop((c) => ({ ...c, width: Number(e.target.value) / 100 }))
                  }
                  className="accent-indigo-600"
                />
              </label>
              <label className="flex flex-col gap-1">
                <span>Height ({Math.round(crop.height * 100)}%)</span>
                <input
                  type="range"
                  min={10}
                  max={Math.round((1 - crop.y) * 100)}
                  value={Math.round(crop.height * 100)}
                  onChange={(e) =>
                    setCrop((c) => ({ ...c, height: Number(e.target.value) / 100 }))
                  }
                  className="accent-indigo-600"
                />
              </label>
            </div>
            <div className="flex items-center justify-between pt-1">
              <button
                type="button"
                onClick={() => {
                  onApplyCustomCrop(null);
                  setCropMode(false);
                }}
                className="inline-flex items-center gap-1.5 rounded-lg px-2.5 py-1 text-xs font-medium text-zinc-500 hover:bg-zinc-200/70 dark:hover:bg-zinc-800 cursor-pointer"
              >
                <RotateCcw className="h-3.5 w-3.5" />
                Clear Crop
              </button>
              <button
                type="button"
                onClick={() => {
                  onApplyCustomCrop(crop);
                  setCropMode(false);
                }}
                className="inline-flex items-center gap-1.5 rounded-xl bg-indigo-600 px-3.5 py-1.5 text-xs font-semibold text-white hover:bg-indigo-500 cursor-pointer"
              >
                <Check className="h-3.5 w-3.5" />
                Apply Visual Crop
              </button>
            </div>
          </div>
        ) : (
          <div className="flex items-center justify-between border-t border-zinc-200 px-5 py-2.5 text-xs text-zinc-500 dark:border-zinc-800 dark:text-zinc-400">
            <span>Drag across the image to compare original vs converted quality.</span>
            <span>
              Quality: {settings.lossless ? "Lossless" : `${settings.quality}%`}
            </span>
          </div>
        )}
      </div>
    </div>
  );
}
