import { useEffect, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Plus, Package, Pencil, Trash2, Save, X } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state";
import { call as invoke } from "@/lib/transport";
import { useNavigate } from "react-router-dom";

interface Unit { id: number; code: string; name: string; }
interface MaterialCategory { id: number; code: string; name: string; }
interface Tag { id: number; code: string; name: string; color?: string; }
interface Material {
  id: number; code: string; name: string; category_id: number | null;
  base_unit_id: number; tags: Tag[]; category?: MaterialCategory; base_unit?: Unit;
}
interface Recipe { id: number; code: string; name: string; recipe_type: string; output_qty: number; }
interface RecipeItem { id: number; recipe_id: number; item_type: string; ref_id: number; qty: number; unit_id: number; wastage_rate: number; note: string | null; sort_no: number; }
interface RecipeWithItems { recipe: Recipe; items: RecipeItem[]; }

interface MaterialsPageProps {
  materials: Material[]; recipes: Recipe[]; categories: MaterialCategory[]; tags: Tag[]; units: Unit[];
  onCreateMaterial: (data: { code: string; name: string; base_unit_id: number; category_id: number | null; tag_ids: number[] }) => void;
  onUpdateMaterial: (id: number, data: { name?: string; category_id?: number | null }) => void;
  onDeleteMaterial: (id: number) => void;
  onRemoveMaterialTag: (material_id: number, tag_id: number) => void;
  onCreateCategory: (data: { code: string; name: string }) => void;
  onDeleteCategory: (id: number) => void;
  onCreateTag: (data: { code: string; name: string; color?: string }) => void;
  onDeleteTag: (id: number) => void;
  searchQuery?: string;
}

export function MaterialsPage({
  materials, recipes, categories, tags, units,
  onCreateMaterial, onUpdateMaterial, onDeleteMaterial, onRemoveMaterialTag,
  onCreateCategory, onDeleteCategory, onCreateTag, onDeleteTag,
  searchQuery,
}: MaterialsPageProps) {
  const navigate = useNavigate();
  const filteredMaterials = materials.filter((m) => {
    if (!searchQuery) return true;
    const q = searchQuery.toLowerCase();
    return m.name.toLowerCase().includes(q) || m.code.toLowerCase().includes(q) ||
      m.category?.name?.toLowerCase().includes(q);
  });
  const [newMaterialName, setNewMaterialName] = useState("");
  const [newMaterialCode, setNewMaterialCode] = useState("");
  const [newMaterialUnit, setNewMaterialUnit] = useState("1");
  const [newMaterialCategory, setNewMaterialCategory] = useState("");
  const [newMaterialTags, setNewMaterialTags] = useState<number[]>([]);

  const [editMaterial, setEditMaterial] = useState<Material | null>(null);
  const [editMaterialName, setEditMaterialName] = useState("");
  const [editMaterialCategory, setEditMaterialCategory] = useState("");

  const [deleteConfirm, setDeleteConfirm] = useState<{ type: string; id: number; name: string } | null>(null);

  const [newCategoryName, setNewCategoryName] = useState("");
  const [newCategoryCode, setNewCategoryCode] = useState("");
  const [newTagName, setNewTagName] = useState("");
  const [newTagCode, setNewTagCode] = useState("");
  const [newTagColor, setNewTagColor] = useState("#3B82F6");
  const [materialRecipeMap, setMaterialRecipeMap] = useState<Record<number, Recipe[]>>({});
  const [usageMaterial, setUsageMaterial] = useState<Material | null>(null);

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
          console.error("加载材料配方引用失败", e);
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

  function openEditMaterial(m: Material) {
    setEditMaterial(m);
    setEditMaterialName(m.name);
    setEditMaterialCategory(m.category_id?.toString() || "");
  }

  function saveEditMaterial() {
    if (!editMaterial) return;
    onUpdateMaterial(editMaterial.id, {
      name: editMaterialName || undefined,
      category_id: editMaterialCategory ? parseInt(editMaterialCategory) : null,
    });
    setEditMaterial(null);
  }

  function toggleNewTag(tagId: number) {
    setNewMaterialTags((prev) => prev.includes(tagId) ? prev.filter((t) => t !== tagId) : [...prev, tagId]);
  }

  function handleCreateMaterial() {
    onCreateMaterial({
      code: newMaterialCode, name: newMaterialName,
      base_unit_id: parseInt(newMaterialUnit),
      category_id: newMaterialCategory ? parseInt(newMaterialCategory) : null,
      tag_ids: newMaterialTags,
    });
    setNewMaterialCode(""); setNewMaterialName(""); setNewMaterialUnit("1"); setNewMaterialCategory(""); setNewMaterialTags([]);
  }

  function openRecipeUsage(material: Material) {
    setUsageMaterial(material);
  }

  function goToRecipe(recipeId: number) {
    navigate("/recipes", { state: { recipeId } });
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">原材料管理</h2>
          <p className="text-sm text-muted-foreground">管理所有原材料、分类和标签</p>
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle className="flex items-center gap-2"><Package className="h-4 w-4" />原材料列表</CardTitle>
            <CardDescription>共 {filteredMaterials.length} 种原材料{filteredMaterials.length !== materials.length ? `（筛选自 ${materials.length} 种）` : ""}</CardDescription>
          </CardHeader>
          <CardContent>
            {filteredMaterials.length === 0 ? (
              <EmptyState icon={Package} title="暂无原材料" description="添加原材料开始管理库存" />
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>代码</TableHead>
                    <TableHead>名称</TableHead>
                    <TableHead>分类</TableHead>
                    <TableHead>单位</TableHead>
                    <TableHead>引用配方</TableHead>
                    <TableHead>标签</TableHead>
                    <TableHead className="text-right">操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {filteredMaterials.map((m) => (
                    <TableRow key={m.id}>
                      <TableCell className="font-mono text-xs">{m.code}</TableCell>
                      <TableCell className="font-medium">{m.name}</TableCell>
                      <TableCell className="text-muted-foreground">{m.category?.name || "-"}</TableCell>
                      <TableCell className="text-muted-foreground">{m.base_unit?.name || "-"}</TableCell>
                      <TableCell>
                        {(materialRecipeMap[m.id]?.length || 0) > 0 ? (
                          <Button variant="outline" size="sm" onClick={() => openRecipeUsage(m)}>
                            查看 {materialRecipeMap[m.id].length} 个配方
                          </Button>
                        ) : (
                          <span className="text-xs text-muted-foreground">未被配方使用</span>
                        )}
                      </TableCell>
                      <TableCell>
                        <div className="flex gap-1 flex-wrap">
                          {m.tags.map((tag) => (
                            <Badge key={tag.id} variant="secondary" style={tag.color ? { backgroundColor: tag.color + "20", color: tag.color } : {}} className="cursor-pointer" onClick={() => onRemoveMaterialTag(m.id, tag.id)}>
                              {tag.name} <X className="ml-1 h-3 w-3" />
                            </Badge>
                          ))}
                        </div>
                      </TableCell>
                      <TableCell className="text-right">
                        <div className="flex justify-end gap-1">
                          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => openEditMaterial(m)}><Pencil className="h-4 w-4" /></Button>
                          <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive" onClick={() => setDeleteConfirm({ type: "material", id: m.id, name: m.name })}><Trash2 className="h-4 w-4" /></Button>
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
            <CardHeader><CardTitle>新增原材料</CardTitle></CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label>代码</Label>
                <Input value={newMaterialCode} onChange={(e) => setNewMaterialCode(e.target.value)} placeholder="如: MAT001" />
              </div>
              <div className="space-y-2">
                <Label>名称</Label>
                <Input value={newMaterialName} onChange={(e) => setNewMaterialName(e.target.value)} placeholder="输入原材料名称..." />
              </div>
              <div className="space-y-2">
                <Label>分类</Label>
                <Select value={newMaterialCategory} onValueChange={setNewMaterialCategory}>
                  <SelectTrigger><SelectValue placeholder="选择分类（可选）" /></SelectTrigger>
                  <SelectContent>
                    {categories.map((c) => <SelectItem key={c.id} value={c.id.toString()}>{c.name}</SelectItem>)}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>单位</Label>
                <Select value={newMaterialUnit} onValueChange={setNewMaterialUnit}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    {units.map((u) => <SelectItem key={u.id} value={u.id.toString()}>{u.name} ({u.code})</SelectItem>)}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>标签（多选）</Label>
                <div className="flex flex-wrap gap-2 max-h-24 overflow-y-auto">
                  {tags.map((t) => (
                    <Badge key={t.id} variant={newMaterialTags.includes(t.id) ? "default" : "outline"} className="cursor-pointer" onClick={() => toggleNewTag(t.id)} style={!newMaterialTags.includes(t.id) && t.color ? { borderColor: t.color, color: t.color } : {}}>
                      {t.name}
                    </Badge>
                  ))}
                </div>
              </div>
              <Button className="w-full" onClick={handleCreateMaterial}><Plus className="mr-2 h-4 w-4" />新增</Button>
            </CardContent>
          </Card>

          <Card>
            <CardHeader><CardTitle>材料分类</CardTitle></CardHeader>
            <CardContent className="space-y-4">
              <div className="flex gap-2">
                <Input placeholder="代码" value={newCategoryCode} onChange={(e) => setNewCategoryCode(e.target.value)} className="flex-1" />
                <Input placeholder="名称" value={newCategoryName} onChange={(e) => setNewCategoryName(e.target.value)} className="flex-1" />
                <Button onClick={() => { onCreateCategory({ code: newCategoryCode, name: newCategoryName }); setNewCategoryCode(""); setNewCategoryName(""); }}>新增</Button>
              </div>
              <Separator />
              <div className="space-y-1">
                {categories.map((cat) => (
                  <div key={cat.id} className="flex items-center justify-between rounded-md border px-3 py-2 text-sm">
                    <span>{cat.name}</span>
                    <div className="flex gap-1">
                      <span className="font-mono text-xs text-muted-foreground">{cat.code}</span>
                      <Button variant="ghost" size="icon" className="h-6 w-6 text-destructive" onClick={() => setDeleteConfirm({ type: "category", id: cat.id, name: cat.name })}><Trash2 className="h-3 w-3" /></Button>
                    </div>
                  </div>
                ))}
              </div>
              <Separator />
              <div className="space-y-2">
                <h4 className="text-sm font-medium">快速添加</h4>
                <div className="flex flex-wrap gap-2">
                  {["一次性消耗品", "原材料", "半成品", "包装材料"].map((name) => {
                    const code = name.charAt(0).toUpperCase() + name.replace(/\s/g, "").slice(1);
                    return (
                      <Button key={name} variant="outline" size="sm" onClick={() => { setNewCategoryCode(code); setNewCategoryName(name); }}>
                        {name}
                      </Button>
                    );
                  })}
                </div>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader><CardTitle>标签</CardTitle></CardHeader>
            <CardContent className="space-y-4">
              <div className="flex gap-2">
                <Input placeholder="代码" value={newTagCode} onChange={(e) => setNewTagCode(e.target.value)} className="flex-1" />
                <Input placeholder="名称" value={newTagName} onChange={(e) => setNewTagName(e.target.value)} className="flex-1" />
                <Input type="color" value={newTagColor} onChange={(e) => setNewTagColor(e.target.value)} className="w-12 h-9 p-1" />
                <Button onClick={() => { onCreateTag({ code: newTagCode, name: newTagName, color: newTagColor }); setNewTagCode(""); setNewTagName(""); }}>新增</Button>
              </div>
              <Separator />
              <div className="flex flex-wrap gap-2">
                {tags.map((tag) => (
                  <Badge key={tag.id} variant="secondary" style={tag.color ? { backgroundColor: tag.color + "20", color: tag.color } : {}} className="gap-1">
                    {tag.name}
                    <Button variant="ghost" size="sm" className="h-4 w-4 p-0 ml-1 hover:opacity-70" onClick={() => setDeleteConfirm({ type: "tag", id: tag.id, name: tag.name })}><X className="h-3 w-3" /></Button>
                  </Badge>
                ))}
              </div>
            </CardContent>
          </Card>
        </div>
      </div>

      <Dialog open={!!editMaterial} onOpenChange={() => setEditMaterial(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>编辑材料</DialogTitle></DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>名称</Label>
              <Input value={editMaterialName} onChange={(e) => setEditMaterialName(e.target.value)} />
            </div>
            <div className="space-y-2">
              <Label>分类</Label>
              <Select value={editMaterialCategory} onValueChange={setEditMaterialCategory}>
                <SelectTrigger><SelectValue placeholder="选择分类（可选）" /></SelectTrigger>
                <SelectContent>
                  {categories.map((c) => <SelectItem key={c.id} value={c.id.toString()}>{c.name}</SelectItem>)}
                </SelectContent>
              </Select>
            </div>
            <p className="text-xs text-muted-foreground bg-muted p-2 rounded">
              单位与标签锁定，如需修改请联系管理员或删除后重新添加。
            </p>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditMaterial(null)}>取消</Button>
            <Button onClick={saveEditMaterial}><Save className="mr-1 h-4 w-4" />保存</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={!!usageMaterial} onOpenChange={() => setUsageMaterial(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>配方引用 - {usageMaterial?.name}</DialogTitle></DialogHeader>
          <div className="space-y-3 py-4">
            {(usageMaterial && materialRecipeMap[usageMaterial.id]?.length) ? (
              materialRecipeMap[usageMaterial.id].map((recipe) => (
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
              <EmptyState icon={Package} title="暂无引用" description="这个材料还没有被任何配方使用" />
            )}
          </div>
          <DialogFooter>
            <Button onClick={() => setUsageMaterial(null)}>关闭</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={!!deleteConfirm} onOpenChange={() => setDeleteConfirm(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>确认删除</DialogTitle></DialogHeader>
          <p className="py-4 text-sm text-muted-foreground">确定要删除 {deleteConfirm?.type === "material" ? "材料" : deleteConfirm?.type === "category" ? "分类" : "标签"}「{deleteConfirm?.name}」吗？</p>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteConfirm(null)}>取消</Button>
            <Button variant="destructive" onClick={() => {
              if (!deleteConfirm) return;
              if (deleteConfirm.type === "material") onDeleteMaterial(deleteConfirm.id);
              else if (deleteConfirm.type === "category") onDeleteCategory(deleteConfirm.id);
              else if (deleteConfirm.type === "tag") onDeleteTag(deleteConfirm.id);
              setDeleteConfirm(null);
            }}>删除</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
