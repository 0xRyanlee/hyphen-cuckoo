import { call as invoke } from "@/lib/transport";
import { appLogger, formatError, sanitizeSensitiveValue } from "@/lib/logger";
import type {
  MaterialCategory, RecipeCostResult, MenuItem, MenuCategory, POSCartItem, MenuItemSpec,
  TicketWithItems, Order, OrderWithItems, OrderItemModifier, Recipe, RecipeWithItems,
  PurchaseOrderWithItems, ProductionOrderWithItems, StocktakeWithItems, Material, Supplier, Unit, InventoryBatch, Expense, SupplierProduct, KitchenStation
} from "../types";
import { toast } from "sonner";

export interface UseAppActionsParams {
  loadMaterials: () => Promise<void>;
  loadRecipes: () => Promise<void>;
  loadMenu: () => Promise<void>;
  loadOrders: (limit?: number, offset?: number) => Promise<void>;
  loadKDS: () => Promise<void>;
  loadInventory: () => Promise<void>;
  loadPurchaseOrders: () => Promise<void>;
  loadProductionOrders: () => Promise<void>;
  loadStocktakes: () => Promise<void>;
  loadSuppliers: () => Promise<void>;
  loadMaterialStates: () => Promise<void>;
  loadExpenses: () => Promise<void>;
  loadSupplierProducts: () => Promise<void>;
  loadCustomers: () => Promise<void>;
  categories: MaterialCategory[];
  menuCategories: MenuCategory[];
  orders: Order[];
  materials: Material[];
  suppliers: Supplier[];
  units: Unit[];
  inventoryBatches: InventoryBatch[];
  menuItems: MenuItem[];
  setOrders: React.Dispatch<React.SetStateAction<Order[]>>;
  setOrdersHasMore: React.Dispatch<React.SetStateAction<boolean>>;
  setSelectedRecipe: React.Dispatch<React.SetStateAction<RecipeWithItems | null>>;
  setRecipeCost: React.Dispatch<React.SetStateAction<RecipeCostResult | null>>;
  setSelectedOrder: React.Dispatch<React.SetStateAction<OrderWithItems | null>>;
  setKdsTickets: React.Dispatch<React.SetStateAction<TicketWithItems[]>>;
  setSelectedPurchaseOrder: React.Dispatch<React.SetStateAction<PurchaseOrderWithItems | null>>;
  setSelectedProductionOrder: React.Dispatch<React.SetStateAction<ProductionOrderWithItems | null>>;
  setSelectedStocktake: React.Dispatch<React.SetStateAction<StocktakeWithItems | null>>;
  setExpenses: React.Dispatch<React.SetStateAction<Expense[]>>;
  setSupplierProducts: React.Dispatch<React.SetStateAction<SupplierProduct[]>>;
  stations: KitchenStation[];
}

export function useAppActions({
  loadMaterials,
  loadRecipes,
  loadMenu,
  loadOrders,
  loadKDS,
  loadInventory,
  loadPurchaseOrders,
  loadProductionOrders,
  loadStocktakes,
  loadSuppliers,
  loadMaterialStates,
  loadExpenses,
  loadSupplierProducts,
  loadCustomers,
  categories,
  menuCategories,
  orders,
  materials,
  suppliers,
  units,
  inventoryBatches,
  menuItems,
  setOrders,
  setOrdersHasMore,
  setSelectedRecipe,
  setRecipeCost,
  setSelectedOrder,
  setKdsTickets,
  setSelectedPurchaseOrder,
  setSelectedProductionOrder,
  setSelectedStocktake,
  setExpenses: _setExpenses,
  setSupplierProducts: _setSupplierProducts,
  stations,
}: UseAppActionsParams) {
  function normalizeRecipeWithItems(raw: unknown): RecipeWithItems {
    if (raw && typeof raw === "object" && "recipe" in raw && "items" in raw) {
      return raw as RecipeWithItems;
    }

    if (raw && typeof raw === "object" && "items" in raw) {
      const flattened = raw as Record<string, unknown>;
      const { items, ...recipe } = flattened;
      return {
        recipe: recipe as unknown as Recipe,
        items: Array.isArray(items) ? items as RecipeWithItems["items"] : [],
      };
    }

    throw new Error("Invalid recipe payload shape");
  }

  async function refreshRecipeSelection(recipeId: number) {
    const [rawData, cost] = await Promise.all([
      invoke<unknown>("get_recipe_with_items", { recipeId }),
      invoke<RecipeCostResult>("calculate_recipe_cost", { recipeId }),
    ]);
    const data = normalizeRecipeWithItems(rawData);
    setSelectedRecipe(data);
    setRecipeCost(cost);
  }

  // Single helper: logs + toasts every action error.
  // operation = Tauri command name (or compound name for multi-step flows).
  function logError(operation: string, raw: unknown, label: string, ctx?: Record<string, unknown>) {
    appLogger.logInvokeError(operation, raw, ctx);
    toast.error(label, { description: formatError(raw) });
  }

  // 材料相關
  const handleCreateMaterial = async (data: { code: string; name: string; base_unit_id: number; category_id: number | null; tag_ids: number[] }) => {
    try { await invoke("create_material", { req: { code: data.code, name: data.name, base_unit_id: data.base_unit_id, category_id: data.category_id, shelf_life_days: null, tag_ids: data.tag_ids } }); toast.success("材料已创建", { description: data.name }); loadMaterials(); }
    catch (e) { logError("create_material", e, "创建材料失败", { name: data.name }); }
  };

  const handleUpdateMaterial = async (id: number, data: { name?: string; category_id?: number | null; min_qty?: number }) => {
    try { await invoke("update_material", { id, name: data.name || null, categoryId: data.category_id, shelfLifeDays: null, minQty: data.min_qty ?? null }); toast.success("材料已更新"); loadMaterials(); }
    catch (e) { logError("update_material", e, "更新材料失败", { id }); }
  };

  const handleDeleteMaterial = async (id: number) => {
    try { await invoke("delete_material", { id }); toast.success("材料已删除"); loadMaterials(); }
    catch (e) { logError("delete_material", e, "删除材料失败", { id }); }
  };

  const handleRemoveMaterialTag = async (material_id: number, tag_id: number) => {
    try { await invoke("remove_material_tag", { materialId: material_id, tagId: tag_id }); loadMaterials(); }
    catch (e) { logError("remove_material_tag", e, "移除标签失败", { material_id, tag_id }); }
  };

  // 分類與標籤
  const handleCreateCategory = async (data: { code: string; name: string }) => {
    try { await invoke("create_material_category", { req: { ...data, sort_no: categories.length + 1 } }); toast.success("分类已创建", { description: data.name }); loadMaterials(); }
    catch (e) { logError("create_material_category", e, "创建分类失败", { name: data.name }); }
  };

  const handleDeleteCategory = async (id: number) => {
    try { await invoke("delete_material_category", { id }); toast.success("分类已删除"); loadMaterials(); }
    catch (e) { logError("delete_material_category", e, "删除分类失败", { id }); }
  };

  const handleCreateTag = async (data: { code: string; name: string; color?: string }) => {
    try { await invoke("create_tag", { req: { ...data, color: data.color || null } }); toast.success("标签已创建", { description: data.name }); loadMaterials(); }
    catch (e) { logError("create_tag", e, "创建标签失败", { name: data.name }); }
  };

  const handleDeleteTag = async (id: number) => {
    try { await invoke("delete_tag", { id }); toast.success("标签已删除"); loadMaterials(); }
    catch (e) { logError("delete_tag", e, "删除标签失败", { id }); }
  };

  // 配方
  const handleCreateRecipe = async (data: { code: string; name: string; recipe_type: string; output_material_id?: number | null; output_unit_id?: number | null }): Promise<number | null> => {
    try {
      const newId = await invoke<number>("create_recipe", { req: { ...data, output_qty: 1.0, output_material_id: data.output_material_id ?? null, output_state_id: null, output_unit_id: data.output_unit_id ?? null, items: null } });
      toast.success("配方已创建", { description: data.name });
      await loadRecipes();
      return newId;
    }
    catch (e) { logError("create_recipe", e, "创建配方失败", { name: data.name }); return null; }
  };

  const handleViewRecipe = async (recipe: Recipe) => {
    try {
      await refreshRecipeSelection(recipe.id);
    } catch (e) { logError("get_recipe_with_items", e, "加载配方失败", { recipe_id: recipe.id }); }
  };

  const handleDeleteRecipe = async (id: number) => {
    try {
      const [usageCount, dependents] = await Promise.all([
        invoke<number>("get_recipe_usage_count", { recipeId: id }),
        invoke<{ menu_items: Array<{ id: number; name: string }>; parent_recipes: Array<{ id: number; name: string }> }>("get_recipe_dependents", { recipeId: id }),
      ]);

      if (usageCount > 0 || dependents.menu_items.length > 0) {
        const reasons: string[] = [];
        if (usageCount > 0) reasons.push(`${usageCount} 个子配方引用`);
        if (dependents.menu_items.length > 0) reasons.push(`${dependents.menu_items.length} 个菜单商品绑定`);
        toast.error(`此配方仍被 ${reasons.join("、")} 引用，无法删除`);
        return;
      }

      await invoke("delete_recipe", { id });
      toast.success("配方已删除");
      setSelectedRecipe(null);
      setRecipeCost(null);
      loadRecipes();
    }
    catch (e) { logError("delete_recipe", e, "删除配方失败", { id }); }
  };

  const handleUpdateRecipe = async (id: number, data: { name: string; recipe_type: string; output_qty: number }) => {
    try {
      await invoke("update_recipe", { id, name: data.name, recipeType: data.recipe_type, outputQty: data.output_qty });
      await Promise.all([loadRecipes(), refreshRecipeSelection(id)]);
      toast.success("配方已更新");
    }
    catch (e) { logError("update_recipe", e, "更新配方失败", { id }); }
  };

  const handleCreateRecipeType = async (data: { code: string; name: string; description?: string | null; sort_no?: number }) => {
    try {
      await invoke("create_recipe_type", { req: data });
      toast.success("配方类型已创建", { description: data.name });
      await loadRecipes();
    } catch (e) { logError("create_recipe_type", e, "创建配方类型失败", { code: data.code }); }
  };

  const handleUpdateRecipeType = async (id: number, data: { code: string; name: string; description?: string | null; sort_no: number }) => {
    try {
      await invoke("update_recipe_type", { id, code: data.code, name: data.name, description: data.description ?? null, sortNo: data.sort_no });
      toast.success("配方类型已更新");
      await loadRecipes();
    } catch (e) { logError("update_recipe_type", e, "更新配方类型失败", { id }); }
  };

  const handleDeleteRecipeType = async (id: number) => {
    try {
      await invoke("delete_recipe_type", { id });
      toast.success("配方类型已删除");
      await loadRecipes();
    } catch (e) { logError("delete_recipe_type", e, "删除配方类型失败", { id }); }
  };

  const handleSeedSampleRecipes = async () => {
    try {
      await invoke("seed_sample_recipes");
      setSelectedRecipe(null);
      setRecipeCost(null);
      await loadRecipes();
      toast.success("示例配方已创建");
    } catch (e) { logError("seed_sample_recipes", e, "创建示例配方失败"); }
  };

  const handleCreatePendingRecipeForMenu = async (menuItemId: number, menuItemName: string): Promise<number | null> => {
    try {
      const code = await invoke<string>("generate_recipe_code");
      const recipeId = await invoke<number>("create_recipe", {
        req: {
          code,
          name: menuItemName.trim(),
          recipe_type: "menu",
          output_qty: 1.0,
          output_material_id: null,
          output_state_id: null,
          output_unit_id: null,
          items: null,
        },
      });
      await invoke("update_menu_item", {
        id: menuItemId,
        name: null,
        categoryId: null,
        recipeId,
        salesPrice: null,
      });
      await loadMenu();
      toast.success("已加入配方清单", { description: `${menuItemName} 已标记为待完善` });
      return recipeId;
    } catch (e) { logError("create_pending_recipe_for_menu", e, "加入配方清单失败", { menu_item_id: menuItemId }); return null; }
  };

  const handleBindMenuItemToRecipe = async (menuItemId: number, recipeId: number) => {
    try {
      await invoke("update_menu_item", {
        id: menuItemId,
        name: null,
        categoryId: null,
        recipeId,
        salesPrice: null,
      });
      await loadMenu();
      toast.success("菜单已绑定配方");
    } catch (e) { logError("bind_menu_item_to_recipe", e, "绑定菜单失败", { menu_item_id: menuItemId, recipe_id: recipeId }); }
  };

  const handleAddRecipeItem = async (recipe_id: number, item_type: string, ref_id: number, qty: number, unit_id: number, wastage_rate: number) => {
    try {
      if (item_type === "sub_recipe") {
        const wouldCreateCycle = await invoke<boolean>("would_create_recipe_cycle", { recipeId: recipe_id, refId: ref_id });
        if (wouldCreateCycle) {
          toast.error("添加此子配方会产生循环引用，请先调整配方嵌套关系");
          return;
        }
      }
      await invoke("add_recipe_item", { recipeId: recipe_id, req: { item_type, ref_id, qty, unit_id, wastage_rate } });
      await refreshRecipeSelection(recipe_id);
      toast.success("配方项已添加");
    }
    catch (e) { logError("add_recipe_item", e, "添加配方项失败", { recipe_id, ref_id }); }
  };

  const handleDeleteRecipeItem = async (item_id: number, recipe_id: number) => {
    try {
      await invoke("delete_recipe_item", { itemId: item_id });
      await refreshRecipeSelection(recipe_id);
      toast.success("配方项已删除");
    }
    catch (e) { logError("delete_recipe_item", e, "删除配方项失败", { item_id }); }
  };

  const handleUpdateRecipeItem = async (item_id: number, recipe_id: number, qty: number, wastage_rate: number) => {
    try {
      await invoke("update_recipe_item", { itemId: item_id, qty, wastageRate: wastage_rate });
      await refreshRecipeSelection(recipe_id);
      toast.success("配方项已更新");
    }
    catch (e) { logError("update_recipe_item", e, "更新配方项失败", { item_id }); }
  };

  const handleRecalculateCost = async (id: number) => {
    try { const cost = await invoke<RecipeCostResult>("calculate_recipe_cost", { recipeId: id }); setRecipeCost(cost); }
    catch (e) { logError("calculate_recipe_cost", e, "计算成本失败", { recipe_id: id }); }
  };

  // 菜單
  const handleCreateMenuCategory = async (name: string) => {
    try { await invoke("create_menu_category", { name, sortNo: menuCategories.length + 1 }); toast.success("菜单分类已创建", { description: name }); loadMenu(); }
    catch (e) { logError("create_menu_category", e, "创建菜单分类失败", { name }); }
  };

  const handleCreateMenuItem = async (data: { name: string; price: number; category_id: number | null; recipe_id: number | null }) => {
    try { await invoke("create_menu_item", { req: { name: data.name, category_id: data.category_id, recipe_id: data.recipe_id, sales_price: data.price } }); toast.success("菜品已添加", { description: data.name }); loadMenu(); }
    catch (e) { logError("create_menu_item", e, "添加菜品失败", { name: data.name }); }
  };

  const handleToggleMenuItem = async (id: number, is_available: boolean) => {
    try { await invoke("set_menu_item_availability", { id, isAvailable: is_available }); loadMenu(); }
    catch (e) { logError("set_menu_item_availability", e, "切换菜品状态失败", { id, is_available }); }
  };

  const handleBatchToggleMenuItem = async (ids: number[], is_available: boolean) => {
    try { const count = await invoke<number>("batch_set_menu_item_availability", { ids, isAvailable: is_available }); toast.success(`已批量切换 ${count} 个菜品`); loadMenu(); }
    catch (e) { logError("batch_set_menu_item_availability", e, "批量切换失败", { count: ids.length }); }
  };

  const handleBatchUpdateMenuItemPrices = async (ids: number[], mode: "set" | "delta" | "percent", value: number) => {
    try {
      const count = await invoke<number>("batch_update_menu_item_prices", { ids, mode, value });
      toast.success(`已批量更新 ${count} 个菜品价格`);
      loadMenu();
    }
    catch (e) { logError("batch_update_menu_item_prices", e, "批量调价失败", { count: ids.length, mode, value }); }
  };

  const handleToggleFavorite = async (id: number) => {
    try { const now = await invoke<boolean>("toggle_menu_item_favorite", { id }); toast.success(now ? "已加入常用" : "已移出常用"); loadMenu(); }
    catch (e) { logError("toggle_menu_item_favorite", e, "切换常用失败", { id }); }
  };

  const handleUpdateMenuItem = async (id: number, data: { name?: string; category_id?: number | null; recipe_id?: number | null; sales_price?: number }) => {
    try { await invoke("update_menu_item", { id, name: data.name || null, categoryId: data.category_id, recipeId: data.recipe_id, salesPrice: data.sales_price }); toast.success("菜品已更新"); loadMenu(); }
    catch (e) { logError("update_menu_item", e, "更新菜品失败", { id }); }
  };

  const handleDeleteMenuItem = async (id: number) => {
    try { await invoke("delete_menu_item", { id }); toast.success("菜品已删除"); loadMenu(); }
    catch (e) { logError("delete_menu_item", e, "删除菜品失败", { id }); }
  };

  const handleUpdateMenuCategory = async (id: number, name: string) => {
    try { await invoke("update_menu_category", { id, name, sortNo: null }); toast.success("分类已更新"); loadMenu(); }
    catch (e) { logError("update_menu_category", e, "更新分类失败", { id }); }
  };

  const handleDeleteMenuCategory = async (id: number) => {
    try { await invoke("delete_menu_category", { id }); toast.success("分类已删除"); loadMenu(); }
    catch (e) { logError("delete_menu_category", e, "删除分类失败", { id }); }
  };

  // 訂單
  const handleCreateOrder = async (dineType: string = "dine_in", tableNo: string | null = null) => {
    try {
      const { order_no } = await invoke<{ id: number; order_no: string }>("create_order", { req: { source: "pos", dine_type: dineType, table_no: tableNo } });
      toast.success(`订单 ${order_no} 已创建`);
      loadOrders();
    }
    catch (e) { logError("create_order", e, "创建订单失败", { dineType, tableNo }); }
  };

  const printTicket = async (ticket: TicketWithItems, dineType: string) => {
    const printItems: [string, number, string | null][] = ticket.items.map((item) => [
      menuItems.find((m) => m.id === item.menu_item_id)?.name || `商品 #${item.menu_item_id}`,
      item.qty,
      item.note || null,
    ]);
    const dineLabel = dineType === "dine_in" ? "堂食" : dineType === "takeout" ? "外卖" : "外送";
    const stationPrinterId = stations.find((s) => s.id === ticket.station_id)?.printer_id ?? null;
    await invoke("print_kitchen_ticket", { orderNo: ticket.order_no, dineType: dineLabel, items: printItems, note: null, printerId: stationPrinterId });
  };

  const handlePrintReceipt = async (order_id: number) => {
    try {
      await invoke("print_order_receipt", { orderId: order_id, printerId: null });
      toast.success("收据已发送打印");
    } catch (e) { logError("print_order_receipt", e, "打印收据失败", { order_id }); }
  };

  const handlePOSAndSubmit = async (cart: POSCartItem[], dineType: string = "dine_in", tableNo: string | null = null) => {
    try {
      const { id: orderId } = await invoke<{ id: number; order_no: string }>("create_order", { req: { source: "pos", dine_type: dineType, table_no: tableNo } });
      for (const item of cart) {
        const orderItemId = await invoke<number>("add_order_item", {
          req: { order_id: orderId, menu_item_id: item.menu_item.id, qty: item.qty, unit_price: item.menu_item.sales_price + (item.spec?.price_delta || 0), spec_code: item.spec?.spec_code || null, note: item.note || null },
        });
        if (item.modifiers?.length) {
          for (const mod of item.modifiers) {
            await invoke("add_order_item_modifier", { req: { order_item_id: orderItemId, modifier_type: mod.modifier_type, material_id: mod.material_id || null, qty: mod.qty, price_delta: mod.price_delta } });
          }
        }
      }
      await invoke("submit_order", { orderId });
      toast.success("订单已提交");
      if (localStorage.getItem("auto_print_kitchen") === "true") {
        try {
          const tickets = await invoke<TicketWithItems[]>("get_tickets_for_order", { orderId });
          for (const ticket of tickets) await printTicket(ticket, dineType);
        } catch (pe) {
          appLogger.logInvokeError("auto_print_kitchen_ticket", pe, { orderId });
        }
      }
      await Promise.all([loadOrders(), loadKDS()]);
      return true;
    } catch (e) { logError("pos_create_and_submit", e, "提交订单失败", { cart_size: cart.length, dineType }); return false; }
  };

  const handleReprintTicket = async (ticket: TicketWithItems) => {
    try {
      await printTicket(ticket, ticket.dine_type);
      toast.success("补打印已发送");
    } catch (e) { logError("reprint_kitchen_ticket", e, "补打印失败", { ticket_id: ticket.id }); }
  };

  const handlePOSOrder = async (cart: POSCartItem[], dineType: string = "dine_in", tableNo: string | null = null) => {
    try {
      const { id: orderId } = await invoke<{ id: number; order_no: string }>("create_order", { req: { source: "pos", dine_type: dineType, table_no: tableNo } });
      for (const item of cart) {
        await invoke("add_order_item", {
          req: { order_id: orderId, menu_item_id: item.menu_item.id, qty: item.qty, unit_price: item.menu_item.sales_price + (item.spec?.price_delta || 0), spec_code: item.spec?.spec_code || null, note: item.note || null },
        });
      }
      toast.success("订单已创建");
      loadOrders();
      return true;
    } catch (e) { logError("pos_create_order", e, "创建订单失败", { cart_size: cart.length, dineType }); return false; }
  };

  const handleSubmitOrder = async (orderId: number) => {
    try { await invoke("submit_order", { orderId }); toast.success("订单已提交"); await Promise.all([loadOrders(), loadKDS()]); }
    catch (e) { logError("submit_order", e, "提交订单失败", { orderId }); }
  };

  const handleCancelOrder = async (orderId: number, is_served: boolean = false, reason: string = "") => {
    try {
      await invoke("cancel_order", { orderId, isServed: is_served, reason: reason || null });
      toast.success("订单已取消");
      loadOrders();
    } catch (e) { logError("cancel_order", e, "取消订单失败", { orderId, is_served }); }
  };

  const handleMarkOrderReady = async (orderId: number) => {
    try { await invoke("mark_order_ready", { orderId }); toast.success("订单已标记出餐"); await Promise.all([loadOrders(), loadKDS(), loadInventory()]); }
    catch (e) { logError("mark_order_ready", e, "标记出餐失败", { orderId }); }
  };

  const handleViewOrder = async (orderId: number) => {
    try { const data = await invoke<OrderWithItems>("get_order_with_items", { orderId }); setSelectedOrder(data); }
    catch (e) { logError("get_order_with_items", e, "加载订单失败", { orderId }); }
  };

  const handleLoadMoreOrders = async () => {
    try {
      const more = await invoke<Order[]>("get_orders", { limit: 200, offset: orders.length });
      setOrders((prev) => [...prev, ...more]);
      setOrdersHasMore(more.length === 200);
    } catch (e) { logError("get_orders_paginated", e, "加载订单失败", { offset: orders.length }); }
  };

  const handleBatchCancelOrder = async (ids: number[]) => {
    try { const count = await invoke<number>("batch_cancel_orders", { ids }); toast.success(`已取消 ${count} 个订单`); await Promise.all([loadOrders(), loadInventory()]); }
    catch (e) { logError("batch_cancel_orders", e, "批量取消失败", { count: ids.length }); }
  };

  // 規格
  const handleGetSpecs = async (menuItemId: number): Promise<MenuItemSpec[]> => {
    return await invoke<MenuItemSpec[]>("get_menu_item_specs", { menuItemId });
  };

  const handleCreateSpec = async (data: { menu_item_id: number; spec_code: string; spec_name: string; price_delta: number; qty_multiplier: number }) => {
    try { await invoke("create_menu_item_spec", { req: data }); toast.success("规格已创建"); await loadMenu(); }
    catch (e) { logError("create_menu_item_spec", e, "创建规格失败", { menu_item_id: data.menu_item_id }); }
  };

  const handleUpdateSpec = async (id: number, data: { spec_code?: string; spec_name?: string; price_delta?: number; qty_multiplier?: number }) => {
    try { await invoke("update_menu_item_spec", { id, specCode: data.spec_code || null, specName: data.spec_name || null, priceDelta: data.price_delta, qtyMultiplier: data.qty_multiplier }); toast.success("规格已更新"); await loadMenu(); }
    catch (e) { logError("update_menu_item_spec", e, "更新规格失败", { id }); }
  };

  const handleDeleteSpec = async (id: number) => {
    try { await invoke("delete_menu_item_spec", { id }); toast.success("规格已删除"); await loadMenu(); }
    catch (e) { logError("delete_menu_item_spec", e, "删除规格失败", { id }); }
  };

  // 供應商
  const handleCreateSupplier = async (data: { name: string; phone?: string; contact_person?: string }) => {
    try { await invoke("create_supplier", { req: { name: data.name, phone: data.phone || null, contact_person: data.contact_person || null } }); toast.success("供应商已创建", { description: data.name }); loadSuppliers(); }
    catch (e) { logError("create_supplier", e, "创建供应商失败", { name: data.name }); }
  };

  const handleUpdateSupplier = async (id: number, data: { name?: string; phone?: string | null; contact_person?: string | null; address?: string | null; note?: string | null }) => {
    try { await invoke("update_supplier", { id, name: data.name || null, phone: data.phone, contactPerson: data.contact_person, address: data.address, note: data.note }); toast.success("供应商已更新"); loadSuppliers(); }
    catch (e) { logError("update_supplier", e, "更新供应商失败", { id }); }
  };

  const handleDeleteSupplier = async (id: number) => {
    try { await invoke("delete_supplier", { id }); toast.success("供应商已删除"); loadSuppliers(); }
    catch (e) { logError("delete_supplier", e, "删除供应商失败", { id }); }
  };

  // 材料狀態
  const handleCreateMaterialState = async (data: { material_id: number; state_code: string; state_name: string; unit_id: number | null; yield_rate: number; cost_multiplier: number }) => {
    try { await invoke("create_material_state", { req: data }); toast.success("材料状态已创建"); await Promise.all([loadMaterials(), loadMaterialStates()]); }
    catch (e) { logError("create_material_state", e, "创建材料状态失败", { material_id: data.material_id }); }
  };

  const handleUpdateMaterialState = async (id: number, data: { state_code?: string; state_name?: string; unit_id?: number | null; yield_rate?: number; cost_multiplier?: number }) => {
    try { await invoke("update_material_state", { id, stateCode: data.state_code || null, stateName: data.state_name || null, unitId: data.unit_id, yieldRate: data.yield_rate, costMultiplier: data.cost_multiplier }); toast.success("材料状态已更新"); await Promise.all([loadMaterials(), loadMaterialStates()]); }
    catch (e) { logError("update_material_state", e, "更新材料状态失败", { id }); }
  };

  const handleDeleteMaterialState = async (id: number) => {
    try { await invoke("delete_material_state", { id }); toast.success("材料状态已删除"); await Promise.all([loadMaterials(), loadMaterialStates()]); }
    catch (e) { logError("delete_material_state", e, "删除材料状态失败", { id }); }
  };

  // 採購單
  const handleCreatePurchaseOrder = async (data: { supplier_id: number | null; expected_date: string | null }) => {
    try { await invoke("create_purchase_order", { supplierId: data.supplier_id, expectedDate: data.expected_date }); toast.success("采购单已创建"); loadPurchaseOrders(); }
    catch (e) { logError("create_purchase_order", e, "创建采购单失败", { supplier_id: data.supplier_id }); }
  };

  const handleAddPurchaseOrderItem = async (data: { po_id: number; material_id: number; qty: number; unit_id: number | null; cost_per_unit: number }) => {
    try { await invoke("add_purchase_order_item", { req: data }); toast.success("采购项已添加"); loadPurchaseOrders(); }
    catch (e) { logError("add_purchase_order_item", e, "添加采购项失败", { po_id: data.po_id, material_id: data.material_id }); }
  };

  const handleViewPurchaseOrder = async (po_id: number) => {
    try { setSelectedPurchaseOrder(await invoke<PurchaseOrderWithItems>("get_purchase_order_with_items", { poId: po_id })); }
    catch (e) { logError("get_purchase_order_with_items", e, "加载采购单失败", { po_id }); }
  };

  const handleDeletePurchaseOrder = async (po_id: number) => {
    try { await invoke("delete_purchase_order", { poId: po_id }); toast.success("采购单已删除"); loadPurchaseOrders(); setSelectedPurchaseOrder(null); }
    catch (e) { logError("delete_purchase_order", e, "删除采购单失败", { po_id }); }
  };

  const handleReceivePurchaseOrder = async (po_id: number) => {
    const autoPrint = localStorage.getItem("auto_print_po") === "true";
    try { await invoke("receive_purchase_order", { poId: po_id, operator: null, autoPrint }); toast.success("采购单已入库"); await Promise.all([loadPurchaseOrders(), loadInventory()]); }
    catch (e) { logError("receive_purchase_order", e, "入库失败", { po_id }); }
  };

  const handleReceivePurchaseOrderItems = async (po_id: number, items: { item_id: number; received_qty: number; lot_no: string | null }[]) => {
    try {
      const result = await invoke<string[]>("receive_purchase_order_items", { poId: po_id, items, operator: null });
      toast.success(`已入库 ${result.length} 项`, { description: result.join("，") || undefined });
      loadOrders();
    } catch (e) { logError("receive_purchase_order_items", e, "部分收货失败", { po_id }); }
  };

  const handleUpdateOrderPayment = async (id: number, payment_status: string, payment_method: string | null, amount_paid: number) => {
    try {
      await invoke("update_order_payment", { req: { order_id: id, payment_status, payment_method, amount_paid } });
      toast.success("收款已登记");
      if (localStorage.getItem("auto_print_receipt") === "true") {
        try { await invoke("print_order_receipt", { orderId: id, printerId: null }); }
        catch (pe) { appLogger.logInvokeError("auto_print_receipt", pe, { orderId: id }); }
      }
      loadOrders();
    } catch (e) { logError("update_order_payment", e, "收款登记失败", { id }); }
  };

  // 生產單
  const handleCreateProductionOrder = async (data: { recipe_id: number; planned_qty: number; operator: string | null }) => {
    try { await invoke("create_production_order", { recipeId: data.recipe_id, plannedQty: data.planned_qty, operator: data.operator }); toast.success("生产单已创建"); loadProductionOrders(); }
    catch (e) { logError("create_production_order", e, "创建生产单失败", { recipe_id: data.recipe_id }); }
  };

  const handleStartProductionOrder = async (production_id: number) => {
    try { await invoke("start_production_order", { productionId: production_id, operator: null }); toast.success("生产已开始"); loadProductionOrders(); }
    catch (e) { logError("start_production_order", e, "开始生产失败", { production_id }); }
  };

  const handleCompleteProductionOrder = async (production_id: number, actual_qty: number) => {
    try { await invoke("complete_production_order", { productionId: production_id, actualQty: actual_qty, operator: null }); toast.success("生产已完成"); await Promise.all([loadProductionOrders(), loadInventory()]); }
    catch (e) { logError("complete_production_order", e, "完成生产失败", { production_id, actual_qty }); }
  };

  const handleViewProductionOrder = async (production_id: number) => {
    try { setSelectedProductionOrder(await invoke<ProductionOrderWithItems>("get_production_order_with_items", { productionId: production_id })); }
    catch (e) { logError("get_production_order_with_items", e, "加载生产单失败", { production_id }); }
  };

  const handleDeleteProductionOrder = async (production_id: number) => {
    try { await invoke("delete_production_order", { productionId: production_id }); toast.success("生产单已删除"); loadProductionOrders(); setSelectedProductionOrder(null); }
    catch (e) { logError("delete_production_order", e, "删除生产单失败", { production_id }); }
  };

  // 盤點
  const handleCreateStocktake = async (data: { operator: string | null; note: string | null }) => {
    try { await invoke("create_stocktake", { operator: data.operator, note: data.note }); toast.success("盘点已创建"); loadStocktakes(); }
    catch (e) { logError("create_stocktake", e, "创建盘点失败"); }
  };

  const handleUpdateStocktakeItem = async (item_id: number, actual_qty: number) => {
    try { await invoke("update_stocktake_item", { itemId: item_id, actualQty: actual_qty }); loadStocktakes(); }
    catch (e) { logError("update_stocktake_item", e, "更新盘点项失败", { item_id, actual_qty }); }
  };

  const handleCompleteStocktake = async (stocktake_id: number) => {
    try { await invoke("complete_stocktake", { stocktakeId: stocktake_id, operator: null }); toast.success("盘点已完成"); await Promise.all([loadStocktakes(), loadInventory()]); }
    catch (e) { logError("complete_stocktake", e, "完成盘点失败", { stocktake_id }); }
  };

  const handleViewStocktake = async (stocktake_id: number) => {
    try { setSelectedStocktake(await invoke<StocktakeWithItems>("get_stocktake_with_items", { stocktakeId: stocktake_id })); }
    catch (e) { logError("get_stocktake_with_items", e, "加载盘点失败", { stocktake_id }); }
  };

  const handleDeleteStocktake = async (stocktake_id: number) => {
    try { await invoke("delete_stocktake", { stocktakeId: stocktake_id }); toast.success("盘点已删除"); loadStocktakes(); setSelectedStocktake(null); }
    catch (e) { logError("delete_stocktake", e, "删除盘点失败", { stocktake_id }); }
  };

  // 日常支出
  const handleCreateExpense = async (data: { expense_type: string; amount: number; expense_date: string; note: string }) => {
    try {
      await invoke("create_expense", { req: { expense_type: data.expense_type, amount: data.amount, expense_date: data.expense_date, note: data.note || null, operator: null } });
      toast.success("支出已记录", { description: `¥${data.amount}` });
      loadExpenses();
    } catch (e) { logError("create_expense", e, "记录支出失败", { expense_type: data.expense_type }); }
  };

  const handleUpdateExpense = async (id: number, data: { expense_type?: string; amount?: number; expense_date?: string; note?: string }) => {
    try { await invoke("update_expense", { id, expenseType: data.expense_type || null, amount: data.amount ?? null, expenseDate: data.expense_date || null, note: data.note || null }); toast.success("支出已更新"); loadExpenses(); }
    catch (e) { logError("update_expense", e, "更新支出失败", { id }); }
  };

  const handleDeleteExpense = async (id: number) => {
    try { await invoke("delete_expense", { id }); toast.success("支出已删除"); loadExpenses(); }
    catch (e) { logError("delete_expense", e, "删除支出失败", { id }); }
  };

  const handleCreateSupplierProduct = async (data: { product_name: string; supplier_name: string; channel: string }) => {
    try {
      await invoke("create_supplier_product", { req: data });
      toast.success("商品已添加");
      loadSupplierProducts();
    } catch (e) { logError("create_supplier_product", e, "添加商品失败", { product_name: data.product_name }); }
  };

  const handleUpdateSupplierProduct = async (id: number, data: { product_name: string; supplier_name: string; channel: string }) => {
    try {
      await invoke("update_supplier_product", { req: { id, ...data } });
      toast.success("商品已更新");
      loadSupplierProducts();
    } catch (e) { logError("update_supplier_product", e, "更新商品失败", { id }); }
  };

  const handleDeleteSupplierProduct = async (id: number) => {
    try { await invoke("delete_supplier_product", { id }); toast.success("商品已删除"); loadSupplierProducts(); }
    catch (e) { logError("delete_supplier_product", e, "删除商品失败", { id }); }
  };

  // 批次
  const handleCreateBatch = async (data: { material_id: number; lot_no: string; quantity: number; cost_per_unit: number; supplier_id: number | null; expiry_date: string | null; production_date: string | null; ice_coating_rate?: number | null; quality_rate?: number | null; seasonal_factor?: number }) => {
    try {
      await invoke("create_inventory_batch", { req: { material_id: data.material_id, state_id: null, lot_no: data.lot_no, supplier_id: data.supplier_id, brand: null, spec: null, quantity: data.quantity, cost_per_unit: data.cost_per_unit, production_date: data.production_date, expiry_date: data.expiry_date, ice_coating_rate: data.ice_coating_rate ?? null, quality_rate: data.quality_rate ?? null, seasonal_factor: data.seasonal_factor ?? 1.0 } });
      toast.success("批次已创建", { description: data.lot_no });
      const mat = materials.find((m) => m.id === data.material_id);
      const sup = suppliers.find((s) => s.id === data.supplier_id);
      const unitCode = mat?.base_unit?.code || units.find((u) => u.id === mat?.base_unit_id)?.code || "";
      try {
        await invoke("print_batch_label", {
          lotNo: data.lot_no,
          materialName: mat?.name || `材料 #${data.material_id}`,
          quantity: data.quantity,
          unit: unitCode,
          expiryDate: data.expiry_date,
          supplierName: sup?.name || null,
          printerId: null,
        });
      } catch (pe) {
        appLogger.logInvokeError("print_batch_label", pe, { lot_no: data.lot_no });
      }
      loadInventory();
    } catch (e) { logError("create_inventory_batch", e, "创建批次失败", { lot_no: data.lot_no, material_id: data.material_id }); }
  };

  const handleAdjustInventory = async (lot_id: number, qty_delta: number, reason: string) => {
    try {
      const batch = inventoryBatches.find((b) => b.id === lot_id);
      if (!batch) return;
      await invoke("adjust_inventory", { req: { material_id: batch.material_id, lot_id, qty_delta, reason, operator: null, note: null } });
      toast.success("库存已调整");
      loadInventory();
    } catch (e) { logError("adjust_inventory", e, "调整库存失败", { lot_id, qty_delta, reason }); }
  };

  const handleRecordWastage = async (lot_id: number, qty: number, wastage_type: string) => {
    try {
      const batch = inventoryBatches.find((b) => b.id === lot_id);
      if (!batch) return;
      await invoke("record_wastage", { req: { material_id: batch.material_id, lot_id, qty, wastage_type, operator: null, note: null } });
      toast.success("废弃已记录");
      loadInventory();
    } catch (e) { logError("record_wastage", e, "记录废弃失败", { lot_id, qty, wastage_type }); }
  };

  const handleDeleteBatch = async (batch_id: number) => {
    try { await invoke("delete_inventory_batch", { batchId: batch_id }); toast.success("批次已删除"); loadInventory(); }
    catch (e) { logError("delete_inventory_batch", e, "删除批次失败", { batch_id }); }
  };

  // KDS
  const handleLoadKDS = async () => {
    try {
      const pendingTickets = await invoke<TicketWithItems[]>("get_all_tickets_with_items", { status: "pending" });
      const startedTickets = await invoke<TicketWithItems[]>("get_all_tickets_with_items", { status: "started" });
      setKdsTickets([...pendingTickets, ...startedTickets]);
    } catch (e) { logError("get_all_tickets_with_items", e, "加载KDS失败"); }
  };

  const handleFinishTicket = async (ticket: TicketWithItems) => {
    try {
      await invoke("finish_ticket", { ticketId: ticket.id, operator: null });
      toast.success("工单已完成");
      try {
        const printItems: [string, number, string | null][] = ticket.items.map((item) => [
          menuItems.find((m) => m.id === item.menu_item_id)?.name || `菜品 #${item.menu_item_id}`,
          item.qty,
          item.note || null,
        ]);
        await invoke("print_kitchen_ticket", {
          orderNo: ticket.order_no,
          dineType: ticket.dine_type === "dine_in" ? "堂食" : ticket.dine_type === "takeout" ? "外卖" : "外送",
          items: printItems,
          note: null,
          printerId: null,
        });
      } catch (pe) {
        appLogger.logInvokeError("print_kitchen_ticket", pe, { ticket_id: ticket.id, order_no: ticket.order_no });
      }
      loadKDS();
    } catch (e) { logError("finish_ticket", e, "完成工单失败", { ticket_id: ticket.id }); }
  };

  // 訂單修改器
  const handleAddModifier = async (data: { order_item_id: number; modifier_type: string; material_id: number | null; qty: number; price_delta: number }) => {
    try { await invoke("add_order_item_modifier", { req: data }); toast.success("加料已添加"); loadOrders(); }
    catch (e) { logError("add_order_item_modifier", e, "添加加料失败", { order_item_id: data.order_item_id }); }
  };

  const handleDeleteModifier = async (modifier_id: number) => {
    try { await invoke("delete_order_item_modifier", { modifierId: modifier_id }); toast.success("加料已删除"); loadOrders(); }
    catch (e) { logError("delete_order_item_modifier", e, "删除加料失败", { modifier_id }); }
  };

  const handleLoadModifiers = async (order_item_id: number): Promise<OrderItemModifier[]> => {
    return await invoke<OrderItemModifier[]>("get_order_item_modifiers", { orderItemId: order_item_id });
  };

  const handleReportTelemetry = async (payload: { client_id: string; version: string; event_type: string; uptime_hours: number; today_sales: number; today_orders: number; metadata: any }) => {
    try {
      await invoke("report_telemetry", {
        payload: {
          ...payload,
          metadata: sanitizeSensitiveValue(payload.metadata),
        },
      });
    }
    catch { /* telemetry is best-effort; do not pollute the user-visible error log */ }
  };

  const handleRefundOrderItem = async (orderId: number, itemId: number): Promise<number> => {
    const refundedAmount = await invoke<number>("refund_order_item", { orderId, itemId });
    await loadCustomers();
    return refundedAmount;
  };

  const handleCreateCustomer = async (name: string, phone: string | null) => {
    try {
      await invoke("create_customer", { name, phone });
      toast.success("顾客已添加");
      await loadCustomers();
    } catch (e) { toast.error(formatError(e)); }
  };

  const handleUpdateCustomer = async (id: number, name: string | null, phone: string | null, clearPhone: boolean) => {
    try {
      await invoke("update_customer", { id, name, phone, clearPhone });
      toast.success("顾客信息已更新");
      await loadCustomers();
    } catch (e) { toast.error(formatError(e)); }
  };

  const handleDeleteCustomer = async (id: number) => {
    try {
      await invoke("delete_customer", { id });
      toast.success("顾客已删除");
      await loadCustomers();
    } catch (e) { toast.error(formatError(e)); }
  };

  const handleAddLoyaltyPoints = async (customerId: number, orderId: number | null, delta: number, reason: string): Promise<number> => {
    const newPoints = await invoke<number>("add_loyalty_points", { customerId, orderId, delta, reason });
    await loadCustomers();
    return newPoints;
  };

  return {
    // 材料
    handleCreateMaterial, handleUpdateMaterial, handleDeleteMaterial, handleRemoveMaterialTag,
    // 分類與標籤
    handleCreateCategory, handleDeleteCategory, handleCreateTag, handleDeleteTag,
    // 配方
    handleCreateRecipe, handleViewRecipe, handleDeleteRecipe, handleUpdateRecipe, handleCreateRecipeType, handleUpdateRecipeType, handleDeleteRecipeType, handleSeedSampleRecipes, handleCreatePendingRecipeForMenu, handleBindMenuItemToRecipe, handleAddRecipeItem, handleDeleteRecipeItem, handleUpdateRecipeItem, handleRecalculateCost,
    // 菜單
    handleCreateMenuCategory, handleUpdateMenuCategory, handleDeleteMenuCategory, handleCreateMenuItem, handleToggleMenuItem, handleBatchToggleMenuItem, handleBatchUpdateMenuItemPrices, handleToggleFavorite, handleUpdateMenuItem, handleDeleteMenuItem,
    // 訂單
    handleCreateOrder, handlePOSOrder, handlePOSAndSubmit, handleSubmitOrder, handleCancelOrder, handleMarkOrderReady, handleBatchCancelOrder, handleViewOrder, handleLoadMoreOrders,
    // 規格
    handleGetSpecs, handleCreateSpec, handleUpdateSpec, handleDeleteSpec,
    // 供應商
    handleCreateSupplier, handleUpdateSupplier, handleDeleteSupplier,
    // 材料狀態
    handleCreateMaterialState, handleUpdateMaterialState, handleDeleteMaterialState,
    // 採購單
    handleCreatePurchaseOrder, handleAddPurchaseOrderItem, handleViewPurchaseOrder, handleDeletePurchaseOrder, handleReceivePurchaseOrder, handleReceivePurchaseOrderItems,
    // 收款
    handleUpdateOrderPayment,
    // 生產單
    handleCreateProductionOrder, handleStartProductionOrder, handleCompleteProductionOrder, handleViewProductionOrder, handleDeleteProductionOrder,
    // 盤點
    handleCreateStocktake, handleUpdateStocktakeItem, handleCompleteStocktake, handleViewStocktake, handleDeleteStocktake,
    // 日常支出
    handleCreateExpense, handleUpdateExpense, handleDeleteExpense,
    // 供應商商品
    handleCreateSupplierProduct, handleUpdateSupplierProduct, handleDeleteSupplierProduct,
    // 批次
    handleCreateBatch, handleAdjustInventory, handleRecordWastage, handleDeleteBatch,
    // KDS
    handleLoadKDS, handleFinishTicket, handleReprintTicket, handlePrintReceipt,
    // 訂單修改器
    handleAddModifier, handleDeleteModifier, handleLoadModifiers,
    // 遙測
    handleReportTelemetry,
    // 单品退单
    handleRefundOrderItem,
    // 顾客
    handleCreateCustomer, handleUpdateCustomer, handleDeleteCustomer, handleAddLoyaltyPoints,
  };
}
