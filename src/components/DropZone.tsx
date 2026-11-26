import { FolderPlus, ImagePlus, UploadCloud } from "lucide-react";

interface DropZoneProps {
  isDragging: boolean;
  isInspecting: boolean;
  onAddFiles: () => void;
  onAddFolder: () => void;
}

export function DropZone({
  isDragging,
  isInspecting,
  onAddFiles,
  onAddFolder,
}: DropZoneProps) {
  return (
    <div
      className={`flex h-full flex-col items-center justify-center rounded-2xl border-2 border-dashed p-8 text-center transition-all duration-200 ${
        isDragging
          ? "border-indigo-500 bg-indigo-500/10 scale-[0.99]"
          : "border-zinc-300 dark:border-zinc-800 bg-zinc-50/60 dark:bg-zinc-900/40 hover:border-zinc-400 dark:hover:border-zinc-700"
      }`}
    >
      <div
        className={`mb-5 flex h-16 w-16 items-center justify-center rounded-2xl transition-colors ${
          isDragging
            ? "bg-indigo-500 text-white shadow-lg shadow-indigo-500/30"
            : "bg-indigo-500/10 text-indigo-600 dark:text-indigo-400"
        }`}
      >
        <UploadCloud className="h-8 w-8" />
      </div>

      <h2 className="text-lg font-semibold tracking-tight text-zinc-900 dark:text-zinc-100">
        {isInspecting
          ? "Inspecting files..."
          : isDragging
          ? "Drop to add to Wandel"
          : "Drop images, videos, or folders here"}
      </h2>
      <p className="mt-1.5 max-w-sm text-sm text-zinc-500 dark:text-zinc-400">
        Single file or massive batches, processed in parallel across all CPU
        cores and GPU encoders. Press{" "}
        <kbd className="rounded bg-zinc-200 px-1.5 py-0.5 text-[11px] font-semibold text-zinc-700 dark:bg-zinc-800 dark:text-zinc-300">
          ⌘V
        </kbd>{" "}
        to paste from clipboard.
      </p>

      <div className="mt-6 flex flex-wrap items-center justify-center gap-2.5">
        <button
          type="button"
          onClick={onAddFiles}
          disabled={isInspecting}
          className="inline-flex items-center gap-2 rounded-xl bg-zinc-900 px-4 py-2.5 text-sm font-medium text-white shadow-sm transition hover:bg-zinc-800 disabled:opacity-50 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-white cursor-pointer"
        >
          <ImagePlus className="h-4 w-4" />
          Add Files
        </button>
        <button
          type="button"
          onClick={onAddFolder}
          disabled={isInspecting}
          className="inline-flex items-center gap-2 rounded-xl border border-zinc-300 bg-white px-4 py-2.5 text-sm font-medium text-zinc-700 transition hover:bg-zinc-100 disabled:opacity-50 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-200 dark:hover:bg-zinc-800 cursor-pointer"
        >
          <FolderPlus className="h-4 w-4" />
          Add Folder
        </button>
      </div>

      <div className="mt-8 flex flex-wrap items-center justify-center gap-1.5 text-[11px] font-medium text-zinc-500 dark:text-zinc-500">
        {[
          "PNG",
          "JPG",
          "WebP",
          "AVIF",
          "HEIC",
          "JXL",
          "SVG",
          "PDF",
          "MP4",
          "WebM",
          "MOV",
          "MKV",
          "GIF",
          "MP3",
          "OPUS",
          "FLAC",
        ].map((fmt) => (
          <span
            key={fmt}
            className="rounded-md bg-zinc-200/70 px-2 py-0.5 dark:bg-zinc-800/80"
          >
            {fmt}
          </span>
        ))}
      </div>
    </div>
  );
}
