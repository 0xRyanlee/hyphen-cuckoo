# Changelog

## [2.3.0] — 2026-05-30

### Features

* **打印創意模組**：收據模板新增 `fortune`（運勢抽籤）、`quote`（今日靈感卡）、`art`（ASCII 藝術圖塊）、`image_block`（圖像佔位）四種 element type；模板編輯器加入快速插入按鈕
* **運勢抽籤**：三級（大吉/中吉/小吉，僅正向），支援 `daily`/`per_table`/`per_order` 三種種子策略，完整文字庫嵌入二進制
* **今日靈感卡**：支援中/英/日多語系語錄，按日期輪替
* **菜品可售狀態 API 語義對齊**：`toggle_menu_item_availability` 重命名為 `set_menu_item_availability`，補充 web server HTTP 端點與 3 個回歸測試
* **配方單位兼容性校驗**：`add_recipe_item` 後端加入 unit_type 一致性驗證（前端已有同等過濾）

### Bug Fixes

* **recipes-page.tsx 刪除語義**：配方「刪除」Dialog 更正為「停用」（邏輯刪除，資料保留），配方明細「刪除」明示永久移除
* **CSP 審計**：確認 `unsafe-eval` 已完全不存在；`unsafe-inline` for style-src 記錄依賴阻礙並延至 v3.0.x

---

## [1.4.0](https://github.com/0xRyanlee/Cuckoo/compare/cuckoo-v1.3.1...cuckoo-v1.4.0) (2026-05-15)


### Features

* add configurable low stock threshold dialog ([377acab](https://github.com/0xRyanlee/Cuckoo/commit/377acabf9c48f10b269853f172ebc214beaeebd1))
* Cuckoo v1.2.2 - Complete Audit Fix, Tree-Table Recipes, Dependency Guard & Telemetry expansion ([f78d11c](https://github.com/0xRyanlee/Cuckoo/commit/f78d11c25b123b7f5fbacde2e900ec7bba163124))
* include release documentation files in tauri application bundle ([32b7805](https://github.com/0xRyanlee/Cuckoo/commit/32b7805bb7fb52c402197070708377d2249aba59))
* update app icons with new cuckoo bird logo ([7e33eb7](https://github.com/0xRyanlee/Cuckoo/commit/7e33eb7452685bd043a48fa28ca9dde60ee05363))
* v1.2.4 — printer management, sample data, nav fixes, Simplified Chinese ([488ae3e](https://github.com/0xRyanlee/Cuckoo/commit/488ae3ec7747d68127df6b70e9aa94a351b95101))
* v1.3.0 — 供應商商品、日常支出、毛利計算、嚴格審計修復 ([dbc1fd7](https://github.com/0xRyanlee/Cuckoo/commit/dbc1fd7eee03cf6dc953479df2bd134eb0b264ba))
* 打印机设置向导 — 傻瓜式接入引导 UI ([00b79f1](https://github.com/0xRyanlee/Cuckoo/commit/00b79f1b209f4f64502ffc6a6d662b31f1007719))
* 自動檢查更新 — GitHub Releases 版本偵測 + 平台下載 + 進度條 ([56c07d9](https://github.com/0xRyanlee/Cuckoo/commit/56c07d9799aced12d32fe553460a536d9431942c))


### Bug Fixes

* add missing companion changes for toggle_menu_item and sidebar badge ([353a0bc](https://github.com/0xRyanlee/Cuckoo/commit/353a0bc2326a62de0b9b799a55609107883df26a))
* add missing logger.ts that was never committed ([1fb98c2](https://github.com/0xRyanlee/Cuckoo/commit/1fb98c225e1c2da15b86c6c4d620d5c025844824))
* add transaction to set_default_ticket_type for atomicity ([bccef34](https://github.com/0xRyanlee/Cuckoo/commit/bccef3460290b42646dfe70520b8723f0832e29c))
* **ci:** 修复 Windows CI 构建失败 — 添加 nsis target，更新 resource 路径 ([8054e41](https://github.com/0xRyanlee/Cuckoo/commit/8054e41a9eb8163b7307b8d986b64cb9ed021d4a))
* print ticket type SQL + build verification ([b17c48e](https://github.com/0xRyanlee/Cuckoo/commit/b17c48eea46d446f027fd74a61c0e50771adad9a))
* regenerate app icons using tauri-cli to fix windows RC format error ([683eec9](https://github.com/0xRyanlee/Cuckoo/commit/683eec969f52a2907f5292ba853339a745b72bca))
* resolve startup crash — menu_categories missing code column ([642e047](https://github.com/0xRyanlee/Cuckoo/commit/642e0471d940688d66d2f8c3b643bfbc19f965e2))
* sidebar logo padding to avoid macOS window controls overlap ([832364b](https://github.com/0xRyanlee/Cuckoo/commit/832364b1b694b6f18912926dd9499dc8067f478e))
* silence telemetry noise and prevent over-firing ([750a96d](https://github.com/0xRyanlee/Cuckoo/commit/750a96d7298bed1d4827f7ad373ea090fc965b50))
* silence telemetry noise and prevent over-firing ([5bb6f66](https://github.com/0xRyanlee/Cuckoo/commit/5bb6f66e669143ffc7a76cfcb7f0c455908b4149))
* TypeScript types and async function missing in recipes page ([9fa8cba](https://github.com/0xRyanlee/Cuckoo/commit/9fa8cba12a081bfc0803aa1918f01fee1d1e2d42))
* use rustls-tls for reqwest to fix windows build without openssl ([accb9b5](https://github.com/0xRyanlee/Cuckoo/commit/accb9b5ac23946bc7667ffa2a8588223cebaf455))
