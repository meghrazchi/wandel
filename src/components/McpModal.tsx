import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Check, Copy, Cpu, FolderLock, Plug, RefreshCw, Terminal, X, Zap } from "lucide-react";

interface McpToolDef {
  name: string;
  description: string;
  annotations?: {
    readOnlyHint?: boolean;
  };
}

interface McpStatus {
  serverName: string;
  serverVersion: string;
  protocolVersion: string;
  maxConcurrentRpcs: number;
  cliCommand: string;
  cliInstalled?: boolean;
  capabilities?: {
    hardware?: {
      cpuCores: number;
      ffmpegAvailable: boolean;
      gpuBackendName?: string | null;
    };
  };
  tools: McpToolDef[];
}

interface McpModalProps {
  onClose: () => void;
}

export function McpModal({ onClose }: McpModalProps) {
  const [status, setStatus] = useState<McpStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [allowedDir, setAllowedDir] = useState<string>("");
  const [transport, setTransport] = useState<"stdio" | "http">("stdio");
  const [httpAddr, setHttpAddr] = useState("127.0.0.1:3847");
  const [copied, setCopied] = useState<"json" | "cmd" | "pipeline" | null>(null);

  const loadStatus = async () => {
    setLoading(true);
    try {
      const res = await invoke<McpStatus>("get_mcp_status");
      setStatus(res);
    } catch (err) {
      console.error("Failed to probe MCP status:", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void loadStatus();
  }, []);

  const pickAllowedDir = async () => {
    const dir = await open({
      multiple: false,
      directory: true,
      title: "Select sandbox allowed folder for MCP server",
    });
    if (typeof dir === "string") setAllowedDir(dir);
  };

  const cliCmd = status?.cliCommand || "wandel";
  const mcpArgs = [
    "mcp",
    ...(transport === "http" ? ["--http", httpAddr.trim() || "127.0.0.1:3847"] : []),
    ...(allowedDir.trim() ? ["--allowed-dir", allowedDir.trim()] : []),
  ];
  const mcpConfigJson =
    transport === "stdio"
      ? JSON.stringify(
          {
            mcpServers: {
              wandel: {
                command: cliCmd,
                args: mcpArgs,
              },
            },
          },
          null,
          2
        )
      : JSON.stringify(
          {
            mcpServers: {
              wandel: {
                url: `http://${httpAddr.trim() || "127.0.0.1:3847"}/mcp`,
                transport: "http",
              },
            },
          },
          null,
          2
        );

  const pipelineExampleJson = JSON.stringify(
    {
      steps: [
        {
          tool: "wandel_convert",
          arguments: {
            inputs: ["./photos"],
            to: "webp",
            quality: 82,
            autoTrim: true,
            autoLevels: true,
          },
        },
        {
          tool: "wandel_pdf_ops",
          arguments: {
            action: "merge_images",
            inputs: "$prev",
            output: "./album.pdf",
            pageFit: "a4",
          },
        },
        {
          tool: "wandel_pdf_ops",
          arguments: {
            action: "number",
            input: "$prev",
            output: "./album_numbered.pdf",
            text: "Page {n} of {total}",
          },
        },
      ],
    },
    null,
    2
  );

  const copyText = (text: string, kind: "json" | "cmd" | "pipeline") => {
    void navigator.clipboard.writeText(text);
    setCopied(kind);
    setTimeout(() => setCopied(null), 1800);
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4 backdrop-blur-xs"
      onClick={onClose}
    >
      <div
        className="flex max-h-[85vh] w-full max-w-2xl flex-col overflow-hidden rounded-2xl border border-zinc-200 bg-white shadow-2xl dark:border-zinc-800 dark:bg-zinc-950"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between border-b border-zinc-200 px-5 py-3.5 dark:border-zinc-800">
          <div className="flex items-center gap-2.5">
            <div className="flex h-8 w-8 items-center justify-center rounded-xl bg-indigo-500/15 text-indigo-500">
              <Plug className="h-4 w-4" />
            </div>
            <div>
              <h2 className="text-sm font-bold text-zinc-900 dark:text-zinc-100">
                Wandel MCP Server (Headless CLI)
              </h2>
              <p className="text-[11px] text-zinc-500">
                Model Context Protocol · Connects directly to the CLI and never opens the UI
              </p>
            </div>
          </div>
          <div className="flex items-center gap-1.5">
            <button
              type="button"
              onClick={() => void loadStatus()}
              className="rounded-lg p-1.5 text-zinc-400 transition hover:bg-zinc-100 hover:text-zinc-700 dark:hover:bg-zinc-900 dark:hover:text-zinc-200 cursor-pointer"
              title="Re-verify MCP server health"
            >
              <RefreshCw className={`h-4 w-4 ${loading ? "animate-spin" : ""}`} />
            </button>
            <button
              type="button"
              onClick={onClose}
              className="rounded-lg p-1.5 text-zinc-400 transition hover:bg-zinc-100 hover:text-zinc-700 dark:hover:bg-zinc-900 dark:hover:text-zinc-200 cursor-pointer"
            >
              <X className="h-4 w-4" />
            </button>
          </div>
        </div>

        {/* Body */}
        <div className="flex-1 space-y-4 overflow-y-auto p-5 text-xs">
          {/* Health Badges (unified Wandel indigo palette) */}
          {status && (
            <div className="flex flex-wrap items-center gap-2">
              <span className="inline-flex items-center gap-1 rounded-full bg-indigo-500/15 px-2.5 py-1 text-[11px] font-semibold text-indigo-600 dark:text-indigo-400">
                <Check className="h-3 w-3" />
                Ready · {status.serverName} v{status.serverVersion}
              </span>
              <span className="rounded-full bg-indigo-500/10 px-2.5 py-1 text-[11px] font-medium text-indigo-600 dark:text-indigo-400">
                MCP {status.protocolVersion}
              </span>
              <span className="inline-flex items-center gap-1 rounded-full bg-indigo-500/10 px-2.5 py-1 text-[11px] font-medium text-indigo-600 dark:text-indigo-400">
                <Cpu className="h-3 w-3" />
                Up to {status.maxConcurrentRpcs} concurrent RPCs
              </span>
              {status.capabilities?.hardware?.gpuBackendName && (
                <span className="inline-flex items-center gap-1 rounded-full bg-indigo-500/10 px-2.5 py-1 text-[11px] font-medium text-indigo-600 dark:text-indigo-400">
                  <Zap className="h-3 w-3" />
                  {status.capabilities.hardware.gpuBackendName}
                </span>
              )}
            </div>
          )}

          {/* Headless CLI Detection Notice */}
          {status && (
            <div className="rounded-xl border border-indigo-500/25 bg-indigo-500/5 px-3 py-2 text-[11px] text-zinc-700 dark:text-zinc-300">
              {status.cliInstalled ? (
                <span>
                  Headless CLI detected at{" "}
                  <code className="font-mono font-semibold text-indigo-600 dark:text-indigo-400">
                    {status.cliCommand}
                  </code>
                  . Claude Desktop and other MCP clients run this CLI directly without opening the Wandel app window.
                </span>
              ) : (
                <span>
                  Install the standalone headless <code className="font-mono font-semibold">wandel</code> CLI so Claude Desktop connects without opening the UI:{" "}
                  <code className="font-mono text-indigo-600 dark:text-indigo-400">
                    curl -fsSL https://raw.githubusercontent.com/meghrazchi/wandel/main/packaging/install.sh | bash
                  </code>
                </span>
              )}
            </div>
          )}

          {/* Transport Mode Switcher */}
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="inline-flex rounded-lg border border-zinc-200 bg-zinc-100/80 p-0.5 dark:border-zinc-800 dark:bg-zinc-900">
              <button
                type="button"
                onClick={() => setTransport("stdio")}
                className={`rounded-md px-2.5 py-1 text-[11px] font-semibold transition cursor-pointer ${
                  transport === "stdio"
                    ? "bg-white text-indigo-600 shadow-xs dark:bg-zinc-800 dark:text-indigo-400"
                    : "text-zinc-500 hover:text-zinc-800 dark:hover:text-zinc-200"
                }`}
              >
                Stdio (Desktop Clients)
              </button>
              <button
                type="button"
                onClick={() => setTransport("http")}
                className={`rounded-md px-2.5 py-1 text-[11px] font-semibold transition cursor-pointer ${
                  transport === "http"
                    ? "bg-white text-indigo-600 shadow-xs dark:bg-zinc-800 dark:text-indigo-400"
                    : "text-zinc-500 hover:text-zinc-800 dark:hover:text-zinc-200"
                }`}
              >
                Local HTTP (Docker / Remote)
              </button>
            </div>
            {transport === "http" && (
              <input
                type="text"
                value={httpAddr}
                onChange={(e) => setHttpAddr(e.target.value)}
                placeholder="127.0.0.1:3847"
                className="w-40 rounded-lg border border-zinc-200 bg-white px-2.5 py-1 font-mono text-[11px] text-zinc-800 outline-none focus:border-indigo-500 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-200"
              />
            )}
          </div>

          {/* Client Config JSON */}
          <div className="space-y-1.5">
            <div className="flex items-center justify-between">
              <span className="font-semibold text-zinc-700 dark:text-zinc-300">
                MCP Client Config (Antigravity, Claude Desktop, Cursor, Windsurf)
              </span>
              <button
                type="button"
                onClick={() => copyText(mcpConfigJson, "json")}
                className="inline-flex items-center gap-1 rounded-lg border border-zinc-200 bg-white px-2 py-1 text-[11px] font-semibold text-zinc-700 transition hover:bg-zinc-100 dark:border-zinc-800 dark:bg-zinc-900 dark:text-zinc-200 cursor-pointer"
              >
                {copied === "json" ? (
                  <>
                    <Check className="h-3 w-3 text-emerald-500" /> Copied
                  </>
                ) : (
                  <>
                    <Copy className="h-3 w-3" /> Copy JSON
                  </>
                )}
              </button>
            </div>
            <pre className="overflow-x-auto rounded-xl border border-zinc-200 bg-zinc-950 p-3 font-mono text-[11px] text-zinc-100 dark:border-zinc-800">
              {mcpConfigJson}
            </pre>
          </div>

          {/* Optional Sandbox Directory Scope */}
          <div className="flex items-center gap-2 rounded-xl border border-zinc-200 bg-zinc-50 px-3 py-2 dark:border-zinc-800 dark:bg-zinc-900/60">
            <FolderLock className="h-3.5 w-3.5 shrink-0 text-emerald-500" />
            <input
              type="text"
              value={allowedDir}
              onChange={(e) => setAllowedDir(e.target.value)}
              placeholder="Optional --allowed-dir sandbox folder (unrestricted if empty)"
              className="min-w-0 flex-1 bg-transparent text-[11px] text-zinc-800 outline-none placeholder:text-zinc-400 dark:text-zinc-200"
            />
            <button
              type="button"
              onClick={() => void pickAllowedDir()}
              className="rounded-md border border-zinc-200 bg-white px-2 py-0.5 text-[10px] font-semibold text-zinc-700 hover:bg-zinc-100 dark:border-zinc-700 dark:bg-zinc-800 dark:text-zinc-200 cursor-pointer"
            >
              Browse
            </button>
          </div>

          {/* Headless CLI Command */}
          <div className="flex items-center justify-between gap-2 rounded-xl border border-zinc-200 bg-zinc-50 px-3 py-2 dark:border-zinc-800 dark:bg-zinc-900/60">
            <div className="flex min-w-0 items-center gap-2 font-mono text-[11px] text-zinc-700 dark:text-zinc-300">
              <Terminal className="h-3.5 w-3.5 shrink-0 text-indigo-500" />
              <span className="truncate">{cliCmd} {mcpArgs.join(" ")}</span>
            </div>
            <div className="flex shrink-0 items-center gap-2">
              <button
                type="button"
                onClick={() => copyText(pipelineExampleJson, "pipeline")}
                className="text-[11px] font-semibold text-emerald-600 hover:underline dark:text-emerald-400 cursor-pointer"
              >
                {copied === "pipeline" ? "Copied Recipe!" : "Copy Pipeline Example"}
              </button>
              <button
                type="button"
                onClick={() => copyText(`${cliCmd} ${mcpArgs.join(" ")}`, "cmd")}
                className="text-[11px] font-semibold text-indigo-600 hover:underline dark:text-indigo-400 cursor-pointer"
              >
                {copied === "cmd" ? "Copied!" : "Copy Command"}
              </button>
            </div>
          </div>

          {/* Tool Catalog */}
          {status?.tools && (
            <div className="space-y-2">
              <div className="font-semibold text-zinc-700 dark:text-zinc-300">
                Exposed Tools ({status.tools.length})
              </div>
              <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                {status.tools.map((tool) => (
                  <div
                    key={tool.name}
                    className="rounded-xl border border-zinc-200 bg-white p-2.5 dark:border-zinc-800 dark:bg-zinc-900/70"
                  >
                    <div className="flex items-center justify-between gap-2">
                      <code className="font-mono text-[11px] font-bold text-indigo-600 dark:text-indigo-400">
                        {tool.name}
                      </code>
                      <span className="rounded bg-zinc-100 px-1.5 py-0.5 text-[9px] font-semibold uppercase text-zinc-500 dark:bg-zinc-800 dark:text-zinc-400">
                        {tool.annotations?.readOnlyHint ? "Read-Only" : "Parallel"}
                      </span>
                    </div>
                    <p className="mt-1 line-clamp-2 text-[11px] text-zinc-500 dark:text-zinc-400">
                      {tool.description}
                    </p>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
