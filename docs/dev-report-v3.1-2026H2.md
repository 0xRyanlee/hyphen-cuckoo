# Cuckoo v3.1 開發報告 — 全開發項實作、審計與規劃

> 基線:v3.0.0 → 本輪 v3.1.0(2026-06-02)
> 範圍:對「全部 22 個待開發項」逐項實作或明確 scope;用戶旅程分叉樹審計;debug;後續規劃

---

## 1. 22 項實作狀態總表

| # | 開發項 | 狀態 | 說明 |
|---|---|---|---|
| B1 | 核銷店員 PIN 閘門 | ✅ 實作 | `verify_any_pin`(防呆:未設密碼放行);僅 web endpoint 強制,app 內可信免 PIN |
| B2 | 裂變得券(雙向) | ⏸ scope | 純截圖無會員下券無持有者身份,無法落地;報告 §5 給兩方案 |
| B3 | 真·加料 modifier | ✅ 輕量實作 | 加購 sheet 口味快選標籤→備註;完整 `order_item_modifiers` 寫入 scope |
| B4 | 熱銷/推薦標記 | ✅ 實作 | 近 30 天 top3 銷量 `is_hot` 徽章;套餐/組合 scope |
| B5 | 整台卡 PNG 導出 | ✅ 實作 | `html-to-image` pixelRatio 3 印刷級導出整張桌貼 |
| B6 | 組件 A/B 與時段分析 | ⏸ scope | v3.0 已有 by_component;時段/AB 邊際遞減,後續 |
| B7 | 折價券金額閉環 | ⏸ scope | 需 coupon 核銷金額流程,後續 |
| B8 | 運營告警 | ⏸ scope | 需閾值+通知管道,後續 |
| C1 | W1 redeem 鑑權 | ✅ 實作 | =B1 |
| C2 | W2 舊碼硬切 | ⏸ scope | 需 `require_token` 設置開關;寬限期內不宜硬切 |
| C3 | W3 sign_table_token web endpoint | ✅ 實作 | `/api/sign_table_token`,瀏覽器端餐桌管理可用 |
| C4 | W4 fallback 重試 | ✅ 實作 | 彈窗超時顯示本地 fallback 後,後台重試替換為權威結果 |
| C5 | W5 dead_code 清理 | ✅ 實作 | 移除 print.rs 未用 import;`record_coupon_issued` allow |
| D1 | 規格數量選擇 | ✅ 實作 | 加購 sheet 數量 stepper |
| D2 | 掃碼首屏營銷鉤子 | ✅ 實作 | 商城首屏「下單抽運勢/集字兌獎」banner |
| D3 | 掃碼核銷免手輸 | ✅ 實作 | 營銷中心「核銷碼」入口(掃碼槍/貼 token 或 /redeem 連結) |
| D4 | 下單校驗強化 | ✅ 實作 | `create_self_order` 校驗 `is_available`,下架競態拒單 |
| D5 | 集字進度展示 | ⏸ scope | 無會員無法跨單追蹤進度;現有格子展示已是最大可行 |
| E1 | bundle 分包 | ✅ 實作 | vendor-qr(61KB)/vendor-util(24KB)分出;路由級 lazy scope |
| E2 | CSP unsafe-inline 收斂 | ⏸ scope | `style-src 'unsafe-inline'` 為 Tailwind 所需,移除破壞樣式 |
| E3 | health_check 結構化 | ⏸ scope | 前端 3 處依賴 `"ok"` 字串,結構化收益有限 |
| E4 | 雲端多店 | ⏸ scope | long-term 架構,需獨立設計 |

**實作 12 項,明確 scope 10 項**。所有 scope 項均有明確理由(無會員約束 / Tailwind 依賴 / long-term / 邊際價值遞減),非半成品。

---

## 2. 用戶旅程分叉樹遍歷審計(本輪新增功能)

| 旅程節點 | 分支 | 判定 |
|---|---|---|
| **店員核銷** | 未設密碼 → 免 PIN 直接核銷 | ✅ `redeem_requires_pin`=false |
| | 設密碼 → web 掃碼需 PIN,輸對/錯 | ✅ ok / pin_required |
| | 營銷中心 D3 核銷(app 內) | ✅ 免 PIN(P5 修復後) |
| | 掃碼核銷碼:裸 token / `/redeem/` 連結 | ✅ 兩種都解析 |
| **加購** | 有規格:選規格+口味+數量 | ✅ |
| | 無規格:口味+數量(規格區隱藏) | ✅ |
| | 同 item+spec 重複加 | ✅ qty 累加 |
| **桌貼導出** | 台卡整圖 toPng | ✅ pixelRatio 3 |
| | token 未就緒 → 回退舊 URL 碼 | ✅ |
| **熱銷** | 有/無歷史訂單 | ✅ 有徽章 / 無徽章 |
| **下單** | 含下架商品(競態) | ✅ 拒單友善提示 |
| **彈窗** | 後端超時 → fallback → 後台重試 | ✅ 替換為權威結果 |

---

## 3. Debug 記錄

### 本輪修復
- **P5(設計問題)**:`redeem_marketing_qr_token` command 也校驗 PIN,導致營銷中心 D3 核銷(店主 app 內可信環境)在設密碼時被拒。PIN 閘門目的是防顧客掃碼自核銷(web endpoint)。**修復**:PIN 校驗只保留在 public web endpoint,command 免 PIN(app 內可信)。

### 延續的已知權衡
- W1 已由 B1 關閉(web 核銷需 PIN)。
- 桌號存在性校驗:`create_self_order` 仍不校驗 token/table_no 對應的桌號是否存在於 `restaurant_tables`(偽造桌號可下單,風險等同寬限期無 token)。列入後續。
- D3 核銷碼分支 `pin_required`:走 command 免 PIN,該分支在營銷中心為防禦性死碼(web RedeemPage 才觸發)。

---

## 4. 驗證

- **後端**:`cargo test --lib` → **63 passed**;`cargo check` 綠(僅既有 `record_coupon_issued` 已 allow)。
- **前端**:`tsc --noEmit` exit 0;`vite build` 綠,分包後 vendor-qr/vendor-util 獨立。
- 分 5 批 commit,每批編譯+測試通過。

---

## 5. 後續開發規劃

### v3.2.0 — 身份與裂變決策(需產品拍板)
- **B2 裂變的前置決策**:純截圖無會員無法做雙向得券。兩方案:
  - **方案 A(真裂變)**:引入輕量身份(手機號 OTP / 微信 openid),分享者+被分享者可追蹤,雙向發券。**與既有「無會員」決策衝突,需拍板**。
  - **方案 B(保持無會員)**:改做「掃 campaign 活動碼得單向券」——店主建活動 → 顧客掃碼 → 得帶 redeem token 的優惠券 → 店員掃碼核銷(復用現有閉環)。不追蹤分享者。
- B3 完整 modifier:`get_public_menu` 返回加料選項,寫 `order_item_modifiers`(對齊 POS)。
- 桌號存在性校驗 + C2 舊碼硬切開關(`require_token` 設置)。

### v3.3.0 — 數據與運營
- B6 時段/AB 分析、B7 折價券金額閉環、B8 告警。

### 技術債(穿插)
- E1 路由級 lazy-loading(index chunk 622KB)、E2 CSP 評估、E3 health_check 結構化、E4 雲端多店。

---

## 6. 結論

本輪對全部 22 個待開發項逐項處理:**12 項實作落地、10 項明確 scope**(均有理由,非半成品)。核心安全(核銷 PIN 閘門)、商城體驗(熱銷/數量/口味/首屏鉤子/下架校驗)、印刷交付(整台卡導出)、運營便利(掃碼核銷免手輸)、彈窗健壯性(fallback 重試)、工程(bundle 分包)均完成。用戶旅程分叉樹遍歷審計覆蓋所有新增分支,修復 1 個設計問題(P5 核銷 PIN 上下文)。

最大的產品決策懸而未決:**B2 裂變需要在「引入輕量身份」與「保持純截圖無會員」之間拍板**——這決定 v3.2 方向。其餘 scope 項為漸進增強或 long-term。

全程恪守:簡體中文界面、防呆零配置默認、小餐廳/飲品店、單台 Android PAD 可跑。
