import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Plus, Pencil, Trash2, Save, Layers } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state";

interface MaterialState {
  id: number;
  material_id: number;
  state_code: string;
  state_name: string;
  unit_id: number | null;
  yield_rate: number;
  cost_multiplier: number;
  is_active: boolean;
}

interface Material {
  id: number;
  code: string;
  name: string;
}

interface Unit {
  id: number;
  code: string;
  name: string;
}

interface MaterialStatesPageProps {
  materialStates: MaterialState[];
  materials: Material[];
  units: Unit[];
  onCreateState: (data: { material_id: number; state_code: string; state_name: string; unit_id: number | null; yield_rate: number; cost_multiplier: number }) => void;
  onUpdateState: (id: number, data: { state_code?: string; state_name?: string; unit_id?: number | null; yield_rate?: number; cost_multiplier?: number }) => void;
  onDeleteState: (id: number) => void;
  searchQuery?: string;
}

export function MaterialStatesPage({
  materialStates, materials, units,
  onCreateState, onUpdateState, onDeleteState,
  searchQuery,
}: MaterialStatesPageProps) {
  const [newMaterialId, setNewMaterialId] = useState("");
  const [newStateCode, setNewStateCode] = useState("");
  const [newStateName, setNewStateName] = useState("");
  const [newUnitId, setNewUnitId] = useState("");
  const [newYieldRate, setNewYieldRate] = useState("1.0");
  const [newCostMultiplier, setNewCostMultiplier] = useState("1.0");

  const [editingState, setEditingState] = useState<MaterialState | null>(null);
  const [editStateCode, setEditStateCode] = useState("");
  const [editStateName, setEditStateName] = useState("");
  const [editUnitId, setEditUnitId] = useState("");
  const [editYieldRate, setEditYieldRate] = useState("1.0");
  const [editCostMultiplier, setEditCostMultiplier] = useState("1.0");

  const [deleteConfirm, setDeleteConfirm] = useState<MaterialState | null>(null);

  function openEdit(s: MaterialState) {
    setEditingState(s);
    setEditStateCode(s.state_code);
    setEditStateName(s.state_name);
    setEditUnitId(s.unit_id?.toString() || "");
    setEditYieldRate(s.yield_rate.toString());
    setEditCostMultiplier(s.cost_multiplier.toString());
  }

  function saveEdit() {
    if (!editingState) return;
    onUpdateState(editingState.id, {
      state_code: editStateCode || undefined,
      state_name: editStateName || undefined,
      unit_id: editUnitId ? parseInt(editUnitId) : null,
      yield_rate: parseFloat(editYieldRate) || 1.0,
      cost_multiplier: parseFloat(editCostMultiplier) || 1.0,
    });
    setEditingState(null);
  }

  function getMaterialName(materialId: number): string {
    return materials.find((m) => m.id === materialId)?.name || `材料 #${materialId}`;
  }

  function getUnitName(unitId: number | null): string {
    if (!unitId) return "-";
    return units.find((u) => u.id === unitId)?.name || `单位 #${unitId}`;
  }

  const filteredStates = materialStates.filter((s) => {
    if (!searchQuery) return true;
    const q = searchQuery.toLowerCase();
    return s.state_code.toLowerCase().includes(q) || s.state_name.toLowerCase().includes(q) ||
      getMaterialName(s.material_id).toLowerCase().includes(q);
  });

  const navigate = useNavigate();
  return (
    <div className="space-y-6">
      <div className="flex border-b border-border">
        <button className="-mb-px pb-2 px-4 text-sm font-medium border-b-2 border-transparent text-muted-foreground hover:text-foreground" onClick={() => navigate("/inventory")}>库存</button>
        <button className="-mb-px pb-2 px-4 text-sm font-medium border-b-2 border-primary text-primary">材料状态</button>
        <button className="-mb-px pb-2 px-4 text-sm font-medium border-b-2 border-transparent text-muted-foreground hover:text-foreground" onClick={() => navigate("/stocktakes")}>盘点</button>
      </div>
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">材料状态管理</h2>
          <p className="text-sm text-muted-foreground">管理材料的加工状态、出成率和成本系数</p>
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle className="flex items-center gap-2"><Layers className="h-4 w-4" />材料状态列表</CardTitle>
            <CardDescription>共 {filteredStates.length} 个状态{filteredStates.length !== materialStates.length ? `（筛选自 ${materialStates.length} 个）` : ""}</CardDescription>
          </CardHeader>
          <CardContent>
            {filteredStates.length === 0 ? (
              <EmptyState icon={Layers} title="暂无材料状态" description="添加材料状态管理加工状态" />
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>材料</TableHead>
                    <TableHead>代码</TableHead>
                    <TableHead>状态名称</TableHead>
                    <TableHead>单位</TableHead>
                    <TableHead className="text-right">出成率</TableHead>
                    <TableHead className="text-right">成本系数</TableHead>
                    <TableHead className="text-right">操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {filteredStates.map((s) => (
                    <TableRow key={s.id}>
                      <TableCell className="font-medium">{getMaterialName(s.material_id)}</TableCell>
                      <TableCell className="font-mono text-xs">{s.state_code}</TableCell>
                      <TableCell>{s.state_name}</TableCell>
                      <TableCell className="text-muted-foreground">{getUnitName(s.unit_id)}</TableCell>
                      <TableCell className="text-right">{(s.yield_rate * 100).toFixed(1)}%</TableCell>
                      <TableCell className="text-right">x{s.cost_multiplier.toFixed(2)}</TableCell>
                      <TableCell className="text-right">
                        <div className="flex justify-end gap-1">
                          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => openEdit(s)}><Pencil className="h-4 w-4" /></Button>
                          <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive" onClick={() => setDeleteConfirm(s)}><Trash2 className="h-4 w-4" /></Button>
                        </div>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader><CardTitle>新增材料状态</CardTitle></CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label>材料</Label>
              <Select value={newMaterialId} onValueChange={setNewMaterialId}>
                <SelectTrigger><SelectValue placeholder="选择原料" /></SelectTrigger>
                <SelectContent>
                  {materials.map((m) => <SelectItem key={m.id} value={m.id.toString()}>{m.name} ({m.code})</SelectItem>)}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>状态代码</Label>
              <Input
                value={newStateCode}
                onChange={(e) => setNewStateCode(e.target.value)}
                list="state-code-presets"
                placeholder="选择或输入代码"
              />
              <datalist id="state-code-presets">
                <option value="raw">raw — 生料</option>
                <option value="semi">semi — 半成品</option>
                <option value="processed">processed — 加工品</option>
                <option value="frozen">frozen — 冷凍</option>
                <option value="ready">ready — 備料完成</option>
              </datalist>
            </div>
            <div className="space-y-2">
              <Label>状态名称</Label>
              <Input value={newStateName} onChange={(e) => setNewStateName(e.target.value)} placeholder="如: 生料, 半成品" />
            </div>
            <div className="space-y-2">
              <Label>单位（可选）</Label>
              <Select value={newUnitId} onValueChange={setNewUnitId}>
                <SelectTrigger><SelectValue placeholder="选择单位" /></SelectTrigger>
                <SelectContent>
                  {units.map((u) => <SelectItem key={u.id} value={u.id.toString()}>{u.name} ({u.code})</SelectItem>)}
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>出成率</Label>
                <Input type="number" value={newYieldRate} onChange={(e) => setNewYieldRate(e.target.value)} placeholder="1.0" step="0.01" />
              </div>
              <div className="space-y-2">
                <Label>成本系数</Label>
                <Input type="number" value={newCostMultiplier} onChange={(e) => setNewCostMultiplier(e.target.value)} placeholder="1.0" step="0.01" />
              </div>
            </div>
            <Button className="w-full" onClick={() => {
              if (!newMaterialId || !newStateCode || !newStateName) return;
              onCreateState({
                material_id: parseInt(newMaterialId),
                state_code: newStateCode,
                state_name: newStateName,
                unit_id: newUnitId ? parseInt(newUnitId) : null,
                yield_rate: parseFloat(newYieldRate) || 1.0,
                cost_multiplier: parseFloat(newCostMultiplier) || 1.0,
              });
              setNewMaterialId(""); setNewStateCode(""); setNewStateName(""); setNewUnitId(""); setNewYieldRate("1.0"); setNewCostMultiplier("1.0");
            }} disabled={!newMaterialId || !newStateCode || !newStateName}>
              <Plus className="mr-2 h-4 w-4" />新增
            </Button>
          </CardContent>
        </Card>
      </div>

      <Dialog open={!!editingState} onOpenChange={() => setEditingState(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>编辑材料状态</DialogTitle></DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>状态代码</Label>
              <Input value={editStateCode} onChange={(e) => setEditStateCode(e.target.value)} list="state-code-presets" />
            </div>
            <div className="space-y-2">
              <Label>状态名称</Label>
              <Input value={editStateName} onChange={(e) => setEditStateName(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label>单位</Label>
              <Select value={editUnitId} onValueChange={setEditUnitId}>
                <SelectTrigger><SelectValue placeholder="选择单位" /></SelectTrigger>
                <SelectContent>
                  {units.map((u) => <SelectItem key={u.id} value={u.id.toString()}>{u.name} ({u.code})</SelectItem>)}
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>出成率</Label>
                <Input type="number" value={editYieldRate} onChange={(e) => setEditYieldRate(e.target.value)} step="0.01" />
              </div>
              <div className="space-y-2">
                <Label>成本系数</Label>
                <Input type="number" value={editCostMultiplier} onChange={(e) => setEditCostMultiplier(e.target.value)} step="0.01" />
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditingState(null)}>取消</Button>
            <Button onClick={saveEdit}><Save className="mr-1 h-4 w-4" />保存</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={!!deleteConfirm} onOpenChange={() => setDeleteConfirm(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>确认删除</DialogTitle></DialogHeader>
          <p className="py-4 text-sm text-muted-foreground">确定要删除材料状态「{deleteConfirm?.state_name}」吗？</p>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteConfirm(null)}>取消</Button>
            <Button variant="destructive" onClick={() => { if (deleteConfirm) { onDeleteState(deleteConfirm.id); } setDeleteConfirm(null); }}>删除</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
