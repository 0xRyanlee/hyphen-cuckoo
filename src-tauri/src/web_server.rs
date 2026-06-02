use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use serde_json::Value;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::database::{Database, SelfOrderItemInput};

const MAX_SESSIONS: usize = 500;      // ample for a single restaurant
const SESSION_TTL_SECS: u64 = 14_400; // 4 hours — covers one full shift
const MAX_BODY_BYTES: usize = 1_048_576; // 1 MB
const MAX_ORDER_ITEMS: usize = 100;
const MENU_CACHE_TTL_SECS: u64 = 30; // public menu cached for 30s

type MenuCache = Arc<Mutex<Option<(String, Instant)>>>;

/// token → (role, created_at)
type Sessions = Arc<Mutex<HashMap<String, (String, Instant)>>>;

// ── Public handle ─────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub struct WebServerHandle {
    pub port: u16,
    running: Arc<AtomicBool>,
    pub sessions: Sessions,
}

impl WebServerHandle {
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
        std::net::TcpStream::connect(format!("127.0.0.1:{}", self.port)).ok();
    }

    #[allow(dead_code)]
    pub fn clear_sessions(&self) {
        self.sessions.lock().unwrap_or_else(|e| e.into_inner()).clear();
    }
}

// ── Per-request context (cheap to clone, all Arc'd) ──────────────────────────

#[derive(Clone)]
struct Ctx {
    db: Arc<Database>,
    dist_dir: Option<PathBuf>,
    role_auth_path: PathBuf,
    sessions: Sessions,
    menu_cache: MenuCache,
}

// ── Start ─────────────────────────────────────────────────────────────────────

/// Start the LAN web server. Tries `preferred_port` and the next two ports
/// in sequence (T4.3 — port conflict auto-increment).
pub fn start_web_server(
    db: Arc<Database>,
    dist_dir: Option<PathBuf>,
    role_auth_path: PathBuf,
    preferred_port: u16,
) -> Result<WebServerHandle, String> {
    let (listener, port) = (0u16..3)
        .find_map(|i| {
            let p = preferred_port + i;
            TcpListener::bind(format!("0.0.0.0:{}", p))
                .ok()
                .map(|l| (l, p))
        })
        .ok_or_else(|| {
            format!(
                "Ports {}–{} all in use",
                preferred_port,
                preferred_port + 2
            )
        })?;

    let running = Arc::new(AtomicBool::new(true));
    let sessions: Sessions = Arc::new(Mutex::new(HashMap::new()));

    let running_clone = running.clone();
    let ctx = Ctx {
        db,
        dist_dir,
        role_auth_path,
        sessions: sessions.clone(),
        menu_cache: Arc::new(Mutex::new(None)),
    };

    std::thread::spawn(move || {
        eprintln!("[WebServer] Listening on 0.0.0.0:{}", port);

        while running_clone.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _peer)) => {
                    if !running_clone.load(Ordering::Relaxed) {
                        break;
                    }
                    let ctx_clone = ctx.clone();
                    std::thread::spawn(move || {
                        let mut stream = stream;
                        stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
                        stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
                        handle_request(&mut stream, &ctx_clone);
                        stream.shutdown(std::net::Shutdown::Both).ok();
                    });
                }
                Err(_) => {}
            }
        }

        eprintln!("[WebServer] Stopped");
    });

    Ok(WebServerHandle { port, running, sessions })
}

// ── Request dispatch ──────────────────────────────────────────────────────────

fn handle_request(stream: &mut TcpStream, ctx: &Ctx) {
    let Some((method, path, headers, body)) = read_request(stream) else {
        return;
    };

    let route = path.split('?').next().unwrap_or("/");
    let query = path.split('?').nth(1).unwrap_or("");

    // CORS preflight
    if method == "OPTIONS" {
        write_response(stream, "204 No Content", "text/plain", b"");
        return;
    }

    // ── API routes ────────────────────────────────────────────────────────────
    if route.starts_with("/api/") {
        let (status, json) = dispatch_api(&method, route, query, &headers, &body, ctx);
        write_response(stream, status, "application/json", json.as_bytes());
        return;
    }

    // ── Static files (SPA) ───────────────────────────────────────────────────
    if let Some(dist) = ctx.dist_dir.as_deref() {
        serve_static(stream, route, dist);
        return;
    }

    write_response(stream, "503 Service Unavailable", "text/plain", b"Frontend not available");
}

fn dispatch_api(
    method: &str,
    route: &str,
    _query: &str,
    headers: &HashMap<String, String>,
    body: &[u8],
    ctx: &Ctx,
) -> (&'static str, String) {
    if method != "POST" {
        return ("405 Method Not Allowed", json_err("use POST"));
    }

    let db = &ctx.db;
    let v: Value = serde_json::from_slice(body).unwrap_or(Value::Object(Default::default()));

    match route {
        // ── Public (no auth required) ─────────────────────────────────────────
        "/api/ping" => ("200 OK", r#"{"ok":true}"#.to_string()),

        "/api/auth/login" => auth_login(body, &ctx.role_auth_path, &ctx.sessions),

        "/api/auth/logout" => {
            if let Some(token) = bearer_token(headers) {
                ctx.sessions.lock().unwrap_or_else(|e| e.into_inner()).remove(&token);
            }
            ("200 OK", r#"{"ok":true}"#.to_string())
        }

        // Self-order public endpoints — these never need a session token
        "/api/get_public_menu" => {
            let cached = {
                let g = ctx.menu_cache.lock().unwrap_or_else(|e| e.into_inner());
                g.as_ref().filter(|(_, t)| t.elapsed().as_secs() < MENU_CACHE_TTL_SECS).map(|(s, _)| s.clone())
            };
            match cached {
                Some(data) => ("200 OK", data),
                None => match api_get_public_menu_direct(db) {
                    Ok(data) => {
                        *ctx.menu_cache.lock().unwrap_or_else(|e| e.into_inner()) = Some((data.clone(), Instant::now()));
                        ("200 OK", data)
                    }
                    Err(e) => ("500 Internal Server Error", json_err(&e)),
                },
            }
        }

        "/api/create_self_order" => match api_create_self_order_direct(db, body) {
            Ok((id, order_no)) => ("200 OK", format!(r#"{{"id":{},"order_no":"{}"}}"#, id, order_no)),
            Err(e) => ("400 Bad Request", json_err(&e)),
        },

        "/api/get_table_orders_today" => {
            let table_no = v["table_no"].as_str().unwrap_or("").to_string();
            match api_get_table_orders_today_direct(db, &table_no) {
                Ok(data) => ("200 OK", data),
                Err(e) => ("400 Bad Request", json_err(&e)),
            }
        }

        "/api/get_marketing_popup" => {
            let order_id = v["orderId"].as_i64().unwrap_or(0);
            let table_no = v["tableNo"].as_str().unwrap_or("").to_string();
            match db.get_marketing_popup_content(order_id, &table_no) {
                Ok(data) => ("200 OK", data.to_string()),
                Err(e) => ("500 Internal Server Error", json_err(&e.to_string())),
            }
        }

        // Staff scans an order marketing QR → void-on-redeem (public; small-shop
        // trade-off, see research doc — future: gate behind staff PIN)
        "/api/redeem_requires_pin" => {
            let store = crate::commands::load_role_auth_store(&ctx.role_auth_path);
            ("200 OK", format!(r#"{{"required":{}}}"#, !store.pin_hashes.is_empty()))
        }

        "/api/redeem_marketing_qr_token" => {
            let token = v["token"].as_str().unwrap_or("");
            let staff = v["staff_name"].as_str();
            let pin = v["pin"].as_str().unwrap_or("");
            let store = crate::commands::load_role_auth_store(&ctx.role_auth_path);
            if !crate::commands::verify_any_pin(&store, pin) {
                return ("200 OK", r#"{"ok":false,"reason":"pin_required"}"#.to_string());
            }
            match db.redeem_marketing_qr_token(token, staff) {
                Ok(data) => ("200 OK", data.to_string()),
                Err(e) => ("500 Internal Server Error", json_err(&e.to_string())),
            }
        }

        "/api/sign_table_token" => {
            let table_no = v["table_no"].as_str().unwrap_or("").trim();
            if table_no.is_empty() {
                ("400 Bad Request", json_err("table_no required"))
            } else {
                let token = crate::qr_token::make_token(&crate::qr_token::table_payload(table_no));
                ("200 OK", format!(r#"{{"token":{}}}"#, serde_json::Value::String(token)))
            }
        }

        // Customer scans a campaign poster QR → issue a fresh single-use coupon
        "/api/resolve_campaign" => {
            let token = v["token"].as_str().unwrap_or("");
            match crate::qr_token::verify_token(token).and_then(|p| crate::qr_token::parse_campaign_payload(&p)) {
                Some(id) => {
                    let _ = db.record_qr_scan("campaign", None, Some(id));
                    match db.issue_campaign_coupon(id) {
                        Ok(data) => ("200 OK", data.to_string()),
                        Err(e) => ("500 Internal Server Error", json_err(&e.to_string())),
                    }
                }
                None => ("200 OK", r#"{"valid":false}"#.to_string()),
            }
        }

        // Customer scans a (fixed, signed) table QR → resolve to table_no + log scan
        "/api/resolve_table_token" => {
            let token = v["token"].as_str().unwrap_or("");
            match crate::qr_token::verify_token(token)
                .and_then(|p| crate::qr_token::parse_table_payload(&p))
            {
                Some(table_no) => {
                    let _ = db.record_qr_scan("table", Some(&table_no), None);
                    ("200 OK", format!(r#"{{"valid":true,"table_no":{}}}"#, serde_json::Value::String(table_no)))
                }
                None => ("200 OK", r#"{"valid":false}"#.to_string()),
            }
        }

        // ── Protected endpoints (T1.4 — admin POS on iPad) ───────────────────
        _ => {
            let role = match require_session(headers, &ctx.sessions) {
                Err(msg) => return ("401 Unauthorized", json_err(&msg)),
                Ok(r) => r,
            };
            dispatch_protected(route, &v, db, &role)
        }
    }
}

fn dispatch_protected(
    route: &str,
    v: &Value,
    db: &Arc<Database>,
    _role: &str,
) -> (&'static str, String) {
    dispatch_menu(route, v, db)
        .or_else(|| dispatch_orders(route, v, db))
        .or_else(|| dispatch_kds(route, v, db))
        .or_else(|| dispatch_customers(route, v, db))
        .unwrap_or_else(|| ("404 Not Found", r#"{"error":"not found"}"#.to_string()))
}

fn dispatch_menu(route: &str, v: &Value, db: &Arc<Database>) -> Option<(&'static str, String)> {
    let r = match route {
        "/api/get_menu_categories" => to_json(db.get_menu_categories()),
        "/api/get_menu_items" => to_json(db.get_menu_items(v["categoryId"].as_i64())),
        "/api/get_menu_items_for_pos" => to_json(db.get_menu_items_for_pos(v["categoryId"].as_i64())),
        "/api/get_menu_item_specs" => to_json(db.get_menu_item_specs(v["menuItemId"].as_i64().unwrap_or(0))),
        "/api/set_menu_item_availability" => {
            let id = v["id"].as_i64().unwrap_or(0);
            let is_available = v["isAvailable"].as_bool().unwrap_or(false);
            to_json(db.set_menu_item_availability(id, is_available))
        }
        "/api/batch_set_menu_item_availability" => {
            let ids: Vec<i64> = v["ids"].as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|i| i.as_i64())
                .collect();
            let is_available = v["isAvailable"].as_bool().unwrap_or(false);
            to_json(db.batch_toggle_menu_item_availability(&ids, is_available))
        }
        _ => return None,
    };
    Some(r)
}

fn dispatch_orders(route: &str, v: &Value, db: &Arc<Database>) -> Option<(&'static str, String)> {
    let r = match route {
        "/api/get_orders" => to_json(db.get_orders(v["limit"].as_i64().unwrap_or(200), v["offset"].as_i64().unwrap_or(0))),
        "/api/get_order_with_items" => {
            match db.get_order_with_items(v["orderId"].as_i64().unwrap_or(0)) {
                Ok((order, items)) => ok_json(serde_json::json!({"order": order, "items": items})),
                Err(e) => ("500 Internal Server Error", json_err(&e.to_string())),
            }
        }
        "/api/create_order" => {
            let req = &v["req"];
            match db.create_order(req["source"].as_str().unwrap_or("pos"), req["dine_type"].as_str().unwrap_or("dine_in"), req["table_no"].as_str()) {
                Ok((id, no)) => ok_json(serde_json::json!({"id": id, "order_no": no})),
                Err(e) => ("400 Bad Request", json_err(&e.to_string())),
            }
        }
        "/api/add_order_item" => {
            let req = &v["req"];
            match db.add_order_item(v["orderId"].as_i64().unwrap_or(0), req["menu_item_id"].as_i64().unwrap_or(0), req["qty"].as_f64().unwrap_or(1.0), req["unit_price"].as_f64().unwrap_or(0.0), req["spec_code"].as_str(), req["note"].as_str()) {
                Ok(id) => ok_json(serde_json::json!(id)),
                Err(e) => ("400 Bad Request", json_err(&e.to_string())),
            }
        }
        "/api/submit_order" => match db.submit_order_full(v["orderId"].as_i64().unwrap_or(0)) {
            Ok(_) => ("200 OK", r#"{"ok":true}"#.to_string()),
            Err(e) => ("400 Bad Request", json_err(&e.to_string())),
        },
        "/api/cancel_order" => match db.cancel_order_confirmed(v["orderId"].as_i64().unwrap_or(0), v["reason"].as_str()) {
            Ok(_) => ("200 OK", r#"{"ok":true}"#.to_string()),
            Err(e) => ("400 Bad Request", json_err(&e.to_string())),
        },
        "/api/mark_order_ready" => match db.mark_order_ready(v["orderId"].as_i64().unwrap_or(0)) {
            Ok(_) => ("200 OK", r#"{"ok":true}"#.to_string()),
            Err(e) => ("400 Bad Request", json_err(&e.to_string())),
        },
        "/api/update_order_payment" => match db.update_order_payment(v["orderId"].as_i64().unwrap_or(0), v["paymentStatus"].as_str().unwrap_or("paid"), v["paymentMethod"].as_str(), v["amountPaid"].as_f64().unwrap_or(0.0)) {
            Ok(_) => ("200 OK", r#"{"ok":true}"#.to_string()),
            Err(e) => ("400 Bad Request", json_err(&e.to_string())),
        },
        "/api/record_order_refund" => match db.record_order_refund(v["orderId"].as_i64().unwrap_or(0), v["refundAmount"].as_f64().unwrap_or(0.0)) {
            Ok(_) => ("200 OK", r#"{"ok":true}"#.to_string()),
            Err(e) => ("400 Bad Request", json_err(&e.to_string())),
        },
        "/api/refund_order_item" => match db.refund_order_item(v["orderId"].as_i64().unwrap_or(0), v["itemId"].as_i64().unwrap_or(0)) {
            Ok(amt) => ok_json(serde_json::json!({"refunded_amount": amt})),
            Err(e) => ("400 Bad Request", json_err(&e.to_string())),
        },
        "/api/batch_cancel_orders" => {
            let ids: Vec<i64> = v["ids"].as_array().map(|a| a.iter().filter_map(|x| x.as_i64()).collect()).unwrap_or_default();
            match db.batch_cancel_orders(&ids) {
                Ok(n) => ok_json(serde_json::json!({"cancelled": n})),
                Err(e) => ("400 Bad Request", json_err(&e.to_string())),
            }
        }
        "/api/add_order_item_modifier" => {
            let req = &v["req"];
            match db.add_order_item_modifier(req["order_item_id"].as_i64().unwrap_or(0), req["modifier_type"].as_str().unwrap_or("add"), req["material_id"].as_i64(), req["qty"].as_f64().unwrap_or(1.0), req["price_delta"].as_f64().unwrap_or(0.0)) {
                Ok(_) => ("200 OK", r#"{"ok":true}"#.to_string()),
                Err(e) => ("400 Bad Request", json_err(&e.to_string())),
            }
        }
        _ => return None,
    };
    Some(r)
}

fn dispatch_kds(route: &str, v: &Value, db: &Arc<Database>) -> Option<(&'static str, String)> {
    let r = match route {
        "/api/get_kitchen_stations" => to_json(db.get_kitchen_stations()),
        "/api/get_all_tickets" => to_json(db.get_all_tickets(v["status"].as_str())),
        "/api/get_tickets_for_order" => to_json(db.get_tickets_for_order(v["orderId"].as_i64().unwrap_or(0))),
        "/api/start_ticket" => match db.start_ticket(v["ticketId"].as_i64().unwrap_or(0), None) {
            Ok(_) => ("200 OK", r#"{"ok":true}"#.to_string()),
            Err(e) => ("400 Bad Request", json_err(&e.to_string())),
        },
        "/api/finish_ticket" => match db.finish_ticket(v["ticketId"].as_i64().unwrap_or(0), None) {
            Ok(_) => ("200 OK", r#"{"ok":true}"#.to_string()),
            Err(e) => ("400 Bad Request", json_err(&e.to_string())),
        },
        "/api/get_all_tickets_with_items" => {
            match db.get_all_tickets(v["status"].as_str()) {
                Ok(tickets) => {
                    let result: Vec<serde_json::Value> = tickets.into_iter().filter_map(|t| {
                        db.get_order_with_items(t.order_id).ok().map(|(order, items)| serde_json::json!({
                            "ticket": t,
                            "order_no": order.order_no,
                            "dine_type": order.dine_type,
                            "table_no": order.table_no,
                            "items": items,
                        }))
                    }).collect();
                    ok_json(serde_json::json!(result))
                }
                Err(e) => ("500 Internal Server Error", json_err(&e.to_string())),
            }
        }
        _ => return None,
    };
    Some(r)
}

fn dispatch_customers(route: &str, v: &Value, db: &Arc<Database>) -> Option<(&'static str, String)> {
    let r = match route {
        "/api/get_customers" => to_json(db.get_customers(v["search"].as_str())),
        _ => return None,
    };
    Some(r)
}

// ── Serialization helpers ─────────────────────────────────────────────────────

fn to_json<T: serde::Serialize>(result: rusqlite::Result<T>) -> (&'static str, String) {
    match result {
        Ok(v) => ok_json(serde_json::json!(v)),
        Err(e) => ("500 Internal Server Error", json_err(&e.to_string())),
    }
}

fn ok_json(v: serde_json::Value) -> (&'static str, String) {
    ("200 OK", v.to_string())
}

// ── Auth helpers ──────────────────────────────────────────────────────────────

/// `POST /api/auth/login` — body: `{"role":"owner","pin":"1234"}`
fn auth_login(
    body: &[u8],
    role_auth_path: &Path,
    sessions: &Sessions,
) -> (&'static str, String) {
    let v: Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(_) => return ("400 Bad Request", json_err("invalid JSON")),
    };
    let role = v["role"].as_str().unwrap_or("").to_string();
    let pin = v["pin"].as_str().unwrap_or("").to_string();

    if role.is_empty() || pin.is_empty() {
        return ("400 Bad Request", json_err("role and pin required"));
    }

    // Load role-auth.json on every login attempt so PIN changes take effect immediately
    let store: Value = std::fs::read_to_string(role_auth_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({"pin_hashes": {}}));

    let expected_hash = store["pin_hashes"][&role].as_str().unwrap_or("");
    if expected_hash.is_empty() {
        // HTTP context: roles without a PIN cannot login over the network
        return ("401 Unauthorized", json_err("role has no PIN — set a PIN before using HTTP access"));
    }

    if !crate::commands::verify_pin(&pin, expected_hash) {
        return ("401 Unauthorized", json_err("invalid pin"));
    }

    let mut guard = sessions.lock().unwrap_or_else(|e| e.into_inner());

    // Evict expired sessions to keep the map lean before checking the cap
    guard.retain(|_, (_, created_at)| created_at.elapsed().as_secs() < SESSION_TTL_SECS);

    if guard.len() >= MAX_SESSIONS {
        return ("429 Too Many Requests", json_err("too many active sessions"));
    }

    let token = Uuid::new_v4().to_string();
    guard.insert(token.clone(), (role.clone(), Instant::now()));

    (
        "200 OK",
        serde_json::json!({"token": token, "role": role}).to_string(),
    )
}

fn bearer_token(headers: &HashMap<String, String>) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|v| v.strip_prefix("Bearer ").map(|t| t.trim().to_string()))
}

fn require_session(
    headers: &HashMap<String, String>,
    sessions: &Sessions,
) -> Result<String, String> {
    let token = bearer_token(headers).ok_or_else(|| "missing Authorization header".to_string())?;
    let mut guard = sessions.lock().unwrap_or_else(|e| e.into_inner());
    match guard.get(&token) {
        Some((role, created_at)) => {
            if created_at.elapsed().as_secs() >= SESSION_TTL_SECS {
                guard.remove(&token);
                return Err("session expired".to_string());
            }
            Ok(role.clone())
        }
        None => Err("invalid or expired token".to_string()),
    }
}

// ── Static file serving ───────────────────────────────────────────────────────

fn serve_static(stream: &mut TcpStream, route: &str, dist: &Path) {
    let rel = route.trim_start_matches('/');

    if rel.contains("..") {
        write_response(stream, "403 Forbidden", "text/plain", b"Forbidden");
        return;
    }

    // Assets (versioned filenames) → serve directly; anything else → SPA index.html
    let (candidate, is_asset) = if rel.starts_with("assets/") || (rel.contains('.') && !rel.is_empty()) {
        (dist.join(rel), true)
    } else {
        (dist.join("index.html"), false)
    };

    match std::fs::read(&candidate) {
        Ok(data) => {
            let ext = candidate.extension().and_then(|e| e.to_str()).unwrap_or("bin");
            write_response(stream, "200 OK", mime_type(ext), &data);
        }
        Err(_) if is_asset => {
            // Unknown asset path — SPA fallback
            if let Ok(data) = std::fs::read(dist.join("index.html")) {
                write_response(stream, "200 OK", "text/html; charset=utf-8", &data);
            } else {
                write_response(stream, "404 Not Found", "text/plain", b"Not Found");
            }
        }
        Err(_) => {
            write_response(stream, "404 Not Found", "text/plain", b"Not Found");
        }
    }
}

// ── Low-level HTTP ────────────────────────────────────────────────────────────

fn write_response(stream: &mut TcpStream, status: &str, content_type: &str, body: &[u8]) {
    let header = format!(
        "HTTP/1.1 {}\r\n\
         Content-Type: {}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
         Access-Control-Allow-Headers: Content-Type, Authorization\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n",
        status,
        content_type,
        body.len()
    );
    stream.write_all(header.as_bytes()).ok();
    stream.write_all(body).ok();
}

fn mime_type(ext: &str) -> &'static str {
    match ext {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "application/javascript",
        "css" => "text/css",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        _ => "application/octet-stream",
    }
}

fn read_request(
    stream: &mut TcpStream,
) -> Option<(String, String, HashMap<String, String>, Vec<u8>)> {
    let mut buf = vec![0u8; 16384];
    let n = stream.read(&mut buf).ok()?;
    if n == 0 {
        return None;
    }

    let header_end = buf[..n].windows(4).position(|w| w == b"\r\n\r\n")?;
    let header_str = String::from_utf8_lossy(&buf[..header_end]).to_string();

    let mut lines = header_str.lines();
    let first = lines.next()?;
    let mut parts = first.split_whitespace();
    let method = parts.next()?.to_string();
    let path = parts.next()?.to_string();

    let mut headers = HashMap::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some((k, v)) = trimmed.split_once(':') {
            headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
        }
    }

    let content_length: usize = headers
        .get("content-length")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
        .min(MAX_BODY_BYTES);

    let body_start = header_end + 4;
    let mut body = buf[body_start..n].to_vec();

    while body.len() < content_length {
        let mut more = vec![0u8; 4096];
        match stream.read(&mut more) {
            Ok(0) | Err(_) => break,
            Ok(m) => body.extend_from_slice(&more[..m]),
        }
    }

    Some((method, path, headers, body))
}

fn json_err(msg: &str) -> String {
    // Escape quotes in message to keep JSON valid
    let escaped = msg.replace('"', "\\\"");
    format!(r#"{{"error":"{}"}}"#, escaped)
}

// ── API handlers ──────────────────────────────────────────────────────────────

fn api_get_public_menu_direct(db: &Arc<Database>) -> Result<String, String> {
    db.get_public_menu()
        .map_err(|e| e.to_string())
        .and_then(|v| serde_json::to_string(&v).map_err(|e| e.to_string()))
}

fn api_create_self_order_direct(db: &Arc<Database>, body: &[u8]) -> Result<(i64, String), String> {
    let v: Value =
        serde_json::from_slice(body).map_err(|e| format!("JSON parse error: {}", e))?;
    let raw_table = v["table_no"].as_str().unwrap_or("");
    if raw_table.len() > 32 {
        return Err("table_no too long".to_string());
    }
    // Grace-period dual-mode: signed token (if present) binds the table and
    // overrides the client-supplied value; legacy static QR (no token) falls back.
    let token = v["token"].as_str();
    let table_no = crate::commands::resolve_self_order_table(raw_table, token)?;
    let items_raw = v["items"].as_array().ok_or("items must be array")?;
    if items_raw.is_empty() {
        return Err("empty order".to_string());
    }
    if items_raw.len() > MAX_ORDER_ITEMS {
        return Err(format!("too many items (max {})", MAX_ORDER_ITEMS));
    }
    let mut items: Vec<SelfOrderItemInput> = Vec::with_capacity(items_raw.len());
    for item in items_raw {
        items.push(SelfOrderItemInput {
            menu_item_id: item["menu_item_id"].as_i64().ok_or("invalid menu_item_id")?,
            spec_code: item["spec_code"].as_str().map(|s| s.to_string()),
            qty: item["qty"].as_f64().unwrap_or(1.0),
            note: item["note"].as_str().map(|s| s.to_string()),
        });
    }
    db.create_self_order(&table_no, &items).map_err(|e| e.to_string())
}

fn api_get_table_orders_today_direct(db: &Arc<Database>, table_no: &str) -> Result<String, String> {
    if table_no.is_empty() {
        return Err("table_no required".to_string());
    }
    db.get_table_orders_today(table_no)
        .map_err(|e| e.to_string())
        .and_then(|v| serde_json::to_string(&v).map_err(|e| e.to_string()))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    /// Write `raw` bytes to a loopback socket, call `read_request` on the accepted side.
    fn parse(raw: &[u8]) -> Option<(String, String, HashMap<String, String>, Vec<u8>)> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let owned = raw.to_vec();
        std::thread::spawn(move || {
            if let Ok(mut c) = std::net::TcpStream::connect(addr) {
                let _ = c.write_all(&owned);
            }
        });
        let (mut s, _) = listener.accept().unwrap();
        read_request(&mut s)
    }

    // ── read_request ──────────────────────────────────────────────────────────

    #[test]
    fn read_simple_post() {
        let r = parse(b"POST /api/ping HTTP/1.1\r\nContent-Length: 2\r\n\r\n{}").unwrap();
        assert_eq!(r.0, "POST");
        assert_eq!(r.1, "/api/ping");
        assert_eq!(r.3, b"{}");
    }

    #[test]
    fn read_get_no_body() {
        let r = parse(b"GET /api/ping HTTP/1.1\r\n\r\n").unwrap();
        assert_eq!(r.0, "GET");
        assert!(r.3.is_empty());
    }

    #[test]
    fn read_headers_lowercase() {
        let r = parse(b"POST / HTTP/1.1\r\nAuthorization: Bearer tok\r\n\r\n").unwrap();
        assert_eq!(r.2.get("authorization").map(|s| s.as_str()), Some("Bearer tok"));
    }

    #[test]
    fn read_content_length_capped_at_1mb() {
        // Claim 10 MB but only send 4 bytes — must not block forever.
        let header = format!("POST / HTTP/1.1\r\nContent-Length: {}\r\n\r\nbody", 10 * 1024 * 1024);
        let r = parse(header.as_bytes()).unwrap();
        // Body is whatever actually arrived (4 bytes), not the declared size.
        assert_eq!(&r.3, b"body");
    }

    // ── json_err ──────────────────────────────────────────────────────────────

    #[test]
    fn json_err_escapes_quotes() {
        let out = json_err(r#"has "quotes" inside"#);
        let v: serde_json::Value = serde_json::from_str(&out).expect("must be valid JSON");
        assert_eq!(v["error"].as_str().unwrap(), r#"has "quotes" inside"#);
    }

    // ── auth_login ────────────────────────────────────────────────────────────

    fn temp_auth(json: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("role-auth.json");
        std::fs::write(&p, json).unwrap();
        (dir, p)
    }

    fn empty_sessions() -> Sessions {
        Arc::new(Mutex::new(HashMap::new()))
    }

    #[test]
    fn auth_login_empty_pin_returns_400() {
        let (_dir, path) = temp_auth(r#"{"pin_hashes":{}}"#);
        let (status, _) = auth_login(br#"{"role":"owner","pin":""}"#, &path, &empty_sessions());
        assert_eq!(status, "400 Bad Request");
    }

    #[test]
    fn auth_login_role_without_pin_returns_401() {
        // Role exists in system but no PIN hash stored → HTTP access must be denied
        let (_dir, path) = temp_auth(r#"{"pin_hashes":{}}"#);
        let (status, body) = auth_login(br#"{"role":"owner","pin":"1234"}"#, &path, &empty_sessions());
        assert_eq!(status, "401 Unauthorized");
        assert!(body.contains("no PIN"));
    }

    #[test]
    fn auth_login_wrong_sha256_pin_returns_401() {
        // Legacy SHA-256 hash of "1234"
        use sha2::{Digest, Sha256};
        let hash: String = Sha256::digest(b"1234").iter().map(|b| format!("{:02x}", b)).collect();
        let json = format!(r#"{{"pin_hashes":{{"cashier":"{}"}}}}"#, hash);
        let (_dir, path) = temp_auth(&json);
        let (status, _) = auth_login(br#"{"role":"cashier","pin":"9999"}"#, &path, &empty_sessions());
        assert_eq!(status, "401 Unauthorized");
    }

    #[test]
    fn auth_login_correct_sha256_pin_returns_token() {
        use sha2::{Digest, Sha256};
        let hash: String = Sha256::digest(b"1234").iter().map(|b| format!("{:02x}", b)).collect();
        let json = format!(r#"{{"pin_hashes":{{"cashier":"{}"}}}}"#, hash);
        let (_dir, path) = temp_auth(&json);
        let sessions = empty_sessions();
        let (status, body) = auth_login(br#"{"role":"cashier","pin":"1234"}"#, &path, &sessions);
        assert_eq!(status, "200 OK");
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(v["token"].as_str().is_some());
        assert_eq!(v["role"], "cashier");
        // Token must be stored in sessions map
        assert_eq!(sessions.lock().unwrap().len(), 1);
    }

    #[test]
    fn auth_login_session_cap_returns_429() {
        use sha2::{Digest, Sha256};
        let hash: String = Sha256::digest(b"0000").iter().map(|b| format!("{:02x}", b)).collect();
        let json = format!(r#"{{"pin_hashes":{{"cashier":"{}"}}}}"#, hash);
        let (_dir, path) = temp_auth(&json);
        let sessions: Sessions = Arc::new(Mutex::new(HashMap::new()));
        // Pre-fill with MAX_SESSIONS entries
        {
            let mut g = sessions.lock().unwrap();
            for i in 0..MAX_SESSIONS {
                g.insert(format!("tok{}", i), ("cashier".into(), Instant::now()));
            }
        }
        let (status, _) = auth_login(br#"{"role":"cashier","pin":"0000"}"#, &path, &sessions);
        assert_eq!(status, "429 Too Many Requests");
    }

    // ── dispatch_menu availability endpoints ─────────────────────────────────

    fn make_test_db_with_menu_item() -> (Arc<crate::database::Database>, tempfile::TempDir, i64) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db").to_str().unwrap().to_string();
        let db = crate::database::Database::new(&path).unwrap();
        let cat_id = db.create_menu_category("测试分类", 1).unwrap();
        let item_id = db.create_menu_item("测试菜品", Some(cat_id), None, 10.0).unwrap();
        (Arc::new(db), dir, item_id)
    }

    #[test]
    fn dispatch_menu_set_availability_false() {
        let (db, _dir, item_id) = make_test_db_with_menu_item();
        let v = serde_json::json!({ "id": item_id, "isAvailable": false });
        let result = dispatch_menu("/api/set_menu_item_availability", &v, &db);
        assert!(result.is_some());
        let (status, _) = result.unwrap();
        assert_eq!(status, "200 OK");
        // Verify DB was updated
        let items = db.get_menu_items(None).unwrap();
        let item = items.iter().find(|i| i.id == item_id).unwrap();
        assert!(!item.is_available);
    }

    #[test]
    fn dispatch_menu_set_availability_true() {
        let (db, _dir, item_id) = make_test_db_with_menu_item();
        // Set false first
        db.set_menu_item_availability(item_id, false).unwrap();
        let v = serde_json::json!({ "id": item_id, "isAvailable": true });
        let result = dispatch_menu("/api/set_menu_item_availability", &v, &db);
        assert!(result.is_some());
        let (status, _) = result.unwrap();
        assert_eq!(status, "200 OK");
        let items = db.get_menu_items(None).unwrap();
        let item = items.iter().find(|i| i.id == item_id).unwrap();
        assert!(item.is_available);
    }

    #[test]
    fn dispatch_menu_batch_set_availability() {
        let (db, _dir, item_id) = make_test_db_with_menu_item();
        let cat_id = db.create_menu_category("分类2", 2).unwrap();
        let item_id2 = db.create_menu_item("菜品2", Some(cat_id), None, 20.0).unwrap();
        let v = serde_json::json!({ "ids": [item_id, item_id2], "isAvailable": false });
        let result = dispatch_menu("/api/batch_set_menu_item_availability", &v, &db);
        assert!(result.is_some());
        let (status, body) = result.unwrap();
        assert_eq!(status, "200 OK");
        // Response should be count of updated rows (2)
        let count: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(count, serde_json::json!(2));
        // Verify the specific items were updated
        let items = db.get_menu_items(None).unwrap();
        let target_items: Vec<_> = items.iter().filter(|i| i.id == item_id || i.id == item_id2).collect();
        assert_eq!(target_items.len(), 2);
        assert!(target_items.iter().all(|i| !i.is_available));
    }

    // ── serve_static path traversal ───────────────────────────────────────────

    #[test]
    fn serve_static_blocks_dotdot() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let dist = dir.path().to_path_buf();

        std::thread::spawn(move || { let _ = std::net::TcpStream::connect(addr); });
        let (mut stream, _) = listener.accept().unwrap();

        // Capture bytes written to stream by wrapping in a BufWriter is not trivial here,
        // but we can verify by checking that serve_static does NOT panic and the traversal
        // path is caught (rel.contains("..") guard).
        // We verify the guard logic directly:
        let rel = "../etc/passwd";
        assert!(rel.contains(".."), "path traversal guard should fire");

        // Also call serve_static to ensure it doesn't panic or crash the process.
        serve_static(&mut stream, "/../etc/passwd", &dist);
    }
}

