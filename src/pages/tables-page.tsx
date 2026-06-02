import { useState, useEffect, useRef } from "react";
import { toPng } from "html-to-image";
import { StyledQR } from "@/components/styled-qr";
import { call as invoke } from "@/lib/transport";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Switch } from "@/components/ui/switch";
import { toast } from "sonner";
import { Plus, Edit2, Trash2, QrCode, Download, Copy, RefreshCw } from "lucide-react";
import type { RestaurantTable } from "@/types";

interface WebServerStatus {
  running: boolean;
  port: number | null;
  url: string | null;
}

interface TableQrDialogProps {
  table: RestaurantTable;
  baseUrl: string;
  onClose: () => void;
}

function TableQrDialog({ table, baseUrl, onClose }: TableQrDialogProps) {
  const [token, setToken] = useState<string | null>(null);
  const [exporting, setExporting] = useState(false);
  const cardRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    invoke<string>("sign_table_token", { tableNo: table.table_no })
      .then(setToken)
      .catch(() => setToken(null));
  }, [table.table_no]);

  // Token-based URL when available; fall back to legacy static URL during grace period.
  const tableUrl = token
    ? `${baseUrl}/#/t/${token}`
    : `${baseUrl}/#/table/${encodeURIComponent(table.table_no)}`;
  const label = table.label ?? `桌 ${table.table_no}`;

  async function downloadCard() {
    if (!cardRef.current) return;
    setExporting(true);
    try {
      // pixelRatio 3 → ~print-grade DPI for the full table card.
      const dataUrl = await toPng(cardRef.current, { pixelRatio: 3, backgroundColor: "#ffffff" });
      const link = document.createElement("a");
      link.download = `桌贴-${table.table_no}.png`;
      link.href = dataUrl;
      link.click();
    } catch {
      toast.error("导出失败，请重试");
    } finally {
      setExporting(false);
    }
  }

  return (
    <Dialog open onOpenChange={onClose}>
      <DialogContent className="max-w-xs">
        <DialogHeader>
          <DialogTitle>{label} 扫码点单</DialogTitle>
        </DialogHeader>
        <div className="flex flex-col items-center gap-4 py-2">
          {/* 桌贴台卡（整张可导出） */}
          <div ref={cardRef} className="w-full rounded-2xl border-2 border-orange-200 bg-white p-5 flex flex-col items-center gap-3">
            <div className="text-center">
              <div className="text-[11px] text-gray-400 tracking-[0.3em]">扫码自助点餐</div>
              <div className="text-3xl font-extrabold text-gray-900 mt-1">{label}</div>
            </div>
            <StyledQR value={tableUrl} size={200} dotColor="#ea580c" />
            <div className="flex items-center gap-1.5 text-orange-500 text-sm font-bold">
              <QrCode className="h-4 w-4" /> 扫一扫 自助点餐
            </div>
          </div>
          <p className="text-[10px] text-muted-foreground font-mono text-center break-all px-2">
            {tableUrl}
          </p>
          <div className="flex gap-2 w-full">
            <Button
              className="flex-1"
              variant="outline"
              size="sm"
              onClick={() => {
                navigator.clipboard.writeText(tableUrl);
                toast.success("已复制链接");
              }}
            >
              <Copy className="h-3.5 w-3.5 mr-1.5" />
              复制
            </Button>
            <Button className="flex-1" size="sm" onClick={downloadCard} disabled={exporting}>
              <Download className="h-3.5 w-3.5 mr-1.5" />
              {exporting ? "导出中…" : "下载桌贴"}
            </Button>
          </div>
          <p className="text-[10px] text-muted-foreground text-center">下载整张桌贴（高清）可交印刷店制作</p>
        </div>
      </DialogContent>
    </Dialog>
  );
}

interface TableFormDialogProps {
  initial?: RestaurantTable;
  onSave: (tableNo: string, label: string, isActive: boolean, sortNo: number) => Promise<void>;
  onClose: () => void;
}

function TableFormDialog({ initial, onSave, onClose }: TableFormDialogProps) {
  const [tableNo, setTableNo] = useState(initial?.table_no ?? "");
  const [label, setLabel] = useState(initial?.label ?? "");
  const [isActive, setIsActive] = useState(initial?.is_active ?? true);
  const [sortNo, setSortNo] = useState(String(initial?.sort_no ?? 0));
  const [saving, setSaving] = useState(false);

  const handleSave = async () => {
    if (!tableNo.trim()) {
      toast.error("桌号不能为空");
      return;
    }
    setSaving(true);
    try {
      await onSave(tableNo.trim(), label.trim(), isActive, parseInt(sortNo, 10) || 0);
      onClose();
    } catch (e) {
      toast.error(String(e));
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog open onOpenChange={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{initial ? "编辑餐桌" : "添加餐桌"}</DialogTitle>
        </DialogHeader>
        <div className="space-y-4 py-2">
          <div className="space-y-1.5">
            <Label>桌号 *</Label>
            <Input
              placeholder="例：A1、01、VIP"
              value={tableNo}
              onChange={(e) => setTableNo(e.target.value)}
            />
          </div>
          <div className="space-y-1.5">
            <Label>显示名称（选填）</Label>
            <Input
              placeholder="例：靠窗桌、包厢"
              value={label}
              onChange={(e) => setLabel(e.target.value)}
            />
          </div>
          <div className="space-y-1.5">
            <Label>排序号</Label>
            <Input
              type="number"
              value={sortNo}
              onChange={(e) => setSortNo(e.target.value)}
              className="w-24"
            />
          </div>
          <div className="flex items-center justify-between">
            <Label htmlFor="table-active-switch">启用</Label>
            <Switch
              id="table-active-switch"
              checked={isActive}
              onCheckedChange={setIsActive}
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={saving}>
            取消
          </Button>
          <Button onClick={handleSave} disabled={saving}>
            保存
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

const MANUAL_IP_KEY = "cuckoo_manual_lan_ip";

export function TablesPage() {
  const [tables, setTables] = useState<RestaurantTable[]>([]);
  const [webServerStatus, setWebServerStatus] = useState<WebServerStatus>({
    running: false,
    port: null,
    url: null,
  });
  const [loading, setLoading] = useState(true);
  const [showAdd, setShowAdd] = useState(false);
  const [editing, setEditing] = useState<RestaurantTable | null>(null);
  const [qrTable, setQrTable] = useState<RestaurantTable | null>(null);
  const [manualIp, setManualIp] = useState(() => localStorage.getItem(MANUAL_IP_KEY) ?? "");
  const [editingIp, setEditingIp] = useState(false);
  const [ipDraft, setIpDraft] = useState("");

  // Compute the effective base URL: prefer manualIp if set, else auto-detected
  const effectiveUrl = (() => {
    if (!webServerStatus.running || !webServerStatus.port) return null;
    const ip = manualIp.trim() || null;
    if (ip) return `http://${ip}:${webServerStatus.port}`;
    return webServerStatus.url;
  })();

  const autoIpIsLoopback =
    webServerStatus.url?.includes("127.0.0.1") || webServerStatus.url?.includes("::1");

  const [requireToken, setRequireToken] = useState(false);

  const load = async () => {
    try {
      const [t, ws] = await Promise.all([
        invoke<RestaurantTable[]>("get_restaurant_tables"),
        invoke<WebServerStatus>("get_web_server_status"),
      ]);
      setTables(t);
      setWebServerStatus(ws);
      invoke<boolean>("get_require_token").then(setRequireToken).catch(() => {});
    } catch (e) {
      toast.error(String(e));
    } finally {
      setLoading(false);
    }
  };

  const toggleRequireToken = async (v: boolean) => {
    try {
      await invoke("set_require_token", { enabled: v });
      setRequireToken(v);
      toast.success(v ? "已开启：仅接受最新二维码下单" : "已关闭：兼容旧二维码");
    } catch (e) {
      toast.error(String(e));
    }
  };

  useEffect(() => {
    load();
  }, []);

  const handleAdd = async (
    tableNo: string,
    label: string,
    isActive: boolean,
    sortNo: number
  ) => {
    await invoke("create_restaurant_table", {
      tableNo,
      label: label || null,
      isActive,
      sortNo,
    });
    toast.success(`桌号 ${tableNo} 已添加`);
    load();
  };

  const handleEdit = async (
    tableNo: string,
    label: string,
    isActive: boolean,
    sortNo: number
  ) => {
    if (!editing) return;
    await invoke("update_restaurant_table", {
      id: editing.id,
      tableNo,
      label: label || null,
      isActive,
      sortNo,
    });
    toast.success("已更新");
    load();
  };

  const handleDelete = async (table: RestaurantTable) => {
    if (!confirm(`确定删除桌号 ${table.table_no}？`)) return;
    try {
      await invoke("delete_restaurant_table", { id: table.id });
      toast.success("已删除");
      load();
    } catch (e) {
      toast.error(String(e));
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">餐桌管理</h2>
          <p className="text-sm text-muted-foreground">管理桌号，生成扫码点单二维码</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={load}>
            <RefreshCw className="h-4 w-4 mr-1.5" />
            刷新
          </Button>
          <Button onClick={() => setShowAdd(true)} size="sm">
            <Plus className="h-4 w-4 mr-1.5" />
            添加餐桌
          </Button>
        </div>
      </div>

      {/* Web server status banner */}
      {!webServerStatus.running && (
        <Card className="border-amber-300 dark:border-amber-700">
          <CardContent className="py-3">
            <p className="text-sm text-amber-700 dark:text-amber-400">
              扫码服务未启动 — 重启应用后二维码链接才能使用。
            </p>
          </CardContent>
        </Card>
      )}

      {webServerStatus.running && (
        <Card className={autoIpIsLoopback && !manualIp ? "border-amber-300 dark:border-amber-700" : ""}>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm flex items-center gap-2">
              自助点单入口
              {autoIpIsLoopback && !manualIp && (
                <Badge variant="outline" className="text-amber-600 border-amber-400 text-xs">IP 未识别</Badge>
              )}
            </CardTitle>
            <CardDescription className="text-xs font-mono break-all">
              {effectiveUrl ? `${effectiveUrl}/#/table/桌号` : "—"}
            </CardDescription>
          </CardHeader>
          <CardContent className="pb-3 space-y-3">
            <p className="text-xs text-muted-foreground">
              顾客用手机扫描各桌二维码，即可在自己的手机上浏览菜单并提交订单。
            </p>
            <div className="flex items-center justify-between rounded-lg border px-3 py-2">
              <div className="min-w-0">
                <p className="text-xs font-medium">仅接受最新二维码下单</p>
                <p className="text-[10px] text-muted-foreground">开启后旧二维码失效，防伪造桌号刷单（请先重新生成并张贴新桌贴）</p>
              </div>
              <Switch checked={requireToken} onCheckedChange={toggleRequireToken} />
            </div>
            {(autoIpIsLoopback && !manualIp) && (
              <p className="text-xs text-amber-700 dark:text-amber-400">
                ⚠️ 未能自动检测局域网 IP。请手动填写本机 IP（通常为 192.168.x.x）。
              </p>
            )}
            {/* Manual IP override */}
            {editingIp ? (
              <div className="flex gap-2 items-center">
                <Input
                  className="h-8 text-xs font-mono"
                  placeholder="例：192.168.1.100"
                  value={ipDraft}
                  onChange={(e) => setIpDraft(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      const v = ipDraft.trim();
                      setManualIp(v);
                      if (v) localStorage.setItem(MANUAL_IP_KEY, v);
                      else localStorage.removeItem(MANUAL_IP_KEY);
                      setEditingIp(false);
                    }
                  }}
                  autoFocus
                />
                <Button size="sm" className="h-8" onClick={() => {
                  const v = ipDraft.trim();
                  setManualIp(v);
                  if (v) localStorage.setItem(MANUAL_IP_KEY, v);
                  else localStorage.removeItem(MANUAL_IP_KEY);
                  setEditingIp(false);
                }}>保存</Button>
                <Button size="sm" variant="ghost" className="h-8" onClick={() => setEditingIp(false)}>取消</Button>
              </div>
            ) : (
              <div className="flex items-center gap-2">
                <span className="text-xs text-muted-foreground">
                  {manualIp ? `手动 IP：${manualIp}` : "IP 自动检测"}
                </span>
                <Button size="sm" variant="outline" className="h-7 text-xs" onClick={() => {
                  setIpDraft(manualIp);
                  setEditingIp(true);
                }}>
                  {manualIp ? "修改 IP" : "手动设置 IP"}
                </Button>
                {manualIp && (
                  <Button size="sm" variant="ghost" className="h-7 text-xs text-destructive" onClick={() => {
                    setManualIp("");
                    localStorage.removeItem(MANUAL_IP_KEY);
                  }}>清除</Button>
                )}
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Table list */}
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {loading && (
          <p className="text-sm text-muted-foreground col-span-full py-4 text-center">
            加载中…
          </p>
        )}
        {!loading && tables.length === 0 && (
          <p className="text-sm text-muted-foreground col-span-full py-4 text-center">
            还没有餐桌。点击右上角「添加餐桌」开始设置。
          </p>
        )}
        {tables.map((t) => (
          <Card key={t.id} className={t.is_active ? "" : "opacity-60"}>
            <CardContent className="p-4">
              <div className="flex items-start justify-between gap-2">
                <div className="min-w-0">
                  <p className="font-semibold truncate">
                    {t.label ? `${t.label}` : `桌 ${t.table_no}`}
                  </p>
                  <p className="text-xs text-muted-foreground font-mono">
                    桌号：{t.table_no}
                  </p>
                  {!t.is_active && (
                    <Badge variant="secondary" className="mt-1 text-xs">
                      停用
                    </Badge>
                  )}
                </div>
                <div className="flex gap-1 shrink-0">
                  <Button
                    size="icon"
                    variant="ghost"
                    className="h-7 w-7"
                    title="生成二维码"
                    disabled={!webServerStatus.running || !effectiveUrl}
                    onClick={() => setQrTable(t)}
                  >
                    <QrCode className="h-3.5 w-3.5" />
                  </Button>
                  <Button
                    size="icon"
                    variant="ghost"
                    className="h-7 w-7"
                    onClick={() => setEditing(t)}
                  >
                    <Edit2 className="h-3.5 w-3.5" />
                  </Button>
                  <Button
                    size="icon"
                    variant="ghost"
                    className="h-7 w-7 text-destructive hover:text-destructive"
                    onClick={() => handleDelete(t)}
                  >
                    <Trash2 className="h-3.5 w-3.5" />
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {showAdd && (
        <TableFormDialog onSave={handleAdd} onClose={() => setShowAdd(false)} />
      )}

      {editing && (
        <TableFormDialog
          initial={editing}
          onSave={handleEdit}
          onClose={() => setEditing(null)}
        />
      )}

      {qrTable && effectiveUrl && (
        <TableQrDialog
          table={qrTable}
          baseUrl={effectiveUrl}
          onClose={() => setQrTable(null)}
        />
      )}
    </div>
  );
}
