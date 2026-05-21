import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { Settings, Database, Wifi, WifiOff, Monitor, Copy, Bug, RefreshCw, Trash2,
         ArrowUpCircle, Sparkles, Bug as BugIcon, Zap, HardDrive, Loader2 } from "lucide-react";
import { toast } from "sonner";
import { appLogger, type LogEntry, type ErrorCategory } from "@/lib/logger";
import { UpdateDialog } from "@/components/UpdateDialog";
import type { UpdateInfo } from "@/components/UpdateDialog";
import { PENDING_KEY } from "@/hooks/useAutoUpdate";

const AUTO_UPDATE_KEY = "cuckoo_auto_update";
const SKIP_KEY = "cuckoo_skipped_version";

interface SettingsPageProps {
  connected: boolean;
}

// ── Category display config ──────────────────────────────────────────────────

const CATEGORY_CONFIG: Record<ErrorCategory, { label: string; className: string }> = {
  db:         { label: "数据库",   className: "bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300" },
  not_found:  { label: "查无数据", className: "bg-blue-100 text-blue-800 dark:bg-blue-900/40 dark:text-blue-300" },
  validation: { label: "参数错误", className: "bg-orange-100 text-orange-800 dark:bg-orange-900/40 dark:text-orange-300" },
  ipc:        { label: "IPC 连线", className: "bg-red-100 text-red-800 dark:bg-red-900/40 dark:text-red-300" },
  print:      { label: "打印机",   className: "bg-zinc-100 text-zinc-600 dark:bg-zinc-800 dark:text-zinc-400" },
  render:     { label: "界面崩溃", className: "bg-red-100 text-red-800 dark:bg-red-900/40 dark:text-red-300" },
  runtime:    { label: "JS 运行期", className: "bg-purple-100 text-purple-800 dark:bg-purple-900/40 dark:text-purple-300" },
  logic:      { label: "业务逻辑", className: "bg-muted text-muted-foreground" },
};

const FILTER_OPTIONS: Array<{ key: ErrorCategory | "all"; label: string }> = [
  { key: "all",      label: "全部" },
  { key: "db",       label: "数据库" },
  { key: "ipc",      label: "IPC" },
  { key: "runtime",  label: "JS" },
  { key: "render",   label: "崩溃" },
  { key: "logic",    label: "逻辑" },
  { key: "print",    label: "打印机" },
];

function timeAgo(isoStr: string): string {
  const diff = Date.now() - new Date(isoStr).getTime();
  const m = Math.floor(diff / 60000);
  if (m < 1) return "刚才";
  if (m < 60) return `${m} 分钟前`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h} 小时前`;
  return `${Math.floor(h / 24)} 天前`;
}

// ── Release notes renderer (shared with banner) ──────────────────────────────

interface ReleaseSection { heading: string | null; items: string[] }

function parseNotes(body: string): ReleaseSection[] {
  const sections: ReleaseSection[] = [];
  let cur: ReleaseSection = { heading: null, items: [] };
  for (const raw of body.split("\n")) {
    const line = raw.trim();
    if (!line) continue;
    if (/^#{1,3}\s/.test(line)) {
      if (cur.heading !== null || cur.items.length > 0) sections.push(cur);
      cur = { heading: line.replace(/^#+\s*/, ""), items: [] };
    } else if (/^[-*]\s/.test(line)) {
      cur.items.push(line.slice(2).trim());
    } else if (cur.heading !== null) {
      cur.items.push(line);
    }
  }
  if (cur.heading !== null || cur.items.length > 0) sections.push(cur);
  return sections.filter((s) => s.items.length > 0);
}

function sectionIcon(heading: string | null) {
  const h = (heading ?? "").toLowerCase();
  if (/✨|新功能|新增|feature|new/.test(h)) return <Sparkles className="h-3.5 w-3.5 text-violet-500" />;
  if (/🐛|🔧|修復|修正|fix|bug/.test(h)) return <BugIcon className="h-3.5 w-3.5 text-rose-500" />;
  if (/⚡|改善|優化|improve|perf/.test(h)) return <Zap className="h-3.5 w-3.5 text-amber-500" />;
  return <ArrowUpCircle className="h-3.5 w-3.5 text-muted-foreground" />;
}

// ── Pending update card ───────────────────────────────────────────────────────

function PendingUpdateCard() {
  const [info, setInfo] = useState<UpdateInfo | null>(null);
  const [showDialog, setShowDialog] = useState(false);

  useEffect(() => {
    const raw = localStorage.getItem(PENDING_KEY);
    if (raw) {
      try { setInfo(JSON.parse(raw)); } catch { /* ignore */ }
    }
  }, []);

  function handleSkip() {
    if (!info) return;
    localStorage.setItem(SKIP_KEY, info.new_version);
    localStorage.removeItem(PENDING_KEY);
    setInfo(null);
  }

  function handleDismissDialog() {
    setShowDialog(false);
    // If update was completed, clear pending
    const raw = localStorage.getItem(PENDING_KEY);
    if (!raw) setInfo(null);
  }

  if (!info) return null;

  const sections = parseNotes(info.release_notes);
  const plainLines = sections.length === 0
    ? info.release_notes.split("\n").map((l) => l.trim()).filter(Boolean).slice(0, 8)
    : [];

  return (
    <>
      <Card className="border-primary/30 bg-primary/5">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center justify-between gap-2 text-base">
            <span className="flex items-center gap-2">
              <ArrowUpCircle className="h-4 w-4 text-primary" />
              有可用更新
            </span>
            <div className="flex items-center gap-2">
              <Badge variant="secondary" className="font-mono text-xs">v{info.current_version}</Badge>
              <span className="text-xs text-muted-foreground">→</span>
              <Badge variant="default" className="font-mono text-xs">v{info.new_version}</Badge>
            </div>
          </CardTitle>
          <CardDescription>此更新在之前的提示中被暫緩，可在此繼續安裝。</CardDescription>
        </CardHeader>

        <CardContent className="space-y-4">
          {/* Release notes */}
          {(sections.length > 0 || plainLines.length > 0) && (
            <div className="rounded-lg border bg-background/60 p-3 space-y-3 max-h-52 overflow-y-auto text-sm">
              {sections.length > 0
                ? sections.map((sec, i) => (
                    <div key={i} className="space-y-1.5">
                      {sec.heading && (
                        <p className="flex items-center gap-1.5 text-xs font-semibold text-foreground">
                          {sectionIcon(sec.heading)}
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
                  ))
                : plainLines.map((line, i) => (
                    <p key={i} className="text-xs text-muted-foreground leading-relaxed">{line}</p>
                  ))
              }
            </div>
          )}

          <div className="flex items-center gap-2">
            <Button variant="ghost" size="sm" className="text-xs text-muted-foreground" onClick={handleSkip}>
              跳過此版本
            </Button>
            <Button size="sm" className="flex-1 gap-1.5" onClick={() => setShowDialog(true)}>
              <ArrowUpCircle className="h-3.5 w-3.5" />
              立即更新
            </Button>
          </div>
        </CardContent>
      </Card>

      {showDialog && (
        <UpdateDialog
          info={info}
          onDismiss={handleDismissDialog}
          onSkip={handleSkip}
        />
      )}
    </>
  );
}

// ── Error log panel ──────────────────────────────────────────────────────────

function ErrorLogPanel() {
  const [entries, setEntries] = useState<LogEntry[]>([]);
  const [filter, setFilter] = useState<ErrorCategory | "all">("all");
  const [expanded, setExpanded] = useState<string | null>(null);

  const refresh = useCallback(() => {
    setEntries(appLogger.getRecent(60));
  }, []);

  useEffect(() => {
    refresh();
    // Stay in sync when new errors arrive while user is on this page
    const handler = () => refresh();
    window.addEventListener("cuckoo:logged", handler);
    return () => window.removeEventListener("cuckoo:logged", handler);
  }, [refresh]);

  function handleClear() {
    appLogger.clear();
    setEntries([]);
    toast.success("错误记录已清除");
  }

  function handleCopyReport() {
    const recent = appLogger.getRecent(60);
    const lines = [
      "=== Cuckoo 错误报告 ===",
      `生成时间: ${new Date().toISOString()}`,
      `版本: 1.2.2`,
      `平台: ${navigator.platform}`,
      `UA: ${navigator.userAgent}`,
      `URL: ${location.href}`,
      "",
      "--- 结构化日志 (JSON) ---",
      JSON.stringify(recent, null, 2),
      "=== 报告结束 ===",
    ].join("\n");

    navigator.clipboard.writeText(lines)
      .then(() => toast.success("报告已复制，可发给开发者"))
      .catch(() => toast.error("复制失败，请手动截图"));
  }

  const filtered = filter === "all" ? entries : entries.filter((e) => e.category === filter);
  const counts = entries.reduce<Partial<Record<ErrorCategory | "all", number>>>((acc, e) => {
    acc.all = (acc.all ?? 0) + 1;
    acc[e.category] = (acc[e.category] ?? 0) + 1;
    return acc;
  }, {});

  return (
    <div className="space-y-3">
      {/* Summary bar */}
      <div className="flex items-center justify-between gap-2 flex-wrap">
        <div className="flex items-center gap-1.5 flex-wrap">
          {FILTER_OPTIONS.map(({ key, label }) => {
            const count = counts[key] ?? 0;
            if (key !== "all" && count === 0) return null;
            return (
              <button
                key={key}
                onClick={() => setFilter(key)}
                className={`inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-xs font-medium transition-colors
                  ${filter === key
                    ? "bg-foreground text-background"
                    : "bg-muted text-muted-foreground hover:bg-muted/80"
                  }`}
              >
                {label}
                {count > 0 && <span className="opacity-70">{count}</span>}
              </button>
            );
          })}
        </div>
        <div className="flex items-center gap-1">
          <Button variant="ghost" size="sm" className="h-7 gap-1 text-xs" onClick={refresh}>
            <RefreshCw className="h-3 w-3" />
            刷新
          </Button>
          <Button variant="ghost" size="sm" className="h-7 gap-1 text-xs text-muted-foreground" onClick={handleClear}>
            <Trash2 className="h-3 w-3" />
            清除
          </Button>
        </div>
      </div>

      {/* Log list */}
      {filtered.length === 0 ? (
        <div className="flex items-center justify-center rounded-lg border border-dashed py-10 text-sm text-muted-foreground">
          {entries.length === 0 ? "暂无错误记录" : "此分类无记录"}
        </div>
      ) : (
        <ScrollArea className="h-72 rounded-md border">
          <div className="divide-y">
            {filtered.map((entry) => {
              const cfg = CATEGORY_CONFIG[entry.category];
              const isExpanded = expanded === entry.id;
              return (
                <div
                  key={entry.id}
                  className="px-3 py-2 hover:bg-accent/30 cursor-pointer transition-colors"
                  onClick={() => setExpanded(isExpanded ? null : entry.id)}
                >
                  <div className="flex items-start gap-2">
                    <span className={`mt-0.5 shrink-0 rounded px-1.5 py-0.5 text-[10px] font-medium ${cfg.className}`}>
                      {cfg.label}
                    </span>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center justify-between gap-2">
                        <span className="text-xs font-mono text-muted-foreground truncate">{entry.operation}</span>
                        <span className="shrink-0 text-[10px] text-muted-foreground/60">{timeAgo(entry.ts)}</span>
                      </div>
                      <p className="text-xs mt-0.5 break-all">{entry.message}</p>
                      {isExpanded && (
                        <div className="mt-2 space-y-1">
                          {entry.context && (
                            <pre className="text-[10px] font-mono bg-muted/60 rounded p-2 overflow-x-auto whitespace-pre-wrap">
                              {JSON.stringify(entry.context, null, 2)}
                            </pre>
                          )}
                          {entry.stack && (
                            <pre className="text-[10px] font-mono bg-muted/60 rounded p-2 overflow-x-auto whitespace-pre-wrap text-muted-foreground">
                              {entry.stack}
                            </pre>
                          )}
                          <p className="text-[10px] text-muted-foreground/50">{entry.ts}</p>
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </ScrollArea>
      )}

      {/* Copy report button */}
      <Button onClick={handleCopyReport} className="w-full gap-2" variant="outline">
        <Copy className="h-4 w-4" />
        复制完整错误报告（供开发者诊断）
      </Button>
    </div>
  );
}

// ── Main page ────────────────────────────────────────────────────────────────

export function SettingsPage({ connected }: SettingsPageProps) {
  const [dbStatus, setDbStatus] = useState<string>("檢查中...");
  const [appVersion, setAppVersion] = useState<string>("...");
  const [autoUpdate, setAutoUpdate] = useState(
    localStorage.getItem(AUTO_UPDATE_KEY) !== "false"
  );
  const [autoBackup, setAutoBackup] = useState(
    localStorage.getItem("cuckoo_auto_backup") === "true"
  );
  const [backupStatus, setBackupStatus] = useState<string | null>(null);
  const [backupLoading, setBackupLoading] = useState(false);

  useEffect(() => {
    invoke<string>("health_check")
      .then((r) => setDbStatus(r === "ok" ? "正常" : "異常"))
      .catch(() => setDbStatus("連線失敗"));
    invoke<string>("get_app_version")
      .then(setAppVersion)
      .catch(() => {});
  }, []);

  function handleAutoUpdateToggle(val: boolean) {
    setAutoUpdate(val);
    localStorage.setItem(AUTO_UPDATE_KEY, val ? "true" : "false");
    if (val) localStorage.removeItem(SKIP_KEY);
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-semibold tracking-tight">系统设置</h2>
        <p className="text-sm text-muted-foreground">系统信息与故障诊断</p>
      </div>

      {/* System Info */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Settings className="h-4 w-4" />
            系统信息
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-1">
          <div className="flex items-center justify-between py-2">
            <div className="flex items-center gap-3">
              <Monitor className="h-4 w-4 text-muted-foreground" />
              <div>
                <p className="text-sm font-medium">应用版本</p>
                <p className="text-xs text-muted-foreground">当前安装的版本</p>
              </div>
            </div>
            <span className="text-sm font-mono text-muted-foreground">v{appVersion}</span>
          </div>
          <Separator />
          <div className="flex items-center justify-between py-2">
            <div className="flex items-center gap-3">
              <Database className="h-4 w-4 text-muted-foreground" />
              <div>
                <p className="text-sm font-medium">数据库状态</p>
                <p className="text-xs text-muted-foreground">SQLite 本地存储</p>
              </div>
            </div>
            <Badge variant={dbStatus === "正常" ? "secondary" : "destructive"} className="text-xs">
              {dbStatus}
            </Badge>
          </div>
          <Separator />
          <div className="flex items-center justify-between py-2">
            <div className="flex items-center gap-3">
              {connected ? (
                <Wifi className="h-4 w-4 text-emerald-500" />
              ) : (
                <WifiOff className="h-4 w-4 text-destructive" />
              )}
              <div>
                <p className="text-sm font-medium">后端连线</p>
                <p className="text-xs text-muted-foreground">Tauri IPC 状态</p>
              </div>
            </div>
            <span className={`text-sm font-medium ${connected ? "text-emerald-500" : "text-destructive"}`}>
              {connected ? "已连线" : "未连线"}
            </span>
          </div>
        </CardContent>
      </Card>

      {/* Pending update card (shown when user dismissed the banner) */}
      <PendingUpdateCard />

      {/* Auto-update settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <ArrowUpCircle className="h-4 w-4" />
            自動更新
          </CardTitle>
          <CardDescription>應用啓動後自動檢查 GitHub 是否有新版本</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <Label htmlFor="auto-update-switch" className="flex flex-col gap-0.5 cursor-pointer">
              <span className="text-sm font-medium">啓動時自動檢查更新</span>
              <span className="text-xs text-muted-foreground">有新版本時會在右下角彈出提示，可選擇立即更新或稍後在此查看</span>
            </Label>
            <Switch
              id="auto-update-switch"
              checked={autoUpdate}
              onCheckedChange={handleAutoUpdateToggle}
            />
          </div>
        </CardContent>
      </Card>

      {/* Backup */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <HardDrive className="h-4 w-4" />
            数据备份
          </CardTitle>
          <CardDescription>备份文件保存至 Documents/Cuckoo 备份/</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <Label htmlFor="auto-backup-switch" className="flex flex-col gap-0.5 cursor-pointer">
              <span className="text-sm font-medium">启动时自动备份</span>
              <span className="text-xs text-muted-foreground">每次打开应用时自动创建一份带时间戳的备份</span>
            </Label>
            <Switch
              id="auto-backup-switch"
              checked={autoBackup}
              onCheckedChange={(val) => {
                setAutoBackup(val);
                localStorage.setItem("cuckoo_auto_backup", val ? "true" : "false");
              }}
            />
          </div>
          <Separator />
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium">立即备份</p>
              {backupStatus && (
                <p className="text-xs text-muted-foreground mt-0.5 max-w-xs truncate">{backupStatus}</p>
              )}
            </div>
            <Button
              variant="outline"
              size="sm"
              disabled={backupLoading}
              onClick={async () => {
                setBackupLoading(true);
                setBackupStatus(null);
                try {
                  const path = await invoke<string>("backup_database", { destDir: null });
                  setBackupStatus(`已备份到: ${path}`);
                  toast.success("备份成功");
                } catch (e) {
                  setBackupStatus(String(e));
                  toast.error("备份失败", { description: String(e) });
                } finally {
                  setBackupLoading(false);
                }
              }}
            >
              {backupLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : <HardDrive className="h-4 w-4" />}
              <span className="ml-2">立即备份</span>
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Error Log Panel */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Bug className="h-4 w-4" />
            错误记录
          </CardTitle>
          <CardDescription>
            应用运行期间自动收集的前端错误。点击任一条目可展开详情，复制报告后发给开发者。
          </CardDescription>
        </CardHeader>
        <CardContent>
          <ErrorLogPanel />
        </CardContent>
      </Card>
    </div>
  );
}
