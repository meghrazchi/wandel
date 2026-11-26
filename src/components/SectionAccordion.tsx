import { ReactNode } from "react";
import { ChevronDown, ChevronRight } from "lucide-react";

interface SectionAccordionProps {
  title: string;
  icon: ReactNode;
  open: boolean;
  onToggle: () => void;
  activeCount?: number;
  children: ReactNode;
}

export function SectionAccordion({
  title,
  icon,
  open,
  onToggle,
  activeCount = 0,
  children,
}: SectionAccordionProps) {
  return (
    <div className="overflow-hidden rounded-xl border border-zinc-200/90 bg-white/80 transition-colors dark:border-zinc-800/90 dark:bg-zinc-900/50">
      <button
        type="button"
        aria-expanded={open}
        onClick={onToggle}
        className="flex w-full items-center justify-between gap-2 px-3 py-2.5 text-left transition hover:bg-zinc-100/70 dark:hover:bg-zinc-800/60 cursor-pointer"
      >
        <span className="flex items-center gap-2 text-xs font-semibold text-zinc-700 dark:text-zinc-200">
          <span className="text-zinc-400 dark:text-zinc-500">{icon}</span>
          {title}
        </span>
        <span className="flex items-center gap-1.5">
          {activeCount > 0 && (
            <span className="rounded-full bg-indigo-500/15 px-1.5 py-0.5 text-[10px] font-bold text-indigo-600 dark:text-indigo-400">
              {activeCount}
            </span>
          )}
          {open ? (
            <ChevronDown className="h-3.5 w-3.5 text-zinc-400" />
          ) : (
            <ChevronRight className="h-3.5 w-3.5 text-zinc-400" />
          )}
        </span>
      </button>
      {open && (
        <div className="space-y-3 border-t border-zinc-200/80 px-3 py-3 dark:border-zinc-800/80">
          {children}
        </div>
      )}
    </div>
  );
}
