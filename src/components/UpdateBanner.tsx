import { useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ArrowUpCircle, X, RefreshCw, Loader2, CheckCircle2, XCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import type { UpdateInfo } from "@/components/UpdateDialog";

type Phase = "idle" | "downloading" | "done" | "error";

interface Props {
  info: UpdateInfo;
  onDismiss: () => void;
  onSkip: () => void;
}

export function UpdateBanner({ info, onDismiss, onSkip }: Props) {
  const [phase, setPhase] = useState<Phase>("idle");
  const [progress, setProgress] = useState(0);
  const [errorMsg, setErrorMsg] = useState("");
  const unlistenRef = useRef<(() => void)[]>([]);

  useEffect(() => () => { unlistenRef.current.forEach((u) => u()); }, []);

  async function handleUpdate() {
    setPhase("downloading");
    setProgress(0);
    const unlisteners: (() => void)[] = [];

    const ulP = await listen<{ downloaded: number; total: number }>("update-progress", (e) => {
      const { downloaded, total } = e.payload;
      if (total > 0) setProgress(Math.round((downloaded / total) * 100));
    });
    unlisteners.push(ulP);

    const ulD = await listen("update-complete", () => {
      setPhase("done");
      unlisteners.forEach((u) => u());
    });
    unlisteners.push(ulD);

    const ulE = await listen<string>("update-error", (e) => {
      setErrorMsg(e.payload);
      setPhase("error");
      unlisteners.forEach((u) => u());
    });
    unlisteners.push(ulE);
    unlistenRef.current = unlisteners;

    try {
      await invoke("download_and_open_update", { url: info.download_url });
    } catch (e) {
      setErrorMsg(String(e));
      setPhase("error");
      unlisteners.forEach((u) => u());
    }
  }

  const firstNote = info.release_notes
    .split("\n")
    .map((l) => l.trim())
    .find((l) => l && !l.startsWith("#") && !l.startsWith("-") && l.length > 4);

  return (
    <div className="fixed bottom-4 right-4 z-50 w-80 rounded-xl border bg-background shadow-xl animate-in slide-in-from-bottom-4 duration-300">
      <div className="p-4 space-y-3">
        {/* Header */}
        <div className="flex items-start justify-between gap-2">
          <div className="flex items-start gap-2.5">
            <div className="mt-0.5 flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-primary/10">
              <ArrowUpCircle className="h-4 w-4 text-primary" />
            </div>
            <div>
              <p className="text-sm font-semibold leading-tight">
                Cuckoo v{info.new_version} 可用
              </p>
              <p className="text-xs text-muted-foreground mt-0.5">
                目前版本：v{info.current_version}
              </p>
              {firstNote && (
                <p className="text-xs text-muted-foreground mt-1 line-clamp-2 leading-relaxed">
                  {firstNote}
                </p>
              )}
            </div>
          </div>
          {phase === "idle" && (
            <button
              onClick={onDismiss}
              className="shrink-0 rounded-md p-1 text-muted-foreground hover:bg-accent hover:text-foreground transition-colors"
              title="稍後在設定查看"
            >
              <X className="h-3.5 w-3.5" />
            </button>
          )}
        </div>

        {/* Progress */}
        {phase === "downloading" && (
          <div className="space-y-1.5">
            <div className="flex items-center justify-between text-xs text-muted-foreground">
              <span className="flex items-center gap-1">
                <Loader2 className="h-3 w-3 animate-spin" />
                下載中...
              </span>
              <span>{progress}%</span>
            </div>
            <Progress value={progress} className="h-1.5" />
          </div>
        )}

        {phase === "done" && (
          <div className="flex items-start gap-2 rounded-lg bg-emerald-500/10 px-3 py-2 text-xs text-emerald-600">
            <CheckCircle2 className="h-3.5 w-3.5 mt-0.5 shrink-0" />
            下載完成！請按提示完成安裝，之後重啓應用即可。
          </div>
        )}

        {phase === "error" && (
          <div className="flex items-start gap-2 rounded-lg bg-destructive/10 px-3 py-2 text-xs text-destructive">
            <XCircle className="h-3.5 w-3.5 mt-0.5 shrink-0" />
            {errorMsg || "下載失敗，請稍後再試。"}
          </div>
        )}

        {/* Actions */}
        <div className="flex items-center gap-2">
          {phase === "idle" && (
            <>
              <button
                onClick={onSkip}
                className="text-xs text-muted-foreground hover:text-foreground transition-colors px-1"
              >
                跳過
              </button>
              <Button size="sm" className="h-7 text-xs flex-1 gap-1.5" onClick={handleUpdate}>
                <RefreshCw className="h-3 w-3" />
                重啓以更新
              </Button>
            </>
          )}
          {phase === "downloading" && (
            <Button size="sm" className="h-7 text-xs w-full" disabled>
              <Loader2 className="mr-1.5 h-3 w-3 animate-spin" />
              下載中...
            </Button>
          )}
          {phase === "done" && (
            <Button size="sm" className="h-7 text-xs w-full" onClick={onDismiss}>
              關閉
            </Button>
          )}
          {phase === "error" && (
            <Button size="sm" variant="outline" className="h-7 text-xs w-full" onClick={handleUpdate}>
              重試
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}
