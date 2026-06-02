use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LanPrinter {
    pub ip: String,
    pub port: i32,
    pub sn: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrinterStatus {
    pub printer_id: i64,
    pub online: bool,
    pub has_paper: bool,
    pub cover_open: bool,
    pub error_msg: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrintTaskRecord {
    pub id: i64,
    pub task_type: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<i64>,
    pub content: String,
    pub status: String,
    pub printer_name: Option<String>,
    pub created_at: String,
    pub printed_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DebugPrintResult {
    pub file_path: String,
    pub html_preview: String,
    pub byte_count: usize,
}

fn debug_output_dir() -> Result<PathBuf, String> {
    let base = dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("Cuckoo")
        .join("debug_prints");
    std::fs::create_dir_all(&base).map_err(|e| format!("創建調試目錄失敗: {}", e))?;
    Ok(base)
}

fn sanitize_filename(input: Option<&str>, default_stem: &str, ext: &str) -> String {
    let raw = input.unwrap_or(default_stem).trim();
    let stem = if raw.is_empty() { default_stem } else { raw };
    let filtered: String = stem
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    let safe = if filtered.is_empty() {
        default_stem.to_string()
    } else {
        filtered
    };
    format!("{}.{}", safe, ext)
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

// ==================== 飛鵝雲 API ====================

const FEIE_API_URL: &str = "http://api.feieyun.cn/Api/Open/";

fn compute_feie_signature(user: &str, ukey: &str, stime: &str) -> String {
    use sha1::{Sha1, Digest};
    let mut hasher = Sha1::new();
    hasher.update(format!("{}{}{}", user, ukey, stime));
    let result = hasher.finalize();
    format!("{:x}", result)
}

fn get_stime() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}

fn feie_request(params: &[(String, String)]) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP 客戶端初始化失敗: {}", e))?;
    let form = params.iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<Vec<_>>();

    let resp = client.post(FEIE_API_URL)
        .form(&form)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .map_err(|e| format!("飛鵝 API 請求失敗: {}", e))?;

    let text = resp.text()
        .map_err(|e| format!("飛鵝 API 響應讀取失敗: {}", e))?;

    Ok(text)
}

#[allow(dead_code)]
pub fn feie_add_printer(user: &str, ukey: &str, sn: &str, key: &str) -> Result<String, String> {
    let stime = get_stime();
    let sig = compute_feie_signature(user, ukey, &stime);

    let params = vec![
        ("user".to_string(), user.to_string()),
        ("stime".to_string(), stime),
        ("sig".to_string(), sig),
        ("apiname".to_string(), "Open_printerAddlist".to_string()),
        ("printerContent".to_string(), format!("{}#{}", sn, key)),
    ];

    feie_request(&params)
}

pub fn feie_print(user: &str, ukey: &str, sn: &str, content: &str) -> Result<String, String> {
    let stime = get_stime();
    let sig = compute_feie_signature(user, ukey, &stime);

    let params = vec![
        ("user".to_string(), user.to_string()),
        ("stime".to_string(), stime),
        ("sig".to_string(), sig),
        ("apiname".to_string(), "Open_printMsg".to_string()),
        ("sn".to_string(), sn.to_string()),
        ("content".to_string(), content.to_string()),
    ];

    feie_request(&params)
}

#[allow(dead_code)]
pub fn feie_query_status(user: &str, ukey: &str, sn: &str) -> Result<String, String> {
    let stime = get_stime();
    let sig = compute_feie_signature(user, ukey, &stime);

    let params = vec![
        ("user".to_string(), user.to_string()),
        ("stime".to_string(), stime),
        ("sig".to_string(), sig),
        ("apiname".to_string(), "Open_queryPrinterStatus".to_string()),
        ("sn".to_string(), sn.to_string()),
    ];

    feie_request(&params)
}

// ==================== ESC/POS 指令構建器 ====================

pub struct EscPosBuilder {
    pub(crate) buffer: Vec<u8>,
}

#[allow(dead_code)]
impl EscPosBuilder {
    pub fn new() -> Self {
        let mut buffer = Vec::new();
        buffer.push(0x1B); buffer.push(0x40); // ESC @ - 初始化
        Self { buffer }
    }

    pub fn text(&mut self, text: &str) -> &mut Self {
        self.buffer.extend_from_slice(text.as_bytes());
        self
    }

    pub fn text_ln(&mut self, text: &str) -> &mut Self {
        self.buffer.extend_from_slice(text.as_bytes());
        self.buffer.push(0x0A); // LF
        self
    }

    pub fn align_left(&mut self) -> &mut Self {
        self.buffer.push(0x1B); self.buffer.push(0x61); self.buffer.push(0x00);
        self
    }

    pub fn align_center(&mut self) -> &mut Self {
        self.buffer.push(0x1B); self.buffer.push(0x61); self.buffer.push(0x01);
        self
    }

    pub fn align_right(&mut self) -> &mut Self {
        self.buffer.push(0x1B); self.buffer.push(0x61); self.buffer.push(0x02);
        self
    }

    pub fn bold_on(&mut self) -> &mut Self {
        self.buffer.push(0x1B); self.buffer.push(0x45); self.buffer.push(0x01);
        self
    }

    pub fn bold_off(&mut self) -> &mut Self {
        self.buffer.push(0x1B); self.buffer.push(0x45); self.buffer.push(0x00);
        self
    }

    pub fn underline_on(&mut self) -> &mut Self {
        self.buffer.push(0x1B); self.buffer.push(0x2D); self.buffer.push(0x01);
        self
    }

    pub fn underline_off(&mut self) -> &mut Self {
        self.buffer.push(0x1B); self.buffer.push(0x2D); self.buffer.push(0x00);
        self
    }

    pub fn underline_double(&mut self) -> &mut Self {
        self.buffer.push(0x1B); self.buffer.push(0x2D); self.buffer.push(0x02);
        self
    }

    pub fn inverse_on(&mut self) -> &mut Self {
        self.buffer.push(0x1D); self.buffer.push(0x42); self.buffer.push(0x01);
        self
    }

    pub fn inverse_off(&mut self) -> &mut Self {
        self.buffer.push(0x1D); self.buffer.push(0x42); self.buffer.push(0x00);
        self
    }

    pub fn font_a(&mut self) -> &mut Self {
        self.buffer.push(0x1B); self.buffer.push(0x4D); self.buffer.push(0x00);
        self
    }

    pub fn font_b(&mut self) -> &mut Self {
        self.buffer.push(0x1B); self.buffer.push(0x4D); self.buffer.push(0x01);
        self
    }

    pub fn double_height(&mut self) -> &mut Self {
        self.buffer.push(0x1B); self.buffer.push(0x21); self.buffer.push(0x10);
        self
    }

    pub fn double_width(&mut self) -> &mut Self {
        self.buffer.push(0x1B); self.buffer.push(0x21); self.buffer.push(0x20);
        self
    }

    pub fn double_size(&mut self) -> &mut Self {
        self.buffer.push(0x1B); self.buffer.push(0x21); self.buffer.push(0x30);
        self
    }

    pub fn normal_size(&mut self) -> &mut Self {
        self.buffer.push(0x1B); self.buffer.push(0x21); self.buffer.push(0x00);
        self
    }

    #[allow(dead_code)]
    pub fn qr_code(&mut self, content: &str, size: u8) -> &mut Self {
        let data = content.as_bytes();
        let len = data.len() + 3;

        self.buffer.push(0x1D); self.buffer.push(0x28); self.buffer.push(0x6B);
        self.buffer.push(0x04); self.buffer.push(0x00);
        self.buffer.push(0x31); self.buffer.push(0x41);
        self.buffer.push(size); self.buffer.push(0x00);

        self.buffer.push(0x1D); self.buffer.push(0x28); self.buffer.push(0x6B);
        self.buffer.push(0x03); self.buffer.push(0x00);
        self.buffer.push(0x31); self.buffer.push(0x43);
        self.buffer.push(size);

        self.buffer.push(0x1D); self.buffer.push(0x28); self.buffer.push(0x6B);
        self.buffer.push(0x03); self.buffer.push(0x00);
        self.buffer.push(0x31); self.buffer.push(0x45);
        self.buffer.push(0x30);

        self.buffer.push(0x1D); self.buffer.push(0x28); self.buffer.push(0x6B);
        self.buffer.push(len as u8); self.buffer.push(0x00);
        self.buffer.push(0x31); self.buffer.push(0x50); self.buffer.push(0x30);
        self.buffer.extend_from_slice(data);

        self
    }

    pub fn feed_lines(&mut self, n: u8) -> &mut Self {
        self.buffer.push(0x1B); self.buffer.push(0x64); self.buffer.push(n);
        self
    }

    pub fn cut_paper(&mut self) -> &mut Self {
        self.buffer.push(0x1D); self.buffer.push(0x56); self.buffer.push(0x01);
        self
    }

    pub fn partial_cut(&mut self) -> &mut Self {
        self.buffer.push(0x1D); self.buffer.push(0x56); self.buffer.push(0x00);
        self
    }

    pub fn separator(&mut self, width: usize) -> &mut Self {
        let sep = "─".repeat(width);
        self.text_ln(&sep);
        self
    }

    pub fn dashed_separator(&mut self, width: usize) -> &mut Self {
        let sep = "-".repeat(width);
        self.text_ln(&sep);
        self
    }

    pub fn build(self) -> Vec<u8> {
        self.buffer
    }
}

// ==================== TSPL 指令構建器 ====================

pub struct TsplBuilder {
    commands: Vec<String>,
}

impl TsplBuilder {
    pub fn new(label_width_mm: f64, label_height_mm: f64) -> Self {
        Self {
            commands: vec![
                format!("SIZE {:.1} mm, {:.1} mm", label_width_mm, label_height_mm),
                "GAP 2 mm, 0 mm".to_string(),
                "DIRECTION 1".to_string(),
                "CLS".to_string(),
            ],
        }
    }

    #[allow(dead_code)]
    pub fn with_gap(label_width_mm: f64, label_height_mm: f64, gap_mm: f64) -> Self {
        Self {
            commands: vec![
                format!("SIZE {:.1} mm, {:.1} mm", label_width_mm, label_height_mm),
                format!("GAP {:.1} mm, 0 mm", gap_mm),
                "DIRECTION 1".to_string(),
                "CLS".to_string(),
            ],
        }
    }

    fn tspl_escape(content: &str) -> String {
        content.replace('"', "\\\"").replace(['\r', '\n'], "")
    }

    pub fn text(&mut self, x: i32, y: i32, font: &str, size: (i32, i32), content: &str) -> &mut Self {
        self.commands.push(format!(
            "TEXT {}, {}, \"{}\", 0, {}, {}, \"{}\"",
            x, y, font, size.0, size.1, Self::tspl_escape(content)
        ));
        self
    }

    #[allow(dead_code)]
    pub fn text_with_rotation(&mut self, x: i32, y: i32, font: &str, size: (i32, i32), rotation: i32, content: &str) -> &mut Self {
        self.commands.push(format!(
            "TEXT {}, {}, \"{}\", {}, {}, {}, \"{}\"",
            x, y, font, rotation, size.0, size.1, Self::tspl_escape(content)
        ));
        self
    }

    pub fn barcode(&mut self, x: i32, y: i32, code_type: &str, height: i32, content: &str) -> &mut Self {
        self.commands.push(format!(
            "BARCODE {}, {}, \"{}\", {}, 1, 0, 2, 2, \"{}\"",
            x, y, code_type, height, Self::tspl_escape(content)
        ));
        self
    }

    #[allow(dead_code)]
    pub fn qr_code(&mut self, x: i32, y: i32, level: &str, cell_size: i32, content: &str) -> &mut Self {
        self.commands.push(format!(
            "QRCODE {}, {}, {}, {}, 0, \"{}\"",
            x, y, level, cell_size, Self::tspl_escape(content)
        ));
        self
    }

    #[allow(dead_code)]
    pub fn box_draw(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, thickness: i32) -> &mut Self {
        self.commands.push(format!(
            "BOX {}, {}, {}, {}, {}",
            x1, y1, x2, y2, thickness
        ));
        self
    }

    pub fn print_label(&mut self, copies: i32) -> &mut Self {
        self.commands.push(format!("PRINT {}", copies));
        self
    }

    pub fn build(&self) -> String {
        self.commands.join("\r\n") + "\r\n"
    }
}

// ==================== 局域網 TCP 打印 ====================

pub fn lan_print(ip: &str, port: i32, data: &[u8]) -> Result<(), String> {
    let addr = format!("{}:{}", ip, port);
    let mut stream = TcpStream::connect_timeout(
        &addr.parse().map_err(|e| format!("無效地址 {}: {}", addr, e))?,
        Duration::from_secs(5),
    ).map_err(|e| format!("連接打印機 {} 失敗: {}", addr, e))?;

    stream.set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| format!("設置超時失敗: {}", e))?;

    stream.write_all(data)
        .map_err(|e| format!("發送數據失敗: {}", e))?;

    Ok(())
}

pub fn lan_print_escpos(ip: &str, port: i32, builder: EscPosBuilder) -> Result<(), String> {
    let data = builder.build();
    lan_print(ip, port, &data)
}

#[allow(dead_code)]
pub fn lan_print_tspl(ip: &str, port: i32, builder: &TsplBuilder) -> Result<(), String> {
    let data = builder.build().into_bytes();
    lan_print(ip, port, &data)
}

// ==================== 局域网扫描 ====================

pub fn scan_lan_printers(subnet: &str, timeout_ms: u64) -> Vec<LanPrinter> {
    let mut found = Vec::new();

    for i in 1..=254 {
        let ip = format!("{}.{}", subnet, i);
        let addr = format!("{}:9100", ip);

        if let Ok(addr) = addr.parse::<std::net::SocketAddr>() {
            if TcpStream::connect_timeout(&addr, Duration::from_millis(timeout_ms)).is_ok() {
                found.push(LanPrinter {
                    ip: ip.clone(),
                    port: 9100,
                    sn: None,
                });
            }
        }
    }

    found
}

// ==================== 局域网状态检查 ====================

#[allow(dead_code)]
pub fn check_lan_printer_status(ip: &str, port: i32) -> Result<String, String> {
    let addr = format!("{}:{}", ip, port);
    let mut stream = TcpStream::connect_timeout(
        &addr.parse().map_err(|_| format!("无效地址"))?,
        Duration::from_secs(3),
    ).map_err(|_| "打印机离线")?;
    
    stream.set_read_timeout(Some(Duration::from_secs(2))).map_err(|_| "设置超时失败")?;
    
    // 发送状态查询指令
    stream.write_all(&[0x1B, 0x45, 0x74]).map_err(|_| "发送失败")?;
    
    let mut response = vec![0u8; 256];
    match stream.read(&mut response) {
        Ok(n) if n > 0 => Ok("在线".to_string()),
        _ => Ok("在线".to_string()),
    }
}

// ==================== 打印模板 ====================

pub fn build_kitchen_ticket_content(
    order_no: &str,
    dine_type: &str,
    items: &[(String, f64, Option<String>)],
    note: Option<&str>,
) -> EscPosBuilder {
    let mut builder = EscPosBuilder::new();

    builder.align_center().bold_on().double_height()
        .text_ln("Cuckoo 廚房單")
        .normal_size().bold_off().align_left()
        .separator(32);

    builder.text_ln(&format!("單號: {}", order_no))
        .text_ln(&format!("類型: {}", dine_type))
        .text_ln(&format!("時間: {}", chrono::Local::now().format("%Y-%m-%d %H:%M")))
        .separator(32);

    builder.bold_on().text_ln("菜品明細").bold_off();

    for (name, qty, item_note) in items {
        if *qty != 1.0 {
            builder.text_ln(&format!("{} x{}", name, *qty as i32));
        } else {
            builder.text_ln(name);
        }
        if let Some(n) = item_note {
            builder.text_ln(&format!("  備註: {}", n));
        }
    }

    builder.separator(32);

    if let Some(n) = note {
        builder.bold_on().text_ln(&format!("訂單備註: {}", n)).bold_off();
    }

    builder.feed_lines(3).cut_paper();

    builder
}

/// Build plain-text kitchen ticket for Feie cloud printing (HTTP text API, no binary ESC/POS).
pub fn build_kitchen_ticket_text(
    order_no: &str,
    dine_type: &str,
    items: &[(String, f64, Option<String>)],
    note: Option<&str>,
) -> String {
    let mut s = String::new();
    s.push_str(&format!("=== Cuckoo 廚房單 ===\n"));
    s.push_str(&format!("單號: {}\n", order_no));
    s.push_str(&format!("類型: {}\n", dine_type));
    s.push_str(&format!("時間: {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M")));
    s.push_str("--------------------------------\n");
    for (name, qty, item_note) in items {
        if *qty != 1.0 {
            s.push_str(&format!("{} x{}\n", name, *qty as i32));
        } else {
            s.push_str(&format!("{}\n", name));
        }
        if let Some(n) = item_note {
            s.push_str(&format!("  備註: {}\n", n));
        }
    }
    s.push_str("--------------------------------\n");
    if let Some(n) = note {
        s.push_str(&format!("訂單備註: {}\n", n));
    }
    s
}

/// items: (name, qty, unit_price)
pub fn build_receipt_content(
    order_no: &str,
    dine_type: &str,
    table_no: Option<&str>,
    items: &[(String, f64, f64)],
    total: f64,
    payment_method: Option<&str>,
    amount_paid: f64,
) -> EscPosBuilder {
    let mut builder = EscPosBuilder::new();
    builder.align_center().bold_on().double_height()
        .text_ln("Cuckoo 收據")
        .normal_size().bold_off().align_left()
        .separator(32);
    builder.text_ln(&format!("單號: {}", order_no));
    builder.text_ln(&format!("類型: {}", dine_type));
    if let Some(t) = table_no {
        builder.text_ln(&format!("桌號: {}", t));
    }
    builder.text_ln(&format!("時間: {}", chrono::Local::now().format("%Y-%m-%d %H:%M")));
    builder.separator(32);
    for (name, qty, unit_price) in items {
        let line_total = qty * unit_price;
        builder.text_ln(&format!("{} x{} = ${:.2}", name, *qty as i32, line_total));
    }
    builder.separator(32);
    builder.bold_on().text_ln(&format!("合計: ${:.2}", total)).bold_off();
    if let Some(m) = payment_method {
        builder.text_ln(&format!("付款: {}", m));
        builder.text_ln(&format!("實收: ${:.2}", amount_paid));
        let change = amount_paid - total;
        if change > 0.001 {
            builder.text_ln(&format!("找零: ${:.2}", change));
        }
    }
    builder.feed_lines(3).cut_paper();
    builder
}

pub fn build_receipt_text(
    order_no: &str,
    dine_type: &str,
    table_no: Option<&str>,
    items: &[(String, f64, f64)],
    total: f64,
    payment_method: Option<&str>,
    amount_paid: f64,
) -> String {
    let mut s = String::new();
    s.push_str("=== Cuckoo 收據 ===\n");
    s.push_str(&format!("單號: {}\n", order_no));
    s.push_str(&format!("類型: {}\n", dine_type));
    if let Some(t) = table_no {
        s.push_str(&format!("桌號: {}\n", t));
    }
    s.push_str(&format!("時間: {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M")));
    s.push_str("--------------------------------\n");
    for (name, qty, unit_price) in items {
        let line_total = qty * unit_price;
        s.push_str(&format!("{} x{} = ${:.2}\n", name, *qty as i32, line_total));
    }
    s.push_str("--------------------------------\n");
    s.push_str(&format!("合計: ${:.2}\n", total));
    if let Some(m) = payment_method {
        s.push_str(&format!("付款: {}\n", m));
        s.push_str(&format!("實收: ${:.2}\n", amount_paid));
        let change = amount_paid - total;
        if change > 0.001 {
            s.push_str(&format!("找零: ${:.2}\n", change));
        }
    }
    s
}

pub fn build_batch_label_content(
    lot_no: &str,
    material_name: &str,
    quantity: f64,
    unit: &str,
    expiry_date: Option<&str>,
    supplier_name: Option<&str>,
) -> TsplBuilder {
    let mut builder = TsplBuilder::new(60.0, 40.0);

    builder.text(20, 20, "Arial", (2, 2), material_name);
    builder.text(20, 60, "Arial", (1, 1), &format!("批次: {}", lot_no));
    builder.text(20, 90, "Arial", (1, 1), &format!("數量: {} {}", quantity, unit));

    if let Some(exp) = expiry_date {
        builder.text(20, 120, "Arial", (1, 1), &format!("到期: {}", exp));
    }

    if let Some(supplier) = supplier_name {
        builder.text(20, 150, "Arial", (1, 1), &format!("供應商: {}", supplier));
    }

    builder.barcode(20, 190, "128", 40, lot_no);
    builder.print_label(1);

    builder
}

// ==================== 打印機狀態解析 ====================

#[allow(dead_code)]
pub fn parse_feie_status(response: &str) -> PrinterStatus {
    let mut status = PrinterStatus {
        printer_id: 0,
        online: false,
        has_paper: true,
        cover_open: false,
        error_msg: None,
    };

    if response.contains("offline") || response.contains("離線") {
        status.online = false;
    } else if response.contains("online") || response.contains("在線") {
        status.online = true;
    }

    if response.contains("no paper") || response.contains("缺紙") {
        status.has_paper = false;
    }

    if response.contains("cover open") || response.contains("開蓋") {
        status.cover_open = true;
    }

    status
}

// ==================== ESC/POS 渲染為 HTML 預覽 ====================

pub struct EscPosRenderer {
    lines: Vec<HtmlLine>,
    current_align: String,
    current_bold: bool,
    current_underline: bool,
    current_double_height: bool,
    current_double_width: bool,
    current_inverse: bool,
    current_font_size: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct HtmlLine {
    text: String,
    align: String,
    bold: bool,
    underline: bool,
    double_height: bool,
    double_width: bool,
    inverse: bool,
    font_size: String,
    is_separator: bool,
    is_blank: bool,
}

impl EscPosRenderer {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            current_align: "left".to_string(),
            current_bold: false,
            current_underline: false,
            current_double_height: false,
            current_double_width: false,
            current_inverse: false,
            current_font_size: "normal".to_string(),
        }
    }

    fn flush_line(&mut self, text: String) {
        if text.is_empty() { return; }
        self.lines.push(HtmlLine {
            text,
            align: self.current_align.clone(),
            bold: self.current_bold,
            underline: self.current_underline,
            double_height: self.current_double_height,
            double_width: self.current_double_width,
            inverse: self.current_inverse,
            font_size: self.current_font_size.clone(),
            is_separator: false,
            is_blank: false,
        });
    }

    pub fn render(&self, buffer: &[u8]) -> String {
        let mut renderer = Self::new();
        renderer.parse_buffer(buffer);
        renderer.to_html()
    }

    fn parse_buffer(&mut self, buffer: &[u8]) {
        let mut i = 0;
        let mut current_text = String::new();

        while i < buffer.len() {
            let b = buffer[i];

            if b == 0x1B && i + 1 < buffer.len() {
                if !current_text.is_empty() {
                    self.flush_line(current_text.clone());
                    current_text.clear();
                }
                let cmd = buffer[i + 1];
                match cmd {
                    0x40 => { // ESC @ - init
                        self.reset_state();
                    }
                    0x61 if i + 2 < buffer.len() => { // ESC a n - align
                        match buffer[i + 2] {
                            0x00 => self.current_align = "left".to_string(),
                            0x01 => self.current_align = "center".to_string(),
                            0x02 => self.current_align = "right".to_string(),
                            _ => {}
                        }
                        i += 1;
                    }
                    0x45 if i + 2 < buffer.len() => { // ESC E n - bold
                        self.current_bold = buffer[i + 2] != 0;
                        i += 1;
                    }
                    0x2D if i + 2 < buffer.len() => { // ESC - n - underline
                        self.current_underline = buffer[i + 2] != 0;
                        i += 1;
                    }
                    0x21 if i + 2 < buffer.len() => { // ESC ! n - char size
                        let mode = buffer[i + 2];
                        self.current_double_height = (mode & 0x10) != 0;
                        self.current_double_width = (mode & 0x20) != 0;
                        if self.current_double_height && self.current_double_width {
                            self.current_font_size = "double".to_string();
                        } else if self.current_double_height || self.current_double_width {
                            self.current_font_size = "large".to_string();
                        } else {
                            self.current_font_size = "normal".to_string();
                        }
                        i += 1;
                    }
                    _ => {}
                }
                i += 2;
                continue;
            }

            if b == 0x1D && i + 1 < buffer.len() {
                if !current_text.is_empty() {
                    self.flush_line(current_text.clone());
                    current_text.clear();
                }
                let cmd = buffer[i + 1];
                match cmd {
                    0x42 if i + 2 < buffer.len() => { // GS B n - inverse
                        self.current_inverse = buffer[i + 2] != 0;
                        i += 1;
                    }
                    0x28 if i + 3 < buffer.len() && buffer[i + 2] == 0x6B => { // GS ( k - QR
                        // Skip QR code data
                        let pl = buffer[i + 3] as usize;
                        let ph = if i + 4 < buffer.len() { buffer[i + 4] as usize } else { 0 };
                        let data_len = pl + (ph << 8);
                        i += 5 + data_len;
                        continue;
                    }
                    0x56 => { // GS V - cut paper
                        if !current_text.is_empty() {
                            self.flush_line(current_text.clone());
                            current_text.clear();
                        }
                        i += 2;
                        continue;
                    }
                    _ => {}
                }
                i += 2;
                continue;
            }

            if b == 0x0A { // LF - line feed
                if !current_text.is_empty() {
                    self.flush_line(current_text.clone());
                    current_text.clear();
                }
                i += 1;
                continue;
            }

            if b == 0x1B || b == 0x1D {
                // Control character, skip
                i += 1;
                continue;
            }

            // Regular byte - try as UTF-8
            current_text.push(b as char);
            i += 1;
        }

        if !current_text.is_empty() {
            self.flush_line(current_text);
        }
    }

    fn reset_state(&mut self) {
        self.current_align = "left".to_string();
        self.current_bold = false;
        self.current_underline = false;
        self.current_double_height = false;
        self.current_double_width = false;
        self.current_inverse = false;
        self.current_font_size = "normal".to_string();
    }

    fn to_html(&self) -> String {
        let mut html = String::new();
        html.push_str("<div class=\"receipt-preview\" style=\"");
        html.push_str("font-family: 'Courier New', monospace; ");
        html.push_str("background: #1a1a2e; ");
        html.push_str("color: #e0e0e0; ");
        html.push_str("padding: 20px; ");
        html.push_str("max-width: 320px; ");
        html.push_str("margin: 0 auto; ");
        html.push_str("border-radius: 8px; ");
        html.push_str("box-shadow: 0 4px 12px rgba(0,0,0,0.3); ");
        html.push_str("\">\n");

        for line in &self.lines {
            let align = match line.align.as_str() {
                "center" => "text-align: center;",
                "right" => "text-align: right;",
                _ => "text-align: left;",
            };

            let mut style = align.to_string();

            if line.bold {
                style.push_str("font-weight: bold;");
            }
            if line.underline {
                style.push_str("text-decoration: underline;");
            }
            if line.inverse {
                style.push_str("background: #e0e0e0; color: #1a1a2e; padding: 0 4px;");
            }

            let font_size = match line.font_size.as_str() {
                "double" => "font-size: 24px; line-height: 1.4;",
                "large" => "font-size: 18px; line-height: 1.4;",
                _ => "font-size: 14px; line-height: 1.3;",
            };
            style.push_str(font_size);

            let escaped = line.text
                .replace("&", "&amp;")
                .replace("<", "&lt;")
                .replace(">", "&gt;");

            html.push_str(&format!("<div style=\"{}\">{}</div>\n", style, escaped));
        }

        html.push_str("</div>");
        html
    }
}

// ==================== 調試模式：打印到文件 ====================

pub fn save_escpos_to_file(builder: EscPosBuilder, filename: Option<&str>) -> Result<DebugPrintResult, String> {
    let data = builder.build();
    let byte_count = data.len();

    let output_dir = debug_output_dir()?;
    let file_name = sanitize_filename(filename, "debug_print", "bin");
    let path = output_dir.join(file_name);

    std::fs::write(&path, &data)
        .map_err(|e| format!("寫入文件失敗: {}", e))?;

    let renderer = EscPosRenderer::new();
    let html_preview = renderer.render(&data);

    Ok(DebugPrintResult {
        file_path: path.canonicalize().unwrap_or(path).to_string_lossy().to_string(),
        html_preview,
        byte_count,
    })
}

pub fn save_kitchen_ticket_to_file(
    order_no: &str,
    dine_type: &str,
    items: &[(String, f64, Option<String>)],
    note: Option<&str>,
    filename: Option<&str>,
) -> Result<DebugPrintResult, String> {
    let builder = build_kitchen_ticket_content(order_no, dine_type, items, note);
    save_escpos_to_file(builder, filename)
}

pub fn save_batch_label_to_file(
    lot_no: &str,
    material_name: &str,
    quantity: f64,
    unit: &str,
    expiry_date: Option<&str>,
    supplier_name: Option<&str>,
    filename: Option<&str>,
) -> Result<DebugPrintResult, String> {
    let builder = build_batch_label_content(lot_no, material_name, quantity, unit, expiry_date, supplier_name);
    let data = builder.build().into_bytes();
    let byte_count = data.len();

    let output_dir = debug_output_dir()?;
    let file_name = sanitize_filename(filename, "debug_label", "txt");
    let path = output_dir.join(file_name);

    std::fs::write(&path, &data)
        .map_err(|e| format!("寫入文件失敗: {}", e))?;

    let html = format!(
        "<div class=\"label-preview\" style=\"\
            font-family: 'Arial', sans-serif; \
            background: #f5f5f5; \
            color: #333; \
            padding: 20px; \
            width: 240px; \
            margin: 0 auto; \
            border: 2px solid #333; \
            border-radius: 4px; \
        \">\
            <div style=\"font-size: 18px; font-weight: bold; margin-bottom: 10px;\">{}</div>\
            <div style=\"font-size: 12px; margin: 4px 0;\">批次: {}</div>\
            <div style=\"font-size: 12px; margin: 4px 0;\">數量: {} {}</div>\
            {}{}\
            <div style=\"margin-top: 10px; text-align: center; font-family: monospace; font-size: 10px; letter-spacing: 2px;\">||||| {} |||||</div>\
        </div>",
        escape_html(material_name),
        escape_html(lot_no),
        quantity,
        escape_html(unit),
        expiry_date
            .map(|d| format!("<div style=\"font-size: 12px; margin: 4px 0;\">到期: {}</div>", escape_html(d)))
            .unwrap_or_default(),
        supplier_name
            .map(|s| format!("<div style=\"font-size: 12px; margin: 4px 0;\">供應商: {}</div>", escape_html(s)))
            .unwrap_or_default(),
        escape_html(lot_no),
    );

    Ok(DebugPrintResult {
        file_path: path.canonicalize().unwrap_or(path).to_string_lossy().to_string(),
        html_preview: html,
        byte_count,
    })
}
