# Cuckoo v3.5 開發報告

> 基線:v3.4.0 → v3.5.0(2026-06-03)
> 主題:商城結構化加料(B3')+ 折價券金額閉環(B7)—— 清掉 v3.3/v3.4 兩個 scope 出去的 O 級項

## 1. 交付

| 項 | Kano | 狀態 |
|---|---|---|
| B3' 完整 modifier(結構化加料) | O | ✅ |
| B7 折價券金額閉環 | O | ✅ |

至此 v3.4 報告中標 ⏸scope 的兩項全部落地,roadmap v3.4「營銷 ROI」與 v3.3 自助下單能力對齊 POS 的目標收口。

### 1.1 B3' 結構化加料

顧客自助下單從「口味快選 → 拼進備註字串」升級為**帶價結構化加料**,金額進訂單。

**後端**
- `SelfOrderItemInput` 新增 `modifiers: Vec<SelfOrderModifierInput>`(`modifier_type` / `price_delta` / `qty`,serde default 容缺省)。
- `create_self_order`:逐 item 取 `last_insert_rowid()` 後寫 `order_item_modifiers`;`amount_total = Σ(item.qty·unit_price) + Σ(modifier.price_delta·qty)`。
- `web_server` 自助下單直連端點解析 `items[i].modifiers`。

**前端**(`self-order-page.tsx`)
- `FLAVOR_OPTIONS`(純字串)→ `MODIFIER_OPTIONS`(帶價硬編碼合理預設:少冰/少糖=0、加珍珠/椰果=+3、加布丁=+4、加蛋=+5、加大份=+8)。
- `CartItem` 加 `modifiers`;`cartKey` 納入加料簽名,使同品不同加料分行;單價 = 基價 + Σ加料;規格 sheet 與購物車展示加料與加價;`submitOrder` 把 `modifiers`(qty 對齊行數量)隨 items 上送。
- **未做**:店主後台 per-item 自定義加料清單(`menu_item_modifiers` 表 + 配置 UI)留待店主有此需求時再開,現用全局預設,零配置即可用。

### 1.2 B7 折價券金額閉環

讓店主從「發了券」走到「券花了多少」,折價券核銷的實付優惠額進報表算促銷成本。

**設計**:小票折價券印的 12-hex `優惠碼`由 `order_id` 確定、天然唯一,直接當去重鍵——**無需在列印時落發券記錄**(避開即時預覽會污染庫的風險),核銷時店員輸碼 + 實付優惠額即可。

- schema:`marketing_redemptions` 加 `amount REAL`(CREATE + 兼容舊庫 migration)。
- `redeem_discount_coupon(code, amount, staff)`:以 `component_type='discount_coupon' AND note=code` 去重,重複返 `already`;`order_id=0` 占位。
- `get_marketing_funnel` 增 `coupon_discount` = 期間折價券 `amount` 合計;`get_marketing_redemptions` 帶出 `amount`。
- command + `/api/redeem_discount_coupon`(PIN 閘門,同既有營銷核銷);Web 邊界拒 `amount<0`。
- 前端營銷中心「兌獎核銷」頁加折價券核銷卡(優惠碼 + 實付優惠);「數據分析」漏斗顯示「折價券促銷成本」;兌獎記錄行顯示 `-¥amount`。

## 2. 驗證

- `cargo test --lib`:**67 passed / 0 failed**(新增 `test_discount_coupon_redeem_records_amount_once`:核銷成功 → 重複 already → funnel `coupon_discount=12.5`)。
- `npx tsc --noEmit`:乾淨。
- `npm run build`:成功。
- `cargo build --release`:Finished(exit 0)。

## 3. 自查(功能完成審查 · A/E/F/H)

- **A 錯漏**:去重按 `component_type` 限定,折價券碼不會與集字/掃碼核銷的 note 串擾。`modifier.qty` 對齊行 qty,後端金額公式與前端單價一致。
- **E 防呆**:`amount` 為外部輸入,Web 端點拒空碼與負值;前端按鈕禁用直到碼非空且金額≥0。
- **F 對抗**:`/api/redeem_discount_coupon` 與既有營銷核銷同走 `verify_any_pin` PIN 閘門;未設 PIN 放行的取捨與既有一致(對齊「未設則放行」反向規避策略)。
- **H 成本**:核銷與漏斗均為單行聚合查詢,`marketing_redemptions` 小表,無新增熱點。

## 4. 版本

`package.json` / `tauri.conf.json` / `Cargo.toml` / `Cargo.lock` 全部 3.4.0 → **3.5.0**;tag `v3.5.0` 觸發 CI / Android 簽名 APK / Release。
