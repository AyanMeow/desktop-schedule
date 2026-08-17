// 与 Rust 端 serde 结构对应的前端类型

export interface Schedule {
  id: number;
  group_id: string | null;
  title: string;
  date: string; // 'YYYY-MM-DD'
  time_of_day: string | null; // 'HH:MM'
  note: string | null;
  priority: number;
  has_ddl: boolean;
  ddl_at: string | null; // 'YYYY-MM-DD HH:MM' 或 'YYYY-MM-DD'
  completed: boolean;
  completed_at: string | null;
  attachment: string | null; // 关联的文件/文件夹路径
  created_at: string;
  updated_at: string;
}

export interface NewSchedule {
  title: string;
  start_date: string;
  end_date: string;
  time_of_day: string | null;
  note: string | null;
  priority?: number;
  ddl_at: string | null;
  attachment: string | null;
}

export interface UpdateSchedule {
  title?: string;
  date?: string;
  time_of_day?: string | null;
  note?: string | null;
  priority?: number;
  ddl_at?: string | null;
  attachment?: string | null;
}

export interface Encouragement {
  id: number;
  text: string;
  category: string | null;
  enabled: boolean;
}

export interface AppConfig {
  window: WindowConfig;
  view: ViewConfig;
  startup: StartupConfig;
  ddl_colors: DdlColors;
  encouragement: EncouragementConfig;
  weather: WeatherConfig;
}

export interface WindowConfig {
  x: number;
  y: number;
  width: number;
  height: number;
  locked: boolean;
  opacity: number;
  always_on_top: boolean;
  bg_mode: string; // 'dark' | 'light' | 'image'
  bg_value: string;
  font_size: number; // px
  font_family: string;
  theme_name: string; // 配色预设名
}

export interface ViewConfig {
  range: string; // 'week' | 'biweek' | 'month'
  week_start: string; // 'monday' | 'sunday'
}

export interface StartupConfig {
  auto_start: boolean;
  delay_seconds: number;
  expand_today_on_launch: boolean;
}

export interface DdlColors {
  overdue: string;
  le1: string;
  le3: string;
  le7: string;
  gt7: string;
}

export interface EncouragementConfig {
  sound: boolean;
  undo_window_seconds: number;
}

export interface WeatherConfig {
  enabled: boolean;
  city: string;
  latitude: number;
  longitude: number;
}

export interface DailyWeather {
  date: string;
  weather_code: number;
  temp_max: number;
  temp_min: number;
  icon: string;
  description: string;
}

export interface Weather {
  temperature: number;
  weather_code: number;
  description: string;
  icon: string;
  city: string;
  updated_at: string;
  daily?: DailyWeather[];
}

export type ViewRange = 'week' | 'biweek' | 'month';

// ============ 成就系统 ============

export interface AchievementStats {
  total: number;          // 累计完成条数（每条日程一生只计一次）
  max_daily: number;      // 单日最大完成条数
  max_streak: number;     // 历史最长连续天数
  current_streak: number; // 当前连续天数
}

export interface AchievementItem {
  id: string;
  title: string;
  desc: string;
  category: string; // 'cumulative' | 'streak' | 'daily'
  target: number;
  progress: number;
  unlocked: boolean;
  unlocked_at: string | null;
}

export interface AchievementOverview {
  stats: AchievementStats;
  items: AchievementItem[];
}

/** achievement-unlocked 事件 payload（Toast 用） */
export interface AchievementUnlockedPayload {
  achievements: AchievementItem[];
}
