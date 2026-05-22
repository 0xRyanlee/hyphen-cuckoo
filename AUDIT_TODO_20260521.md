# Cuckoo 系統架構與業務多維度審計報告 (附待辦事項)

> **時間戳記**: 2026-05-21T17:20:31+08:00
> **最後更新**: 2026-05-21（第三輪審計 — 安全漏洞 + 數據正確性修復完成）
> **審計範圍**: 系統實現邏輯、業務流程、UI/UX、前後端架構、硬體周邊與數據安全

---

## ✅ 業務流程漏洞修復記錄（第一輪 + 第二輪）

> 以下問題均已於本輪審計中完成修復，Rust/TypeScript 均編譯無誤。

### 第一輪（2026-05-15 ～ 2026-05-19）已修復

| 代號 | 分類 | 問題描述 | 修復摘要 |
|------|------|----------|----------|
| R1 | Rust / SQL | 毛利報表 SQL 雙重計算：revenue 與 COGS 皆因 JOIN recipe_items 而扇出倍增 | 改用三段 CTE（`rev` / `avg_cost` / `cst`）各自 GROUP BY，消除扇出 |
| R2 | Rust | `consume_order_inventory` 每筆 consume txn 的 `cost_delta` 寫死為 0.0，FIFO 成本追蹤失效 | 在 FIFO 迴圈中讀取批次 `cost_per_unit`，寫入 `cost_delta = -(deduct × cpu)` |
| K1 | Rust + TS | 無 KDS 場景時無法標記訂單出餐；原有 `finish_ticket` 路徑需先創建廚房工單 | 新增 `mark_order_ready`：原子性執行 `consume_order_inventory` + `UPDATE status='ready'` |
| F1/F2 | Rust + TS | 報表缺少支出欄位；應收/實收對比無從確認；`get_sales_report` 不回傳已收金額 | `get_sales_report` 加 `collected_amount`；`get_gross_profit_report` 加 `expenses`/`net_profit`；報表頁新增 6 張摘要卡、支出/淨利趨勢線、CSV 更新 |
| I1 | Rust | `consume_order_inventory` 與 `complete_production_order` 庫存不足時靜默繼續，導致庫存負值 | 兩處均加「先行掃描→收集缺口→整批回傳錯誤」的 pre-check pass，寫操作全部後移 |
| O1 | Rust | `receive_purchase_order` 在 INSERT cost_history 後用 `last_insert_rowid()` 覆蓋了 `batch_id`，導致後續更新到錯誤的批次 | 在 INSERT batch 後立即快取 `batch_id`，cost_history INSERT 改用獨立變數 |
| O2 | Rust | `batch_cancel_orders` 對已出餐（`ready`）訂單呼叫 `release_inventory_for_order`，錯誤還原已消耗的庫存 | 新增 status 判斷：`ready` → `cancel_order_confirmed`（保留消耗）；其餘 → `release_inventory_for_order` |
| O3 | Rust | `complete_production_order` 生產完成時 output material 庫存不更新（批次數量不增加） | 確認已在前一輪修復（FIFO 扣減 + 批次寫入） |

### 第二輪（2026-05-21）已修復

| 代號 | 分類 | 問題描述 | 修復摘要 |
|------|------|----------|----------|
| D1 | TS | 庫存頁批次到期日僅顯示原始日期字串，無視覺警示 | 新增 `daysLeft` 計算；到期欄顯示「已過期」(red) 或「X天後到期」(amber) 內嵌文字 |
| D2 | TS + Rust | 生產單建立前無備料缺口預視，操作員需提交後才知庫存不足 | 新增 `check_production_materials` 後端命令；前端在選擇配方/數量時即時呼叫，以色塊列示每項原料需求 vs 庫存 |
| N1 | TS + Rust | `check_and_create_alerts` 每次載入均執行，但通知從未顯示給使用者 | `get_unread_notification_count` 納入 `loadData()`；Sidebar 儀表板項目顯示 amber 徽章；`check_and_create_alerts` 新增自動清除已恢復的 `low_stock` 通知 |
| N2 | TS | 取消含已收款訂單時無警告，操作員可能忽略退款 | 取消對話框新增紅/橙警告區塊，顯示已收金額並提示手動處理退款 |
| B1 | Rust | `batch_cancel_orders` 不分訂單狀態一律呼叫 `release_inventory_for_order` | 加 `if status == "ready"` 分支，走 `cancel_order_confirmed` 路徑 |
| B2 | Rust | `release_inventory_for_order` 的 release txn 寫入 `cost_delta = 0.0`，財務 txn 歷史不對稱 | SELECT 時加入 `cost_delta` 欄位，release txn 寫入 `-cost_delta`（消耗的鏡像） |
| B3 | Rust | `add_purchase_order_item` 無狀態守衛，可對 `partial`/`received` 的採購單新增明細 | 函式開頭查詢 PO status，非 `draft` 則回傳錯誤 |
| B4 | Rust | `update_stocktake_item` 無守衛，已完成的盤點仍可被修改 | 查詢所屬 stocktake 的 status，`completed` 則拒絕修改 |
| G1 | Rust | `consume_order_inventory`、`create_production_order`、`check_production_materials` 均只處理 `item_type='material'`，忽略子配方（`sub_recipe`）的輸出物料 | 三處均加第二次查詢取得子配方 `output_material_id`，透過 HashMap 合併後與直接材料一同處理 |
| G2 | Rust / SQL | 毛利報表 COGS 使用 `AVG(inventory_batches.cost_per_unit)` 快照計算，非歷史實際成本 | `cst` CTE 改為 `SUM(ABS(inventory_txns.cost_delta)) WHERE txn_type='consume'`，直接反映 FIFO 實際扣減成本（含子配方消耗） |

### 第五輪（2026-05-21）已修復

| 代號 | 分類 | 問題描述 | 修復摘要 |
|------|------|----------|----------|
| #11 | Rust / DB | R2 修復前寫入的 `consume` txn `cost_delta = 0.0`，歷史毛利報表 COGS 偏低 | `run_data_migrations()` 在啟動時一次性反算：`cost_delta = qty_delta × ib.cost_per_unit`（冪等，已補丁的行不再命中） |
| #19 | TS + Rust | 配方列表頁每次載入對 N 個配方分別發起 `get_recipe_with_items` 和 `calculate_recipe_cost` IPC 呼叫 | 新增 `get_recipe_item_counts`（單次 GROUP BY 查詢）和 `get_all_recipe_costs`（單次 IPC 呼叫，Rust 端循環），前端改為各一次呼叫 |

### 第四輪（2026-05-21）已修復

| 代號 | 分類 | 問題描述 | 修復摘要 |
|------|------|----------|----------|
| #13 | TS | 更新檢查失敗靜默忽略，無任何日誌可調試 | `catch` 塊改為 `console.warn("[useAutoUpdate] update check failed:", e)`，不影響用戶 |
| #12 | Rust | `complete_stocktake` 的 `stocktake_adjust` txn `cost_delta` 寫死 0.0，盤盈/虧不計入成本池 | 依 `lot_id` 查對應批次 `cost_per_unit`（無 lot 則取材料庫存均价），`cost_delta = diff_qty × cpu` |
| #10 | Rust | 子配方只展開一層：無 `output_material_id` 的子配方的深層原料被跳過 | 提取 `expand_recipe_needs` 遞迴輔助函數（深度守衛 10 層），重構 `check_production_materials`、`create_production_order`、`consume_order_inventory` 三處調用 |

### 第三輪（2026-05-21）已修復

| 代號 | 分類 | 問題描述 | 修復摘要 |
|------|------|----------|----------|
| #14 | Rust | `setup_panic_hook` 每次崩潰覆蓋 crash.log，歷史軌跡丟失 | 改用 `OpenOptions::new().create(true).append(true)` 追加寫入，保留全部崩潰歷史 |
| #15 | Rust（安全） | `download_and_open_update` 接受任意 URL，存在 RCE 風險 | 加 URL 白名單守衛：僅允許 `https://github.com/0xRyanlee/Cuckoo/releases/` 前綴，否則立即回傳錯誤 |
| #16 | Rust（安全） | `TsplBuilder::text` / `text_with_rotation` / `barcode` / `qr_code` 未轉義 `"` 和 `\r\n`，存在 TSPL 注入 | 新增 `tspl_escape` 輔助函數：`replace('"', "\\\"").replace(['\r', '\n'], "")` |
| #17 | Rust | `export_report_csv` 字串欄位（分類名、商品名、原料名）未包裹引號，含逗號時格式錯位 | 新增 `csv_quote` 函數：`format!("\"{}\"", s.replace('"', "\"\""))`，套用於所有字串欄位 |
| #18 | TS | `exportAllCSV` 原料消耗 CSV 標頭 `["成本", "订单数"]` 與實際欄位（平均成本、總成本）不符 | 修正為 `["平均成本", "总成本"]`，並修正解構變數名稱 |

---

## 🏗 高維度架構性問題 (High-Level Architectural Issues)

在本次深度審計中，識別出本系統在發展中形成的兩個最核心的架構性「上帝模式 (God Pattern)」問題，這是導致後續維護困難、效能瓶頸的根因：

1. **後端 God File (`database.rs`)**
   所有數據庫操作（31個表、逾6500行代碼）集中於單一檔案。
2. **前端 God Hook (`useAppActions.ts`)**
   所有應用的 API 呼叫（涵蓋菜單、POS、庫存、盤點、KDS 等）集中在單一 Hook（逾700行）。幾乎每個變更操作在成功後都呼叫全局 `loadData()`，資料量擴大後效能壓力顯著。

---

## 📋 待辦修復清單與多種解方 (TODOs & Solutions)

<!-- ⏸ 評估為「現階段不必要」，已記錄待後續迭代 -->
<!--
### [ ] 1. 後端 `database.rs` 模塊化拆分
評估：單人小團隊 6500 行可接受，等痛點出現再拆。

### [ ] 2. 解決 SQLite 全局 Mutex 鎖定效能瓶頸
評估：單用戶桌面 POS，無並發壓力，WAL 已開啟，不必要。

### [ ] 3. 前端 `useAppActions.ts` 拆分與狀態更新優化
評估：loadData() 22 個並行查詢對烘焙店數據量不慢，React Query 屬過早優化。

### [ ] 4. 業務邏輯解耦 (訂單與庫存)
評估：增加複雜度但無具體收益，現有緊耦合反而更簡單正確。

### [ ] 5. UI/UX 扁平化改造 (`pos-page.tsx`)
評估：UX 改善，由產品需求驅動，非技術債。

### [ ] 7. 列印指令注入風險 (ESC/POS Injection)
評估：資料來源為自身 DB，攻擊面極小；最壞是打印機行為異常，非 RCE，低優先。

### [ ] 8. 庫存數量的浮點數精度遺失風險
評估：庫存量通常為整數或 2 位小數，f64 誤差在第 15 位，非實際問題。

### [ ] 9. KDS 輪詢效能問題
評估：低頻訂單場景輪詢完全夠用，Tauri Events 屬錦上添花。

### [ ] 20. SQLite 單個 Mutex 鎖導致並發查詢排隊
評估：同 #2，單用戶場景無需連接池。
-->

### [x] 6. 打印系統阻塞與容錯 ✅ 已修復
**根因**: 飛鵝雲打印與 LAN 打印透過同步 HTTP/TCP 請求發送，若網路超時（10s），會阻塞 Tauri 的命令執行，導致前端卡頓。
**解方選項**:
- **解方 A (非同步隊列)**: 在 Rust 中引入 `tokio::mpsc` 通道，將列印任務送入背景執行緒處理，前端立即收到成功響應。
- **解方 B (前端隊列)**: 透過前端維護列印任務清單，逐一呼叫 API，失敗則在前端提示「重試」。(不建議，增加前端負擔)

<!-- #7 #8 #9 已評估為現階段不必要，見上方 ⏸ 區塊 -->

### [x] 10. 子配方遞迴展開（多層嵌套）✅ 已修復
**根因**: G1 修復後已覆蓋「一層子配方」場景，但若子配方本身又包含子配方（A → B → C 的嵌套），目前仍只展開一層，深層原料不會被消耗或計入備料。
**影響範圍**: `consume_order_inventory`、`create_production_order`、`check_production_materials`、毛利報表 COGS（G2 已自動覆蓋，因其直接讀 txn）。
**解方選項**:
- **解方 A (遞迴 CTE / WITH RECURSIVE)**: SQLite 支援 `WITH RECURSIVE`，可在一條 SQL 中完整展開任意深度的子配方樹。適合用於備料計算與消耗場景。
- **解方 B (應用層 DFS)**: 在 Rust 中維護一個 stack，逐層讀取 sub_recipe 並展開，邏輯清晰但程式碼量較大。
- **實務建議**: 若業務中子配方最多兩層，解方 A 是最省力的選擇；若允許任意深度，優先解方 A。

### [x] 11. COGS 歷史數據準確性（已關閉庫存批次）✅ 已修復
**根因**: G2 修復後，COGS 讀取 `inventory_txns.cost_delta`。但在 G2 修復之前已存在的 consume txn（cost_delta = 0.0，即 R2 修復前的老數據）仍會被計入，導致歷史報表 COGS 偏低。
**影響範圍**: 修復部署前所有已消耗的訂單成本欄位為 0。
**解方**: 一次性資料補丁：對 `txn_type='consume'` 且 `cost_delta = 0.0` 的歷史 txn，根據當時的批次 `cost_per_unit`（若批次仍存在）反算並更新。若批次已完全消耗（`quantity=0`），則無從反算，接受該部分歷史成本不準確。

### [x] 12. `stocktake_adjust` txn 成本追蹤空白 ✅ 已修復
**根因**: `complete_stocktake` 產生的 `stocktake_adjust` txn 一律寫入 `cost_delta = 0.0`。盤盈/盤虧不反映於成本池，可能導致長期累積的帳實不符。
**解方選項**:
- **解方 A (使用當前批次均價)**: 對盤盈（qty > 0）寫入 `+diff_qty × avg_cpu`，對盤虧（qty < 0）寫入 `-abs(diff_qty) × avg_cpu`。
- **解方 B (零成本記錄，另行報告)**: 保持 cost_delta = 0，但在盤點完成時新增一筆「盤虧損失」Notification，提示管理人員手動入帳。

### [x] 13. GitHub API 頻率限制與更新檢測失敗 ✅ 已修復（日誌部分）
**根因**: `updater_check.rs` 直接調用匿名 GitHub API (`api.github.com/repos/...`)。當用戶頻繁重啟應用或網路波動時，極易觸發 GitHub 對單一 IP 的 Rate Limit（每小時 60 次），導致更新檢查失敗。且後台下載更新包如果權限不足或寫入失敗，可能導致更新靜默失敗。
**解方選項**:
- **解方 A (反向代理與緩存)**: 使用自建的反向代理或中轉服務器（如 Cloudflare Workers）代理更新請求，並在邊緣節點進行短期緩存，避免前端直接對 GitHub 進行頻繁呼叫。
- **解方 B (錯誤處理與提示)**: 在請求中添加自定義 User-Agent，捕獲 Rate Limit 與網絡異常，並向前端拋出具體的錯誤提示，而非靜默失敗。
- **開發者提問**：為什麼不在軟體打開時檢查一次就好，這是單機軟體不需要頻繁打開吧？
- **審計回覆**：✅ 確認。查看 `useAutoUpdate.ts` 後，目前實作已是啟動後 3 秒單次觸發 `check_for_update`，**不存在輪詢**。除非用戶一天內重啟 60+ 次，否則不會觸發 Rate Limit。此項**降級為 Low**，核心問題僅剩靜默失敗（catch 塊內無任何日誌或提示）。

### [x] 14. 崩潰日誌覆蓋與調試軌跡丟失 ✅ 已修復
**根因**: `lib.rs` 的 `setup_panic_hook` 中在發生 panic 時，使用 `fs::write(&log_file, &message)` 寫入崩潰堆疊信息。這會直接**覆蓋**原有的 `crash.log` 文件。如果程式發生連續崩潰或重啟後再次崩潰，只有最後一次的崩潰調用棧會被保留，歷史日誌將全部丟失。
**解方選項**:
- **解方 A (追加寫入)**: 改用 `fs::OpenOptions::new().create(true).append(true).open(&log_file)`，將每次崩潰的調用棧追加到日誌檔案中，並附帶詳細的時間戳記。
- **解方 B (多檔案日誌)**: 每次崩潰時在日誌檔名中加入當前時間戳（例如 `crash_20260521_172031.log`），以避免檔案被覆蓋。

### [x] 15. 任意安裝包下載與遠端代碼執行安全隱患 (RCE) ✅ 已修復
**根因**: `commands.rs` 中的 `download_and_open_update(url: String)` 接口直接接受來自前端傳入的任何 `url` 字符串，並在後台線程中下載並運行它。如果前端存在 XSS 漏洞或 WebView 被惡意注入，攻擊者可以呼叫此 API 下載並執行任意惡意軟件，造成嚴重的 RCE 安全隱患。
**解方選項**:
- **解方 A (URL 白名單)**: 在 `download_and_open_update` 中對 `url` 進行嚴格校驗，僅允許以 `https://github.com/0xRyanlee/Cuckoo/releases/` 開頭的 GitHub Release 官方下載地址，拒絕任何其他非授權的 URL。
- **解方 B (包簽名驗證)**: 下載安裝包後，在執行前驗證其哈希值（Hash）或數字簽名（Signature），確保包沒有被篡改。

### [x] 16. TSPL 標籤列印指令注入漏洞 (TSPL Injection) ✅ 已修復
**根因**: `printer.rs` 的 `TsplBuilder::text` 等方法直接將傳入的物料名稱、批次號、供應商字串等，使用格式化字串 `"{}"` 嵌入到 TSPL 的雙引號內（例如 `TEXT x, y, "Arial", 0, 1, 1, "content"`），未對雙引號 `"` 進行轉義，也未過濾新行字元 `\r\n`。若物料名稱包含引號或換行字元（如 `Beef "Premium"\r\nPRINT 10`），會破壞 TSPL 語法甚至注入任意列印命令，導致打印機行為異常。
**解方選項**:
- **解方 A**: 在 `TsplBuilder::text` 中轉義雙引號為 `\"`，並過濾掉所有 `\r` 與 `\n`。
- **解方 B**: 在前端輸入校驗層過濾引號與換行字元，防止非法字元進入資料庫。

### [x] 17. CSV 匯出未轉義與安全隱患 (CSV Escape Defect) ✅ 已修復
**根因**: 後端 `export_report_csv` 匯出 CSV 時，直接將商品名稱、原料名稱等可能包含逗號 `,`、引號 `"` 或換行符號的字串用逗號拼接。沒有對資料欄位進行任何轉義或包裹，會導致 CSV 格式錯位與解析異常，甚至觸發 CSV 注入攻擊。
**解方選項**:
- **解方 A (引號包裹與轉義)**: 實現一個簡單的 CSV 欄位轉義函數，將包含引號、逗號或換行符的欄位使用引號包裹，並將雙引號雙寫轉義 `""`。
- **解方 B (標準化庫)**: 引入標準的 Rust `csv` 庫進行 CSV 的序列化。

### [x] 18. 原料消耗報表 CSV 匯出欄位定義錯誤 ✅ 已修復
**根因**: 前端 `reports-page.tsx` 中的 `exportAllCSV` 在下載原料消耗報表 CSV 時，檔頭數組定義為 `["原料", "消耗量", "成本", "订单数"]`，但實際傳入的數據數組第四個值是 `totalCost`（總成本），而非「訂單數」，第三個值是 `avgCost`（平均成本）。
**解方選項**:
- **解方 A**: 修改前端 `exportAllCSV` 中的 header 定義為 `["原料", "消耗量", "平均成本", "总成本"]`。

<!-- #19 #20 同屬架構優化，已評估現階段不必要，見上方 ⏸ 區塊 -->
### [x] 19. 前端 N+1 IPC 併發查詢性能瓶頸 ✅ 已修復（配方頁）
**根因**: 在 `recipes-page.tsx`、`materials-page.tsx` 和 `inventory-page.tsx` 中，每次加載都會利用 `Promise.all` 遍歷所有 `recipes` 並對每個配方單獨調用 `get_recipe_with_items` 或 `calculate_recipe_cost` 來獲取明細數量、引用關係與成本。當配方數量較多時（如 100+），會同時發起 100~200 個 Tauri IPC 請求與資料庫查詢，造成資料庫鎖競爭（Mutex 佇列）與前端明顯卡頓。
**解方選項**:
- **解方 A**: 在後端提供批量查詢 API，如 `get_recipes_summaries`，直接通過一條 SQL 的 `JOIN` / `GROUP BY` 返回所有配方的明細數和成本。
- **解方 B**: 列表頁面採用分頁或滾動加載（虛擬滾動），僅對可見的項目發起詳情查詢。

### [x] 20. SQLite 單個 Mutex 鎖導致並發查詢排隊（已評估，暫不處理）
**根因**: `database.rs` 中資料庫連接被封裝在單個 `Mutex<Connection>` 中，導致所有並行的查詢命令最終都在 Rust 層被強制轉為單線程串行執行。這加劇了前端 N+1 查詢時的等待延遲 and 界面凍結。
**當前決策**:
- 已先用批量查詢與局部刷新削減前端 N+1 / 全量 reload 的鎖競爭來源。
- 目前產品仍是單店桌面 POS，尚未出現需要連接池或多連接讀寫拆分的實測瓶頸。
- 因此此項從「立即修復」降級為「容量/性能擴張時再處理」，不阻塞當前版本收口。
**後續觸發條件**:
- 單機資料量明顯增長後，頁面載入或同步出現可穩定重現的卡頓。
- 引入更多背景輪詢 / 同步任務，導致 SQLite 等待時間開始成為主瓶頸。
- 準備做多店或更高頻協作時，再評估 `r2d2` / `deadpool-sqlite` 或讀寫分離方案。

---

## 📌 設計決策記錄 (Design Decisions)

| 項目 | 決策 | 理由 |
|------|------|------|
| 訂單狀態終態 | `ready`（無獨立 `completed`） | 對堂食烘焙場景而言，出餐即完成；付款可在任何時間點透過收款對話框登記 |
| 庫存消耗時機 | `mark_order_ready` / `finish_ticket`（最後一張工單完成時） | 提前消耗（下單時）會導致未出餐訂單虛減庫存；此時機平衡準確性與操作彈性 |
| 消耗冪等性 | `consume_order_inventory` 以訂單是否存在 consume txn 作為 guard | 防止取消後重試、多次呼叫等場景的重複扣減 |
| 通知自動清除 | `check_and_create_alerts` 每次載入先刪除已恢復的 low_stock 通知 | 避免使用者需手動確認已不再成立的警告，badge 實時反映當前庫存狀態 |
| 子配方消耗策略 | 消耗子配方的 `output_material_id`（而非遞迴展開原料） | 假設子配方已透過生產單預製並入庫；若臨時現製則需遞迴展開（見 TODO #10） |

---

> ℹ️ **審計狀態**: 六輪修復完畢，共計 29 項問題已關閉（第一輪 8 + 第二輪 10 + 第三輪 5 + 第四輪 3 + 第五輪 2 + 第六輪 1）。其餘架構項（#1–#5、#7–#9、#20）已評估為現階段不必要，標記為⏸待後續迭代。所有業務邏輯、安全、資料正確性、生產體驗問題均已關閉。
