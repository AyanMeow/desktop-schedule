import { defineStore } from 'pinia';
import { ref } from 'vue';
import { api } from '../api';
import type { AppConfig } from '../types';

const defaultConfig: AppConfig = {
  window: {
    x: 100, y: 100, width: 600, height: 450,
    locked: false, opacity: 0.9, always_on_top: false,
    bg_mode: 'dark', bg_value: '#2b2d3a',
    font_size: 15, font_family: "system-ui, 'Microsoft YaHei', sans-serif",
  },
  view: { range: 'week', week_start: 'monday' },
  startup: { auto_start: false, delay_seconds: 5, expand_today_on_launch: true },
  ddl_colors: {
    overdue: '#c0392b', le1: '#e74c3c', le3: '#e67e22',
    le7: '#f1c40f', gt7: '#95a5a6',
  },
  encouragement: { sound: false, undo_window_seconds: 5 },
  weather: { enabled: false, city: '北京', latitude: 39.9042, longitude: 116.4074 },
};

export const useConfigStore = defineStore('config', () => {
  const config = ref<AppConfig>(JSON.parse(JSON.stringify(defaultConfig)));
  const loaded = ref(false);

  async function load() {
    try {
      config.value = await api.getConfig();
    } catch {
      config.value = JSON.parse(JSON.stringify(defaultConfig));
    }
    loaded.value = true;
  }

  async function save() {
    await api.saveConfig(config.value);
  }

  /** 更新局部配置并立即持久化 */
  async function patch(partial: Partial<AppConfig>) {
    config.value = { ...config.value, ...partial };
    await save();
  }

  return { config, loaded, load, save, patch };
});
