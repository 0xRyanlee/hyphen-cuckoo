# Issues Analysis — 2026-06-06

用戶反饋 + 崩潰報告 + 截圖分析。分三層：**Bug（需立即修）→ UX重構（大改）→ 功能補齊（待排期）**。

---

## 一、確認 Bug（已修 / 修復中）

### BUG-01 ✅ print-tickets-page 崩潰：`SelectItem value=""`
**症狀**：開啟 `/print-tickets` 即崩潰，錯誤報告 `A <Select.Item /> must have a value prop that is not an empty string`。

**根因**：`print-tickets-page.tsx` 第 344、484 行
```tsx
<SelectItem value="">全部工作站</SelectItem>
```
Radix UI 的 Select 用空字串 `""` 表示「無選中值/顯示 placeholder」，SelectItem 不允許用 `""` 作 value，否則 render 時直接 throw。

**修復**（v3.11）：改成 `value="__none__"`，同步更新 `useState` 初始值和兩個 `parseInt(stationId)` 條件判斷。

---

### BUG-02 ✅ 自助點單頁面瀏覽器開啟白屏

**症狀**：從「餐桌 / QR」預覽，在瀏覽器開啟 `http://IP:9001/#/table/A1` 顯示完全空白頁面。

**根因**：`App.tsx` 第 96 行直接呼叫 Tauri 事件 API：
```typescript
import { listen } from "@tauri-apps/api/event";
// ...
listen("print-result", ...).then(fn => { unlisten = fn; });
// 沒有 .catch()，沒有 isTauri() 保護
```
在瀏覽器（非 Tauri）環境中，`listen` 嘗試存取 `window.__TAURI_INTERNALS__` 拋出 TypeError。雖然是 async 函數拋出（rejected promise），但加上 React 的其他初始化鏈，可能使整個元件樹在首次 render 時就進入錯誤狀態，ErrorBoundary 顯示極簡錯誤 UI 或完全空白。

**修復**（v3.11）：
1. `App.tsx`：加 `if (!isTauri()) return;` 保護，並補上 `.catch(() => {})`
2. 順帶修正 `error-boundary.tsx` 版本硬編碼 `1.2.2` → 改用 vite define 注入的 `__APP_VERSION__`

---

### BUG-03 供應商商品表單：供應商欄位是自由輸入
**症狀**（截圖 Image #4）：新增商品時，「供應商」欄位顯示 `如：东海鲜物贸易` 的 Input 框，可以亂填。當已有供應商但沒有選單，防呆完全失效。

**根因**：`suppliers-page.tsx` 第 305-308 行用 `<Input>` 而非 `<Select>`，資料沒有關聯到 `suppliers` 清單。

**修復**（排入 v3.11）：
- 新增商品 Dialog 的供應商欄位改為 `<Select>`，options 來自 `suppliers` 列表
- 編輯商品 Dialog 同步修改
- `supplier_name` 欄位改由前端填入對應的 `supplier.name`（不需要 schema 改動）

---

## 二、UX 重構項目（大改）

### UX-01 供應商 + 採購渠道合頁（中高優先）
**問題**：左欄有「供應商」和「採購單」兩個分開的入口，相關資料卻要跨頁查。

**方案**：
- 合併為單一頁面「進貨管理」，用 Tab 切換：`[採購單 | 供應商 & 商品]`
- 供應商清單 + 商品清單並排於同一 Tab（現有 `suppliers-page.tsx` 的佈局已接近）
- 左欄移除「供應商」，只保留「採購單」（或重命名為「進貨管理」）

**影響**：`app-sidebar.tsx` 路由調整 + 兩個 page component 合一

---

### UX-02 採購單新增材料 — 多行表格輸入（中優先）
**問題**：目前只能逐筆新增一個材料，操作繁瑣；材料欄位是 Input 自由輸入（沒有防呆）。

**方案**：
- 把「新增採購項目」改成可展開的表格，每行 `[材料 Select | 數量 | 單位 | 單價 | 刪除]`
- 底部有「+ 新增一行」按鈕
- 使用 shadcn DataTable 的可編輯 row 模式，或直接用 `<table>` + dynamic row state
- 批次送出，後端 `add_purchase_order_item` 可以循環呼叫

---

### UX-03 材料狀態管理合入庫存區（低-中優先）
**問題**：「材料狀態」在左欄是獨立入口，跟庫存/盤點邏輯上高度相關，用戶難以發現。

**方案**：
- 「材料狀態」頁面作為「庫存」主頁的一個子 Tab（`[庫存總覽 | 批次 | 交易記錄 | 狀態管理]`）
- 左欄移除「材料狀態」獨立入口
- `stocktakes-page.tsx` 或 inventory page 加 Tab 導航

---

### UX-04 材料狀態代碼 — Combobox 替代自由輸入（低優先）
**問題**：「狀態代碼」欄位是自由文字 Input（placeholder: `raw, processed`），違反防呆原則，使用者不知道輸什麼。

**方案**：
- 改用 shadcn `Combobox`（可搜尋 + 可新增）
- 預設選項：`raw（生料）、semi（半成品）、processed（加工品）、frozen（冷凍）、ready（備料完成）`
- 輸入不在選項中的值時顯示「新增：XXX」可直接建立

---

### UX-05 POS 當前訂單 — 緊湊單行佈局（高優先）
**問題**（截圖 Image #6）：每個商品卡片佔用兩行高度，9 件品項就已撐滿面板，下方結帳按鈕看不到。

**根因**：cart item 的 JSX 使用垂直排列（品名一行 + 數量控制一行），高約 90px/項。

**方案**：
- 重設計 cart item 為單行：`[品名（flex-1）][−][數量][+][小計（w-16）][備注圖示][×]`
- 目標高度：40–44px/行（符合 PAD 觸控最低標準）
- 規格文字縮小為 badge 放在品名後

---

### UX-06 POS 菜單分類 — Tab 形式替代 Button pills（中優先）
**問題**（截圖 Image #6）：分類用按鈕組排列，視覺上像按鈕不像 Tab，且佔用較多垂直空間。

**方案**：
- 使用 shadcn `<Tabs>` 替換 `<button>` 群組
- 或改為水平 scrollable tabs（`flex overflow-x-auto`，每個 tab 44px 高）
- 位置從「菜單分類」區域移到商品列表頂部緊貼

---

### UX-07 POS 結帳按鈕不可見（高優先）
**問題**：用戶說找不到「下單」和「抹零」等按鈕。

**根因分析**：
- 代碼顯示按鈕在 `ScrollArea` 外的 `flex-shrink-0` footer，理論上應可見
- 但 `Card` 只設 `min-h-0` 而無 `h-full`，Card 的高度可能被外容器壓縮，導致 footer 被擠出視口

**方案**：確認 `Card` 得到 `h-full` 或父容器 flex 正確分配高度；如有必要改為 `position: sticky bottom-0`。

---

### UX-08 QR Code 海報多風格（低優先，功能性需求）
**問題**：用戶想要 QR 海報有多種樣式，可以自訂配色、文字標題、導語。

**方案**（新功能）：
在 `tables-page.tsx` QR 預覽 Dialog 中增加：
- 風格選擇：`簡約（minimal）/ 現代（modern）/ 經典（classic）`
- 主色調 color picker（影響 QR 顏色 + border）
- 可編輯欄位：標題文字、桌號顯示、副標題/導語（如「掃碼點餐，快速上桌」）
- 底部生成 PNG 下載按鈕（現有的 `html-to-image` 已支持）

---

### UX-09 票據類型 vs 打印中心 — 概念澄清與 UI 整合（中優先）

**現狀說明**：
| 功能模組 | 職責 |
|---|---|
| **打印中心** | 管理打印機設備（飛鵝 / LAN IP）+ 打印任務佇列 + 自動打印開關 |
| **票據類型** | 管理打印模板的版型（顯示哪些欄位、字型大小、紙寬）— 類似「打印格式設定」 |

兩者是**設備 vs 格式**的關係，不重疊，但命名容易混淆。

**建議**：
- 「票據類型」改名為「打印格式」（或合入「打印中心」作為 Tab：`[設備 | 格式 | 任務記錄]`）
- 左欄合為一個入口「打印中心」含子 Tab

---

### UX-10 庫存盤點入口不明確（中優先）
**問題**（截圖 Image #3）：庫存盤點頁面能看到盤點單列表，但要開始「填入實際數量」的操作入口不直覺（需要知道要點 👁 圖示）。

**方案**：
- 草稿狀態盤點單顯示「開始填寫 →」按鈕而非僅有 👁 icon
- 盤點明細展開後每行直接顯示 Input 讓用戶填入，不需要二次點擊

---

## 三、所有未解決待開發項清單

### A. 舊期待辦（來自 ux-audit-print-marketing.md + kano-backlog）

| 代號 | 描述 | 來源 | 優先 |
|---|---|---|---|
| D3 | 退款上限校驗缺失（可超額退款） | UX Audit | P0 Bug |
| D4 | PurchaseOrder handlers silent catch | UX Audit | P1 |
| D2 | 商品圖片缺佔位符（破圖） | UX Audit | P2 |
| B2 | 桌況圖 floor plan | Kano Backlog | P2 |
| B3 | 材料低庫存提醒 | Kano Backlog | P2 |
| B5 | 套餐勸敗上菜圖 combo upsell | Kano Backlog | P3 |
| B8 | 菜單促銷標籤 | Kano Backlog | P3 |
| M3 | 大屏展示頁 / 響應式 | UX Audit | P3 |

### B. 本輪新發現待辦

| 代號 | 描述 | 優先 |
|---|---|---|
| BUG-03 | 供應商商品表單：供應商欄位改 Select | P1 |
| UX-01 | 供應商 + 採購單合頁 | P1 |
| UX-02 | 採購單多行材料表格輸入 | P1 |
| UX-05 | POS cart item 緊湊單行佈局 | P1 |
| UX-07 | POS 結帳按鈕不可見排查 | P1 |
| UX-06 | POS 分類改 Tab 形式 | P2 |
| UX-03 | 材料狀態合入庫存 Tab | P2 |
| UX-09 | 票據類型改名/合入打印中心 | P2 |
| UX-10 | 盤點單草稿明確填寫入口 | P2 |
| UX-04 | 材料狀態代碼改 Combobox | P3 |
| UX-08 | QR 海報多風格 / 自訂文字 | P3 |

### C. 本輪已修復（v3.10 / v3.11）

| 代號 | 描述 |
|---|---|
| BUG-01 ✅ | print-tickets SelectItem empty value crash |
| BUG-02 ✅ | 自助點單白屏（App.tsx listen 無 isTauri 保護）|
| AUDIT ✅ | PrintTemplatesPage 隱藏 marketing_popup |
| AUDIT ✅ | Auto-print settings 從 localStorage 改為 file 持久化 |
| B4 ✅ | 套餐/組合菜品（Kano Backlog）|
| BAK/WIZ/C-IMG ✅ | 資料庫備份匯出 / 首次引導 / Campaign 封面圖 |

---

## 四、下一輪開發建議優先順序

**v3.11（本批 hotfix + 高優先 UX）：**
1. BUG-01/02/03 修復（已完成 01/02，BUG-03 採購渠道 Select 待做）
2. UX-05 POS cart 緊湊單行佈局
3. UX-07 POS 結帳按鈕可見性確認
4. D3 退款上限校驗

**v3.12（進貨管理重構）：**
1. UX-01 供應商 + 採購單合頁
2. UX-02 採購單多行材料表格
3. UX-03 材料狀態 Tab 合入

**v3.13（POS + QR 改善）：**
1. UX-06 POS 分類 Tab
2. UX-08 QR 多風格
3. UX-09 打印中心整合
