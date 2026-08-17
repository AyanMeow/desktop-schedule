// 封装所有 Tauri IPC 调用
import { invoke } from '@tauri-apps/api/core';
import type {
  Schedule,
  NewSchedule,
  UpdateSchedule,
  Encouragement,
  AppConfig,
  Weather,
  AchievementOverview,
  UpdateInfo,
  WhatsNew,
} from './types';

export const api = {
  // 日程
  listSchedules: (start: string, end: string) =>
    invoke<Schedule[]>('list_schedules', { start, end }),

  createSchedule: (input: NewSchedule) =>
    invoke<number[]>('create_schedule', { input }),

  toggleComplete: (id: number) =>
    invoke<Schedule>('toggle_complete', { id }),

  updateSchedule: (id: number, update: UpdateSchedule) =>
    invoke<Schedule>('update_schedule', { id, update }),

  deleteSchedule: (id: number) =>
    invoke<void>('delete_schedule', { id }),

  deleteGroup: (groupId: string) =>
    invoke<void>('delete_group', { groupId }),

  countGroup: (groupId: string) =>
    invoke<number>('count_group', { groupId }),

  openAttachment: (path: string) =>
    invoke<void>('open_attachment', { path }),

  fetchWeather: (lat: number, lon: number, city: string) =>
    invoke<Weather>('fetch_weather', { lat, lon, city }),

  getLastWeather: () => invoke<Weather | null>('get_last_weather'),

  listEncouragements: () =>
    invoke<Encouragement[]>('list_encouragements'),

  // 成就
  achievementOverview: () =>
    invoke<AchievementOverview>('achievement_overview'),

  // 自动更新
  checkUpdate: () => invoke<UpdateInfo>('check_update'),
  downloadUpdate: () => invoke<void>('download_update'),
  applyUpdate: () => invoke<void>('apply_update'),
  detectUpdateProxy: () => invoke<string | null>('detect_update_proxy'),
  getAppVersion: () => invoke<string>('get_app_version'),
  getWhatsNew: () => invoke<WhatsNew | null>('get_whats_new'),
  markVersionSeen: () => invoke<void>('mark_version_seen'),

  // 配置
  getConfig: () => invoke<AppConfig>('get_config'),
  saveConfig: (cfg: AppConfig) => invoke<void>('save_config', { cfg }),

  // 窗口
  showWindow: (label: string) => invoke<void>('show_window', { label }),
  isAutostartFlag: () => invoke<boolean>('is_autostart_flag'),
  markMainReady: () => invoke<void>('mark_main_ready'),
  isMainReady: () => invoke<boolean>('is_main_ready'),
  toggleLock: () => invoke<boolean>('toggle_lock'),
  toggleAlwaysOnTop: () => invoke<boolean>('toggle_always_on_top'),
  setAutostart: (enabled: boolean) => invoke<boolean>('set_autostart', { enabled }),
  isAutostartEnabled: () => invoke<boolean>('is_autostart_enabled'),

  // 导入 / 导出
  exportSchedules: () => invoke<string>('export_schedules'),
  importSchedules: () => invoke<number>('import_schedules'),
};
