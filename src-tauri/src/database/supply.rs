use rusqlite::{params, Result};
use super::*;

impl Database {
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


}

