export interface Unit { id: number; code: string; name: string; unit_type: string; ratio_to_base: number; }
export interface DependentRef { id: number; name: string; }
export interface RecipeDependents { menu_items: DependentRef[]; parent_recipes: DependentRef[]; }
export interface MaterialCategory { id: number; code: string; name: string; sort_no: number; is_active: boolean; }
export interface TagItem { id: number; code: string; name: string; color?: string; }
export interface Material {
  id: number;
  code: string;
  name: string;
  category_id: number | null;
  base_unit_id: number;
  shelf_life_days: number | null;
  min_qty: number;
  is_active: boolean;
  created_at: string;
  updated_at: string;
  tags: TagItem[];
  category?: MaterialCategory;
  base_unit?: Unit;
}
export interface Recipe {
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
export interface RecipeType {
  id: number;
  code: string;
  name: string;
  description?: string | null;
  sort_no: number;
  is_system: boolean;
  is_active: boolean;
}
export interface RecipeItem {
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
export interface RecipeWithItems {
  recipe: Recipe;
  items: RecipeItem[];
}
export interface RecipeCostItem {
  material_name: string;
  qty: number;
  unit: string;
  cost_per_unit: number;
  wastage_rate: number;
  line_cost: number;
}
export interface RecipeCostResult {
  recipe_id: number;
  recipe_name: string;
  total_cost: number;
  cost_per_unit: number;
  output_qty: number;
  items: RecipeCostItem[];
}
export interface MenuItem {
  id: number;
  code: string | null;
  name: string;
  sales_price: number;
  is_available: boolean;
  recipe_id: number | null;
  category_id: number | null;
  created_at: string;
}
export interface MenuCategory { id: number; name: string; sort_no: number; is_active: boolean; }
export interface OrderItem {
  id: number;
  order_id: number;
  menu_item_id: number;
  spec_code: string | null;
  qty: number;
  unit_price: number;
  note: string | null;
}
export interface OrderItemModifier {
  id: number;
  order_item_id: number;
  modifier_type: string;
  material_id: number | null;
  material_name: string | null;
  qty: number;
  price_delta: number;
}
export interface Order {
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
export interface OrderWithItems {
  order: Order;
  items: OrderItem[];
}
export interface MenuItemSpec {
  id: number;
  menu_item_id: number;
  spec_code: string;
  spec_name: string;
  price_delta: number;
  qty_multiplier: number;
}
export interface POSCartItem {
  menu_item: MenuItem;
  spec: MenuItemSpec | null;
  qty: number;
  note: string;
  modifiers: { modifier_type: string; material_id?: number; qty: number; price_delta: number }[];
}
export interface KitchenStation {
  id: number;
  code: string;
  name: string;
  station_type: string;
}
export interface TicketWithItems {
  id: number;
  order_id: number;
  station_id: number;
  status: string;
  priority: number;
  printed_at: string | null;
  started_at: string | null;
  finished_at: string | null;
  created_at: string;
  order_no: string;
  dine_type: string;
  table_no: string | null;
  items: OrderItem[];
}
export interface InventoryBatch {
  id: number;
  material_id: number;
  lot_no: string;
  quantity: number;
  cost_per_unit: number;
  expiry_date: string | null;
  supplier_id: number | null;
}
export interface InventorySummary {
  material_id: number;
  material_name: string;
  total_qty: number;
  reserved_qty: number;
  available_qty: number;
}
export interface AttributeTemplate {
  id: number;
  entity_type: string;
  category: string | null;
  attr_code: string;
  attr_name: string;
  data_type: string;
  unit: string | null;
  default_value: number | null;
  formula: string | null;
}
export interface InventoryTxn {
  id: number;
  txn_no: string;
  txn_type: string;
  ref_type: string | null;
  ref_id: number | null;
  lot_id: number | null;
  material_id: number;
  state_id: number | null;
  qty_delta: number;
  cost_delta: number | null;
  operator: string | null;
  note: string | null;
  created_at: string;
}
export interface Supplier {
  id: number;
  name: string;
  phone: string | null;
  contact_person: string | null;
  address: string | null;
  note: string | null;
  is_active: boolean;
  created_at: string;
}
export interface Expense {
  id: number;
  expense_type: string;
  amount: number;
  expense_date: string;
  note: string | null;
  operator: string | null;
  is_active: boolean;
  created_at: string;
}
export interface SupplierProduct {
  id: number;
  product_name: string;
  supplier_name: string;
  channel: string;
}
export interface MaterialState {
  id: number;
  material_id: number;
  state_code: string;
  state_name: string;
  unit_id: number | null;
  yield_rate: number;
  cost_multiplier: number;
  is_active: boolean;
}
export interface PurchaseOrder {
  id: number;
  po_no: string;
  supplier_id: number | null;
  supplier_name: string | null;
  status: string;
  expected_date: string | null;
  total_cost: number;
  created_at: string;
}
export interface PurchaseOrderItem {
  id: number;
  po_id: number;
  material_id: number;
  material_name: string | null;
  qty: number;
  unit_id: number | null;
  unit_name: string | null;
  cost_per_unit: number;
  received_qty: number;
}
export interface PurchaseOrderWithItems {
  order: PurchaseOrder;
  items: PurchaseOrderItem[];
}
export interface ProductionOrder {
  id: number;
  production_no: string;
  recipe_id: number;
  recipe_name: string | null;
  status: string;
  planned_qty: number;
  actual_qty: number | null;
  operator: string | null;
  started_at: string | null;
  completed_at: string | null;
  created_at: string;
}
export interface ProductionOrderItem {
  id: number;
  production_id: number;
  material_id: number;
  material_name: string | null;
  lot_id: number | null;
  planned_qty: number;
  actual_qty: number | null;
}
export interface ProductionOrderWithItems {
  order: ProductionOrder;
  items: ProductionOrderItem[];
}
export interface Stocktake {
  id: number;
  stocktake_no: string;
  status: string;
  operator: string | null;
  note: string | null;
  created_at: string;
  completed_at: string | null;
}
export interface StocktakeItem {
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
export interface StocktakeWithItems {
  stocktake: Stocktake;
  items: StocktakeItem[];
}
export interface PrintTicketType {
  id: number;
  code: string;
  name: string;
  description: string | null;
  is_active: boolean;
  is_default: boolean;
  show_price: boolean;
  show_seq: boolean;
  show_note_field: boolean;
  station_id: number | null;
  paper_width: string;
  font_size: string;
  cut_mode: string;
  print_speed: string;
  print_density: string;
  show_order_no: boolean;
  show_table_no: boolean;
  show_dine_type: boolean;
  show_item_name: boolean;
  show_item_qty: boolean;
  show_item_price: boolean;
  show_item_subtotal: boolean;
  show_item_spec: boolean;
  show_item_note: boolean;
  show_created_at: boolean;
  show_total_amount: boolean;
  show_lot_no: boolean;
  show_qty_info: boolean;
  show_expiry_date: boolean;
  show_supplier: boolean;
  created_at: string;
  updated_at: string;
}
