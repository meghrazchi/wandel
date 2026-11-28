import { useEffect, useMemo, useState } from "react";
import {
  AlertCircle,
  ArrowDown,
  ArrowRight,
  ArrowUp,
  ArrowUpDown,
  AudioLines,
  CheckCircle2,
  ChevronDown,
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
  MoreHorizontal,
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
  Wrench,
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
  onGenerateSocialKit: (item: QueueItem) => void;
  onExtractGifFrames: (item: QueueItem) => void;
  onOverrideImageFormat: (id: string, format: ImageFormat | undefined) => void;
  onOverrideVideoTarget: (id: string, target: VideoTarget | undefined) => void;
  onMoveItem: (index: number, direction: -1 | 1) => void;
  onRetryFailed: () => void;
  onMergePdf: (pageFit: PdfPageFit) => void;
  onCreateGrid: () => void;
  onCompareImages: () => void;
  onPixelDiff: () => void;
  onCreateSlideshow: () => void;
  onCreateAnimatedGif: () => void;
  onStitchImages: (vertical: boolean) => void;
  onCreateSpriteSheet: () => void;
  onMergeVideos: () => void;
  onMergePdfs: () => void;
  onInterleavePdfs: () => void;
  onSplitPdf: (item: QueueItem) => void;
  onCompressPdf: (item: QueueItem) => void;
  onRotatePdf: (item: QueueItem) => void;
  onReorderPdf: (item: QueueItem) => void;
  onOrganizePdf: (item: QueueItem) => void;
  onStampPdf: (item: QueueItem) => void;
  onNumberPdf: (item: QueueItem) => void;
  onExtractPdfImages: (item: QueueItem) => void;
  onExtractPdfText: (item: QueueItem) => void;
  onExportPdfMarkdown: (item: QueueItem) => void;
  onTwoUpPdf: (item: QueueItem) => void;
  onEditPdfMeta: (item: QueueItem) => void;
  onDeblankPdf: (item: QueueItem) => void;
  onDarkModePdf: (item: QueueItem) => void;
  onScrubPdf: (item: QueueItem) => void;
  onPaletteItem: (item: QueueItem) => void;
  onDataUrlItem: (item: QueueItem) => void;
  onResponsiveSetItem: (item: QueueItem) => void;
  onSliceImage: (item: QueueItem) => void;
  onStoryboardItem: (item: QueueItem) => void;
  onExtractFrames: (item: QueueItem) => void;
  onExtractPoster: (item: QueueItem) => void;
  onAudioWaveform: (item: QueueItem) => void;
  onPodcastVideo: (item: QueueItem) => void;
  onSplitMedia: (item: QueueItem) => void;
  onExportCsv: () => void;
  onRequeueOutputs: () => void;
  onMergeAudio: () => void;
  onFindDuplicates: () => void;
  onOpenPipeline: () => void;
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
  onGenerateSocialKit,
  onExtractGifFrames,
  onOverrideImageFormat,
  onOverrideVideoTarget,
  onMoveItem,
  onRetryFailed,
  onMergePdf,
  onCreateGrid,
  onCompareImages,
  onPixelDiff,
  onCreateSlideshow,
  onCreateAnimatedGif,
  onStitchImages,
  onCreateSpriteSheet,
  onMergeVideos,
  onMergePdfs,
  onInterleavePdfs,
  onSplitPdf,
  onCompressPdf,
  onRotatePdf,
  onReorderPdf,
  onOrganizePdf,
  onStampPdf,
  onNumberPdf,
  onExtractPdfImages,
  onExtractPdfText,
  onExportPdfMarkdown,
  onTwoUpPdf,
  onEditPdfMeta,
  onDeblankPdf,
  onDarkModePdf,
  onScrubPdf,
  onPaletteItem,
  onDataUrlItem,
  onResponsiveSetItem,
  onSliceImage,
  onStoryboardItem,
  onExtractFrames,
  onExtractPoster,
  onAudioWaveform,
  onPodcastVideo,
  onSplitMedia,
  onExportCsv,
  onRequeueOutputs,
  onMergeAudio,
  onFindDuplicates,
  onOpenPipeline,
}: FileQueueProps) {
  const [filter, setFilter] = useState<"all" | "image" | "video" | "failed">("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [sortBy, setSortBy] = useState<"order" | "name" | "size_desc" | "size_asc">("order");
  const [pdfFit, setPdfFit] = useState<PdfPageFit>("fit_to_image");
  const [batchMenuOpen, setBatchMenuOpen] = useState(false);
  const [openRowMenuId, setOpenRowMenuId] = useState<string | null>(null);

  useEffect(() => {
    const closeAll = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setBatchMenuOpen(false);
        setOpenRowMenuId(null);
      }
    };
    const onDocClick = () => {
      setBatchMenuOpen(false);
      setOpenRowMenuId(null);
    };
    window.addEventListener("keydown", closeAll);
    window.addEventListener("click", onDocClick);
    return () => {
      window.removeEventListener("keydown", closeAll);
      window.removeEventListener("click", onDocClick);
    };
  }, []);

  const completedCount = items.filter((i) => i.status === "completed").length;
  const failedCount = items.filter((i) => i.status === "failed").length;
  const rawImageCount = items.filter((i) => i.mediaKind === "image").length;
  const rawPdfCount = items.filter((i) => i.mediaKind === "pdf").length;
  const rawVideoCount = items.filter((i) => i.mediaKind === "video").length;
  const rawAudioCount = items.filter((i) => i.mediaKind === "audio").length;
  const imageCount = rawImageCount + rawPdfCount;
  const videoCount = rawVideoCount + rawAudioCount;

  const hasBatchTools = items.length > 0;

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
  const totalOutputBytes = items.reduce((acc, i) => acc + (i.outputBytes ?? 0), 0);
  const completedInputBytes = items
    .filter((i) => i.status === "completed")
    .reduce((acc, i) => acc + i.sizeBytes, 0);
  const savedPercent =
    completedCount > 0 && completedInputBytes > 0
      ? Math.round((1 - totalOutputBytes / completedInputBytes) * 100)
      : null;

  return (
    <div className="flex h-full flex-col">
      {/* Clean Single-Row Queue Toolbar */}
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

          {/* Consolidated Batch Tools Menu */}
          {hasBatchTools && !isConverting && (
            <div className="relative" onClick={(e) => e.stopPropagation()}>
              <button
                type="button"
                aria-expanded={batchMenuOpen}
                onClick={() => {
                  setOpenRowMenuId(null);
                  setBatchMenuOpen((v) => !v);
                }}
                className="inline-flex items-center gap-1.5 rounded-lg border border-indigo-500/30 bg-indigo-500/10 px-2.5 py-1.5 text-xs font-semibold text-indigo-600 transition hover:bg-indigo-500/20 dark:text-indigo-400 cursor-pointer"
              >
                <Wrench className="h-3.5 w-3.5" />
                Batch Tools
                <ChevronDown className="h-3 w-3" />
              </button>

              {batchMenuOpen && (
                <div className="absolute right-0 z-30 mt-1.5 w-60 overflow-hidden rounded-xl border border-zinc-200 bg-white p-1.5 shadow-xl dark:border-zinc-800 dark:bg-zinc-950">
                  {rawImageCount >= 2 && (
                    <>
                      <div className="px-2 py-1 text-[10px] font-semibold uppercase tracking-wider text-zinc-400">
                        Images ({rawImageCount})
                      </div>
                      <div className="flex items-center justify-between rounded-lg px-2 py-1.5 text-xs hover:bg-zinc-100 dark:hover:bg-zinc-900">
                        <button
                          type="button"
                          onClick={() => {
                            setBatchMenuOpen(false);
                            onMergePdf(pdfFit);
                          }}
                          className="flex flex-1 items-center gap-2 text-left font-medium text-zinc-700 dark:text-zinc-200 cursor-pointer"
                        >
                          <FileText className="h-3.5 w-3.5 text-indigo-500" />
                          Merge Images → PDF
                        </button>
                        <select
                          value={pdfFit}
                          onChange={(e) => setPdfFit(e.target.value as PdfPageFit)}
                          className="rounded border border-zinc-200 bg-zinc-50 px-1 py-0.5 text-[10px] font-semibold dark:border-zinc-700 dark:bg-zinc-800"
                        >
                          <option value="fit_to_image">Fit</option>
                          <option value="a4">A4</option>
                          <option value="letter">Letter</option>
                        </select>
                      </div>
                      <MenuRow icon={<Grid2x2 className="h-3.5 w-3.5 text-violet-500" />} label="Contact Sheet Grid" onClick={() => { setBatchMenuOpen(false); onCreateGrid(); }} />
                      <MenuRow icon={<Columns2 className="h-3.5 w-3.5 text-cyan-500" />} label="Before / After Compare" onClick={() => { setBatchMenuOpen(false); onCompareImages(); }} />
                      <MenuRow icon={<Eye className="h-3.5 w-3.5 text-rose-500" />} label="Visual Pixel Diff" onClick={() => { setBatchMenuOpen(false); onPixelDiff(); }} />
                      <MenuRow icon={<Video className="h-3.5 w-3.5 text-pink-500" />} label="Slideshow MP4 Video" onClick={() => { setBatchMenuOpen(false); onCreateSlideshow(); }} />
                      <MenuRow icon={<Film className="h-3.5 w-3.5 text-emerald-500" />} label="Create Animated GIF" onClick={() => { setBatchMenuOpen(false); onCreateAnimatedGif(); }} />
                      <MenuRow icon={<ArrowUpDown className="h-3.5 w-3.5 text-indigo-500" />} label="Stitch Vertically" onClick={() => { setBatchMenuOpen(false); onStitchImages(true); }} />
                      <MenuRow icon={<Columns2 className="h-3.5 w-3.5 text-sky-500" />} label="Stitch Horizontally" onClick={() => { setBatchMenuOpen(false); onStitchImages(false); }} />
                      <MenuRow icon={<Layers className="h-3.5 w-3.5 text-teal-500" />} label="Sprite Sheet + CSS Map" onClick={() => { setBatchMenuOpen(false); onCreateSpriteSheet(); }} />
                      <MenuRow icon={<CopyCheck className="h-3.5 w-3.5 text-amber-500" />} label="Find & Remove Duplicates" onClick={() => { setBatchMenuOpen(false); onFindDuplicates(); }} />
                    </>
                  )}
                  {rawPdfCount >= 2 && (
                    <>
                      <MenuRow icon={<FileText className="h-3.5 w-3.5 text-rose-500" />} label={`Merge ${rawPdfCount} PDFs (Lossless)`} onClick={() => { setBatchMenuOpen(false); onMergePdfs(); }} />
                      <MenuRow icon={<Layers className="h-3.5 w-3.5 text-indigo-500" />} label="Interleave 2 PDFs (Duplex)" onClick={() => { setBatchMenuOpen(false); onInterleavePdfs(); }} />
                    </>
                  )}
                  {rawVideoCount >= 2 && (
                    <MenuRow icon={<Film className="h-3.5 w-3.5 text-indigo-500" />} label={`Merge ${rawVideoCount} Videos`} onClick={() => { setBatchMenuOpen(false); onMergeVideos(); }} />
                  )}
                  <div className="my-1 border-t border-zinc-200 dark:border-zinc-800" />
                  <MenuRow icon={<Wrench className="h-3.5 w-3.5 text-indigo-500" />} label="Run Multi-Step Pipeline..." onClick={() => { setBatchMenuOpen(false); onOpenPipeline(); }} />
                  {rawAudioCount >= 2 && (
                    <MenuRow icon={<Music className="h-3.5 w-3.5 text-violet-500" />} label={`Merge ${rawAudioCount} Audio Tracks`} onClick={() => { setBatchMenuOpen(false); onMergeAudio(); }} />
                  )}
                  {completedCount > 0 && (
                    <>
                      <div className="my-1 border-t border-zinc-200 dark:border-zinc-800" />
                      <MenuRow icon={<RotateCw className="h-3.5 w-3.5 text-emerald-500" />} label="Re-queue Completed Outputs" onClick={() => { setBatchMenuOpen(false); onRequeueOutputs(); }} />
                      <MenuRow icon={<Download className="h-3.5 w-3.5 text-zinc-500" />} label="Export Batch CSV Report" onClick={() => { setBatchMenuOpen(false); onExportCsv(); }} />
                    </>
                  )}
                </div>
              )}
            </div>
          )}

          {failedCount > 0 && !isConverting && (
            <button
              type="button"
              onClick={onRetryFailed}
              className="inline-flex items-center gap-1 rounded-lg border border-red-500/30 bg-red-500/10 px-2.5 py-1.5 text-xs font-medium text-red-600 transition hover:bg-red-500/20 dark:text-red-400 cursor-pointer"
            >
              <RefreshCw className="h-3.5 w-3.5" />
              Retry ({failedCount})
            </button>
          )}
          {completedCount > 0 && !isConverting && (
            <button
              type="button"
              onClick={onClearCompleted}
              className="rounded-lg px-2 py-1.5 text-xs font-medium text-zinc-500 transition hover:bg-zinc-200/70 hover:text-zinc-800 dark:text-zinc-400 dark:hover:bg-zinc-800 cursor-pointer"
            >
              Clear Done
            </button>
          )}
          <button
            type="button"
            onClick={onClearAll}
            disabled={isConverting}
            className="rounded-lg p-1.5 text-zinc-500 transition hover:bg-red-500/10 hover:text-red-500 disabled:opacity-40 cursor-pointer"
            title="Clear all files"
            aria-label="Clear all files"
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
              ...(imageCount > 0 ? [{ id: "image" as const, label: `Images (${imageCount})` }] : []),
              ...(videoCount > 0 ? [{ id: "video" as const, label: `Video/Audio (${videoCount})` }] : []),
              ...(failedCount > 0 ? [{ id: "failed" as const, label: `Failed (${failedCount})` }] : []),
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
              setSortBy(e.target.value as "order" | "name" | "size_desc" | "size_asc")
            }
            className="rounded-lg border border-zinc-200 bg-white px-2.5 py-1.5 text-xs font-medium text-zinc-700 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 cursor-pointer"
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
          const menuOpen = openRowMenuId === item.id;

          return (
            <div
              key={item.id}
              className={`group relative rounded-xl border p-3 transition-colors ${
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
                  <p
                    className="truncate text-sm font-medium text-zinc-900 dark:text-zinc-100"
                    title={item.path}
                  >
                    {item.fileName}
                  </p>

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
                        title="Override format for this file"
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
                        title="Override format for this file"
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

                {/* Clean Status & Primary Row Actions */}
                <div className="flex shrink-0 items-center gap-1.5">
                  {item.status === "converting" && (
                    <div className="flex items-center gap-1.5 text-xs font-medium text-indigo-500">
                      <Loader2 className="h-4 w-4 animate-spin" />
                      <span>{Math.round(item.progress)}%</span>
                    </div>
                  )}

                  {item.status === "completed" && (
                    <div className="flex items-center gap-1.5">
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
                            {itemSaved >= 0 ? `-${itemSaved}%` : `+${Math.abs(itemSaved)}%`}
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

                  {(item.mediaKind === "image" ||
                    item.mediaKind === "video" ||
                    item.mediaKind === "audio") &&
                    !isConverting && (
                      <button
                        type="button"
                        onClick={() => onPreviewItem(item)}
                        className="rounded-lg p-1.5 text-zinc-400 transition hover:bg-indigo-500/10 hover:text-indigo-500 cursor-pointer"
                        title="Preview, crop or trim"
                        aria-label="Preview item"
                      >
                        <Eye className="h-4 w-4" />
                      </button>
                    )}

                  {/* Per-item contextual Tools popover */}
                  {!isConverting && (
                    <div className="relative" onClick={(e) => e.stopPropagation()}>
                      <button
                        type="button"
                        aria-expanded={menuOpen}
                        aria-label="More item tools"
                        onClick={() => {
                          setBatchMenuOpen(false);
                          setOpenRowMenuId(menuOpen ? null : item.id);
                        }}
                        className="rounded-lg p-1.5 text-zinc-400 transition hover:bg-zinc-200/80 hover:text-zinc-800 dark:hover:bg-zinc-800 dark:hover:text-zinc-200 cursor-pointer"
                        title="More tools..."
                      >
                        <MoreHorizontal className="h-4 w-4" />
                      </button>

                      {menuOpen && (
                        <div className="absolute right-0 z-30 mt-1 w-56 overflow-hidden rounded-xl border border-zinc-200 bg-white p-1.5 shadow-xl dark:border-zinc-800 dark:bg-zinc-950">
                          <MenuRow icon={<Info className="h-3.5 w-3.5 text-zinc-500" />} label="Inspect Metadata / EXIF" onClick={() => { setOpenRowMenuId(null); onInspectItem(item); }} />
                          <div className="my-1 border-t border-zinc-200 dark:border-zinc-800" />

                          {item.mediaKind === "image" && (
                            <>
                              <MenuRow icon={<MonitorSmartphone className="h-3.5 w-3.5 text-teal-500" />} label="Responsive srcset Bundle" onClick={() => { setOpenRowMenuId(null); onResponsiveSetItem(item); }} />
                              <MenuRow icon={<Grid3x3 className="h-3.5 w-3.5 text-amber-500" />} label="Slice Grid (3×1 / 3×3)" onClick={() => { setOpenRowMenuId(null); onSliceImage(item); }} />
                              <MenuRow icon={<Package className="h-3.5 w-3.5 text-violet-500" />} label="Favicon & App Icon Pack" onClick={() => { setOpenRowMenuId(null); onGenerateIconPack(item); }} />
                              <MenuRow icon={<LayoutGrid className="h-3.5 w-3.5 text-indigo-500" />} label="Social Media Kit (5 Sizes)" onClick={() => { setOpenRowMenuId(null); onGenerateSocialKit(item); }} />
                              {item.sourceFormat.toUpperCase() === "GIF" && (
                                <MenuRow icon={<Images className="h-3.5 w-3.5 text-emerald-500" />} label="Extract GIF Frames" onClick={() => { setOpenRowMenuId(null); onExtractGifFrames(item); }} />
                              )}
                              <MenuRow icon={<Palette className="h-3.5 w-3.5 text-pink-500" />} label="Extract Color Palette" onClick={() => { setOpenRowMenuId(null); onPaletteItem(item); }} />
                              <MenuRow icon={<Code2 className="h-3.5 w-3.5 text-sky-500" />} label="Base64 Data URL" onClick={() => { setOpenRowMenuId(null); onDataUrlItem(item); }} />
                            </>
                          )}

                          {item.mediaKind === "pdf" && (
                            <>
                              <MenuRow icon={<LayoutGrid className="h-3.5 w-3.5 text-indigo-500" />} label="Visual Page Organizer" onClick={() => { setOpenRowMenuId(null); onOrganizePdf(item); }} />
                              <MenuRow icon={<RotateCw className="h-3.5 w-3.5 text-amber-500" />} label="Rotate Pages 90° CW" onClick={() => { setOpenRowMenuId(null); onRotatePdf(item); }} />
                              <MenuRow icon={<ArrowUpDown className="h-3.5 w-3.5 text-indigo-500" />} label="Reorder / Extract Pages" onClick={() => { setOpenRowMenuId(null); onReorderPdf(item); }} />
                              <MenuRow icon={<Stamp className="h-3.5 w-3.5 text-red-500" />} label="Stamp Watermark Text" onClick={() => { setOpenRowMenuId(null); onStampPdf(item); }} />
                              <MenuRow icon={<Hash className="h-3.5 w-3.5 text-sky-500" />} label="Add Page Numbers" onClick={() => { setOpenRowMenuId(null); onNumberPdf(item); }} />
                              <MenuRow icon={<Images className="h-3.5 w-3.5 text-teal-500" />} label="Extract Embedded Images" onClick={() => { setOpenRowMenuId(null); onExtractPdfImages(item); }} />
                              <MenuRow icon={<FileText className="h-3.5 w-3.5 text-indigo-500" />} label="Extract Text (.txt)" onClick={() => { setOpenRowMenuId(null); onExtractPdfText(item); }} />
                              <MenuRow icon={<Code2 className="h-3.5 w-3.5 text-sky-500" />} label="Export as Markdown (.md)" onClick={() => { setOpenRowMenuId(null); onExportPdfMarkdown(item); }} />
                              <MenuRow icon={<Columns2 className="h-3.5 w-3.5 text-cyan-500" />} label="2-Up Side-by-Side PDF" onClick={() => { setOpenRowMenuId(null); onTwoUpPdf(item); }} />
                              <MenuRow icon={<Info className="h-3.5 w-3.5 text-amber-500" />} label="Edit PDF Metadata" onClick={() => { setOpenRowMenuId(null); onEditPdfMeta(item); }} />
                              <MenuRow icon={<Trash2 className="h-3.5 w-3.5 text-indigo-500" />} label="Remove Blank Pages" onClick={() => { setOpenRowMenuId(null); onDeblankPdf(item); }} />
                              <MenuRow icon={<Eye className="h-3.5 w-3.5 text-violet-500" />} label="Convert to Dark Mode PDF" onClick={() => { setOpenRowMenuId(null); onDarkModePdf(item); }} />
                              <MenuRow icon={<ShieldCheck className="h-3.5 w-3.5 text-emerald-500" />} label="Scrub Metadata (Privacy)" onClick={() => { setOpenRowMenuId(null); onScrubPdf(item); }} />
                              <MenuRow icon={<Minimize2 className="h-3.5 w-3.5 text-emerald-500" />} label="Compress PDF Size" onClick={() => { setOpenRowMenuId(null); onCompressPdf(item); }} />
                              <MenuRow icon={<Scissors className="h-3.5 w-3.5 text-rose-500" />} label="Split Page Range" onClick={() => { setOpenRowMenuId(null); onSplitPdf(item); }} />
                            </>
                          )}

                          {item.mediaKind === "video" && (
                            <>
                              <MenuRow icon={<LayoutGrid className="h-3.5 w-3.5 text-indigo-500" />} label="3×3 Video Storyboard" onClick={() => { setOpenRowMenuId(null); onStoryboardItem(item); }} />
                              <MenuRow icon={<ImageIcon className="h-3.5 w-3.5 text-emerald-500" />} label="Save Poster Frame (.jpg)" onClick={() => { setOpenRowMenuId(null); onExtractPoster(item); }} />
                              <MenuRow icon={<Images className="h-3.5 w-3.5 text-cyan-500" />} label="Extract Frame Sequence" onClick={() => { setOpenRowMenuId(null); onExtractFrames(item); }} />
                              <MenuRow icon={<Scissors className="h-3.5 w-3.5 text-amber-500" />} label="Split into Chunks (Lossless)" onClick={() => { setOpenRowMenuId(null); onSplitMedia(item); }} />
                            </>
                          )}

                          {item.mediaKind === "audio" && (
                            <>
                              <MenuRow icon={<AudioLines className="h-3.5 w-3.5 text-violet-500" />} label="Waveform PNG Poster" onClick={() => { setOpenRowMenuId(null); onAudioWaveform(item); }} />
                              <MenuRow icon={<Video className="h-3.5 w-3.5 text-pink-500" />} label="Cover + Audio → MP4" onClick={() => { setOpenRowMenuId(null); onPodcastVideo(item); }} />
                              <MenuRow icon={<Scissors className="h-3.5 w-3.5 text-amber-500" />} label="Split into Chunks (Lossless)" onClick={() => { setOpenRowMenuId(null); onSplitMedia(item); }} />
                            </>
                          )}
                        </div>
                      )}
                    </div>
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
                      aria-label="Remove from queue"
                    >
                      <X className="h-3.5 w-3.5" />
                    </button>
                  )}
                </div>
              </div>

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

function MenuRow({
  icon,
  label,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="flex w-full items-center gap-2 rounded-lg px-2 py-1.5 text-left text-xs font-medium text-zinc-700 transition hover:bg-zinc-100 dark:text-zinc-200 dark:hover:bg-zinc-900 cursor-pointer"
    >
      {icon}
      <span>{label}</span>
    </button>
  );
}
