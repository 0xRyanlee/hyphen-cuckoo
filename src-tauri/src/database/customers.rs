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

        let mut result = Vec::new();
        for (cat_id, cat_name) in cats {
            let mut item_stmt = conn.prepare(
                "SELECT id, name, description, image_path, sales_price FROM menu_items
                 WHERE category_id = ?1 AND is_available = 1 ORDER BY id ASC"
            )?;
            let items: Vec<PublicMenuItem> = item_stmt.query_map(params![cat_id], |row| {
                Ok(PublicMenuItem {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    image_path: row.get(3)?,
                    sales_price: row.get(4)?,
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
        let conn = self.conn.lock().unwrap();
        // Fetch order_no and created_at for display
        let (order_no, created_at, amount_total): (String, String, f64) = conn.query_row(
            "SELECT order_no, strftime('%H:%M', created_at, 'localtime'), COALESCE(amount_total, 0) FROM orders WHERE id = ?1",
            rusqlite::params![order_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        ).unwrap_or_else(|_| (format!("#{}", order_id), "".to_string(), 0.0));

        // Get marketing popup template (type = marketing_popup), create default if absent
        let content_json = conn.query_row(
            "SELECT content FROM print_templates WHERE template_type = 'marketing_popup' AND is_active = 1 LIMIT 1",
            [],
            |r| r.get::<_, String>(0),
        ).unwrap_or_else(|_| {
            r#"{"elements":[
                {"type":"fortune","seed_strategy":"per_order"},
                {"type":"character_collect","game_name":"集字兌獎","characters":["恭","喜","發","財"],"prize":"集齊四字兌換免費飲品","seed_strategy":"per_order","style":"box"},
                {"type":"quote","language":"multilingual"}
            ]}"#.to_string()
        });

        Ok(serde_json::json!({
            "order_id": order_id,
            "order_no": order_no,
            "table_no": table_no,
            "created_at": created_at,
            "amount_total": amount_total,
            "template_content": content_json,
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
}
