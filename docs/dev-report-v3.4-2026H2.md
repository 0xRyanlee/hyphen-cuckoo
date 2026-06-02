# Cuckoo v3.4 開發報告

> 基線:v3.3.1 → v3.4.0(2026-06-02)
> 主題:營銷數據閉環 + campaign 防刷後端 + 閃退殘留收口

## 1. 交付

| 項 | Kano | 狀態 |
|---|---|---|
| E1 顧客頁路由 lazy-loading | O | ✅ v3.3.1 |
| campaign 效果看板(領取/核銷/核銷率) | O→A | ✅ |
| MUL campaign 後端每日領取上限 | O | ✅ |
| backup/printer 的 dirs Android fallback 收口 | M(安全) | ✅ |
| B3' 完整 modifier | O | ⏸ scope |
| B7 折價券金額閉環 | O | ⏸ scope |

### 1.1 campaign 效果看板
`list_campaigns` 子查詢統計每活動 `claimed`/`redeemed`,營銷中心列表顯示「領取 X · 核銷 Y · 核銷率 Z%」,復用 `marketing_qr_tokens`。

### 1.2 MUL 後端每日上限
`campaigns.daily_limit`(schema + migration 兼容舊庫);`issue_campaign_coupon` 當日計數 ≥ 上限(>0)時拒發(`daily_limit_reached`);創建表單「每日領取上限(0=不限)」;CampaignPage「今日名額已領完」。與前端 localStorage 守衛構成**雙層防刷**。

### 1.3 閃退殘留收口(RCA 的 W)
- `backup_database`:Android(`document_dir`→None)降級到 db 同目錄沙盒,備份不再失敗。
- `printer debug_output_dir`:fallback `"."` → `temp_dir`(Android 可寫)。
- 至此啟動 + backup + printer 三處 `dirs` Android 隱患全部收口。

## 2. Scope 說明(需充足上下文做完整,非半成品)

### B3' 完整 modifier(→ 後續)
完整版拆解:
- a. `menu_item_modifiers` 表(item_id, name, price_delta, sort)
- b. 菜單頁 per-item 加料配置 UI
- c. `get_public_menu` 返回每 item 的 modifier 選項
- d. 商城規格 sheet 結構化加料(替代口味快選→備註)
- e. `create_self_order` 寫 `order_item_modifiers` + 單價含加料
- 現狀:口味快選→備註已滿足顧客定制,**非阻塞**。

### B7 折價券金額閉環(→ 後續)
拆解:
- a. `discount_coupon` 核銷流程(目前無獨立掃碼閉環,靠人工)
- b. 核銷時記實付優惠金額
- c. 進營收報表,算促銷成本/ROI
- 依賴:可復用 campaign 券核銷閉環(掃碼 void),為 discount_coupon 補金額字段。

## 3. 驗證
- `cargo test --lib` → 66 passed;`cargo check` 綠;`tsc` 0;`vite build` 綠。
- 分批 commit,每批編譯通過。

## 4. 結論
v3.4 完成營銷數據閉環(效果看板)、campaign 雙層防刷(前端守衛 + 後端上限)、閃退殘留三處收口。**所有當前 context 可可靠完成的開發項已落地**;B3'/B7 為需充足上下文的大工程,明確 scope 並附拆解,避免半成品。守住「單店、無會員、本地、零配置」邊界。
