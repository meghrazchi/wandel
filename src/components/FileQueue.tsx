import { useMemo, useState } from "react";
import {
  AlertCircle,
  ArrowDown,
  ArrowRight,
  ArrowUp,
  ArrowUpDown,
  AudioLines,
  CheckCircle2,
  Code2,
  Columns2,
  CopyCheck,
  Download,
  Eye,
  FileText,
  Film,
  FolderOpen,
  FolderPlus,
  Grid2x2,
  Grid3x3,
  Hash,
  Image as ImageIcon,
  ImagePlus,
  Images,
  Info,
  Layers,
  LayoutGrid,
  Loader2,
  Minimize2,
  MonitorSmartphone,
  Music,
  Package,
  Palette,
  RefreshCw,
  RotateCw,
  Scissors,
  Search,
  ShieldCheck,
  Stamp,
  Trash2,
  Video,
  X,
} from "lucide-react";
import {
  AUDIO_OUTPUT_FORMATS,
  AudioFormat,
  ConversionSettings,
  IMAGE_OUTPUT_FORMATS,
  ImageFormat,
  PdfPageFit,
  QueueItem,
  VIDEO_OUTPUT_FORMATS,
  VideoFormat,
  VideoTarget,
  formatBytes,
  formatDuration,
  targetLabelForItem,
} from "../types";

interface FileQueueProps {
  items: QueueItem[];
  settings: ConversionSettings;
  isConverting: boolean;
  onRemoveItem: (id: string) => void;
  onClearAll: () => void;
  onClearCompleted: () => void;
  onAddFiles: () => void;
  onAddFolder: () => void;
  onRevealFile: (path: string) => void;
  onPreviewItem: (item: QueueItem) => void;
  onInspectItem: (item: QueueItem) => void;
  onGenerateIconPack: (item: QueueItem) => void;
  onOverrideImageFormat: (id: string, format: ImageFormat | undefined) => void;
  onOverrideVideoTarget: (id: string, target: VideoTarget | undefined) => void;
  onMoveItem: (index: number, direction: -1 | 1) => void;
  onRetryFailed: () => void;
  onMergePdf: (pageFit: PdfPageFit) => void;
  onCreateGrid: () => void;
  onCompareImages: () => void;
  onCreateSlideshow: () => void;
  onCreateSpriteSheet: () => void;
  onMergeVideos: () => void;
  onMergePdfs: () => void;
  onSplitPdf: (item: QueueItem) => void;
  onCompressPdf: (item: QueueItem) => void;
  onRotatePdf: (item: QueueItem) => void;
  onReorderPdf: (item: QueueItem) => void;
  onStampPdf: (item: QueueItem) => void;
  onNumberPdf: (item: QueueItem) => void;
  onExtractPdfImages: (item: QueueItem) => void;
  onScrubPdf: (item: QueueItem) => void;
  onPaletteItem: (item: QueueItem) => void;
  onDataUrlItem: (item: QueueItem) => void;
  onResponsiveSetItem: (item: QueueItem) => void;
  onSliceImage: (item: QueueItem) => void;
  onStoryboardItem: (item: QueueItem) => void;
  onExtractFrames: (item: QueueItem) => void;
  onAudioWaveform: (item: QueueItem) => void;
  onPodcastVideo: (item: QueueItem) => void;
  onSplitMedia: (item: QueueItem) => void;
  onExportCsv: () => void;
  onRequeueOutputs: () => void;
  onMergeAudio: () => void;
  onFindDuplicates: () => void;
}

export function FileQueue({
  items,
  settings,
  isConverting,
  onRemoveItem,
  onClearAll,
  onClearCompleted,
  onAddFiles,
  onAddFolder,
  onRevealFile,
  onPreviewItem,
  onInspectItem,
  onGenerateIconPack,
  onOverrideImageFormat,
  onOverrideVideoTarget,
  onMoveItem,
  onRetryFailed,
  onMergePdf,
  onCreateGrid,
  onCompareImages,
  onCreateSlideshow,
  onCreateSpriteSheet,
  onMergeVideos,
  onMergePdfs,
  onSplitPdf,
  onCompressPdf,
  onRotatePdf,
  onReorderPdf,
  onStampPdf,
  onNumberPdf,
  onExtractPdfImages,
  onScrubPdf,
  onPaletteItem,
  onDataUrlItem,
  onResponsiveSetItem,
  onSliceImage,
  onStoryboardItem,
  onExtractFrames,
  onAudioWaveform,
  onPodcastVideo,
  onSplitMedia,
  onExportCsv,
  onRequeueOutputs,
  onMergeAudio,
  onFindDuplicates,
}: FileQueueProps) {
  const [filter, setFilter] = useState<"all" | "image" | "video" | "failed">("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [sortBy, setSortBy] = useState<"order" | "name" | "size_desc" | "size_asc">("order");
  const [pdfFit, setPdfFit] = useState<PdfPageFit>("fit_to_image");

  const completedCount = items.filter((i) => i.status === "completed").length;
  const failedCount = items.filter((i) => i.status === "failed").length;
  const imageCount = items.filter(
    (i) => i.mediaKind === "image" || i.mediaKind === "pdf"
  ).length;
  const videoCount = items.filter(
    (i) => i.mediaKind === "video" || i.mediaKind === "audio"
  ).length;

  const q = searchQuery.trim().toLowerCase();
  const visibleItems = useMemo(
    () =>
      items
        .filter((item) => {
          if (filter === "image" && item.mediaKind !== "image" && item.mediaKind !== "pdf")
            return false;
          if (filter === "video" && item.mediaKind !== "video" && item.mediaKind !== "audio")
            return false;
          if (filter === "failed" && item.status !== "failed") return false;
          if (q && !item.fileName.toLowerCase().includes(q)) return false;
          return true;
        })
        .sort((a, b) => {
          if (sortBy === "name") return a.fileName.localeCompare(b.fileName);
          if (sortBy === "size_desc") return b.sizeBytes - a.sizeBytes;
          if (sortBy === "size_asc") return a.sizeBytes - b.sizeBytes;
          return 0;
        }),
    [items, filter, q, sortBy]
  );
  const totalInputBytes = items.reduce((acc, i) => acc + i.sizeBytes, 0);
  const totalOutputBytes = items.reduce(
    (acc, i) => acc + (i.outputBytes ?? 0),
    0
  );
  const completedInputBytes = items
    .filter((i) => i.status === "completed")
    .reduce((acc, i) => acc + i.sizeBytes, 0);

  const savedPercent =
    completedCount > 0 && completedInputBytes > 0
      ? Math.round((1 - totalOutputBytes / completedInputBytes) * 100)
      : null;

  return (
    <div className="flex h-full flex-col">
      {/* Queue Toolbar */}
      <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
        <div className="flex items-center gap-2.5">
          <span className="text-sm font-semibold text-zinc-900 dark:text-zinc-100">
            Queue ({items.length})
          </span>
          <span className="text-xs text-zinc-500 dark:text-zinc-400">
            {formatBytes(totalInputBytes)}
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
                ? `Saved ${savedPercent}%`
                : `+${Math.abs(savedPercent)}% size`}
            </span>
          )}
        </div>

        <div className="flex items-center gap-1.5">
          <button
            type="button"
            onClick={onAddFiles}
            disabled={isConverting}
            className="inline-flex items-center gap-1.5 rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs font-medium text-zinc-700 transition hover:bg-zinc-100 disabled:opacity-40 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 dark:hover:bg-zinc-800 cursor-pointer"
          >
            <ImagePlus className="h-3.5 w-3.5" />
            Files
          </button>
          <button
            type="button"
            onClick={onAddFolder}
            disabled={isConverting}
            className="inline-flex items-center gap-1.5 rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs font-medium text-zinc-700 transition hover:bg-zinc-100 disabled:opacity-40 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 dark:hover:bg-zinc-800 cursor-pointer"
          >
            <FolderPlus className="h-3.5 w-3.5" />
            Folder
          </button>
          {failedCount > 0 && !isConverting && (
            <button
              type="button"
              onClick={onRetryFailed}
              className="inline-flex items-center gap-1 rounded-lg border border-red-500/30 bg-red-500/10 px-2.5 py-1.5 text-xs font-medium text-red-600 transition hover:bg-red-500/20 dark:text-red-400 cursor-pointer"
            >
              <RefreshCw className="h-3.5 w-3.5" />
              Retry Failed ({failedCount})
            </button>
          )}
          {items.filter((i) => i.mediaKind === "image").length >= 2 &&
            !isConverting && (
              <>
                <button
                  type="button"
                  onClick={onFindDuplicates}
                  className="inline-flex items-center gap-1 rounded-lg border border-amber-500/30 bg-amber-500/10 px-2.5 py-1.5 text-xs font-semibold text-amber-600 transition hover:bg-amber-500/20 dark:text-amber-400 cursor-pointer"
                  title="Detect & remove exact or visually similar duplicate images"
                >
                  <CopyCheck className="h-3.5 w-3.5" />
                  Dedupe
                </button>
                <button
                  type="button"
                  onClick={onCreateGrid}
                  className="inline-flex items-center gap-1 rounded-lg border border-violet-500/30 bg-violet-500/10 px-2.5 py-1.5 text-xs font-semibold text-violet-600 transition hover:bg-violet-500/20 dark:text-violet-400 cursor-pointer"
                  title="Stitch queued images into a Contact Sheet / Grid collage"
                >
                  <Grid2x2 className="h-3.5 w-3.5" />
                  Grid
                </button>
                <button
                  type="button"
                  onClick={onCompareImages}
                  className="inline-flex items-center gap-1 rounded-lg border border-cyan-500/30 bg-cyan-500/10 px-2.5 py-1.5 text-xs font-semibold text-cyan-600 transition hover:bg-cyan-500/20 dark:text-cyan-400 cursor-pointer"
                  title="Create a side-by-side Before/After comparison from the first two queued images"
                >
                  <Columns2 className="h-3.5 w-3.5" />
                  Compare
                </button>
                <button
                  type="button"
                  onClick={onCreateSlideshow}
                  className="inline-flex items-center gap-1 rounded-lg border border-pink-500/30 bg-pink-500/10 px-2.5 py-1.5 text-xs font-semibold text-pink-600 transition hover:bg-pink-500/20 dark:text-pink-400 cursor-pointer"
                  title="Turn queued images into an MP4 slideshow video"
                >
                  <Video className="h-3.5 w-3.5" />
                  Slideshow
                </button>
                <button
                  type="button"
                  onClick={onCreateSpriteSheet}
                  className="inline-flex items-center gap-1 rounded-lg border border-teal-500/30 bg-teal-500/10 px-2.5 py-1.5 text-xs font-semibold text-teal-600 transition hover:bg-teal-500/20 dark:text-teal-400 cursor-pointer"
                  title="Pack queued images into a transparent Sprite Sheet + CSS map"
                >
                  <Layers className="h-3.5 w-3.5" />
                  Sprites
                </button>
                <div className="inline-flex items-center overflow-hidden rounded-lg border border-indigo-500/30 bg-indigo-500/10">
                  <button
                    type="button"
                    onClick={() => onMergePdf(pdfFit)}
                    className="inline-flex items-center gap-1 px-2.5 py-1.5 text-xs font-semibold text-indigo-600 transition hover:bg-indigo-500/20 dark:text-indigo-400 cursor-pointer"
                    title="Merge all queued images into one multi-page PDF"
                  >
                    <FileText className="h-3.5 w-3.5" />
                    Merge PDF
                  </button>
                  <select
                    value={pdfFit}
                    onChange={(e) => setPdfFit(e.target.value as PdfPageFit)}
                    className="border-l border-indigo-500/20 bg-transparent px-1.5 py-1.5 text-[10px] font-semibold text-indigo-600 outline-none dark:text-indigo-400 cursor-pointer"
                    title="PDF page size"
                  >
                    <option value="fit_to_image">Fit</option>
                    <option value="a4">A4</option>
                    <option value="letter">Letter</option>
                  </select>
                </div>
              </>
            )}
          {items.filter((i) => i.mediaKind === "pdf").length >= 2 &&
            !isConverting && (
              <button
                type="button"
                onClick={onMergePdfs}
                className="inline-flex items-center gap-1 rounded-lg border border-rose-500/30 bg-rose-500/10 px-2.5 py-1.5 text-xs font-semibold text-rose-600 transition hover:bg-rose-500/20 dark:text-rose-400 cursor-pointer"
                title="Merge queued PDFs losslessly into one PDF"
              >
                <FileText className="h-3.5 w-3.5" />
                Merge PDFs
              </button>
            )}
          {items.filter((i) => i.mediaKind === "video").length >= 2 &&
            !isConverting && (
              <button
                type="button"
                onClick={onMergeVideos}
                className="inline-flex items-center gap-1 rounded-lg border border-indigo-500/30 bg-indigo-500/10 px-2.5 py-1.5 text-xs font-semibold text-indigo-600 transition hover:bg-indigo-500/20 dark:text-indigo-400 cursor-pointer"
                title="Concatenate queued videos into a single MP4"
              >
                <Film className="h-3.5 w-3.5" />
                Merge Videos
              </button>
            )}
          {items.filter((i) => i.mediaKind === "audio").length >= 2 &&
            !isConverting && (
              <button
                type="button"
                onClick={onMergeAudio}
                className="inline-flex items-center gap-1 rounded-lg border border-violet-500/30 bg-violet-500/10 px-2.5 py-1.5 text-xs font-semibold text-violet-600 transition hover:bg-violet-500/20 dark:text-violet-400 cursor-pointer"
                title="Concatenate queued audio files into a single track"
              >
                <Music className="h-3.5 w-3.5" />
                Merge Audio
              </button>
            )}
          {completedCount > 0 && !isConverting && (
            <>
              <button
                type="button"
                onClick={onExportCsv}
                className="inline-flex items-center gap-1 rounded-lg border border-zinc-300 bg-white px-2.5 py-1.5 text-xs font-semibold text-zinc-700 transition hover:bg-zinc-100 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-200 dark:hover:bg-zinc-800 cursor-pointer"
                title="Download a CSV report of this batch conversion"
              >
                <Download className="h-3.5 w-3.5" />
                CSV
              </button>
              <button
                type="button"
                onClick={onRequeueOutputs}
                className="inline-flex items-center gap-1 rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-2.5 py-1.5 text-xs font-semibold text-emerald-600 transition hover:bg-emerald-500/20 dark:text-emerald-400 cursor-pointer"
                title="Load completed output files back into the queue to chain another step"
              >
                <RotateCw className="h-3.5 w-3.5" />
                Re-queue Outputs
              </button>
              <button
                type="button"
                onClick={onClearCompleted}
                className="rounded-lg px-2.5 py-1.5 text-xs font-medium text-zinc-500 transition hover:bg-zinc-200/70 hover:text-zinc-800 dark:text-zinc-400 dark:hover:bg-zinc-800 dark:hover:text-zinc-200 cursor-pointer"
              >
                Clear Done
              </button>
            </>
          )}
          <button
            type="button"
            onClick={onClearAll}
            disabled={isConverting}
            className="inline-flex items-center gap-1 rounded-lg px-2 py-1.5 text-xs font-medium text-zinc-500 transition hover:bg-red-500/10 hover:text-red-500 disabled:opacity-40 cursor-pointer"
            title="Clear all files"
          >
            <Trash2 className="h-3.5 w-3.5" />
          </button>
        </div>
      </div>

      {/* Filter Pills */}
      {(imageCount > 0 && videoCount > 0) || failedCount > 0 ? (
        <div className="mb-2.5 flex items-center gap-1.5">
          {(
            [
              { id: "all", label: `All (${items.length})` },
              ...(imageCount > 0
                ? [{ id: "image" as const, label: `Images (${imageCount})` }]
                : []),
              ...(videoCount > 0
                ? [{ id: "video" as const, label: `Video/Audio (${videoCount})` }]
                : []),
              ...(failedCount > 0
                ? [{ id: "failed" as const, label: `Failed (${failedCount})` }]
                : []),
            ] as { id: "all" | "image" | "video" | "failed"; label: string }[]
          ).map((f) => (
            <button
              key={f.id}
              type="button"
              onClick={() => setFilter(f.id)}
              className={`rounded-lg px-2.5 py-1 text-[11px] font-semibold transition cursor-pointer ${
                filter === f.id
                  ? "bg-zinc-900 text-white dark:bg-zinc-100 dark:text-zinc-900"
                  : "bg-zinc-200/60 text-zinc-600 hover:bg-zinc-200 dark:bg-zinc-900 dark:text-zinc-400"
              }`}
            >
              {f.label}
            </button>
          ))}
        </div>
      ) : null}

      {/* Search & Sort bar for larger queues */}
      {items.length >= 4 && (
        <div className="mb-2.5 flex items-center gap-2">
          <div className="relative flex-1">
            <Search className="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-zinc-400" />
            <input
              type="text"
              placeholder="Filter files by name..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full rounded-lg border border-zinc-200 bg-white py-1.5 pl-8 pr-2.5 text-xs text-zinc-900 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
            />
          </div>
          <select
            value={sortBy}
            onChange={(e) =>
              setSortBy(
                e.target.value as "order" | "name" | "size_desc" | "size_asc"
              )
            }
            className="rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs font-medium text-zinc-700 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 cursor-pointer"
            title="Sort queue"
          >
            <option value="order">Queue Order</option>
            <option value="name">Name (A–Z)</option>
            <option value="size_desc">Largest First</option>
            <option value="size_asc">Smallest First</option>
          </select>
        </div>
      )}

      {/* Queue List */}
      <div className="flex-1 space-y-2 overflow-y-auto pr-1">
        {visibleItems.map((item) => {
          const globalIndex = items.findIndex((x) => x.id === item.id);
          const targetLabel = targetLabelForItem(item, settings);
          const itemSaved =
            item.status === "completed" &&
            item.outputBytes != null &&
            item.sizeBytes > 0
              ? Math.round((1 - item.outputBytes / item.sizeBytes) * 100)
              : null;

          return (
            <div
              key={item.id}
              className={`group relative overflow-hidden rounded-xl border p-3 transition-colors ${
                item.status === "failed"
                  ? "border-red-500/30 bg-red-500/5"
                  : item.status === "completed"
                  ? "border-emerald-500/25 bg-emerald-500/[0.03] dark:bg-emerald-500/[0.04]"
                  : "border-zinc-200 bg-white dark:border-zinc-800/90 dark:bg-zinc-900/70"
              }`}
            >
              <div className="flex items-center gap-3">
                {/* Thumbnail / Kind Icon */}
                <div className="relative flex h-11 w-11 shrink-0 items-center justify-center overflow-hidden rounded-lg bg-zinc-100 dark:bg-zinc-800">
                  {item.thumbnailDataUrl ? (
                    <img
                      src={item.thumbnailDataUrl}
                      alt={item.fileName}
                      className="h-full w-full object-cover"
                    />
                  ) : item.mediaKind === "pdf" ? (
                    <FileText className="h-5 w-5 text-rose-500" />
                  ) : item.mediaKind === "video" ? (
                    <Film className="h-5 w-5 text-indigo-500" />
                  ) : item.mediaKind === "audio" ? (
                    <Music className="h-5 w-5 text-violet-500" />
                  ) : (
                    <ImageIcon className="h-5 w-5 text-zinc-400" />
                  )}
                </div>

                {/* Details */}
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <p
                      className="truncate text-sm font-medium text-zinc-900 dark:text-zinc-100"
                      title={item.path}
                    >
                      {item.fileName}
                    </p>
                  </div>

                  <div className="mt-0.5 flex flex-wrap items-center gap-1.5 text-[11px] text-zinc-500 dark:text-zinc-400">
                    <span className="rounded bg-zinc-200/70 px-1.5 py-0.5 font-semibold text-zinc-700 dark:bg-zinc-800 dark:text-zinc-300">
                      {item.sourceFormat}
                    </span>
                    <ArrowRight className="h-3 w-3 text-zinc-400" />
                    {item.mediaKind === "image" || item.mediaKind === "pdf" ? (
                      <select
                        disabled={isConverting}
                        value={item.overrideImageFormat ?? ""}
                        onChange={(e) =>
                          onOverrideImageFormat(
                            item.id,
                            e.target.value
                              ? (e.target.value as ImageFormat)
                              : undefined
                          )
                        }
                        title="Click to override format for this file only"
                        className={`rounded px-1.5 py-0.5 text-[11px] font-semibold outline-none cursor-pointer ${
                          item.overrideImageFormat
                            ? "bg-amber-500/20 text-amber-700 dark:text-amber-300 ring-1 ring-amber-500/40"
                            : "bg-indigo-500/15 text-indigo-600 dark:text-indigo-400"
                        }`}
                      >
                        <option value="">{targetLabel} (Global)</option>
                        {IMAGE_OUTPUT_FORMATS.map((f) => (
                          <option key={f.value} value={f.value}>
                            {f.label}
                          </option>
                        ))}
                      </select>
                    ) : (
                      <select
                        disabled={isConverting}
                        value={
                          item.overrideVideoTarget
                            ? `${item.overrideVideoTarget.kind}:${item.overrideVideoTarget.format}`
                            : ""
                        }
                        onChange={(e) => {
                          if (!e.target.value) {
                            onOverrideVideoTarget(item.id, undefined);
                          } else {
                            const [kind, fmt] = e.target.value.split(":");
                            onOverrideVideoTarget(
                              item.id,
                              kind === "video"
                                ? { kind: "video", format: fmt as VideoFormat }
                                : { kind: "audio", format: fmt as AudioFormat }
                            );
                          }
                        }}
                        title="Click to override format for this file only"
                        className={`rounded px-1.5 py-0.5 text-[11px] font-semibold outline-none cursor-pointer ${
                          item.overrideVideoTarget
                            ? "bg-amber-500/20 text-amber-700 dark:text-amber-300 ring-1 ring-amber-500/40"
                            : "bg-indigo-500/15 text-indigo-600 dark:text-indigo-400"
                        }`}
                      >
                        <option value="">{targetLabel} (Global)</option>
                        {VIDEO_OUTPUT_FORMATS.map((f) => (
                          <option key={f.value} value={`video:${f.value}`}>
                            {f.label}
                          </option>
                        ))}
                        {AUDIO_OUTPUT_FORMATS.map((f) => (
                          <option key={f.value} value={`audio:${f.value}`}>
                            {f.label}
                          </option>
                        ))}
                      </select>
                    )}
                    <span>·</span>
                    <span>{formatBytes(item.sizeBytes)}</span>
                    {item.width && item.height && (
                      <>
                        <span>·</span>
                        <span>
                          {item.width}×{item.height}
                        </span>
                      </>
                    )}
                    {item.pageCount != null && (
                      <>
                        <span>·</span>
                        <span>{item.pageCount} pages</span>
                      </>
                    )}
                    {item.durationSecs != null && (
                      <>
                        <span>·</span>
                        <span>{formatDuration(item.durationSecs)}</span>
                      </>
                    )}
                  </div>

                  {item.status === "failed" && item.error && (
                    <p
                      className="mt-1 line-clamp-2 break-words text-xs text-red-500"
                      title={item.error}
                    >
                      {item.error}
                    </p>
                  )}
                </div>

                {/* Status / Actions */}
                <div className="flex shrink-0 items-center gap-2">
                  {item.status === "converting" && (
                    <div className="flex items-center gap-1.5 text-xs font-medium text-indigo-500">
                      <Loader2 className="h-4 w-4 animate-spin" />
                      <span>{Math.round(item.progress)}%</span>
                    </div>
                  )}

                  {item.status === "completed" && (
                    <div className="flex items-center gap-2">
                      <div className="text-right">
                        <div className="flex items-center justify-end gap-1 text-xs font-semibold text-emerald-600 dark:text-emerald-400">
                          <CheckCircle2 className="h-3.5 w-3.5" />
                          <span>
                            {item.outputBytes != null
                              ? formatBytes(item.outputBytes)
                              : "Done"}
                          </span>
                        </div>
                        {itemSaved !== null && (
                          <span className="text-[11px] text-zinc-500 dark:text-zinc-400">
                            {itemSaved >= 0
                              ? `-${itemSaved}%`
                              : `+${Math.abs(itemSaved)}%`}
                            {item.durationMs != null
                              ? ` · ${(item.durationMs / 1000).toFixed(2)}s`
                              : ""}
                          </span>
                        )}
                      </div>
                      {item.outputPath && (
                        <button
                          type="button"
                          onClick={() => onRevealFile(item.outputPath!)}
                          className="rounded-lg p-1.5 text-zinc-400 transition hover:bg-zinc-200/80 hover:text-zinc-800 dark:hover:bg-zinc-800 dark:hover:text-zinc-200 cursor-pointer"
                          title="Show in folder"
                        >
                          <FolderOpen className="h-4 w-4" />
                        </button>
                      )}
                    </div>
                  )}

                  {item.status === "failed" && (
                    <AlertCircle className="h-4 w-4 text-red-500" />
                  )}

                  {!isConverting && (
                    <button
                      type="button"
                      onClick={() => onInspectItem(item)}
                      className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-zinc-200/80 hover:text-zinc-800 hover:opacity-100 dark:hover:bg-zinc-800 dark:hover:text-zinc-200 cursor-pointer"
                      title="Inspect EXIF / Media Metadata"
                    >
                      <Info className="h-4 w-4" />
                    </button>
                  )}

                  {item.mediaKind === "image" && !isConverting && (
                    <>
                      <button
                        type="button"
                        onClick={() => onDataUrlItem(item)}
                        className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-sky-500/10 hover:text-sky-500 hover:opacity-100 cursor-pointer"
                        title="Export as Base64 Data URL / HTML / CSS"
                      >
                        <Code2 className="h-4 w-4" />
                      </button>
                      <button
                        type="button"
                        onClick={() => onPaletteItem(item)}
                        className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-pink-500/10 hover:text-pink-500 hover:opacity-100 cursor-pointer"
                        title="Extract dominant color palette"
                      >
                        <Palette className="h-4 w-4" />
                      </button>
                      <button
                        type="button"
                        onClick={() => onResponsiveSetItem(item)}
                        className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-teal-500/10 hover:text-teal-500 hover:opacity-100 cursor-pointer"
                        title="Generate Responsive Web srcset bundle (320w..1920w)"
                      >
                        <MonitorSmartphone className="h-4 w-4" />
                      </button>
                      <button
                        type="button"
                        onClick={() => onSliceImage(item)}
                        className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-amber-500/10 hover:text-amber-500 hover:opacity-100 cursor-pointer"
                        title="Slice image into an equal cols×rows grid (e.g. 3×1 carousel)"
                      >
                        <Grid3x3 className="h-4 w-4" />
                      </button>
                      <button
                        type="button"
                        onClick={() => onGenerateIconPack(item)}
                        className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-violet-500/10 hover:text-violet-500 hover:opacity-100 cursor-pointer"
                        title="Generate Web & App Favicon / Icon Pack"
                      >
                        <Package className="h-4 w-4" />
                      </button>
                    </>
                  )}

                  {item.mediaKind === "video" && !isConverting && (
                    <>
                      <button
                        type="button"
                        onClick={() => onStoryboardItem(item)}
                        className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-indigo-500/10 hover:text-indigo-500 hover:opacity-100 cursor-pointer"
                        title="Generate 3×3 Video Storyboard Contact Sheet"
                      >
                        <LayoutGrid className="h-4 w-4" />
                      </button>
                      <button
                        type="button"
                        onClick={() => onExtractFrames(item)}
                        className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-cyan-500/10 hover:text-cyan-500 hover:opacity-100 cursor-pointer"
                        title="Extract Video Frame Sequence (FPS -> PNG/WebP/JPG)"
                      >
                        <Images className="h-4 w-4" />
                      </button>
                      <button
                        type="button"
                        onClick={() => onSplitMedia(item)}
                        className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-amber-500/10 hover:text-amber-500 hover:opacity-100 cursor-pointer"
                        title="Split video losslessly into equal-duration chunks"
                      >
                        <Scissors className="h-4 w-4" />
                      </button>
                    </>
                  )}

                  {item.mediaKind === "audio" && !isConverting && (
                    <>
                      <button
                        type="button"
                        onClick={() => onAudioWaveform(item)}
                        className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-violet-500/10 hover:text-violet-500 hover:opacity-100 cursor-pointer"
                        title="Generate Audio Waveform PNG Poster"
                      >
                        <AudioLines className="h-4 w-4" />
                      </button>
                      <button
                        type="button"
                        onClick={() => onPodcastVideo(item)}
                        className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-pink-500/10 hover:text-pink-500 hover:opacity-100 cursor-pointer"
                        title="Combine audio + cover image into an MP4 video"
                      >
                        <Video className="h-4 w-4" />
                      </button>
                      <button
                        type="button"
                        onClick={() => onSplitMedia(item)}
                        className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-amber-500/10 hover:text-amber-500 hover:opacity-100 cursor-pointer"
                        title="Split audio losslessly into equal-duration chunks"
                      >
                        <Scissors className="h-4 w-4" />
                      </button>
                    </>
                  )}

                  {item.mediaKind === "pdf" && !isConverting && (
                    <>
                      <button
                        type="button"
                        onClick={() => onRotatePdf(item)}
                        className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-amber-500/10 hover:text-amber-500 hover:opacity-100 cursor-pointer"
                        title="Rotate PDF pages 90° clockwise (lossless)"
                      >
                        <RotateCw className="h-4 w-4" />
                      </button>
                      <button
                        type="button"
                        onClick={() => onReorderPdf(item)}
                        className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-indigo-500/10 hover:text-indigo-500 hover:opacity-100 cursor-pointer"
                        title="Reorder or extract PDF pages (lossless)"
                      >
                        <ArrowUpDown className="h-4 w-4" />
                      </button>
                      <button
                        type="button"
                        onClick={() => onStampPdf(item)}
                        className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-red-500/10 hover:text-red-500 hover:opacity-100 cursor-pointer"
                        title="Stamp diagonal watermark text onto PDF pages (lossless)"
                      >
                        <Stamp className="h-4 w-4" />
                      </button>
                      <button
                        type="button"
                        onClick={() => onNumberPdf(item)}
                        className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-sky-500/10 hover:text-sky-500 hover:opacity-100 cursor-pointer"
                        title="Add centered page numbers to every PDF page (lossless)"
                      >
                        <Hash className="h-4 w-4" />
                      </button>
                      <button
                        type="button"
                        onClick={() => onExtractPdfImages(item)}
                        className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-teal-500/10 hover:text-teal-500 hover:opacity-100 cursor-pointer"
                        title="Extract original embedded images from PDF (lossless)"
                      >
                        <Images className="h-4 w-4" />
                      </button>
                      <button
                        type="button"
                        onClick={() => onScrubPdf(item)}
                        className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-emerald-500/10 hover:text-emerald-500 hover:opacity-100 cursor-pointer"
                        title="Scrub identifying PDF metadata (/Info & XMP) losslessly"
                      >
                        <ShieldCheck className="h-4 w-4" />
                      </button>
                      <button
                        type="button"
                        onClick={() => onCompressPdf(item)}
                        className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-emerald-500/10 hover:text-emerald-500 hover:opacity-100 cursor-pointer"
                        title="Compress / optimize PDF file size"
                      >
                        <Minimize2 className="h-4 w-4" />
                      </button>
                      <button
                        type="button"
                        onClick={() => onSplitPdf(item)}
                        className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-rose-500/10 hover:text-rose-500 hover:opacity-100 cursor-pointer"
                        title="Split / extract page range into a new PDF"
                      >
                        <Scissors className="h-4 w-4" />
                      </button>
                    </>
                  )}

                  {(item.mediaKind === "image" ||
                    item.mediaKind === "video" ||
                    item.mediaKind === "audio") &&
                    !isConverting && (
                      <button
                        type="button"
                        onClick={() => onPreviewItem(item)}
                        className="rounded-lg p-1.5 text-zinc-400 opacity-70 transition hover:bg-indigo-500/10 hover:text-indigo-500 hover:opacity-100 cursor-pointer"
                        title={
                          item.mediaKind === "image"
                            ? "Before / After preview & visual crop"
                            : item.mediaKind === "video"
                            ? "Video preview, visual trim & frame snapshot"
                            : "Audio preview & visual trim"
                        }
                      >
                        <Eye className="h-4 w-4" />
                      </button>
                    )}

                  {!isConverting &&
                    filter === "all" &&
                    !q &&
                    sortBy === "order" &&
                    items.length > 1 && (
                    <div className="flex flex-col opacity-0 transition group-hover:opacity-100">
                      <button
                        type="button"
                        disabled={globalIndex <= 0}
                        onClick={() => onMoveItem(globalIndex, -1)}
                        className="rounded p-0.5 text-zinc-400 hover:text-zinc-800 disabled:opacity-20 dark:hover:text-zinc-200 cursor-pointer"
                        title="Move up"
                      >
                        <ArrowUp className="h-3 w-3" />
                      </button>
                      <button
                        type="button"
                        disabled={globalIndex >= items.length - 1}
                        onClick={() => onMoveItem(globalIndex, 1)}
                        className="rounded p-0.5 text-zinc-400 hover:text-zinc-800 disabled:opacity-20 dark:hover:text-zinc-200 cursor-pointer"
                        title="Move down"
                      >
                        <ArrowDown className="h-3 w-3" />
                      </button>
                    </div>
                  )}

                  {!isConverting && (
                    <button
                      type="button"
                      onClick={() => onRemoveItem(item.id)}
                      className="rounded-lg p-1.5 text-zinc-400 opacity-60 transition hover:bg-zinc-200/80 hover:text-zinc-800 hover:opacity-100 dark:hover:bg-zinc-800 dark:hover:text-zinc-200 cursor-pointer"
                      title="Remove from queue"
                    >
                      <X className="h-3.5 w-3.5" />
                    </button>
                  )}
                </div>
              </div>

              {/* Progress Bar */}
              {item.status === "converting" && (
                <div className="mt-2.5 h-1 w-full overflow-hidden rounded-full bg-zinc-200 dark:bg-zinc-800">
                  <div
                    className="h-full rounded-full bg-gradient-to-r from-indigo-500 to-violet-500 transition-all duration-150"
                    style={{ width: `${Math.max(4, Math.min(100, item.progress))}%` }}
                  />
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
