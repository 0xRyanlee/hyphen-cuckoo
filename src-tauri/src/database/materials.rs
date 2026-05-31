use rusqlite::{params, Result};
use super::*;

impl Database {

    pub fn get_units(&self) -> Result<Vec<Unit>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, code, name, unit_type, ratio_to_base FROM units ORDER BY id")?;
        let units = stmt.query_map([], |row| {
            Ok(Unit { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, unit_type: row.get(3)?, ratio_to_base: row.get(4)? })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(units)
    }

    pub fn get_material_categories(&self) -> Result<Vec<MaterialCategory>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, code, name, sort_no, is_active FROM material_categories ORDER BY sort_no")?;
        let cats = stmt.query_map([], |row| {
            Ok(MaterialCategory { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, sort_no: row.get(3)?, is_active: row.get::<_, i32>(4)? != 0 })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(cats)
    }

    pub fn create_material_category(&self, code: &str, name: &str, sort_no: i32) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        if let Ok(existing_id) = conn.query_row(
            "SELECT id FROM material_categories WHERE code = ?1 AND is_active = 0",
            params![code],
            |row| row.get::<_, i64>(0),
        ) {
            conn.execute("UPDATE material_categories SET is_active = 1, name = ?1, sort_no = ?2 WHERE id = ?3", params![name, sort_no, existing_id])?;
            return Ok(existing_id);
        }
        conn.execute("INSERT INTO material_categories (code, name, sort_no) VALUES (?1, ?2, ?3)", params![code, name, sort_no])?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_material_category(&self, id: i64, name: &str, sort_no: i32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE material_categories SET name = ?1, sort_no = ?2 WHERE id = ?3", params![name, sort_no, id])?;
        Ok(())
    }

    pub fn delete_material_category(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM materials WHERE category_id = ?1 AND is_active = 1",
            params![id], |r| r.get(0),
        )?;
        if count > 0 {
            return Err(rusqlite::Error::InvalidParameterName(
                format!("該分類下有 {} 個活躍原料，刪除前請先更換分類", count),
            ));
        }
        conn.execute("UPDATE material_categories SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_tags(&self) -> Result<Vec<Tag>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, code, name, color, is_active FROM tags ORDER BY id")?;
        let tags = stmt.query_map([], |row| {
            Ok(Tag { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, color: row.get(3)?, is_active: row.get::<_, i32>(4)? != 0 })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(tags)
    }

    pub fn create_tag(&self, code: &str, name: &str, color: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        if let Ok(existing_id) = conn.query_row(
            "SELECT id FROM tags WHERE code = ?1 AND is_active = 0",
            params![code],
            |row| row.get::<_, i64>(0),
        ) {
            conn.execute("UPDATE tags SET is_active = 1, name = ?1, color = ?2 WHERE id = ?3", params![name, color, existing_id])?;
            return Ok(existing_id);
        }
        conn.execute("INSERT INTO tags (code, name, color) VALUES (?1, ?2, ?3)", params![code, name, color])?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_tag(&self, id: i64, name: &str, color: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE tags SET name = ?1, color = ?2 WHERE id = ?3", params![name, color, id])?;
        Ok(())
    }

    pub fn delete_tag(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE tags SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_materials(&self, category_id: Option<i64>) -> Result<Vec<MaterialWithTags>> {
        use std::collections::HashMap;
        let conn = self.conn.lock().unwrap();

        // Single query: materials JOIN categories JOIN units
        let row_to_item = |row: &rusqlite::Row| -> rusqlite::Result<(Material, Option<MaterialCategory>, Option<Unit>)> {
            let mat = Material {
                id: row.get(0)?, code: row.get(1)?, name: row.get(2)?,
                category_id: row.get(3)?, base_unit_id: row.get(4)?,
                shelf_life_days: row.get(5)?, min_qty: row.get(6)?,
                is_active: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?, updated_at: row.get(9)?,
            };
            let category: Option<MaterialCategory> = match row.get::<_, Option<i64>>(10)? {
                Some(id) => Some(MaterialCategory {
                    id,
                    code: row.get(11)?, name: row.get(12)?,
                    sort_no: row.get(13)?, is_active: row.get::<_, i32>(14)? != 0,
                }),
                None => None,
            };
            let base_unit: Option<Unit> = match row.get::<_, Option<i64>>(15)? {
                Some(id) => Some(Unit {
                    id,
                    code: row.get(16)?, name: row.get(17)?,
                    unit_type: row.get(18)?, ratio_to_base: row.get(19)?,
                }),
                None => None,
            };
            Ok((mat, category, base_unit))
        };

        let base_sql = "SELECT m.id, m.code, m.name, m.category_id, m.base_unit_id, m.shelf_life_days, m.min_qty, m.is_active, m.created_at, m.updated_at, \
            c.id, c.code, c.name, c.sort_no, c.is_active, \
            u.id, u.code, u.name, u.unit_type, u.ratio_to_base \
            FROM materials m \
            LEFT JOIN material_categories c ON m.category_id = c.id \
            LEFT JOIN units u ON m.base_unit_id = u.id \
            WHERE m.is_active = 1";

        let rows: Vec<(Material, Option<MaterialCategory>, Option<Unit>)> = if let Some(cat_id) = category_id {
            let sql = format!("{} AND m.category_id = ?1 ORDER BY m.id", base_sql);
            conn.prepare(&sql)?.query_map(params![cat_id], row_to_item)?.collect::<Result<Vec<_>>>()?
        } else {
            let sql = format!("{} ORDER BY m.id", base_sql);
            conn.prepare(&sql)?.query_map([], row_to_item)?.collect::<Result<Vec<_>>>()?
        };

        // Batch fetch all tags in one query and group by material_id
        let mut tags_map: HashMap<i64, Vec<Tag>> = HashMap::new();
        let tag_rows: Vec<(i64, Tag)> = if let Some(cat_id) = category_id {
            let mut tag_stmt = conn.prepare(
                "SELECT mt.material_id, t.id, t.code, t.name, t.color, t.is_active FROM material_tags mt JOIN tags t ON mt.tag_id = t.id WHERE mt.material_id IN (SELECT id FROM materials WHERE is_active = 1 AND category_id = ?1)"
            )?;
            let x = tag_stmt.query_map(params![cat_id], |row| {
                Ok((row.get::<_, i64>(0)?, Tag { id: row.get(1)?, code: row.get(2)?, name: row.get(3)?, color: row.get(4)?, is_active: row.get::<_, i32>(5)? != 0 }))
            })?.collect::<Result<Vec<_>>>()?; x
        } else {
            let mut tag_stmt = conn.prepare(
                "SELECT mt.material_id, t.id, t.code, t.name, t.color, t.is_active FROM material_tags mt JOIN tags t ON mt.tag_id = t.id WHERE mt.material_id IN (SELECT id FROM materials WHERE is_active = 1)"
            )?;
            let x = tag_stmt.query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, Tag { id: row.get(1)?, code: row.get(2)?, name: row.get(3)?, color: row.get(4)?, is_active: row.get::<_, i32>(5)? != 0 }))
            })?.collect::<Result<Vec<_>>>()?; x
        };
        for (mat_id, tag) in tag_rows {
            tags_map.entry(mat_id).or_default().push(tag);
        }

        let result = rows.into_iter().map(|(mat, category, base_unit)| {
            let tags = tags_map.remove(&mat.id).unwrap_or_default();
            MaterialWithTags { material: mat, tags, category, base_unit }
        }).collect();
        Ok(result)
    }

    pub fn create_material(&self, code: &str, name: &str, category_id: Option<i64>, base_unit_id: i64, shelf_life_days: Option<i32>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        if let Ok(existing_id) = conn.query_row(
            "SELECT id FROM materials WHERE code = ?1 AND is_active = 0",
            params![code],
            |row| row.get::<_, i64>(0),
        ) {
            conn.execute(
                "UPDATE materials SET is_active = 1, name = ?1, category_id = ?2, base_unit_id = ?3, shelf_life_days = ?4 WHERE id = ?5",
                params![name, category_id, base_unit_id, shelf_life_days, existing_id],
            )?;
            return Ok(existing_id);
        }
        conn.execute(
            "INSERT INTO materials (code, name, category_id, base_unit_id, shelf_life_days) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![code, name, category_id, base_unit_id, shelf_life_days],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_material(&self, id: i64, name: Option<&str>, category_id: Option<i64>, shelf_life_days: Option<i32>, min_qty: Option<f64>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE materials SET name = COALESCE(?1, name), category_id = COALESCE(?2, category_id), shelf_life_days = COALESCE(?3, shelf_life_days), min_qty = COALESCE(?4, min_qty), updated_at = datetime('now') WHERE id = ?5",
            params![name, category_id, shelf_life_days, min_qty, id],
        )?;
        Ok(())
    }

    pub fn delete_material(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE materials SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn add_material_tags(&self, material_id: i64, tag_ids: &[i64]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        for tag_id in tag_ids {
            conn.execute(
                "INSERT OR IGNORE INTO material_tags (material_id, tag_id) VALUES (?1, ?2)",
                params![material_id, tag_id],
            )?;
        }
        Ok(())
    }

    pub fn remove_material_tag(&self, material_id: i64, tag_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM material_tags WHERE material_id = ?1 AND tag_id = ?2", params![material_id, tag_id])?;
        Ok(())
    }

    pub fn get_material_states(&self, material_id: i64) -> Result<Vec<MaterialState>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, material_id, state_code, state_name, unit_id, yield_rate, cost_multiplier, is_active FROM material_states WHERE material_id = ?1 AND is_active = 1 ORDER BY id")?;
        let states = stmt.query_map(params![material_id], |row| {
            Ok(MaterialState { id: row.get(0)?, material_id: row.get(1)?, state_code: row.get(2)?, state_name: row.get(3)?, unit_id: row.get(4)?, yield_rate: row.get(5)?, cost_multiplier: row.get(6)?, is_active: row.get::<_, i32>(7)? != 0 })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(states)
    }

    pub fn create_material_state(&self, material_id: i64, state_code: &str, state_name: &str, unit_id: Option<i64>, yield_rate: f64, cost_multiplier: f64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO material_states (material_id, state_code, state_name, unit_id, yield_rate, cost_multiplier) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![material_id, state_code, state_name, unit_id, yield_rate, cost_multiplier],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_all_material_states(&self) -> Result<Vec<MaterialState>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, material_id, state_code, state_name, unit_id, yield_rate, cost_multiplier, is_active FROM material_states ORDER BY material_id, id")?;
        let states = stmt.query_map([], |row| {
            Ok(MaterialState { id: row.get(0)?, material_id: row.get(1)?, state_code: row.get(2)?, state_name: row.get(3)?, unit_id: row.get(4)?, yield_rate: row.get(5)?, cost_multiplier: row.get(6)?, is_active: row.get::<_, i32>(7)? != 0 })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(states)
    }

    pub fn update_material_state(&self, id: i64, state_code: Option<&str>, state_name: Option<&str>, unit_id: Option<i64>, yield_rate: Option<f64>, cost_multiplier: Option<f64>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE material_states SET state_code = COALESCE(?1, state_code), state_name = COALESCE(?2, state_name), unit_id = COALESCE(?3, unit_id), yield_rate = COALESCE(?4, yield_rate), cost_multiplier = COALESCE(?5, cost_multiplier) WHERE id = ?6",
            params![state_code, state_name, unit_id, yield_rate, cost_multiplier, id],
        )?;
        Ok(())
    }

    pub fn delete_material_state(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE material_states SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_suppliers(&self) -> Result<Vec<Supplier>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, phone, contact_person, address, note, is_active, created_at FROM suppliers WHERE is_active = 1 ORDER BY name")?;
        let suppliers = stmt.query_map([], |row| {
            Ok(Supplier { id: row.get(0)?, name: row.get(1)?, phone: row.get(2)?, contact_person: row.get(3)?, address: row.get(4)?, note: row.get(5)?, is_active: row.get::<_, i32>(6)? != 0, created_at: row.get(7)? })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(suppliers)
    }

    pub fn create_supplier(&self, name: &str, phone: Option<&str>, contact_person: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT INTO suppliers (name, phone, contact_person) VALUES (?1, ?2, ?3)", params![name, phone, contact_person])?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_supplier(&self, id: i64, name: Option<&str>, phone: Option<&str>, contact_person: Option<&str>, address: Option<&str>, note: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE suppliers SET name = COALESCE(?1, name), phone = COALESCE(?2, phone), contact_person = COALESCE(?3, contact_person), address = COALESCE(?4, address), note = COALESCE(?5, note) WHERE id = ?6",
            params![name, phone, contact_person, address, note, id],
        )?;
        Ok(())
    }

    pub fn delete_supplier(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE suppliers SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ==================== 日常支出 API ====================

    pub fn get_expenses(&self, expense_type: Option<&str>, start_date: Option<&str>, end_date: Option<&str>) -> Result<Vec<Expense>> {
        let conn = self.conn.lock().unwrap();
        let mut sql = "SELECT id, expense_type, amount, expense_date, note, operator, is_active, created_at FROM expenses WHERE is_active = 1".to_string();
        let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        if expense_type.is_some() {
            sql.push_str(" AND expense_type = ?1");
            args.push(Box::new(expense_type.unwrap().to_string()));
        }
        if start_date.is_some() {
            let idx = args.len() + 1;
            sql.push_str(&format!(" AND expense_date >= ?{}", idx));
            args.push(Box::new(start_date.unwrap().to_string()));
        }
        if end_date.is_some() {
            let idx = args.len() + 1;
            sql.push_str(&format!(" AND expense_date <= ?{}", idx));
            args.push(Box::new(end_date.unwrap().to_string()));
        }
        sql.push_str(" ORDER BY expense_date DESC, id DESC");
        let mut stmt = conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> = args.iter().map(|b| b.as_ref()).collect();
        let expenses = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(Expense { id: row.get(0)?, expense_type: row.get(1)?, amount: row.get(2)?, expense_date: row.get(3)?, note: row.get(4)?, operator: row.get(5)?, is_active: row.get::<_, i32>(6)? != 0, created_at: row.get(7)? })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(expenses)
    }

    pub fn create_expense(&self, expense_type: &str, amount: f64, expense_date: &str, note: Option<&str>, operator: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT INTO expenses (expense_type, amount, expense_date, note, operator) VALUES (?1, ?2, ?3, ?4, ?5)", params![expense_type, amount, expense_date, note, operator])?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_expense(&self, id: i64, expense_type: Option<&str>, amount: Option<f64>, expense_date: Option<&str>, note: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE expenses SET expense_type = COALESCE(?1, expense_type), amount = COALESCE(?2, amount), expense_date = COALESCE(?3, expense_date), note = COALESCE(?4, note) WHERE id = ?5",
            params![expense_type, amount, expense_date, note, id],
        )?;
        Ok(())
    }

    pub fn delete_expense(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE expenses SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_supplier_products(&self, channel: Option<&str>) -> Result<Vec<SupplierProduct>> {
        let conn = self.conn.lock().unwrap();
        let sql = if channel.is_some() {
            "SELECT id, product_name, supplier_name, channel FROM supplier_products WHERE is_active = 1 AND channel = ?1 ORDER BY id"
        } else {
            "SELECT id, product_name, supplier_name, channel FROM supplier_products WHERE is_active = 1 ORDER BY id"
        };
        let mut stmt = conn.prepare(sql)?;
        let items = if let Some(ch) = channel {
            stmt.query_map(params![ch], |row| {
                Ok(SupplierProduct { id: row.get(0)?, product_name: row.get(1)?, supplier_name: row.get(2)?, channel: row.get(3)? })
            })?.collect::<Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |row| {
                Ok(SupplierProduct { id: row.get(0)?, product_name: row.get(1)?, supplier_name: row.get(2)?, channel: row.get(3)? })
            })?.collect::<Result<Vec<_>>>()?
        };
        Ok(items)
    }

    pub fn create_supplier_product(&self, product_name: &str, supplier_name: &str, channel: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT INTO supplier_products (product_name, supplier_name, channel) VALUES (?1, ?2, ?3)", params![product_name, supplier_name, channel])?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_supplier_product(&self, id: i64, product_name: &str, supplier_name: &str, channel: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE supplier_products SET product_name = ?1, supplier_name = ?2, channel = ?3 WHERE id = ?4",
            params![product_name, supplier_name, channel, id],
        )?;
        Ok(())
    }

    pub fn delete_supplier_product(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE supplier_products SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Computes the food cost for an order by summing each item's recipe cost_per_unit × qty.
    /// Respects spec qty_multiplier. Returns 0.0 if a menu item has no recipe.
    pub fn get_order_cost(&self, order_id: i64) -> Result<f64> {
        let items: Vec<(f64, Option<i64>, f64)> = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT oi.qty, mi.recipe_id, COALESCE(mis.qty_multiplier, 1.0)
                 FROM order_items oi
                 JOIN menu_items mi ON mi.id = oi.menu_item_id
                 LEFT JOIN menu_item_specs mis ON mis.menu_item_id = mi.id AND mis.spec_code = oi.spec_code
                 WHERE oi.order_id = ?1"
            )?;
            let result = stmt.query_map(params![order_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
                .collect::<Result<Vec<_>>>()?;
            result
        };
        let mut total = 0.0;
        for (qty, recipe_id, qty_multiplier) in items {
            if let Some(rid) = recipe_id {
                if let Ok(cost) = self.calculate_recipe_cost(rid) {
                    total += qty * qty_multiplier * cost.cost_per_unit;
                }
            }
        }
        Ok(total)
    }

    pub fn get_attribute_templates(&self, entity_type: Option<&str>, category: Option<&str>) -> Result<Vec<AttributeTemplate>> {
        let conn = self.conn.lock().unwrap();
        let query = match (entity_type, category) {
            (Some(_et), Some(_cat)) => "SELECT id, entity_type, category, attr_code, attr_name, data_type, unit, default_value, formula, is_template, is_active FROM attribute_templates WHERE entity_type = ?1 AND category = ?2 ORDER BY id",
            (Some(_et), None) => "SELECT id, entity_type, category, attr_code, attr_name, data_type, unit, default_value, formula, is_template, is_active FROM attribute_templates WHERE entity_type = ?1 ORDER BY id",
            (None, Some(_cat)) => "SELECT id, entity_type, category, attr_code, attr_name, data_type, unit, default_value, formula, is_template, is_active FROM attribute_templates WHERE category = ?1 ORDER BY id",
            (None, None) => "SELECT id, entity_type, category, attr_code, attr_name, data_type, unit, default_value, formula, is_template, is_active FROM attribute_templates ORDER BY id",
        };
        let mut stmt = conn.prepare(query)?;
        let templates = match (entity_type, category) {
            (Some(et), Some(cat)) => stmt.query_map(params![et, cat], |row| {
                Ok(AttributeTemplate { id: row.get(0)?, entity_type: row.get(1)?, category: row.get(2)?, attr_code: row.get(3)?, attr_name: row.get(4)?, data_type: row.get(5)?, unit: row.get(6)?, default_value: row.get(7)?, formula: row.get(8)?, is_template: row.get::<_, i32>(9)? != 0, is_active: row.get::<_, i32>(10)? != 0 })
            })?.collect::<Result<Vec<_>>>()?,
            (Some(et), None) => stmt.query_map(params![et], |row| {
                Ok(AttributeTemplate { id: row.get(0)?, entity_type: row.get(1)?, category: row.get(2)?, attr_code: row.get(3)?, attr_name: row.get(4)?, data_type: row.get(5)?, unit: row.get(6)?, default_value: row.get(7)?, formula: row.get(8)?, is_template: row.get::<_, i32>(9)? != 0, is_active: row.get::<_, i32>(10)? != 0 })
            })?.collect::<Result<Vec<_>>>()?,
            (None, Some(cat)) => stmt.query_map(params![cat], |row| {
                Ok(AttributeTemplate { id: row.get(0)?, entity_type: row.get(1)?, category: row.get(2)?, attr_code: row.get(3)?, attr_name: row.get(4)?, data_type: row.get(5)?, unit: row.get(6)?, default_value: row.get(7)?, formula: row.get(8)?, is_template: row.get::<_, i32>(9)? != 0, is_active: row.get::<_, i32>(10)? != 0 })
            })?.collect::<Result<Vec<_>>>()?,
            (None, None) => stmt.query_map([], |row| {
                Ok(AttributeTemplate { id: row.get(0)?, entity_type: row.get(1)?, category: row.get(2)?, attr_code: row.get(3)?, attr_name: row.get(4)?, data_type: row.get(5)?, unit: row.get(6)?, default_value: row.get(7)?, formula: row.get(8)?, is_template: row.get::<_, i32>(9)? != 0, is_active: row.get::<_, i32>(10)? != 0 })
            })?.collect::<Result<Vec<_>>>()?,
        };
        Ok(templates)
    }

    pub fn create_attribute_template(&self, entity_type: &str, category: Option<&str>, attr_code: &str, attr_name: &str, data_type: &str, unit: Option<&str>, default_value: Option<f64>, formula: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO attribute_templates (entity_type, category, attr_code, attr_name, data_type, unit, default_value, formula, is_template, is_active) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, 1)",
            params![entity_type, category, attr_code, attr_name, data_type, unit, default_value, formula],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_attribute_template(&self, id: i64, entity_type: &str, category: Option<&str>, attr_code: &str, attr_name: &str, data_type: &str, unit: Option<&str>, default_value: Option<f64>, formula: Option<&str>, is_active: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE attribute_templates SET entity_type = ?1, category = ?2, attr_code = ?3, attr_name = ?4, data_type = ?5, unit = ?6, default_value = ?7, formula = ?8, is_active = ?9 WHERE id = ?10",
            params![entity_type, category, attr_code, attr_name, data_type, unit, default_value, formula, is_active as i32, id],
        )?;
        Ok(())
    }

    pub fn delete_attribute_template(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM attribute_templates WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn set_entity_attribute(&self, entity_type: &str, entity_id: i64, attr_code: &str, value: Option<f64>, value_text: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO entity_attributes (entity_type, entity_id, attr_code, value, value_text) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![entity_type, entity_id, attr_code, value, value_text],
        )?;
        Ok(())
    }

    pub fn get_entity_attributes(&self, entity_type: &str, entity_id: i64) -> Result<Vec<EntityAttribute>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, entity_type, entity_id, attr_code, value, value_text, calculated, updated_at FROM entity_attributes WHERE entity_type = ?1 AND entity_id = ?2")?;
        let attrs = stmt.query_map(params![entity_type, entity_id], |row| {
            Ok(EntityAttribute { id: row.get(0)?, entity_type: row.get(1)?, entity_id: row.get(2)?, attr_code: row.get(3)?, value: row.get(4)?, value_text: row.get(5)?, calculated: row.get::<_, i32>(6)? != 0, updated_at: row.get(7)? })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(attrs)
    }


}
