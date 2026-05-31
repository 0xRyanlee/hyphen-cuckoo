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
            "SELECT id, name, code, category_id, recipe_id, sales_price, cost, is_available, is_favorite, image_path, description, created_at FROM menu_items WHERE category_id = ?1 ORDER BY name"
        } else {
            "SELECT id, name, code, category_id, recipe_id, sales_price, cost, is_available, is_favorite, image_path, description, created_at FROM menu_items ORDER BY name"
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
            "SELECT id, name, code, category_id, recipe_id, sales_price, cost, is_available, is_favorite, image_path, description, created_at FROM menu_items WHERE is_available = 1 AND category_id = ?1 ORDER BY name"
        } else {
            "SELECT id, name, code, category_id, recipe_id, sales_price, cost, is_available, is_favorite, image_path, description, created_at FROM menu_items WHERE is_available = 1 ORDER BY name"
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


}
