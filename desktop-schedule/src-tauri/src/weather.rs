// 天气模块：Open-Meteo 数据源 + WMO 编码映射 + 定时刷新
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// 前端展示用的天气数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weather {
    pub temperature: f64,      // 摄氏度
    pub weather_code: i32,     // WMO 编码
    pub description: String,   // 中文描述（如"晴"）
    pub icon: String,          // 图标名（映射到 Icon.vue）
    pub city: String,
    pub updated_at: String,    // 更新时间
}

/// Open-Meteo API 返回结构（只取需要的字段）
#[derive(Debug, Deserialize)]
struct OwmResp {
    current: OwmCurrent,
}
#[derive(Debug, Deserialize)]
struct OwmCurrent {
    temperature_2m: f64,
    weather_code: i32,
}

/// 调用 Open-Meteo 拉取一次天气
pub async fn fetch(lat: f64, lon: f64, city: &str) -> anyhow::Result<Weather> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current=temperature_2m,weather_code"
    );
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()?;
    let resp: OwmResp = client.get(&url).send().await?.json().await?;
    let (desc, icon) = wmo_to_desc(resp.current.weather_code);
    Ok(Weather {
        temperature: resp.current.temperature_2m,
        weather_code: resp.current.weather_code,
        description: desc.into(),
        icon: icon.into(),
        city: city.into(),
        updated_at: chrono::Local::now().format("%H:%M").to_string(),
    })
}

/// WMO 标准天气编码 → 中文描述 + 图标名
/// 参考 https://open-meteo.com/en/docs#weathervariables
fn wmo_to_desc(code: i32) -> (&'static str, &'static str) {
    match code {
        0 => ("晴", "sun"),
        1 => ("大部晴", "sun"),
        2 => ("多云", "cloud"),
        3 => ("阴", "cloud"),
        45 | 48 => ("雾", "cloud"),
        51 | 53 | 55 => ("毛毛雨", "rain"),
        56 | 57 => ("冻毛毛雨", "rain"),
        61 | 63 | 65 => ("雨", "rain"),
        66 | 67 => ("冻雨", "rain"),
        71 | 73 | 75 => ("雪", "snow"),
        77 => ("米雪", "snow"),
        80 | 81 | 82 => ("阵雨", "rain"),
        85 | 86 => ("阵雪", "snow"),
        95 => ("雷阵雨", "rain"),
        96 | 99 => ("雷阵雨伴冰雹", "rain"),
        _ => ("未知", "cloud"),
    }
}

/// 启动后台定时任务：每 30 分钟刷新一次，结果通过 weather-updated 事件广播给前端
pub fn start_refresh_loop(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            // 读配置
            let cfg = {
                let state = app.try_state::<std::sync::Mutex<crate::AppState>>();
                match state {
                    Some(s) => {
                        let st = s.lock().unwrap();
                        crate::config::load(&st.config_path).ok()
                    }
                    None => None,
                }
            };
            if let Some(cfg) = cfg {
                if cfg.weather.enabled {
                    match fetch(cfg.weather.latitude, cfg.weather.longitude, &cfg.weather.city).await {
                        Ok(w) => {
                            let _ = app.emit("weather-updated", Some(&w));
                            // 缓存最近一次成功结果到 AppState，供前端首次加载用
                            if let Some(state) = app.try_state::<std::sync::Mutex<crate::AppState>>() {
                                let mut st = state.lock().unwrap();
                                st.last_weather = Some(w);
                            }
                        }
                        Err(_) => {
                            let _ = app.emit::<Option<Weather>>("weather-updated", None);
                        }
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(30 * 60)).await;
        }
    });
}
