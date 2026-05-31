use rusqlite::{params, Result};
use super::*;

impl Database {
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
            "SELECT m.id, m.name, m.code, m.category_id, m.recipe_id, m.sales_price, m.cost, m.is_available, m.is_favorite, m.image_path, m.description, m.created_at
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
                image_path: row.get(9)?,
                description: row.get(10)?,
                created_at: row.get(11)?,
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
    pub(super) fn consume_order_inventory(conn: &Connection, order_id: i64) -> Result<()> {
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
            "SELECT id, order_id, menu_item_id, spec_code, qty, unit_price, note, COALESCE(refunded,0) FROM order_items WHERE order_id = ?1"
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
                refunded: row.get::<_, i64>(7)? != 0,
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


    // ── Order fulfillment (moved from purchase.rs) ─────────────────────────
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

    pub fn refund_order_item(&self, order_id: i64, item_id: i64) -> Result<f64> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("BEGIN")?;
        let result: Result<f64> = (|| {
            // Verify item belongs to this order and is not already refunded
            let (qty, unit_price, already_refunded): (f64, f64, i64) = conn.query_row(
                "SELECT qty, unit_price, COALESCE(refunded,0) FROM order_items WHERE id = ?1 AND order_id = ?2",
                params![item_id, order_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            ).map_err(|_| rusqlite::Error::QueryReturnedNoRows)?;
            if already_refunded != 0 {
                return Err(rusqlite::Error::InvalidQuery);
            }
            let item_amount = qty * unit_price;
            conn.execute(
                "UPDATE order_items SET refunded = 1 WHERE id = ?1",
                params![item_id],
            )?;
            conn.execute(
                "UPDATE orders SET refund_amount = COALESCE(refund_amount,0) + ?1, updated_at = datetime('now') WHERE id = ?2",
                params![item_amount, order_id],
            )?;
            Ok(item_amount)
        })();
        match result {
            Ok(amt) => { conn.execute_batch("COMMIT")?; Ok(amt) }
            Err(e) => { conn.execute_batch("ROLLBACK").ok(); Err(e) }
        }
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


}