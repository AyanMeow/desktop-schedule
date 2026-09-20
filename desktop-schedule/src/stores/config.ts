import { defineStore } from 'pinia';
import { ref } from 'vue';
import { api } from '../api';
import type { AppConfig } from '../types';

const defaultConfig: AppConfig = {
  window: {
    x: 100, y: 100, width: 900, height: 675,
    locked: false, opacity: 0.9, always_on_top: false,
    bg_mode: 'dark', bg_value: '#2b2d3a',
    font_size: 15, font_family: "system-ui, 'Microsoft YaHei', sans-serif",
    theme_name: 'slate',
    mouse_passthrough: false,
  },
  view: { range: 'week', week_start: 'monday' },
  startup: { auto_start: false, delay_seconds: 5, expand_today_on_launch: true },
  ddl_colors: {
    overdue: '#c0392b', le1: '#e74c3c', le3: '#e67e22',
    le7: '#f1c40f', gt7: '#95a5a6',
  },
  encouragement: { sound: false, undo_window_seconds: 5 },
  weather: { enabled: false, city: '北京', latitude: 39.9042, longitude: 116.4074 },
  update: { auto_check: true, proxy_mode: 'auto', proxy: '', last_check: '', last_seen_version: '', source: 'auto' },
};

export const useConfigStore = defineStore('config', () => {
  const config = ref<AppConfig>(JSON.parse(JSON.stringify(defaultConfig)));
  const loaded = ref(false);

  async function load() {
    // 启动竞态防护：后端 setup（建库/迁移/回填）可能尚未 app.manage(AppState)，
    // 此时 get_config 报 "state not managed"——重试等待（100ms×50=最多5秒），
    // 超时才回退默认值。避免"更新后首次启动配置短暂失忆"。
    let lastErr: unknown = null;
    for (let i = 0; i < 50; i++) {
      try {
        config.value = await api.getConfig();
        loaded.value = true;
        return;
      } catch (e) {
        lastErr = e;
        await new Promise((r) => setTimeout(r, 100));
      }
    }
    void api.appendLog(`配置加载失败（已重试5秒），回退默认值：${String(lastErr)}`);
    config.value = JSON.parse(JSON.stringify(defaultConfig));
    loaded.value = true;
  }

  async function save() {
    // 守卫：load 完成前禁止保存，避免把内存默认值覆盖用户配置
    if (!loaded.value) return;
    await api.saveConfig(config.value);
  }

  /** 更新局部配置并立即持久化 */
  async function patch(partial: Partial<AppConfig>) {
    config.value = { ...config.value, ...partial };
    await save();
  }

  return { config, loaded, load, save, patch };
});
