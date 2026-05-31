use rusqlite::{params, OptionalExtension, Result};
use super::*;

impl Database {
    pub fn get_recipes(&self, recipe_type: Option<&str>) -> Result<Vec<Recipe>> {
        let conn = self.conn.lock().unwrap();
        let query = if let Some(_rt) = recipe_type {
            "SELECT id, code, name, recipe_type, output_material_id, output_state_id, output_qty, output_unit_id, cost, is_active, created_at, updated_at FROM recipes WHERE is_active = 1 AND recipe_type = ?1 ORDER BY name"
        } else {
            "SELECT id, code, name, recipe_type, output_material_id, output_state_id, output_qty, output_unit_id, cost, is_active, created_at, updated_at FROM recipes WHERE is_active = 1 ORDER BY name"
        };
        let mut stmt = conn.prepare(query)?;
        let recipes = if let Some(rt) = recipe_type {
            stmt.query_map(params![rt], |row| {
                Ok(Recipe { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, recipe_type: row.get(3)?, output_material_id: row.get(4)?, output_state_id: row.get(5)?, output_qty: row.get(6)?, output_unit_id: row.get(7)?, cost: row.get(8)?, is_active: row.get::<_, i32>(9)? != 0, created_at: row.get(10)?, updated_at: row.get(11)? })
            })?.collect::<Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |row| {
                Ok(Recipe { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, recipe_type: row.get(3)?, output_material_id: row.get(4)?, output_state_id: row.get(5)?, output_qty: row.get(6)?, output_unit_id: row.get(7)?, cost: row.get(8)?, is_active: row.get::<_, i32>(9)? != 0, created_at: row.get(10)?, updated_at: row.get(11)? })
            })?.collect::<Result<Vec<_>>>()?
        };
        Ok(recipes)
    }

    pub fn get_recipe_types(&self) -> Result<Vec<RecipeType>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, code, name, description, sort_no, is_system, is_active
             FROM recipe_types
             WHERE is_active = 1
             ORDER BY sort_no, id"
        )?;
        let items = stmt.query_map([], |row| {
            Ok(RecipeType {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                sort_no: row.get(4)?,
                is_system: row.get::<_, i32>(5)? != 0,
                is_active: row.get::<_, i32>(6)? != 0,
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(items)
    }

    pub fn get_recipe_component_types(&self, category: Option<&str>) -> Result<Vec<RecipeComponentType>> {
        let conn = self.conn.lock().unwrap();
        let sql = if category.is_some() {
            "SELECT id, code, name, description, category, sort_no, is_active FROM recipe_component_types WHERE is_active = 1 AND category = ?1 ORDER BY sort_no"
        } else {
            "SELECT id, code, name, description, category, sort_no, is_active FROM recipe_component_types WHERE is_active = 1 ORDER BY sort_no"
        };
        let mut stmt = conn.prepare(sql)?;
        let items = if let Some(cat) = category {
            stmt.query_map(params![cat], |row| {
                Ok(RecipeComponentType { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, description: row.get(3)?, category: row.get(4)?, sort_no: row.get(5)?, is_active: row.get::<_, i32>(6)? != 0 })
            })?.collect::<Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |row| {
                Ok(RecipeComponentType { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, description: row.get(3)?, category: row.get(4)?, sort_no: row.get(5)?, is_active: row.get::<_, i32>(6)? != 0 })
            })?.collect::<Result<Vec<_>>>()?
        };
        Ok(items)
    }

    pub fn get_order_component_types(&self, is_packaging: Option<bool>) -> Result<Vec<OrderComponentType>> {
        let conn = self.conn.lock().unwrap();
        let sql = match is_packaging {
            Some(true) => "SELECT id, code, name, description, is_packaging, cost_per_unit, unit, sort_no, is_active FROM order_component_types WHERE is_active = 1 AND is_packaging = 1 ORDER BY sort_no",
            Some(false) => "SELECT id, code, name, description, is_packaging, cost_per_unit, unit, sort_no, is_active FROM order_component_types WHERE is_active = 1 AND is_packaging = 0 ORDER BY sort_no",
            None => "SELECT id, code, name, description, is_packaging, cost_per_unit, unit, sort_no, is_active FROM order_component_types WHERE is_active = 1 ORDER BY sort_no",
        };
        let mut stmt = conn.prepare(sql)?;
        let items: Vec<OrderComponentType> = stmt.query_map([], |row| {
            Ok(OrderComponentType { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, description: row.get(3)?, is_packaging: row.get::<_, i32>(4)? != 0, cost_per_unit: row.get(5)?, unit: row.get(6)?, sort_no: row.get(7)?, is_active: row.get::<_, i32>(8)? != 0 })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(items)
    }

    pub fn create_recipe_type(&self, code: &str, name: &str, description: Option<&str>, sort_no: i32) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO recipe_types (code, name, description, sort_no, is_system, is_active)
             VALUES (?1, ?2, ?3, ?4, 0, 1)",
            params![code.trim(), name.trim(), description, sort_no],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn generate_recipe_code(&self) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let max_num: Option<i64> = conn.query_row(
            "SELECT MAX(CAST(SUBSTR(code, 4) AS INTEGER)) FROM recipes WHERE code GLOB 'RCP[0-9]*'",
            [],
            |row| row.get(0),
        ).ok();

        Ok(format!("RCP{:03}", max_num.unwrap_or(0) + 1))
    }

    pub fn seed_sample_recipes(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let lookup_material_id = |code: &str| -> Result<i64> {
            conn.query_row(
                "SELECT id FROM materials WHERE code = ?1 AND is_active = 1",
                params![code],
                |row| row.get(0),
            )
        };
        let lookup_unit_id = |code: &str| -> Result<i64> {
            conn.query_row(
                "SELECT id FROM units WHERE code = ?1",
                params![code],
                |row| row.get(0),
            )
        };

        let dongfengluo_id = lookup_material_id("MAT001")?;
        let maodu_id = lookup_material_id("MAT011")?;
        let xianggu_id = lookup_material_id("MAT015")?;
        let jinzhengu_id = lookup_material_id("MAT017")?;
        let fangbianmian_id = lookup_material_id("MAT021")?;
        let maladiliu_id = lookup_material_id("MAT022")?;
        let lajiaoyou_id = lookup_material_id("MAT023")?;

        let pc_unit_id = lookup_unit_id("pc")?;
        let kg_unit_id = lookup_unit_id("kg")?;
        let l_unit_id = lookup_unit_id("l")?;

        conn.execute_batch("BEGIN")?;
        let result: Result<()> = (|| {
            conn.execute(
                "DELETE FROM recipe_items
                 WHERE recipe_id IN (SELECT id FROM recipes WHERE code IN ('RCP001', 'RCP002', 'RCP003'))",
                [],
            )?;
            conn.execute(
                "DELETE FROM recipes WHERE code IN ('RCP001', 'RCP002', 'RCP003')",
                [],
            )?;

            // 菜品組成結構（Recipe Component Types）
            let recipe_components = [
                ("料油", "lsy", "base", "油脂基底，玉米油+香辛料熬製"),
                ("調味蚝油", "dthy", "base", "蚝油+水+調味料攪拌"),
                ("醬料", "jiangliao", "base", "辣椒+麻椒+花椒+調味料熬製"),
                ("食材", "shicai", "main", "主要原材料"),
                ("調味品", "tiaoweipin", "main", "鹽、糖、醬油等調料"),
                ("餐盒", "canhe", "packaging", "包裝容器"),
            ];
            for (name, code, cat, desc) in recipe_components {
                conn.execute("INSERT OR IGNORE INTO recipe_component_types (code, name, category, description) VALUES (?1, ?2, ?3, ?4)", params![code, name, cat, desc])?;
            }

            // 訂單組成結構（Order Component Types）
            let order_components = [
                ("外包裝袋", "waibao", false, 0.5, "個"),
                ("保溫袋", "baowen", false, 1.0, "個"),
                ("手套", "shoutao", false, 0.1, "雙"),
                ("牙籤", "yaci", false, 0.05, "支"),
                ("筷子", "kuaizi", false, 0.2, "雙"),
                ("芝麻", "zhima", false, 0.3, "克"),
                ("垃圾袋", "lajidai", false, 0.2, "個"),
                ("配送費", "peisong", false, 5.0, "元"),
            ];
            for (name, code, is_pkg, cost, unit) in order_components {
                conn.execute("INSERT OR IGNORE INTO order_component_types (code, name, is_packaging, cost_per_unit, unit) VALUES (?1, ?2, ?3, ?4, ?5)", params![code, name, is_pkg, cost, unit])?;
            }

            conn.execute(
                "INSERT INTO recipes (code, name, recipe_type, output_qty) VALUES
                    ('RCP001', '麻辣东风螺', 'menu', 1.0),
                    ('RCP002', '麻辣毛肚', 'menu', 1.0),
                    ('RCP003', '麻辣方便面', 'menu', 1.0)",
                [],
            )?;

            let recipe1_id: i64 = conn.query_row(
                "SELECT id FROM recipes WHERE code = 'RCP001'",
                [],
                |row| row.get(0),
            )?;
            let recipe2_id: i64 = conn.query_row(
                "SELECT id FROM recipes WHERE code = 'RCP002'",
                [],
                |row| row.get(0),
            )?;
            let recipe3_id: i64 = conn.query_row(
                "SELECT id FROM recipes WHERE code = 'RCP003'",
                [],
                |row| row.get(0),
            )?;

            conn.execute(
                "INSERT INTO recipe_items (recipe_id, item_type, ref_id, qty, unit_id, wastage_rate, sort_no) VALUES
                    (?1, 'material', ?2, 0.5, ?3, 0.05, 1),
                    (?1, 'material', ?4, 0.1, ?3, 0, 2),
                    (?1, 'material', ?5, 0.02, ?6, 0, 3)",
                params![recipe1_id, dongfengluo_id, kg_unit_id, maladiliu_id, lajiaoyou_id, l_unit_id],
            )?;

            conn.execute(
                "INSERT INTO recipe_items (recipe_id, item_type, ref_id, qty, unit_id, wastage_rate, sort_no) VALUES
                    (?1, 'material', ?2, 0.3, ?3, 0.1, 1),
                    (?1, 'material', ?4, 0.08, ?3, 0, 2),
                    (?1, 'material', ?5, 0.015, ?6, 0, 3),
                    (?1, 'material', ?7, 0.05, ?3, 0.1, 4)",
                params![recipe2_id, maodu_id, kg_unit_id, maladiliu_id, lajiaoyou_id, l_unit_id, xianggu_id],
            )?;

            conn.execute(
                "INSERT INTO recipe_items (recipe_id, item_type, ref_id, qty, unit_id, wastage_rate, sort_no) VALUES
                    (?1, 'material', ?2, 1.0, ?3, 0, 1),
                    (?1, 'material', ?4, 0.05, ?5, 0, 2),
                    (?1, 'material', ?6, 0.01, ?7, 0, 3),
                    (?1, 'material', ?8, 0.03, ?5, 0.1, 4)",
                params![recipe3_id, fangbianmian_id, pc_unit_id, maladiliu_id, kg_unit_id, lajiaoyou_id, l_unit_id, jinzhengu_id],
            )?;

            conn.execute(
                "UPDATE menu_items
                 SET recipe_id = CASE code
                   WHEN 'MENU001' THEN ?1
                   WHEN 'MENU011' THEN ?2
                   WHEN 'MENU021' THEN ?3
                   ELSE recipe_id
                 END
                 WHERE code IN ('MENU001', 'MENU011', 'MENU021')",
                params![recipe1_id, recipe2_id, recipe3_id],
            )?;

            Ok(())
        })();
        match result {
            Ok(_) => { conn.execute_batch("COMMIT")?; Ok(()) }
            Err(e) => { conn.execute_batch("ROLLBACK").ok(); Err(e) }
        }
    }

    pub fn get_recipe_with_items(&self, recipe_id: i64) -> Result<RecipeWithItems> {
        let conn = self.conn.lock().unwrap();
        let recipe = conn.query_row(
            "SELECT id, code, name, recipe_type, output_material_id, output_state_id, output_qty, output_unit_id, cost, is_active, created_at, updated_at FROM recipes WHERE id = ?1",
            params![recipe_id],
            |row| Ok(Recipe { id: row.get(0)?, code: row.get(1)?, name: row.get(2)?, recipe_type: row.get(3)?, output_material_id: row.get(4)?, output_state_id: row.get(5)?, output_qty: row.get(6)?, output_unit_id: row.get(7)?, cost: row.get(8)?, is_active: row.get::<_, i32>(9)? != 0, created_at: row.get(10)?, updated_at: row.get(11)? })
        )?;
        let mut stmt = conn.prepare("SELECT id, recipe_id, item_type, ref_id, qty, unit_id, wastage_rate, note, sort_no FROM recipe_items WHERE recipe_id = ?1 ORDER BY sort_no, id")?;
        let items = stmt.query_map(params![recipe_id], |row| {
            Ok(RecipeItem { id: row.get(0)?, recipe_id: row.get(1)?, item_type: row.get(2)?, ref_id: row.get(3)?, qty: row.get(4)?, unit_id: row.get(5)?, wastage_rate: row.get(6)?, note: row.get(7)?, sort_no: row.get(8)? })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(RecipeWithItems { recipe, items })
    }

    pub fn create_recipe(&self, code: &str, name: &str, recipe_type: &str, output_qty: f64, output_material_id: Option<i64>, output_state_id: Option<i64>, output_unit_id: Option<i64>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            return Err(rusqlite::Error::InvalidParameterName("配方名稱不能為空".to_string()));
        }

        let trimmed_code = code.trim();
        for _attempt in 0..5 {
            let candidate_code = if trimmed_code.is_empty() {
                let max_num: i64 = conn.query_row(
                    "SELECT COALESCE(MAX(CAST(SUBSTR(code, 4) AS INTEGER)), 0) FROM recipes WHERE code GLOB 'RCP[0-9]*'",
                    [],
                    |row| row.get(0),
                )?;
                format!("RCP{:03}", max_num + 1)
            } else {
                trimmed_code.to_string()
            };

            let result = conn.execute(
                "INSERT INTO recipes (code, name, recipe_type, output_qty, output_material_id, output_state_id, output_unit_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![candidate_code, trimmed_name, recipe_type, output_qty, output_material_id, output_state_id, output_unit_id],
            );

            match result {
                Ok(_) => return Ok(conn.last_insert_rowid()),
                Err(err) => {
                    let is_unique_code_error =
                        err.to_string().contains("UNIQUE constraint failed: recipes.code");

                    if is_unique_code_error && (trimmed_code.is_empty() || trimmed_code.starts_with("RCP")) {
                        continue;
                    }

                    return Err(err.into());
                }
            }
        }

        Err(rusqlite::Error::InvalidParameterName("生成配方編號失敗，請重試".to_string()))
    }

    pub fn add_recipe_item(&self, recipe_id: i64, item_type: &str, ref_id: i64, qty: f64, unit_id: i64, wastage_rate: f64, sort_no: i32) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        if item_type == "sub_recipe" {
            if ref_id == recipe_id {
                return Err(rusqlite::Error::InvalidParameterName(
                    "不能將配方添加為自身的子配方".to_string(),
                ));
            }
            if Self::recipe_has_ancestor(&conn, ref_id, recipe_id)? {
                return Err(rusqlite::Error::InvalidParameterName(
                    "添加此子配方會產生循環引用，請檢查配方嵌套關係".to_string(),
                ));
            }
        }
        if item_type == "material" {
            let base_unit_type: Option<String> = conn.query_row(
                "SELECT u.unit_type FROM materials m JOIN units u ON m.base_unit_id = u.id WHERE m.id = ?1",
                params![ref_id],
                |row| row.get(0),
            ).optional()?;
            let selected_unit_type: Option<String> = conn.query_row(
                "SELECT unit_type FROM units WHERE id = ?1",
                params![unit_id],
                |row| row.get(0),
            ).optional()?;
            match (base_unit_type, selected_unit_type) {
                (Some(base), Some(sel)) if base != sel => {
                    return Err(rusqlite::Error::InvalidParameterName(
                        format!("單位類型不兼容：材料基準單位類型為「{}」，所選單位類型為「{}」", base, sel),
                    ));
                }
                _ => {}
            }
        }
        conn.execute(
            "INSERT INTO recipe_items (recipe_id, item_type, ref_id, qty, unit_id, wastage_rate, sort_no) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![recipe_id, item_type, ref_id, qty, unit_id, wastage_rate, sort_no],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// BFS: returns true if `ancestor_id` is reachable via sub_recipe links starting from `recipe_id`.
    /// Depth-capped at 50 nodes to guard against pathological graphs.
    fn recipe_has_ancestor(conn: &Connection, recipe_id: i64, ancestor_id: i64) -> Result<bool> {
        use std::collections::HashSet;
        let mut visited: HashSet<i64> = HashSet::new();
        let mut queue = vec![recipe_id];
        while let Some(current) = queue.pop() {
            if current == ancestor_id { return Ok(true); }
            if visited.contains(&current) { continue; }
            visited.insert(current);
            if visited.len() > 50 { break; }
            let mut stmt = conn.prepare(
                "SELECT ref_id FROM recipe_items WHERE recipe_id = ?1 AND item_type = 'sub_recipe'"
            )?;
            let children: Vec<i64> = stmt.query_map(params![current], |row| row.get(0))?
                .collect::<Result<Vec<_>>>()?;
            queue.extend(children);
        }
        Ok(false)
    }

    pub fn get_recipe_usage_count(&self, recipe_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        // 檢查在 recipe_items 表中被作為子配方引用的次數
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM recipe_items WHERE item_type = 'sub_recipe' AND ref_id = ?1",
            params![recipe_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn would_create_recipe_cycle(&self, recipe_id: i64, ref_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        if ref_id == recipe_id {
            return Ok(true);
        }
        Self::recipe_has_ancestor(&conn, ref_id, recipe_id)
    }

    pub fn get_recipe_dependents(&self, recipe_id: i64) -> Result<(Vec<(i64, String)>, Vec<(i64, String)>)> {
        let conn = self.conn.lock().unwrap();
        // Menu items that link to this recipe
        let mut stmt = conn.prepare(
            "SELECT mi.id, mi.name FROM menu_items mi WHERE mi.recipe_id = ?1"
        )?;
        let menu_items: Vec<(i64, String)> = stmt.query_map(params![recipe_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?.filter_map(|r| r.ok()).collect();

        // Parent recipes that use this recipe as a sub-recipe
        let mut stmt2 = conn.prepare(
            "SELECT r.id, r.name FROM recipes r
             JOIN recipe_items ri ON ri.recipe_id = r.id
             WHERE ri.item_type = 'sub_recipe' AND ri.ref_id = ?1"
        )?;
        let parent_recipes: Vec<(i64, String)> = stmt2.query_map(params![recipe_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?.filter_map(|r| r.ok()).collect();

        Ok((menu_items, parent_recipes))
    }

    pub fn get_material_dependents(&self, material_id: i64) -> Result<Vec<(i64, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT DISTINCT r.id, r.name FROM recipes r
             JOIN recipe_items ri ON ri.recipe_id = r.id
             WHERE ri.item_type = 'material' AND ri.ref_id = ?1"
        )?;
        let recipes: Vec<(i64, String)> = stmt.query_map(params![material_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?.filter_map(|r| r.ok()).collect();
        Ok(recipes)
    }

    pub fn calculate_recipe_cost(&self, recipe_id: i64) -> Result<RecipeCostResult> {
        let conn = self.conn.lock().unwrap();
        let (recipe_name, output_qty): (String, f64) = conn.query_row(
            "SELECT name, output_qty FROM recipes WHERE id = ?1",
            params![recipe_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        let mut stmt = conn.prepare(
            "SELECT m.name,
                    ri.qty,
                    COALESCE(u.code, ''),
                    COALESCE((SELECT AVG(ib2.cost_per_unit) FROM inventory_batches ib2
                               WHERE ib2.material_id = ri.ref_id AND ib2.quantity > 0), 0.0),
                    ri.wastage_rate
             FROM recipe_items ri
             JOIN materials m ON m.id = ri.ref_id
             LEFT JOIN units u ON u.id = ri.unit_id
             WHERE ri.recipe_id = ?1 AND ri.item_type = 'material'
             ORDER BY ri.sort_no",
        )?;
        let mut items: Vec<RecipeCostItem> = stmt.query_map(params![recipe_id], |row| {
            let material_name: String = row.get(0)?;
            let qty: f64 = row.get(1)?;
            let unit: String = row.get(2)?;
            let cost_per_unit: f64 = row.get(3)?;
            let wastage_rate: f64 = row.get(4)?;
            let line_cost = qty * cost_per_unit * (1.0 + wastage_rate);
            Ok(RecipeCostItem { material_name, qty, unit, cost_per_unit, wastage_rate, line_cost, item_type: "material".to_string() })
        })?.collect::<Result<Vec<_>>>()?;

        // Include sub-recipe cost contributions (one level, using their material avg costs)
        let mut sub_stmt = conn.prepare(
            "SELECT r.name,
                    ri.qty,
                    COALESCE(u.code, ''),
                    ri.wastage_rate,
                    r.output_qty,
                    COALESCE((
                        SELECT SUM(ri2.qty *
                            COALESCE((SELECT AVG(ib.cost_per_unit) FROM inventory_batches ib
                                      WHERE ib.material_id = ri2.ref_id AND ib.quantity > 0), 0.0)
                            * (1.0 + ri2.wastage_rate))
                        FROM recipe_items ri2
                        WHERE ri2.recipe_id = r.id AND ri2.item_type = 'material'
                    ), 0.0)
             FROM recipe_items ri
             JOIN recipes r ON r.id = ri.ref_id
             LEFT JOIN units u ON u.id = ri.unit_id
             WHERE ri.recipe_id = ?1 AND ri.item_type = 'sub_recipe'
             ORDER BY ri.sort_no"
        )?;
        let sub_items: Vec<RecipeCostItem> = sub_stmt.query_map(params![recipe_id], |row| {
            let material_name: String = row.get(0)?;
            let qty: f64 = row.get(1)?;
            let unit: String = row.get(2)?;
            let wastage_rate: f64 = row.get(3)?;
            let sub_output_qty: f64 = row.get(4)?;
            let sub_total_cost: f64 = row.get(5)?;
            let cost_per_unit = if sub_output_qty > 0.0 { sub_total_cost / sub_output_qty } else { 0.0 };
            let line_cost = qty * cost_per_unit * (1.0 + wastage_rate);
            Ok(RecipeCostItem { material_name, qty, unit, cost_per_unit, wastage_rate, line_cost, item_type: "sub_recipe".to_string() })
        })?.collect::<Result<Vec<_>>>()?;
        items.extend(sub_items);

        let total_cost: f64 = items.iter().map(|i| i.line_cost).sum();
        let cost_per_unit = if output_qty > 0.0 { total_cost / output_qty } else { 0.0 };
        Ok(RecipeCostResult { recipe_id, recipe_name, total_cost, cost_per_unit, output_qty, items })
    }


}
