import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import {
  ArrowLeft,
  ArrowRight,
  ArrowUpDown,
  Copy,
  FileText,
  Loader2,
  RotateCcw,
  RotateCw,
  Save,
  Trash2,
  X,
} from "lucide-react";
import { QueueItem } from "../types";

interface PdfPageThumb {
  page: number;
  width: number;
  height: number;
  thumbnailDataUrl: string;
}

interface PageCard {
  uid: string;
  sourcePage: number;
  width: number;
  height: number;
  thumbnailDataUrl: string;
  rotationDelta: number;
}

interface PdfOrganizerModalProps {
  item: QueueItem;
  onClose: () => void;
  onSaved: (summary: string) => void;
}

export function PdfOrganizerModal({
  item,
  onClose,
  onSaved,
}: PdfOrganizerModalProps) {
  const [originalCards, setOriginalCards] = useState<PageCard[]>([]);
  const [cards, setCards] = useState<PageCard[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [dragIndex, setDragIndex] = useState<number | null>(null);
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !saving) onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose, saving]);

  useEffect(() => {
    let active = true;
    setLoading(true);
    setError(null);
    invoke<PdfPageThumb[]>("render_pdf_thumbnails", {
      path: item.path,
      maxEdgePx: 240,
    })
      .then((thumbs) => {
        if (!active) return;
        const initial: PageCard[] = thumbs.map((th) => ({
          uid: `p-${th.page}-0`,
          sourcePage: th.page,
          width: th.width,
          height: th.height,
          thumbnailDataUrl: th.thumbnailDataUrl,
          rotationDelta: 0,
        }));
        setOriginalCards(initial);
        setCards(initial);
        setLoading(false);
      })
      .catch((err) => {
        if (!active) return;
        setError(String(err));
        setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [item.path]);

  const moveCard = (from: number, to: number) => {
    if (from === to || to < 0 || to >= cards.length) return;
    setCards((prev) => {
      const next = [...prev];
      const [moved] = next.splice(from, 1);
      next.splice(to, 0, moved);
      return next;
    });
  };

  const rotateCard = (uid: string) => {
    setCards((prev) =>
      prev.map((c) =>
        c.uid === uid ? { ...c, rotationDelta: (c.rotationDelta + 90) % 360 } : c
      )
    );
  };

  const duplicateCard = (index: number) => {
    setCards((prev) => {
      const src = prev[index];
      const copy: PageCard = {
        ...src,
        uid: `p-${src.sourcePage}-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
      };
      const next = [...prev];
      next.splice(index + 1, 0, copy);
      return next;
    });
  };

  const removeCard = (uid: string) => {
    setCards((prev) => (prev.length > 1 ? prev.filter((c) => c.uid !== uid) : prev));
  };

  const handleSave = async () => {
    if (cards.length === 0 || saving) return;
    const dest = await save({
      title: "Save Organized PDF",
      defaultPath: item.fileName.replace(/\.pdf$/i, "_organized.pdf"),
      filters: [{ name: "PDF Document", extensions: ["pdf"] }],
    });
    if (!dest) return;

    setSaving(true);
    setError(null);
    try {
      const plan = cards.map((c) => ({
        page: c.sourcePage,
        rotationDelta: c.rotationDelta,
      }));
      const bytes = await invoke<number>("organize_pdf_pages", {
        path: item.path,
        outputPath: dest,
        plan,
      });
      await revealItemInDir(dest);
      onSaved(
        `Saved organized PDF (${cards.length} page${cards.length === 1 ? "" : "s"}, ${(bytes / 1024).toFixed(0)} KB)`
      );
      onClose();
    } catch (err) {
      setError(String(err));
      setSaving(false);
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/65 p-5 backdrop-blur-xs"
      onClick={() => !saving && onClose()}
    >
      <div
        className="flex max-h-[88vh] w-full max-w-4xl flex-col overflow-hidden rounded-2xl border border-zinc-200 bg-white shadow-2xl dark:border-zinc-800 dark:bg-zinc-950"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between border-b border-zinc-200 px-5 py-3.5 dark:border-zinc-800">
          <div className="flex min-w-0 items-center gap-2.5">
            <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-xl bg-rose-500/15 text-rose-500">
              <FileText className="h-4 w-4" />
            </div>
            <div className="min-w-0">
              <h3 className="truncate text-sm font-bold text-zinc-900 dark:text-zinc-100">
                Organize PDF Pages — {item.fileName}
              </h3>
              <p className="text-[11px] text-zinc-500">
                Drag cards to reorder, rotate individual pages, duplicate, or remove — lossless export
              </p>
            </div>
          </div>
          <button
            type="button"
            disabled={saving}
            onClick={onClose}
            className="rounded-lg p-1.5 text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 cursor-pointer"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        {/* Toolbar */}
        {!loading && cards.length > 0 && (
          <div className="flex flex-wrap items-center justify-between gap-2 border-b border-zinc-200 bg-zinc-50/80 px-5 py-2.5 text-xs dark:border-zinc-800 dark:bg-zinc-900/50">
            <div className="flex items-center gap-2 text-zinc-600 dark:text-zinc-300">
              <span className="font-semibold">{cards.length}</span>
              <span>
                of {originalCards.length} page{originalCards.length === 1 ? "" : "s"} selected
              </span>
            </div>
            <div className="flex flex-wrap items-center gap-1.5">
              <button
                type="button"
                onClick={() => setCards((prev) => [...prev].reverse())}
                className="flex items-center gap-1 rounded-lg border border-zinc-200 bg-white px-2.5 py-1 text-[11px] font-medium text-zinc-700 hover:bg-zinc-100 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-200 dark:hover:bg-zinc-800 cursor-pointer"
              >
                <ArrowUpDown className="h-3 w-3" />
                Reverse
              </button>
              <button
                type="button"
                onClick={() =>
                  setCards((prev) =>
                    prev.map((c) => ({ ...c, rotationDelta: (c.rotationDelta + 90) % 360 }))
                  )
                }
                className="flex items-center gap-1 rounded-lg border border-zinc-200 bg-white px-2.5 py-1 text-[11px] font-medium text-zinc-700 hover:bg-zinc-100 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-200 dark:hover:bg-zinc-800 cursor-pointer"
              >
                <RotateCw className="h-3 w-3" />
                Rotate All 90°
              </button>
              <button
                type="button"
                onClick={() => setCards(originalCards)}
                className="flex items-center gap-1 rounded-lg border border-zinc-200 bg-white px-2.5 py-1 text-[11px] font-medium text-zinc-700 hover:bg-zinc-100 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-200 dark:hover:bg-zinc-800 cursor-pointer"
              >
                <RotateCcw className="h-3 w-3" />
                Reset
              </button>
            </div>
          </div>
        )}

        {/* Body */}
        <div className="flex-1 overflow-y-auto p-5">
          {error && (
            <div className="mb-4 rounded-xl border border-red-500/30 bg-red-500/10 px-3.5 py-2.5 text-xs text-red-600 dark:text-red-400">
              {error}
            </div>
          )}

          {loading ? (
            <div className="flex flex-col items-center justify-center py-16 text-zinc-500">
              <Loader2 className="mb-2.5 h-6 w-6 animate-spin text-indigo-500" />
              <p className="text-xs font-medium">Rendering PDF page thumbnails in parallel...</p>
            </div>
          ) : (
            <div className="grid grid-cols-2 gap-3.5 sm:grid-cols-3 md:grid-cols-4">
              {cards.map((card, idx) => (
                <div
                  key={card.uid}
                  draggable
                  onDragStart={() => setDragIndex(idx)}
                  onDragOver={(e) => {
                    e.preventDefault();
                    if (dragOverIndex !== idx) setDragOverIndex(idx);
                  }}
                  onDrop={() => {
                    if (dragIndex !== null && dragIndex !== idx) {
                      moveCard(dragIndex, idx);
                    }
                    setDragIndex(null);
                    setDragOverIndex(null);
                  }}
                  onDragEnd={() => {
                    setDragIndex(null);
                    setDragOverIndex(null);
                  }}
                  className={`group flex flex-col rounded-xl border p-2.5 transition cursor-grab active:cursor-grabbing ${
                    dragOverIndex === idx && dragIndex !== idx
                      ? "border-indigo-500 bg-indigo-500/10 ring-2 ring-indigo-500/30"
                      : "border-zinc-200 bg-zinc-50/70 hover:border-zinc-300 dark:border-zinc-800 dark:bg-zinc-900/60 dark:hover:border-zinc-700"
                  }`}
                >
                  {/* Thumbnail canvas */}
                  <div className="relative flex h-40 items-center justify-center overflow-hidden rounded-lg bg-zinc-200/70 p-2 dark:bg-zinc-950">
                    <img
                      src={card.thumbnailDataUrl}
                      alt={`Page ${card.sourcePage}`}
                      draggable={false}
                      style={{ transform: `rotate(${card.rotationDelta}deg)` }}
                      className="max-h-full max-w-full rounded object-contain shadow-sm transition-transform duration-200"
                    />
                    <span className="absolute left-1.5 top-1.5 rounded-md bg-zinc-900/80 px-1.5 py-0.5 text-[10px] font-bold text-white">
                      #{idx + 1}
                    </span>
                    {card.rotationDelta !== 0 && (
                      <span className="absolute right-1.5 top-1.5 rounded-md bg-amber-500/90 px-1.5 py-0.5 text-[10px] font-bold text-white">
                        +{card.rotationDelta}°
                      </span>
                    )}
                  </div>

                  {/* Page meta & controls */}
                  <div className="mt-2 flex items-center justify-between text-[11px] text-zinc-500">
                    <span className="truncate">Src p.{card.sourcePage}</span>
                    <div className="flex items-center gap-0.5">
                      <button
                        type="button"
                        disabled={idx === 0}
                        onClick={() => moveCard(idx, idx - 1)}
                        className="rounded p-1 hover:bg-zinc-200 disabled:opacity-30 dark:hover:bg-zinc-800 cursor-pointer"
                        title="Move left"
                      >
                        <ArrowLeft className="h-3 w-3" />
                      </button>
                      <button
                        type="button"
                        disabled={idx === cards.length - 1}
                        onClick={() => moveCard(idx, idx + 1)}
                        className="rounded p-1 hover:bg-zinc-200 disabled:opacity-30 dark:hover:bg-zinc-800 cursor-pointer"
                        title="Move right"
                      >
                        <ArrowRight className="h-3 w-3" />
                      </button>
                      <button
                        type="button"
                        onClick={() => rotateCard(card.uid)}
                        className="rounded p-1 text-amber-600 hover:bg-amber-500/15 dark:text-amber-400 cursor-pointer"
                        title="Rotate 90° clockwise"
                      >
                        <RotateCw className="h-3 w-3" />
                      </button>
                      <button
                        type="button"
                        onClick={() => duplicateCard(idx)}
                        className="rounded p-1 text-indigo-600 hover:bg-indigo-500/15 dark:text-indigo-400 cursor-pointer"
                        title="Duplicate page"
                      >
                        <Copy className="h-3 w-3" />
                      </button>
                      <button
                        type="button"
                        disabled={cards.length <= 1}
                        onClick={() => removeCard(card.uid)}
                        className="rounded p-1 text-rose-500 hover:bg-rose-500/15 disabled:opacity-25 cursor-pointer"
                        title="Remove page"
                      >
                        <Trash2 className="h-3 w-3" />
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-2 border-t border-zinc-200 px-5 py-3.5 dark:border-zinc-800">
          <button
            type="button"
            disabled={saving}
            onClick={onClose}
            className="rounded-xl border border-zinc-200 px-3.5 py-2 text-xs font-semibold text-zinc-600 hover:bg-zinc-100 dark:border-zinc-700 dark:text-zinc-300 dark:hover:bg-zinc-800 cursor-pointer"
          >
            Cancel
          </button>
          <button
            type="button"
            disabled={loading || saving || cards.length === 0}
            onClick={handleSave}
            className="flex items-center gap-1.5 rounded-xl bg-indigo-600 px-4 py-2 text-xs font-semibold text-white shadow-sm hover:bg-indigo-500 disabled:opacity-50 cursor-pointer"
          >
            {saving ? (
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
            ) : (
              <Save className="h-3.5 w-3.5" />
            )}
            <span>Save Organized PDF</span>
          </button>
        </div>
      </div>
    </div>
  );
}
