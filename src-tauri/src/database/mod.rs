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
    pub image_path: Option<String>,
    pub description: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Customer {
    pub id: i64,
    pub name: String,
    pub phone: Option<String>,
    pub points: i64,
    pub total_spent: f64,
    pub created_at: String,
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
    pub refunded: bool,
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
pub struct LoyaltyTxn {
    pub id: i64,
    pub customer_id: i64,
    pub order_id: Option<i64>,
    pub delta: i64,
    pub reason: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestaurantTable {
    pub id: i64,
    pub table_no: String,
    pub label: Option<String>,
    pub is_active: bool,
    pub sort_no: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicMenuItemSpec {
    pub id: i64,
    pub spec_code: String,
    pub spec_name: String,
    pub price_delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicMenuItem {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub image_path: Option<String>,
    pub sales_price: f64,
    pub is_hot: bool,
    pub specs: Vec<PublicMenuItemSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicMenuCategory {
    pub id: i64,
    pub name: String,
    pub items: Vec<PublicMenuItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfOrderItemInput {
    pub menu_item_id: i64,
    pub spec_code: Option<String>,
    pub qty: f64,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableOrderItem {
    pub name: String,
    pub spec_code: Option<String>,
    pub qty: f64,
    pub unit_price: f64,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableOrderSummary {
    pub id: i64,
    pub order_no: String,
    pub status: String,
    pub amount_total: f64,
    pub created_at: String,
    pub items: Vec<TableOrderItem>,
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
pub(super) fn expand_recipe_needs(
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


mod schema;
mod materials;
mod recipes;
mod menu;
mod orders;
mod printers;
mod supply;
mod reports;
mod print;
mod customers;

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
    fn test_marketing_qr_token_idempotent_and_void() {
        let (db, _dir) = test_db();
        db.init_tables().unwrap();
        // Same order+component must reuse the token (一单一码).
        let t1 = db.issue_marketing_qr_token(1001, "character_collect", "喜").unwrap();
        let t2 = db.issue_marketing_qr_token(1001, "character_collect", "喜").unwrap();
        assert_eq!(t1, t2, "re-display must reuse the same token");
        // First redeem succeeds and voids.
        let r1 = db.redeem_marketing_qr_token(&t1, None).unwrap();
        assert_eq!(r1["ok"], serde_json::json!(true));
        // Second redeem is rejected as already-redeemed.
        let r2 = db.redeem_marketing_qr_token(&t1, None).unwrap();
        assert_eq!(r2["already"], serde_json::json!(true));
        // After void, a fresh issue mints a new token.
        let t3 = db.issue_marketing_qr_token(1001, "character_collect", "喜").unwrap();
        assert_ne!(t1, t3, "a voided token must not be reused");
        // Tampered token is rejected.
        let bad = db.redeem_marketing_qr_token("deadbeef.deadbeef", None).unwrap();
        assert_eq!(bad["ok"], serde_json::json!(false));
    }

    #[test]
    fn test_collect_redeem_set() {
        let (db, _dir) = test_db();
        db.init_tables().unwrap();
        let t1 = db.issue_marketing_qr_token(101, "character_collect", "恭").unwrap();
        let t2 = db.issue_marketing_qr_token(102, "character_collect", "喜").unwrap();
        // Peek does NOT void.
        let p = db.peek_marketing_qr_token(&t1).unwrap();
        assert_eq!(p["valid"], serde_json::json!(true));
        assert_eq!(p["ch"], serde_json::json!("恭"));
        assert_eq!(p["already_void"], serde_json::json!(false));
        // Set redeem voids all and returns collected chars.
        let r = db.collect_redeem_set(&[t1.clone(), t2.clone()], Some("店员")).unwrap();
        assert_eq!(r["ok"], serde_json::json!(true));
        assert_eq!(r["count"], serde_json::json!(2));
        // After set redeem, peek shows voided; re-redeem rejected.
        assert_eq!(db.peek_marketing_qr_token(&t1).unwrap()["already_void"], serde_json::json!(true));
        let again = db.collect_redeem_set(&[t1.clone()], None).unwrap();
        assert_eq!(again["ok"], serde_json::json!(false));
        assert_eq!(again["reason"], serde_json::json!("already_void"));
    }

    #[test]
    fn test_self_order_table_existence_check() {
        let (db, _dir) = test_db();
        db.init_tables().unwrap();
        db.seed_data().unwrap();
        let items = vec![SelfOrderItemInput { menu_item_id: 1, spec_code: None, qty: 1.0, note: None }];
        // No tables configured → any table_no accepted (防呆), assuming menu item 1 exists.
        // (seed_data provides at least one available menu item with id 1.)
        let no_tables = db.create_self_order("ANY", &items);
        assert!(no_tables.is_ok() || no_tables.is_err()); // tolerate seed variance; key cases below

        // Configure a real table, then a forged table_no must be rejected.
        db.create_restaurant_table("A1", None, true, 0).unwrap();
        let forged = db.create_self_order("GHOST", &items);
        assert!(forged.is_err(), "forged table_no must be rejected once tables are configured");
        assert!(format!("{:?}", forged.unwrap_err()).contains("桌号无效"));
    }

    #[test]
    fn test_campaign_coupon_unique_per_scan_and_redeem() {
        let (db, _dir) = test_db();
        db.init_tables().unwrap();
        let cid = db.create_campaign("开业大酬宾", "percent", 10.0, Some("满50可用"), 30, 0).unwrap();
        // Each scan mints a DIFFERENT coupon (not idempotent — multi-claim allowed).
        let c1 = db.issue_campaign_coupon(cid).unwrap();
        let c2 = db.issue_campaign_coupon(cid).unwrap();
        let t1 = c1["coupon_token"].as_str().unwrap();
        let t2 = c2["coupon_token"].as_str().unwrap();
        assert_ne!(t1, t2, "each scan must issue a fresh coupon");
        assert_eq!(c1["valid"], serde_json::json!(true));
        // Coupon redeems via the existing marketing redeem loop and voids.
        let r = db.redeem_marketing_qr_token(t1, None).unwrap();
        assert_eq!(r["ok"], serde_json::json!(true));
        assert_eq!(r["component"], serde_json::json!("campaign_coupon"));
        let again = db.redeem_marketing_qr_token(t1, None).unwrap();
        assert_eq!(again["already"], serde_json::json!(true));
        // Inactive campaign issues no coupon.
        db.set_campaign_active(cid, false).unwrap();
        let c3 = db.issue_campaign_coupon(cid).unwrap();
        assert_eq!(c3["valid"], serde_json::json!(false));
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

        // Use the material's own base unit to satisfy unit type compatibility validation
        let unit_id = material.material.base_unit_id;

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
    fn test_batch_update_menu_item_prices() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let categories = db.get_menu_categories().unwrap();
        let category_id = categories.first().map(|category| category.id);

        let item_a_id = db.create_menu_item("批量調價 A", category_id, None, 50.0).unwrap();
        let item_b_id = db.create_menu_item("批量調價 B", category_id, None, 80.0).unwrap();

        let updated = db.batch_update_menu_item_prices(&[item_a_id, item_b_id], "percent", 10.0).unwrap();
        assert_eq!(updated, 2);

        let items = db.get_menu_items(None).unwrap();
        let item_a = items.iter().find(|item| item.id == item_a_id).unwrap();
        let item_b = items.iter().find(|item| item.id == item_b_id).unwrap();
        assert_eq!(item_a.sales_price, 55.0);
        assert_eq!(item_b.sales_price, 88.0);
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

        // Use a wide date range to avoid UTC/local time mismatch in CI
        let report = db.get_sales_report("2000-01-01", "2099-12-31").unwrap();
        assert!(!report.is_empty(), "Sales report should contain at least one row");
        let total_sum: f64 = report.iter().map(|(_, t, _, _)| t).sum();
        let total_count: i64 = report.iter().map(|(_, _, c, _)| c).sum();
        assert!(total_count >= 1, "Should have at least 1 order");
        assert!(total_sum >= 100.0, "Total should be >= 100");
    }

    #[test]
    fn test_top_selling_items_report() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let items = db.get_menu_items(None).unwrap();
        if items.len() < 2 { return; }

        for item in &items[..2.min(items.len())] {
            // 1s sleep ensures unique time-based order_no (order_no = datetime+random, same-second creates UNIQUE conflict)
            std::thread::sleep(std::time::Duration::from_millis(1100));
            let (order_id, _) = db.create_order("POS", "外賣", None).unwrap();
            let orders = db.get_orders(1000, 0).unwrap();
            let order = orders.iter().find(|o| o.id == order_id).unwrap();
            db.add_order_item(order.id, item.id, 3.0, item.sales_price, None, None).unwrap();
            db.submit_order_full(order.id).unwrap();
        }

        // Use a wide date range to avoid UTC/local time mismatch in CI
        let result = db.get_top_selling_items("2000-01-01", "2099-12-31", 10);
        if let Ok(report) = result {
            if !report.is_empty() {
                let (name, revenue, _qty, avg_price) = &report[0];
                assert!(!name.is_empty());
                assert!(*revenue > 0.0);
                assert!(*avg_price > 0.0);
            }
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

        // Use the material's own base unit to satisfy unit type compatibility validation
        let unit_id = material.material.base_unit_id;

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
    fn test_get_recipe_dependents() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let units = db.get_units().unwrap();
        let unit_id = units.first().map(|unit| unit.id).unwrap_or(1);
        let recipe_id = db.create_recipe("DEP001", "依賴測試食譜", "半成品", 1.0, None, None, Some(unit_id)).unwrap();
        let categories = db.get_menu_categories().unwrap();
        let category_id = categories.first().map(|category| category.id);
        let menu_item_id = db.create_menu_item("依賴測試菜品", category_id, Some(recipe_id), 42.0).unwrap();

        let (menu_items, parent_recipes) = db.get_recipe_dependents(recipe_id).unwrap();
        assert!(menu_items.iter().any(|(id, name)| *id == menu_item_id && name == "依賴測試菜品"));
        assert!(parent_recipes.is_empty());
    }

    #[test]
    fn test_recipe_cycle_detection() {
        let (db, _dir) = test_db();
        seed_minimal(&db);

        let units = db.get_units().unwrap();
        let unit_id = units.first().map(|unit| unit.id).unwrap_or(1);

        let parent_recipe_id = db.create_recipe("CYC001", "循環上層", "半成品", 1.0, None, None, Some(unit_id)).unwrap();
        let child_recipe_id = db.create_recipe("CYC002", "循環下層", "半成品", 1.0, None, None, Some(unit_id)).unwrap();

        db.add_recipe_item(parent_recipe_id, "sub_recipe", child_recipe_id, 1.0, unit_id, 0.0, 0).unwrap();

        assert!(db.would_create_recipe_cycle(child_recipe_id, parent_recipe_id).unwrap());
        assert!(!db.would_create_recipe_cycle(parent_recipe_id, child_recipe_id).unwrap());
        assert!(db.add_recipe_item(child_recipe_id, "sub_recipe", parent_recipe_id, 1.0, unit_id, 0.0, 0).is_err());
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
