# Cuckoo 待開發與修復清單 (Backlog & Fix List)

> **最後更新日期**: 2026-05-30  
> **當前審計版本**: v2.3.0（代碼完成，待 GitHub Release）

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
- [x] **配方刪除防呆補全**: 在 `onDeleteRecipe` 前調用 `get_recipe_usage_count`。若有引用，攔截並提示具體引用數量 ✅ 2026-05-23
- [x] **遙測安全強化**: `report_telemetry` 已加 Webhook URL 白名單校驗，前端遙測 metadata 也已脫敏，避免任意 SSRF 與敏感字段外洩 ✅ 2026-05-23
- [x] **配方計算防死循環**: 已評估 `calculate_recipe_cost` 當前為非遞迴展開，沒有 stack overflow 路徑；後續若改為遞迴展開再補深度上限 ✅ 2026-05-23
- [x] **打印預覽注入面收斂**: `print-page.tsx`、`print-templates-page.tsx` 都已先 sanitize 再渲染，Rust 端打印 HTML 拼接也已做 escaping ✅ 2026-05-23
- [x] **調試打印檔案寫入邊界**: `filename` 已限制為安全檔名，且只會落地到受控 debug 目錄並回傳該目錄內路徑 ✅ 2026-05-23
- [x] **遙測出口控制**: Rust `report_telemetry` 已限制為白名單/預設端點，並配合 metadata 脫敏 ✅ 2026-05-23

## 🟡 P1 - 功能修復/體驗 (Medium Priority)
- [x] **刪除語義對齊**: 配方 Dialog 更正為「停用」（邏輯刪除），明細 Dialog 明示「永久刪除」 ✅ 2026-05-30
- [x] **循環引用前端攔截**: 在 `add_recipe_item` 時檢查目標子配方是否已反向引用當前配方 ✅ 2026-05-23
- [x] **庫存搜索功能**: Inventory 頁面已補全搜索過濾器並對齊 Phase 5 需求 ✅ 2026-05-23
- [x] **菜品可售狀態 API 語義對齊**: `toggle_menu_item_availability` → `set_menu_item_availability`；補 web server HTTP 端點；3 個回歸測試 ✅ 2026-05-30
- [x] **錯誤日誌治理**: `appLogger` 已對 context / message / stack 做敏感字段遮罩（單號、電話、URL 等）；後續如需更細的報告匯出再補 UI 選項 ✅ 2026-05-23

## 🔵 P2 - 優化/架構 (Low Priority)
- [x] **Shadcn 元件替換**: `recipes-page.tsx` 已無 `confirm()`（grep 確認） ✅ 2026-05-25
- [x] **單位兼容性校驗**: `add_recipe_item` 後端加 unit_type 一致性驗證；前端已有同等過濾 ✅ 2026-05-30
- [x] **CSP 收斂（已評估 2026-05-30）**:
    - [x] `unsafe-eval`：已完全不存在於 CSP 與代碼庫 ✅
    - [ ] `unsafe-inline` for `style-src`：**暫緩，延至 v3.0.x**。移除條件：(1) `chart.tsx` `<style dangerouslySetInnerHTML>` 改為 CSS modules；(2) 9 處 React `style={{}}` 改為 CSS class；(3) 確認 Recharts 無 inline style 依賴。

## 🔴 P0 - 打印传播力（Kano Must-Do）
- [x] **餐單/訂單打印創意模組**: 模板新增 `art`/`fortune`/`quote`/`image_block` element type；編輯器加快速插入按鈕 ✅ 2026-05-30
- [x] **運勢 / 抽籤模組**: `fortune` element，大吉/中吉/小吉三檔，per_table/per_order/daily 三種種子策略 ✅ 2026-05-30
- [x] **今日靈感卡**: `quote` element，中/英/日多語系語錄庫，按日輪替 ✅ 2026-05-30
- [x] **圖像素材位**: `image_block` element 佔位，未來接 ESC/POS GS v 0 點陣圖 ✅ 2026-05-30

---

## 📊 God File 增長追蹤（審計基線 2026-05-22 → 2026-05-25）

| 檔案 | 2026-05-22 | 2026-05-25 | Δ | 警戒線 |
|---|---|---|---|---|
| `database.rs` | 6,506 | 6,891 | +385 | 🔴 已超 6,000 |
| `commands.rs` | 1,993 | 2,516 | +523 | 🟠 接近 2,500 |
| `App.tsx` | 421 | 547 | +126 | 🟡 增速快 |
| `useAppActions.ts` | 803 | 871 | +68 | 🟡 持續增長 |
| `web_server.rs` | — | 779 | 新增 | 🟢 含 9 個單元測試 |

> **建議**：v2.3.0 週期內啟動 `database.rs` 按業務域拆分（inventory / orders / recipes），避免下版超過 7,000 行。

## 🚀 v2.2.0 待發布動作

- [ ] `git push origin v2.2.0` — 推送本地 tag 觸發 CI 打包 DMG + EXE
- [ ] 確認 CI Release 產物掛在 GitHub Release v2.2.0 下

---
*本清單由 AI 審計代理根據代碼庫現狀自動生成與維護。*
