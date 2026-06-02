# Cuckoo v3.3 開發報告

> 基線:v3.2.1 → v3.3.0(2026-06-02)
> 主題:**Android 閃退 P0 修復 + 集字兌換閉環(亮點)+ 掃碼安全收口**
> 依據:`roadmap-kano-directions-2026H2.md` v3.3 候選

---

## 1. 交付總覽

| 項 | Kano | 狀態 | 版本 |
|---|---|---|---|
| **P0 Android PAD 啟動閃退修復** | M | ✅ | v3.2.2 hotfix |
| **集字兌換閉環(4.0)** | A | ✅ | v3.3.0 |
| C2 舊碼硬切開關 | O | ✅ | v3.3.0 |
| MUL campaign 多領限制(前端防呆) | O | ✅ | v3.3.0 |
| B3' 完整 modifier | O | ⏸ scope→v3.4 | — |
| E1 路由級 lazy-loading | O | ⏸ scope→v3.4 | — |

### 1.1 P0 閃退修復(v3.2.2,已發布)
根因:`dirs::data_local_dir()` 在 Android 返 None → fallback "." 不可寫 → `create_dir_all().expect()` panic → `panic=abort` 閃退。修復:FS 初始化移入 `setup`(app handle 可用),Android 用 `app_data_dir()` 沙盒路徑,桌面保持 `dirs`(數據不變),去除啟動 `expect`,QR secret 持久化,crash log 強化。詳見 `docs/android-crash-rca-2026H2.md`。**⚠️ 需用戶 Android 實測確認**。

### 1.2 集字兌換閉環(4.0,亮點)
修正先前「需會員」誤判——**純截圖無會員即可實現**:
- 顧客每單得【字】+ 核銷碼 + **序號(order_no)**,截圖自持(系統無狀態)
- 老闆「集字兌換」(營銷中心兌獎 tab):逐張**掃碼 / 貼核銷碼 / 輸序號** → `peek`(驗簽返字不銷)累積已集字 chips → 湊齊 → `collect_redeem_set` 一次性批量銷碼作廢
- 後端:`peek_marketing_qr_token` / `collect_redeem_set`(all-or-nothing) / `find_collect_token_by_order_no`(序號後備),復用 `marketing_qr_tokens` + HMAC 簽名 + void

### 1.3 C2 舊碼硬切開關
`RoleAuthStore.require_token`:開啟後無有效 token 的舊靜態碼下單被拒(寬限期結束的安全收口);餐桌管理開關 + 防呆提示。command + web_server 雙路生效。

### 1.4 MUL campaign 多領限制
CampaignPage 同設備+同海報+同日 localStorage 領取守衛,已領顯示「今日已領取」。防呆級(防誤領);後端每日總量上限留 v3.4。

---

## 2. 用戶旅程分叉樹審計(本輪新增)

| 旅程 | 分支 | 判定 |
|---|---|---|
| **啟動(Android)** | data_dir 可寫 / 不可寫 | ✅ 沙盒路徑 / 降級不崩+crash log |
| **集字兌換** | 掃碼/貼碼/輸序號加入 | ✅ 三入口解析 |
| | peek 已 void | ✅ 提示已核銷,不加入 |
| | 重複加同券 | ✅ 提示已在列表 |
| | 湊齊批量銷 | ✅ all-or-nothing,某碼已 void 則整批拒 |
| | 序號 order_no 後備 | ✅ find by order_no |
| **C2 硬切** | 開啟+舊碼 / 開啟+新碼 / 關閉 | ✅ 拒 / 過 / 寬限 |
| **MUL** | 同設備同日重領 | ✅ 今日已領 |
| **集字券截圖** | 顯示字+碼+序號 | ✅ 截圖友善 |

**審計結論**:新增分支全部覆蓋,無錯誤點;集字兌換 all-or-nothing 保證批量核銷一致性。

---

## 3. 驗證
- `cargo test --lib` → **66 passed**(含 collect_redeem / table_existence / token / campaign 回歸)
- `tsc --noEmit` exit 0;`vite build` 綠
- 分批 commit,每批編譯+測試通過

---

## 4. Scope 說明(v3.4)
- **B3' 完整 modifier**:`get_public_menu` 返回加料選項 + 寫 `order_item_modifiers`,對齊 POS。當前商城用口味快選→備註(輕量近似已可用)。
- **E1 路由 lazy-loading**:顧客頁 React.lazy 分包(index 622KB)。
- 兩項為 O 級漸進優化,本輪因聚焦 P0 安全 + 集字亮點而順延,不影響現有功能。

---

## 5. 後續規劃
- **v3.4**:B3' 完整 modifier + E1 路由分包 + MUL 後端總量 + B7 折價券金額 + campaign 效果看板
- **持續**:Android 閃退實測閉環(收用戶反饋);backup/printer 的 `dirs` 用法核查(非啟動,低危)
- **封存**:雲端多店、雙向裂變(需突破無會員/本地邊界,待戰略決策)

---

## 6. 結論
v3.3 以 **Kano M 級(閃退修復)優先、A 級(集字兌換)為亮點、O 級(C2/MUL)安全收口** 落地。集字兌換閉環印證了「純截圖無會員」模型的延展力——無需身份即實現完整集字玩法。B3'/E1 明確順延 v3.4。守住「單店、無會員、本地、零配置」邊界。
