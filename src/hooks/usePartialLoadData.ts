import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  Unit, MaterialCategory, TagItem, Material, Recipe, RecipeWithItems,
  RecipeCostResult, RecipeType, MenuItem, MenuCategory, Order, OrderWithItems, KitchenStation,
  TicketWithItems, InventoryBatch, InventorySummary, InventoryTxn, AttributeTemplate,
  Supplier, MaterialState, PurchaseOrder, PurchaseOrderWithItems,
  ProductionOrder, ProductionOrderWithItems, Stocktake, StocktakeWithItems
} from "../types";
import type { Expense, SupplierProduct, Customer } from "../types";

interface AppState {
  loading: boolean;
  connected: boolean;
  units: Unit[];
  categories: MaterialCategory[];
  tags: TagItem[];
  materials: Material[];
  recipes: Recipe[];
  recipeTypes: RecipeType[];
  selectedRecipe: RecipeWithItems | null;
  recipeCost: RecipeCostResult | null;
  menuCategories: MenuCategory[];
  menuItems: MenuItem[];
  orders: Order[];
  ordersHasMore: boolean;
  selectedOrder: OrderWithItems | null;
  stations: KitchenStation[];
  kdsTickets: TicketWithItems[];
  inventoryBatches: InventoryBatch[];
  inventorySummary: InventorySummary[];
  inventoryTxns: InventoryTxn[];
  attributeTemplates: AttributeTemplate[];
  suppliers: Supplier[];
  materialStates: MaterialState[];
  purchaseOrders: PurchaseOrder[];
  selectedPurchaseOrder: PurchaseOrderWithItems | null;
  productionOrders: ProductionOrder[];
  selectedProductionOrder: ProductionOrderWithItems | null;
  stocktakes: Stocktake[];
  selectedStocktake: StocktakeWithItems | null;
  expenses: Expense[];
  supplierProducts: SupplierProduct[];
  customers: Customer[];
}

interface AppSetters {
  setUnits: (v: Unit[]) => void;
  setCategories: (v: MaterialCategory[]) => void;
  setTags: (v: TagItem[]) => void;
  setMaterials: (v: Material[]) => void;
  setRecipes: (v: Recipe[]) => void;
  setRecipeTypes: (v: RecipeType[]) => void;
  setSelectedRecipe: (v: RecipeWithItems | null) => void;
  setRecipeCost: (v: RecipeCostResult | null) => void;
  setMenuCategories: (v: MenuCategory[]) => void;
  setMenuItems: (v: MenuItem[]) => void;
  setOrders: (v: Order[]) => void;
  setOrdersHasMore: (v: boolean) => void;
  setSelectedOrder: (v: OrderWithItems | null) => void;
  setStations: (v: KitchenStation[]) => void;
  setKdsTickets: (v: TicketWithItems[]) => void;
  setInventoryBatches: (v: InventoryBatch[]) => void;
  setInventorySummary: (v: InventorySummary[]) => void;
  setInventoryTxns: (v: InventoryTxn[]) => void;
  setAttributeTemplates: (v: AttributeTemplate[]) => void;
  setSuppliers: (v: Supplier[]) => void;
  setMaterialStates: (v: MaterialState[]) => void;
  setPurchaseOrders: (v: PurchaseOrder[]) => void;
  setSelectedPurchaseOrder: (v: PurchaseOrderWithItems | null) => void;
  setProductionOrders: (v: ProductionOrder[]) => void;
  setSelectedProductionOrder: (v: ProductionOrderWithItems | null) => void;
  setStocktakes: (v: Stocktake[]) => void;
  setSelectedStocktake: (v: StocktakeWithItems | null) => void;
  setExpenses: (v: Expense[]) => void;
  setSupplierProducts: (v: SupplierProduct[]) => void;
  setCustomers: (v: Customer[]) => void;
  setLoading: (v: boolean) => void;
  setConnected: (v: boolean) => void;
}

export function usePartialLoadData(state: AppState, setters: AppSetters) {
  const s = setters;

  const loadAll = useCallback(async () => {
    try {
      s.setLoading(true);
      const result = await invoke<string>("health_check");
      s.setConnected(result === "ok");
      try { await invoke("check_and_create_alerts"); } catch { /* ignore */ }
      s.setUnits(await invoke<Unit[]>("get_units"));
      s.setCategories(await invoke<MaterialCategory[]>("get_material_categories"));
      s.setTags(await invoke<TagItem[]>("get_tags"));
      s.setMaterials(await invoke<Material[]>("get_materials"));
      s.setRecipes(await invoke<Recipe[]>("get_recipes"));
      s.setRecipeTypes(await invoke<RecipeType[]>("get_recipe_types"));
      s.setMenuCategories(await invoke<MenuCategory[]>("get_menu_categories"));
      s.setMenuItems(await invoke<MenuItem[]>("get_menu_items"));
      const fetchedOrders = await invoke<Order[]>("get_orders", { limit: 200, offset: 0 });
      s.setOrders(fetchedOrders);
      s.setOrdersHasMore(fetchedOrders.length === 200);
      s.setStations(await invoke<KitchenStation[]>("get_kitchen_stations"));
      s.setInventoryBatches(await invoke<InventoryBatch[]>("get_inventory_batches"));
      s.setInventorySummary(await invoke<InventorySummary[]>("get_inventory_summary"));
      s.setInventoryTxns(await invoke<InventoryTxn[]>("get_inventory_txns", { limit: 50 }));
      s.setAttributeTemplates(await invoke<AttributeTemplate[]>("get_attribute_templates"));
      s.setSuppliers(await invoke<Supplier[]>("get_suppliers"));
      s.setMaterialStates(await invoke<MaterialState[]>("get_all_material_states"));
      s.setPurchaseOrders(await invoke<PurchaseOrder[]>("get_purchase_orders"));
      s.setProductionOrders(await invoke<ProductionOrder[]>("get_production_orders"));
      s.setStocktakes(await invoke<Stocktake[]>("get_stocktakes"));
      const pending = await invoke<TicketWithItems[]>("get_all_tickets_with_items", { status: "pending" });
      const started = await invoke<TicketWithItems[]>("get_all_tickets_with_items", { status: "started" });
      s.setKdsTickets([...pending, ...started]);
    } catch (e) {
      s.setConnected(false);
    } finally {
      s.setLoading(false);
    }
  }, [s]);

  const loadMaterials = useCallback(async () => {
    s.setMaterials(await invoke<Material[]>("get_materials"));
    s.setCategories(await invoke<MaterialCategory[]>("get_material_categories"));
    s.setTags(await invoke<TagItem[]>("get_tags"));
  }, [s]);

  const loadRecipes = useCallback(async () => {
    s.setRecipes(await invoke<Recipe[]>("get_recipes"));
    s.setRecipeTypes(await invoke<RecipeType[]>("get_recipe_types"));
  }, [s]);

  const loadMenu = useCallback(async () => {
    s.setMenuCategories(await invoke<MenuCategory[]>("get_menu_categories"));
    s.setMenuItems(await invoke<MenuItem[]>("get_menu_items"));
  }, [s]);

  const loadOrders = useCallback(async (limit = 200, offset = 0) => {
    const fetched = await invoke<Order[]>("get_orders", { limit, offset });
    if (offset === 0) {
      s.setOrders(fetched);
    } else {
      s.setOrders([...state.orders, ...fetched]);
    }
    s.setOrdersHasMore(fetched.length === limit);
  }, [s, state.orders]);

  const loadKDS = useCallback(async () => {
    const pending = await invoke<TicketWithItems[]>("get_all_tickets_with_items", { status: "pending" });
    const started = await invoke<TicketWithItems[]>("get_all_tickets_with_items", { status: "started" });
    s.setKdsTickets([...pending, ...started]);
  }, [s]);

  const loadInventory = useCallback(async () => {
    s.setInventoryBatches(await invoke<InventoryBatch[]>("get_inventory_batches"));
    s.setInventorySummary(await invoke<InventorySummary[]>("get_inventory_summary"));
    s.setInventoryTxns(await invoke<InventoryTxn[]>("get_inventory_txns", { limit: 50 }));
  }, [s]);

  const loadPurchaseOrders = useCallback(async () => {
    s.setPurchaseOrders(await invoke<PurchaseOrder[]>("get_purchase_orders"));
  }, [s]);

  const loadProductionOrders = useCallback(async () => {
    s.setProductionOrders(await invoke<ProductionOrder[]>("get_production_orders"));
  }, [s]);

  const loadStocktakes = useCallback(async () => {
    s.setStocktakes(await invoke<Stocktake[]>("get_stocktakes"));
  }, [s]);

  const loadSuppliers = useCallback(async () => {
    s.setSuppliers(await invoke<Supplier[]>("get_suppliers"));
  }, [s]);

  const loadMaterialStates = useCallback(async () => {
    s.setMaterialStates(await invoke<MaterialState[]>("get_all_material_states"));
  }, [s]);

  const loadExpenses = useCallback(async () => {
    s.setExpenses(await invoke<Expense[]>("get_expenses", { expenseType: null, startDate: null, endDate: null }));
  }, [s]);

  const loadSupplierProducts = useCallback(async () => {
    s.setSupplierProducts(await invoke<SupplierProduct[]>("get_supplier_products", { channel: null }));
  }, [s]);

  const loadCustomers = useCallback(async () => {
    s.setCustomers(await invoke<Customer[]>("get_customers", { search: null }));
  }, [s]);

  return {
    loadAll,
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
  };
}
