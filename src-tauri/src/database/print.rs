use rusqlite::{params, Result};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use super::*;
use chrono::{self, Datelike};

// ==================== 打印創意模組靜態資源 ====================

static FORTUNE_TEXTS: &[(&str, &str)] = &[
    ("大吉", "今日萬事俱備，美食開路，好運隨行。"),
    ("大吉", "口福即天福，飽食者心寬，心寬者天下大吉。"),
    ("大吉", "今日食得好菜，出門定遇貴人，萬事如意。"),
    ("大吉", "此番點單合天意，財運、健康、緣分三喜臨門。"),
    ("大吉", "今日吉星高照，用餐愉快，好事接連而至。"),
    ("中吉", "菜香心靜，凡事不急，好事自來。"),
    ("中吉", "今日宜慢食慢行，細品生活每一味。"),
    ("中吉", "飯吃七分飽，事做三分穩，中吉福報至。"),
    ("中吉", "今日偏財運旺，不妨飯後小試，或有驚喜。"),
    ("中吉", "心情如湯底，需要文火慢煨，今日宜沉澱。"),
    ("小吉", "小吉已是福，知足者常樂，今日享受當下。"),
    ("小吉", "凡事稍作等候，如等上菜，值得的都值得等。"),
    ("小吉", "今日宜靜不宜動，食飽小憩，養精蓄銳。"),
    ("小吉", "小吉亦是吉，今日腳步可緩，細品身邊美好。"),
    ("小吉", "平穩是福，今日安步當車，無驚無險皆好事。"),
];

static QUOTES_ZH: &[&str] = &[
    "人間有味是清歡 — 蘇軾",
    "莫笑農家臘酒渾，豐年留客足雞豚 — 陸游",
    "此刻此味，是最好的時刻",
    "食之以誠，暖之以心",
    "舉杯邀明月，對影成三人 — 李白",
];

static QUOTES_EN: &[&str] = &[
    "Good food is the foundation of genuine happiness.",
    "Life is short. Eat the good stuff first.",
    "You had me at the menu.",
    "Every meal is a love letter to your body.",
    "Eat well. Live well. Repeat.",
];

static QUOTES_JA: &[&str] = &[
    "食べることは生きること、愛すること",
    "一碗の温もり、心に満ちる幸せ",
    "美味しさは言葉を超える",
];

static ART_BLOCKS: &[&str] = &[
    "  ╔══════════════════════╗\n  ║  ( ˘◡˘ )♪  用心料理  ║\n  ╚══════════════════════╝",
    "  ☆ ☆ ☆  LUCKY RECEIPT  ☆ ☆ ☆\n  ／￣＼\n （  °▽°）  感謝光臨！\n  ＼＿／",
    "  /ᐠ｡ꞈ｡ᐟ\\  感謝惠顧！\n  ♪ 布穀！布穀！ ♪",
    "  ʕ•ᴥ•ʔ  吃飽了嗎？\n  ☕ ☕ ☕ ☕ ☕ ☕",
    "  ✿ ✿ ✿  用心烹飪  ✿ ✿ ✿",
];

pub(crate) fn creative_fortune_seed(strategy: &str, table_no: Option<&str>, order_id: Option<i64>, date_str: &str) -> u64 {
    let mut h = DefaultHasher::new();
    match strategy {
        "per_table" => { table_no.unwrap_or("").hash(&mut h); date_str.hash(&mut h); }
        "per_order" => { order_id.unwrap_or(0).hash(&mut h); }
        _ => { date_str.hash(&mut h); }
    }
    h.finish()
}

impl Database {
    // ==================== 打印模板 ====================

    pub fn get_print_templates(&self, template_type: Option<&str>) -> Result<Vec<PrintTemplate>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, template_type, paper_size, label_width_mm, label_height_mm, content, is_default, is_active, theme, restaurant_name, tagline, logo_data, show_price, show_tax, show_service_charge, item_sort, modifiers_color, created_at, updated_at FROM print_templates WHERE is_active = 1 ORDER BY is_default DESC, name")?;
        let templates: Vec<PrintTemplate> = stmt.query_map([], |row| {
            Ok(PrintTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                template_type: row.get(2)?,
                paper_size: row.get(3)?,
                label_width_mm: row.get(4)?,
                label_height_mm: row.get(5)?,
                content: row.get(6)?,
                is_default: { let v: i64 = row.get(7)?; v == 1 },
                is_active: { let v: i64 = row.get(8)?; v == 1 },
                theme: row.get(9)?,
                restaurant_name: row.get(10)?,
                tagline: row.get(11)?,
                logo_data: row.get(12)?,
                show_price: row.get::<_, Option<i64>>(13)?.map(|v| v == 1),
                show_tax: row.get::<_, Option<i64>>(14)?.map(|v| v == 1),
                show_service_charge: row.get::<_, Option<i64>>(15)?.map(|v| v == 1),
                item_sort: row.get(16)?,
                modifiers_color: row.get(17)?,
                created_at: row.get(18)?,
                updated_at: row.get(19)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        if let Some(t) = template_type {
            Ok(templates.into_iter().filter(|tpl| tpl.template_type == t).collect())
        } else {
            Ok(templates)
        }
    }

    pub fn get_print_template(&self, id: i64) -> Result<PrintTemplate> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT id, name, template_type, paper_size, label_width_mm, label_height_mm, content, is_default, is_active, theme, restaurant_name, tagline, logo_data, show_price, show_tax, show_service_charge, item_sort, modifiers_color, created_at, updated_at FROM print_templates WHERE id = ?1", params![id], |row| {
            Ok(PrintTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                template_type: row.get(2)?,
                paper_size: row.get(3)?,
                label_width_mm: row.get(4)?,
                label_height_mm: row.get(5)?,
                content: row.get(6)?,
                is_default: { let v: i64 = row.get(7)?; v == 1 },
                is_active: { let v: i64 = row.get(8)?; v == 1 },
                theme: row.get(9)?,
                restaurant_name: row.get(10)?,
                tagline: row.get(11)?,
                logo_data: row.get(12)?,
                show_price: row.get::<_, Option<i64>>(13)?.map(|v| v == 1),
                show_tax: row.get::<_, Option<i64>>(14)?.map(|v| v == 1),
                show_service_charge: row.get::<_, Option<i64>>(15)?.map(|v| v == 1),
                item_sort: row.get(16)?,
                modifiers_color: row.get(17)?,
                created_at: row.get(18)?,
                updated_at: row.get(19)?,
            })
        })
    }

    pub fn create_print_template(&self, req: &CreatePrintTemplateRequest) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT INTO print_templates (name, template_type, paper_size, label_width_mm, label_height_mm, content, theme, restaurant_name, tagline, logo_data, show_price, show_tax, show_service_charge, item_sort, modifiers_color, is_active, is_default) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, 0)", params![
            req.name, req.template_type, req.paper_size, req.label_width_mm, req.label_height_mm, req.content,
            req.theme, req.restaurant_name, req.tagline, req.logo_data,
            req.show_price.map(|v| v as i64), req.show_tax.map(|v| v as i64), req.show_service_charge.map(|v| v as i64),
            req.item_sort, req.modifiers_color, req.is_active.map(|v| v as i64)
        ])?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_print_template(&self, id: i64, name: Option<String>, content: Option<String>, paper_size: Option<String>, label_width_mm: Option<f64>, label_height_mm: Option<f64>, theme: Option<String>, restaurant_name: Option<String>, tagline: Option<String>, logo_data: Option<String>, show_price: Option<bool>, show_tax: Option<bool>, show_service_charge: Option<bool>, item_sort: Option<String>, modifiers_color: Option<String>, is_active: Option<bool>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        if let Some(n) = name { conn.execute("UPDATE print_templates SET name = ?1, updated_at = datetime('now') WHERE id = ?2", params![n, id])?; }
        if let Some(c) = content { conn.execute("UPDATE print_templates SET content = ?1, updated_at = datetime('now') WHERE id = ?2", params![c, id])?; }
        if let Some(p) = paper_size { conn.execute("UPDATE print_templates SET paper_size = ?1, updated_at = datetime('now') WHERE id = ?2", params![p, id])?; }
        if let Some(w) = label_width_mm { conn.execute("UPDATE print_templates SET label_width_mm = ?1, updated_at = datetime('now') WHERE id = ?2", params![w, id])?; }
        if let Some(h) = label_height_mm { conn.execute("UPDATE print_templates SET label_height_mm = ?1, updated_at = datetime('now') WHERE id = ?2", params![h, id])?; }
        if let Some(t) = theme { conn.execute("UPDATE print_templates SET theme = ?1, updated_at = datetime('now') WHERE id = ?2", params![t, id])?; }
        if let Some(rn) = restaurant_name { conn.execute("UPDATE print_templates SET restaurant_name = ?1, updated_at = datetime('now') WHERE id = ?2", params![rn, id])?; }
        if let Some(tl) = tagline { conn.execute("UPDATE print_templates SET tagline = ?1, updated_at = datetime('now') WHERE id = ?2", params![tl, id])?; }
        if let Some(ld) = logo_data { conn.execute("UPDATE print_templates SET logo_data = ?1, updated_at = datetime('now') WHERE id = ?2", params![ld, id])?; }
        if let Some(sp) = show_price { conn.execute("UPDATE print_templates SET show_price = ?1, updated_at = datetime('now') WHERE id = ?2", params![sp as i64, id])?; }
        if let Some(st) = show_tax { conn.execute("UPDATE print_templates SET show_tax = ?1, updated_at = datetime('now') WHERE id = ?2", params![st as i64, id])?; }
        if let Some(ss) = show_service_charge { conn.execute("UPDATE print_templates SET show_service_charge = ?1, updated_at = datetime('now') WHERE id = ?2", params![ss as i64, id])?; }
        if let Some(is) = item_sort { conn.execute("UPDATE print_templates SET item_sort = ?1, updated_at = datetime('now') WHERE id = ?2", params![is, id])?; }
        if let Some(mc) = modifiers_color { conn.execute("UPDATE print_templates SET modifiers_color = ?1, updated_at = datetime('now') WHERE id = ?2", params![mc, id])?; }
        if let Some(ia) = is_active { conn.execute("UPDATE print_templates SET is_active = ?1, updated_at = datetime('now') WHERE id = ?2", params![ia as i64, id])?; }
        Ok(())
    }

    pub fn delete_print_template(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE print_templates SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn set_default_template(&self, id: i64, template_type: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE print_templates SET is_default = 0 WHERE template_type = ?1", params![template_type])?;
        conn.execute("UPDATE print_templates SET is_default = 1 WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ==================== 票據類型 ====================

    pub fn get_print_ticket_types(&self) -> Result<Vec<PrintTicketType>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, code, name, description, is_active, is_default, show_price, show_seq, show_note_field, station_id, paper_width, font_size, cut_mode, print_speed, print_density, show_order_no, show_table_no, show_dine_type, show_item_name, show_item_qty, show_item_price, show_item_subtotal, show_item_spec, show_item_note, show_created_at, show_total_amount, show_lot_no, show_qty_info, show_expiry_date, show_supplier, created_at, updated_at FROM print_ticket_types ORDER BY is_default DESC, name")?;
        let types: Vec<PrintTicketType> = stmt.query_map([], |row| {
            Ok(PrintTicketType {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                is_active: { let v: i64 = row.get(4)?; v == 1 },
                is_default: { let v: i64 = row.get(5)?; v == 1 },
                show_price: { let v: i64 = row.get(6)?; v == 1 },
                show_seq: { let v: i64 = row.get(7)?; v == 1 },
                show_note_field: { let v: i64 = row.get(8)?; v == 1 },
                station_id: row.get(9)?,
                paper_width: row.get(10)?,
                font_size: row.get(11)?,
                cut_mode: row.get(12)?,
                print_speed: row.get(13)?,
                print_density: row.get(14)?,
                show_order_no: { let v: i64 = row.get(15)?; v == 1 },
                show_table_no: { let v: i64 = row.get(16)?; v == 1 },
                show_dine_type: { let v: i64 = row.get(17)?; v == 1 },
                show_item_name: { let v: i64 = row.get(18)?; v == 1 },
                show_item_qty: { let v: i64 = row.get(19)?; v == 1 },
                show_item_price: { let v: i64 = row.get(20)?; v == 1 },
                show_item_subtotal: { let v: i64 = row.get(21)?; v == 1 },
                show_item_spec: { let v: i64 = row.get(22)?; v == 1 },
                show_item_note: { let v: i64 = row.get(23)?; v == 1 },
                show_created_at: { let v: i64 = row.get(24)?; v == 1 },
                show_total_amount: { let v: i64 = row.get(25)?; v == 1 },
                show_lot_no: { let v: i64 = row.get(26)?; v == 1 },
                show_qty_info: { let v: i64 = row.get(27)?; v == 1 },
                show_expiry_date: { let v: i64 = row.get(28)?; v == 1 },
                show_supplier: { let v: i64 = row.get(29)?; v == 1 },
                created_at: row.get(30)?,
                updated_at: row.get(31)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(types)
    }

    pub fn get_print_ticket_type(&self, id: i64) -> Result<PrintTicketType> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT id, code, name, description, is_active, is_default, show_price, show_seq, show_note_field, station_id, paper_width, font_size, cut_mode, print_speed, print_density, show_order_no, show_table_no, show_dine_type, show_item_name, show_item_qty, show_item_price, show_item_subtotal, show_item_spec, show_item_note, show_created_at, show_total_amount, show_lot_no, show_qty_info, show_expiry_date, show_supplier, created_at, updated_at FROM print_ticket_types WHERE id = ?1", params![id], |row| {
            Ok(PrintTicketType {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                is_active: { let v: i64 = row.get(4)?; v == 1 },
                is_default: { let v: i64 = row.get(5)?; v == 1 },
                show_price: { let v: i64 = row.get(6)?; v == 1 },
                show_seq: { let v: i64 = row.get(7)?; v == 1 },
                show_note_field: { let v: i64 = row.get(8)?; v == 1 },
                station_id: row.get(9)?,
                paper_width: row.get(10)?,
                font_size: row.get(11)?,
                cut_mode: row.get(12)?,
                print_speed: row.get(13)?,
                print_density: row.get(14)?,
                show_order_no: { let v: i64 = row.get(15)?; v == 1 },
                show_table_no: { let v: i64 = row.get(16)?; v == 1 },
                show_dine_type: { let v: i64 = row.get(17)?; v == 1 },
                show_item_name: { let v: i64 = row.get(18)?; v == 1 },
                show_item_qty: { let v: i64 = row.get(19)?; v == 1 },
                show_item_price: { let v: i64 = row.get(20)?; v == 1 },
                show_item_subtotal: { let v: i64 = row.get(21)?; v == 1 },
                show_item_spec: { let v: i64 = row.get(22)?; v == 1 },
                show_item_note: { let v: i64 = row.get(23)?; v == 1 },
                show_created_at: { let v: i64 = row.get(24)?; v == 1 },
                show_total_amount: { let v: i64 = row.get(25)?; v == 1 },
                show_lot_no: { let v: i64 = row.get(26)?; v == 1 },
                show_qty_info: { let v: i64 = row.get(27)?; v == 1 },
                show_expiry_date: { let v: i64 = row.get(28)?; v == 1 },
                show_supplier: { let v: i64 = row.get(29)?; v == 1 },
                created_at: row.get(30)?,
                updated_at: row.get(31)?,
            })
        })
    }

    pub fn create_print_ticket_type(&self, req: &CreatePrintTicketTypeRequest) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO print_ticket_types (code, name, description, is_active, is_default, show_price, show_seq, show_note_field, station_id, paper_width, font_size, cut_mode, print_speed, print_density, show_order_no, show_table_no, show_dine_type, show_item_name, show_item_qty, show_item_price, show_item_subtotal, show_item_spec, show_item_note, show_created_at, show_total_amount, show_lot_no, show_qty_info, show_expiry_date, show_supplier) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29)",
            params![
                req.code, req.name, req.description, req.is_active as i64, req.is_default as i64,
                req.show_price as i64, req.show_seq as i64, req.show_note_field as i64, req.station_id,
                req.paper_width, req.font_size, req.cut_mode, req.print_speed, req.print_density,
                req.show_order_no as i64, req.show_table_no as i64, req.show_dine_type as i64,
                req.show_item_name as i64, req.show_item_qty as i64, req.show_item_price as i64,
                req.show_item_subtotal as i64, req.show_item_spec as i64, req.show_item_note as i64,
                req.show_created_at as i64, req.show_total_amount as i64,
                req.show_lot_no as i64, req.show_qty_info as i64, req.show_expiry_date as i64, req.show_supplier as i64
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_print_ticket_type(&self, id: i64, req: &UpdatePrintTicketTypeRequest) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE print_ticket_types SET code = ?1, name = ?2, description = ?3, is_active = ?4, is_default = ?5, show_price = ?6, show_seq = ?7, show_note_field = ?8, station_id = ?9, paper_width = ?10, font_size = ?11, cut_mode = ?12, print_speed = ?13, print_density = ?14, show_order_no = ?15, show_table_no = ?16, show_dine_type = ?17, show_item_name = ?18, show_item_qty = ?19, show_item_price = ?20, show_item_subtotal = ?21, show_item_spec = ?22, show_item_note = ?23, show_created_at = ?24, show_total_amount = ?25, show_lot_no = ?26, show_qty_info = ?27, show_expiry_date = ?28, show_supplier = ?29, updated_at = datetime('now') WHERE id = ?30",
            params![
                req.code, req.name, req.description, req.is_active as i64, req.is_default as i64, req.show_price as i64, req.show_seq as i64,
                req.show_note_field as i64, req.station_id, req.paper_width, req.font_size, req.cut_mode,
                req.print_speed, req.print_density, req.show_order_no as i64, req.show_table_no as i64,
                req.show_dine_type as i64, req.show_item_name as i64, req.show_item_qty as i64,
                req.show_item_price as i64, req.show_item_subtotal as i64, req.show_item_spec as i64,
                req.show_item_note as i64, req.show_created_at as i64, req.show_total_amount as i64,
                req.show_lot_no as i64, req.show_qty_info as i64, req.show_expiry_date as i64, req.show_supplier as i64, id
            ],
        )?;
        Ok(())
    }

    pub fn delete_print_ticket_type(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM print_ticket_types WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn set_default_ticket_type(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;
        tx.execute("UPDATE print_ticket_types SET is_default = 0", [])?;
        tx.execute("UPDATE print_ticket_types SET is_default = 1 WHERE id = ?1", params![id])?;
        tx.commit()?;
        Ok(())
    }

    pub fn ensure_default_ticket_types(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM print_ticket_types", [], |row| row.get(0))?;
        if count == 0 {
            conn.execute(
                "INSERT INTO print_ticket_types (code, name, description, is_active, is_default, show_price, show_seq, show_note_field, station_id, paper_width, font_size, cut_mode, print_speed, print_density, show_order_no, show_table_no, show_dine_type, show_item_name, show_item_qty, show_item_price, show_item_subtotal, show_item_spec, show_item_note, show_created_at, show_total_amount, show_lot_no, show_qty_info, show_expiry_date, show_supplier) VALUES ('kitchen', '廚房單', '後廚備餐用', 1, 1, 0, 1, 1, NULL, '58mm', 'medium', 'full', 'medium', 'medium', 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0)",
                [],
            )?;
            conn.execute(
                "INSERT INTO print_ticket_types (code, name, description, is_active, is_default, show_price, show_seq, show_note_field, station_id, paper_width, font_size, cut_mode, print_speed, print_density, show_order_no, show_table_no, show_dine_type, show_item_name, show_item_qty, show_item_price, show_item_subtotal, show_item_spec, show_item_note, show_created_at, show_total_amount, show_lot_no, show_qty_info, show_expiry_date, show_supplier) VALUES ('receipt', '出餐單', '客人結帳用', 1, 0, 1, 0, 1, NULL, '58mm', 'medium', 'full', 'medium', 'medium', 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 0, 0, 0)",
                [],
            )?;
            conn.execute(
                "INSERT INTO print_ticket_types (code, name, description, is_active, is_default, show_price, show_seq, show_note_field, station_id, paper_width, font_size, cut_mode, print_speed, print_density, show_order_no, show_table_no, show_dine_type, show_item_name, show_item_qty, show_item_price, show_item_subtotal, show_item_spec, show_item_note, show_created_at, show_total_amount, show_lot_no, show_qty_info, show_expiry_date, show_supplier) VALUES ('label', '批次標籤', '庫存標識用', 1, 0, 0, 0, 0, NULL, '50mm', 'small', 'full', 'medium', 'medium', 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1)",
                [],
            )?;
        }
        Ok(())
    }

    // ==================== 模板引擎 ====================

    pub fn render_template_content_preview(
        &self,
        content: &str,
        paper_size: &str,
        _theme: &str,
        restaurant_name: &str,
        tagline: &str,
        logo_data: Option<&str>,
        data: &serde_json::Value,
    ) -> Result<PrintPreviewResult> {
        let template: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?;

        let mut lines: Vec<String> = Vec::new();
        if let Some(elements) = template.get("elements").and_then(|e| e.as_array()) {
            for elem in elements {
                if let Some(kind) = elem.get("type").and_then(|t| t.as_str()) {
                    match kind {
                        "text" => {
                            let content_str = elem.get("content").and_then(|c| c.as_str()).unwrap_or("");
                            let rendered = self.interpolate_template_string(content_str, data);
                            let align = elem.get("align").and_then(|a| a.as_str()).unwrap_or("left");
                            let bold = elem.get("bold").and_then(|b| b.as_bool()).unwrap_or(false);
                            let size = elem.get("size").and_then(|s| s.as_str()).unwrap_or("normal");
                            let width = if paper_size == "58mm" { 32 } else { 48 };
                            let mut line = rendered;
                            if align == "center" {
                                let pad = if width > line.chars().count() { (width - line.chars().count()) / 2 } else { 0 };
                                line = " ".repeat(pad) + &line;
                            } else if align == "right" {
                                let pad = if width > line.chars().count() { width - line.chars().count() } else { 0 };
                                line = " ".repeat(pad) + &line;
                            }
                            let prefix = if bold { "**" } else { "" };
                            let suffix = if bold { "**" } else { "" };
                            let size_prefix = match size { "large" => "##", "small" => "", _ => "" };
                            lines.push(format!("{}{}{}{}", size_prefix, prefix, line, suffix));
                        }
                        "separator" => {
                            let width = if paper_size == "58mm" { 32 } else { 48 };
                            lines.push("─".repeat(width));
                        }
                        "blank_lines" => {
                            let count = elem.get("count").and_then(|c| c.as_u64()).unwrap_or(1);
                            for _ in 0..count { lines.push(String::new()); }
                        }
                        "items" => {
                            if let Some(items) = data.get("items").and_then(|i| i.as_array()) {
                                for item in items {
                                    let name = item.get("name").and_then(|n| n.as_str()).unwrap_or("");
                                    let qty = item.get("qty").and_then(|q| q.as_f64()).unwrap_or(1.0);
                                    let note = item.get("note").and_then(|n| n.as_str());
                                    let item_line = format!("{} x{}", name, qty as i64);
                                    lines.push(item_line);
                                    if let Some(n) = note {
                                        let note_line = format!("  備註: {}", n);
                                        lines.push(note_line);
                                    }
                                }
                            }
                        }
                        "fortune" => {
                            let strategy = elem.get("seed_strategy").and_then(|s| s.as_str()).unwrap_or("daily");
                            let date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
                            let table_no = data.get("table_no").and_then(|t| t.as_str());
                            let order_id = data.get("order_id").and_then(|o| o.as_i64());
                            let seed = creative_fortune_seed(strategy, table_no, order_id, &date_str);
                            let pct = seed % 100;
                            let level = if pct < 20 { "小吉" } else if pct < 70 { "中吉" } else { "大吉" };
                            let level_texts: Vec<&str> = FORTUNE_TEXTS.iter()
                                .filter(|(l, _)| *l == level).map(|(_, t)| *t).collect();
                            let fortune_text = level_texts.get(((seed / 100) as usize) % level_texts.len().max(1))
                                .copied().unwrap_or("");
                            let stars = if pct < 20 { "★" } else if pct < 70 { "★ ★" } else { "★ ★ ★" };
                            let width = if paper_size == "58mm" { 32 } else { 48 };
                            let dash = "─".repeat(width);
                            lines.push(dash.clone());
                            lines.push(format!("       {} {} {} {}       ", stars, level, level, stars));
                            lines.push(fortune_text.to_string());
                            lines.push(dash);
                        }
                        "quote" => {
                            let lang = elem.get("language").and_then(|l| l.as_str()).unwrap_or("multilingual");
                            let day_seed = chrono::Local::now().format("%Y%m%d").to_string().parse::<u64>().unwrap_or(0);
                            let quotes: &[&str] = match lang {
                                "en" => QUOTES_EN,
                                "ja" => QUOTES_JA,
                                "zh" => QUOTES_ZH,
                                _ => match day_seed % 3 { 0 => QUOTES_ZH, 1 => QUOTES_EN, _ => QUOTES_JA },
                            };
                            let quote = quotes.get((day_seed as usize) % quotes.len().max(1))
                                .copied().unwrap_or("");
                            let width = if paper_size == "58mm" { 32 } else { 48 };
                            lines.push("─".repeat(width));
                            lines.push(quote.to_string());
                            lines.push("─".repeat(width));
                        }
                        "art" => {
                            let variant = elem.get("variant").and_then(|v| v.as_str()).unwrap_or("random");
                            let idx = if variant == "random" {
                                let day_seed = chrono::Local::now().format("%Y%m%d").to_string().parse::<usize>().unwrap_or(0);
                                day_seed % ART_BLOCKS.len().max(1)
                            } else {
                                0
                            };
                            let block = ART_BLOCKS.get(idx).copied().unwrap_or("");
                            for line in block.split('\n') {
                                lines.push(line.to_string());
                            }
                        }
                        "image_block" => {
                            // Placeholder — future ESC/POS GS v 0 bitmap support
                            lines.push(String::new());
                            lines.push("  [自訂圖像]".to_string());
                            lines.push(String::new());
                        }
                        "discount_coupon" => {
                            render_discount_coupon(elem, data, paper_size, &mut lines);
                        }
                        "product_spotlight" => {
                            render_product_spotlight(elem, paper_size, &mut lines);
                        }
                        "qr_code" => {
                            render_qr_code_element(elem, paper_size, &mut lines);
                        }
                        "character_collect" => {
                            render_character_collect(elem, data, paper_size, &mut lines);
                        }
                        "rich_text" => {
                            render_rich_text(elem, paper_size, &mut lines);
                        }
                        "solar_term" => {
                            render_solar_term(elem, paper_size, &mut lines);
                        }
                        "chef_message" => {
                            render_chef_message(elem, paper_size, &mut lines);
                        }
                        "riddle" => {
                            render_riddle(elem, paper_size, &mut lines);
                        }
                        "dish_easter_egg" => {
                            render_dish_easter_egg(elem, data, paper_size, &mut lines);
                        }
                        _ => {}
                    }
                }
            }
        }

        let mut html = String::new();
        html.push_str("<div class=\"receipt-preview\" style=\"");
        html.push_str("font-family: 'Courier New', monospace; ");
        html.push_str("background: #fff; ");
        html.push_str("color: #1a1a1a; ");
        html.push_str("padding: 16px; ");
        html.push_str(&format!("max-width: {}px; ", if paper_size == "58mm" { 240 } else { 320 }));
        html.push_str("margin: 0 auto; ");
        html.push_str("border: 1px solid #e2e8f0; ");
        html.push_str("border-radius: 8px; ");
        html.push_str("\">\n");

        if !restaurant_name.is_empty() || logo_data.is_some() {
            html.push_str("<div style=\"text-align: center; margin-bottom: 12px;\">");
            if let Some(logo) = logo_data {
                if !logo.is_empty() {
                    html.push_str(&format!("<img src=\"{}\" style=\"max-height: 48px; max-width: 80px; margin-bottom: 6px;\" />", logo));
                }
            }
            if !restaurant_name.is_empty() {
                html.push_str(&format!("<div style=\"font-size: 18px; font-weight: bold;\">{}</div>", restaurant_name));
            }
            if !tagline.is_empty() {
                html.push_str(&format!("<div style=\"font-size: 11px; color: #666;\">{}</div>", tagline));
            }
            html.push_str("</div>");
            html.push_str("<div style=\"border-bottom: 1px dashed #ccc; margin: 8px 0;\"></div>");
        }

        for line in &lines {
            if line.is_empty() {
                html.push_str("<div style=\"height: 8px;\"></div>\n");
            } else if line.starts_with("##") {
                let content = line.trim_start_matches("##").trim_matches('*');
                html.push_str(&format!("<div style=\"font-size: 16px; font-weight: bold; text-align: center;\">{}</div>\n", content));
            } else if line.starts_with("**") {
                let content = line.trim_matches('*');
                html.push_str(&format!("<div style=\"font-weight: bold;\">{}</div>\n", content));
            } else {
                let escaped = line.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;");
                if line.chars().all(|c| c == '─') {
                    html.push_str("<div style=\"border-bottom: 1px dashed #ccc; margin: 6px 0;\"></div>\n");
                } else {
                    html.push_str(&format!("<div style=\"font-size: 13px; line-height: 1.4;\">{}</div>\n", escaped));
                }
            }
        }

        html.push_str("</div>");

        Ok(PrintPreviewResult {
            html,
            lines,
            paper_width: paper_size.to_string(),
        })
    }

    pub fn render_template_preview(&self, template_id: i64, data: &serde_json::Value) -> Result<PrintPreviewResult> {
        let tpl = self.get_print_template(template_id)?;
        self.render_template_content_preview(
            &tpl.content,
            &tpl.paper_size,
            tpl.theme.as_deref().unwrap_or("classic"),
            tpl.restaurant_name.as_deref().unwrap_or(""),
            tpl.tagline.as_deref().unwrap_or(""),
            tpl.logo_data.as_deref(),
            data,
        )
    }

    fn interpolate_template_string(&self, template: &str, data: &serde_json::Value) -> String {
        let mut result = template.to_string();
        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                let placeholder = format!("{{{{{}}}}}", key);
                let replacement = match value {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Null => "".to_string(),
                    _ => value.to_string(),
                };
                result = result.replace(&placeholder, &replacement);
            }
        }
        result
    }

    #[allow(dead_code)]
    pub fn render_kitchen_ticket_from_template(&self, template_id: i64, data: &serde_json::Value, paper_size: &str) -> Result<(String, Vec<u8>)> {
        use crate::printer::EscPosBuilder;
        
        let tpl = self.get_print_template(template_id)?;
        let theme = tpl.theme.as_deref().unwrap_or("classic");
        
        let restaurant_name = tpl.restaurant_name.as_deref().unwrap_or("Cuckoo");
        let tagline = tpl.tagline.as_deref().unwrap_or("");
        let modifiers_color = tpl.modifiers_color.as_deref().unwrap_or("red");
        
        let order_no = data.get("order_no").and_then(|v| v.as_str()).unwrap_or("");
        let dine_type = data.get("dine_type").and_then(|v| v.as_str()).unwrap_or("");
        let note = data.get("note").and_then(|v| v.as_str());
        let items = data.get("items").and_then(|v| v.as_array());
        
        let width = if paper_size == "58mm" { 32 } else { 48 };
        
        let mut builder = EscPosBuilder::new();
        
        match theme {
            "minimal" => {
                builder.align_center().bold_on().double_height()
                    .text_ln(restaurant_name)
                    .normal_size().bold_off();
                if !tagline.is_empty() {
                    builder.text_ln(tagline);
                }
                builder.separator(width);
                builder.text_ln(&format!("單號: {}", order_no))
                    .text_ln(&format!("類型: {}", dine_type))
                    .separator(width);
                builder.bold_on().text_ln("項目").bold_off();
                if let Some(arr) = items {
                    for item in arr {
                        let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let qty = item.get("qty").and_then(|v| v.as_f64()).unwrap_or(1.0);
                        let item_note = item.get("note").and_then(|v| v.as_str());
                        if qty != 1.0 {
                            builder.text_ln(&format!("{} x{}", name, qty as i32));
                        } else {
                            builder.text_ln(name);
                        }
                        if let Some(n) = item_note {
                            let prefix = if modifiers_color == "red" { "  " } else { "**" };
                            builder.text_ln(&format!("{}{}", prefix, n));
                        }
                    }
                }
                if let Some(n) = note {
                    builder.separator(width).bold_on().text_ln(&format!("備註: {}", n)).bold_off();
                }
                builder.feed_lines(3).cut_paper();
            }
            "modern" => {
                builder.align_center()
                    .text_ln(&"-".repeat(width))
                    .bold_on().text_ln(restaurant_name).bold_off();
                if !tagline.is_empty() {
                    builder.text_ln(tagline);
                }
                builder.text_ln(&"-".repeat(width))
                    .align_left()
                    .text_ln(&format!("NO: {}", order_no))
                    .text_ln(&format!("TYPE: {}", dine_type))
                    .text_ln(&chrono::Local::now().format("%Y-%m-%d %H:%M").to_string())
                    .separator(width);
                if let Some(arr) = items {
                    for item in arr {
                        let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let qty = item.get("qty").and_then(|v| v.as_f64()).unwrap_or(1.0);
                        let item_note = item.get("note").and_then(|v| v.as_str());
                        if qty != 1.0 {
                            builder.text_ln(&format!("[{}] {}", qty as i32, name));
                        } else {
                            builder.text_ln(&format!("[1] {}", name));
                        }
                        if let Some(n) = item_note {
                            let style = if modifiers_color == "bold" { "**" } else { "→" };
                            builder.text_ln(&format!("  {} {}", style, n));
                        }
                    }
                }
                if let Some(n) = note {
                    builder.separator(width).text_ln(&format!("備註: {}", n));
                }
                builder.separator(width);
                builder.feed_lines(3).cut_paper();
            }
            _ => {
                builder.align_center().bold_on().double_height()
                    .text_ln(restaurant_name)
                    .normal_size().bold_off();
                if !tagline.is_empty() {
                    builder.text_ln(tagline);
                }
                builder.separator(width);
                builder.align_left()
                    .text_ln(&format!("單號: {}", order_no))
                    .text_ln(&format!("類型: {}", dine_type))
                    .text_ln(&chrono::Local::now().format("%Y-%m-%d %H:%M").to_string())
                    .separator(width);
                builder.bold_on().text_ln("項目明細").bold_off();
                if let Some(arr) = items {
                    for item in arr {
                        let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let qty = item.get("qty").and_then(|v| v.as_f64()).unwrap_or(1.0);
                        let item_note = item.get("note").and_then(|v| v.as_str());
                        if qty != 1.0 {
                            builder.text_ln(&format!("{} x{}", name, qty as i32));
                        } else {
                            builder.text_ln(name);
                        }
                        if let Some(n) = item_note {
                            let prefix = if modifiers_color == "red" { "  [紅] " } else { "  **" };
                            builder.text_ln(&format!("{}{}", prefix, n));
                        }
                    }
                }
                builder.separator(width);
                if let Some(n) = note {
                    builder.bold_on().text_ln(&format!("訂單備註: {}", n)).bold_off();
                }
                builder.feed_lines(3).cut_paper();
            }
        }
        
        let content = String::from_utf8_lossy(&builder.buffer).to_string();

        Ok((content, builder.buffer))
    }
}

// ==================== 行銷元件渲染函數 ====================

fn render_discount_coupon(elem: &serde_json::Value, data: &serde_json::Value, paper_size: &str, lines: &mut Vec<String>) {
    let width = if paper_size == "58mm" { 32 } else { 48 };
    let dash = "─".repeat(width);
    let discount_type = elem["discount_type"].as_str().unwrap_or("percent");
    let value = elem["value"].as_f64().unwrap_or(0.0);
    let condition = elem["condition"].as_str().unwrap_or("");
    let valid_days = elem["valid_days"].as_u64().unwrap_or(30);
    let label = elem["label"].as_str().unwrap_or("下次消費享折扣");

    let order_id = data["order_id"].as_i64().unwrap_or(0);

    let valid_until = {
        let today = chrono::Local::now();
        let days = chrono::Duration::days(valid_days as i64);
        (today + days).format("%Y-%m-%d").to_string()
    };

    // Mix order_id with date salt — harder to reverse than plain hash
    let date_salt = valid_until.chars().filter(|c| c.is_ascii_digit()).take(8)
        .fold(0u64, |acc, c| acc.wrapping_mul(31).wrapping_add(c as u64));
    let mixed = (order_id as u64)
        .wrapping_mul(0x9E3779B97F4A7C15) // Knuth multiplicative hash (64-bit)
        .wrapping_add(date_salt)
        .rotate_left(17)
        ^ 0xA3B4C5D6E7F80901u64;
    let code = format!("{:012X}", mixed & 0x0000_FFFF_FFFF_FFFF); // 12 hex chars

    let discount_str = match discount_type {
        "percent" => format!("{:.0}折", (1.0 - value / 100.0) * 10.0),
        "amount"  => format!("立減 {:.0}元", value),
        "free_item" => "指定免費".to_string(),
        _ => format!("{}", value),
    };

    lines.push(dash.clone());
    lines.push(format!("  🎟  {}  🎟", label));
    lines.push(format!("        【 {} 】", discount_str));
    if !condition.is_empty() {
        lines.push(format!("  條件：{}", condition));
    }
    lines.push(format!("  有效期至：{}", valid_until));
    lines.push(format!("  優惠碼：{}", code));
    lines.push(dash);
}

fn render_product_spotlight(elem: &serde_json::Value, paper_size: &str, lines: &mut Vec<String>) {
    let width = if paper_size == "58mm" { 32 } else { 48 };
    let dash = "─".repeat(width);
    let title = elem["title"].as_str().unwrap_or("本週新品");
    let name = elem["name"].as_str().unwrap_or("");
    let description = elem["description"].as_str().unwrap_or("");
    let price = elem["price"].as_f64();
    let badge = elem["badge"].as_str().unwrap_or("NEW");

    lines.push(dash.clone());
    lines.push(format!("  ★ {} ★", title));
    lines.push(format!("  [{}] {}", badge, name));
    if !description.is_empty() {
        lines.push(format!("  {}", description));
    }
    if let Some(p) = price {
        lines.push(format!("  定價：¥{:.0}", p));
    }
    lines.push(dash);
}

fn render_qr_code_element(elem: &serde_json::Value, paper_size: &str, lines: &mut Vec<String>) {
    let width = if paper_size == "58mm" { 32 } else { 48 };
    let label = elem["label"].as_str().unwrap_or("掃碼了解更多");
    let url = elem["url"].as_str().unwrap_or("");

    lines.push("─".repeat(width));
    lines.push(format!("  📱  {}", label));
    // In ESC/POS actual print, QR is rendered via EscPosBuilder::qr_code()
    // In text preview, show the URL
    if !url.is_empty() {
        let display_url = if url.len() > width - 4 { &url[..width - 4] } else { url };
        lines.push(format!("  {}", display_url));
    }
    lines.push("  [QR Code]".to_string());
    lines.push("─".repeat(width));
}

fn render_character_collect(elem: &serde_json::Value, data: &serde_json::Value, paper_size: &str, lines: &mut Vec<String>) {
    let width = if paper_size == "58mm" { 32 } else { 48 };
    let game_name = elem["game_name"].as_str().unwrap_or("集字兌獎");
    let prize = elem["prize"].as_str().unwrap_or("");
    let style = elem["style"].as_str().unwrap_or("box");

    let characters: Vec<&str> = elem["characters"].as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    if characters.is_empty() {
        return;
    }

    let order_id = data["order_id"].as_i64().unwrap_or(0);
    let seed_strategy = elem["seed_strategy"].as_str().unwrap_or("per_order");
    let date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
    let seed = creative_fortune_seed(seed_strategy, data["table_no"].as_str(), Some(order_id), &date_str);
    let char_idx = (seed as usize) % characters.len();
    let this_char = characters[char_idx];

    let (open_b, close_b) = if style == "mahjong" { ("", "") } else { ("【", "】" ) };

    lines.push("─".repeat(width));
    lines.push(format!("  {}", game_name));
    lines.push(format!("  本單獲得：{}{}{}", open_b, this_char, close_b));

    // Show collection grid: highlight this_char, others as □
    let grid: Vec<String> = characters.iter().enumerate().map(|(i, &ch)| {
        if i == char_idx { format!("[{}]", ch) } else { "□ ".to_string() }
    }).collect();
    lines.push(format!("  {}", grid.join(" ")));

    if !prize.is_empty() {
        lines.push(format!("  → {}", prize));
    }
    lines.push("─".repeat(width));
}

// ── S2 新组件静态资源 ────────────────────────────────────────────────────────

// 24节气：(月, 日范围起, 名称, 主题文案, 推荐语)
// 月份从1开始，日期区间取节气前后约7天
static SOLAR_TERMS: &[(u32, u32, u32, &str, &str, &str)] = &[
    (1, 5, 8,   "小寒", "小寒已至，寒气渐深。", "今日推荐：暖心热汤，驱寒养胃。"),
    (1, 19, 22, "大寒", "大寒岁末，寒意最浓。", "今日推荐：滋补靓汤，暖身御寒。"),
    (2, 3, 6,   "立春", "立春一至，万象更新。", "今日推荐：时令鲜蔬，迎接新春。"),
    (2, 18, 21, "雨水", "雨水滋润，春意渐浓。", "今日推荐：清淡食材，滋养脾胃。"),
    (3, 5, 8,   "惊蛰", "惊蛰春雷，万物复苏。", "今日推荐：春笋鲜菜，清肠排毒。"),
    (3, 20, 23, "春分", "春分昼夜，阴阳调和。", "今日推荐：营养均衡，顺应时令。"),
    (4, 4, 7,   "清明", "清明时节，慎终追远。", "今日推荐：青团糍粑，清淡养生。"),
    (4, 19, 22, "谷雨", "谷雨茶香，春末将至。", "今日推荐：新茶配点心，清雅怡人。"),
    (5, 5, 8,   "立夏", "立夏已到，暑气渐升。", "今日推荐：清凉饮品，消暑解渴。"),
    (5, 20, 23, "小满", "小满时节，麦粒初熟。", "今日推荐：杂粮主食，健脾养胃。"),
    (6, 5, 8,   "芒种", "芒种忙碌，夏日渐盛。", "今日推荐：冷饮冰品，清凉解暑。"),
    (6, 21, 24, "夏至", "夏至阳极，昼长夜短。", "今日推荐：冰饮甜品，清热解暑。"),
    (7, 7, 10,  "小暑", "小暑伏天，暑热渐盛。", "今日推荐：清凉食材，防暑降温。"),
    (7, 22, 25, "大暑", "大暑酷热，一年最热。", "今日推荐：绿豆冰饮，消暑开胃。"),
    (8, 7, 10,  "立秋", "立秋一至，暑退秋来。", "今日推荐：秋梨润肺，养阴生津。"),
    (8, 22, 25, "处暑", "处暑暑退，秋意渐起。", "今日推荐：滋阴润燥，应对秋燥。"),
    (9, 7, 10,  "白露", "白露秋深，露水初凝。", "今日推荐：润肺食材，养生防燥。"),
    (9, 22, 25, "秋分", "秋分昼夜，天高气爽。", "今日推荐：时令海鲜，鲜美当季。"),
    (10, 8, 11, "寒露", "寒露秋深，晨露渐寒。", "今日推荐：温热食物，驱散寒意。"),
    (10, 23, 26,"霜降", "霜降秋末，万物收藏。", "今日推荐：滋补食材，为冬储能。"),
    (11, 7, 10, "立冬", "立冬已至，寒冬将临。", "今日推荐：炖汤暖锅，抵御寒冬。"),
    (11, 22, 25,"小雪", "小雪初降，寒意渐深。", "今日推荐：热腾火锅，暖胃暖心。"),
    (12, 7, 10, "大雪", "大雪纷飞，严冬正盛。", "今日推荐：暖锅热汤，驱寒暖身。"),
    (12, 21, 24,"冬至", "冬至阳生，一阳来复。", "今日推荐：饺子汤圆，阖家团圆。"),
];

fn render_dish_easter_egg(elem: &serde_json::Value, data: &serde_json::Value, paper_size: &str, lines: &mut Vec<String>) {
    let width = if paper_size == "58mm" { 32 } else { 48 };

    // eggs: [{keyword: "虾", message: "解锁：海鲜老饕称号！下单此组合95折"}]
    let eggs = match elem["eggs"].as_array() {
        Some(e) => e,
        None => return,
    };

    // Get order item names from data
    let item_names: Vec<String> = data["items"].as_array()
        .map(|items| items.iter()
            .filter_map(|i| i["name"].as_str().map(|s| s.to_string()))
            .collect())
        .unwrap_or_default();

    if item_names.is_empty() { return; }

    let all_names = item_names.join(" ");
    let matched: Vec<(&str, &str)> = eggs.iter().filter_map(|egg| {
        let kw = egg["keyword"].as_str()?;
        let msg = egg["message"].as_str()?;
        if all_names.contains(kw) { Some((kw, msg)) } else { None }
    }).collect();

    if matched.is_empty() { return; }

    let dash = "─".repeat(width);
    lines.push(dash.clone());
    lines.push("  🎉 隐藏彩蛋解锁！".to_string());
    for (_, msg) in &matched {
        lines.push(format!("  {}", msg));
    }
    lines.push(dash);
}

fn render_solar_term(elem: &serde_json::Value, paper_size: &str, lines: &mut Vec<String>) {
    let now = chrono::Local::now();
    let month = now.month();
    let day = now.day();
    let width = if paper_size == "58mm" { 32 } else { 48 };

    let term = SOLAR_TERMS.iter().find(|(m, d_start, d_end, ..)| {
        *m == month && day >= *d_start && day <= *d_end
    });

    let show_all = elem["show_all"].as_bool().unwrap_or(false);

    if let Some((_, _, _, name, theme, recommend)) = term {
        let dash = "─".repeat(width);
        lines.push(dash.clone());
        lines.push(format!("  ✦ 节气 · {} ✦", name));
        lines.push(theme.to_string());
        lines.push(recommend.to_string());
        lines.push(dash);
    } else if show_all {
        // 不在任何节气期间，显示当前月份的节气提示
        let upcoming = SOLAR_TERMS.iter().find(|(m, ..)| *m == month || *m == month % 12 + 1);
        if let Some((_, _, _, name, _, _)) = upcoming {
            let dash = "─".repeat(width);
            lines.push(dash.clone());
            lines.push(format!("  下一个节气：{}", name));
            lines.push(dash);
        }
    }
    // 不在节气期间且 show_all=false 时，静默跳过（不输出任何内容）
}

// 内置谜语库（20条）+ 店家可在 config 中添加
static BUILTIN_RIDDLES: &[(&str, &str)] = &[
    ("身穿白大褂，不是医生帮，你吃它肉，它喝你汤。", "豆腐"),
    ("生在山上，卖到山下，一到家里，就受刀斧。", "柴火"),
    ("麻屋子，红帐子，里面睡个白胖子。", "花生"),
    ("远看似座山，近看似朵花，吃饭要用它，洗碗也用它。", "猪"),
    ("金箱子，银箱子，打开是个宝贝子。", "鸡蛋"),
    ("身穿绿衣裳，肚里水汪汪，生的子儿多，个个黑脸膛。", "西瓜"),
    ("头戴红帽子，身穿白袍子，走路摆架子，说话伸脖子。", "鹅"),
    ("有眼看不见，有嘴说不出，一生辛苦做，人人都感谢。", "针"),
    ("一个老头走山坡，走一步，退两步。", "蜗牛"),
    ("一条长虫子，有嘴没眼睛，天天流眼泪，心里不伤心。", "蜡烛"),
    ("千根线，万根线，落到水里看不见。", "雨"),
    ("一棵大树开白花，花谢结果不离家，打开来一看，里边全是宝。", "棉花"),
    ("白白胖胖圆溜溜，冬天躲在地里头，不怕寒冷不怕霜，春来破土往上蹿。", "萝卜"),
    ("生得白嫩嫩，裹着绿衣裳，长大离娘去，漂洋过了海。", "蒜"),
    ("一个铜锣响，一个铜锣哑，两个铜锣合在一起打。", "蛤蜊"),
    ("千锤百炼出深山，烈火焚身若等闲，粉身碎骨全不怕，要留清白在人间。", "石灰"),
    ("开门似个山，关门似弦弓，上面毛茸茸，下面红彤彤。", "嘴"),
    ("远看山有色，近听水无声，春去花还在，人来鸟不惊。", "画"),
    ("你走它也走，你停它也停，你进房间里，它站门外等。", "鞋"),
    ("有时落在山腰，有时挂在树梢，有时像圆盘，有时像镰刀。", "月亮"),
];

fn render_riddle(elem: &serde_json::Value, paper_size: &str, lines: &mut Vec<String>) {
    let width = if paper_size == "58mm" { 32 } else { 48 };
    let prize = elem["prize"].as_str().unwrap_or("下次来店说出答案，赢取小惊喜！");

    // 自定义谜语优先，没有则从内置库随机选
    let (question, _answer) = if let (Some(q), Some(a)) = (elem["question"].as_str(), elem["answer"].as_str()) {
        (q, a)
    } else {
        let seed = creative_fortune_seed("daily", None, None, &chrono::Local::now().format("%Y-%m-%d").to_string());
        BUILTIN_RIDDLES[(seed as usize) % BUILTIN_RIDDLES.len()]
    };

    let dash = "─".repeat(width);
    lines.push(dash.clone());
    lines.push("  🤔 今日谜语".to_string());
    // 换行显示谜面（超过 width-4 字符时自动断行）
    let max_chars = (width - 4).max(10);
    let mut q = question;
    while !q.is_empty() {
        let take: usize = q.char_indices().nth(max_chars).map(|(i, _)| i).unwrap_or(q.len());
        lines.push(format!("  {}", &q[..take]));
        q = &q[take..];
    }
    lines.push(format!("  → {}", prize));
    lines.push("  答案下次来告诉我们！".to_string());
    lines.push(dash);
}

fn render_chef_message(elem: &serde_json::Value, paper_size: &str, lines: &mut Vec<String>) {
    let width = if paper_size == "58mm" { 32 } else { 48 };

    // messages 是数组，按星期循环（周一=0）
    let messages = elem["messages"].as_array();
    let default_msgs = ["今天的食材格外新鲜，用心为您烹饪。",
                        "感谢光临，愿每一口都让您满意。",
                        "好食材，慢火候，是我们对美食的承诺。",
                        "今日特别推荐，请向店员询问。",
                        "周末快乐！今天让我们为您做最好的一道菜。",
                        "感恩有您，是您让这里变得温暖。",
                        "新的一周，新的心情，祝您用餐愉快！"];

    let msg = if let Some(msgs) = messages {
        let weekday = chrono::Local::now().weekday().num_days_from_monday() as usize;
        msgs.get(weekday % msgs.len())
            .and_then(|v| v.as_str())
            .unwrap_or(default_msgs[weekday % default_msgs.len()])
    } else {
        let weekday = chrono::Local::now().weekday().num_days_from_monday() as usize;
        default_msgs[weekday % default_msgs.len()]
    };

    let title = elem["title"].as_str().unwrap_or("厨师寄语");
    let author = elem["author"].as_str().unwrap_or("本店厨师");

    let dash = "─".repeat(width);
    lines.push(dash.clone());
    lines.push(format!("  👨‍🍳 {}", title));
    lines.push(format!("  {}", msg));
    lines.push(format!("  — {}", author));
    lines.push(dash);
}

fn render_rich_text(elem: &serde_json::Value, paper_size: &str, lines: &mut Vec<String>) {
    let content = elem["content"].as_str().unwrap_or("");
    let width = if paper_size == "58mm" { 32 } else { 48 };

    for raw_line in content.split('\n') {
        let line = raw_line.trim_end();
        if line.starts_with("## ") {
            // H2 → centered bold-ish
            let text = line.trim_start_matches("## ").trim();
            let pad = if width > text.chars().count() { (width - text.chars().count()) / 2 } else { 0 };
            lines.push(format!("{}**{}**", " ".repeat(pad), text));
        } else if line.starts_with("# ") {
            let text = line.trim_start_matches("# ").trim();
            let pad = if width > text.chars().count() { (width - text.chars().count()) / 2 } else { 0 };
            lines.push(format!("{}**{}**", " ".repeat(pad), text));
        } else if line.starts_with("- ") || line.starts_with("* ") {
            let rest: String = line.chars().skip(2).collect();
            lines.push(format!("  • {}", rest));
        } else if line.starts_with("> ") {
            let rest: String = line.chars().skip(2).collect();
            lines.push(format!("  {}", rest));
        } else if line.starts_with("```") {
            // Mermaid or code block — ESC/POS just shows placeholder
            lines.push("  [圖表/代碼塊]".to_string());
        } else {
            lines.push(line.to_string());
        }
    }
}
