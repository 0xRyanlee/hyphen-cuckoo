use tauri::{State, Manager};
use std::sync::Arc;
use crate::database::{Database, MaterialWithTags, Unit, MaterialCategory, Tag, MaterialState, Supplier, Expense, SupplierProduct, Recipe, RecipeWithItems, RecipeCostResult, RecipeType, MenuItem, MenuCategory, Order, OrderItem, OrderItemModifier, KitchenStation, KitchenTicket, InventoryBatch, InventorySummary, AttributeTemplate, EntityAttribute, InventoryTxn, MenuItemSpec, PrinterConfig, PrintTask, PurchaseOrder, PurchaseOrderWithItems, ProductionOrder, ProductionOrderWithItems, Stocktake, StocktakeWithItems, Notification, Customer, LoyaltyTxn, Coupon, RecipeComponentType, OrderComponentType, RestaurantTable, PublicMenuCategory, SelfOrderItemInput, TableOrderSummary, ComboWithComponents};
use crate::printer::{self, EscPosBuilder, LanPrinter, scan_lan_printers as LAN_SCAN};
use serde::{Deserialize, Serialize};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::OsRng;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub struct AppState {
    pub db: Arc<Database>,
    pub db_path: std::path::PathBuf,
    pub sync_server: std::sync::Mutex<Option<crate::sync_server::SyncServerHandle>>,
    pub web_server: std::sync::Mutex<Option<crate::web_server::WebServerHandle>>,
    pub role_auth: std::sync::Mutex<RoleAuthStore>,
    pub role_auth_path: std::path::PathBuf,
}

const SYNC_PROTOCOL_VERSION: &str = "2";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Owner,
    Cashier,
    Chef,
    Warehouse,
}

impl UserRole {
    fn as_str(&self) -> &'static str {
        match self {
            UserRole::Owner => "owner",
            UserRole::Cashier => "cashier",
            UserRole::Chef => "chef",
            UserRole::Warehouse => "warehouse",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            UserRole::Owner => "老板",
            UserRole::Cashier => "收银",
            UserRole::Chef => "厨师",
            UserRole::Warehouse => "仓库",
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_lowercase().as_str() {
            "owner" => Ok(UserRole::Owner),
            "cashier" => Ok(UserRole::Cashier),
            "chef" => Ok(UserRole::Chef),
            "warehouse" => Ok(UserRole::Warehouse),
            _ => Err("未知角色".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAuthStore {
    pub current_role: UserRole,
    #[serde(default)]
    pub pin_hashes: HashMap<String, String>,
    /// C2: when true, self-orders MUST carry a valid signed token (legacy static
    /// /table/ QR codes are rejected — used to harden after the grace period).
    #[serde(default)]
    pub require_token: bool,
}

impl Default for RoleAuthStore {
    fn default() -> Self {
        Self {
            current_role: UserRole::Owner,
            pin_hashes: HashMap::new(),
            require_token: false,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RolePinStatus {
    pub role: String,
    pub has_pin: bool,
}

fn sha256_hex(pin: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(pin.as_bytes());
    hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

fn hash_pin(pin: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(pin.as_bytes(), &salt)
        .expect("argon2 hash failed")
        .to_string()
}

/// Verify a PIN against a stored hash.
/// Supports both the new argon2id format (`$argon2id$...`) and
/// the legacy SHA-256 hex format (for hashes stored before the upgrade).
/// On next PIN change, the stored hash is automatically re-hashed with argon2.
pub fn verify_pin(pin: &str, stored_hash: &str) -> bool {
    if stored_hash.starts_with("$argon2") {
        PasswordHash::new(stored_hash)
            .map(|h| Argon2::default().verify_password(pin.as_bytes(), &h).is_ok())
            .unwrap_or(false)
    } else {
        sha256_hex(pin) == stored_hash
    }
}

/// True if `pin` matches any configured role PIN. 防呆:if no PIN is set at all
/// (small shop that never configured passwords), redemption is allowed PIN-free.
pub fn verify_any_pin(store: &RoleAuthStore, pin: &str) -> bool {
    if store.pin_hashes.is_empty() {
        return true;
    }
    store.pin_hashes.values().any(|h| verify_pin(pin, h))
}

pub fn load_role_auth_store(path: &std::path::Path) -> RoleAuthStore {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<RoleAuthStore>(&raw).ok())
        .unwrap_or_default()
}

fn save_role_auth_store(path: &std::path::Path, store: &RoleAuthStore) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建权限目录失败: {}", e))?;
    }
    let json = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| format!("保存权限配置失败: {}", e))
}

fn current_role(state: &State<AppState>) -> Result<UserRole, String> {
    Ok(state.role_auth.lock().map_err(|_| "角色权限状态不可用".to_string())?.current_role)
}

fn require_roles(state: &State<AppState>, allowed: &[UserRole], action: &str) -> Result<UserRole, String> {
    let role = current_role(state)?;
    if allowed.iter().any(|allowed_role| *allowed_role == role) {
        Ok(role)
    } else {
        Err(format!("当前角色「{}」无权执行：{}", role.label(), action))
    }
}

fn db_err(e: rusqlite::Error) -> String {
    let s = e.to_string();
    if s.contains("UNIQUE constraint failed: materials.code") { return "原料编号已存在，请更换编号后重试".to_string(); }
    if s.contains("UNIQUE constraint failed: materials.name") { return "原料名称已存在".to_string(); }
    if s.contains("UNIQUE constraint failed: recipes.code") { return "配方编号已存在，请更换编号后重试".to_string(); }
    if s.contains("UNIQUE constraint failed: menu_items.code") { return "菜品编号已存在".to_string(); }
    if s.contains("UNIQUE constraint failed: suppliers.name") { return "供应商名称已存在".to_string(); }
    if s.contains("UNIQUE constraint failed: tags.code") { return "标签编号已存在".to_string(); }
    if s.contains("UNIQUE constraint failed: material_categories.code") { return "分类编号已存在".to_string(); }
    if s.contains("UNIQUE constraint failed") { return "记录已存在，请修改后重试".to_string(); }
    if s.contains("FOREIGN KEY constraint failed") { return "关联数据不存在或已被删除".to_string(); }
    if s.contains("NOT NULL constraint failed") { return "必填字段不能为空".to_string(); }
    s
}

/// Payload prepared before spawning the background print thread.
enum PrintPayload {
    Feie { user: String, ukey: String, sn: String, content: String },
    Lan  { ip: String, port: i32, data: Vec<u8> },
}

/// Dispatches a print job in a background thread; emits `print-result` on completion.
/// The DB task record must already exist with status "pending".
fn dispatch_print(app: tauri::AppHandle, task_id: i64, payload: PrintPayload) {
    use tauri::Emitter;
    std::thread::spawn(move || {
        let outcome = match payload {
            PrintPayload::Feie { ref user, ref ukey, ref sn, ref content } => {
                printer::feie_print(user, ukey, sn, content).map(|_| ())
            }
            PrintPayload::Lan { ref ip, port, ref data } => {
                printer::lan_print(ip, port, data).map(|_| ())
            }
        };
        let db = &app.state::<AppState>().db;
        match outcome {
            Ok(_) => {
                let _ = db.update_print_task_status(task_id, "printed", None);
                let _ = app.emit("print-result", serde_json::json!({ "taskId": task_id, "success": true }));
            }
            Err(e) => {
                let friendly = print_err(e.clone());
                let _ = db.update_print_task_status(task_id, "failed", Some(&e));
                let _ = app.emit("print-result", serde_json::json!({ "taskId": task_id, "success": false, "error": friendly }));
            }
        }
    });
}

fn print_err(raw: String) -> String {
    let s = raw.to_lowercase();
    if s.contains("飛鵝") || s.contains("feie") || s.contains("api") {
        if s.contains("不在线") || s.contains("offline") { return "打印机不在线，请检查打印机电源和网络".to_string(); }
        if s.contains("sn") && (s.contains("错") || s.contains("invalid") || s.contains("exist")) { return "打印机 SN 无效或未注册，请重新绑定".to_string(); }
        if s.contains("user") || s.contains("ukey") || s.contains("sig") { return "飞鹅账号凭证错误（USER/UKEY），请检查打印设置".to_string(); }
        if s.contains("timeout") || s.contains("超时") || s.contains("timed out") { return "连接飞鹅服务器超时，请检查网络".to_string(); }
        if s.contains("print") && s.contains("fail") { return "打印任务发送失败，打印机可能正在处理其他任务".to_string(); }
    }
    if s.contains("connection refused") || s.contains("connection reset") { return "无法连接到打印机，请检查 IP 地址和端口".to_string(); }
    if s.contains("timed out") || s.contains("timeout") { return "连接打印机超时，请确认打印机已开机且在同一局域网".to_string(); }
    if s.contains("未配置") || s.contains("not configured") { return "打印机未配置，请在打印设置中添加打印机".to_string(); }
    if s.contains("no printer") || s.contains("找不到") { return "未找到默认打印机，请在打印设置中设置默认打印机".to_string(); }
    raw
}

#[tauri::command]
pub fn get_current_role(state: State<AppState>) -> Result<String, String> {
    Ok(current_role(&state)?.as_str().to_string())
}

#[tauri::command]
pub fn get_role_pin_statuses(state: State<AppState>) -> Result<Vec<RolePinStatus>, String> {
    let auth = state.role_auth.lock().map_err(|_| "角色权限状态不可用".to_string())?;
    Ok([
        UserRole::Owner,
        UserRole::Cashier,
        UserRole::Chef,
        UserRole::Warehouse,
    ]
    .into_iter()
    .map(|role| RolePinStatus {
        role: role.as_str().to_string(),
        has_pin: auth.pin_hashes.contains_key(role.as_str()),
    })
    .collect())
}

#[tauri::command]
pub fn set_role_pin(state: State<AppState>, role: String, pin: Option<String>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "设置角色 PIN")?;
    let role = role.parse::<UserRole>()?;
    let mut auth = state.role_auth.lock().map_err(|_| "角色权限状态不可用".to_string())?;
    match pin.map(|value| value.trim().to_string()).filter(|value| !value.is_empty()) {
        Some(value) => {
            if value.len() < 4 {
                return Err("PIN 至少 4 位".to_string());
            }
            auth.pin_hashes.insert(role.as_str().to_string(), hash_pin(&value));
        }
        None => {
            auth.pin_hashes.remove(role.as_str());
        }
    }
    save_role_auth_store(&state.role_auth_path, &auth)
}

#[tauri::command]
pub fn switch_role(state: State<AppState>, role: String, pin: Option<String>) -> Result<String, String> {
    let target = role.parse::<UserRole>()?;
    let mut auth = state.role_auth.lock().map_err(|_| "角色权限状态不可用".to_string())?;
    if let Some(expected_hash) = auth.pin_hashes.get(target.as_str()) {
        let provided = pin.unwrap_or_default();
        if !verify_pin(provided.trim(), expected_hash) {
            return Err("PIN 错误".to_string());
        }
    }
    auth.current_role = target;
    save_role_auth_store(&state.role_auth_path, &auth)?;
    Ok(target.as_str().to_string())
}

// ==================== 請求體 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMaterialRequest {
    pub code: String,
    pub name: String,
    pub category_id: Option<i64>,
    pub base_unit_id: i64,
    pub shelf_life_days: Option<i32>,
    pub tag_ids: Option<Vec<i64>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCategoryRequest {
    pub code: String,
    pub name: String,
    pub sort_no: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTagRequest {
    pub code: String,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMaterialStateRequest {
    pub material_id: i64,
    pub state_code: String,
    pub state_name: String,
    pub unit_id: Option<i64>,
    pub yield_rate: Option<f64>,
    pub cost_multiplier: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSupplierRequest {
    pub name: String,
    pub phone: Option<String>,
    pub contact_person: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetAttributeRequest {
    pub entity_type: String,
    pub entity_id: i64,
    pub attr_code: String,
    pub value: Option<f64>,
    pub value_text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRecipeRequest {
    pub code: String,
    pub name: String,
    pub recipe_type: String,
    pub output_qty: f64,
    pub output_material_id: Option<i64>,
    pub output_state_id: Option<i64>,
    pub output_unit_id: Option<i64>,
    pub items: Option<Vec<RecipeItemRequest>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRecipeTypeRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_no: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecipeItemRequest {
    pub item_type: String,
    pub ref_id: i64,
    pub qty: f64,
    pub unit_id: i64,
    pub wastage_rate: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMenuItemRequest {
    pub name: String,
    pub category_id: Option<i64>,
    pub recipe_id: Option<i64>,
    pub sales_price: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMenuItemSpecRequest {
    pub menu_item_id: i64,
    pub spec_code: String,
    pub spec_name: String,
    pub price_delta: f64,
    pub qty_multiplier: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub source: String,
    pub dine_type: String,
    pub table_no: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInventoryTxnRequest {
    pub txn_type: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<i64>,
    pub lot_id: Option<i64>,
    pub material_id: i64,
    pub state_id: Option<i64>,
    pub qty_delta: f64,
    pub cost_delta: Option<f64>,
    pub operator: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddOrderItemRequest {
    pub order_id: i64,
    pub menu_item_id: i64,
    pub qty: f64,
    pub unit_price: f64,
    pub spec_code: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderWithItems {
    #[serde(flatten)]
    pub order: Order,
    pub items: Vec<OrderItem>,
}

// ==================== 單位 API ====================

#[tauri::command]
pub fn get_units(state: State<AppState>) -> Result<Vec<Unit>, String> {
    state.db.get_units().map_err(|e| e.to_string())
}

// ==================== 材料分類 API ====================

#[tauri::command]
pub fn get_material_categories(state: State<AppState>) -> Result<Vec<MaterialCategory>, String> {
    state.db.get_material_categories().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_material_category(state: State<AppState>, req: CreateCategoryRequest) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "创建材料分类")?;
    let sort_no = req.sort_no.unwrap_or(0);
    state.db.create_material_category(&req.code, &req.name, sort_no).map_err(db_err)
}

// ==================== 標籤 API ====================

#[tauri::command]
pub fn get_tags(state: State<AppState>) -> Result<Vec<Tag>, String> {
    state.db.get_tags().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_tag(state: State<AppState>, req: CreateTagRequest) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "创建标签")?;
    state.db.create_tag(&req.code, &req.name, req.color.as_deref()).map_err(db_err)
}

// ==================== 材料 API ====================

#[tauri::command]
pub fn get_materials(state: State<AppState>, category_id: Option<i64>) -> Result<Vec<MaterialWithTags>, String> {
    state.db.get_materials(category_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_material(state: State<AppState>, req: CreateMaterialRequest) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "创建材料")?;
    let id = state.db.create_material(
        &req.code,
        &req.name,
        req.category_id,
        req.base_unit_id,
        req.shelf_life_days,
    ).map_err(db_err)?;
    
    if let Some(tag_ids) = req.tag_ids {
        state.db.add_material_tags(id, &tag_ids).map_err(|e| e.to_string())?;
    }
    
    Ok(id)
}

#[tauri::command]
pub fn add_material_tags(state: State<AppState>, material_id: i64, tag_ids: Vec<i64>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "更新材料标签")?;
    state.db.add_material_tags(material_id, &tag_ids).map_err(|e| e.to_string())
}

// ==================== 材料狀態 API ====================

#[tauri::command]
pub fn get_material_states(state: State<AppState>, material_id: i64) -> Result<Vec<MaterialState>, String> {
    state.db.get_material_states(material_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_material_state(state: State<AppState>, req: CreateMaterialStateRequest) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "创建材料状态")?;
    let yield_rate = req.yield_rate.unwrap_or(1.0);
    let cost_multiplier = req.cost_multiplier.unwrap_or(1.0);
    state.db.create_material_state(
        req.material_id,
        &req.state_code,
        &req.state_name,
        req.unit_id,
        yield_rate,
        cost_multiplier,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_material_states(state: State<AppState>) -> Result<Vec<MaterialState>, String> {
    state.db.get_all_material_states().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_material_state(state: State<AppState>, id: i64, state_code: Option<String>, state_name: Option<String>, unit_id: Option<i64>, yield_rate: Option<f64>, cost_multiplier: Option<f64>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "更新材料状态")?;
    state.db.update_material_state(id, state_code.as_deref(), state_name.as_deref(), unit_id, yield_rate, cost_multiplier).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_material_state(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "删除材料状态")?;
    state.db.delete_material_state(id).map_err(|e| e.to_string())
}

// ==================== 供應商 API ====================

#[tauri::command]
pub fn get_suppliers(state: State<AppState>) -> Result<Vec<Supplier>, String> {
    state.db.get_suppliers().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_supplier(state: State<AppState>, req: CreateSupplierRequest) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "创建供应商")?;
    state.db.create_supplier(&req.name, req.phone.as_deref(), req.contact_person.as_deref()).map_err(db_err)
}

// ==================== 日常支出 API ====================

#[tauri::command]
pub fn get_expenses(state: State<AppState>, expense_type: Option<String>, start_date: Option<String>, end_date: Option<String>) -> Result<Vec<Expense>, String> {
    state.db.get_expenses(expense_type.as_deref(), start_date.as_deref(), end_date.as_deref()).map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct CreateExpenseRequest {
    pub expense_type: String,
    pub amount: f64,
    pub expense_date: String,
    pub note: Option<String>,
    pub operator: Option<String>,
}

#[tauri::command]
pub fn create_expense(state: State<AppState>, req: CreateExpenseRequest) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "创建支出")?;
    state.db.create_expense(&req.expense_type, req.amount, &req.expense_date, req.note.as_deref(), req.operator.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_expense(state: State<AppState>, id: i64, expense_type: Option<String>, amount: Option<f64>, expense_date: Option<String>, note: Option<String>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "更新支出")?;
    state.db.update_expense(id, expense_type.as_deref(), amount, expense_date.as_deref(), note.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_expense(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "删除支出")?;
    state.db.delete_expense(id).map_err(|e| e.to_string())
}

// ==================== 供應商商品 API ====================

#[tauri::command]
pub fn get_supplier_products(state: State<AppState>, channel: Option<String>) -> Result<Vec<SupplierProduct>, String> {
    state.db.get_supplier_products(channel.as_deref()).map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct CreateSupplierProductRequest {
    pub product_name: String,
    pub supplier_name: String,
    pub channel: String,
}

#[tauri::command]
pub fn create_supplier_product(state: State<AppState>, req: CreateSupplierProductRequest) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "创建供应商商品")?;
    state.db.create_supplier_product(&req.product_name, &req.supplier_name, &req.channel).map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct UpdateSupplierProductRequest {
    pub id: i64,
    pub product_name: String,
    pub supplier_name: String,
    pub channel: String,
}

#[tauri::command]
pub fn update_supplier_product(state: State<AppState>, req: UpdateSupplierProductRequest) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "更新供应商商品")?;
    state.db.update_supplier_product(req.id, &req.product_name, &req.supplier_name, &req.channel).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_supplier_product(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "删除供应商商品")?;
    state.db.delete_supplier_product(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_order_cost(state: State<AppState>, order_id: i64) -> Result<f64, String> {
    state.db.get_order_cost(order_id).map_err(|e| e.to_string())
}

// ==================== 屬性模板 API ====================

#[tauri::command]
pub fn get_attribute_templates(state: State<AppState>, entity_type: Option<String>, category: Option<String>) -> Result<Vec<AttributeTemplate>, String> {
    state.db.get_attribute_templates(entity_type.as_deref(), category.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_attribute_template(state: State<AppState>, entity_type: String, category: Option<String>, attr_code: String, attr_name: String, data_type: String, unit: Option<String>, default_value: Option<f64>, formula: Option<String>) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner], "创建属性模板")?;
    state.db.create_attribute_template(
        &entity_type,
        category.as_deref(),
        &attr_code,
        &attr_name,
        &data_type,
        unit.as_deref(),
        default_value,
        formula.as_deref(),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_attribute_template(state: State<AppState>, id: i64, entity_type: String, category: Option<String>, attr_code: String, attr_name: String, data_type: String, unit: Option<String>, default_value: Option<f64>, formula: Option<String>, is_active: bool) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "更新属性模板")?;
    state.db.update_attribute_template(
        id,
        &entity_type,
        category.as_deref(),
        &attr_code,
        &attr_name,
        &data_type,
        unit.as_deref(),
        default_value,
        formula.as_deref(),
        is_active,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_attribute_template(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "删除属性模板")?;
    state.db.delete_attribute_template(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_entity_attribute(state: State<AppState>, req: SetAttributeRequest) -> Result<(), String> {
    state.db.set_entity_attribute(
        &req.entity_type,
        req.entity_id,
        &req.attr_code,
        req.value,
        req.value_text.as_deref(),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_entity_attributes(state: State<AppState>, entity_type: String, entity_id: i64) -> Result<Vec<EntityAttribute>, String> {
    state.db.get_entity_attributes(&entity_type, entity_id).map_err(|e| e.to_string())
}

// ==================== 配方 API ====================

#[tauri::command]
pub fn get_recipes(state: State<AppState>, recipe_type: Option<String>) -> Result<Vec<Recipe>, String> {
    state.db.get_recipes(recipe_type.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_recipe_types(state: State<AppState>) -> Result<Vec<RecipeType>, String> {
    state.db.get_recipe_types().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_recipe_component_types(state: State<AppState>, category: Option<String>) -> Result<Vec<RecipeComponentType>, String> {
    state.db.get_recipe_component_types(category.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_order_component_types(state: State<AppState>, is_packaging: Option<bool>) -> Result<Vec<OrderComponentType>, String> {
    state.db.get_order_component_types(is_packaging).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_recipe_type(state: State<AppState>, req: CreateRecipeTypeRequest) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner], "创建配方类型")?;
    state.db.create_recipe_type(&req.code, &req.name, req.description.as_deref(), req.sort_no.unwrap_or(0)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_recipe_with_items(state: State<AppState>, recipe_id: i64) -> Result<RecipeWithItems, String> {
    state.db.get_recipe_with_items(recipe_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn generate_recipe_code(state: State<AppState>) -> Result<String, String> {
    state.db.generate_recipe_code().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn seed_sample_recipes(state: State<AppState>) -> Result<String, String> {
    require_roles(&state, &[UserRole::Owner], "创建示例配方")?;
    state.db.seed_sample_recipes().map_err(|e| e.to_string())?;
    Ok("示例配方已创建".to_string())
}

#[tauri::command]
pub fn create_recipe(state: State<AppState>, req: CreateRecipeRequest) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner], "创建配方")?;
    let id = state.db.create_recipe(
        &req.code,
        &req.name,
        &req.recipe_type,
        req.output_qty,
        req.output_material_id,
        req.output_state_id,
        req.output_unit_id,
    ).map_err(db_err)?;
    
    if let Some(items) = req.items {
        for item in items {
            let wastage_rate = item.wastage_rate.unwrap_or(0.0);
            state.db.add_recipe_item(id, &item.item_type, item.ref_id, item.qty, item.unit_id, wastage_rate, 0)
                .map_err(|e| e.to_string())?;
        }
    }
    
    Ok(id)
}

#[tauri::command]
pub fn add_recipe_item(state: State<AppState>, recipe_id: i64, req: RecipeItemRequest) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner], "添加配方项")?;
    let wastage_rate = req.wastage_rate.unwrap_or(0.0);
    state.db.add_recipe_item(recipe_id, &req.item_type, req.ref_id, req.qty, req.unit_id, wastage_rate, 0)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn calculate_recipe_cost(state: State<AppState>, recipe_id: i64) -> Result<RecipeCostResult, String> {
    state.db.calculate_recipe_cost(recipe_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_recipe_item_counts(state: State<AppState>) -> Result<Vec<(i64, i64)>, String> {
    state.db.get_recipe_item_counts().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_recipe_costs(state: State<AppState>) -> Result<Vec<(i64, f64)>, String> {
    state.db.get_all_recipe_costs().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_recipe_usage_count(state: State<AppState>, recipe_id: i64) -> Result<i64, String> {
    let count = state.db.get_recipe_usage_count(recipe_id).map_err(|e| e.to_string())?;
    Ok(count)
}

#[tauri::command]
pub fn would_create_recipe_cycle(state: State<AppState>, recipe_id: i64, ref_id: i64) -> Result<bool, String> {
    state.db.would_create_recipe_cycle(recipe_id, ref_id).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct RecipeDependents {
    pub menu_items: Vec<DependentRef>,
    pub parent_recipes: Vec<DependentRef>,
}

#[derive(serde::Serialize)]
pub struct DependentRef {
    pub id: i64,
    pub name: String,
}

#[tauri::command]
pub fn get_recipe_dependents(state: State<AppState>, recipe_id: i64) -> Result<RecipeDependents, String> {
    let (menu_items, parent_recipes) = state.db.get_recipe_dependents(recipe_id).map_err(|e| e.to_string())?;
    Ok(RecipeDependents {
        menu_items: menu_items.into_iter().map(|(id, name)| DependentRef { id, name }).collect(),
        parent_recipes: parent_recipes.into_iter().map(|(id, name)| DependentRef { id, name }).collect(),
    })
}

#[tauri::command]
pub fn get_material_dependents(state: State<AppState>, material_id: i64) -> Result<Vec<DependentRef>, String> {
    let recipes = state.db.get_material_dependents(material_id).map_err(|e| e.to_string())?;
    Ok(recipes.into_iter().map(|(id, name)| DependentRef { id, name }).collect())
}

// ==================== 菜單 API ====================

#[tauri::command]
pub fn get_menu_categories(state: State<AppState>) -> Result<Vec<MenuCategory>, String> {
    state.db.get_menu_categories().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_menu_category(state: State<AppState>, name: String, sort_no: Option<i32>) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner], "创建菜单分类")?;
    state.db.create_menu_category(&name, sort_no.unwrap_or(0)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_menu_items(state: State<AppState>, category_id: Option<i64>) -> Result<Vec<MenuItem>, String> {
    state.db.get_menu_items(category_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_menu_item(state: State<AppState>, req: CreateMenuItemRequest) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner], "创建菜单项")?;
    state.db.create_menu_item(&req.name, req.category_id, req.recipe_id, req.sales_price).map_err(db_err)
}

#[tauri::command]
pub fn get_menu_item_specs(state: State<AppState>, menu_item_id: i64) -> Result<Vec<MenuItemSpec>, String> {
    state.db.get_menu_item_specs(menu_item_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_menu_item_spec(state: State<AppState>, req: CreateMenuItemSpecRequest) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner], "创建菜单规格")?;
    state.db.create_menu_item_spec(req.menu_item_id, &req.spec_code, &req.spec_name, req.price_delta, req.qty_multiplier, 0).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_menu_item_spec(state: State<AppState>, id: i64, spec_code: Option<String>, spec_name: Option<String>, price_delta: Option<f64>, qty_multiplier: Option<f64>, sort_no: Option<i32>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "更新菜单规格")?;
    state.db.update_menu_item_spec(id, spec_code.as_deref(), spec_name.as_deref(), price_delta, qty_multiplier, sort_no).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_menu_item_spec(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "删除菜单规格")?;
    state.db.delete_menu_item_spec(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_menu_items_for_pos(state: State<AppState>, category_id: Option<i64>) -> Result<Vec<MenuItem>, String> {
    state.db.get_menu_items_for_pos(category_id).map_err(|e| e.to_string())
}

// ==================== 訂單 API ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    pub id: i64,
    pub order_no: String,
}

#[tauri::command]
pub fn create_order(state: State<AppState>, req: CreateOrderRequest) -> Result<CreateOrderResponse, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier], "创建订单")?;
    let (id, order_no) = state.db.create_order(&req.source, &req.dine_type, req.table_no.as_deref()).map_err(|e| e.to_string())?;
    Ok(CreateOrderResponse { id, order_no })
}

#[tauri::command]
pub fn get_orders(state: State<AppState>, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Order>, String> {
    state.db.get_orders(limit.unwrap_or(200), offset.unwrap_or(0)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_order_with_items(state: State<AppState>, order_id: i64) -> Result<OrderWithItems, String> {
    state.db.get_order_with_items(order_id).map(|(order, items)| OrderWithItems { order, items }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_order_item(state: State<AppState>, req: AddOrderItemRequest) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier], "添加订单商品")?;
    state.db.add_order_item(
        req.order_id,
        req.menu_item_id,
        req.qty,
        req.unit_price,
        req.spec_code.as_deref(),
        req.note.as_deref(),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn submit_order(state: State<AppState>, order_id: i64) -> Result<Vec<String>, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier], "提交订单")?;
    state.db.submit_order_full(order_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cancel_order(state: State<AppState>, order_id: i64, is_served: bool, reason: Option<String>) -> Result<Vec<String>, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier], "取消订单")?;
    if is_served {
        state.db.cancel_order_confirmed(order_id, reason.as_deref()).map_err(|e| e.to_string())
    } else {
        state.db.release_inventory_for_order(order_id, reason.as_deref()).map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn check_expiry_alerts(state: State<AppState>) -> Result<i64, String> {
    state.db.check_and_create_expiry_alerts().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_order_ready(state: State<AppState>, order_id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier], "标记订单出餐")?;
    state.db.mark_order_ready(order_id).map_err(|e| e.to_string())
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateOrderPaymentRequest {
    pub order_id: i64,
    pub payment_status: String,
    pub payment_method: Option<String>,
    pub amount_paid: f64,
}

#[tauri::command]
pub fn update_order_payment(state: State<AppState>, req: UpdateOrderPaymentRequest) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier], "更新订单支付")?;
    state.db.update_order_payment(req.order_id, &req.payment_status, req.payment_method.as_deref(), req.amount_paid)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn record_order_refund(state: State<AppState>, order_id: i64, refund_amount: f64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier], "记录退款")?;
    state.db.record_order_refund(order_id, refund_amount).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn refund_order_item(state: State<AppState>, order_id: i64, item_id: i64) -> Result<f64, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier], "退款订单项")?;
    state.db.refund_order_item(order_id, item_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_sales_by_hour(state: State<AppState>, start_date: String, end_date: String) -> Result<Vec<(i32, i64, f64)>, String> {
    state.db.get_sales_by_hour(&start_date, &end_date).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_sales_by_weekday(state: State<AppState>, start_date: String, end_date: String) -> Result<Vec<(i32, i64, f64)>, String> {
    state.db.get_sales_by_weekday(&start_date, &end_date).map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReceivePOItemRequest {
    pub item_id: i64,
    pub received_qty: f64,
    pub lot_no: Option<String>,
}

#[tauri::command]
pub fn receive_purchase_order_items(state: State<AppState>, po_id: i64, items: Vec<ReceivePOItemRequest>, operator: Option<String>) -> Result<Vec<String>, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "采购单入库")?;
    let mapped: Vec<(i64, f64, Option<String>)> = items.into_iter().map(|r| (r.item_id, r.received_qty, r.lot_no)).collect();
    state.db.receive_purchase_order_items(po_id, mapped, operator.as_deref())
        .map(|details| details.into_iter().map(|(_, lot, name, _, _, _)| format!("{}: {}", name, lot)).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn batch_cancel_orders(state: State<AppState>, ids: Vec<i64>) -> Result<usize, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier], "批量取消订单")?;
    state.db.batch_cancel_orders(&ids).map_err(|e| e.to_string())
}

// ==================== KDS API ====================

#[tauri::command]
pub fn get_kitchen_stations(state: State<AppState>) -> Result<Vec<KitchenStation>, String> {
    state.db.get_kitchen_stations().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_station_printer(state: State<AppState>, station_id: i64, printer_id: Option<i64>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "更新工作站打印机")?;
    state.db.update_station_printer(station_id, printer_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_station_tickets(state: State<AppState>, station_id: i64, status: Option<String>) -> Result<Vec<KitchenTicket>, String> {
    state.db.get_station_tickets(station_id, status.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_tickets(state: State<AppState>, status: Option<String>) -> Result<Vec<KitchenTicket>, String> {
    state.db.get_all_tickets(status.as_deref()).map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketWithItems {
    #[serde(flatten)]
    pub ticket: KitchenTicket,
    pub order_no: String,
    pub dine_type: String,
    pub table_no: Option<String>,
    pub items: Vec<OrderItem>,
}

#[tauri::command]
pub fn get_all_tickets_with_items(state: State<AppState>, status: Option<String>) -> Result<Vec<TicketWithItems>, String> {
    let tickets = state.db.get_all_tickets(status.as_deref()).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for ticket in tickets {
        if let Ok((order, items)) = state.db.get_order_with_items(ticket.order_id) {
            result.push(TicketWithItems {
                ticket,
                order_no: order.order_no,
                dine_type: order.dine_type,
                table_no: order.table_no,
                items,
            });
        }
    }
    Ok(result)
}

#[tauri::command]
pub fn get_tickets_for_order(state: State<AppState>, order_id: i64) -> Result<Vec<TicketWithItems>, String> {
    let tickets = state.db.get_tickets_for_order(order_id).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for ticket in tickets {
        if let Ok((order, items)) = state.db.get_order_with_items(ticket.order_id) {
            result.push(TicketWithItems { ticket, order_no: order.order_no, dine_type: order.dine_type, table_no: order.table_no, items });
        }
    }
    Ok(result)
}

#[tauri::command]
pub fn start_ticket(state: State<AppState>, ticket_id: i64, operator: Option<String>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier, UserRole::Chef], "开始工单")?;
    state.db.start_ticket(ticket_id, operator.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn finish_ticket(state: State<AppState>, ticket_id: i64, operator: Option<String>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier, UserRole::Chef], "完成工单")?;
    state.db.finish_ticket(ticket_id, operator.as_deref()).map_err(|e| e.to_string())
}

// ==================== 庫存 API ====================

#[tauri::command]
pub fn get_inventory_batches(state: State<AppState>, material_id: Option<i64>) -> Result<Vec<InventoryBatch>, String> {
    state.db.get_inventory_batches(material_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_inventory_summary(state: State<AppState>) -> Result<Vec<InventorySummary>, String> {
    state.db.get_inventory_summary().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_inventory_txn(state: State<AppState>, req: CreateInventoryTxnRequest) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "创建库存流水")?;
    let cost_delta = req.cost_delta.unwrap_or(0.0);
    state.db.create_inventory_txn(
        &req.txn_type,
        req.ref_type.as_deref(),
        req.ref_id,
        req.lot_id,
        req.material_id,
        req.state_id,
        req.qty_delta,
        Some(cost_delta),
        req.operator.as_deref(),
        req.note.as_deref(),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_inventory_txns(state: State<AppState>, material_id: Option<i64>, limit: Option<i64>) -> Result<Vec<InventoryTxn>, String> {
    state.db.get_inventory_txns(material_id, limit.unwrap_or(100)).map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBatchRequest {
    pub material_id: i64,
    pub state_id: Option<i64>,
    pub lot_no: String,
    pub supplier_id: Option<i64>,
    pub brand: Option<String>,
    pub spec: Option<String>,
    pub quantity: f64,
    pub cost_per_unit: f64,
    pub production_date: Option<String>,
    pub expiry_date: Option<String>,
    pub ice_coating_rate: Option<f64>,
    pub quality_rate: Option<f64>,
    pub seasonal_factor: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdjustInventoryRequest {
    pub material_id: i64,
    pub lot_id: i64,
    pub qty_delta: f64,
    pub reason: String,
    pub operator: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordWastageRequest {
    pub material_id: i64,
    pub lot_id: i64,
    pub qty: f64,
    pub wastage_type: String,
    pub operator: Option<String>,
    pub note: Option<String>,
}

#[tauri::command]
pub fn create_inventory_batch(state: State<AppState>, req: CreateBatchRequest) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "创建库存批次")?;
    let seasonal = req.seasonal_factor.unwrap_or(1.0);
    state.db.create_inventory_batch(
        req.material_id, req.state_id, &req.lot_no, req.supplier_id,
        req.brand.as_deref(), req.spec.as_deref(), req.quantity, req.cost_per_unit,
        req.production_date.as_deref(), req.expiry_date.as_deref(),
        req.ice_coating_rate, req.quality_rate, seasonal,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_inventory_batch(state: State<AppState>, batch_id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "删除库存批次")?;
    state.db.delete_inventory_batch(batch_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn adjust_inventory(state: State<AppState>, req: AdjustInventoryRequest) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "调整库存")?;
    state.db.adjust_inventory(req.lot_id, req.qty_delta, req.operator.as_deref(), req.note.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn record_wastage(state: State<AppState>, req: RecordWastageRequest) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "记录损耗")?;
    state.db.record_wastage(req.lot_id, req.qty, Some(&req.wastage_type), req.operator.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_material(state: State<AppState>, id: i64, name: Option<String>, category_id: Option<i64>, shelf_life_days: Option<i32>, min_qty: Option<f64>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "更新材料")?;
    state.db.update_material(id, name.as_deref(), category_id, shelf_life_days, min_qty).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_material(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "删除材料")?;
    state.db.delete_material(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_material_category(state: State<AppState>, id: i64, name: String, sort_no: i32) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "更新材料分类")?;
    state.db.update_material_category(id, &name, sort_no).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_material_category(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "删除材料分类")?;
    state.db.delete_material_category(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_tag(state: State<AppState>, id: i64, name: String, color: Option<String>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "更新标签")?;
    state.db.update_tag(id, &name, color.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_tag(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "删除标签")?;
    state.db.delete_tag(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_material_tag(state: State<AppState>, material_id: i64, tag_id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "移除材料标签")?;
    state.db.remove_material_tag(material_id, tag_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_supplier(state: State<AppState>, id: i64, name: Option<String>, phone: Option<String>, contact_person: Option<String>, address: Option<String>, note: Option<String>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "更新供应商")?;
    state.db.update_supplier(id, name.as_deref(), phone.as_deref(), contact_person.as_deref(), address.as_deref(), note.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_supplier(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "删除供应商")?;
    state.db.delete_supplier(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_menu_item(state: State<AppState>, id: i64, name: Option<String>, category_id: Option<i64>, recipe_id: Option<i64>, sales_price: Option<f64>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "更新菜单项")?;
    state.db.update_menu_item(id, name.as_deref(), category_id, recipe_id, sales_price).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_menu_item_availability(state: State<AppState>, id: i64, is_available: bool) -> Result<bool, String> {
    require_roles(&state, &[UserRole::Owner], "更新菜单上下架")?;
    state.db.set_menu_item_availability(id, is_available).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn batch_set_menu_item_availability(state: State<AppState>, ids: Vec<i64>, is_available: bool) -> Result<usize, String> {
    require_roles(&state, &[UserRole::Owner], "批量更新菜单上下架")?;
    state.db.batch_toggle_menu_item_availability(&ids, is_available).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn batch_update_menu_item_prices(state: State<AppState>, ids: Vec<i64>, mode: String, value: f64) -> Result<usize, String> {
    require_roles(&state, &[UserRole::Owner], "批量更新菜单价格")?;
    let normalized_mode = mode.trim().to_lowercase();
    match normalized_mode.as_str() {
        "set" | "delta" | "percent" => state.db.batch_update_menu_item_prices(&ids, &normalized_mode, value).map_err(|e| e.to_string()),
        _ => Err("不支持的调价模式".to_string()),
    }
}

#[tauri::command]
pub fn toggle_menu_item_favorite(state: State<AppState>, id: i64) -> Result<bool, String> {
    require_roles(&state, &[UserRole::Owner], "更新菜单收藏状态")?;
    state.db.toggle_menu_item_favorite(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_menu_item(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "删除菜单项")?;
    state.db.delete_menu_item(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_menu_category(state: State<AppState>, id: i64, name: String, sort_no: i32) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "更新菜单分类")?;
    state.db.update_menu_category(id, Some(&name), Some(sort_no)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_menu_category(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "删除菜单分类")?;
    state.db.delete_menu_category(id).map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComboComponentInput {
    pub component_item_id: i64,
    pub qty: i32,
}

#[tauri::command]
pub fn create_combo(state: State<AppState>, name: String, sales_price: f64, description: Option<String>, components: Vec<ComboComponentInput>) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner], "创建套餐")?;
    let comps: Vec<(i64, i32)> = components.into_iter().map(|c| (c.component_item_id, c.qty)).collect();
    state.db.create_combo(&name, sales_price, description.as_deref(), &comps).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_combos(state: State<AppState>) -> Result<Vec<ComboWithComponents>, String> {
    state.db.list_combos().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_combo(state: State<AppState>, menu_item_id: i64, name: Option<String>, sales_price: Option<f64>, description: Option<String>, is_available: Option<bool>, components: Option<Vec<ComboComponentInput>>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "更新套餐")?;
    let comps: Option<Vec<(i64, i32)>> = components.map(|cs| cs.into_iter().map(|c| (c.component_item_id, c.qty)).collect());
    state.db.update_combo(menu_item_id, name.as_deref(), sales_price, description.as_deref(), is_available, comps.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_combo(state: State<AppState>, menu_item_id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "删除套餐")?;
    state.db.delete_combo(menu_item_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_recipe(state: State<AppState>, id: i64, name: Option<String>, recipe_type: Option<String>, output_qty: Option<f64>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "更新配方")?;
    state.db.update_recipe(id, name.as_deref(), recipe_type.as_deref(), output_qty, None).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_recipe_type(state: State<AppState>, id: i64, code: String, name: String, description: Option<String>, sort_no: i32) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "更新配方类型")?;
    state.db.update_recipe_type(id, &code, &name, description.as_deref(), sort_no).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_recipe_type(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "删除配方类型")?;
    state.db.delete_recipe_type(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_recipe(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "删除配方")?;
    state.db.delete_recipe(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_recipe_item(state: State<AppState>, item_id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "删除配方项")?;
    state.db.delete_recipe_item(item_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_station_menu_item(state: State<AppState>, station_id: i64, menu_item_id: i64) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner], "配置工作站菜单")?;
    state.db.add_station_menu_item(station_id, menu_item_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_station_menu_item(state: State<AppState>, station_id: i64, menu_item_id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "移除工作站菜单")?;
    state.db.remove_station_menu_item(station_id, menu_item_id).map_err(|e| e.to_string())
}

// ==================== 健康檢查 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct TelemetryPayload {
    pub client_id: String,
    pub version: String,
    pub event_type: String, // "heartbeat", "error", "action"
    pub uptime_hours: f64,
    pub today_sales: f64,
    pub today_orders: i32,
    pub metadata: Option<serde_json::Value>,
}

fn telemetry_endpoint(candidate: Option<&str>) -> String {
    const DEFAULT_URL: &str = "https://your-cloud-server.com/api/telemetry/heartbeat";
    let default = std::env::var("CUCKOO_TELEMETRY_URL").unwrap_or_else(|_| DEFAULT_URL.to_string());
    let allowlist = std::env::var("CUCKOO_TELEMETRY_ALLOWLIST")
        .unwrap_or_else(|_| default.clone())
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    if let Some(url) = candidate {
        if allowlist.iter().any(|allowed| allowed == url) && url.starts_with("https://") {
            return url.to_string();
        }
    }
    if default.starts_with("https://") { default } else { DEFAULT_URL.to_string() }
}

#[tauri::command]
pub async fn report_telemetry(
    payload: TelemetryPayload,
    webhook_url: Option<String>,
) -> Result<(), String> {
    let url = telemetry_endpoint(webhook_url.as_deref());

    // Skip if the URL is still the unconfigured placeholder — avoids noisy
    // network errors in deployments that haven't set up a telemetry server.
    if url.contains("your-cloud-server.com") {
        return Ok(());
    }

    let client = reqwest::Client::new();
    match client.post(&url)
        .json(&payload)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                Ok(())
            } else {
                Err(format!("server returned: {}", resp.status()))
            }
        }
        Err(e) => Err(format!("request failed: {}", e))
    }
}

#[tauri::command]
pub fn health_check(state: State<AppState>) -> String {
    match state.db.health_check() {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {}", e),
    }
}

#[tauri::command]
pub fn backup_database(state: State<AppState>, dest_dir: Option<String>) -> Result<String, String> {
    require_roles(&state, &[UserRole::Owner], "备份数据库")?;
    let backup_dir = match dest_dir {
        Some(d) => std::path::PathBuf::from(d),
        // Desktop: user-visible Documents. Android (document_dir → None): fall back
        // to the app data dir (db's parent, sandbox) so backup never fails on launch path.
        None => dirs::document_dir()
            .map(|d| d.join("Cuckoo 备份"))
            .unwrap_or_else(|| {
                state.db_path.parent()
                    .map(|p| p.join("backups"))
                    .unwrap_or_else(|| std::path::PathBuf::from("backups"))
            }),
    };
    std::fs::create_dir_all(&backup_dir).map_err(|e| format!("创建备份目录失败: {}", e))?;
    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let dest = backup_dir.join(format!("cuckoo_{}.db", ts));
    state.db.backup_to(dest.to_str().ok_or("路径含非法字符")?)
        .map_err(|e| format!("备份失败: {}", e))?;
    Ok(dest.to_string_lossy().to_string())
}

/// Export a backup to a specific user-chosen full file path (for off-device backup).
#[tauri::command]
pub fn export_backup(state: State<AppState>, dest_path: String) -> Result<String, String> {
    require_roles(&state, &[UserRole::Owner], "导出备份")?;
    let dest = std::path::Path::new(&dest_path);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    state.db.backup_to(dest.to_str().ok_or("路径含非法字符")?)
        .map_err(|e| format!("导出失败: {}", e))?;
    Ok(dest.to_string_lossy().to_string())
}

#[tauri::command]
pub fn restore_database(state: State<AppState>, path: String) -> Result<String, String> {
    require_roles(&state, &[UserRole::Owner], "恢复数据库")?;
    let src = std::path::Path::new(&path);
    if !src.exists() {
        return Err("备份文件不存在".to_string());
    }
    // Validate SQLite magic bytes
    let magic = {
        let mut f = std::fs::File::open(src).map_err(|e| format!("无法读取文件: {}", e))?;
        let mut buf = [0u8; 16];
        use std::io::Read;
        f.read_exact(&mut buf).map_err(|e| format!("读取文件失败: {}", e))?;
        buf
    };
    if &magic[..15] != b"SQLite format 3" {
        return Err("所选文件不是有效的 SQLite 数据库".to_string());
    }
    // Safety backup of current DB first
    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
    if let Some(parent) = state.db_path.parent() {
        let safety = parent.join(format!("cuckoo_pre_restore_{}.db", ts));
        let _ = state.db.backup_to(safety.to_str().unwrap_or("pre_restore.db"));
    }
    // Stage restore file next to DB for startup swap
    let pending = state.db_path.with_file_name("pending_restore.db");
    std::fs::copy(src, &pending).map_err(|e| format!("复制文件失败: {}", e))?;
    Ok("备份已暂存，请重启应用完成恢复".to_string())
}

fn payment_config_path(db_path: &std::path::Path) -> std::path::PathBuf {
    db_path.with_file_name("payment-config.json")
}

#[tauri::command]
pub fn get_payment_qr(state: State<AppState>) -> Result<Option<String>, String> {
    let path = payment_config_path(&state.db_path);
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path).map_err(|e| format!("读取失败: {}", e))?;
    let v: serde_json::Value = serde_json::from_str(&content).unwrap_or(serde_json::Value::Null);
    Ok(v["data"].as_str().map(|s| s.to_string()))
}

#[tauri::command]
pub fn set_payment_qr(state: State<AppState>, data: Option<String>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "设置收款码")?;
    let path = payment_config_path(&state.db_path);
    let content = serde_json::json!({ "data": data }).to_string();
    std::fs::write(&path, content).map_err(|e| format!("写入失败: {}", e))?;
    Ok(())
}

// ==================== 自動打印設置 ====================

fn auto_print_config_path(db_path: &std::path::Path) -> std::path::PathBuf {
    db_path.with_file_name("auto-print-config.json")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoPrintSettings {
    pub kitchen: bool,
    pub po: bool,
    pub receipt: bool,
}

#[tauri::command]
pub fn get_auto_print_settings(state: State<AppState>) -> Result<AutoPrintSettings, String> {
    let path = auto_print_config_path(&state.db_path);
    if !path.exists() {
        return Ok(AutoPrintSettings { kitchen: false, po: false, receipt: false });
    }
    let content = std::fs::read_to_string(&path).map_err(|e| format!("读取失败: {}", e))?;
    let v: serde_json::Value = serde_json::from_str(&content).unwrap_or(serde_json::Value::Null);
    Ok(AutoPrintSettings {
        kitchen: v["kitchen"].as_bool().unwrap_or(false),
        po: v["po"].as_bool().unwrap_or(false),
        receipt: v["receipt"].as_bool().unwrap_or(false),
    })
}

#[tauri::command]
pub fn set_auto_print_settings(state: State<AppState>, kitchen: bool, po: bool, receipt: bool) -> Result<(), String> {
    let path = auto_print_config_path(&state.db_path);
    let content = serde_json::json!({ "kitchen": kitchen, "po": po, "receipt": receipt }).to_string();
    std::fs::write(&path, content).map_err(|e| format!("写入失败: {}", e))?;
    Ok(())
}

// ==================== 打印機 API ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePrinterRequest {
    pub name: String,
    pub printer_type: String,
    pub connection_type: String,
    pub feie_user: Option<String>,
    pub feie_ukey: Option<String>,
    pub feie_sn: Option<String>,
    pub feie_key: Option<String>,
    pub lan_ip: Option<String>,
    pub lan_port: Option<i32>,
    pub paper_width: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendPrintTaskRequest {
    pub printer_id: i64,
    pub task_type: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<i64>,
    pub content: String,
}

#[tauri::command]
pub fn get_printers(state: State<AppState>) -> Result<Vec<PrinterConfig>, String> {
    state.db.get_printers().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_default_printer(state: State<AppState>) -> Result<Option<PrinterConfig>, String> {
    state.db.get_default_printer().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_printer(state: State<AppState>, req: CreatePrinterRequest) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner], "创建打印机")?;
    state.db.create_printer(
        &req.name,
        &req.printer_type,
        &req.connection_type,
        req.feie_user.as_deref(),
        req.feie_ukey.as_deref(),
        req.feie_sn.as_deref(),
        req.feie_key.as_deref(),
        req.lan_ip.as_deref(),
        req.lan_port.unwrap_or(9100),
        req.paper_width.as_deref().unwrap_or("80mm"),
        req.is_default.unwrap_or(false),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_printer(state: State<AppState>, id: i64, name: Option<String>, printer_type: Option<String>, connection_type: Option<String>, feie_user: Option<String>, feie_ukey: Option<String>, feie_sn: Option<String>, feie_key: Option<String>, lan_ip: Option<String>, lan_port: Option<i32>, paper_width: Option<String>, is_default: Option<bool>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "更新打印机")?;
    state.db.update_printer(
        id,
        name.as_deref(),
        printer_type.as_deref(),
        connection_type.as_deref(),
        feie_user.as_deref(),
        feie_ukey.as_deref(),
        feie_sn.as_deref(),
        feie_key.as_deref(),
        lan_ip.as_deref(),
        lan_port,
        paper_width.as_deref(),
        is_default,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_printer(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "删除打印机")?;
    state.db.delete_printer(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn scan_lan_printers(_state: State<AppState>, subnet: String, timeout_ms: Option<u64>) -> Result<Vec<LanPrinter>, String> {
    let timeout = timeout_ms.unwrap_or(500);
    Ok(LAN_SCAN(&subnet, timeout))
}

#[tauri::command]
pub fn test_feie_printer(_state: State<AppState>, user: String, ukey: String, sn: String) -> Result<String, String> {
    let content = format!(
        "=== Cuckoo 打印測試 ===\n打印機: {}\n時間: {}\n",
        sn,
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
    );
    printer::feie_print(&user, &ukey, &sn, &content).map_err(print_err)
}

#[tauri::command]
pub fn test_lan_printer(_state: State<AppState>, ip: String, port: Option<i32>) -> Result<String, String> {
    let port = port.unwrap_or(9100);
    let mut builder = EscPosBuilder::new();
    builder.align_center().bold_on().double_height()
        .text_ln("Cuckoo 打印測試")
        .normal_size().bold_off()
        .text_ln(&format!("打印機: {}:{}", ip, port))
        .text_ln(&format!("時間: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")))
        .feed_lines(3).cut_paper();

    printer::lan_print_escpos(&ip, port, builder)
        .map(|_| "測試頁發送成功".to_string())
        .map_err(print_err)
}

#[tauri::command]
pub fn send_print_task(state: State<AppState>, app: tauri::AppHandle, req: SendPrintTaskRequest) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner], "发送打印任务")?;
    let printer = state.db.get_printers().map_err(|e| e.to_string())?
        .into_iter()
        .find(|p| p.id == req.printer_id)
        .ok_or(format!("打印機 #{} 不存在", req.printer_id))?;

    let task_id = state.db.create_print_task(
        &req.task_type,
        req.ref_type.as_deref(),
        req.ref_id,
        &req.content,
        Some(req.printer_id),
        Some(&printer.name),
    ).map_err(|e| e.to_string())?;

    let payload = match printer.connection_type.as_str() {
        "feie" => PrintPayload::Feie {
            user: printer.feie_user.ok_or("飛鵝用戶未配置")?,
            ukey: printer.feie_ukey.ok_or("飛鵝 UKEY 未配置")?,
            sn:   printer.feie_sn.ok_or("飛鵝 SN 未配置")?,
            content: req.content,
        },
        "lan" => PrintPayload::Lan {
            ip:   printer.lan_ip.ok_or("局域網 IP 未配置")?,
            port: printer.lan_port,
            data: req.content.into_bytes(),
        },
        other => return Err(format!("不支持的連接類型: {}", other)),
    };

    dispatch_print(app, task_id, payload);
    Ok(task_id)
}

#[tauri::command]
pub fn print_kitchen_ticket(state: State<AppState>, app: tauri::AppHandle, order_no: String, dine_type: String, items: Vec<(String, f64, Option<String>)>, note: Option<String>, printer_id: Option<i64>) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier, UserRole::Chef], "打印厨房票")?;
    let printer = match printer_id {
        Some(id) => state.db.get_printers().map_err(|e| e.to_string())?
            .into_iter()
            .find(|p| p.id == id)
            .ok_or(format!("打印機 #{} 不存在", id))?,
        None => state.db.get_default_printer().map_err(|e| e.to_string())?
            .ok_or("未配置默認打印機")?,
    };

    let text_content = printer::build_kitchen_ticket_text(&order_no, &dine_type, &items, note.as_deref());
    let task_id = state.db.create_print_task(
        "kitchen_ticket",
        Some("order"),
        None,
        &text_content,
        Some(printer.id),
        Some(&printer.name),
    ).map_err(|e| e.to_string())?;

    let payload = match printer.connection_type.as_str() {
        "feie" => PrintPayload::Feie {
            user: printer.feie_user.ok_or("飛鵝用戶未配置")?,
            ukey: printer.feie_ukey.ok_or("飛鵝 UKEY 未配置")?,
            sn:   printer.feie_sn.ok_or("飛鵝 SN 未配置")?,
            content: text_content,
        },
        "lan" => PrintPayload::Lan {
            ip:   printer.lan_ip.ok_or("局域網 IP 未配置")?,
            port: printer.lan_port,
            data: printer::build_kitchen_ticket_content(&order_no, &dine_type, &items, note.as_deref()).build(),
        },
        other => return Err(format!("不支持的連接類型: {}", other)),
    };

    dispatch_print(app, task_id, payload);
    Ok(task_id)
}

#[tauri::command]
pub fn print_order_receipt(state: State<AppState>, app: tauri::AppHandle, order_id: i64, printer_id: Option<i64>) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier], "打印小票")?;
    let (order, order_items) = state.db.get_order_with_items(order_id).map_err(|e| e.to_string())?;
    let menu_items = state.db.get_menu_items(None).map_err(|e| e.to_string())?;
    let item_map: std::collections::HashMap<i64, String> = menu_items.into_iter().map(|m| (m.id, m.name)).collect();
    let items: Vec<(String, f64, f64)> = order_items.iter().map(|oi| {
        let name = item_map.get(&oi.menu_item_id).cloned().unwrap_or_else(|| format!("商品#{}", oi.menu_item_id));
        (name, oi.qty, oi.unit_price)
    }).collect();

    let printer = match printer_id {
        Some(id) => state.db.get_printers().map_err(|e| e.to_string())?
            .into_iter().find(|p| p.id == id)
            .ok_or(format!("打印機 #{} 不存在", id))?,
        None => state.db.get_default_printer().map_err(|e| e.to_string())?
            .ok_or("未配置默認打印機")?,
    };

    let pay_method = order.payment_method.as_deref();
    let text_content = printer::build_receipt_text(
        &order.order_no, &order.dine_type, order.table_no.as_deref(),
        &items, order.amount_total, pay_method, order.amount_paid,
    );
    let task_id = state.db.create_print_task(
        "receipt", Some("order"), Some(order_id), &text_content, Some(printer.id), Some(&printer.name),
    ).map_err(|e| e.to_string())?;

    let payload = match printer.connection_type.as_str() {
        "feie" => PrintPayload::Feie {
            user: printer.feie_user.ok_or("飛鵝用戶未配置")?,
            ukey: printer.feie_ukey.ok_or("飛鵝 UKEY 未配置")?,
            sn:   printer.feie_sn.ok_or("飛鵝 SN 未配置")?,
            content: text_content,
        },
        "lan" => PrintPayload::Lan {
            ip:   printer.lan_ip.ok_or("局域網 IP 未配置")?,
            port: printer.lan_port,
            data: printer::build_receipt_content(
                &order.order_no, &order.dine_type, order.table_no.as_deref(),
                &items, order.amount_total, pay_method, order.amount_paid,
            ).build(),
        },
        other => return Err(format!("不支持的連接類型: {}", other)),
    };

    dispatch_print(app, task_id, payload);
    Ok(task_id)
}

#[tauri::command]
pub fn print_batch_label(state: State<AppState>, app: tauri::AppHandle, lot_no: String, material_name: String, quantity: f64, unit: String, expiry_date: Option<String>, supplier_name: Option<String>, printer_id: Option<i64>) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "打印批次标签")?;
    let printer = match printer_id {
        Some(id) => state.db.get_printers().map_err(|e| e.to_string())?
            .into_iter()
            .find(|p| p.id == id)
            .ok_or(format!("打印機 #{} 不存在", id))?,
        None => state.db.get_default_printer().map_err(|e| e.to_string())?
            .ok_or("未配置默認打印機")?,
    };

    let builder = printer::build_batch_label_content(
        &lot_no, &material_name, quantity, &unit,
        expiry_date.as_deref(), supplier_name.as_deref(),
    );

    let tspl_text = builder.build();
    let task_id = state.db.create_print_task(
        "batch_label",
        Some("batch"),
        None,
        &tspl_text,
        Some(printer.id),
        Some(&printer.name),
    ).map_err(|e| e.to_string())?;

    let payload = match printer.connection_type.as_str() {
        "feie" => PrintPayload::Feie {
            user: printer.feie_user.ok_or("飛鵝用戶未配置")?,
            ukey: printer.feie_ukey.ok_or("飛鵝 UKEY 未配置")?,
            sn:   printer.feie_sn.ok_or("飛鵝 SN 未配置")?,
            content: tspl_text,
        },
        "lan" => PrintPayload::Lan {
            ip:   printer.lan_ip.ok_or("局域網 IP 未配置")?,
            port: printer.lan_port,
            data: builder.build().into_bytes(),
        },
        other => return Err(format!("不支持的連接類型: {}", other)),
    };

    dispatch_print(app, task_id, payload);
    Ok(task_id)
}

#[tauri::command]
pub fn get_print_tasks(state: State<AppState>, limit: Option<i64>) -> Result<Vec<PrintTask>, String> {
    state.db.get_print_tasks(limit.unwrap_or(50)).map_err(|e| e.to_string())
}

// ==================== 採購單命令 ====================

#[tauri::command]
pub fn get_purchase_orders(state: State<AppState>, status: Option<String>) -> Result<Vec<PurchaseOrder>, String> {
    state.db.get_purchase_orders(status.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_purchase_order_with_items(state: State<AppState>, po_id: i64) -> Result<PurchaseOrderWithItems, String> {
    state.db.get_purchase_order_with_items(po_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_purchase_order(state: State<AppState>, supplier_id: Option<i64>, expected_date: Option<String>) -> Result<String, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "创建采购单")?;
    state.db.create_purchase_order(supplier_id, expected_date.as_deref()).map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePurchaseOrderItemRequest {
    pub po_id: i64,
    pub material_id: i64,
    pub qty: f64,
    pub unit_id: Option<i64>,
    pub cost_per_unit: f64,
}

#[tauri::command]
pub fn add_purchase_order_item(state: State<AppState>, req: CreatePurchaseOrderItemRequest) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "添加采购项")?;
    state.db.add_purchase_order_item(req.po_id, req.material_id, req.qty, req.unit_id, req.cost_per_unit).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_purchase_order_status(state: State<AppState>, po_id: i64, status: String) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "更新采购单状态")?;
    state.db.update_purchase_order_status(po_id, &status).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_purchase_order(state: State<AppState>, po_id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "删除采购单")?;
    state.db.delete_purchase_order(po_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn receive_purchase_order(state: State<AppState>, app: tauri::AppHandle, po_id: i64, operator: Option<String>, auto_print: Option<bool>) -> Result<Vec<i64>, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "采购单入库")?;
    let batches = state.db.receive_purchase_order(po_id, operator.as_deref()).map_err(|e| e.to_string())?;
    if auto_print.unwrap_or(false) {
        let mut print_ids = Vec::new();
        for (_batch_id, lot_no, material_name, unit, qty, supplier_name) in batches {
            let print_id = print_batch_label(
                state.clone(),
                app.clone(),
                lot_no,
                material_name,
                qty,
                unit,
                None,
                supplier_name,
                None,
            ).map_err(|e| e.to_string())?;
            print_ids.push(print_id);
        }
        Ok(print_ids)
    } else {
        Ok(batches.iter().map(|(id, _, _, _, _, _)| *id).collect())
    }
}

// ==================== 生產單命令 ====================

#[tauri::command]
pub fn get_production_orders(state: State<AppState>, status: Option<String>) -> Result<Vec<ProductionOrder>, String> {
    state.db.get_production_orders(status.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_production_order_with_items(state: State<AppState>, production_id: i64) -> Result<ProductionOrderWithItems, String> {
    state.db.get_production_order_with_items(production_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_production_order(state: State<AppState>, recipe_id: i64, planned_qty: f64, operator: Option<String>) -> Result<String, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Chef, UserRole::Warehouse], "创建生产单")?;
    state.db.create_production_order(recipe_id, planned_qty, operator.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_production_order(state: State<AppState>, production_id: i64, operator: Option<String>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Chef, UserRole::Warehouse], "开始生产")?;
    state.db.start_production_order(production_id, operator.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn complete_production_order(state: State<AppState>, production_id: i64, actual_qty: f64, operator: Option<String>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Chef, UserRole::Warehouse], "完成生产")?;
    state.db.complete_production_order(production_id, actual_qty, operator.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_production_order(state: State<AppState>, production_id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Chef, UserRole::Warehouse], "删除生产单")?;
    state.db.delete_production_order(production_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn check_production_materials(state: State<AppState>, recipe_id: i64, planned_qty: f64) -> Result<Vec<(String, f64, f64)>, String> {
    state.db.check_production_materials(recipe_id, planned_qty).map_err(|e| e.to_string())
}

// ==================== 盤點命令 ====================

#[tauri::command]
pub fn get_stocktakes(state: State<AppState>, status: Option<String>) -> Result<Vec<Stocktake>, String> {
    state.db.get_stocktakes(status.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_stocktake_with_items(state: State<AppState>, stocktake_id: i64) -> Result<StocktakeWithItems, String> {
    state.db.get_stocktake_with_items(stocktake_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_stocktake(state: State<AppState>, operator: Option<String>, note: Option<String>) -> Result<String, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "创建盘点")?;
    state.db.create_stocktake(operator.as_deref(), note.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_stocktake_item(state: State<AppState>, item_id: i64, actual_qty: f64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "更新盘点项")?;
    state.db.update_stocktake_item(item_id, actual_qty).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn complete_stocktake(state: State<AppState>, stocktake_id: i64, operator: Option<String>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "完成盘点")?;
    state.db.complete_stocktake(stocktake_id, operator.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_stocktake(state: State<AppState>, stocktake_id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Warehouse], "删除盘点")?;
    state.db.delete_stocktake(stocktake_id).map_err(|e| e.to_string())
}

// ==================== 加料/去料命令 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct AddModifierRequest {
    pub order_item_id: i64,
    pub modifier_type: String,
    pub material_id: Option<i64>,
    pub qty: f64,
    pub price_delta: f64,
}

#[tauri::command]
pub fn add_order_item_modifier(state: State<AppState>, req: AddModifierRequest) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier], "添加订单加料")?;
    state.db.add_order_item_modifier(req.order_item_id, &req.modifier_type, req.material_id, req.qty, req.price_delta).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_order_item_modifiers(state: State<AppState>, order_item_id: i64) -> Result<Vec<OrderItemModifier>, String> {
    state.db.get_order_item_modifiers(order_item_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_order_item_modifier(state: State<AppState>, modifier_id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier], "删除订单加料")?;
    state.db.delete_order_item_modifier(modifier_id).map_err(|e| e.to_string())
}

// ==================== 報表命令 ====================

#[tauri::command]
pub fn get_sales_report(state: State<AppState>, start_date: String, end_date: String) -> Result<Vec<(String, f64, i64, f64)>, String> {
    state.db.get_sales_report(&start_date, &end_date).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_sales_by_category(state: State<AppState>, start_date: String, end_date: String) -> Result<Vec<(String, f64, i64)>, String> {
    state.db.get_sales_by_category(&start_date, &end_date).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_gross_profit_report(state: State<AppState>, start_date: String, end_date: String) -> Result<Vec<(String, f64, f64, f64, f64, f64)>, String> {
    state.db.get_gross_profit_report(&start_date, &end_date).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_top_selling_items(state: State<AppState>, start_date: String, end_date: String, limit: Option<i64>) -> Result<Vec<(String, f64, i64, f64)>, String> {
    state.db.get_top_selling_items(&start_date, &end_date, limit.unwrap_or(10)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_material_consumption_report(state: State<AppState>, start_date: String, end_date: String) -> Result<Vec<(String, f64, f64, f64)>, String> {
    state.db.get_material_consumption_report(&start_date, &end_date).map_err(|e| e.to_string())
}

fn csv_quote(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

#[allow(dead_code)]
#[tauri::command]
pub fn export_report_csv(state: State<AppState>, report_type: String, start_date: String, end_date: String) -> Result<String, String> {
    let csv = match report_type.as_str() {
        "sales" => {
            let data = state.db.get_sales_report(&start_date, &end_date).map_err(|e| e.to_string())?;
            let mut lines = vec!["日期,销售额,订单数,实收".to_string()];
            for (date, sales, count, collected) in data {
                lines.push(format!("{},{:.2},{},{:.2}", csv_quote(&date), sales, count, collected));
            }
            lines.join("\n")
        }
        "category" => {
            let data = state.db.get_sales_by_category(&start_date, &end_date).map_err(|e| e.to_string())?;
            let mut lines = vec!["分类,销售额,订单数".to_string()];
            for (cat, sales, count) in data {
                lines.push(format!("{},{:.2},{}", csv_quote(&cat), sales, count));
            }
            lines.join("\n")
        }
        "top_items" => {
            let data = state.db.get_top_selling_items(&start_date, &end_date, 50).map_err(|e| e.to_string())?;
            let mut lines = vec!["菜品,销量,订单数,销售额".to_string()];
            for (name, qty, count, sales) in data {
                lines.push(format!("{},{:.2},{},{:.2}", csv_quote(&name), qty, count, sales));
            }
            lines.join("\n")
        }
        "material" => {
            let data = state.db.get_material_consumption_report(&start_date, &end_date).map_err(|e| e.to_string())?;
            let mut lines = vec!["原料,消耗数量,平均成本,总成本".to_string()];
            for (name, qty, avg_cost, total) in data {
                lines.push(format!("{},{:.2},{:.2},{:.2}", csv_quote(&name), qty, avg_cost, total));
            }
            lines.join("\n")
        }
        _ => return Err(format!("不支持的报告类型: {}", report_type)),
    };
    Ok(csv)
}

#[tauri::command]
pub fn update_recipe_item(state: State<AppState>, item_id: i64, qty: Option<f64>, wastage_rate: Option<f64>, note: Option<String>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "更新配方项")?;
    state.db.update_recipe_item(item_id, qty, wastage_rate, note.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_station_menu_items(state: State<AppState>, station_id: i64) -> Result<Vec<MenuItem>, String> {
    state.db.get_station_menu_items(station_id).map_err(|e| e.to_string())
}

// ==================== 打印模板命令 ====================

use crate::database::{PrintTemplate, PrintPreviewResult, CreatePrintTemplateRequest, PrintTicketType, CreatePrintTicketTypeRequest, UpdatePrintTicketTypeRequest};

#[tauri::command]
pub fn get_print_templates(state: State<AppState>, template_type: Option<String>) -> Result<Vec<PrintTemplate>, String> {
    state.db.get_print_templates(template_type.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_print_ticket_types(state: State<AppState>) -> Result<Vec<PrintTicketType>, String> {
    state.db.get_print_ticket_types().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_print_ticket_type(state: State<AppState>, id: i64) -> Result<PrintTicketType, String> {
    state.db.get_print_ticket_type(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_print_ticket_type(state: State<AppState>, req: CreatePrintTicketTypeRequest) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner], "创建票据类型")?;
    state.db.create_print_ticket_type(&req).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_print_ticket_type(state: State<AppState>, id: i64, req: UpdatePrintTicketTypeRequest) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "更新票据类型")?;
    state.db.update_print_ticket_type(id, &req).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_print_ticket_type(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "删除票据类型")?;
    state.db.delete_print_ticket_type(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_default_ticket_type(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "设置默认票据类型")?;
    state.db.set_default_ticket_type(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ensure_default_ticket_types(state: State<AppState>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "初始化默认票据类型")?;
    state.db.ensure_default_ticket_types().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_print_template(state: State<AppState>, id: i64) -> Result<PrintTemplate, String> {
    state.db.get_print_template(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_print_template(state: State<AppState>, req: CreatePrintTemplateRequest) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner], "创建打印模板")?;
    state.db.create_print_template(&req).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_print_template(state: State<AppState>, id: i64, name: Option<String>, content: Option<String>, paper_size: Option<String>, label_width_mm: Option<f64>, label_height_mm: Option<f64>, theme: Option<String>, restaurant_name: Option<String>, tagline: Option<String>, logo_data: Option<String>, show_price: Option<bool>, show_tax: Option<bool>, show_service_charge: Option<bool>, item_sort: Option<String>, modifiers_color: Option<String>, is_active: Option<bool>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "更新打印模板")?;
    state.db.update_print_template(id, name, content, paper_size, label_width_mm, label_height_mm, theme, restaurant_name, tagline, logo_data, show_price, show_tax, show_service_charge, item_sort, modifiers_color, is_active).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_print_template(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "删除打印模板")?;
    state.db.delete_print_template(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_default_template(state: State<AppState>, id: i64, template_type: String) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "设置默认打印模板")?;
    state.db.set_default_template(id, &template_type).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn render_template_preview(state: State<AppState>, template_id: i64, data: serde_json::Value) -> Result<PrintPreviewResult, String> {
    state.db.render_template_preview(template_id, &data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn render_template_content_preview(
    state: State<AppState>,
    content: String,
    paper_size: String,
    theme: String,
    restaurant_name: String,
    tagline: String,
    logo_data: Option<String>,
    data: serde_json::Value,
) -> Result<PrintPreviewResult, String> {
    state.db.render_template_content_preview(
        &content,
        &paper_size,
        &theme,
        &restaurant_name,
        &tagline,
        logo_data.as_deref(),
        &data,
    ).map_err(|e| e.to_string())
}

// ==================== 打印機命令 ====================

#[allow(dead_code)]
#[tauri::command]
pub fn bind_feie_printer(state: State<AppState>, printer_id: i64, printer_key: String) -> Result<String, String> {
    require_roles(&state, &[UserRole::Owner], "绑定飞鹅打印机")?;
    let printer = state.db.get_printers().map_err(|e| e.to_string())?
        .into_iter()
        .find(|p| p.id == printer_id)
        .ok_or(format!("打印機 #{} 不存在", printer_id))?;
    
    let user = printer.feie_user.as_deref().ok_or("飛鵝用戶未配置")?;
    let ukey = printer.feie_ukey.as_deref().ok_or("飛鵝 UKEY 未配置")?;
    let sn = printer.feie_sn.as_deref().ok_or("飛鵝 SN 未配置")?;
    
    printer::feie_add_printer(user, ukey, sn, &printer_key).map_err(print_err)
}

#[allow(dead_code)]
#[tauri::command]
pub fn check_printer_status(state: State<AppState>, printer_id: i64) -> Result<String, String> {
    let printer = state.db.get_printers().map_err(|e| e.to_string())?
        .into_iter()
        .find(|p| p.id == printer_id)
        .ok_or(format!("打印機 #{} 不存在", printer_id))?;
    
    match printer.connection_type.as_str() {
        "feie" => {
            let user = printer.feie_user.as_deref().ok_or("飛鵝用戶未配置")?;
            let ukey = printer.feie_ukey.as_deref().ok_or("飛鵝 UKEY 未配置")?;
            let sn = printer.feie_sn.as_deref().ok_or("飛鵝 SN 未配置")?;
            printer::feie_query_status(user, ukey, sn)
        }
        "lan" => {
            let ip = printer.lan_ip.as_deref().ok_or("局域網 IP 未配置")?;
            let port = printer.lan_port;
            printer::check_lan_printer_status(ip, port)
        }
        _ => Err(format!("不支持的連接類型"))
    }
}

#[tauri::command]
pub fn get_notifications(state: State<AppState>, limit: Option<i64>, unread_only: Option<bool>) -> Result<Vec<Notification>, String> {
    state.db.get_notifications(limit.unwrap_or(50), unread_only.unwrap_or(false)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_unread_notification_count(state: State<AppState>) -> Result<i64, String> {
    state.db.get_unread_count().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_notification_read(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier, UserRole::Chef, UserRole::Warehouse], "标记通知已读")?;
    state.db.mark_notification_read(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_all_notifications_read(state: State<AppState>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier, UserRole::Chef, UserRole::Warehouse], "全部通知已读")?;
    state.db.mark_all_notifications_read().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_notification(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "删除通知")?;
    state.db.delete_notification(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn check_and_create_alerts(state: State<AppState>) -> Result<(), String> {
    state.db.check_and_create_alerts().map_err(|e| e.to_string())
}

// ==================== 会员系统命令 ====================

#[tauri::command]
pub fn get_customers(state: State<AppState>, search: Option<String>) -> Result<Vec<Customer>, String> {
    state.db.get_customers(search.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_customer(state: State<AppState>, name: String, phone: Option<String>) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier], "创建顾客")?;
    state.db.create_customer(&name, phone.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_customer(state: State<AppState>, id: i64, name: Option<String>, phone: Option<String>, clear_phone: bool) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier], "更新顾客")?;
    let phone_param: Option<Option<&str>> = if clear_phone {
        Some(None)
    } else {
        phone.as_ref().map(|p| Some(p.as_str()))
    };
    state.db.update_customer(id, name.as_deref(), phone_param).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_customer(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier], "删除顾客")?;
    state.db.delete_customer(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_loyalty_txns(state: State<AppState>, customer_id: i64) -> Result<Vec<LoyaltyTxn>, String> {
    state.db.get_loyalty_txns(customer_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_loyalty_points(state: State<AppState>, customer_id: i64, order_id: Option<i64>, delta: i64, reason: String) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier], "调整积分")?;
    state.db.add_loyalty_points(customer_id, order_id, delta, &reason).map_err(|e| e.to_string())
}

// ==================== 优惠券命令 ====================

#[allow(dead_code)]
#[tauri::command]
pub fn get_coupons(state: State<AppState>) -> Result<Vec<Coupon>, String> {
    state.db.get_coupons().map_err(|e| e.to_string())
}

#[allow(dead_code)]
#[tauri::command]
pub fn create_coupon(state: State<AppState>, name: String, code: String, discount_percent: Option<f64>, discount_amount: Option<f64>, min_amount: Option<f64>, valid_from: Option<String>, valid_until: Option<String>) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner], "创建优惠券")?;
    state.db.create_coupon(&name, &code, discount_percent, discount_amount, min_amount, valid_from.as_deref(), valid_until.as_deref()).map_err(|e| e.to_string())
}

#[allow(dead_code)]
#[tauri::command]
pub fn use_coupon(state: State<AppState>, customer_id: i64, coupon_id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier], "使用优惠券")?;
    state.db.use_coupon(customer_id, coupon_id).map_err(|e| e.to_string())
}

// ==================== 门店命令 ====================

#[allow(dead_code)]
#[tauri::command]
pub fn get_stores(state: State<AppState>) -> Result<Vec<(i64, String, String)>, String> {
    state.db.get_stores().map_err(|e| e.to_string())
}

// ==================== 打印調試命令 ====================

use crate::printer::DebugPrintResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugKitchenTicketRequest {
    pub order_no: String,
    pub dine_type: String,
    pub items: Vec<(String, f64, Option<String>)>,
    pub note: Option<String>,
    pub filename: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugBatchLabelRequest {
    pub lot_no: String,
    pub material_name: String,
    pub quantity: f64,
    pub unit: String,
    pub expiry_date: Option<String>,
    pub supplier_name: Option<String>,
    pub filename: Option<String>,
}

#[tauri::command]
pub fn debug_print_kitchen_ticket(req: DebugKitchenTicketRequest) -> Result<DebugPrintResult, String> {
    let items: Vec<(String, f64, Option<String>)> = req.items;
    crate::printer::save_kitchen_ticket_to_file(
        &req.order_no,
        &req.dine_type,
        &items,
        req.note.as_deref(),
        req.filename.as_deref(),
    )
}

#[tauri::command]
pub fn debug_print_batch_label(req: DebugBatchLabelRequest) -> Result<DebugPrintResult, String> {
    crate::printer::save_batch_label_to_file(
        &req.lot_no,
        &req.material_name,
        req.quantity,
        &req.unit,
        req.expiry_date.as_deref(),
        req.supplier_name.as_deref(),
        req.filename.as_deref(),
    )
}

#[tauri::command]
pub fn debug_print_escpos(content: String, filename: Option<String>) -> Result<DebugPrintResult, String> {
    let mut builder = crate::printer::EscPosBuilder::new();
    builder.text_ln(&content);
    crate::printer::save_escpos_to_file(builder, filename.as_deref())
}

// ── 自動更新 ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_app_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
pub async fn check_for_update(app: tauri::AppHandle) -> Result<Option<crate::updater_check::UpdateInfo>, String> {
    let version = app.package_info().version.to_string();
    crate::updater_check::fetch_update(&version).await
}

#[tauri::command]
pub fn download_and_open_update(url: String, app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(target_os = "android")]
    { let _ = (url, app); return Err("Android 不支持此操作".to_string()); }
    #[cfg(not(target_os = "android"))]
    {
        require_roles(&app.state::<AppState>(), &[UserRole::Owner], "安装更新")?;
        if !url.starts_with("https://github.com/0xRyanlee/hyphen-cuckoo/releases/") {
            return Err("更新下载地址无效".to_string());
        }
        std::thread::spawn(move || {
            crate::updater_check::download_and_open(&url, app);
        });
        Ok(())
    }
}

// ==================== 局域网同步命令 ====================

#[tauri::command]
pub fn start_sync_server(
    state: State<AppState>,
    port: u16,
    shared_secret: String,
) -> Result<String, String> {
    require_roles(&state, &[UserRole::Owner], "启动同步服务")?;
    let shared_secret = shared_secret.trim().to_string();
    if shared_secret.is_empty() {
        return Err("同步密钥不能为空".to_string());
    }
    let mut guard = state.sync_server.lock().unwrap();
    if let Some(ref h) = *guard {
        if h.port == port {
            let ip = crate::sync_server::get_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());
            return Ok(format!("http://{}:{}", ip, port));
        }
        h.stop();
    }
    let handle = crate::sync_server::start_server(
        state.db_path.clone(),
        port,
        Some(shared_secret),
        SYNC_PROTOCOL_VERSION.to_string(),
    )?;
    *guard = Some(handle);
    let ip = crate::sync_server::get_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());
    Ok(format!("http://{}:{}", ip, port))
}

#[tauri::command]
pub fn stop_sync_server(state: State<AppState>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "停止同步服务")?;
    let mut guard = state.sync_server.lock().unwrap();
    if let Some(h) = guard.take() {
        h.stop();
    }
    Ok(())
}

#[tauri::command]
pub fn get_sync_server_status(state: State<AppState>) -> Result<Option<u16>, String> {
    let guard = state.sync_server.lock().unwrap();
    Ok(guard.as_ref().map(|h| h.port))
}

#[tauri::command]
pub fn get_local_ips() -> Result<Vec<String>, String> {
    Ok(crate::sync_server::get_local_ip().into_iter().collect())
}

#[tauri::command]
pub async fn fetch_sync_orders(
    server_url: String,
    since_epoch_s: i64,
    shared_secret: String,
    client_version: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let shared_secret = shared_secret.trim().to_string();
    if shared_secret.is_empty() {
        return Err("同步密钥不能为空".to_string());
    }
    let url = format!("{}/api/orders?since={}", server_url.trim_end_matches('/'), since_epoch_s);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;
    let protocol_version = client_version.unwrap_or_else(|| SYNC_PROTOCOL_VERSION.to_string());
    let text = client.get(&url)
        .header("X-Cuckoo-Sync-Token", shared_secret)
        .header("X-Cuckoo-Sync-Version", protocol_version)
        .send().await
        .map_err(|e| e.to_string())?
        .text().await
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| format!("解析失败: {}", e))
}

#[tauri::command]
pub async fn fetch_sync_tickets(
    server_url: String,
    shared_secret: String,
    client_version: Option<String>,
) -> Result<Vec<TicketWithItems>, String> {
    let shared_secret = shared_secret.trim().to_string();
    if shared_secret.is_empty() {
        return Err("同步密钥不能为空".to_string());
    }
    let url = format!("{}/api/tickets", server_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;
    let protocol_version = client_version.unwrap_or_else(|| SYNC_PROTOCOL_VERSION.to_string());
    let text = client.get(&url)
        .header("X-Cuckoo-Sync-Token", shared_secret)
        .header("X-Cuckoo-Sync-Version", protocol_version)
        .send().await
        .map_err(|e| e.to_string())?
        .text().await
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| format!("解析失败: {}", e))
}

#[tauri::command]
pub async fn mutate_sync_ticket(
    state: State<'_, AppState>,
    server_url: String,
    ticket_id: i64,
    action: String,
    shared_secret: String,
    client_version: Option<String>,
) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner, UserRole::Cashier, UserRole::Chef], "远程更新工单")?;
    let shared_secret = shared_secret.trim().to_string();
    if shared_secret.is_empty() {
        return Err("同步密钥不能为空".to_string());
    }
    let action = action.trim().to_string();
    if action != "start" && action != "finish" {
        return Err("不支持的工单动作".to_string());
    }
    let url = format!(
        "{}/api/tickets/{}/{}",
        server_url.trim_end_matches('/'),
        ticket_id,
        action
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;
    let protocol_version = client_version.unwrap_or_else(|| SYNC_PROTOCOL_VERSION.to_string());
    client.post(&url)
        .header("X-Cuckoo-Sync-Token", shared_secret)
        .header("X-Cuckoo-Sync-Version", protocol_version)
        .send().await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ==================== 餐桌管理 ====================

#[tauri::command]
pub fn get_restaurant_tables(state: State<AppState>) -> Result<Vec<RestaurantTable>, String> {
    state.db.get_restaurant_tables().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_restaurant_table(state: State<AppState>, table_no: String, label: Option<String>, is_active: bool, sort_no: i64) -> Result<i64, String> {
    require_roles(&state, &[UserRole::Owner], "创建餐桌")?;
    state.db.create_restaurant_table(&table_no, label.as_deref(), is_active, sort_no).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_restaurant_table(state: State<AppState>, id: i64, table_no: String, label: Option<String>, is_active: bool, sort_no: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "更新餐桌")?;
    state.db.update_restaurant_table(id, &table_no, label.as_deref(), is_active, sort_no).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_restaurant_table(state: State<AppState>, id: i64) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "删除餐桌")?;
    state.db.delete_restaurant_table(id).map_err(|e| e.to_string())
}

// ==================== 自助點單 ====================

#[tauri::command]
pub fn get_public_menu(state: State<AppState>) -> Result<Vec<PublicMenuCategory>, String> {
    state.db.get_public_menu().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_self_order(state: State<AppState>, table_no: String, items: Vec<SelfOrderItemInput>, token: Option<String>) -> Result<serde_json::Value, String> {
    let require_token = state.role_auth.lock().map(|a| a.require_token).unwrap_or(false);
    let table_no = resolve_self_order_table(&table_no, token.as_deref(), require_token)?;
    if items.is_empty() {
        return Err("订单不能为空".to_string());
    }
    let (order_id, order_no) = state.db.create_self_order(&table_no, &items).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "id": order_id, "order_no": order_no }))
}

/// Dual-mode table resolution. A valid signed token always wins (its bound
/// table_no overrides the client value). Without a token: rejected if
/// `require_token` (C2 hardened mode); otherwise grace-period fallback to the
/// raw table_no (legacy static stickers).
pub fn resolve_self_order_table(table_no: &str, token: Option<&str>, require_token: bool) -> Result<String, String> {
    if let Some(tok) = token.filter(|t| !t.is_empty()) {
        let payload = crate::qr_token::verify_token(tok).ok_or("二维码无效或已过期，请重新扫码")?;
        let bound = crate::qr_token::parse_table_payload(&payload).ok_or("二维码类型错误")?;
        return Ok(bound);
    }
    if require_token {
        return Err("请扫描餐桌上的最新二维码下单".to_string());
    }
    if table_no.trim().is_empty() {
        return Err("桌号不能为空".to_string());
    }
    Ok(table_no.trim().to_string())
}

#[tauri::command]
pub fn get_require_token(state: State<AppState>) -> Result<bool, String> {
    Ok(state.role_auth.lock().map(|a| a.require_token).unwrap_or(false))
}

#[tauri::command]
pub fn set_require_token(state: State<AppState>, enabled: bool) -> Result<(), String> {
    let mut auth = state.role_auth.lock().map_err(|_| "角色权限状态不可用".to_string())?;
    auth.require_token = enabled;
    save_role_auth_store(&state.role_auth_path, &auth)
}

#[tauri::command]
pub fn sign_table_token(table_no: String) -> Result<String, String> {
    if table_no.trim().is_empty() {
        return Err("桌号不能为空".to_string());
    }
    Ok(crate::qr_token::make_token(&crate::qr_token::table_payload(table_no.trim())))
}

#[tauri::command]
pub fn resolve_table_token(state: State<AppState>, token: String) -> Result<serde_json::Value, String> {
    match crate::qr_token::verify_token(&token).and_then(|p| crate::qr_token::parse_table_payload(&p)) {
        Some(table_no) => {
            let _ = state.db.record_qr_scan("table", Some(&table_no), None);
            Ok(serde_json::json!({ "valid": true, "table_no": table_no }))
        }
        None => Ok(serde_json::json!({ "valid": false })),
    }
}

#[tauri::command]
pub fn issue_marketing_qr_token(state: State<AppState>, order_id: i64, component: String, ch: String) -> Result<String, String> {
    state.db.issue_marketing_qr_token(order_id, &component, &ch).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn redeem_marketing_qr_token(state: State<AppState>, token: String, staff_name: Option<String>, pin: Option<String>) -> Result<serde_json::Value, String> {
    // PIN is enforced only at the public web endpoint (untrusted browser, prevents
    // customers self-redeeming). In-app invocation is already a trusted, role-gated
    // context, so it redeems PIN-free.
    let _ = pin;
    state.db.redeem_marketing_qr_token(&token, staff_name.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn redeem_discount_coupon(state: State<AppState>, code: String, amount: f64, staff_name: Option<String>, pin: Option<String>) -> Result<serde_json::Value, String> {
    // Same trust model as redeem_marketing_qr_token: in-app is role-gated, PIN only at web endpoint.
    let _ = pin;
    state.db.redeem_discount_coupon(code.trim(), amount, staff_name.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn redeem_requires_pin(state: State<AppState>) -> Result<bool, String> {
    let auth = state.role_auth.lock().map_err(|_| "角色权限状态不可用".to_string())?;
    Ok(!auth.pin_hashes.is_empty())
}

#[tauri::command]
pub fn get_marketing_funnel(state: State<AppState>, days: Option<i64>) -> Result<serde_json::Value, String> {
    state.db.get_marketing_funnel(days.unwrap_or(7)).map_err(|e| e.to_string())
}

// ── Campaigns (v3.2 方案B 扫码活动码得券) ──────────────────────────────────

#[tauri::command]
pub fn sign_campaign_token(campaign_id: i64) -> Result<String, String> {
    Ok(crate::qr_token::make_token(&crate::qr_token::campaign_payload(campaign_id)))
}

#[tauri::command]
pub fn create_campaign(state: State<AppState>, name: String, discount_type: String, discount_value: f64, condition_text: Option<String>, valid_days: i64, daily_limit: Option<i64>) -> Result<i64, String> {
    if name.trim().is_empty() {
        return Err("活动名称不能为空".to_string());
    }
    state.db.create_campaign(name.trim(), &discount_type, discount_value, condition_text.as_deref(), valid_days, daily_limit.unwrap_or(0)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_campaigns(state: State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    state.db.list_campaigns().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_campaign_cover(state: State<AppState>, id: i64, cover_image: Option<String>) -> Result<(), String> {
    require_roles(&state, &[UserRole::Owner], "更新活动封面")?;
    state.db.update_campaign_cover(id, cover_image.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_campaign_active(state: State<AppState>, id: i64, active: bool) -> Result<(), String> {
    state.db.set_campaign_active(id, active).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_campaign(state: State<AppState>, id: i64) -> Result<(), String> {
    state.db.delete_campaign(id).map_err(|e| e.to_string())
}

// ── 集字兑换闭环 (v3.3 4.0) ──────────────────────────────────────────────

#[tauri::command]
pub fn peek_marketing_qr_token(state: State<AppState>, token: String) -> Result<serde_json::Value, String> {
    state.db.peek_marketing_qr_token(&token).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn find_collect_token_by_order_no(state: State<AppState>, order_no: String) -> Result<serde_json::Value, String> {
    state.db.find_collect_token_by_order_no(order_no.trim()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn collect_redeem_set(state: State<AppState>, tokens: Vec<String>, staff_name: Option<String>) -> Result<serde_json::Value, String> {
    state.db.collect_redeem_set(&tokens, staff_name.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn resolve_campaign(state: State<AppState>, token: String) -> Result<serde_json::Value, String> {
    match crate::qr_token::verify_token(&token).and_then(|p| crate::qr_token::parse_campaign_payload(&p)) {
        Some(id) => {
            let _ = state.db.record_qr_scan("campaign", None, Some(id));
            state.db.issue_campaign_coupon(id).map_err(|e| e.to_string())
        }
        None => Ok(serde_json::json!({ "valid": false })),
    }
}

#[tauri::command]
pub fn get_marketing_popup(state: State<AppState>, order_id: i64, table_no: String) -> Result<serde_json::Value, String> {
    state.db.get_marketing_popup_content(order_id, &table_no).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn record_marketing_redemption(state: State<AppState>, order_id: i64, component_type: String, note: Option<String>, staff_name: Option<String>) -> Result<i64, String> {
    state.db.record_marketing_redemption(order_id, &component_type, note.as_deref(), staff_name.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_marketing_redemptions(state: State<AppState>, date: Option<String>) -> Result<Vec<serde_json::Value>, String> {
    state.db.get_marketing_redemptions(date.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_marketing_stats_today(state: State<AppState>) -> Result<serde_json::Value, String> {
    state.db.get_marketing_stats_today().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn redeem_coupon(state: State<AppState>, order_id: i64, staff_name: Option<String>) -> Result<bool, String> {
    state.db.redeem_coupon(order_id, staff_name.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_table_orders_today(state: State<AppState>, table_no: String) -> Result<Vec<TableOrderSummary>, String> {
    state.db.get_table_orders_today(&table_no).map_err(|e| e.to_string())
}

// ==================== Web 伺服器（LAN iPad 存取）====================

#[derive(Serialize)]
pub struct WebServerStatus {
    pub running: bool,
    pub port: Option<u16>,
    pub url: Option<String>,
}

#[tauri::command]
pub fn get_web_server_status(state: State<AppState>) -> WebServerStatus {
    let guard = state.web_server.lock().unwrap();
    match &*guard {
        Some(handle) => {
            let ip = crate::sync_server::get_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());
            WebServerStatus {
                running: true,
                port: Some(handle.port),
                url: Some(format!("http://{}:{}", ip, handle.port)),
            }
        }
        None => WebServerStatus {
            running: false,
            port: None,
            url: None,
        },
    }
}

#[tauri::command]
pub fn stop_web_server(state: State<AppState>) -> Result<(), String> {
    let mut guard = state.web_server.lock().unwrap();
    if let Some(handle) = guard.take() {
        handle.stop();
    }
    Ok(())
}

#[tauri::command]
pub fn restart_web_server(state: State<AppState>, app: tauri::AppHandle) -> Result<(), String> {
    {
        let mut guard = state.web_server.lock().unwrap();
        if let Some(handle) = guard.take() {
            handle.stop();
        }
    }
    let resource_dist = app.path().resource_dir().ok().map(|r| r.join("dist"));
    let exe_dist = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.to_path_buf()))
        .map(|p| p.join("dist"));
    let cwd_dist = std::env::current_dir().ok().map(|d| d.join("dist"));
    let dist_dir = [resource_dist, exe_dist, cwd_dist]
        .into_iter()
        .flatten()
        .find(|p| p.join("index.html").exists());
    let role_auth_path = state.role_auth_path.clone();
    let db = state.db.clone();
    match crate::web_server::start_web_server(db, dist_dir, role_auth_path, 9001) {
        Ok(handle) => {
            *state.web_server.lock().unwrap() = Some(handle);
            Ok(())
        }
        Err(e) => Err(format!("啟動失敗: {e}")),
    }
}

fn encode_base64(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((n >> 18) & 63) as usize] as char);
        out.push(CHARS[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 { CHARS[((n >> 6) & 63) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { CHARS[(n & 63) as usize] as char } else { '=' });
    }
    out
}

fn mime_from_ext(path: &str) -> &'static str {
    let lower = path.to_lowercase();
    if lower.ends_with(".jpg") || lower.ends_with(".jpeg") { "image/jpeg" }
    else if lower.ends_with(".png") { "image/png" }
    else if lower.ends_with(".gif") { "image/gif" }
    else if lower.ends_with(".webp") { "image/webp" }
    else if lower.ends_with(".svg") { "image/svg+xml" }
    else { "image/jpeg" }
}

#[tauri::command]
pub async fn load_image_as_data_url(source: String) -> Result<String, String> {
    if source.starts_with("http://") || source.starts_with("https://") {
        let url = source.clone();
        let (mime, data) = tauri::async_runtime::spawn_blocking(move || -> Result<(String, Vec<u8>), String> {
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .map_err(|e| e.to_string())?;
            let resp = client.get(&url).send().map_err(|e| e.to_string())?;
            let mime = resp.headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .and_then(|ct| ct.split(';').next())
                .unwrap_or("image/jpeg")
                .to_string();
            let bytes = resp.bytes().map_err(|e| e.to_string())?;
            Ok((mime, bytes.to_vec()))
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e)?;
        Ok(format!("data:{};base64,{}", mime, encode_base64(&data)))
    } else {
        let data = std::fs::read(&source).map_err(|e| e.to_string())?;
        let mime = mime_from_ext(&source);
        Ok(format!("data:{};base64,{}", mime, encode_base64(&data)))
    }
}
