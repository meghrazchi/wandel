import { useEffect } from "react";
import { Cpu, Keyboard, X, Zap } from "lucide-react";
import { HardwareCapabilities } from "../types";
import { WandelIcon } from "./WandelIcon";

interface ShortcutsModalProps {
  hardware: HardwareCapabilities | null;
  onClose: () => void;
}

export function ShortcutsModal({ hardware, onClose }: ShortcutsModalProps) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

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
          <div className="flex items-center gap-2.5">
            <WandelIcon className="h-8 w-8 shrink-0" />
            <div>
              <h3 className="text-sm font-bold text-zinc-900 dark:text-zinc-100">
                Wandel v2.3.3
              </h3>
              <p className="text-[11px] text-zinc-500">
                Rust + Tauri v2 Parallel Media Studio
              </p>
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

        <div className="space-y-4 p-5">
          {hardware && (
            <div className="flex items-center justify-between rounded-xl border border-zinc-200 bg-zinc-50 px-3.5 py-2.5 text-xs dark:border-zinc-800 dark:bg-zinc-900">
              <span className="flex items-center gap-1.5 font-medium text-zinc-700 dark:text-zinc-300">
                <Cpu className="h-3.5 w-3.5 text-indigo-500" />
                {hardware.cpuCores} CPU Threads
              </span>
              <span className="flex items-center gap-1.5 font-medium text-zinc-700 dark:text-zinc-300">
                <Zap className="h-3.5 w-3.5 text-amber-500" />
                {hardware.gpuBackendName ?? "CPU Software"}
              </span>
            </div>
          )}

          <div>
            <div className="mb-2 flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider text-zinc-500">
              <Keyboard className="h-3.5 w-3.5" />
              Keyboard Shortcuts
            </div>
            <div className="divide-y divide-zinc-200 rounded-xl border border-zinc-200 text-xs dark:divide-zinc-800 dark:border-zinc-800">
              {[
                { keys: "⌘ / Ctrl + Enter", desc: "Start Wandel conversion" },
                { keys: "⌘ / Ctrl + K", desc: "Open Command Palette" },
                { keys: "⌘ / Ctrl + V", desc: "Paste image from clipboard" },
                { keys: "?", desc: "Show Shortcuts & About" },
                { keys: "Esc", desc: "Cancel batch / close modal" },
              ].map((s) => (
                <div
                  key={s.keys}
                  className="flex items-center justify-between px-3.5 py-2"
                >
                  <span className="text-zinc-600 dark:text-zinc-400">
                    {s.desc}
                  </span>
                  <kbd className="rounded bg-zinc-100 px-2 py-0.5 font-mono text-[11px] font-semibold text-zinc-800 dark:bg-zinc-900 dark:text-zinc-200">
                    {s.keys}
                  </kbd>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
