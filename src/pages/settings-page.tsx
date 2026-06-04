import { useState, useEffect, useCallback } from "react";
import { call as invoke } from "@/lib/transport";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { Settings, Database, Wifi, WifiOff, Monitor, Copy, Bug, RefreshCw, Trash2,
         ArrowUpCircle, Sparkles, Bug as BugIcon, Zap, HardDrive, Loader2, ShieldCheck, Eye, EyeOff,
         Radio, ServerCrash, Link2, Smartphone, AlertTriangle } from "lucide-react";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { type Role, ROLE_LABELS, ROLE_COLORS, ROLE_DESCRIPTIONS, getRolePinStatuses, saveRolePin } from "@/lib/roles";
import { toast } from "sonner";
import { appLogger, type LogEntry, type ErrorCategory } from "@/lib/logger";
import { UpdateDialog } from "@/components/UpdateDialog";
import type { UpdateInfo } from "@/components/UpdateDialog";
import { PENDING_KEY } from "@/hooks/useAutoUpdate";
import { open } from "@tauri-apps/plugin-dialog";

const AUTO_UPDATE_KEY = "cuckoo_auto_update";
const SKIP_KEY = "cuckoo_skipped_version";

interface SettingsPageProps {
  connected: boolean;
}

// ── Category display config ──────────────────────────────────────────────────

const CATEGORY_CONFIG: Record<ErrorCategory, { label: string; className: string }> = {
  db:         { label: "数据库",   className: "bg-muted text-muted-foreground" },
  not_found:  { label: "查无数据", className: "bg-muted text-muted-foreground" },
  validation: { label: "参数错误", className: "bg-muted text-muted-foreground" },
  ipc:        { label: "IPC 连线", className: "bg-muted text-muted-foreground" },
  print:      { label: "打印机",   className: "bg-muted text-muted-foreground" },
  render:     { label: "界面崩溃", className: "bg-destructive/10 text-destructive" },
  runtime:    { label: "JS 运行期", className: "bg-muted text-muted-foreground" },
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

function ErrorLogPanel({ appVersion }: { appVersion: string }) {
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
      `版本: ${appVersion}`,
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

// ── LAN Sync ─────────────────────────────────────────────────────────────────

const SYNC_CLIENT_URL_KEY = "cuckoo_sync_client_url";
const SYNC_CLIENT_ACTIVE_KEY = "cuckoo_sync_client_active";
const SYNC_SHARED_SECRET_KEY = "cuckoo_sync_shared_secret";
const SYNC_PROTOCOL_VERSION = "2";

interface WebServerStatus {
  running: boolean;
  port: number | null;
  url: string | null;
}

function WebServerCard() {
  const [status, setStatus] = useState<WebServerStatus>({ running: false, port: null, url: null });
  const [busy, setBusy] = useState(false);

  const refresh = () => invoke<WebServerStatus>("get_web_server_status").then(setStatus).catch(() => {});

  useEffect(() => { refresh(); }, []);

  const handleStop = async () => {
    setBusy(true);
    try {
      await invoke("stop_web_server");
      toast.success("Web 服務已停止");
      await refresh();
    } catch (e) {
      toast.error(String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleRestart = async () => {
    setBusy(true);
    try {
      await invoke("restart_web_server");
      toast.success("Web 服務已重啟");
      await refresh();
    } catch (e) {
      toast.error(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Smartphone className="h-4 w-4" />
          自助點單 Web 服務
        </CardTitle>
        <CardDescription>iPad / 手機掃碼自助點單入口，與主機在同一區域網路即可存取</CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className={`h-2 w-2 rounded-full ${status.running ? "bg-emerald-500" : "bg-muted-foreground"}`} />
            <span className="text-sm text-muted-foreground">
              {status.running ? `埠號 ${status.port} · 已啟動` : "未啟動"}
            </span>
          </div>
          <div className="flex gap-2">
            <Button size="sm" variant="outline" onClick={handleRestart} disabled={busy}>
              <RefreshCw className="h-3.5 w-3.5 mr-1.5" />
              重啟
            </Button>
            {status.running && (
              <Button size="sm" variant="destructive" onClick={handleStop} disabled={busy}>
                停止
              </Button>
            )}
          </div>
        </div>
        {status.running && status.url && (
          <>
            <div className="flex items-center gap-2 rounded-lg bg-muted p-2.5">
              <span className="text-xs font-mono flex-1 text-emerald-600 dark:text-emerald-400 break-all">
                {status.url}
              </span>
              <Button size="icon" variant="ghost" className="h-6 w-6 shrink-0" onClick={() => {
                navigator.clipboard.writeText(status.url!);
                toast.success("已複製");
              }}>
                <Copy className="h-3 w-3" />
              </Button>
            </div>
            <p className="text-xs text-muted-foreground">
              客戶掃碼點單 URL 格式：<span className="font-mono">{status.url}/#/table/桌號</span>
            </p>
          </>
        )}
        {!status.running && (
          <p className="text-xs text-muted-foreground">
            點選「重啟」重新在 9001 埠啟動，或重新啟動應用程式。
          </p>
        )}
      </CardContent>
    </Card>
  );
}

function LanSyncCard() {
  const [serverRunning, setServerRunning] = useState(false);
  const [serverPort, setServerPort] = useState("7070");
  const [serverUrl, setServerUrl] = useState("");
  const [sharedSecret, setSharedSecret] = useState(
    () => localStorage.getItem(SYNC_SHARED_SECRET_KEY) || ""
  );

  const [clientUrl, setClientUrl] = useState(
    () => localStorage.getItem(SYNC_CLIENT_URL_KEY) || ""
  );
  const [clientActive, setClientActive] = useState(
    () => localStorage.getItem(SYNC_CLIENT_ACTIVE_KEY) === "true"
  );
  const [clientStatus, setClientStatus] = useState<"idle" | "ok" | "error">("idle");

  // Restore server state on mount
  useEffect(() => {
    invoke<number | null>("get_sync_server_status").then(port => {
      if (port) {
        setServerRunning(true);
        setServerPort(String(port));
        invoke<string[]>("get_local_ips").then(ips => {
          if (ips[0]) setServerUrl(`http://${ips[0]}:${port}`);
        });
      }
    }).catch(() => {});
  }, []);

  const handleStartServer = async () => {
    try {
      const port = parseInt(serverPort, 10);
      if (isNaN(port) || port < 1024 || port > 65535) {
        toast.error("端口号必须在 1024–65535 之间");
        return;
      }
      const secret = sharedSecret.trim();
      if (!secret) {
        toast.error("请先设置同步密钥");
        return;
      }
      localStorage.setItem(SYNC_SHARED_SECRET_KEY, secret);
      const url = await invoke<string>("start_sync_server", { port, sharedSecret: secret });
      setServerUrl(url);
      setServerRunning(true);
      toast.success(`服务已启动：${url}`);
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleStopServer = async () => {
    try {
      await invoke("stop_sync_server");
      setServerRunning(false);
      setServerUrl("");
      toast.success("服务已停止");
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleTestClient = async () => {
    if (!clientUrl.trim()) { toast.error("请输入服务端地址"); return; }
    if (!sharedSecret.trim()) { toast.error("请输入同步密钥"); return; }
    try {
      const url = clientUrl.trim().replace(/\/$/, "");
      await invoke("fetch_sync_orders", {
        serverUrl: url,
        sinceEpochS: Math.floor(Date.now() / 1000) - 10,
        sharedSecret: sharedSecret.trim(),
        clientVersion: SYNC_PROTOCOL_VERSION,
      });
      setClientStatus("ok");
      toast.success("连接成功");
    } catch (e) {
      setClientStatus("error");
      toast.error(`连接失败: ${String(e)}`);
    }
  };

  const handleClientActivate = (active: boolean) => {
    setClientActive(active);
    localStorage.setItem(SYNC_CLIENT_ACTIVE_KEY, active ? "true" : "false");
    localStorage.setItem(SYNC_CLIENT_URL_KEY, clientUrl.trim());
    localStorage.setItem(SYNC_SHARED_SECRET_KEY, sharedSecret.trim());
    window.dispatchEvent(new CustomEvent("cuckoo:sync-settings-changed"));
  };

  const handleGenerateSecret = () => {
    const next = crypto.randomUUID().replace(/-/g, "");
    setSharedSecret(next);
    localStorage.setItem(SYNC_SHARED_SECRET_KEY, next);
    window.dispatchEvent(new CustomEvent("cuckoo:sync-settings-changed"));
    toast.success("已生成新的同步密钥");
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Radio className="h-4 w-4" />
          局域网多设备同步
        </CardTitle>
        <CardDescription>同一局域网内的多台设备可实时共享订单数据</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-3">
          <p className="text-sm font-medium flex items-center gap-1.5">
            <ShieldCheck className="w-3.5 h-3.5" />
            同步密钥
          </p>
          <div className="flex gap-2 items-center">
            <Input
              value={sharedSecret}
              onChange={e => { setSharedSecret(e.target.value); setClientStatus("idle"); }}
              placeholder="主从设备需使用同一共享密钥"
              className="font-mono text-xs"
            />
            <Button size="sm" variant="outline" onClick={handleGenerateSecret}>生成</Button>
          </div>
          <p className="text-xs text-muted-foreground">
            主机和从机必须使用同一密钥；同步协议版本固定为 v{SYNC_PROTOCOL_VERSION}。
          </p>
        </div>

        <Separator />

        {/* Server mode */}
        <div className="space-y-3">
          <p className="text-sm font-medium flex items-center gap-1.5">
            <ServerCrash className="w-3.5 h-3.5" />
            主机模式（此设备作为数据源）
          </p>
          <div className="flex gap-2 items-center">
            <Input
              className="w-28 text-sm"
              value={serverPort}
              onChange={e => setServerPort(e.target.value)}
              placeholder="7070"
              disabled={serverRunning}
            />
            <span className="text-xs text-muted-foreground">端口</span>
            {serverRunning ? (
              <Button size="sm" variant="destructive" className="h-8" onClick={handleStopServer}>停止</Button>
            ) : (
              <Button size="sm" className="h-8" onClick={handleStartServer}>启动</Button>
            )}
          </div>
          {serverRunning && serverUrl && (
            <div className="flex items-center gap-2 rounded-lg bg-muted p-2.5">
              <span className="text-xs font-mono flex-1 text-emerald-600 dark:text-emerald-400 break-all">
                {serverUrl}
              </span>
              <Button size="icon" variant="ghost" className="h-6 w-6 shrink-0" onClick={() => {
                navigator.clipboard.writeText(serverUrl);
                toast.success("已复制");
              }}>
                <Copy className="h-3 w-3" />
              </Button>
            </div>
          )}
          {serverRunning && (
            <p className="text-xs text-muted-foreground">将此地址填入其他设备的「从机模式」</p>
          )}
        </div>

        <Separator />

        {/* Client mode */}
        <div className="space-y-3">
          <p className="text-sm font-medium flex items-center gap-1.5">
            <Link2 className="w-3.5 h-3.5" />
            从机模式（连接到主机，只读同步）
          </p>
          <div className="flex gap-2">
            <Input
              className="flex-1 text-sm font-mono"
              value={clientUrl}
              onChange={e => { setClientUrl(e.target.value); setClientStatus("idle"); }}
              placeholder="http://192.168.x.x:7070"
              disabled={clientActive}
            />
            <Button size="sm" variant="outline" className="h-9 shrink-0" onClick={handleTestClient} disabled={clientActive}>
              测试
            </Button>
          </div>
          {clientStatus === "ok" && (
            <p className="text-xs text-emerald-600 flex items-center gap-1"><Wifi className="w-3 h-3" />连接正常</p>
          )}
          {clientStatus === "error" && (
            <p className="text-xs text-destructive flex items-center gap-1"><WifiOff className="w-3 h-3" />连接失败，请检查地址与网络</p>
          )}
          <div className="flex items-center justify-between">
            <Label htmlFor="client-active-switch" className="flex flex-col gap-0.5 cursor-pointer">
              <span className="text-sm font-medium">启用同步（每 4 秒拉取一次）</span>
              <span className="text-xs text-muted-foreground">启用后此设备订单列表将包含主机数据</span>
            </Label>
            <Switch
              id="client-active-switch"
              checked={clientActive}
              onCheckedChange={handleClientActivate}
            />
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

// ── Role PIN management ───────────────────────────────────────────────────────

const ROLES: Role[] = ["owner", "cashier", "chef", "warehouse"];

function RolePinCard() {
  const [editingRole, setEditingRole] = useState<Role | null>(null);
  const [newPin, setNewPin] = useState("");
  const [confirmPin, setConfirmPin] = useState("");
  const [showPin, setShowPin] = useState(false);
  const [pinStatuses, setPinStatuses] = useState<Record<Role, boolean>>({
    owner: false,
    cashier: false,
    chef: false,
    warehouse: false,
  });

  const refreshPinStatuses = useCallback(() => {
    getRolePinStatuses().then(setPinStatuses).catch((e) => {
      toast.error("加载角色 PIN 状态失败", { description: String(e) });
    });
  }, []);

  useEffect(() => {
    refreshPinStatuses();
  }, [refreshPinStatuses]);

  const startEdit = (role: Role) => {
    setEditingRole(role);
    setNewPin("");
    setConfirmPin("");
    setShowPin(false);
  };

  const handleSave = async () => {
    if (!editingRole) return;
    if (newPin && newPin !== confirmPin) {
      toast.error("两次输入的 PIN 不一致");
      return;
    }
    if (newPin && newPin.length < 4) {
      toast.error("PIN 至少 4 位");
      return;
    }
    try {
      await saveRolePin(editingRole, newPin || null);
      toast.success(newPin ? `${ROLE_LABELS[editingRole]} PIN 已设置` : `${ROLE_LABELS[editingRole]} PIN 已清除`);
      setEditingRole(null);
      refreshPinStatuses();
    } catch (e) {
      toast.error("保存角色 PIN 失败", { description: String(e) });
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <ShieldCheck className="h-4 w-4" />
          角色权限 PIN 管理
        </CardTitle>
        <CardDescription>为每个角色设置 PIN 码。切换到该角色时需要输入对应 PIN。不设置 PIN 则可自由切换。</CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        {ROLES.map((role) => {
          const hasPin = pinStatuses[role];
          const isEditing = editingRole === role;
          return (
            <div key={role} className="rounded-lg border p-3 space-y-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <span className={`rounded px-1.5 py-0.5 text-xs font-semibold ${ROLE_COLORS[role]}`}>
                    {ROLE_LABELS[role]}
                  </span>
                  <span className="text-xs text-muted-foreground">{ROLE_DESCRIPTIONS[role]}</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-xs text-muted-foreground">{hasPin ? "● ● ● ●" : "未设置"}</span>
                  <Button size="sm" variant="outline" className="h-7 text-xs" onClick={() => isEditing ? setEditingRole(null) : startEdit(role)}>
                    {isEditing ? "取消" : hasPin ? "修改" : "设置"}
                  </Button>
                  {hasPin && !isEditing && (
                    <Button size="sm" variant="ghost" className="h-7 text-xs text-destructive" onClick={async () => {
                      try {
                        await saveRolePin(role, null);
                        toast.success(`${ROLE_LABELS[role]} PIN 已清除`);
                        refreshPinStatuses();
                      } catch (e) {
                        toast.error("清除角色 PIN 失败", { description: String(e) });
                      }
                    }}>清除</Button>
                  )}
                </div>
              </div>
              {isEditing && (
                <div className="space-y-2 pt-1">
                  <div className="relative">
                    <Input
                      type={showPin ? "text" : "password"}
                      inputMode="numeric"
                      maxLength={8}
                      value={newPin}
                      onChange={(e) => setNewPin(e.target.value)}
                      placeholder="新 PIN（留空则清除）"
                      className="pr-9 text-sm"
                    />
                    <button
                      type="button"
                      className="absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground"
                      onClick={() => setShowPin(v => !v)}
                    >
                      {showPin ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
                    </button>
                  </div>
                  {newPin && (
                    <Input
                      type={showPin ? "text" : "password"}
                      inputMode="numeric"
                      maxLength={8}
                      value={confirmPin}
                      onChange={(e) => setConfirmPin(e.target.value)}
                      placeholder="确认 PIN"
                      className="text-sm"
                      onKeyDown={(e) => e.key === "Enter" && handleSave()}
                    />
                  )}
                  <Button size="sm" className="h-7 text-xs" onClick={handleSave}>保存</Button>
                </div>
              )}
            </div>
          );
        })}
      </CardContent>
    </Card>
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
  const [backupLoading, setBackupLoading] = useState(false);
  const [backupPath, setBackupPath] = useState<string | null>(null);
  const [restoreLoading, setRestoreLoading] = useState(false);
  const [restoreConfirmPath, setRestoreConfirmPath] = useState<string | null>(null);
  const [paymentQr, setPaymentQr] = useState<string | null>(null);
  const [paymentQrLoading, setPaymentQrLoading] = useState(false);

  useEffect(() => {
    invoke<string | null>("get_payment_qr").then((d) => setPaymentQr(d ?? null)).catch(() => {});
  }, []);

  useEffect(() => {
    invoke<string>("health_check")
      .then((r) => setDbStatus(r === "ok" ? "正常" : "異常"))
      .catch(() => setDbStatus("連線失敗"));
    invoke<string>("get_app_version")
      .then(setAppVersion)
      .catch(() => {});
  }, []);

  async function handleBackup() {
    setBackupLoading(true);
    try {
      const path = await invoke<string>("backup_database");
      setBackupPath(path);
      toast.success("備份成功");
    } catch (e) {
      toast.error(String(e));
    } finally {
      setBackupLoading(false);
    }
  }

  async function handleRestore() {
    const selected = await open({
      title: "選擇備份文件",
      filters: [{ name: "Cuckoo 備份", extensions: ["db"] }],
      multiple: false,
      directory: false,
    });
    if (!selected || typeof selected !== "string") return;
    setRestoreConfirmPath(selected);
  }

  async function confirmRestore() {
    if (!restoreConfirmPath) return;
    setRestoreConfirmPath(null);
    setRestoreLoading(true);
    try {
      const msg = await invoke<string>("restore_database", { path: restoreConfirmPath });
      toast.success(msg + " — 請重啟應用生效");
    } catch (e) {
      toast.error("恢復失敗", { description: String(e) });
    } finally {
      setRestoreLoading(false);
    }
  }

  async function handlePaymentQrUpload(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    setPaymentQrLoading(true);
    try {
      const base64 = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve((reader.result as string).split(",")[1]);
        reader.onerror = reject;
        reader.readAsDataURL(file);
      });
      await invoke("set_payment_qr", { data: base64 });
      setPaymentQr(base64);
      toast.success("收款碼已保存");
    } catch (e) {
      toast.error(String(e));
    } finally {
      setPaymentQrLoading(false);
      e.target.value = "";
    }
  }

  async function handleClearPaymentQr() {
    try {
      await invoke("set_payment_qr", { data: null });
      setPaymentQr(null);
      toast.success("收款碼已清除");
    } catch (e) {
      toast.error(String(e));
    }
  }

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

      {/* 數據備份與恢復（合併為單一 Card） */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-base">
            <HardDrive className="h-4 w-4" />
            数据备份与恢复
          </CardTitle>
          <CardDescription>备份文件保存至 Documents/Cuckoo 备份/，恢复前自动创建当前备份</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <Label htmlFor="auto-backup-switch" className="flex flex-col gap-0.5 cursor-pointer">
              <span className="text-sm font-medium">启动时自动备份</span>
              <span className="text-xs text-muted-foreground">每次打开应用时自动创建带时间戳的备份</span>
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
          <div className="flex gap-2">
            <Button onClick={handleBackup} disabled={backupLoading} className="flex-1" variant="outline">
              {backupLoading ? <Loader2 className="h-4 w-4 animate-spin mr-2" /> : <HardDrive className="h-4 w-4 mr-2" />}
              立即备份
            </Button>
            <Button onClick={handleRestore} disabled={restoreLoading} className="flex-1" variant="outline">
              {restoreLoading ? <Loader2 className="h-4 w-4 animate-spin mr-2" /> : <RefreshCw className="h-4 w-4 mr-2" />}
              从备份恢复
            </Button>
          </div>
          {backupPath && (
            <div className="rounded-lg bg-muted px-3 py-2 text-xs text-muted-foreground break-all">
              已备份：{backupPath}
            </div>
          )}
        </CardContent>
      </Card>

      {/* 恢復確認 Dialog */}
      <Dialog open={!!restoreConfirmPath} onOpenChange={(v) => !v && setRestoreConfirmPath(null)}>
        <DialogContent className="max-w-sm">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <AlertTriangle className="h-4 w-4 text-destructive" />
              确认恢复数据库
            </DialogTitle>
            <DialogDescription className="space-y-1 pt-1">
              <span className="block">将从以下文件恢复：</span>
              <span className="block font-mono text-xs break-all text-foreground">{restoreConfirmPath}</span>
              <span className="block pt-1 text-destructive">当前数据将被替换，操作不可撤销。恢复前会自动备份一次。</span>
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setRestoreConfirmPath(null)}>取消</Button>
            <Button variant="destructive" onClick={confirmRestore}>确认恢复</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* PAY 收款碼設定 */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-2 text-base">
            <Smartphone className="h-4 w-4" />
            自助點單收款碼
          </CardTitle>
          <CardDescription>顧客下單後顯示此碼，引導掃碼付款</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {paymentQr ? (
            <div className="flex items-center gap-4">
              <img
                src={`data:image/png;base64,${paymentQr}`}
                alt="收款碼"
                className="h-24 w-24 rounded-lg border object-contain"
              />
              <div className="flex flex-col gap-2">
                <p className="text-xs text-muted-foreground">當前收款碼</p>
                <label className="cursor-pointer">
                  <Button variant="outline" size="sm" type="button" onClick={(e) => { e.preventDefault(); (e.currentTarget.nextElementSibling as HTMLInputElement)?.click(); }}>
                    <RefreshCw className="h-3.5 w-3.5 mr-1.5" />更換圖片
                  </Button>
                  <input type="file" accept="image/*" className="hidden" onChange={handlePaymentQrUpload} disabled={paymentQrLoading} />
                </label>
                <Button variant="ghost" size="sm" className="text-destructive hover:text-destructive" onClick={handleClearPaymentQr}>
                  移除收款碼
                </Button>
              </div>
            </div>
          ) : (
            <label className="flex flex-col items-center justify-center rounded-lg border-2 border-dashed py-6 gap-2 cursor-pointer hover:bg-muted/50 transition-colors">
              <Smartphone className="h-8 w-8 text-muted-foreground/50" />
              <span className="text-sm text-muted-foreground">點擊上傳微信/支付寶收款碼</span>
              <span className="text-xs text-muted-foreground/70">支援 PNG/JPG</span>
              <input type="file" accept="image/*" className="hidden" onChange={handlePaymentQrUpload} disabled={paymentQrLoading} />
            </label>
          )}
        </CardContent>
      </Card>

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

      {/* Self-order web server */}
      <WebServerCard />

      {/* LAN sync */}
      <LanSyncCard />

      {/* Role PIN management */}
      <RolePinCard />

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
          <ErrorLogPanel appVersion={appVersion} />
        </CardContent>
      </Card>
    </div>
  );
}
