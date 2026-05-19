import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Plus, FileText, ToggleRight, ToggleLeft, Link, Pencil, Trash2, Edit2, Tag, Save, X, EyeOff, Eye } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state";
import { Checkbox } from "@/components/ui/checkbox";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { toast } from "sonner";
import { parseSafeFloat } from "@/lib/utils";
import { useNavigate } from "react-router-dom";

interface MenuCategory { id: number; name: string; }
interface MenuItem { id: number; name: string; sales_price: number; is_available: boolean; recipe_id: number | null; category_id: number | null; }
interface Recipe { id: number; name: string; code: string; }
interface MenuItemSpec { id: number; menu_item_id: number; spec_code: string; spec_name: string; price_delta: number; qty_multiplier: number; }

interface MenuPageProps {
  menuCategories: MenuCategory[]; menuItems: MenuItem[]; recipes: Recipe[];
  onCreateMenuCategory: (name: string) => void;
  onCreateMenuItem: (data: { name: string; price: number; category_id: number | null; recipe_id: number | null }) => void;
  onCreatePendingRecipeForMenu: (menuItemId: number, menuItemName: string) => Promise<number | null>;
  onToggleAvailability: (id: number, is_available: boolean) => void;
  onBatchToggleAvailability?: (ids: number[], is_available: boolean) => void;
  onUpdateMenuItem: (id: number, data: { name?: string; category_id?: number | null; recipe_id?: number | null; sales_price?: number }) => void;
  onDeleteMenuItem: (id: number) => void;
  onUpdateMenuCategory: (id: number, name: string) => void;
  onDeleteMenuCategory: (id: number) => void;
  onGetSpecs: (menuItemId: number) => Promise<MenuItemSpec[]>;
  onCreateSpec: (data: { menu_item_id: number; spec_code: string; spec_name: string; price_delta: number; qty_multiplier: number }) => void;
  onUpdateSpec: (id: number, data: { spec_code?: string; spec_name?: string; price_delta?: number; qty_multiplier?: number }) => void;
  onDeleteSpec: (id: number) => void;
  searchQuery?: string;
}

export function MenuPage({
  menuCategories, menuItems, recipes,
  onCreateMenuCategory, onCreateMenuItem, onCreatePendingRecipeForMenu, onToggleAvailability,
  onBatchToggleAvailability,
  onUpdateMenuItem, onDeleteMenuItem, onUpdateMenuCategory, onDeleteMenuCategory,
  onGetSpecs, onCreateSpec, onUpdateSpec, onDeleteSpec,
  searchQuery,
}: MenuPageProps) {
  const navigate = useNavigate();
  const [showUnavailable, setShowUnavailable] = useState(false);
  const filteredMenuItems = menuItems.filter((item) => {
    if (!showUnavailable && !item.is_available) return false;
    if (!searchQuery) return true;
    const q = searchQuery.toLowerCase();
    return item.name.toLowerCase().includes(q) || menuCategories.find((c) => c.id === item.category_id)?.name.toLowerCase().includes(q);
  });
  const [newMenuCategoryName, setNewMenuCategoryName] = useState("");
  const [newMenuItemName, setNewMenuItemName] = useState("");
  const [newMenuItemPrice, setNewMenuItemPrice] = useState("");
  const [newMenuItemCategory, setNewMenuItemCategory] = useState("");
  const [newMenuItemRecipe, setNewMenuItemRecipe] = useState("");
  const [priceError, setPriceError] = useState("");
  const [selectedItems, setSelectedItems] = useState<number[]>([]);

  const toggleSelect = (id: number) => {
    setSelectedItems(prev => prev.includes(id) ? prev.filter(i => i !== id) : [...prev, id]);
  };
  const selectAll = () => {
    if (selectedItems.length === filteredMenuItems.length) {
      setSelectedItems([]);
    } else {
      setSelectedItems(filteredMenuItems.map(i => i.id));
    }
  };
  const handleBatchEnable = () => {
    if (onBatchToggleAvailability && selectedItems.length > 0) {
      onBatchToggleAvailability(selectedItems, true);
      setSelectedItems([]);
    }
  };
  const handleBatchDisable = () => {
    if (onBatchToggleAvailability && selectedItems.length > 0) {
      onBatchToggleAvailability(selectedItems, false);
      setSelectedItems([]);
    }
  };

  const handleCreateMenuItem = () => {
    const price = parseFloat(newMenuItemPrice);
    if (!newMenuItemName.trim()) return;
    if (isNaN(price) || price <= 0) {
      setPriceError("售价必须大于 0");
      return;
    }
    setPriceError("");
    onCreateMenuItem({ name: newMenuItemName, price, category_id: newMenuItemCategory ? parseInt(newMenuItemCategory) : null, recipe_id: newMenuItemRecipe ? parseInt(newMenuItemRecipe) : null });
    setNewMenuItemName("");
    setNewMenuItemPrice("");
    setNewMenuItemCategory("");
    setNewMenuItemRecipe("");
  };

  const [editMenuItem, setEditMenuItem] = useState<MenuItem | null>(null);
  const [editMenuItemName, setEditMenuItemName] = useState("");
  const [editMenuItemPrice, setEditMenuItemPrice] = useState("");
  const [editMenuItemCategory, setEditMenuItemCategory] = useState("");
  const [editMenuItemRecipe, setEditMenuItemRecipe] = useState("");

  const [editCategoryId, setEditCategoryId] = useState<number | null>(null);
  const [editCategoryName, setEditCategoryName] = useState("");

  const [deleteConfirmTarget, setDeleteConfirmTarget] = useState<{ type: 'item' | 'category'; id: number; name: string } | null>(null);

  const [specDialogOpen, setSpecDialogOpen] = useState(false);
  const [specMenuItem, setSpecMenuItem] = useState<MenuItem | null>(null);
  const [specList, setSpecList] = useState<MenuItemSpec[]>([]);
  const [editingSpec, setEditingSpec] = useState<MenuItemSpec | null>(null);
  const [newSpecCode, setNewSpecCode] = useState("");
  const [newSpecName, setNewSpecName] = useState("");
  const [newSpecPriceDelta, setNewSpecPriceDelta] = useState("0");
  const [newSpecQtyMultiplier, setNewSpecQtyMultiplier] = useState("1");

  function openEditItem(item: MenuItem) {
    setEditMenuItem(item); setEditMenuItemName(item.name); setEditMenuItemPrice(item.sales_price.toString());
    setEditMenuItemCategory(item.category_id?.toString() || ""); setEditMenuItemRecipe(item.recipe_id?.toString() || "");
  }

  function saveEditItem() {
    if (!editMenuItem) return;
    if (editMenuItemName !== undefined && !editMenuItemName.trim()) {
      toast.error("名称不能为空");
      return;
    }
    const price = parseSafeFloat(editMenuItemPrice);
    if (price !== null && price < 0) {
      toast.error("单价不能为负数");
      return;
    }
    onUpdateMenuItem(editMenuItem.id, { name: editMenuItemName || undefined, category_id: editMenuItemCategory ? parseInt(editMenuItemCategory) : null, recipe_id: editMenuItemRecipe ? parseInt(editMenuItemRecipe) : null, sales_price: price ?? 0 });
    setEditMenuItem(null);
  }

  function openEditCategory(cat: MenuCategory) { setEditCategoryId(cat.id); setEditCategoryName(cat.name); }
  function saveEditCategory() { if (!editCategoryId) return; onUpdateMenuCategory(editCategoryId, editCategoryName); setEditCategoryId(null); setEditCategoryName(""); }
  function confirmDelete(type: 'item' | 'category', id: number, name: string) { setDeleteConfirmTarget({ type, id, name }); }
  function executeDelete() {
    if (!deleteConfirmTarget) return;
    if (deleteConfirmTarget.type === 'item') onDeleteMenuItem(deleteConfirmTarget.id);
    else onDeleteMenuCategory(deleteConfirmTarget.id);
    setDeleteConfirmTarget(null);
  }

  async function openSpecDialog(item: MenuItem) {
    setSpecMenuItem(item);
    try { const specs = await onGetSpecs(item.id); setSpecList(specs); } catch { setSpecList([]); }
    setSpecDialogOpen(true);
  }

  function handleCreateSpec() {
    if (!specMenuItem) return;
    if (!newSpecCode.trim() || !newSpecName.trim()) {
      toast.error("请填写规格代码和名称");
      return;
    }
    const qtyMultiplier = parseSafeFloat(newSpecQtyMultiplier);
    if (qtyMultiplier === null || qtyMultiplier <= 0) {
      toast.error("數量倍數必須大於 0");
      return;
    }
    const priceDelta = parseSafeFloat(newSpecPriceDelta);
    if (priceDelta === null) {
      toast("价格调整无效，已设为 0", { icon: "⚠️" });
    }
    onCreateSpec({ menu_item_id: specMenuItem.id, spec_code: newSpecCode.trim(), spec_name: newSpecName.trim(), price_delta: priceDelta ?? 0, qty_multiplier: qtyMultiplier });
    setNewSpecCode(""); setNewSpecName(""); setNewSpecPriceDelta("0"); setNewSpecQtyMultiplier("1");
    onGetSpecs(specMenuItem.id).then(setSpecList);
  }

  function handleUpdateSpec(id: number) {
    if (!editingSpec) return;
    onUpdateSpec(id, { spec_code: editingSpec.spec_code, spec_name: editingSpec.spec_name, price_delta: editingSpec.price_delta, qty_multiplier: editingSpec.qty_multiplier });
    setEditingSpec(null);
    if (specMenuItem) onGetSpecs(specMenuItem.id).then(setSpecList);
  }

  function handleDeleteSpec(id: number) {
    onDeleteSpec(id);
    if (specMenuItem) onGetSpecs(specMenuItem.id).then(setSpecList);
  }

  async function handleCompleteRecipe(item: MenuItem) {
    if (item.recipe_id) {
      navigate("/recipes", { state: { recipeId: item.recipe_id } });
      return;
    }

    const recipeId = await onCreatePendingRecipeForMenu(item.id, item.name);
    if (recipeId) {
      navigate("/recipes", { state: { guideRecipeId: recipeId } });
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div><h2 className="text-2xl font-semibold tracking-tight">菜单管理</h2><p className="text-sm text-muted-foreground">管理菜单分类、商品和配方绑定</p></div>
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        <Card className="lg:col-span-2">
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle className="flex items-center gap-2"><FileText className="h-4 w-4" />菜单商品</CardTitle>
                <CardDescription>共 {filteredMenuItems.length} 个商品{menuItems.filter(i => !i.is_available).length > 0 && !showUnavailable ? `（${menuItems.filter(i => !i.is_available).length} 个停售已隐藏）` : ""}</CardDescription>
              </div>
              <div className="flex gap-2">
                <Button size="sm" variant="outline" onClick={() => setShowUnavailable(v => !v)} title={showUnavailable ? "隐藏停售商品" : "显示停售商品"}>
                  {showUnavailable ? <EyeOff className="h-4 w-4 mr-1" /> : <Eye className="h-4 w-4 mr-1" />}
                  {showUnavailable ? "隐藏停售" : "显示停售"}
                </Button>
                {selectedItems.length > 0 && onBatchToggleAvailability && (<>
                  <span className="text-sm text-muted-foreground self-center">已选 {selectedItems.length} 项</span>
                  <Button size="sm" variant="outline" onClick={handleBatchEnable}><ToggleRight className="h-4 w-4 mr-1" />批量上架</Button>
                  <Button size="sm" variant="outline" onClick={handleBatchDisable}><ToggleLeft className="h-4 w-4 mr-1" />批量下架</Button>
                </>)}
              </div>
            </div>
          </CardHeader>
          <CardContent>
            {filteredMenuItems.length === 0 ? (
              <EmptyState icon={FileText} title="暂无菜单商品" description="新增菜单商品开始销售" />
            ) : (
              <Table>
                <TableHeader><TableRow>
                    <TableHead className="w-10">
                      <Checkbox checked={selectedItems.length === filteredMenuItems.length && filteredMenuItems.length > 0} onClick={selectAll} />
                    </TableHead>
                    <TableHead>名称</TableHead><TableHead>分类</TableHead><TableHead>配方</TableHead><TableHead className="text-right">售价</TableHead><TableHead>状态</TableHead><TableHead className="text-right">操作</TableHead></TableRow></TableHeader>
                <TableBody>
                  {filteredMenuItems.map((item) => (
                    <TableRow key={item.id}>
                      <TableCell><Checkbox checked={selectedItems.includes(item.id)} onClick={() => toggleSelect(item.id)} /></TableCell>
                      <TableCell className="font-medium">{item.name}</TableCell>
                      <TableCell className="text-xs text-muted-foreground">{menuCategories.find((c) => c.id === item.category_id)?.name || "-"}</TableCell>
                      <TableCell className="text-xs">{item.recipe_id ? (<div className="flex items-center gap-1"><Link className="h-3 w-3 text-emerald-500" /><span className="text-emerald-500">{recipes.find((r) => r.id === item.recipe_id)?.name || `配方 #${item.recipe_id}`}</span></div>) : (<span className="text-muted-foreground">未绑定</span>)}</TableCell>
                      <TableCell className="text-right">¥{item.sales_price.toFixed(2)}</TableCell>
                      <TableCell><Badge variant={item.is_available ? "default" : "secondary"}>{item.is_available ? "可售" : "停售"}</Badge></TableCell>
                      <TableCell className="text-right">
                        <div className="flex justify-end gap-1">
                          <Button variant="outline" size="sm" onClick={() => handleCompleteRecipe(item)}>
                            {item.recipe_id ? "查看配方" : "补配方"}
                          </Button>
                          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => onToggleAvailability(item.id, !item.is_available)}>{item.is_available ? <ToggleRight className="h-4 w-4 text-emerald-500" /> : <ToggleLeft className="h-4 w-4 text-muted-foreground" />}</Button>
                          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => openSpecDialog(item)}><Tag className="h-4 w-4" /></Button>
                          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => openEditItem(item)}><Pencil className="h-4 w-4" /></Button>
                          <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive" onClick={() => confirmDelete('item', item.id, item.name)}><Trash2 className="h-4 w-4" /></Button>
                        </div>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </CardContent>
        </Card>

        <div className="space-y-6">
          <Card>
            <CardHeader><CardTitle>菜单分类</CardTitle></CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label>新增分类</Label>
                <div className="flex gap-2">
                  <Input value={newMenuCategoryName} onChange={(e) => setNewMenuCategoryName(e.target.value)} placeholder="如: 海鲜类" />
                  <Button onClick={() => { onCreateMenuCategory(newMenuCategoryName); setNewMenuCategoryName(""); }}><Plus className="h-4 w-4" /></Button>
                </div>
              </div>
              <Separator />
              <div className="space-y-1">
                {menuCategories.map((cat) => (
                  <div key={cat.id} className="flex items-center justify-between rounded-md border px-3 py-2 text-sm">
                    {editCategoryId === cat.id ? (
                      <div className="flex gap-2 w-full">
                        <Input value={editCategoryName} onChange={(e) => setEditCategoryName(e.target.value)} className="h-8" />
                        <Button size="sm" onClick={saveEditCategory}>保存</Button>
                        <Button size="sm" variant="outline" onClick={() => { setEditCategoryId(null); setEditCategoryName(""); }}>取消</Button>
                      </div>
                    ) : (
                      <><span>{cat.name}</span><div className="flex gap-1">
                        <Button variant="ghost" size="icon" className="h-6 w-6" onClick={() => openEditCategory(cat)}><Edit2 className="h-3 w-3" /></Button>
                        <Button variant="ghost" size="icon" className="h-6 w-6 text-destructive" onClick={() => confirmDelete('category', cat.id, cat.name)}><Trash2 className="h-3 w-3" /></Button>
                      </div></>
                    )}
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader><CardTitle>新增菜单商品</CardTitle></CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2"><Label>商品名称</Label><Input value={newMenuItemName} onChange={(e) => setNewMenuItemName(e.target.value)} placeholder="如: 麻辣小龙虾" /></div>
              <div className="space-y-2"><Label>分类</Label><Select value={newMenuItemCategory} onValueChange={setNewMenuItemCategory}><SelectTrigger><SelectValue placeholder="选择分类（可选）" /></SelectTrigger><SelectContent>{menuCategories.map((cat) => <SelectItem key={cat.id} value={cat.id.toString()}>{cat.name}</SelectItem>)}</SelectContent></Select></div>
              <div className="space-y-2"><Label>绑定配方</Label><Select value={newMenuItemRecipe} onValueChange={setNewMenuItemRecipe}><SelectTrigger><SelectValue placeholder="选择配方（可选）" /></SelectTrigger><SelectContent>{recipes.map((r) => <SelectItem key={r.id} value={r.id.toString()}>{r.name} ({r.code})</SelectItem>)}</SelectContent></Select></div>
              <div className="space-y-2"><Label>售价</Label><Input type="number" value={newMenuItemPrice} onChange={(e) => { setNewMenuItemPrice(e.target.value); setPriceError(""); }} placeholder="0.00" />
                {priceError && <p className="text-xs text-destructive">{priceError}</p>}</div>
              <Button className="w-full" onClick={handleCreateMenuItem} disabled={!newMenuItemName.trim()}><Plus className="mr-2 h-4 w-4" />新增</Button>
            </CardContent>
          </Card>
        </div>
      </div>

      <Dialog open={!!editMenuItem} onOpenChange={() => setEditMenuItem(null)}>
        <DialogContent><DialogHeader><DialogTitle>编辑菜单商品</DialogTitle></DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2"><Label>商品名称</Label><Input value={editMenuItemName} onChange={(e) => setEditMenuItemName(e.target.value)} /></div>
            <div className="space-y-2"><Label>分类</Label><Select value={editMenuItemCategory} onValueChange={setEditMenuItemCategory}><SelectTrigger><SelectValue placeholder="选择分类（可选）" /></SelectTrigger><SelectContent>{menuCategories.map((cat) => <SelectItem key={cat.id} value={cat.id.toString()}>{cat.name}</SelectItem>)}</SelectContent></Select></div>
            <div className="space-y-2"><Label>绑定配方</Label><Select value={editMenuItemRecipe} onValueChange={setEditMenuItemRecipe}><SelectTrigger><SelectValue placeholder="选择配方（可选）" /></SelectTrigger><SelectContent>{recipes.map((r) => <SelectItem key={r.id} value={r.id.toString()}>{r.name} ({r.code})</SelectItem>)}</SelectContent></Select></div>
            <div className="space-y-2"><Label>售价</Label><Input type="number" value={editMenuItemPrice} onChange={(e) => setEditMenuItemPrice(e.target.value)} /></div>
          </div>
          <DialogFooter><Button variant="outline" onClick={() => setEditMenuItem(null)}>取消</Button><Button onClick={saveEditItem}>保存</Button></DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={specDialogOpen} onOpenChange={setSpecDialogOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader><DialogTitle>规格管理 - {specMenuItem?.name}</DialogTitle></DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-3">
              <h4 className="text-sm font-medium">现有规格 ({specList.length})</h4>
              {specList.length === 0 ? (
                <EmptyState icon={Tag} title="暂无规格" description="添加规格支持多选项" />
              ) : (
                <Table>
                  <TableHeader><TableRow><TableHead>代码</TableHead><TableHead>名称</TableHead><TableHead className="text-right">价格变动</TableHead><TableHead className="text-right">数量倍率</TableHead><TableHead className="text-right">操作</TableHead></TableRow></TableHeader>
                  <TableBody>
                    {specList.map((spec) => (
                      <TableRow key={spec.id}>
                        <TableCell>
                          {editingSpec?.id === spec.id ? (
                            <Input value={editingSpec.spec_code} onChange={(e) => setEditingSpec({ ...editingSpec, spec_code: e.target.value })} className="h-8" />
                          ) : spec.spec_code}
                        </TableCell>
                        <TableCell>
                          {editingSpec?.id === spec.id ? (
                            <Input value={editingSpec.spec_name} onChange={(e) => setEditingSpec({ ...editingSpec, spec_name: e.target.value })} className="h-8" />
                          ) : spec.spec_name}
                        </TableCell>
                        <TableCell className="text-right">
                          {editingSpec?.id === spec.id ? (
                            <Input type="number" value={editingSpec.price_delta} onChange={(e) => { const v = parseSafeFloat(e.target.value); setEditingSpec({ ...editingSpec, price_delta: v ?? editingSpec.price_delta }); }} className="h-8 w-20 ml-auto" />
                          ) : (<span className={spec.price_delta > 0 ? "text-destructive" : spec.price_delta < 0 ? "text-emerald-500" : ""}>{spec.price_delta > 0 ? "+" : ""}¥{spec.price_delta.toFixed(2)}</span>)}
                        </TableCell>
                        <TableCell className="text-right">
                          {editingSpec?.id === spec.id ? (
                            <Input type="number" value={editingSpec.qty_multiplier} onChange={(e) => setEditingSpec({ ...editingSpec, qty_multiplier: parseFloat(e.target.value) || 1 })} className="h-8 w-16 ml-auto" />
                          ) : `x${spec.qty_multiplier}`}
                        </TableCell>
                        <TableCell className="text-right">
                          <div className="flex justify-end gap-1">
                            {editingSpec?.id === spec.id ? (
                              <><Button size="sm" variant="ghost" className="h-7 w-7 p-0" onClick={() => handleUpdateSpec(spec.id)}><Save className="h-3 w-3" /></Button><Button size="sm" variant="ghost" className="h-7 w-7 p-0" onClick={() => setEditingSpec(null)}><X className="h-3 w-3" /></Button></>
                            ) : (
                              <><Button size="sm" variant="ghost" className="h-7 w-7 p-0" onClick={() => setEditingSpec(spec)}><Pencil className="h-3 w-3" /></Button><Button size="sm" variant="ghost" className="h-7 w-7 p-0 text-destructive" onClick={() => handleDeleteSpec(spec.id)}><Trash2 className="h-3 w-3" /></Button></>
                            )}
                          </div>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </div>
            <Separator />
            <div className="space-y-3">
              <h4 className="text-sm font-medium">新增规格</h4>
              <div className="grid grid-cols-4 gap-2">
                <Input value={newSpecCode} onChange={(e) => setNewSpecCode(e.target.value)} placeholder="代码" className="h-9" />
                <Input value={newSpecName} onChange={(e) => setNewSpecName(e.target.value)} placeholder="名称" className="h-9" />
                <Input type="number" value={newSpecPriceDelta} onChange={(e) => setNewSpecPriceDelta(e.target.value)} placeholder="价格变动" className="h-9" />
                <div className="flex gap-2">
                  <Input type="number" value={newSpecQtyMultiplier} onChange={(e) => setNewSpecQtyMultiplier(e.target.value)} placeholder="数量倍率" className="h-9" />
                  <Button size="sm" onClick={handleCreateSpec}><Plus className="h-4 w-4" /></Button>
                </div>
              </div>
            </div>
          </div>
          <DialogFooter><Button onClick={() => setSpecDialogOpen(false)}>关闭</Button></DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={!!deleteConfirmTarget} onOpenChange={() => setDeleteConfirmTarget(null)}>
        <DialogContent><DialogHeader><DialogTitle>确认删除</DialogTitle></DialogHeader>
          <p className="py-4 text-sm text-muted-foreground">确定要删除 {deleteConfirmTarget?.type === 'item' ? '商品' : '分类'}「{deleteConfirmTarget?.name}」吗？</p>
          <DialogFooter><Button variant="outline" onClick={() => setDeleteConfirmTarget(null)}>取消</Button><Button variant="destructive" onClick={executeDelete}>删除</Button></DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
