import { useState, useCallback } from "react";
import { call as invoke } from "@/lib/transport";
import { appLogger } from "@/lib/logger";
import type {
  Unit, MaterialCategory, TagItem, Material, Recipe, RecipeWithItems,
  RecipeCostResult, RecipeType, MenuItem, MenuCategory, Order, OrderWithItems, KitchenStation,
  TicketWithItems, InventoryBatch, InventorySummary, InventoryTxn, AttributeTemplate,
  Supplier, MaterialState, PurchaseOrder, PurchaseOrderWithItems,
  ProductionOrder, ProductionOrderWithItems, Stocktake, StocktakeWithItems, Expense, SupplierProduct,
  Customer
} from "../types";
import { usePartialLoadData } from "./usePartialLoadData";

// Wraps invoke so any failure is logged with the operation name before re-throwing.
// This lets the pipeline immediately identify which specific IPC call caused a black screen.
async function tracked<T>(operation: string, promise: Promise<T>): Promise<T> {
  try {
    return await promise;
  } catch (e) {
    appLogger.logInvokeError(operation, e);
    throw e;
  }
}

export function useAppData() {
  const [loading, setLoading] = useState(true);
  const [connected, setConnected] = useState(false);

  const [units, setUnits] = useState<Unit[]>([]);
  const [categories, setCategories] = useState<MaterialCategory[]>([]);
  const [tags, setTags] = useState<TagItem[]>([]);
  const [materials, setMaterials] = useState<Material[]>([]);
  const [recipes, setRecipes] = useState<Recipe[]>([]);
  const [recipeTypes, setRecipeTypes] = useState<RecipeType[]>([]);
  const [selectedRecipe, setSelectedRecipe] = useState<RecipeWithItems | null>(null);
  const [recipeCost, setRecipeCost] = useState<RecipeCostResult | null>(null);
  const [menuCategories, setMenuCategories] = useState<MenuCategory[]>([]);
  const [menuItems, setMenuItems] = useState<MenuItem[]>([]);
  const [orders, setOrders] = useState<Order[]>([]);
  const [ordersHasMore, setOrdersHasMore] = useState(false);
  const [selectedOrder, setSelectedOrder] = useState<OrderWithItems | null>(null);
  const [stations, setStations] = useState<KitchenStation[]>([]);
  const [kdsTickets, setKdsTickets] = useState<TicketWithItems[]>([]);
  const [inventoryBatches, setInventoryBatches] = useState<InventoryBatch[]>([]);
  const [inventorySummary, setInventorySummary] = useState<InventorySummary[]>([]);
  const [inventoryTxns, setInventoryTxns] = useState<InventoryTxn[]>([]);
  const [attributeTemplates, setAttributeTemplates] = useState<AttributeTemplate[]>([]);
  const [suppliers, setSuppliers] = useState<Supplier[]>([]);
  const [materialStates, setMaterialStates] = useState<MaterialState[]>([]);
  const [purchaseOrders, setPurchaseOrders] = useState<PurchaseOrder[]>([]);
  const [selectedPurchaseOrder, setSelectedPurchaseOrder] = useState<PurchaseOrderWithItems | null>(null);
  const [productionOrders, setProductionOrders] = useState<ProductionOrder[]>([]);
  const [selectedProductionOrder, setSelectedProductionOrder] = useState<ProductionOrderWithItems | null>(null);
  const [stocktakes, setStocktakes] = useState<Stocktake[]>([]);
  const [selectedStocktake, setSelectedStocktake] = useState<StocktakeWithItems | null>(null);
  const [expenses, setExpenses] = useState<Expense[]>([]);
  const [supplierProducts, setSupplierProducts] = useState<SupplierProduct[]>([]);
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [unreadNotificationCount, setUnreadNotificationCount] = useState(0);

  const partialLoaders = usePartialLoadData(
    {
      loading,
      connected,
      units,
      categories,
      tags,
      materials,
      recipes,
      recipeTypes,
      selectedRecipe,
      recipeCost,
      menuCategories,
      menuItems,
      orders,
      ordersHasMore,
      selectedOrder,
      stations,
      kdsTickets,
      inventoryBatches,
      inventorySummary,
      inventoryTxns,
      attributeTemplates,
      suppliers,
      materialStates,
      purchaseOrders,
      selectedPurchaseOrder,
      productionOrders,
      selectedProductionOrder,
      stocktakes,
      selectedStocktake,
      expenses,
      supplierProducts,
      customers,
    },
    {
      setUnits,
      setCategories,
      setTags,
      setMaterials,
      setRecipes,
      setRecipeTypes,
      setSelectedRecipe,
      setRecipeCost,
      setMenuCategories,
      setMenuItems,
      setOrders,
      setOrdersHasMore,
      setSelectedOrder,
      setStations,
      setKdsTickets,
      setInventoryBatches,
      setInventorySummary,
      setInventoryTxns,
      setAttributeTemplates,
      setSuppliers,
      setMaterialStates,
      setPurchaseOrders,
      setSelectedPurchaseOrder,
      setProductionOrders,
      setSelectedProductionOrder,
      setStocktakes,
      setSelectedStocktake,
      setExpenses,
      setSupplierProducts,
      setCustomers,
      setLoading,
      setConnected,
    },
  );

  const loadData = useCallback(async () => {
    setLoading(true);
    // Health check is the only thing that sets the connection state
    try {
      const result = await tracked("health_check", invoke<string>("health_check"));
      setConnected(result === "ok");
    } catch {
      setConnected(false);
      setLoading(false);
      return;
    }
    try { await invoke("check_and_create_alerts"); } catch { /* ignore */ }
    try { const count = await invoke<number>("get_unread_notification_count"); setUnreadNotificationCount(count); } catch { /* ignore */ }
    // Fetch all data in parallel — each independently so one failure doesn't block others
    const [
      unitsR, catsR, tagsR, matsR, recsR, recTypesR,
      menuCatsR, menuItemsR, ordersR, stationsR,
      batchesR, summaryR, txnsR, attrsR, suppliersR,
      statesR, posR, prodsR, stocktakesR, expensesR,
      supplierProdsR, pendingR, startedR, customersR,
    ] = await Promise.allSettled([
      tracked("get_units", invoke<Unit[]>("get_units")),
      tracked("get_material_categories", invoke<MaterialCategory[]>("get_material_categories")),
      tracked("get_tags", invoke<TagItem[]>("get_tags")),
      tracked("get_materials", invoke<Material[]>("get_materials")),
      tracked("get_recipes", invoke<Recipe[]>("get_recipes")),
      tracked("get_recipe_types", invoke<RecipeType[]>("get_recipe_types")),
      tracked("get_menu_categories", invoke<MenuCategory[]>("get_menu_categories")),
      tracked("get_menu_items", invoke<MenuItem[]>("get_menu_items")),
      tracked("get_orders", invoke<Order[]>("get_orders", { limit: 200, offset: 0 })),
      tracked("get_kitchen_stations", invoke<KitchenStation[]>("get_kitchen_stations")),
      tracked("get_inventory_batches", invoke<InventoryBatch[]>("get_inventory_batches")),
      tracked("get_inventory_summary", invoke<InventorySummary[]>("get_inventory_summary")),
      tracked("get_inventory_txns", invoke<InventoryTxn[]>("get_inventory_txns", { limit: 50 })),
      tracked("get_attribute_templates", invoke<AttributeTemplate[]>("get_attribute_templates")),
      tracked("get_suppliers", invoke<Supplier[]>("get_suppliers")),
      tracked("get_all_material_states", invoke<MaterialState[]>("get_all_material_states")),
      tracked("get_purchase_orders", invoke<PurchaseOrder[]>("get_purchase_orders")),
      tracked("get_production_orders", invoke<ProductionOrder[]>("get_production_orders")),
      tracked("get_stocktakes", invoke<Stocktake[]>("get_stocktakes")),
      tracked("get_expenses", invoke<Expense[]>("get_expenses", { expenseType: null, startDate: null, endDate: null })),
      tracked("get_supplier_products", invoke<SupplierProduct[]>("get_supplier_products", { channel: null })),
      tracked("get_tickets_pending", invoke<TicketWithItems[]>("get_all_tickets_with_items", { status: "pending" })),
      tracked("get_tickets_started", invoke<TicketWithItems[]>("get_all_tickets_with_items", { status: "started" })),
      tracked("get_customers", invoke<Customer[]>("get_customers", { search: null })),
    ]);
    if (unitsR.status === "fulfilled") setUnits(unitsR.value);
    if (catsR.status === "fulfilled") setCategories(catsR.value);
    if (tagsR.status === "fulfilled") setTags(tagsR.value);
    if (matsR.status === "fulfilled") setMaterials(matsR.value);
    if (recsR.status === "fulfilled") setRecipes(recsR.value);
    if (recTypesR.status === "fulfilled") setRecipeTypes(recTypesR.value);
    if (menuCatsR.status === "fulfilled") setMenuCategories(menuCatsR.value);
    if (menuItemsR.status === "fulfilled") setMenuItems(menuItemsR.value);
    if (ordersR.status === "fulfilled") {
      setOrders(ordersR.value);
      setOrdersHasMore(ordersR.value.length === 200);
    }
    if (stationsR.status === "fulfilled") setStations(stationsR.value);
    if (batchesR.status === "fulfilled") setInventoryBatches(batchesR.value);
    if (summaryR.status === "fulfilled") setInventorySummary(summaryR.value);
    if (txnsR.status === "fulfilled") setInventoryTxns(txnsR.value);
    if (attrsR.status === "fulfilled") setAttributeTemplates(attrsR.value);
    if (suppliersR.status === "fulfilled") setSuppliers(suppliersR.value);
    if (statesR.status === "fulfilled") setMaterialStates(statesR.value);
    if (posR.status === "fulfilled") setPurchaseOrders(posR.value);
    if (prodsR.status === "fulfilled") setProductionOrders(prodsR.value);
    if (stocktakesR.status === "fulfilled") setStocktakes(stocktakesR.value);
    if (expensesR.status === "fulfilled") setExpenses(expensesR.value);
    if (supplierProdsR.status === "fulfilled") setSupplierProducts(supplierProdsR.value);
    if (pendingR.status === "fulfilled" && startedR.status === "fulfilled") {
      setKdsTickets([...pendingR.value, ...startedR.value]);
    }
    if (customersR.status === "fulfilled") setCustomers(customersR.value);
    setLoading(false);
  }, []);

  return {
    loading, setLoading, connected,
    units, setUnits,
    categories, setCategories,
    tags, setTags,
    materials, setMaterials,
    recipes, setRecipes,
    recipeTypes, setRecipeTypes,
    selectedRecipe, setSelectedRecipe,
    recipeCost, setRecipeCost,
    menuCategories, setMenuCategories,
    menuItems, setMenuItems,
    orders, setOrders,
    ordersHasMore, setOrdersHasMore,
    selectedOrder, setSelectedOrder,
    stations, setStations,
    kdsTickets, setKdsTickets,
    inventoryBatches, setInventoryBatches,
    inventorySummary, setInventorySummary,
    inventoryTxns, setInventoryTxns,
    attributeTemplates, setAttributeTemplates,
    suppliers, setSuppliers,
    materialStates, setMaterialStates,
    purchaseOrders, setPurchaseOrders,
    selectedPurchaseOrder, setSelectedPurchaseOrder,
    productionOrders, setProductionOrders,
    selectedProductionOrder, setSelectedProductionOrder,
    stocktakes, setStocktakes,
    selectedStocktake, setSelectedStocktake,
    expenses, setExpenses,
    supplierProducts, setSupplierProducts,
    customers, setCustomers,
    unreadNotificationCount, setUnreadNotificationCount,
    loadData,
    ...partialLoaders,
  };
}
