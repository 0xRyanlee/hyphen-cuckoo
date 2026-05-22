import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Plus, ChefHat, Eye, Trash2, Play, CheckCircle, Package, Factory } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state";

interface ProductionOrder {
  id: number;
  production_no: string;
  recipe_id: number;
  recipe_name: string | null;
  status: string;
  planned_qty: number;
  actual_qty: number | null;
  operator: string | null;
  started_at: string | null;
  completed_at: string | null;
  created_at: string;
}

interface ProductionOrderItem {
  id: number;
  production_id: number;
  material_id: number;
  material_name: string | null;
  lot_id: number | null;
  planned_qty: number;
  actual_qty: number | null;
}

interface ProductionOrderWithItems {
  order: ProductionOrder;
  items: ProductionOrderItem[];
}

interface Recipe { id: number; code: string; name: string; }

interface ProductionOrdersPageProps {
  orders: ProductionOrder[];
  recipes: Recipe[];
  onCreateOrder: (data: { recipe_id: number; planned_qty: number; operator: string | null }) => void;
  onStartOrder: (production_id: number) => void;
  onCompleteOrder: (production_id: number, actual_qty: number) => void;
  onViewOrder: (production_id: number) => void;
  onDeleteOrder: (production_id: number) => void;
  selectedOrder: ProductionOrderWithItems | null;
  searchQuery?: string;
}

export function ProductionOrdersPage({
  orders, recipes,
  onCreateOrder, onStartOrder, onCompleteOrder, onViewOrder, onDeleteOrder,
  selectedOrder,
  searchQuery,
}: ProductionOrdersPageProps) {
  const filteredOrders = orders.filter((o) => {
    if (!searchQuery) return true;
    const q = searchQuery.toLowerCase();
    return o.production_no.toLowerCase().includes(q) || (o.recipe_name || "").toLowerCase().includes(q) || o.status.toLowerCase().includes(q);
  });
  const [newRecipeId, setNewRecipeId] = useState("");
  const [newPlannedQty, setNewPlannedQty] = useState("1");
  const [newOperator, setNewOperator] = useState("");
  const [materialCheck, setMaterialCheck] = useState<[string, number, number][] | null>(null);

  useEffect(() => {
    if (!newRecipeId) { setMaterialCheck(null); return; }
    const qty = parseFloat(newPlannedQty) || 1;
    invoke<[string, number, number][]>("check_production_materials", { recipeId: parseInt(newRecipeId), plannedQty: qty })
      .then(setMaterialCheck)
      .catch(() => setMaterialCheck(null));
  }, [newRecipeId, newPlannedQty]);

  const [completeId, setCompleteId] = useState<number | null>(null);
  const [completeActualQty, setCompleteActualQty] = useState("1");

  const getStatusBadge = (status: string) => {
    switch (status) {
      case "draft": return <Badge variant="outline">草稿</Badge>;
      case "in_progress": return <Badge>生产中</Badge>;
      case "completed": return <Badge variant="secondary">已完成</Badge>;
      case "cancelled": return <Badge variant="destructive">已取消</Badge>;
      default: return <Badge variant="secondary">{status}</Badge>;
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">生产管理</h2>
          <p className="text-sm text-muted-foreground">管理生产单、半成品加工</p>
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle className="flex items-center gap-2"><ChefHat className="h-4 w-4" />生产单列表</CardTitle>
            <CardDescription>共 {filteredOrders.length} 张生产单{filteredOrders.length !== orders.length ? `（筛选自 ${orders.length} 张）` : ""}</CardDescription>
          </CardHeader>
          <CardContent>
            {filteredOrders.length === 0 ? (
              <EmptyState icon={Factory} title="暂无生产单" description="新增生产单开始生产" />
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>单号</TableHead>
                    <TableHead>配方</TableHead>
                    <TableHead>状态</TableHead>
                    <TableHead className="text-right">计划量</TableHead>
                    <TableHead className="text-right">实际量</TableHead>
                    <TableHead className="text-right">操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {filteredOrders.map((order) => (
                    <TableRow key={order.id}>
                      <TableCell className="font-mono text-xs">{order.production_no}</TableCell>
                      <TableCell className="text-muted-foreground">{order.recipe_name || `配方 #${order.recipe_id}`}</TableCell>
                      <TableCell>{getStatusBadge(order.status)}</TableCell>
                      <TableCell className="text-right">{order.planned_qty}</TableCell>
                      <TableCell className="text-right">{order.actual_qty ?? "-"}</TableCell>
                      <TableCell className="text-right">
                        <div className="flex justify-end gap-1">
                          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => onViewOrder(order.id)}><Eye className="h-4 w-4" /></Button>
                          {order.status === "draft" && (
                            <Button variant="ghost" size="icon" className="h-8 w-8 text-muted-foreground" onClick={() => onStartOrder(order.id)}><Play className="h-4 w-4" /></Button>
                          )}
                          {order.status === "in_progress" && (
                            <Button variant="ghost" size="icon" className="h-8 w-8 text-muted-foreground" onClick={() => { setCompleteId(order.id); setCompleteActualQty(order.planned_qty.toString()); }}><CheckCircle className="h-4 w-4" /></Button>
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
            <CardHeader><CardTitle>新增生产单</CardTitle></CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label>配方</Label>
                <Select value={newRecipeId} onValueChange={setNewRecipeId}>
                  <SelectTrigger><SelectValue placeholder="选择配方" /></SelectTrigger>
                  <SelectContent>
                    {recipes.map((r) => <SelectItem key={r.id} value={r.id.toString()}>{r.name} ({r.code})</SelectItem>)}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>计划产量</Label>
                <Input type="number" value={newPlannedQty} onChange={(e) => setNewPlannedQty(e.target.value)} step="0.01" />
              </div>
              <div className="space-y-2">
                <Label>操作人（可选）</Label>
                <Input value={newOperator} onChange={(e) => setNewOperator(e.target.value)} placeholder="操作人姓名" />
              </div>
              {materialCheck && materialCheck.length > 0 && (
                <div className="rounded-md border p-3 space-y-1.5">
                  <p className="text-xs font-medium text-muted-foreground">备料检查</p>
                  {materialCheck.map(([name, needed, available]) => {
                    const ok = available >= needed - 1e-9;
                    return (
                      <div key={name} className={`flex justify-between text-xs rounded px-2 py-1 ${ok ? "bg-emerald-500/10 text-emerald-700 dark:text-emerald-400" : "bg-destructive/10 text-destructive"}`}>
                        <span>{name}</span>
                        <span>{ok ? "✓" : "✗"} 需 {needed.toFixed(2)} / 有 {available.toFixed(2)}</span>
                      </div>
                    );
                  })}
                  {materialCheck.some(([, needed, available]) => available < needed - 1e-9) && (
                    <p className="text-xs text-destructive pt-1">部分原料不足，完成生产时将报错</p>
                  )}
                </div>
              )}
              <Button className="w-full" onClick={() => {
                if (newRecipeId) {
                  onCreateOrder({
                    recipe_id: parseInt(newRecipeId),
                    planned_qty: parseFloat(newPlannedQty) || 1,
                    operator: newOperator || null,
                  });
                  setNewRecipeId(""); setNewPlannedQty("1"); setNewOperator("");
                }
              }} disabled={!newRecipeId}>
                <Plus className="mr-2 h-4 w-4" />新增生产单
              </Button>
            </CardContent>
          </Card>

          {selectedOrder && (
            <Card>
              <CardHeader>
                <CardTitle>生产单详情</CardTitle>
                <CardDescription>{selectedOrder.order.production_no}</CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="space-y-2 text-sm">
                  <div className="flex justify-between"><span className="text-muted-foreground">配方</span><span>{selectedOrder.order.recipe_name || "-"}</span></div>
                  <div className="flex justify-between"><span className="text-muted-foreground">状态</span>{getStatusBadge(selectedOrder.order.status)}</div>
                  <div className="flex justify-between"><span className="text-muted-foreground">计划量</span><span>{selectedOrder.order.planned_qty}</span></div>
                  <div className="flex justify-between"><span className="text-muted-foreground">实际量</span><span>{selectedOrder.order.actual_qty ?? "-"}</span></div>
                  <div className="flex justify-between"><span className="text-muted-foreground">操作人</span><span>{selectedOrder.order.operator || "-"}</span></div>
                </div>
                <div>
                  <h4 className="text-sm font-medium mb-2 flex items-center gap-1"><Package className="h-3 w-3" />耗用材料</h4>
                  {selectedOrder.items.length === 0 ? (
                    <EmptyState icon={Package} title="暂无材料" description="生产单中没有耗用材料" />
                  ) : (
                    <div className="space-y-2">
                      {selectedOrder.items.map((item) => (
                        <div key={item.id} className="flex justify-between text-sm p-2 bg-muted rounded">
                          <span>{item.material_name || `材料 #${item.material_id}`}</span>
                          <span>{item.planned_qty}</span>
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

      <Dialog open={!!completeId} onOpenChange={() => setCompleteId(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>完成生产单</DialogTitle></DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>实际产量</Label>
              <Input type="number" value={completeActualQty} onChange={(e) => setCompleteActualQty(e.target.value)} step="0.01" />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setCompleteId(null)}>取消</Button>
            <Button onClick={() => {
              if (completeId) {
                const qty = parseFloat(completeActualQty);
                if (isNaN(qty) || qty <= 0) return;
                onCompleteOrder(completeId, qty);
                setCompleteId(null);
              }
            }}>完成</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
