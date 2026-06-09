use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const GITHUB_API: &str =
    "https://api.github.com/repos/0xRyanlee/hyphen-cuckoo/releases/latest";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateInfo {
    pub current_version: String,
    pub new_version: String,
    pub release_notes: String,
    pub download_url: String,
    pub release_url: String,
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    body: Option<String>,
    html_url: String,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

pub async fn fetch_update(current_version: &str) -> Result<Option<UpdateInfo>, String> {
    let client = Client::builder()
        .user_agent(format!("CuckooApp/{current_version}"))
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let release: GithubRelease = client
        .get(GITHUB_API)
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {e}"))?
        .json()
        .await
        .map_err(|e| format!("解析失败: {e}"))?;

    let new_ver = release.tag_name.trim_start_matches('v');
    if !is_newer(new_ver, current_version) {
        return Ok(None);
    }

    #[cfg(target_os = "macos")]
    let asset = release.assets.iter().find(|a| a.name.ends_with(".dmg"));

    #[cfg(target_os = "windows")]
    let asset = release
        .assets
        .iter()
        .find(|a| a.name.ends_with("-setup.exe"));

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let asset: Option<&GithubAsset> = None;

    let url = match asset {
        Some(a) => a.browser_download_url.clone(),
        None => return Ok(None),
    };

    Ok(Some(UpdateInfo {
        current_version: current_version.to_string(),
        new_version: new_ver.to_string(),
        release_notes: release.body.unwrap_or_default(),
        download_url: url,
        release_url: release.html_url,
    }))
}

/// Blocking download with progress; runs in a dedicated thread.
/// Emits `update-progress {downloaded, total}`, `update-complete`, `update-error` to all windows.
pub fn download_and_open(url: &str, app: tauri::AppHandle) {
    use std::io::{Read, Write};
    use tauri::Emitter;

    let emit_err = |msg: String| {
        let _ = app.emit("update-error", msg);
    };

    // ── Download ──────────────────────────────────────────────────────────────
    let client = match reqwest::blocking::Client::builder()
        .user_agent("CuckooApp")
        .timeout(Duration::from_secs(600))
        .build()
    {
        Ok(c) => c,
        Err(e) => { emit_err(e.to_string()); return; }
    };

    let mut resp = match client.get(url).send() {
        Ok(r) => r,
        Err(e) => { emit_err(format!("下载失败: {e}")); return; }
    };

    let total = resp.content_length().unwrap_or(0);
    let filename = url.split('/').last().unwrap_or("cuckoo-update").to_string();
    let temp_path = std::env::temp_dir().join(&filename);

    let mut file = match std::fs::File::create(&temp_path) {
        Ok(f) => f,
        Err(e) => { emit_err(format!("创建临时文件失败: {e}")); return; }
    };

    let mut downloaded: u64 = 0;
    let mut buf = vec![0u8; 65_536]; // 64 KB chunks
    loop {
        match resp.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if let Err(e) = file.write_all(&buf[..n]) {
                    emit_err(format!("写入失败: {e}"));
                    return;
                }
                downloaded += n as u64;
                let _ = app.emit(
                    "update-progress",
                    serde_json::json!({ "downloaded": downloaded, "total": total }),
                );
            }
            Err(e) => { emit_err(format!("读取失败: {e}")); return; }
        }
    }

    drop(file);

    // ── Open installer ────────────────────────────────────────────────────────
    #[cfg(target_os = "macos")]
    {
        if std::process::Command::new("open").arg(&temp_path).spawn().is_err() {
            emit_err("无法打开安装文件".to_string());
            return;
        }
    }

    #[cfg(target_os = "windows")]
    {
        if std::process::Command::new(&temp_path).spawn().is_err() {
            emit_err("无法运行安装程序".to_string());
            return;
        }
    }

    let _ = app.emit("update-complete", ());
}

fn is_newer(new: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.').filter_map(|p| p.parse().ok()).collect()
    };
    parse(new) > parse(current)
}
