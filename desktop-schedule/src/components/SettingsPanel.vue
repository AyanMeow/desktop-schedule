<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { useConfigStore } from '../stores/config';
import { api } from '../api';
import Icon from './Icon.vue';
import { PALETTES } from '../themes';

const configStore = useConfigStore();
const emit = defineEmits<{ close: [] }>();

const autostart = ref(false);
const alwaysOnTop = ref(false);

type Theme = 'dark' | 'light' | 'image';
const theme = computed<Theme>(() => {
  const m = configStore.config.window.bg_mode;
  return m === 'image' ? 'image' : (m === 'light' ? 'light' : 'dark');
});

const fontFamilies = [
  { label: '系统默认', value: "system-ui, 'Microsoft YaHei', sans-serif" },
  { label: '微软雅黑', value: "'Microsoft YaHei', sans-serif" },
  { label: '等线', value: "'DengXian', sans-serif" },
  { label: '宋体', value: "'SimSun', serif" },
  { label: '黑体', value: "'SimHei', sans-serif" },
  { label: '楷体', value: "'KaiTi', serif" },
];

// 常用城市（Open-Meteo 经纬度，免费无 Key）
const cities = [
  { label: '北京', lat: 39.9042, lon: 116.4074 },
  { label: '上海', lat: 31.2304, lon: 121.4737 },
  { label: '广州', lat: 23.1291, lon: 113.2644 },
  { label: '深圳', lat: 22.5431, lon: 114.0579 },
  { label: '杭州', lat: 30.2741, lon: 120.1551 },
  { label: '成都', lat: 30.5728, lon: 104.0668 },
  { label: '武汉', lat: 30.5928, lon: 114.3055 },
  { label: '西安', lat: 34.3416, lon: 108.9398 },
  { label: '南京', lat: 32.0603, lon: 118.7969 },
  { label: '重庆', lat: 29.4316, lon: 106.9123 },
  { label: '天津', lat: 39.3434, lon: 117.3616 },
  { label: '哈尔滨', lat: 45.8038, lon: 126.5350 },
];

function onCityChange(e: Event) {
  const sel = (e.target as HTMLSelectElement).value;
  const found = cities.find((c) => c.label === sel);
  if (found) {
    configStore.config.weather.city = found.label;
    configStore.config.weather.latitude = found.lat;
    configStore.config.weather.longitude = found.lon;
    configStore.save();
  }
}

async function onWeatherToggle() {
  configStore.config.weather.enabled = !configStore.config.weather.enabled;
  await configStore.save();
}

async function pickPalette(name: string) {
  configStore.config.window.theme_name = name;
  await configStore.save();
}

onMounted(async () => {
  autostart.value = await api.isAutostartEnabled();
  alwaysOnTop.value = configStore.config.window.always_on_top;
});

async function save() { await configStore.save(); }
async function onOpacity() { await save(); }
async function onFont() { await save(); }

async function onTop() {
  alwaysOnTop.value = await api.toggleAlwaysOnTop();
  configStore.config.window.always_on_top = alwaysOnTop.value;
  await save();
}
async function onAutostart() {
  try {
    autostart.value = await api.setAutostart(autostart.value);
    configStore.config.startup.auto_start = autostart.value;
    await save();
  } catch (e) {
    alert('设置开机自启失败：' + e);
    // 失败时回滚开关状态
    autostart.value = await api.isAutostartEnabled();
  }
}

async function pickTheme(t: Theme) {
  if (t === 'dark') {
    configStore.config.window.bg_mode = 'dark';
    configStore.config.window.bg_value = '#2b2d3a';
  } else if (t === 'light') {
    configStore.config.window.bg_mode = 'light';
    configStore.config.window.bg_value = '#f5f5f0';
  }
  await save();
}

async function pickImage() {
  const selected = await open({
    multiple: false,
    filters: [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp'] }],
  });
  if (typeof selected === 'string') {
    configStore.config.window.bg_mode = 'image';
    configStore.config.window.bg_value = selected;
    await save();
  }
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal">
      <div class="modal-head">
        <h3><Icon name="settings" :size="18" /> 设置</h3>
        <button class="close-btn" @click="emit('close')" title="关闭"><Icon name="x" :size="18" /></button>
      </div>

      <section class="group">
        <h4><Icon name="image" :size="14" /> 背景主题</h4>
        <div class="theme-row">
          <button :class="{ sel: theme === 'dark' }" @click="pickTheme('dark')">
            <Icon name="calendar" :size="14" /> 深色
          </button>
          <button :class="{ sel: theme === 'light' }" @click="pickTheme('light')">
            <Icon name="calendar" :size="14" /> 浅色
          </button>
          <button :class="{ sel: theme === 'image' }" @click="pickImage">
            <Icon name="image" :size="14" /> 图片
          </button>
        </div>
      </section>

      <section class="group">
        <h4><Icon name="star" :size="14" /> 配色</h4>
        <div class="palette-grid">
          <button
            v-for="p in PALETTES"
            :key="p.name"
            class="palette-chip"
            :class="{ sel: configStore.config.window.theme_name === p.name }"
            @click="pickPalette(p.name)"
          >
            <span class="chip-dot" :style="{ background: p.darkBg }">
              <span class="chip-accent" :style="{ background: p.darkAccent }"></span>
            </span>
            <span class="chip-label">{{ p.label }}</span>
          </button>
        </div>
      </section>

      <section class="group">
        <h4><Icon name="type" :size="14" /> 字体</h4>
        <label class="line">
          <span>字号 {{ configStore.config.window.font_size }}px</span>
          <input
            type="range" min="12" max="22" step="1"
            v-model.number="configStore.config.window.font_size"
            @change="onFont"
          />
        </label>
        <label class="line">
          <span>字体</span>
          <select v-model="configStore.config.window.font_family" @change="onFont">
            <option v-for="f in fontFamilies" :key="f.value" :value="f.value">{{ f.label }}</option>
          </select>
        </label>
      </section>

      <section class="group">
        <h4><Icon name="image" :size="14" /> 背景透明度</h4>
        <label class="line">
          <span>{{ Math.round(configStore.config.window.opacity * 100) }}%</span>
          <input
            type="range" min="0.1" max="1" step="0.05"
            v-model.number="configStore.config.window.opacity"
            @change="onOpacity"
          />
        </label>
        <p class="tip">仅影响背景，文字保持清晰</p>
      </section>

      <section class="group">
        <h4><Icon name="sun" :size="14" /> 天气</h4>
        <label class="line">
          <span><Icon name="sun" :size="13" /> 启用天气</span>
          <input type="checkbox" :checked="configStore.config.weather.enabled" @change="onWeatherToggle" />
        </label>
        <label class="line" v-if="configStore.config.weather.enabled">
          <span><Icon name="pin" :size="13" /> 城市</span>
          <select :value="configStore.config.weather.city" @change="onCityChange">
            <option v-for="c in cities" :key="c.label" :value="c.label">{{ c.label }}</option>
          </select>
        </label>
      </section>

      <section class="group">
        <h4><Icon name="pin" :size="14" /> 窗口</h4>
        <label class="line">
          <span><Icon name="pin" :size="13" /> 始终置顶</span>
          <input type="checkbox" :checked="alwaysOnTop" @change="onTop" />
        </label>
        <label class="line">
          <span><Icon name="settings" :size="13" /> 开机自启</span>
          <input type="checkbox" :checked="autostart" @change="onAutostart" />
        </label>
      </section>

      <div class="actions">
        <button class="btn primary" @click="emit('close')">完成</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed; inset: 0;
  background: rgba(0, 0, 0, 0.45);
  backdrop-filter: blur(2px);
  display: flex; align-items: center; justify-content: center;
  z-index: 100; padding: 12px;
}
.modal {
  background: color-mix(in srgb, var(--modal-bg, #2a2c3a) 92%, transparent);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid rgba(128, 128, 128, 0.25);
  border-radius: 14px;
  width: 100%; max-width: 360px;
  max-height: 80vh; overflow-y: auto;
  color: inherit;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.6);
}
.modal-head {
  display: flex; align-items: center; justify-content: space-between;
  padding: 14px 16px 10px;
  border-bottom: 1px solid rgba(128, 128, 128, 0.2);
  position: sticky; top: 0; background: inherit; z-index: 2;
}
h3 {
  margin: 0; font-size: 15px; font-weight: 600;
  display: inline-flex; align-items: center; gap: 0.4em;
}
.close-btn {
  background: transparent; border: none; color: inherit;
  opacity: 0.5; cursor: pointer; padding: 4px; border-radius: 6px;
  display: flex;
}
.close-btn:hover { opacity: 1; background: rgba(128, 128, 128, 0.2); }
.group { margin: 12px 16px 16px; }
h4 {
  margin: 0 0 10px; font-size: 12px; opacity: 0.7; font-weight: 600;
  display: inline-flex; align-items: center; gap: 0.35em;
  text-transform: uppercase; letter-spacing: 0.5px;
}
.theme-row { display: flex; gap: 6px; }
.theme-row button {
  flex: 1;
  background: rgba(128, 128, 128, 0.12);
  border: 1px solid rgba(128, 128, 128, 0.2);
  color: inherit; opacity: 0.8;
  padding: 8px 4px; border-radius: 6px;
  font-size: 12px; cursor: pointer; font-family: inherit;
  display: flex; align-items: center; justify-content: center; gap: 4px;
}
.theme-row button:hover { background: rgba(128, 128, 128, 0.22); opacity: 1; }
.theme-row button.sel {
  background: var(--accent); border-color: var(--accent); color: #fff; opacity: 1;
}
.line {
  display: flex; align-items: center; justify-content: space-between;
  font-size: 13px; margin-bottom: 10px; gap: 12px;
}
.line span {
  flex-shrink: 0;
  display: inline-flex; align-items: center; gap: 0.35em;
}
.tip { font-size: 11px; opacity: 0.5; margin: 4px 0 0; }
input[type='range'] { flex: 1; min-width: 0; }
input[type='checkbox'] { width: 16px; height: 16px; accent-color: var(--accent); }
select {
  background: rgba(128,128,128,0.15); color: inherit;
  border: 1px solid rgba(128,128,128,0.25); border-radius: 6px;
  padding: 6px 8px; font-size: 13px; font-family: inherit;
}
/* select 下拉选项用系统默认深底浅字，避免在浅色面板里灰看不清 */
select option { background: #2a2c3a; color: #f0f0f5; }
.actions {
  display: flex; justify-content: flex-end;
  margin: 6px 16px 16px; padding-top: 10px;
  border-top: 1px solid rgba(128, 128, 128, 0.15);
}
.btn {
  padding: 8px 18px; border-radius: 6px; border: none;
  cursor: pointer; font-size: 13px; font-family: inherit;
}
.btn.primary { background: var(--accent); color: #fff; }

.palette-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 6px;
}
.palette-chip {
  background: rgba(128,128,128,0.1);
  border: 1px solid rgba(128,128,128,0.2);
  border-radius: 7px;
  padding: 6px 4px 5px;
  cursor: pointer;
  font-family: inherit;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  color: inherit;
}
.palette-chip:hover { background: rgba(128,128,128,0.2); }
.palette-chip.sel { border-color: var(--accent); background: var(--accent-soft); }
.chip-dot {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}
.chip-accent {
  width: 9px;
  height: 9px;
  border-radius: 50%;
}
.chip-label { font-size: 11px; opacity: 0.9; }
</style>
