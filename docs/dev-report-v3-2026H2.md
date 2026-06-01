# Cuckoo v2.8 → v3.0 開發報告與後續規劃

> 基線:v2.7.1 → 本輪交付 v3.0.0(2026-06-02)
> 範圍:二維碼體系 + 掃碼安全 + 營銷組件對齊 + 數據閉環;含用戶旅程分叉樹審計與 debug

---

## 1. 本輪交付總覽

依 `docs/research-qrcode-marketing-2026H2.md` 的三版本里程碑,一輪實作至 v3.0.0。

### v2.8.0 — 二維碼體系 + 掃碼安全
- **HMAC token 基礎設施**(`src-tauri/src/qr_token.rs`,新增):`hmac`+`hex` 依賴;secret 首次運行生成 32 byte 存 `data_local_dir/Cuckoo/qr-secret.bin`(不入 git);`make_token`/`verify_token` 截斷 16 byte 簽名、常數時間比較;payload 助手 `table_payload`/`marketing_payload`。4 個單元測試。
- **桌號碼(固定 + 簽名綁定)**:`sign_table_token` command 生成簽名 token;`/t/:token` 路由 + `resolve_table_token`(command + public endpoint)驗簽並記 scan;舊 `/table/:tableNo` 寬限期保留。
- **下單防刷**:`create_self_order` 增 `token` 參數;`resolve_self_order_table` 寬限期雙模——有 token 則 payload 綁定的桌號覆蓋客戶端值,無 token(舊碼)回退原值。`web_server` 與 command 雙路一致。
- **品牌台卡**:`qr-code-styling` 微信/支付寶風格(圓角碼點 + extra-rounded 定位點),`StyledQR` 組件 + `downloadStyledQR` 高 DPI(1024px)PNG 導出;餐桌管理 QR 對話框改為桌貼台卡(桌號大字 + 掃碼引導)。

### v2.8.0 — 訂單集字碼核銷廢止
- `marketing_qr_tokens` 表(token/order_id/component/ch/redeemed_at/void);`issue_marketing_qr_token`(每單冪等)、`redeem_marketing_qr_token`(驗簽 → void → 寫 `marketing_redemptions`)。
- `get_marketing_popup_content` 為每個 `character_collect` 注入 `qr_token` + 後端算出的 `picked_char`(統一前後端「抽到的字」)。
- 店員核銷閉環:集字卡顯示 `…/#/redeem/{token}` 設計感碼 → `RedeemPage`(`/redeem/:token`)→ 確認核銷;`/api/redeem_marketing_qr_token` public endpoint;首次/重複/無效三態。

### v2.9.0 — 營銷組件對齊 + 商城增強
- **前後端組件對齊**:`MarketingCard` 補齊此前缺失的 `qr_code`/`discount_coupon`/`product_spotlight`/`rich_text`/`dish_easter_egg`(原僅 7 種,現覆蓋商家可配置的全部展示型組件);`qr_code` 加群碼用 `StyledQR` 渲染。
- **商城搜索**:頭部搜索框,跨分類過濾(名稱/描述),搜索時隱藏分類導航 + 空結果提示。
- 加料/去料:沿用既有購物車逐項自由備註(已覆蓋少冰/去蔥等需求)。

### v3.0.0 — 數據閉環
- `qr_scan_events` 表 + `record_qr_scan` 埋點(掃 token 桌碼即記);`get_marketing_funnel(days)`:掃碼→自助下單→核銷漏斗 + 掃碼轉化率 + 各組件核銷分布。
- 營銷中心新增「數據分析」tab:三段漏斗卡 + 轉化率 + 組件核銷條形分布。

---

## 2. 用戶旅程分叉樹遍歷審計

| 旅程節點 | 分支 | 判定 |
|---|---|---|
| **掃碼進店** | 新 token 碼·有效 | ✅ resolve→tableNo,記 scan |
| | 新 token 碼·無效/篡改 | ✅ `tokenInvalid` 友善頁「請重新掃描」 |
| | 舊靜態碼(寬限期) | ✅ token=null,正常進店 |
| **瀏覽下單** | 菜單加載(不依賴 token) | ✅ |
| | 搜索有/無結果 | ✅ 過濾 / 空提示 |
| | 規格商品 / 無規格 | ✅ spec sheet / 直接加購 |
| | 有 token 下單 | ✅ payload 綁桌覆蓋,防偽 |
| | 無 token 下單(舊碼) | ✅ 寬限放行 |
| **下單後彈窗** | 後端注入 token+字 | ✅ 集字卡顯示設計感核銷碼 |
| | 後端超時 → fallback | ⚠️ 降級用前端 djb2 算字,可能與收據不一致(低危,僅斷網降級) |
| | 全 13 種營銷組件 | ✅ 對齊渲染 |
| **店員核銷** | 掃碼首次 | ✅ ok,void |
| | 掃碼重複 | ✅ already 提示 |
| | 無效簽名 | ✅ invalid 提示 |
| | 營銷中心手動驗單核銷 | ✅ 原流程保留 |
| **桌號台卡** | PAD app 內生成(invoke) | ✅ token + 高 DPI 下載 |
| | 瀏覽器訪問餐桌管理 | ⚠️ `sign_table_token` 無 web endpoint → 回退舊 URL(店主多用 app,降級可接受) |
| **數據漏斗** | token 碼掃描 | ✅ 記 scan |
| | 舊靜態碼掃描 | ⚠️ 不經 resolve → 不記 scan(漏斗低估,寬限期後消解) |

---

## 3. Debug 記錄

### 已修復(本輪)
- **P4(真 bug)**:`get_marketing_popup_content` 被顧客彈窗與商家「驗證訂單」兩處調用,原 `issue_marketing_qr_token` 每次新建 token → 同單多碼、核銷狀態分裂。**修復**:issue 改為按 `(order_id, component)` 冪等(查現有未 void token 則復用)。新增回歸測試 `test_marketing_qr_token_idempotent_and_void`(冪等 / 首次核銷 / 重複攔截 / void 後重發新碼 / 篡改拒絕)。
- **隱性 bug(順帶修)**:前端 `djb2` 與後端 `DefaultHasher(SipHash)` 算法不同,集字「抽到的字」在收據與手機彈窗可能不一致。**修復**:後端 `get_marketing_popup_content` 注入 `picked_char`,前端優先採用,單一真相源。
- 清理:`MarketingPayload.nonce` warning(加 `#[allow(dead_code)]`,僅測試讀取)。

### 已知權衡(報告留檔,非阻塞)
| # | 項 | 影響 | 後續方案 |
|---|---|---|---|
| W1 | `redeem` public endpoint 無 auth | 顧客理論上可自核銷(單字無獎勵,動機低) | v3.1 加店員 PIN 閘門 |
| W2 | 舊靜態碼掃描不記 scan | 漏斗 scans 低估 | 寬限期結束、全量換 token 碼後消解 |
| W3 | `sign_table_token` 無 web endpoint | 瀏覽器端餐桌管理回退舊 URL | 如需 web admin,補 public endpoint |
| W4 | 彈窗後端超時 fallback 字不一致 | 僅斷網降級 | 可緩存後端結果重試 |
| W5 | 既有 dead_code warning(`record_coupon_issued`、print.rs `Timelike/Weekday`) | 無功能影響 | 後續清理 |

---

## 4. 驗證

- **後端**:`cargo test --lib` → **63 passed**(原 58 + token 4 + 冪等回歸 1);`cargo check` 綠(僅既有 dead_code warning)。
- **前端**:`tsc --noEmit` exit 0;`vite build` 成功。
- **回歸**:token 簽名/驗簽/篡改/payload swap、集字碼冪等與 void、核銷三態,均有測試覆蓋。

---

## 5. 接下來開發規劃

### v3.1.0 — 核銷安全與裂變(P0/P1)
- **店員 PIN 閘門**:`redeem` 頁/endpoint 校驗店員身份(復用 role_auth PIN),關閉 W1。
- **裂變得券**:`campaign` token(分享連結帶簽名),好友首單雙方得券,基於 token 不引入會員。
- **舊碼埋點補齊**(W2):舊 `/table/` 入口也記 scan,或寬限期結束後硬切 token。

### v3.2.0 — 商城體驗深化(P1)
- **真·加料 modifier**:`get_public_menu` 返回 modifier 選項,商城規格 sheet 結構化加料/去料(對齊 POS),取代自由備註近似。
- **熱銷/推薦標記**:基於銷量驅動;套餐/組合提客單價。
- **台卡整圖導出**(印刷增強):`html-to-image` 合成「桌號 + 店名 + logo + 碼」整張台卡 PNG/SVG,直接交印刷店。

### v3.3.0 — 數據與運營(P1/P2)
- **組件 A/B 與時段分析**:各營銷組件對轉化/復購的貢獻;按時段/星期的掃碼熱力。
- **折價券金額閉環**:`discount_coupon` 核銷記錄實付優惠金額,進營收報表。
- **告警**:掃碼異常激增(防刷)、核銷率異常。

### 技術債清理(穿插)
- 既有 dead_code warning(W5);`unsafe-inline` CSP 收斂評估;前端 bundle 分包(當前單 chunk >500KB)。

---

## 6. 結論

三版本里程碑已一輪落地至 v3.0.0:二維碼體系(固定桌號碼 + 每單可廢止集字碼 + 加群碼)+ 掃碼防刷安全(寬限期雙模)+ 全營銷組件前後端對齊 + 掃碼核銷閉環 + 數據漏斗。用戶旅程分叉樹遍歷審計定位並修復 1 個真 bug(P4 token 重複簽發)與 1 個隱性 bug(前後端抽字算法不一致),5 項已知權衡留檔。全量測試與構建通過。後續以「核銷安全 + 裂變(v3.1)→ 商城深化(v3.2)→ 數據運營(v3.3)」推進。

全程恪守:簡體中文界面、防呆零配置默認、小餐廳/飲品店場景、單台 Android PAD 可跑。
