use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

use rusqlite::{params, Connection};
use serde::Serialize;

pub struct SyncServerHandle {
    pub port: u16,
    running: Arc<AtomicBool>,
}

#[derive(Clone)]
struct SyncServerConfig {
    shared_secret: Option<String>,
    protocol_version: String,
}

#[derive(Debug, Serialize)]
struct SyncOrderItem {
    id: i64,
    order_id: i64,
    menu_item_id: i64,
    spec_code: Option<String>,
    qty: f64,
    unit_price: f64,
    note: Option<String>,
    refunded: bool,
}

#[derive(Debug, Serialize)]
struct SyncTicketWithItems {
    id: i64,
    order_id: i64,
    station_id: i64,
    status: String,
    priority: i32,
    printed_at: Option<String>,
    started_at: Option<String>,
    finished_at: Option<String>,
    created_at: String,
    order_no: String,
    dine_type: String,
    table_no: Option<String>,
    items: Vec<SyncOrderItem>,
}

impl SyncServerHandle {
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
        // Unblock accept() by connecting to self
        std::net::TcpStream::connect(format!("127.0.0.1:{}", self.port)).ok();
    }
}

pub fn start_server(
    db_path: PathBuf,
    port: u16,
    shared_secret: Option<String>,
    protocol_version: String,
) -> Result<SyncServerHandle, String> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .map_err(|e| format!("端口 {} 绑定失败: {}", port, e))?;

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let config = SyncServerConfig {
        shared_secret: shared_secret.and_then(|s| {
            let trimmed = s.trim().to_string();
            if trimmed.is_empty() { None } else { Some(trimmed) }
        }),
        protocol_version,
    };

    std::thread::spawn(move || {
        let conn = match Connection::open(&db_path) {
            Ok(c) => {
                c.busy_timeout(Duration::from_secs(3)).ok();
                c
            }
            Err(_) => return,
        };

        while running_clone.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _)) => {
                    if !running_clone.load(Ordering::Relaxed) {
                        break;
                    }
                    stream.set_read_timeout(Some(Duration::from_secs(3))).ok();
                    stream.set_write_timeout(Some(Duration::from_secs(3))).ok();
                    handle_connection(stream, &conn, &config);
                }
                Err(_) => {}
            }
        }
    });

    Ok(SyncServerHandle { port, running })
}

fn handle_connection(mut stream: TcpStream, conn: &Connection, config: &SyncServerConfig) {
    let mut buf = [0u8; 2048];
    let n = match stream.read(&mut buf) {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let req = String::from_utf8_lossy(&buf[..n]);
    let mut lines = req.lines();
    let first_line = lines.next().unwrap_or("");
    let path = first_line.split_whitespace().nth(1).unwrap_or("/");
    let route = path.split('?').next().unwrap_or("");
    let query = path.split('?').nth(1).unwrap_or("");
    let headers = parse_headers(lines);

    let since_epoch_s: i64 = query
        .split('&')
        .find(|p| p.starts_with("since="))
        .and_then(|p| p.strip_prefix("since="))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let method = first_line.split_whitespace().next().unwrap_or("GET");

    let (status, body) = match validate_request(method, route, &headers, config) {
        Err(err) => err,
        Ok(_) => match (method, route) {
            ("GET", "/api/ping") => ("200 OK", r#"{"ok":true}"#.to_string()),
            ("GET", "/api/orders") => match query_orders_since(conn, since_epoch_s) {
                Ok(json) => ("200 OK", json),
                Err(e) => (
                    "500 Internal Server Error",
                    format!(r#"{{"error":"{}"}}"#, e),
                ),
            },
            ("GET", "/api/tickets") => match query_open_tickets(conn) {
                Ok(json) => ("200 OK", json),
                Err(e) => (
                    "500 Internal Server Error",
                    format!(r#"{{"error":"{}"}}"#, e),
                ),
            },
            ("POST", _) if route.starts_with("/api/tickets/") && route.ends_with("/start") => {
                match parse_ticket_action_id(route)
                    .and_then(|ticket_id| apply_ticket_action(conn, ticket_id, "start"))
                {
                    Ok(_) => ("200 OK", r#"{"ok":true}"#.to_string()),
                    Err(e) => (
                        "500 Internal Server Error",
                        format!(r#"{{"error":"{}"}}"#, e),
                    ),
                }
            }
            ("POST", _) if route.starts_with("/api/tickets/") && route.ends_with("/finish") => {
                match parse_ticket_action_id(route)
                    .and_then(|ticket_id| apply_ticket_action(conn, ticket_id, "finish"))
                {
                    Ok(_) => ("200 OK", r#"{"ok":true}"#.to_string()),
                    Err(e) => (
                        "500 Internal Server Error",
                        format!(r#"{{"error":"{}"}}"#, e),
                    ),
                }
            }
            _ => ("404 Not Found", r#"{"error":"not found"}"#.to_string()),
        }
    };

    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        body.len(),
        body
    );
    stream.write_all(response.as_bytes()).ok();
}

fn parse_headers<'a>(lines: impl Iterator<Item = &'a str>) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }
    headers
}

fn validate_request(
    method: &str,
    route: &str,
    headers: &HashMap<String, String>,
    config: &SyncServerConfig,
) -> Result<(), (&'static str, String)> {
    let route_allowed = route == "/api/ping"
        || route == "/api/orders"
        || route == "/api/tickets"
        || (method == "POST"
            && route.starts_with("/api/tickets/")
            && (route.ends_with("/start") || route.ends_with("/finish")));
    if !route_allowed {
        return Ok(());
    }

    if let Some(expected) = &config.shared_secret {
        let provided = headers
            .get("x-cuckoo-sync-token")
            .map(|v| v.trim())
            .filter(|v| !v.is_empty());
        if provided != Some(expected.as_str()) {
            return Err((
                "401 Unauthorized",
                r#"{"error":"missing or invalid sync token"}"#.to_string(),
            ));
        }
    }

    let client_version = headers
        .get("x-cuckoo-sync-version")
        .map(|v| v.trim())
        .unwrap_or("");
    if client_version != config.protocol_version {
        return Err((
            "409 Conflict",
            format!(
                r#"{{"error":"sync protocol mismatch","expected":"{}"}}"#,
                config.protocol_version
            ),
        ));
    }

    Ok(())
}

fn parse_ticket_action_id(route: &str) -> Result<i64, String> {
    let mut segments = route.trim_matches('/').split('/');
    match (
        segments.next(),
        segments.next(),
        segments.next(),
        segments.next(),
        segments.next(),
    ) {
        (Some("api"), Some("tickets"), Some(id), Some(_action), None) => {
            id.parse::<i64>().map_err(|_| "invalid ticket id".to_string())
        }
        _ => Err("invalid ticket route".to_string()),
    }
}

fn apply_ticket_action(conn: &Connection, ticket_id: i64, action: &str) -> Result<(), String> {
    match action {
        "start" => {
            conn.execute(
                "UPDATE kitchen_tickets SET status = 'started', started_at = datetime('now') WHERE id = ?1",
                params![ticket_id],
            ).map_err(|e| e.to_string())?;
            Ok(())
        }
        "finish" => {
            let order_id: i64 = conn.query_row(
                "SELECT order_id FROM kitchen_tickets WHERE id = ?1",
                params![ticket_id],
                |row| row.get(0),
            ).map_err(|e| e.to_string())?;

            conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;
            let result = (|| -> Result<(), String> {
                conn.execute(
                    "UPDATE kitchen_tickets SET status = 'finished', finished_at = datetime('now') WHERE id = ?1",
                    params![ticket_id],
                ).map_err(|e| e.to_string())?;

                let unfinished: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM kitchen_tickets WHERE order_id = ?1 AND status != 'finished'",
                    params![order_id],
                    |row| row.get(0),
                ).map_err(|e| e.to_string())?;

                if unfinished == 0 {
                    consume_order_inventory(conn, order_id)?;
                    conn.execute(
                        "UPDATE orders SET status = 'ready', updated_at = datetime('now') WHERE id = ?1",
                        params![order_id],
                    ).map_err(|e| e.to_string())?;
                }
                Ok(())
            })();

            match result {
                Ok(_) => conn.execute_batch("COMMIT").map_err(|e| e.to_string()),
                Err(e) => {
                    conn.execute_batch("ROLLBACK").ok();
                    Err(e)
                }
            }
        }
        _ => Err("unsupported action".to_string()),
    }
}

fn consume_order_inventory(conn: &Connection, order_id: i64) -> Result<(), String> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM inventory_txns WHERE ref_type = 'order' AND ref_id = ?1 AND txn_type = 'consume'",
        params![order_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    if exists > 0 {
        return Ok(());
    }

    let items: Vec<(i64, f64)> = {
        let mut stmt = conn.prepare(
            "SELECT menu_item_id, qty FROM order_items WHERE order_id = ?1 AND COALESCE(refunded,0) = 0"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(params![order_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        rows
    };

    let mut needs: std::collections::HashMap<i64, f64> = std::collections::HashMap::new();
    for (menu_item_id, qty) in items {
        let recipe_id: Option<i64> = conn.query_row(
            "SELECT recipe_id FROM menu_items WHERE id = ?1",
            params![menu_item_id],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;

        if let Some(recipe_id) = recipe_id {
            expand_recipe_needs(conn, recipe_id, qty, 0, &mut needs)?;
        }
    }

    for (material_id, req_qty) in &needs {
        let available: f64 = conn.query_row(
            "SELECT COALESCE(SUM(quantity), 0.0) FROM inventory_batches WHERE material_id = ?1 AND quantity > 0",
            params![material_id],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;
        if available + 1e-9 < *req_qty {
            return Err(format!("库存不足，material_id={} need={}, available={}", material_id, req_qty, available));
        }
    }

    for (material_id, req_qty) in needs {
        let mut remain = req_qty;
        let mut stmt = conn.prepare(
            "SELECT id, quantity, cost_per_unit FROM inventory_batches
             WHERE material_id = ?1 AND quantity > 0
             ORDER BY COALESCE(expiry_date, '9999-12-31') ASC, created_at ASC, id ASC"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(params![material_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, f64>(2)?,
            ))
        }).map_err(|e| e.to_string())?;

        for row in rows {
            let (batch_id, batch_qty, cpu) = row.map_err(|e| e.to_string())?;
            if remain <= 0.0 {
                break;
            }
            let deduct = remain.min(batch_qty);
            conn.execute(
                "UPDATE inventory_batches SET quantity = quantity - ?1 WHERE id = ?2",
                params![deduct, batch_id],
            ).map_err(|e| e.to_string())?;
            conn.execute(
                "INSERT INTO inventory_txns
                 (txn_no, txn_type, ref_type, ref_id, lot_id, material_id, state_id, qty_delta, cost_delta, operator, note, created_at)
                 VALUES
                 (lower(hex(randomblob(16))), 'consume', 'order', ?1, ?2, ?3, NULL, ?4, ?5, NULL, NULL, datetime('now'))",
                params![order_id, batch_id, material_id, -deduct, -(deduct * cpu)],
            ).map_err(|e| e.to_string())?;
            remain -= deduct;
        }
    }

    Ok(())
}

fn expand_recipe_needs(
    conn: &Connection,
    recipe_id: i64,
    multiplier: f64,
    depth: u32,
    result: &mut std::collections::HashMap<i64, f64>,
) -> Result<(), String> {
    if depth > 10 {
        return Ok(());
    }
    let mut stmt = conn.prepare(
        "SELECT item_type, ref_id, qty * (1.0 + COALESCE(wastage_rate, 0.0))
         FROM recipe_items WHERE recipe_id = ?1"
    ).map_err(|e| e.to_string())?;
    let items: Vec<(String, i64, f64)> = stmt
        .query_map(params![recipe_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (item_type, ref_id, qty) in items {
        match item_type.as_str() {
            "material" => {
                *result.entry(ref_id).or_insert(0.0) += qty * multiplier;
            }
            "sub_recipe" => {
                let output_mid: Option<i64> = conn.query_row(
                    "SELECT output_material_id FROM recipes WHERE id = ?1",
                    params![ref_id],
                    |row| row.get(0),
                ).map_err(|e| e.to_string())?;
                if let Some(mid) = output_mid {
                    *result.entry(mid).or_insert(0.0) += qty * multiplier;
                } else {
                    expand_recipe_needs(conn, ref_id, qty * multiplier, depth + 1, result)?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn query_open_tickets(conn: &Connection) -> Result<String, String> {
    let mut stmt = conn.prepare(
        "SELECT kt.id, kt.order_id, kt.station_id, kt.status, kt.priority, kt.printed_at, kt.started_at, kt.finished_at, kt.created_at,
                o.order_no, o.dine_type, o.table_no
         FROM kitchen_tickets kt
         JOIN orders o ON o.id = kt.order_id
         WHERE kt.status IN ('pending', 'started')
         ORDER BY kt.priority DESC, kt.created_at ASC"
    ).map_err(|e| e.to_string())?;

    let ticket_rows: Vec<(
        i64, i64, i64, String, i32, Option<String>, Option<String>, Option<String>, String, String, String, Option<String>
    )> = stmt.query_map([], |row| {
        Ok((
            row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?,
            row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?, row.get(9)?,
            row.get(10)?, row.get(11)?,
        ))
    }).map_err(|e| e.to_string())?
      .collect::<Result<Vec<_>, _>>()
      .map_err(|e| e.to_string())?;

    let mut out = Vec::with_capacity(ticket_rows.len());
    for row in ticket_rows {
        let items = query_order_items(conn, row.1)?;
        out.push(SyncTicketWithItems {
            id: row.0,
            order_id: row.1,
            station_id: row.2,
            status: row.3,
            priority: row.4,
            printed_at: row.5,
            started_at: row.6,
            finished_at: row.7,
            created_at: row.8,
            order_no: row.9,
            dine_type: row.10,
            table_no: row.11,
            items,
        });
    }

    serde_json::to_string(&out).map_err(|e| e.to_string())
}

fn query_order_items(conn: &Connection, order_id: i64) -> Result<Vec<SyncOrderItem>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, order_id, menu_item_id, spec_code, qty, unit_price, note, COALESCE(refunded,0)
         FROM order_items WHERE order_id = ?1"
    ).map_err(|e| e.to_string())?;
    let items = stmt.query_map(params![order_id], |row| {
        Ok(SyncOrderItem {
            id: row.get(0)?,
            order_id: row.get(1)?,
            menu_item_id: row.get(2)?,
            spec_code: row.get(3)?,
            qty: row.get(4)?,
            unit_price: row.get(5)?,
            note: row.get(6)?,
            refunded: row.get::<_, i64>(7)? != 0,
        })
    }).map_err(|e| e.to_string())?
      .collect::<Result<Vec<_>, _>>()
      .map_err(|e| e.to_string())?;
    Ok(items)
}

fn query_orders_since(conn: &Connection, since_epoch_s: i64) -> Result<String, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, order_no, source, dine_type, table_no, status, amount_total, note,
                    COALESCE(payment_status,'unpaid'), payment_method, COALESCE(amount_paid,0.0),
                    created_at, updated_at
             FROM orders
             WHERE CAST(strftime('%s', updated_at) AS INTEGER) > ?1
               AND status != 'cancelled'
             ORDER BY updated_at DESC LIMIT 200",
        )
        .map_err(|e| e.to_string())?;

    let mut out = String::from("[");
    let mut first = true;
    let mut rows = stmt.query(params![since_epoch_s]).map_err(|e| e.to_string())?;

    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        if !first {
            out.push(',');
        }
        first = false;

        let id: i64 = row.get(0).unwrap_or(0);
        let order_no: String = row.get(1).unwrap_or_default();
        let source: String = row.get(2).unwrap_or_default();
        let dine_type: String = row.get(3).unwrap_or_default();
        let table_no: Option<String> = row.get(4).unwrap_or(None);
        let status: String = row.get(5).unwrap_or_default();
        let amount_total: f64 = row.get(6).unwrap_or(0.0);
        let note: Option<String> = row.get(7).unwrap_or(None);
        let payment_status: String = row.get(8).unwrap_or_default();
        let payment_method: Option<String> = row.get(9).unwrap_or(None);
        let amount_paid: f64 = row.get(10).unwrap_or(0.0);
        let created_at: String = row.get(11).unwrap_or_default();
        let updated_at: String = row.get(12).unwrap_or_default();

        let table_no_json = table_no
            .map(|v| format!(r#""{}""#, v.replace('"', "\\\"")))
            .unwrap_or_else(|| "null".to_string());
        let note_json = note
            .map(|v| format!(r#""{}""#, v.replace('"', "\\\"")))
            .unwrap_or_else(|| "null".to_string());
        let pm_json = payment_method
            .map(|v| format!(r#""{}""#, v))
            .unwrap_or_else(|| "null".to_string());

        out.push_str(&format!(
            r#"{{"id":{},"order_no":"{}","source":"{}","dine_type":"{}","table_no":{},"status":"{}","amount_total":{},"note":{},"payment_status":"{}","payment_method":{},"amount_paid":{},"created_at":"{}","updated_at":"{}"}}"#,
            id, order_no, source, dine_type, table_no_json, status, amount_total,
            note_json, payment_status, pm_json, amount_paid, created_at, updated_at
        ));
    }

    out.push(']');
    Ok(out)
}

pub fn get_local_ip() -> Option<String> {
    // UDP routing-table trick: no packets sent, just determines outbound interface.
    // Works on Android WiFi even without internet — only needs a default route.
    for target in &["8.8.8.8:80", "1.1.1.1:80", "192.168.1.1:80"] {
        if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
            if socket.connect(target).is_ok() {
                if let Ok(addr) = socket.local_addr() {
                    let ip = addr.ip().to_string();
                    if !ip.starts_with("127.") && !ip.starts_with("169.254.") {
                        return Some(ip);
                    }
                }
            }
        }
    }
    None
}
