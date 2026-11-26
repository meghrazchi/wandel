import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Check, Copy, Loader2, Palette, X } from "lucide-react";
import { ColorSwatch, QueueItem } from "../types";

interface PaletteModalProps {
  item: QueueItem;
  onClose: () => void;
}

export function PaletteModal({ item, onClose }: PaletteModalProps) {
  const [swatches, setSwatches] = useState<ColorSwatch[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    invoke<ColorSwatch[]>("extract_color_palette", { path: item.path, count: 6 })
      .then((res) => {
        if (!cancelled) {
          setSwatches(res);
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

  const copyText = (text: string, key: string) => {
    void navigator.clipboard.writeText(text);
    setCopied(key);
    setTimeout(() => setCopied(null), 1500);
  };

  const cssVars = swatches
    .map((s, i) => `  --color-${i + 1}: ${s.hex};`)
    .join("\n");

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
          <div className="flex items-center gap-2 min-w-0">
            <Palette className="h-4.5 w-4.5 text-indigo-500" />
            <h3 className="truncate text-sm font-semibold text-zinc-900 dark:text-zinc-100">
              Color Palette — {item.fileName}
            </h3>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg p-1.5 text-zinc-400 hover:bg-zinc-100 hover:text-zinc-800 dark:hover:bg-zinc-800 cursor-pointer"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <div className="p-5 space-y-4">
          {loading ? (
            <div className="flex items-center justify-center gap-2 py-8 text-sm text-zinc-500">
              <Loader2 className="h-5 w-5 animate-spin text-indigo-500" />
              Extracting dominant colors...
            </div>
          ) : error ? (
            <p className="text-sm text-red-500">{error}</p>
          ) : (
            <>
              <div className="grid grid-cols-2 gap-2.5 sm:grid-cols-3">
                {swatches.map((s) => (
                  <button
                    key={s.hex}
                    type="button"
                    onClick={() => copyText(s.hex, s.hex)}
                    className="group overflow-hidden rounded-xl border border-zinc-200 bg-zinc-50 text-left transition hover:scale-[1.02] dark:border-zinc-800 dark:bg-zinc-900 cursor-pointer"
                  >
                    <div
                      className="h-16 w-full"
                      style={{ backgroundColor: s.hex }}
                    />
                    <div className="p-2">
                      <div className="flex items-center justify-between text-xs font-bold text-zinc-900 dark:text-zinc-100">
                        <span className="font-mono uppercase">{s.hex}</span>
                        {copied === s.hex ? (
                          <Check className="h-3.5 w-3.5 text-emerald-500" />
                        ) : (
                          <Copy className="h-3 w-3 opacity-40 group-hover:opacity-100" />
                        )}
                      </div>
                      <div className="mt-0.5 text-[10px] text-zinc-500">
                        rgb({s.r}, {s.g}, {s.b}) · {s.percent}%
                      </div>
                    </div>
                  </button>
                ))}
              </div>

              <button
                type="button"
                onClick={() => copyText(`:root {\n${cssVars}\n}`, "css")}
                className="flex w-full items-center justify-center gap-2 rounded-xl bg-indigo-600 py-2.5 text-xs font-semibold text-white transition hover:bg-indigo-500 cursor-pointer"
              >
                {copied === "css" ? (
                  <>
                    <Check className="h-3.5 w-3.5" />
                    Copied CSS Variables!
                  </>
                ) : (
                  <>
                    <Copy className="h-3.5 w-3.5" />
                    Copy CSS Variables
                  </>
                )}
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
