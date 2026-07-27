<script setup lang="ts">
import { computed } from 'vue';
import { useWeatherStore } from '../stores/weather';
import { useConfigStore } from '../stores/config';
import Icon from './Icon.vue';

const weatherStore = useWeatherStore();
const configStore = useConfigStore();

const enabled = computed(() => configStore.config.weather.enabled);
const weather = computed(() => weatherStore.weather);
const loading = computed(() => weatherStore.loading);
</script>

<template>
  <button
    v-if="enabled"
    class="weather-badge"
    @click="weatherStore.refresh()"
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
.weather-badge:hover { background: var(--accent-soft); opacity: 1; }
.temp { font-weight: 700; }
.desc { opacity: 0.7; font-size: 0.9em; }
.loading { opacity: 0.5; }
</style>
