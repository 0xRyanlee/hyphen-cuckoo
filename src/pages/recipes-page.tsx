import { useState, useEffect, Fragment } from "react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue, SelectGroup, SelectLabel } from "@/components/ui/select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Plus, ChefHat, Trash2, Pencil, Save, X, ChevronRight, ChevronDown, Network } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state";
import { toast } from "sonner";
import { parseSafeFloat } from "@/lib/utils";
import { invoke } from "@tauri-apps/api/core";
import { useLocation, useNavigate } from "react-router-dom";

interface Recipe {
  id: number;
  code: string;
  name: string;
  recipe_type: string;
  output_qty: number;
  output_material_id: number | null;
  output_state_id: number | null;
  output_unit_id: number | null;
  cost: number | null;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

interface RecipeType {
  id: number;
  code: string;
  name: string;
  description?: string | null;
  sort_no: number;
  is_system: boolean;
  is_active: boolean;
}

interface MenuItem {
  id: number;
  name: string;
  recipe_id: number | null;
  sales_price: number;
}

interface RecipeItem {
  id: number;
  recipe_id: number;
  item_type: string;
  ref_id: number;
  qty: number;
  unit_id: number;
  wastage_rate: number;
  note: string | null;
  sort_no: number;
}

interface RecipeWithItems {
  recipe: Recipe;
  items: RecipeItem[];
}

interface RecipeCostItem { material_name: string; qty: number; unit: string; cost_per_unit: number; wastage_rate: number; line_cost: number; item_type: string; }
interface RecipeCostResult {
  recipe_id: number;
  recipe_name: string;
  total_cost: number;
  cost_per_unit: number;
  output_qty: number;
  items: RecipeCostItem[];
}

interface Material {
  id: number;
  code: string;
  name: string;
  category_id: number | null;
  base_unit_id: number;
  tags: { id: number; code: string; name: string; color?: string }[];
  category?: { id: number; code: string; name: string };
  base_unit?: { id: number; code: string; name: string; unit_type: string; ratio_to_base: number };
}

interface Unit {
  id: number;
  code: string;
  name: string;
  unit_type: string;
  ratio_to_base: number;
}

interface DependentRef { id: number; name: string; }
interface RecipeDependents { menu_items: DependentRef[]; parent_recipes: DependentRef[]; }

interface RecipesPageProps {
  recipes: Recipe[];
  recipeTypes: RecipeType[];
  selectedRecipe: RecipeWithItems | null;
  recipeCost: RecipeCostResult | null;
  materials: Material[];
  menuItems: MenuItem[];
  units: Unit[];
  onCreateRecipe: (data: { code: string; name: string; recipe_type: string }) => Promise<number | null>;
  onViewRecipe: (recipe: Recipe) => void;
  onDeleteRecipe: (id: number) => void;
  onUpdateRecipe: (id: number, data: { name: string; recipe_type: string; output_qty: number }) => void;
  onCreateRecipeType: (data: { code: string; name: string; description?: string | null; sort_no?: number }) => void;
  onUpdateRecipeType: (id: number, data: { code: string; name: string; description?: string | null; sort_no: number }) => void;
  onDeleteRecipeType: (id: number) => void;
  onSeedSampleRecipes: () => Promise<void>;
  onCreatePendingRecipeForMenu: (menuItemId: number, menuItemName: string) => Promise<number | null>;
  onBindMenuItemToRecipe: (menuItemId: number, recipeId: number) => Promise<void>;
  onAddRecipeItem: (recipe_id: number, item_type: string, ref_id: number, qty: number, unit_id: number, wastage_rate: number) => void;
  onDeleteRecipeItem: (item_id: number, recipe_id: number) => void;
  onUpdateRecipeItem: (item_id: number, recipe_id: number, qty: number, wastage_rate: number) => void;
  onRecalculateCost: (recipe_id: number) => void;
  searchQuery?: string;
}

export function RecipesPage({
  recipes,
  recipeTypes,
  selectedRecipe,
  recipeCost,
  materials,
  menuItems,
  units,
  onCreateRecipe,
  onViewRecipe,
  onDeleteRecipe,
  onUpdateRecipe,
  onCreateRecipeType,
  onUpdateRecipeType,
  onDeleteRecipeType,
  onSeedSampleRecipes,
  onCreatePendingRecipeForMenu,
  onBindMenuItemToRecipe,
  onAddRecipeItem,
  onDeleteRecipeItem,
  onUpdateRecipeItem,
  onRecalculateCost,
  searchQuery,
}: RecipesPageProps) {
  const location = useLocation();
  const navigate = useNavigate();
  const activeRecipe = selectedRecipe?.recipe ? selectedRecipe : null;
  const [pendingRecipeId, setPendingRecipeId] = useState<number | null>(null);
  const [pageTab, setPageTab] = useState("recipes");
  const [recipeListFilter, setRecipeListFilter] = useState("all");
  const [seedingSamples, setSeedingSamples] = useState(false);
  
  const filteredRecipes = recipes.filter((r) => {
    if (!searchQuery) return true;
    const q = searchQuery.toLowerCase();
    return r.name.toLowerCase().includes(q) || r.code.toLowerCase().includes(q);
  });
  const [newRecipeName, setNewRecipeName] = useState("");
  const [newRecipeCode, setNewRecipeCode] = useState("");
  const [newRecipeType, setNewRecipeType] = useState("");
  const [creatingRecipe, setCreatingRecipe] = useState(false);
  const [guidedRecipeId, setGuidedRecipeId] = useState<number | null>(null);

  useEffect(() => {
    invoke<string>("generate_recipe_code").then(setNewRecipeCode).catch(console.error);
  }, []);

  useEffect(() => {
    if (!newRecipeType && recipeTypes.length > 0) {
      setNewRecipeType(recipeTypes[0].code);
    }
  }, [newRecipeType, recipeTypes]);

  useEffect(() => {
    if (pendingRecipeId) {
      const recipe = recipes.find(r => r.id === pendingRecipeId);
      if (recipe) {
        onViewRecipe(recipe);
        setPendingRecipeId(null);
      }
    }
  }, [recipes, pendingRecipeId, onViewRecipe]);

  useEffect(() => {
    if (!guidedRecipeId || activeRecipe?.recipe.id !== guidedRecipeId) return;
    openQuickAdd(guidedRecipeId);
    setGuidedRecipeId(null);
  }, [guidedRecipeId, activeRecipe?.recipe.id]);

  useEffect(() => {
    const routeState = location.state as { recipeId?: number; guideRecipeId?: number } | null;
    const targetRecipeId = routeState?.guideRecipeId ?? routeState?.recipeId;
    if (!targetRecipeId) return;

    const targetRecipe = recipes.find((recipe) => recipe.id === targetRecipeId);
    if (!targetRecipe) return;

    if (routeState?.guideRecipeId) {
      setGuidedRecipeId(targetRecipe.id);
    }
    handleSelectRecipe(targetRecipe);
    navigate(location.pathname, { replace: true, state: null });
  }, [location.pathname, location.state, navigate, recipes]);

  const [editRecipeId, setEditRecipeId] = useState<number | null>(null);
  const [editRecipeName, setEditRecipeName] = useState("");
  const [editRecipeType, setEditRecipeType] = useState("");
  const [editRecipeOutputQty, setEditRecipeOutputQty] = useState("");

  const [newRecipeTypeCode, setNewRecipeTypeCode] = useState("");
  const [newRecipeTypeName, setNewRecipeTypeName] = useState("");
  const [newRecipeTypeDescription, setNewRecipeTypeDescription] = useState("");
  const [editingRecipeTypeId, setEditingRecipeTypeId] = useState<number | null>(null);
  const [editingRecipeTypeCode, setEditingRecipeTypeCode] = useState("");
  const [editingRecipeTypeName, setEditingRecipeTypeName] = useState("");
  const [editingRecipeTypeDescription, setEditingRecipeTypeDescription] = useState("");
  const [deleteRecipeTypeConfirm, setDeleteRecipeTypeConfirm] = useState<number | null>(null);

  const [deleteItemConfirm, setDeleteItemConfirm] = useState<number | null>(null);
  const [deleteRecipeConfirm, setDeleteRecipeConfirm] = useState<number | null>(null);
  const [warningDialog, setWarningDialog] = useState<{ message: string; onConfirm: () => void } | null>(null);
  const [dependsDialog, setDependsDialog] = useState<{ recipeId: number; recipeName: string } | null>(null);
  const [dependents, setDependents] = useState<RecipeDependents | null>(null);

  useEffect(() => {
    if (!dependsDialog) { setDependents(null); return; }
    invoke<RecipeDependents>("get_recipe_dependents", { recipeId: dependsDialog.recipeId })
      .then(setDependents)
      .catch(() => setDependents({ menu_items: [], parent_recipes: [] }));
  }, [dependsDialog]);

  const [expandedItems, setExpandedItems] = useState<Set<number>>(new Set<number>());
  const [subRecipes, setSubRecipes] = useState<Record<number, RecipeItem[]>>({});

  const [editingItemId, setEditingItemId] = useState<number | null>(null);
  const [editingItemQty, setEditingItemQty] = useState("");
  const [editingItemWastage, setEditingItemWastage] = useState("");

  const [quickAddRecipeId, setQuickAddRecipeId] = useState<number | null>(null);
  const [quickAddMaterial, setQuickAddMaterial] = useState("");
  const [quickAddQty, setQuickAddQty] = useState("");
  const [quickAddUnit, setQuickAddUnit] = useState("");
  const [quickAddWastage, setQuickAddWastage] = useState("0");
  const [selectedRecipeUsageCount, setSelectedRecipeUsageCount] = useState<number | null>(null);
  const [recipeItemCounts, setRecipeItemCounts] = useState<Record<number, number>>({});
  const [recipeCostTotals, setRecipeCostTotals] = useState<Record<number, number | null>>({});

  useEffect(() => {
    if (!activeRecipe?.recipe?.id) {
      setSelectedRecipeUsageCount(null);
      return;
    }

    invoke<number>("get_recipe_usage_count", { recipeId: activeRecipe.recipe.id })
      .then(setSelectedRecipeUsageCount)
      .catch(() => setSelectedRecipeUsageCount(null));
  }, [activeRecipe?.recipe?.id]);

  // 選擇物料後自動設定單位，並限制只顯示同 unit_type 的單位
  const quickAddCompatibleUnits = (() => {
    if (!quickAddMaterial.startsWith("m_")) return units;
    const materialId = parseInt(quickAddMaterial.substring(2), 10);
    const material = materials.find((item) => item.id === materialId);
    if (!material?.base_unit?.unit_type) return units;
    return units.filter((u) => u.unit_type === material.base_unit!.unit_type);
  })();

  useEffect(() => {
    if (!quickAddMaterial.startsWith("m_")) return;
    const materialId = parseInt(quickAddMaterial.substring(2), 10);
    const material = materials.find((item) => item.id === materialId);
    if (material) {
      setQuickAddUnit(material.base_unit_id.toString());
    }
  }, [quickAddMaterial, materials]);

  useEffect(() => {
    let cancelled = false;

    async function loadRecipeItemCounts() {
      try {
        const pairs = await Promise.all(
          recipes.map(async (recipe) => {
            const data = await invoke<RecipeWithItems>("get_recipe_with_items", { recipeId: recipe.id });
            return [recipe.id, data.items.length] as const;
          }),
        );
        if (!cancelled) {
          setRecipeItemCounts(Object.fromEntries(pairs));
        }
      } catch (e) {
        if (!cancelled) {
          console.error("加载配方明细数量失败", e);
        }
      }
    }

    if (recipes.length > 0) {
      loadRecipeItemCounts();
    } else {
      setRecipeItemCounts({});
    }

    return () => {
      cancelled = true;
    };
  }, [recipes]);

  useEffect(() => {
    let cancelled = false;

    async function loadRecipeCostTotals() {
      try {
        const pairs = await Promise.all(
          recipes.map(async (recipe) => {
            const itemCount = recipeItemCounts[recipe.id] ?? 0;
            if (itemCount === 0) {
              return [recipe.id, null] as const;
            }
            try {
              const cost = await invoke<RecipeCostResult>("calculate_recipe_cost", { recipeId: recipe.id });
              return [recipe.id, cost.total_cost] as const;
            } catch {
              return [recipe.id, null] as const;
            }
          }),
        );
        if (!cancelled) {
          setRecipeCostTotals(Object.fromEntries(pairs));
        }
      } catch (e) {
        if (!cancelled) {
          console.error("加载配方成本摘要失败", e);
        }
      }
    }

    if (recipes.length > 0 && Object.keys(recipeItemCounts).length > 0) {
      loadRecipeCostTotals();
    }

    return () => {
      cancelled = true;
    };
  }, [recipes, recipeItemCounts]);

  async function toggleExpand(refId: number) {
    if (expandedItems.has(refId)) {
      setExpandedItems((prev) => {
        const next = new Set(prev);
        next.delete(refId);
        return next;
      });
    } else {
      if (!subRecipes[refId]) {
        try {
          const data = await invoke<RecipeWithItems>("get_recipe_with_items", { recipeId: refId });
          setSubRecipes((prev) => ({ ...prev, [refId]: data.items }));
        } catch (e) {
          toast.error("加载子配方明细失败");
          return;
        }
      }
      setExpandedItems((prev) => {
        const next = new Set(prev);
        next.add(refId);
        return next;
      });
    }
  }

  function startEditItem(item: RecipeItem) {
    setEditingItemId(item.id);
    setEditingItemQty(item.qty.toString());
    setEditingItemWastage((item.wastage_rate * 100).toString());
  }

  async function saveEditItem() {
    if (!editingItemId || !activeRecipe) return;
    const qty = parseSafeFloat(editingItemQty);
    const wastage = parseSafeFloat(editingItemWastage);
    if (qty === null || qty <= 0) {
      toast.error("数量必须大于 0");
      return;
    }
    if (wastage !== null && (wastage < 0 || wastage > 100)) {
      toast.error("损耗率必须在 0–100 之间");
      return;
    }

    const currentItem = activeRecipe.items.find(i => i.id === editingItemId);
    if (currentItem?.item_type === "sub_recipe") {
      try {
        const count = await invoke<number>("get_recipe_usage_count", { recipeId: currentItem.ref_id });
        if (count > 0) {
          const itemId = editingItemId;
          const recipeId = activeRecipe.recipe.id;
          const finalQty = qty;
          const finalWastage = (wastage ?? 0) / 100;
          setWarningDialog({
            message: `此半成品被 ${count} 个其他配方引用，修改其用量或损耗会影响相关成本，确定修改吗？`,
            onConfirm: () => { onUpdateRecipeItem(itemId, recipeId, finalQty, finalWastage); setEditingItemId(null); },
          });
          return;
        }
      } catch (e) {
        console.error("检查依赖失败", e);
      }
    }

    onUpdateRecipeItem(editingItemId, activeRecipe.recipe.id, qty, (wastage ?? 0) / 100);
    setEditingItemId(null);
  }

  function openQuickAdd(recipeId: number) {
    setQuickAddRecipeId(recipeId);
    setQuickAddMaterial("");
    setQuickAddQty("");
    setQuickAddUnit("");
    setQuickAddWastage("0");
  }

  async function saveQuickAdd() {
    if (!quickAddRecipeId || !quickAddMaterial || !quickAddQty || !quickAddUnit) {
      toast.error("请填写必填字段");
      return;
    }
    const qty = parseSafeFloat(quickAddQty);
    const wastage = parseSafeFloat(quickAddWastage);
    if (qty === null || qty <= 0) {
      toast.error("数量必须大于 0");
      return;
    }
    if (wastage !== null && (wastage < 0 || wastage > 100)) {
      toast.error("损耗率必须在 0–100 之间");
      return;
    }
    const isMaterial = quickAddMaterial.startsWith("m_");
    const refId = parseInt(quickAddMaterial.substring(2));

    if (!isMaterial && refId === quickAddRecipeId) {
      toast.error("不能添加自身作为子配方");
      return;
    }

    onAddRecipeItem(
      quickAddRecipeId,
      isMaterial ? "material" : "sub_recipe",
      refId,
      qty,
      parseInt(quickAddUnit),
      (wastage ?? 0) / 100,
    );
    setQuickAddRecipeId(null);
  }

  function openEditRecipe(recipe: Recipe) {
    setEditRecipeId(recipe.id);
    setEditRecipeName(recipe.name);
    setEditRecipeType(recipe.recipe_type);
    setEditRecipeOutputQty(recipe.output_qty.toString());
  }

  async function saveEditRecipe() {
    if (!editRecipeId) return;
    
    try {
      const count = await invoke<number>("get_recipe_usage_count", { recipeId: editRecipeId });
      if (count > 0) {
        const id = editRecipeId;
        const name = editRecipeName.trim();
        const type = editRecipeType;
        const outputQty = parseFloat(editRecipeOutputQty) || 1.0;
        setWarningDialog({
          message: `此配方作为半成品被 ${count} 个其他配方引用，修改名称或产出量会直接影响相关成本核算，确定继续吗？`,
          onConfirm: () => { onUpdateRecipe(id, { name, recipe_type: type, output_qty: outputQty }); setEditRecipeId(null); },
        });
        return;
      }
    } catch (e) {
      console.error("检查依赖失败", e);
    }

    onUpdateRecipe(editRecipeId, {
      name: editRecipeName,
      recipe_type: editRecipeType,
      output_qty: parseFloat(editRecipeOutputQty) || 1,
    });
    setEditRecipeId(null);
  }

  function getRefName(itemType: string, refId: number): string {
    if (itemType === "sub_recipe") {
      return recipes.find((r) => r.id === refId)?.name || `配方 #${refId}`;
    }
    return materials.find((m) => m.id === refId)?.name || `材料 #${refId}`;
  }


  function getUnitName(unitId: number): string {
    return units.find((u) => u.id === unitId)?.name || `单位 #${unitId}`;
  }

  async function handleCreateRecipeClick() {
    if (creatingRecipe) return;
    if (!newRecipeName.trim()) {
      toast.error("配方名称不能为空");
      return;
    }
    if (!newRecipeType) {
      toast.error("请选择配方类型");
      return;
    }

    setCreatingRecipe(true);
    try {
      const newId = await onCreateRecipe({ code: newRecipeCode, name: newRecipeName.trim(), recipe_type: newRecipeType });
      if (newId) {
        setPendingRecipeId(newId);
        setNewRecipeName("");
      }
      const nextCode = await invoke<string>("generate_recipe_code");
      setNewRecipeCode(nextCode);
    } catch (e) {
      console.error("创建配方失败", e);
    } finally {
      setCreatingRecipe(false);
    }
  }

  function handleCreateRecipeTypeClick() {
    if (!newRecipeTypeCode.trim() || !newRecipeTypeName.trim()) {
      toast.error("请填写配方类型代码与名称");
      return;
    }
    onCreateRecipeType({
      code: newRecipeTypeCode.trim(),
      name: newRecipeTypeName.trim(),
      description: newRecipeTypeDescription.trim() || null,
      sort_no: recipeTypes.length + 1,
    });
    setNewRecipeTypeCode("");
    setNewRecipeTypeName("");
    setNewRecipeTypeDescription("");
  }

  function openEditRecipeType(type: RecipeType) {
    setEditingRecipeTypeId(type.id);
    setEditingRecipeTypeCode(type.code);
    setEditingRecipeTypeName(type.name);
    setEditingRecipeTypeDescription(type.description || "");
  }

  function saveRecipeTypeEdit() {
    if (!editingRecipeTypeId) return;
    onUpdateRecipeType(editingRecipeTypeId, {
      code: editingRecipeTypeCode.trim(),
      name: editingRecipeTypeName.trim(),
      description: editingRecipeTypeDescription.trim() || null,
      sort_no: recipeTypes.find((item) => item.id === editingRecipeTypeId)?.sort_no || 0,
    });
    setEditingRecipeTypeId(null);
    setEditingRecipeTypeCode("");
    setEditingRecipeTypeName("");
    setEditingRecipeTypeDescription("");
  }

  const recipeTypeMap = Object.fromEntries(recipeTypes.map((item) => [item.code, item]));

  const directMaterialCount = activeRecipe?.items.filter((item) => item.item_type === "material").length ?? 0;
  const subRecipeCount = activeRecipe?.items.filter((item) => item.item_type === "sub_recipe").length ?? 0;
  const boundMenuItems = activeRecipe
    ? menuItems.filter((item) => item.recipe_id === activeRecipe.recipe.id)
    : [];
  const pendingMenuItems = menuItems.filter((item) => item.recipe_id === null);
  const pendingRecipes = filteredRecipes.filter((recipe) => recipeItemCounts[recipe.id] === 0);
  const visibleRecipes = recipeListFilter === "pending" ? pendingRecipes : filteredRecipes;

  function handleSelectRecipe(recipe: Recipe) {
    onViewRecipe(recipe);
    setPageTab("editor");
  }

  function handleGuideRecipe(recipe: Recipe) {
    setGuidedRecipeId(recipe.id);
    handleSelectRecipe(recipe);
  }

  async function handleSeedSamples() {
    if (seedingSamples) return;
    setSeedingSamples(true);
    try {
      await onSeedSampleRecipes();
      setPageTab("recipes");
    } finally {
      setSeedingSamples(false);
    }
  }

  function getRecipeStatus(recipe: Recipe) {
    const itemCount = recipeItemCounts[recipe.id] ?? 0;
    const boundCount = menuItems.filter((item) => item.recipe_id === recipe.id).length;
    const totalCost = recipeCostTotals[recipe.id];

    if (itemCount === 0) {
      return {
        code: "pending",
        label: "待完善",
        description: "还没有任何配方明细项",
        badgeVariant: "secondary" as const,
      };
    }

    if (boundCount > 0) {
      return {
        code: "bound",
        label: "已绑定菜单",
        description: `已绑定 ${boundCount} 个菜单商品`,
        badgeVariant: "default" as const,
      };
    }

    if (totalCost === null || totalCost <= 0) {
      return {
        code: "review",
        label: "待核对",
        description: "已有结构，但成本来源还需要核对",
        badgeVariant: "outline" as const,
      };
    }

    return {
      code: "ready",
      label: "可出品",
      description: "结构完整，可继续绑定菜单",
      badgeVariant: "default" as const,
    };
  }

  function goToMenuBinding(recipe: Recipe) {
    navigate("/menu");
    toast.message("已跳转到菜单页", {
      description: `请为配方“${recipe.name}”绑定菜单商品`,
    });
  }

  function goToInventoryCost(materialId?: number) {
    navigate("/inventory", { state: materialId ? { materialId } : null });
    toast.message("已跳转到库存页", {
      description: materialId ? "请为该原材料补充批次与单位成本" : "请补充相关原材料的批次与单位成本",
    });
  }

  function renderSubItems(subRecipeId: number, depth: number = 1): React.ReactNode {
    if (depth > 5) return <div className="text-destructive text-xs py-1">超过最大展开深度，存在循环依赖风险</div>;
    const items = subRecipes[subRecipeId];
    if (!items) return <div className="text-muted-foreground text-xs py-1 px-4">加载中...</div>;
    if (items.length === 0) return <div className="text-muted-foreground text-xs py-1 px-4">空配方</div>;
    
    return (
      <div className="space-y-1">
        {items.map(item => {
          const hasSub = item.item_type === "sub_recipe";
          const isExpanded = expandedItems.has(item.ref_id);
          return (
            <div key={item.id} className="flex flex-col border-b last:border-0 border-muted/30 py-1">
              <div className="flex items-center text-sm">
                <div className="flex-1 flex items-center gap-1">
                  {hasSub ? (
                    <Button variant="ghost" size="icon" className="h-4 w-4 p-0 mr-1" onClick={() => toggleExpand(item.ref_id)}>
                      {isExpanded ? <ChevronDown className="h-3 w-3" /> : <ChevronRight className="h-3 w-3" />}
                    </Button>
                  ) : <span className="w-5 inline-block" />}
                  {hasSub && <Badge variant="outline" className="mr-1 text-[10px] px-1 py-0 h-4">半成品</Badge>}
                  {getRefName(item.item_type, item.ref_id)}
                </div>
                <div className="w-24 text-right text-muted-foreground mr-[5.5rem]">{item.qty} {getUnitName(item.unit_id)}</div>
                <div className="w-24 text-right text-muted-foreground mr-[7.5rem]">{(item.wastage_rate * 100).toFixed(1)}%</div>
              </div>
              {hasSub && isExpanded && (
                <div className="pl-6 mt-1 border-l-2 border-muted/50 ml-2">
                  {renderSubItems(item.ref_id, depth + 1)}
                </div>
              )}
            </div>
          );
        })}
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">配方管理</h2>
          <p className="text-sm text-muted-foreground">围绕材料、半成品、成品和菜单绑定，统一管理配方结构与成本</p>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">配方总数</CardTitle>
          </CardHeader>
          <CardContent><div className="text-2xl font-bold">{recipes.length}</div></CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">配方类型</CardTitle>
          </CardHeader>
          <CardContent><div className="text-2xl font-bold">{recipeTypes.length}</div></CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">已绑定菜单</CardTitle>
          </CardHeader>
          <CardContent><div className="text-2xl font-bold">{menuItems.filter((item) => item.recipe_id !== null).length}</div></CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">可选材料</CardTitle>
          </CardHeader>
          <CardContent><div className="text-2xl font-bold">{materials.length}</div></CardContent>
        </Card>
      </div>

      <Tabs value={pageTab} onValueChange={setPageTab} className="w-full">
        <TabsList variant="line" className="w-full">
          <TabsTrigger value="recipes">配方列表</TabsTrigger>
          <TabsTrigger value="editor">配方编辑</TabsTrigger>
          <TabsTrigger value="types">类型管理</TabsTrigger>
        </TabsList>

        <TabsContent value="recipes" className="mt-6">
          <div className="grid gap-6 lg:grid-cols-3">
            <Card className="lg:col-span-2">
              <CardHeader>
                <div className="flex items-start justify-between gap-4">
                  <div>
                    <CardTitle className="flex items-center gap-2">
                      <ChefHat className="h-4 w-4" />
                      配方列表
                    </CardTitle>
                    <CardDescription>共 {visibleRecipes.length} 个配方{recipeListFilter === "all" && filteredRecipes.length !== recipes.length ? `（筛选自 ${recipes.length} 个）` : ""}</CardDescription>
                  </div>
                  <Tabs value={recipeListFilter} onValueChange={setRecipeListFilter} className="w-auto">
                    <TabsList>
                      <TabsTrigger value="all">全部</TabsTrigger>
                      <TabsTrigger value="pending">待完善 {pendingRecipes.length}</TabsTrigger>
                    </TabsList>
                  </Tabs>
                </div>
              </CardHeader>
              <CardContent>
                {visibleRecipes.length === 0 ? (
                  <EmptyState
                    icon={ChefHat}
                    title={recipeListFilter === "pending" ? "暂无待完善配方" : "暂无配方"}
                    description={recipeListFilter === "pending" ? "当前所有配方都已有明细项" : "先创建一个配方，再补充材料、半成品和成本信息"}
                  />
                ) : (
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>代码</TableHead>
                        <TableHead>名称</TableHead>
                        <TableHead>类型</TableHead>
                        <TableHead>状态</TableHead>
                        <TableHead className="text-right">产出量</TableHead>
                        <TableHead className="text-right">操作</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {visibleRecipes.map((r) => (
                        <TableRow key={r.id}>
                          <TableCell className="font-mono text-xs">{r.code}</TableCell>
                          <TableCell className="font-medium">
                            {editRecipeId === r.id ? (
                              <div className="flex gap-2">
                                <Input value={editRecipeName} onChange={(e) => setEditRecipeName(e.target.value)} className="h-8" />
                                <Button size="sm" variant="ghost" className="h-8 w-8 p-0" onClick={saveEditRecipe}><Save className="h-4 w-4" /></Button>
                                <Button size="sm" variant="ghost" className="h-8 w-8 p-0" onClick={() => setEditRecipeId(null)}><X className="h-4 w-4" /></Button>
                              </div>
                            ) : (
                              <div className="flex items-center gap-2">
                                <span>{r.name}</span>
                                {recipeItemCounts[r.id] === 0 && (
                                  <Badge variant="secondary">待完善</Badge>
                                )}
                              </div>
                            )}
                          </TableCell>
                          <TableCell><Badge variant="outline">{recipeTypeMap[r.recipe_type]?.name || r.recipe_type}</Badge></TableCell>
                          <TableCell><Badge variant={getRecipeStatus(r).badgeVariant}>{getRecipeStatus(r).label}</Badge></TableCell>
                          <TableCell className="text-right">{r.output_qty}</TableCell>
                          <TableCell className="text-right">
                            <div className="flex justify-end gap-1">
                              {recipeItemCounts[r.id] === 0 && (
                                <Button variant="outline" size="sm" onClick={() => handleGuideRecipe(r)}>开始完善</Button>
                              )}
                              {getRecipeStatus(r).code === "ready" && (
                                <Button variant="outline" size="sm" onClick={() => goToMenuBinding(r)}>去绑定菜单</Button>
                              )}
                              <Button variant="ghost" size="sm" onClick={() => handleSelectRecipe(r)}>查看/编辑</Button>
                              <Button variant="ghost" size="icon" className="h-8 w-8 text-blue-500" title="查看依赖" onClick={() => setDependsDialog({ recipeId: r.id, recipeName: r.name })}><Network className="h-4 w-4" /></Button>
                              <Button variant="ghost" size="icon" className="h-8 w-8 text-destructive" onClick={() => setDeleteRecipeConfirm(r.id)}><Trash2 className="h-4 w-4" /></Button>
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
                <CardHeader>
                  <CardTitle>新增配方</CardTitle>
                  <CardDescription>先定类型，再补充原料、子配方和成本</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <Button variant="outline" className="w-full" onClick={handleSeedSamples} disabled={seedingSamples}>
                    {seedingSamples ? "创建中..." : "创建示例配方"}
                  </Button>
                  <div className="space-y-2">
                    <Label>配方编号</Label>
                    <div className="flex gap-2">
                      <Input value={newRecipeCode} readOnly className="bg-muted" />
                      <Button variant="outline" size="icon" onClick={() => invoke<string>("generate_recipe_code").then(setNewRecipeCode).catch(console.error)} title="重新生成">
                        <Plus className="h-4 w-4" />
                      </Button>
                    </div>
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="recipe-name">配方名称</Label>
                    <Input id="recipe-name" value={newRecipeName} onChange={(e) => setNewRecipeName(e.target.value)} placeholder="输入配方名称..." />
                  </div>
                  <div className="space-y-2">
                    <Label>配方类型</Label>
                    <Select value={newRecipeType} onValueChange={setNewRecipeType}>
                      <SelectTrigger><SelectValue placeholder="选择配方类型" /></SelectTrigger>
                      <SelectContent>
                        {recipeTypes.map((type) => (
                          <SelectItem key={type.id} value={type.code}>{type.name}</SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <Button className="w-full" onClick={handleCreateRecipeClick} disabled={creatingRecipe || !newRecipeName.trim()}>
                    <Plus className="mr-2 h-4 w-4" />{creatingRecipe ? "创建中..." : "新增配方"}
                  </Button>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>路线说明</CardTitle>
                  <CardDescription>建议按这条业务路径建设数据</CardDescription>
                </CardHeader>
                <CardContent className="space-y-3 text-sm text-muted-foreground">
                  <div className="rounded-lg border p-3">
                    1. 先注册材料，确认基础单位、分类和最小库存。
                  </div>
                  <div className="rounded-lg border p-3">
                    2. 再录入批次和成本，让配方有可靠的成本基线。
                  </div>
                  <div className="rounded-lg border p-3">
                    3. 用半成品配方沉淀预制物，再组合成成品配方并绑定菜单。
                  </div>
                </CardContent>
              </Card>
            </div>
          </div>

          {pendingMenuItems.length > 0 && (
            <Card className="mt-6">
              <CardHeader>
                <CardTitle>待完善菜单产品</CardTitle>
                <CardDescription>这些菜单产品还没有绑定配方，可以一键加入配方清单并标记为待完善</CardDescription>
              </CardHeader>
              <CardContent>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>菜单名称</TableHead>
                      <TableHead className="text-right">售价</TableHead>
                      <TableHead>状态</TableHead>
                      <TableHead className="text-right">操作</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {pendingMenuItems.map((item) => (
                      <TableRow key={item.id}>
                        <TableCell className="font-medium">{item.name}</TableCell>
                        <TableCell className="text-right">¥{item.sales_price.toFixed(2)}</TableCell>
                        <TableCell>
                          <Badge variant="secondary">待完善</Badge>
                        </TableCell>
                        <TableCell className="text-right">
                          <Button variant="outline" size="sm" onClick={() => onCreatePendingRecipeForMenu(item.id, item.name)}>
                            加入配方清单
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </CardContent>
            </Card>
          )}
        </TabsContent>

        <TabsContent value="editor" className="mt-6">
          {activeRecipe?.recipe ? (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center justify-between">
              {editRecipeId === activeRecipe.recipe.id ? (
                <div className="flex items-center gap-2 flex-1">
                  <Input value={editRecipeName} onChange={(e) => setEditRecipeName(e.target.value)} className="h-8 max-w-[200px]" />
                  <Select value={editRecipeType} onValueChange={setEditRecipeType}>
                    <SelectTrigger className="h-8 w-40"><SelectValue placeholder="选择类型" /></SelectTrigger>
                    <SelectContent>
                      {recipeTypes.map((type) => (
                        <SelectItem key={type.id} value={type.code}>{type.name}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <span className="text-muted-foreground">产出量:</span>
                  <Input type="number" value={editRecipeOutputQty} onChange={(e) => setEditRecipeOutputQty(e.target.value)} className="h-8 w-20" />
                  <Button size="sm" onClick={saveEditRecipe}><Save className="h-4 w-4 mr-1" />保存</Button>
                  <Button size="sm" variant="ghost" onClick={() => setEditRecipeId(null)}><X className="h-4 w-4" /></Button>
                </div>
              ) : (
                <div className="flex items-center gap-2">
                  <span>配方详情: {activeRecipe.recipe.name}</span>
                  <Button variant="ghost" size="sm" onClick={() => openEditRecipe(activeRecipe.recipe)}>
                    <Pencil className="h-4 w-4 mr-1" />编辑
                  </Button>
                </div>
              )}
              <Button variant="outline" size="sm" onClick={() => onRecalculateCost(activeRecipe.recipe.id)}>重新计算成本</Button>
            </CardTitle>
            <CardDescription>代码: {activeRecipe.recipe.code} | 类型: {recipeTypeMap[activeRecipe.recipe.recipe_type]?.name || activeRecipe.recipe.recipe_type} | 产出: {activeRecipe.recipe.output_qty}</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="mb-4 flex items-center justify-between rounded-lg border bg-muted/20 p-4">
              <div>
                <p className="text-sm font-medium">当前流程状态</p>
                <p className="mt-1 text-sm text-muted-foreground">{getRecipeStatus(activeRecipe.recipe).description}</p>
              </div>
              <div className="flex items-center gap-2">
                <Badge variant={getRecipeStatus(activeRecipe.recipe).badgeVariant}>{getRecipeStatus(activeRecipe.recipe).label}</Badge>
                {getRecipeStatus(activeRecipe.recipe).code === "ready" && (
                  <Button variant="outline" size="sm" onClick={() => goToMenuBinding(activeRecipe.recipe)}>去绑定菜单</Button>
                )}
              </div>
            </div>
            <div className="mb-6 grid gap-3 md:grid-cols-4">
              <div className="rounded-lg border bg-muted/20 p-3">
                <p className="text-xs text-muted-foreground">直接原物料</p>
                <p className="mt-1 text-lg font-semibold">{directMaterialCount}</p>
              </div>
              <div className="rounded-lg border bg-muted/20 p-3">
                <p className="text-xs text-muted-foreground">子配方</p>
                <p className="mt-1 text-lg font-semibold">{subRecipeCount}</p>
              </div>
              <div className="rounded-lg border bg-muted/20 p-3">
                <p className="text-xs text-muted-foreground">绑定菜单</p>
                <p className="mt-1 text-lg font-semibold">{boundMenuItems.length}</p>
              </div>
              <div className="rounded-lg border bg-muted/20 p-3">
                <p className="text-xs text-muted-foreground">被其他配方引用</p>
                <p className="mt-1 text-lg font-semibold">{selectedRecipeUsageCount ?? "-"}</p>
              </div>
            </div>
            <div className="grid gap-6 md:grid-cols-2">
              <div>
                <div className="flex items-center justify-between mb-3">
                  <h3 className="font-medium">配方明细 ({activeRecipe.items.length} 项)</h3>
                  <div className="flex gap-2">
                    <Button variant="outline" size="sm" onClick={() => openQuickAdd(activeRecipe.recipe.id)}>
                      <Plus className="h-3 w-3 mr-1" />新增配方项
                    </Button>
                  </div>
                </div>
                <p className="text-xs text-muted-foreground mb-3">一个配方由原物料与子配方构成；菜单只绑定配方，不直接绑定原物料。点击“用量”或“损耗率”可直接编辑数值。</p>
                {activeRecipe.items.length === 0 ? (
                  <EmptyState icon={ChefHat} title="暂无明细" description="点击添加材料开始" />
                ) : (
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead className="w-8"></TableHead>
                        <TableHead>原料 / 半成品</TableHead>
                        <TableHead className="text-right w-32 cursor-help" title="点击可编辑">用量 ↗</TableHead>
                        <TableHead className="text-right w-24 cursor-help" title="点击可编辑">损耗率 ↗</TableHead>
                        <TableHead className="text-right w-20">操作</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {activeRecipe.items.map((item) => {
                        const hasSubItems = item.item_type === "sub_recipe";
                        const isExpanded = expandedItems.has(item.ref_id);
                        return (
                          <Fragment key={item.id}>
                            <TableRow>
                              <TableCell className="p-2">
                                {hasSubItems && (
                                  <Button variant="ghost" size="icon" className="h-6 w-6" onClick={() => toggleExpand(item.ref_id)}>
                                    {isExpanded ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
                                  </Button>
                                )}
                              </TableCell>
                              <TableCell className="font-medium">
                                {item.item_type === "sub_recipe" && <Badge variant="outline" className="mr-1 text-xs">半成品</Badge>}
                                {getRefName(item.item_type, item.ref_id)}
                              </TableCell>
                              {editingItemId === item.id ? (
                                <>
                                  <TableCell className="text-right">
                                    <div className="flex items-center justify-end gap-1">
                                      <Input className="h-7 w-20 text-right" type="number" value={editingItemQty} onChange={(e) => setEditingItemQty(e.target.value)} onKeyDown={(e) => e.key === "Enter" && saveEditItem()} />
                                      <span className="text-xs text-muted-foreground">{getUnitName(item.unit_id)}</span>
                                    </div>
                                  </TableCell>
                                  <TableCell className="text-right">
                                    <div className="flex items-center justify-end gap-1">
                                      <Input className="h-7 w-16 text-right" type="number" value={editingItemWastage} onChange={(e) => setEditingItemWastage(e.target.value)} onKeyDown={(e) => e.key === "Enter" && saveEditItem()} />
                                      <span className="text-xs text-muted-foreground">%</span>
                                    </div>
                                  </TableCell>
                                  <TableCell className="text-right">
                                    <div className="flex justify-end gap-1">
                                      <Button variant="ghost" size="icon" className="h-7 w-7" onClick={saveEditItem}><Save className="h-3.5 w-3.5" /></Button>
                                      <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => setEditingItemId(null)}><X className="h-3.5 w-3.5" /></Button>
                                    </div>
                                  </TableCell>
                                </>
                              ) : (
                                <>
                                  <TableCell className="text-right cursor-pointer hover:bg-muted" onClick={() => startEditItem(item)}>
                                    {item.qty} {getUnitName(item.unit_id)}
                                  </TableCell>
                                  <TableCell className="text-right cursor-pointer hover:bg-muted" onClick={() => startEditItem(item)}>
                                    {(item.wastage_rate * 100).toFixed(1)}%
                                  </TableCell>
                                  <TableCell className="text-right">
                                    <div className="flex justify-end gap-1">
                                      <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => startEditItem(item)}><Pencil className="h-3.5 w-3.5" /></Button>
                                      <Button variant="ghost" size="icon" className="h-7 w-7 text-destructive" onClick={() => setDeleteItemConfirm(item.id)}><Trash2 className="h-3.5 w-3.5" /></Button>
                                    </div>
                                  </TableCell>
                                </>
                              )}
                            </TableRow>
                            {hasSubItems && isExpanded && (
                              <TableRow className="bg-muted/30">
                                <TableCell colSpan={5} className="p-0 pl-10 border-l-[3px] border-l-primary/30">
                                  <div className="py-2 pr-4 bg-muted/10">
                                    {renderSubItems(item.ref_id)}
                                  </div>
                                </TableCell>
                              </TableRow>
                            )}
                          </Fragment>
                        );
                      })}
                    </TableBody>
                  </Table>
                )}
                {quickAddRecipeId === activeRecipe.recipe.id && (
                  <div className="flex items-center gap-2 mt-2 p-2 border rounded-md bg-muted/30">
                    <Select value={quickAddMaterial} onValueChange={setQuickAddMaterial}>
                      <SelectTrigger className="flex-1"><SelectValue placeholder="选择材料 / 半成品" /></SelectTrigger>
                      <SelectContent className="max-h-64">
                        <SelectGroup>
                          <SelectLabel>原材料</SelectLabel>
                          {materials.map((m) => <SelectItem key={`m_${m.id}`} value={`m_${m.id}`}>{m.name}</SelectItem>)}
                        </SelectGroup>
                        <SelectGroup>
                          <SelectLabel>半成品（子配方）</SelectLabel>
                          {recipes.filter((r) => r.id !== quickAddRecipeId).map((r) => <SelectItem key={`r_${r.id}`} value={`r_${r.id}`}>{r.name}</SelectItem>)}
                        </SelectGroup>
                      </SelectContent>
                    </Select>
                    <Input className="w-24" type="number" placeholder="用量" value={quickAddQty} onChange={(e) => setQuickAddQty(e.target.value)} onKeyDown={(e) => e.key === "Enter" && saveQuickAdd()} />
                    <Select value={quickAddUnit} onValueChange={setQuickAddUnit}>
                      <SelectTrigger className="w-24"><SelectValue placeholder="单位" /></SelectTrigger>
                      <SelectContent>
                        {quickAddCompatibleUnits.map((u) => <SelectItem key={u.id} value={u.id.toString()}>{u.name}</SelectItem>)}
                      </SelectContent>
                    </Select>
                    <Input className="w-16" type="number" placeholder="0" value={quickAddWastage} onChange={(e) => setQuickAddWastage(e.target.value)} onKeyDown={(e) => e.key === "Enter" && saveQuickAdd()} />
                    <span className="text-xs text-muted-foreground">%</span>
                    <Button size="sm" onClick={saveQuickAdd} disabled={!quickAddMaterial || !quickAddQty || !quickAddUnit}><Save className="h-4 w-4" /></Button>
                    <Button size="sm" variant="ghost" onClick={() => setQuickAddRecipeId(null)}><X className="h-4 w-4" /></Button>
                  </div>
                )}
                {boundMenuItems.length > 0 && (
                  <div className="mt-4 rounded-lg border border-emerald-200 bg-emerald-50/60 p-3">
                    <p className="text-sm font-medium text-emerald-800">已绑定菜单</p>
                    <div className="mt-2 flex flex-wrap gap-2">
                      {boundMenuItems.map((item) => (
                        <Badge key={item.id} variant="outline" className="border-emerald-300 text-emerald-700">
                          {item.name}
                        </Badge>
                      ))}
                    </div>
                  </div>
                )}
                {boundMenuItems.length === 0 && recipeItemCounts[activeRecipe.recipe.id] > 0 && (
                  <div className="mt-4 rounded-lg border border-amber-200 bg-amber-50/70 p-3">
                    <p className="text-sm font-medium text-amber-800">下一步建议</p>
                    <p className="mt-1 text-sm text-amber-700">这个配方已经有结构，但还没有绑定菜单商品。</p>
                    <div className="mt-3 flex flex-wrap gap-2">
                      <Button variant="outline" size="sm" onClick={() => goToMenuBinding(activeRecipe.recipe)}>去绑定菜单</Button>
                      {pendingMenuItems.some((item) => item.name === activeRecipe.recipe.name) && (
                        <Button
                          size="sm"
                          onClick={() => {
                            const sameNameItem = pendingMenuItems.find((item) => item.name === activeRecipe.recipe.name);
                            if (sameNameItem) {
                              onBindMenuItemToRecipe(sameNameItem.id, activeRecipe.recipe.id);
                            }
                          }}
                        >
                          绑定同名菜单
                        </Button>
                      )}
                    </div>
                  </div>
                )}
                {getRecipeStatus(activeRecipe.recipe).code === "review" && (
                  <div className="mt-4 rounded-lg border border-blue-200 bg-blue-50/70 p-3">
                    <p className="text-sm font-medium text-blue-800">成本补充建议</p>
                    <p className="mt-1 text-sm text-blue-700">这个配方已经有明细，但批次成本还不完整，建议先补库存批次和单位成本。</p>
                    <div className="mt-3 flex flex-wrap gap-2">
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => {
                          const firstMaterial = activeRecipe.items.find((item) => item.item_type === "material");
                          goToInventoryCost(firstMaterial?.ref_id);
                        }}
                      >
                        去补批次成本
                      </Button>
                    </div>
                  </div>
                )}
              </div>
              {recipeCost && (
                <div>
                  <h3 className="font-medium mb-3">成本计算</h3>
                  <div className="space-y-3">
                    <div className="flex items-center justify-between rounded-lg border p-4">
                      <span className="text-sm text-muted-foreground">总成本</span>
                      <span className="text-lg font-semibold">¥{recipeCost.total_cost.toFixed(2)}</span>
                    </div>
                    <div className="flex items-center justify-between rounded-lg border p-4">
                      <span className="text-sm text-muted-foreground">单位成本</span>
                      <span className="text-lg font-semibold">¥{recipeCost.cost_per_unit.toFixed(2)}</span>
                    </div>
                    <div className="flex items-center justify-between rounded-lg border p-4">
                      <span className="text-sm text-muted-foreground">产出量</span>
                      <span className="text-lg font-semibold">{recipeCost.output_qty}</span>
                    </div>
                    <Separator />
                    {recipeCost.items && recipeCost.items.length > 0 && (
                      <div>
                        <h4 className="text-sm font-medium mb-2">成本明细</h4>
                        <div className="space-y-2">
                          {recipeCost.items.map((ci, i) => {
                            const pct = recipeCost.total_cost > 0 ? (ci.line_cost / recipeCost.total_cost) * 100 : 0;
                            const barColor = pct > 50 ? "bg-red-500" : pct > 20 ? "bg-amber-500" : "bg-green-500";
                            const isSubRecipe = ci.item_type === "sub_recipe";
                            return (
                              <div key={i}>
                                <div className="flex items-center justify-between text-sm py-1 gap-1">
                                  <div className="flex items-center gap-1 flex-1 min-w-0">
                                    {isSubRecipe && <Badge variant="outline" className="text-xs shrink-0">半成品</Badge>}
                                    <span className="text-muted-foreground truncate">{ci.material_name}</span>
                                  </div>
                                  <span className="font-mono shrink-0">¥{ci.line_cost.toFixed(2)} ({pct.toFixed(1)}%)</span>
                                </div>
                                <div className="h-1.5 w-full bg-muted rounded-full overflow-hidden">
                                  <div className={`h-full ${isSubRecipe ? "bg-purple-500" : barColor} transition-all`} style={{ width: `${pct}%` }} />
                                </div>
                              </div>
                            );
                          })}
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>
          </CardContent>
        </Card>
          ) : (
            <Card>
              <CardContent className="pt-6">
                <EmptyState icon={ChefHat} title="尚未选择配方" description="先在“配方列表”中选择一个配方，再查看明细、成本和绑定关系" />
              </CardContent>
            </Card>
          )}
        </TabsContent>

        <TabsContent value="types" className="mt-6">
          <div className="grid gap-6 lg:grid-cols-3">
            <Card className="lg:col-span-2">
              <CardHeader>
                <CardTitle>配方类型</CardTitle>
                <CardDescription>管理配方的业务分类，供创建与编辑时选择</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid gap-2 md:grid-cols-3">
                  <Input value={newRecipeTypeCode} onChange={(e) => setNewRecipeTypeCode(e.target.value)} placeholder="代码，如：combo" />
                  <Input value={newRecipeTypeName} onChange={(e) => setNewRecipeTypeName(e.target.value)} placeholder="名称，如：套餐配方" />
                  <Input value={newRecipeTypeDescription} onChange={(e) => setNewRecipeTypeDescription(e.target.value)} placeholder="说明（可选）" />
                </div>
                <Button variant="outline" onClick={handleCreateRecipeTypeClick}>
                  <Plus className="mr-2 h-4 w-4" />新增配方类型
                </Button>
                <Separator />
                <div className="space-y-2">
                  {recipeTypes.map((type) => (
                    <div key={type.id} className="rounded-md border p-3">
                      {editingRecipeTypeId === type.id ? (
                        <div className="space-y-2">
                          <Input value={editingRecipeTypeCode} onChange={(e) => setEditingRecipeTypeCode(e.target.value)} />
                          <Input value={editingRecipeTypeName} onChange={(e) => setEditingRecipeTypeName(e.target.value)} />
                          <Input value={editingRecipeTypeDescription} onChange={(e) => setEditingRecipeTypeDescription(e.target.value)} />
                          <div className="flex gap-2">
                            <Button size="sm" onClick={saveRecipeTypeEdit}>保存</Button>
                            <Button size="sm" variant="ghost" onClick={() => setEditingRecipeTypeId(null)}>取消</Button>
                          </div>
                        </div>
                      ) : (
                        <div className="flex items-start justify-between gap-3">
                          <div>
                            <div className="flex items-center gap-2">
                              <span className="font-medium">{type.name}</span>
                              <Badge variant="outline">{type.code}</Badge>
                              {type.is_system && <Badge variant="secondary">系统</Badge>}
                            </div>
                            {type.description && <p className="mt-1 text-xs text-muted-foreground">{type.description}</p>}
                          </div>
                          <div className="flex gap-1">
                            <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => openEditRecipeType(type)}><Pencil className="h-3.5 w-3.5" /></Button>
                            <Button variant="ghost" size="icon" className="h-7 w-7 text-destructive" onClick={() => setDeleteRecipeTypeConfirm(type.id)} disabled={type.is_system}><Trash2 className="h-3.5 w-3.5" /></Button>
                          </div>
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>当前审计重点</CardTitle>
                <CardDescription>这几项会继续在后续迭代中补齐</CardDescription>
              </CardHeader>
              <CardContent className="space-y-3 text-sm text-muted-foreground">
                <div className="rounded-lg border p-3">缺少面向“材料 → 半成品 → 成品 → 菜单”的引导式创建流程。</div>
                <div className="rounded-lg border p-3">批量编辑、批量删除和依赖预览仍不一致，需要统一设计。</div>
                <div className="rounded-lg border p-3">半成品产出物、产出单位、状态映射还需要进一步建模和落地 UI。</div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>



      <Dialog open={!!deleteItemConfirm} onOpenChange={() => setDeleteItemConfirm(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>确认删除</DialogTitle>
          </DialogHeader>
          <p className="py-4 text-sm text-muted-foreground">确定要删除此配方材料项吗？</p>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteItemConfirm(null)}>取消</Button>
            <Button variant="destructive" onClick={() => { if (deleteItemConfirm && activeRecipe?.recipe) { onDeleteRecipeItem(deleteItemConfirm, activeRecipe.recipe.id); } setDeleteItemConfirm(null); }}>删除</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={!!deleteRecipeConfirm} onOpenChange={() => setDeleteRecipeConfirm(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>确认删除配方</DialogTitle>
          </DialogHeader>
          <p className="py-4 text-sm text-muted-foreground">确定要删除此配方吗？此操作不可撤销，将同时删除所有关联的配方明细。</p>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteRecipeConfirm(null)}>取消</Button>
            <Button variant="destructive" onClick={() => { if (deleteRecipeConfirm) { onDeleteRecipe(deleteRecipeConfirm); } setDeleteRecipeConfirm(null); }}>删除</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={!!deleteRecipeTypeConfirm} onOpenChange={() => setDeleteRecipeTypeConfirm(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>确认删除配方类型</DialogTitle>
          </DialogHeader>
          <p className="py-4 text-sm text-muted-foreground">删除后该类型将不再可选；若仍有配方使用它，系统会阻止删除。</p>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteRecipeTypeConfirm(null)}>取消</Button>
            <Button variant="destructive" onClick={() => {
              if (deleteRecipeTypeConfirm) {
                onDeleteRecipeType(deleteRecipeTypeConfirm);
              }
              setDeleteRecipeTypeConfirm(null);
            }}>删除</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={!!warningDialog} onOpenChange={() => setWarningDialog(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>操作确认</DialogTitle></DialogHeader>
          <p className="py-4 text-sm text-muted-foreground">{warningDialog?.message}</p>
          <DialogFooter>
            <Button variant="outline" onClick={() => setWarningDialog(null)}>取消</Button>
            <Button onClick={() => { warningDialog?.onConfirm(); setWarningDialog(null); }}>确定继续</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={!!dependsDialog} onOpenChange={() => setDependsDialog(null)}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Network className="h-4 w-4 text-blue-500" />
              依赖视图 — {dependsDialog?.recipeName}
            </DialogTitle>
          </DialogHeader>
          <div className="py-4 space-y-4">
            <div>
              <p className="text-sm font-medium mb-2">被哪些菜单商品使用</p>
              {!dependents ? (
                <p className="text-sm text-muted-foreground">加载中…</p>
              ) : dependents.menu_items.length === 0 ? (
                <p className="text-sm text-muted-foreground">暂无菜单商品引用此配方</p>
              ) : (
                <ul className="space-y-1">
                  {dependents.menu_items.map((item) => (
                    <li key={item.id} className="text-sm flex items-center gap-2">
                      <Badge variant="secondary" className="text-xs">菜单</Badge>
                      {item.name}
                    </li>
                  ))}
                </ul>
              )}
            </div>
            <Separator />
            <div>
              <p className="text-sm font-medium mb-2">被哪些配方作为子配方引用</p>
              {!dependents ? (
                <p className="text-sm text-muted-foreground">加载中…</p>
              ) : dependents.parent_recipes.length === 0 ? (
                <p className="text-sm text-muted-foreground">暂无配方将此配方作为半成品引用</p>
              ) : (
                <ul className="space-y-1">
                  {dependents.parent_recipes.map((r) => (
                    <li key={r.id} className="text-sm flex items-center gap-2">
                      <Badge variant="outline" className="text-xs">配方</Badge>
                      {r.name}
                    </li>
                  ))}
                </ul>
              )}
            </div>
            {dependents && dependents.menu_items.length === 0 && dependents.parent_recipes.length === 0 && (
              <p className="text-xs text-muted-foreground mt-2">该配方未被任何菜单或其他配方引用，可安全删除。</p>
            )}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDependsDialog(null)}>关闭</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
