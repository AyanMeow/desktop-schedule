import { defineStore } from 'pinia';
import { ref } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { api } from '../api';
import { useConfigStore } from './config';
import type { Weather, DailyWeather } from '../types';

export const useWeatherStore = defineStore('weather', () => {
  const weather = ref<Weather | null>(null);
  const loading = ref(false);
  let unlisten: UnlistenFn | undefined;
  let inited = false;

  // 按日期查预报（用于日历格显示）
  function dailyByDate(dateISO: string): DailyWeather | null {
    return weather.value?.daily?.find((d) => d.date === dateISO) || null;
  }

  async function init() {
    if (inited) return;
    inited = true;
    // 监听后台刷新
    unlisten = await listen<Weather | null>('weather-updated', (e) => {
      if (e.payload) weather.value = e.payload;
    });
    // 取缓存
    const last = await api.getLastWeather();
    if (last) weather.value = last;
    // 启用且无缓存时主动拉
    const cfg = useConfigStore();
    await cfg.load();
    if (cfg.config.weather.enabled && !weather.value) {
      await refresh();
    }
  }

  async function refresh() {
    if (loading.value) return;
    const cfg = useConfigStore();
    const w = cfg.config.weather;
    if (!w.enabled) return;
    loading.value = true;
    try {
      weather.value = await api.fetchWeather(w.latitude, w.longitude, w.city);
    } catch { /* 静默 */ }
    finally { loading.value = false; }
  }

  function dispose() { unlisten?.(); }

  return { weather, loading, dailyByDate, init, refresh, dispose };
});
