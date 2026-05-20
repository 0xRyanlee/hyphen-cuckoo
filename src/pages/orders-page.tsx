import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Plus, ShoppingCart, Eye, Send, X, Search, Filter, MinusCircle, PlusCircle, Package } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state";
import { Checkbox } from "@/components/ui/checkbox";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { parseSafeFloat } from "@/lib/utils";

interface OrderItem {
  id: number;
  order_id: number;
  menu_item_id: number;
  spec_code: string | null;
  qty: number;
  unit_price: number;
  note: string | null;
}

interface OrderItemModifier {
  id: number;
  order_item_id: number;
  modifier_type: string;
  material_id: number | null;
  material_name: string | null;
  qty: number;
  price_delta: number;
}

interface Order {
  id: number;
  order_no: string;
  source: string;
  dine_type: string;
  table_no: string | null;
  status: string;
  amount_total: number;
  note: string | null;
  created_at: string;
  updated_at: string;
}

interface OrderWithItems {
  order: Order;
  items: OrderItem[];
}

interface Material { id: number; code: string; name: string; }

interface OrdersPageProps {
  orders: Order[];
  selectedOrder: OrderWithItems | null;
  menuItems: Record<number, string>;
  materials: Material[];
  onCreateOrder: () => void;
  onSubmitOrder: (id: number) => void;
  onCancelOrder: (id: number, is_served: boolean) => void;
  onBatchCancelOrder: (ids: number[]) => void;
  onViewOrder: (id: number) => void;
  onViewOrderWithModifiers: (id: number) => Promise<{ orderData: OrderWithItems; modifiers: Record<number, OrderItemModifier[]> }>;
  onAddModifier: (data: { order_item_id: number; modifier_type: string; material_id: number | null; qty: number; price_delta: number }) => void;
  onDeleteModifier: (modifier_id: number) => void;
  onLoadModifiers: (order_item_id: number) => Promise<OrderItemModifier[]>;
  onLoadMore: () => void;
  hasMore: boolean;
  searchQuery?: string;
}

export function OrdersPage({
  orders,
  selectedOrder,
  menuItems,
  materials,
  onCreateOrder,
  onSubmitOrder,
  onCancelOrder,
  onBatchCancelOrder,
  onViewOrder,
  onViewOrderWithModifiers,
  onAddModifier,
  onDeleteModifier,
  onLoadModifiers,
  onLoadMore,
  hasMore,
}: OrdersPageProps) {
  const [searchQuery, setSearchQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<string>("all");
  const [modifierDialogOpen, setModifierDialogOpen] = useState(false);
  const [modifierOrderItemId, setModifierOrderItemId] = useState<number | null>(null);
  const [modifierType, setModifierType] = useState("add");
  const [modifierMaterialId, setModifierMaterialId] = useState("");
  const [modifierQty, setModifierQty] = useState("1");
  const [modifierPriceDelta, setModifierPriceDelta] = useState("0");
  const [selectedOrders, setSelectedOrders] = useState<number[]>([]);
  const [cancelDialogOpen, setCancelDialogOpen] = useState(false);
  const [cancelTargetOrder, setCancelTargetOrder] = useState<Order | null>(null);
  const [cancelIsServed, setCancelIsServed] = useState(false);
  const [batchCancelDialogOpen, setBatchCancelDialogOpen] = useState(false);
  const [orderCost, setOrderCost] = useState<number | null>(null);

  const [itemModifiers, setItemModifiers] = useState<Record<number, OrderItemModifier[]>>({});

  useEffect(() => {
    if (!selectedOrder) { setOrderCost(null); return; }
    let cancelled = false;
    setOrderCost(null);
    invoke<number>("get_order_cost", { orderId: selectedOrder.order.id })
      .then((cost) => { if (!cancelled) setOrderCost(cost); })
      .catch(() => { if (!cancelled) setOrderCost(null); });
    return () => { cancelled = true; };
  }, [selectedOrder?.order.id]);

  const toggleSelectOrder = (id: number) => {
    setSelectedOrders(prev => prev.includes(id) ? prev.filter(i => i !== id) : [...prev, id]);
  };
  const selectAllOrders = () => {
    if (selectedOrders.length === filteredOrders.length) {
      setSelectedOrders([]);
    } else {
      setSelectedOrders(filteredOrders.map(o => o.id));
    }
  };
  const handleBatchCancel = () => {
    if (selectedOrders.length > 0) setBatchCancelDialogOpen(true);
  };
  const confirmBatchCancel = () => {
    if (onBatchCancelOrder && selectedOrders.length > 0) {
      onBatchCancelOrder(selectedOrders);
      setSelectedOrders([]);
    }
    setBatchCancelDialogOpen(false);
  };

  const filteredOrders = orders.filter((order) => {
    const matchesSearch = !searchQuery ||
      order.order_no.toLowerCase().includes(searchQuery.toLowerCase()) ||
      order.dine_type.toLowerCase().includes(searchQuery.toLowerCase()) ||
      order.status.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesStatus = statusFilter === "all" || order.status === statusFilter;
    return matchesSearch && matchesStatus;
  });
  const getStatusBadge = (status: string) => {
    switch (status) {
      case "pending":
        return <Badge variant="outline">待提交</Badge>;
      case "submitted":
        return <Badge className="bg-blue-600">已提交</Badge>;
      case "ready":
        return <Badge className="bg-emerald-600">已完成</Badge>;
      case "cancelled":
        return <Badge variant="destructive">已取消</Badge>;
      default:
        return <Badge variant="secondary">{status}</Badge>;
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">订单管理</h2>
          <p className="text-sm text-muted-foreground">查看和管理所有订单</p>
        </div>
        <Button onClick={onCreateOrder}>
          <Plus className="mr-2 h-4 w-4" />
          创建订单
        </Button>
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        <Card className="lg:col-span-2">
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle className="flex items-center gap-2">
                  <ShoppingCart className="h-4 w-4" />
                  订单列表
                </CardTitle>
                <CardDescription>共 {filteredOrders.length} 个订单{filteredOrders.length !== orders.length ? `（筛选自 ${orders.length} 个）` : ""}</CardDescription>
              </div>
              {selectedOrders.length > 0 && (
                <div className="flex gap-2">
                  <span className="text-sm text-muted-foreground self-center">已选 {selectedOrders.length} 单</span>
                  <Button size="sm" variant="outline" onClick={handleBatchCancel}>批量取消</Button>
                </div>
              )}
            </div>
          </CardHeader>
          <CardContent>
            <div className="flex gap-2 mb-4">
              <div className="relative flex-1 max-w-xs">
                <Search className="absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  placeholder="搜索订单号、类型..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-8"
                />
              </div>
              <div className="flex items-center gap-1">
                <Filter className="h-4 w-4 text-muted-foreground" />
                <Select value={statusFilter} onValueChange={setStatusFilter}>
                  <SelectTrigger className="w-36"><SelectValue placeholder="状态筛选" /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">全部状态</SelectItem>
                    <SelectItem value="pending">待提交</SelectItem>
                    <SelectItem value="submitted">已提交</SelectItem>
                    <SelectItem value="ready">已完成</SelectItem>
                    <SelectItem value="cancelled">已取消</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            {orders.length === 0 ? (
              <EmptyState icon={ShoppingCart} title="暂无订单" description="创建订单后将在此显示" />
            ) : filteredOrders.length === 0 ? (
              <EmptyState icon={Search} title="无搜索结果" description="尝试调整搜索条件" />
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-10">
                      <Checkbox checked={selectedOrders.length === filteredOrders.length && filteredOrders.length > 0} onClick={selectAllOrders} />
                    </TableHead>
                    <TableHead>订单号</TableHead>
                    <TableHead>类型</TableHead>
                    <TableHead>状态</TableHead>
                    <TableHead className="text-right">总额</TableHead>
                    <TableHead className="text-right">操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {filteredOrders.map((order) => (
                    <TableRow key={order.id}>
                      <TableCell><Checkbox checked={selectedOrders.includes(order.id)} onClick={() => toggleSelectOrder(order.id)} /></TableCell>
                      <TableCell className="font-mono text-xs">{order.order_no}</TableCell>
                      <TableCell className="text-muted-foreground">{order.dine_type}</TableCell>
                      <TableCell>{getStatusBadge(order.status)}</TableCell>
                      <TableCell className="text-right font-medium">
                        ¥{order.amount_total.toFixed(2)}
                      </TableCell>
                      <TableCell className="text-right">
                        <div className="flex justify-end gap-1">
                          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={async () => { onViewOrder(order.id); const { orderData } = await onViewOrderWithModifiers(order.id); const mods: Record<number, OrderItemModifier[]> = {}; for (const item of orderData.items) { try { mods[item.id] = await onLoadModifiers(item.id); } catch { mods[item.id] = []; } } setItemModifiers(mods); }}>
                            <Eye className="h-4 w-4" />
                          </Button>
                          {order.status === "pending" && (
                            <>
                              <Button variant="ghost" size="icon" className="h-8 w-8 text-blue-500" onClick={() => onSubmitOrder(order.id)}>
                                <Send className="h-4 w-4" />
                              </Button>
                              <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive" onClick={() => { setCancelTargetOrder(order); setCancelIsServed(false); setCancelDialogOpen(true); }}>
                                <X className="h-4 w-4" />
                              </Button>
                            </>
                          )}
                        </div>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
            {hasMore && onLoadMore && (
              <div className="flex justify-center pt-2 pb-4">
                <Button variant="outline" size="sm" onClick={onLoadMore}>载入更多订单</Button>
              </div>
            )}
          </CardContent>
        </Card>

        {selectedOrder && (
          <Card>
            <CardHeader>
              <CardTitle>订单详情</CardTitle>
              <CardDescription>{selectedOrder.order.order_no}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <div className="flex justify-between text-sm">
                  <span className="text-muted-foreground">状态</span>
                  {getStatusBadge(selectedOrder.order.status)}
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-muted-foreground">类型</span>
                  <span>{selectedOrder.order.dine_type}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-muted-foreground">总额</span>
                  <span className="font-medium">¥{selectedOrder.order.amount_total.toFixed(2)}</span>
                </div>
                {orderCost !== null && (
                  <>
                    <div className="flex justify-between text-sm">
                      <span className="text-muted-foreground">食材成本</span>
                      <span className="font-medium text-amber-600">¥{orderCost.toFixed(2)}</span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span className="text-muted-foreground">毛利</span>
                      <span className={`font-medium ${selectedOrder.order.amount_total - orderCost >= 0 ? "text-emerald-600" : "text-destructive"}`}>
                        ¥{(selectedOrder.order.amount_total - orderCost).toFixed(2)}
                        {selectedOrder.order.amount_total > 0 && (
                          <span className="text-xs ml-1 opacity-70">
                            ({((selectedOrder.order.amount_total - orderCost) / selectedOrder.order.amount_total * 100).toFixed(1)}%)
                          </span>
                        )}
                      </span>
                    </div>
                  </>
                )}
                {orderCost === null && (
                  <div className="text-xs text-muted-foreground">（毛利核算中…）</div>
                )}
              </div>
              <div>
                <h4 className="text-sm font-medium mb-2">商品明细</h4>
                {selectedOrder.items.length === 0 ? (
                  <EmptyState icon={Package} title="暂无商品" description="订单中没有商品" />
                ) : (
                  <ScrollArea className="max-h-[50vh]">
                  <div className="space-y-3 pr-2">
                    {selectedOrder.items.map((item) => (
                      <div key={item.id} className="p-2 bg-muted rounded space-y-1">
                        <div className="flex justify-between text-sm">
                          <span className="font-medium">
                            {menuItems[item.menu_item_id] || `商品 #${item.menu_item_id}`} x{item.qty}
                          </span>
                          <span>¥{(item.qty * item.unit_price).toFixed(2)}</span>
                        </div>
                        {item.note && <div className="text-xs text-amber-500 ml-2">備註: {item.note}</div>}
                        <div className="flex items-center gap-1 mt-1">
                          <Button variant="ghost" size="sm" className="h-5 px-1.5 text-xs" onClick={() => {
                            setModifierOrderItemId(item.id);
                            setModifierType("add");
                            setModifierMaterialId("");
                            setModifierQty("1");
                            setModifierPriceDelta("0");
                            setModifierDialogOpen(true);
                          }}><PlusCircle className="h-3 w-3 mr-1" />加料</Button>
                          <Button variant="ghost" size="sm" className="h-5 px-1.5 text-xs" onClick={() => {
                            setModifierOrderItemId(item.id);
                            setModifierType("remove");
                            setModifierMaterialId("");
                            setModifierQty("1");
                            setModifierPriceDelta("0");
                            setModifierDialogOpen(true);
                          }}><MinusCircle className="h-3 w-3 mr-1" />去料</Button>
                          {itemModifiers[item.id] && itemModifiers[item.id].length > 0 && (
                            <div className="flex flex-wrap gap-1 ml-2">
                              {itemModifiers[item.id].map((mod) => (
                                <Badge key={mod.id} variant={mod.modifier_type === "add" ? "default" : "destructive"} className="text-xs h-5 flex items-center gap-1">
                                  {mod.modifier_type === "add" ? "+" : "-"}{mod.material_name || "材料"} x{mod.qty}
                                  <X className="h-2.5 w-2.5 cursor-pointer ml-0.5" onClick={() => { onDeleteModifier(mod.id); setItemModifiers((prev) => ({ ...prev, [item.id]: prev[item.id]?.filter((m) => m.id !== mod.id) || [] })); }} />
                                </Badge>
                              ))}
                            </div>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                  </ScrollArea>
                )}
              </div>
            </CardContent>
          </Card>
        )}
      </div>

      <Dialog open={cancelDialogOpen} onOpenChange={setCancelDialogOpen}>
        <DialogContent>
          <DialogHeader><DialogTitle>取消订单</DialogTitle></DialogHeader>
          <div className="space-y-4 py-4">
            <p className="text-sm text-muted-foreground">
              确定要取消订单「{cancelTargetOrder?.order_no}」吗？
            </p>
            <div className="space-y-2">
              <Label>取消时是否扣除食材成本？</Label>
              <div className="flex gap-2">
                <Button
                  variant={cancelIsServed ? "outline" : "default"}
                  onClick={() => setCancelIsServed(false)}
                >
                  未出餐（不扣成本）
                </Button>
                <Button
                  variant={cancelIsServed ? "default" : "outline"}
                  onClick={() => setCancelIsServed(true)}
                >
                  已出餐（扣除食材）
                </Button>
              </div>
              {cancelIsServed && (
                <p className="text-xs text-amber-500">已出餐订单将扣除已用食材成本</p>
              )}
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setCancelDialogOpen(false)}>取消</Button>
            <Button variant="destructive" onClick={() => {
              if (cancelTargetOrder) onCancelOrder(cancelTargetOrder.id, cancelIsServed);
              setCancelDialogOpen(false);
            }}>确认取消</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={batchCancelDialogOpen} onOpenChange={setBatchCancelDialogOpen}>
        <DialogContent>
          <DialogHeader><DialogTitle>批量取消订单</DialogTitle></DialogHeader>
          <div className="py-4">
            <p className="text-sm text-muted-foreground">
              确定要取消已选的 {selectedOrders.length} 笔订单吗？此操作将释放相关库存，不可撤销。
            </p>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setBatchCancelDialogOpen(false)}>返回</Button>
            <Button variant="destructive" onClick={confirmBatchCancel}>确认取消</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={modifierDialogOpen} onOpenChange={setModifierDialogOpen}>
        <DialogContent>
          <DialogHeader><DialogTitle>{modifierType === "add" ? "加料" : "去料"}</DialogTitle></DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>材料</Label>
              <Select value={modifierMaterialId} onValueChange={setModifierMaterialId}>
                <SelectTrigger><SelectValue placeholder="选择材料" /></SelectTrigger>
                <SelectContent>
                  {materials.map((m) => <SelectItem key={m.id} value={m.id.toString()}>{m.name} ({m.code})</SelectItem>)}
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>数量</Label>
                <Input type="number" value={modifierQty} onChange={(e) => setModifierQty(e.target.value)} step="0.01" />
              </div>
              <div className="space-y-2">
                <Label>价格调整</Label>
                <Input type="number" value={modifierPriceDelta} onChange={(e) => setModifierPriceDelta(e.target.value)} step="0.01" />
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setModifierDialogOpen(false)}>取消</Button>
            <Button onClick={() => {
              if (!modifierOrderItemId || !modifierMaterialId) { toast.error("请选择材料和数量"); return; }
              const qty = parseSafeFloat(modifierQty);
              if (qty === null || qty <= 0) { toast.error("数量格式错误"); return; }
              const priceDelta = parseSafeFloat(modifierPriceDelta);
              if (priceDelta === null) { toast("价格调整已设为 0（元）", { icon: "⚠️" }); }
              onAddModifier({ order_item_id: modifierOrderItemId, modifier_type: modifierType, material_id: parseInt(modifierMaterialId), qty, price_delta: priceDelta ?? 0 });
              setModifierDialogOpen(false);
            }}>确认</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
