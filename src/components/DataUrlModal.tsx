import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Check, Code2, Copy, Loader2, X } from "lucide-react";
import {
  DataUrlExport,
  ImageSettings,
  QueueItem,
  formatBytes,
} from "../types";

interface DataUrlModalProps {
  item: QueueItem;
  settings: ImageSettings;
  onClose: () => void;
}

export function DataUrlModal({ item, settings, onClose }: DataUrlModalProps) {
  const [data, setData] = useState<DataUrlExport | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [tab, setTab] = useState<"uri" | "html" | "css">("uri");
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    let cancelled = false;
    invoke<DataUrlExport>("export_image_data_url", {
      path: item.path,
      settings,
    })
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
  }, [item.path, settings, onClose]);

  const snippet = data
    ? tab === "uri"
      ? data.dataUrl
      : tab === "html"
      ? `<img src="${data.dataUrl}" width="${data.width}" height="${data.height}" alt="${item.fileName}" />`
      : `background-image: url("${data.dataUrl}");`
    : "";

  const handleCopy = () => {
    if (!snippet) return;
    void navigator.clipboard.writeText(snippet);
    setCopied(true);
    setTimeout(() => setCopied(false), 1800);
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/65 p-6 backdrop-blur-xs"
      onClick={onClose}
    >
      <div
        className="w-full max-w-2xl overflow-hidden rounded-2xl border border-zinc-200 bg-white shadow-2xl dark:border-zinc-800 dark:bg-zinc-950"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between border-b border-zinc-200 px-5 py-4 dark:border-zinc-800">
          <div className="flex items-center gap-2 min-w-0">
            <Code2 className="h-4.5 w-4.5 text-indigo-500" />
            <div className="min-w-0">
              <h3 className="truncate text-sm font-semibold text-zinc-900 dark:text-zinc-100">
                Base64 / Data URL — {item.fileName}
              </h3>
              {data && (
                <p className="text-[11px] text-zinc-500">
                  {data.mime} · {data.width}×{data.height} ·{" "}
                  {formatBytes(data.bytes)} binary ({formatBytes(data.dataUrl.length)} base64)
                </p>
              )}
            </div>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg p-1.5 text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 cursor-pointer"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <div className="space-y-3 p-5">
          {loading ? (
            <div className="flex items-center justify-center gap-2 py-10 text-sm text-zinc-500">
              <Loader2 className="h-5 w-5 animate-spin text-indigo-500" />
              Encoding Base64 Data URL...
            </div>
          ) : error ? (
            <p className="text-sm text-red-500">{error}</p>
          ) : (
            <>
              <div className="grid grid-cols-3 gap-1 rounded-xl bg-zinc-200/70 p-1 dark:bg-zinc-900">
                {(
                  [
                    { id: "uri", label: "Data URI" },
                    { id: "html", label: "HTML <img>" },
                    { id: "css", label: "CSS url()" },
                  ] as const
                ).map((t) => (
                  <button
                    key={t.id}
                    type="button"
                    onClick={() => setTab(t.id)}
                    className={`rounded-lg py-1.5 text-xs font-semibold transition cursor-pointer ${
                      tab === t.id
                        ? "bg-white text-zinc-900 shadow-sm dark:bg-zinc-800 dark:text-zinc-100"
                        : "text-zinc-500 dark:text-zinc-400"
                    }`}
                  >
                    {t.label}
                  </button>
                ))}
              </div>

              <textarea
                readOnly
                value={snippet}
                rows={7}
                className="w-full rounded-xl border border-zinc-200 bg-zinc-50 p-3 font-mono text-[11px] text-zinc-800 outline-none dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-200"
              />

              <button
                type="button"
                onClick={handleCopy}
                className="flex w-full items-center justify-center gap-2 rounded-xl bg-indigo-600 py-2.5 text-xs font-semibold text-white transition hover:bg-indigo-500 cursor-pointer"
              >
                {copied ? (
                  <>
                    <Check className="h-3.5 w-3.5" />
                    Copied to Clipboard!
                  </>
                ) : (
                  <>
                    <Copy className="h-3.5 w-3.5" />
                    Copy {tab.toUpperCase()} Snippet
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
