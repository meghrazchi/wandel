import { useEffect, useRef, useState } from "react";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { Camera, Check, Pause, Play, RotateCcw, Scissors, X } from "lucide-react";
import { QueueItem, formatDuration } from "../types";

interface VideoPreviewModalProps {
  item: QueueItem;
  trimStartSecs: number | null;
  trimEndSecs: number | null;
  onApplyTrim: (start: number | null, end: number | null) => void;
  onClose: () => void;
}

export function VideoPreviewModal({
  item,
  trimStartSecs,
  trimEndSecs,
  onApplyTrim,
  onClose,
}: VideoPreviewModalProps) {
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const [duration, setDuration] = useState<number>(item.durationSecs ?? 0);
  const [currentTime, setCurrentTime] = useState<number>(trimStartSecs ?? 0);
  const [isPlaying, setIsPlaying] = useState(false);
  const [start, setStart] = useState<number>(trimStartSecs ?? 0);
  const [end, setEnd] = useState<number>(
    trimEndSecs ?? item.durationSecs ?? 0
  );
  const [snapshotStatus, setSnapshotStatus] = useState<string | null>(null);

  const srcUrl = convertFileSrc(item.path);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const handleLoadedMetadata = () => {
    const v = videoRef.current;
    if (!v || !Number.isFinite(v.duration)) return;
    setDuration(v.duration);
    if (!trimEndSecs || trimEndSecs > v.duration) {
      setEnd(v.duration);
    }
    if (start > 0 && start < v.duration) {
      v.currentTime = start;
    }
  };

  const handleTimeUpdate = () => {
    const v = videoRef.current;
    if (!v) return;
    setCurrentTime(v.currentTime);
    if (end > 0 && v.currentTime >= end) {
      v.pause();
      v.currentTime = start;
      setIsPlaying(false);
    }
  };

  const togglePlay = () => {
    const v = videoRef.current;
    if (!v) return;
    if (v.paused) {
      if (v.currentTime < start || v.currentTime >= end) {
        v.currentTime = start;
      }
      void v.play();
      setIsPlaying(true);
    } else {
      v.pause();
      setIsPlaying(false);
    }
  };

  const handleCaptureFrame = async () => {
    const v = videoRef.current;
    const ts = v ? v.currentTime : currentTime;
    const defaultName = item.fileName.replace(/\.[^.]+$/, `_frame_${Math.round(ts)}s.png`);
    const dest = await save({
      title: "Save Video Frame Snapshot",
      defaultPath: defaultName,
      filters: [{ name: "PNG Image", extensions: ["png"] }],
    });
    if (!dest) return;
    try {
      await invoke("capture_video_frame", {
        path: item.path,
        timestampSecs: ts,
        outputPath: dest,
      });
      setSnapshotStatus("Frame saved!");
      setTimeout(() => setSnapshotStatus(null), 2500);
    } catch (err) {
      setSnapshotStatus(`Error: ${String(err)}`);
    }
  };

  const effectiveEnd = end > 0 ? end : duration;
  const clipLen = Math.max(0, effectiveEnd - start);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 p-6 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="flex max-h-[90vh] w-full max-w-4xl flex-col overflow-hidden rounded-2xl border border-zinc-200 bg-white shadow-2xl dark:border-zinc-800 dark:bg-zinc-950"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between border-b border-zinc-200 px-5 py-3.5 dark:border-zinc-800">
          <div className="min-w-0">
            <h3 className="truncate text-sm font-semibold text-zinc-900 dark:text-zinc-100">
              Video Preview & Visual Trim — {item.fileName}
            </h3>
            <p className="text-xs text-zinc-500">
              Selected range: {start.toFixed(1)}s → {effectiveEnd.toFixed(1)}s ({clipLen.toFixed(1)}s)
            </p>
          </div>
          <div className="flex items-center gap-2">
            {snapshotStatus && (
              <span className="rounded-full bg-emerald-500/15 px-2.5 py-0.5 text-xs font-medium text-emerald-500">
                {snapshotStatus}
              </span>
            )}
            <button
              type="button"
              onClick={handleCaptureFrame}
              className="inline-flex items-center gap-1.5 rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs font-medium text-zinc-700 transition hover:bg-zinc-100 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-200 dark:hover:bg-zinc-800 cursor-pointer"
            >
              <Camera className="h-3.5 w-3.5 text-indigo-500" />
              Save Frame
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

        {/* Video Player */}
        <div className="flex flex-1 items-center justify-center bg-black p-4 min-h-[300px]">
          <video
            ref={videoRef}
            src={srcUrl}
            onLoadedMetadata={handleLoadedMetadata}
            onTimeUpdate={handleTimeUpdate}
            onEnded={() => setIsPlaying(false)}
            onClick={togglePlay}
            className="max-h-[54vh] max-w-full rounded-lg cursor-pointer"
          />
        </div>

        {/* Interactive Visual Trim Controls */}
        <div className="space-y-3 border-t border-zinc-200 bg-zinc-50 p-4 dark:border-zinc-800 dark:bg-zinc-900/80">
          <div className="flex items-center gap-3">
            <button
              type="button"
              onClick={togglePlay}
              className="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-indigo-600 text-white shadow-sm transition hover:bg-indigo-500 cursor-pointer"
            >
              {isPlaying ? (
                <Pause className="h-4 w-4" />
              ) : (
                <Play className="h-4 w-4 ml-0.5" />
              )}
            </button>

            <div className="flex-1 space-y-1.5">
              <div className="flex justify-between text-[11px] font-medium text-zinc-500 dark:text-zinc-400">
                <span>Current: {formatDuration(currentTime)}</span>
                <span>Total: {formatDuration(duration)}</span>
              </div>

              {/* Start Trim Slider */}
              <div className="flex items-center gap-2 text-[11px]">
                <span className="w-12 font-semibold text-emerald-600 dark:text-emerald-400">
                  Start
                </span>
                <input
                  type="range"
                  min={0}
                  max={Math.max(duration, 0.1)}
                  step={0.1}
                  value={start}
                  onChange={(e) => {
                    const val = Number(e.target.value);
                    const clamped = Math.min(val, Math.max(0, effectiveEnd - 0.2));
                    setStart(clamped);
                    if (videoRef.current) videoRef.current.currentTime = clamped;
                  }}
                  className="flex-1 accent-emerald-500"
                />
                <span className="w-12 text-right font-mono text-zinc-600 dark:text-zinc-300">
                  {start.toFixed(1)}s
                </span>
              </div>

              {/* End Trim Slider */}
              <div className="flex items-center gap-2 text-[11px]">
                <span className="w-12 font-semibold text-amber-600 dark:text-amber-400">
                  End
                </span>
                <input
                  type="range"
                  min={0}
                  max={Math.max(duration, 0.1)}
                  step={0.1}
                  value={effectiveEnd}
                  onChange={(e) => {
                    const val = Number(e.target.value);
                    const clamped = Math.max(val, Math.min(duration, start + 0.2));
                    setEnd(clamped);
                    if (videoRef.current) videoRef.current.currentTime = clamped;
                  }}
                  className="flex-1 accent-amber-500"
                />
                <span className="w-12 text-right font-mono text-zinc-600 dark:text-zinc-300">
                  {effectiveEnd.toFixed(1)}s
                </span>
              </div>
            </div>
          </div>

          <div className="flex items-center justify-between pt-1">
            <button
              type="button"
              onClick={() => {
                setStart(0);
                setEnd(duration);
                onApplyTrim(null, null);
              }}
              className="inline-flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs font-medium text-zinc-500 transition hover:bg-zinc-200/70 hover:text-zinc-800 dark:text-zinc-400 dark:hover:bg-zinc-800 dark:hover:text-zinc-200 cursor-pointer"
            >
              <RotateCcw className="h-3.5 w-3.5" />
              Reset Trim
            </button>

            <button
              type="button"
              onClick={() => {
                const s = start > 0.05 ? Number(start.toFixed(2)) : null;
                const e =
                  duration > 0 && effectiveEnd < duration - 0.05
                    ? Number(effectiveEnd.toFixed(2))
                    : null;
                onApplyTrim(s, e);
                onClose();
              }}
              className="inline-flex items-center gap-1.5 rounded-xl bg-indigo-600 px-4 py-2 text-xs font-semibold text-white shadow-sm transition hover:bg-indigo-500 cursor-pointer"
            >
              <Scissors className="h-3.5 w-3.5" />
              <Check className="h-3.5 w-3.5" />
              Apply Trim Range
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
