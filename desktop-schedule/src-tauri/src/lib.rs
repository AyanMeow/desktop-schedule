use serde::Serialize;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State, WindowEvent,
};

mod config;
mod db;
mod weather;
mod window_state;
use window_state::{WindowState, WindowStateExt};

// 最小 Win32 FFI 声明（仅为抗"显示桌面"监控用 3 个函数，不引入 windows crate 直接依赖）
#[allow(non_snake_case)]
mod win32 {
    pub type HWND = *mut core::ffi::c_void;
    extern "system" {
        pub fn IsIconic(hWnd: HWND) -> i32;
        pub fn DwmGetWindowAttribute(
            hwnd: HWND,
            dwAttribute: u32,
            pvAttribute: *mut core::ffi::c_void,
            cbAttribute: u32,
        ) -> i32;
        pub fn DwmSetWindowAttribute(
            hwnd: HWND,
            dwAttribute: u32,
            pvAttribute: *const core::ffi::c_void,
            cbAttribute: u32,
        ) -> i32;
    }
}
const DWMWA_TRANSITIONS_FORCEDISABLED: u32 = 3;
const DWMWA_CLOAKED: u32 = 13;

/// 应用状态：数据库连接 + 配置文件路径 + 最近天气缓存
pub struct AppState {
    pub db: db::Db,
    pub config_path: std::path::PathBuf,
    pub last_weather: Option<weather::Weather>,
}

/// 启动时是否带 --autostart 参数（供前端查询）
pub struct IsAutostartFlag(pub bool);

/// 主贴片前端是否已完成首次加载（供控制面板查询，避免提前 show 暴露默认界面）
pub struct MainReady(pub AtomicBool);

// ============ 日程命令 ============

// 辅助：锁定 state 并取 db 连接 guard 的闭包包装，避免 MutexGuard 生命周期问题。
// 用法：with_db(&state, |conn| db::xxx(conn, ...))
fn with_db<T, F>(state: &State<'_, Mutex<AppState>>, f: F) -> Result<T, String>
where
    F: FnOnce(&rusqlite::Connection) -> Result<T, anyhow::Error>,
{
    let st = state.lock().map_err(|e| e.to_string())?;
    let conn = st.db.0.lock().map_err(|e| e.to_string())?;
    f(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_schedules(state: State<'_, Mutex<AppState>>, start: String, end: String) -> Result<Vec<db::Schedule>, String> {
    with_db(&state, |c| db::list_schedules_in_range(c, &start, &end))
}

// ============ 成就系统 ============

/// 成就条件类型
#[derive(Debug, Clone)]
enum AchKind {
    Cumulative(i64), // 累计完成 n 条
    Streak(i64),     // 最长连续 n 天
    DailyBurst(i64), // 单日完成 n 条
}

/// 成就定义（硬编码：新增/调整成就只改这里，老用户升级后自动参与判定）
struct AchDef {
    id: &'static str,
    title: &'static str,
    desc: &'static str,
    kind: AchKind,
}

impl AchDef {
    fn target(&self) -> i64 {
        match &self.kind {
            AchKind::Cumulative(n) | AchKind::Streak(n) | AchKind::DailyBurst(n) => *n,
        }
    }
    fn category(&self) -> &'static str {
        match &self.kind {
            AchKind::Cumulative(_) => "cumulative",
            AchKind::Streak(_) => "streak",
            AchKind::DailyBurst(_) => "daily",
        }
    }
}

/// 当前统计值对某条件的进度（封顶到目标值）
fn ach_progress(kind: &AchKind, s: &db::AchievementStats) -> i64 {
    match kind {
        AchKind::Cumulative(n) => s.total.min(*n),
        AchKind::Streak(n) => s.max_streak.min(*n),
        AchKind::DailyBurst(n) => s.max_daily.min(*n),
    }
}

/// 面板单项（前端隐藏成就展示由前端处理，后端返回完整数据）
#[derive(Debug, Clone, Serialize)]
struct AchievementItem {
    id: String,
    title: String,
    desc: String,
    category: String,
    target: i64,
    progress: i64,
    unlocked: bool,
    unlocked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct AchievementOverview {
    stats: db::AchievementStats,
    items: Vec<AchievementItem>,
}

/// 解锁事件 payload（Toast 用）
#[derive(Debug, Clone, Serialize)]
struct AchievementUnlockedPayload {
    achievements: Vec<AchievementItem>,
}

/// 全部成就（19 枚，艾露猫风格命名）
static ACHIEVEMENTS: &[AchDef] = &[
    // 累计完成（每 50 一档，上限 500）
    AchDef { id: "c50",  title: "初露锋芒", desc: "累计完成 50 条日程",  kind: AchKind::Cumulative(50) },
    AchDef { id: "c100", title: "小有名气", desc: "累计完成 100 条日程", kind: AchKind::Cumulative(100) },
    AchDef { id: "c150", title: "稳定发挥", desc: "累计完成 150 条日程", kind: AchKind::Cumulative(150) },
    AchDef { id: "c200", title: "部落常客", desc: "累计完成 200 条日程", kind: AchKind::Cumulative(200) },
    AchDef { id: "c250", title: "资深猎人", desc: "累计完成 250 条日程", kind: AchKind::Cumulative(250) },
    AchDef { id: "c300", title: "百战沙场", desc: "累计完成 300 条日程", kind: AchKind::Cumulative(300) },
    AchDef { id: "c350", title: "猎场老将", desc: "累计完成 350 条日程", kind: AchKind::Cumulative(350) },
    AchDef { id: "c400", title: "传奇猎手", desc: "累计完成 400 条日程", kind: AchKind::Cumulative(400) },
    AchDef { id: "c450", title: "王牌猎人", desc: "累计完成 450 条日程", kind: AchKind::Cumulative(450) },
    AchDef { id: "c500", title: "猫车之王", desc: "累计完成 500 条日程", kind: AchKind::Cumulative(500) },
    // 连续天数
    AchDef { id: "s3",   title: "连胜起步", desc: "连续 3 天有完成日程",   kind: AchKind::Streak(3) },
    AchDef { id: "s7",   title: "一周坚持", desc: "连续 7 天有完成日程",   kind: AchKind::Streak(7) },
    AchDef { id: "s14",  title: "半月不辍", desc: "连续 14 天有完成日程",  kind: AchKind::Streak(14) },
    AchDef { id: "s30",  title: "月度毅力", desc: "连续 30 天有完成日程",  kind: AchKind::Streak(30) },
    AchDef { id: "s100", title: "百日铸魂", desc: "连续 100 天有完成日程", kind: AchKind::Streak(100) },
    // 单日爆发（上限 20）
    AchDef { id: "d5",   title: "干劲十足", desc: "单日完成 5 条日程",  kind: AchKind::DailyBurst(5) },
    AchDef { id: "d10",  title: "效率惊人", desc: "单日完成 10 条日程", kind: AchKind::DailyBurst(10) },
    AchDef { id: "d15",  title: "火力全开", desc: "单日完成 15 条日程", kind: AchKind::DailyBurst(15) },
    AchDef { id: "d20",  title: "单日传说", desc: "单日完成 20 条日程", kind: AchKind::DailyBurst(20) },
];

fn ach_item(def: &AchDef, stats: &db::AchievementStats, unlocked_at: Option<String>) -> AchievementItem {
    let progress = ach_progress(&def.kind, stats);
    AchievementItem {
        id: def.id.to_string(),
        title: def.title.to_string(),
        desc: def.desc.to_string(),
        category: def.category().to_string(),
        target: def.target(),
        progress,
        unlocked: unlocked_at.is_some(),
        unlocked_at,
    }
}

/// 成就判定：解锁满足条件且未解锁的成就；notify=true 时广播事件（Toast）
/// 供 toggle_complete（notify=true）与启动回填（notify=false，老用户静默解锁）共用
fn check_unlocks(app: &tauri::AppHandle, state: &Mutex<AppState>, notify: bool) {
    let Ok(st) = state.lock() else { return };
    let Ok(conn) = st.db.0.lock() else { return };
    let Ok(stats) = db::achievement_stats(&conn) else { return };
    let Ok(unlocked) = db::unlocked_map(&conn) else { return };

    let mut newly: Vec<AchievementItem> = vec![];
    for def in ACHIEVEMENTS {
        if unlocked.contains_key(def.id) {
            continue;
        }
        if ach_progress(&def.kind, &stats) >= def.target() {
            if let Ok(at) = db::insert_unlock(&conn, def.id) {
                newly.push(ach_item(def, &stats, Some(at)));
            }
        }
    }
    if notify && !newly.is_empty() {
        let _ = app.emit(
            "achievement-unlocked",
            AchievementUnlockedPayload { achievements: newly },
        );
    }
}

#[tauri::command]
fn achievement_overview(
    state: State<'_, Mutex<AppState>>,
) -> Result<AchievementOverview, String> {
    with_db(&state, |c| {
        let stats = db::achievement_stats(c)?;
        let unlocked = db::unlocked_map(c)?;
        let items = ACHIEVEMENTS
            .iter()
            .map(|d| ach_item(d, &stats, unlocked.get(d.id).cloned()))
            .collect();
        Ok(AchievementOverview { stats, items })
    })
}

#[tauri::command]
fn create_schedule(state: State<'_, Mutex<AppState>>, input: db::NewSchedule) -> Result<Vec<i64>, String> {
    with_db(&state, |c| db::create_schedules(c, &input))
}

#[tauri::command]
fn toggle_complete(
    app: tauri::AppHandle,
    state: State<'_, Mutex<AppState>>,
    id: i64,
) -> Result<db::Schedule, String> {
    let (schedule, became_completed) = with_db(&state, |c| db::toggle_completed(c, id))?;
    // 若变为完成，抽取一句鼓励语，前端监听 encouragement 事件展示
    if became_completed {
        let enc = with_db(&state, |c| db::random_encouragement(c))?;
        let _ = app.emit("encouragement", enc);
        // 成就判定：新解锁的成就广播 achievement-unlocked 事件
        check_unlocks(&app, &state, true);
    }
    Ok(schedule)
}

#[tauri::command]
fn update_schedule(state: State<'_, Mutex<AppState>>, id: i64, update: db::UpdateSchedule) -> Result<db::Schedule, String> {
    with_db(&state, |c| db::update_schedule(c, id, update))
}

#[tauri::command]
fn delete_schedule(state: State<'_, Mutex<AppState>>, id: i64) -> Result<(), String> {
    with_db(&state, |c| db::delete_schedule(c, id))
}

#[tauri::command]
fn delete_group(state: State<'_, Mutex<AppState>>, group_id: String) -> Result<(), String> {
    with_db(&state, |c| db::delete_group(c, &group_id))
}

#[tauri::command]
fn list_encouragements(state: State<'_, Mutex<AppState>>) -> Result<Vec<db::Encouragement>, String> {
    with_db(&state, |c| db::list_encouragements(c))
}

#[tauri::command]
fn count_group(state: State<'_, Mutex<AppState>>, group_id: String) -> Result<i64, String> {
    with_db(&state, |c| db::count_by_group(c, &group_id))
}

/// 用系统默认程序打开日程关联的文件/文件夹
#[tauri::command]
async fn open_attachment(path: String) -> Result<(), String> {
    tauri_plugin_opener::open_path(path, None::<&str>).map_err(|e| e.to_string())
}

/// 手动拉取一次天气
#[tauri::command]
async fn fetch_weather(lat: f64, lon: f64, city: String) -> Result<weather::Weather, String> {
    weather::fetch(lat, lon, &city).await.map_err(|e| e.to_string())
}

/// 取最近一次缓存的天气（启动时用，避免空状态）
#[tauri::command]
fn get_last_weather(state: State<'_, Mutex<AppState>>) -> Option<weather::Weather> {
    let st = state.lock().ok()?;
    st.last_weather.clone()
}

// ============ 导入 / 导出 ============

/// 导出全部日程为 JSON 文本，写到用户选择的文件
#[tauri::command]
async fn export_schedules(
    state: State<'_, Mutex<AppState>>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    // 生成内容
    let content = {
        let st = state.lock().map_err(|e| e.to_string())?;
        let conn = st.db.0.lock().map_err(|e| e.to_string())?;
        db::export_all(&conn).map_err(|e| e.to_string())?
    };
    // 弹保存对话框（file() + blocking_save_file）
    let path = app
        .dialog()
        .file()
        .add_filter("文本文件", &["txt"])
        .set_file_name(format!(
            "日程备份-{}.txt",
            chrono::Local::now().format("%Y%m%d_%H%M%S")
        ))
        .blocking_save_file();
    let path = path.ok_or_else(|| "未选择保存位置".to_string())?;
    let path = path.into_path().map_err(|_| "路径无效".to_string())?;
    // 写文件
    std::fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

/// 从用户选择的 txt 文件导入日程
#[tauri::command]
async fn import_schedules(
    state: State<'_, Mutex<AppState>>,
    app: tauri::AppHandle,
) -> Result<usize, String> {
    use tauri_plugin_dialog::DialogExt;
    let path = app
        .dialog()
        .file()
        .add_filter("文本文件", &["txt"])
        .blocking_pick_file();
    let path = path.ok_or_else(|| "未选择文件".to_string())?;
    let path = path.into_path().map_err(|_| "路径无效".to_string())?;
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let count = {
        let st = state.lock().map_err(|e| e.to_string())?;
        let conn = st.db.0.lock().map_err(|e| e.to_string())?;
        db::import_all(&conn, &content).map_err(|e| e.to_string())?
    };
    Ok(count)
}

// ============ 配置命令 ============

#[tauri::command]
fn get_config(state: State<'_, Mutex<AppState>>) -> Result<config::AppConfig, String> {
    let st = state.lock().map_err(|e| e.to_string())?;
    config::load(&st.config_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_config(state: State<'_, Mutex<AppState>>, cfg: config::AppConfig) -> Result<(), String> {
    let st = state.lock().map_err(|e| e.to_string())?;
    config::save(&st.config_path, &cfg).map_err(|e| e.to_string())
}

/// 前端加载完成后调用，显示指定窗口
#[tauri::command]
fn show_window(app: tauri::AppHandle, label: String) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(&label) {
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 查询启动时是否带 --autostart 参数
#[tauri::command]
fn is_autostart_flag(flag: State<'_, IsAutostartFlag>) -> bool {
    flag.0
}

/// 主贴片前端完成首次加载后调用，标记就绪
#[tauri::command]
fn mark_main_ready(flag: State<'_, MainReady>) {
    flag.0.store(true, std::sync::atomic::Ordering::SeqCst);
}

/// 查询主贴片前端是否已完成首次加载
#[tauri::command]
fn is_main_ready(flag: State<'_, MainReady>) -> bool {
    flag.0.load(std::sync::atomic::Ordering::SeqCst)
}

// ============ 窗口命令（沿用 M0） ============

#[tauri::command]
fn toggle_lock(window: tauri::Window, state: State<'_, WindowState>) -> bool {
    let locked = state.toggle_locked();
    let _ = window.emit("lock-changed", locked);
    locked
}

#[tauri::command]
fn set_opacity(_window: tauri::Window, value: f64) -> f64 {
    // Tauri 2 无窗口级 set_alpha，透明度由前端 CSS opacity 控制
    value.clamp(0.1, 1.0)
}

#[tauri::command]
fn toggle_always_on_top(window: tauri::Window) -> Result<bool, String> {
    let cur = window.is_always_on_top().map_err(|e| e.to_string())?;
    window.set_always_on_top(!cur).map_err(|e| e.to_string())?;
    Ok(!cur)
}

// ===== 开机自启：直接操作注册表（绕过 autostart 插件的 os error 2 问题）=====
#[cfg(windows)]
const AUTOSTART_REGKEY: &str = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run";
#[cfg(windows)]
const AUTOSTART_NAME: &str = "DesktopSchedule";

#[tauri::command]
fn set_autostart(enabled: bool) -> Result<bool, String> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe_path = exe.display().to_string();
        if enabled {
            let cmd = format!("\"{}\" --autostart", exe_path);
            hkcu.open_subkey_with_flags(AUTOSTART_REGKEY, KEY_SET_VALUE)
                .map_err(|e| e.to_string())?
                .set_value(AUTOSTART_NAME, &cmd)
                .map_err(|e| e.to_string())?;
        } else {
            // 忽略"值不存在"错误
            let _ = hkcu
                .open_subkey_with_flags(AUTOSTART_REGKEY, KEY_SET_VALUE)
                .and_then(|k| k.delete_value(AUTOSTART_NAME));
        }
        Ok(enabled)
    }
    #[cfg(not(windows))]
    {
        let _ = enabled;
        Err("仅支持 Windows".into())
    }
}

#[tauri::command]
fn is_autostart_enabled() -> bool {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        // 读出注册表值，校验其指向的 exe 是否为当前进程
        let registered = hkcu
            .open_subkey_with_flags(AUTOSTART_REGKEY, KEY_READ)
            .and_then(|k| k.get_value::<String, _>(AUTOSTART_NAME));
        let stored = match registered {
            Ok(v) => v,
            Err(_) => return false, // 项不存在
        };
        // 注册表值格式: "C:\path\app.exe" --autostart
        // 提取引号内的 exe 路径（或无引号时取首个空白前的 token）
        let stored_exe = extract_exe_path(&stored);
        let current = match std::env::current_exe() {
            Ok(p) => p,
            Err(_) => return false,
        };
        // 路径归一化后比对（不区分大小写、统一分隔符）
        let stored_norm = stored_exe.to_lowercase().replace('/', "\\");
        let current_norm = current.display().to_string().to_lowercase().replace('/', "\\");
        stored_norm == current_norm
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// 从注册表值（如 `"C:\app.exe" --autostart`）提取 exe 路径
#[cfg(windows)]
fn extract_exe_path(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with('"') {
        // 带引号：取两个引号之间
        if let Some(end) = trimmed[1..].find('"') {
            return trimmed[1..1 + end].to_string();
        }
    }
    // 无引号：取首个空白前
    trimmed.split_whitespace().next().unwrap_or("").to_string()
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(WindowState::default())
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                match window.label() {
                    // 贴片窗口：永不关闭，拦截
                    "main" => {
                        api.prevent_close();
                    }
                    // 控制面板窗口：关闭=最小化到托盘（隐藏），但贴片保持显示
                    "taskbar" => {
                        let _ = window.hide();
                        api.prevent_close();
                    }
                    _ => {}
                }
            }
        })
        .setup(|app| {
            // 数据库与配置路径：放在 app_data_dir
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("无法获取 app_data_dir");
            std::fs::create_dir_all(&data_dir).ok();
            let db_path = data_dir.join("schedules.db");
            let config_path = data_dir.join("config.json");
            let database = db::open(&db_path).expect("数据库初始化失败");
            app.manage(Mutex::new(AppState {
                db: database,
                config_path,
                last_weather: None,
            }));

            // 成就回填：老用户按历史完成记录静默解锁已满足的成就（不弹通知）
            {
                let st = app.state::<Mutex<AppState>>();
                check_unlocks(app.handle(), &st, false);
            }

            // 启动天气定时刷新（每 30 分钟）
            weather::start_refresh_loop(app.handle().clone());

            // 窗口可见性：不在 Rust 端 show，由前端加载完成后调 show_window 命令
            // 这样用户看到的是加载完毕的正确界面，无默认值闪烁
            // 自启参数存在 AppState 供前端查询
            let is_autostart = std::env::args().any(|a| a == "--autostart");
            app.manage(IsAutostartFlag(is_autostart));
            app.manage(MainReady(AtomicBool::new(false)));

            // 抗"显示桌面"：Win+D / Win+M / 任务栏显示桌面按钮会把贴片 DWM 隐藏（cloak）
            // 或最小化。先禁用该窗口动画，再用 300ms 轮询检测并立即恢复。
            // 注意只检测 cloak/最小化，不检测 IsWindowVisible——托盘"隐藏贴片"用的
            // 是 hide()（可见性=false），不触发 cloak，因此不会跟用户主动隐藏打架。
            if let Some(main_win) = app.get_webview_window("main") {
                if let Ok(hwnd) = main_win.hwnd() {
                    unsafe {
                        let yes: i32 = 1;
                        let _ = win32::DwmSetWindowAttribute(
                            hwnd.0 as win32::HWND,
                            DWMWA_TRANSITIONS_FORCEDISABLED,
                            &yes as *const i32 as *const core::ffi::c_void,
                            4,
                        );
                    }
                }
            }
            let monitor_handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_millis(300));
                let Some(win) = monitor_handle.get_webview_window("main") else {
                    continue;
                };
                let Ok(hwnd) = win.hwnd() else { continue };
                if !win.is_visible().unwrap_or(false) {
                    continue; // 用户主动隐藏，不干预
                }
                unsafe {
                    let mut cloaked: u32 = 0;
                    let hr = win32::DwmGetWindowAttribute(
                        hwnd.0 as win32::HWND,
                        DWMWA_CLOAKED,
                        &mut cloaked as *mut u32 as *mut core::ffi::c_void,
                        4,
                    );
                    let iconic = win32::IsIconic(hwnd.0 as win32::HWND) != 0;
                    if iconic || (hr == 0 && cloaked != 0) {
                        if iconic {
                            let _ = win.unminimize();
                        }
                        // cloak 状态无法被 show() 直接解除，hide+show 强制 DWM 重算可见性
                        let _ = win.hide();
                        let _ = win.show();
                    }
                }
            });

            // 托盘菜单
            let panel = MenuItem::with_id(app, "panel", "控制面板", true, None::<&str>)?;
            let toggle_widget = MenuItem::with_id(app, "toggle_widget", "显示/隐藏贴片", true, None::<&str>)?;
            let top = MenuItem::with_id(app, "top", "切换置顶", true, None::<&str>)?;
            let sep = tauri::menu::PredefinedMenuItem::separator(app)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&panel, &toggle_widget, &top, &sep, &quit])?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("桌面日程")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "panel" => {
                        if let Some(win) = app.get_webview_window("taskbar") {
                            if win.is_visible().unwrap_or(false) {
                                let _ = win.set_focus();
                            } else {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                    }
                    "toggle_widget" => {
                        if let Some(win) = app.get_webview_window("main") {
                            if win.is_visible().unwrap_or(false) {
                                let _ = win.hide();
                            } else {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                    }
                    "top" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let cur = win.is_always_on_top().unwrap_or(false);
                            let _ = win.set_always_on_top(!cur);
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // 左键单击：切换控制面板窗口（贴片始终不受影响）
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("taskbar") {
                            if win.is_visible().unwrap_or(false) {
                                let _ = win.set_focus();
                            } else {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 日程
            list_schedules,
            create_schedule,
            toggle_complete,
            update_schedule,
            delete_schedule,
            delete_group,
            count_group,
            open_attachment,
            fetch_weather,
            get_last_weather,
            export_schedules,
            import_schedules,
            list_encouragements,
            achievement_overview,
            // 配置
            get_config,
            save_config,
            // 窗口
            show_window,
            is_autostart_flag,
            mark_main_ready,
            is_main_ready,
            toggle_lock,
            set_opacity,
            toggle_always_on_top,
            set_autostart,
            is_autostart_enabled
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
