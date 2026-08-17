// 自动更新：检查 GitHub Releases → 下载（SHA-256 校验）→ 覆盖式换壳
// 网络与文件操作均为阻塞实现，调用方须放入 spawn_blocking。
use anyhow::{Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Duration;
use tauri::{Emitter, Manager};

const RELEASE_API: &str = "https://api.github.com/repos/AyanMeow/desktop-schedule/releases/latest";
const EXE_NAME: &str = "desktop-schedule.exe";
/// GitHub API 要求所有请求带 User-Agent
const UA: &str = concat!("desktop-schedule-updater/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub has_update: bool,
    pub current: String,
    pub latest: String,
    pub notes: String,
    pub asset_url: String,
    pub asset_size: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateProgress {
    pub downloaded: u64,
    pub total: u64,
    pub percent: u32,
}

pub fn current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ============ 代理（三档） ============

/// 按配置解析应使用的代理地址（None = 直连）
fn resolve_proxy(mode: &str, manual: &str) -> Option<String> {
    match mode {
        "direct" => None,
        "manual" => {
            let m = manual.trim();
            if m.is_empty() {
                None
            } else {
                Some(normalize_proxy(m))
            }
        }
        _ => detect_proxy(), // auto
    }
}

fn normalize_proxy(addr: &str) -> String {
    if addr.starts_with("http://") || addr.starts_with("https://") || addr.starts_with("socks5://") {
        addr.to_string()
    } else {
        format!("http://{addr}")
    }
}

/// 自动检测本地代理：系统代理（注册表）→ 常见端口探测验证 → None（直连）
pub fn detect_proxy() -> Option<String> {
    if let Some(p) = system_proxy() {
        return Some(p);
    }
    const PORTS: [u16; 7] = [7890, 7897, 7899, 10809, 1080, 8888, 2080];
    for port in PORTS {
        if port_listening(port) {
            let addr = format!("http://127.0.0.1:{port}");
            if proxy_works(&addr) {
                return Some(addr);
            }
        }
    }
    None
}

/// Windows 系统代理（HKCU Internet Settings）
fn system_proxy() -> Option<String> {
    use winreg::enums::*;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Internet Settings")
        .ok()?;
    let enabled: u32 = key.get_value("ProxyEnable").ok()?;
    if enabled != 1 {
        return None;
    }
    let server: String = key.get_value("ProxyServer").ok()?;
    if server.is_empty() {
        return None;
    }
    // 两种格式："127.0.0.1:7899" 或 "http=...;https=..."
    let addr = if server.contains('=') {
        server
            .split(';')
            .find(|s| s.starts_with("https=") || s.starts_with("http="))
            .and_then(|s| s.split('=').nth(1))
            .unwrap_or("")
    } else {
        server.as_str()
    };
    if addr.is_empty() {
        return None;
    }
    let p = normalize_proxy(addr);
    if proxy_works(&p) {
        Some(p)
    } else {
        None
    }
}

fn port_listening(port: u16) -> bool {
    let addr: std::net::SocketAddr = match format!("127.0.0.1:{port}").parse() {
        Ok(a) => a,
        Err(_) => return false,
    };
    std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok()
}

/// 验证代理真实可转发（对 GitHub API 发一次请求；403 限流也算通）
fn proxy_works(proxy: &str) -> bool {
    let client = reqwest::blocking::Client::builder()
        .proxy(match reqwest::Proxy::all(proxy) {
            Ok(p) => p,
            Err(_) => return false,
        })
        .timeout(Duration::from_secs(6))
        .build();
    let client = match client {
        Ok(c) => c,
        Err(_) => return false,
    };
    match client.head("https://api.github.com").send() {
        Ok(r) => r.status().as_u16() < 500,
        Err(_) => false,
    }
}

// ============ 检查 ============

#[derive(serde::Deserialize)]
struct Release {
    tag_name: String,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    assets: Vec<Asset>,
}

#[derive(serde::Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: i64,
}

fn build_client(proxy: Option<&str>) -> Result<reqwest::blocking::Client> {
    let mut b = reqwest::blocking::Client::builder()
        .user_agent(UA)
        .timeout(Duration::from_secs(15));
    if let Some(p) = proxy {
        b = b.proxy(reqwest::Proxy::all(p)?);
    }
    Ok(b.build()?)
}

fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let s = s.trim().trim_start_matches('v');
    let mut it = s.split('.');
    let a = it.next()?.parse().ok()?;
    let b = it.next()?.parse().ok()?;
    let c = it.next().unwrap_or("0").parse().ok()?;
    Some((a, b, c))
}

/// 单次检查（不重试）
pub fn check_once(mode: &str, manual: &str) -> Result<UpdateInfo> {
    let proxy = resolve_proxy(mode, manual);
    let client = build_client(proxy.as_deref())?;
    let resp = client.get(RELEASE_API).send()?.error_for_status()?;
    let rel: Release = resp.json()?;
    let cur = current_version();
    let latest = rel.tag_name.trim().trim_start_matches('v').to_string();
    let has = match (parse_version(&cur), parse_version(&latest)) {
        (Some(a), Some(b)) => b > a,
        _ => false,
    };
    let asset = rel.assets.iter().find(|a| a.name == EXE_NAME);
    Ok(UpdateInfo {
        has_update: has,
        current: cur,
        latest,
        notes: rel.body.unwrap_or_default(),
        asset_url: asset.map(|a| a.browser_download_url.clone()).unwrap_or_default(),
        asset_size: asset.map(|a| a.size).unwrap_or(0),
    })
}

/// 带重试的检查
pub fn check_with_retry(
    mode: &str,
    manual: &str,
    attempts: u32,
    interval: Duration,
) -> Result<UpdateInfo> {
    let mut last_err: Option<anyhow::Error> = None;
    for i in 0..attempts {
        match check_once(mode, manual) {
            Ok(info) => return Ok(info),
            Err(e) => {
                last_err = Some(e);
                if i + 1 < attempts {
                    std::thread::sleep(interval);
                }
            }
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("未知错误")))
}

// ============ 下载 + 校验 ============

/// 从 Release notes 解析 "SHA256: <hash>" 行
fn parse_sha256(notes: &str) -> Option<String> {
    notes
        .lines()
        .find_map(|l| l.trim().strip_prefix("SHA256:"))
        .map(|h| h.trim().to_ascii_lowercase())
}

fn sha256_file(path: &PathBuf) -> Result<String> {
    let mut f = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// 流式下载（进度事件）+ 完整性校验，返回下载好的 exe 路径
pub fn download_blocking(
    app: &tauri::AppHandle,
    mode: &str,
    manual: &str,
    info: &UpdateInfo,
    updates_dir: &PathBuf,
) -> Result<PathBuf> {
    std::fs::create_dir_all(updates_dir)?;
    let dest = updates_dir.join(EXE_NAME);
    let part = updates_dir.join("download.part");

    let proxy = resolve_proxy(mode, manual);
    let client = build_client(proxy.as_deref())?;
    let mut resp = client.get(&info.asset_url).send()?.error_for_status()?;
    let total = resp
        .content_length()
        .map(|l| l as u64)
        .unwrap_or(info.asset_size.max(0) as u64);

    let mut file = std::fs::File::create(&part)?;
    let mut downloaded: u64 = 0;
    let mut buf = [0u8; 65536];
    let mut last_pct = u32::MAX;
    loop {
        let n = resp.read(&mut buf)?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        downloaded += n as u64;
        let pct = if total > 0 {
            (downloaded * 100 / total) as u32
        } else {
            0
        };
        if pct != last_pct {
            last_pct = pct;
            let _ = app.emit(
                "update-progress",
                UpdateProgress {
                    downloaded,
                    total,
                    percent: pct,
                },
            );
        }
    }
    drop(file);

    // 完整性：优先 SHA-256（notes 提供），退回大小核对
    if let Some(expect) = parse_sha256(&info.notes) {
        let got = sha256_file(&part)?;
        if got != expect {
            let _ = std::fs::remove_file(&part);
            anyhow::bail!("SHA-256 校验失败：期望 {expect}，实际 {got}");
        }
    } else if total > 0 && downloaded != total {
        let _ = std::fs::remove_file(&part);
        anyhow::bail!("下载不完整（{downloaded}/{total} 字节）");
    }

    std::fs::rename(&part, &dest)?;
    Ok(dest)
}

/// 检查并下载（供命令/定时循环复用，内含下载重试），成功后 emit update-ready
pub fn check_and_download(
    app: &tauri::AppHandle,
    mode: &str,
    manual: &str,
    updates_dir: &PathBuf,
) -> Result<UpdateInfo> {
    let info = check_with_retry(mode, manual, 3, Duration::from_secs(5))?;
    if !info.has_update || info.asset_url.is_empty() {
        anyhow::bail!("当前已是最新版本");
    }
    let mut last: Option<anyhow::Error> = None;
    for i in 0..3 {
        // 每次重试用最新 info（asset_url 可能因 Release 更新而变化）
        let fresh = check_once(mode, manual).unwrap_or_else(|_| info.clone());
        match download_blocking(app, mode, manual, &fresh, updates_dir) {
            Ok(_) => {
                let _ = app.emit("update-ready", &fresh);
                return Ok(fresh);
            }
            Err(e) => {
                last = Some(e);
                if i + 1 < 3 {
                    std::thread::sleep(Duration::from_secs(10));
                }
            }
        }
    }
    Err(last.unwrap_or_else(|| anyhow::anyhow!("下载失败")))
}

// ============ 应用（覆盖式换壳） ============

/// 用下载好的 exe 替换当前程序并重启
pub fn apply_update(app: &tauri::AppHandle, downloaded: &PathBuf) -> Result<()> {
    let cur = std::env::current_exe().context("获取当前程序路径失败")?;
    let old = cur.with_extension("exe.old");
    let _ = std::fs::remove_file(&old); // 清理上次残留
    std::fs::rename(&cur, &old).context("重命名当前程序失败")?;
    if let Err(e) = std::fs::copy(downloaded, &cur) {
        let _ = std::fs::rename(&old, &cur); // 回滚
        return Err(anyhow::anyhow!("写入新版本失败：{e}（已回滚）"));
    }
    std::process::Command::new(&cur)
        .spawn()
        .context("启动新版本失败")?;
    app.exit(0);
    Ok(())
}

/// 启动时清理换壳残留（.old 与半成品）
pub fn cleanup_leftovers() {
    if let Ok(cur) = std::env::current_exe() {
        let _ = std::fs::remove_file(cur.with_extension("exe.old"));
    }
}

// ============ 定时循环（每 24h） ============

fn should_check(last: &str) -> bool {
    if last.is_empty() {
        return true;
    }
    match chrono::NaiveDateTime::parse_from_str(last, "%Y-%m-%d %H:%M:%S") {
        Ok(t) => chrono::Local::now().naive_local() - t > chrono::Duration::hours(20),
        Err(_) => true,
    }
}

fn mark_last_check(path: &std::path::Path) {
    if let Ok(mut cfg) = crate::config::load(&path.to_path_buf()) {
        cfg.update.last_check =
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let _ = crate::config::save(&path.to_path_buf(), &cfg);
    }
}

/// 启动后台定时检查：60s 后首查（距上次 <20h 跳过），此后每 24h
pub fn start_check_loop(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(60)).await;
        loop {
            let parsed = {
                let state = app.try_state::<std::sync::Mutex<crate::AppState>>();
                match state {
                    Some(s) => {
                        let st = s.lock().unwrap();
                        crate::config::load(&st.config_path).ok().map(|cfg| {
                            (
                                cfg.update.proxy_mode.clone(),
                                cfg.update.proxy.clone(),
                                st.config_path.clone(),
                                cfg.update.auto_check,
                                cfg.update.last_check.clone(),
                            )
                        })
                    }
                    None => None,
                }
            };

            if let Some((mode, manual, cfg_path, enabled, last)) = parsed {
                if enabled && should_check(&last) {
                    let updates_dir = app
                        .path()
                        .app_data_dir()
                        .unwrap_or_default()
                        .join("updates");

                    // 检查（重试 5 次间隔 30s）
                    let mode2 = mode.clone();
                    let manual2 = manual.clone();
                    let res = tauri::async_runtime::spawn_blocking(move || {
                        check_with_retry(&mode2, &manual2, 5, Duration::from_secs(30))
                    })
                    .await;
                    mark_last_check(&cfg_path);

                    if let Ok(Ok(info)) = res {
                        if info.has_update && !info.asset_url.is_empty() {
                            let _ = app.emit("update-available", &info);
                            // 自动下载（重试 3 次），完成后 emit update-ready
                            let app3 = app.clone();
                            let mode3 = mode.clone();
                            let manual3 = manual.clone();
                            let info3 = info.clone();
                            let dir3 = updates_dir.clone();
                            let _ = tauri::async_runtime::spawn_blocking(move || {
                                let mut last_err: Option<anyhow::Error> = None;
                                for i in 0..3 {
                                    let fresh = check_once(&mode3, &manual3)
                                        .unwrap_or_else(|_| info3.clone());
                                    match download_blocking(&app3, &mode3, &manual3, &fresh, &dir3)
                                    {
                                        Ok(_) => {
                                            let _ = app3.emit("update-ready", &fresh);
                                            return Ok(());
                                        }
                                        Err(e) => {
                                            last_err = Some(e);
                                            if i + 1 < 3 {
                                                std::thread::sleep(Duration::from_secs(10));
                                            }
                                        }
                                    }
                                }
                                Err(last_err.unwrap_or_else(|| anyhow::anyhow!("下载失败")))
                            })
                            .await;
                        }
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(24 * 3600)).await;
        }
    });
}
