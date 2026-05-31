use rusqlite::{params, Result};
use super::*;

impl Database {
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


}
