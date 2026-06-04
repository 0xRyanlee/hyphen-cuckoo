# 開發報告 — Cuckoo v3.6.1 修復批次

**日期：** 2026-06-04  
**基礎版本：** v3.6.0  
**TypeScript 檢查：** `npx tsc --noEmit` — 零錯誤

---

## 修改檔案清單

| 檔案 | 類別 | 說明 |
|------|------|------|
| `src/pages/print-tickets-page.tsx` | Bug 修復 | EditDialog 移出 map loop，key=id 強制 remount；刪除確認 Dialog |
| `src/pages/settings-page.tsx` | Bug 修復 | 合併備份 Card；移除 window.confirm/alert；ErrorLogPanel 版本號動態化 |
| `src/pages/orders-page.tsx` | Bug 修復 | 退款上限校驗；silent catch → toast.error；payment_status dead code |
| `src/pages/marketing-page.tsx` | 功能 + 色彩 | 元件行內編輯；Receipt Tab 唯讀；全站色彩修復 |
| `src/pages/print-templates-page.tsx` | 功能 + 色彩 | 過濾器加 receipt/marketing_popup；badge 色彩修復 |
| `src/pages/campaign-manager.tsx` | Bug 修復 | confirm() → Dialog |
| `src/pages/print-settings-page.tsx` | 文案 | LAN scan 文案修正 |
| `src/pages/dashboard-page.tsx` | 色彩 | 統計數字去除硬編碼色彩 |
| `src/pages/print-page.tsx` | 色彩 | 預覽容器 bg-gray-100 → bg-muted |
| `src/components/app-sidebar.tsx` | 色彩 | 通知 badge bg-amber-500 → bg-primary |
| `src/index.css` | UX | 熱敏紙預覽外陰影 |

---

## Bug 詳情

### B1：print-tickets-page EditDialog useState 不重置
**根本原因：** Dialog 組件在 `ticketTypes.map()` 內實例化，導致 N 個 Dialog 共享同一 `editDialog` state，但各自的 `useState` 只在 mount 時初始化，切換編輯對象時初始值不更新。  
**修復：** 將 `EditTicketTypeDialog` 移到 map 外部，加 `key={editDialog?.id ?? "none"}` 強制 remount。

### B2：settings-page 備份 UI 重複
**根本原因：** 兩個不同名稱的 Card 都有備份功能，且呼叫參數不一致（一個無參數，一個傳 `{destDir: null}`）。  
**修復：** 合併為單一「数据备份与恢复」Card，統一使用 `{destDir: null}`，移除冗餘 `backupStatus` state。

### B3：orders-page 退款無上限
**根本原因：** `refundAmount` Input 沒有 `max` 約束，且 `record_order_refund` invoke 的 catch block 為空。  
**修復：** Input 加 `max={cancelTargetOrder?.amount_paid}`；提交時驗證 `amt <= amount_paid`；catch 改為 `toast.error`。

### B4：payment_status ternary dead code
**根本原因：** `setPaymentStatus(order.payment_status === "paid" ? "paid" : "paid")` 兩個分支相同，導致開啟付款登記 Dialog 時永遠初始化為 "paid"，無法正確顯示現有的 "partial" 或 "unpaid" 狀態。  
**修復：** 改為 `setPaymentStatus(order.payment_status)`。

---

## 色彩系統修復

**原則：** 凡 Tailwind 具名色彩（`text-blue-600`、`bg-green-100` 等）在 UI chrome 層面出現，均替換為 CSS variable 系統色（`text-foreground`、`bg-muted`、`bg-primary` 等），確保 dark mode 正確渲染。

**保留：** `print-page.tsx` 熱敏紙模擬區塊內部的灰色、橙色文字——這些模擬實體打印輸出，不屬於 UI chrome。

---

## 驗證

```
npx tsc --noEmit
# Exit 0 — 無 TypeScript 錯誤
```

---

## 未解決項目（移交 v3.7）

1. KDS 時區 Z suffix bug（訂單時間顯示偏移 8 小時）
2. Receipt Tab 預覽仍使用靜態 DEFAULT_RECEIPT_ELEMENTS，未從後端拉取真實模板
3. 打印中心 JSON 輸入框用戶體驗差，需 WYSIWYG 重構
4. ElementEditorFields 應抽取為共用 component
