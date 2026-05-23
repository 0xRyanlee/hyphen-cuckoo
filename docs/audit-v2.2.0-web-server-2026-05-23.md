# Cuckoo v2.2.0 審計報告 — 多端接入與桌位自助

**審計日期**：2026-05-23  
**審計範圍**：web_server.rs、commands.rs（AppState 變更）、lib.rs、transport.ts、self-order-page.tsx、tables-page.tsx、settings-page.tsx  
**審計方式**：12 維度靜態分析

---

## 維度 1：語義對齊

**1-A** 🟠 P1 — **`/api/get_table_orders_today` GET/POST 語意混淆**  
`dispatch_api` 讀 POST body 的 `v["table_no"]`，而路由名稱語意像 GET。實際行為正確，但設計說明缺失，混淆維護者。

**1-B** 🟡 P2 — **`create_restaurant_table` 沒有傳 `isActive`**  
`tables-page.tsx` 的 `handleAdd` 呼叫時丟棄 `_isActive`，新增餐桌無法設定停用狀態。

**1-C** 🟡 P2 — **`stop_web_server` command 沒有前端入口**  
功能存在（command 已暴露），但 settings-page 的 `WebServerCard` 沒有 Stop/Restart 按鈕。

**1-D** 🔵 P3 — **`get_web_server_status` 中 `running: true` 可能不準確**  
AppState 有 handle 就回傳 true，即使 `stop()` 已把 AtomicBool 設為 false。

---

## 維度 2：資安審計

**2-A** 🔴 P0 — **Session HashMap 無上限，可被 DoS 爆記憶體**  
每次成功登入插入一筆記錄，無 TTL、無容量上限。攻擊者持續 POST `/api/auth/login` 數小時後可耗盡記憶體，造成 Tauri 應用崩潰。無 PIN 角色尤其危險（見 2-B）。

**2-B** 🔴 P0 — **角色無 PIN 時任意人可取得 admin session token**  
`auth_login` 當 `expected_hash` 為空時直接 fall-through，任何人 POST `{"role":"owner","pin":""}` 即可獲得完整 owner session。Tauri app 是本地信任環境，此行為可接受；HTTP server 面向局域網，形成嚴重越權。

**2-C** 🟠 P1 — **CORS 對所有端點（含 admin）回傳 `Access-Control-Allow-Origin: *`**  
保護端點應在 ACAO header 上更嚴謹；公開端點寬鬆 CORS 可接受。

**2-D** 🟠 P1 — **PIN 以無 salt SHA-256 雜湊儲存**  
4–6 位數字 PIN 僅 10^4–10^6 種組合，可被 rainbow table 暴力破解。應改用 `argon2` 或加入 per-role random salt。

**2-E** 🔵 P3 — **Bearer token 無過期時間**  
Session token 永不過期（除非重啟或明確 logout）。應加入 `created_at` + 最大存活時間（建議 24 小時）。

---

## 維度 3：數據流審計

**3-A** 🔴 P0 — **Content-Length 無上限，可觸發長時間阻塞**  
`read_request` 中 `content_length` 直接來自 header，無上限檢查。攻擊者設定超大 `Content-Length`，在 `while body.len() < content_length` 迴圈中阻塞整個單線程 server（結合 5-B 效應最大化）。  
修復：加 `.min(1 * 1024 * 1024)` 上限，超限回傳 413。

**3-B** 🟠 P1 — **`table_no` 未做長度/字元限制即存入 DB**  
僅做空字串判斷，不限長度。雖然 rusqlite parameterized query 防 SQL injection，但仍應限制長度（≤ 32 字元）。

**3-C** 🟠 P1 — **`items` 陣列數量沒有上限**  
攻擊者可在單次請求中放入數萬個項目，觸發大量 DB 寫入，阻塞整個 server。建議限制 `items.len() <= 100`。

**3-D** 🔵 P3 — **`note` 欄位長度無限制**  
建議後端限制 note ≤ 200 字元。

---

## 維度 4：狀態機審計

**4-A** 🟠 P1 — **自助點單 `"pending"` 和 `"submitted"` 在 UI 上顯示相同文字**  
`statusLabel` 函數兩者都顯示「備餐中」，客戶無法區分「等待確認」與「已送廚房」。

**4-B** 🟠 P1 — **`get_web_server_status` 的 `running` 值不追蹤 `AtomicBool`**  
`stop()` 只把 AtomicBool 設為 false，不從 AppState 移除 handle，UI 仍顯示「已啟動」。

**4-C** 🔵 P3 — **無 `restart_web_server` command**  
停止後只能重啟整個應用，無法從 UI 重新啟動 web server。

---

## 維度 5：邊界條件審計

**5-A** 🔴 P0 — **16 KB 緩衝限制 + Content-Length 無上限（結合 3-A）**  
首次 read 只讀 16 KB；Headers 超過 16 KB 時 `header_end` 搜尋失敗，靜默關閉連線。Content-Length 無上限 + 5 秒 read timeout 阻塞 server 最多 5 秒（單線程情況下所有後續請求排隊）。

**5-B** 🟠 P1 — **並發請求完全序列化**  
`accept` loop 直接呼叫 `handle_request`（無 spawn 子執行緒），任何慢請求阻塞所有連線。多台 iPad 同時點單場景下延遲隨設備數線性增長。  
修復：每個 connection spawn 一條子執行緒（short term）或 bounded thread pool（long term）。

**5-C** 🟡 P2 — **`table_no` 為空字串時仍可建立「無桌號」訂單**  
`api_create_self_order_direct` 對空字串不拒絕，`db.create_self_order("", &items)` 會成功。

**5-D** 🟡 P2 — **`removeFromCart` 規格 bug：有規格品項無法正確移除**  
`self-order-page.tsx` 第 276 行：key 中規格 code 被寫死為空字串 `${item.id}__${""}` 而非使用傳入的 spec。帶規格品項的「−」按鈕行為異常。

---

## 維度 6：錯誤處理審計

**6-A** 🔴 P0 — **Mutex `lock().unwrap()` Poison 問題**  
`sessions.lock().unwrap()`、`state.web_server.lock().unwrap()` 等多處：若某線程 panic 時持有鎖，Mutex 毒化，後續所有 `unwrap()` 二次 panic，server thread 崩潰且不可恢復。  
修復：改為 `.lock().unwrap_or_else(|e| e.into_inner())`。

**6-B** 🟠 P1 — **`transport.ts` 無 fetch timeout**  
`fetch()` 沒有 `AbortController` timeout，iPad 被系統 suspend 後請求永久掛起。建議加入 8 秒 timeout。

**6-C** 🟡 P2 — **`self-order-page.tsx` 提交失敗只顯示固定字串 alert**  
catch block 丟棄實際錯誤訊息，客戶無法知道具體原因（如「菜品已下架」）。

**6-D** 🟡 P2 — **`serve_static` 找不到 index.html 時靜默回 404，無日誌**  
生產環境 dist 目錄設定問題難以 debug。

**6-E** 🔵 P3 — **`read_request` 的 `Err(_) => {}` 完全吞掉網路錯誤**  
缺乏可見性，生產環境難以排查問題。

---

## 維度 7：效能路徑審計

**7-A** 🟠 P1 — **Web server 完全單線程**（見 5-B 深化）  
多台 iPad + 4 秒輪詢場景：第 N 台設備需等前 N-1 台完成，延遲線性增長。

**7-B** 🟡 P2 — **`SelfOrderPage` 4 秒輪詢在頁面不可見時仍繼續**  
iOS Safari 鎖屏或切 Tab 後輪詢仍執行。建議加 `visibilitychange` listener 暫停。

**7-C** 🔵 P3 — **`get_public_menu` 無快取，每次重讀 DB**  
菜單不常變動，可加 in-memory 快取 + 菜單更新時失效。

---

## 維度 8：複雜度與可維護性

**8-A** 🔵 P3 — **`dispatch_protected` 函數 142 行，完全無分組**  
建議按業務領域拆分：`dispatch_menu`、`dispatch_orders`、`dispatch_kds`、`dispatch_customers`。

**8-B** 🟠 P1 — **Tauri command 與 HTTP 路由沒有共享 schema**  
新增 Tauri command 時必須手動同步 `dispatch_protected`，否則 browser 模式靜默 404，無任何編譯期保障。

**8-C** 🔵 P3 — **`read_request` 職責不單一**  
混合 header 解析、body 讀取、Content-Length 處理，建議拆分便於測試。

---

## 維度 9：副作用審計

**9-A** 🟠 P1 — **`Arc<Database>` Mutex 鎖競爭未受控**  
Tauri 主線程的長事務（如 `complete_stocktake`）持有 DB 鎖時，web server 所有 DB 操作全部阻塞。應確認 DB 初始化時執行 `PRAGMA journal_mode=WAL;`（WAL 模式允許並發讀）。

**9-B** 🔵 P3 — **`WebServerHandle.stop()` 用 loopback TCP 連線踢醒 accept，可能被防火牆攔截**  
更可靠做法是對 `listener` 設置短 timeout 或使用 `set_nonblocking`。

---

## 維度 10：資源生命週期

**10-A** 🟠 P1 — **`TcpStream` 沒有明確 `shutdown()`**  
`write_all` 失敗時（.ok() 吞掉錯誤），fd 可能不能立刻被 OS 回收。建議在 `handle_request` 末尾呼叫 `stream.shutdown(std::net::Shutdown::Both).ok()`。

**10-B** 🔴 P0 — **Session HashMap 無上限增長**（詳見 2-A）  
修復：加入 `max_sessions: 10_000` 上限，超出回傳 429，並實作 TTL 定時清理。

**10-C** 🔵 P3 — **靜態檔案每次請求都讀磁碟，無記憶體快取**  
大型 JS bundle（2–5 MB）在 HDD 上每次讀取延遲明顯。可加 `Arc<Bytes>` 小檔案快取。

---

## 維度 11：API 合約審計

**11-A** 🟠 P1 — **`dispatch_api` 對寫入操作允許 GET 方法**  
第 150 行 `method != "POST" && method != "GET"` 允許 GET 觸發寫入端點，違反 HTTP 語意，GET 可能被代理快取導致重複提交。

**11-B** 🟠 P1 — **多個 Tauri commands 在 `dispatch_protected` 中缺少 HTTP 路由**  
`record_order_refund`、`refund_order_item`、`batch_cancel_orders`、`get_all_tickets_with_items` 等均無對應路由，iPad admin POS 使用這些功能時靜默 404。

**11-C** 🟡 P2 — **自助點單頁呼叫公開 API 時可能帶 admin token**  
`transport.ts` 在 token 存在時無條件附上 Authorization header，增加 token 暴露機會。

**11-D** 🔵 P3 — **`get_menu_item_specs` 接受 `menuItemId: 0` 作為預設值**  
未傳 `menuItemId` 時查詢 id=0，不報錯但容易誤用。

---

## 維度 12：測試覆蓋

**12-A** 🔴 P0 — **`web_server.rs` 完全無測試**  
`read_request` header 解析、`auth_login` PIN 驗證、`dispatch_api` 路由分派、`serve_static` 路徑穿越防護，全部無 `#[cfg(test)]` 保護。

**12-B** 🟠 P1 — **`removeFromCart` 規格 bug（5-D）無前端測試保護**  
`self-order-page.tsx` 無任何 Jest/Vitest 測試，邏輯 bug 可能長期存在。

**12-C** 🟠 P1 — **Session 管理邏輯無測試**  
無 PIN 自動放行、PIN 錯誤回 401、token 驗證失敗，均無測試覆蓋。

**12-D** 🔵 P3 — **`transport.ts` 的環境切換邏輯無測試**  
`isTauri()` 判斷、`getBaseUrl()` 在不同環境的回傳值無測試，環境切換時可能靜默 regression。

---

## 總結表

| 優先級 | 問題數 | 代表性問題 |
|--------|--------|-----------|
| 🔴 P0  | 6 | Session 無上限 DoS（2-A/10-B）、無 PIN 任取 admin token（2-B）、Content-Length 阻塞（3-A）、Mutex poison（6-A）、web_server.rs 無測試（12-A） |
| 🟠 P1  | 11 | 完全單線程（5-B）、管理員 API 缺少 HTTP 路由（11-B）、GET 允許寫入（11-A）、PIN 無 salt（2-D）、DB 鎖競爭（9-A） |
| 🟡 P2  | 8 | removeFromCart 規格 bug（5-D）、狀態顯示不一致（4-A）、輪詢無 visibility 控制（7-B）、提交失敗訊息（6-C） |
| 🔵 P3  | 10 | dispatch_protected 過長（8-A）、session TTL 缺失（2-E）、靜態檔無快取（10-C）、無 restart command（4-C） |

---

## Backlog（按優先級排序，可直接追加到 backlog-and-fix-list.md）

### 🔴 P0
1. **Content-Length 無上限阻塞** → `read_request` 中加 `.min(1_048_576)` 上限，超限回傳 413
2. **無 PIN 角色任取 admin token** → `auth_login` 改為無 PIN 時回傳 401，或加 `allow_no_pin_http_login` 顯式開關（預設 false）
3. **Session 無上限增長** → 入口加 `sessions.len() >= 10_000` 檢查回傳 429，實作 TTL 定時清理
4. **Mutex Poison 崩潰** → 所有 `lock().unwrap()` 改為 `.lock().unwrap_or_else(|e| e.into_inner())`
5. **Content-Length + 單線程 DoS** → 每個 connection spawn 子執行緒（thread-per-connection）
6. **web_server.rs 無測試** → 補 `read_request`、`auth_login`、`serve_static`（路徑穿越）最小測試集

### 🟠 P1
7. **管理員 API 缺少 HTTP 路由** → `dispatch_protected` 補 `record_order_refund`、`refund_order_item`、`batch_cancel_orders`、`get_all_tickets_with_items`
8. **GET 允許寫入端點** → `dispatch_api` 對非 safe 路由強制 POST
9. **PIN 無 salt** → 改用 `argon2` crate 或加入 per-role random salt
10. **`removeFromCart` 規格 bug** → `self-order-page.tsx` 第 276 行改為 `` `${item.id}__${_spec?.spec_code ?? ""}` ``
11. **`transport.ts` 無 fetch timeout** → 加入 `AbortController`，8 秒 timeout
12. **`items` 陣列無上限** → 加 `items_raw.len() > 100` 檢查回傳 400
13. **WAL mode 確認** → DB 初始化時確認執行 `PRAGMA journal_mode=WAL;`

### 🟡 P2
14. **訂單狀態顯示不一致** → `"pending"` 改為「等待確認」，與「備餐中」區分
15. **輪詢無 Page Visibility 控制** → `self-order-page.tsx` 加 `visibilitychange` listener
16. **提交失敗訊息** → catch block 改為 `alert(\`下單失敗：${e.message}\`)`
17. **`create_restaurant_table` 傳 `isActive`** → `tables-page.tsx` `handleAdd` 補傳參數

### 🔵 P3
18. **Session TTL** → value 改為 `(String, Instant)`，`require_session` 加 24 小時過期檢查
19. **`dispatch_protected` 重構** → 拆分為 menu/orders/kds/customers 子 dispatch
20. **`TcpStream` 明確 shutdown** → `handle_request` 末尾加 `stream.shutdown(Shutdown::Both).ok()`
21. **公開菜單快取** → in-memory `Option<Arc<String>>` 快取，菜單更新時失效
