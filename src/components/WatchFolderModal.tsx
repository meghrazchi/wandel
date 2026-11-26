import { FolderPlus, Radio, Square, X } from "lucide-react";
import { WatchFolderInfo } from "../types";

interface WatchFolderModalProps {
  folders: WatchFolderInfo[];
  onAddFolder: () => void;
  onStopFolder: (folderPath: string) => void;
  onClose: () => void;
}

export function WatchFolderModal({
  folders,
  onAddFolder,
  onStopFolder,
  onClose,
}: WatchFolderModalProps) {
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
          <div className="flex items-center gap-2">
            <Radio className="h-4.5 w-4.5 text-emerald-500" />
            <div>
              <h3 className="text-sm font-semibold text-zinc-900 dark:text-zinc-100">
                Watch Folders (Auto-Wandel)
              </h3>
              <p className="text-[11px] text-zinc-500">
                Automatically converts new files dropped into watched folders using your current settings.
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

        <div className="p-5 space-y-3">
          <button
            type="button"
            onClick={onAddFolder}
            className="flex w-full items-center justify-center gap-2 rounded-xl bg-indigo-600 py-2.5 text-xs font-semibold text-white transition hover:bg-indigo-500 cursor-pointer"
          >
            <FolderPlus className="h-4 w-4" />
            Add Watch Folder (uses current settings)
          </button>

          {folders.length === 0 ? (
            <div className="rounded-xl border border-dashed border-zinc-300 py-8 text-center text-xs text-zinc-500 dark:border-zinc-800">
              No active watch folders. Add a folder like Screenshots or Downloads to auto-convert new files.
            </div>
          ) : (
            <div className="space-y-2 max-h-64 overflow-y-auto">
              {folders.map((wf) => (
                <div
                  key={wf.folderPath}
                  className="flex items-center justify-between gap-3 rounded-xl border border-zinc-200 bg-zinc-50 px-3.5 py-2.5 dark:border-zinc-800 dark:bg-zinc-900"
                >
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span className="h-2 w-2 rounded-full bg-emerald-500 animate-pulse" />
                      <p
                        className="truncate text-xs font-medium text-zinc-900 dark:text-zinc-100"
                        title={wf.folderPath}
                      >
                        {wf.folderPath}
                      </p>
                    </div>
                    <p className="mt-0.5 pl-4 text-[11px] text-zinc-500">
                      {wf.convertedCount} file(s) auto-converted this session
                    </p>
                  </div>
                  <button
                    type="button"
                    onClick={() => onStopFolder(wf.folderPath)}
                    className="inline-flex items-center gap-1 rounded-lg border border-red-500/30 bg-red-500/10 px-2.5 py-1 text-[11px] font-semibold text-red-600 transition hover:bg-red-500/20 dark:text-red-400 cursor-pointer"
                  >
                    <Square className="h-3 w-3 fill-current" />
                    Stop
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
