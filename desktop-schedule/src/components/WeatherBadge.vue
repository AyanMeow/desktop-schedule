<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { api } from '../api';
import { useConfigStore } from '../stores/config';
import Icon from './Icon.vue';
import type { Weather } from '../types';

const configStore = useConfigStore();
const weather = ref<Weather | null>(null);
const loading = ref(false);
let unlisten: UnlistenFn | undefined;

const enabled = computed(() => configStore.config.weather.enabled);

async function manualRefresh() {
  if (loading.value) return;
  const w = configStore.config.weather;
  if (!w.enabled) return;
  loading.value = true;
  try {
    weather.value = await api.fetchWeather(w.latitude, w.longitude, w.city);
  } catch { /* 静默失败，保持上次结果 */ }
  finally { loading.value = false; }
}

onMounted(async () => {
  // 先取缓存
  const last = await api.getLastWeather();
  if (last) weather.value = last;
  // 监听后台刷新
  unlisten = await listen<Weather | null>('weather-updated', (e) => {
    if (e.payload) weather.value = e.payload;
  });
  // 启用且无缓存时主动拉一次
  if (configStore.config.weather.enabled && !weather.value) {
    manualRefresh();
  }
});

onUnmounted(() => unlisten?.());
</script>

<template>
  <button
    v-if="enabled"
    class="weather-badge"
    @click="manualRefresh"
    :title="weather ? `${weather.city} · 更新于 ${weather.updated_at}` : '点击刷新'"
  >
    <template v-if="weather">
      <Icon :name="weather.icon" :size="16" />
      <span class="temp">{{ Math.round(weather.temperature) }}°</span>
      <span class="desc">{{ weather.description }}</span>
    </template>
    <template v-else-if="loading">
      <span class="loading">···</span>
    </template>
    <template v-else>
      <Icon name="sun" :size="16" />
      <span class="temp">--°</span>
    </template>
  </button>
</template>

<style scoped>
.weather-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.3em;
  background: transparent;
  border: none;
  color: inherit;
  cursor: pointer;
  font-size: 0.8em;
  padding: 0.2em 0.4em;
  border-radius: 6px;
  opacity: 0.9;
}
.weather-badge:hover { background: rgba(128, 128, 128, 0.2); opacity: 1; }
.temp { font-weight: 600; }
.desc { opacity: 0.7; font-size: 0.9em; }
.loading { opacity: 0.5; }
</style>
