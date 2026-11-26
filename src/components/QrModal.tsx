import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { Download, QrCode, X } from "lucide-react";

interface QrModalProps {
  onClose: () => void;
}

export function QrModal({ onClose }: QrModalProps) {
  const [content, setContent] = useState("https://github.com");
  const [darkHex, setDarkHex] = useState("#111827");
  const [lightHex, setLightHex] = useState("#ffffff");
  const [svg, setSvg] = useState<string>("");
  const [status, setStatus] = useState<string | null>(null);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  useEffect(() => {
    if (!content.trim()) {
      setSvg("");
      return;
    }
    invoke<string>("generate_qr_svg", { content, darkHex, lightHex })
      .then(setSvg)
      .catch(() => setSvg(""));
  }, [content, darkHex, lightHex]);

  const handleExport = async (ext: "png" | "svg") => {
    if (!content.trim()) return;
    const dest = await save({
      title: `Save QR Code (${ext.toUpperCase()})`,
      defaultPath: `wandel_qr.${ext}`,
      filters: [{ name: ext.toUpperCase(), extensions: [ext] }],
    });
    if (!dest) return;
    try {
      await invoke("save_qr_code", {
        content,
        darkHex,
        lightHex,
        sizePx: 1024,
        outputPath: dest,
      });
      setStatus(`Saved ${ext.toUpperCase()}!`);
      await revealItemInDir(dest);
    } catch (err) {
      setStatus(`Error: ${String(err)}`);
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/65 p-6 backdrop-blur-xs"
      onClick={onClose}
    >
      <div
        className="w-full max-w-md overflow-hidden rounded-2xl border border-zinc-200 bg-white shadow-2xl dark:border-zinc-800 dark:bg-zinc-950"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between border-b border-zinc-200 px-5 py-4 dark:border-zinc-800">
          <div className="flex items-center gap-2">
            <QrCode className="h-4.5 w-4.5 text-indigo-500" />
            <h3 className="text-sm font-semibold text-zinc-900 dark:text-zinc-100">
              QR Code Generator
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

        <div className="space-y-4 p-5">
          <input
            type="text"
            value={content}
            onChange={(e) => setContent(e.target.value)}
            placeholder="Enter URL or text..."
            autoFocus
            className="w-full rounded-xl border border-zinc-200 bg-white px-3 py-2 text-xs text-zinc-900 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-100"
          />

          <div className="flex items-center justify-center rounded-xl border border-zinc-200 bg-zinc-50 p-4 dark:border-zinc-800 dark:bg-zinc-900">
            {svg ? (
              <div
                className="h-48 w-48 overflow-hidden rounded-lg shadow-sm [&>svg]:h-full [&>svg]:w-full"
                dangerouslySetInnerHTML={{ __html: svg }}
              />
            ) : (
              <div className="flex h-48 w-48 items-center justify-center text-xs text-zinc-400">
                Enter text to preview QR
              </div>
            )}
          </div>

          <div className="grid grid-cols-2 gap-3 text-xs">
            <label className="flex items-center justify-between rounded-xl border border-zinc-200 px-3 py-2 dark:border-zinc-800">
              <span className="text-zinc-500">Foreground</span>
              <input
                type="color"
                value={darkHex}
                onChange={(e) => setDarkHex(e.target.value)}
                className="h-5 w-7 cursor-pointer border-0 bg-transparent"
              />
            </label>
            <label className="flex items-center justify-between rounded-xl border border-zinc-200 px-3 py-2 dark:border-zinc-800">
              <span className="text-zinc-500">Background</span>
              <input
                type="color"
                value={lightHex}
                onChange={(e) => setLightHex(e.target.value)}
                className="h-5 w-7 cursor-pointer border-0 bg-transparent"
              />
            </label>
          </div>

          {status && (
            <p className="text-center text-xs font-medium text-emerald-500">
              {status}
            </p>
          )}

          <div className="grid grid-cols-2 gap-2">
            <button
              type="button"
              disabled={!content.trim()}
              onClick={() => void handleExport("svg")}
              className="inline-flex items-center justify-center gap-1.5 rounded-xl border border-zinc-200 bg-white py-2.5 text-xs font-semibold text-zinc-800 transition hover:bg-zinc-100 disabled:opacity-40 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-200 cursor-pointer"
            >
              <Download className="h-3.5 w-3.5" />
              Export SVG
            </button>
            <button
              type="button"
              disabled={!content.trim()}
              onClick={() => void handleExport("png")}
              className="inline-flex items-center justify-center gap-1.5 rounded-xl bg-indigo-600 py-2.5 text-xs font-semibold text-white transition hover:bg-indigo-500 disabled:opacity-40 cursor-pointer"
            >
              <Download className="h-3.5 w-3.5" />
              Export PNG (1024px)
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
