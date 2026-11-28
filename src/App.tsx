import { useCallback, useEffect, useMemo, useState } from "react";
import { Channel, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open, save } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import {
  Command,
  Cpu,
  HelpCircle,
  History,
  Moon,
  Plug,
  QrCode,
  Radio,
  Sun,
  UploadCloud,
  X,
  Zap,
} from "lucide-react";
import { AudioPreviewModal } from "./components/AudioPreviewModal";
import { CommandPalette, PaletteAction } from "./components/CommandPalette";
import { DataUrlModal } from "./components/DataUrlModal";
import { DropZone } from "./components/DropZone";
import { FileQueue } from "./components/FileQueue";
import { HistoryDrawer } from "./components/HistoryDrawer";
import { McpModal } from "./components/McpModal";
import { MetadataModal } from "./components/MetadataModal";
import { PaletteModal } from "./components/PaletteModal";
import { PdfOrganizerModal } from "./components/PdfOrganizerModal";
import { PipelineModal } from "./components/PipelineModal";
import { PreviewModal } from "./components/PreviewModal";
import { QrModal } from "./components/QrModal";
import { SettingsPanel } from "./components/SettingsPanel";
import { ShortcutsModal } from "./components/ShortcutsModal";
import { VideoPreviewModal } from "./components/VideoPreviewModal";
import { WandelButton } from "./components/WandelButton";
import { WandelIcon } from "./components/WandelIcon";
import { WatchFolderModal } from "./components/WatchFolderModal";
import { t } from "./i18n";
import {
  ConversionSettings,
  CustomPreset,
  DEFAULT_SETTINGS,
  DuplicateGroup,
  FileMetadata,
  HardwareCapabilities,
  HistoryEntry,
  ImageFormat,
  LifetimeStats,
  Locale,
  PdfPageFit,
  Preset,
  ProgressEvent,
  QueueItem,
  ResponsiveSetResult,
  SpriteSheetResult,
  VideoTarget,
  WatchConvertedPayload,
  WatchFolderInfo,
} from "./types";

const CUSTOM_PRESETS_KEY = "wandel-custom-presets";
const HISTORY_KEY = "wandel-history";
const STATS_KEY = "wandel-lifetime-stats";
const SETTINGS_KEY = "wandel-settings-v1";

export default function App() {
  const [items, setItems] = useState<QueueItem[]>([]);
  const [settings, setSettings] = useState<ConversionSettings>(() => {
    try {
      const raw = localStorage.getItem(SETTINGS_KEY);
      if (!raw) return DEFAULT_SETTINGS;
      const saved = JSON.parse(raw) as Partial<ConversionSettings>;
      return {
        ...DEFAULT_SETTINGS,
        ...saved,
        image: { ...DEFAULT_SETTINGS.image, ...(saved.image ?? {}) },
        video: { ...DEFAULT_SETTINGS.video, ...(saved.video ?? {}) },
        pdf: { ...DEFAULT_SETTINGS.pdf, ...(saved.pdf ?? {}) },
        output: { ...DEFAULT_SETTINGS.output, ...(saved.output ?? {}) },
      };
    } catch {
      return DEFAULT_SETTINGS;
    }
  });
  const [activePreset, setActivePreset] = useState<string | null>(null);
  const [customPresets, setCustomPresets] = useState<CustomPreset[]>(() => {
    try {
      const raw = localStorage.getItem(CUSTOM_PRESETS_KEY);
      return raw ? (JSON.parse(raw) as CustomPreset[]) : [];
    } catch {
      return [];
    }
  });
  const [history, setHistory] = useState<HistoryEntry[]>(() => {
    try {
      const raw = localStorage.getItem(HISTORY_KEY);
      return raw ? (JSON.parse(raw) as HistoryEntry[]) : [];
    } catch {
      return [];
    }
  });
  const [lifetimeStats, setLifetimeStats] = useState<LifetimeStats>(() => {
    try {
      const raw = localStorage.getItem(STATS_KEY);
      return raw
        ? (JSON.parse(raw) as LifetimeStats)
        : { totalFiles: 0, totalInputBytes: 0, totalOutputBytes: 0 };
    } catch {
      return { totalFiles: 0, totalInputBytes: 0, totalOutputBytes: 0 };
    }
  });
  const [showHistory, setShowHistory] = useState(false);
  const [showWatchModal, setShowWatchModal] = useState(false);
  const [showCommandPalette, setShowCommandPalette] = useState(false);
  const [showQrModal, setShowQrModal] = useState(false);
  const [showMcpModal, setShowMcpModal] = useState(false);
  const [showPipelineModal, setShowPipelineModal] = useState(false);
  const [showShortcuts, setShowShortcuts] = useState(false);
  const [inspectItem, setInspectItem] = useState<QueueItem | null>(null);
  const [paletteItem, setPaletteItem] = useState<QueueItem | null>(null);
  const [pdfOrganizerItem, setPdfOrganizerItem] = useState<QueueItem | null>(null);
  const [dataUrlItem, setDataUrlItem] = useState<QueueItem | null>(null);
  const [watchFolders, setWatchFolders] = useState<WatchFolderInfo[]>([]);
  const [previewItem, setPreviewItem] = useState<QueueItem | null>(null);
  const [hardware, setHardware] = useState<HardwareCapabilities | null>(null);
  const [locale, setLocale] = useState<Locale>(() => {
    return localStorage.getItem("wandel-locale") === "de" ? "de" : "en";
  });
  const [minimizeToTray, setMinimizeToTray] = useState(() => {
    return localStorage.getItem("wandel-minimize-tray") === "true";
  });
  const [isDragging, setIsDragging] = useState(false);
  const [isInspecting, setIsInspecting] = useState(false);
  const [isConverting, setIsConverting] = useState(false);
  const [mobileTab, setMobileTab] = useState<"queue" | "settings">("queue");
  const [batchSummary, setBatchSummary] = useState<string | null>(null);
  const [darkMode, setDarkMode] = useState(() => {
    const saved = localStorage.getItem("wandel-theme");
    return saved ? saved === "dark" : true;
  });

  // Sync theme class on <html>
  useEffect(() => {
    const root = document.documentElement;
    if (darkMode) root.classList.add("dark");
    else root.classList.remove("dark");
    localStorage.setItem("wandel-theme", darkMode ? "dark" : "light");
  }, [darkMode]);

  const recordHistory = useCallback(
    (entry: Omit<HistoryEntry, "id" | "timestamp">) => {
      const full: HistoryEntry = {
        ...entry,
        id: `${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
        timestamp: Date.now(),
      };
      setHistory((prev) => {
        const next = [full, ...prev].slice(0, 200);
        localStorage.setItem(HISTORY_KEY, JSON.stringify(next));
        return next;
      });
      setLifetimeStats((prev) => {
        const next: LifetimeStats = {
          totalFiles: prev.totalFiles + 1,
          totalInputBytes: prev.totalInputBytes + entry.inputBytes,
          totalOutputBytes: prev.totalOutputBytes + entry.outputBytes,
        };
        localStorage.setItem(STATS_KEY, JSON.stringify(next));
        return next;
      });
    },
    []
  );

  // Persist active settings across restarts
  useEffect(() => {
    try {
      localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
    } catch {
      // ignore quota errors
    }
  }, [settings]);

  // Sync minimize-to-tray preference with Tauri backend
  useEffect(() => {
    localStorage.setItem("wandel-minimize-tray", String(minimizeToTray));
    invoke("set_minimize_to_tray", { enabled: minimizeToTray }).catch(
      console.error
    );
  }, [minimizeToTray]);

  // Fetch CPU/GPU hardware info & active watch folders on startup
  useEffect(() => {
    invoke<HardwareCapabilities>("get_hardware_info")
      .then(setHardware)
      .catch(console.error);
    invoke<WatchFolderInfo[]>("list_watch_folders")
      .then(setWatchFolders)
      .catch(console.error);

    let unlisten: (() => void) | undefined;
    listen<WatchConvertedPayload>("wandel://watch-converted", (event) => {
      const p = event.payload;
      setWatchFolders((prev) =>
        prev.map((w) =>
          w.folderPath === p.folderPath
            ? { ...w, convertedCount: w.convertedCount + 1 }
            : w
        )
      );
      recordHistory({
        fileName: p.fileName,
        outputPath: p.outputPath,
        inputBytes: p.inputBytes,
        outputBytes: p.outputBytes,
        source: "watch",
      });
    })
      .then((fn) => {
        unlisten = fn;
      })
      .catch(console.error);

    return () => {
      if (unlisten) unlisten();
    };
  }, [recordHistory]);

  const addPaths = useCallback(async (paths: string[]) => {
    if (paths.length === 0) return;
    setIsInspecting(true);
    setBatchSummary(null);
    try {
      const inspected = await invoke<FileMetadata[]>("inspect_paths", {
        paths,
      });
      setItems((prev) => {
        const existing = new Set(prev.map((i) => i.id));
        const additions: QueueItem[] = inspected
          .filter((meta) => !existing.has(meta.id))
          .map((meta) => ({
            ...meta,
            status: "pending",
            progress: 0,
          }));
        return [...prev, ...additions];
      });
    } catch (err) {
      console.error("Failed to inspect paths:", err);
    } finally {
      setIsInspecting(false);
    }
  }, []);

  // Native Tauri v2 drag-and-drop listener
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const win = getCurrentWindow();
    win
      .onDragDropEvent((event) => {
        if (event.payload.type === "enter" || event.payload.type === "over") {
          setIsDragging(true);
        } else if (event.payload.type === "leave") {
          setIsDragging(false);
        } else if (event.payload.type === "drop") {
          setIsDragging(false);
          if (!isConverting) {
            void addPaths(event.payload.paths);
          }
        }
      })
      .then((fn) => {
        unlisten = fn;
      })
      .catch(console.error);

    return () => {
      if (unlisten) unlisten();
    };
  }, [addPaths, isConverting]);

  const handleAddFiles = useCallback(async () => {
    const selected = await open({
      multiple: true,
      directory: false,
      title: "Select images or videos",
    });
    if (!selected) return;
    const paths = Array.isArray(selected) ? selected : [selected];
    await addPaths(paths);
  }, [addPaths]);

  const handleAddFolder = useCallback(async () => {
    const selected = await open({
      multiple: true,
      directory: true,
      title: "Select folder to convert",
    });
    if (!selected) return;
    const paths = Array.isArray(selected) ? selected : [selected];
    await addPaths(paths);
  }, [addPaths]);

  const handlePickOutputDir = useCallback(async () => {
    const selected = await open({
      multiple: false,
      directory: true,
      title: "Select output folder",
    });
    if (typeof selected === "string") {
      setSettings((prev) => ({
        ...prev,
        output: { ...prev.output, outputDir: selected },
      }));
    }
  }, []);

  const handlePickWatermark = useCallback(async () => {
    const selected = await open({
      multiple: false,
      directory: false,
      title: "Select watermark logo (PNG, SVG, WebP)",
      filters: [{ name: "Images", extensions: ["png", "svg", "webp", "jpg"] }],
    });
    if (typeof selected === "string") {
      setSettings((prev) => ({
        ...prev,
        image: {
          ...prev.image,
          watermark: {
            imagePath: selected,
            opacity: prev.image.watermark?.opacity ?? 0.75,
            scalePercent: prev.image.watermark?.scalePercent ?? 18,
            marginPx: prev.image.watermark?.marginPx ?? 24,
            position: prev.image.watermark?.position ?? "bottom_right",
          },
        },
      }));
      setActivePreset(null);
    }
  }, []);

  const handleAddWatchFolder = useCallback(async () => {
    const selected = await open({
      multiple: false,
      directory: true,
      title: "Select folder to watch for Auto-Wandel",
    });
    if (typeof selected === "string") {
      try {
        const info = await invoke<WatchFolderInfo>("start_watch_folder", {
          folderPath: selected,
          settings,
        });
        setWatchFolders((prev) => [
          ...prev.filter((w) => w.folderPath !== info.folderPath),
          info,
        ]);
      } catch (err) {
        console.error("Failed to start watch folder:", err);
      }
    }
  }, [settings]);

  const handleStopWatchFolder = useCallback(async (folderPath: string) => {
    try {
      await invoke("stop_watch_folder", { folderPath });
      setWatchFolders((prev) => prev.filter((w) => w.folderPath !== folderPath));
    } catch (err) {
      console.error("Failed to stop watch folder:", err);
    }
  }, []);

  const handlePickVideoWatermark = useCallback(async () => {
    const selected = await open({
      multiple: false,
      directory: false,
      title: "Select video watermark logo (PNG / WebP)",
      filters: [{ name: "Images", extensions: ["png", "webp", "jpg"] }],
    });
    if (typeof selected === "string") {
      setSettings((prev) => ({
        ...prev,
        video: {
          ...prev.video,
          watermark: {
            imagePath: selected,
            opacity: 0.8,
            scalePercent: 15,
            marginPx: 24,
            position: "bottom_right",
          },
        },
      }));
    }
  }, []);

  const handleGenerateSocialKit = useCallback(async (item: QueueItem) => {
    const dir = await open({
      multiple: false,
      directory: true,
      title: "Select folder for Social Media Kit (5 platform sizes)",
    });
    if (typeof dir !== "string") return;

    setBatchSummary("Generating 5-platform Social Media Kit...");
    try {
      const files = await invoke<string[]>("generate_social_kit", {
        path: item.path,
        outputDir: dir,
        format: settings.image.format,
      });
      setBatchSummary(`Generated ${files.length} social media assets!`);
      if (files[0]) await revealItemInDir(files[0]);
    } catch (err) {
      setBatchSummary(`Social kit failed: ${String(err)}`);
    }
  }, [settings.image.format]);

  const handleCreateAnimatedGif = useCallback(async () => {
    const imagePaths = items
      .filter((i) => i.mediaKind === "image")
      .map((i) => i.path);
    if (imagePaths.length < 2) return;

    const dest = await save({
      title: "Save Animated GIF",
      defaultPath: "wandel_animation.gif",
      filters: [{ name: "GIF Animation", extensions: ["gif"] }],
    });
    if (!dest) return;

    setBatchSummary(`Creating animated GIF from ${imagePaths.length} images...`);
    try {
      const bytes = await invoke<number>("create_animated_gif", {
        paths: imagePaths,
        outputPath: dest,
        delayMs: 300,
        maxWidth: 800,
      });
      setBatchSummary(`Created animated GIF (${(bytes / 1024).toFixed(0)} KB)`);
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`Animated GIF failed: ${String(err)}`);
    }
  }, [items]);

  const handleExtractGifFrames = useCallback(async (item: QueueItem) => {
    const dir = await open({
      multiple: false,
      directory: true,
      title: "Select folder for extracted GIF frames",
    });
    if (typeof dir !== "string") return;

    setBatchSummary("Extracting GIF frames in parallel...");
    try {
      const files = await invoke<string[]>("extract_gif_frames", {
        path: item.path,
        outputDir: dir,
        format: "png",
      });
      setBatchSummary(`Extracted ${files.length} GIF frame(s)!`);
      if (files[0]) await revealItemInDir(files[0]);
    } catch (err) {
      setBatchSummary(`GIF frame extraction failed: ${String(err)}`);
    }
  }, []);

  const handleStitchImages = useCallback(
    async (vertical: boolean) => {
      const imagePaths = items
        .filter((i) => i.mediaKind === "image")
        .map((i) => i.path);
      if (imagePaths.length < 2) return;

      const dest = await save({
        title: `Save ${vertical ? "Vertical" : "Horizontal"} Stitched Image`,
        defaultPath: `wandel_stitch_${vertical ? "v" : "h"}.png`,
        filters: [{ name: "Images", extensions: ["png", "jpg", "webp"] }],
      });
      if (!dest) return;

      const ext = dest.split(".").pop()?.toLowerCase();
      const fmt: ImageFormat =
        ext === "jpg" || ext === "jpeg"
          ? "jpg"
          : ext === "webp"
          ? "webp"
          : "png";

      setBatchSummary(`Stitching ${imagePaths.length} images...`);
      try {
        const bytes = await invoke<number>("stitch_images", {
          paths: imagePaths,
          outputPath: dest,
          vertical,
          gapPx: 0,
          bgHex: "#ffffff",
          settings: { ...settings.image, format: fmt },
        });
        setBatchSummary(`Stitched ${imagePaths.length} images (${(bytes / 1024).toFixed(0)} KB)`);
        await revealItemInDir(dest);
      } catch (err) {
        setBatchSummary(`Image stitch failed: ${String(err)}`);
      }
    },
    [items, settings.image]
  );

  const handleDeblankPdf = useCallback(async (item: QueueItem) => {
    const dest = await save({
      title: "Save PDF Without Blank Pages",
      defaultPath: item.fileName.replace(/\.pdf$/i, "_deblank.pdf"),
      filters: [{ name: "PDF Document", extensions: ["pdf"] }],
    });
    if (!dest) return;

    setBatchSummary("Scanning and removing blank PDF pages...");
    try {
      const [kept, removed, bytes] = await invoke<[number, number, number]>(
        "remove_blank_pdf_pages",
        {
          path: item.path,
          outputPath: dest,
          whiteThreshold: 0.995,
        }
      );
      setBatchSummary(
        `Removed ${removed} blank page(s), kept ${kept} (${(bytes / 1024).toFixed(0)} KB)`
      );
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`Remove blank pages failed: ${String(err)}`);
    }
  }, []);

  const handleEditPdfMeta = useCallback(async (item: QueueItem) => {
    const title = window.prompt("PDF Title (leave blank to skip):", "");
    if (title === null) return;
    const author = window.prompt("PDF Author (leave blank to skip):", "");
    if (author === null) return;

    const dest = await save({
      title: "Save PDF with Updated Metadata",
      defaultPath: item.fileName.replace(/\.pdf$/i, "_meta.pdf"),
      filters: [{ name: "PDF Document", extensions: ["pdf"] }],
    });
    if (!dest) return;

    setBatchSummary("Updating PDF metadata...");
    try {
      const bytes = await invoke<number>("set_pdf_metadata", {
        path: item.path,
        outputPath: dest,
        title: title.trim() || null,
        author: author.trim() || null,
        subject: null,
        keywords: null,
      });
      setBatchSummary(`Updated PDF metadata (${(bytes / 1024).toFixed(0)} KB)`);
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`PDF metadata edit failed: ${String(err)}`);
    }
  }, []);

  const handleExportPdfMarkdown = useCallback(async (item: QueueItem) => {
    const dest = await save({
      title: "Export PDF as Markdown",
      defaultPath: item.fileName.replace(/\.pdf$/i, ".md"),
      filters: [{ name: "Markdown", extensions: ["md"] }],
    });
    if (!dest) return;

    setBatchSummary("Exporting PDF to Markdown...");
    try {
      const [pages, bytes] = await invoke<[number, number]>("export_pdf_markdown_file", {
        path: item.path,
        outputPath: dest,
      });
      setBatchSummary(
        `Exported ${pages} page(s) to Markdown (${(bytes / 1024).toFixed(1)} KB)`
      );
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`PDF Markdown export failed: ${String(err)}`);
    }
  }, []);

  const handleTwoUpPdf = useCallback(async (item: QueueItem) => {
    const dest = await save({
      title: "Save 2-Up Side-by-Side PDF",
      defaultPath: item.fileName.replace(/\.pdf$/i, "_2up.pdf"),
      filters: [{ name: "PDF Document", extensions: ["pdf"] }],
    });
    if (!dest) return;

    setBatchSummary("Creating 2-Up side-by-side PDF...");
    try {
      const [sheets, bytes] = await invoke<[number, number]>("convert_pdf_two_up", {
        path: item.path,
        outputPath: dest,
        dpi: 150,
      });
      setBatchSummary(
        `Saved 2-Up PDF (${sheets} sheet${sheets === 1 ? "" : "s"}, ${(bytes / 1024).toFixed(0)} KB)`
      );
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`2-Up PDF failed: ${String(err)}`);
    }
  }, []);

  const handleDarkModePdf = useCallback(async (item: QueueItem) => {
    const dest = await save({
      title: "Save Dark-Mode PDF",
      defaultPath: item.fileName.replace(/\.pdf$/i, "_dark.pdf"),
      filters: [{ name: "PDF Document", extensions: ["pdf"] }],
    });
    if (!dest) return;

    setBatchSummary("Converting PDF to Dark Mode...");
    try {
      const bytes = await invoke<number>("convert_pdf_dark_mode", {
        path: item.path,
        outputPath: dest,
        dpi: 150,
      });
      setBatchSummary(`Saved Dark-Mode PDF (${(bytes / 1024).toFixed(0)} KB)`);
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`Dark-mode PDF failed: ${String(err)}`);
    }
  }, []);

  const handleExtractPoster = useCallback(async (item: QueueItem) => {
    const dest = await save({
      title: "Save Video Poster Frame",
      defaultPath: item.fileName.replace(/\.[^.]+$/, "_poster.jpg"),
      filters: [{ name: "JPEG Image", extensions: ["jpg", "jpeg"] }],
    });
    if (!dest) return;

    const ts = item.durationSecs && item.durationSecs > 2 ? Math.min(2.0, item.durationSecs * 0.2) : 0.2;
    setBatchSummary("Extracting crisp video poster frame...");
    try {
      const bytes = await invoke<number>("extract_video_poster", {
        path: item.path,
        outputPath: dest,
        timestampSecs: ts,
      });
      setBatchSummary(`Saved video poster (${(bytes / 1024).toFixed(0)} KB)`);
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`Poster extraction failed: ${String(err)}`);
    }
  }, []);

  const handleExtractPdfText = useCallback(async (item: QueueItem) => {
    const dest = await save({
      title: "Save Extracted PDF Text",
      defaultPath: item.fileName.replace(/\.pdf$/i, ".txt"),
      filters: [{ name: "Text File", extensions: ["txt"] }],
    });
    if (!dest) return;

    setBatchSummary("Extracting text from PDF...");
    try {
      const [pages, bytes] = await invoke<[number, number]>("export_pdf_text_file", {
        path: item.path,
        outputPath: dest,
      });
      setBatchSummary(
        `Extracted text from ${pages} page(s) (${(bytes / 1024).toFixed(1)} KB)`
      );
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`PDF text extraction failed: ${String(err)}`);
    }
  }, []);

  const handleScrubPdf = useCallback(async (item: QueueItem) => {
    const dest = await save({
      title: "Save Metadata-Cleaned PDF",
      defaultPath: item.fileName.replace(/\.pdf$/i, "_clean.pdf"),
      filters: [{ name: "PDF Document", extensions: ["pdf"] }],
    });
    if (!dest) return;

    setBatchSummary("Scrubbing PDF metadata...");
    try {
      const bytes = await invoke<number>("scrub_pdf_metadata", {
        path: item.path,
        outputPath: dest,
      });
      setBatchSummary(`Cleaned PDF saved (${(bytes / 1024).toFixed(0)} KB)`);
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`PDF scrub failed: ${String(err)}`);
    }
  }, []);

  const handleSplitMedia = useCallback(async (item: QueueItem) => {
    const secsStr = window.prompt(
      `Segment duration in seconds for ${item.fileName} (e.g. 60):`,
      "60"
    );
    if (!secsStr) return;
    const secs = Number(secsStr);
    if (!Number.isFinite(secs) || secs <= 0) return;

    const dir = await open({
      multiple: false,
      directory: true,
      title: "Select folder for split media chunks",
    });
    if (typeof dir !== "string") return;

    setBatchSummary(`Splitting into ${secs}s chunks...`);
    try {
      const parts = await invoke<string[]>("split_media_segments", {
        path: item.path,
        outputDir: dir,
        segmentSecs: secs,
      });
      setBatchSummary(`Split into ${parts.length} chunk(s)!`);
      if (parts[0]) await revealItemInDir(parts[0]);
    } catch (err) {
      setBatchSummary(`Media split failed: ${String(err)}`);
    }
  }, []);

  const handleExportCsv = useCallback(() => {
    const esc = (v: string | number | undefined) =>
      `"${String(v ?? "").replace(/"/g, '""')}"`;
    const rows = [
      ["File", "Source", "Input Bytes", "Output Bytes", "Savings %", "Duration ms", "Status", "Output Path"].join(","),
      ...items.map((i) => {
        const saved =
          i.outputBytes != null && i.sizeBytes > 0
            ? Math.round((1 - i.outputBytes / i.sizeBytes) * 100)
            : "";
        return [
          esc(i.fileName),
          esc(i.sourceFormat),
          i.sizeBytes,
          i.outputBytes ?? "",
          saved,
          i.durationMs ?? "",
          esc(i.status),
          esc(i.outputPath ?? ""),
        ].join(",");
      }),
    ].join("\n");

    const blob = new Blob([rows], { type: "text/csv;charset=utf-8;" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `wandel-report-${new Date().toISOString().slice(0, 10)}.csv`;
    a.click();
    URL.revokeObjectURL(url);
    setBatchSummary("Exported batch CSV report");
  }, [items]);

  const handleCreateSpriteSheet = useCallback(async () => {
    const imagePaths = items
      .filter((i) => i.mediaKind === "image")
      .map((i) => i.path);
    if (imagePaths.length < 2) return;

    const dest = await save({
      title: "Save Sprite Sheet (+ CSS Map)",
      defaultPath: "wandel_sprites.png",
      filters: [{ name: "Images", extensions: ["png", "webp"] }],
    });
    if (!dest) return;

    setBatchSummary("Packing sprite sheet & CSS map...");
    try {
      const ext = dest.split(".").pop()?.toLowerCase();
      const fmt: ImageFormat = ext === "webp" ? "webp" : "png";
      const res = await invoke<SpriteSheetResult>("create_sprite_sheet", {
        paths: imagePaths,
        outputPath: dest,
        paddingPx: 4,
        settings: { ...settings.image, format: fmt },
      });
      void navigator.clipboard.writeText(res.cssSnippet);
      setBatchSummary(
        `Packed ${res.count} sprites (${res.width}×${res.height}) & copied CSS!`
      );
      await revealItemInDir(res.imagePath);
    } catch (err) {
      setBatchSummary(`Sprite sheet failed: ${String(err)}`);
    }
  }, [items, settings.image]);

  const handleExtractPdfImages = useCallback(async (item: QueueItem) => {
    const dir = await open({
      multiple: false,
      directory: true,
      title: "Select folder for extracted PDF images",
    });
    if (typeof dir !== "string") return;

    setBatchSummary("Extracting embedded images from PDF...");
    try {
      const files = await invoke<string[]>("extract_pdf_images", {
        path: item.path,
        outputDir: dir,
      });
      setBatchSummary(`Extracted ${files.length} embedded image(s)!`);
      if (files[0]) await revealItemInDir(files[0]);
    } catch (err) {
      setBatchSummary(`PDF image extract failed: ${String(err)}`);
    }
  }, []);

  const handlePodcastVideo = useCallback(async (item: QueueItem) => {
    const cover = await open({
      multiple: false,
      directory: false,
      title: "Select cover art image (PNG / JPG / WebP)",
      filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp"] }],
    });
    if (typeof cover !== "string") return;

    const dest = await save({
      title: "Save Audiogram / Podcast MP4",
      defaultPath: item.fileName.replace(/\.[^.]+$/, "_video.mp4"),
      filters: [{ name: "MP4 Video", extensions: ["mp4"] }],
    });
    if (!dest) return;

    setBatchSummary("Creating Cover + Audio MP4 video...");
    try {
      const bytes = await invoke<number>("create_podcast_video", {
        audioPath: item.path,
        coverImage: cover,
        outputPath: dest,
        width: 1280,
        height: 720,
      });
      setBatchSummary(
        `Created MP4 video (${(bytes / (1024 * 1024)).toFixed(1)} MB)`
      );
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`Podcast video failed: ${String(err)}`);
    }
  }, []);

  const handleCreateSlideshow = useCallback(async () => {
    const imagePaths = items
      .filter((i) => i.mediaKind === "image")
      .map((i) => i.path);
    if (imagePaths.length < 2) return;

    const secsStr = window.prompt("Seconds per slide (0.5 - 30):", "3");
    if (!secsStr) return;
    const secs = Number(secsStr);
    if (!Number.isFinite(secs) || secs <= 0) return;

    const dest = await save({
      title: "Save Slideshow MP4",
      defaultPath: "wandel_slideshow.mp4",
      filters: [{ name: "MP4 Video", extensions: ["mp4"] }],
    });
    if (!dest) return;

    setBatchSummary(`Creating slideshow (${imagePaths.length} slides)...`);
    try {
      const bytes = await invoke<number>("create_slideshow_video", {
        paths: imagePaths,
        outputPath: dest,
        secsPerSlide: secs,
        width: 1920,
        height: 1080,
        audioPath: settings.video.replaceAudioPath,
      });
      setBatchSummary(
        `Created slideshow (${(bytes / (1024 * 1024)).toFixed(1)} MB)`
      );
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`Slideshow failed: ${String(err)}`);
    }
  }, [items, settings.video.replaceAudioPath]);

  const handleSliceImage = useCallback(
    async (item: QueueItem) => {
      const spec = window.prompt(
        `Enter grid dimensions (cols x rows) to slice ${item.fileName} (e.g. 3x1 or 3x3):`,
        "3x1"
      );
      if (!spec) return;
      const parts = spec.toLowerCase().split(/[x×,]/).map((s) => Number(s.trim()));
      const cols = parts[0];
      const rows = parts[1] ?? 1;
      if (!Number.isFinite(cols) || !Number.isFinite(rows) || cols < 1 || rows < 1) return;

      const dir = await open({
        multiple: false,
        directory: true,
        title: "Select folder for sliced image tiles",
      });
      if (typeof dir !== "string") return;

      setBatchSummary(`Slicing image into ${cols}×${rows} tiles...`);
      try {
        const files = await invoke<string[]>("split_image_grid", {
          path: item.path,
          outputDir: dir,
          cols,
          rows,
          settings: item.overrideImageFormat
            ? { ...settings.image, format: item.overrideImageFormat }
            : settings.image,
        });
        setBatchSummary(`Sliced into ${files.length} tiles!`);
        if (files[0]) await revealItemInDir(files[0]);
      } catch (err) {
        setBatchSummary(`Image slice failed: ${String(err)}`);
      }
    },
    [settings.image]
  );

  const handleNumberPdf = useCallback(async (item: QueueItem) => {
    const fmt = window.prompt(
      `Page number format for ${item.fileName} ({n} = page, {total} = total pages):`,
      "{n} / {total}"
    );
    if (!fmt || !fmt.trim()) return;

    const dest = await save({
      title: "Save Numbered PDF",
      defaultPath: item.fileName.replace(/\.pdf$/i, "_numbered.pdf"),
      filters: [{ name: "PDF Document", extensions: ["pdf"] }],
    });
    if (!dest) return;

    setBatchSummary("Adding page numbers to PDF...");
    try {
      const bytes = await invoke<number>("number_pdf_pages", {
        path: item.path,
        outputPath: dest,
        formatTemplate: fmt.trim(),
      });
      setBatchSummary(`Numbered PDF saved (${(bytes / 1024).toFixed(0)} KB)`);
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`PDF numbering failed: ${String(err)}`);
    }
  }, []);

  const handleStampPdf = useCallback(async (item: QueueItem) => {
    const text = window.prompt(
      `Enter watermark stamp text for ${item.fileName}:`,
      "CONFIDENTIAL"
    );
    if (!text || !text.trim()) return;

    const dest = await save({
      title: "Save Stamped PDF",
      defaultPath: item.fileName.replace(/\.pdf$/i, "_stamped.pdf"),
      filters: [{ name: "PDF Document", extensions: ["pdf"] }],
    });
    if (!dest) return;

    setBatchSummary(`Stamping PDF ("${text.trim()}")...`);
    try {
      const bytes = await invoke<number>("stamp_pdf_text", {
        path: item.path,
        outputPath: dest,
        text: text.trim(),
      });
      setBatchSummary(`Stamped PDF saved (${(bytes / 1024).toFixed(0)} KB)`);
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`PDF stamp failed: ${String(err)}`);
    }
  }, []);

  const handlePixelDiff = useCallback(async () => {
    const imagePaths = items
      .filter((i) => i.mediaKind === "image")
      .map((i) => i.path);
    if (imagePaths.length < 2) return;

    const dest = await save({
      title: "Save Visual Pixel Diff PNG",
      defaultPath: "wandel_diff.png",
      filters: [{ name: "PNG Image", extensions: ["png"] }],
    });
    if (!dest) return;

    setBatchSummary("Computing visual pixel diff...");
    try {
      const res = await invoke<{
        outputPath: string;
        width: number;
        height: number;
        changedPixels: number;
        totalPixels: number;
        diffPercent: number;
      }>("create_pixel_diff", {
        firstPath: imagePaths[0],
        secondPath: imagePaths[1],
        outputPath: dest,
        threshold: 10,
      });
      setBatchSummary(
        `Pixel diff: ${res.changedPixels.toLocaleString()} px changed (${res.diffPercent.toFixed(2)}%)`
      );
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`Pixel diff failed: ${String(err)}`);
    }
  }, [items]);

  const handleInterleavePdfs = useCallback(async () => {
    const pdfPaths = items
      .filter((i) => i.mediaKind === "pdf")
      .map((i) => i.path);
    if (pdfPaths.length < 2) return;

    const reverseEven = window.confirm(
      "Reverse the even pages PDF? Click OK for standard single-sided duplex scans, or Cancel to keep even pages in forward order."
    );
    const dest = await save({
      title: "Save Interleaved Duplex PDF",
      defaultPath: "wandel_interleaved.pdf",
      filters: [{ name: "PDF Document", extensions: ["pdf"] }],
    });
    if (!dest) return;

    setBatchSummary("Interleaving odd and even PDFs...");
    try {
      const [pages, bytes] = await invoke<[number, number]>("interleave_pdfs", {
        oddPath: pdfPaths[0],
        evenPath: pdfPaths[1],
        reverseEven,
        outputPath: dest,
      });
      setBatchSummary(
        `Interleaved ${pages} page(s) (${(bytes / 1024).toFixed(0)} KB)`
      );
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`PDF interleave failed: ${String(err)}`);
    }
  }, [items]);

  const handleCompareImages = useCallback(async () => {
    const imagePaths = items
      .filter((i) => i.mediaKind === "image")
      .map((i) => i.path);
    if (imagePaths.length < 2) return;

    const dest = await save({
      title: "Save Before/After Comparison Collage",
      defaultPath: "wandel_compare.png",
      filters: [{ name: "Images", extensions: ["png", "jpg", "webp"] }],
    });
    if (!dest) return;

    setBatchSummary("Creating Before/After comparison collage...");
    try {
      const ext = dest.split(".").pop()?.toLowerCase();
      const fmt: ImageFormat =
        ext === "jpg" || ext === "jpeg"
          ? "jpg"
          : ext === "webp"
          ? "webp"
          : "png";
      const bytes = await invoke<number>("create_comparison_image", {
        firstPath: imagePaths[0],
        secondPath: imagePaths[1],
        outputPath: dest,
        options: {
          vertical: false,
          gapPx: 8,
          backgroundHex: "#18181b",
          leftLabel: "Before",
          rightLabel: "After",
        },
        settings: { ...settings.image, format: fmt },
      });
      setBatchSummary(`Created comparison collage (${(bytes / 1024).toFixed(0)} KB)`);
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`Compare failed: ${String(err)}`);
    }
  }, [items, settings.image]);

  const handleExtractFrames = useCallback(async (item: QueueItem) => {
    const fpsStr = window.prompt(
      `Frames per second to extract from ${item.fileName} (e.g. 1, 2, or 0.5):`,
      "1"
    );
    if (!fpsStr) return;
    const fps = Number(fpsStr);
    if (!Number.isFinite(fps) || fps <= 0) return;

    const dir = await open({
      multiple: false,
      directory: true,
      title: "Select folder for extracted video frames",
    });
    if (typeof dir !== "string") return;

    setBatchSummary(`Extracting frames at ${fps} fps...`);
    try {
      const files = await invoke<string[]>("extract_video_frames", {
        path: item.path,
        outputDir: dir,
        fps,
        formatExt: "png",
      });
      setBatchSummary(`Extracted ${files.length} frame(s)!`);
      if (files[0]) await revealItemInDir(files[0]);
    } catch (err) {
      setBatchSummary(`Frame extraction failed: ${String(err)}`);
    }
  }, []);

  const handleAudioWaveform = useCallback(async (item: QueueItem) => {
    const dest = await save({
      title: "Save Audio Waveform PNG",
      defaultPath: item.fileName.replace(/\.[^.]+$/, "_waveform.png"),
      filters: [{ name: "PNG Image", extensions: ["png"] }],
    });
    if (!dest) return;

    setBatchSummary("Generating audio waveform PNG...");
    try {
      const bytes = await invoke<number>("create_audio_waveform", {
        path: item.path,
        outputPath: dest,
        width: 1200,
        height: 320,
        colorHex: "#6366f1",
        bgHex: "#18181b",
      });
      setBatchSummary(`Generated waveform PNG (${(bytes / 1024).toFixed(0)} KB)`);
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`Waveform failed: ${String(err)}`);
    }
  }, []);

  const handleCreateGrid = useCallback(async () => {
    const imagePaths = items
      .filter((i) => i.mediaKind === "image")
      .map((i) => i.path);
    if (imagePaths.length < 2) return;

    const dest = await save({
      title: "Save Contact Sheet / Grid Collage",
      defaultPath: "wandel_grid.png",
      filters: [
        { name: "Images", extensions: ["png", "jpg", "webp"] },
      ],
    });
    if (!dest) return;

    setBatchSummary("Creating contact sheet grid...");
    try {
      const ext = dest.split(".").pop()?.toLowerCase();
      const fmt: ImageFormat =
        ext === "jpg" || ext === "jpeg"
          ? "jpg"
          : ext === "webp"
          ? "webp"
          : "png";
      const bytes = await invoke<number>("create_contact_sheet", {
        paths: imagePaths,
        outputPath: dest,
        options: {
          columns: 0,
          cellSize: 400,
          gapPx: 16,
          backgroundHex: settings.image.backgroundHex ?? "#18181b",
        },
        settings: { ...settings.image, format: fmt },
      });
      setBatchSummary(
        `Created grid (${imagePaths.length} images, ${(bytes / 1024).toFixed(0)} KB)`
      );
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`Grid failed: ${String(err)}`);
    }
  }, [items, settings.image]);

  const handleMergeVideos = useCallback(async () => {
    const videoPaths = items
      .filter((i) => i.mediaKind === "video")
      .map((i) => i.path);
    if (videoPaths.length < 2) return;

    const dest = await save({
      title: "Save Merged Video",
      defaultPath: "wandel_merged.mp4",
      filters: [{ name: "MP4 Video", extensions: ["mp4"] }],
    });
    if (!dest) return;

    setBatchSummary("Merging videos...");
    try {
      const bytes = await invoke<number>("merge_videos", {
        paths: videoPaths,
        outputPath: dest,
        settings: settings.video,
      });
      setBatchSummary(
        `Merged ${videoPaths.length} videos (${(bytes / (1024 * 1024)).toFixed(1)} MB)`
      );
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`Video merge failed: ${String(err)}`);
    }
  }, [items, settings.video]);

  const handleMergePdfs = useCallback(async () => {
    const pdfPaths = items
      .filter((i) => i.mediaKind === "pdf")
      .map((i) => i.path);
    if (pdfPaths.length < 2) return;

    const dest = await save({
      title: "Save Merged PDF",
      defaultPath: "wandel_combined.pdf",
      filters: [{ name: "PDF Document", extensions: ["pdf"] }],
    });
    if (!dest) return;

    setBatchSummary("Merging PDFs...");
    try {
      const bytes = await invoke<number>("merge_pdfs", {
        paths: pdfPaths,
        outputPath: dest,
      });
      setBatchSummary(
        `Merged ${pdfPaths.length} PDFs (${(bytes / 1024).toFixed(0)} KB)`
      );
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`PDF merge failed: ${String(err)}`);
    }
  }, [items]);

  const handleRotatePdf = useCallback(
    async (item: QueueItem) => {
      const dest = await save({
        title: `Save Rotated PDF (90° CW)`,
        defaultPath: item.fileName.replace(/\.pdf$/i, "_rotated.pdf"),
        filters: [{ name: "PDF Document", extensions: ["pdf"] }],
      });
      if (!dest) return;

      const pageSpec = settings.pdf.pageRange.trim() || null;
      setBatchSummary("Rotating PDF pages 90°...");
      try {
        const bytes = await invoke<number>("rotate_pdf_pages", {
          path: item.path,
          outputPath: dest,
          degrees: 90,
          pageSpec,
        });
        setBatchSummary(`Rotated PDF saved (${(bytes / 1024).toFixed(0)} KB)`);
        await revealItemInDir(dest);
      } catch (err) {
        setBatchSummary(`PDF rotate failed: ${String(err)}`);
      }
    },
    [settings.pdf.pageRange]
  );

  const handleReorderPdf = useCallback(
    async (item: QueueItem) => {
      const defaultSpec =
        settings.pdf.pageRange.trim() ||
        (item.pageCount && item.pageCount > 1 ? `${item.pageCount}-1` : "1");
      const spec = window.prompt(
        `Enter ordered page list for ${item.fileName} (e.g. "3,1-2" or "2-5"):`,
        defaultSpec
      );
      if (!spec || !spec.trim()) return;

      const dest = await save({
        title: `Save Reordered PDF`,
        defaultPath: item.fileName.replace(/\.pdf$/i, "_reordered.pdf"),
        filters: [{ name: "PDF Document", extensions: ["pdf"] }],
      });
      if (!dest) return;

      setBatchSummary(`Reordering PDF pages (${spec.trim()})...`);
      try {
        const bytes = await invoke<number>("reorder_pdf_pages", {
          path: item.path,
          outputPath: dest,
          pageSpec: spec.trim(),
        });
        setBatchSummary(`Reordered PDF saved (${(bytes / 1024).toFixed(0)} KB)`);
        await revealItemInDir(dest);
      } catch (err) {
        setBatchSummary(`PDF reorder failed: ${String(err)}`);
      }
    },
    [settings.pdf.pageRange]
  );

  const handleSplitPdf = useCallback(
    async (item: QueueItem) => {
      const range = settings.pdf.pageRange.trim() || "1";
      const dest = await save({
        title: `Extract Pages (${range}) from ${item.fileName}`,
        defaultPath: item.fileName.replace(/\.pdf$/i, `_pages_${range.replace(/[^0-9a-zA-Z_-]/g, "_")}.pdf`),
        filters: [{ name: "PDF Document", extensions: ["pdf"] }],
      });
      if (!dest) return;

      setBatchSummary(`Extracting pages (${range})...`);
      try {
        const bytes = await invoke<number>("split_pdf", {
          path: item.path,
          pageRange: range,
          outputPath: dest,
        });
        setBatchSummary(`Split PDF saved (${(bytes / 1024).toFixed(0)} KB)`);
        await revealItemInDir(dest);
      } catch (err) {
        setBatchSummary(`PDF split failed: ${String(err)}`);
      }
    },
    [settings.pdf.pageRange]
  );

  const handleFindDuplicates = useCallback(async () => {
    const imgPaths = items
      .filter((i) => i.mediaKind === "image")
      .map((i) => i.path);
    if (imgPaths.length < 2) return;

    setBatchSummary("Scanning for duplicate images...");
    try {
      const groups = await invoke<DuplicateGroup[]>("find_duplicate_images", {
        paths: imgPaths,
        maxHamming: 5,
      });
      const dupPaths = new Set(groups.flatMap((g) => g.duplicates));
      if (dupPaths.size === 0) {
        setBatchSummary("No duplicate images found!");
      } else {
        setItems((prev) => prev.filter((i) => !dupPaths.has(i.path)));
        setBatchSummary(
          `Removed ${dupPaths.size} duplicate image(s) from queue`
        );
      }
    } catch (err) {
      setBatchSummary(`Dedupe failed: ${String(err)}`);
    }
  }, [items]);

  const handleCompressPdf = useCallback(
    async (item: QueueItem) => {
      const dest = await save({
        title: `Save Compressed PDF`,
        defaultPath: item.fileName.replace(/\.pdf$/i, "_compressed.pdf"),
        filters: [{ name: "PDF Document", extensions: ["pdf"] }],
      });
      if (!dest) return;

      setBatchSummary("Compressing PDF...");
      try {
        const bytes = await invoke<number>("compress_pdf", {
          path: item.path,
          outputPath: dest,
          dpi: settings.pdf.dpi,
          quality: settings.image.quality,
        });
        const pct =
          item.sizeBytes > 0
            ? Math.round((1 - bytes / item.sizeBytes) * 100)
            : 0;
        setBatchSummary(
          `Compressed PDF (${(bytes / 1024).toFixed(0)} KB${pct > 0 ? `, -${pct}%` : ""})`
        );
        await revealItemInDir(dest);
      } catch (err) {
        setBatchSummary(`PDF compress failed: ${String(err)}`);
      }
    },
    [settings.pdf.dpi, settings.image.quality]
  );

  const handleMergeAudio = useCallback(async () => {
    const audioPaths = items
      .filter((i) => i.mediaKind === "audio")
      .map((i) => i.path);
    if (audioPaths.length < 2) return;

    const dest = await save({
      title: "Save Merged Audio",
      defaultPath: "wandel_merged.mp3",
      filters: [{ name: "Audio", extensions: ["mp3", "m4a", "ogg", "wav", "flac"] }],
    });
    if (!dest) return;

    setBatchSummary("Merging audio tracks...");
    try {
      const bytes = await invoke<number>("merge_audio", {
        paths: audioPaths,
        outputPath: dest,
        settings: settings.video,
      });
      setBatchSummary(
        `Merged ${audioPaths.length} audio tracks (${(bytes / 1024).toFixed(0)} KB)`
      );
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`Audio merge failed: ${String(err)}`);
    }
  }, [items, settings.video]);

  const handleCreateStoryboard = useCallback(async (item: QueueItem) => {
    const dest = await save({
      title: `Save 3×3 Video Storyboard`,
      defaultPath: item.fileName.replace(/\.[^.]+$/, "_storyboard.png"),
      filters: [{ name: "Images", extensions: ["png", "jpg", "webp"] }],
    });
    if (!dest) return;

    setBatchSummary("Generating 3×3 video storyboard...");
    try {
      const bytes = await invoke<number>("create_video_storyboard", {
        path: item.path,
        outputPath: dest,
        frameCount: 9,
        columns: 3,
      });
      setBatchSummary(
        `Created video storyboard (${(bytes / 1024).toFixed(0)} KB)`
      );
      await revealItemInDir(dest);
    } catch (err) {
      setBatchSummary(`Storyboard failed: ${String(err)}`);
    }
  }, []);

  const handleRequeueOutputs = useCallback(async () => {
    const outputPaths = items
      .filter((i) => i.status === "completed" && i.outputPath)
      .map((i) => i.outputPath as string);
    if (outputPaths.length === 0) return;
    setItems([]);
    await addPaths(outputPaths);
    setBatchSummary(`Loaded ${outputPaths.length} output file(s) into queue`);
  }, [items, addPaths]);

  const handleGenerateResponsiveSet = useCallback(
    async (item: QueueItem) => {
      const dir = await open({
        multiple: false,
        directory: true,
        title: "Select folder for Responsive Web srcset bundle",
      });
      if (typeof dir !== "string") return;
      setBatchSummary("Generating responsive srcset bundle...");
      try {
        const res = await invoke<ResponsiveSetResult>("generate_responsive_set", {
          path: item.path,
          outputDir: dir,
          settings: item.overrideImageFormat
            ? { ...settings.image, format: item.overrideImageFormat }
            : settings.image,
        });
        void navigator.clipboard.writeText(res.htmlSnippet);
        setBatchSummary(
          `Generated ${res.files.length} srcset files & copied HTML snippet!`
        );
        if (res.files[0]) await revealItemInDir(res.files[0]);
      } catch (err) {
        setBatchSummary(`Responsive set failed: ${String(err)}`);
      }
    },
    [settings.image]
  );

  const handleResetSettings = useCallback(() => {
    setSettings(DEFAULT_SETTINGS);
    setActivePreset(null);
    setBatchSummary("Settings reset to defaults");
  }, []);

  const handlePickSubtitle = useCallback(async () => {
    const selected = await open({
      multiple: false,
      directory: false,
      title: "Select subtitle file (.srt / .vtt)",
      filters: [{ name: "Subtitles", extensions: ["srt", "vtt", "ass"] }],
    });
    if (typeof selected === "string") {
      setSettings((prev) => ({
        ...prev,
        video: { ...prev.video, subtitlePath: selected },
      }));
    }
  }, []);

  const handlePickReplaceAudio = useCallback(async () => {
    const selected = await open({
      multiple: false,
      directory: false,
      title: "Select audio track to attach/replace",
      filters: [
        { name: "Audio", extensions: ["mp3", "wav", "aac", "m4a", "flac", "ogg"] },
      ],
    });
    if (typeof selected === "string") {
      setSettings((prev) => ({
        ...prev,
        video: { ...prev.video, replaceAudioPath: selected },
      }));
    }
  }, []);

  const handleGenerateIconPack = useCallback(async (item: QueueItem) => {
    const dir = await open({
      multiple: false,
      directory: true,
      title: "Select folder for Favicon & App Icon Pack",
    });
    if (typeof dir !== "string") return;
    setBatchSummary("Generating Favicon & App Icon Pack...");
    try {
      const files = await invoke<string[]>("generate_icon_pack", {
        path: item.path,
        outputDir: dir,
      });
      setBatchSummary(`Generated ${files.length} icon pack files!`);
      if (files[0]) await revealItemInDir(files[0]);
    } catch (err) {
      setBatchSummary(`Icon pack failed: ${String(err)}`);
    }
  }, []);

  const handleApplyPreset = useCallback(async (preset: Preset) => {
    try {
      const presetSettings = await invoke<ConversionSettings>(
        "get_preset_settings",
        { preset }
      );
      setSettings((prev) => ({
        ...presetSettings,
        output: prev.output,
        maxCpuThreads: prev.maxCpuThreads,
        maxConcurrentVideos: prev.maxConcurrentVideos,
      }));
      setActivePreset(preset);
    } catch (err) {
      console.error("Failed to apply preset:", err);
    }
  }, []);

  const handleSettingsChange = useCallback((next: ConversionSettings) => {
    setSettings(next);
    setActivePreset(null);
  }, []);

  const handleSaveCustomPreset = useCallback(
    (name: string) => {
      const preset: CustomPreset = {
        id: `custom-${Date.now()}`,
        name,
        settings,
      };
      setCustomPresets((prev) => {
        const next = [...prev, preset];
        localStorage.setItem(CUSTOM_PRESETS_KEY, JSON.stringify(next));
        return next;
      });
      setActivePreset(preset.id);
    },
    [settings]
  );

  const handleApplyCustomPreset = useCallback((preset: CustomPreset) => {
    setSettings((prev) => ({
      ...preset.settings,
      output: prev.output,
      maxCpuThreads: prev.maxCpuThreads,
      maxConcurrentVideos: prev.maxConcurrentVideos,
    }));
    setActivePreset(preset.id);
  }, []);

  const handleDeleteCustomPreset = useCallback((id: string) => {
    setCustomPresets((prev) => {
      const next = prev.filter((p) => p.id !== id);
      localStorage.setItem(CUSTOM_PRESETS_KEY, JSON.stringify(next));
      return next;
    });
    setActivePreset((curr) => (curr === id ? null : curr));
  }, []);

  const handleExportPresets = useCallback(() => {
    const blob = new Blob([JSON.stringify(customPresets, null, 2)], {
      type: "application/json",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "wandel-presets.json";
    a.click();
    URL.revokeObjectURL(url);
  }, [customPresets]);

  const handleImportPresets = useCallback(() => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json,application/json";
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return;
      try {
        const parsed = JSON.parse(await file.text()) as CustomPreset[];
        if (!Array.isArray(parsed)) return;
        setCustomPresets((prev) => {
          const merged = [...prev];
          const existingIds = new Set(prev.map((p) => p.id));
          for (const p of parsed) {
            if (p?.name && p?.settings && !existingIds.has(p.id)) {
              merged.push(p);
            }
          }
          localStorage.setItem(CUSTOM_PRESETS_KEY, JSON.stringify(merged));
          return merged;
        });
        setBatchSummary(`Imported ${parsed.length} preset(s)`);
      } catch (err) {
        console.error("Failed to import presets:", err);
      }
    };
    input.click();
  }, []);

  const handleWandel = useCallback(async () => {
    if (items.length === 0 || isConverting) return;

    setIsConverting(true);
    setBatchSummary(null);

    // Reset statuses for a fresh run
    setItems((prev) =>
      prev.map((item) => ({
        ...item,
        status: "pending",
        progress: 0,
        error: undefined,
      }))
    );

    const channel = new Channel<ProgressEvent>();
    channel.onmessage = (msg) => {
      switch (msg.event) {
        case "fileStarted":
          setItems((prev) =>
            prev.map((i) =>
              i.id === msg.data.id
                ? {
                    ...i,
                    status: "converting",
                    progress: 5,
                    outputPath: msg.data.outputPath,
                  }
                : i
            )
          );
          break;
        case "fileProgress":
          setItems((prev) =>
            prev.map((i) =>
              i.id === msg.data.id ? { ...i, progress: msg.data.percent } : i
            )
          );
          break;
        case "fileCompleted": {
          const matched = items.find((i) => i.id === msg.data.id);
          if (matched) {
            recordHistory({
              fileName: matched.fileName,
              outputPath: msg.data.outputPath,
              inputBytes: matched.sizeBytes,
              outputBytes: msg.data.outputBytes,
              source: "manual",
            });
          }
          setItems((prev) =>
            prev.map((i) =>
              i.id === msg.data.id
                ? {
                    ...i,
                    status: "completed",
                    progress: 100,
                    outputPath: msg.data.outputPath,
                    outputBytes: msg.data.outputBytes,
                    durationMs: msg.data.durationMs,
                  }
                : i
            )
          );
          break;
        }
        case "fileFailed":
          setItems((prev) =>
            prev.map((i) =>
              i.id === msg.data.id
                ? { ...i, status: "failed", error: msg.data.error }
                : i
            )
          );
          break;
        case "fileCancelled":
          setItems((prev) =>
            prev.map((i) =>
              i.id === msg.data.id ? { ...i, status: "cancelled" } : i
            )
          );
          break;
        case "batchFinished": {
          const secs = (msg.data.totalDurationMs / 1000).toFixed(2);
          const parts = [`${msg.data.completed} converted`];
          if (msg.data.failed > 0) parts.push(`${msg.data.failed} failed`);
          if (msg.data.cancelled > 0)
            parts.push(`${msg.data.cancelled} cancelled`);
          setBatchSummary(`${parts.join(" · ")} in ${secs}s`);
          break;
        }
      }
    };

    try {
      const jobs = items.map((item) => ({
        id: item.id,
        path: item.path,
        mediaKind: item.mediaKind,
        durationSecs: item.durationSecs,
        width: item.width,
        height: item.height,
        relativeDir: item.relativeDir,
        imageOverride: item.overrideImageFormat
          ? { ...settings.image, format: item.overrideImageFormat }
          : null,
        videoOverride: item.overrideVideoTarget
          ? { ...settings.video, target: item.overrideVideoTarget }
          : null,
      }));
      await invoke("start_wandel", {
        jobs,
        settings,
        onEvent: channel,
      });
    } catch (err) {
      console.error("Conversion error:", err);
    } finally {
      setIsConverting(false);
    }
  }, [items, isConverting, settings, recordHistory]);

  const handleCancel = useCallback(async () => {
    try {
      await invoke("cancel_wandel");
    } catch (err) {
      console.error("Cancel error:", err);
    }
  }, []);

  const handleMoveItem = useCallback((index: number, direction: -1 | 1) => {
    setItems((prev) => {
      const target = index + direction;
      if (target < 0 || target >= prev.length) return prev;
      const next = [...prev];
      const [moved] = next.splice(index, 1);
      next.splice(target, 0, moved);
      return next;
    });
  }, []);

  const handleMergePdf = useCallback(
    async (pageFit: PdfPageFit) => {
      const imagePaths = items
        .filter((i) => i.mediaKind === "image")
        .map((i) => i.path);
      if (imagePaths.length < 2) return;

      const dest = await save({
        title: "Save Merged PDF",
        defaultPath: "wandel_merged.pdf",
        filters: [{ name: "PDF Document", extensions: ["pdf"] }],
      });
      if (!dest) return;

      setBatchSummary("Merging images to PDF...");
      try {
        const bytes = await invoke<number>("merge_images_to_pdf", {
          paths: imagePaths,
          outputPath: dest,
          settings: settings.image,
          pageFit,
        });
        setBatchSummary(
          `Merged ${imagePaths.length} images into PDF (${(bytes / 1024).toFixed(0)} KB)`
        );
        await revealItemInDir(dest);
      } catch (err) {
        setBatchSummary(`PDF merge failed: ${String(err)}`);
      }
    },
    [items, settings.image]
  );

  const handleRetryFailed = useCallback(async () => {
    const failedItems = items.filter((i) => i.status === "failed");
    if (failedItems.length === 0 || isConverting) return;
    setIsConverting(true);
    setBatchSummary(null);

    const failedIds = new Set(failedItems.map((i) => i.id));
    setItems((prev) =>
      prev.map((item) =>
        failedIds.has(item.id)
          ? { ...item, status: "pending", progress: 0, error: undefined }
          : item
      )
    );

    const channel = new Channel<ProgressEvent>();
    channel.onmessage = (msg) => {
      if (msg.event === "fileStarted") {
        setItems((prev) =>
          prev.map((i) =>
            i.id === msg.data.id
              ? { ...i, status: "converting", progress: 5, outputPath: msg.data.outputPath }
              : i
          )
        );
      } else if (msg.event === "fileProgress") {
        setItems((prev) =>
          prev.map((i) =>
            i.id === msg.data.id ? { ...i, progress: msg.data.percent } : i
          )
        );
      } else if (msg.event === "fileCompleted") {
        setItems((prev) =>
          prev.map((i) =>
            i.id === msg.data.id
              ? {
                  ...i,
                  status: "completed",
                  progress: 100,
                  outputPath: msg.data.outputPath,
                  outputBytes: msg.data.outputBytes,
                  durationMs: msg.data.durationMs,
                }
              : i
          )
        );
      } else if (msg.event === "fileFailed") {
        setItems((prev) =>
          prev.map((i) =>
            i.id === msg.data.id
              ? { ...i, status: "failed", error: msg.data.error }
              : i
          )
        );
      } else if (msg.event === "batchFinished") {
        setBatchSummary(
          `Retry: ${msg.data.completed} converted · ${msg.data.failed} failed`
        );
      }
    };

    try {
      const jobs = failedItems.map((item) => ({
        id: item.id,
        path: item.path,
        mediaKind: item.mediaKind,
        durationSecs: item.durationSecs,
        width: item.width,
        height: item.height,
        relativeDir: item.relativeDir,
        imageOverride: item.overrideImageFormat
          ? { ...settings.image, format: item.overrideImageFormat }
          : null,
        videoOverride: item.overrideVideoTarget
          ? { ...settings.video, target: item.overrideVideoTarget }
          : null,
      }));
      await invoke("start_wandel", { jobs, settings, onEvent: channel });
    } finally {
      setIsConverting(false);
    }
  }, [items, isConverting, settings]);

  // Keyboard shortcuts: Cmd/Ctrl + K Command Palette, Cmd/Ctrl + Enter to Wandel, Esc to cancel, Cmd/Ctrl + V to paste images
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setShowCommandPalette((v) => !v);
      } else if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
        e.preventDefault();
        void handleWandel();
      } else if (
        e.key === "?" &&
        !(e.target instanceof HTMLInputElement) &&
        !(e.target instanceof HTMLTextAreaElement)
      ) {
        e.preventDefault();
        setShowShortcuts((v) => !v);
      } else if (e.key === "Escape" && isConverting) {
        void handleCancel();
      }
    };

    const onPaste = async (e: ClipboardEvent) => {
      if (isConverting || !e.clipboardData) return;
      const imageItems = Array.from(e.clipboardData.items).filter((item) =>
        item.type.startsWith("image/")
      );
      if (imageItems.length === 0) return;

      e.preventDefault();
      setIsInspecting(true);
      try {
        for (const clipItem of imageItems) {
          const file = clipItem.getAsFile();
          if (!file) continue;
          const buf = new Uint8Array(await file.arrayBuffer());
          const ext = clipItem.type.split("/")[1]?.replace("jpeg", "jpg") || "png";
          const inspected = await invoke<FileMetadata[]>("import_clipboard_image", {
            bytes: Array.from(buf),
            ext,
          });
          setItems((prev) => {
            const existing = new Set(prev.map((i) => i.id));
            const additions: QueueItem[] = inspected
              .filter((meta) => !existing.has(meta.id))
              .map((meta) => ({
                ...meta,
                status: "pending",
                progress: 0,
              }));
            return [...prev, ...additions];
          });
        }
      } catch (err) {
        console.error("Clipboard paste failed:", err);
      } finally {
        setIsInspecting(false);
      }
    };

    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("paste", onPaste);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("paste", onPaste);
    };
  }, [handleWandel, handleCancel, isConverting]);

  const imageCount = useMemo(
    () =>
      items.filter((i) => i.mediaKind === "image" || i.mediaKind === "pdf")
        .length,
    [items]
  );
  const pdfCount = useMemo(
    () => items.filter((i) => i.mediaKind === "pdf").length,
    [items]
  );
  const videoCount = useMemo(
    () =>
      items.filter((i) => i.mediaKind === "video" || i.mediaKind === "audio")
        .length,
    [items]
  );

  const overallProgress = useMemo(() => {
    if (items.length === 0) return 0;
    const sum = items.reduce((acc, i) => {
      if (i.status === "completed") return acc + 100;
      if (i.status === "converting") return acc + i.progress;
      return acc;
    }, 0);
    return sum / items.length;
  }, [items]);

  const handleRevealFile = useCallback(async (path: string) => {
    try {
      await revealItemInDir(path);
    } catch (err) {
      console.error("Reveal error:", err);
    }
  }, []);

  const toggleLocale = useCallback(() => {
    setLocale((prev) => {
      const next: Locale = prev === "en" ? "de" : "en";
      localStorage.setItem("wandel-locale", next);
      return next;
    });
  }, []);

  const paletteActions: PaletteAction[] = useMemo(
    () => [
      {
        id: "wandel",
        label: `Wandel (${items.length} file${items.length === 1 ? "" : "s"})`,
        shortcut: "⌘↵",
        run: () => void handleWandel(),
      },
      {
        id: "add-files",
        label: t(locale, "addFiles"),
        run: () => void handleAddFiles(),
      },
      {
        id: "add-folder",
        label: t(locale, "addFolder"),
        run: () => void handleAddFolder(),
      },
      {
        id: "preset-web",
        label: "Apply Preset: Web Optimized",
        run: () => void handleApplyPreset("web_optimized"),
      },
      {
        id: "preset-hq",
        label: "Apply Preset: High Quality",
        run: () => void handleApplyPreset("high_quality"),
      },
      {
        id: "preset-small",
        label: "Apply Preset: Small Size",
        run: () => void handleApplyPreset("small_size"),
      },
      ...customPresets.map((cp) => ({
        id: `custom-preset-${cp.id}`,
        label: `Apply Custom Preset: ${cp.name}`,
        run: () => handleApplyCustomPreset(cp),
      })),
      {
        id: "qr",
        label: "Open QR Code Generator",
        run: () => setShowQrModal(true),
      },
      {
        id: "mcp",
        label: "Open MCP Server Integration (AI Agents)",
        run: () => setShowMcpModal(true),
      },
      {
        id: "pipeline",
        label: "Open Composable Pipeline Runner",
        run: () => setShowPipelineModal(true),
      },
      {
        id: "shortcuts",
        label: "Show Keyboard Shortcuts & About",
        shortcut: "?",
        run: () => setShowShortcuts(true),
      },
      {
        id: "watch",
        label: "Open Watch Folders (Auto-Wandel)",
        run: () => setShowWatchModal(true),
      },
      {
        id: "history",
        label: "Open Conversion History",
        run: () => setShowHistory(true),
      },
      {
        id: "theme",
        label: darkMode ? "Switch to Light Mode" : "Switch to Dark Mode",
        run: () => setDarkMode((v) => !v),
      },
      {
        id: "lang",
        label: `Switch Language (${locale === "en" ? "Deutsch" : "English"})`,
        run: toggleLocale,
      },
    ],
    [
      items.length,
      locale,
      darkMode,
      handleWandel,
      handleAddFiles,
      handleAddFolder,
      handleApplyPreset,
      customPresets,
      handleApplyCustomPreset,
      toggleLocale,
    ]
  );

  return (
    <div className="relative flex h-full flex-col bg-zinc-50 text-zinc-900 dark:bg-zinc-950 dark:text-zinc-100">
      {/* Top Header */}
      <header className="flex h-14 shrink-0 items-center justify-between border-b border-zinc-200 bg-white/80 px-5 backdrop-blur-md dark:border-zinc-800/80 dark:bg-zinc-950/80">
        <div className="flex items-center gap-2.5">
          <WandelIcon className="h-8 w-8 shrink-0 drop-shadow-sm" />
          <div>
            <h1 className="text-sm font-bold tracking-tight">Wandel</h1>
            <p className="text-[11px] text-zinc-500 dark:text-zinc-400">
              {t(locale, "subtitle")}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {hardware && (
            <div className="hidden lg:flex items-center gap-2 rounded-full border border-zinc-200 bg-zinc-100/80 px-3 py-1 text-[11px] font-medium text-zinc-600 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-400">
              <span className="flex items-center gap-1">
                <Cpu className="h-3 w-3 text-indigo-500" />
                {hardware.cpuCores} cores
              </span>
              {hardware.gpuBackendName && (
                <>
                  <span className="text-zinc-300 dark:text-zinc-700">·</span>
                  <span className="flex items-center gap-1">
                    <Zap className="h-3 w-3 text-amber-500" />
                    {hardware.gpuBackendName}
                  </span>
                </>
              )}
            </div>
          )}

          <button
            type="button"
            onClick={() => setShowCommandPalette(true)}
            className="inline-flex items-center gap-1 rounded-lg border border-zinc-200 px-2 py-1.5 text-[11px] font-semibold text-zinc-500 transition hover:bg-zinc-100 dark:border-zinc-800 dark:text-zinc-400 dark:hover:bg-zinc-900 cursor-pointer"
            title="Command Palette (⌘K)"
          >
            <Command className="h-3 w-3" />
            <span>K</span>
          </button>

          <button
            type="button"
            onClick={() => setShowWatchModal(true)}
            className="relative inline-flex items-center gap-1.5 rounded-lg border border-zinc-200 px-2.5 py-1.5 text-xs font-medium text-zinc-600 transition hover:bg-zinc-100 dark:border-zinc-800 dark:text-zinc-300 dark:hover:bg-zinc-900 cursor-pointer"
            title="Watch Folders (Auto-Wandel)"
          >
            <Radio
              className={`h-3.5 w-3.5 ${
                watchFolders.length > 0 ? "text-emerald-500" : "text-zinc-400"
              }`}
            />
            <span className="hidden md:inline">{t(locale, "watch")}</span>
            {watchFolders.length > 0 && (
              <span className="h-2 w-2 rounded-full bg-emerald-500 animate-pulse" />
            )}
          </button>

          <button
            type="button"
            onClick={() => setShowHistory(true)}
            className="inline-flex items-center gap-1.5 rounded-lg border border-zinc-200 px-2.5 py-1.5 text-xs font-medium text-zinc-600 transition hover:bg-zinc-100 dark:border-zinc-800 dark:text-zinc-300 dark:hover:bg-zinc-900 cursor-pointer"
            title="Conversion History & Stats"
          >
            <History className="h-3.5 w-3.5 text-indigo-500" />
            <span className="hidden md:inline">{t(locale, "history")}</span>
          </button>

          <button
            type="button"
            onClick={() => setShowMcpModal(true)}
            className="inline-flex items-center gap-1 rounded-lg border border-zinc-200 px-2 py-1.5 text-[11px] font-semibold text-zinc-600 transition hover:bg-zinc-100 dark:border-zinc-800 dark:text-zinc-300 dark:hover:bg-zinc-900 cursor-pointer"
            title="MCP Server Integration (Headless & UI)"
          >
            <Plug className="h-3.5 w-3.5 text-indigo-500" />
            <span className="hidden md:inline">MCP</span>
          </button>

          <button
            type="button"
            onClick={() => setShowQrModal(true)}
            className="rounded-lg border border-zinc-200 p-1.5 text-zinc-500 transition hover:bg-zinc-100 hover:text-zinc-800 dark:border-zinc-800 dark:text-zinc-400 dark:hover:bg-zinc-900 cursor-pointer"
            title="QR Code Generator"
          >
            <QrCode className="h-4 w-4" />
          </button>

          <button
            type="button"
            onClick={() => setShowShortcuts(true)}
            className="rounded-lg border border-zinc-200 p-1.5 text-zinc-500 transition hover:bg-zinc-100 hover:text-zinc-800 dark:border-zinc-800 dark:text-zinc-400 dark:hover:bg-zinc-900 cursor-pointer"
            title="Keyboard Shortcuts & About (?)"
          >
            <HelpCircle className="h-4 w-4" />
          </button>

          <button
            type="button"
            onClick={toggleLocale}
            className="rounded-lg border border-zinc-200 px-2 py-1.5 text-[11px] font-bold uppercase text-zinc-600 transition hover:bg-zinc-100 dark:border-zinc-800 dark:text-zinc-400 dark:hover:bg-zinc-900 cursor-pointer"
            title="Switch Language (EN / DE)"
          >
            {locale}
          </button>

          <button
            type="button"
            onClick={() => setDarkMode((v) => !v)}
            className="rounded-lg border border-zinc-200 p-2 text-zinc-600 transition hover:bg-zinc-100 dark:border-zinc-800 dark:text-zinc-400 dark:hover:bg-zinc-900 cursor-pointer"
            title="Toggle light/dark theme"
          >
            {darkMode ? (
              <Sun className="h-4 w-4" />
            ) : (
              <Moon className="h-4 w-4" />
            )}
          </button>
        </div>
      </header>

      {/* Mobile / Narrow Viewport Tab Switcher */}
      <div className="flex border-b border-zinc-200 bg-white/90 px-4 py-1.5 md:hidden dark:border-zinc-800 dark:bg-zinc-950/90">
        <div className="grid w-full grid-cols-2 gap-1 rounded-lg bg-zinc-100 p-0.5 text-xs font-semibold dark:bg-zinc-900">
          <button
            type="button"
            onClick={() => setMobileTab("queue")}
            className={`rounded-md py-1.5 transition cursor-pointer ${
              mobileTab === "queue"
                ? "bg-white text-indigo-600 shadow-xs dark:bg-zinc-800 dark:text-indigo-400"
                : "text-zinc-500"
            }`}
          >
            Queue ({items.length})
          </button>
          <button
            type="button"
            onClick={() => setMobileTab("settings")}
            className={`rounded-md py-1.5 transition cursor-pointer ${
              mobileTab === "settings"
                ? "bg-white text-indigo-600 shadow-xs dark:bg-zinc-800 dark:text-indigo-400"
                : "text-zinc-500"
            }`}
          >
            Settings &amp; Wandel
          </button>
        </div>
      </div>

      {/* Main Content Split */}
      <main className="flex min-h-0 flex-1">
        {/* Left: Drop Zone / File Queue */}
        <section
          className={`${
            mobileTab === "queue" ? "flex" : "hidden"
          } min-w-0 flex-1 flex-col p-4 md:flex md:p-5`}
        >
          {items.length === 0 ? (
            <DropZone
              isDragging={isDragging}
              isInspecting={isInspecting}
              onAddFiles={handleAddFiles}
              onAddFolder={handleAddFolder}
            />
          ) : (
            <FileQueue
              items={items}
              settings={settings}
              isConverting={isConverting}
              onRemoveItem={(id) =>
                setItems((prev) => prev.filter((i) => i.id !== id))
              }
              onClearAll={() => {
                setItems([]);
                setBatchSummary(null);
              }}
              onClearCompleted={() =>
                setItems((prev) => prev.filter((i) => i.status !== "completed"))
              }
              onAddFiles={handleAddFiles}
              onAddFolder={handleAddFolder}
              onRevealFile={handleRevealFile}
              onPreviewItem={setPreviewItem}
              onInspectItem={setInspectItem}
              onGenerateIconPack={handleGenerateIconPack}
              onGenerateSocialKit={handleGenerateSocialKit}
              onExtractGifFrames={handleExtractGifFrames}
              onOverrideImageFormat={(id: string, format: ImageFormat | undefined) =>
                setItems((prev) =>
                  prev.map((i) =>
                    i.id === id ? { ...i, overrideImageFormat: format } : i
                  )
                )
              }
              onOverrideVideoTarget={(id: string, target: VideoTarget | undefined) =>
                setItems((prev) =>
                  prev.map((i) =>
                    i.id === id ? { ...i, overrideVideoTarget: target } : i
                  )
                )
              }
              onMoveItem={handleMoveItem}
              onRetryFailed={handleRetryFailed}
              onMergePdf={handleMergePdf}
              onCreateGrid={handleCreateGrid}
              onCompareImages={handleCompareImages}
              onPixelDiff={handlePixelDiff}
              onCreateSlideshow={handleCreateSlideshow}
              onCreateAnimatedGif={handleCreateAnimatedGif}
              onStitchImages={handleStitchImages}
              onCreateSpriteSheet={handleCreateSpriteSheet}
              onMergeVideos={handleMergeVideos}
              onMergePdfs={handleMergePdfs}
              onInterleavePdfs={handleInterleavePdfs}
              onSplitPdf={handleSplitPdf}
              onCompressPdf={handleCompressPdf}
              onRotatePdf={handleRotatePdf}
              onReorderPdf={handleReorderPdf}
              onOrganizePdf={setPdfOrganizerItem}
              onStampPdf={handleStampPdf}
              onNumberPdf={handleNumberPdf}
              onExtractPdfImages={handleExtractPdfImages}
              onExtractPdfText={handleExtractPdfText}
              onExportPdfMarkdown={handleExportPdfMarkdown}
              onTwoUpPdf={handleTwoUpPdf}
              onEditPdfMeta={handleEditPdfMeta}
              onDeblankPdf={handleDeblankPdf}
              onDarkModePdf={handleDarkModePdf}
              onScrubPdf={handleScrubPdf}
              onPaletteItem={setPaletteItem}
              onDataUrlItem={setDataUrlItem}
              onResponsiveSetItem={handleGenerateResponsiveSet}
              onSliceImage={handleSliceImage}
              onStoryboardItem={handleCreateStoryboard}
              onExtractFrames={handleExtractFrames}
              onExtractPoster={handleExtractPoster}
              onAudioWaveform={handleAudioWaveform}
              onPodcastVideo={handlePodcastVideo}
              onSplitMedia={handleSplitMedia}
              onExportCsv={handleExportCsv}
              onRequeueOutputs={handleRequeueOutputs}
              onMergeAudio={handleMergeAudio}
              onFindDuplicates={handleFindDuplicates}
              onOpenPipeline={() => setShowPipelineModal(true)}
            />
          )}
        </section>

        {/* Right: Settings & Wandel CTA */}
        <aside
          className={`${
            mobileTab === "settings" ? "flex w-full" : "hidden"
          } shrink-0 flex-col border-l border-zinc-200 bg-white/70 p-4 md:flex md:w-[340px] dark:border-zinc-800/80 dark:bg-zinc-900/40`}
        >
          <SettingsPanel
            settings={settings}
            activePreset={activePreset}
            customPresets={customPresets}
            hardware={hardware}
            imageCount={imageCount}
            pdfCount={pdfCount}
            videoCount={videoCount}
            disabled={isConverting}
            minimizeToTray={minimizeToTray}
            onToggleMinimizeToTray={setMinimizeToTray}
            onChange={handleSettingsChange}
            onApplyPreset={handleApplyPreset}
            onApplyCustomPreset={handleApplyCustomPreset}
            onSaveCustomPreset={handleSaveCustomPreset}
            onDeleteCustomPreset={handleDeleteCustomPreset}
            onExportPresets={handleExportPresets}
            onImportPresets={handleImportPresets}
            onResetSettings={handleResetSettings}
            onPickOutputDir={handlePickOutputDir}
            onPickWatermark={handlePickWatermark}
            onPickVideoWatermark={handlePickVideoWatermark}
            onPickSubtitle={handlePickSubtitle}
            onPickReplaceAudio={handlePickReplaceAudio}
          />
          <WandelButton
            totalFiles={items.length}
            isConverting={isConverting}
            overallProgress={overallProgress}
            onWandel={handleWandel}
            onCancel={handleCancel}
          />
        </aside>
      </main>

      {previewItem && previewItem.mediaKind === "image" && (
        <PreviewModal
          item={previewItem}
          settings={
            previewItem.overrideImageFormat
              ? { ...settings.image, format: previewItem.overrideImageFormat }
              : settings.image
          }
          onApplyCustomCrop={(customCrop) =>
            setSettings((prev) => ({
              ...prev,
              image: { ...prev.image, customCrop },
            }))
          }
          onApplyBlurRegions={(blurRegions) =>
            setSettings((prev) => ({
              ...prev,
              image: { ...prev.image, blurRegions },
            }))
          }
          onClose={() => setPreviewItem(null)}
        />
      )}

      {previewItem && previewItem.mediaKind === "audio" && (
        <AudioPreviewModal
          item={previewItem}
          trimStartSecs={settings.video.trimStartSecs}
          trimEndSecs={settings.video.trimEndSecs}
          onApplyTrim={(start, end) =>
            setSettings((prev) => ({
              ...prev,
              video: {
                ...prev.video,
                trimStartSecs: start,
                trimEndSecs: end,
              },
            }))
          }
          onClose={() => setPreviewItem(null)}
        />
      )}

      {previewItem && previewItem.mediaKind === "video" && (
        <VideoPreviewModal
          item={previewItem}
          trimStartSecs={settings.video.trimStartSecs}
          trimEndSecs={settings.video.trimEndSecs}
          onApplyTrim={(start, end) =>
            setSettings((prev) => ({
              ...prev,
              video: {
                ...prev.video,
                trimStartSecs: start,
                trimEndSecs: end,
              },
            }))
          }
          onClose={() => setPreviewItem(null)}
        />
      )}

      {showHistory && (
        <HistoryDrawer
          entries={history}
          stats={lifetimeStats}
          onClose={() => setShowHistory(false)}
          onClear={() => {
            setHistory([]);
            localStorage.removeItem(HISTORY_KEY);
          }}
          onRevealFile={handleRevealFile}
        />
      )}

      {showWatchModal && (
        <WatchFolderModal
          folders={watchFolders}
          onAddFolder={handleAddWatchFolder}
          onStopFolder={handleStopWatchFolder}
          onClose={() => setShowWatchModal(false)}
        />
      )}

      {inspectItem && (
        <MetadataModal
          item={inspectItem}
          onClose={() => setInspectItem(null)}
        />
      )}

      {showCommandPalette && (
        <CommandPalette
          locale={locale}
          actions={paletteActions}
          onClose={() => setShowCommandPalette(false)}
        />
      )}

      {showMcpModal && <McpModal onClose={() => setShowMcpModal(false)} />}

      {showPipelineModal && (
        <PipelineModal
          items={items}
          onClose={() => setShowPipelineModal(false)}
          onCompleted={(summary) => setBatchSummary(summary)}
        />
      )}

      {/* Floating Dismissible Toast Notification */}
      {batchSummary && (
        <div
          role="status"
          className="fixed bottom-4 left-4 z-40 flex max-w-md items-center gap-2.5 rounded-xl border border-emerald-500/30 bg-white/95 px-3.5 py-2.5 text-xs font-medium text-zinc-800 shadow-lg backdrop-blur-md dark:bg-zinc-900/95 dark:text-zinc-100"
        >
          <span className="h-2 w-2 shrink-0 rounded-full bg-emerald-500" />
          <span className="truncate">{batchSummary}</span>
          <button
            type="button"
            onClick={() => setBatchSummary(null)}
            className="ml-1 rounded p-0.5 text-zinc-400 transition hover:bg-zinc-200/70 hover:text-zinc-700 dark:hover:bg-zinc-800 dark:hover:text-zinc-200 cursor-pointer"
            aria-label="Dismiss notification"
          >
            <X className="h-3.5 w-3.5" />
          </button>
        </div>
      )}

      {paletteItem && (
        <PaletteModal
          item={paletteItem}
          onClose={() => setPaletteItem(null)}
        />
      )}

      {pdfOrganizerItem && (
        <PdfOrganizerModal
          item={pdfOrganizerItem}
          onClose={() => setPdfOrganizerItem(null)}
          onSaved={(summary) => setBatchSummary(summary)}
        />
      )}

      {dataUrlItem && (
        <DataUrlModal
          item={dataUrlItem}
          settings={
            dataUrlItem.overrideImageFormat
              ? { ...settings.image, format: dataUrlItem.overrideImageFormat }
              : settings.image
          }
          onClose={() => setDataUrlItem(null)}
        />
      )}

      {showQrModal && <QrModal onClose={() => setShowQrModal(false)} />}

      {showShortcuts && (
        <ShortcutsModal
          hardware={hardware}
          onClose={() => setShowShortcuts(false)}
        />
      )}

      {/* Full-Window Drag Overlay when Queue already has files */}
      {isDragging && items.length > 0 && (
        <div className="pointer-events-none fixed inset-0 z-50 flex items-center justify-center bg-indigo-950/50 backdrop-blur-sm">
          <div className="flex flex-col items-center rounded-2xl border-2 border-dashed border-indigo-400 bg-zinc-900/90 px-10 py-8 text-white shadow-2xl">
            <UploadCloud className="mb-3 h-10 w-10 text-indigo-400" />
            <p className="text-base font-semibold">Drop files to add to Wandel</p>
          </div>
        </div>
      )}
    </div>
  );
}
