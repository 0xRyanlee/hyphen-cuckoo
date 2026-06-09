import { useEffect, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { AlertTriangle, Plus, Package, ArrowUpDown, Trash2, ArrowRightLeft, Settings, Download } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state";
import { toast } from "sonner";
import { parseSafeFloat } from "@/lib/utils";
import { call as invoke } from "@/lib/transport";
import { useLocation, useNavigate } from "react-router-dom";

interface Material {
  id: number;
  name: string;
  code: string;
  min_qty?: number;
}
interface Recipe { id: number; code: string; name: string; recipe_type: string; output_qty: number; }
interface RecipeItem { id: number; recipe_id: number; item_type: string; ref_id: number; qty: number; unit_id: number; wastage_rate: number; note: string | null; sort_no: number; }
interface RecipeWithItems { recipe: Recipe; items: RecipeItem[]; }

interface Supplier {
  id: number;
  name: string;
}

interface InventorySummary {
  material_id: number;
  material_name: string;
  total_qty: number;
  reserved_qty: number;
  available_qty: number;
}

interface InventoryBatch {
  id: number;
  material_id: number;
  material_name?: string;
  lot_no: string;
  quantity: number;
  cost_per_unit: number;
  expiry_date: string | null;
  supplier_id: number | null;
}

interface InventoryTxn {
  id: number;
  txn_no: string;
  txn_type: string;
  ref_type: string | null;
  ref_id: number | null;
  lot_id: number | null;
  material_id: number;
  qty_delta: number;
  created_at: string;
}

interface InventoryPageProps {
  inventorySummary: InventorySummary[];
  inventoryBatches: InventoryBatch[];
  inventoryTxns: InventoryTxn[];
  materials: Material[];
  recipes: Recipe[];
  suppliers: Supplier[];
  onCreateBatch: (data: {
    material_id: number; lot_no: string; quantity: number; cost_per_unit: number;
    supplier_id: number | null; expiry_date: string | null; production_date: string | null;
    ice_coating_rate?: number; quality_rate?: number; seasonal_factor?: number;
  }) => void;
  onAdjustInventory: (lot_id: number, qty_delta: number, reason: string) => void;
  onRecordWastage: (lot_id: number, qty: number, wastage_type: string) => void;
  onDeleteBatch: (batch_id: number) => void;
  onUpdateMaterial: (id: number, data: { min_qty: number }) => void;
  searchQuery?: string;
}

export function InventoryPage({
  inventorySummary, inventoryBatches, inventoryTxns,
  materials, recipes, suppliers, onCreateBatch, onAdjustInventory, onRecordWastage,
  onUpdateMaterial, searchQuery,
}: InventoryPageProps) {
  function downloadCSV(filename: string, headers: string[], rows: (string | number)[][]) {
    const escape = (v: string | number) => `"${String(v).replace(/"/g, '""')}"`;
    const csv = [headers.map(escape), ...rows.map((r) => r.map(escape))].map((r) => r.join(",")).join("\n");
    const blob = new Blob(["﻿" + csv], { type: "text/csv;charset=utf-8;" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  }

  const location = useLocation();
  const navigate = useNavigate();
  const filteredSummary = inventorySummary.filter((s) => {
    if (!searchQuery) return true;
    const q = searchQuery.toLowerCase();
    return s.material_name.toLowerCase().includes(q);
  });
  const filteredBatches = inventoryBatches.filter((b) => {
    if (!searchQuery) return true;
    const q = searchQuery.toLowerCase();
    return b.lot_no.toLowerCase().includes(q) || (b.material_name || "").toLowerCase().includes(q);
  });
  const filteredTxns = inventoryTxns.filter((t) => {
    if (!searchQuery) return true;
    const q = searchQuery.toLowerCase();
    return t.txn_no.toLowerCase().includes(q) || t.txn_type.toLowerCase().includes(q);
  });
  const [batchDialogOpen, setBatchDialogOpen] = useState(false);
  const [adjustDialogOpen, setAdjustDialogOpen] = useState(false);
  const [wastageDialogOpen, setWastageDialogOpen] = useState(false);
  const [thresholdDialogOpen, setThresholdDialogOpen] = useState(false);
  const [usageDialogOpen, setUsageDialogOpen] = useState(false);
  const [usageBatch, setUsageBatch] = useState<InventoryBatch | null>(null);
  const [batchForm, setBatchForm] = useState({ material_id: "", lot_no: "", quantity: "", cost_per_unit: "", supplier_id: "", expiry_date: "", production_date: "", ice_coating_rate: "", quality_rate: "", seasonal_factor: "1.0" });
  const [adjustForm, setAdjustForm] = useState({ lot_id: 0, qty_delta: "", reason: "" });
  const [adjustDirection, setAdjustDirection] = useState<"add" | "sub">("add");
  const [wastageForm, setWastageForm] = useState({ lot_id: 0, qty: "", wastage_type: "normal" });
  const [thresholdForm, setThresholdForm] = useState({ material_id: 0, min_qty: "10" });
  const [materialRecipeMap, setMaterialRecipeMap] = useState<Record<number, Recipe[]>>({});

  const minQtyMap = Object.fromEntries(materials.map((m) => [m.id, m.min_qty ?? 10]));
  const getMinQty = (materialId: number) => minQtyMap[materialId] ?? 10;
  const lowStockItems = filteredSummary.filter((s) => s.available_qty < getMinQty(s.material_id));

  useEffect(() => {
    let cancelled = false;

    async function loadMaterialUsage() {
      try {
        const recipeDetails = await Promise.all(
          recipes.map((recipe) => invoke<RecipeWithItems>("get_recipe_with_items", { recipeId: recipe.id })),
        );
        const nextMap: Record<number, Recipe[]> = {};
        for (const detail of recipeDetails) {
          for (const item of detail.items) {
            if (item.item_type !== "material") continue;
            if (!nextMap[item.ref_id]) nextMap[item.ref_id] = [];
            nextMap[item.ref_id].push(detail.recipe);
          }
        }
        if (!cancelled) {
          setMaterialRecipeMap(nextMap);
        }
      } catch (e) {
        if (!cancelled) {
          console.error("加载批次影响配方失败", e);
        }
      }
    }

    if (recipes.length > 0) {
      loadMaterialUsage();
    } else {
      setMaterialRecipeMap({});
    }

    return () => {
      cancelled = true;
    };
  }, [recipes]);

  useEffect(() => {
    const routeState = location.state as { materialId?: number } | null;
    if (!routeState?.materialId) return;
    setBatchForm((prev) => ({ ...prev, material_id: String(routeState.materialId), lot_no: generateLotNo() }));
    setBatchDialogOpen(true);
    navigate(location.pathname, { replace: true, state: null });
  }, [location.pathname, location.state, navigate]);

  async function saveThreshold() {
    const qty = parseInt(thresholdForm.min_qty);
    if (isNaN(qty) || qty < 0) {
      toast.error("请输入有效的数字");
      return;
    }
    onUpdateMaterial(thresholdForm.material_id, { min_qty: qty });
    setThresholdDialogOpen(false);
  }

  const getTxnTypeBadge = (type: string) => {
    switch (type) {
      case "reserve": return <Badge variant="secondary">预扣</Badge>;
      case "consume": return <Badge variant="secondary">实扣</Badge>;
      case "release": return <Badge variant="outline">回补</Badge>;
      case "purchase_in": return <Badge>入库</Badge>;
      case "adjustment": return <Badge variant="outline">调整</Badge>;
      case "wastage": return <Badge variant="destructive">损耗</Badge>;
      default: return <Badge variant="outline">{type}</Badge>;
    }
  };

  const getMaterialName = (id: number) => materials.find((m) => m.id === id)?.name || `材料 #${id}`;

  function generateLotNo(): string {
    const d = new Date();
    const pad = (n: number) => n.toString().padStart(2, "0");
    const date = `${d.getFullYear()}${pad(d.getMonth() + 1)}${pad(d.getDate())}`;
    const seq = pad(d.getHours()) + pad(d.getMinutes());
    return `LOT-${date}-${seq}`;
  }

  function openBatchDialog() {
    setBatchForm({ material_id: "", lot_no: generateLotNo(), quantity: "", cost_per_unit: "", supplier_id: "", expiry_date: "", production_date: "", ice_coating_rate: "", quality_rate: "", seasonal_factor: "1.0" });
    setBatchDialogOpen(true);
  }

  function openBatchDialogForMaterial(materialId: number) {
    setBatchForm({ material_id: String(materialId), lot_no: generateLotNo(), quantity: "", cost_per_unit: "", supplier_id: "", expiry_date: "", production_date: "", ice_coating_rate: "", quality_rate: "", seasonal_factor: "1.0" });
    setBatchDialogOpen(true);
  }

  function openUsageDialog(batch: InventoryBatch) {
    setUsageBatch(batch);
    setUsageDialogOpen(true);
  }

  function goToRecipe(recipeId: number) {
    navigate("/recipes", { state: { recipeId } });
  }

  function handleCreateBatch() {
    if (!batchForm.material_id || !batchForm.lot_no || !batchForm.quantity) {
      toast.error("请填写必填字段");
      return;
    }
    const quantity = parseSafeFloat(batchForm.quantity);
    const costPerUnit = parseSafeFloat(batchForm.cost_per_unit);
    if (quantity === null || quantity <= 0) {
      toast.error("数量格式错误，请输入有效数字");
      return;
    }
    if (costPerUnit === null || costPerUnit < 0) {
      toast.error("单价格式错误，请输入有效数字");
      return;
    }
    onCreateBatch({
      material_id: parseInt(batchForm.material_id),
      lot_no: batchForm.lot_no,
      quantity,
      cost_per_unit: costPerUnit,
      supplier_id: batchForm.supplier_id ? parseInt(batchForm.supplier_id) : null,
      expiry_date: batchForm.expiry_date || null,
      production_date: batchForm.production_date || null,
      ice_coating_rate: batchForm.ice_coating_rate ? parseFloat(batchForm.ice_coating_rate) : undefined,
      quality_rate: batchForm.quality_rate ? parseFloat(batchForm.quality_rate) : undefined,
      seasonal_factor: batchForm.seasonal_factor ? parseFloat(batchForm.seasonal_factor) : 1.0,
    });
    setBatchForm({ material_id: "", lot_no: "", quantity: "", cost_per_unit: "", supplier_id: "", expiry_date: "", production_date: "", ice_coating_rate: "", quality_rate: "", seasonal_factor: "1.0" });
    setBatchDialogOpen(false);
  }

  const exportInventoryCSV = () => {
    downloadCSV(
      "库存汇总.csv",
      ["材料", "总库存", "预扣", "可用"],
      filteredSummary.map((item) => [
        item.material_name,
        item.total_qty.toFixed(4),
        item.reserved_qty.toFixed(4),
        item.available_qty.toFixed(4),
      ]),
    );

    downloadCSV(
      "库存批次.csv",
      ["批次号", "材料", "数量", "单成本", "到期日", "供应商"],
      filteredBatches.map((batch) => [
        batch.lot_no,
        batch.material_name || "",
        batch.quantity.toFixed(4),
        batch.cost_per_unit.toFixed(2),
        batch.expiry_date || "",
        suppliers.find((supplier) => supplier.id === batch.supplier_id)?.name || "",
      ]),
    );

    downloadCSV(
      "库存流水.csv",
      ["流水号", "类型", "材料", "批次 ID", "数量变化", "关联类型", "关联 ID", "创建时间"],
      filteredTxns.map((txn) => [
        txn.txn_no,
        txn.txn_type,
        getMaterialName(txn.material_id),
        txn.lot_id || "",
        txn.qty_delta.toFixed(4),
        txn.ref_type || "",
        txn.ref_id || "",
        txn.created_at,
      ]),
    );
  };

function handleAdjust() {
    if (!adjustForm.lot_id || !adjustForm.qty_delta || !adjustForm.reason) {
      toast.error("请填写所有字段");
      return;
    }
    const qtyDelta = parseSafeFloat(adjustForm.qty_delta);
    if (qtyDelta === null || qtyDelta <= 0) {
      toast.error("调整数量必须大于 0");
      return;
    }
    const signedDelta = adjustDirection === "sub" ? -qtyDelta : qtyDelta;
    onAdjustInventory(adjustForm.lot_id, signedDelta, adjustForm.reason);
    setAdjustForm({ lot_id: 0, qty_delta: "", reason: "" });
    setAdjustDirection("add");
    setAdjustDialogOpen(false);
  }

  function handleWastage() {
    if (!wastageForm.lot_id || !wastageForm.qty) {
      toast.error("请填写所有字段");
      return;
    }
    const qty = parseSafeFloat(wastageForm.qty);
    if (qty === null || qty <= 0) {
      toast.error("损耗数量格式错误");
      return;
    }
    onRecordWastage(wastageForm.lot_id, qty, wastageForm.wastage_type);
    setWastageForm({ lot_id: 0, qty: "", wastage_type: "normal" });
    setWastageDialogOpen(false);
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">库存管理</h2>
          <p className="text-sm text-muted-foreground">入库、调整、损耗与库存追踪</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={exportInventoryCSV} disabled={filteredSummary.length === 0 && filteredBatches.length === 0 && filteredTxns.length === 0}>
            <Download className="mr-2 h-4 w-4" />导出 CSV
          </Button>
          <Button onClick={openBatchDialog}>
            <Plus className="mr-2 h-4 w-4" />进货入库
          </Button>
        </div>
      </div>
      <div className="flex border-b border-border">
        <button className="-mb-px pb-2 px-4 text-sm font-medium border-b-2 border-primary text-primary">库存</button>
        <button className="-mb-px pb-2 px-4 text-sm font-medium border-b-2 border-transparent text-muted-foreground hover:text-foreground" onClick={() => navigate("/material-states")}>材料状态</button>
        <button className="-mb-px pb-2 px-4 text-sm font-medium border-b-2 border-transparent text-muted-foreground hover:text-foreground" onClick={() => navigate("/stocktakes")}>盘点</button>
      </div>

{lowStockItems.length > 0 && (
        <Card className="border-destructive/50">
          <CardHeader>
            <CardTitle className="flex items-center justify-between text-destructive">
              <span className="flex items-center gap-2">
                <AlertTriangle className="h-4 w-4" />库存预警 ({lowStockItems.length})
              </span>
              <Button variant="outline" size="sm" onClick={() => setThresholdDialogOpen(true)}>
                <Settings className="h-4 w-4 mr-2" />配置閾值
              </Button>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>材料</TableHead>
                  <TableHead className="text-right">总量</TableHead>
                  <TableHead className="text-right">预扣</TableHead>
                  <TableHead className="text-right">可用</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {lowStockItems.map((summary) => (
                  <TableRow key={summary.material_id}>
                    <TableCell className="font-medium">{summary.material_name}</TableCell>
                    <TableCell className="text-right">{summary.total_qty.toFixed(2)}</TableCell>
                    <TableCell className="text-right text-muted-foreground">{summary.reserved_qty.toFixed(2)}</TableCell>
                    <TableCell className="text-right text-destructive font-medium">{summary.available_qty.toFixed(2)}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      )}

      <div className="grid gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2"><Package className="h-4 w-4" />库存汇总</CardTitle>
            <CardDescription>{filteredSummary.length} 种材料</CardDescription>
          </CardHeader>
          <CardContent>
            {inventorySummary.length === 0 ? (
              <EmptyState icon={Package} title="暂无库存数据" description="进货入库后将显示库存" />
            ) : (
              <ScrollArea className="h-64">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>材料</TableHead>
                      <TableHead className="text-right">总量</TableHead>
                      <TableHead className="text-right">可用</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {filteredSummary.map((summary) => (
                      <TableRow key={summary.material_id}>
                        <TableCell className="font-medium">{summary.material_name}</TableCell>
                        <TableCell className="text-right">{summary.total_qty.toFixed(2)}</TableCell>
                        <TableCell className={`text-right font-medium ${summary.available_qty < getMinQty(summary.material_id) ? "text-destructive" : "text-primary"}`}>
                          {summary.available_qty.toFixed(2)}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </ScrollArea>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>批次库存</CardTitle>
            <CardDescription>{filteredBatches.length} 个批次{filteredBatches.length !== inventoryBatches.length ? `（筛选自 ${inventoryBatches.length} 个）` : ""}</CardDescription>
          </CardHeader>
          <CardContent>
            {filteredBatches.length === 0 ? (
              <EmptyState icon={Package} title="暂无批次数据" description="入库批次将在此显示" />
            ) : (
              <ScrollArea className="h-64">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>批次号</TableHead>
                      <TableHead className="text-right">数量</TableHead>
                      <TableHead>效期</TableHead>
                      <TableHead>影响配方</TableHead>
                      <TableHead className="text-right">操作</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {filteredBatches.map((batch) => {
                      const now = new Date(); now.setHours(0,0,0,0);
                      const expiryDate = batch.expiry_date ? new Date(batch.expiry_date) : null;
                      const isExpired = expiryDate && expiryDate < now;
                      const daysLeft = expiryDate ? Math.ceil((expiryDate.getTime() - now.getTime()) / 86400000) : null;
                      const isExpiringSoon = !isExpired && daysLeft !== null && daysLeft <= 7;
                      return (
                      <TableRow key={batch.id} className={isExpired ? "bg-destructive/5" : isExpiringSoon ? "bg-amber-500/5" : ""}>
                        <TableCell>
                          <div className="font-mono text-xs">{batch.lot_no}</div>
                          <div className="text-xs text-muted-foreground">{getMaterialName(batch.material_id)}</div>
                        </TableCell>
                        <TableCell className="text-right">{batch.quantity.toFixed(2)}</TableCell>
                        <TableCell>
                          {batch.expiry_date ? (
                            <span className={`text-xs font-mono ${isExpired ? "text-destructive font-medium" : isExpiringSoon ? "text-amber-500 font-medium" : "text-muted-foreground"}`}>
                              {batch.expiry_date}
                              {isExpired ? (
                                <span className="ml-1 text-destructive font-medium">已過期</span>
                              ) : isExpiringSoon ? (
                                <span className="ml-1 text-amber-500 font-medium">{daysLeft}天後到期</span>
                              ) : null}
                            </span>
                          ) : <span className="text-xs text-muted-foreground">—</span>}
                        </TableCell>
                        <TableCell>
                          {(materialRecipeMap[batch.material_id]?.length || 0) > 0 ? (
                            <Button variant="outline" size="sm" onClick={() => openUsageDialog(batch)}>
                              查看 {materialRecipeMap[batch.material_id].length} 个配方
                            </Button>
                          ) : (
                            <span className="text-xs text-muted-foreground">未影响配方</span>
                          )}
                        </TableCell>
                        <TableCell className="text-right">
                          <div className="flex justify-end gap-1">
                            <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => openBatchDialogForMaterial(batch.material_id)} title="为同一材料补批次">
                              <Plus className="h-3 w-3" />
                            </Button>
                            <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => { setAdjustForm({ lot_id: batch.id, qty_delta: "", reason: "" }); setAdjustDirection("add"); setAdjustDialogOpen(true); }}>
                              <ArrowUpDown className="h-3 w-3" />
                            </Button>
                            <Button variant="ghost" size="icon" className="h-7 w-7 text-destructive" onClick={() => { setWastageForm({ lot_id: batch.id, qty: "", wastage_type: "normal" }); setWastageDialogOpen(true); }}>
                              <Trash2 className="h-3 w-3" />
                            </Button>
                          </div>
                        </TableCell>
                      </TableRow>
                      );
                    })}
                  </TableBody>
                </Table>
              </ScrollArea>
            )}
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>库存交易流水</CardTitle>
          <CardDescription>最近 {filteredTxns.length} 条记录{filteredTxns.length !== inventoryTxns.length ? `（筛选自 ${inventoryTxns.length} 条）` : ""}</CardDescription>
        </CardHeader>
        <CardContent>
          {filteredTxns.length === 0 ? (
            <EmptyState icon={ArrowRightLeft} title="暂无交易记录" description="库存变动将在此显示" />
          ) : (
            <ScrollArea className="h-72">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>交易号</TableHead>
                    <TableHead>类型</TableHead>
                    <TableHead>材料</TableHead>
                    <TableHead className="text-right">数量变化</TableHead>
                    <TableHead>时间</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {filteredTxns.map((txn) => (
                    <TableRow key={txn.id}>
                      <TableCell className="font-mono text-xs">{txn.txn_no}</TableCell>
                      <TableCell>{getTxnTypeBadge(txn.txn_type)}</TableCell>
                      <TableCell className="text-xs">{getMaterialName(txn.material_id)}</TableCell>
                      <TableCell className={`text-right font-medium ${txn.qty_delta < 0 ? "text-destructive" : "text-primary"}`}>
                        {txn.qty_delta > 0 ? "+" : ""}{txn.qty_delta.toFixed(2)}
                      </TableCell>
                      <TableCell className="text-muted-foreground text-xs">{txn.created_at}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </ScrollArea>
          )}
        </CardContent>
      </Card>

      <Dialog open={batchDialogOpen} onOpenChange={setBatchDialogOpen}>
        <DialogContent>
          <DialogHeader><DialogTitle>进货入库</DialogTitle></DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="batch-material">材料</Label>
              <Select value={batchForm.material_id} onValueChange={(v) => setBatchForm({ ...batchForm, material_id: v })}>
                <SelectTrigger><SelectValue placeholder="选择材料" /></SelectTrigger>
                <SelectContent>
                  {materials.map((m) => <SelectItem key={m.id} value={m.id.toString()}>{m.name} ({m.code})</SelectItem>)}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="batch-lot">批次号</Label>
              <Input id="batch-lot" value={batchForm.lot_no} onChange={(e) => setBatchForm({ ...batchForm, lot_no: e.target.value })} placeholder="如：LOT-20260423-001" />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="batch-qty">数量</Label>
                <Input id="batch-qty" type="number" value={batchForm.quantity} onChange={(e) => setBatchForm({ ...batchForm, quantity: e.target.value })} placeholder="0" />
              </div>
              <div className="space-y-2">
                <Label htmlFor="batch-cost">单位成本</Label>
                <Input id="batch-cost" type="number" value={batchForm.cost_per_unit} onChange={(e) => setBatchForm({ ...batchForm, cost_per_unit: e.target.value })} placeholder="0.00" />
              </div>
            </div>
            <div className="space-y-2">
              <Label htmlFor="batch-supplier">供应商</Label>
              <Select value={batchForm.supplier_id} onValueChange={(v) => setBatchForm({ ...batchForm, supplier_id: v })}>
                <SelectTrigger><SelectValue placeholder="选择供应商（可选）" /></SelectTrigger>
                <SelectContent>
                  {suppliers.map((s) => <SelectItem key={s.id} value={s.id.toString()}>{s.name}</SelectItem>)}
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="batch-production">生产日期</Label>
                <Input id="batch-production" type="date" value={batchForm.production_date} onChange={(e) => setBatchForm({ ...batchForm, production_date: e.target.value })} />
              </div>
              <div className="space-y-2">
                <Label htmlFor="batch-expiry">过期日期</Label>
                <Input id="batch-expiry" type="date" value={batchForm.expiry_date} onChange={(e) => setBatchForm({ ...batchForm, expiry_date: e.target.value })} />
              </div>
            </div>
            
            <div className="border-t pt-4 mt-2">
              <Label className="text-sm font-medium mb-2 block">属性字段（可选）</Label>
              <div className="grid grid-cols-3 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="batch-ice" className="text-xs text-muted-foreground">冰衣率 (%)</Label>
                  <Input id="batch-ice" type="number" value={batchForm.ice_coating_rate} onChange={(e) => setBatchForm({ ...batchForm, ice_coating_rate: e.target.value })} placeholder="0" />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="batch-quality" className="text-xs text-muted-foreground">品质等级 (%)</Label>
                  <Input id="batch-quality" type="number" value={batchForm.quality_rate} onChange={(e) => setBatchForm({ ...batchForm, quality_rate: e.target.value })} placeholder="100" />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="batch-seasonal" className="text-xs text-muted-foreground">季节系数</Label>
                  <Input id="batch-seasonal" type="number" step="0.1" value={batchForm.seasonal_factor} onChange={(e) => setBatchForm({ ...batchForm, seasonal_factor: e.target.value })} placeholder="1.0" />
                </div>
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setBatchDialogOpen(false)}>取消</Button>
            <Button onClick={handleCreateBatch}>确认入库</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={adjustDialogOpen} onOpenChange={setAdjustDialogOpen}>
        <DialogContent>
          <DialogHeader><DialogTitle>库存调整</DialogTitle></DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>调整数量</Label>
              <div className="flex gap-2">
                <div className="flex rounded-md border">
                  <Button variant={adjustDirection === "add" ? "default" : "ghost"} size="sm" onClick={() => setAdjustDirection("add")} className="rounded-r-none">增加</Button>
                  <Button variant={adjustDirection === "sub" ? "default" : "ghost"} size="sm" onClick={() => setAdjustDirection("sub")} className="rounded-l-none">减少</Button>
                </div>
                <Input id="adjust-delta" type="number" min="0" step="0.01" value={adjustForm.qty_delta} onChange={(e) => setAdjustForm({ ...adjustForm, qty_delta: e.target.value })} placeholder="输入数量" className="flex-1" />
              </div>
            </div>
            <div className="space-y-2">
              <Label htmlFor="adjust-reason">调整原因</Label>
              <Input id="adjust-reason" value={adjustForm.reason} onChange={(e) => setAdjustForm({ ...adjustForm, reason: e.target.value })} placeholder="如：盘点差异、报损" />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setAdjustDialogOpen(false)}>取消</Button>
            <Button onClick={handleAdjust}>确认调整</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={wastageDialogOpen} onOpenChange={setWastageDialogOpen}>
        <DialogContent>
          <DialogHeader><DialogTitle>记录损耗</DialogTitle></DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="wastage-qty">损耗数量</Label>
              <Input id="wastage-qty" type="number" min="0" step="0.01" value={wastageForm.qty} onChange={(e) => setWastageForm({ ...wastageForm, qty: e.target.value })} placeholder="0" />
            </div>
            <div className="space-y-2">
              <Label htmlFor="wastage-type">损耗类型</Label>
              <Select value={wastageForm.wastage_type} onValueChange={(v) => setWastageForm({ ...wastageForm, wastage_type: v })}>
                <SelectTrigger><SelectValue placeholder="选择损耗类型" /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="normal">正常损耗</SelectItem>
                  <SelectItem value="rd">研发损耗</SelectItem>
                  <SelectItem value="fail">失败损耗</SelectItem>
                  <SelectItem value="seasonal">季节性损耗</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
<DialogFooter>
            <Button variant="outline" onClick={() => setWastageDialogOpen(false)}>取消</Button>
            <Button variant="destructive" onClick={handleWastage}>确认损耗</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={thresholdDialogOpen} onOpenChange={setThresholdDialogOpen}>
        <DialogContent>
          <DialogHeader><DialogTitle>设置低库存阈值</DialogTitle></DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="threshold-material">材料</Label>
              <Select value={String(thresholdForm.material_id)} onValueChange={(v) => {
                const m = materials.find(m => m.id === Number(v));
                setThresholdForm({ material_id: Number(v), min_qty: m?.min_qty?.toString() || "10" });
              }}>
                <SelectTrigger><SelectValue placeholder="选择材料" /></SelectTrigger>
                <SelectContent>
                  {materials.map(m => (
                    <SelectItem key={m.id} value={String(m.id)}>{m.name}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="threshold-value">閾值數量</Label>
              <Input id="threshold-value" type="number" value={thresholdForm.min_qty} onChange={(e) => setThresholdForm({ ...thresholdForm, min_qty: e.target.value })} placeholder="10" />
              <p className="text-xs text-muted-foreground">当库存低于此数量时显示预警</p>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setThresholdDialogOpen(false)}>取消</Button>
            <Button onClick={saveThreshold}>保存</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={usageDialogOpen} onOpenChange={setUsageDialogOpen}>
        <DialogContent>
          <DialogHeader><DialogTitle>批次影响配方 - {usageBatch?.lot_no}</DialogTitle></DialogHeader>
          <div className="space-y-3 py-4">
            {(usageBatch && materialRecipeMap[usageBatch.material_id]?.length) ? (
              materialRecipeMap[usageBatch.material_id].map((recipe) => (
                <div key={recipe.id} className="flex items-center justify-between rounded-lg border p-3">
                  <div>
                    <p className="font-medium">{recipe.name}</p>
                    <p className="text-xs text-muted-foreground">{recipe.code}</p>
                  </div>
                  <Button variant="outline" size="sm" onClick={() => goToRecipe(recipe.id)}>
                    查看配方
                  </Button>
                </div>
              ))
            ) : (
              <EmptyState icon={Package} title="暂无影响配方" description="这个材料当前没有被任何配方使用" />
            )}
          </div>
          <DialogFooter>
            <Button onClick={() => setUsageDialogOpen(false)}>关闭</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
