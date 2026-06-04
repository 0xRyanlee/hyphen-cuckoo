use rusqlite::{params, Result};
use super::*;

impl Database {
    pub fn get_menu_categories(&self) -> Result<Vec<MenuCategory>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, sort_no, is_active FROM menu_categories ORDER BY sort_no")?;
        let cats = stmt.query_map([], |row| {
            Ok(MenuCategory { id: row.get(0)?, name: row.get(1)?, sort_no: row.get(2)?, is_active: row.get::<_, i32>(3)? != 0 })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(cats)
    }

    pub fn create_menu_category(&self, name: &str, sort_no: i32) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let code = name.to_uppercase().replace(' ', "_");
        conn.execute("INSERT INTO menu_categories (code, name, sort_no) VALUES (?1, ?2, ?3)", params![code, name, sort_no])?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_menu_items(&self, category_id: Option<i64>) -> Result<Vec<MenuItem>> {
        let conn = self.conn.lock().unwrap();
        let query = if category_id.is_some() {
            "SELECT id, name, code, category_id, recipe_id, sales_price, cost, is_available, is_favorite, image_path, description, created_at FROM menu_items WHERE category_id = ?1 AND is_combo = 0 ORDER BY name"
        } else {
            "SELECT id, name, code, category_id, recipe_id, sales_price, cost, is_available, is_favorite, image_path, description, created_at FROM menu_items WHERE is_combo = 0 ORDER BY name"
        };
        let mut stmt = conn.prepare(query)?;
        let items = if let Some(cat_id) = category_id {
            stmt.query_map(params![cat_id], |row| Ok(MenuItem {
                id: row.get(0)?, name: row.get(1)?, code: row.get(2)?,
                category_id: row.get(3)?, recipe_id: row.get(4)?,
                sales_price: row.get(5)?, cost: row.get(6)?,
                is_available: row.get::<_, i32>(7)? != 0, is_favorite: row.get::<_, i32>(8)? != 0,
                image_path: row.get(9)?, description: row.get(10)?, created_at: row.get(11)?,
            }))?.collect::<Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |row| Ok(MenuItem {
                id: row.get(0)?, name: row.get(1)?, code: row.get(2)?,
                category_id: row.get(3)?, recipe_id: row.get(4)?,
                sales_price: row.get(5)?, cost: row.get(6)?,
                is_available: row.get::<_, i32>(7)? != 0, is_favorite: row.get::<_, i32>(8)? != 0,
                image_path: row.get(9)?, description: row.get(10)?, created_at: row.get(11)?,
            }))?.collect::<Result<Vec<_>>>()?
        };
        Ok(items)
    }

    pub fn create_menu_item(&self, name: &str, category_id: Option<i64>, recipe_id: Option<i64>, sales_price: f64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO menu_items (name, category_id, recipe_id, sales_price) VALUES (?1, ?2, ?3, ?4)",
            params![name, category_id, recipe_id, sales_price],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_menu_item_specs(&self, menu_item_id: i64) -> Result<Vec<MenuItemSpec>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, menu_item_id, spec_code, spec_name, price_delta, qty_multiplier, sort_no FROM menu_item_specs WHERE menu_item_id = ?1 ORDER BY sort_no")?;
        let specs = stmt.query_map(params![menu_item_id], |row| {
            Ok(MenuItemSpec { id: row.get(0)?, menu_item_id: row.get(1)?, spec_code: row.get(2)?, spec_name: row.get(3)?, price_delta: row.get(4)?, qty_multiplier: row.get(5)?, sort_no: row.get(6)? })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(specs)
    }

    pub fn create_menu_item_spec(&self, menu_item_id: i64, spec_code: &str, spec_name: &str, price_delta: f64, qty_multiplier: f64, sort_no: i32) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO menu_item_specs (menu_item_id, spec_code, spec_name, price_delta, qty_multiplier, sort_no) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![menu_item_id, spec_code, spec_name, price_delta, qty_multiplier, sort_no],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_menu_item_spec(&self, id: i64, spec_code: Option<&str>, spec_name: Option<&str>, price_delta: Option<f64>, qty_multiplier: Option<f64>, sort_no: Option<i32>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        if let Some(sc) = spec_code { conn.execute("UPDATE menu_item_specs SET spec_code = ?1 WHERE id = ?2", params![sc, id])?; }
        if let Some(sn) = spec_name { conn.execute("UPDATE menu_item_specs SET spec_name = ?1 WHERE id = ?2", params![sn, id])?; }
        if let Some(pd) = price_delta { conn.execute("UPDATE menu_item_specs SET price_delta = ?1 WHERE id = ?2", params![pd, id])?; }
        if let Some(qm) = qty_multiplier { conn.execute("UPDATE menu_item_specs SET qty_multiplier = ?1 WHERE id = ?2", params![qm, id])?; }
        if let Some(s) = sort_no { conn.execute("UPDATE menu_item_specs SET sort_no = ?1 WHERE id = ?2", params![s, id])?; }
        Ok(())
    }

    pub fn delete_menu_item_spec(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM menu_item_specs WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_menu_items_for_pos(&self, category_id: Option<i64>) -> Result<Vec<MenuItem>> {
        let conn = self.conn.lock().unwrap();
        let query = if category_id.is_some() {
            "SELECT id, name, code, category_id, recipe_id, sales_price, cost, is_available, is_favorite, image_path, description, created_at FROM menu_items WHERE is_available = 1 AND is_combo = 0 AND category_id = ?1 ORDER BY name"
        } else {
            "SELECT id, name, code, category_id, recipe_id, sales_price, cost, is_available, is_favorite, image_path, description, created_at FROM menu_items WHERE is_available = 1 AND is_combo = 0 ORDER BY name"
        };
        let mut stmt = conn.prepare(query)?;
        let items = if let Some(cat_id) = category_id {
            stmt.query_map(params![cat_id], |row| Ok(MenuItem {
                id: row.get(0)?, name: row.get(1)?, code: row.get(2)?,
                category_id: row.get(3)?, recipe_id: row.get(4)?,
                sales_price: row.get(5)?, cost: row.get(6)?,
                is_available: row.get::<_, i32>(7)? != 0, is_favorite: row.get::<_, i32>(8)? != 0,
                image_path: row.get(9)?, description: row.get(10)?, created_at: row.get(11)?,
            }))?.collect::<Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |row| Ok(MenuItem {
                id: row.get(0)?, name: row.get(1)?, code: row.get(2)?,
                category_id: row.get(3)?, recipe_id: row.get(4)?,
                sales_price: row.get(5)?, cost: row.get(6)?,
                is_available: row.get::<_, i32>(7)? != 0, is_favorite: row.get::<_, i32>(8)? != 0,
                image_path: row.get(9)?, description: row.get(10)?, created_at: row.get(11)?,
            }))?.collect::<Result<Vec<_>>>()?
        };
        Ok(items)
    }


    // ── Menu item mutations (moved from orders.rs) ──────────────────────────
    pub fn update_menu_item(&self, id: i64, name: Option<&str>, category_id: Option<i64>, recipe_id: Option<i64>, sales_price: Option<f64>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE menu_items SET name = COALESCE(?1, name), category_id = COALESCE(?2, category_id), recipe_id = ?3, sales_price = COALESCE(?4, sales_price) WHERE id = ?5",
            params![name, category_id, recipe_id, sales_price, id],
        )?;
        Ok(())
    }

    pub fn set_menu_item_availability(&self, id: i64, is_available: bool) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE menu_items SET is_available = ?1 WHERE id = ?2",
            params![if is_available { 1 } else { 0 }, id],
        )?;
        Ok(is_available)
    }

    pub fn toggle_menu_item_favorite(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let current: i32 = conn.query_row(
            "SELECT is_favorite FROM menu_items WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        let new_val = if current == 0 { 1 } else { 0 };
        conn.execute("UPDATE menu_items SET is_favorite = ?1 WHERE id = ?2", params![new_val, id])?;
        Ok(new_val != 0)
    }

    pub fn batch_toggle_menu_item_availability(&self, ids: &[i64], is_available: bool) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let mut count = 0;
        for id in ids {
            let result = conn.execute("UPDATE menu_items SET is_available = ?1 WHERE id = ?2", params![if is_available { 1 } else { 0 }, id]);
            if result.is_ok() {
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn batch_update_menu_item_prices(&self, ids: &[i64], mode: &str, value: f64) -> Result<usize> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let mut count = 0;

        for id in ids {
            let current_price: f64 = tx.query_row(
                "SELECT sales_price FROM menu_items WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )?;

            let next_price = match mode {
                "set" => value,
                "delta" => current_price + value,
                "percent" => current_price * (1.0 + value / 100.0),
                _ => return Err(rusqlite::Error::InvalidParameterName("invalid batch price mode".to_string())),
            };

            let rounded_price = (next_price * 100.0).round() / 100.0;
            if rounded_price < 0.0 {
                return Err(rusqlite::Error::InvalidParameterName("negative menu price".to_string()));
            }

            tx.execute(
                "UPDATE menu_items SET sales_price = ?1 WHERE id = ?2",
                params![rounded_price, id],
            )?;
            count += 1;
        }

        tx.commit()?;
        Ok(count)
    }

    pub fn delete_menu_item(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM menu_item_specs WHERE menu_item_id = ?1", params![id])?;
        conn.execute("DELETE FROM station_menu_items WHERE menu_item_id = ?1", params![id])?;
        let affected = conn.execute("DELETE FROM menu_items WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(())
    }

    pub fn update_menu_category(&self, id: i64, name: Option<&str>, sort_no: Option<i32>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        if let Some(n) = name {
            let code = n.to_uppercase().replace(' ', "_");
            conn.execute("UPDATE menu_categories SET code = ?1, name = ?2, sort_no = COALESCE(?3, sort_no) WHERE id = ?4", params![code, n, sort_no, id])?;
        } else if let Some(s) = sort_no {
            conn.execute("UPDATE menu_categories SET sort_no = ?1 WHERE id = ?2", params![s, id])?;
        }
        Ok(())
    }

    pub fn delete_menu_category(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE menu_items SET category_id = NULL WHERE category_id = ?1", params![id])?;
        conn.execute("UPDATE menu_categories SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn create_combo(&self, name: &str, sales_price: f64, description: Option<&str>, components: &[(i64, i32)]) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("BEGIN")?;
        let result: Result<i64> = (|| {
            conn.execute(
                "INSERT INTO menu_items (name, sales_price, description, is_combo, is_available) VALUES (?1, ?2, ?3, 1, 1)",
                params![name, sales_price, description],
            )?;
            let menu_item_id = conn.last_insert_rowid();
            for (component_item_id, qty) in components {
                conn.execute(
                    "INSERT INTO combo_components (combo_item_id, component_item_id, qty) VALUES (?1, ?2, ?3)",
                    params![menu_item_id, component_item_id, qty],
                )?;
            }
            Ok(menu_item_id)
        })();
        match result {
            Ok(id) => { conn.execute_batch("COMMIT")?; Ok(id) }
            Err(e) => { conn.execute_batch("ROLLBACK").ok(); Err(e) }
        }
    }

    pub fn list_combos(&self) -> Result<Vec<ComboWithComponents>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, sales_price, is_available FROM menu_items WHERE is_combo = 1 ORDER BY name"
        )?;
        let combos: Vec<ComboWithComponents> = stmt.query_map([], |row| {
            Ok(ComboWithComponents {
                menu_item_id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                sales_price: row.get(3)?,
                is_available: row.get::<_, i32>(4)? != 0,
                components: vec![],
            })
        })?.collect::<Result<Vec<_>>>()?;

        let mut result = Vec::with_capacity(combos.len());
        for mut combo in combos {
            let mut comp_stmt = conn.prepare(
                "SELECT cc.component_item_id, mi.name, cc.qty FROM combo_components cc
                 JOIN menu_items mi ON mi.id = cc.component_item_id
                 WHERE cc.combo_item_id = ?1 ORDER BY cc.id"
            )?;
            combo.components = comp_stmt.query_map(params![combo.menu_item_id], |row| {
                Ok(ComboComponent {
                    component_item_id: row.get(0)?,
                    component_name: row.get(1)?,
                    qty: row.get(2)?,
                })
            })?.collect::<Result<Vec<_>>>()?;
            result.push(combo);
        }
        Ok(result)
    }

    pub fn update_combo(&self, menu_item_id: i64, name: Option<&str>, sales_price: Option<f64>, description: Option<&str>, is_available: Option<bool>, components: Option<&[(i64, i32)]>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("BEGIN")?;
        let result: Result<()> = (|| {
            if let Some(n) = name {
                conn.execute("UPDATE menu_items SET name = ?1 WHERE id = ?2 AND is_combo = 1", params![n, menu_item_id])?;
            }
            if let Some(p) = sales_price {
                conn.execute("UPDATE menu_items SET sales_price = ?1 WHERE id = ?2 AND is_combo = 1", params![p, menu_item_id])?;
            }
            if description.is_some() {
                conn.execute("UPDATE menu_items SET description = ?1 WHERE id = ?2 AND is_combo = 1", params![description, menu_item_id])?;
            }
            if let Some(av) = is_available {
                conn.execute("UPDATE menu_items SET is_available = ?1 WHERE id = ?2 AND is_combo = 1", params![if av { 1 } else { 0 }, menu_item_id])?;
            }
            if let Some(comps) = components {
                conn.execute("DELETE FROM combo_components WHERE combo_item_id = ?1", params![menu_item_id])?;
                for (component_item_id, qty) in comps {
                    conn.execute(
                        "INSERT INTO combo_components (combo_item_id, component_item_id, qty) VALUES (?1, ?2, ?3)",
                        params![menu_item_id, component_item_id, qty],
                    )?;
                }
            }
            Ok(())
        })();
        match result {
            Ok(_) => { conn.execute_batch("COMMIT")?; Ok(()) }
            Err(e) => { conn.execute_batch("ROLLBACK").ok(); Err(e) }
        }
    }

    pub fn delete_combo(&self, menu_item_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("BEGIN")?;
        let result: Result<()> = (|| {
            conn.execute("DELETE FROM combo_components WHERE combo_item_id = ?1", params![menu_item_id])?;
            conn.execute("DELETE FROM menu_item_specs WHERE menu_item_id = ?1", params![menu_item_id])?;
            conn.execute("DELETE FROM station_menu_items WHERE menu_item_id = ?1", params![menu_item_id])?;
            conn.execute("DELETE FROM menu_items WHERE id = ?1 AND is_combo = 1", params![menu_item_id])?;
            Ok(())
        })();
        match result {
            Ok(_) => { conn.execute_batch("COMMIT")?; Ok(()) }
            Err(e) => { conn.execute_batch("ROLLBACK").ok(); Err(e) }
        }
    }


}