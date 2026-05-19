import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Plus, ClipboardList, Eye, Trash2, CheckCircle, AlertTriangle, Package } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state";

interface Stocktake {
  id: number;
  stocktake_no: string;
  status: string;
  operator: string | null;
  note: string | null;
  created_at: string;
  completed_at: string | null;
}

interface StocktakeItem {
  id: number;
  stocktake_id: number;
  lot_id: number | null;
  material_id: number;
  material_name: string | null;
  system_qty: number;
  actual_qty: number;
  diff_qty: number | null;
  note: string | null;
}

interface StocktakeWithItems {
  stocktake: Stocktake;
  items: StocktakeItem[];
}

interface StocktakesPageProps {
  stocktakes: Stocktake[];
  onCreateStocktake: (data: { operator: string | null; note: string | null }) => void;
  onUpdateItem: (item_id: number, actual_qty: number) => void;
  onCompleteStocktake: (stocktake_id: number) => void;
  onViewStocktake: (stocktake_id: number) => void;
  onDeleteStocktake: (stocktake_id: number) => void;
  selectedStocktake: StocktakeWithItems | null;
  searchQuery?: string;
}

export function StocktakesPage({
  stocktakes,
  onCreateStocktake, onUpdateItem, onCompleteStocktake,
  onViewStocktake, onDeleteStocktake,
  selectedStocktake,
  searchQuery,
}: StocktakesPageProps) {
  const filteredStocktakes = stocktakes.filter((st) => {
    if (!searchQuery) return true;
    const q = searchQuery.toLowerCase();
    return st.stocktake_no.toLowerCase().includes(q) || (st.operator || "").toLowerCase().includes(q) || st.status.toLowerCase().includes(q);
  });
  const [newOperator, setNewOperator] = useState("");
  const [newNote, setNewNote] = useState("");

  const [editingItemId, setEditingItemId] = useState<number | null>(null);
  const [editingActualQty, setEditingActualQty] = useState("");

  const getStatusBadge = (status: string) => {
    switch (status) {
      case "draft": return <Badge variant="outline">草稿</Badge>;
      case "completed": return <Badge className="bg-emerald-600">已完成</Badge>;
      case "cancelled": return <Badge variant="destructive">已取消</Badge>;
      default: return <Badge variant="secondary">{status}</Badge>;
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">库存盘点</h2>
          <p className="text-sm text-muted-foreground">管理库存盘点、差异调整</p>
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle className="flex items-center gap-2"><ClipboardList className="h-4 w-4" />盘点单列表</CardTitle>
            <CardDescription>共 {filteredStocktakes.length} 张盘点单{filteredStocktakes.length !== stocktakes.length ? `（筛选自 ${stocktakes.length} 张）` : ""}</CardDescription>
          </CardHeader>
          <CardContent>
            {filteredStocktakes.length === 0 ? (
              <EmptyState icon={ClipboardList} title="暂无盘点单" description="新增盘点单开始盘点" />
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>单号</TableHead>
                    <TableHead>状态</TableHead>
                    <TableHead>操作人</TableHead>
                    <TableHead>创建时间</TableHead>
                    <TableHead className="text-right">操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {filteredStocktakes.map((st) => (
                    <TableRow key={st.id}>
                      <TableCell className="font-mono text-xs">{st.stocktake_no}</TableCell>
                      <TableCell>{getStatusBadge(st.status)}</TableCell>
                      <TableCell className="text-muted-foreground">{st.operator || "-"}</TableCell>
                      <TableCell className="text-xs text-muted-foreground">{st.created_at}</TableCell>
                      <TableCell className="text-right">
                        <div className="flex justify-end gap-1">
                          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => onViewStocktake(st.id)}><Eye className="h-4 w-4" /></Button>
                          {st.status === "draft" && (
                            <Button variant="ghost" size="icon" className="h-8 w-8 text-emerald-500" onClick={() => onCompleteStocktake(st.id)}><CheckCircle className="h-4 w-4" /></Button>
                          )}
                          {st.status === "draft" && (
                            <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive" onClick={() => onDeleteStocktake(st.id)}><Trash2 className="h-4 w-4" /></Button>
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
            <CardHeader><CardTitle>新增盘点</CardTitle></CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label>操作人（可选）</Label>
                <Input value={newOperator} onChange={(e) => setNewOperator(e.target.value)} placeholder="操作人姓名" />
              </div>
              <div className="space-y-2">
                <Label>备注（可选）</Label>
                <Input value={newNote} onChange={(e) => setNewNote(e.target.value)} placeholder="盘点备注" />
              </div>
              <Button className="w-full" onClick={() => {
                onCreateStocktake({
                  operator: newOperator || null,
                  note: newNote || null,
                });
                setNewOperator(""); setNewNote("");
              }}>
                <Plus className="mr-2 h-4 w-4" />新增盘点单
              </Button>
              <p className="text-xs text-muted-foreground">自动带入所有有库存的批次，实际数量预设等于系统数量</p>
            </CardContent>
          </Card>

          {selectedStocktake && (
            <Card>
              <CardHeader>
                <CardTitle>盘点详情</CardTitle>
                <CardDescription>{selectedStocktake.stocktake.stocktake_no}</CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="space-y-2 text-sm">
                  <div className="flex justify-between"><span className="text-muted-foreground">状态</span>{getStatusBadge(selectedStocktake.stocktake.status)}</div>
                  <div className="flex justify-between"><span className="text-muted-foreground">操作人</span><span>{selectedStocktake.stocktake.operator || "-"}</span></div>
                  <div className="flex justify-between"><span className="text-muted-foreground">備註</span><span>{selectedStocktake.stocktake.note || "-"}</span></div>
                </div>
                <div>
                  <h4 className="text-sm font-medium mb-2 flex items-center gap-1"><AlertTriangle className="h-3 w-3" />盘点明细</h4>
                  {selectedStocktake.items.length === 0 ? (
                    <EmptyState icon={Package} title="暂无材料" description="盘点单中没有材料" />
                  ) : (
                    <div className="space-y-2">
                      {selectedStocktake.items.map((item) => (
                        <div key={item.id} className="p-2 bg-muted rounded space-y-1">
                          <div className="flex justify-between text-sm">
                            <span className="font-medium">{item.material_name || `材料 #${item.material_id}`}</span>
                            {item.diff_qty !== null && Math.abs(item.diff_qty) > 0.001 && (
                              <span className="text-xs text-amber-500">差異: {item.diff_qty > 0 ? "+" : ""}{item.diff_qty.toFixed(2)}</span>
                            )}
                          </div>
                          <div className="flex items-center gap-2 text-xs">
                            <span className="text-muted-foreground">系統: {item.system_qty}</span>
                            {editingItemId === item.id ? (
                              <>
                                <Input
                                  className="h-6 w-20 text-xs"
                                  type="number"
                                  value={editingActualQty}
                                  onChange={(e) => setEditingActualQty(e.target.value)}
                                  autoFocus
                                />
                                <Button size="sm" className="h-6 px-2 text-xs" onClick={() => {
                                  const qty = parseFloat(editingActualQty);
                                  if (editingActualQty === "" || isNaN(qty) || qty < 0) return;
                                  onUpdateItem(item.id, qty);
                                  setEditingItemId(null);
                                }}>确认</Button>
                              </>
                            ) : (
                              <Button variant="ghost" size="sm" className="h-6 px-2 text-xs" onClick={() => {
                                setEditingItemId(item.id);
                                setEditingActualQty(item.actual_qty.toString());
                              }}>实际: {item.actual_qty} ✏️</Button>
                            )}
                          </div>
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
    </div>
  );
}
