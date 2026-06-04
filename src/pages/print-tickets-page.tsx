import { useState, useEffect } from "react";
import { call as invoke } from "@/lib/transport";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import { Printer, Tag, FileText, Plus, Edit, Trash2, Settings } from "lucide-react";
import { toast } from "sonner";
import type { PrintTicketType, KitchenStation } from "../types";

export function PrintTicketsPage() {
  const [ticketTypes, setTicketTypes] = useState<PrintTicketType[]>([]);
  const [stations, setStations] = useState<KitchenStation[]>([]);
  const [loading, setLoading] = useState(true);
  const [editDialog, setEditDialog] = useState<PrintTicketType | null>(null);
  const [createDialog, setCreateDialog] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<PrintTicketType | null>(null);

  useEffect(() => {
    loadData();
  }, []);

  async function loadData() {
    setLoading(true);
    try {
      await invoke("ensure_default_ticket_types");
      const [types, stationData] = await Promise.all([
        invoke<PrintTicketType[]>("get_print_ticket_types"),
        invoke<KitchenStation[]>("get_kitchen_stations"),
      ]);
      setTicketTypes(types);
      setStations(stationData);
    } catch (e) {
      toast.error("加载失败", { description: String(e) });
    } finally {
      setLoading(false);
    }
  }

  async function handleCreate(req: CreateTicketTypeRequest) {
    try {
      await invoke<number>("create_print_ticket_type", { req });
      toast.success("票据类型已创建");
      loadData();
      setCreateDialog(false);
    } catch (e) {
      toast.error("创建失败", { description: String(e) });
    }
  }

  async function handleUpdate(id: number, req: UpdateTicketTypeRequest) {
    try {
      await invoke("update_print_ticket_type", { id, req });
      toast.success("票据类型已更新");
      loadData();
      setEditDialog(null);
    } catch (e) {
      toast.error("更新失败", { description: String(e) });
    }
  }

  async function handleDelete(id: number) {
    try {
      await invoke("delete_print_ticket_type", { id });
      toast.success("已删除");
      setDeleteTarget(null);
      loadData();
    } catch (e) {
      toast.error("删除失败", { description: String(e) });
    }
  }

  async function handleSetDefault(id: number) {
    try {
      await invoke("set_default_ticket_type", { id });
      toast.success("已设为默认");
      loadData();
    } catch (e) {
      toast.error("设置失败", { description: String(e) });
    }
  }

  if (loading) {
    return (
      <div className="p-6 flex items-center justify-center">
        <div className="text-muted-foreground">加载中...</div>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">票据类型</h1>
          <p className="text-muted-foreground mt-1">管理打印票据的类型和配置</p>
        </div>
        <Button onClick={() => setCreateDialog(true)}>
          <Plus className="h-4 w-4 mr-2" />
          新建票据类型
        </Button>
      </div>

      <div className="grid gap-4">
        {ticketTypes.map((ticket) => (
          <Card key={ticket.id} className={ticket.is_active ? "" : "opacity-60"}>
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  {ticket.code === "kitchen" && <FileText className="h-5 w-5" />}
                  {ticket.code === "receipt" && <Printer className="h-5 w-5" />}
                  {ticket.code === "label" && <Tag className="h-5 w-5" />}
                  <div>
                    <CardTitle className="text-lg">{ticket.name}</CardTitle>
                    <CardDescription>{ticket.description || ticket.code}</CardDescription>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  {ticket.is_default && <Badge>默认</Badge>}
                  {!ticket.is_active && <Badge variant="secondary">已停用</Badge>}
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex flex-wrap gap-4 text-sm">
                <div>
                  <span className="text-muted-foreground">纸宽:</span>{" "}
                  <span className="font-medium">{ticket.paper_width}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">字体:</span>{" "}
                  <span className="font-medium">{ticket.font_size}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">切割:</span>{" "}
                  <span className="font-medium">{ticket.cut_mode}</span>
                </div>
                {ticket.station_id && (
                  <div>
                    <span className="text-muted-foreground">工作站:</span>{" "}
                    <span className="font-medium">
                      {stations.find(s => s.id === ticket.station_id)?.name || "未知"}
                    </span>
                  </div>
                )}
              </div>

              <div className="text-sm">
                <span className="text-muted-foreground">显示项:</span>
                <div className="flex flex-wrap gap-2 mt-1">
                  {ticket.show_order_no && <Badge variant="outline">订单号</Badge>}
                  {ticket.show_table_no && <Badge variant="outline">桌号</Badge>}
                  {ticket.show_dine_type && <Badge variant="outline">用餐类型</Badge>}
                  {ticket.show_item_name && <Badge variant="outline">菜品</Badge>}
                  {ticket.show_item_qty && <Badge variant="outline">数量</Badge>}
                  {ticket.show_item_price && <Badge variant="outline">单价</Badge>}
                  {ticket.show_price && <Badge variant="outline" className="bg-muted">显示价格</Badge>}
                  {ticket.show_seq && <Badge variant="outline" className="bg-muted">显示序号</Badge>}
                </div>
              </div>

              <div className="flex gap-2">
                <Button variant="outline" size="sm" onClick={() => handleSetDefault(ticket.id)} disabled={ticket.is_default}>
                  <Settings className="h-3 w-3 mr-1" />
                  设为默认
                </Button>
                <Button variant="outline" size="sm" onClick={() => setEditDialog(ticket)}>
                  <Edit className="h-3 w-3 mr-1" />
                  编辑
                </Button>
                <Button variant="outline" size="sm" className="text-destructive hover:text-destructive" onClick={() => setDeleteTarget(ticket)}>
                  <Trash2 className="h-3 w-3 mr-1" />
                  删除
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}

        {ticketTypes.length === 0 && (
          <Card>
            <CardContent className="py-10 text-center text-muted-foreground">
              暂无票据类型，点击上方按钮创建
            </CardContent>
          </Card>
        )}
      </div>

      {/* EditDialog 在 loop 外，key={editDialog?.id} 確保切換 ticket 時 useState 重置 */}
      <EditTicketTypeDialog
        key={editDialog?.id ?? "none"}
        ticket={editDialog}
        open={!!editDialog}
        stations={stations}
        onSubmit={(req) => editDialog && handleUpdate(editDialog.id, req)}
        onCancel={() => setEditDialog(null)}
      />

      <CreateTicketTypeDialog
        stations={stations}
        open={createDialog}
        onSubmit={handleCreate}
        onCancel={() => setCreateDialog(false)}
      />

      {/* 刪除確認 Dialog */}
      <Dialog open={!!deleteTarget} onOpenChange={(v) => !v && setDeleteTarget(null)}>
        <DialogContent className="max-w-sm">
          <DialogHeader>
            <DialogTitle>确认删除</DialogTitle>
            <DialogDescription>
              删除「{deleteTarget?.name}」票据类型后不可恢复，确认继续？
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteTarget(null)}>取消</Button>
            <Button variant="destructive" onClick={() => deleteTarget && handleDelete(deleteTarget.id)}>
              删除
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

interface CreateTicketTypeRequest {
  code: string;
  name: string;
  description: string | null;
  is_active: boolean;
  is_default: boolean;
  show_price: boolean;
  show_seq: boolean;
  show_note_field: boolean;
  station_id: number | null;
  paper_width: string;
  font_size: string;
  cut_mode: string;
  print_speed: string;
  print_density: string;
  show_order_no: boolean;
  show_table_no: boolean;
  show_dine_type: boolean;
  show_item_name: boolean;
  show_item_qty: boolean;
  show_item_price: boolean;
  show_item_subtotal: boolean;
  show_item_spec: boolean;
  show_item_note: boolean;
  show_created_at: boolean;
  show_total_amount: boolean;
}

interface UpdateTicketTypeRequest extends CreateTicketTypeRequest {}

function CreateTicketTypeDialog({ stations, open, onSubmit, onCancel }: { stations: KitchenStation[]; open: boolean; onSubmit: (req: CreateTicketTypeRequest) => void; onCancel: () => void }) {
  const [code, setCode] = useState("");
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [stationId, setStationId] = useState<string>("");
  const [paperWidth, setPaperWidth] = useState("58mm");
  const [showPrice, setShowPrice] = useState(false);
  const [showSeq, setShowSeq] = useState(true);

  const handleSubmit = () => {
    onSubmit({
      code,
      name,
      description: description || null,
      is_active: true,
      is_default: false,
      show_price: showPrice,
      show_seq: showSeq,
      show_note_field: true,
      station_id: stationId ? parseInt(stationId) : null,
      paper_width: paperWidth,
      font_size: "medium",
      cut_mode: "full",
      print_speed: "medium",
      print_density: "medium",
      show_order_no: true,
      show_table_no: true,
      show_dine_type: true,
      show_item_name: true,
      show_item_qty: true,
      show_item_price: showPrice,
      show_item_subtotal: showPrice,
      show_item_spec: true,
      show_item_note: true,
      show_created_at: true,
      show_total_amount: showPrice,
    });
  };

  return (
    <Dialog open={open} onOpenChange={(v) => !v && onCancel()}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>新建票据类型</DialogTitle>
          <DialogDescription>创建新的打印票据类型配置</DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label>编码</Label>
              <Input value={code} onChange={(e) => setCode(e.target.value)} placeholder="如: receipt, kitchen, label" />
            </div>
            <div className="space-y-2">
              <Label>名称</Label>
              <Input value={name} onChange={(e) => setName(e.target.value)} placeholder="如: 出餐单" />
            </div>
          </div>
          <div className="space-y-2">
            <Label>描述</Label>
            <Input value={description} onChange={(e) => setDescription(e.target.value)} placeholder="用途描述" />
          </div>
          <div className="grid grid-cols-3 gap-4">
            <div className="space-y-2">
              <Label>纸宽</Label>
              <Select value={paperWidth} onValueChange={setPaperWidth}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="58mm">58mm</SelectItem>
                  <SelectItem value="80mm">80mm</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>工作站</Label>
              <Select value={stationId} onValueChange={setStationId}>
                <SelectTrigger><SelectValue placeholder="全部" /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="">全部工作站</SelectItem>
                  {stations.map(s => (
                    <SelectItem key={s.id} value={String(s.id)}>{s.name}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
          <div className="space-y-2">
            <Label>配置选项</Label>
            <div className="flex flex-wrap gap-4">
              <div className="flex items-center gap-2">
                <Checkbox id="create-showPrice" checked={showPrice} onCheckedChange={(v) => setShowPrice(!!v)} />
                <Label htmlFor="create-showPrice">显示单价</Label>
              </div>
              <div className="flex items-center gap-2">
                <Checkbox id="create-showSeq" checked={showSeq} onCheckedChange={(v) => setShowSeq(!!v)} />
                <Label htmlFor="create-showSeq">显示序号</Label>
              </div>
            </div>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onCancel}>取消</Button>
          <Button onClick={handleSubmit} disabled={!code || !name}>创建</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function EditTicketTypeDialog({ ticket, open, stations, onSubmit, onCancel }: { ticket: PrintTicketType | null; open: boolean; stations: KitchenStation[]; onSubmit: (req: UpdateTicketTypeRequest) => void; onCancel: () => void }) {
  if (!ticket) return null;

  const [name, setName] = useState(ticket.name);
  const [description, setDescription] = useState(ticket.description || "");
  const [stationId, setStationId] = useState(ticket.station_id ? String(ticket.station_id) : "");
  const [paperWidth, setPaperWidth] = useState(ticket.paper_width);
  const [fontSize, setFontSize] = useState(ticket.font_size);
  const [cutMode, setCutMode] = useState(ticket.cut_mode);
  const [showPrice, setShowPrice] = useState(ticket.show_price);
  const [showSeq, setShowSeq] = useState(ticket.show_seq);
  const [showOrderNo, setShowOrderNo] = useState(ticket.show_order_no);
  const [showTableNo, setShowTableNo] = useState(ticket.show_table_no);
  const [showDineType, setShowDineType] = useState(ticket.show_dine_type);
  const [showItemName, setShowItemName] = useState(ticket.show_item_name);
  const [showItemQty, setShowItemQty] = useState(ticket.show_item_qty);
  const [showItemPrice, setShowItemPrice] = useState(ticket.show_item_price);
  const [showItemSubtotal, setShowItemSubtotal] = useState(ticket.show_item_subtotal);
  const [showItemSpec, setShowItemSpec] = useState(ticket.show_item_spec);
  const [showItemNote, setShowItemNote] = useState(ticket.show_item_note);
  const [showTotalAmount, setShowTotalAmount] = useState(ticket.show_total_amount);
  const [isActive, setIsActive] = useState(ticket.is_active);

  const handleSubmit = () => {
    onSubmit({
      code: ticket.code,
      name,
      description: description || null,
      is_active: isActive,
      is_default: ticket.is_default,
      show_price: showPrice,
      show_seq: showSeq,
      show_note_field: ticket.show_note_field,
      station_id: stationId ? parseInt(stationId) : null,
      paper_width: paperWidth,
      font_size: fontSize,
      cut_mode: cutMode,
      print_speed: ticket.print_speed,
      print_density: ticket.print_density,
      show_order_no: showOrderNo,
      show_table_no: showTableNo,
      show_dine_type: showDineType,
      show_item_name: showItemName,
      show_item_qty: showItemQty,
      show_item_price: showItemPrice,
      show_item_subtotal: showItemSubtotal,
      show_item_spec: showItemSpec,
      show_item_note: showItemNote,
      show_created_at: ticket.show_created_at,
      show_total_amount: showTotalAmount,
    });
  };

  return (
    <Dialog open={open} onOpenChange={(v) => !v && onCancel()}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>编辑票据类型 — {ticket.name}</DialogTitle>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label>名称</Label>
              <Input value={name} onChange={(e) => setName(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label>描述</Label>
              <Input value={description} onChange={(e) => setDescription(e.target.value)} />
            </div>
          </div>
          <div className="grid grid-cols-3 gap-4">
            <div className="space-y-2">
              <Label>纸宽</Label>
              <Select value={paperWidth} onValueChange={setPaperWidth}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="58mm">58mm</SelectItem>
                  <SelectItem value="80mm">80mm</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>字体</Label>
              <Select value={fontSize} onValueChange={setFontSize}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="small">小</SelectItem>
                  <SelectItem value="medium">中</SelectItem>
                  <SelectItem value="large">大</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>切割</Label>
              <Select value={cutMode} onValueChange={setCutMode}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="full">全切</SelectItem>
                  <SelectItem value="half">半切</SelectItem>
                  <SelectItem value="none">不切</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
          <div className="space-y-2">
            <Label>工作站绑定</Label>
            <Select value={stationId} onValueChange={setStationId}>
              <SelectTrigger><SelectValue placeholder="全部工作站" /></SelectTrigger>
              <SelectContent>
                <SelectItem value="">全部工作站</SelectItem>
                {stations.map(s => (
                  <SelectItem key={s.id} value={String(s.id)}>{s.name}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>显示配置</Label>
            <div className="grid grid-cols-4 gap-2">
              {[
                { id: "edit-showPrice", label: "单价", val: showPrice, set: setShowPrice },
                { id: "edit-showSeq", label: "序号", val: showSeq, set: setShowSeq },
                { id: "edit-showOrderNo", label: "订单号", val: showOrderNo, set: setShowOrderNo },
                { id: "edit-showTableNo", label: "桌号", val: showTableNo, set: setShowTableNo },
                { id: "edit-showDineType", label: "用餐类型", val: showDineType, set: setShowDineType },
                { id: "edit-showItemName", label: "菜品名", val: showItemName, set: setShowItemName },
                { id: "edit-showItemQty", label: "数量", val: showItemQty, set: setShowItemQty },
                { id: "edit-showItemPrice", label: "小计单价", val: showItemPrice, set: setShowItemPrice },
                { id: "edit-showItemSubtotal", label: "小计", val: showItemSubtotal, set: setShowItemSubtotal },
                { id: "edit-showItemSpec", label: "规格", val: showItemSpec, set: setShowItemSpec },
                { id: "edit-showItemNote", label: "备注", val: showItemNote, set: setShowItemNote },
                { id: "edit-showTotalAmount", label: "合计", val: showTotalAmount, set: setShowTotalAmount },
              ].map(({ id, label, val, set }) => (
                <div key={id} className="flex items-center gap-2">
                  <Checkbox id={id} checked={val} onCheckedChange={(v) => set(!!v)} />
                  <Label htmlFor={id}>{label}</Label>
                </div>
              ))}
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Checkbox id="edit-isActive" checked={isActive} onCheckedChange={(v) => setIsActive(!!v)} />
            <Label htmlFor="edit-isActive">启用状态</Label>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onCancel}>取消</Button>
          <Button onClick={handleSubmit}>保存</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
