# Android PAD 啟動閃退 — 根因分析與修復(RCA)

> 報告:用戶回報安卓 PAD 安裝後啟動即閃退
> 修復:v3.2.2(2026-06-02)
> 狀態:已修復(代碼+CI 構建);**需用戶 Android 實測確認**(本地無 Android 設備)

## 1. 現象
APK 能安裝,但一啟動立即閃退,無錯誤提示。桌面版(macOS/Windows)正常。

## 2. 根因鏈(高確定性)
```
dirs::data_local_dir() 在 Android 返回 None
  → fallback PathBuf::from(".")（Android 沙盒外/不可寫)
  → fs::create_dir_all(".../Cuckoo").expect("Failed to create data directory")  ← panic
  → Cargo.toml [profile.release] panic = "abort"
  → 進程直接 abort = 閃退(無 unwind、無提示)
```
- 觸發點:`lib.rs` 啟動期 `get_db_path` / `get_log_dir` / `get_role_auth_path` 三處 `expect`。
- 為何桌面正常:桌面 `dirs::data_local_dir()` 返回有效路徑(`~/Library/Application Support` 等),`dirs` crate 在 **Android 平台多數目錄函數返回 None**。
- 為何裝得上卻啟動崩:安裝不執行路徑邏輯;啟動 `run()` 早期(Builder 之前、無 app handle)即調用 `expect`。

## 3. 修復(v3.2.2)
**核心**:檔案系統初始化從「Builder 之前」移入 `setup`(此時有 app handle → 可取 Android 沙盒路徑)。

| 修復 | 說明 |
|---|---|
| 路徑來源平台分支 | Android 用 Tauri `app.path().app_data_dir()`(沙盒 `/data/data/<pkg>/files`);桌面保持 `dirs::data_local_dir()`(**現有用戶數據路徑不變**) |
| 去除啟動 `expect` | `create_dir_all` 失敗只記錄不 panic;`db_path.to_str().unwrap()` → `unwrap_or` 降級;DB 初始化失敗 `return Err`(Tauri 顯示錯誤而非裸 abort) |
| 初始化移入 setup | db / role_auth / web_server / `app.manage(AppState)` 全在 setup 內(app handle 可用) |
| QR secret 持久化 | `qr_token::set_secret_dir(data_dir)`:secret 存同一沙盒目錄,**避免 Android 重啟 secret 變 → 已印桌號碼失效** |
| crash log 強化 | 早期 panic hook 只寫 stderr(Android logcat 可見);拿到 data_dir 後升級為寫 `logs/crash.log` |

## 4. 防呆設計
- **降級不崩**:路徑/DB 失敗走降級或顯式錯誤頁,絕不啟動期 panic。
- **可診斷**:閃退前 panic hook 寫 stderr(logcat) + crash.log,便於現場排查。
- **數據不丟**:桌面路徑保持原 `dirs` 路徑,升級不遷移、不丟現有數據。
- **secret 持久**:桌號碼/集字碼 token 跨重啟有效。

## 5. 殘留與後續(併入開發計劃)
| 項 | 風險 | 處置 |
|---|---|---|
| `commands.rs` backup 用 `dirs::document_dir`(Android 或 None) | 低(用戶主動操作,失敗返 Err 不崩) | v3.3 改用 app 路徑 |
| `printer.rs` `dirs::data_local_dir` | 低(非啟動,mock/打印路徑) | v3.3 核查 |
| 無 Android 設備本地驗證 | 中 | **需用戶實測**;CI 構建成功僅證明可編譯,運行時靠實測 |
| Android 運行時權限/WebView | 低 | 若實測仍崩,讀 logcat / crash.log 二次定位 |

## 6. 驗證
- `cargo test --lib` → 66 passed;`cargo check` Android target 由 CI 驗證。
- **待用戶在 Android PAD 安裝 v3.2.2 APK 實測確認不再閃退**;若仍崩,提供 `logcat` 或應用數據目錄 `logs/crash.log`。
