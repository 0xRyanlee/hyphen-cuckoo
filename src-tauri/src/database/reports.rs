use rusqlite::{params, Result};
use super::*;

impl Database {
    // ==================== 報表 ====================

    // Returns (date, billed_amount, order_count, collected_amount)
    pub fn get_sales_report(&self, start_date: &str, end_date: &str) -> Result<Vec<(String, f64, i64, f64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT DATE(o.created_at),
                    SUM(oi.qty * oi.unit_price),
                    COUNT(DISTINCT o.id),
                    SUM(CASE WHEN COALESCE(o.payment_status,'unpaid') != 'unpaid' THEN COALESCE(o.amount_paid, 0) ELSE 0 END)
             FROM orders o
             JOIN order_items oi ON o.id = oi.order_id
             WHERE o.status IN ('submitted','ready')
               AND DATE(o.created_at) BETWEEN ?1 AND ?2
             GROUP BY DATE(o.created_at)
             ORDER BY DATE(o.created_at)"
        )?;
        let rows = stmt.query_map(params![start_date, end_date], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?.collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn get_sales_by_category(&self, start_date: &str, end_date: &str) -> Result<Vec<(String, f64, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT mc.name, SUM(oi.qty * oi.unit_price), SUM(oi.qty) FROM orders o JOIN order_items oi ON o.id = oi.order_id JOIN menu_items mi ON oi.menu_item_id = mi.id LEFT JOIN menu_categories mc ON mi.category_id = mc.id WHERE o.status IN ('submitted', 'ready') AND DATE(o.created_at) BETWEEN ?1 AND ?2 GROUP BY mc.name ORDER BY SUM(oi.qty * oi.unit_price) DESC")?;
        let rows = stmt.query_map(params![start_date, end_date], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?.collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    // Returns (date, revenue, cogs, gross_profit, expenses, net_profit)
    pub fn get_gross_profit_report(&self, start_date: &str, end_date: &str) -> Result<Vec<(String, f64, f64, f64, f64, f64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "WITH rev AS (
                SELECT DATE(o.created_at) AS day, SUM(oi.qty * oi.unit_price) AS revenue
                FROM orders o
                JOIN order_items oi ON o.id = oi.order_id
                WHERE o.status IN ('submitted','ready')
                  AND DATE(o.created_at) BETWEEN ?1 AND ?2
                GROUP BY day
            ),
            cst AS (
                SELECT DATE(o.created_at) AS day, SUM(ABS(it.cost_delta)) AS cost
                FROM orders o
                JOIN inventory_txns it ON it.ref_type = 'order' AND it.ref_id = o.id
                                      AND it.txn_type = 'consume'
                WHERE o.status IN ('submitted','ready')
                  AND DATE(o.created_at) BETWEEN ?1 AND ?2
                GROUP BY day
            ),
            exp AS (
                SELECT expense_date AS day, SUM(amount) AS expenses
                FROM expenses
                WHERE is_active = 1
                  AND expense_date BETWEEN ?1 AND ?2
                GROUP BY day
            )
            SELECT rev.day,
                   rev.revenue,
                   COALESCE(cst.cost, 0),
                   rev.revenue - COALESCE(cst.cost, 0),
                   COALESCE(exp.expenses, 0),
                   rev.revenue - COALESCE(cst.cost, 0) - COALESCE(exp.expenses, 0)
            FROM rev
            LEFT JOIN cst ON cst.day = rev.day
            LEFT JOIN exp ON exp.day = rev.day
            ORDER BY rev.day"
        )?;
        let rows = stmt.query_map(params![start_date, end_date], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
        })?.collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn get_top_selling_items(&self, start_date: &str, end_date: &str, limit: i64) -> Result<Vec<(String, f64, i64, f64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT mi.name, SUM(oi.qty * oi.unit_price), SUM(oi.qty), AVG(oi.unit_price) FROM orders o JOIN order_items oi ON o.id = oi.order_id JOIN menu_items mi ON oi.menu_item_id = mi.id WHERE o.status IN ('submitted', 'ready') AND DATE(o.created_at) BETWEEN ?1 AND ?2 GROUP BY mi.name ORDER BY SUM(oi.qty) DESC LIMIT ?3")?;
        let rows = stmt.query_map(params![start_date, end_date, limit], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?.collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// 原料消耗报表
    pub fn get_material_consumption_report(&self, start_date: &str, end_date: &str) -> Result<Vec<(String, f64, f64, f64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT m.name, 
                    SUM(it.qty_delta) as total_consumed,
                    COALESCE(AVG(ib.cost_per_unit), 0) as avg_cost,
                    SUM(it.qty_delta * COALESCE(ib.cost_per_unit, 0)) as total_cost
             FROM inventory_txns it
             LEFT JOIN materials m ON it.material_id = m.id
             LEFT JOIN inventory_batches ib ON it.lot_id = ib.id
             WHERE it.txn_type = 'consume' 
               AND DATE(it.created_at) BETWEEN ?1 AND ?2
             GROUP BY m.id, m.name
             ORDER BY total_consumed DESC"
        )?;
        let rows = stmt.query_map(params![start_date, end_date], |row| {
            Ok((
                row.get(0)?,
                row.get::<_, f64>(1)?.abs(),
                row.get(2)?,
                row.get(3)?,
            ))
        })?.collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }


}
