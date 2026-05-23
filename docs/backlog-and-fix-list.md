# Cuckoo 待開發與修復清單 (Backlog & Fix List)

> **最後更新日期**: 2026-05-23  
> **當前審計版本**: v1.2.2 + v2.2.0 web server

---

## 🔴 P0 — v2.2.0 Web Server（來源：audit-v2.2.0-web-server-2026-05-23.md）
- [x] **Content-Length 無上限阻塞** — `read_request` 加 `.min(MAX_BODY_BYTES)` ✅ 2026-05-23
- [x] **無 PIN 角色任取 admin token** — `auth_login` 無 PIN 時回傳 401 ✅ 2026-05-23
- [x] **Session 無上限增長 / DoS** — 加 10,000 條上限 + TTL 24h + 登入時 retain 清理 ✅ 2026-05-23
- [x] **Mutex Poison 崩潰** — 所有 `lock().unwrap()` 改為 `.lock().unwrap_or_else(|e| e.into_inner())` ✅ 2026-05-23
- [x] **Web server 完全單線程** — thread-per-connection + `stream.shutdown()` ✅ 2026-05-23
- [x] **web_server.rs 零測試** — 補 9 個單元測試（read_request、auth_login 4 cases、session cap、json_err、serve_static path traversal）✅ 2026-05-23

## 🟠 P1 — v2.2.0 Web Server
- [x] **管理員 API HTTP 路由缺失** — `dispatch_protected` 補 `record_order_refund`、`refund_order_item`、`batch_cancel_orders`、`get_all_tickets_with_items` ✅ 2026-05-23
- [x] **GET 允許寫入端點** — `dispatch_api` 改為只允許 POST ✅ 2026-05-23
- [x] **PIN 無 salt** — 改用 `argon2id`（`argon2 v0.5`）；舊 SHA-256 hash 向後兼容，下次 PIN 更改時自動升級 ✅ 2026-05-23
- [x] **`removeFromCart` 規格 bug** — 使用 `spec?.spec_code` 建立 key ✅ 2026-05-23
- [x] **`transport.ts` 無 fetch timeout** — 加 `AbortController`，8 秒 timeout ✅ 2026-05-23
- [x] **`items` 陣列無上限** — 加 `MAX_ORDER_ITEMS = 100` 限制 ✅ 2026-05-23
- [x] **WAL mode 確認** — `database.rs:846` 已有 `PRAGMA journal_mode=WAL` ✅ 已存在

## 🟡 P2 — v2.2.0 Web Server
- [x] **自助點單狀態顯示不一致** — `"pending"` 改「等待確認」 ✅ 2026-05-23
- [x] **輪詢無 Page Visibility 控制** — 加 `visibilitychange` listener ✅ 2026-05-23
- [x] **提交失敗訊息不具體** — catch block 顯示 `e.message` ✅ 2026-05-23
- [x] **`create_restaurant_table` 缺 `isActive` 參數** — DB/command/前端三層全部補上 ✅ 2026-05-23

## 🔵 P3 — v2.2.0 Web Server
- [x] **Session 無 TTL** — value 改 `(String, Instant)`，`require_session` 加 24 小時過期 ✅ 2026-05-23
- [x] **`dispatch_protected` 過長** — 拆分為 menu/orders/kds/customers 子 dispatch ✅ 2026-05-23
- [x] **`TcpStream` 未明確 shutdown** — `start_web_server` 子執行緒末加 `stream.shutdown(Both).ok()` ✅ 2026-05-23
- [x] **公開菜單無快取** — in-memory 快取（30s TTL），菜單更新時失效 ✅ 2026-05-23
- [x] **`stop_web_server` 無前端入口** — settings WebServerCard 補 Stop/Restart 按鈕 ✅ 2026-05-23

---

## 🔴 P0 - 嚴重/數據安全 (High Priority)
- [ ] **配方刪除防呆補全**: 在 `onDeleteRecipe` 前調用 `get_recipe_usage_count`。若有引用，應攔截並提示具體引用數量。
- [ ] **遙測安全強化**: 
    - [ ] `report_telemetry` 命令增加 Webhook URL 白名單校驗，防止 SSRF。
    - [ ] 傳輸敏感數據（銷售額）前應進行加密或使用固定的雲端公鑰。
- [ ] **配方計算防死循環**: 在 Rust `calculate_recipe_cost` 中增加遞歸深度計數器（如 max 10），防止循環依賴導致 Stack Overflow。
- [ ] **打印預覽注入面收斂**:
    - [ ] `print-page.tsx` 的 `result.html_preview` 改為先 sanitize 再渲染。
    - [ ] `print-templates-page.tsx` 的 `previewHtml`（預覽 Dialog）改為 sanitize 渲染。
    - [ ] `printer.rs` 中 HTML 拼接字段（材料名/供應商/批次）輸出前做 escaping。
- [ ] **調試打印檔案寫入邊界**:
    - [ ] 禁止 `filename` 含路徑分隔符與 `..`。
    - [ ] 強制落地到受控 debug 目錄，不允許任意相對路徑寫檔。
    - [ ] 回傳路徑時只回傳受控目錄內路徑。
- [ ] **遙測出口控制**:
    - [ ] Rust `report_telemetry` 停止接受任意 `webhook_url`，改固定端點或白名單。
    - [ ] metadata 增加脫敏策略（堆疊、業務字段分級上報）。

## 🟡 P1 - 功能修復/體驗 (Medium Priority)
- [ ] **刪除語義對齊**: 修正 `recipes-page.tsx` UI 提示，明確區分「邏輯刪除（不啟用）」與「物理刪除（清空明細）」。
- [ ] **循環引用前端攔截**: 在 `add_recipe_item` 時檢查目標子配方是否已反向引用當前配方。
- [ ] **庫存搜索功能**: 為 Inventory 頁面補全搜索過濾器（對齊 Phase 5 需求）。
- [ ] **菜品可售狀態 API 語義對齊**:
    - [ ] 單項切換命令從「toggle」改為「顯式設定 is_available」。
    - [ ] 前後端參數命名統一（`is_available` vs `isAvailable`）並補回歸測試。
- [ ] **錯誤日誌治理**:
    - [ ] `appLogger` context 欄位做敏感字段遮罩（單號、電話、URL 等）。
    - [ ] 設定頁「複製報告」增加隱私提示與脫敏選項。

## 🔵 P2 - 優化/架構 (Low Priority)
- [ ] **Shadcn 元件替換**: 將 `recipes-page.tsx` 中的原生 `confirm()` 替換為 `AlertDialog`。
- [ ] **單位兼容性校驗**: 在配方編輯時，限制只能選擇與材料基準單位相同類型的單位（如重量類只能選 kg/g）。
- [ ] **CSP 收斂**:
    - [ ] 評估並移除非必要 `unsafe-eval`。
    - [ ] 逐步收斂 `unsafe-inline`，避免未來注入擴大化。

---
*本清單由 AI 審計代理根據代碼庫現狀自動生成。*
