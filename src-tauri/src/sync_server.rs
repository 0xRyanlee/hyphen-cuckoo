use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

use rusqlite::{params, Connection};

pub struct SyncServerHandle {
    pub port: u16,
    running: Arc<AtomicBool>,
}

impl SyncServerHandle {
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
        // Unblock accept() by connecting to self
        std::net::TcpStream::connect(format!("127.0.0.1:{}", self.port)).ok();
    }
}

pub fn start_server(db_path: PathBuf, port: u16) -> Result<SyncServerHandle, String> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .map_err(|e| format!("端口 {} 绑定失败: {}", port, e))?;

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

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
                    handle_connection(stream, &conn);
                }
                Err(_) => {}
            }
        }
    });

    Ok(SyncServerHandle { port, running })
}

fn handle_connection(mut stream: TcpStream, conn: &Connection) {
    let mut buf = [0u8; 2048];
    let n = match stream.read(&mut buf) {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let req = String::from_utf8_lossy(&buf[..n]);
    let first_line = req.lines().next().unwrap_or("");
    let path = first_line.split_whitespace().nth(1).unwrap_or("/");
    let route = path.split('?').next().unwrap_or("");
    let query = path.split('?').nth(1).unwrap_or("");

    let since_epoch_s: i64 = query
        .split('&')
        .find(|p| p.starts_with("since="))
        .and_then(|p| p.strip_prefix("since="))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let (status, body) = match route {
        "/api/ping" => ("200 OK", r#"{"ok":true}"#.to_string()),
        "/api/orders" => match query_orders_since(conn, since_epoch_s) {
            Ok(json) => ("200 OK", json),
            Err(e) => (
                "500 Internal Server Error",
                format!(r#"{{"error":"{}"}}"#, e),
            ),
        },
        _ => ("404 Not Found", r#"{"error":"not found"}"#.to_string()),
    };

    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        body.len(),
        body
    );
    stream.write_all(response.as_bytes()).ok();
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
    // UDP connect trick: no packets sent, just determines routing interface
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|a| a.ip().to_string())
}
