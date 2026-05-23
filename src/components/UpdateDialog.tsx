import { useState, useRef, useEffect } from "react";
import { call as invoke } from "@/lib/transport";
import { listen } from "@tauri-apps/api/event";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import { ArrowUpCircle, Download, CheckCircle2, XCircle, Loader2, Sparkles, Bug, Zap } from "lucide-react";

export interface UpdateInfo {
  current_version: string;
  new_version: string;
  release_notes: string;
  download_url: string;
  release_url: string;
}

type Phase = "idle" | "downloading" | "done" | "error";

interface Props {
  info: UpdateInfo;
  onDismiss: () => void;
  onSkip?: () => void;
}

// ── Release notes parser ─────────────────────────────────────────────────────

interface Section {
  heading: string | null;
  items: string[];
}

function parseReleaseNotes(body: string): Section[] {
  const sections: Section[] = [];
  let current: Section = { heading: null, items: [] };

  for (const raw of body.split("\n")) {
    const line = raw.trim();
    if (!line) continue;

    if (/^#{1,3}\s/.test(line)) {
      if (current.heading !== null || current.items.length > 0) sections.push(current);
      current = { heading: line.replace(/^#+\s*/, ""), items: [] };
    } else if (/^[-*]\s/.test(line)) {
      current.items.push(line.slice(2).trim());
    } else if (current.heading !== null) {
      current.items.push(line);
    }
  }

  if (current.heading !== null || current.items.length > 0) sections.push(current);
  return sections.filter((s) => s.items.length > 0);
}

function sectionMeta(heading: string | null): { Icon: React.ElementType; color: string } {
  const h = (heading ?? "").toLowerCase();
  if (/✨|新功能|新增|feature|new/.test(h))
    return { Icon: Sparkles, color: "text-violet-500" };
  if (/🐛|🔧|修復|修正|fix|bug/.test(h))
    return { Icon: Bug, color: "text-rose-500" };
  if (/⚡|改善|優化|improve|perf|performance/.test(h))
    return { Icon: Zap, color: "text-amber-500" };
  return { Icon: ArrowUpCircle, color: "text-muted-foreground" };
}

// ── Main component ────────────────────────────────────────────────────────────

export function UpdateDialog({ info, onDismiss, onSkip }: Props) {
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

  const sections = parseReleaseNotes(info.release_notes);
  const hasSections = sections.length > 0;

  // Fallback: plain lines if no markdown structure detected
  const plainLines = !hasSections
    ? info.release_notes.split("\n").map((l) => l.trim()).filter(Boolean).slice(0, 12)
    : [];

  return (
    <Dialog open onOpenChange={(o) => { if (!o && phase === "idle") onDismiss(); }}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <ArrowUpCircle className="h-5 w-5 text-primary" />
            版本更新
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-1">
          {/* Version badges */}
          <div className="flex items-center gap-3">
            <Badge variant="secondary" className="font-mono">v{info.current_version}</Badge>
            <span className="text-muted-foreground text-sm">→</span>
            <Badge variant="default" className="font-mono">v{info.new_version}</Badge>
          </div>

          {/* Release notes */}
          {(hasSections || plainLines.length > 0) && (
            <div className="rounded-lg border bg-muted/30 p-3 space-y-3 max-h-52 overflow-y-auto text-sm">
              {hasSections
                ? sections.map((sec, i) => {
                    const { Icon, color } = sectionMeta(sec.heading);
                    return (
                      <div key={i} className="space-y-1.5">
                        {sec.heading && (
                          <p className={`flex items-center gap-1.5 text-xs font-semibold ${color}`}>
                            <Icon className="h-3.5 w-3.5 shrink-0" />
                            {sec.heading}
                          </p>
                        )}
                        <ul className="space-y-1 pl-1">
                          {sec.items.map((item, j) => (
                            <li key={j} className="flex items-start gap-1.5 text-xs text-muted-foreground">
                              <span className="mt-1.5 h-1 w-1 shrink-0 rounded-full bg-muted-foreground/50" />
                              {item}
                            </li>
                          ))}
                        </ul>
                      </div>
                    );
                  })
                : plainLines.map((line, i) => (
                    <p key={i} className="text-xs text-muted-foreground leading-relaxed">{line}</p>
                  ))
              }
            </div>
          )}

          {/* Progress states */}
          {phase === "downloading" && (
            <div className="space-y-2">
              <div className="flex items-center justify-between text-xs text-muted-foreground">
                <span className="flex items-center gap-1.5">
                  <Loader2 className="h-3 w-3 animate-spin" />
                  正在下載...
                </span>
                <span>{progress}%</span>
              </div>
              <Progress value={progress} className="h-2" />
            </div>
          )}

          {phase === "done" && (
            <div className="flex items-center gap-2 rounded-lg bg-emerald-500/10 p-3 text-sm text-emerald-600">
              <CheckCircle2 className="h-4 w-4 shrink-0" />
              下載完成！安裝程式已自動開啓，請按提示完成安裝。
            </div>
          )}

          {phase === "error" && (
            <div className="flex items-start gap-2 rounded-lg bg-destructive/10 p-3 text-sm text-destructive">
              <XCircle className="h-4 w-4 mt-0.5 shrink-0" />
              <div>
                <p className="font-medium">下載失敗</p>
                <p className="mt-0.5 text-xs opacity-80">{errorMsg}</p>
              </div>
            </div>
          )}
        </div>

        <DialogFooter className="gap-2">
          {phase === "idle" && (
            <>
              {onSkip && (
                <Button variant="ghost" size="sm" onClick={onSkip}>跳過此版本</Button>
              )}
              <Button variant="outline" size="sm" onClick={onDismiss}>稍後</Button>
              <Button onClick={handleUpdate}>
                <Download className="mr-2 h-4 w-4" />立即更新
              </Button>
            </>
          )}
          {phase === "downloading" && (
            <Button disabled>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />下載中...
            </Button>
          )}
          {phase === "done" && (
            <Button onClick={onDismiss}>
              <CheckCircle2 className="mr-2 h-4 w-4" />關閉
            </Button>
          )}
          {phase === "error" && (
            <>
              <a
                href={info.release_url}
                target="_blank"
                rel="noreferrer"
                className="inline-flex items-center gap-1 rounded-md border px-3 py-1.5 text-sm font-medium transition-colors hover:bg-accent"
              >
                手動下載
              </a>
              <Button onClick={handleUpdate}>重試</Button>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
