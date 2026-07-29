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

/// 应用状态：数据库连接 + 配置文件路径 + 最近天气缓存
pub struct AppState {
    pub db: db::Db,
    pub config_path: std::path::PathBuf,
    pub last_weather: Option<weather::Weather>,
}

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
        hkcu.open_subkey_with_flags(AUTOSTART_REGKEY, KEY_READ)
            .and_then(|k| k.get_value::<String, _>(AUTOSTART_NAME))
            .is_ok()
    }
    #[cfg(not(windows))]
    {
        false
    }
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

            // 启动天气定时刷新（每 30 分钟）
            weather::start_refresh_loop(app.handle().clone());

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
            // 配置
            get_config,
            save_config,
            // 窗口
            toggle_lock,
            set_opacity,
            toggle_always_on_top,
            set_autostart,
            is_autostart_enabled
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
