# Cuckoo v3.2 開發報告 — 掃碼活動碼得券(方案B)

> 基線:v3.1.0 → 本輪 v3.2.0(2026-06-02)
> 決策:B2 裂變採**方案 B**(保持純截圖無會員,做單向「掃 campaign 活動碼得券」)

## 1. 交付:掃碼活動碼得券閉環

**完整鏈路**
```
店主建活動(营销中心→扫码活动 tab)
  → 生成活动码海报(StyledQR + html-to-image 高清导出，可张贴/投朋友圈)
顾客扫活动码 /c/{token}
  → 每扫领一张专属优惠券(per-scan 唯一 token，非幂等)
  → 截图保存
店员扫券上的核销码 /redeem/{coupon_token}
  → 复用 redeem_marketing_qr_token 闭环 → void(一码一次)
```

**後端**
- `campaigns` 表(name/discount_type/discount_value/condition/valid_days/is_active)
- `qr_token`:`campaign_payload`/`parse_campaign_payload`(固定碼 `c|{id}`)
- DB:`create_campaign`/`list_campaigns`/`set_campaign_active`/`delete_campaign`/`issue_campaign_coupon`(非冪等,每掃新券,component=`campaign_coupon`,復用 `marketing_qr_tokens` + redeem 閉環)
- commands(6)+ public endpoint `/api/resolve_campaign`

**前端**
- `CampaignManager`(营销中心「扫码活动」tab):建活動表單 + 列表(啟用/刪除)+ 活動碼海報對話框(高清下載)
- `CampaignPage`(`/c/:token`):顧客領券頁,設計感券 + 核銷碼
- 核銷頁加 `campaign_coupon` 中文 label

## 2. 用戶旅程分叉樹審計

| 節點 | 分支 | 判定 |
|---|---|---|
| 顧客掃活動碼 | 有效啟用 → 領券 | ✅ |
| | 停用/不存在 → 「活动已结束」 | ✅ valid:false |
| | 多次掃 | ✅ 每次新券(非冪等) |
| 店員核銷 | 活動券掃碼 | ✅ 復用 redeem,void |
| | 重複核銷 | ✅ already |
| 店主管理 | 建/列表/啟停/刪除 | ✅ |
| | 活動碼海報導出 | ✅ html-to-image 高清 |
| | 海報 baseUrl(loopback) | ✅ 復用餐桌管理手動 IP override |

## 3. Debug(審計修復)
- **A**:核銷頁缺 `campaign_coupon` 中文 label → 補「活动优惠券」。
- **B**:活動碼海報 baseUrl 未復用手動 IP override,自動 IP 失敗時海報碼回環 → 讀 `localStorage` 手動 IP 替換 host。

## 4. 設計權衡(留檔)
- **多領**:無會員下無法限制一人多次掃碼領券。多領是**店主可控的促銷成本**(優惠由店主設定),核銷時店員把關。可接受;如需限制,後續可加設備指紋/每日上限。
- **刪除活動**:刪 campaign 後,已發出的券(marketing_qr_tokens)仍存在但 redeem 時查不到 campaign——實際 redeem 只校驗 token 簽名 + void 狀態,不依賴 campaign 行,故已發券仍可核銷(component 顯示 campaign_coupon)。刪除前有確認提示。

## 5. 驗證
- `cargo test --lib` → **64 passed**(+`test_campaign_coupon_unique_per_scan_and_redeem`:每掃唯一/核銷/重複攔截/停用拒發)
- `tsc` 0;`vite build` 綠

## 6. 結論
B2 裂變按用戶拍板的**方案 B** 落地:保持純截圖無會員,以「掃 campaign 活動碼得單向券」實現可投放、可追蹤(掃碼埋點 kind=campaign)、可核銷的促銷裂變,完全復用既有 token 核銷閉環。方案 A(輕量身份做雙向裂變)仍封存,如未來引入身份系統再啟。
