use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 应用配置（持久化到 config.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub view: ViewConfig,
    #[serde(default)]
    pub startup: StartupConfig,
    #[serde(default)]
    pub ddl_colors: DdlColors,
    #[serde(default)]
    pub encouragement: EncouragementConfig,
    #[serde(default)]
    pub weather: WeatherConfig,
    #[serde(default)]
    pub update: UpdateConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_x")]
    pub x: i32,
    #[serde(default = "default_y")]
    pub y: i32,
    #[serde(default = "default_w")]
    pub width: u32,
    #[serde(default = "default_h")]
    pub height: u32,
    #[serde(default)]
    pub locked: bool,
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(default = "default_bg_mode")]
    pub bg_mode: String, // "dark" | "light" | "image"
    #[serde(default = "default_bg_value")]
    pub bg_value: String,
    #[serde(default = "default_font_size")]
    pub font_size: u32, // px
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default = "default_theme_name")]
    pub theme_name: String, // 配色预设名（slate/forest/ocean/...）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewConfig {
    #[serde(default = "default_range")]
    pub range: String, // "week" | "biweek" | "month"
    #[serde(default = "default_week_start")]
    pub week_start: String, // "monday" | "sunday"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupConfig {
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default = "default_delay")]
    pub delay_seconds: u64,
    #[serde(default)]
    pub expand_today_on_launch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdlColors {
    #[serde(default = "default_c_overdue")]
    pub overdue: String,
    #[serde(default = "default_c_le1")]
    pub le1: String,
    #[serde(default = "default_c_le3")]
    pub le3: String,
    #[serde(default = "default_c_le7")]
    pub le7: String,
    #[serde(default = "default_c_gt7")]
    pub gt7: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncouragementConfig {
    #[serde(default)]
    pub sound: bool,
    #[serde(default = "default_undo_window")]
    pub undo_window_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub city: String,      // 城市名（显示用）
    #[serde(default)]
    pub latitude: f64,     // 纬度
    #[serde(default)]
    pub longitude: f64,    // 经度
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    #[serde(default = "default_auto_check")]
    pub auto_check: bool,          // 每日自动检查
    #[serde(default = "default_proxy_mode")]
    pub proxy_mode: String,        // "auto" | "manual" | "direct"
    #[serde(default)]
    pub proxy: String,             // manual 模式下的代理地址（如 http://127.0.0.1:7899）
    #[serde(default)]
    pub last_check: String,        // 上次检查时间（ISO），防频繁重启重复查
    #[serde(default)]
    pub last_seen_version: String, // 用户已查看过公告的版本；与当前不一致时控制面板显示更新公告
    #[serde(default = "default_update_source")]
    pub source: String,           // 更新源："auto"（双源取新）| "gitee" | "github"
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            auto_check: true,
            proxy_mode: "auto".into(),
            proxy: String::new(),
            last_check: String::new(),
            last_seen_version: String::new(),
            source: "auto".into(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig::default(),
            view: ViewConfig::default(),
            startup: StartupConfig::default(),
            ddl_colors: DdlColors::default(),
            encouragement: EncouragementConfig::default(),
            weather: WeatherConfig::default(),
            update: UpdateConfig::default(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            x: default_x(),
            y: default_y(),
            width: default_w(),
            height: default_h(),
            locked: false,
            opacity: default_opacity(),
            always_on_top: false,
            bg_mode: default_bg_mode(),
            bg_value: default_bg_value(),
            font_size: default_font_size(),
            font_family: default_font_family(),
            theme_name: default_theme_name(),
        }
    }
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            range: default_range(),
            week_start: default_week_start(),
        }
    }
}

impl Default for StartupConfig {
    fn default() -> Self {
        Self {
            auto_start: false,
            delay_seconds: default_delay(),
            expand_today_on_launch: true,
        }
    }
}

impl Default for DdlColors {
    fn default() -> Self {
        Self {
            overdue: default_c_overdue(),
            le1: default_c_le1(),
            le3: default_c_le3(),
            le7: default_c_le7(),
            gt7: default_c_gt7(),
        }
    }
}

impl Default for EncouragementConfig {
    fn default() -> Self {
        Self {
            sound: false,
            undo_window_seconds: default_undo_window(),
        }
    }
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            city: "北京".into(),
            latitude: 39.9042,
            longitude: 116.4074,
        }
    }
}

// 默认值函数
fn default_x() -> i32 { 100 }
fn default_y() -> i32 { 100 }
fn default_w() -> u32 { 900 }
fn default_h() -> u32 { 675 }
fn default_opacity() -> f64 { 0.9 }
fn default_bg_mode() -> String { "dark".into() }
fn default_bg_value() -> String { "#2b2d3a".into() }
fn default_font_size() -> u32 { 15 }
fn default_font_family() -> String { "system-ui".into() }
fn default_theme_name() -> String { "slate".into() }
fn default_range() -> String { "week".into() }
fn default_week_start() -> String { "monday".into() }
fn default_delay() -> u64 { 5 }
fn default_undo_window() -> u64 { 5 }
fn default_auto_check() -> bool { true }
fn default_proxy_mode() -> String { "auto".into() }
fn default_update_source() -> String { "auto".into() }
fn default_c_overdue() -> String { "#c0392b".into() }
fn default_c_le1() -> String { "#e74c3c".into() }
fn default_c_le3() -> String { "#e67e22".into() }
fn default_c_le7() -> String { "#f1c40f".into() }
fn default_c_gt7() -> String { "#95a5a6".into() }

/// 读取配置；文件不存在则返回默认值并写盘
pub fn load(path: &PathBuf) -> anyhow::Result<AppConfig> {
    if !path.exists() {
        let cfg = AppConfig::default();
        let _ = save(path, &cfg);
        return Ok(cfg);
    }
    let text = std::fs::read_to_string(path).context("读取 config.json 失败")?;
    let cfg: AppConfig = serde_json::from_str(&text).context("解析 config.json 失败")?;
    Ok(cfg)
}

/// 写入配置
pub fn save(path: &PathBuf, cfg: &AppConfig) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let text = serde_json::to_string_pretty(cfg)?;
    std::fs::write(path, text)?;
    Ok(())
}
