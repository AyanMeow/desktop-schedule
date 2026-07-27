// 封装所有 Tauri IPC 调用
import { invoke } from '@tauri-apps/api/core';
import type {
  Schedule,
  NewSchedule,
  UpdateSchedule,
  Encouragement,
  AppConfig,
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

  listEncouragements: () =>
    invoke<Encouragement[]>('list_encouragements'),

  // 配置
  getConfig: () => invoke<AppConfig>('get_config'),
  saveConfig: (cfg: AppConfig) => invoke<void>('save_config', { cfg }),

  // 窗口
  toggleLock: () => invoke<boolean>('toggle_lock'),
  toggleAlwaysOnTop: () => invoke<boolean>('toggle_always_on_top'),
  setAutostart: (enabled: boolean) => invoke<boolean>('set_autostart', { enabled }),
  isAutostartEnabled: () => invoke<boolean>('is_autostart_enabled'),
};
