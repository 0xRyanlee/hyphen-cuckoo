use tauri::{State, Manager};
use crate::database::{Database, MaterialWithTags, Unit, MaterialCategory, Tag, MaterialState, Supplier, Expense, SupplierProduct, Recipe, RecipeWithItems, RecipeCostResult, RecipeType, MenuItem, MenuCategory, Order, OrderItem, OrderItemModifier, KitchenStation, KitchenTicket, InventoryBatch, InventorySummary, AttributeTemplate, EntityAttribute, InventoryTxn, MenuItemSpec, PrinterConfig, PrintTask, PurchaseOrder, PurchaseOrderWithItems, ProductionOrder, ProductionOrderWithItems, Stocktake, StocktakeWithItems, Notification, Customer, Coupon, RecipeComponentType, OrderComponentType};
use crate::printer::{self, EscPosBuilder, LanPrinter, scan_lan_printers as LAN_SCAN};
use serde::{Deserialize, Serialize};

pub struct AppState {
    pub db: Database,
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
    state.db.create_tag(&req.code, &req.name, req.color.as_deref()).map_err(db_err)
}

// ==================== 材料 API ====================

#[tauri::command]
pub fn get_materials(state: State<AppState>, category_id: Option<i64>) -> Result<Vec<MaterialWithTags>, String> {
    state.db.get_materials(category_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_material(state: State<AppState>, req: CreateMaterialRequest) -> Result<i64, String> {
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
    state.db.add_material_tags(material_id, &tag_ids).map_err(|e| e.to_string())
}

// ==================== 材料狀態 API ====================

#[tauri::command]
pub fn get_material_states(state: State<AppState>, material_id: i64) -> Result<Vec<MaterialState>, String> {
    state.db.get_material_states(material_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_material_state(state: State<AppState>, req: CreateMaterialStateRequest) -> Result<i64, String> {
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
    state.db.update_material_state(id, state_code.as_deref(), state_name.as_deref(), unit_id, yield_rate, cost_multiplier).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_material_state(state: State<AppState>, id: i64) -> Result<(), String> {
    state.db.delete_material_state(id).map_err(|e| e.to_string())
}

// ==================== 供應商 API ====================

#[tauri::command]
pub fn get_suppliers(state: State<AppState>) -> Result<Vec<Supplier>, String> {
    state.db.get_suppliers().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_supplier(state: State<AppState>, req: CreateSupplierRequest) -> Result<i64, String> {
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
    state.db.create_expense(&req.expense_type, req.amount, &req.expense_date, req.note.as_deref(), req.operator.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_expense(state: State<AppState>, id: i64, expense_type: Option<String>, amount: Option<f64>, expense_date: Option<String>, note: Option<String>) -> Result<(), String> {
    state.db.update_expense(id, expense_type.as_deref(), amount, expense_date.as_deref(), note.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_expense(state: State<AppState>, id: i64) -> Result<(), String> {
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
    state.db.update_supplier_product(req.id, &req.product_name, &req.supplier_name, &req.channel).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_supplier_product(state: State<AppState>, id: i64) -> Result<(), String> {
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
    state.db.seed_sample_recipes().map_err(|e| e.to_string())?;
    Ok("示例配方已创建".to_string())
}

#[tauri::command]
pub fn create_recipe(state: State<AppState>, req: CreateRecipeRequest) -> Result<i64, String> {
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
    state.db.create_menu_category(&name, sort_no.unwrap_or(0)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_menu_items(state: State<AppState>, category_id: Option<i64>) -> Result<Vec<MenuItem>, String> {
    state.db.get_menu_items(category_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_menu_item(state: State<AppState>, req: CreateMenuItemRequest) -> Result<i64, String> {
    state.db.create_menu_item(&req.name, req.category_id, req.recipe_id, req.sales_price).map_err(db_err)
}

#[tauri::command]
pub fn get_menu_item_specs(state: State<AppState>, menu_item_id: i64) -> Result<Vec<MenuItemSpec>, String> {
    state.db.get_menu_item_specs(menu_item_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_menu_item_spec(state: State<AppState>, req: CreateMenuItemSpecRequest) -> Result<i64, String> {
    state.db.create_menu_item_spec(req.menu_item_id, &req.spec_code, &req.spec_name, req.price_delta, req.qty_multiplier, 0).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_menu_item_spec(state: State<AppState>, id: i64, spec_code: Option<String>, spec_name: Option<String>, price_delta: Option<f64>, qty_multiplier: Option<f64>, sort_no: Option<i32>) -> Result<(), String> {
    state.db.update_menu_item_spec(id, spec_code.as_deref(), spec_name.as_deref(), price_delta, qty_multiplier, sort_no).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_menu_item_spec(state: State<AppState>, id: i64) -> Result<(), String> {
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
    state.db.submit_order_full(order_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cancel_order(state: State<AppState>, order_id: i64, is_served: bool, reason: Option<String>) -> Result<Vec<String>, String> {
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
    state.db.update_order_payment(req.order_id, &req.payment_status, req.payment_method.as_deref(), req.amount_paid)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn record_order_refund(state: State<AppState>, order_id: i64, refund_amount: f64) -> Result<(), String> {
    state.db.record_order_refund(order_id, refund_amount).map_err(|e| e.to_string())
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
    let mapped: Vec<(i64, f64, Option<String>)> = items.into_iter().map(|r| (r.item_id, r.received_qty, r.lot_no)).collect();
    state.db.receive_purchase_order_items(po_id, mapped, operator.as_deref())
        .map(|details| details.into_iter().map(|(_, lot, name, _, _, _)| format!("{}: {}", name, lot)).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn batch_cancel_orders(state: State<AppState>, ids: Vec<i64>) -> Result<usize, String> {
    state.db.batch_cancel_orders(&ids).map_err(|e| e.to_string())
}

// ==================== KDS API ====================

#[tauri::command]
pub fn get_kitchen_stations(state: State<AppState>) -> Result<Vec<KitchenStation>, String> {
    state.db.get_kitchen_stations().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_station_printer(state: State<AppState>, station_id: i64, printer_id: Option<i64>) -> Result<(), String> {
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
    state.db.start_ticket(ticket_id, operator.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn finish_ticket(state: State<AppState>, ticket_id: i64, operator: Option<String>) -> Result<(), String> {
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
    state.db.delete_inventory_batch(batch_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn adjust_inventory(state: State<AppState>, req: AdjustInventoryRequest) -> Result<(), String> {
    state.db.adjust_inventory(req.lot_id, req.qty_delta, req.operator.as_deref(), req.note.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn record_wastage(state: State<AppState>, req: RecordWastageRequest) -> Result<(), String> {
    state.db.record_wastage(req.lot_id, req.qty, Some(&req.wastage_type), req.operator.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_material(state: State<AppState>, id: i64, name: Option<String>, category_id: Option<i64>, shelf_life_days: Option<i32>, min_qty: Option<f64>) -> Result<(), String> {
    state.db.update_material(id, name.as_deref(), category_id, shelf_life_days, min_qty).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_material(state: State<AppState>, id: i64) -> Result<(), String> {
    state.db.delete_material(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_material_category(state: State<AppState>, id: i64, name: String, sort_no: i32) -> Result<(), String> {
    state.db.update_material_category(id, &name, sort_no).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_material_category(state: State<AppState>, id: i64) -> Result<(), String> {
    state.db.delete_material_category(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_tag(state: State<AppState>, id: i64, name: String, color: Option<String>) -> Result<(), String> {
    state.db.update_tag(id, &name, color.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_tag(state: State<AppState>, id: i64) -> Result<(), String> {
    state.db.delete_tag(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_material_tag(state: State<AppState>, material_id: i64, tag_id: i64) -> Result<(), String> {
    state.db.remove_material_tag(material_id, tag_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_supplier(state: State<AppState>, id: i64, name: Option<String>, phone: Option<String>, contact_person: Option<String>, address: Option<String>, note: Option<String>) -> Result<(), String> {
    state.db.update_supplier(id, name.as_deref(), phone.as_deref(), contact_person.as_deref(), address.as_deref(), note.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_supplier(state: State<AppState>, id: i64) -> Result<(), String> {
    state.db.delete_supplier(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_menu_item(state: State<AppState>, id: i64, name: Option<String>, category_id: Option<i64>, recipe_id: Option<i64>, sales_price: Option<f64>) -> Result<(), String> {
    state.db.update_menu_item(id, name.as_deref(), category_id, recipe_id, sales_price).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_menu_item_availability(state: State<AppState>, id: i64, is_available: bool) -> Result<bool, String> {
    state.db.set_menu_item_availability(id, is_available).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn batch_toggle_menu_item_availability(state: State<AppState>, ids: Vec<i64>, is_available: bool) -> Result<usize, String> {
    state.db.batch_toggle_menu_item_availability(&ids, is_available).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_menu_item(state: State<AppState>, id: i64) -> Result<(), String> {
    state.db.delete_menu_item(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_menu_category(state: State<AppState>, id: i64, name: String, sort_no: i32) -> Result<(), String> {
    state.db.update_menu_category(id, Some(&name), Some(sort_no)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_menu_category(state: State<AppState>, id: i64) -> Result<(), String> {
    state.db.delete_menu_category(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_recipe(state: State<AppState>, id: i64, name: Option<String>, recipe_type: Option<String>, output_qty: Option<f64>) -> Result<(), String> {
    state.db.update_recipe(id, name.as_deref(), recipe_type.as_deref(), output_qty, None).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_recipe_type(state: State<AppState>, id: i64, code: String, name: String, description: Option<String>, sort_no: i32) -> Result<(), String> {
    state.db.update_recipe_type(id, &code, &name, description.as_deref(), sort_no).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_recipe_type(state: State<AppState>, id: i64) -> Result<(), String> {
    state.db.delete_recipe_type(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_recipe(state: State<AppState>, id: i64) -> Result<(), String> {
    state.db.delete_recipe(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_recipe_item(state: State<AppState>, item_id: i64) -> Result<(), String> {
    state.db.delete_recipe_item(item_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_station_menu_item(state: State<AppState>, station_id: i64, menu_item_id: i64) -> Result<i64, String> {
    state.db.add_station_menu_item(station_id, menu_item_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_station_menu_item(state: State<AppState>, station_id: i64, menu_item_id: i64) -> Result<(), String> {
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
    let backup_dir = match dest_dir {
        Some(d) => std::path::PathBuf::from(d),
        None => dirs::document_dir()
            .unwrap_or_else(|| dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from(".")))
            .join("Cuckoo 备份"),
    };
    std::fs::create_dir_all(&backup_dir).map_err(|e| format!("创建备份目录失败: {}", e))?;
    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let dest = backup_dir.join(format!("cuckoo_{}.db", ts));
    state.db.backup_to(dest.to_str().ok_or("路径含非法字符")?)
        .map_err(|e| format!("备份失败: {}", e))?;
    Ok(dest.to_string_lossy().to_string())
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
    state.db.add_purchase_order_item(req.po_id, req.material_id, req.qty, req.unit_id, req.cost_per_unit).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_purchase_order_status(state: State<AppState>, po_id: i64, status: String) -> Result<(), String> {
    state.db.update_purchase_order_status(po_id, &status).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_purchase_order(state: State<AppState>, po_id: i64) -> Result<(), String> {
    state.db.delete_purchase_order(po_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn receive_purchase_order(state: State<AppState>, app: tauri::AppHandle, po_id: i64, operator: Option<String>, auto_print: Option<bool>) -> Result<Vec<i64>, String> {
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
    state.db.create_production_order(recipe_id, planned_qty, operator.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_production_order(state: State<AppState>, production_id: i64, operator: Option<String>) -> Result<(), String> {
    state.db.start_production_order(production_id, operator.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn complete_production_order(state: State<AppState>, production_id: i64, actual_qty: f64, operator: Option<String>) -> Result<(), String> {
    state.db.complete_production_order(production_id, actual_qty, operator.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_production_order(state: State<AppState>, production_id: i64) -> Result<(), String> {
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
    state.db.create_stocktake(operator.as_deref(), note.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_stocktake_item(state: State<AppState>, item_id: i64, actual_qty: f64) -> Result<(), String> {
    state.db.update_stocktake_item(item_id, actual_qty).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn complete_stocktake(state: State<AppState>, stocktake_id: i64, operator: Option<String>) -> Result<(), String> {
    state.db.complete_stocktake(stocktake_id, operator.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_stocktake(state: State<AppState>, stocktake_id: i64) -> Result<(), String> {
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
    state.db.add_order_item_modifier(req.order_item_id, &req.modifier_type, req.material_id, req.qty, req.price_delta).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_order_item_modifiers(state: State<AppState>, order_item_id: i64) -> Result<Vec<OrderItemModifier>, String> {
    state.db.get_order_item_modifiers(order_item_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_order_item_modifier(state: State<AppState>, modifier_id: i64) -> Result<(), String> {
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
    state.db.create_print_ticket_type(&req).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_print_ticket_type(state: State<AppState>, id: i64, req: UpdatePrintTicketTypeRequest) -> Result<(), String> {
    state.db.update_print_ticket_type(id, &req).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_print_ticket_type(state: State<AppState>, id: i64) -> Result<(), String> {
    state.db.delete_print_ticket_type(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_default_ticket_type(state: State<AppState>, id: i64) -> Result<(), String> {
    state.db.set_default_ticket_type(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ensure_default_ticket_types(state: State<AppState>) -> Result<(), String> {
    state.db.ensure_default_ticket_types().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_print_template(state: State<AppState>, id: i64) -> Result<PrintTemplate, String> {
    state.db.get_print_template(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_print_template(state: State<AppState>, req: CreatePrintTemplateRequest) -> Result<i64, String> {
    state.db.create_print_template(&req).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_print_template(state: State<AppState>, id: i64, name: Option<String>, content: Option<String>, paper_size: Option<String>, label_width_mm: Option<f64>, label_height_mm: Option<f64>, theme: Option<String>, restaurant_name: Option<String>, tagline: Option<String>, logo_data: Option<String>, show_price: Option<bool>, show_tax: Option<bool>, show_service_charge: Option<bool>, item_sort: Option<String>, modifiers_color: Option<String>, is_active: Option<bool>) -> Result<(), String> {
    state.db.update_print_template(id, name, content, paper_size, label_width_mm, label_height_mm, theme, restaurant_name, tagline, logo_data, show_price, show_tax, show_service_charge, item_sort, modifiers_color, is_active).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_print_template(state: State<AppState>, id: i64) -> Result<(), String> {
    state.db.delete_print_template(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_default_template(state: State<AppState>, id: i64, template_type: String) -> Result<(), String> {
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
    state.db.mark_notification_read(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mark_all_notifications_read(state: State<AppState>) -> Result<(), String> {
    state.db.mark_all_notifications_read().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_notification(state: State<AppState>, id: i64) -> Result<(), String> {
    state.db.delete_notification(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn check_and_create_alerts(state: State<AppState>) -> Result<(), String> {
    state.db.check_and_create_alerts().map_err(|e| e.to_string())
}

// ==================== 会员系统命令 ====================

#[allow(dead_code)]
#[tauri::command]
pub fn get_customers(state: State<AppState>) -> Result<Vec<Customer>, String> {
    state.db.get_customers().map_err(|e| e.to_string())
}

#[allow(dead_code)]
#[tauri::command]
pub fn create_customer(state: State<AppState>, name: Option<String>, phone: Option<String>, wechat_openid: Option<String>) -> Result<i64, String> {
    state.db.create_customer(name.as_deref(), phone.as_deref(), wechat_openid.as_deref()).map_err(|e| e.to_string())
}

#[allow(dead_code)]
#[tauri::command]
pub fn update_customer_points(state: State<AppState>, customer_id: i64, points_delta: i64) -> Result<(), String> {
    state.db.update_customer_points(customer_id, points_delta).map_err(|e| e.to_string())
}

#[allow(dead_code)]
#[tauri::command]
pub fn update_customer_balance(state: State<AppState>, customer_id: i64, balance_delta: f64) -> Result<(), String> {
    state.db.update_customer_balance(customer_id, balance_delta).map_err(|e| e.to_string())
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
    state.db.create_coupon(&name, &code, discount_percent, discount_amount, min_amount, valid_from.as_deref(), valid_until.as_deref()).map_err(|e| e.to_string())
}

#[allow(dead_code)]
#[tauri::command]
pub fn use_coupon(state: State<AppState>, customer_id: i64, coupon_id: i64) -> Result<(), String> {
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
    if !url.starts_with("https://github.com/0xRyanlee/Cuckoo/releases/") {
        return Err("更新下载地址无效".to_string());
    }
    std::thread::spawn(move || {
        crate::updater_check::download_and_open(&url, app);
    });
    Ok(())
}
