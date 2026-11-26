import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AlertTriangle, Info, Loader2, ShieldCheck, X } from "lucide-react";
import { DetailedMetadata, QueueItem, formatBytes } from "../types";

interface MetadataModalProps {
  item: QueueItem;
  onClose: () => void;
}

export function MetadataModal({ item, onClose }: MetadataModalProps) {
  const [data, setData] = useState<DetailedMetadata | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    invoke<DetailedMetadata>("inspect_detailed_metadata", { path: item.path })
      .then((res) => {
        if (!cancelled) {
          setData(res);
          setLoading(false);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(String(err));
          setLoading(false);
        }
      });

    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => {
      cancelled = true;
      window.removeEventListener("keydown", onKey);
    };
  }, [item.path, onClose]);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-6 backdrop-blur-xs"
      onClick={onClose}
    >
      <div
        className="flex w-full max-w-lg flex-col overflow-hidden rounded-2xl border border-zinc-200 bg-white shadow-2xl dark:border-zinc-800 dark:bg-zinc-950"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between border-b border-zinc-200 px-5 py-4 dark:border-zinc-800">
          <div className="flex items-center gap-2 min-w-0">
            <Info className="h-4.5 w-4.5 shrink-0 text-indigo-500" />
            <div className="min-w-0">
              <h3 className="truncate text-sm font-semibold text-zinc-900 dark:text-zinc-100">
                {item.fileName}
              </h3>
              <p className="truncate text-[11px] text-zinc-500">{item.path}</p>
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

        <div className="p-5 space-y-3 max-h-[70vh] overflow-y-auto">
          {loading ? (
            <div className="flex items-center justify-center gap-2 py-8 text-sm text-zinc-500">
              <Loader2 className="h-5 w-5 animate-spin text-indigo-500" />
              Inspecting metadata...
            </div>
          ) : error ? (
            <p className="text-sm text-red-500">{error}</p>
          ) : (
            data && (
              <>
                {data.hasGpsWarning ? (
                  <div className="flex items-start gap-2.5 rounded-xl border border-amber-500/30 bg-amber-500/10 p-3 text-xs text-amber-700 dark:text-amber-300">
                    <AlertTriangle className="h-4 w-4 shrink-0 mt-0.5" />
                    <div>
                      <div className="font-semibold">
                        GPS / Location Metadata Detected
                      </div>
                      <p className="mt-0.5 opacity-90">
                        Keep &ldquo;Strip EXIF &amp; Metadata&rdquo; enabled to remove location data before sharing.
                      </p>
                    </div>
                  </div>
                ) : (
                  <div className="flex items-center gap-2 rounded-xl border border-emerald-500/20 bg-emerald-500/5 px-3 py-2 text-xs text-emerald-700 dark:text-emerald-400">
                    <ShieldCheck className="h-4 w-4 shrink-0" />
                    <span>No GPS location tags detected.</span>
                  </div>
                )}

                <div className="divide-y divide-zinc-200 rounded-xl border border-zinc-200 dark:divide-zinc-800 dark:border-zinc-800">
                  <div className="flex items-center justify-between px-3.5 py-2 text-xs">
                    <span className="text-zinc-500">File Size</span>
                    <span className="font-semibold text-zinc-900 dark:text-zinc-100">
                      {formatBytes(data.sizeBytes)}
                    </span>
                  </div>
                  {data.entries.map((entry, idx) => (
                    <div
                      key={`${entry.label}-${idx}`}
                      className="flex items-center justify-between px-3.5 py-2 text-xs"
                    >
                      <span className="text-zinc-500">{entry.label}</span>
                      <span className="font-semibold text-zinc-900 dark:text-zinc-100">
                        {entry.value}
                      </span>
                    </div>
                  ))}
                </div>
              </>
            )
          )}
        </div>
      </div>
    </div>
  );
}
