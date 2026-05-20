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
         ArrowUpCircle, CheckCircle2, Loader2 } from "lucide-react";
import { toast } from "sonner";
import { appLogger, type LogEntry, type ErrorCategory } from "@/lib/logger";
import { UpdateDialog } from "@/components/UpdateDialog";
import type { UpdateInfo } from "@/components/UpdateDialog";

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
  const [checking, setChecking] = useState(false);
  const [manualUpdateInfo, setManualUpdateInfo] = useState<UpdateInfo | null>(null);
  const [latestChecked, setLatestChecked] = useState(false);

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

  async function handleCheckNow() {
    setChecking(true);
    setLatestChecked(false);
    setManualUpdateInfo(null);
    try {
      const info = await invoke<UpdateInfo | null>("check_for_update");
      if (info) {
        setManualUpdateInfo(info);
      } else {
        setLatestChecked(true);
        toast.success("已是最新版本");
      }
    } catch (e) {
      toast.error("检查失败", { description: String(e) });
    } finally {
      setChecking(false);
    }
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

      {/* Auto-update settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <ArrowUpCircle className="h-4 w-4" />
            自动更新
          </CardTitle>
          <CardDescription>应用启动后自动检查 GitHub 是否有新版本</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <Label htmlFor="auto-update-switch" className="flex flex-col gap-0.5 cursor-pointer">
              <span className="text-sm font-medium">启动时自动检查更新</span>
              <span className="text-xs text-muted-foreground">有新版本时会弹窗提示，可选择立即更新或跳过</span>
            </Label>
            <Switch
              id="auto-update-switch"
              checked={autoUpdate}
              onCheckedChange={handleAutoUpdateToggle}
            />
          </div>
          <Separator />
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium">立即检查</p>
              <p className="text-xs text-muted-foreground">手动检查是否有可用更新</p>
            </div>
            <div className="flex items-center gap-2">
              {latestChecked && (
                <span className="flex items-center gap-1 text-xs text-emerald-600">
                  <CheckCircle2 className="h-3 w-3" />已是最新版
                </span>
              )}
              <Button variant="outline" size="sm" onClick={handleCheckNow} disabled={checking}>
                {checking
                  ? <><Loader2 className="mr-2 h-3 w-3 animate-spin" />检查中...</>
                  : <><RefreshCw className="mr-2 h-3 w-3" />检查更新</>
                }
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {manualUpdateInfo && (
        <UpdateDialog
          info={manualUpdateInfo}
          onDismiss={() => setManualUpdateInfo(null)}
        />
      )}

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
