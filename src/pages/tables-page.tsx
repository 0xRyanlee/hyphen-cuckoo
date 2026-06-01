import { useState, useEffect } from "react";
import { QRCodeCanvas } from "qrcode.react";
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
  const tableUrl = `${baseUrl}/#/table/${encodeURIComponent(table.table_no)}`;

  const handleDownload = () => {
    const canvas = document.getElementById(`qr-canvas-${table.id}`) as HTMLCanvasElement | null;
    if (!canvas) return;
    const link = document.createElement("a");
    link.download = `二维码-桌${table.table_no}.png`;
    link.href = canvas.toDataURL("image/png");
    link.click();
  };

  return (
    <Dialog open onOpenChange={onClose}>
      <DialogContent className="max-w-xs">
        <DialogHeader>
          <DialogTitle>
            {table.label ?? `桌 ${table.table_no}`} 扫码点单
          </DialogTitle>
        </DialogHeader>
        <div className="flex flex-col items-center gap-4 py-2">
          <div className="rounded-xl border bg-white p-3">
            <QRCodeCanvas
              id={`qr-canvas-${table.id}`}
              value={tableUrl}
              size={200}
              level="M"
              includeMargin={false}
            />
          </div>
          <p className="text-xs text-muted-foreground font-mono text-center break-all px-2">
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
            <Button className="flex-1" size="sm" onClick={handleDownload}>
              <Download className="h-3.5 w-3.5 mr-1.5" />
              下载图片
            </Button>
          </div>
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

  const load = async () => {
    try {
      const [t, ws] = await Promise.all([
        invoke<RestaurantTable[]>("get_restaurant_tables"),
        invoke<WebServerStatus>("get_web_server_status"),
      ]);
      setTables(t);
      setWebServerStatus(ws);
    } catch (e) {
      toast.error(String(e));
    } finally {
      setLoading(false);
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

      {webServerStatus.running && webServerStatus.url && (
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">自助点单入口</CardTitle>
            <CardDescription className="text-xs font-mono break-all">
              {webServerStatus.url}/#/table/桌号
            </CardDescription>
          </CardHeader>
          <CardContent className="pb-3">
            <p className="text-xs text-muted-foreground">
              顾客用手机扫描各桌二维码，即可在自己的手机上浏览菜单并提交订单。
            </p>
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
                    disabled={!webServerStatus.running}
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

      {qrTable && webServerStatus.url && (
        <TableQrDialog
          table={qrTable}
          baseUrl={webServerStatus.url}
          onClose={() => setQrTable(null)}
        />
      )}
    </div>
  );
}
