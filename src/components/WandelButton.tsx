import { Loader2, Square, Wand2 } from "lucide-react";

interface WandelButtonProps {
  totalFiles: number;
  isConverting: boolean;
  overallProgress: number;
  onWandel: () => void;
  onCancel: () => void;
}

export function WandelButton({
  totalFiles,
  isConverting,
  overallProgress,
  onWandel,
  onCancel,
}: WandelButtonProps) {
  if (isConverting) {
    return (
      <div className="space-y-2 pt-3 border-t border-zinc-200 dark:border-zinc-800">
        <div className="flex items-center justify-between text-xs font-medium text-zinc-600 dark:text-zinc-300">
          <span className="flex items-center gap-1.5">
            <Loader2 className="h-3.5 w-3.5 animate-spin text-indigo-500" />
            Wandeling...
          </span>
          <span>{Math.round(overallProgress)}%</span>
        </div>
        <div className="h-1.5 w-full overflow-hidden rounded-full bg-zinc-200 dark:bg-zinc-800">
          <div
            className="h-full rounded-full bg-gradient-to-r from-indigo-500 via-violet-500 to-fuchsia-500 transition-all duration-200"
            style={{ width: `${Math.max(4, Math.min(100, overallProgress))}%` }}
          />
        </div>
        <button
          type="button"
          onClick={onCancel}
          className="flex w-full items-center justify-center gap-2 rounded-xl border border-red-500/30 bg-red-500/10 py-3 text-sm font-semibold text-red-600 transition hover:bg-red-500/20 dark:text-red-400 cursor-pointer"
        >
          <Square className="h-3.5 w-3.5 fill-current" />
          Cancel
        </button>
      </div>
    );
  }

  return (
    <div className="pt-3 border-t border-zinc-200 dark:border-zinc-800">
      <button
        type="button"
        disabled={totalFiles === 0}
        onClick={onWandel}
        className="group relative flex w-full items-center justify-center gap-2.5 overflow-hidden rounded-xl bg-gradient-to-r from-indigo-600 via-violet-600 to-indigo-600 bg-[length:200%_100%] py-3.5 text-base font-bold tracking-wide text-white shadow-lg shadow-indigo-600/25 transition-all duration-300 hover:bg-right hover:shadow-indigo-600/40 active:scale-[0.99] disabled:cursor-not-allowed disabled:opacity-40 disabled:shadow-none cursor-pointer"
      >
        <Wand2 className="h-4.5 w-4.5 transition-transform duration-300 group-hover:rotate-12" />
        <span>Wandel</span>
        {totalFiles > 0 && (
          <span className="rounded-full bg-white/20 px-2 py-0.5 text-xs font-semibold">
            {totalFiles}
          </span>
        )}
      </button>
    </div>
  );
}
