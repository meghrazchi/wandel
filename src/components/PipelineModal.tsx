import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import {
  CheckCircle2,
  Eye,
  FolderOpen,
  GitBranch,
  Loader2,
  Play,
  X,
} from "lucide-react";
import { QueueItem } from "../types";

interface PipelineStepResult {
  step: number;
  tool: string;
  durationMs: number;
  isError: boolean;
  outputs?: string[];
  result?: unknown;
}

interface PipelineRunResult {
  completedSteps: number;
  totalDurationMs: number;
  finalOutputs?: string[];
  steps?: PipelineStepResult[];
}

interface PipelineModalProps {
  items: QueueItem[];
  onClose: () => void;
  onCompleted: (summary: string) => void;
}

function buildTemplates(items: QueueItem[]) {
  const imagePaths = items
    .filter((i) => i.mediaKind === "image")
    .map((i) => i.path);
  const sampleInputs =
    imagePaths.length > 0
      ? imagePaths
      : items.length > 0
      ? items.map((i) => i.path)
      : ["/path/to/photo1.png", "/path/to/photo2.png"];

  const firstDir = sampleInputs[0]?.replace(/[/\\][^/\\]+$/, "") || "/tmp";

  return [
    {
      id: "web-grid",
      label: "Web Optimize → Contact Sheet",
      recipe: {
        steps: [
          {
            tool: "wandel_convert",
            arguments: {
              inputs: sampleInputs,
              to: "webp",
              quality: 82,
              autoLevels: true,
            },
          },
          {
            tool: "wandel_image_tools",
            arguments: {
              action: "contact_sheet",
              inputs: "$prev",
              output: `${firstDir}/wandel_pipeline_grid.png`,
              columns: 3,
            },
          },
        ],
      },
    },
    {
      id: "proof-pdf",
      label: "Watermark → Numbered PDF",
      recipe: {
        steps: [
          {
            tool: "wandel_convert",
            arguments: {
              inputs: sampleInputs,
              to: "png",
              textWatermark: "DRAFT",
              tiledWatermark: true,
            },
          },
          {
            tool: "wandel_pdf_ops",
            arguments: {
              action: "merge_images",
              inputs: "$prev",
              output: `${firstDir}/wandel_proof.pdf`,
              pageFit: "a4",
            },
          },
          {
            tool: "wandel_pdf_ops",
            arguments: {
              action: "number",
              input: "$prev",
              output: `${firstDir}/wandel_proof_numbered.pdf`,
              text: "Page {n} of {total}",
            },
          },
        ],
      },
    },
    {
      id: "dry-run",
      label: "Batch Convert (Dry-Run)",
      recipe: {
        steps: [
          {
            tool: "wandel_convert",
            arguments: {
              inputs: sampleInputs,
              to: "webp",
              quality: 80,
              dryRun: true,
            },
          },
        ],
      },
    },
  ];
}

export function PipelineModal({
  items,
  onClose,
  onCompleted,
}: PipelineModalProps) {
  const templates = buildTemplates(items);
  const [recipeText, setRecipeText] = useState(() =>
    JSON.stringify(templates[0].recipe, null, 2)
  );
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<PipelineRunResult | null>(null);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !running) onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose, running]);

  const execute = async (forceDryRun: boolean) => {
    setRunning(true);
    setError(null);
    setResult(null);
    try {
      let payloadStr = recipeText;
      if (forceDryRun) {
        const parsed = JSON.parse(recipeText) as {
          steps?: Array<{ tool?: string; arguments?: Record<string, unknown> }>;
        };
        if (Array.isArray(parsed.steps)) {
          parsed.steps = parsed.steps.map((s) =>
            s.tool === "wandel_convert"
              ? { ...s, arguments: { ...(s.arguments ?? {}), dryRun: true } }
              : s
          );
        }
        payloadStr = JSON.stringify(parsed);
      }

      const res = await invoke<PipelineRunResult>("run_pipeline_recipe", {
        recipeJson: payloadStr,
      });
      setResult(res);
      if (!forceDryRun) {
        onCompleted(
          `Pipeline completed ${res.completedSteps} step(s) in ${(res.totalDurationMs / 1000).toFixed(2)}s`
        );
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setRunning(false);
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/65 p-5 backdrop-blur-xs"
      onClick={() => !running && onClose()}
    >
      <div
        className="flex max-h-[88vh] w-full max-w-3xl flex-col overflow-hidden rounded-2xl border border-zinc-200 bg-white shadow-2xl dark:border-zinc-800 dark:bg-zinc-950"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between border-b border-zinc-200 px-5 py-3.5 dark:border-zinc-800">
          <div className="flex items-center gap-2.5">
            <div className="flex h-8 w-8 items-center justify-center rounded-xl bg-indigo-500/15 text-indigo-500">
              <GitBranch className="h-4 w-4" />
            </div>
            <div>
              <h3 className="text-sm font-bold text-zinc-900 dark:text-zinc-100">
                Composable Pipeline Runner
              </h3>
              <p className="text-[11px] text-zinc-500">
                Chain multiple stages with <code className="font-mono">$prev</code> and{" "}
                <code className="font-mono">$step.N</code> output piping
              </p>
            </div>
          </div>
          <button
            type="button"
            disabled={running}
            onClick={onClose}
            className="rounded-lg p-1.5 text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 cursor-pointer"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        {/* Body */}
        <div className="flex-1 space-y-4 overflow-y-auto p-5 text-xs">
          {/* Starter Templates */}
          <div className="flex flex-wrap items-center gap-1.5">
            <span className="text-[11px] font-semibold text-zinc-500">
              Starter Recipes:
            </span>
            {templates.map((tpl) => (
              <button
                key={tpl.id}
                type="button"
                onClick={() => {
                  setRecipeText(JSON.stringify(tpl.recipe, null, 2));
                  setResult(null);
                  setError(null);
                }}
                className="rounded-lg border border-zinc-200 bg-zinc-50 px-2.5 py-1 text-[11px] font-medium text-zinc-700 hover:border-indigo-500/40 hover:bg-indigo-500/10 hover:text-indigo-600 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-300 dark:hover:text-indigo-400 cursor-pointer"
              >
                {tpl.label}
              </button>
            ))}
          </div>

          {/* JSON Editor */}
          <textarea
            value={recipeText}
            onChange={(e) => setRecipeText(e.target.value)}
            rows={13}
            spellCheck={false}
            className="w-full rounded-xl border border-zinc-200 bg-zinc-950 p-3.5 font-mono text-[11px] leading-relaxed text-zinc-100 outline-none focus:border-indigo-500 dark:border-zinc-800"
          />

          {error && (
            <div className="rounded-xl border border-red-500/30 bg-red-500/10 px-3.5 py-2.5 text-xs text-red-600 dark:text-red-400">
              {error}
            </div>
          )}

          {result && (
            <div className="space-y-2 rounded-xl border border-emerald-500/30 bg-emerald-500/5 p-3.5">
              <div className="flex items-center justify-between text-emerald-600 dark:text-emerald-400">
                <span className="flex items-center gap-1.5 font-semibold">
                  <CheckCircle2 className="h-4 w-4" />
                  Completed {result.completedSteps} step(s)
                </span>
                <span className="font-mono text-[11px]">
                  {(result.totalDurationMs / 1000).toFixed(2)}s
                </span>
              </div>

              {result.steps && (
                <div className="space-y-1">
                  {result.steps.map((st) => (
                    <div
                      key={st.step}
                      className="flex items-center justify-between rounded-lg bg-white/80 px-2.5 py-1.5 text-[11px] dark:bg-zinc-900/80"
                    >
                      <span className="font-mono font-semibold text-zinc-800 dark:text-zinc-200">
                        Step {st.step + 1}: {st.tool}
                      </span>
                      <span className="text-zinc-500">{st.durationMs} ms</span>
                    </div>
                  ))}
                </div>
              )}

              {result.finalOutputs && result.finalOutputs.length > 0 && (
                <div className="space-y-1 pt-1">
                  <div className="text-[10px] font-semibold uppercase tracking-wider text-zinc-500">
                    Final Outputs
                  </div>
                  {result.finalOutputs.map((outPath) => (
                    <div
                      key={outPath}
                      className="flex items-center justify-between gap-2 rounded-lg bg-white px-2.5 py-1.5 font-mono text-[11px] text-zinc-700 dark:bg-zinc-900 dark:text-zinc-300"
                    >
                      <span className="truncate">{outPath}</span>
                      <button
                        type="button"
                        onClick={() => void revealItemInDir(outPath)}
                        className="inline-flex shrink-0 items-center gap-1 text-indigo-600 hover:underline dark:text-indigo-400 cursor-pointer"
                      >
                        <FolderOpen className="h-3 w-3" />
                        Reveal
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-2 border-t border-zinc-200 px-5 py-3.5 dark:border-zinc-800">
          <button
            type="button"
            disabled={running}
            onClick={() => void execute(true)}
            className="inline-flex items-center gap-1.5 rounded-xl border border-zinc-200 px-3.5 py-2 text-xs font-semibold text-zinc-700 hover:bg-zinc-100 disabled:opacity-50 dark:border-zinc-700 dark:text-zinc-200 dark:hover:bg-zinc-800 cursor-pointer"
          >
            <Eye className="h-3.5 w-3.5" />
            Dry Run
          </button>
          <button
            type="button"
            disabled={running}
            onClick={() => void execute(false)}
            className="inline-flex items-center gap-1.5 rounded-xl bg-indigo-600 px-4 py-2 text-xs font-semibold text-white shadow-sm hover:bg-indigo-500 disabled:opacity-50 cursor-pointer"
          >
            {running ? (
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
            ) : (
              <Play className="h-3.5 w-3.5" />
            )}
            <span>Run Pipeline</span>
          </button>
        </div>
      </div>
    </div>
  );
}
