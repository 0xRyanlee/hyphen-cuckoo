mod database;
mod commands;
mod printer;
mod updater_check;
mod sync_server;
mod web_server;
mod qr_token;

use commands::AppState;
use database::Database;
use tauri::Manager;
use std::sync::Arc;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::panic;

/// Platform-correct, writable app data dir. Android MUST use Tauri's path API
/// (returns the app sandbox `/data/data/<pkg>/files`); `dirs` returns None on
/// Android → previously fell back to "." (unwritable) → create_dir_all().expect()
/// panic → with panic=abort = instant crash on launch. Desktop keeps `dirs` so
/// existing users' data path is unchanged.
fn resolve_data_dir(app: &tauri::App) -> PathBuf {
    #[cfg(target_os = "android")]
    {
        use tauri::Manager;
        app.path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("/data/local/tmp"))
            .join("Cuckoo")
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = app;
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Cuckoo")
    }
}

fn install_crash_log_hook(log_dir: PathBuf) {
    let start_time = std::time::Instant::now();
    panic::set_hook(Box::new(move |panic_info| {
        let elapsed = start_time.elapsed().as_secs();
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let message = format!(
            "[{}] PANIC after {}s: {}\nLocation: {:?}\n\n",
            timestamp, elapsed, panic_info, panic_info.location()
        );
        let _ = fs::create_dir_all(&log_dir);
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_dir.join("crash.log")) {
            let _ = file.write_all(message.as_bytes());
        }
        eprintln!("{}", message);
    }));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    eprintln!("[Cuckoo] Starting application...");
    // Early hook: stderr only (visible in Android logcat) until the data dir is known.
    panic::set_hook(Box::new(|info| eprintln!("[Cuckoo] PANIC: {}", info)));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(move |app| {
            // All filesystem init happens here (app handle available → Android-safe path).
            let data_dir = resolve_data_dir(app);
            if let Err(e) = fs::create_dir_all(&data_dir) {
                eprintln!("[Cuckoo] FATAL: cannot create data dir {:?}: {}", data_dir, e);
            }
            let log_dir = data_dir.join("logs");
            install_crash_log_hook(log_dir);

            // Persist the QR HMAC secret under the same dir (Android-safe).
            qr_token::set_secret_dir(data_dir.clone());

            let db_path = data_dir.join("cuckoo.db");
            eprintln!("[Cuckoo] Database path: {:?}", db_path);
            let db = match Database::new(db_path.to_str().unwrap_or("cuckoo.db")) {
                Ok(d) => { eprintln!("[Cuckoo] Database created successfully"); Arc::new(d) }
                Err(e) => {
                    eprintln!("[Cuckoo] Database error: {}", e);
                    return Err(Box::new(e) as Box<dyn std::error::Error>);
                }
            };
            let role_auth_path = data_dir.join("role-auth.json");
            let role_auth = commands::load_role_auth_store(&role_auth_path);

            app.manage(AppState {
                db: db.clone(),
                db_path: db_path.clone(),
                sync_server: std::sync::Mutex::new(None),
                web_server: std::sync::Mutex::new(None),
                role_auth: std::sync::Mutex::new(role_auth),
                role_auth_path: role_auth_path.clone(),
            });

            // Locate the frontend dist — several candidates so it works in dev,
            // the desktop bundle, and the Android asset dir.
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

            if let Some(ref d) = dist_dir {
                eprintln!("[WebServer] Serving frontend from {:?}", d);
            } else {
                eprintln!("[WebServer] No dist/ found — API-only mode");
            }

            match web_server::start_web_server(db.clone(), dist_dir, role_auth_path, 9001) {
                Ok(handle) => {
                    let ip = sync_server::get_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());
                    eprintln!("[WebServer] Started — http://{}:{}", ip, handle.port);
                    *app.state::<AppState>().web_server.lock().unwrap() = Some(handle);
                }
                Err(e) => eprintln!("[WebServer] Failed to start: {}", e),
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 健康檢查
            commands::health_check,
            commands::backup_database,
            commands::check_expiry_alerts,
            commands::report_telemetry,
            // 角色權限
            commands::get_current_role,
            commands::get_role_pin_statuses,
            commands::set_role_pin,
            commands::switch_role,
            // 單位
            commands::get_units,
            // 材料分類
            commands::get_material_categories,
            commands::create_material_category,
            // 標籤
            commands::get_tags,
            commands::create_tag,
            // 材料
            commands::get_materials,
            commands::create_material,
            commands::add_material_tags,
            // 材料狀態
            commands::get_material_states,
            commands::create_material_state,
            commands::get_all_material_states,
            commands::update_material_state,
            commands::delete_material_state,
            // 供應商
            commands::get_suppliers,
            commands::create_supplier,
            // 屬性模板
            commands::get_attribute_templates,
            commands::create_attribute_template,
            commands::update_attribute_template,
            commands::delete_attribute_template,
            commands::set_entity_attribute,
            commands::get_entity_attributes,
            // 配方
            commands::get_recipes,
            commands::get_recipe_types,
            commands::get_recipe_with_items,
            commands::generate_recipe_code,
            commands::seed_sample_recipes,
            commands::create_recipe,
            commands::create_recipe_type,
            commands::get_recipe_component_types,
            commands::get_order_component_types,
            commands::add_recipe_item,
            commands::calculate_recipe_cost,
            commands::get_recipe_item_counts,
            commands::get_all_recipe_costs,
            commands::get_recipe_usage_count,
            commands::get_recipe_dependents,
            commands::get_material_dependents,
            // 菜單
            commands::get_menu_categories,
            commands::create_menu_category,
            commands::get_menu_items,
            commands::create_menu_item,
            commands::get_menu_item_specs,
            commands::create_menu_item_spec,
            commands::update_menu_item_spec,
            commands::delete_menu_item_spec,
            commands::get_menu_items_for_pos,
            // 訂單
            commands::create_order,
            commands::get_orders,
            commands::get_order_with_items,
            commands::add_order_item,
            commands::submit_order,
            commands::cancel_order,
            commands::mark_order_ready,
            commands::update_order_payment,
            commands::record_order_refund,
            commands::refund_order_item,
            commands::get_sales_by_hour,
            commands::get_sales_by_weekday,
            commands::batch_cancel_orders,
            // KDS
            commands::get_kitchen_stations,
            commands::update_station_printer,
            commands::get_station_tickets,
            commands::get_all_tickets,
            commands::get_all_tickets_with_items,
            commands::get_tickets_for_order,
            commands::start_ticket,
            commands::finish_ticket,
            // 庫存
            commands::get_inventory_batches,
            commands::get_inventory_summary,
            commands::create_inventory_txn,
            commands::get_inventory_txns,
            commands::create_inventory_batch,
            commands::delete_inventory_batch,
            commands::adjust_inventory,
            commands::record_wastage,
            // 更新/刪除
            commands::update_material,
            commands::delete_material,
            commands::update_material_category,
            commands::delete_material_category,
            commands::update_tag,
            commands::delete_tag,
            commands::remove_material_tag,
            commands::update_supplier,
            commands::delete_supplier,
            // 日常支出
            commands::get_expenses,
            commands::create_expense,
            commands::update_expense,
            commands::delete_expense,
            // 供應商商品
            commands::get_supplier_products,
            commands::create_supplier_product,
            commands::update_supplier_product,
            commands::delete_supplier_product,
            commands::get_order_cost,
            commands::update_menu_item,
            commands::set_menu_item_availability,
            commands::batch_set_menu_item_availability,
            commands::batch_update_menu_item_prices,
            commands::toggle_menu_item_favorite,
            commands::delete_menu_item,
            commands::update_menu_category,
            commands::delete_menu_category,
            commands::update_recipe,
            commands::update_recipe_type,
            commands::delete_recipe_type,
            commands::delete_recipe,
            commands::delete_recipe_item,
            commands::add_station_menu_item,
            commands::remove_station_menu_item,
            // 打印機
            commands::get_printers,
            commands::get_default_printer,
            commands::create_printer,
            commands::update_printer,
            commands::delete_printer,
            commands::scan_lan_printers,
            commands::test_feie_printer,
            commands::test_lan_printer,
            commands::bind_feie_printer,
            commands::get_app_version,
            commands::check_for_update,
            commands::download_and_open_update,
            commands::send_print_task,
            commands::print_kitchen_ticket,
            commands::print_order_receipt,
            commands::print_batch_label,
            commands::get_print_tasks,
            // 採購單
            commands::get_purchase_orders,
            commands::get_purchase_order_with_items,
            commands::create_purchase_order,
            commands::add_purchase_order_item,
            commands::update_purchase_order_status,
            commands::delete_purchase_order,
            commands::receive_purchase_order,
            commands::receive_purchase_order_items,
            // 生產單
            commands::get_production_orders,
            commands::get_production_order_with_items,
            commands::create_production_order,
            commands::start_production_order,
            commands::complete_production_order,
            commands::delete_production_order,
            commands::check_production_materials,
            // 盤點
            commands::get_stocktakes,
            commands::get_stocktake_with_items,
            commands::create_stocktake,
            commands::update_stocktake_item,
            commands::complete_stocktake,
            commands::delete_stocktake,
            // 加料/去料
            commands::add_order_item_modifier,
            commands::get_order_item_modifiers,
            commands::delete_order_item_modifier,
            // 報表
            commands::get_sales_report,
            commands::get_sales_by_category,
            commands::get_gross_profit_report,
            commands::get_top_selling_items,
            commands::get_material_consumption_report,
            // 配方項目更新
            commands::update_recipe_item,
            commands::would_create_recipe_cycle,
            // 工作站菜單映射
            commands::get_station_menu_items,
            // 打印模板
            commands::get_print_templates,
            commands::get_print_template,
            commands::create_print_template,
            commands::update_print_template,
            commands::delete_print_template,
            commands::set_default_template,
            commands::render_template_preview,
            commands::render_template_content_preview,
            // 票據類型
            commands::get_print_ticket_types,
            commands::get_print_ticket_type,
            commands::create_print_ticket_type,
            commands::update_print_ticket_type,
            commands::delete_print_ticket_type,
            commands::set_default_ticket_type,
            commands::ensure_default_ticket_types,
            // 通知系统
            commands::get_notifications,
            commands::get_unread_notification_count,
            commands::mark_notification_read,
            commands::mark_all_notifications_read,
            commands::delete_notification,
            commands::check_and_create_alerts,
            // 会员积分
            commands::get_customers,
            commands::create_customer,
            commands::update_customer,
            commands::delete_customer,
            commands::get_loyalty_txns,
            commands::add_loyalty_points,
            // 局域网同步
            commands::start_sync_server,
            commands::stop_sync_server,
            commands::get_sync_server_status,
            commands::get_local_ips,
            commands::fetch_sync_orders,
            commands::fetch_sync_tickets,
            commands::mutate_sync_ticket,
            // 打印調試
            commands::debug_print_kitchen_ticket,
            commands::debug_print_batch_label,
            commands::debug_print_escpos,
            // 餐桌管理
            commands::get_restaurant_tables,
            commands::create_restaurant_table,
            commands::update_restaurant_table,
            commands::delete_restaurant_table,
            // 自助點單
            commands::get_public_menu,
            commands::create_self_order,
            commands::get_table_orders_today,
            commands::get_marketing_popup,
            commands::record_marketing_redemption,
            commands::get_marketing_redemptions,
            commands::get_marketing_stats_today,
            commands::sign_table_token,
            commands::resolve_table_token,
            commands::issue_marketing_qr_token,
            commands::redeem_marketing_qr_token,
            commands::redeem_requires_pin,
            commands::get_marketing_funnel,
            commands::sign_campaign_token,
            commands::create_campaign,
            commands::list_campaigns,
            commands::set_campaign_active,
            commands::delete_campaign,
            commands::resolve_campaign,
            commands::peek_marketing_qr_token,
            commands::find_collect_token_by_order_no,
            commands::collect_redeem_set,
            commands::get_require_token,
            commands::set_require_token,
            commands::redeem_coupon,
            // Web 伺服器
            commands::get_web_server_status,
            commands::stop_web_server,
            commands::restart_web_server,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
