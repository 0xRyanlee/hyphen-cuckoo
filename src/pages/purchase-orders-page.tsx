import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { call as invoke } from "@/lib/transport";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Plus, ShoppingCart, Eye, Trash2, Truck, Package, FileBox } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state";
import { toast } from "sonner";
import { parseSafeFloat } from "@/lib/utils";

interface PurchaseOrder {
  id: number;
  po_no: string;
  supplier_id: number | null;
  supplier_name: string | null;
  status: string;
  expected_date: string | null;
  total_cost: number;
  created_at: string;
}

interface PurchaseOrderItem {
  id: number;
  po_id: number;
  material_id: number;
  material_name: string | null;
  qty: number;
  unit_id: number | null;
  unit_name: string | null;
  cost_per_unit: number;
  received_qty: number;
}

interface PurchaseOrderWithItems {
  order: PurchaseOrder;
  items: PurchaseOrderItem[];
}

interface Material { id: number; code: string; name: string; }
interface Unit { id: number; code: string; name: string; }
interface Supplier { id: number; name: string; }

interface PurchaseOrdersPageProps {
  orders: PurchaseOrder[];
  materials: Material[];
  units: Unit[];
  suppliers: Supplier[];
  onCreateOrder: (data: { supplier_id: number | null; expected_date: string | null }) => void;
  onAddItem: (data: { po_id: number; material_id: number; qty: number; unit_id: number | null; cost_per_unit: number }) => void;
  onViewOrder: (po_id: number) => void;
  onDeleteOrder: (po_id: number) => void;
  onReceiveOrder: (po_id: number) => void;
  onReceiveItems: (po_id: number, items: { item_id: number; received_qty: number; lot_no: string | null }[]) => void;
  selectedOrder: PurchaseOrderWithItems | null;
  searchQuery?: string;
}

export function PurchaseOrdersPage({
  orders, materials, units, suppliers,
  onCreateOrder, onAddItem, onViewOrder, onDeleteOrder, onReceiveOrder: _onReceiveOrder, onReceiveItems,
  selectedOrder,
  searchQuery,
}: PurchaseOrdersPageProps) {
  const filteredOrders = orders.filter((o) => {
    if (!searchQuery) return true;
    const q = searchQuery.toLowerCase();
    return o.po_no.toLowerCase().includes(q) || (o.supplier_name || "").toLowerCase().includes(q) || o.status.toLowerCase().includes(q);
  });
  const [newSupplierId, setNewSupplierId] = useState("");
  const [newExpectedDate, setNewExpectedDate] = useState("");

  const [addItemPoId, setAddItemPoId] = useState<number | null>(null);
  const [addItemRows, setAddItemRows] = useState<{ materialId: string; qty: string; unitId: string; cost: string }[]>([]);

  function openAddItem(poId: number) {
    setAddItemPoId(poId);
    setAddItemRows([{ materialId: "", qty: "1", unitId: "", cost: "0" }]);
  }
  function updateAddRow(i: number, key: string, val: string) {
    setAddItemRows((prev) => prev.map((r, idx) => idx === i ? { ...r, [key]: val } : r));
  }
  function addAddRow() {
    setAddItemRows((prev) => [...prev, { materialId: "", qty: "1", unitId: "", cost: "0" }]);
  }
  function removeAddRow(i: number) {
    setAddItemRows((prev) => prev.filter((_, idx) => idx !== i));
  }

  const [receivePoId, setReceivePoId] = useState<number | null>(null);
  const [receiveExpiryDate, setReceiveExpiryDate] = useState("");
  const [receiveOrderItems, setReceiveOrderItems] = useState<PurchaseOrderItem[]>([]);
  const [receiveItemQtys, setReceiveItemQtys] = useState<Record<number, string>>({});

  const getStatusBadge = (status: string) => {
    switch (status) {
      case "draft": return <Badge variant="outline">草稿</Badge>;
      case "confirmed": return <Badge variant="default">已确认</Badge>;
      case "received": return <Badge variant="default">已收貨</Badge>;
      case "partial": return <Badge className="bg-amber-500">部分收貨</Badge>;
      case "cancelled": return <Badge variant="destructive">已取消</Badge>;
      default: return <Badge variant="secondary">{status}</Badge>;
    }
  };

  const navigate = useNavigate();
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">采购管理</h2>
          <p className="text-sm text-muted-foreground">管理采购单、收货入库</p>
        </div>
      </div>
      <div className="flex border-b border-border">
        <button className="-mb-px pb-2 px-4 text-sm font-medium border-b-2 border-primary text-primary">采购单</button>
        <button className="-mb-px pb-2 px-4 text-sm font-medium border-b-2 border-transparent text-muted-foreground hover:text-foreground" onClick={() => navigate("/suppliers")}>供应商 & 商品</button>
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle className="flex items-center gap-2"><ShoppingCart className="h-4 w-4" />采购单列表</CardTitle>
            <CardDescription>共 {filteredOrders.length} 张采购单{filteredOrders.length !== orders.length ? `（筛选自 ${orders.length} 张）` : ""}</CardDescription>
          </CardHeader>
          <CardContent>
            {filteredOrders.length === 0 ? (
              <EmptyState icon={FileBox} title="暂无采购单" description="新增采购单开始采购" />
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>单号</TableHead>
                    <TableHead>供应商</TableHead>
                    <TableHead>状态</TableHead>
                    <TableHead className="text-right">总金额</TableHead>
                    <TableHead className="text-right">操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {filteredOrders.map((order) => (
                    <TableRow key={order.id}>
                      <TableCell className="font-mono text-xs">{order.po_no}</TableCell>
                      <TableCell className="text-muted-foreground">{order.supplier_name || "-"}</TableCell>
                      <TableCell>{getStatusBadge(order.status)}</TableCell>
                      <TableCell className="text-right font-medium">¥{order.total_cost.toFixed(2)}</TableCell>
                      <TableCell className="text-right">
                        <div className="flex justify-end gap-1">
                          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => onViewOrder(order.id)}><Eye className="h-4 w-4" /></Button>
                          {order.status === "draft" && (
                            <Button variant="ghost" size="icon" className="h-8 w-8 text-blue-500" onClick={() => openAddItem(order.id)}><Plus className="h-4 w-4" /></Button>
                          )}
                          {(order.status === "draft" || order.status === "partial") && (
                            <Button variant="ghost" size="icon" className="h-8 w-8 text-primary" onClick={async () => {
                              setReceivePoId(order.id);
                              setReceiveExpiryDate("");
                              try {
                                const data = await invoke<PurchaseOrderWithItems>("get_purchase_order_with_items", { poId: order.id });
                                const items = data.items.filter(i => i.qty - i.received_qty > 0.001);
                                setReceiveOrderItems(items);
                                const initQtys: Record<number, string> = {};
                                items.forEach(i => { initQtys[i.id] = (i.qty - i.received_qty).toFixed(2); });
                                setReceiveItemQtys(initQtys);
                              } catch { setReceiveOrderItems([]); setReceiveItemQtys({}); }
                            }}><Truck className="h-4 w-4" /></Button>
                          )}
                          {order.status === "draft" && (
                            <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive" onClick={() => onDeleteOrder(order.id)}><Trash2 className="h-4 w-4" /></Button>
                          )}
                        </div>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </CardContent>
        </Card>

        <div className="space-y-4">
          <Card>
            <CardHeader><CardTitle>新增采购单</CardTitle></CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label>供应商（可选）</Label>
                <Select value={newSupplierId} onValueChange={setNewSupplierId}>
                  <SelectTrigger><SelectValue placeholder="选择供应商" /></SelectTrigger>
                  <SelectContent>
                    {suppliers.map((s) => <SelectItem key={s.id} value={s.id.toString()}>{s.name}</SelectItem>)}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>预计到货日期</Label>
                <Input type="date" value={newExpectedDate} onChange={(e) => setNewExpectedDate(e.target.value)} />
              </div>
              <Button className="w-full" onClick={() => {
                onCreateOrder({
                  supplier_id: newSupplierId ? parseInt(newSupplierId) : null,
                  expected_date: newExpectedDate || null,
                });
                setNewSupplierId(""); setNewExpectedDate("");
              }}>
                <Plus className="mr-2 h-4 w-4" />新增采购单
              </Button>
            </CardContent>
          </Card>

          {selectedOrder && (
            <Card>
              <CardHeader>
                <CardTitle>采购单详情</CardTitle>
                <CardDescription>{selectedOrder.order.po_no}</CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="space-y-2 text-sm">
                  <div className="flex justify-between"><span className="text-muted-foreground">供应商</span><span>{selectedOrder.order.supplier_name || "-"}</span></div>
                  <div className="flex justify-between"><span className="text-muted-foreground">状态</span>{getStatusBadge(selectedOrder.order.status)}</div>
                  <div className="flex justify-between"><span className="text-muted-foreground">总金额</span><span className="font-medium">¥{selectedOrder.order.total_cost.toFixed(2)}</span></div>
                </div>
                <div>
                  <h4 className="text-sm font-medium mb-2 flex items-center gap-1"><Package className="h-3 w-3" />材料明细</h4>
                  {selectedOrder.items.length === 0 ? (
                    <EmptyState icon={Package} title="暂无材料" description="采购单中没有材料" />
                  ) : (
                    <div className="space-y-2">
                      {selectedOrder.items.map((item) => (
                        <div key={item.id} className="flex justify-between text-sm p-2 bg-muted rounded">
                          <span>{item.material_name || `材料 #${item.material_id}`} x{item.qty} {item.unit_name || ""}</span>
                          <span>¥{(item.qty * item.cost_per_unit).toFixed(2)}</span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      </div>

      <Dialog open={!!addItemPoId} onOpenChange={() => setAddItemPoId(null)}>
        <DialogContent className="max-w-2xl">
          <DialogHeader><DialogTitle>添加采购材料</DialogTitle></DialogHeader>
          <div className="py-2 space-y-2">
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b text-muted-foreground">
                    <th className="text-left py-1.5 pr-2 font-medium w-[40%]">材料</th>
                    <th className="text-left py-1.5 pr-2 font-medium w-[12%]">数量</th>
                    <th className="text-left py-1.5 pr-2 font-medium w-[18%]">单位</th>
                    <th className="text-left py-1.5 pr-2 font-medium w-[16%]">单价</th>
                    <th className="w-8" />
                  </tr>
                </thead>
                <tbody>
                  {addItemRows.map((row, i) => (
                    <tr key={i} className="border-b last:border-b-0">
                      <td className="py-1.5 pr-2">
                        <Select value={row.materialId} onValueChange={(v) => updateAddRow(i, "materialId", v)}>
                          <SelectTrigger className="h-8"><SelectValue placeholder="选择材料" /></SelectTrigger>
                          <SelectContent>
                            {materials.map((m) => <SelectItem key={m.id} value={m.id.toString()}>{m.name} ({m.code})</SelectItem>)}
                          </SelectContent>
                        </Select>
                      </td>
                      <td className="py-1.5 pr-2">
                        <Input type="number" className="h-8 w-full" value={row.qty} onChange={(e) => updateAddRow(i, "qty", e.target.value)} step="0.01" min="0" />
                      </td>
                      <td className="py-1.5 pr-2">
                        <Select value={row.unitId} onValueChange={(v) => updateAddRow(i, "unitId", v)}>
                          <SelectTrigger className="h-8"><SelectValue placeholder="单位" /></SelectTrigger>
                          <SelectContent>
                            {units.map((u) => <SelectItem key={u.id} value={u.id.toString()}>{u.name}</SelectItem>)}
                          </SelectContent>
                        </Select>
                      </td>
                      <td className="py-1.5 pr-2">
                        <Input type="number" className="h-8 w-full" value={row.cost} onChange={(e) => updateAddRow(i, "cost", e.target.value)} step="0.01" min="0" />
                      </td>
                      <td className="py-1.5">
                        <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive/60 hover:text-destructive" onClick={() => removeAddRow(i)} disabled={addItemRows.length === 1}>
                          <Trash2 className="h-3.5 w-3.5" />
                        </Button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            <Button variant="outline" size="sm" className="w-full" onClick={addAddRow}>
              <Plus className="h-3.5 w-3.5 mr-1" />新增一行
            </Button>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setAddItemPoId(null)}>取消</Button>
            <Button onClick={() => {
              if (!addItemPoId) return;
              const validRows = addItemRows.filter(r => r.materialId);
              if (validRows.length === 0) { toast.error("请至少选择一种材料"); return; }
              for (const row of validRows) {
                const qty = parseSafeFloat(row.qty);
                const cost = parseSafeFloat(row.cost);
                if (!qty || qty <= 0) { toast.error(`材料 ${materials.find(m => m.id.toString() === row.materialId)?.name || ""} 数量无效`); return; }
                onAddItem({
                  po_id: addItemPoId,
                  material_id: parseInt(row.materialId),
                  qty,
                  unit_id: row.unitId ? parseInt(row.unitId) : null,
                  cost_per_unit: cost ?? 0,
                });
              }
              setAddItemPoId(null);
            }}>添加 {addItemRows.filter(r => r.materialId).length} 项</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={!!receivePoId} onOpenChange={(open) => { if (!open) { setReceivePoId(null); setReceiveOrderItems([]); setReceiveItemQtys({}); } }}>
        <DialogContent className="max-w-lg">
          <DialogHeader><DialogTitle>收货入库</DialogTitle></DialogHeader>
          <div className="space-y-4 py-4">
            {receiveOrderItems.length === 0 ? (
              <p className="text-sm text-muted-foreground">加载中…</p>
            ) : (
              <div className="space-y-3">
                <p className="text-xs text-muted-foreground">请确认各材料的实收数量（只有数量 &gt; 0 的项会入库）</p>
                {receiveOrderItems.map((item) => (
                  <div key={item.id} className="flex items-center gap-3">
                    <div className="flex-1 text-sm">
                      <div className="font-medium">{item.material_name || `材料 #${item.material_id}`}</div>
                      <div className="text-xs text-muted-foreground">已订 {item.qty} {item.unit_name || ""} · 已收 {item.received_qty}</div>
                    </div>
                    <div className="w-24">
                      <Input
                        type="number"
                        step="0.01"
                        min="0"
                        className="h-7 text-xs"
                        value={receiveItemQtys[item.id] ?? ""}
                        onChange={(e) => setReceiveItemQtys((prev) => ({ ...prev, [item.id]: e.target.value }))}
                      />
                    </div>
                  </div>
                ))}
              </div>
            )}
            <div className="space-y-2">
              <Label>过期日期（统一，可选）</Label>
              <Input type="date" value={receiveExpiryDate} onChange={(e) => setReceiveExpiryDate(e.target.value)} />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => { setReceivePoId(null); setReceiveOrderItems([]); setReceiveItemQtys({}); }}>取消</Button>
            <Button onClick={() => {
              if (!receivePoId) return;
              const itemsToReceive = receiveOrderItems
                .map((item) => ({ item_id: item.id, received_qty: parseFloat(receiveItemQtys[item.id] ?? "0") || 0, lot_no: null as string | null }))
                .filter((i) => i.received_qty > 0);
              if (itemsToReceive.length === 0) { toast.error("请至少输入一项的收货数量"); return; }
              onReceiveItems(receivePoId, itemsToReceive);
              setReceivePoId(null); setReceiveOrderItems([]); setReceiveItemQtys({});
            }}>
              <Truck className="mr-2 h-4 w-4" />确认收货
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
