import { Download, FolderOpen, History, Trash2, X } from "lucide-react";
import { HistoryEntry, LifetimeStats, formatBytes } from "../types";

interface HistoryDrawerProps {
  entries: HistoryEntry[];
  stats: LifetimeStats;
  onClose: () => void;
  onClear: () => void;
  onRevealFile: (path: string) => void;
}

export function HistoryDrawer({
  entries,
  stats,
  onClose,
  onClear,
  onRevealFile,
}: HistoryDrawerProps) {
  const savedBytes = Math.max(0, stats.totalInputBytes - stats.totalOutputBytes);
  const avgReduction =
    stats.totalInputBytes > 0
      ? Math.round((1 - stats.totalOutputBytes / stats.totalInputBytes) * 100)
      : 0;

  const handleExportCsv = () => {
    const header = "timestamp,fileName,inputBytes,outputBytes,savedPercent,source,outputPath";
    const rows = entries.map((e) => {
      const pct =
        e.inputBytes > 0 ? Math.round((1 - e.outputBytes / e.inputBytes) * 100) : 0;
      const esc = (s: string) => `"${s.replace(/"/g, '""')}"`;
      return [
        new Date(e.timestamp).toISOString(),
        esc(e.fileName),
        e.inputBytes,
        e.outputBytes,
        pct,
        e.source,
        esc(e.outputPath),
      ].join(",");
    });
    const csv = [header, ...rows].join("\n");
    const blob = new Blob([csv], { type: "text/csv;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `wandel_history_${new Date().toISOString().slice(0, 10)}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div
      className="fixed inset-0 z-50 flex justify-end bg-black/50 backdrop-blur-xs"
      onClick={onClose}
    >
      <div
        className="flex h-full w-full max-w-md flex-col border-l border-zinc-200 bg-white shadow-2xl dark:border-zinc-800 dark:bg-zinc-950"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between border-b border-zinc-200 px-5 py-4 dark:border-zinc-800">
          <div className="flex items-center gap-2">
            <History className="h-4.5 w-4.5 text-indigo-500" />
            <h3 className="text-sm font-semibold text-zinc-900 dark:text-zinc-100">
              Conversion History
            </h3>
          </div>
          <div className="flex items-center gap-1.5">
            {entries.length > 0 && (
              <>
                <button
                  type="button"
                  onClick={handleExportCsv}
                  className="inline-flex items-center gap-1 rounded-lg px-2 py-1 text-xs font-medium text-indigo-600 transition hover:bg-indigo-500/10 dark:text-indigo-400 cursor-pointer"
                  title="Export conversion history as CSV"
                >
                  <Download className="h-3.5 w-3.5" />
                  CSV
                </button>
                <button
                  type="button"
                  onClick={onClear}
                  className="inline-flex items-center gap-1 rounded-lg px-2 py-1 text-xs font-medium text-zinc-500 transition hover:bg-red-500/10 hover:text-red-500 cursor-pointer"
                >
                  <Trash2 className="h-3.5 w-3.5" />
                  Clear
                </button>
              </>
            )}
            <button
              type="button"
              onClick={onClose}
              className="rounded-lg p-1.5 text-zinc-400 transition hover:bg-zinc-100 hover:text-zinc-800 dark:hover:bg-zinc-800 dark:hover:text-zinc-100 cursor-pointer"
            >
              <X className="h-4 w-4" />
            </button>
          </div>
        </div>

        {/* Lifetime Stats Banner */}
        <div className="grid grid-cols-3 gap-2 border-b border-zinc-200 bg-zinc-50 p-4 dark:border-zinc-800 dark:bg-zinc-900/60">
          <div className="rounded-xl border border-zinc-200 bg-white p-2.5 text-center dark:border-zinc-800 dark:bg-zinc-900">
            <div className="text-base font-bold text-zinc-900 dark:text-zinc-100">
              {stats.totalFiles}
            </div>
            <div className="text-[10px] font-medium text-zinc-500">
              Files Converted
            </div>
          </div>
          <div className="rounded-xl border border-zinc-200 bg-white p-2.5 text-center dark:border-zinc-800 dark:bg-zinc-900">
            <div className="text-base font-bold text-emerald-600 dark:text-emerald-400">
              {formatBytes(savedBytes)}
            </div>
            <div className="text-[10px] font-medium text-zinc-500">
              Space Saved
            </div>
          </div>
          <div className="rounded-xl border border-zinc-200 bg-white p-2.5 text-center dark:border-zinc-800 dark:bg-zinc-900">
            <div className="text-base font-bold text-indigo-600 dark:text-indigo-400">
              {avgReduction > 0 ? `-${avgReduction}%` : "0%"}
            </div>
            <div className="text-[10px] font-medium text-zinc-500">
              Avg Reduction
            </div>
          </div>
        </div>

        {/* Entries List */}
        <div className="flex-1 space-y-2 overflow-y-auto p-4">
          {entries.length === 0 ? (
            <div className="flex h-full flex-col items-center justify-center text-center text-sm text-zinc-400">
              <History className="mb-2 h-8 w-8 opacity-40" />
              <p>No conversions recorded yet.</p>
            </div>
          ) : (
            entries.map((entry) => {
              const savedPct =
                entry.inputBytes > 0
                  ? Math.round((1 - entry.outputBytes / entry.inputBytes) * 100)
                  : 0;
              return (
                <div
                  key={entry.id}
                  className="flex items-center justify-between gap-3 rounded-xl border border-zinc-200 bg-white p-3 dark:border-zinc-800 dark:bg-zinc-900/70"
                >
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-1.5">
                      <p className="truncate text-xs font-semibold text-zinc-900 dark:text-zinc-100">
                        {entry.fileName}
                      </p>
                      {entry.source === "watch" && (
                        <span className="rounded bg-emerald-500/15 px-1.5 py-0.5 text-[9px] font-semibold uppercase text-emerald-600 dark:text-emerald-400">
                          Auto
                        </span>
                      )}
                    </div>
                    <p className="mt-0.5 text-[11px] text-zinc-500 dark:text-zinc-400">
                      {formatBytes(entry.inputBytes)} →{" "}
                      <span className="font-medium text-zinc-700 dark:text-zinc-300">
                        {formatBytes(entry.outputBytes)}
                      </span>{" "}
                      ({savedPct >= 0 ? `-${savedPct}%` : `+${Math.abs(savedPct)}%`})
                    </p>
                  </div>
                  <button
                    type="button"
                    onClick={() => onRevealFile(entry.outputPath)}
                    className="rounded-lg p-1.5 text-zinc-400 transition hover:bg-zinc-100 hover:text-zinc-800 dark:hover:bg-zinc-800 dark:hover:text-zinc-100 cursor-pointer"
                    title="Show in folder"
                  >
                    <FolderOpen className="h-4 w-4" />
                  </button>
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
}
