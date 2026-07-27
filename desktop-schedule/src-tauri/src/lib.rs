use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State, WindowEvent,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

mod config;
mod db;
mod window_state;
use window_state::{WindowState, WindowStateExt};

/// 应用状态：数据库连接 + 配置文件路径
pub struct AppState {
    pub db: db::Db,
    pub config_path: std::path::PathBuf,
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

#[tauri::command]
fn set_autostart(app: tauri::AppHandle, enabled: bool) -> Result<bool, String> {
    let mgr = app.autolaunch();
    if enabled {
        mgr.enable().map_err(|e| e.to_string())?;
    } else {
        mgr.disable().map_err(|e| e.to_string())?;
    }
    Ok(enabled)
}

#[tauri::command]
fn is_autostart_enabled(app: tauri::AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .manage(WindowState::default())
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // 关闭按钮改为隐藏到托盘
                let _ = window.hide();
                api.prevent_close();
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
            }));

            // 托盘菜单
            let show = MenuItem::with_id(app, "show", "显示/隐藏", true, None::<&str>)?;
            let top = MenuItem::with_id(app, "top", "切换置顶", true, None::<&str>)?;
            let sep = tauri::menu::PredefinedMenuItem::separator(app)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &top, &sep, &quit])?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("桌面日程")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
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
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            if win.is_visible().unwrap_or(false) {
                                let _ = win.hide();
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
