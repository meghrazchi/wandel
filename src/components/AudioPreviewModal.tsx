import { useEffect, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Check, Music, Pause, Play, RotateCcw, Scissors, X } from "lucide-react";
import { QueueItem, formatDuration } from "../types";

interface AudioPreviewModalProps {
  item: QueueItem;
  trimStartSecs: number | null;
  trimEndSecs: number | null;
  onApplyTrim: (start: number | null, end: number | null) => void;
  onClose: () => void;
}

export function AudioPreviewModal({
  item,
  trimStartSecs,
  trimEndSecs,
  onApplyTrim,
  onClose,
}: AudioPreviewModalProps) {
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const [duration, setDuration] = useState<number>(item.durationSecs ?? 0);
  const [currentTime, setCurrentTime] = useState<number>(trimStartSecs ?? 0);
  const [isPlaying, setIsPlaying] = useState(false);
  const [start, setStart] = useState<number>(trimStartSecs ?? 0);
  const [end, setEnd] = useState<number>(trimEndSecs ?? item.durationSecs ?? 0);

  const srcUrl = convertFileSrc(item.path);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const handleLoadedMetadata = () => {
    const a = audioRef.current;
    if (!a || !Number.isFinite(a.duration)) return;
    setDuration(a.duration);
    if (!trimEndSecs || trimEndSecs > a.duration) {
      setEnd(a.duration);
    }
    if (start > 0 && start < a.duration) {
      a.currentTime = start;
    }
  };

  const handleTimeUpdate = () => {
    const a = audioRef.current;
    if (!a) return;
    setCurrentTime(a.currentTime);
    if (end > 0 && a.currentTime >= end) {
      a.pause();
      a.currentTime = start;
      setIsPlaying(false);
    }
  };

  const togglePlay = () => {
    const a = audioRef.current;
    if (!a) return;
    if (a.paused) {
      if (a.currentTime < start || a.currentTime >= end) {
        a.currentTime = start;
      }
      void a.play();
      setIsPlaying(true);
    } else {
      a.pause();
      setIsPlaying(false);
    }
  };

  const effectiveEnd = end > 0 ? end : duration;
  const clipLen = Math.max(0, effectiveEnd - start);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/65 p-6 backdrop-blur-xs"
      onClick={onClose}
    >
      <div
        className="w-full max-w-lg overflow-hidden rounded-2xl border border-zinc-200 bg-white shadow-2xl dark:border-zinc-800 dark:bg-zinc-950"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between border-b border-zinc-200 px-5 py-4 dark:border-zinc-800">
          <div className="flex items-center gap-2.5 min-w-0">
            <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-violet-500/15 text-violet-500">
              <Music className="h-4.5 w-4.5" />
            </div>
            <div className="min-w-0">
              <h3 className="truncate text-sm font-semibold text-zinc-900 dark:text-zinc-100">
                {item.fileName}
              </h3>
              <p className="text-xs text-zinc-500">
                Range: {start.toFixed(1)}s → {effectiveEnd.toFixed(1)}s ({clipLen.toFixed(1)}s)
              </p>
            </div>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg p-1.5 text-zinc-400 transition hover:bg-zinc-100 hover:text-zinc-800 dark:hover:bg-zinc-800 dark:hover:text-zinc-100 cursor-pointer"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <audio
          ref={audioRef}
          src={srcUrl}
          onLoadedMetadata={handleLoadedMetadata}
          onTimeUpdate={handleTimeUpdate}
          onEnded={() => setIsPlaying(false)}
        />

        <div className="space-y-4 p-5">
          <div className="flex items-center gap-3">
            <button
              type="button"
              onClick={togglePlay}
              className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-violet-600 text-white shadow-sm transition hover:bg-violet-500 cursor-pointer"
            >
              {isPlaying ? (
                <Pause className="h-4 w-4" />
              ) : (
                <Play className="h-4 w-4 ml-0.5" />
              )}
            </button>
            <div className="flex-1 space-y-2">
              <div className="flex justify-between text-[11px] text-zinc-500">
                <span>Current: {formatDuration(currentTime)}</span>
                <span>Total: {formatDuration(duration)}</span>
              </div>
              <div className="flex items-center gap-2 text-[11px]">
                <span className="w-10 font-semibold text-emerald-500">Start</span>
                <input
                  type="range"
                  min={0}
                  max={Math.max(duration, 0.1)}
                  step={0.1}
                  value={start}
                  onChange={(e) => {
                    const val = Math.min(Number(e.target.value), Math.max(0, effectiveEnd - 0.2));
                    setStart(val);
                    if (audioRef.current) audioRef.current.currentTime = val;
                  }}
                  className="flex-1 accent-emerald-500"
                />
                <span className="w-11 text-right font-mono">{start.toFixed(1)}s</span>
              </div>
              <div className="flex items-center gap-2 text-[11px]">
                <span className="w-10 font-semibold text-amber-500">End</span>
                <input
                  type="range"
                  min={0}
                  max={Math.max(duration, 0.1)}
                  step={0.1}
                  value={effectiveEnd}
                  onChange={(e) => {
                    const val = Math.max(Number(e.target.value), Math.min(duration, start + 0.2));
                    setEnd(val);
                    if (audioRef.current) audioRef.current.currentTime = val;
                  }}
                  className="flex-1 accent-amber-500"
                />
                <span className="w-11 text-right font-mono">{effectiveEnd.toFixed(1)}s</span>
              </div>
            </div>
          </div>

          <div className="flex items-center justify-between border-t border-zinc-200 pt-3 dark:border-zinc-800">
            <button
              type="button"
              onClick={() => {
                setStart(0);
                setEnd(duration);
                onApplyTrim(null, null);
              }}
              className="inline-flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs font-medium text-zinc-500 hover:bg-zinc-100 dark:hover:bg-zinc-800 cursor-pointer"
            >
              <RotateCcw className="h-3.5 w-3.5" />
              Reset
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
              className="inline-flex items-center gap-1.5 rounded-xl bg-violet-600 px-4 py-2 text-xs font-semibold text-white hover:bg-violet-500 cursor-pointer"
            >
              <Scissors className="h-3.5 w-3.5" />
              <Check className="h-3.5 w-3.5" />
              Apply Trim
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
