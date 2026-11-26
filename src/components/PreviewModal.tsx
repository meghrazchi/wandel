import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Check, Crop, EyeOff, Loader2, MoveHorizontal, RotateCcw, Undo2, X } from "lucide-react";
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
  onApplyBlurRegions: (regions: NormalizedCrop[]) => void;
  onClose: () => void;
}

export function PreviewModal({
  item,
  settings,
  onApplyCustomCrop,
  onApplyBlurRegions,
  onClose,
}: PreviewModalProps) {
  const [preview, setPreview] = useState<ImagePreview | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [splitPercent, setSplitPercent] = useState(50);
  const [mode, setMode] = useState<"compare" | "crop" | "redact">("compare");
  const [crop, setCrop] = useState<NormalizedCrop>(
    settings.customCrop ?? { x: 0.1, y: 0.1, width: 0.8, height: 0.8 }
  );
  const [blurRegions, setBlurRegions] = useState<NormalizedCrop[]>(
    settings.blurRegions ?? []
  );
  const [draftBox, setDraftBox] = useState<NormalizedCrop | null>(null);
  const dragStartRef = useRef<{ x: number; y: number } | null>(null);
  const redactCanvasRef = useRef<HTMLDivElement | null>(null);

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
              onClick={() => setMode((m) => (m === "crop" ? "compare" : "crop"))}
              className={`inline-flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs font-semibold transition cursor-pointer ${
                mode === "crop"
                  ? "bg-indigo-600 text-white"
                  : "border border-zinc-200 bg-white text-zinc-700 hover:bg-zinc-100 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-200"
              }`}
            >
              <Crop className="h-3.5 w-3.5" />
              {mode === "crop" ? "Back to Compare" : "Visual Crop"}
            </button>
            <button
              type="button"
              onClick={() => setMode((m) => (m === "redact" ? "compare" : "redact"))}
              className={`inline-flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs font-semibold transition cursor-pointer ${
                mode === "redact"
                  ? "bg-rose-600 text-white"
                  : "border border-zinc-200 bg-white text-zinc-700 hover:bg-zinc-100 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-200"
              }`}
            >
              <EyeOff className="h-3.5 w-3.5" />
              {mode === "redact"
                ? "Back to Compare"
                : `Redact Blur${blurRegions.length > 0 ? ` (${blurRegions.length})` : ""}`}
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

        {/* Comparison / Visual Crop / Redact Blur Canvas */}
        <div className="relative flex min-h-[380px] flex-1 items-center justify-center overflow-hidden bg-zinc-100 p-4 dark:bg-zinc-900">
          {loading ? (
            <div className="flex flex-col items-center gap-2 text-sm text-zinc-500">
              <Loader2 className="h-7 w-7 animate-spin text-indigo-500" />
              <span>Encoding live preview...</span>
            </div>
          ) : error ? (
            <p className="max-w-md text-center text-sm text-red-500">{error}</p>
          ) : preview && mode === "crop" ? (
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
          ) : preview && mode === "redact" ? (
            <div
              ref={redactCanvasRef}
              onMouseDown={(e) => {
                const rect = redactCanvasRef.current?.getBoundingClientRect();
                if (!rect || rect.width === 0 || rect.height === 0) return;
                const nx = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
                const ny = Math.max(0, Math.min(1, (e.clientY - rect.top) / rect.height));
                dragStartRef.current = { x: nx, y: ny };
                setDraftBox({ x: nx, y: ny, width: 0, height: 0 });
              }}
              onMouseMove={(e) => {
                if (!dragStartRef.current) return;
                const rect = redactCanvasRef.current?.getBoundingClientRect();
                if (!rect || rect.width === 0 || rect.height === 0) return;
                const nx = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
                const ny = Math.max(0, Math.min(1, (e.clientY - rect.top) / rect.height));
                const sx = dragStartRef.current.x;
                const sy = dragStartRef.current.y;
                setDraftBox({
                  x: Math.min(sx, nx),
                  y: Math.min(sy, ny),
                  width: Math.abs(nx - sx),
                  height: Math.abs(ny - sy),
                });
              }}
              onMouseUp={() => {
                if (draftBox && draftBox.width > 0.015 && draftBox.height > 0.015) {
                  setBlurRegions((prev) => [...prev, draftBox]);
                }
                dragStartRef.current = null;
                setDraftBox(null);
              }}
              className="relative inline-block max-h-[60vh] max-w-full cursor-crosshair select-none overflow-hidden rounded-xl border border-zinc-300 shadow-lg dark:border-zinc-700"
            >
              <img
                src={preview.originalDataUrl}
                alt="Redact target"
                className="block max-h-[60vh] max-w-full object-contain"
                draggable={false}
              />
              {[...blurRegions, ...(draftBox ? [draftBox] : [])].map((box, i) => (
                <div
                  key={i}
                  className="pointer-events-none absolute border-2 border-rose-500 bg-rose-500/20 backdrop-blur-md"
                  style={{
                    left: `${box.x * 100}%`,
                    top: `${box.y * 100}%`,
                    width: `${box.width * 100}%`,
                    height: `${box.height * 100}%`,
                  }}
                />
              ))}
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

        {/* Footer / Visual Crop & Redact Controls */}
        {mode === "crop" ? (
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
                  setMode("compare");
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
                  setMode("compare");
                }}
                className="inline-flex items-center gap-1.5 rounded-xl bg-indigo-600 px-3.5 py-1.5 text-xs font-semibold text-white hover:bg-indigo-500 cursor-pointer"
              >
                <Check className="h-3.5 w-3.5" />
                Apply Visual Crop
              </button>
            </div>
          </div>
        ) : mode === "redact" ? (
          <div className="flex flex-wrap items-center justify-between gap-2 border-t border-zinc-200 bg-zinc-50 px-5 py-3 text-xs dark:border-zinc-800 dark:bg-zinc-900/80">
            <span className="text-zinc-500 dark:text-zinc-400">
              Click and drag on the image to add privacy blur boxes ({blurRegions.length} active).
            </span>
            <div className="flex items-center gap-2">
              <button
                type="button"
                disabled={blurRegions.length === 0}
                onClick={() => setBlurRegions((prev) => prev.slice(0, -1))}
                className="inline-flex items-center gap-1 rounded-lg px-2.5 py-1 text-xs font-medium text-zinc-600 hover:bg-zinc-200/70 disabled:opacity-35 dark:text-zinc-300 dark:hover:bg-zinc-800 cursor-pointer"
              >
                <Undo2 className="h-3.5 w-3.5" />
                Undo
              </button>
              <button
                type="button"
                disabled={blurRegions.length === 0}
                onClick={() => {
                  setBlurRegions([]);
                  onApplyBlurRegions([]);
                }}
                className="inline-flex items-center gap-1 rounded-lg px-2.5 py-1 text-xs font-medium text-zinc-500 hover:bg-zinc-200/70 disabled:opacity-35 dark:hover:bg-zinc-800 cursor-pointer"
              >
                <RotateCcw className="h-3.5 w-3.5" />
                Clear All
              </button>
              <button
                type="button"
                onClick={() => {
                  onApplyBlurRegions(blurRegions);
                  setMode("compare");
                }}
                className="inline-flex items-center gap-1.5 rounded-xl bg-rose-600 px-3.5 py-1.5 text-xs font-semibold text-white hover:bg-rose-500 cursor-pointer"
              >
                <Check className="h-3.5 w-3.5" />
                Apply Blur Redaction
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
