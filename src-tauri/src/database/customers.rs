use rusqlite::{params, Result};
use super::*;

impl Database {
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

    // ==================== 顾客积分系统 ====================

    pub fn get_customers(&self, search: Option<&str>) -> Result<Vec<Customer>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = if let Some(q) = search {
            let pat = format!("%{}%", q);
            let mut s = conn.prepare(
                "SELECT id, COALESCE(name,''), phone, points, COALESCE(total_spent,0.0), created_at FROM customers WHERE is_active = 1 AND (name LIKE ?1 OR phone LIKE ?1) ORDER BY name"
            )?;
            let rows = s.query_map(params![pat], |row| Ok(Customer { id: row.get(0)?, name: row.get(1)?, phone: row.get(2)?, points: row.get(3)?, total_spent: row.get(4)?, created_at: row.get(5)? }))?.collect::<Result<Vec<_>>>()?;
            return Ok(rows);
        } else {
            conn.prepare("SELECT id, COALESCE(name,''), phone, points, COALESCE(total_spent,0.0), created_at FROM customers WHERE is_active = 1 ORDER BY name")?
        };
        let rows = stmt.query_map([], |row| Ok(Customer { id: row.get(0)?, name: row.get(1)?, phone: row.get(2)?, points: row.get(3)?, total_spent: row.get(4)?, created_at: row.get(5)? }))?.collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn create_customer(&self, name: &str, phone: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO customers (name, phone) VALUES (?1, ?2)",
            params![name, phone],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_customer(&self, id: i64, name: Option<&str>, phone: Option<Option<&str>>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        if let Some(n) = name {
            conn.execute("UPDATE customers SET name = ?1 WHERE id = ?2", params![n, id])?;
        }
        if let Some(p) = phone {
            conn.execute("UPDATE customers SET phone = ?1 WHERE id = ?2", params![p, id])?;
        }
        Ok(())
    }

    pub fn delete_customer(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE customers SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_loyalty_txns(&self, customer_id: i64) -> Result<Vec<LoyaltyTxn>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, customer_id, order_id, delta, reason, created_at FROM loyalty_txns WHERE customer_id = ?1 ORDER BY created_at DESC LIMIT 100"
        )?;
        let rows = stmt.query_map(params![customer_id], |row| {
            Ok(LoyaltyTxn { id: row.get(0)?, customer_id: row.get(1)?, order_id: row.get(2)?, delta: row.get(3)?, reason: row.get(4)?, created_at: row.get(5)? })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn add_loyalty_points(&self, customer_id: i64, order_id: Option<i64>, delta: i64, reason: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("BEGIN")?;
        let result: Result<i64> = (|| {
            conn.execute(
                "UPDATE customers SET points = points + ?1, total_spent = total_spent + CASE WHEN ?1 > 0 THEN CAST(?1 AS REAL) ELSE 0 END WHERE id = ?2",
                params![delta, customer_id],
            )?;
            conn.execute(
                "INSERT INTO loyalty_txns (customer_id, order_id, delta, reason) VALUES (?1, ?2, ?3, ?4)",
                params![customer_id, order_id, delta, reason],
            )?;
            let balance: i64 = conn.query_row("SELECT points FROM customers WHERE id = ?1", params![customer_id], |r| r.get(0))?;
            Ok(balance)
        })();
        match result {
            Ok(bal) => { conn.execute_batch("COMMIT")?; Ok(bal) }
            Err(e) => { conn.execute_batch("ROLLBACK").ok(); Err(e) }
        }
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

    // ==================== 餐桌管理 ====================

    pub fn get_restaurant_tables(&self) -> Result<Vec<RestaurantTable>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, table_no, label, is_active, sort_no, created_at FROM restaurant_tables ORDER BY sort_no ASC, id ASC"
        )?;
        let rows = stmt.query_map([], |row| Ok(RestaurantTable {
            id: row.get(0)?,
            table_no: row.get(1)?,
            label: row.get(2)?,
            is_active: row.get::<_, i64>(3)? != 0,
            sort_no: row.get(4)?,
            created_at: row.get(5)?,
        }))?
        .collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn create_restaurant_table(&self, table_no: &str, label: Option<&str>, is_active: bool, sort_no: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO restaurant_tables (table_no, label, is_active, sort_no) VALUES (?1, ?2, ?3, ?4)",
            params![table_no, label, is_active, sort_no],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_restaurant_table(&self, id: i64, table_no: &str, label: Option<&str>, is_active: bool, sort_no: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE restaurant_tables SET table_no = ?1, label = ?2, is_active = ?3, sort_no = ?4 WHERE id = ?5",
            params![table_no, label, is_active as i64, sort_no, id],
        )?;
        Ok(())
    }

    pub fn delete_restaurant_table(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM restaurant_tables WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ==================== 自助點單 ====================

    pub fn get_public_menu(&self) -> Result<Vec<PublicMenuCategory>> {
        let conn = self.conn.lock().unwrap();
        let mut cat_stmt = conn.prepare(
            "SELECT id, name FROM menu_categories ORDER BY id ASC"
        )?;
        let cats: Vec<(i64, String)> = cat_stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>>>()?;

        // Top-3 best sellers over the last 30 days → "热销" badge.
        let hot_ids: std::collections::HashSet<i64> = {
            let mut stmt = conn.prepare(
                "SELECT oi.menu_item_id FROM order_items oi JOIN orders o ON oi.order_id = o.id
                 WHERE o.created_at >= datetime('now','localtime','-30 days')
                 GROUP BY oi.menu_item_id ORDER BY SUM(oi.qty) DESC LIMIT 3"
            )?;
            let ids: Vec<i64> = stmt.query_map([], |r| r.get::<_, i64>(0))?.collect::<Result<Vec<_>>>()?;
            ids.into_iter().collect()
        };

        let mut result = Vec::new();
        for (cat_id, cat_name) in cats {
            let mut item_stmt = conn.prepare(
                "SELECT id, name, description, image_path, sales_price FROM menu_items
                 WHERE category_id = ?1 AND is_available = 1 ORDER BY id ASC"
            )?;
            let items: Vec<PublicMenuItem> = item_stmt.query_map(params![cat_id], |row| {
                let id: i64 = row.get(0)?;
                Ok(PublicMenuItem {
                    id,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    image_path: row.get(3)?,
                    sales_price: row.get(4)?,
                    is_hot: hot_ids.contains(&id),
                    specs: vec![],
                })
            })?.collect::<Result<Vec<_>>>()?;

            let mut items_with_specs = Vec::new();
            for mut item in items {
                let mut spec_stmt = conn.prepare(
                    "SELECT id, spec_code, spec_name, price_delta FROM menu_item_specs WHERE menu_item_id = ?1 ORDER BY sort_no ASC"
                )?;
                item.specs = spec_stmt.query_map(params![item.id], |row| Ok(PublicMenuItemSpec {
                    id: row.get(0)?,
                    spec_code: row.get(1)?,
                    spec_name: row.get(2)?,
                    price_delta: row.get(3)?,
                }))?.collect::<Result<Vec<_>>>()?;
                items_with_specs.push(item);
            }

            if !items_with_specs.is_empty() {
                result.push(PublicMenuCategory {
                    id: cat_id,
                    name: cat_name,
                    items: items_with_specs,
                });
            }
        }
        Ok(result)
    }

    pub fn create_self_order(&self, table_no: &str, items: &[SelfOrderItemInput]) -> Result<(i64, String)> {
        let conn = self.conn.lock().unwrap();
        // SEC1: if the shop has configured tables, the table_no must be a real one.
        // Blocks forged table numbers (guessed URLs / arbitrary tokens). 防呆: shops
        // that never configured tables skip this check and accept any table_no.
        let table_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM restaurant_tables", [], |r| r.get(0))
            .unwrap_or(0);
        if table_count > 0 {
            let exists: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM restaurant_tables WHERE table_no = ?1",
                    params![table_no],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            if exists == 0 {
                return Err(rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
                    Some("桌号无效，请扫描餐桌上的二维码".to_string()),
                ));
            }
        }
        // D4: reject the order if any item went unavailable since the menu was cached.
        for item in items {
            let avail: Option<i64> = conn.query_row(
                "SELECT is_available FROM menu_items WHERE id = ?1",
                params![item.menu_item_id],
                |r| r.get(0),
            ).ok();
            if avail != Some(1) {
                return Err(rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
                    Some("部分商品已售罄或下架，请刷新菜单后重试".to_string()),
                ));
            }
        }
        let order_no: String = conn.query_row(
            "SELECT 'SO' || strftime('%Y%m%d%H%M%S', 'now', 'localtime') || substr(lower(hex(randomblob(2))),1,4)",
            [],
            |r| r.get(0),
        )?;
        conn.execute(
            "INSERT INTO orders (order_no, source, dine_type, table_no, status) VALUES (?1, 'self_order', 'dine_in', ?2, 'pending')",
            params![order_no, table_no],
        )?;
        let order_id = conn.last_insert_rowid();
        for item in items {
            conn.execute(
                "INSERT INTO order_items (order_id, menu_item_id, spec_code, qty, unit_price, note)
                 SELECT ?1, ?2, ?3, ?4, sales_price + COALESCE((SELECT price_delta FROM menu_item_specs WHERE menu_item_id = ?2 AND spec_code = ?3), 0), ?5
                 FROM menu_items WHERE id = ?2",
                params![order_id, item.menu_item_id, item.spec_code, item.qty, item.note],
            )?;
        }
        conn.execute(
            "UPDATE orders SET amount_total = (SELECT COALESCE(SUM(qty * unit_price), 0) FROM order_items WHERE order_id = ?1) WHERE id = ?1",
            params![order_id],
        )?;
        // Auto-submit: create kitchen tickets
        let mut stations_stmt = conn.prepare("SELECT id FROM kitchen_stations WHERE is_active IS NULL OR is_active = 1")?;
        let station_ids: Vec<i64> = stations_stmt.query_map([], |r| r.get(0))?
            .collect::<Result<Vec<_>>>()?;
        if station_ids.is_empty() {
            conn.execute(
                "UPDATE orders SET status = 'submitted', submitted_at = datetime('now', 'localtime') WHERE id = ?1",
                params![order_id],
            )?;
        } else {
            for station_id in &station_ids {
                conn.execute(
                    "INSERT INTO kitchen_tickets (order_id, station_id, status, priority) VALUES (?1, ?2, 'pending', 0)",
                    params![order_id, station_id],
                )?;
            }
            conn.execute(
                "UPDATE orders SET status = 'submitted', submitted_at = datetime('now', 'localtime') WHERE id = ?1",
                params![order_id],
            )?;
        }
        Ok((order_id, order_no))
    }

    pub fn get_marketing_popup_content(&self, order_id: i64, table_no: &str) -> Result<serde_json::Value> {
        let (order_no, created_at, amount_total, content_json) = {
            let conn = self.conn.lock().unwrap();
            let meta: (String, String, f64) = conn.query_row(
                "SELECT order_no, strftime('%H:%M', created_at, 'localtime'), COALESCE(amount_total, 0) FROM orders WHERE id = ?1",
                rusqlite::params![order_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            ).unwrap_or_else(|_| (format!("#{}", order_id), "".to_string(), 0.0));
            let content = conn.query_row(
                "SELECT content FROM print_templates WHERE template_type = 'marketing_popup' AND is_active = 1 LIMIT 1",
                [],
                |r| r.get::<_, String>(0),
            ).unwrap_or_else(|_| {
                r#"{"elements":[
                    {"type":"fortune","seed_strategy":"per_order"},
                    {"type":"character_collect","game_name":"集字兑奖","characters":["恭","喜","发","财"],"prize":"集齐四字兑换免费饮品","seed_strategy":"per_order","style":"box"},
                    {"type":"quote","language":"multilingual"}
                ]}"#.to_string()
            });
            (meta.0, meta.1, meta.2, content)
        }; // conn released before issuing tokens (issue re-locks)

        // Inject per-order single-use QR token + backend-computed character into
        // each character_collect element, so the phone popup and receipt agree on
        // the same字 and the scan-to-redeem code is bound to this order.
        let mut parsed: serde_json::Value =
            serde_json::from_str(&content_json).unwrap_or_else(|_| serde_json::json!({ "elements": [] }));
        if let Some(elements) = parsed.get_mut("elements").and_then(|e| e.as_array_mut()) {
            let date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
            for elem in elements.iter_mut() {
                if elem.get("type").and_then(|t| t.as_str()) != Some("character_collect") {
                    continue;
                }
                let chars: Vec<String> = elem.get("characters").and_then(|c| c.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                if chars.is_empty() {
                    continue;
                }
                let strategy = elem.get("seed_strategy").and_then(|s| s.as_str()).unwrap_or("per_order");
                let seed = super::print::creative_fortune_seed(strategy, Some(table_no), Some(order_id), &date_str);
                let ch = chars[(seed as usize) % chars.len()].clone();
                if let Ok(token) = self.issue_marketing_qr_token(order_id, "character_collect", &ch) {
                    elem["qr_token"] = serde_json::json!(token);
                }
                elem["picked_char"] = serde_json::json!(ch);
            }
        }

        Ok(serde_json::json!({
            "order_id": order_id,
            "order_no": order_no,
            "table_no": table_no,
            "created_at": created_at,
            "amount_total": amount_total,
            "template_content": parsed.to_string(),
        }))
    }

    pub fn get_table_orders_today(&self, table_no: &str) -> Result<Vec<TableOrderSummary>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, order_no, status, amount_total, created_at
             FROM orders
             WHERE table_no = ?1
               AND date(created_at) = date('now', 'localtime')
               AND status != 'cancelled'
             ORDER BY created_at ASC"
        )?;
        let rows = stmt.query_map(params![table_no], |row| Ok(TableOrderSummary {
            id: row.get(0)?,
            order_no: row.get(1)?,
            status: row.get(2)?,
            amount_total: row.get(3)?,
            created_at: row.get(4)?,
            items: vec![],
        }))?
        .collect::<Result<Vec<_>>>()?;

        let mut result = Vec::new();
        for mut order in rows {
            let mut item_stmt = conn.prepare(
                "SELECT oi.qty, oi.unit_price, oi.note, m.name, oi.spec_code
                 FROM order_items oi JOIN menu_items m ON m.id = oi.menu_item_id
                 WHERE oi.order_id = ?1 AND COALESCE(oi.refunded, 0) = 0"
            )?;
            order.items = item_stmt.query_map(params![order.id], |row| Ok(TableOrderItem {
                qty: row.get(0)?,
                unit_price: row.get(1)?,
                note: row.get(2)?,
                name: row.get(3)?,
                spec_code: row.get(4)?,
            }))?
            .collect::<Result<Vec<_>>>()?;
            result.push(order);
        }
        Ok(result)
    }

    // ── 行销兑奖追踪 ─────────────────────────────────────────────────────────

    #[allow(dead_code)] // reserved for discount_coupon issuance pipeline (v3.3 W7)
    pub fn record_coupon_issued(&self, order_id: i64, code: &str, discount_type: &str, discount_value: f64, condition_text: Option<&str>, valid_until: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO issued_coupons (order_id, code, discount_type, discount_value, condition_text, valid_until) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![order_id, code, discount_type, discount_value, condition_text, valid_until],
        )?;
        Ok(())
    }

    pub fn redeem_coupon(&self, order_id: i64, staff_name: Option<&str>) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let already: i64 = conn.query_row(
            "SELECT COUNT(*) FROM issued_coupons WHERE order_id = ?1 AND redeemed_at IS NOT NULL",
            rusqlite::params![order_id], |r| r.get(0),
        ).unwrap_or(0);
        if already > 0 { return Ok(false); }
        conn.execute(
            "UPDATE issued_coupons SET redeemed_at = datetime('now','localtime'), redeemed_by = ?2 WHERE order_id = ?1",
            rusqlite::params![order_id, staff_name],
        )?;
        Ok(true)
    }

    pub fn record_marketing_redemption(&self, order_id: i64, component_type: &str, note: Option<&str>, staff_name: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO marketing_redemptions (order_id, component_type, note, staff_name) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![order_id, component_type, note, staff_name],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_marketing_redemptions(&self, date: Option<&str>) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let sql = if date.is_some() {
            "SELECT mr.id, mr.order_id, o.order_no, mr.component_type, mr.note, mr.staff_name, mr.redeemed_at FROM marketing_redemptions mr LEFT JOIN orders o ON mr.order_id = o.id WHERE date(mr.redeemed_at) = ?1 ORDER BY mr.redeemed_at DESC"
        } else {
            "SELECT mr.id, mr.order_id, o.order_no, mr.component_type, mr.note, mr.staff_name, mr.redeemed_at FROM marketing_redemptions mr LEFT JOIN orders o ON mr.order_id = o.id ORDER BY mr.redeemed_at DESC LIMIT 100"
        };
        let mut stmt = conn.prepare(sql)?;
        let rows = if let Some(d) = date {
            stmt.query_map(rusqlite::params![d], |r| Ok(serde_json::json!({
                "id": r.get::<_,i64>(0)?, "order_id": r.get::<_,i64>(1)?,
                "order_no": r.get::<_,String>(2).unwrap_or_default(),
                "component_type": r.get::<_,String>(3)?,
                "note": r.get::<_,Option<String>>(4)?,
                "staff_name": r.get::<_,Option<String>>(5)?,
                "redeemed_at": r.get::<_,String>(6)?,
            })))?.collect::<Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |r| Ok(serde_json::json!({
                "id": r.get::<_,i64>(0)?, "order_id": r.get::<_,i64>(1)?,
                "order_no": r.get::<_,String>(2).unwrap_or_default(),
                "component_type": r.get::<_,String>(3)?,
                "note": r.get::<_,Option<String>>(4)?,
                "staff_name": r.get::<_,Option<String>>(5)?,
                "redeemed_at": r.get::<_,String>(6)?,
            })))?.collect::<Result<Vec<_>>>()?
        };
        Ok(rows)
    }

    pub fn get_marketing_stats_today(&self) -> Result<serde_json::Value> {
        let conn = self.conn.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let redemptions: i64 = conn.query_row(
            "SELECT COUNT(*) FROM marketing_redemptions WHERE date(redeemed_at) = ?1",
            rusqlite::params![today], |r| r.get(0),
        ).unwrap_or(0);
        let coupons_issued: i64 = conn.query_row(
            "SELECT COUNT(*) FROM issued_coupons WHERE date(created_at) = ?1",
            rusqlite::params![today], |r| r.get(0),
        ).unwrap_or(0);
        let coupons_redeemed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM issued_coupons WHERE date(redeemed_at) = ?1",
            rusqlite::params![today], |r| r.get(0),
        ).unwrap_or(0);
        Ok(serde_json::json!({
            "redemptions_today": redemptions,
            "coupons_issued_today": coupons_issued,
            "coupons_redeemed_today": coupons_redeemed,
        }))
    }

    // ==================== QR Token (v2.8.0) ====================

    /// Issues a per-order, single-use marketing QR token (集字类). The same字
    /// on two receipts yields different tokens; redeeming voids it.
    pub fn issue_marketing_qr_token(&self, order_id: i64, component: &str, ch: &str) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        // Idempotent per (order, component): "每单唯一" means one code per order, so a
        // re-display (popup re-open, staff verify-order screen) reuses the same
        // non-void token instead of minting duplicates.
        if let Some(existing) = conn.query_row(
            "SELECT token FROM marketing_qr_tokens WHERE order_id = ?1 AND component = ?2 AND void = 0 ORDER BY rowid LIMIT 1",
            params![order_id, component],
            |r| r.get::<_, String>(0),
        ).ok() {
            return Ok(existing);
        }
        let mut nonce_bytes = [0u8; 4];
        rand_core::RngCore::fill_bytes(&mut rand_core::OsRng, &mut nonce_bytes);
        let nonce = hex::encode(nonce_bytes);
        let payload = crate::qr_token::marketing_payload(order_id, component, ch, &nonce);
        let token = crate::qr_token::make_token(&payload);
        conn.execute(
            "INSERT OR IGNORE INTO marketing_qr_tokens (token, order_id, component, ch) VALUES (?1, ?2, ?3, ?4)",
            params![token, order_id, component, ch],
        )?;
        Ok(token)
    }

    /// Redeems (voids) a marketing QR token. Returns a JSON verdict:
    /// `{ ok, already, reason?, order_id?, order_no?, component?, ch? }`.
    pub fn redeem_marketing_qr_token(&self, token: &str, staff_name: Option<&str>) -> Result<serde_json::Value> {
        let payload = match crate::qr_token::verify_token(token) {
            Some(p) => p,
            None => return Ok(serde_json::json!({ "ok": false, "reason": "invalid_signature" })),
        };
        let mp = match crate::qr_token::parse_marketing_payload(&payload) {
            Some(m) => m,
            None => return Ok(serde_json::json!({ "ok": false, "reason": "not_marketing_token" })),
        };
        let conn = self.conn.lock().unwrap();
        let existing: Option<(i64, Option<String>)> = conn.query_row(
            "SELECT void, redeemed_at FROM marketing_qr_tokens WHERE token = ?1",
            params![token],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ).ok();
        if let Some((void, _)) = &existing {
            if *void == 1 {
                let order_no: String = conn.query_row(
                    "SELECT order_no FROM orders WHERE id = ?1", params![mp.order_id], |r| r.get(0),
                ).unwrap_or_else(|_| format!("#{}", mp.order_id));
                return Ok(serde_json::json!({
                    "ok": false, "already": true, "order_id": mp.order_id, "order_no": order_no,
                    "component": mp.component, "ch": mp.ch,
                }));
            }
        }
        // Valid signature but token row absent (issued on a device that didn't persist)
        // → upsert so redemption is still recorded.
        conn.execute(
            "INSERT OR IGNORE INTO marketing_qr_tokens (token, order_id, component, ch) VALUES (?1, ?2, ?3, ?4)",
            params![token, mp.order_id, mp.component, mp.ch],
        )?;
        conn.execute(
            "UPDATE marketing_qr_tokens SET void = 1, redeemed_at = datetime('now','localtime') WHERE token = ?1",
            params![token],
        )?;
        conn.execute(
            "INSERT INTO marketing_redemptions (order_id, component_type, note, staff_name) VALUES (?1, ?2, '扫码核销', ?3)",
            params![mp.order_id, mp.component, staff_name],
        )?;
        let order_no: String = conn.query_row(
            "SELECT order_no FROM orders WHERE id = ?1", params![mp.order_id], |r| r.get(0),
        ).unwrap_or_else(|_| format!("#{}", mp.order_id));
        Ok(serde_json::json!({
            "ok": true, "already": false, "order_id": mp.order_id, "order_no": order_no,
            "component": mp.component, "ch": mp.ch,
        }))
    }

    pub fn record_qr_scan(&self, kind: &str, table_no: Option<&str>, order_id: Option<i64>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO qr_scan_events (kind, table_no, order_id) VALUES (?1, ?2, ?3)",
            params![kind, table_no, order_id],
        )?;
        Ok(())
    }

    /// Scan→order→redeem funnel over the last `days` days (v3.0.0 analytics).
    pub fn get_marketing_funnel(&self, days: i64) -> Result<serde_json::Value> {
        let conn = self.conn.lock().unwrap();
        let since = format!("-{} days", days.max(1));
        let scans: i64 = conn.query_row(
            "SELECT COUNT(*) FROM qr_scan_events WHERE kind = 'table' AND created_at >= datetime('now','localtime',?1)",
            params![since], |r| r.get(0),
        ).unwrap_or(0);
        let self_orders: i64 = conn.query_row(
            "SELECT COUNT(*) FROM orders WHERE source = 'self_order' AND created_at >= datetime('now','localtime',?1)",
            params![since], |r| r.get(0),
        ).unwrap_or(0);
        let redemptions: i64 = conn.query_row(
            "SELECT COUNT(*) FROM marketing_redemptions WHERE redeemed_at >= datetime('now','localtime',?1)",
            params![since], |r| r.get(0),
        ).unwrap_or(0);
        // Per-component redemption breakdown
        let mut stmt = conn.prepare(
            "SELECT component_type, COUNT(*) FROM marketing_redemptions WHERE redeemed_at >= datetime('now','localtime',?1) GROUP BY component_type ORDER BY COUNT(*) DESC",
        )?;
        let by_component: Vec<serde_json::Value> = stmt.query_map(params![since], |r| Ok(serde_json::json!({
            "component": r.get::<_,String>(0)?,
            "count": r.get::<_,i64>(1)?,
        })))?.collect::<Result<Vec<_>>>()?;
        Ok(serde_json::json!({
            "days": days,
            "scans": scans,
            "self_orders": self_orders,
            "redemptions": redemptions,
            "scan_to_order": if scans > 0 { self_orders as f64 / scans as f64 } else { 0.0 },
            "by_component": by_component,
        }))
    }

    // ==================== Campaigns (v3.2 方案B 扫码活动码得券) ====================

    pub fn create_campaign(&self, name: &str, discount_type: &str, discount_value: f64, condition_text: Option<&str>, valid_days: i64, daily_limit: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO campaigns (name, discount_type, discount_value, condition_text, valid_days, daily_limit) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![name, discount_type, discount_value, condition_text, valid_days, daily_limit],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_campaigns(&self) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT c.id, c.name, c.discount_type, c.discount_value, c.condition_text, c.valid_days, c.is_active, c.created_at, c.daily_limit,
                    (SELECT COUNT(*) FROM marketing_qr_tokens t WHERE t.order_id = c.id AND t.component = 'campaign_coupon') AS claimed,
                    (SELECT COUNT(*) FROM marketing_qr_tokens t WHERE t.order_id = c.id AND t.component = 'campaign_coupon' AND t.void = 1) AS redeemed
             FROM campaigns c ORDER BY c.id DESC"
        )?;
        let rows = stmt.query_map([], |r| Ok(serde_json::json!({
            "id": r.get::<_, i64>(0)?,
            "name": r.get::<_, String>(1)?,
            "discount_type": r.get::<_, String>(2)?,
            "discount_value": r.get::<_, f64>(3)?,
            "condition_text": r.get::<_, Option<String>>(4)?,
            "valid_days": r.get::<_, i64>(5)?,
            "is_active": r.get::<_, i64>(6)? == 1,
            "created_at": r.get::<_, String>(7)?,
            "daily_limit": r.get::<_, i64>(8)?,
            "claimed": r.get::<_, i64>(9)?,
            "redeemed": r.get::<_, i64>(10)?,
        })))?.collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn set_campaign_active(&self, id: i64, active: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE campaigns SET is_active = ?2 WHERE id = ?1", params![id, active as i64])?;
        Ok(())
    }

    pub fn delete_campaign(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM campaigns WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Resolve a scanned campaign poster code → issue ONE fresh coupon. Not
    /// idempotent: each scan mints a new single-use coupon (multi-claim is an
    /// accepted promo cost under the no-membership model). Coupon redeems via the
    /// existing `redeem_marketing_qr_token` loop (component = campaign_coupon).
    pub fn issue_campaign_coupon(&self, campaign_id: i64) -> Result<serde_json::Value> {
        let conn = self.conn.lock().unwrap();
        let camp: Option<(String, String, f64, Option<String>, i64, i64, i64)> = conn.query_row(
            "SELECT name, discount_type, discount_value, condition_text, valid_days, is_active, daily_limit FROM campaigns WHERE id = ?1",
            params![campaign_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)),
        ).ok();
        let (name, dtype, dval, cond, valid_days, is_active, daily_limit) = match camp {
            Some(c) => c,
            None => return Ok(serde_json::json!({ "valid": false })),
        };
        if is_active != 1 {
            return Ok(serde_json::json!({ "valid": false, "reason": "inactive" }));
        }
        // MUL backend: enforce a per-day issuance cap when set (daily_limit > 0).
        if daily_limit > 0 {
            let today_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM marketing_qr_tokens WHERE order_id = ?1 AND component = 'campaign_coupon' AND date(created_at) = date('now','localtime')",
                params![campaign_id], |r| r.get(0),
            ).unwrap_or(0);
            if today_count >= daily_limit {
                return Ok(serde_json::json!({ "valid": false, "reason": "daily_limit_reached" }));
            }
        }
        let mut nonce_bytes = [0u8; 6];
        rand_core::RngCore::fill_bytes(&mut rand_core::OsRng, &mut nonce_bytes);
        let nonce = hex::encode(nonce_bytes);
        let payload = crate::qr_token::marketing_payload(campaign_id, "campaign_coupon", &dtype, &nonce);
        let token = crate::qr_token::make_token(&payload);
        conn.execute(
            "INSERT OR IGNORE INTO marketing_qr_tokens (token, order_id, component, ch) VALUES (?1, ?2, 'campaign_coupon', ?3)",
            params![token, campaign_id, dtype],
        )?;
        Ok(serde_json::json!({
            "valid": true,
            "coupon_token": token,
            "campaign": {
                "id": campaign_id, "name": name, "discount_type": dtype,
                "discount_value": dval, "condition_text": cond, "valid_days": valid_days,
            },
        }))
    }

    // ==================== 集字兑换闭环 (v3.3 4.0) ====================

    /// Verify a marketing token and return its character + void status WITHOUT
    /// voiding it. Used by the staff 集字兑换 mode to accumulate scanned chars
    /// before redeeming the whole set at once.
    pub fn peek_marketing_qr_token(&self, token: &str) -> Result<serde_json::Value> {
        let payload = match crate::qr_token::verify_token(token) {
            Some(p) => p,
            None => return Ok(serde_json::json!({ "valid": false, "reason": "invalid_signature" })),
        };
        let mp = match crate::qr_token::parse_marketing_payload(&payload) {
            Some(m) => m,
            None => return Ok(serde_json::json!({ "valid": false, "reason": "not_marketing_token" })),
        };
        let conn = self.conn.lock().unwrap();
        let void: Option<i64> = conn.query_row(
            "SELECT void FROM marketing_qr_tokens WHERE token = ?1",
            params![token],
            |r| r.get(0),
        ).ok();
        Ok(serde_json::json!({
            "valid": true,
            "token": token,
            "order_id": mp.order_id,
            "component": mp.component,
            "ch": mp.ch,
            "already_void": void == Some(1),
        }))
    }

    /// Find the集字 token issued for a given order_no (序号后备:扫码失败时手输).
    pub fn find_collect_token_by_order_no(&self, order_no: &str) -> Result<serde_json::Value> {
        let conn = self.conn.lock().unwrap();
        let row: Option<(String, Option<String>, i64)> = conn.query_row(
            "SELECT t.token, t.ch, t.void FROM marketing_qr_tokens t
             JOIN orders o ON o.id = t.order_id
             WHERE o.order_no = ?1 AND t.component = 'character_collect'
             ORDER BY t.rowid DESC LIMIT 1",
            params![order_no],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        ).ok();
        match row {
            Some((token, ch, void)) => Ok(serde_json::json!({
                "valid": true, "token": token, "ch": ch, "already_void": void == 1,
            })),
            None => Ok(serde_json::json!({ "valid": false })),
        }
    }

    /// Redeem a SET of集字 tokens at once: re-verify each (signature + not voided),
    /// then void all and record a single set-redemption. Returns collected chars.
    pub fn collect_redeem_set(&self, tokens: &[String], staff_name: Option<&str>) -> Result<serde_json::Value> {
        if tokens.is_empty() {
            return Ok(serde_json::json!({ "ok": false, "reason": "empty" }));
        }
        let conn = self.conn.lock().unwrap();
        let mut chars: Vec<String> = Vec::new();
        let mut order_ids: Vec<i64> = Vec::new();
        // Validate all before mutating (all-or-nothing).
        for token in tokens {
            let payload = match crate::qr_token::verify_token(token) {
                Some(p) => p,
                None => return Ok(serde_json::json!({ "ok": false, "reason": "invalid_signature" })),
            };
            let mp = match crate::qr_token::parse_marketing_payload(&payload) {
                Some(m) => m,
                None => return Ok(serde_json::json!({ "ok": false, "reason": "not_marketing_token" })),
            };
            let void: Option<i64> = conn.query_row(
                "SELECT void FROM marketing_qr_tokens WHERE token = ?1",
                params![token], |r| r.get(0),
            ).ok();
            if void == Some(1) {
                return Ok(serde_json::json!({ "ok": false, "reason": "already_void", "ch": mp.ch }));
            }
            chars.push(mp.ch.clone());
            order_ids.push(mp.order_id);
        }
        // All valid → void each + record one set redemption.
        for token in tokens {
            conn.execute(
                "INSERT OR IGNORE INTO marketing_qr_tokens (token, order_id, component, ch)
                 SELECT ?1, 0, 'character_collect', '' WHERE NOT EXISTS (SELECT 1 FROM marketing_qr_tokens WHERE token = ?1)",
                params![token],
            )?;
            conn.execute(
                "UPDATE marketing_qr_tokens SET void = 1, redeemed_at = datetime('now','localtime') WHERE token = ?1",
                params![token],
            )?;
        }
        let note = format!("集字兑换:{}", chars.join(""));
        conn.execute(
            "INSERT INTO marketing_redemptions (order_id, component_type, note, staff_name) VALUES (?1, 'character_collect_set', ?2, ?3)",
            params![order_ids.first().copied().unwrap_or(0), note, staff_name],
        )?;
        Ok(serde_json::json!({ "ok": true, "chars": chars, "count": chars.len() }))
    }
}
