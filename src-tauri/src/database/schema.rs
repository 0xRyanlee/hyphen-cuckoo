use rusqlite::{params, Result};
use super::*;

impl Database {
    pub(super) fn init_tables(&self) -> Result<()> {
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

            CREATE TABLE IF NOT EXISTS loyalty_txns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                customer_id INTEGER NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
                order_id INTEGER REFERENCES orders(id) ON DELETE SET NULL,
                delta INTEGER NOT NULL,
                reason TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
            );

            CREATE INDEX IF NOT EXISTS idx_loyalty_txns_customer ON loyalty_txns(customer_id);

            CREATE TABLE IF NOT EXISTS restaurant_tables (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                table_no TEXT NOT NULL UNIQUE,
                label TEXT,
                is_active INTEGER NOT NULL DEFAULT 1,
                sort_no INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
            );

            CREATE TABLE IF NOT EXISTS issued_coupons (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                order_id INTEGER NOT NULL,
                code TEXT NOT NULL,
                discount_type TEXT NOT NULL DEFAULT 'percent',
                discount_value REAL NOT NULL DEFAULT 0,
                condition_text TEXT,
                valid_until TEXT NOT NULL,
                redeemed_at TEXT,
                redeemed_by TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                UNIQUE(order_id, code)
            );

            CREATE TABLE IF NOT EXISTS marketing_redemptions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                order_id INTEGER NOT NULL,
                component_type TEXT NOT NULL,
                note TEXT,
                staff_name TEXT,
                amount REAL NOT NULL DEFAULT 0,
                redeemed_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
            );

            CREATE TABLE IF NOT EXISTS marketing_qr_tokens (
                token TEXT PRIMARY KEY,
                order_id INTEGER NOT NULL,
                component TEXT NOT NULL,
                ch TEXT,
                redeemed_at TEXT,
                void INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
            );

            CREATE TABLE IF NOT EXISTS qr_scan_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                kind TEXT NOT NULL,
                table_no TEXT,
                order_id INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
            );

            CREATE TABLE IF NOT EXISTS campaigns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                discount_type TEXT NOT NULL,
                discount_value REAL NOT NULL DEFAULT 0,
                condition_text TEXT,
                valid_days INTEGER NOT NULL DEFAULT 30,
                is_active INTEGER NOT NULL DEFAULT 1,
                daily_limit INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
            );

            CREATE UNIQUE INDEX IF NOT EXISTS idx_menu_categories_name ON menu_categories(name);
            CREATE INDEX IF NOT EXISTS idx_qr_scan_events_kind ON qr_scan_events(kind, created_at);
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

    pub(super) fn seed_data(&self) -> Result<()> {
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
    pub(super) fn run_data_migrations(&self) -> Result<()> {
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
        let has_daily_limit: bool = conn
            .prepare("SELECT 1 FROM pragma_table_info('campaigns') WHERE name='daily_limit'")?
            .exists([])?;
        if !has_daily_limit {
            // campaigns table may not exist on very old DBs; ignore failure there.
            let _ = conn.execute_batch("ALTER TABLE campaigns ADD COLUMN daily_limit INTEGER NOT NULL DEFAULT 0");
        }
        let has_cover_image: bool = conn
            .prepare("SELECT 1 FROM pragma_table_info('campaigns') WHERE name='cover_image'")?
            .exists([])?;
        if !has_cover_image {
            let _ = conn.execute_batch("ALTER TABLE campaigns ADD COLUMN cover_image TEXT");
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
        let has_total_spent: bool = conn
            .prepare("SELECT 1 FROM pragma_table_info('customers') WHERE name='total_spent'")?
            .exists([])?;
        if !has_total_spent {
            conn.execute_batch("ALTER TABLE customers ADD COLUMN total_spent REAL NOT NULL DEFAULT 0.0")?;
        }
        let has_item_refunded: bool = conn
            .prepare("SELECT 1 FROM pragma_table_info('order_items') WHERE name='refunded'")?
            .exists([])?;
        if !has_item_refunded {
            conn.execute_batch("ALTER TABLE order_items ADD COLUMN refunded INTEGER NOT NULL DEFAULT 0")?;
        }
        let has_menu_image: bool = conn
            .prepare("SELECT 1 FROM pragma_table_info('menu_items') WHERE name='image_path'")?
            .exists([])?;
        if !has_menu_image {
            conn.execute_batch("ALTER TABLE menu_items ADD COLUMN image_path TEXT")?;
        }
        let has_menu_desc: bool = conn
            .prepare("SELECT 1 FROM pragma_table_info('menu_items') WHERE name='description'")?
            .exists([])?;
        if !has_menu_desc {
            conn.execute_batch("ALTER TABLE menu_items ADD COLUMN description TEXT")?;
        }
        let has_redemption_amount: bool = conn
            .prepare("SELECT 1 FROM pragma_table_info('marketing_redemptions') WHERE name='amount'")?
            .exists([])?;
        if !has_redemption_amount {
            let _ = conn.execute_batch("ALTER TABLE marketing_redemptions ADD COLUMN amount REAL NOT NULL DEFAULT 0");
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

}
