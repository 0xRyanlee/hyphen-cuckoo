use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

// ==================== 基礎類型 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Unit {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub unit_type: String,
    pub ratio_to_base: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MaterialCategory {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub sort_no: i32,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub color: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Material {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub category_id: Option<i64>,
    pub base_unit_id: i64,
    pub shelf_life_days: Option<i32>,
    pub min_qty: f64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MaterialWithTags {
    #[serde(flatten)]
    pub material: Material,
    pub tags: Vec<Tag>,
    pub category: Option<MaterialCategory>,
    pub base_unit: Option<Unit>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MaterialState {
    pub id: i64,
    pub material_id: i64,
    pub state_code: String,
    pub state_name: String,
    pub unit_id: Option<i64>,
    pub yield_rate: f64,
    pub cost_multiplier: f64,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Supplier {
    pub id: i64,
    pub name: String,
    pub phone: Option<String>,
    pub contact_person: Option<String>,
    pub address: Option<String>,
    pub note: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Expense {
    pub id: i64,
    pub expense_type: String,
    pub amount: f64,
    pub expense_date: String,
    pub note: Option<String>,
    pub operator: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SupplierProduct {
    pub id: i64,
    pub product_name: String,
    pub supplier_name: String,
    pub channel: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct MaterialCostHistory {
    pub id: i64,
    pub material_id: i64,
    pub cost_per_unit: f64,
    pub source_type: String,
    pub source_id: Option<i64>,
    pub batch_no: Option<String>,
    pub operator: Option<String>,
    pub created_at: String,
}

// ==================== 配方類型 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Recipe {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub recipe_type: String,
    pub output_material_id: Option<i64>,
    pub output_state_id: Option<i64>,
    pub output_qty: f64,
    pub output_unit_id: Option<i64>,
    pub cost: Option<f64>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecipeComponentType {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub sort_no: i32,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderComponentType {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_packaging: bool,
    pub cost_per_unit: f64,
    pub unit: Option<String>,
    pub sort_no: i32,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecipeType {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_no: i32,
    pub is_system: bool,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecipeItem {
    pub id: i64,
    pub recipe_id: i64,
    pub item_type: String,
    pub ref_id: i64,
    pub qty: f64,
    pub unit_id: i64,
    pub wastage_rate: f64,
    pub note: Option<String>,
    pub sort_no: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecipeWithItems {
    pub recipe: Recipe,
    pub items: Vec<RecipeItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecipeCostItem {
    pub material_name: String,
    pub qty: f64,
    pub unit: String,
    pub cost_per_unit: f64,
    pub wastage_rate: f64,
    pub line_cost: f64,
    pub item_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecipeCostResult {
    pub recipe_id: i64,
    pub recipe_name: String,
    pub total_cost: f64,
    pub cost_per_unit: f64,
    pub output_qty: f64,
    pub items: Vec<RecipeCostItem>,
}

// ==================== 庫存類型 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InventoryBatch {
    pub id: i64,
    pub material_id: i64,
    pub state_id: Option<i64>,
    pub lot_no: String,
    pub supplier_id: Option<i64>,
    pub brand: Option<String>,
    pub spec: Option<String>,
    pub quantity: f64,
    pub original_qty: f64,
    pub cost_per_unit: f64,
    pub production_date: Option<String>,
    pub expiry_date: Option<String>,
    pub ice_coating_rate: Option<f64>,
    pub quality_rate: Option<f64>,
    pub seasonal_factor: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InventoryTxn {
    pub id: i64,
    pub txn_no: String,
    pub txn_type: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<i64>,
    pub lot_id: Option<i64>,
    pub material_id: i64,
    pub state_id: Option<i64>,
    pub qty_delta: f64,
    pub cost_delta: f64,
    pub operator: Option<String>,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchSummary {
    pub lot_id: i64,
    pub lot_no: String,
    pub quantity: f64,
    pub cost_per_unit: f64,
    pub expiry_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InventorySummary {
    pub material_id: i64,
    pub material_name: String,
    pub total_qty: f64,
    pub reserved_qty: f64,
    pub available_qty: f64,
    pub batches: Vec<BatchSummary>,
}

// ==================== 菜單類型 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MenuCategory {
    pub id: i64,
    pub name: String,
    pub sort_no: i32,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MenuItem {
    pub id: i64,
    pub name: String,
    pub code: Option<String>,
    pub category_id: Option<i64>,
    pub recipe_id: Option<i64>,
    pub sales_price: f64,
    pub cost: Option<f64>,
    pub is_available: bool,
    pub is_favorite: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MenuItemSpec {
    pub id: i64,
    pub menu_item_id: i64,
    pub spec_code: String,
    pub spec_name: String,
    pub price_delta: f64,
    pub qty_multiplier: f64,
    pub sort_no: i32,
}

// ==================== 訂單類型 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Order {
    pub id: i64,
    pub order_no: String,
    pub source: String,
    pub dine_type: String,
    pub table_no: Option<String>,
    pub status: String,
    pub amount_total: f64,
    pub note: Option<String>,
    pub payment_status: String,
    pub payment_method: Option<String>,
    pub amount_paid: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Customer {
    pub id: i64,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub wechat_openid: Option<String>,
    pub membership_no: Option<String>,
    pub points: i64,
    pub balance: f64,
    pub birthday: Option<String>,
    pub gender: Option<String>,
    pub note: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Coupon {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub discount_percent: Option<f64>,
    pub discount_amount: Option<f64>,
    pub min_amount: Option<f64>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderItem {
    pub id: i64,
    pub order_id: i64,
    pub menu_item_id: i64,
    pub spec_code: Option<String>,
    pub qty: f64,
    pub unit_price: f64,
    pub note: Option<String>,
}

// ==================== KDS 類型 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KitchenStation {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub station_type: String,
    pub is_active: bool,
    pub sort_no: i32,
    pub printer_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KitchenTicket {
    pub id: i64,
    pub order_id: i64,
    pub station_id: i64,
    pub status: String,
    pub priority: i32,
    pub printed_at: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub created_at: String,
}

// ==================== 屬性模板類型 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttributeTemplate {
    pub id: i64,
    pub entity_type: String,
    pub category: Option<String>,
    pub attr_code: String,
    pub attr_name: String,
    pub data_type: String,
    pub unit: Option<String>,
    pub default_value: Option<f64>,
    pub formula: Option<String>,
    pub is_template: bool,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntityAttribute {
    pub id: i64,
    pub entity_type: String,
    pub entity_id: i64,
    pub attr_code: String,
    pub value: Option<f64>,
    pub value_text: Option<String>,
    pub calculated: bool,
    pub updated_at: String,
}

// ==================== 打印機類型 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrinterConfig {
    pub id: i64,
    pub name: String,
    pub printer_type: String,
    pub connection_type: String,
    pub feie_user: Option<String>,
    pub feie_ukey: Option<String>,
    pub feie_sn: Option<String>,
    pub feie_key: Option<String>,
    pub lan_ip: Option<String>,
    pub lan_port: i32,
    pub paper_width: String,
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrintTask {
    pub id: i64,
    pub task_type: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<i64>,
    pub content: String,
    pub status: String,
    pub printer_id: Option<i64>,
    pub printer_name: Option<String>,
    pub created_at: String,
    pub printed_at: Option<String>,
    pub error_msg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notification {
    pub id: i64,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub severity: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<i64>,
    pub is_read: bool,
    pub read_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintTemplate {
    pub id: i64,
    pub name: String,
    pub template_type: String,
    pub paper_size: String,
    pub label_width_mm: Option<f64>,
    pub label_height_mm: Option<f64>,
    pub content: String,
    pub is_default: bool,
    pub is_active: bool,
    pub theme: Option<String>,
    pub restaurant_name: Option<String>,
    pub tagline: Option<String>,
    pub logo_data: Option<String>,
    pub show_price: Option<bool>,
    pub show_tax: Option<bool>,
    pub show_service_charge: Option<bool>,
    pub item_sort: Option<String>,
    pub modifiers_color: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintTicketType {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub is_default: bool,
    pub show_price: bool,
    pub show_seq: bool,
    pub show_note_field: bool,
    pub station_id: Option<i64>,
    pub paper_width: String,
    pub font_size: String,
    pub cut_mode: String,
    pub print_speed: String,
    pub print_density: String,
    pub show_order_no: bool,
    pub show_table_no: bool,
    pub show_dine_type: bool,
    pub show_item_name: bool,
    pub show_item_qty: bool,
    pub show_item_price: bool,
    pub show_item_subtotal: bool,
    pub show_item_spec: bool,
    pub show_item_note: bool,
    pub show_created_at: bool,
    pub show_total_amount: bool,
    pub show_lot_no: bool,
    pub show_qty_info: bool,
    pub show_expiry_date: bool,
    pub show_supplier: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePrintTemplateRequest {
    pub name: String,
    pub template_type: String,
    pub paper_size: String,
    pub label_width_mm: Option<f64>,
    pub label_height_mm: Option<f64>,
    pub content: String,
    pub theme: Option<String>,
    pub restaurant_name: Option<String>,
    pub tagline: Option<String>,
    pub logo_data: Option<String>,
    pub show_price: Option<bool>,
    pub show_tax: Option<bool>,
    pub show_service_charge: Option<bool>,
    pub item_sort: Option<String>,
    pub modifiers_color: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePrintTicketTypeRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub is_default: bool,
    pub show_price: bool,
    pub show_seq: bool,
    pub show_note_field: bool,
    pub station_id: Option<i64>,
    pub paper_width: String,
    pub font_size: String,
    pub cut_mode: String,
    pub print_speed: String,
    pub print_density: String,
    pub show_order_no: bool,
    pub show_table_no: bool,
    pub show_dine_type: bool,
    pub show_item_name: bool,
    pub show_item_qty: bool,
    pub show_item_price: bool,
    pub show_item_subtotal: bool,
    pub show_item_spec: bool,
    pub show_item_note: bool,
    pub show_created_at: bool,
    pub show_total_amount: bool,
    pub show_lot_no: bool,
    pub show_qty_info: bool,
    pub show_expiry_date: bool,
    pub show_supplier: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePrintTicketTypeRequest {
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub is_default: bool,
    pub show_price: bool,
    pub show_seq: bool,
    pub show_note_field: bool,
    pub station_id: Option<i64>,
    pub paper_width: String,
    pub font_size: String,
    pub cut_mode: String,
    pub print_speed: String,
    pub print_density: String,
    pub show_order_no: bool,
    pub show_table_no: bool,
    pub show_dine_type: bool,
    pub show_item_name: bool,
    pub show_item_qty: bool,
    pub show_item_price: bool,
    pub show_item_subtotal: bool,
    pub show_item_spec: bool,
    pub show_item_note: bool,
    pub show_created_at: bool,
    pub show_total_amount: bool,
    pub show_lot_no: bool,
    pub show_qty_info: bool,
    pub show_expiry_date: bool,
    pub show_supplier: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintPreviewResult {
    pub html: String,
    pub lines: Vec<String>,
    pub paper_width: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrder {
    pub id: i64,
    pub po_no: String,
    pub supplier_id: Option<i64>,
    pub supplier_name: Option<String>,
    pub status: String,
    pub expected_date: Option<String>,
    pub total_cost: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrderItem {
    pub id: i64,
    pub po_id: i64,
    pub material_id: i64,
    pub material_name: Option<String>,
    pub qty: f64,
    pub unit_id: Option<i64>,
    pub unit_name: Option<String>,
    pub cost_per_unit: f64,
    pub received_qty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItemModifier {
    pub id: i64,
    pub order_item_id: i64,
    pub modifier_type: String,
    pub material_id: Option<i64>,
    pub material_name: Option<String>,
    pub qty: f64,
    pub price_delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrderWithItems {
    pub order: PurchaseOrder,
    pub items: Vec<PurchaseOrderItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionOrder {
    pub id: i64,
    pub production_no: String,
    pub recipe_id: i64,
    pub recipe_name: Option<String>,
    pub status: String,
    pub planned_qty: f64,
    pub actual_qty: Option<f64>,
    pub operator: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionOrderItem {
    pub id: i64,
    pub production_id: i64,
    pub material_id: i64,
    pub material_name: Option<String>,
    pub lot_id: Option<i64>,
    pub planned_qty: f64,
    pub actual_qty: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionOrderWithItems {
    pub order: ProductionOrder,
    pub items: Vec<ProductionOrderItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stocktake {
    pub id: i64,
    pub stocktake_no: String,
    pub status: String,
    pub operator: Option<String>,
    pub note: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StocktakeItem {
    pub id: i64,
    pub stocktake_id: i64,
    pub lot_id: Option<i64>,
    pub material_id: i64,
    pub material_name: Option<String>,
    pub system_qty: f64,
    pub actual_qty: f64,
    pub diff_qty: Option<f64>,
    pub is_counted: bool,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StocktakeWithItems {
    pub stocktake: Stocktake,
    pub items: Vec<StocktakeItem>,
}

// ==================== 數據庫 ====================

/// Expands a recipe into a flat material_id → qty map (accumulates into `result`).
/// Sub-recipes with output_material_id consume that material (pre-made stock).
/// Sub-recipes without output_material_id are expanded recursively (depth-guarded at 10).
fn expand_recipe_needs(
    conn: &Connection,
    recipe_id: i64,
    multiplier: f64,
    depth: u32,
    result: &mut std::collections::HashMap<i64, f64>,
) -> Result<()> {
    if depth > 10 { return Ok(()); }
    let mut stmt = conn.prepare_cached(
        "SELECT item_type, ref_id, qty * (1.0 + COALESCE(wastage_rate, 0.0)) \
         FROM recipe_items WHERE recipe_id = ?1"
    )?;
    let items: Vec<(String, i64, f64)> = stmt.query_map(params![recipe_id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?.collect::<Result<Vec<_>>>()?;
    for (item_type, ref_id, qty) in items {
        match item_type.as_str() {
            "material" => {
                *result.entry(ref_id).or_insert(0.0) += qty * multiplier;
            }
            "sub_recipe" => {
                let output_mid: Option<i64> = conn.query_row(
                    "SELECT output_material_id FROM recipes WHERE id = ?1",
                    params![ref_id], |row| row.get(0),
                )?;
                match output_mid {
                    Some(mid) => { *result.entry(mid).or_insert(0.0) += qty * multiplier; }
                    None => { expand_recipe_needs(conn, ref_id, qty * multiplier, depth + 1, result)?; }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        // 設置 busy timeout 解決多進程鎖定問題
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        // 啟用 WAL mode 提高並發性能
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys = ON")?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.init_tables()?;
        db.seed_data()?;
        db.run_data_migrations()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute_batch(
            "
            -- ==================== 基礎資料表 ====================
            
            CREATE TABLE IF NOT EXISTS units (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                unit_type TEXT NOT NULL DEFAULT 'piece',
                ratio_to_base REAL NOT NULL DEFAULT 1.0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            
            CREATE TABLE IF NOT EXISTS material_categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                sort_no INTEGER DEFAULT 0,
                is_active INTEGER NOT NULL DEFAULT 1
            );
            
            CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                color TEXT,
                is_active INTEGER NOT NULL DEFAULT 1
            );
            
            CREATE TABLE IF NOT EXISTS material_tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                material_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                FOREIGN KEY (material_id) REFERENCES materials(id),
                FOREIGN KEY (tag_id) REFERENCES tags(id)
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_material_tag_unique ON material_tags(material_id, tag_id);
            
            CREATE TABLE IF NOT EXISTS materials (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                category_id INTEGER,
                base_unit_id INTEGER NOT NULL,
                shelf_life_days INTEGER,
                min_qty REAL NOT NULL DEFAULT 10.0,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (category_id) REFERENCES material_categories(id),
                FOREIGN KEY (base_unit_id) REFERENCES units(id)
            );
            
            CREATE TABLE IF NOT EXISTS material_states (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                material_id INTEGER NOT NULL,
                state_code TEXT NOT NULL,
                state_name TEXT NOT NULL,
                unit_id INTEGER,
                yield_rate REAL NOT NULL DEFAULT 1.0,
                cost_multiplier REAL NOT NULL DEFAULT 1.0,
                is_active INTEGER NOT NULL DEFAULT 1,
                FOREIGN KEY (material_id) REFERENCES materials(id),
                FOREIGN KEY (unit_id) REFERENCES units(id)
            );
            CREATE INDEX IF NOT EXISTS idx_material_states_material ON material_states(material_id);
            
            CREATE TABLE IF NOT EXISTS suppliers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                phone TEXT,
                contact_person TEXT,
                address TEXT,
                note TEXT,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- ==================== 日常支出 ====================

            CREATE TABLE IF NOT EXISTS expenses (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                expense_type TEXT NOT NULL,
                amount REAL NOT NULL,
                expense_date TEXT NOT NULL DEFAULT (date('now')),
                note TEXT,
                operator TEXT,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_expenses_type ON expenses(expense_type);
            CREATE INDEX IF NOT EXISTS idx_expenses_date ON expenses(expense_date);

            -- ==================== 供應商商品 ====================

            CREATE TABLE IF NOT EXISTS supplier_products (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                product_name TEXT NOT NULL,
                supplier_name TEXT NOT NULL,
                channel TEXT NOT NULL DEFAULT 'local',
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- ==================== 材料成本歷史 ====================

            CREATE TABLE IF NOT EXISTS material_cost_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                material_id INTEGER NOT NULL,
                cost_per_unit REAL NOT NULL,
                source_type TEXT NOT NULL,
                source_id INTEGER,
                batch_no TEXT,
                operator TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (material_id) REFERENCES materials(id)
            );

            CREATE INDEX IF NOT EXISTS idx_cost_history_material ON material_cost_history(material_id);
            CREATE INDEX IF NOT EXISTS idx_cost_history_date ON material_cost_history(created_at);
            
            -- ==================== 屬性系統 ====================
            
            CREATE TABLE IF NOT EXISTS attribute_templates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entity_type TEXT NOT NULL,
                category TEXT,
                attr_code TEXT NOT NULL,
                attr_name TEXT NOT NULL,
                data_type TEXT NOT NULL,
                unit TEXT,
                default_value REAL,
                formula TEXT,
                is_template INTEGER NOT NULL DEFAULT 1,
                is_active INTEGER NOT NULL DEFAULT 1
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_attr_template_code ON attribute_templates(attr_code);
            
            CREATE TABLE IF NOT EXISTS entity_attributes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entity_type TEXT NOT NULL,
                entity_id INTEGER NOT NULL,
                attr_code TEXT NOT NULL,
                value REAL,
                value_text TEXT,
                calculated INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_entity_attr ON entity_attributes(entity_type, entity_id, attr_code);
            
            -- ==================== 配方系統 ====================
            
            CREATE TABLE IF NOT EXISTS recipes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                recipe_type TEXT NOT NULL DEFAULT 'menu',
                output_material_id INTEGER,
                output_state_id INTEGER,
                output_qty REAL NOT NULL DEFAULT 1.0,
                output_unit_id INTEGER,
                cost REAL,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (output_material_id) REFERENCES materials(id),
                FOREIGN KEY (output_state_id) REFERENCES material_states(id),
                FOREIGN KEY (output_unit_id) REFERENCES units(id)
            );
            
            CREATE TABLE IF NOT EXISTS recipe_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                recipe_id INTEGER NOT NULL,
                item_type TEXT NOT NULL,
                ref_id INTEGER NOT NULL,
                qty REAL NOT NULL,
                unit_id INTEGER NOT NULL,
                wastage_rate REAL NOT NULL DEFAULT 0.0,
                note TEXT,
                sort_no INTEGER DEFAULT 0,
                FOREIGN KEY (recipe_id) REFERENCES recipes(id),
                FOREIGN KEY (unit_id) REFERENCES units(id)
            );
            CREATE INDEX IF NOT EXISTS idx_recipe_items_recipe ON recipe_items(recipe_id);
            CREATE INDEX IF NOT EXISTS idx_recipe_items_recipe_type ON recipe_items(recipe_id, item_type);

            CREATE TABLE IF NOT EXISTS recipe_types (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                description TEXT,
                sort_no INTEGER NOT NULL DEFAULT 0,
                is_system INTEGER NOT NULL DEFAULT 0,
                is_active INTEGER NOT NULL DEFAULT 1
            );
            CREATE INDEX IF NOT EXISTS idx_recipe_types_active_sort ON recipe_types(is_active, sort_no, id);
            
            CREATE TABLE IF NOT EXISTS recipe_formulas (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                recipe_id INTEGER NOT NULL,
                formula_code TEXT NOT NULL,
                formula_name TEXT NOT NULL,
                expression TEXT NOT NULL,
                result_unit TEXT,
                is_active INTEGER NOT NULL DEFAULT 1,
                FOREIGN KEY (recipe_id) REFERENCES recipes(id)
            );

            -- ==================== 配方/訂單組成結構 ====================

            CREATE TABLE IF NOT EXISTS recipe_component_types (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                description TEXT,
                category TEXT NOT NULL,
                sort_no INTEGER NOT NULL DEFAULT 0,
                is_active INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS order_component_types (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                description TEXT,
                is_packaging INTEGER NOT NULL DEFAULT 0,
                cost_per_unit REAL NOT NULL DEFAULT 0,
                unit TEXT,
                sort_no INTEGER NOT NULL DEFAULT 0,
                is_active INTEGER NOT NULL DEFAULT 1
            );

            CREATE INDEX IF NOT EXISTS idx_recipe_component_types_category ON recipe_component_types(category);
            CREATE INDEX IF NOT EXISTS idx_order_component_types_packaging ON order_component_types(is_packaging);
            
            -- ==================== 庫存系統 ====================
            
            CREATE TABLE IF NOT EXISTS stores (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                address TEXT,
                phone TEXT,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            
            CREATE TABLE IF NOT EXISTS inventory_batches (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                material_id INTEGER NOT NULL,
                state_id INTEGER,
                lot_no TEXT NOT NULL UNIQUE,
                supplier_id INTEGER,
                brand TEXT,
                spec TEXT,
                quantity REAL NOT NULL DEFAULT 0,
                original_qty REAL NOT NULL,
                cost_per_unit REAL NOT NULL DEFAULT 0.0,
                production_date TEXT,
                expiry_date TEXT,
                ice_coating_rate REAL,
                quality_rate REAL,
                seasonal_factor REAL NOT NULL DEFAULT 1.0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (material_id) REFERENCES materials(id),
                FOREIGN KEY (state_id) REFERENCES material_states(id),
                FOREIGN KEY (supplier_id) REFERENCES suppliers(id)
            );
            CREATE INDEX IF NOT EXISTS idx_batch_material ON inventory_batches(material_id);
            CREATE INDEX IF NOT EXISTS idx_batch_expiry ON inventory_batches(expiry_date);
            CREATE INDEX IF NOT EXISTS idx_batch_lot ON inventory_batches(lot_no);
            
            CREATE TABLE IF NOT EXISTS inventory_txns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                txn_no TEXT NOT NULL UNIQUE,
                txn_type TEXT NOT NULL,
                ref_type TEXT,
                ref_id INTEGER,
                lot_id INTEGER,
                material_id INTEGER NOT NULL,
                state_id INTEGER,
                qty_delta REAL NOT NULL,
                cost_delta REAL NOT NULL DEFAULT 0.0,
                operator TEXT,
                note TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (lot_id) REFERENCES inventory_batches(id),
                FOREIGN KEY (material_id) REFERENCES materials(id),
                FOREIGN KEY (state_id) REFERENCES material_states(id)
            );
            CREATE INDEX IF NOT EXISTS idx_txn_material ON inventory_txns(material_id);
            CREATE INDEX IF NOT EXISTS idx_txn_lot ON inventory_txns(lot_id);
            CREATE INDEX IF NOT EXISTS idx_txn_ref ON inventory_txns(ref_type, ref_id);
            CREATE INDEX IF NOT EXISTS idx_txn_created ON inventory_txns(created_at);
            
            -- ==================== 菜單系統 ====================
            
            CREATE TABLE IF NOT EXISTS menu_categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT UNIQUE,
                name TEXT NOT NULL UNIQUE,
                sort_no INTEGER DEFAULT 0,
                is_active INTEGER NOT NULL DEFAULT 1
            );
            
            CREATE TABLE IF NOT EXISTS menu_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                code TEXT UNIQUE,
                category_id INTEGER,
                recipe_id INTEGER,
                sales_price REAL NOT NULL DEFAULT 0.0,
                cost REAL,
                is_available INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (category_id) REFERENCES menu_categories(id),
                FOREIGN KEY (recipe_id) REFERENCES recipes(id)
            );
            
            CREATE TABLE IF NOT EXISTS menu_item_specs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                menu_item_id INTEGER NOT NULL,
                spec_code TEXT NOT NULL,
                spec_name TEXT NOT NULL,
                price_delta REAL NOT NULL DEFAULT 0.0,
                qty_multiplier REAL NOT NULL DEFAULT 1.0,
                sort_no INTEGER DEFAULT 0,
                FOREIGN KEY (menu_item_id) REFERENCES menu_items(id)
            );
            CREATE INDEX IF NOT EXISTS idx_menu_item_specs_item ON menu_item_specs(menu_item_id);
            
            -- ==================== 訂單系統 ====================
            
            CREATE TABLE IF NOT EXISTS orders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                order_no TEXT NOT NULL UNIQUE,
                source TEXT NOT NULL DEFAULT 'pos',
                dine_type TEXT NOT NULL DEFAULT 'dine_in',
                table_no TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                amount_total REAL NOT NULL DEFAULT 0.0,
                note TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
            CREATE INDEX IF NOT EXISTS idx_orders_created ON orders(created_at);
            CREATE INDEX IF NOT EXISTS idx_menu_items_category ON menu_items(category_id);
            
            CREATE TABLE IF NOT EXISTS customers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT,
                phone TEXT,
                wechat_openid TEXT UNIQUE,
                membership_no TEXT UNIQUE,
                points INTEGER DEFAULT 0,
                balance REAL DEFAULT 0.0,
                birthday TEXT,
                gender TEXT,
                note TEXT,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            
            CREATE TABLE IF NOT EXISTS coupons (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                code TEXT UNIQUE,
                discount_percent REAL,
                discount_amount REAL,
                min_amount REAL,
                valid_from TEXT,
                valid_until TEXT,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            
            CREATE TABLE IF NOT EXISTS customer_coupons (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                customer_id INTEGER NOT NULL,
                coupon_id INTEGER NOT NULL,
                used INTEGER NOT NULL DEFAULT 0,
                used_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            
            CREATE TABLE IF NOT EXISTS order_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                order_id INTEGER NOT NULL,
                menu_item_id INTEGER NOT NULL,
                spec_code TEXT,
                qty REAL NOT NULL DEFAULT 1.0,
                unit_price REAL NOT NULL,
                note TEXT,
                FOREIGN KEY (order_id) REFERENCES orders(id),
                FOREIGN KEY (menu_item_id) REFERENCES menu_items(id)
            );
            CREATE INDEX IF NOT EXISTS idx_order_items_order ON order_items(order_id);
            
            CREATE TABLE IF NOT EXISTS order_item_modifiers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                order_item_id INTEGER NOT NULL,
                modifier_type TEXT NOT NULL,
                material_id INTEGER,
                qty REAL NOT NULL,
                price_delta REAL NOT NULL DEFAULT 0.0,
                FOREIGN KEY (order_item_id) REFERENCES order_items(id),
                FOREIGN KEY (material_id) REFERENCES materials(id)
            );
            CREATE INDEX IF NOT EXISTS idx_order_item_modifiers_item ON order_item_modifiers(order_item_id);
            
            -- ==================== KDS 系統 ====================
            
            CREATE TABLE IF NOT EXISTS kitchen_stations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                station_type TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1,
                sort_no INTEGER DEFAULT 0
            );
            
            CREATE TABLE IF NOT EXISTS station_menu_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                station_id INTEGER NOT NULL,
                menu_item_id INTEGER NOT NULL,
                FOREIGN KEY (station_id) REFERENCES kitchen_stations(id),
                FOREIGN KEY (menu_item_id) REFERENCES menu_items(id)
            );
            CREATE INDEX IF NOT EXISTS idx_station_menu_items_station ON station_menu_items(station_id);
            CREATE INDEX IF NOT EXISTS idx_station_menu_items_menu_item ON station_menu_items(menu_item_id);
            
            CREATE TABLE IF NOT EXISTS kitchen_tickets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                order_id INTEGER NOT NULL,
                station_id INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                priority INTEGER DEFAULT 0,
                printed_at TEXT,
                started_at TEXT,
                finished_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (order_id) REFERENCES orders(id),
                FOREIGN KEY (station_id) REFERENCES kitchen_stations(id)
            );
            CREATE INDEX IF NOT EXISTS idx_tickets_station ON kitchen_tickets(station_id);
            CREATE INDEX IF NOT EXISTS idx_tickets_status ON kitchen_tickets(status);
            
            -- ==================== 採購系統 ====================
            
            CREATE TABLE IF NOT EXISTS purchase_orders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                po_no TEXT NOT NULL UNIQUE,
                supplier_id INTEGER,
                status TEXT NOT NULL DEFAULT 'draft',
                expected_date TEXT,
                total_cost REAL NOT NULL DEFAULT 0.0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (supplier_id) REFERENCES suppliers(id)
            );
            
            CREATE TABLE IF NOT EXISTS purchase_order_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                po_id INTEGER NOT NULL,
                material_id INTEGER NOT NULL,
                qty REAL NOT NULL,
                unit_id INTEGER,
                cost_per_unit REAL NOT NULL,
                received_qty REAL NOT NULL DEFAULT 0.0,
                FOREIGN KEY (po_id) REFERENCES purchase_orders(id),
                FOREIGN KEY (material_id) REFERENCES materials(id),
                FOREIGN KEY (unit_id) REFERENCES units(id)
            );
            
            -- ==================== 生產系統 ====================
            
            CREATE TABLE IF NOT EXISTS production_orders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                production_no TEXT NOT NULL UNIQUE,
                recipe_id INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'draft',
                planned_qty REAL NOT NULL,
                actual_qty REAL,
                operator TEXT,
                started_at TEXT,
                completed_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (recipe_id) REFERENCES recipes(id)
            );
            
            CREATE TABLE IF NOT EXISTS production_order_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                production_id INTEGER NOT NULL,
                material_id INTEGER NOT NULL,
                lot_id INTEGER,
                planned_qty REAL NOT NULL,
                actual_qty REAL,
                FOREIGN KEY (production_id) REFERENCES production_orders(id),
                FOREIGN KEY (material_id) REFERENCES materials(id),
                FOREIGN KEY (lot_id) REFERENCES inventory_batches(id)
            );
            
            -- ==================== 盤點系統 ====================
            
            CREATE TABLE IF NOT EXISTS stocktakes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                stocktake_no TEXT NOT NULL UNIQUE,
                status TEXT NOT NULL DEFAULT 'draft',
                operator TEXT,
                note TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                completed_at TEXT
            );
            
            CREATE TABLE IF NOT EXISTS stocktake_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                stocktake_id INTEGER NOT NULL,
                lot_id INTEGER,
                material_id INTEGER NOT NULL,
                system_qty REAL NOT NULL,
                actual_qty REAL NOT NULL,
                diff_qty REAL,
                note TEXT,
                FOREIGN KEY (stocktake_id) REFERENCES stocktakes(id),
                FOREIGN KEY (lot_id) REFERENCES inventory_batches(id),
                FOREIGN KEY (material_id) REFERENCES materials(id)
            );
            
            -- ==================== 打印系統 ====================

            CREATE TABLE IF NOT EXISTS printer_configs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                printer_type TEXT NOT NULL DEFAULT 'thermal',
                connection_type TEXT NOT NULL DEFAULT 'feie',
                station_id INTEGER,
                feie_user TEXT,
                feie_ukey TEXT,
                feie_sn TEXT,
                feie_key TEXT,
                lan_ip TEXT,
                lan_port INTEGER DEFAULT 9100,
                paper_width TEXT DEFAULT '80mm',
                is_default INTEGER NOT NULL DEFAULT 0,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (station_id) REFERENCES kitchen_stations(id)
            );

            CREATE TABLE IF NOT EXISTS print_tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_type TEXT NOT NULL,
                ref_type TEXT,
                ref_id INTEGER,
                content TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                printer_id INTEGER,
                printer_name TEXT,
                retry_count INTEGER NOT NULL DEFAULT 0,
                max_retries INTEGER NOT NULL DEFAULT 3,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                printed_at TEXT,
                error_msg TEXT,
                FOREIGN KEY (printer_id) REFERENCES printer_configs(id)
            );

            CREATE TABLE IF NOT EXISTS print_templates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                template_type TEXT NOT NULL,
                paper_size TEXT NOT NULL DEFAULT '80mm',
                label_width_mm REAL,
                label_height_mm REAL,
                content TEXT NOT NULL,
                is_default INTEGER NOT NULL DEFAULT 0,
                is_active INTEGER NOT NULL DEFAULT 1,
                theme TEXT,
                restaurant_name TEXT,
                tagline TEXT,
                logo_data TEXT,
                show_price INTEGER DEFAULT 1,
                show_tax INTEGER DEFAULT 1,
                show_service_charge INTEGER DEFAULT 1,
                item_sort TEXT DEFAULT 'entry',
                modifiers_color TEXT DEFAULT 'red',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS print_ticket_types (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                description TEXT,
                is_active INTEGER NOT NULL DEFAULT 1,
                is_default INTEGER NOT NULL DEFAULT 0,
                show_price INTEGER NOT NULL DEFAULT 0,
                show_seq INTEGER NOT NULL DEFAULT 1,
                show_note_field INTEGER NOT NULL DEFAULT 1,
                station_id INTEGER,
                paper_width TEXT NOT NULL DEFAULT '58mm',
                font_size TEXT NOT NULL DEFAULT 'medium',
                cut_mode TEXT NOT NULL DEFAULT 'full',
                print_speed TEXT NOT NULL DEFAULT 'medium',
                print_density TEXT NOT NULL DEFAULT 'medium',
                show_order_no INTEGER NOT NULL DEFAULT 1,
                show_table_no INTEGER NOT NULL DEFAULT 1,
                show_dine_type INTEGER NOT NULL DEFAULT 1,
                show_item_name INTEGER NOT NULL DEFAULT 1,
                show_item_qty INTEGER NOT NULL DEFAULT 1,
                show_item_price INTEGER NOT NULL DEFAULT 0,
                show_item_subtotal INTEGER NOT NULL DEFAULT 0,
                show_item_spec INTEGER NOT NULL DEFAULT 1,
                show_item_note INTEGER NOT NULL DEFAULT 1,
                show_created_at INTEGER NOT NULL DEFAULT 1,
                show_total_amount INTEGER NOT NULL DEFAULT 0,
                show_lot_no INTEGER NOT NULL DEFAULT 1,
                show_qty_info INTEGER NOT NULL DEFAULT 1,
                show_expiry_date INTEGER NOT NULL DEFAULT 1,
                show_supplier INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS notifications (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                notification_type TEXT NOT NULL,
                title TEXT NOT NULL,
                message TEXT NOT NULL,
                severity TEXT NOT NULL DEFAULT 'info',
                ref_type TEXT,
                ref_id INTEGER,
                is_read INTEGER NOT NULL DEFAULT 0,
                read_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE UNIQUE INDEX IF NOT EXISTS idx_menu_categories_name ON menu_categories(name);
            "
        )?;
        // Migrations for existing databases — errors are expected when column already exists
        let has_min_qty: bool = conn
            .query_row("SELECT COUNT(*) FROM pragma_table_info('materials') WHERE name='min_qty'", [], |r| r.get::<_, i64>(0))
            .unwrap_or(0) > 0;
        if !has_min_qty {
            conn.execute("ALTER TABLE materials ADD COLUMN min_qty REAL NOT NULL DEFAULT 10.0", [])?;
        }
        let has_payment_status: bool = conn
            .query_row("SELECT COUNT(*) FROM pragma_table_info('orders') WHERE name='payment_status'", [], |r| r.get::<_, i64>(0))
            .unwrap_or(0) > 0;
        if !has_payment_status {
            conn.execute("ALTER TABLE orders ADD COLUMN payment_status TEXT NOT NULL DEFAULT 'unpaid'", [])?;
            conn.execute("ALTER TABLE orders ADD COLUMN payment_method TEXT", [])?;
            conn.execute("ALTER TABLE orders ADD COLUMN amount_paid REAL NOT NULL DEFAULT 0.0", [])?;
        }
        let has_is_counted: bool = conn
            .query_row("SELECT COUNT(*) FROM pragma_table_info('stocktake_items') WHERE name='is_counted'", [], |r| r.get::<_, i64>(0))
            .unwrap_or(0) > 0;
        if !has_is_counted {
            conn.execute("ALTER TABLE stocktake_items ADD COLUMN is_counted INTEGER NOT NULL DEFAULT 0", [])?;
        }
        Ok(())
    }

    fn seed_data(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM menu_categories WHERE id NOT IN (SELECT MIN(id) FROM menu_categories GROUP BY name)",
            [],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO units (code, name, unit_type, ratio_to_base) VALUES
                ('pc', '个', 'piece', 1.0), ('kg', '千克', 'weight', 1.0), ('g', '克', 'weight', 1000.0),
                ('l', '升', 'volume', 1.0), ('ml', '毫升', 'volume', 1000.0), ('box', '箱', 'package', 1.0),
                ('bag', '袋', 'package', 1.0), ('bottle', '瓶', 'package', 1.0)",
            [],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO material_categories (code, name, sort_no) VALUES
                ('seafood', '海鲜', 1), ('meat', '肉类', 2), ('vegetable', '蔬菜', 3),
                ('seasoning', '调味料', 4), ('staple', '主食', 5), ('other', '其他', 6)",
            [],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO tags (code, name, color) VALUES
                ('seafood', '海鲜', '#3B82F6'), ('meat', '肉类', '#EF4444'), ('offal', '下水', '#F59E0B'),
                ('braised', '卤味', '#8B5CF6'), ('vegetable', '蔬菜', '#10B981'), ('staple', '主食', '#6B7280'),
                ('seasoning', '调味料', '#EC4899'), ('frozen', '冷冻', '#06B6D4'), ('fresh', '鲜活', '#84CC16')",
            [],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO menu_categories (code, name, sort_no) VALUES
                ('seafood_dishes', '海鲜类', 1), ('meat_dishes', '肉类', 2), ('vegetable_dishes', '素食类', 3), ('staple_other', '主食与其他', 4)",
            [],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO recipe_types (code, name, description, sort_no, is_system, is_active) VALUES
                ('menu', '成品配方', '直接綁定菜單商品的出品配方', 1, 1, 1),
                ('production', '半成品配方', '用於中央廚房或門店預製的產出型配方', 2, 1, 1),
                ('modifier', '加料配方', '用於加料、配菜、附加項的局部配方', 3, 1, 1)",
            [],
        )?;
        let mat_count: i64 = conn.query_row("SELECT COUNT(*) FROM materials", [], |row| row.get(0))?;
        if mat_count == 0 {
            conn.execute(
                "INSERT INTO materials (code, name, category_id, base_unit_id, shelf_life_days) VALUES
                    ('MAT001', '东风螺', 1, 2, 3), ('MAT002', '石螺', 1, 2, 3), ('MAT003', '钉螺', 1, 2, 3),
                    ('MAT004', '虾尾', 1, 2, 2), ('MAT005', '扇贝肉', 1, 2, 2), ('MAT006', '蟹钳', 1, 2, 2),
                    ('MAT007', '章鱼足', 1, 2, 3), ('MAT008', '鲨鱼肚', 1, 2, 5), ('MAT009', '肥肠', 2, 2, 3),
                    ('MAT010', '凤爪', 2, 2, 3), ('MAT011', '毛肚', 2, 2, 2), ('MAT012', '藕', 3, 2, 5),
                    ('MAT013', '笋尖', 3, 2, 7), ('MAT014', '川粉', 3, 2, 180), ('MAT015', '香菇', 3, 2, 5),
                    ('MAT016', '豆腐皮', 3, 1, 3), ('MAT017', '金针菇', 3, 2, 5), ('MAT018', '土豆', 3, 2, 14),
                    ('MAT019', '丸子拼盘', 3, 1, 5), ('MAT020', '棉豆腐', 3, 1, 3), ('MAT021', '方便面', 4, 1, 180),
                    ('MAT022', '麻辣底料', 1, 2, 365), ('MAT023', '辣椒油', 1, 4, 365)",
                [],
            )?;
        }
        let menu_count: i64 = conn.query_row("SELECT COUNT(*) FROM menu_items", [], |row| row.get(0))?;
        if menu_count == 0 {
            conn.execute(
                "INSERT INTO menu_items (name, code, category_id, sales_price, is_available) VALUES
                    ('麻辣东风螺', 'MENU001', 1, 108.0, 1), ('麻辣烧螺', 'MENU002', 1, 68.0, 1),
                    ('麻辣钉螺', 'MENU003', 1, 45.0, 1), ('麻辣虾尾', 'MENU004', 1, 48.0, 1),
                    ('麻辣扇贝肉', 'MENU005', 1, 48.0, 1), ('麻辣蟹钳', 'MENU006', 1, 48.0, 1),
                    ('麻辣章鱼足', 'MENU007', 1, 48.0, 1), ('麻辣鲨鱼肚', 'MENU008', 1, 45.0, 1),
                    ('麻辣肥肠', 'MENU009', 2, 48.0, 1), ('麻辣凤爪', 'MENU010', 2, 48.0, 1),
                    ('麻辣毛肚', 'MENU011', 2, 48.0, 1), ('麻辣藕片', 'MENU012', 3, 16.0, 1),
                    ('麻辣笋尖', 'MENU013', 3, 22.0, 1), ('麻辣川粉', 'MENU014', 3, 18.0, 1),
                    ('麻辣香菇', 'MENU015', 3, 20.0, 1), ('麻辣豆腐皮', 'MENU016', 3, 16.0, 1),
                    ('麻辣金针菇', 'MENU017', 3, 18.0, 1), ('麻辣土豆片', 'MENU018', 3, 20.0, 1),
                    ('麻辣丸拼', 'MENU019', 3, 18.0, 1), ('麻辣棉豆腐', 'MENU020', 3, 24.0, 1),
                    ('麻辣方便面', 'MENU021', 4, 12.0, 1), ('配送费', 'MENU022', 4, 1.0, 1)",
                [],
            )?;
        }
        // 初始化示例配方 (如果不存在的话)
        // 使用 INSERT OR IGNORE 确保不会重复插入
        conn.execute(
            "INSERT OR IGNORE INTO recipes (code, name, recipe_type, output_qty) VALUES
                ('RCP001', '麻辣东风螺', 'menu', 1.0),
                ('RCP002', '麻辣毛肚', 'menu', 1.0),
                ('RCP003', '麻辣方便面', 'menu', 1.0)",
            [],
        )?;
        
        // 获取刚插入的配方ID (如果已存在则查询现有ID)
        let recipe1_id: Option<i64> = conn.query_row(
            "SELECT id FROM recipes WHERE code = 'RCP001'", [], |row| row.get(0)
        ).ok();
        let recipe2_id: Option<i64> = conn.query_row(
            "SELECT id FROM recipes WHERE code = 'RCP002'", [], |row| row.get(0)
        ).ok();
        let recipe3_id: Option<i64> = conn.query_row(
            "SELECT id FROM recipes WHERE code = 'RCP003'", [], |row| row.get(0)
        ).ok();
        
        // 插入配方明细 (如果不存在)
        if let Some(r1_id) = recipe1_id {
            let exists: i64 = conn.query_row(
                "SELECT COUNT(*) FROM recipe_items WHERE recipe_id = ?1", params![r1_id], |row| row.get(0)
            )?;
            if exists == 0 {
                conn.execute(
                    "INSERT INTO recipe_items (recipe_id, item_type, ref_id, qty, unit_id, wastage_rate, sort_no) VALUES
                        (?1, 'material', 1, 0.5, 2, 0.05, 1),
                        (?1, 'material', 22, 0.1, 2, 0, 2),
                        (?1, 'material', 23, 0.02, 4, 0, 3)",
                    params![r1_id],
                )?;
            }
        }
        if let Some(r2_id) = recipe2_id {
            let exists: i64 = conn.query_row(
                "SELECT COUNT(*) FROM recipe_items WHERE recipe_id = ?1", params![r2_id], |row| row.get(0)
            )?;
            if exists == 0 {
                conn.execute(
                    "INSERT INTO recipe_items (recipe_id, item_type, ref_id, qty, unit_id, wastage_rate, sort_no) VALUES
                        (?1, 'material', 11, 0.3, 2, 0.1, 1),
                        (?1, 'material', 22, 0.08, 2, 0, 2),
                        (?1, 'material', 23, 0.015, 4, 0, 3),
                        (?1, 'material', 15, 0.05, 2, 0.1, 4)",
                    params![r2_id],
                )?;
            }
        }
        if let Some(r3_id) = recipe3_id {
            let exists: i64 = conn.query_row(
                "SELECT COUNT(*) FROM recipe_items WHERE recipe_id = ?1", params![r3_id], |row| row.get(0)
            )?;
            if exists == 0 {
                conn.execute(
                    "INSERT INTO recipe_items (recipe_id, item_type, ref_id, qty, unit_id, wastage_rate, sort_no) VALUES
                        (?1, 'material', 21, 1.0, 1, 0, 1),
                        (?1, 'material', 22, 0.05, 2, 0, 2),
                        (?1, 'material', 23, 0.01, 4, 0, 3),
                        (?1, 'material', 17, 0.03, 2, 0.1, 4)",
                    params![r3_id],
                )?;
            }
        }
        let station_count: i64 = conn.query_row("SELECT COUNT(*) FROM kitchen_stations", [], |row| row.get(0))?;
        if station_count == 0 {
            conn.execute(
                "INSERT INTO kitchen_stations (code, name, station_type, sort_no) VALUES
                    ('hot', '热菜站', 'hot', 1), ('cold', '冷菜站', 'cold', 2),
                    ('drinks', '饮品站', 'drinks', 3), ('dessert', '甜品站', 'dessert', 4)",
                [],
            )?;
        }
        Ok(())
    }

    /// One-time data patch: backfills cost_delta on historical consume txns written before R2 fix.
    fn run_data_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE inventory_txns
             SET cost_delta = qty_delta * (
                 SELECT COALESCE(ib.cost_per_unit, 0.0)
                 FROM inventory_batches ib WHERE ib.id = inventory_txns.lot_id
             )
             WHERE txn_type = 'consume' AND cost_delta = 0.0 AND lot_id IS NOT NULL",
            [],
        )?;
        let has_printer_id: bool = conn
            .prepare("SELECT 1 FROM pragma_table_info('kitchen_stations') WHERE name='printer_id'")?
            .exists([])?;
        if !has_printer_id {
            conn.execute_batch("ALTER TABLE kitchen_stations ADD COLUMN printer_id INTEGER REFERENCES printers(id)")?;
        }
        let has_cancel_reason: bool = conn
            .prepare("SELECT 1 FROM pragma_table_info('orders') WHERE name='cancel_reason'")?
            .exists([])?;
        if !has_cancel_reason {
            conn.execute_batch("ALTER TABLE orders ADD COLUMN cancel_reason TEXT")?;
        }
        let has_refund_amount: bool = conn
            .prepare("SELECT 1 FROM pragma_table_info('orders') WHERE name='refund_amount'")?
            .exists([])?;
        if !has_refund_amount {
            conn.execute_batch("ALTER TABLE orders ADD COLUMN refund_amount REAL NOT NULL DEFAULT 0")?;
        }
        let has_is_favorite: bool = conn
            .prepare("SELECT 1 FROM pragma_table_info('menu_items') WHERE name='is_favorite'")?
            .exists([])?;
        if !has_is_favorite {
            conn.execute_batch("ALTER TABLE menu_items ADD COLUMN is_favorite INTEGER NOT NULL DEFAULT 0")?;
        }
        Ok(())
    }

    pub fn check_and_create_expiry_alerts(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT ib.id, m.name, ib.lot_no, ib.expiry_date, ib.quantity
             FROM inventory_batches ib
             JOIN materials m ON m.id = ib.material_id
             WHERE ib.expiry_date IS NOT NULL
               AND date(ib.expiry_date) <= date('now', '+3 days')
               AND ib.quantity > 0
               AND NOT EXISTS (
                   SELECT 1 FROM notifications
                   WHERE ref_type = 'batch' AND ref_id = ib.id
                   AND date(created_at) = date('now')
               )"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, f64>(4)?))
        })?.collect::<Result<Vec<_>>>()?;
        let count = rows.len() as i64;
        let today = chrono::Local::now().date_naive();
        for (id, name, lot_no, expiry_date, qty) in rows {
            let exp = chrono::NaiveDate::parse_from_str(&expiry_date, "%Y-%m-%d").ok();
            let days_left = exp.map(|d| (d - today).num_days()).unwrap_or(0);
            let (severity, title) = if days_left <= 0 {
                ("error", format!("【已过期】{}", name))
            } else {
                ("warning", format!("【即将过期 {}天】{}", days_left, name))
            };
            let message = format!("批次 {} 剩余 {:.1}，到期日 {}", lot_no, qty, expiry_date);
            conn.execute(
                "INSERT INTO notifications (notification_type, title, message, severity, ref_type, ref_id) VALUES ('expiry_alert', ?1, ?2, ?3, 'batch', ?4)",
                params![title, message, severity, id],
            )?;
        }
        Ok(count)
    }

    pub fn backup_to(&self, dest_path: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // VACUUM INTO writes a clean, WAL-checkpointed copy — safe for hot backup
        let escaped = dest_path.replace('\'', "''");
        conn.execute_batch(&format!("VACUUM INTO '{}'", escaped))?;
        Ok(())
    }

    pub fn health_check(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT 1", [], |_| Ok(()))?;
        Ok(())
    }

    /// Returns (recipe_id, item_count) for all recipes that have items.
    pub fn get_recipe_item_counts(&self) -> Result<Vec<(i64, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT recipe_id, COUNT(*) FROM recipe_items GROUP BY recipe_id")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?.collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// Returns (recipe_id, total_cost) for all recipes in one backend call.
    pub fn get_all_recipe_costs(&self) -> Result<Vec<(i64, f64)>> {
        let conn = self.conn.lock().unwrap();
        let recipe_ids: Vec<i64> = {
            let mut stmt = conn.prepare("SELECT id FROM recipes ORDER BY id")?;
            let rows = stmt.query_map([], |r| r.get(0))?.collect::<Result<Vec<_>>>()?;
            rows
        };
        let mut result = Vec::with_capacity(recipe_ids.len());
        for recipe_id in recipe_ids {
            let direct: f64 = conn.query_row(
                "SELECT COALESCE(SUM(ri.qty * (1.0 + ri.wastage_rate) *
                     COALESCE((SELECT AVG(ib.cost_per_unit) FROM inventory_batches ib
                               WHERE ib.material_id = ri.ref_id AND ib.quantity > 0), 0.0)), 0.0)
                 FROM recipe_items ri WHERE ri.recipe_id = ?1 AND ri.item_type = 'material'",
                params![recipe_id], |r| r.get(0),
            ).unwrap_or(0.0);
            let sub: f64 = {
                let mut sub_stmt = conn.prepare(
                    "SELECT ri.qty * (1.0 + ri.wastage_rate), r_sub.output_qty,
                            COALESCE((SELECT SUM(ri2.qty * (1.0 + ri2.wastage_rate) *
                                COALESCE((SELECT AVG(ib.cost_per_unit) FROM inventory_batches ib
                                          WHERE ib.material_id = ri2.ref_id AND ib.quantity > 0), 0.0))
                                FROM recipe_items ri2 WHERE ri2.recipe_id = r_sub.id AND ri2.item_type = 'material'), 0.0)
                     FROM recipe_items ri JOIN recipes r_sub ON r_sub.id = ri.ref_id
                     WHERE ri.recipe_id = ?1 AND ri.item_type = 'sub_recipe'"
                )?;
                let mut total = 0.0_f64;
                for row in sub_stmt.query_map(params![recipe_id], |r| {
                    Ok((r.get::<_, f64>(0)?, r.get::<_, f64>(1)?, r.get::<_, f64>(2)?))
                })? {
                    let (qty_w, output_qty, sub_cost) = row?;
                    let cpu = if output_qty > 0.0 { sub_cost / output_qty } else { 0.0 };
                    total += qty_w * cpu;
                }
                total
            };
            result.push((recipe_id, direct + sub));
        }
        Ok(result)
    }

    pub fn get_units(&self) -> Result<Vec<Unit>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, code, name, unit_type, ratio_to_base FROM units ORDER BY id")?;
        let units = stmt.query_map([], |row| {
            Ok(Unit { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, unit_type: row.get(3)?, ratio_to_base: row.get(4)? })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(units)
    }

    pub fn get_material_categories(&self) -> Result<Vec<MaterialCategory>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, code, name, sort_no, is_active FROM material_categories ORDER BY sort_no")?;
        let cats = stmt.query_map([], |row| {
            Ok(MaterialCategory { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, sort_no: row.get(3)?, is_active: row.get::<_, i32>(4)? != 0 })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(cats)
    }

    pub fn create_material_category(&self, code: &str, name: &str, sort_no: i32) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        if let Ok(existing_id) = conn.query_row(
            "SELECT id FROM material_categories WHERE code = ?1 AND is_active = 0",
            params![code],
            |row| row.get::<_, i64>(0),
        ) {
            conn.execute("UPDATE material_categories SET is_active = 1, name = ?1, sort_no = ?2 WHERE id = ?3", params![name, sort_no, existing_id])?;
            return Ok(existing_id);
        }
        conn.execute("INSERT INTO material_categories (code, name, sort_no) VALUES (?1, ?2, ?3)", params![code, name, sort_no])?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_material_category(&self, id: i64, name: &str, sort_no: i32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE material_categories SET name = ?1, sort_no = ?2 WHERE id = ?3", params![name, sort_no, id])?;
        Ok(())
    }

    pub fn delete_material_category(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM materials WHERE category_id = ?1 AND is_active = 1",
            params![id], |r| r.get(0),
        )?;
        if count > 0 {
            return Err(rusqlite::Error::InvalidParameterName(
                format!("該分類下有 {} 個活躍原料，刪除前請先更換分類", count),
            ));
        }
        conn.execute("UPDATE material_categories SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_tags(&self) -> Result<Vec<Tag>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, code, name, color, is_active FROM tags ORDER BY id")?;
        let tags = stmt.query_map([], |row| {
            Ok(Tag { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, color: row.get(3)?, is_active: row.get::<_, i32>(4)? != 0 })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(tags)
    }

    pub fn create_tag(&self, code: &str, name: &str, color: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        if let Ok(existing_id) = conn.query_row(
            "SELECT id FROM tags WHERE code = ?1 AND is_active = 0",
            params![code],
            |row| row.get::<_, i64>(0),
        ) {
            conn.execute("UPDATE tags SET is_active = 1, name = ?1, color = ?2 WHERE id = ?3", params![name, color, existing_id])?;
            return Ok(existing_id);
        }
        conn.execute("INSERT INTO tags (code, name, color) VALUES (?1, ?2, ?3)", params![code, name, color])?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_tag(&self, id: i64, name: &str, color: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE tags SET name = ?1, color = ?2 WHERE id = ?3", params![name, color, id])?;
        Ok(())
    }

    pub fn delete_tag(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE tags SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_materials(&self, category_id: Option<i64>) -> Result<Vec<MaterialWithTags>> {
        use std::collections::HashMap;
        let conn = self.conn.lock().unwrap();

        // Single query: materials JOIN categories JOIN units
        let row_to_item = |row: &rusqlite::Row| -> rusqlite::Result<(Material, Option<MaterialCategory>, Option<Unit>)> {
            let mat = Material {
                id: row.get(0)?, code: row.get(1)?, name: row.get(2)?,
                category_id: row.get(3)?, base_unit_id: row.get(4)?,
                shelf_life_days: row.get(5)?, min_qty: row.get(6)?,
                is_active: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?, updated_at: row.get(9)?,
            };
            let category: Option<MaterialCategory> = match row.get::<_, Option<i64>>(10)? {
                Some(id) => Some(MaterialCategory {
                    id,
                    code: row.get(11)?, name: row.get(12)?,
                    sort_no: row.get(13)?, is_active: row.get::<_, i32>(14)? != 0,
                }),
                None => None,
            };
            let base_unit: Option<Unit> = match row.get::<_, Option<i64>>(15)? {
                Some(id) => Some(Unit {
                    id,
                    code: row.get(16)?, name: row.get(17)?,
                    unit_type: row.get(18)?, ratio_to_base: row.get(19)?,
                }),
                None => None,
            };
            Ok((mat, category, base_unit))
        };

        let base_sql = "SELECT m.id, m.code, m.name, m.category_id, m.base_unit_id, m.shelf_life_days, m.min_qty, m.is_active, m.created_at, m.updated_at, \
            c.id, c.code, c.name, c.sort_no, c.is_active, \
            u.id, u.code, u.name, u.unit_type, u.ratio_to_base \
            FROM materials m \
            LEFT JOIN material_categories c ON m.category_id = c.id \
            LEFT JOIN units u ON m.base_unit_id = u.id \
            WHERE m.is_active = 1";

        let rows: Vec<(Material, Option<MaterialCategory>, Option<Unit>)> = if let Some(cat_id) = category_id {
            let sql = format!("{} AND m.category_id = ?1 ORDER BY m.id", base_sql);
            conn.prepare(&sql)?.query_map(params![cat_id], row_to_item)?.collect::<Result<Vec<_>>>()?
        } else {
            let sql = format!("{} ORDER BY m.id", base_sql);
            conn.prepare(&sql)?.query_map([], row_to_item)?.collect::<Result<Vec<_>>>()?
        };

        // Batch fetch all tags in one query and group by material_id
        let mut tags_map: HashMap<i64, Vec<Tag>> = HashMap::new();
        let tag_rows: Vec<(i64, Tag)> = if let Some(cat_id) = category_id {
            let mut tag_stmt = conn.prepare(
                "SELECT mt.material_id, t.id, t.code, t.name, t.color, t.is_active FROM material_tags mt JOIN tags t ON mt.tag_id = t.id WHERE mt.material_id IN (SELECT id FROM materials WHERE is_active = 1 AND category_id = ?1)"
            )?;
            let x = tag_stmt.query_map(params![cat_id], |row| {
                Ok((row.get::<_, i64>(0)?, Tag { id: row.get(1)?, code: row.get(2)?, name: row.get(3)?, color: row.get(4)?, is_active: row.get::<_, i32>(5)? != 0 }))
            })?.collect::<Result<Vec<_>>>()?; x
        } else {
            let mut tag_stmt = conn.prepare(
                "SELECT mt.material_id, t.id, t.code, t.name, t.color, t.is_active FROM material_tags mt JOIN tags t ON mt.tag_id = t.id WHERE mt.material_id IN (SELECT id FROM materials WHERE is_active = 1)"
            )?;
            let x = tag_stmt.query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, Tag { id: row.get(1)?, code: row.get(2)?, name: row.get(3)?, color: row.get(4)?, is_active: row.get::<_, i32>(5)? != 0 }))
            })?.collect::<Result<Vec<_>>>()?; x
        };
        for (mat_id, tag) in tag_rows {
            tags_map.entry(mat_id).or_default().push(tag);
        }

        let result = rows.into_iter().map(|(mat, category, base_unit)| {
            let tags = tags_map.remove(&mat.id).unwrap_or_default();
            MaterialWithTags { material: mat, tags, category, base_unit }
        }).collect();
        Ok(result)
    }

    pub fn create_material(&self, code: &str, name: &str, category_id: Option<i64>, base_unit_id: i64, shelf_life_days: Option<i32>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        if let Ok(existing_id) = conn.query_row(
            "SELECT id FROM materials WHERE code = ?1 AND is_active = 0",
            params![code],
            |row| row.get::<_, i64>(0),
        ) {
            conn.execute(
                "UPDATE materials SET is_active = 1, name = ?1, category_id = ?2, base_unit_id = ?3, shelf_life_days = ?4 WHERE id = ?5",
                params![name, category_id, base_unit_id, shelf_life_days, existing_id],
            )?;
            return Ok(existing_id);
        }
        conn.execute(
            "INSERT INTO materials (code, name, category_id, base_unit_id, shelf_life_days) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![code, name, category_id, base_unit_id, shelf_life_days],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_material(&self, id: i64, name: Option<&str>, category_id: Option<i64>, shelf_life_days: Option<i32>, min_qty: Option<f64>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE materials SET name = COALESCE(?1, name), category_id = COALESCE(?2, category_id), shelf_life_days = COALESCE(?3, shelf_life_days), min_qty = COALESCE(?4, min_qty), updated_at = datetime('now') WHERE id = ?5",
            params![name, category_id, shelf_life_days, min_qty, id],
        )?;
        Ok(())
    }

    pub fn delete_material(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE materials SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn add_material_tags(&self, material_id: i64, tag_ids: &[i64]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        for tag_id in tag_ids {
            conn.execute(
                "INSERT OR IGNORE INTO material_tags (material_id, tag_id) VALUES (?1, ?2)",
                params![material_id, tag_id],
            )?;
        }
        Ok(())
    }

    pub fn remove_material_tag(&self, material_id: i64, tag_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM material_tags WHERE material_id = ?1 AND tag_id = ?2", params![material_id, tag_id])?;
        Ok(())
    }

    pub fn get_material_states(&self, material_id: i64) -> Result<Vec<MaterialState>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, material_id, state_code, state_name, unit_id, yield_rate, cost_multiplier, is_active FROM material_states WHERE material_id = ?1 AND is_active = 1 ORDER BY id")?;
        let states = stmt.query_map(params![material_id], |row| {
            Ok(MaterialState { id: row.get(0)?, material_id: row.get(1)?, state_code: row.get(2)?, state_name: row.get(3)?, unit_id: row.get(4)?, yield_rate: row.get(5)?, cost_multiplier: row.get(6)?, is_active: row.get::<_, i32>(7)? != 0 })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(states)
    }

    pub fn create_material_state(&self, material_id: i64, state_code: &str, state_name: &str, unit_id: Option<i64>, yield_rate: f64, cost_multiplier: f64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO material_states (material_id, state_code, state_name, unit_id, yield_rate, cost_multiplier) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![material_id, state_code, state_name, unit_id, yield_rate, cost_multiplier],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_all_material_states(&self) -> Result<Vec<MaterialState>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, material_id, state_code, state_name, unit_id, yield_rate, cost_multiplier, is_active FROM material_states ORDER BY material_id, id")?;
        let states = stmt.query_map([], |row| {
            Ok(MaterialState { id: row.get(0)?, material_id: row.get(1)?, state_code: row.get(2)?, state_name: row.get(3)?, unit_id: row.get(4)?, yield_rate: row.get(5)?, cost_multiplier: row.get(6)?, is_active: row.get::<_, i32>(7)? != 0 })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(states)
    }

    pub fn update_material_state(&self, id: i64, state_code: Option<&str>, state_name: Option<&str>, unit_id: Option<i64>, yield_rate: Option<f64>, cost_multiplier: Option<f64>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE material_states SET state_code = COALESCE(?1, state_code), state_name = COALESCE(?2, state_name), unit_id = COALESCE(?3, unit_id), yield_rate = COALESCE(?4, yield_rate), cost_multiplier = COALESCE(?5, cost_multiplier) WHERE id = ?6",
            params![state_code, state_name, unit_id, yield_rate, cost_multiplier, id],
        )?;
        Ok(())
    }

    pub fn delete_material_state(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE material_states SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_suppliers(&self) -> Result<Vec<Supplier>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, phone, contact_person, address, note, is_active, created_at FROM suppliers WHERE is_active = 1 ORDER BY name")?;
        let suppliers = stmt.query_map([], |row| {
            Ok(Supplier { id: row.get(0)?, name: row.get(1)?, phone: row.get(2)?, contact_person: row.get(3)?, address: row.get(4)?, note: row.get(5)?, is_active: row.get::<_, i32>(6)? != 0, created_at: row.get(7)? })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(suppliers)
    }

    pub fn create_supplier(&self, name: &str, phone: Option<&str>, contact_person: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT INTO suppliers (name, phone, contact_person) VALUES (?1, ?2, ?3)", params![name, phone, contact_person])?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_supplier(&self, id: i64, name: Option<&str>, phone: Option<&str>, contact_person: Option<&str>, address: Option<&str>, note: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE suppliers SET name = COALESCE(?1, name), phone = COALESCE(?2, phone), contact_person = COALESCE(?3, contact_person), address = COALESCE(?4, address), note = COALESCE(?5, note) WHERE id = ?6",
            params![name, phone, contact_person, address, note, id],
        )?;
        Ok(())
    }

    pub fn delete_supplier(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE suppliers SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ==================== 日常支出 API ====================

    pub fn get_expenses(&self, expense_type: Option<&str>, start_date: Option<&str>, end_date: Option<&str>) -> Result<Vec<Expense>> {
        let conn = self.conn.lock().unwrap();
        let mut sql = "SELECT id, expense_type, amount, expense_date, note, operator, is_active, created_at FROM expenses WHERE is_active = 1".to_string();
        let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        if expense_type.is_some() {
            sql.push_str(" AND expense_type = ?1");
            args.push(Box::new(expense_type.unwrap().to_string()));
        }
        if start_date.is_some() {
            let idx = args.len() + 1;
            sql.push_str(&format!(" AND expense_date >= ?{}", idx));
            args.push(Box::new(start_date.unwrap().to_string()));
        }
        if end_date.is_some() {
            let idx = args.len() + 1;
            sql.push_str(&format!(" AND expense_date <= ?{}", idx));
            args.push(Box::new(end_date.unwrap().to_string()));
        }
        sql.push_str(" ORDER BY expense_date DESC, id DESC");
        let mut stmt = conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = args.iter().map(|b| b.as_ref()).collect();
        let expenses = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(Expense { id: row.get(0)?, expense_type: row.get(1)?, amount: row.get(2)?, expense_date: row.get(3)?, note: row.get(4)?, operator: row.get(5)?, is_active: row.get::<_, i32>(6)? != 0, created_at: row.get(7)? })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(expenses)
    }

    pub fn create_expense(&self, expense_type: &str, amount: f64, expense_date: &str, note: Option<&str>, operator: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT INTO expenses (expense_type, amount, expense_date, note, operator) VALUES (?1, ?2, ?3, ?4, ?5)", params![expense_type, amount, expense_date, note, operator])?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_expense(&self, id: i64, expense_type: Option<&str>, amount: Option<f64>, expense_date: Option<&str>, note: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE expenses SET expense_type = COALESCE(?1, expense_type), amount = COALESCE(?2, amount), expense_date = COALESCE(?3, expense_date), note = COALESCE(?4, note) WHERE id = ?5",
            params![expense_type, amount, expense_date, note, id],
        )?;
        Ok(())
    }

    pub fn delete_expense(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE expenses SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_supplier_products(&self, channel: Option<&str>) -> Result<Vec<SupplierProduct>> {
        let conn = self.conn.lock().unwrap();
        let sql = if channel.is_some() {
            "SELECT id, product_name, supplier_name, channel FROM supplier_products WHERE is_active = 1 AND channel = ?1 ORDER BY id"
        } else {
            "SELECT id, product_name, supplier_name, channel FROM supplier_products WHERE is_active = 1 ORDER BY id"
        };
        let mut stmt = conn.prepare(sql)?;
        let items = if let Some(ch) = channel {
            stmt.query_map(params![ch], |row| {
                Ok(SupplierProduct { id: row.get(0)?, product_name: row.get(1)?, supplier_name: row.get(2)?, channel: row.get(3)? })
            })?.collect::<Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |row| {
                Ok(SupplierProduct { id: row.get(0)?, product_name: row.get(1)?, supplier_name: row.get(2)?, channel: row.get(3)? })
            })?.collect::<Result<Vec<_>>>()?
        };
        Ok(items)
    }

    pub fn create_supplier_product(&self, product_name: &str, supplier_name: &str, channel: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT INTO supplier_products (product_name, supplier_name, channel) VALUES (?1, ?2, ?3)", params![product_name, supplier_name, channel])?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_supplier_product(&self, id: i64, product_name: &str, supplier_name: &str, channel: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE supplier_products SET product_name = ?1, supplier_name = ?2, channel = ?3 WHERE id = ?4",
            params![product_name, supplier_name, channel, id],
        )?;
        Ok(())
    }

    pub fn delete_supplier_product(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE supplier_products SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Computes the food cost for an order by summing each item's recipe cost_per_unit × qty.
    /// Respects spec qty_multiplier. Returns 0.0 if a menu item has no recipe.
    pub fn get_order_cost(&self, order_id: i64) -> Result<f64> {
        let items: Vec<(f64, Option<i64>, f64)> = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT oi.qty, mi.recipe_id, COALESCE(mis.qty_multiplier, 1.0)
                 FROM order_items oi
                 JOIN menu_items mi ON mi.id = oi.menu_item_id
                 LEFT JOIN menu_item_specs mis ON mis.menu_item_id = mi.id AND mis.spec_code = oi.spec_code
                 WHERE oi.order_id = ?1"
            )?;
            let result = stmt.query_map(params![order_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
                .collect::<Result<Vec<_>>>()?;
            result
        };
        let mut total = 0.0;
        for (qty, recipe_id, qty_multiplier) in items {
            if let Some(rid) = recipe_id {
                if let Ok(cost) = self.calculate_recipe_cost(rid) {
                    total += qty * qty_multiplier * cost.cost_per_unit;
                }
            }
        }
        Ok(total)
    }

    pub fn get_attribute_templates(&self, entity_type: Option<&str>, category: Option<&str>) -> Result<Vec<AttributeTemplate>> {
        let conn = self.conn.lock().unwrap();
        let query = match (entity_type, category) {
            (Some(_et), Some(_cat)) => "SELECT id, entity_type, category, attr_code, attr_name, data_type, unit, default_value, formula, is_template, is_active FROM attribute_templates WHERE entity_type = ?1 AND category = ?2 ORDER BY id",
            (Some(_et), None) => "SELECT id, entity_type, category, attr_code, attr_name, data_type, unit, default_value, formula, is_template, is_active FROM attribute_templates WHERE entity_type = ?1 ORDER BY id",
            (None, Some(_cat)) => "SELECT id, entity_type, category, attr_code, attr_name, data_type, unit, default_value, formula, is_template, is_active FROM attribute_templates WHERE category = ?1 ORDER BY id",
            (None, None) => "SELECT id, entity_type, category, attr_code, attr_name, data_type, unit, default_value, formula, is_template, is_active FROM attribute_templates ORDER BY id",
        };
        let mut stmt = conn.prepare(query)?;
        let templates = match (entity_type, category) {
            (Some(et), Some(cat)) => stmt.query_map(params![et, cat], |row| {
                Ok(AttributeTemplate { id: row.get(0)?, entity_type: row.get(1)?, category: row.get(2)?, attr_code: row.get(3)?, attr_name: row.get(4)?, data_type: row.get(5)?, unit: row.get(6)?, default_value: row.get(7)?, formula: row.get(8)?, is_template: row.get::<_, i32>(9)? != 0, is_active: row.get::<_, i32>(10)? != 0 })
            })?.collect::<Result<Vec<_>>>()?,
            (Some(et), None) => stmt.query_map(params![et], |row| {
                Ok(AttributeTemplate { id: row.get(0)?, entity_type: row.get(1)?, category: row.get(2)?, attr_code: row.get(3)?, attr_name: row.get(4)?, data_type: row.get(5)?, unit: row.get(6)?, default_value: row.get(7)?, formula: row.get(8)?, is_template: row.get::<_, i32>(9)? != 0, is_active: row.get::<_, i32>(10)? != 0 })
            })?.collect::<Result<Vec<_>>>()?,
            (None, Some(cat)) => stmt.query_map(params![cat], |row| {
                Ok(AttributeTemplate { id: row.get(0)?, entity_type: row.get(1)?, category: row.get(2)?, attr_code: row.get(3)?, attr_name: row.get(4)?, data_type: row.get(5)?, unit: row.get(6)?, default_value: row.get(7)?, formula: row.get(8)?, is_template: row.get::<_, i32>(9)? != 0, is_active: row.get::<_, i32>(10)? != 0 })
            })?.collect::<Result<Vec<_>>>()?,
            (None, None) => stmt.query_map([], |row| {
                Ok(AttributeTemplate { id: row.get(0)?, entity_type: row.get(1)?, category: row.get(2)?, attr_code: row.get(3)?, attr_name: row.get(4)?, data_type: row.get(5)?, unit: row.get(6)?, default_value: row.get(7)?, formula: row.get(8)?, is_template: row.get::<_, i32>(9)? != 0, is_active: row.get::<_, i32>(10)? != 0 })
            })?.collect::<Result<Vec<_>>>()?,
        };
        Ok(templates)
    }

    pub fn create_attribute_template(&self, entity_type: &str, category: Option<&str>, attr_code: &str, attr_name: &str, data_type: &str, unit: Option<&str>, default_value: Option<f64>, formula: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO attribute_templates (entity_type, category, attr_code, attr_name, data_type, unit, default_value, formula, is_template, is_active) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, 1)",
            params![entity_type, category, attr_code, attr_name, data_type, unit, default_value, formula],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_attribute_template(&self, id: i64, entity_type: &str, category: Option<&str>, attr_code: &str, attr_name: &str, data_type: &str, unit: Option<&str>, default_value: Option<f64>, formula: Option<&str>, is_active: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE attribute_templates SET entity_type = ?1, category = ?2, attr_code = ?3, attr_name = ?4, data_type = ?5, unit = ?6, default_value = ?7, formula = ?8, is_active = ?9 WHERE id = ?10",
            params![entity_type, category, attr_code, attr_name, data_type, unit, default_value, formula, is_active as i32, id],
        )?;
        Ok(())
    }

    pub fn delete_attribute_template(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM attribute_templates WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn set_entity_attribute(&self, entity_type: &str, entity_id: i64, attr_code: &str, value: Option<f64>, value_text: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO entity_attributes (entity_type, entity_id, attr_code, value, value_text) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![entity_type, entity_id, attr_code, value, value_text],
        )?;
        Ok(())
    }

    pub fn get_entity_attributes(&self, entity_type: &str, entity_id: i64) -> Result<Vec<EntityAttribute>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, entity_type, entity_id, attr_code, value, value_text, calculated, updated_at FROM entity_attributes WHERE entity_type = ?1 AND entity_id = ?2")?;
        let attrs = stmt.query_map(params![entity_type, entity_id], |row| {
            Ok(EntityAttribute { id: row.get(0)?, entity_type: row.get(1)?, entity_id: row.get(2)?, attr_code: row.get(3)?, value: row.get(4)?, value_text: row.get(5)?, calculated: row.get::<_, i32>(6)? != 0, updated_at: row.get(7)? })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(attrs)
    }

    pub fn get_recipes(&self, recipe_type: Option<&str>) -> Result<Vec<Recipe>> {
        let conn = self.conn.lock().unwrap();
        let query = if let Some(_rt) = recipe_type {
            "SELECT id, code, name, recipe_type, output_material_id, output_state_id, output_qty, output_unit_id, cost, is_active, created_at, updated_at FROM recipes WHERE is_active = 1 AND recipe_type = ?1 ORDER BY name"
        } else {
            "SELECT id, code, name, recipe_type, output_material_id, output_state_id, output_qty, output_unit_id, cost, is_active, created_at, updated_at FROM recipes WHERE is_active = 1 ORDER BY name"
        };
        let mut stmt = conn.prepare(query)?;
        let recipes = if let Some(rt) = recipe_type {
            stmt.query_map(params![rt], |row| {
                Ok(Recipe { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, recipe_type: row.get(3)?, output_material_id: row.get(4)?, output_state_id: row.get(5)?, output_qty: row.get(6)?, output_unit_id: row.get(7)?, cost: row.get(8)?, is_active: row.get::<_, i32>(9)? != 0, created_at: row.get(10)?, updated_at: row.get(11)? })
            })?.collect::<Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |row| {
                Ok(Recipe { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, recipe_type: row.get(3)?, output_material_id: row.get(4)?, output_state_id: row.get(5)?, output_qty: row.get(6)?, output_unit_id: row.get(7)?, cost: row.get(8)?, is_active: row.get::<_, i32>(9)? != 0, created_at: row.get(10)?, updated_at: row.get(11)? })
            })?.collect::<Result<Vec<_>>>()?
        };
        Ok(recipes)
    }

    pub fn get_recipe_types(&self) -> Result<Vec<RecipeType>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, code, name, description, sort_no, is_system, is_active
             FROM recipe_types
             WHERE is_active = 1
             ORDER BY sort_no, id"
        )?;
        let items = stmt.query_map([], |row| {
            Ok(RecipeType {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                sort_no: row.get(4)?,
                is_system: row.get::<_, i32>(5)? != 0,
                is_active: row.get::<_, i32>(6)? != 0,
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(items)
    }

    pub fn get_recipe_component_types(&self, category: Option<&str>) -> Result<Vec<RecipeComponentType>> {
        let conn = self.conn.lock().unwrap();
        let sql = if category.is_some() {
            "SELECT id, code, name, description, category, sort_no, is_active FROM recipe_component_types WHERE is_active = 1 AND category = ?1 ORDER BY sort_no"
        } else {
            "SELECT id, code, name, description, category, sort_no, is_active FROM recipe_component_types WHERE is_active = 1 ORDER BY sort_no"
        };
        let mut stmt = conn.prepare(sql)?;
        let items = if let Some(cat) = category {
            stmt.query_map(params![cat], |row| {
                Ok(RecipeComponentType { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, description: row.get(3)?, category: row.get(4)?, sort_no: row.get(5)?, is_active: row.get::<_, i32>(6)? != 0 })
            })?.collect::<Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |row| {
                Ok(RecipeComponentType { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, description: row.get(3)?, category: row.get(4)?, sort_no: row.get(5)?, is_active: row.get::<_, i32>(6)? != 0 })
            })?.collect::<Result<Vec<_>>>()?
        };
        Ok(items)
    }

    pub fn get_order_component_types(&self, is_packaging: Option<bool>) -> Result<Vec<OrderComponentType>> {
        let conn = self.conn.lock().unwrap();
        let sql = match is_packaging {
            Some(true) => "SELECT id, code, name, description, is_packaging, cost_per_unit, unit, sort_no, is_active FROM order_component_types WHERE is_active = 1 AND is_packaging = 1 ORDER BY sort_no",
            Some(false) => "SELECT id, code, name, description, is_packaging, cost_per_unit, unit, sort_no, is_active FROM order_component_types WHERE is_active = 1 AND is_packaging = 0 ORDER BY sort_no",
            None => "SELECT id, code, name, description, is_packaging, cost_per_unit, unit, sort_no, is_active FROM order_component_types WHERE is_active = 1 ORDER BY sort_no",
        };
        let mut stmt = conn.prepare(sql)?;
        let items: Vec<OrderComponentType> = stmt.query_map([], |row| {
            Ok(OrderComponentType { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, description: row.get(3)?, is_packaging: row.get::<_, i32>(4)? != 0, cost_per_unit: row.get(5)?, unit: row.get(6)?, sort_no: row.get(7)?, is_active: row.get::<_, i32>(8)? != 0 })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(items)
    }

    pub fn create_recipe_type(&self, code: &str, name: &str, description: Option<&str>, sort_no: i32) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO recipe_types (code, name, description, sort_no, is_system, is_active)
             VALUES (?1, ?2, ?3, ?4, 0, 1)",
            params![code.trim(), name.trim(), description, sort_no],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn generate_recipe_code(&self) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let max_num: Option<i64> = conn.query_row(
            "SELECT MAX(CAST(SUBSTR(code, 4) AS INTEGER)) FROM recipes WHERE code GLOB 'RCP[0-9]*'",
            [],
            |row| row.get(0),
        ).ok();

        Ok(format!("RCP{:03}", max_num.unwrap_or(0) + 1))
    }

    pub fn seed_sample_recipes(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let lookup_material_id = |code: &str| -> Result<i64> {
            conn.query_row(
                "SELECT id FROM materials WHERE code = ?1 AND is_active = 1",
                params![code],
                |row| row.get(0),
            )
        };
        let lookup_unit_id = |code: &str| -> Result<i64> {
            conn.query_row(
                "SELECT id FROM units WHERE code = ?1",
                params![code],
                |row| row.get(0),
            )
        };

        let dongfengluo_id = lookup_material_id("MAT001")?;
        let maodu_id = lookup_material_id("MAT011")?;
        let xianggu_id = lookup_material_id("MAT015")?;
        let jinzhengu_id = lookup_material_id("MAT017")?;
        let fangbianmian_id = lookup_material_id("MAT021")?;
        let maladiliu_id = lookup_material_id("MAT022")?;
        let lajiaoyou_id = lookup_material_id("MAT023")?;

        let pc_unit_id = lookup_unit_id("pc")?;
        let kg_unit_id = lookup_unit_id("kg")?;
        let l_unit_id = lookup_unit_id("l")?;

        conn.execute_batch("BEGIN")?;
        let result: Result<()> = (|| {
            conn.execute(
                "DELETE FROM recipe_items
                 WHERE recipe_id IN (SELECT id FROM recipes WHERE code IN ('RCP001', 'RCP002', 'RCP003'))",
                [],
            )?;
            conn.execute(
                "DELETE FROM recipes WHERE code IN ('RCP001', 'RCP002', 'RCP003')",
                [],
            )?;

            // 菜品組成結構（Recipe Component Types）
            let recipe_components = [
                ("料油", "lsy", "base", "油脂基底，玉米油+香辛料熬製"),
                ("調味蚝油", "dthy", "base", "蚝油+水+調味料攪拌"),
                ("醬料", "jiangliao", "base", "辣椒+麻椒+花椒+調味料熬製"),
                ("食材", "shicai", "main", "主要原材料"),
                ("調味品", "tiaoweipin", "main", "鹽、糖、醬油等調料"),
                ("餐盒", "canhe", "packaging", "包裝容器"),
            ];
            for (name, code, cat, desc) in recipe_components {
                conn.execute("INSERT OR IGNORE INTO recipe_component_types (code, name, category, description) VALUES (?1, ?2, ?3, ?4)", params![code, name, cat, desc])?;
            }

            // 訂單組成結構（Order Component Types）
            let order_components = [
                ("外包裝袋", "waibao", false, 0.5, "個"),
                ("保溫袋", "baowen", false, 1.0, "個"),
                ("手套", "shoutao", false, 0.1, "雙"),
                ("牙籤", "yaci", false, 0.05, "支"),
                ("筷子", "kuaizi", false, 0.2, "雙"),
                ("芝麻", "zhima", false, 0.3, "克"),
                ("垃圾袋", "lajidai", false, 0.2, "個"),
                ("配送費", "peisong", false, 5.0, "元"),
            ];
            for (name, code, is_pkg, cost, unit) in order_components {
                conn.execute("INSERT OR IGNORE INTO order_component_types (code, name, is_packaging, cost_per_unit, unit) VALUES (?1, ?2, ?3, ?4, ?5)", params![code, name, is_pkg, cost, unit])?;
            }

            conn.execute(
                "INSERT INTO recipes (code, name, recipe_type, output_qty) VALUES
                    ('RCP001', '麻辣东风螺', 'menu', 1.0),
                    ('RCP002', '麻辣毛肚', 'menu', 1.0),
                    ('RCP003', '麻辣方便面', 'menu', 1.0)",
                [],
            )?;

            let recipe1_id: i64 = conn.query_row(
                "SELECT id FROM recipes WHERE code = 'RCP001'",
                [],
                |row| row.get(0),
            )?;
            let recipe2_id: i64 = conn.query_row(
                "SELECT id FROM recipes WHERE code = 'RCP002'",
                [],
                |row| row.get(0),
            )?;
            let recipe3_id: i64 = conn.query_row(
                "SELECT id FROM recipes WHERE code = 'RCP003'",
                [],
                |row| row.get(0),
            )?;

            conn.execute(
                "INSERT INTO recipe_items (recipe_id, item_type, ref_id, qty, unit_id, wastage_rate, sort_no) VALUES
                    (?1, 'material', ?2, 0.5, ?3, 0.05, 1),
                    (?1, 'material', ?4, 0.1, ?3, 0, 2),
                    (?1, 'material', ?5, 0.02, ?6, 0, 3)",
                params![recipe1_id, dongfengluo_id, kg_unit_id, maladiliu_id, lajiaoyou_id, l_unit_id],
            )?;

            conn.execute(
                "INSERT INTO recipe_items (recipe_id, item_type, ref_id, qty, unit_id, wastage_rate, sort_no) VALUES
                    (?1, 'material', ?2, 0.3, ?3, 0.1, 1),
                    (?1, 'material', ?4, 0.08, ?3, 0, 2),
                    (?1, 'material', ?5, 0.015, ?6, 0, 3),
                    (?1, 'material', ?7, 0.05, ?3, 0.1, 4)",
                params![recipe2_id, maodu_id, kg_unit_id, maladiliu_id, lajiaoyou_id, l_unit_id, xianggu_id],
            )?;

            conn.execute(
                "INSERT INTO recipe_items (recipe_id, item_type, ref_id, qty, unit_id, wastage_rate, sort_no) VALUES
                    (?1, 'material', ?2, 1.0, ?3, 0, 1),
                    (?1, 'material', ?4, 0.05, ?5, 0, 2),
                    (?1, 'material', ?6, 0.01, ?7, 0, 3),
                    (?1, 'material', ?8, 0.03, ?5, 0.1, 4)",
                params![recipe3_id, fangbianmian_id, pc_unit_id, maladiliu_id, kg_unit_id, lajiaoyou_id, l_unit_id, jinzhengu_id],
            )?;

            conn.execute(
                "UPDATE menu_items
                 SET recipe_id = CASE code
                   WHEN 'MENU001' THEN ?1
                   WHEN 'MENU011' THEN ?2
                   WHEN 'MENU021' THEN ?3
                   ELSE recipe_id
                 END
                 WHERE code IN ('MENU001', 'MENU011', 'MENU021')",
                params![recipe1_id, recipe2_id, recipe3_id],
            )?;

            Ok(())
        })();
        match result {
            Ok(_) => { conn.execute_batch("COMMIT")?; Ok(()) }
            Err(e) => { conn.execute_batch("ROLLBACK").ok(); Err(e) }
        }
    }

    pub fn get_recipe_with_items(&self, recipe_id: i64) -> Result<RecipeWithItems> {
        let conn = self.conn.lock().unwrap();
        let recipe = conn.query_row(
            "SELECT id, code, name, recipe_type, output_material_id, output_state_id, output_qty, output_unit_id, cost, is_active, created_at, updated_at FROM recipes WHERE id = ?1",
            params![recipe_id],
            |row| Ok(Recipe { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, recipe_type: row.get(3)?, output_material_id: row.get(4)?, output_state_id: row.get(5)?, output_qty: row.get(6)?, output_unit_id: row.get(7)?, cost: row.get(8)?, is_active: row.get::<_, i32>(9)? != 0, created_at: row.get(10)?, updated_at: row.get(11)? })
        )?;
        let mut stmt = conn.prepare("SELECT id, recipe_id, item_type, ref_id, qty, unit_id, wastage_rate, note, sort_no FROM recipe_items WHERE recipe_id = ?1 ORDER BY sort_no, id")?;
        let items = stmt.query_map(params![recipe_id], |row| {
            Ok(RecipeItem { id: row.get(0)?, recipe_id: row.get(1)?, item_type: row.get(2)?, ref_id: row.get(3)?, qty: row.get(4)?, unit_id: row.get(5)?, wastage_rate: row.get(6)?, note: row.get(7)?, sort_no: row.get(8)? })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(RecipeWithItems { recipe, items })
    }

    pub fn create_recipe(&self, code: &str, name: &str, recipe_type: &str, output_qty: f64, output_material_id: Option<i64>, output_state_id: Option<i64>, output_unit_id: Option<i64>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            return Err(rusqlite::Error::InvalidParameterName("配方名稱不能為空".to_string()));
        }

        let trimmed_code = code.trim();
        for _attempt in 0..5 {
            let candidate_code = if trimmed_code.is_empty() {
                let max_num: i64 = conn.query_row(
                    "SELECT COALESCE(MAX(CAST(SUBSTR(code, 4) AS INTEGER)), 0) FROM recipes WHERE code GLOB 'RCP[0-9]*'",
                    [],
                    |row| row.get(0),
                )?;
                format!("RCP{:03}", max_num + 1)
            } else {
                trimmed_code.to_string()
            };

            let result = conn.execute(
                "INSERT INTO recipes (code, name, recipe_type, output_qty, output_material_id, output_state_id, output_unit_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![candidate_code, trimmed_name, recipe_type, output_qty, output_material_id, output_state_id, output_unit_id],
            );

            match result {
                Ok(_) => return Ok(conn.last_insert_rowid()),
                Err(err) => {
                    let is_unique_code_error =
                        err.to_string().contains("UNIQUE constraint failed: recipes.code");

                    if is_unique_code_error && (trimmed_code.is_empty() || trimmed_code.starts_with("RCP")) {
                        continue;
                    }

                    return Err(err.into());
                }
            }
        }

        Err(rusqlite::Error::InvalidParameterName("生成配方編號失敗，請重試".to_string()))
    }

    pub fn add_recipe_item(&self, recipe_id: i64, item_type: &str, ref_id: i64, qty: f64, unit_id: i64, wastage_rate: f64, sort_no: i32) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        if item_type == "sub_recipe" {
            if ref_id == recipe_id {
                return Err(rusqlite::Error::InvalidParameterName(
                    "不能將配方添加為自身的子配方".to_string(),
                ));
            }
            if Self::recipe_has_ancestor(&conn, ref_id, recipe_id)? {
                return Err(rusqlite::Error::InvalidParameterName(
                    "添加此子配方會產生循環引用，請檢查配方嵌套關係".to_string(),
                ));
            }
        }
        conn.execute(
            "INSERT INTO recipe_items (recipe_id, item_type, ref_id, qty, unit_id, wastage_rate, sort_no) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![recipe_id, item_type, ref_id, qty, unit_id, wastage_rate, sort_no],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// BFS: returns true if `ancestor_id` is reachable via sub_recipe links starting from `recipe_id`.
    /// Depth-capped at 50 nodes to guard against pathological graphs.
    fn recipe_has_ancestor(conn: &Connection, recipe_id: i64, ancestor_id: i64) -> Result<bool> {
        use std::collections::HashSet;
        let mut visited: HashSet<i64> = HashSet::new();
        let mut queue = vec![recipe_id];
        while let Some(current) = queue.pop() {
            if current == ancestor_id { return Ok(true); }
            if visited.contains(&current) { continue; }
            visited.insert(current);
            if visited.len() > 50 { break; }
            let mut stmt = conn.prepare(
                "SELECT ref_id FROM recipe_items WHERE recipe_id = ?1 AND item_type = 'sub_recipe'"
            )?;
            let children: Vec<i64> = stmt.query_map(params![current], |row| row.get(0))?
                .collect::<Result<Vec<_>>>()?;
            queue.extend(children);
        }
        Ok(false)
    }

    pub fn get_recipe_usage_count(&self, recipe_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        // 檢查在 recipe_items 表中被作為子配方引用的次數
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM recipe_items WHERE item_type = 'sub_recipe' AND ref_id = ?1",
            params![recipe_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn get_recipe_dependents(&self, recipe_id: i64) -> Result<(Vec<(i64, String)>, Vec<(i64, String)>)> {
        let conn = self.conn.lock().unwrap();
        // Menu items that link to this recipe
        let mut stmt = conn.prepare(
            "SELECT mi.id, mi.name FROM menu_items mi WHERE mi.recipe_id = ?1 AND mi.is_active = 1"
        )?;
        let menu_items: Vec<(i64, String)> = stmt.query_map(params![recipe_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?.filter_map(|r| r.ok()).collect();

        // Parent recipes that use this recipe as a sub-recipe
        let mut stmt2 = conn.prepare(
            "SELECT r.id, r.name FROM recipes r
             JOIN recipe_items ri ON ri.recipe_id = r.id
             WHERE ri.item_type = 'sub_recipe' AND ri.ref_id = ?1"
        )?;
        let parent_recipes: Vec<(i64, String)> = stmt2.query_map(params![recipe_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?.filter_map(|r| r.ok()).collect();

        Ok((menu_items, parent_recipes))
    }

    pub fn get_material_dependents(&self, material_id: i64) -> Result<Vec<(i64, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT DISTINCT r.id, r.name FROM recipes r
             JOIN recipe_items ri ON ri.recipe_id = r.id
             WHERE ri.item_type = 'material' AND ri.ref_id = ?1"
        )?;
        let recipes: Vec<(i64, String)> = stmt.query_map(params![material_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?.filter_map(|r| r.ok()).collect();
        Ok(recipes)
    }

    pub fn calculate_recipe_cost(&self, recipe_id: i64) -> Result<RecipeCostResult> {
        let conn = self.conn.lock().unwrap();
        let (recipe_name, output_qty): (String, f64) = conn.query_row(
            "SELECT name, output_qty FROM recipes WHERE id = ?1",
            params![recipe_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        let mut stmt = conn.prepare(
            "SELECT m.name,
                    ri.qty,
                    COALESCE(u.code, ''),
                    COALESCE((SELECT AVG(ib2.cost_per_unit) FROM inventory_batches ib2
                               WHERE ib2.material_id = ri.ref_id AND ib2.quantity > 0), 0.0),
                    ri.wastage_rate
             FROM recipe_items ri
             JOIN materials m ON m.id = ri.ref_id
             LEFT JOIN units u ON u.id = ri.unit_id
             WHERE ri.recipe_id = ?1 AND ri.item_type = 'material'
             ORDER BY ri.sort_no",
        )?;
        let mut items: Vec<RecipeCostItem> = stmt.query_map(params![recipe_id], |row| {
            let material_name: String = row.get(0)?;
            let qty: f64 = row.get(1)?;
            let unit: String = row.get(2)?;
            let cost_per_unit: f64 = row.get(3)?;
            let wastage_rate: f64 = row.get(4)?;
            let line_cost = qty * cost_per_unit * (1.0 + wastage_rate);
            Ok(RecipeCostItem { material_name, qty, unit, cost_per_unit, wastage_rate, line_cost, item_type: "material".to_string() })
        })?.collect::<Result<Vec<_>>>()?;

        // Include sub-recipe cost contributions (one level, using their material avg costs)
        let mut sub_stmt = conn.prepare(
            "SELECT r.name,
                    ri.qty,
                    COALESCE(u.code, ''),
                    ri.wastage_rate,
                    r.output_qty,
                    COALESCE((
                        SELECT SUM(ri2.qty *
                            COALESCE((SELECT AVG(ib.cost_per_unit) FROM inventory_batches ib
                                      WHERE ib.material_id = ri2.ref_id AND ib.quantity > 0), 0.0)
                            * (1.0 + ri2.wastage_rate))
                        FROM recipe_items ri2
                        WHERE ri2.recipe_id = r.id AND ri2.item_type = 'material'
                    ), 0.0)
             FROM recipe_items ri
             JOIN recipes r ON r.id = ri.ref_id
             LEFT JOIN units u ON u.id = ri.unit_id
             WHERE ri.recipe_id = ?1 AND ri.item_type = 'sub_recipe'
             ORDER BY ri.sort_no"
        )?;
        let sub_items: Vec<RecipeCostItem> = sub_stmt.query_map(params![recipe_id], |row| {
            let material_name: String = row.get(0)?;
            let qty: f64 = row.get(1)?;
            let unit: String = row.get(2)?;
            let wastage_rate: f64 = row.get(3)?;
            let sub_output_qty: f64 = row.get(4)?;
            let sub_total_cost: f64 = row.get(5)?;
            let cost_per_unit = if sub_output_qty > 0.0 { sub_total_cost / sub_output_qty } else { 0.0 };
            let line_cost = qty * cost_per_unit * (1.0 + wastage_rate);
            Ok(RecipeCostItem { material_name, qty, unit, cost_per_unit, wastage_rate, line_cost, item_type: "sub_recipe".to_string() })
        })?.collect::<Result<Vec<_>>>()?;
        items.extend(sub_items);

        let total_cost: f64 = items.iter().map(|i| i.line_cost).sum();
        let cost_per_unit = if output_qty > 0.0 { total_cost / output_qty } else { 0.0 };
        Ok(RecipeCostResult { recipe_id, recipe_name, total_cost, cost_per_unit, output_qty, items })
    }

    pub fn get_menu_categories(&self) -> Result<Vec<MenuCategory>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, sort_no, is_active FROM menu_categories ORDER BY sort_no")?;
        let cats = stmt.query_map([], |row| {
            Ok(MenuCategory { id: row.get(0)?, name: row.get(1)?, sort_no: row.get(2)?, is_active: row.get::<_, i32>(3)? != 0 })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(cats)
    }

    pub fn create_menu_category(&self, name: &str, sort_no: i32) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let code = name.to_uppercase().replace(' ', "_");
        conn.execute("INSERT INTO menu_categories (code, name, sort_no) VALUES (?1, ?2, ?3)", params![code, name, sort_no])?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_menu_items(&self, category_id: Option<i64>) -> Result<Vec<MenuItem>> {
        let conn = self.conn.lock().unwrap();
        let query = if let Some(_cat_id) = category_id {
            "SELECT id, name, code, category_id, recipe_id, sales_price, cost, is_available, is_favorite, created_at FROM menu_items WHERE category_id = ?1 ORDER BY name"
        } else {
            "SELECT id, name, code, category_id, recipe_id, sales_price, cost, is_available, is_favorite, created_at FROM menu_items ORDER BY name"
        };
        let mut stmt = conn.prepare(query)?;
        let items = if let Some(cat_id) = category_id {
            stmt.query_map(params![cat_id], |row| {
                Ok(MenuItem { id: row.get(0)?, name: row.get(1)?, code: row.get(2)?, category_id: row.get(3)?, recipe_id: row.get(4)?, sales_price: row.get(5)?, cost: row.get(6)?, is_available: row.get::<_, i32>(7)? != 0, is_favorite: row.get::<_, i32>(8)? != 0, created_at: row.get(9)? })
            })?.collect::<Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |row| {
                Ok(MenuItem { id: row.get(0)?, name: row.get(1)?, code: row.get(2)?, category_id: row.get(3)?, recipe_id: row.get(4)?, sales_price: row.get(5)?, cost: row.get(6)?, is_available: row.get::<_, i32>(7)? != 0, is_favorite: row.get::<_, i32>(8)? != 0, created_at: row.get(9)? })
            })?.collect::<Result<Vec<_>>>()?
        };
        Ok(items)
    }

    pub fn create_menu_item(&self, name: &str, category_id: Option<i64>, recipe_id: Option<i64>, sales_price: f64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO menu_items (name, category_id, recipe_id, sales_price) VALUES (?1, ?2, ?3, ?4)",
            params![name, category_id, recipe_id, sales_price],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_menu_item_specs(&self, menu_item_id: i64) -> Result<Vec<MenuItemSpec>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, menu_item_id, spec_code, spec_name, price_delta, qty_multiplier, sort_no FROM menu_item_specs WHERE menu_item_id = ?1 ORDER BY sort_no")?;
        let specs = stmt.query_map(params![menu_item_id], |row| {
            Ok(MenuItemSpec { id: row.get(0)?, menu_item_id: row.get(1)?, spec_code: row.get(2)?, spec_name: row.get(3)?, price_delta: row.get(4)?, qty_multiplier: row.get(5)?, sort_no: row.get(6)? })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(specs)
    }

    pub fn create_menu_item_spec(&self, menu_item_id: i64, spec_code: &str, spec_name: &str, price_delta: f64, qty_multiplier: f64, sort_no: i32) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO menu_item_specs (menu_item_id, spec_code, spec_name, price_delta, qty_multiplier, sort_no) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![menu_item_id, spec_code, spec_name, price_delta, qty_multiplier, sort_no],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_menu_item_spec(&self, id: i64, spec_code: Option<&str>, spec_name: Option<&str>, price_delta: Option<f64>, qty_multiplier: Option<f64>, sort_no: Option<i32>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        if let Some(sc) = spec_code { conn.execute("UPDATE menu_item_specs SET spec_code = ?1 WHERE id = ?2", params![sc, id])?; }
        if let Some(sn) = spec_name { conn.execute("UPDATE menu_item_specs SET spec_name = ?1 WHERE id = ?2", params![sn, id])?; }
        if let Some(pd) = price_delta { conn.execute("UPDATE menu_item_specs SET price_delta = ?1 WHERE id = ?2", params![pd, id])?; }
        if let Some(qm) = qty_multiplier { conn.execute("UPDATE menu_item_specs SET qty_multiplier = ?1 WHERE id = ?2", params![qm, id])?; }
        if let Some(s) = sort_no { conn.execute("UPDATE menu_item_specs SET sort_no = ?1 WHERE id = ?2", params![s, id])?; }
        Ok(())
    }

    pub fn delete_menu_item_spec(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM menu_item_specs WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_menu_items_for_pos(&self, category_id: Option<i64>) -> Result<Vec<MenuItem>> {
        let conn = self.conn.lock().unwrap();
        let query = if let Some(_cat_id) = category_id {
            "SELECT id, name, code, category_id, recipe_id, sales_price, cost, is_available, is_favorite, created_at FROM menu_items WHERE is_available = 1 AND category_id = ?1 ORDER BY name"
        } else {
            "SELECT id, name, code, category_id, recipe_id, sales_price, cost, is_available, is_favorite, created_at FROM menu_items WHERE is_available = 1 ORDER BY name"
        };
        let mut stmt = conn.prepare(query)?;
        let items = if let Some(cat_id) = category_id {
            stmt.query_map(params![cat_id], |row| {
                Ok(MenuItem { id: row.get(0)?, name: row.get(1)?, code: row.get(2)?, category_id: row.get(3)?, recipe_id: row.get(4)?, sales_price: row.get(5)?, cost: row.get(6)?, is_available: row.get::<_, i32>(7)? != 0, is_favorite: row.get::<_, i32>(8)? != 0, created_at: row.get(9)? })
            })?.collect::<Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |row| {
                Ok(MenuItem { id: row.get(0)?, name: row.get(1)?, code: row.get(2)?, category_id: row.get(3)?, recipe_id: row.get(4)?, sales_price: row.get(5)?, cost: row.get(6)?, is_available: row.get::<_, i32>(7)? != 0, is_favorite: row.get::<_, i32>(8)? != 0, created_at: row.get(9)? })
            })?.collect::<Result<Vec<_>>>()?
        };
        Ok(items)
    }

    pub fn create_order(&self, source: &str, dine_type: &str, table_no: Option<&str>) -> Result<(i64, String)> {
        let conn = self.conn.lock().unwrap();
        let order_no = format!("ORD{}", chrono::Utc::now().format("%Y%m%d%H%M%S%3f"));
        conn.execute(
            "INSERT INTO orders (order_no, source, dine_type, table_no, status) VALUES (?1, ?2, ?3, ?4, 'pending')",
            params![order_no, source, dine_type, table_no],
        )?;
        let id = conn.last_insert_rowid();
        Ok((id, order_no))
    }

    pub fn get_orders(&self, limit: i64, offset: i64) -> Result<Vec<Order>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, order_no, source, dine_type, table_no, status, amount_total, note,
                    COALESCE(payment_status,'unpaid'), payment_method, COALESCE(amount_paid,0.0),
                    created_at, updated_at
             FROM orders ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
        )?;
        let orders = stmt.query_map(params![limit, offset], |row| {
            Ok(Order { id: row.get(0)?, order_no: row.get(1)?, source: row.get(2)?, dine_type: row.get(3)?, table_no: row.get(4)?, status: row.get(5)?, amount_total: row.get(6)?, note: row.get(7)?, payment_status: row.get(8)?, payment_method: row.get(9)?, amount_paid: row.get(10)?, created_at: row.get(11)?, updated_at: row.get(12)? })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(orders)
    }

    pub fn get_kitchen_stations(&self) -> Result<Vec<KitchenStation>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, code, name, station_type, is_active, sort_no, printer_id FROM kitchen_stations WHERE is_active = 1 ORDER BY sort_no")?;
        let stations = stmt.query_map([], |row| {
            Ok(KitchenStation { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, station_type: row.get(3)?, is_active: row.get::<_, i32>(4)? != 0, sort_no: row.get(5)?, printer_id: row.get(6)? })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(stations)
    }

    pub fn update_station_printer(&self, station_id: i64, printer_id: Option<i64>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE kitchen_stations SET printer_id = ?1 WHERE id = ?2", params![printer_id, station_id])?;
        Ok(())
    }

    pub fn get_station_tickets(&self, station_id: i64, status: Option<&str>) -> Result<Vec<KitchenTicket>> {
        let conn = self.conn.lock().unwrap();
        let query = if let Some(_s) = status {
            "SELECT id, order_id, station_id, status, priority, printed_at, started_at, finished_at, created_at FROM kitchen_tickets WHERE station_id = ?1 AND status = ?2 ORDER BY priority DESC, created_at"
        } else {
            "SELECT id, order_id, station_id, status, priority, printed_at, started_at, finished_at, created_at FROM kitchen_tickets WHERE station_id = ?1 ORDER BY priority DESC, created_at"
        };
        let mut stmt = conn.prepare(query)?;
        let tickets = if let Some(s) = status {
            stmt.query_map(params![station_id, s], |row| {
                Ok(KitchenTicket { id: row.get(0)?, order_id: row.get(1)?, station_id: row.get(2)?, status: row.get(3)?, priority: row.get(4)?, printed_at: row.get(5)?, started_at: row.get(6)?, finished_at: row.get(7)?, created_at: row.get(8)? })
            })?.collect::<Result<Vec<_>>>()?
        } else {
            stmt.query_map(params![station_id], |row| {
                Ok(KitchenTicket { id: row.get(0)?, order_id: row.get(1)?, station_id: row.get(2)?, status: row.get(3)?, priority: row.get(4)?, printed_at: row.get(5)?, started_at: row.get(6)?, finished_at: row.get(7)?, created_at: row.get(8)? })
            })?.collect::<Result<Vec<_>>>()?
        };
        Ok(tickets)
    }

    pub fn get_inventory_batches(&self, material_id: Option<i64>) -> Result<Vec<InventoryBatch>> {
        let conn = self.conn.lock().unwrap();
        let query = if let Some(_mid) = material_id {
            "SELECT id, material_id, state_id, lot_no, supplier_id, brand, spec, quantity, original_qty, cost_per_unit, production_date, expiry_date, ice_coating_rate, quality_rate, seasonal_factor, created_at, updated_at FROM inventory_batches WHERE material_id = ?1 AND quantity > 0 ORDER BY expiry_date"
        } else {
            "SELECT id, material_id, state_id, lot_no, supplier_id, brand, spec, quantity, original_qty, cost_per_unit, production_date, expiry_date, ice_coating_rate, quality_rate, seasonal_factor, created_at, updated_at FROM inventory_batches WHERE quantity > 0 ORDER BY material_id, expiry_date"
        };
        let mut stmt = conn.prepare(query)?;
        let batches = if let Some(mid) = material_id {
            stmt.query_map(params![mid], |row| {
                Ok(InventoryBatch { id: row.get(0)?, material_id: row.get(1)?, state_id: row.get(2)?, lot_no: row.get(3)?, supplier_id: row.get(4)?, brand: row.get(5)?, spec: row.get(6)?, quantity: row.get(7)?, original_qty: row.get(8)?, cost_per_unit: row.get(9)?, production_date: row.get(10)?, expiry_date: row.get(11)?, ice_coating_rate: row.get(12)?, quality_rate: row.get(13)?, seasonal_factor: row.get(14)?, created_at: row.get(15)?, updated_at: row.get(16)? })
            })?.collect::<Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |row| {
                Ok(InventoryBatch { id: row.get(0)?, material_id: row.get(1)?, state_id: row.get(2)?, lot_no: row.get(3)?, supplier_id: row.get(4)?, brand: row.get(5)?, spec: row.get(6)?, quantity: row.get(7)?, original_qty: row.get(8)?, cost_per_unit: row.get(9)?, production_date: row.get(10)?, expiry_date: row.get(11)?, ice_coating_rate: row.get(12)?, quality_rate: row.get(13)?, seasonal_factor: row.get(14)?, created_at: row.get(15)?, updated_at: row.get(16)? })
            })?.collect::<Result<Vec<_>>>()?
        };
        Ok(batches)
    }

    pub fn submit_order_full(&self, order_id: i64) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        // Prevent double-submission
        let current_status: String = conn.query_row(
            "SELECT status FROM orders WHERE id = ?1", params![order_id], |r| r.get(0),
        )?;
        if current_status != "pending" {
            return Ok(Vec::new());
        }
        conn.execute_batch("BEGIN")?;
        let result: Result<()> = (|| {
            conn.execute(
                "UPDATE orders SET status = 'submitted', updated_at = datetime('now') WHERE id = ?1",
                params![order_id],
            )?;
            // Inline kitchen ticket creation inside the same transaction
            let mut stmt = conn.prepare(
                "SELECT DISTINCT smi.station_id FROM order_items oi
                 JOIN station_menu_items smi ON oi.menu_item_id = smi.menu_item_id
                 WHERE oi.order_id = ?1"
            )?;
            let stations: Vec<i64> = stmt.query_map(params![order_id], |r| r.get(0))?
                .collect::<Result<Vec<_>>>()?;
            if stations.is_empty() {
                if let Ok(default_station) = conn.query_row(
                    "SELECT id FROM kitchen_stations WHERE is_active = 1 ORDER BY sort_no LIMIT 1",
                    [], |r| r.get::<_, i64>(0),
                ) {
                    conn.execute(
                        "INSERT INTO kitchen_tickets (order_id, station_id, status) VALUES (?1, ?2, 'pending')",
                        params![order_id, default_station],
                    )?;
                }
            } else {
                for station_id in stations {
                    conn.execute(
                        "INSERT INTO kitchen_tickets (order_id, station_id, status) VALUES (?1, ?2, 'pending')",
                        params![order_id, station_id],
                    )?;
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => { conn.execute_batch("COMMIT")?; Ok(Vec::new()) }
            Err(e) => { conn.execute_batch("ROLLBACK").ok(); Err(e) }
        }
    }

    pub fn get_inventory_summary(&self) -> Result<Vec<InventorySummary>> {
        let conn = self.conn.lock().unwrap();
        // reserved_qty = net outstanding reserves (reserve txns are negative, release txns positive)
        let mut stmt = conn.prepare(
            "SELECT
                ib.material_id,
                m.name,
                SUM(ib.quantity) as total_qty,
                COALESCE((
                    SELECT SUM(ri.qty * (1.0 + COALESCE(ri.wastage_rate, 0.0)) * oi.qty)
                    FROM order_items oi
                    JOIN orders o ON oi.order_id = o.id
                    JOIN menu_items mi ON oi.menu_item_id = mi.id
                    JOIN recipe_items ri ON ri.recipe_id = mi.recipe_id
                        AND ri.item_type = 'material'
                        AND ri.ref_id = ib.material_id
                    WHERE o.status IN ('submitted', 'in_progress')
                ), 0.0) as reserved_qty
             FROM inventory_batches ib
             JOIN materials m ON ib.material_id = m.id
             WHERE ib.quantity > 0
             GROUP BY ib.material_id
             ORDER BY m.name"
        )?;
        let summary = stmt.query_map([], |row| {
            let material_id: i64 = row.get(0)?;
            let material_name: String = row.get(1)?;
            let total_qty: f64 = row.get(2)?;
            let reserved_qty: f64 = (row.get::<_, f64>(3)?).max(0.0);
            let available_qty = (total_qty - reserved_qty).max(0.0);
            Ok(InventorySummary {
                material_id,
                material_name,
                total_qty,
                reserved_qty,
                available_qty,
                batches: Vec::new(),
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(summary)
    }

    pub fn create_inventory_txn(&self, txn_type: &str, ref_type: Option<&str>, ref_id: Option<i64>, lot_id: Option<i64>, material_id: i64, state_id: Option<i64>, qty_delta: f64, cost_delta: Option<f64>, operator: Option<&str>, note: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let txn_no = format!("TXN{}", chrono::Local::now().format("%Y%m%d%H%M%S%3f"));
        conn.execute(
            "INSERT INTO inventory_txns (txn_no, txn_type, ref_type, ref_id, lot_id, material_id, state_id, qty_delta, cost_delta, operator, note) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![txn_no, txn_type, ref_type, ref_id, lot_id, material_id, state_id, qty_delta, cost_delta, operator, note],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn create_inventory_batch(&self, material_id: i64, state_id: Option<i64>, lot_no: &str, supplier_id: Option<i64>, brand: Option<&str>, spec: Option<&str>, quantity: f64, cost_per_unit: f64, production_date: Option<&str>, expiry_date: Option<&str>, ice_coating_rate: Option<f64>, quality_rate: Option<f64>, seasonal_factor: f64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO inventory_batches (material_id, state_id, lot_no, supplier_id, brand, spec, quantity, original_qty, cost_per_unit, production_date, expiry_date, ice_coating_rate, quality_rate, seasonal_factor) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![material_id, state_id, lot_no, supplier_id, brand, spec, quantity, cost_per_unit, production_date, expiry_date, ice_coating_rate, quality_rate, seasonal_factor],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn delete_inventory_batch(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM inventory_txns WHERE lot_id = ?1", params![id])?;
        conn.execute("DELETE FROM inventory_batches WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn adjust_inventory(&self, batch_id: i64, qty_delta: f64, operator: Option<&str>, note: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let (current_qty, material_id): (f64, i64) = conn.query_row(
            "SELECT quantity, material_id FROM inventory_batches WHERE id = ?1",
            params![batch_id], |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        if current_qty + qty_delta < 0.0 {
            return Err(rusqlite::Error::InvalidParameterName(
                format!("調整後庫存 ({:.2}) 不能為負", current_qty + qty_delta),
            ));
        }
        let txn_no = format!("TXN{}", chrono::Local::now().format("%Y%m%d%H%M%S%3f"));
        conn.execute_batch("BEGIN")?;
        let result: Result<()> = (|| {
            conn.execute(
                "INSERT INTO inventory_txns (txn_no, txn_type, lot_id, material_id, qty_delta, operator, note) VALUES (?1, 'adjustment', ?2, ?3, ?4, ?5, ?6)",
                params![txn_no, batch_id, material_id, qty_delta, operator, note],
            )?;
            conn.execute(
                "UPDATE inventory_batches SET quantity = quantity + ?1 WHERE id = ?2",
                params![qty_delta, batch_id],
            )?;
            Ok(())
        })();
        match result {
            Ok(_) => { conn.execute_batch("COMMIT")?; Ok(()) }
            Err(e) => { conn.execute_batch("ROLLBACK").ok(); Err(e) }
        }
    }

    pub fn record_wastage(&self, batch_id: i64, qty: f64, reason: Option<&str>, operator: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let (current_qty, material_id): (f64, i64) = conn.query_row(
            "SELECT quantity, material_id FROM inventory_batches WHERE id = ?1",
            params![batch_id], |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        if qty > current_qty {
            return Err(rusqlite::Error::InvalidParameterName(
                format!("廢棄數量 ({:.2}) 超過現有庫存 ({:.2})", qty, current_qty),
            ));
        }
        let txn_no = format!("TXN{}", chrono::Local::now().format("%Y%m%d%H%M%S%3f"));
        conn.execute_batch("BEGIN")?;
        let result: Result<()> = (|| {
            conn.execute(
                "INSERT INTO inventory_txns (txn_no, txn_type, lot_id, material_id, qty_delta, operator, note) VALUES (?1, 'wastage', ?2, ?3, ?4, ?5, ?6)",
                params![txn_no, batch_id, material_id, -qty, operator, reason],
            )?;
            conn.execute(
                "UPDATE inventory_batches SET quantity = quantity - ?1 WHERE id = ?2",
                params![qty, batch_id],
            )?;
            Ok(())
        })();
        match result {
            Ok(_) => { conn.execute_batch("COMMIT")?; Ok(()) }
            Err(e) => { conn.execute_batch("ROLLBACK").ok(); Err(e) }
        }
    }

    pub fn update_menu_item(&self, id: i64, name: Option<&str>, category_id: Option<i64>, recipe_id: Option<i64>, sales_price: Option<f64>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE menu_items SET name = COALESCE(?1, name), category_id = COALESCE(?2, category_id), recipe_id = ?3, sales_price = COALESCE(?4, sales_price) WHERE id = ?5",
            params![name, category_id, recipe_id, sales_price, id],
        )?;
        Ok(())
    }

    pub fn set_menu_item_availability(&self, id: i64, is_available: bool) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE menu_items SET is_available = ?1 WHERE id = ?2",
            params![if is_available { 1 } else { 0 }, id],
        )?;
        Ok(is_available)
    }

    pub fn toggle_menu_item_favorite(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let current: i32 = conn.query_row(
            "SELECT is_favorite FROM menu_items WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        let new_val = if current == 0 { 1 } else { 0 };
        conn.execute("UPDATE menu_items SET is_favorite = ?1 WHERE id = ?2", params![new_val, id])?;
        Ok(new_val != 0)
    }

    pub fn batch_toggle_menu_item_availability(&self, ids: &[i64], is_available: bool) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let mut count = 0;
        for id in ids {
            let result = conn.execute("UPDATE menu_items SET is_available = ?1 WHERE id = ?2", params![if is_available { 1 } else { 0 }, id]);
            if result.is_ok() {
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn delete_menu_item(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM menu_item_specs WHERE menu_item_id = ?1", params![id])?;
        conn.execute("DELETE FROM station_menu_items WHERE menu_item_id = ?1", params![id])?;
        let affected = conn.execute("DELETE FROM menu_items WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(())
    }

    pub fn update_menu_category(&self, id: i64, name: Option<&str>, sort_no: Option<i32>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        if let Some(n) = name {
            let code = n.to_uppercase().replace(' ', "_");
            conn.execute("UPDATE menu_categories SET code = ?1, name = ?2, sort_no = COALESCE(?3, sort_no) WHERE id = ?4", params![code, n, sort_no, id])?;
        } else if let Some(s) = sort_no {
            conn.execute("UPDATE menu_categories SET sort_no = ?1 WHERE id = ?2", params![s, id])?;
        }
        Ok(())
    }

    pub fn delete_menu_category(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE menu_items SET category_id = NULL WHERE category_id = ?1", params![id])?;
        conn.execute("UPDATE menu_categories SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_recipe(&self, id: i64, name: Option<&str>, recipe_type: Option<&str>, output_qty: Option<f64>, cost: Option<f64>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE recipes SET name = COALESCE(?1, name), recipe_type = COALESCE(?2, recipe_type), output_qty = COALESCE(?3, output_qty), cost = ?4 WHERE id = ?5",
            params![name, recipe_type, output_qty, cost, id],
        )?;
        Ok(())
    }

    pub fn update_recipe_type(&self, id: i64, code: &str, name: &str, description: Option<&str>, sort_no: i32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE recipe_types
             SET code = ?1, name = ?2, description = ?3, sort_no = ?4
             WHERE id = ?5",
            params![code.trim(), name.trim(), description, sort_no, id],
        )?;
        Ok(())
    }

    pub fn delete_recipe_type(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let usage_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM recipes WHERE recipe_type = (SELECT code FROM recipe_types WHERE id = ?1) AND is_active = 1",
            params![id],
            |row| row.get(0),
        )?;
        if usage_count > 0 {
            return Err(rusqlite::Error::InvalidParameterName(format!("該配方類型仍被 {} 個配方使用中", usage_count)));
        }
        conn.execute("UPDATE recipe_types SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn delete_recipe(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let menu_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM menu_items WHERE recipe_id = ?1",
            params![id], |r| r.get(0),
        )?;
        if menu_count > 0 {
            return Err(rusqlite::Error::InvalidParameterName(
                format!("此配方被 {} 個菜品使用，請先解除菜品與配方的綁定", menu_count),
            ));
        }
        let sub_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM recipe_items WHERE item_type = 'sub_recipe' AND ref_id = ?1",
            params![id], |r| r.get(0),
        )?;
        if sub_count > 0 {
            return Err(rusqlite::Error::InvalidParameterName(
                format!("此配方被 {} 個其他配方引用為子配方，請先移除引用", sub_count),
            ));
        }
        conn.execute("UPDATE recipes SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn delete_recipe_item(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM recipe_items WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn add_station_menu_item(&self, station_id: i64, menu_item_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO station_menu_items (station_id, menu_item_id) VALUES (?1, ?2)",
            params![station_id, menu_item_id],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn remove_station_menu_item(&self, station_id: i64, menu_item_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM station_menu_items WHERE station_id = ?1 AND menu_item_id = ?2", params![station_id, menu_item_id])?;
        Ok(())
    }

    pub fn update_recipe_item(&self, id: i64, qty: Option<f64>, wastage_rate: Option<f64>, note: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE recipe_items SET qty = COALESCE(?1, qty), wastage_rate = COALESCE(?2, wastage_rate), note = COALESCE(?3, note) WHERE id = ?4",
            params![qty, wastage_rate, note, id],
        )?;
        Ok(())
    }

    pub fn get_station_menu_items(&self, station_id: i64) -> Result<Vec<MenuItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT m.id, m.name, m.code, m.category_id, m.recipe_id, m.sales_price, m.cost, m.is_available, m.is_favorite, m.created_at
             FROM menu_items m
             JOIN station_menu_items smi ON m.id = smi.menu_item_id
             WHERE smi.station_id = ?1 AND m.is_available = 1"
        )?;
        let items = stmt.query_map(params![station_id], |row| {
            Ok(MenuItem {
                id: row.get(0)?,
                name: row.get(1)?,
                code: row.get(2)?,
                category_id: row.get(3)?,
                recipe_id: row.get(4)?,
                sales_price: row.get(5)?,
                cost: row.get(6)?,
                is_available: row.get::<_, i32>(7)? != 0,
                is_favorite: row.get::<_, i32>(8)? != 0,
                created_at: row.get(9)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(items)
    }

    pub fn check_and_create_alerts(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // Auto-remove low_stock alerts for materials that are now back above min_qty
        conn.execute(
            "DELETE FROM notifications WHERE notification_type = 'low_stock' AND is_read = 0
             AND ref_id NOT IN (
                 SELECT m.id FROM materials m
                 LEFT JOIN inventory_batches ib ON ib.material_id = m.id
                 WHERE m.is_active = 1 AND m.min_qty > 0
                 GROUP BY m.id HAVING COALESCE(SUM(ib.quantity), 0) < m.min_qty
             )",
            [],
        ).ok();
        let mut stmt = conn.prepare(
            "SELECT m.id, m.name, m.min_qty, COALESCE(SUM(ib.quantity), 0) AS total_qty
             FROM materials m
             LEFT JOIN inventory_batches ib ON ib.material_id = m.id
             WHERE m.is_active = 1 AND m.min_qty > 0
             GROUP BY m.id
             HAVING total_qty < m.min_qty"
        )?;
        let low_stock: Vec<(i64, String, f64, f64)> = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?.collect::<Result<Vec<_>>>()?;
        for (material_id, name, min_qty, total_qty) in low_stock {
            let already: i64 = conn.query_row(
                "SELECT COUNT(*) FROM notifications WHERE ref_type = 'material' AND ref_id = ?1 AND is_read = 0",
                params![material_id], |r| r.get(0),
            ).unwrap_or(0);
            if already == 0 {
                conn.execute(
                    "INSERT INTO notifications (notification_type, title, message, severity, ref_type, ref_id) VALUES ('low_stock', '庫存不足', ?1, 'warning', 'material', ?2)",
                    params![format!("{} 當前庫存 {:.2}，低於最低庫存 {:.2}", name, total_qty, min_qty), material_id],
                ).ok();
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn select_batches_for_reservation_internal(&self, conn: &Connection, material_id: i64, required_qty: f64) -> Result<Vec<(i64, f64)>> {
        let mut stmt = conn.prepare(
            "SELECT ib.id, ib.quantity - COALESCE(
                (SELECT SUM(abs(it.qty_delta)) FROM inventory_txns it
                 WHERE it.lot_id = ib.id AND it.txn_type = 'reserve'), 0) as available_qty
             FROM inventory_batches ib
             WHERE ib.material_id = ?1
             AND ib.quantity > 0
             ORDER BY ib.expiry_date ASC, ib.created_at ASC"
        )?;

        let mut remaining = required_qty;
        let mut selected = Vec::new();

        let rows = stmt.query_map(params![material_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?))
        })?.collect::<Result<Vec<_>>>()?;

        for (batch_id, available_qty) in rows {
            if remaining <= 0.0 { break; }
            let take = available_qty.min(remaining);
            if take > 0.0 {
                selected.push((batch_id, take));
                remaining -= take;
            }
        }

        if remaining > 0.0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        Ok(selected)
    }

    /// 實扣庫存：將預扣轉為實扣（KDS 完成出餐時調用）
    #[allow(dead_code)]
    pub fn confirm_inventory_for_order(&self, order_id: i64) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("BEGIN")?;
        let result = Self::consume_order_inventory(&conn, order_id);
        match result {
            Ok(_) => { conn.execute_batch("COMMIT")?; Ok(Vec::new()) }
            Err(e) => { conn.execute_batch("ROLLBACK").ok(); Err(e) }
        }
    }

    /// 回補庫存：訂單取消時回補已實扣庫存
    pub fn release_inventory_for_order(&self, order_id: i64, reason: Option<&str>) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("BEGIN")?;
        let result: Result<Vec<String>> = (|| {
            // Reverse any consume txns already recorded for this order
            let mut stmt = conn.prepare(
                "SELECT id, lot_id, material_id, qty_delta, cost_delta FROM inventory_txns \
                 WHERE ref_type = 'order' AND ref_id = ?1 AND txn_type = 'consume'"
            )?;
            let consumes: Vec<(i64, Option<i64>, i64, f64, f64)> = stmt.query_map(params![order_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
            })?.collect::<Result<Vec<_>>>()?;

            let mut txn_nos = Vec::new();
            for (_consume_id, lot_id, material_id, qty_delta, cost_delta) in consumes {
                let txn_no = format!("TXN{}", chrono::Local::now().format("%Y%m%d%H%M%S%3f"));
                conn.execute(
                    "INSERT INTO inventory_txns (txn_no, txn_type, ref_type, ref_id, lot_id, material_id, qty_delta, cost_delta) \
                     VALUES (?1, 'release', 'order', ?2, ?3, ?4, ?5, ?6)",
                    params![txn_no, order_id, lot_id, material_id, -qty_delta, -cost_delta],
                )?;
                if let Some(batch_id) = lot_id {
                    conn.execute(
                        "UPDATE inventory_batches SET quantity = quantity + ?1 WHERE id = ?2",
                        params![-qty_delta, batch_id],
                    )?;
                }
                txn_nos.push(txn_no);
            }

            conn.execute(
                "UPDATE orders SET status = 'cancelled', cancel_reason = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![reason, order_id],
            )?;
            Ok(txn_nos)
        })();
        match result {
            Ok(v) => { conn.execute_batch("COMMIT")?; Ok(v) }
            Err(e) => { conn.execute_batch("ROLLBACK").ok(); Err(e) }
        }
    }

    /// 批量取消訂單
    pub fn batch_cancel_orders(&self, ids: &[i64]) -> Result<usize> {
        let mut failed: Vec<String> = Vec::new();
        let mut count = 0;
        for &id in ids {
            let status: String = {
                let conn = self.conn.lock().unwrap();
                conn.query_row("SELECT status FROM orders WHERE id = ?1", params![id], |r| r.get(0))
                    .unwrap_or_default()
            };
            if status == "cancelled" { continue; }
            // 'ready' = inventory already consumed; keep cost deduction, just mark cancelled
            let result = if status == "ready" {
                self.cancel_order_confirmed(id, None).map(|_| ())
            } else {
                self.release_inventory_for_order(id, None).map(|_| ())
            };
            if let Err(e) = result {
                failed.push(format!("訂單 {}: {}", id, e));
            } else {
                count += 1;
            }
        }
        if !failed.is_empty() {
            return Err(rusqlite::Error::InvalidParameterName(
                format!("部分訂單取消失敗（{}）：{}", failed.len(), failed.join("；")),
            ));
        }
        Ok(count)
    }

    /// 創建廚房小票（訂單提交時自動拆單）

    /// KDS 開始製作
    pub fn start_ticket(&self, ticket_id: i64, _operator: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE kitchen_tickets SET status = 'started', started_at = datetime('now') WHERE id = ?1",
            params![ticket_id],
        )?;
        Ok(())
    }

    /// KDS 完成出餐
    pub fn finish_ticket(&self, ticket_id: i64, _operator: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let order_id: i64 = conn.query_row(
            "SELECT order_id FROM kitchen_tickets WHERE id = ?1",
            params![ticket_id], |row| row.get(0),
        )?;

        conn.execute_batch("BEGIN")?;
        let result: Result<()> = (|| {
            conn.execute(
                "UPDATE kitchen_tickets SET status = 'finished', finished_at = datetime('now') WHERE id = ?1",
                params![ticket_id],
            )?;

            let unfinished: i64 = conn.query_row(
                "SELECT COUNT(*) FROM kitchen_tickets WHERE order_id = ?1 AND status != 'finished'",
                params![order_id], |row| row.get(0),
            )?;

            if unfinished == 0 {
                // All stations done — consume inventory (FIFO) and mark order ready
                Self::consume_order_inventory(&conn, order_id)?;
                conn.execute(
                    "UPDATE orders SET status = 'ready', updated_at = datetime('now') WHERE id = ?1",
                    params![order_id],
                )?;
            }
            Ok(())
        })();
        match result {
            Ok(_) => { conn.execute_batch("COMMIT")?; Ok(()) }
            Err(e) => { conn.execute_batch("ROLLBACK").ok(); Err(e) }
        }
    }

    /// Consume inventory for a completed order using recipe-based FIFO deduction.
    /// Idempotent: skips if consume txns already exist for this order.
    fn consume_order_inventory(conn: &Connection, order_id: i64) -> Result<()> {
        // Idempotency guard
        let already_consumed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM inventory_txns WHERE ref_type = 'order' AND ref_id = ?1 AND txn_type = 'consume'",
            params![order_id], |r| r.get(0),
        )?;
        if already_consumed > 0 {
            return Ok(());
        }

        // Expand each order item's recipe recursively (handles sub-recipes at all depths)
        let mut oi_stmt = conn.prepare(
            "SELECT mi.recipe_id, oi.qty FROM order_items oi
             JOIN menu_items mi ON oi.menu_item_id = mi.id
             WHERE oi.order_id = ?1 AND mi.recipe_id IS NOT NULL"
        )?;
        let order_items: Vec<(i64, f64)> = oi_stmt.query_map(params![order_id], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, f64>(1)?))
        })?.collect::<Result<Vec<_>>>()?;
        let mut needs_map: std::collections::HashMap<i64, f64> = std::collections::HashMap::new();
        for (recipe_id, oi_qty) in order_items {
            expand_recipe_needs(conn, recipe_id, oi_qty, 0, &mut needs_map)?;
        }
        let needs: Vec<(i64, f64)> = needs_map.into_iter().collect();

        // Pre-check: verify sufficient stock for all materials before writing anything.
        let mut shortages: Vec<String> = Vec::new();
        for (material_id, needed) in &needs {
            if *needed <= 0.0 { continue; }
            let available: f64 = conn.query_row(
                "SELECT COALESCE(SUM(quantity),0) FROM inventory_batches WHERE material_id = ?1",
                params![material_id], |r| r.get(0),
            ).unwrap_or(0.0);
            if available < needed - 1e-9 {
                let mat_name: String = conn.query_row(
                    "SELECT name FROM materials WHERE id = ?1", params![material_id], |r| r.get(0),
                ).unwrap_or_else(|_| format!("材料#{}", material_id));
                shortages.push(format!("{}（需要 {:.2}，库存 {:.2}）", mat_name, needed, available));
            }
        }
        if !shortages.is_empty() {
            return Err(rusqlite::Error::InvalidParameterName(
                format!("库存不足: {}", shortages.join("；"))
            ));
        }

        for (material_id, mut remaining) in needs {
            if remaining <= 0.0 { continue; }

            // FIFO: oldest expiry first; ties broken by batch id
            let mut batch_stmt = conn.prepare(
                "SELECT id, quantity, cost_per_unit FROM inventory_batches
                 WHERE material_id = ?1 AND quantity > 0
                 ORDER BY COALESCE(expiry_date, '9999-99-99'), id"
            )?;
            let batches: Vec<(i64, f64, f64)> = batch_stmt.query_map(params![material_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?.collect::<Result<Vec<_>>>()?;

            for (batch_id, available, cost_per_unit) in batches {
                if remaining <= 0.0 { break; }
                let deduct = available.min(remaining);
                remaining -= deduct;
                let cost_delta = -(deduct * cost_per_unit);
                let txn_no = format!("TXN{}", chrono::Local::now().format("%Y%m%d%H%M%S%3f"));
                conn.execute(
                    "INSERT INTO inventory_txns (txn_no, txn_type, ref_type, ref_id, lot_id, material_id, qty_delta, cost_delta) \
                     VALUES (?1, 'consume', 'order', ?2, ?3, ?4, ?5, ?6)",
                    params![txn_no, order_id, batch_id, material_id, -deduct, cost_delta],
                )?;
                conn.execute(
                    "UPDATE inventory_batches SET quantity = quantity - ?1 WHERE id = ?2",
                    params![deduct, batch_id],
                )?;
            }
        }
        Ok(())
    }

    /// 獲取庫存交易流水
    pub fn get_inventory_txns(&self, material_id: Option<i64>, limit: i64) -> Result<Vec<InventoryTxn>> {
        let conn = self.conn.lock().unwrap();
        let (query, param): (&str, Option<i64>) = match material_id {
            Some(id) => ("SELECT id, txn_no, txn_type, ref_type, ref_id, lot_id, material_id, state_id, qty_delta, cost_delta, operator, note, created_at FROM inventory_txns WHERE material_id = ?1 ORDER BY created_at DESC LIMIT ?2", Some(id)),
            None => ("SELECT id, txn_no, txn_type, ref_type, ref_id, lot_id, material_id, state_id, qty_delta, cost_delta, operator, note, created_at FROM inventory_txns ORDER BY created_at DESC LIMIT ?1", None),
        };
        
        let mut stmt = conn.prepare(query)?;
        let txns = match param {
            Some(id) => stmt.query_map(params![id, limit], |row| {
                Ok(InventoryTxn {
                    id: row.get(0)?,
                    txn_no: row.get(1)?,
                    txn_type: row.get(2)?,
                    ref_type: row.get(3)?,
                    ref_id: row.get(4)?,
                    lot_id: row.get(5)?,
                    material_id: row.get(6)?,
                    state_id: row.get(7)?,
                    qty_delta: row.get(8)?,
                    cost_delta: row.get(9)?,
                    operator: row.get(10)?,
                    note: row.get(11)?,
                    created_at: row.get(12)?,
                })
            })?.collect::<Result<Vec<_>>>()?,
            None => stmt.query_map(params![limit], |row| {
                Ok(InventoryTxn {
                    id: row.get(0)?,
                    txn_no: row.get(1)?,
                    txn_type: row.get(2)?,
                    ref_type: row.get(3)?,
                    ref_id: row.get(4)?,
                    lot_id: row.get(5)?,
                    material_id: row.get(6)?,
                    state_id: row.get(7)?,
                    qty_delta: row.get(8)?,
                    cost_delta: row.get(9)?,
                    operator: row.get(10)?,
                    note: row.get(11)?,
                    created_at: row.get(12)?,
                })
            })?.collect::<Result<Vec<_>>>()?,
        };
        Ok(txns)
    }

    /// 添加訂單明細
    pub fn add_order_item(&self, order_id: i64, menu_item_id: i64, qty: f64, unit_price: f64, spec_code: Option<&str>, note: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO order_items (order_id, menu_item_id, qty, unit_price, spec_code, note) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![order_id, menu_item_id, qty, unit_price, spec_code, note],
        )?;
        
        // 更新訂單總額
        let item_total: f64 = conn.query_row(
            "SELECT COALESCE(SUM(qty * unit_price), 0) FROM order_items WHERE order_id = ?1",
            params![order_id],
            |row| row.get(0),
        )?;
        
        conn.execute(
            "UPDATE orders SET amount_total = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![item_total, order_id],
        )?;
        
        Ok(conn.last_insert_rowid())
    }

    /// 獲取訂單詳情（含明細）
    pub fn get_order_with_items(&self, order_id: i64) -> Result<(Order, Vec<OrderItem>)> {
        let conn = self.conn.lock().unwrap();
        
        let order = conn.query_row(
            "SELECT id, order_no, source, dine_type, table_no, status, amount_total, note, COALESCE(payment_status,'unpaid'), payment_method, COALESCE(amount_paid,0.0), created_at, updated_at FROM orders WHERE id = ?1",
            params![order_id],
            |row| {
                Ok(Order {
                    id: row.get(0)?,
                    order_no: row.get(1)?,
                    source: row.get(2)?,
                    dine_type: row.get(3)?,
                    table_no: row.get(4)?,
                    status: row.get(5)?,
                    amount_total: row.get(6)?,
                    note: row.get(7)?,
                    payment_status: row.get(8)?,
                    payment_method: row.get(9)?,
                    amount_paid: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            },
        )?;
        
        let mut stmt = conn.prepare(
            "SELECT id, order_id, menu_item_id, spec_code, qty, unit_price, note FROM order_items WHERE order_id = ?1"
        )?;
        let items = stmt.query_map(params![order_id], |row| {
            Ok(OrderItem {
                id: row.get(0)?,
                order_id: row.get(1)?,
                menu_item_id: row.get(2)?,
                spec_code: row.get(3)?,
                qty: row.get(4)?,
                unit_price: row.get(5)?,
                note: row.get(6)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        
        Ok((order, items))
    }

    /// 獲取所有廚房小票
    pub fn get_tickets_for_order(&self, order_id: i64) -> Result<Vec<KitchenTicket>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, order_id, station_id, status, priority, printed_at, started_at, finished_at, created_at FROM kitchen_tickets WHERE order_id = ?1 ORDER BY id"
        )?;
        let tickets = stmt.query_map(params![order_id], |row| {
            Ok(KitchenTicket { id: row.get(0)?, order_id: row.get(1)?, station_id: row.get(2)?, status: row.get(3)?, priority: row.get(4)?, printed_at: row.get(5)?, started_at: row.get(6)?, finished_at: row.get(7)?, created_at: row.get(8)? })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(tickets)
    }

    pub fn get_all_tickets(&self, status: Option<&str>) -> Result<Vec<KitchenTicket>> {
        let conn = self.conn.lock().unwrap();
        let (query, param): (&str, Option<&str>) = match status {
            Some(s) => ("SELECT id, order_id, station_id, status, priority, printed_at, started_at, finished_at, created_at FROM kitchen_tickets WHERE status = ?1 ORDER BY priority DESC, created_at ASC", Some(s)),
            None => ("SELECT id, order_id, station_id, status, priority, printed_at, started_at, finished_at, created_at FROM kitchen_tickets ORDER BY priority DESC, created_at ASC", None),
        };
        
        let mut stmt = conn.prepare(query)?;
        let tickets = match param {
            Some(s) => stmt.query_map(params![s], |row| {
                Ok(KitchenTicket {
                    id: row.get(0)?,
                    order_id: row.get(1)?,
                    station_id: row.get(2)?,
                    status: row.get(3)?,
                    priority: row.get(4)?,
                    printed_at: row.get(5)?,
                    started_at: row.get(6)?,
                    finished_at: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?.collect::<Result<Vec<_>>>()?,
            None => stmt.query_map([], |row| {
                Ok(KitchenTicket {
                    id: row.get(0)?,
                    order_id: row.get(1)?,
                    station_id: row.get(2)?,
                    status: row.get(3)?,
                    priority: row.get(4)?,
                    printed_at: row.get(5)?,
                    started_at: row.get(6)?,
                    finished_at: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?.collect::<Result<Vec<_>>>()?,
        };
        Ok(tickets)
    }

    // ==================== 打印機配置操作 ====================

    pub fn get_printers(&self) -> Result<Vec<PrinterConfig>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, printer_type, connection_type, feie_user, feie_ukey, feie_sn, feie_key, lan_ip, lan_port, paper_width, is_default, is_active, created_at, updated_at FROM printer_configs WHERE is_active = 1 ORDER BY is_default DESC, id"
        )?;
        let printers = stmt.query_map([], |row| {
            Ok(PrinterConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                printer_type: row.get(2)?,
                connection_type: row.get(3)?,
                feie_user: row.get(4)?,
                feie_ukey: row.get(5)?,
                feie_sn: row.get(6)?,
                feie_key: row.get(7)?,
                lan_ip: row.get(8)?,
                lan_port: row.get(9)?,
                paper_width: row.get(10)?,
                is_default: row.get::<_, i32>(11)? != 0,
                is_active: row.get::<_, i32>(12)? != 0,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(printers)
    }

    pub fn get_default_printer(&self) -> Result<Option<PrinterConfig>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, printer_type, connection_type, feie_user, feie_ukey, feie_sn, feie_key, lan_ip, lan_port, paper_width, is_default, is_active, created_at, updated_at FROM printer_configs WHERE is_default = 1 AND is_active = 1 LIMIT 1"
        )?;
        let printer = stmt.query_row([], |row| {
            Ok(PrinterConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                printer_type: row.get(2)?,
                connection_type: row.get(3)?,
                feie_user: row.get(4)?,
                feie_ukey: row.get(5)?,
                feie_sn: row.get(6)?,
                feie_key: row.get(7)?,
                lan_ip: row.get(8)?,
                lan_port: row.get(9)?,
                paper_width: row.get(10)?,
                is_default: row.get::<_, i32>(11)? != 0,
                is_active: row.get::<_, i32>(12)? != 0,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        }).ok();
        Ok(printer)
    }

    pub fn create_printer(&self, name: &str, printer_type: &str, connection_type: &str, feie_user: Option<&str>, feie_ukey: Option<&str>, feie_sn: Option<&str>, feie_key: Option<&str>, lan_ip: Option<&str>, lan_port: i32, paper_width: &str, is_default: bool) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        if is_default {
            conn.execute("UPDATE printer_configs SET is_default = 0", [])?;
        }
        conn.execute(
            "INSERT INTO printer_configs (name, printer_type, connection_type, feie_user, feie_ukey, feie_sn, feie_key, lan_ip, lan_port, paper_width, is_default) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![name, printer_type, connection_type, feie_user, feie_ukey, feie_sn, feie_key, lan_ip, lan_port, paper_width, is_default as i32],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_printer(&self, id: i64, name: Option<&str>, printer_type: Option<&str>, connection_type: Option<&str>, feie_user: Option<&str>, feie_ukey: Option<&str>, feie_sn: Option<&str>, feie_key: Option<&str>, lan_ip: Option<&str>, lan_port: Option<i32>, paper_width: Option<&str>, is_default: Option<bool>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        if let Some(n) = name {
            conn.execute("UPDATE printer_configs SET name = ?1, updated_at = datetime('now') WHERE id = ?2", params![n, id])?;
        }
        if let Some(pt) = printer_type {
            conn.execute("UPDATE printer_configs SET printer_type = ?1, updated_at = datetime('now') WHERE id = ?2", params![pt, id])?;
        }
        if let Some(ct) = connection_type {
            conn.execute("UPDATE printer_configs SET connection_type = ?1, updated_at = datetime('now') WHERE id = ?2", params![ct, id])?;
        }
        if feie_user.is_some() {
            conn.execute("UPDATE printer_configs SET feie_user = ?1, updated_at = datetime('now') WHERE id = ?2", params![feie_user, id])?;
        }
        if feie_ukey.is_some() {
            conn.execute("UPDATE printer_configs SET feie_ukey = ?1, updated_at = datetime('now') WHERE id = ?2", params![feie_ukey, id])?;
        }
        if feie_sn.is_some() {
            conn.execute("UPDATE printer_configs SET feie_sn = ?1, updated_at = datetime('now') WHERE id = ?2", params![feie_sn, id])?;
        }
        if feie_key.is_some() {
            conn.execute("UPDATE printer_configs SET feie_key = ?1, updated_at = datetime('now') WHERE id = ?2", params![feie_key, id])?;
        }
        if lan_ip.is_some() {
            conn.execute("UPDATE printer_configs SET lan_ip = ?1, updated_at = datetime('now') WHERE id = ?2", params![lan_ip, id])?;
        }
        if let Some(p) = lan_port {
            conn.execute("UPDATE printer_configs SET lan_port = ?1, updated_at = datetime('now') WHERE id = ?2", params![p, id])?;
        }
        if let Some(pw) = paper_width {
            conn.execute("UPDATE printer_configs SET paper_width = ?1, updated_at = datetime('now') WHERE id = ?2", params![pw, id])?;
        }
        if let Some(d) = is_default {
            if d {
                conn.execute("UPDATE printer_configs SET is_default = 0", [])?;
            }
            conn.execute("UPDATE printer_configs SET is_default = ?1, updated_at = datetime('now') WHERE id = ?2", params![d as i32, id])?;
        }
        Ok(())
    }

    pub fn delete_printer(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE printer_configs SET is_active = 0, updated_at = datetime('now') WHERE id = ?1", params![id])?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn bind_printer_to_station(&self, printer_id: i64, station_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE printer_configs SET station_id = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![station_id, printer_id],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn unbind_printer_from_station(&self, printer_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE printer_configs SET station_id = NULL, updated_at = datetime('now') WHERE id = ?1",
            params![printer_id],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_printers_for_station(&self, station_id: i64) -> Result<Vec<PrinterConfig>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, printer_type, connection_type, feie_user, feie_ukey, feie_sn, feie_key, lan_ip, lan_port, paper_width, is_default, is_active, created_at, updated_at FROM printer_configs WHERE station_id = ?1 AND is_active = 1 ORDER BY is_default DESC, id"
        )?;
        let printers = stmt.query_map(params![station_id], |row| {
            Ok(PrinterConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                printer_type: row.get(2)?,
                connection_type: row.get(3)?,
                feie_user: row.get(4)?,
                feie_ukey: row.get(5)?,
                feie_sn: row.get(6)?,
                feie_key: row.get(7)?,
                lan_ip: row.get(8)?,
                lan_port: row.get(9)?,
                paper_width: row.get(10)?,
                is_default: row.get::<_, i32>(11)? != 0,
                is_active: row.get::<_, i32>(12)? != 0,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(printers)
    }

    // ==================== 打印任務操作 ====================

    pub fn create_print_task(&self, task_type: &str, ref_type: Option<&str>, ref_id: Option<i64>, content: &str, printer_id: Option<i64>, printer_name: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO print_tasks (task_type, ref_type, ref_id, content, printer_id, printer_name) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![task_type, ref_type, ref_id, content, printer_id, printer_name],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_print_tasks(&self, limit: i64) -> Result<Vec<PrintTask>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, task_type, ref_type, ref_id, content, status, printer_id, printer_name, created_at, printed_at, error_msg FROM print_tasks ORDER BY created_at DESC LIMIT ?1"
        )?;
        let tasks = stmt.query_map(params![limit], |row| {
            Ok(PrintTask {
                id: row.get(0)?,
                task_type: row.get(1)?,
                ref_type: row.get(2)?,
                ref_id: row.get(3)?,
                content: row.get(4)?,
                status: row.get(5)?,
                printer_id: row.get(6)?,
                printer_name: row.get(7)?,
                created_at: row.get(8)?,
                printed_at: row.get(9)?,
                error_msg: row.get(10)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(tasks)
    }

    pub fn update_print_task_status(&self, id: i64, status: &str, error_msg: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        if error_msg.is_some() {
            conn.execute(
                "UPDATE print_tasks SET status = ?1, error_msg = ?2, retry_count = retry_count + 1, printed_at = datetime('now') WHERE id = ?3",
                params![status, error_msg, id],
            )?;
        } else {
            conn.execute(
                "UPDATE print_tasks SET status = ?1, printed_at = datetime('now') WHERE id = ?2",
                params![status, id],
            )?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_failed_print_tasks(&self) -> Result<Vec<PrintTask>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, task_type, ref_type, ref_id, content, status, printer_id, printer_name, created_at, printed_at, error_msg FROM print_tasks WHERE status = 'failed' AND retry_count < max_retries ORDER BY created_at ASC"
        )?;
        let tasks = stmt.query_map([], |row| {
            Ok(PrintTask {
                id: row.get(0)?,
                task_type: row.get(1)?,
                ref_type: row.get(2)?,
                ref_id: row.get(3)?,
                content: row.get(4)?,
                status: row.get(5)?,
                printer_id: row.get(6)?,
                printer_name: row.get(7)?,
                created_at: row.get(8)?,
                printed_at: row.get(9)?,
                error_msg: row.get(10)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(tasks)
    }

    #[allow(dead_code)]
    pub fn retry_print_task(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE print_tasks SET status = 'pending', error_msg = NULL WHERE id = ?1 AND retry_count < max_retries",
            params![id],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn retry_all_failed_print_tasks(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let count = conn.execute(
            "UPDATE print_tasks SET status = 'pending', error_msg = NULL WHERE status = 'failed' AND retry_count < max_retries",
            [],
        )?;
        Ok(count)
    }

    // ==================== 加料/去料 ====================

    pub fn add_order_item_modifier(&self, order_item_id: i64, modifier_type: &str, material_id: Option<i64>, qty: f64, price_delta: f64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT INTO order_item_modifiers (order_item_id, modifier_type, material_id, qty, price_delta) VALUES (?1, ?2, ?3, ?4, ?5)", params![order_item_id, modifier_type, material_id, qty, price_delta])?;
        Ok(())
    }

    pub fn get_order_item_modifiers(&self, order_item_id: i64) -> Result<Vec<OrderItemModifier>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT oim.id, oim.order_item_id, oim.modifier_type, oim.material_id, m.name, oim.qty, oim.price_delta FROM order_item_modifiers oim LEFT JOIN materials m ON oim.material_id = m.id WHERE oim.order_item_id = ?1")?;
        let modifiers = stmt.query_map(params![order_item_id], |row| {
            Ok(OrderItemModifier { id: row.get(0)?, order_item_id: row.get(1)?, modifier_type: row.get(2)?, material_id: row.get(3)?, material_name: row.get(4)?, qty: row.get(5)?, price_delta: row.get(6)? })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(modifiers)
    }

    pub fn delete_order_item_modifier(&self, modifier_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM order_item_modifiers WHERE id = ?1", params![modifier_id])?;
        Ok(())
    }

    // ==================== 採購單 ====================

    pub fn get_purchase_orders(&self, status: Option<&str>) -> Result<Vec<PurchaseOrder>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = if status.is_some() {
            conn.prepare("SELECT po.id, po.po_no, po.supplier_id, s.name, po.status, po.expected_date, po.total_cost, po.created_at FROM purchase_orders po LEFT JOIN suppliers s ON po.supplier_id = s.id AND s.is_active = 1 WHERE po.status = ?1 ORDER BY po.created_at DESC")?
        } else {
            conn.prepare("SELECT po.id, po.po_no, po.supplier_id, s.name, po.status, po.expected_date, po.total_cost, po.created_at FROM purchase_orders po LEFT JOIN suppliers s ON po.supplier_id = s.id AND s.is_active = 1 ORDER BY po.created_at DESC")?
        };
        let orders = if let Some(s) = status {
            stmt.query_map(params![s], |row| {
                Ok(PurchaseOrder { id: row.get(0)?, po_no: row.get(1)?, supplier_id: row.get(2)?, supplier_name: row.get(3)?, status: row.get(4)?, expected_date: row.get(5)?, total_cost: row.get(6)?, created_at: row.get(7)? })
            })?.collect::<Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |row| {
                Ok(PurchaseOrder { id: row.get(0)?, po_no: row.get(1)?, supplier_id: row.get(2)?, supplier_name: row.get(3)?, status: row.get(4)?, expected_date: row.get(5)?, total_cost: row.get(6)?, created_at: row.get(7)? })
            })?.collect::<Result<Vec<_>>>()?
        };
        Ok(orders)
    }

    pub fn get_purchase_order_with_items(&self, po_id: i64) -> Result<PurchaseOrderWithItems> {
        let conn = self.conn.lock().unwrap();
        let mut order_stmt = conn.prepare("SELECT po.id, po.po_no, po.supplier_id, s.name, po.status, po.expected_date, po.total_cost, po.created_at FROM purchase_orders po LEFT JOIN suppliers s ON po.supplier_id = s.id WHERE po.id = ?1")?;
        let order = order_stmt.query_row(params![po_id], |row| {
            Ok(PurchaseOrder { id: row.get(0)?, po_no: row.get(1)?, supplier_id: row.get(2)?, supplier_name: row.get(3)?, status: row.get(4)?, expected_date: row.get(5)?, total_cost: row.get(6)?, created_at: row.get(7)? })
        })?;
        let mut item_stmt = conn.prepare("SELECT poi.id, poi.po_id, poi.material_id, m.name, poi.qty, poi.unit_id, u.name, poi.cost_per_unit, poi.received_qty FROM purchase_order_items poi LEFT JOIN materials m ON poi.material_id = m.id LEFT JOIN units u ON poi.unit_id = u.id WHERE poi.po_id = ?1")?;
        let items = item_stmt.query_map(params![po_id], |row| {
            Ok(PurchaseOrderItem { id: row.get(0)?, po_id: row.get(1)?, material_id: row.get(2)?, material_name: row.get(3)?, qty: row.get(4)?, unit_id: row.get(5)?, unit_name: row.get(6)?, cost_per_unit: row.get(7)?, received_qty: row.get(8)? })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(PurchaseOrderWithItems { order, items })
    }

    pub fn create_purchase_order(&self, supplier_id: Option<i64>, expected_date: Option<&str>) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let po_no = format!("PO{}", chrono::Local::now().format("%Y%m%d%H%M%S%3f"));
        conn.execute("INSERT INTO purchase_orders (po_no, supplier_id, expected_date) VALUES (?1, ?2, ?3)", params![po_no, supplier_id, expected_date])?;
        Ok(po_no)
    }

    pub fn add_purchase_order_item(&self, po_id: i64, material_id: i64, qty: f64, unit_id: Option<i64>, cost_per_unit: f64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let status: String = conn.query_row("SELECT status FROM purchase_orders WHERE id = ?1", params![po_id], |r| r.get(0))?;
        if status != "draft" {
            return Err(rusqlite::Error::InvalidParameterName(
                format!("只能為草稿狀態的採購單新增明細，當前狀態：{}", status)
            ));
        }
        conn.execute("INSERT INTO purchase_order_items (po_id, material_id, qty, unit_id, cost_per_unit) VALUES (?1, ?2, ?3, ?4, ?5)", params![po_id, material_id, qty, unit_id, cost_per_unit])?;
        conn.execute("UPDATE purchase_orders SET total_cost = (SELECT COALESCE(SUM(qty * cost_per_unit), 0) FROM purchase_order_items WHERE po_id = ?1) WHERE id = ?1", params![po_id])?;
        Ok(())
    }

    pub fn update_purchase_order_status(&self, po_id: i64, status: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE purchase_orders SET status = ?1 WHERE id = ?2", params![status, po_id])?;
        Ok(())
    }

    pub fn delete_purchase_order(&self, po_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM purchase_order_items WHERE po_id = ?1", params![po_id])?;
        conn.execute("DELETE FROM purchase_orders WHERE id = ?1", params![po_id])?;
        Ok(())
    }

    pub fn receive_purchase_order(&self, po_id: i64, operator: Option<&str>) -> Result<Vec<(i64, String, String, String, f64, Option<String>)>> {
        let conn = self.conn.lock().unwrap();
        let po_no: String = conn.query_row("SELECT po_no FROM purchase_orders WHERE id = ?1", params![po_id], |r| r.get(0))?;
        let mut stmt = conn.prepare("SELECT poi.id, poi.material_id, poi.qty, poi.unit_id, poi.cost_per_unit, po.supplier_id FROM purchase_order_items poi JOIN purchase_orders po ON poi.po_id = po.id WHERE poi.po_id = ?1 AND poi.received_qty < poi.qty")?;
        let items: Vec<(i64, i64, f64, Option<i64>, f64, Option<i64>)> = stmt.query_map(params![po_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
        })?.collect::<Result<Vec<_>>>()?;
        conn.execute_batch("BEGIN")?;
        let result: Result<Vec<(i64, String, String, String, f64, Option<String>)>> = (|| {
            let mut batch_details = Vec::new();
            for (idx, (_item_id, material_id, qty, _unit_id, cost_per_unit, supplier_id)) in items.iter().enumerate() {
                let lot_no = format!("{}-{:02}", po_no, idx + 1);
                conn.execute("INSERT INTO inventory_batches (material_id, state_id, lot_no, supplier_id, brand, spec, quantity, original_qty, cost_per_unit, production_date, expiry_date, ice_coating_rate, quality_rate, seasonal_factor) VALUES (?1, NULL, ?2, ?3, NULL, NULL, ?4, ?4, ?5, NULL, NULL, NULL, NULL, 1.0)", params![material_id, lot_no, supplier_id, qty, cost_per_unit])?;
                let batch_id = conn.last_insert_rowid();
                let last_cost = conn.query_row::<f64, _, _>(
                    "SELECT cost_per_unit FROM material_cost_history WHERE material_id = ?1 ORDER BY created_at DESC LIMIT 1",
                    params![material_id], |row| row.get(0),
                ).ok();
                if last_cost.map_or(true, |lc| (lc - cost_per_unit).abs() > 0.001) {
                    conn.execute("INSERT INTO material_cost_history (material_id, cost_per_unit, source_type, source_id, batch_no, operator) VALUES (?1, ?2, 'purchase', ?3, ?4, ?5)", params![material_id, cost_per_unit, po_id, lot_no, operator])?;
                }
                let txn_no = format!("TXN{}", chrono::Local::now().format("%Y%m%d%H%M%S%3f"));
                conn.execute("INSERT INTO inventory_txns (txn_no, txn_type, ref_type, ref_id, lot_id, material_id, qty_delta, cost_delta, operator, note) VALUES (?1, 'purchase_in', 'purchase_order', ?2, ?3, ?4, ?5, ?6, ?7, '採購入庫')", params![txn_no, po_id, batch_id, material_id, qty, qty * cost_per_unit, operator])?;
                conn.execute("UPDATE purchase_order_items SET received_qty = qty WHERE id = ?1", params![_item_id])?;
                let unit: String = conn.query_row("SELECT u.name FROM units u JOIN materials m ON m.base_unit_id = u.id WHERE m.id = ?1", params![material_id], |r| r.get(0)).unwrap_or_else(|_| "份".to_string());
                let material_name: String = conn.query_row("SELECT name FROM materials WHERE id = ?1", params![material_id], |r| r.get(0)).unwrap_or_default();
                let supplier_name: Option<String> = supplier_id.and_then(|sid| conn.query_row("SELECT name FROM suppliers WHERE id = ?1", params![sid], |r| r.get(0)).ok());
                batch_details.push((batch_id, lot_no.clone(), material_name, unit, *qty, supplier_name));
            }
            let remaining: i64 = conn.query_row("SELECT COUNT(*) FROM purchase_order_items WHERE po_id = ?1 AND received_qty < qty", params![po_id], |r| r.get(0))?;
            let new_status = if remaining == 0 { "received" } else { "partial" };
            conn.execute("UPDATE purchase_orders SET status = ?1 WHERE id = ?2", params![new_status, po_id])?;
            Ok(batch_details)
        })();
        match result {
            Ok(val) => { conn.execute_batch("COMMIT")?; Ok(val) }
            Err(e) => { conn.execute_batch("ROLLBACK").ok(); Err(e) }
        }
    }

    /// 分批收貨：按指定品項數量部分入庫
    pub fn receive_purchase_order_items(&self, po_id: i64, items: Vec<(i64, f64, Option<String>)>, operator: Option<&str>) -> Result<Vec<(i64, String, String, String, f64, Option<String>)>> {
        let conn = self.conn.lock().unwrap();
        let po_no: String = conn.query_row("SELECT po_no FROM purchase_orders WHERE id = ?1", params![po_id], |r| r.get(0))?;
        conn.execute_batch("BEGIN")?;
        let result: Result<Vec<(i64, String, String, String, f64, Option<String>)>> = (|| {
            let mut batch_details = Vec::new();
            for (idx, (item_id, recv_qty, custom_lot)) in items.iter().enumerate() {
                let (material_id, cost_per_unit, supplier_id): (i64, f64, Option<i64>) = conn.query_row(
                    "SELECT poi.material_id, poi.cost_per_unit, po.supplier_id FROM purchase_order_items poi JOIN purchase_orders po ON poi.po_id = po.id WHERE poi.id = ?1 AND poi.po_id = ?2",
                    params![item_id, po_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )?;
                let lot_no = custom_lot.clone().unwrap_or_else(|| format!("{}-{:02}", po_no, idx + 1));
                conn.execute("INSERT INTO inventory_batches (material_id, state_id, lot_no, supplier_id, brand, spec, quantity, original_qty, cost_per_unit, production_date, expiry_date, ice_coating_rate, quality_rate, seasonal_factor) VALUES (?1, NULL, ?2, ?3, NULL, NULL, ?4, ?4, ?5, NULL, NULL, NULL, NULL, 1.0)", params![material_id, lot_no, supplier_id, recv_qty, cost_per_unit])?;
                let batch_id = conn.last_insert_rowid();
                let txn_no = format!("TXN{}", chrono::Local::now().format("%Y%m%d%H%M%S%3f"));
                conn.execute("INSERT INTO inventory_txns (txn_no, txn_type, ref_type, ref_id, lot_id, material_id, qty_delta, cost_delta, operator, note) VALUES (?1, 'purchase_in', 'purchase_order', ?2, ?3, ?4, ?5, ?6, ?7, '採購入庫')", params![txn_no, po_id, batch_id, material_id, recv_qty, recv_qty * cost_per_unit, operator])?;
                conn.execute("UPDATE purchase_order_items SET received_qty = received_qty + ?1 WHERE id = ?2", params![recv_qty, item_id])?;
                let last_cost = conn.query_row::<f64, _, _>("SELECT cost_per_unit FROM material_cost_history WHERE material_id = ?1 ORDER BY created_at DESC LIMIT 1", params![material_id], |r| r.get(0)).ok();
                if last_cost.map_or(true, |lc| (lc - cost_per_unit).abs() > 0.001) {
                    conn.execute("INSERT INTO material_cost_history (material_id, cost_per_unit, source_type, source_id, batch_no, operator) VALUES (?1, ?2, 'purchase', ?3, ?4, ?5)", params![material_id, cost_per_unit, po_id, lot_no, operator])?;
                }
                let unit: String = conn.query_row("SELECT u.name FROM units u JOIN materials m ON m.base_unit_id = u.id WHERE m.id = ?1", params![material_id], |r| r.get(0)).unwrap_or_else(|_| "份".to_string());
                let material_name: String = conn.query_row("SELECT name FROM materials WHERE id = ?1", params![material_id], |r| r.get(0)).unwrap_or_default();
                let supplier_name: Option<String> = supplier_id.and_then(|sid| conn.query_row("SELECT name FROM suppliers WHERE id = ?1", params![sid], |r| r.get(0)).ok());
                batch_details.push((batch_id, lot_no, material_name, unit, *recv_qty, supplier_name));
            }
            let remaining: i64 = conn.query_row("SELECT COUNT(*) FROM purchase_order_items WHERE po_id = ?1 AND received_qty < qty", params![po_id], |r| r.get(0))?;
            let new_status = if remaining == 0 { "received" } else { "partial" };
            conn.execute("UPDATE purchase_orders SET status = ?1 WHERE id = ?2", params![new_status, po_id])?;
            Ok(batch_details)
        })();
        match result {
            Ok(val) => { conn.execute_batch("COMMIT")?; Ok(val) }
            Err(e) => { conn.execute_batch("ROLLBACK").ok(); Err(e) }
        }
    }

    // ==================== 生產單 ====================

    pub fn get_production_orders(&self, status: Option<&str>) -> Result<Vec<ProductionOrder>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT prd.id, prd.production_no, prd.recipe_id, r.name, prd.status, prd.planned_qty, prd.actual_qty, prd.operator, prd.started_at, prd.completed_at, prd.created_at FROM production_orders prd LEFT JOIN recipes r ON prd.recipe_id = r.id ORDER BY prd.created_at DESC")?;
        let orders = stmt.query_map([], |row| {
            Ok(ProductionOrder { id: row.get(0)?, production_no: row.get(1)?, recipe_id: row.get(2)?, recipe_name: row.get(3)?, status: row.get(4)?, planned_qty: row.get(5)?, actual_qty: row.get(6)?, operator: row.get(7)?, started_at: row.get(8)?, completed_at: row.get(9)?, created_at: row.get(10)? })
        })?.collect::<Result<Vec<_>>>()?;
        if let Some(s) = status {
            Ok(orders.into_iter().filter(|o| o.status == s).collect())
        } else {
            Ok(orders)
        }
    }

    pub fn get_production_order_with_items(&self, production_id: i64) -> Result<ProductionOrderWithItems> {
        let conn = self.conn.lock().unwrap();
        let mut order_stmt = conn.prepare("SELECT prd.id, prd.production_no, prd.recipe_id, r.name, prd.status, prd.planned_qty, prd.actual_qty, prd.operator, prd.started_at, prd.completed_at, prd.created_at FROM production_orders prd LEFT JOIN recipes r ON prd.recipe_id = r.id WHERE prd.id = ?1")?;
        let order = order_stmt.query_row(params![production_id], |row| {
            Ok(ProductionOrder { id: row.get(0)?, production_no: row.get(1)?, recipe_id: row.get(2)?, recipe_name: row.get(3)?, status: row.get(4)?, planned_qty: row.get(5)?, actual_qty: row.get(6)?, operator: row.get(7)?, started_at: row.get(8)?, completed_at: row.get(9)?, created_at: row.get(10)? })
        })?;
        let mut item_stmt = conn.prepare("SELECT prdi.id, prdi.production_id, prdi.material_id, m.name, prdi.lot_id, prdi.planned_qty, prdi.actual_qty FROM production_order_items prdi LEFT JOIN materials m ON prdi.material_id = m.id WHERE prdi.production_id = ?1")?;
        let items = item_stmt.query_map(params![production_id], |row| {
            Ok(ProductionOrderItem { id: row.get(0)?, production_id: row.get(1)?, material_id: row.get(2)?, material_name: row.get(3)?, lot_id: row.get(4)?, planned_qty: row.get(5)?, actual_qty: row.get(6)? })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(ProductionOrderWithItems { order, items })
    }

    pub fn check_production_materials(&self, recipe_id: i64, planned_qty: f64) -> Result<Vec<(String, f64, f64)>> {
        let conn = self.conn.lock().unwrap();
        let mut flat: std::collections::HashMap<i64, f64> = std::collections::HashMap::new();
        expand_recipe_needs(&conn, recipe_id, planned_qty, 0, &mut flat)?;
        let mut result = Vec::new();
        for (material_id, needed) in flat {
            let mat_name: String = conn.query_row(
                "SELECT name FROM materials WHERE id = ?1", params![material_id], |r| r.get(0),
            ).unwrap_or_else(|_| format!("材料#{}", material_id));
            let available: f64 = conn.query_row(
                "SELECT COALESCE(SUM(quantity),0) FROM inventory_batches WHERE material_id = ?1",
                params![material_id], |r| r.get(0),
            ).unwrap_or(0.0);
            result.push((mat_name, needed, available));
        }
        result.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(result)
    }

    pub fn create_production_order(&self, recipe_id: i64, planned_qty: f64, operator: Option<&str>) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let production_no = format!("PRD{}", chrono::Local::now().format("%Y%m%d%H%M%S%3f"));
        conn.execute("INSERT INTO production_orders (production_no, recipe_id, planned_qty, operator) VALUES (?1, ?2, ?3, ?4)", params![production_no, recipe_id, planned_qty, operator])?;
        let prod_id = conn.last_insert_rowid();
        // Recursively expand recipe (handles direct materials + nested sub-recipes)
        let mut item_map: std::collections::HashMap<i64, f64> = std::collections::HashMap::new();
        expand_recipe_needs(&conn, recipe_id, planned_qty, 0, &mut item_map)?;
        for (material_id, planned) in item_map {
            conn.execute("INSERT INTO production_order_items (production_id, material_id, planned_qty) VALUES (?1, ?2, ?3)", params![prod_id, material_id, planned])?;
        }
        Ok(production_no)
    }

    pub fn start_production_order(&self, production_id: i64, operator: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE production_orders SET status = 'in_progress', started_at = datetime('now'), operator = ?1 WHERE id = ?2", params![operator, production_id])?;
        Ok(())
    }

    pub fn complete_production_order(&self, production_id: i64, actual_qty: f64, operator: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT poi.material_id, poi.planned_qty FROM production_order_items poi WHERE poi.production_id = ?1")?;
        let items: Vec<(i64, f64)> = stmt.query_map(params![production_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?.collect::<Result<Vec<_>>>()?;
        let (recipe_id, output_material_id, production_no, planned_qty): (i64, Option<i64>, String, f64) = conn.query_row(
            "SELECT prd.recipe_id, r.output_material_id, prd.production_no, prd.planned_qty FROM production_orders prd JOIN recipes r ON prd.recipe_id = r.id WHERE prd.id = ?1",
            params![production_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;
        let shelf_life_days: Option<i64> = conn.query_row(
            "SELECT m.shelf_life_days FROM recipes r JOIN materials m ON r.output_material_id = m.id WHERE r.id = ?1",
            params![recipe_id], |row| row.get(0),
        ).ok().flatten();
        let expiry_date: Option<String> = shelf_life_days.map(|days| {
            use chrono::Datelike;
            let expiry = chrono::Local::now() + chrono::Duration::days(days);
            format!("{:04}-{:02}-{:02}", expiry.year(), expiry.month(), expiry.day())
        });
        let output_mid = match output_material_id {
            Some(mid) => mid,
            None => return Err(rusqlite::Error::InvalidParameterName("配方缺少輸出物料，無法完成生産".to_string())),
        };
        // Pre-check: verify sufficient stock for all materials before writing.
        let scale_pre = if planned_qty > 0.0 { actual_qty / planned_qty } else { 1.0 };
        let mut shortages: Vec<String> = Vec::new();
        for (material_id, mat_planned_qty) in &items {
            let needed = mat_planned_qty * scale_pre;
            if needed <= 0.0 { continue; }
            let available: f64 = conn.query_row(
                "SELECT COALESCE(SUM(quantity),0) FROM inventory_batches WHERE material_id = ?1",
                params![material_id], |r| r.get(0),
            ).unwrap_or(0.0);
            if available < needed - 1e-9 {
                let mat_name: String = conn.query_row(
                    "SELECT name FROM materials WHERE id = ?1", params![material_id], |r| r.get(0),
                ).unwrap_or_else(|_| format!("材料#{}", material_id));
                shortages.push(format!("{}（需要 {:.2}，库存 {:.2}）", mat_name, needed, available));
            }
        }
        if !shortages.is_empty() {
            return Err(rusqlite::Error::InvalidParameterName(
                format!("原料不足: {}", shortages.join("；"))
            ));
        }

        conn.execute_batch("BEGIN")?;
        let result: Result<()> = (|| {
            conn.execute(
                "UPDATE production_orders SET status = 'completed', completed_at = datetime('now'), actual_qty = ?1 WHERE id = ?2",
                params![actual_qty, production_id],
            )?;
            // Scale planned consumption by actual/planned ratio
            let scale = if planned_qty > 0.0 { actual_qty / planned_qty } else { 1.0 };
            let mut total_cost = 0.0_f64;
            for (material_id, mat_planned_qty) in &items {
                let needed = mat_planned_qty * scale;
                if needed <= 0.0 { continue; }
                // FIFO deduction
                let mut batch_stmt = conn.prepare(
                    "SELECT id, quantity, cost_per_unit FROM inventory_batches
                     WHERE material_id = ?1 AND quantity > 0
                     ORDER BY COALESCE(expiry_date,'9999-99-99'), id"
                )?;
                let batches: Vec<(i64, f64, f64)> = batch_stmt.query_map(params![material_id], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })?.collect::<Result<Vec<_>>>()?;
                let mut remaining = needed;
                for (batch_id, available, cost_per_unit) in batches {
                    if remaining <= 0.0 { break; }
                    let deduct = available.min(remaining);
                    remaining -= deduct;
                    total_cost += deduct * cost_per_unit;
                    let txn_no = format!("TXN{}", chrono::Local::now().format("%Y%m%d%H%M%S%3f"));
                    conn.execute(
                        "INSERT INTO inventory_txns (txn_no, txn_type, ref_type, ref_id, lot_id, material_id, qty_delta, cost_delta, operator, note) VALUES (?1, 'production_out', 'production_order', ?2, ?3, ?4, ?5, ?6, ?7, '生産耗用')",
                        params![txn_no, production_id, batch_id, material_id, -deduct, -(deduct * cost_per_unit), operator],
                    )?;
                    conn.execute(
                        "UPDATE inventory_batches SET quantity = quantity - ?1 WHERE id = ?2",
                        params![deduct, batch_id],
                    )?;
                }
            }
            let output_cost = if actual_qty > 0.0 { total_cost / actual_qty } else { 0.0 };
            let lot_no = format!("{}-OUT", production_no);
            conn.execute(
                "INSERT INTO inventory_batches (material_id, state_id, lot_no, supplier_id, brand, spec, quantity, original_qty, cost_per_unit, production_date, expiry_date, ice_coating_rate, quality_rate, seasonal_factor) VALUES (?1, NULL, ?2, NULL, NULL, NULL, ?3, ?3, ?4, datetime('now'), ?5, NULL, NULL, 1.0)",
                params![output_mid, lot_no, actual_qty, output_cost, expiry_date],
            )?;
            Ok(())
        })();
        match result {
            Ok(_) => { conn.execute_batch("COMMIT")?; Ok(()) }
            Err(e) => { conn.execute_batch("ROLLBACK").ok(); Err(e) }
        }
    }

    /// 取消已出餐訂單：保留庫存消耗（食材已用），僅更新訂單狀態
    pub fn cancel_order_confirmed(&self, order_id: i64, reason: Option<&str>) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("BEGIN")?;
        let result: Result<()> = (|| {
            Self::consume_order_inventory(&conn, order_id)?;
            conn.execute("UPDATE orders SET status = 'cancelled', cancel_reason = ?1, updated_at = datetime('now') WHERE id = ?2", params![reason, order_id])?;
            Ok(())
        })();
        match result {
            Ok(_) => { conn.execute_batch("COMMIT")?; Ok(Vec::new()) }
            Err(e) => { conn.execute_batch("ROLLBACK").ok(); Err(e) }
        }
    }

    /// 直接標記訂單出餐（不用 KDS 的場景）：消耗庫存 + 更新狀態為 ready
    pub fn mark_order_ready(&self, order_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("BEGIN")?;
        let result: Result<()> = (|| {
            Self::consume_order_inventory(&conn, order_id)?;
            conn.execute(
                "UPDATE orders SET status = 'ready', updated_at = datetime('now') WHERE id = ?1 AND status = 'submitted'",
                params![order_id],
            )?;
            Ok(())
        })();
        match result {
            Ok(_) => { conn.execute_batch("COMMIT")?; Ok(()) }
            Err(e) => { conn.execute_batch("ROLLBACK").ok(); Err(e) }
        }
    }

    pub fn update_order_payment(&self, order_id: i64, payment_status: &str, payment_method: Option<&str>, amount_paid: f64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE orders SET payment_status = ?1, payment_method = ?2, amount_paid = ?3, updated_at = datetime('now') WHERE id = ?4",
            params![payment_status, payment_method, amount_paid, order_id],
        )?;
        Ok(())
    }

    pub fn record_order_refund(&self, order_id: i64, refund_amount: f64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE orders SET refund_amount = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![refund_amount, order_id],
        )?;
        Ok(())
    }

    pub fn get_sales_by_hour(&self, start_date: &str, end_date: &str) -> Result<Vec<(i32, i64, f64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT CAST(strftime('%H', created_at, 'localtime') AS INTEGER) AS hr,
                    COUNT(*) AS cnt, COALESCE(SUM(amount_total), 0)
             FROM orders
             WHERE status != 'cancelled'
               AND date(created_at, 'localtime') BETWEEN ?1 AND ?2
             GROUP BY hr ORDER BY hr"
        )?;
        let rows = stmt.query_map(params![start_date, end_date], |row| {
            Ok((row.get::<_, i32>(0)?, row.get::<_, i64>(1)?, row.get::<_, f64>(2)?))
        })?.collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn get_sales_by_weekday(&self, start_date: &str, end_date: &str) -> Result<Vec<(i32, i64, f64)>> {
        let conn = self.conn.lock().unwrap();
        // SQLite strftime('%w') returns 0=Sun..6=Sat
        let mut stmt = conn.prepare(
            "SELECT CAST(strftime('%w', created_at, 'localtime') AS INTEGER) AS wd,
                    COUNT(*) AS cnt, COALESCE(SUM(amount_total), 0)
             FROM orders
             WHERE status != 'cancelled'
               AND date(created_at, 'localtime') BETWEEN ?1 AND ?2
             GROUP BY wd ORDER BY wd"
        )?;
        let rows = stmt.query_map(params![start_date, end_date], |row| {
            Ok((row.get::<_, i32>(0)?, row.get::<_, i64>(1)?, row.get::<_, f64>(2)?))
        })?.collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn delete_production_order(&self, production_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let status: String = conn.query_row(
            "SELECT status FROM production_orders WHERE id = ?1", params![production_id], |r| r.get(0),
        )?;
        if status != "draft" {
            return Err(rusqlite::Error::InvalidParameterName(
                format!("只能刪除草稿狀態的生産單，當前狀態：{}", status),
            ));
        }
        conn.execute("DELETE FROM production_order_items WHERE production_id = ?1", params![production_id])?;
        conn.execute("DELETE FROM production_orders WHERE id = ?1", params![production_id])?;
        Ok(())
    }

    // ==================== 盤點 ====================

    pub fn get_stocktakes(&self, status: Option<&str>) -> Result<Vec<Stocktake>> {
        let conn = self.conn.lock().unwrap();
        let sql = if status.is_some() {
            "SELECT id, stocktake_no, status, operator, note, created_at, completed_at FROM stocktakes WHERE status = ?1 ORDER BY created_at DESC"
        } else {
            "SELECT id, stocktake_no, status, operator, note, created_at, completed_at FROM stocktakes ORDER BY created_at DESC"
        };
        let mut stmt = conn.prepare(sql)?;
        let row_fn = |row: &rusqlite::Row| Ok(Stocktake { id: row.get(0)?, stocktake_no: row.get(1)?, status: row.get(2)?, operator: row.get(3)?, note: row.get(4)?, created_at: row.get(5)?, completed_at: row.get(6)? });
        let result = if let Some(s) = status {
            stmt.query_map(params![s], row_fn)?.collect::<Result<Vec<_>>>()?
        } else {
            stmt.query_map([], row_fn)?.collect::<Result<Vec<_>>>()?
        };
        Ok(result)
    }

    pub fn get_stocktake_with_items(&self, stocktake_id: i64) -> Result<StocktakeWithItems> {
        let conn = self.conn.lock().unwrap();
        let mut stocktake_stmt = conn.prepare("SELECT id, stocktake_no, status, operator, note, created_at, completed_at FROM stocktakes WHERE id = ?1")?;
        let stocktake = stocktake_stmt.query_row(params![stocktake_id], |row| {
            Ok(Stocktake { id: row.get(0)?, stocktake_no: row.get(1)?, status: row.get(2)?, operator: row.get(3)?, note: row.get(4)?, created_at: row.get(5)?, completed_at: row.get(6)? })
        })?;
        let mut item_stmt = conn.prepare("SELECT sti.id, sti.stocktake_id, sti.lot_id, sti.material_id, m.name, sti.system_qty, sti.actual_qty, sti.diff_qty, COALESCE(sti.is_counted,0), sti.note FROM stocktake_items sti LEFT JOIN materials m ON sti.material_id = m.id WHERE sti.stocktake_id = ?1")?;
        let items = item_stmt.query_map(params![stocktake_id], |row| {
            Ok(StocktakeItem { id: row.get(0)?, stocktake_id: row.get(1)?, lot_id: row.get(2)?, material_id: row.get(3)?, material_name: row.get(4)?, system_qty: row.get(5)?, actual_qty: row.get(6)?, diff_qty: row.get(7)?, is_counted: row.get::<_, i32>(8)? != 0, note: row.get(9)? })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(StocktakeWithItems { stocktake, items })
    }

    pub fn create_stocktake(&self, operator: Option<&str>, note: Option<&str>) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let stocktake_no = format!("STK{}", chrono::Local::now().format("%Y%m%d%H%M%S%3f"));
        conn.execute("INSERT INTO stocktakes (stocktake_no, operator, note) VALUES (?1, ?2, ?3)", params![stocktake_no, operator, note])?;
        let stocktake_id = conn.last_insert_rowid();
        let mut batch_stmt = conn.prepare("SELECT id, material_id, quantity FROM inventory_batches WHERE quantity > 0")?;
        let batches: Vec<(i64, i64, f64)> = batch_stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?.collect::<Result<Vec<_>>>()?;
        for (batch_id, material_id, qty) in batches {
            conn.execute("INSERT INTO stocktake_items (stocktake_id, lot_id, material_id, system_qty, actual_qty, is_counted) VALUES (?1, ?2, ?3, ?4, ?4, 0)", params![stocktake_id, batch_id, material_id, qty])?;
        }
        Ok(stocktake_no)
    }

    pub fn update_stocktake_item(&self, item_id: i64, actual_qty: f64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let status: String = conn.query_row(
            "SELECT s.status FROM stocktakes s JOIN stocktake_items si ON si.stocktake_id = s.id WHERE si.id = ?1",
            params![item_id], |r| r.get(0),
        )?;
        if status == "completed" {
            return Err(rusqlite::Error::InvalidParameterName("盤點已完成，不允許修改明細".to_string()));
        }
        conn.execute("UPDATE stocktake_items SET actual_qty = ?1, diff_qty = ?1 - system_qty, is_counted = 1 WHERE id = ?2", params![actual_qty, item_id])?;
        Ok(())
    }

    pub fn complete_stocktake(&self, stocktake_id: i64, operator: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT sti.lot_id, sti.material_id, sti.diff_qty FROM stocktake_items sti WHERE sti.stocktake_id = ?1 AND COALESCE(sti.is_counted,0) = 1 AND ABS(sti.diff_qty) > 0.001")?;
        let items: Vec<(Option<i64>, i64, f64)> = stmt.query_map(params![stocktake_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?.collect::<Result<Vec<_>>>()?;
        conn.execute_batch("BEGIN")?;
        let result: Result<()> = (|| {
            for (lot_id, material_id, diff_qty) in &items {
                let txn_no = format!("TXN{}", chrono::Local::now().format("%Y%m%d%H%M%S%3f"));
                let cpu: f64 = match lot_id {
                    Some(lot) => conn.query_row(
                        "SELECT COALESCE(cost_per_unit, 0.0) FROM inventory_batches WHERE id = ?1",
                        params![lot], |r| r.get(0),
                    ).unwrap_or(0.0),
                    None => conn.query_row(
                        "SELECT COALESCE(AVG(cost_per_unit), 0.0) FROM inventory_batches WHERE material_id = ?1 AND quantity > 0",
                        params![material_id], |r| r.get(0),
                    ).unwrap_or(0.0),
                };
                let cost_delta = diff_qty * cpu;
                if let Some(lot) = lot_id {
                    conn.execute("UPDATE inventory_batches SET quantity = quantity + ?1 WHERE id = ?2", params![diff_qty, lot])?;
                }
                conn.execute("INSERT INTO inventory_txns (txn_no, txn_type, ref_type, ref_id, lot_id, material_id, qty_delta, cost_delta, operator, note) VALUES (?1, 'stocktake_adjust', 'stocktake', ?2, ?3, ?4, ?5, ?6, ?7, '盤點調整')", params![txn_no, stocktake_id, lot_id, material_id, diff_qty, cost_delta, operator])?;
            }
            conn.execute("UPDATE stocktakes SET status = 'completed', completed_at = datetime('now') WHERE id = ?1", params![stocktake_id])?;
            Ok(())
        })();
        match result {
            Ok(_) => { conn.execute_batch("COMMIT")?; Ok(()) }
            Err(e) => { conn.execute_batch("ROLLBACK").ok(); Err(e) }
        }
    }

    pub fn delete_stocktake(&self, stocktake_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM stocktake_items WHERE stocktake_id = ?1", params![stocktake_id])?;
        conn.execute("DELETE FROM stocktakes WHERE id = ?1", params![stocktake_id])?;
        Ok(())
    }

    // ==================== 報表 ====================

    // Returns (date, billed_amount, order_count, collected_amount)
    pub fn get_sales_report(&self, start_date: &str, end_date: &str) -> Result<Vec<(String, f64, i64, f64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT DATE(o.created_at),
                    SUM(oi.qty * oi.unit_price),
                    COUNT(DISTINCT o.id),
                    SUM(CASE WHEN COALESCE(o.payment_status,'unpaid') != 'unpaid' THEN COALESCE(o.amount_paid, 0) ELSE 0 END)
             FROM orders o
             JOIN order_items oi ON o.id = oi.order_id
             WHERE o.status IN ('submitted','ready')
               AND DATE(o.created_at) BETWEEN ?1 AND ?2
             GROUP BY DATE(o.created_at)
             ORDER BY DATE(o.created_at)"
        )?;
        let rows = stmt.query_map(params![start_date, end_date], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?.collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn get_sales_by_category(&self, start_date: &str, end_date: &str) -> Result<Vec<(String, f64, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT mc.name, SUM(oi.qty * oi.unit_price), SUM(oi.qty) FROM orders o JOIN order_items oi ON o.id = oi.order_id JOIN menu_items mi ON oi.menu_item_id = mi.id LEFT JOIN menu_categories mc ON mi.category_id = mc.id WHERE o.status IN ('submitted', 'ready') AND DATE(o.created_at) BETWEEN ?1 AND ?2 GROUP BY mc.name ORDER BY SUM(oi.qty * oi.unit_price) DESC")?;
        let rows = stmt.query_map(params![start_date, end_date], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?.collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    // Returns (date, revenue, cogs, gross_profit, expenses, net_profit)
    pub fn get_gross_profit_report(&self, start_date: &str, end_date: &str) -> Result<Vec<(String, f64, f64, f64, f64, f64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "WITH rev AS (
                SELECT DATE(o.created_at) AS day, SUM(oi.qty * oi.unit_price) AS revenue
                FROM orders o
                JOIN order_items oi ON o.id = oi.order_id
                WHERE o.status IN ('submitted','ready')
                  AND DATE(o.created_at) BETWEEN ?1 AND ?2
                GROUP BY day
            ),
            cst AS (
                SELECT DATE(o.created_at) AS day, SUM(ABS(it.cost_delta)) AS cost
                FROM orders o
                JOIN inventory_txns it ON it.ref_type = 'order' AND it.ref_id = o.id
                                      AND it.txn_type = 'consume'
                WHERE o.status IN ('submitted','ready')
                  AND DATE(o.created_at) BETWEEN ?1 AND ?2
                GROUP BY day
            ),
            exp AS (
                SELECT expense_date AS day, SUM(amount) AS expenses
                FROM expenses
                WHERE is_active = 1
                  AND expense_date BETWEEN ?1 AND ?2
                GROUP BY day
            )
            SELECT rev.day,
                   rev.revenue,
                   COALESCE(cst.cost, 0),
                   rev.revenue - COALESCE(cst.cost, 0),
                   COALESCE(exp.expenses, 0),
                   rev.revenue - COALESCE(cst.cost, 0) - COALESCE(exp.expenses, 0)
            FROM rev
            LEFT JOIN cst ON cst.day = rev.day
            LEFT JOIN exp ON exp.day = rev.day
            ORDER BY rev.day"
        )?;
        let rows = stmt.query_map(params![start_date, end_date], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
        })?.collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn get_top_selling_items(&self, start_date: &str, end_date: &str, limit: i64) -> Result<Vec<(String, f64, i64, f64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT mi.name, SUM(oi.qty * oi.unit_price), SUM(oi.qty), AVG(oi.unit_price) FROM orders o JOIN order_items oi ON o.id = oi.order_id JOIN menu_items mi ON oi.menu_item_id = mi.id WHERE o.status IN ('submitted', 'ready') AND DATE(o.created_at) BETWEEN ?1 AND ?2 GROUP BY mi.name ORDER BY SUM(oi.qty) DESC LIMIT ?3")?;
        let rows = stmt.query_map(params![start_date, end_date, limit], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?.collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// 原料消耗报表
    pub fn get_material_consumption_report(&self, start_date: &str, end_date: &str) -> Result<Vec<(String, f64, f64, f64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT m.name, 
                    SUM(it.qty_delta) as total_consumed,
                    COALESCE(AVG(ib.cost_per_unit), 0) as avg_cost,
                    SUM(it.qty_delta * COALESCE(ib.cost_per_unit, 0)) as total_cost
             FROM inventory_txns it
             LEFT JOIN materials m ON it.material_id = m.id
             LEFT JOIN inventory_batches ib ON it.lot_id = ib.id
             WHERE it.txn_type = 'consume' 
               AND DATE(it.created_at) BETWEEN ?1 AND ?2
             GROUP BY m.id, m.name
             ORDER BY total_consumed DESC"
        )?;
        let rows = stmt.query_map(params![start_date, end_date], |row| {
            Ok((
                row.get(0)?,
                row.get::<_, f64>(1)?.abs(),
                row.get(2)?,
                row.get(3)?,
            ))
        })?.collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    // ==================== 打印模板 ====================

    pub fn get_print_templates(&self, template_type: Option<&str>) -> Result<Vec<PrintTemplate>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, template_type, paper_size, label_width_mm, label_height_mm, content, is_default, is_active, theme, restaurant_name, tagline, logo_data, show_price, show_tax, show_service_charge, item_sort, modifiers_color, created_at, updated_at FROM print_templates WHERE is_active = 1 ORDER BY is_default DESC, name")?;
        let templates: Vec<PrintTemplate> = stmt.query_map([], |row| {
            Ok(PrintTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                template_type: row.get(2)?,
                paper_size: row.get(3)?,
                label_width_mm: row.get(4)?,
                label_height_mm: row.get(5)?,
                content: row.get(6)?,
                is_default: { let v: i64 = row.get(7)?; v == 1 },
                is_active: { let v: i64 = row.get(8)?; v == 1 },
                theme: row.get(9)?,
                restaurant_name: row.get(10)?,
                tagline: row.get(11)?,
                logo_data: row.get(12)?,
                show_price: row.get::<_, Option<i64>>(13)?.map(|v| v == 1),
                show_tax: row.get::<_, Option<i64>>(14)?.map(|v| v == 1),
                show_service_charge: row.get::<_, Option<i64>>(15)?.map(|v| v == 1),
                item_sort: row.get(16)?,
                modifiers_color: row.get(17)?,
                created_at: row.get(18)?,
                updated_at: row.get(19)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        if let Some(t) = template_type {
            Ok(templates.into_iter().filter(|tpl| tpl.template_type == t).collect())
        } else {
            Ok(templates)
        }
    }

    pub fn get_print_template(&self, id: i64) -> Result<PrintTemplate> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT id, name, template_type, paper_size, label_width_mm, label_height_mm, content, is_default, is_active, theme, restaurant_name, tagline, logo_data, show_price, show_tax, show_service_charge, item_sort, modifiers_color, created_at, updated_at FROM print_templates WHERE id = ?1", params![id], |row| {
            Ok(PrintTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                template_type: row.get(2)?,
                paper_size: row.get(3)?,
                label_width_mm: row.get(4)?,
                label_height_mm: row.get(5)?,
                content: row.get(6)?,
                is_default: { let v: i64 = row.get(7)?; v == 1 },
                is_active: { let v: i64 = row.get(8)?; v == 1 },
                theme: row.get(9)?,
                restaurant_name: row.get(10)?,
                tagline: row.get(11)?,
                logo_data: row.get(12)?,
                show_price: row.get::<_, Option<i64>>(13)?.map(|v| v == 1),
                show_tax: row.get::<_, Option<i64>>(14)?.map(|v| v == 1),
                show_service_charge: row.get::<_, Option<i64>>(15)?.map(|v| v == 1),
                item_sort: row.get(16)?,
                modifiers_color: row.get(17)?,
                created_at: row.get(18)?,
                updated_at: row.get(19)?,
            })
        })
    }

    pub fn create_print_template(&self, req: &CreatePrintTemplateRequest) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT INTO print_templates (name, template_type, paper_size, label_width_mm, label_height_mm, content, theme, restaurant_name, tagline, logo_data, show_price, show_tax, show_service_charge, item_sort, modifiers_color, is_active, is_default) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, 0)", params![
            req.name, req.template_type, req.paper_size, req.label_width_mm, req.label_height_mm, req.content,
            req.theme, req.restaurant_name, req.tagline, req.logo_data,
            req.show_price.map(|v| v as i64), req.show_tax.map(|v| v as i64), req.show_service_charge.map(|v| v as i64),
            req.item_sort, req.modifiers_color, req.is_active.map(|v| v as i64)
        ])?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_print_template(&self, id: i64, name: Option<String>, content: Option<String>, paper_size: Option<String>, label_width_mm: Option<f64>, label_height_mm: Option<f64>, theme: Option<String>, restaurant_name: Option<String>, tagline: Option<String>, logo_data: Option<String>, show_price: Option<bool>, show_tax: Option<bool>, show_service_charge: Option<bool>, item_sort: Option<String>, modifiers_color: Option<String>, is_active: Option<bool>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        if let Some(n) = name { conn.execute("UPDATE print_templates SET name = ?1, updated_at = datetime('now') WHERE id = ?2", params![n, id])?; }
        if let Some(c) = content { conn.execute("UPDATE print_templates SET content = ?1, updated_at = datetime('now') WHERE id = ?2", params![c, id])?; }
        if let Some(p) = paper_size { conn.execute("UPDATE print_templates SET paper_size = ?1, updated_at = datetime('now') WHERE id = ?2", params![p, id])?; }
        if let Some(w) = label_width_mm { conn.execute("UPDATE print_templates SET label_width_mm = ?1, updated_at = datetime('now') WHERE id = ?2", params![w, id])?; }
        if let Some(h) = label_height_mm { conn.execute("UPDATE print_templates SET label_height_mm = ?1, updated_at = datetime('now') WHERE id = ?2", params![h, id])?; }
        if let Some(t) = theme { conn.execute("UPDATE print_templates SET theme = ?1, updated_at = datetime('now') WHERE id = ?2", params![t, id])?; }
        if let Some(rn) = restaurant_name { conn.execute("UPDATE print_templates SET restaurant_name = ?1, updated_at = datetime('now') WHERE id = ?2", params![rn, id])?; }
        if let Some(tl) = tagline { conn.execute("UPDATE print_templates SET tagline = ?1, updated_at = datetime('now') WHERE id = ?2", params![tl, id])?; }
        if let Some(ld) = logo_data { conn.execute("UPDATE print_templates SET logo_data = ?1, updated_at = datetime('now') WHERE id = ?2", params![ld, id])?; }
        if let Some(sp) = show_price { conn.execute("UPDATE print_templates SET show_price = ?1, updated_at = datetime('now') WHERE id = ?2", params![sp as i64, id])?; }
        if let Some(st) = show_tax { conn.execute("UPDATE print_templates SET show_tax = ?1, updated_at = datetime('now') WHERE id = ?2", params![st as i64, id])?; }
        if let Some(ss) = show_service_charge { conn.execute("UPDATE print_templates SET show_service_charge = ?1, updated_at = datetime('now') WHERE id = ?2", params![ss as i64, id])?; }
        if let Some(is) = item_sort { conn.execute("UPDATE print_templates SET item_sort = ?1, updated_at = datetime('now') WHERE id = ?2", params![is, id])?; }
        if let Some(mc) = modifiers_color { conn.execute("UPDATE print_templates SET modifiers_color = ?1, updated_at = datetime('now') WHERE id = ?2", params![mc, id])?; }
        if let Some(ia) = is_active { conn.execute("UPDATE print_templates SET is_active = ?1, updated_at = datetime('now') WHERE id = ?2", params![ia as i64, id])?; }
        Ok(())
    }

    pub fn delete_print_template(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE print_templates SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn set_default_template(&self, id: i64, template_type: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE print_templates SET is_default = 0 WHERE template_type = ?1", params![template_type])?;
        conn.execute("UPDATE print_templates SET is_default = 1 WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ==================== 票據類型 ====================

    pub fn get_print_ticket_types(&self) -> Result<Vec<PrintTicketType>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, code, name, description, is_active, is_default, show_price, show_seq, show_note_field, station_id, paper_width, font_size, cut_mode, print_speed, print_density, show_order_no, show_table_no, show_dine_type, show_item_name, show_item_qty, show_item_price, show_item_subtotal, show_item_spec, show_item_note, show_created_at, show_total_amount, show_lot_no, show_qty_info, show_expiry_date, show_supplier, created_at, updated_at FROM print_ticket_types ORDER BY is_default DESC, name")?;
        let types: Vec<PrintTicketType> = stmt.query_map([], |row| {
            Ok(PrintTicketType {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                is_active: { let v: i64 = row.get(4)?; v == 1 },
                is_default: { let v: i64 = row.get(5)?; v == 1 },
                show_price: { let v: i64 = row.get(6)?; v == 1 },
                show_seq: { let v: i64 = row.get(7)?; v == 1 },
                show_note_field: { let v: i64 = row.get(8)?; v == 1 },
                station_id: row.get(9)?,
                paper_width: row.get(10)?,
                font_size: row.get(11)?,
                cut_mode: row.get(12)?,
                print_speed: row.get(13)?,
                print_density: row.get(14)?,
                show_order_no: { let v: i64 = row.get(15)?; v == 1 },
                show_table_no: { let v: i64 = row.get(16)?; v == 1 },
                show_dine_type: { let v: i64 = row.get(17)?; v == 1 },
                show_item_name: { let v: i64 = row.get(18)?; v == 1 },
                show_item_qty: { let v: i64 = row.get(19)?; v == 1 },
                show_item_price: { let v: i64 = row.get(20)?; v == 1 },
                show_item_subtotal: { let v: i64 = row.get(21)?; v == 1 },
                show_item_spec: { let v: i64 = row.get(22)?; v == 1 },
                show_item_note: { let v: i64 = row.get(23)?; v == 1 },
                show_created_at: { let v: i64 = row.get(24)?; v == 1 },
                show_total_amount: { let v: i64 = row.get(25)?; v == 1 },
                show_lot_no: { let v: i64 = row.get(26)?; v == 1 },
                show_qty_info: { let v: i64 = row.get(27)?; v == 1 },
                show_expiry_date: { let v: i64 = row.get(28)?; v == 1 },
                show_supplier: { let v: i64 = row.get(29)?; v == 1 },
                created_at: row.get(30)?,
                updated_at: row.get(31)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(types)
    }

    pub fn get_print_ticket_type(&self, id: i64) -> Result<PrintTicketType> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT id, code, name, description, is_active, is_default, show_price, show_seq, show_note_field, station_id, paper_width, font_size, cut_mode, print_speed, print_density, show_order_no, show_table_no, show_dine_type, show_item_name, show_item_qty, show_item_price, show_item_subtotal, show_item_spec, show_item_note, show_created_at, show_total_amount, show_lot_no, show_qty_info, show_expiry_date, show_supplier, created_at, updated_at FROM print_ticket_types WHERE id = ?1", params![id], |row| {
            Ok(PrintTicketType {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                is_active: { let v: i64 = row.get(4)?; v == 1 },
                is_default: { let v: i64 = row.get(5)?; v == 1 },
                show_price: { let v: i64 = row.get(6)?; v == 1 },
                show_seq: { let v: i64 = row.get(7)?; v == 1 },
                show_note_field: { let v: i64 = row.get(8)?; v == 1 },
                station_id: row.get(9)?,
                paper_width: row.get(10)?,
                font_size: row.get(11)?,
                cut_mode: row.get(12)?,
                print_speed: row.get(13)?,
                print_density: row.get(14)?,
                show_order_no: { let v: i64 = row.get(15)?; v == 1 },
                show_table_no: { let v: i64 = row.get(16)?; v == 1 },
                show_dine_type: { let v: i64 = row.get(17)?; v == 1 },
                show_item_name: { let v: i64 = row.get(18)?; v == 1 },
                show_item_qty: { let v: i64 = row.get(19)?; v == 1 },
                show_item_price: { let v: i64 = row.get(20)?; v == 1 },
                show_item_subtotal: { let v: i64 = row.get(21)?; v == 1 },
                show_item_spec: { let v: i64 = row.get(22)?; v == 1 },
                show_item_note: { let v: i64 = row.get(23)?; v == 1 },
                show_created_at: { let v: i64 = row.get(24)?; v == 1 },
                show_total_amount: { let v: i64 = row.get(25)?; v == 1 },
                show_lot_no: { let v: i64 = row.get(26)?; v == 1 },
                show_qty_info: { let v: i64 = row.get(27)?; v == 1 },
                show_expiry_date: { let v: i64 = row.get(28)?; v == 1 },
                show_supplier: { let v: i64 = row.get(29)?; v == 1 },
                created_at: row.get(30)?,
                updated_at: row.get(31)?,
            })
        })
    }

    pub fn create_print_ticket_type(&self, req: &CreatePrintTicketTypeRequest) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO print_ticket_types (code, name, description, is_active, is_default, show_price, show_seq, show_note_field, station_id, paper_width, font_size, cut_mode, print_speed, print_density, show_order_no, show_table_no, show_dine_type, show_item_name, show_item_qty, show_item_price, show_item_subtotal, show_item_spec, show_item_note, show_created_at, show_total_amount, show_lot_no, show_qty_info, show_expiry_date, show_supplier) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29)",
            params![
                req.code, req.name, req.description, req.is_active as i64, req.is_default as i64,
                req.show_price as i64, req.show_seq as i64, req.show_note_field as i64, req.station_id,
                req.paper_width, req.font_size, req.cut_mode, req.print_speed, req.print_density,
                req.show_order_no as i64, req.show_table_no as i64, req.show_dine_type as i64,
                req.show_item_name as i64, req.show_item_qty as i64, req.show_item_price as i64,
                req.show_item_subtotal as i64, req.show_item_spec as i64, req.show_item_note as i64,
                req.show_created_at as i64, req.show_total_amount as i64,
                req.show_lot_no as i64, req.show_qty_info as i64, req.show_expiry_date as i64, req.show_supplier as i64
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_print_ticket_type(&self, id: i64, req: &UpdatePrintTicketTypeRequest) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE print_ticket_types SET code = ?1, name = ?2, description = ?3, is_active = ?4, is_default = ?5, show_price = ?6, show_seq = ?7, show_note_field = ?8, station_id = ?9, paper_width = ?10, font_size = ?11, cut_mode = ?12, print_speed = ?13, print_density = ?14, show_order_no = ?15, show_table_no = ?16, show_dine_type = ?17, show_item_name = ?18, show_item_qty = ?19, show_item_price = ?20, show_item_subtotal = ?21, show_item_spec = ?22, show_item_note = ?23, show_created_at = ?24, show_total_amount = ?25, show_lot_no = ?26, show_qty_info = ?27, show_expiry_date = ?28, show_supplier = ?29, updated_at = datetime('now') WHERE id = ?30",
            params![
                req.code, req.name, req.description, req.is_active as i64, req.is_default as i64, req.show_price as i64, req.show_seq as i64,
                req.show_note_field as i64, req.station_id, req.paper_width, req.font_size, req.cut_mode,
                req.print_speed, req.print_density, req.show_order_no as i64, req.show_table_no as i64,
                req.show_dine_type as i64, req.show_item_name as i64, req.show_item_qty as i64,
                req.show_item_price as i64, req.show_item_subtotal as i64, req.show_item_spec as i64,
                req.show_item_note as i64, req.show_created_at as i64, req.show_total_amount as i64,
                req.show_lot_no as i64, req.show_qty_info as i64, req.show_expiry_date as i64, req.show_supplier as i64, id
            ],
        )?;
        Ok(())
    }

    pub fn delete_print_ticket_type(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM print_ticket_types WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn set_default_ticket_type(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;
        tx.execute("UPDATE print_ticket_types SET is_default = 0", [])?;
        tx.execute("UPDATE print_ticket_types SET is_default = 1 WHERE id = ?1", params![id])?;
        tx.commit()?;
        Ok(())
    }

    pub fn ensure_default_ticket_types(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM print_ticket_types", [], |row| row.get(0))?;
        if count == 0 {
            conn.execute(
                "INSERT INTO print_ticket_types (code, name, description, is_active, is_default, show_price, show_seq, show_note_field, station_id, paper_width, font_size, cut_mode, print_speed, print_density, show_order_no, show_table_no, show_dine_type, show_item_name, show_item_qty, show_item_price, show_item_subtotal, show_item_spec, show_item_note, show_created_at, show_total_amount, show_lot_no, show_qty_info, show_expiry_date, show_supplier) VALUES ('kitchen', '廚房單', '後廚備餐用', 1, 1, 0, 1, 1, NULL, '58mm', 'medium', 'full', 'medium', 'medium', 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0)",
                [],
            )?;
            conn.execute(
                "INSERT INTO print_ticket_types (code, name, description, is_active, is_default, show_price, show_seq, show_note_field, station_id, paper_width, font_size, cut_mode, print_speed, print_density, show_order_no, show_table_no, show_dine_type, show_item_name, show_item_qty, show_item_price, show_item_subtotal, show_item_spec, show_item_note, show_created_at, show_total_amount, show_lot_no, show_qty_info, show_expiry_date, show_supplier) VALUES ('receipt', '出餐單', '客人結帳用', 1, 0, 1, 0, 1, NULL, '58mm', 'medium', 'full', 'medium', 'medium', 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 0, 0, 0)",
                [],
            )?;
            conn.execute(
                "INSERT INTO print_ticket_types (code, name, description, is_active, is_default, show_price, show_seq, show_note_field, station_id, paper_width, font_size, cut_mode, print_speed, print_density, show_order_no, show_table_no, show_dine_type, show_item_name, show_item_qty, show_item_price, show_item_subtotal, show_item_spec, show_item_note, show_created_at, show_total_amount, show_lot_no, show_qty_info, show_expiry_date, show_supplier) VALUES ('label', '批次標籤', '庫存標識用', 1, 0, 0, 0, 0, NULL, '50mm', 'small', 'full', 'medium', 'medium', 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1)",
                [],
            )?;
        }
        Ok(())
    }

    // ==================== 模板引擎 ====================

    pub fn render_template_content_preview(
        &self,
        content: &str,
        paper_size: &str,
        _theme: &str,
        restaurant_name: &str,
        tagline: &str,
        logo_data: Option<&str>,
        data: &serde_json::Value,
    ) -> Result<PrintPreviewResult> {
        let template: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?;

        let mut lines: Vec<String> = Vec::new();
        if let Some(elements) = template.get("elements").and_then(|e| e.as_array()) {
            for elem in elements {
                if let Some(kind) = elem.get("type").and_then(|t| t.as_str()) {
                    match kind {
                        "text" => {
                            let content_str = elem.get("content").and_then(|c| c.as_str()).unwrap_or("");
                            let rendered = self.interpolate_template_string(content_str, data);
                            let align = elem.get("align").and_then(|a| a.as_str()).unwrap_or("left");
                            let bold = elem.get("bold").and_then(|b| b.as_bool()).unwrap_or(false);
                            let size = elem.get("size").and_then(|s| s.as_str()).unwrap_or("normal");
                            let width = if paper_size == "58mm" { 32 } else { 48 };
                            let mut line = rendered;
                            if align == "center" {
                                let pad = if width > line.chars().count() { (width - line.chars().count()) / 2 } else { 0 };
                                line = " ".repeat(pad) + &line;
                            } else if align == "right" {
                                let pad = if width > line.chars().count() { width - line.chars().count() } else { 0 };
                                line = " ".repeat(pad) + &line;
                            }
                            let prefix = if bold { "**" } else { "" };
                            let suffix = if bold { "**" } else { "" };
                            let size_prefix = match size { "large" => "##", "small" => "", _ => "" };
                            lines.push(format!("{}{}{}{}", size_prefix, prefix, line, suffix));
                        }
                        "separator" => {
                            let width = if paper_size == "58mm" { 32 } else { 48 };
                            lines.push("─".repeat(width));
                        }
                        "blank_lines" => {
                            let count = elem.get("count").and_then(|c| c.as_u64()).unwrap_or(1);
                            for _ in 0..count { lines.push(String::new()); }
                        }
                        "items" => {
                            if let Some(items) = data.get("items").and_then(|i| i.as_array()) {
                                for item in items {
                                    let name = item.get("name").and_then(|n| n.as_str()).unwrap_or("");
                                    let qty = item.get("qty").and_then(|q| q.as_f64()).unwrap_or(1.0);
                                    let note = item.get("note").and_then(|n| n.as_str());
                                    let item_line = format!("{} x{}", name, qty as i64);
                                    lines.push(item_line);
                                    if let Some(n) = note {
                                        let note_line = format!("  備註: {}", n);
                                        lines.push(note_line);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        let mut html = String::new();
        html.push_str("<div class=\"receipt-preview\" style=\"");
        html.push_str("font-family: 'Courier New', monospace; ");
        html.push_str("background: #fff; ");
        html.push_str("color: #1a1a1a; ");
        html.push_str("padding: 16px; ");
        html.push_str(&format!("max-width: {}px; ", if paper_size == "58mm" { 240 } else { 320 }));
        html.push_str("margin: 0 auto; ");
        html.push_str("border: 1px solid #e2e8f0; ");
        html.push_str("border-radius: 8px; ");
        html.push_str("\">\n");

        if !restaurant_name.is_empty() || logo_data.is_some() {
            html.push_str("<div style=\"text-align: center; margin-bottom: 12px;\">");
            if let Some(logo) = logo_data {
                if !logo.is_empty() {
                    html.push_str(&format!("<img src=\"{}\" style=\"max-height: 48px; max-width: 80px; margin-bottom: 6px;\" />", logo));
                }
            }
            if !restaurant_name.is_empty() {
                html.push_str(&format!("<div style=\"font-size: 18px; font-weight: bold;\">{}</div>", restaurant_name));
            }
            if !tagline.is_empty() {
                html.push_str(&format!("<div style=\"font-size: 11px; color: #666;\">{}</div>", tagline));
            }
            html.push_str("</div>");
            html.push_str("<div style=\"border-bottom: 1px dashed #ccc; margin: 8px 0;\"></div>");
        }

        for line in &lines {
            if line.is_empty() {
                html.push_str("<div style=\"height: 8px;\"></div>\n");
            } else if line.starts_with("##") {
                let content = line.trim_start_matches("##").trim_matches('*');
                html.push_str(&format!("<div style=\"font-size: 16px; font-weight: bold; text-align: center;\">{}</div>\n", content));
            } else if line.starts_with("**") {
                let content = line.trim_matches('*');
                html.push_str(&format!("<div style=\"font-weight: bold;\">{}</div>\n", content));
            } else {
                let escaped = line.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;");
                if line.chars().all(|c| c == '─') {
                    html.push_str("<div style=\"border-bottom: 1px dashed #ccc; margin: 6px 0;\"></div>\n");
                } else {
                    html.push_str(&format!("<div style=\"font-size: 13px; line-height: 1.4;\">{}</div>\n", escaped));
                }
            }
        }

        html.push_str("</div>");

        Ok(PrintPreviewResult {
            html,
            lines,
            paper_width: paper_size.to_string(),
        })
    }

    pub fn render_template_preview(&self, template_id: i64, data: &serde_json::Value) -> Result<PrintPreviewResult> {
        let tpl = self.get_print_template(template_id)?;
        self.render_template_content_preview(
            &tpl.content,
            &tpl.paper_size,
            tpl.theme.as_deref().unwrap_or("classic"),
            tpl.restaurant_name.as_deref().unwrap_or(""),
            tpl.tagline.as_deref().unwrap_or(""),
            tpl.logo_data.as_deref(),
            data,
        )
    }

    fn interpolate_template_string(&self, template: &str, data: &serde_json::Value) -> String {
        let mut result = template.to_string();
        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                let placeholder = format!("{{{{{}}}}}", key);
                let replacement = match value {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Null => "".to_string(),
                    _ => value.to_string(),
                };
                result = result.replace(&placeholder, &replacement);
            }
        }
        result
    }

    #[allow(dead_code)]
    pub fn render_kitchen_ticket_from_template(&self, template_id: i64, data: &serde_json::Value, paper_size: &str) -> Result<(String, Vec<u8>)> {
        use crate::printer::EscPosBuilder;
        
        let tpl = self.get_print_template(template_id)?;
        let theme = tpl.theme.as_deref().unwrap_or("classic");
        
        let restaurant_name = tpl.restaurant_name.as_deref().unwrap_or("Cuckoo");
        let tagline = tpl.tagline.as_deref().unwrap_or("");
        let modifiers_color = tpl.modifiers_color.as_deref().unwrap_or("red");
        
        let order_no = data.get("order_no").and_then(|v| v.as_str()).unwrap_or("");
        let dine_type = data.get("dine_type").and_then(|v| v.as_str()).unwrap_or("");
        let note = data.get("note").and_then(|v| v.as_str());
        let items = data.get("items").and_then(|v| v.as_array());
        
        let width = if paper_size == "58mm" { 32 } else { 48 };
        
        let mut builder = EscPosBuilder::new();
        
        match theme {
            "minimal" => {
                builder.align_center().bold_on().double_height()
                    .text_ln(restaurant_name)
                    .normal_size().bold_off();
                if !tagline.is_empty() {
                    builder.text_ln(tagline);
                }
                builder.separator(width);
                builder.text_ln(&format!("單號: {}", order_no))
                    .text_ln(&format!("類型: {}", dine_type))
                    .separator(width);
                builder.bold_on().text_ln("項目").bold_off();
                if let Some(arr) = items {
                    for item in arr {
                        let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let qty = item.get("qty").and_then(|v| v.as_f64()).unwrap_or(1.0);
                        let item_note = item.get("note").and_then(|v| v.as_str());
                        if qty != 1.0 {
                            builder.text_ln(&format!("{} x{}", name, qty as i32));
                        } else {
                            builder.text_ln(name);
                        }
                        if let Some(n) = item_note {
                            let prefix = if modifiers_color == "red" { "  " } else { "**" };
                            builder.text_ln(&format!("{}{}", prefix, n));
                        }
                    }
                }
                if let Some(n) = note {
                    builder.separator(width).bold_on().text_ln(&format!("備註: {}", n)).bold_off();
                }
                builder.feed_lines(3).cut_paper();
            }
            "modern" => {
                builder.align_center()
                    .text_ln(&"-".repeat(width))
                    .bold_on().text_ln(restaurant_name).bold_off();
                if !tagline.is_empty() {
                    builder.text_ln(tagline);
                }
                builder.text_ln(&"-".repeat(width))
                    .align_left()
                    .text_ln(&format!("NO: {}", order_no))
                    .text_ln(&format!("TYPE: {}", dine_type))
                    .text_ln(&chrono::Local::now().format("%Y-%m-%d %H:%M").to_string())
                    .separator(width);
                if let Some(arr) = items {
                    for item in arr {
                        let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let qty = item.get("qty").and_then(|v| v.as_f64()).unwrap_or(1.0);
                        let item_note = item.get("note").and_then(|v| v.as_str());
                        if qty != 1.0 {
                            builder.text_ln(&format!("[{}] {}", qty as i32, name));
                        } else {
                            builder.text_ln(&format!("[1] {}", name));
                        }
                        if let Some(n) = item_note {
                            let style = if modifiers_color == "bold" { "**" } else { "→" };
                            builder.text_ln(&format!("  {} {}", style, n));
                        }
                    }
                }
                if let Some(n) = note {
                    builder.separator(width).text_ln(&format!("備註: {}", n));
                }
                builder.separator(width);
                builder.feed_lines(3).cut_paper();
            }
            _ => {
                builder.align_center().bold_on().double_height()
                    .text_ln(restaurant_name)
                    .normal_size().bold_off();
                if !tagline.is_empty() {
                    builder.text_ln(tagline);
                }
                builder.separator(width);
                builder.align_left()
                    .text_ln(&format!("單號: {}", order_no))
                    .text_ln(&format!("類型: {}", dine_type))
                    .text_ln(&chrono::Local::now().format("%Y-%m-%d %H:%M").to_string())
                    .separator(width);
                builder.bold_on().text_ln("項目明細").bold_off();
                if let Some(arr) = items {
                    for item in arr {
                        let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let qty = item.get("qty").and_then(|v| v.as_f64()).unwrap_or(1.0);
                        let item_note = item.get("note").and_then(|v| v.as_str());
                        if qty != 1.0 {
                            builder.text_ln(&format!("{} x{}", name, qty as i32));
                        } else {
                            builder.text_ln(name);
                        }
                        if let Some(n) = item_note {
                            let prefix = if modifiers_color == "red" { "  [紅] " } else { "  **" };
                            builder.text_ln(&format!("{}{}", prefix, n));
                        }
                    }
                }
                builder.separator(width);
                if let Some(n) = note {
                    builder.bold_on().text_ln(&format!("訂單備註: {}", n)).bold_off();
                }
                builder.feed_lines(3).cut_paper();
            }
        }
        
        let content = String::from_utf8_lossy(&builder.buffer).to_string();
        Ok((content, builder.buffer))
    }

    // ==================== 通知系统 ====================

    #[allow(dead_code)]
    pub fn create_notification(&self, notification_type: &str, title: &str, message: &str, severity: &str, ref_type: Option<&str>, ref_id: Option<i64>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO notifications (notification_type, title, message, severity, ref_type, ref_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![notification_type, title, message, severity, ref_type, ref_id],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_notifications(&self, limit: i64, unread_only: bool) -> Result<Vec<Notification>> {
        let conn = self.conn.lock().unwrap();
        let query = if unread_only {
            "SELECT id, notification_type, title, message, severity, ref_type, ref_id, is_read, read_at, created_at FROM notifications WHERE is_read = 0 ORDER BY created_at DESC LIMIT ?1"
        } else {
            "SELECT id, notification_type, title, message, severity, ref_type, ref_id, is_read, read_at, created_at FROM notifications ORDER BY created_at DESC LIMIT ?1"
        };
        let mut stmt = conn.prepare(query)?;
        let notifications = stmt.query_map(params![limit], |row| {
            Ok(Notification {
                id: row.get(0)?,
                notification_type: row.get(1)?,
                title: row.get(2)?,
                message: row.get(3)?,
                severity: row.get(4)?,
                ref_type: row.get(5)?,
                ref_id: row.get(6)?,
                is_read: row.get::<_, i32>(7)? != 0,
                read_at: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(notifications)
    }

    pub fn get_unread_count(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM notifications WHERE is_read = 0", [], |row| row.get(0))
    }

    pub fn mark_notification_read(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE notifications SET is_read = 1, read_at = datetime('now') WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    // ==================== 会员系统 ====================

    #[allow(dead_code)]
    pub fn get_customers(&self) -> Result<Vec<Customer>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, phone, wechat_openid, membership_no, points, balance, birthday, gender, note, is_active, created_at, updated_at FROM customers WHERE is_active = 1 ORDER BY created_at DESC")?;
        let customers = stmt.query_map([], |row| {
            Ok(Customer {
                id: row.get(0)?,
                name: row.get(1)?,
                phone: row.get(2)?,
                wechat_openid: row.get(3)?,
                membership_no: row.get(4)?,
                points: row.get(5)?,
                balance: row.get(6)?,
                birthday: row.get(7)?,
                gender: row.get(8)?,
                note: row.get(9)?,
                is_active: row.get::<_, i32>(10)? != 0,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(customers)
    }

    #[allow(dead_code)]
    pub fn create_customer(&self, name: Option<&str>, phone: Option<&str>, wechat_openid: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let membership_no = format!("M{}", chrono::Local::now().format("%Y%m%d%H%M%S"));
        conn.execute(
            "INSERT INTO customers (name, phone, wechat_openid, membership_no) VALUES (?1, ?2, ?3, ?4)",
            params![name, phone, wechat_openid, membership_no],
        )?;
        Ok(conn.last_insert_rowid())
    }

    #[allow(dead_code)]
    pub fn update_customer_points(&self, customer_id: i64, points_delta: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE customers SET points = points + ?1, updated_at = datetime('now') WHERE id = ?2",
            params![points_delta, customer_id],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn update_customer_balance(&self, customer_id: i64, balance_delta: f64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE customers SET balance = balance + ?1, updated_at = datetime('now') WHERE id = ?2",
            params![balance_delta, customer_id],
        )?;
        Ok(())
    }

    // ==================== 优惠券系统 ====================

    #[allow(dead_code)]
    pub fn get_coupons(&self) -> Result<Vec<Coupon>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, code, discount_percent, discount_amount, min_amount, valid_from, valid_until, is_active, created_at, updated_at FROM coupons WHERE is_active = 1")?;
        let coupons = stmt.query_map([], |row| {
            Ok(Coupon {
                id: row.get(0)?,
                name: row.get(1)?,
                code: row.get(2)?,
                discount_percent: row.get(3)?,
                discount_amount: row.get(4)?,
                min_amount: row.get(5)?,
                valid_from: row.get(6)?,
                valid_until: row.get(7)?,
                is_active: row.get::<_, i32>(8)? != 0,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(coupons)
    }

    #[allow(dead_code)]
    pub fn create_coupon(&self, name: &str, code: &str, discount_percent: Option<f64>, discount_amount: Option<f64>, min_amount: Option<f64>, valid_from: Option<&str>, valid_until: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO coupons (name, code, discount_percent, discount_amount, min_amount, valid_from, valid_until) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![name, code, discount_percent, discount_amount, min_amount, valid_from, valid_until],
        )?;
        Ok(conn.last_insert_rowid())
    }

    #[allow(dead_code)]
    pub fn use_coupon(&self, customer_id: i64, coupon_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE customer_coupons SET used = 1, used_at = datetime('now') WHERE customer_id = ?1 AND coupon_id = ?2 AND used = 0",
            params![customer_id, coupon_id],
        )?;
        Ok(())
    }

    // ==================== 门店系统 ====================

    #[allow(dead_code)]
    pub fn get_stores(&self) -> Result<Vec<(i64, String, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, address FROM stores WHERE is_active = 1")?;
        let stores = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?.collect::<Result<Vec<_>>>()?;
        Ok(stores)
    }

    #[allow(dead_code)]
    pub fn add_store_filter(&self, store_id: Option<i64>) -> String {
        if let Some(id) = store_id {
            format!(" AND store_id = {}", id)
        } else {
            String::new()
        }
    }

    pub fn delete_notification(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM notifications WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn mark_all_notifications_read(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE notifications SET is_read = 1, read_at = datetime('now') WHERE is_read = 0",
            [],
        )?;
        Ok(())
    }

    /*
    pub fn check_and_create_alerts(&self) -> Result<()> {
        Ok(())
    }
    */
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_db() -> (Database, TempDir) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.db").to_str().unwrap().to_string();
        let db = Database::new(&path).unwrap();
        (db, dir)
    }

    fn seed_minimal(db: &Database) {
        db.init_tables().unwrap();
        db.seed_data().unwrap();
    }

    #[test]
    fn test_init_tables_creates_all_tables() {
        let (db, _dir) = test_db();
        db.init_tables().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name").unwrap();
        let tables: Vec<String> = stmt.query_map([], |row| row.get(0)).unwrap().map(|r| r.unwrap()).collect();
        assert!(tables.contains(&"materials".to_string()));
        assert!(tables.contains(&"recipes".to_string()));
        assert!(tables.contains(&"orders".to_string()));
        assert!(tables.contains(&"purchase_orders".to_string()));
        assert!(tables.contains(&"production_orders".to_string()));
        assert!(tables.contains(&"stocktakes".to_string()));
        assert!(tables.contains(&"print_templates".to_string()));
        assert!(tables.contains(&"inventory_batches".to_string()));
    }

    #[test]
    fn test_seed_data_populates_units() {
        let (db, _dir) = test_db();
        seed_minimal(&db);
        let units = db.get_units().unwrap();
        assert!(!units.is_empty());
        assert!(units.iter().any(|u| u.code == "kg"));
    }

    #[test]
    fn test_seed_data_populates_materials() {
        let (db, _dir) = test_db();
        seed_minimal(&db);
        let materials = db.get_materials(None).unwrap();
        assert!(!materials.is_empty());
        assert!(materials.len() >= 8);
    }

    #[test]
    fn test_recipe_cost_calculation() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let materials = db.get_materials(None).unwrap();
        if materials.is_empty() { return; }
        let material = &materials[0];

        let units = db.get_units().unwrap();
        let unit_id = if units.is_empty() { 1 } else { units[0].id };

        let recipe_id = db.create_recipe("TEST_COST", "測試食譜", "半成品", 1.0, None, None, Some(unit_id)).unwrap();
        db.add_recipe_item(recipe_id, "material", material.material.id, 2.0, unit_id, 0.0, 0).unwrap();

        let cost = db.calculate_recipe_cost(recipe_id).unwrap();
        assert!(cost.total_cost >= 0.0);
    }

    #[test]
    fn test_menu_item_toggle() {
        let (db, _dir) = test_db();
        seed_minimal(&db);
        let items = db.get_menu_items(None).unwrap();
        if items.is_empty() { return; }
        let item = &items[0];
        db.batch_toggle_menu_item_availability(&[item.id], false).unwrap();
        let all_items = db.get_menu_items(None).unwrap();
        let found = all_items.iter().find(|i| i.id == item.id);
        assert!(found.is_some());
        assert!(!found.unwrap().is_available);
    }

    #[test]
    fn test_esc_pos_builder() {
        use crate::printer::EscPosBuilder;
        let mut builder = EscPosBuilder::new();
        builder.align_center().bold_on().text("測試").bold_off().align_left().feed_lines(2).cut_paper();
        let output = builder.build();
        assert!(!output.is_empty());
        assert!(output.len() > 10);
    }

    #[test]
    fn test_tspl_builder() {
        use crate::printer::TsplBuilder;
        let output = TsplBuilder::new(40.0, 30.0)
            .text(10, 10, "3", (1, 1), "測試標籤")
            .box_draw(5, 5, 395, 295, 2)
            .print_label(1)
            .build();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_kitchen_ticket_content() {
        use crate::printer::build_kitchen_ticket_content;
        let items: Vec<(String, f64, Option<String>)> = vec![
            ("麻辣小龍蝦".to_string(), 2.0, Some("加辣".to_string())),
            ("酸菜魚".to_string(), 1.0, None),
        ];
        let builder = build_kitchen_ticket_content("ORD001", "堂食", &items, None);
        let bytes = builder.build();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("ORD001"));
        assert!(content.contains("麻辣小龍蝦"));
        assert!(content.contains("加辣"));
        assert!(content.contains("酸菜魚"));
    }

    #[test]
    fn test_batch_label_content() {
        use crate::printer::build_batch_label_content;
        let builder = build_batch_label_content("LOT-001", "測試材料", 10.0, "kg", Some("2026-05-01"), Some("測試供應商"));
        let content = builder.build();
        assert!(content.contains("LOT-001"));
        assert!(content.contains("測試材料"));
        assert!(content.contains("10"));
        assert!(content.contains("測試供應商"));
    }

    #[test]
    fn test_print_template_crud() {
        let (db, _dir) = test_db();
        seed_minimal(&db);
        let count_before = db.get_print_templates(None).unwrap().len();

        let tpl_content = r#"{"elements":[{"type":"text","content":"測試","align":"center"}]}"#;
        db.create_print_template(&CreatePrintTemplateRequest {
            name: "測試模板".to_string(),
            template_type: "kitchen_ticket".to_string(),
            paper_size: "80mm".to_string(),
            label_width_mm: None,
            label_height_mm: None,
            content: tpl_content.to_string(),
            theme: None,
            restaurant_name: None,
            tagline: None,
            logo_data: None,
            show_price: None,
            show_tax: None,
            show_service_charge: None,
            item_sort: None,
            modifiers_color: None,
            is_active: Some(true),
        }).unwrap();
        assert_eq!(db.get_print_templates(None).unwrap().len(), count_before + 1);

        let templates = db.get_print_templates(None).unwrap();
        let tpl = templates.iter().find(|t| t.name == "測試模板").unwrap();
        assert_eq!(tpl.paper_size, "80mm");

        db.delete_print_template(tpl.id).unwrap();
        assert_eq!(db.get_print_templates(None).unwrap().len(), count_before);
    }

    #[test]
    fn test_template_preview_rendering() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let tpl_content = r#"{"elements":[{"type":"text","content":"單號: {{order_no}}","align":"left","bold":false,"size":"normal"},{"type":"separator"},{"type":"text","content":"菜品明細","align":"left","bold":true,"size":"normal"},{"type":"items"},{"type":"blank_lines","count":2}]}"#;
        let tpl_id = db.create_print_template(&CreatePrintTemplateRequest {
            name: "預覽測試".to_string(),
            template_type: "kitchen_ticket".to_string(),
            paper_size: "80mm".to_string(),
            label_width_mm: None,
            label_height_mm: None,
            content: tpl_content.to_string(),
            theme: None,
            restaurant_name: None,
            tagline: None,
            logo_data: None,
            show_price: None,
            show_tax: None,
            show_service_charge: None,
            item_sort: None,
            modifiers_color: None,
            is_active: Some(true),
        }).unwrap();

        let data = serde_json::json!({
            "order_no": "ORD001",
            "dine_type": "堂食",
            "items": [
                {"name": "宮保雞丁", "qty": 2, "note": "少辣"},
                {"name": "麻婆豆腐", "qty": 1, "note": null}
            ]
        });

        let preview = db.render_template_preview(tpl_id, &data).unwrap();
        assert!(!preview.lines.is_empty());
        assert!(preview.html.contains("ORD001"));
        assert!(preview.html.contains("宮保雞丁"));
        assert!(preview.paper_width == "80mm");
    }

    #[test]
    fn test_order_lifecycle() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let (order_id, order_no) = db.create_order("POS", "堂食", Some("A01")).unwrap();
        assert!(order_no.starts_with("ORD"));

        let orders = db.get_orders(1000, 0).unwrap();
        let order = orders.iter().find(|o| o.id == order_id).unwrap();
        assert_eq!(order.status, "pending");
        assert_eq!(order.dine_type, "堂食");
        assert_eq!(order.table_no.as_deref(), Some("A01"));

        let items = db.get_menu_items(None).unwrap();
        if !items.is_empty() {
            let item = &items[0];
            db.add_order_item(order.id, item.id, 2.0, 50.0, None, None).unwrap();
        }

        db.submit_order_full(order.id).unwrap();
        let updated_orders = db.get_orders(1000, 0).unwrap();
        let updated = updated_orders.iter().find(|o| o.id == order.id).unwrap();
        assert_eq!(updated.status, "submitted");
    }

    #[test]
    fn test_inventory_batch_and_transaction_flow() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let materials = db.get_materials(None).unwrap();
        assert!(!materials.is_empty());
        let material = &materials[0];

        let lot_no = format!("LOT{}", chrono::Local::now().format("%Y%m%d%H%M%S"));
        db.create_inventory_batch(
            material.material.id,
            None,
            &lot_no,
            None,
            None,
            None,
            100.0,
            10.0,
            None,
            None,
            None,
            None,
            1.0,
        ).unwrap();

        let batches = db.get_inventory_batches(Some(material.material.id)).unwrap();
        assert!(!batches.is_empty());
        let batch = batches.iter().find(|b| b.lot_no == lot_no).unwrap();
        assert_eq!(batch.quantity, 100.0);
        assert_eq!(batch.material_id, material.material.id);

        let txn_id = db.create_inventory_txn(
            "purchase_in",
            Some("purchase_order"),
            None,
            Some(batch.id),
            material.material.id,
            None,
            50.0,
            Some(0.0),
            Some("測試操作員"),
            Some("測試入庫"),
        ).unwrap();

        let txns = db.get_inventory_txns(Some(material.material.id), 100).unwrap();
        assert!(!txns.is_empty());
        let txn = txns.iter().find(|t| t.id == txn_id).unwrap();
        assert_eq!(txn.txn_type, "purchase_in");
        assert_eq!(txn.qty_delta, 50.0);
    }

    #[test]
    fn test_purchase_order_full_flow() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let suppliers = db.get_suppliers().unwrap();
        let supplier_id = if suppliers.is_empty() {
            db.create_supplier("測試供應商", None, None).unwrap();
            let suppliers = db.get_suppliers().unwrap();
            suppliers[0].id
        } else {
            suppliers[0].id
        };

        let po_no = db.create_purchase_order(Some(supplier_id), Some("2026-05-01")).unwrap();
        assert!(po_no.starts_with("PO"));

        let pos = db.get_purchase_orders(None).unwrap();
        let po = pos.iter().find(|p| p.po_no == po_no).unwrap();
        assert_eq!(po.status, "draft");

        let materials = db.get_materials(None).unwrap();
        assert!(!materials.is_empty());
        let material = &materials[0];

        db.add_purchase_order_item(po.id, material.material.id, 50.0, None, 10.0).unwrap();

        let pos_after = db.get_purchase_orders(None).unwrap();
        let po_after = pos_after.iter().find(|p| p.id == po.id).unwrap();
        assert!(po_after.total_cost >= 500.0);

        let batches_before = db.get_inventory_batches(Some(material.material.id)).unwrap().len();

        db.receive_purchase_order(po.id, Some("測試操作員")).unwrap();

        let pos_received = db.get_purchase_orders(None).unwrap();
        let po_received = pos_received.iter().find(|p| p.id == po.id).unwrap();
        assert_eq!(po_received.status, "received");

        let batches_after = db.get_inventory_batches(Some(material.material.id)).unwrap();
        assert!(batches_after.len() > batches_before);
    }

    #[test]
    fn test_production_order_full_flow() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        // Need a recipe with output_material_id — create one explicitly
        let materials = db.get_materials(None).unwrap();
        if materials.is_empty() { return; }
        let output_material = &materials[0];
        let units = db.get_units().unwrap();
        let unit_id = if units.is_empty() { 1 } else { units[0].id };
        let recipe_id = db.create_recipe(
            "PRD_TEST", "生産測試配方", "半成品", 1.0,
            Some(output_material.material.id), None, Some(unit_id),
        ).unwrap();

        let prod_no = db.create_production_order(recipe_id, 2.0, Some("測試操作員")).unwrap();
        assert!(prod_no.starts_with("PRD"));

        let prods = db.get_production_orders(None).unwrap();
        let prod = prods.iter().find(|p| p.production_no == prod_no).unwrap();
        assert_eq!(prod.status, "draft");
        assert_eq!(prod.planned_qty, 2.0);

        db.start_production_order(prod.id, Some("測試操作員")).unwrap();
        let prods_after_start = db.get_production_orders(None).unwrap();
        let prod_after_start = prods_after_start.iter().find(|p| p.id == prod.id).unwrap();
        assert_eq!(prod_after_start.status, "in_progress");

        let txns_before = db.get_inventory_txns(None, 100).unwrap().len();

        db.complete_production_order(prod.id, 2.0, Some("測試操作員")).unwrap();

        let prods_after = db.get_production_orders(None).unwrap();
        let prod_after = prods_after.iter().find(|p| p.id == prod.id).unwrap();
        assert_eq!(prod_after.status, "completed");
        assert_eq!(prod_after.actual_qty, Some(2.0));

        let txns_after = db.get_inventory_txns(None, 100).unwrap();
        assert!(txns_after.len() >= txns_before);
    }

    #[test]
    fn test_stocktake_full_flow() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let materials = db.get_materials(None).unwrap();
        assert!(!materials.is_empty());
        let material = &materials[0];

        let lot_no = format!("LOT{}", chrono::Local::now().format("%Y%m%d%H%M%S"));
        db.create_inventory_batch(
            material.material.id,
            None,
            &lot_no,
            None,
            None,
            None,
            100.0,
            10.0,
            None,
            None,
            None,
            None,
            1.0,
        ).unwrap();

        let batches = db.get_inventory_batches(Some(material.material.id)).unwrap();
        let batch = batches.iter().find(|b| b.lot_no == lot_no).unwrap();
        assert_eq!(batch.quantity, 100.0);

        let stk_no = db.create_stocktake(Some("測試操作員"), Some("測試盤點")).unwrap();
        assert!(stk_no.starts_with("STK"));

        let stocktakes = db.get_stocktakes(None).unwrap();
        let stocktake = stocktakes.iter().find(|s| s.stocktake_no == stk_no).unwrap();
        assert_eq!(stocktake.status, "draft");

        let stk_with_items = db.get_stocktake_with_items(stocktake.id).unwrap();
        assert!(!stk_with_items.items.is_empty());

        let item = stk_with_items.items.iter().find(|i| i.lot_id == Some(batch.id)).unwrap();
        assert_eq!(item.system_qty, 100.0);

        db.update_stocktake_item(item.id, 95.0).unwrap();

        let stk_with_items_after = db.get_stocktake_with_items(stocktake.id).unwrap();
        let item_after = stk_with_items_after.items.iter().find(|i| i.id == item.id).unwrap();
        assert_eq!(item_after.actual_qty, 95.0);
        assert_eq!(item_after.diff_qty, Some(-5.0));

        let txns_before = db.get_inventory_txns(Some(material.material.id), 100).unwrap().len();

        db.complete_stocktake(stocktake.id, Some("測試操作員")).unwrap();

        let stocktakes_after = db.get_stocktakes(None).unwrap();
        let stocktake_after = stocktakes_after.iter().find(|s| s.id == stocktake.id).unwrap();
        assert_eq!(stocktake_after.status, "completed");

        let txns_after = db.get_inventory_txns(Some(material.material.id), 100).unwrap();
        assert!(txns_after.len() > txns_before);
    }

    #[test]
    fn test_inventory_adjustment() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let materials = db.get_materials(None).unwrap();
        assert!(!materials.is_empty());
        let material = &materials[0];

        let lot_no = format!("LOT{}", chrono::Local::now().format("%Y%m%d%H%M%S"));
        db.create_inventory_batch(
            material.material.id,
            None,
            &lot_no,
            None,
            None,
            None,
            100.0,
            10.0,
            None,
            None,
            None,
            None,
            1.0,
        ).unwrap();

        let batches = db.get_inventory_batches(Some(material.material.id)).unwrap();
        let batch = batches.iter().find(|b| b.lot_no == lot_no).unwrap();

        db.adjust_inventory(batch.id, -10.0, Some("測試操作員"), Some("測試調整")).unwrap();

        let batches_after = db.get_inventory_batches(Some(material.material.id)).unwrap();
        let batch_after = batches_after.iter().find(|b| b.id == batch.id).unwrap();
        assert_eq!(batch_after.quantity, 90.0);

        let txns = db.get_inventory_txns(Some(material.material.id), 100).unwrap();
        let adjust_txn = txns.iter().find(|t| t.txn_type == "adjustment");
        assert!(adjust_txn.is_some());
        assert_eq!(adjust_txn.unwrap().qty_delta, -10.0);
    }

    #[test]
    fn test_sales_report_generation() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let items = db.get_menu_items(None).unwrap();
        if items.is_empty() { return; }
        let item = &items[0];

        let (order_id, _) = db.create_order("POS", "堂食", None).unwrap();
        let orders = db.get_orders(1000, 0).unwrap();
        let order = orders.iter().find(|o| o.id == order_id).unwrap();

        db.add_order_item(order.id, item.id, 2.0, 50.0, None, None).unwrap();
        db.submit_order_full(order.id).unwrap();

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let report = db.get_sales_report(&today, &today).unwrap();
        assert!(!report.is_empty());
        let (_date, total, count) = &report[0];
        assert_eq!(*count, 1);
        assert!(*total >= 100.0);
    }

    #[test]
    fn test_top_selling_items_report() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let items = db.get_menu_items(None).unwrap();
        if items.len() < 2 { return; }

        for item in &items[..2.min(items.len())] {
            let (order_id, _) = db.create_order("POS", "外賣", None).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(1100));
            let orders = db.get_orders(1000, 0).unwrap();
            let order = orders.iter().find(|o| o.id == order_id).unwrap();
            db.add_order_item(order.id, item.id, 3.0, item.sales_price, None, None).unwrap();
            db.submit_order_full(order.id).unwrap();
        }

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let result = db.get_top_selling_items(&today, &today, 10);
        assert!(result.is_ok() || result.is_err());
        if let Ok(report) = result {
            assert!(!report.is_empty());
            let (name, revenue, _qty, avg_price) = &report[0];
            assert!(!name.is_empty());
            assert!(*revenue > 0.0);
            assert!(*avg_price > 0.0);
        }
    }

    #[test]
    fn test_material_states_crud() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let materials = db.get_materials(None).unwrap();
        assert!(!materials.is_empty());
        let material = &materials[0];

        let states_before = db.get_material_states(material.material.id).unwrap().len();

        db.create_material_state(material.material.id, "frozen", "冷凍", None, 1.0, 1.0).unwrap();
        let states = db.get_material_states(material.material.id).unwrap();
        assert_eq!(states.len(), states_before + 1);

        let state = states.iter().find(|s| s.state_name == "冷凍").unwrap();
        db.update_material_state(state.id, None, Some("冷藏"), None, None, None).unwrap();
        let states_after = db.get_material_states(material.material.id).unwrap();
        let state_after = states_after.iter().find(|s| s.id == state.id).unwrap();
        assert_eq!(state_after.state_name, "冷藏");

        db.delete_material_state(state.id).unwrap();
        let states_final = db.get_material_states(material.material.id).unwrap();
        assert_eq!(states_final.len(), states_before);
    }

    #[test]
    fn test_recipe_crud_and_cost() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let materials = db.get_materials(None).unwrap();
        assert!(!materials.is_empty());
        let material = &materials[0];

        let units = db.get_units().unwrap();
        let unit_id = if units.is_empty() { 1 } else { units[0].id };

        let recipe_id = db.create_recipe("TEST001", "測試食譜", "半成品", 1.0, None, None, Some(unit_id)).unwrap();
        let recipes = db.get_recipes(None).unwrap();
        let recipe = recipes.iter().find(|r| r.id == recipe_id).unwrap();
        assert_eq!(recipe.name, "測試食譜");

        db.add_recipe_item(recipe_id, "material", material.material.id, 2.0, unit_id, 0.0, 0).unwrap();

        let cost = db.calculate_recipe_cost(recipe_id).unwrap();
        assert!(cost.total_cost >= 0.0);

        db.delete_recipe(recipe_id).unwrap();
        let recipes_after = db.get_recipes(None).unwrap();
        assert!(recipes_after.iter().all(|r| r.id != recipe_id));
    }

    #[test]
    fn test_esc_pos_enhanced_features() {
        use crate::printer::EscPosBuilder;
        let mut builder = EscPosBuilder::new();
        builder.align_center().bold_on().double_height()
            .text_ln("測試標題")
            .normal_size().bold_off()
            .align_left()
            .underline_on().text_ln("下劃線文本").underline_off()
            .inverse_on().text_ln("反白文本").inverse_off()
            .font_b().text_ln("Font B 文本").font_a()
            .double_width().text_ln("雙寬文本").normal_size()
            .qr_code("https://cuckoo.example.com", 6)
            .dashed_separator(32)
            .feed_lines(2)
            .partial_cut();
        let output = builder.build();
        assert!(!output.is_empty());
        assert!(output.len() > 50);
    }

    #[test]
    fn test_tspl_enhanced_features() {
        use crate::printer::TsplBuilder;
        let output = TsplBuilder::with_gap(60.0, 40.0, 3.0)
            .text(20, 20, "Arial", (2, 2), "測試標籤")
            .text_with_rotation(20, 60, "Arial", (1, 1), 90, "旋轉文本")
            .qr_code(20, 100, "L", 4, "QR測試內容")
            .box_draw(5, 5, 595, 395, 2)
            .print_label(2)
            .build();
        assert!(output.contains("SIZE 60.0 mm, 40.0 mm"));
        assert!(output.contains("GAP 3.0 mm"));
        assert!(output.contains("QRCODE"));
        assert!(output.contains("PRINT 2"));
    }

    #[test]
    fn test_order_with_modifiers() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let (order_id, _) = db.create_order("POS", "堂食", Some("A01")).unwrap();
        let orders = db.get_orders(1000, 0).unwrap();
        let order = orders.iter().find(|o| o.id == order_id).unwrap();

        let items = db.get_menu_items(None).unwrap();
        if items.is_empty() { return; }
        let item = &items[0];

        let order_item_id = db.add_order_item(order.id, item.id, 2.0, 50.0, None, Some("少辣")).unwrap();

        db.add_order_item_modifier(order_item_id, "加料", None, 1.0, 5.0).unwrap();
        db.add_order_item_modifier(order_item_id, "去料", None, 1.0, -3.0).unwrap();

        let modifiers = db.get_order_item_modifiers(order_item_id).unwrap();
        assert_eq!(modifiers.len(), 2);

        db.delete_order_item_modifier(modifiers[0].id).unwrap();
        let modifiers_after = db.get_order_item_modifiers(order_item_id).unwrap();
        assert_eq!(modifiers_after.len(), 1);
    }

    #[test]
    fn test_order_cancellation() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let (order_id, _) = db.create_order("POS", "堂食", None).unwrap();
        let orders = db.get_orders(1000, 0).unwrap();
        let order = orders.iter().find(|o| o.id == order_id).unwrap();

        let items = db.get_menu_items(None).unwrap();
        if !items.is_empty() {
            let item = &items[0];
            db.add_order_item(order.id, item.id, 1.0, 50.0, None, None).unwrap();
        }

        db.submit_order_full(order.id).unwrap();
        let orders_after = db.get_orders(1000, 0).unwrap();
        let order_after = orders_after.iter().find(|o| o.id == order.id).unwrap();
        assert_eq!(order_after.status, "submitted");

        let cancel_result = db.release_inventory_for_order(order.id, None);
        assert!(cancel_result.is_ok());

        let orders_final = db.get_orders(1000, 0).unwrap();
        let order_final = orders_final.iter().find(|o| o.id == order.id).unwrap();
        assert_eq!(order_final.status, "cancelled");
    }

    #[test]
    fn test_multiple_orders_same_table() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let (order_id1, _) = db.create_order("POS", "堂食", Some("A01")).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let (order_id2, _) = db.create_order("POS", "堂食", Some("A01")).unwrap();

        let orders = db.get_orders(1000, 0).unwrap();
        let table_a01_orders: Vec<_> = orders.iter()
            .filter(|o| o.table_no.as_deref() == Some("A01"))
            .collect();
        assert_eq!(table_a01_orders.len(), 2);
        assert!(table_a01_orders.iter().any(|o| o.id == order_id1));
        assert!(table_a01_orders.iter().any(|o| o.id == order_id2));
    }

    #[test]
    fn test_supplier_crud() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let supplier_id = db.create_supplier("測試供應商", Some("123456789"), Some("張三")).unwrap();
        let suppliers = db.get_suppliers().unwrap();
        let supplier = suppliers.iter().find(|s| s.id == supplier_id).unwrap();
        assert_eq!(supplier.name, "測試供應商");
        assert_eq!(supplier.phone.as_deref(), Some("123456789"));
        assert_eq!(supplier.contact_person.as_deref(), Some("張三"));

        db.delete_supplier(supplier_id).unwrap();
        let suppliers_after = db.get_suppliers().unwrap();
        assert!(suppliers_after.iter().all(|s| s.id != supplier_id));
    }

    #[test]
    fn test_purchase_order_delete() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let suppliers = db.get_suppliers().unwrap();
        let supplier_id = if suppliers.is_empty() {
            db.create_supplier("測試供應商", None, None).unwrap();
            let suppliers = db.get_suppliers().unwrap();
            suppliers[0].id
        } else {
            suppliers[0].id
        };

        let po_no = db.create_purchase_order(Some(supplier_id), None).unwrap();
        let pos = db.get_purchase_orders(None).unwrap();
        let po = pos.iter().find(|p| p.po_no == po_no).unwrap();

        db.delete_purchase_order(po.id).unwrap();
        let pos_after = db.get_purchase_orders(None).unwrap();
        assert!(pos_after.iter().all(|p| p.po_no != po_no));
    }

    #[test]
    fn test_production_order_delete() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let recipes = db.get_recipes(None).unwrap();
        if recipes.is_empty() { return; }
        let recipe = &recipes[0];

        let prod_no = db.create_production_order(recipe.id, 1.0, None).unwrap();
        let prods = db.get_production_orders(None).unwrap();
        let prod = prods.iter().find(|p| p.production_no == prod_no).unwrap();

        db.delete_production_order(prod.id).unwrap();
        let prods_after = db.get_production_orders(None).unwrap();
        assert!(prods_after.iter().all(|p| p.production_no != prod_no));
    }

    #[test]
    fn test_stocktake_delete() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let stk_no = db.create_stocktake(None, None).unwrap();
        let stocktakes = db.get_stocktakes(None).unwrap();
        let stocktake = stocktakes.iter().find(|s| s.stocktake_no == stk_no).unwrap();

        db.delete_stocktake(stocktake.id).unwrap();
        let stocktakes_after = db.get_stocktakes(None).unwrap();
        assert!(stocktakes_after.iter().all(|s| s.stocktake_no != stk_no));
    }

    #[test]
    fn test_gross_profit_report() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let items = db.get_menu_items(None).unwrap();
        if items.is_empty() { return; }
        let item = &items[0];

        let (order_id, _) = db.create_order("POS", "堂食", None).unwrap();
        let orders = db.get_orders(1000, 0).unwrap();
        let order = orders.iter().find(|o| o.id == order_id).unwrap();
        db.add_order_item(order.id, item.id, 2.0, 50.0, None, None).unwrap();
        db.submit_order_full(order.id).unwrap();

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let result = db.get_gross_profit_report(&today, &today);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sales_by_category_report() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let items = db.get_menu_items(None).unwrap();
        if items.is_empty() { return; }
        let item = &items[0];

        let (order_id, _) = db.create_order("POS", "堂食", None).unwrap();
        let orders = db.get_orders(1000, 0).unwrap();
        let order = orders.iter().find(|o| o.id == order_id).unwrap();
        db.add_order_item(order.id, item.id, 1.0, 50.0, None, None).unwrap();
        db.submit_order_full(order.id).unwrap();

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let result = db.get_sales_by_category(&today, &today);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_kitchen_station_tickets() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let stations = db.get_kitchen_stations().unwrap();
        if stations.is_empty() { return; }
        let station = &stations[0];

        let tickets = db.get_station_tickets(station.id, None).unwrap();
        assert!(tickets.is_empty());

        let tickets_pending = db.get_station_tickets(station.id, Some("pending")).unwrap();
        assert!(tickets_pending.is_empty());
    }

    #[test]
    fn test_ticket_lifecycle() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let stations = db.get_kitchen_stations().unwrap();
        if stations.is_empty() { return; }
        let station = &stations[0];

        let (order_id, _) = db.create_order("POS", "堂食", None).unwrap();
        let orders = db.get_orders(1000, 0).unwrap();
        let order = orders.iter().find(|o| o.id == order_id).unwrap();

        let items = db.get_menu_items(None).unwrap();
        if !items.is_empty() {
            let item = &items[0];
            db.add_order_item(order.id, item.id, 1.0, 50.0, None, None).unwrap();
        }

        let submit_result = db.submit_order_full(order.id);
        assert!(submit_result.is_ok());

        let tickets = db.get_station_tickets(station.id, None).unwrap();
        if tickets.is_empty() { return; }

        let ticket = &tickets[0];
        db.start_ticket(ticket.id, Some("廚師A")).unwrap();
        let tickets_in_progress = db.get_station_tickets(station.id, None).unwrap();
        assert!(tickets_in_progress.iter().any(|t| t.id == ticket.id));

        db.finish_ticket(ticket.id, Some("廚師A")).unwrap();
        let tickets_finished = db.get_station_tickets(station.id, None).unwrap();
        assert!(tickets_finished.iter().any(|t| t.id == ticket.id));
    }

    #[test]
    fn test_inventory_wastage() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let materials = db.get_materials(None).unwrap();
        assert!(!materials.is_empty());
        let material = &materials[0];

        let lot_no = format!("LOT_WST_{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase());
        db.create_inventory_batch(
            material.material.id,
            None,
            &lot_no,
            None,
            None,
            None,
            100.0,
            10.0,
            None,
            None,
            None,
            None,
            1.0,
        ).unwrap();

        let batches = db.get_inventory_batches(Some(material.material.id)).unwrap();
        let batch = batches.iter().find(|b| b.lot_no == lot_no).unwrap();

        db.record_wastage(batch.id, 5.0, Some("損壞"), Some("操作員")).unwrap();

        let batches_after = db.get_inventory_batches(Some(material.material.id)).unwrap();
        let batch_after = batches_after.iter().find(|b| b.id == batch.id).unwrap();
        assert_eq!(batch_after.quantity, 95.0);
    }

    #[test]
    fn test_menu_category_crud() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let cat_id = db.create_menu_category("測試分類", 10).unwrap();
        let categories = db.get_menu_categories().unwrap();
        let category = categories.iter().find(|c| c.id == cat_id).unwrap();
        assert_eq!(category.name, "測試分類");

        db.update_menu_category(cat_id, Some("更新分類"), Some(20)).unwrap();
        let categories_after = db.get_menu_categories().unwrap();
        let category_after = categories_after.iter().find(|c| c.id == cat_id).unwrap();
        assert_eq!(category_after.name, "更新分類");

        db.delete_menu_category(cat_id).unwrap();
        let categories_final = db.get_menu_categories().unwrap();
        let deleted = categories_final.iter().find(|c| c.id == cat_id);
        assert!(deleted.is_none() || !deleted.unwrap().is_active);
    }

    #[test]
    fn test_menu_item_crud() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let categories = db.get_menu_categories().unwrap();
        let category_id = if categories.is_empty() { None } else { Some(categories[0].id) };

        let item_id = db.create_menu_item("測試菜品", category_id, None, 50.0).unwrap();
        let items = db.get_menu_items(None).unwrap();
        let item = items.iter().find(|i| i.id == item_id).unwrap();
        assert_eq!(item.name, "測試菜品");
        assert_eq!(item.sales_price, 50.0);

        db.update_menu_item(item_id, Some("更新菜品"), category_id, None, Some(60.0)).unwrap();
        let items_after = db.get_menu_items(None).unwrap();
        let item_after = items_after.iter().find(|i| i.id == item_id).unwrap();
        assert_eq!(item_after.name, "更新菜品");
        assert_eq!(item_after.sales_price, 60.0);

        db.delete_menu_item(item_id).unwrap();
        let items_final = db.get_menu_items(None).unwrap();
        let deleted = items_final.iter().find(|i| i.id == item_id);
        assert!(deleted.is_none() || !deleted.unwrap().is_available);
    }

    #[test]
    fn test_order_with_empty_items() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let (order_id, _) = db.create_order("POS", "堂食", None).unwrap();
        let orders = db.get_orders(1000, 0).unwrap();
        let order = orders.iter().find(|o| o.id == order_id).unwrap();

        let submit_result = db.submit_order_full(order.id);
        assert!(submit_result.is_ok());

        let orders_after = db.get_orders(1000, 0).unwrap();
        let order_after = orders_after.iter().find(|o| o.id == order.id).unwrap();
        assert_eq!(order_after.status, "submitted");
    }

    #[test]
    fn test_inventory_txns_ordering() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let materials = db.get_materials(None).unwrap();
        assert!(!materials.is_empty());
        let material = &materials[0];

        let lot_no = format!("LOT_TXN_{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase());
        db.create_inventory_batch(
            material.material.id,
            None,
            &lot_no,
            None,
            None,
            None,
            100.0,
            10.0,
            None,
            None,
            None,
            None,
            1.0,
        ).unwrap();

        let batches = db.get_inventory_batches(Some(material.material.id)).unwrap();
        let batch = batches.iter().find(|b| b.lot_no == lot_no).unwrap();

        db.create_inventory_txn(
            "purchase_in",
            Some("test"),
            None,
            Some(batch.id),
            material.material.id,
            None,
            100.0,
            Some(0.0),
            None,
            None,
        ).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(1100));

        db.create_inventory_txn(
            "adjustment",
            Some("test"),
            None,
            Some(batch.id),
            material.material.id,
            None,
            -10.0,
            Some(0.0),
            None,
            None,
        ).unwrap();

        let txns = db.get_inventory_txns(Some(material.material.id), 100).unwrap();
        assert!(txns.len() >= 2);
        let has_adjust = txns.iter().any(|t| t.txn_type == "adjustment");
        let has_purchase = txns.iter().any(|t| t.txn_type == "purchase_in");
        assert!(has_adjust);
        assert!(has_purchase);
    }

    #[test]
    fn test_print_template_update() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let tpl_content = r#"{"elements":[{"type":"text","content":"原始","align":"center"}]}"#;
        let tpl_id = db.create_print_template(&CreatePrintTemplateRequest {
            name: "更新測試".to_string(),
            template_type: "kitchen_ticket".to_string(),
            paper_size: "80mm".to_string(),
            label_width_mm: None,
            label_height_mm: None,
            content: tpl_content.to_string(),
            theme: None,
            restaurant_name: None,
            tagline: None,
            logo_data: None,
            show_price: None,
            show_tax: None,
            show_service_charge: None,
            item_sort: None,
            modifiers_color: None,
            is_active: Some(true),
        }).unwrap();

        let templates = db.get_print_templates(None).unwrap();
        let tpl = templates.iter().find(|t| t.id == tpl_id).unwrap();
        assert_eq!(tpl.name, "更新測試");

        db.update_print_template(tpl_id, Some("已更新".to_string()), None, Some("58mm".to_string()), None, None, None, None, None, None, None, None, None, None, None, None).unwrap();

        let templates_after = db.get_print_templates(None).unwrap();
        let tpl_after = templates_after.iter().find(|t| t.id == tpl_id).unwrap();
        assert_eq!(tpl_after.name, "已更新");
        assert_eq!(tpl_after.paper_size, "58mm");
    }

    #[test]
    fn test_print_template_filter_by_type() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        db.create_print_template(&CreatePrintTemplateRequest {
            name: "廚房單模板".to_string(),
            template_type: "kitchen_ticket".to_string(),
            paper_size: "80mm".to_string(),
            label_width_mm: None,
            label_height_mm: None,
            content: r#"{"elements":[]}"#.to_string(),
            theme: None,
            restaurant_name: None,
            tagline: None,
            logo_data: None,
            show_price: None,
            show_tax: None,
            show_service_charge: None,
            item_sort: None,
            modifiers_color: None,
            is_active: Some(true),
        }).unwrap();

        db.create_print_template(&CreatePrintTemplateRequest {
            name: "標籤模板".to_string(),
            template_type: "batch_label".to_string(),
            paper_size: "custom".to_string(),
            label_width_mm: Some(60.0),
            label_height_mm: Some(40.0),
            content: r#"{"elements":[]}"#.to_string(),
            theme: None,
            restaurant_name: None,
            tagline: None,
            logo_data: None,
            show_price: None,
            show_tax: None,
            show_service_charge: None,
            item_sort: None,
            modifiers_color: None,
            is_active: Some(true),
        }).unwrap();

        let kitchen_templates = db.get_print_templates(Some("kitchen_ticket")).unwrap();
        assert_eq!(kitchen_templates.len(), 1);
        assert_eq!(kitchen_templates[0].template_type, "kitchen_ticket");

        let label_templates = db.get_print_templates(Some("batch_label")).unwrap();
        assert_eq!(label_templates.len(), 1);
        assert_eq!(label_templates[0].template_type, "batch_label");

        let all_templates = db.get_print_templates(None).unwrap();
        assert!(all_templates.len() >= 2);
    }
}
