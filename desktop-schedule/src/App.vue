<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, getAllWindows, LogicalPosition, LogicalSize } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { useScheduleStore } from './stores/schedules';
import { useConfigStore } from './stores/config';
import { useWeatherStore } from './stores/weather';
import CalendarGrid from './components/CalendarGrid.vue';
import DayPanel from './components/DayPanel.vue';
import AddScheduleModal from './components/AddScheduleModal.vue';
import SettingsPanel from './components/SettingsPanel.vue';
import EncouragementToast from './components/EncouragementToast.vue';
import WeatherBadge from './components/WeatherBadge.vue';
import Icon from './components/Icon.vue';
import { getPalette, getDdlScale } from './themes';
import { toISO } from './utils/date';

const scheduleStore = useScheduleStore();
const configStore = useConfigStore();
const weatherStore = useWeatherStore();

// 是否为控制面板窗口（taskbar 窗口 url 带 #panel）
const isPanel = computed(() => window.location.hash === '#panel');

const locked = ref(false);
const expandedDate = ref<string | null>(null);
const showAdd = ref(false);
const showSettings = ref(false);
const showMenu = ref(false);
const menuLocked = ref(false);
const menuTop = ref(false);
const addPresetDate = ref<string | null>(null);
const contentRef = ref<HTMLElement | null>(null);

const theme = computed(() => configStore.config.window.bg_mode);
const isLight = computed(() => theme.value === 'light');

// 根容器样式：字体由 CSS 变量驱动，子组件用 em 继承
const rootStyle = computed(() => {
  const light = isLight.value;
  const p = getPalette(configStore.config.window.theme_name);
  const fg = light ? p.lightFg : p.darkFg;
  const accent = light ? p.lightAccent : p.darkAccent;
  const warning = light ? p.lightWarning : p.darkWarning;
  const danger = light ? p.lightDanger : p.darkDanger;
  const ddl = getDdlScale(p, light);
  // veil：文字背后的磨砂底板
  const veilBg = light
    ? hexToRgba(p.lightBg, 0.55)
    : hexToRgba(p.darkBg, 0.55);
  return {
    '--app-font-size': `${configStore.config.window.font_size}px`,
    '--app-font-family': configStore.config.window.font_family,
    '--app-fg': fg,
    '--app-fg-soft': fg + 'a6', // 约 65% 不透明
    '--modal-bg': light ? p.lightBg : p.darkBg,
    '--veil-bg': veilBg,
    '--accent': accent,
    '--accent-soft': accent + '2e', // 约 18%
    '--warning': warning,
    '--danger': danger,
    '--ddl-overdue': ddl.overdue,
    '--ddl-le1': ddl.le1,
    '--ddl-le3': ddl.le3,
    '--ddl-le7': ddl.le7,
    '--ddl-gt7': ddl.gt7,
    fontFamily: configStore.config.window.font_family,
    fontSize: `${configStore.config.window.font_size}px`,
    color: fg,
  };
});

// hex 转 rgba（veil 半透明用）
function hexToRgba(hex: string, alpha: number): string {
  const h = hex.replace('#', '');
  const r = parseInt(h.slice(0, 2), 16);
  const g = parseInt(h.slice(2, 4), 16);
  const b = parseInt(h.slice(4, 6), 16);
  return `rgba(${r},${g},${b},${alpha})`;
}

// 背景层样式：透明度只作用于背景，前景文字不受影响
const bgLayerStyle = computed(() => {
  const w = configStore.config.window;
  if (w.bg_mode === 'image' && w.bg_value) {
    return {
      backgroundImage: `url(${w.bg_value})`,
      opacity: w.opacity,
    };
  }
  // dark/light 模式：背景色由 palette 决定（切主题时即时生效）
  const p = getPalette(w.theme_name);
  const bg = isLight.value ? p.lightBg : p.darkBg;
  return {
    background: bg,
    opacity: w.opacity,
  };
});

// 仅拖拽顶栏
async function onTitlebarDown(e: MouseEvent) {
  if (locked.value || menuLocked.value) return;
  const target = e.target as HTMLElement;
  if (target.closest('button, input, .dropdown')) return;
  await getCurrentWindow().startDragging();
}

async function onResizeDown(e: MouseEvent) {
  e.stopPropagation();
  e.preventDefault();
  if (locked.value || menuLocked.value) return;
  // SouthEast = 同时向右(东)和向下(南)拉伸，支持斜向拖动
  await getCurrentWindow().startResizeDragging('SouthEast');
}

async function toggleLock() {
  menuLocked.value = await invoke<boolean>('toggle_lock');
  locked.value = menuLocked.value;
  showMenu.value = false;
}

async function toggleTop() {
  menuTop.value = await invoke<boolean>('toggle_always_on_top');
  configStore.config.window.always_on_top = menuTop.value;
  await configStore.save();
  showMenu.value = false;
}

function openAddFor(dateISO?: string | null) {
  addPresetDate.value = dateISO || null;
  showAdd.value = true;
}

function onAdded(dateISO: string) {
  expandedDate.value = dateISO;
}

// ===== 控制面板（taskbar 窗口）方法 =====
async function panelToggleWidget() {
  const wins = await getAllWindows();
  const main = wins.find((w) => w.label === 'main');
  if (!main) return;
  const vis = await main.isVisible();
  if (vis) await main.hide();
  else { await main.show(); await main.setFocus(); }
}
async function panelFocusWidget() {
  const wins = await getAllWindows();
  const main = wins.find((w) => w.label === 'main');
  if (main) { await main.show(); await main.setFocus(); }
}

// 展开日期变化时，把面板滚入视野（解决月视图看不到面板）
watch(expandedDate, async () => {
  await nextTick();
  if (contentRef.value) {
    const panel = contentRef.value.querySelector('.day-panel');
    if (panel) {
      panel.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  }
});

// 窗口几何持久化：防抖保存位置/大小
let geomTimer: number | undefined;
let unlistenMoved: UnlistenFn | undefined;
let unlistenResized: UnlistenFn | undefined;
let unlistenImported: UnlistenFn | undefined;
async function saveGeometry() {
  const win = getCurrentWindow();
  try {
    const pos = await win.outerPosition();
    const size = await win.outerSize();
    // 转换物理像素→逻辑像素（除以缩放因子）
    const sf = await win.scaleFactor();
    configStore.config.window.x = Math.round(pos.x / sf);
    configStore.config.window.y = Math.round(pos.y / sf);
    configStore.config.window.width = Math.round(size.width / sf);
    configStore.config.window.height = Math.round(size.height / sf);
    await configStore.save();
  } catch { /* 忽略 */ }
}
function debounceSaveGeometry() {
  if (geomTimer) window.clearTimeout(geomTimer);
  geomTimer = window.setTimeout(saveGeometry, 500);
}

onMounted(async () => {
  // 控制面板窗口不需要加载日程
  if (isPanel.value) return;
  await configStore.load();

  // 恢复窗口几何（位置/大小）
  const win = getCurrentWindow();
  const w = configStore.config.window;
  try {
    await win.setPosition(new LogicalPosition(w.x, w.y));
    await win.setSize(new LogicalSize(w.width, w.height));
  } catch { /* 忽略 */ }

  await scheduleStore.refresh();
  locked.value = configStore.config.window.locked;
  menuLocked.value = locked.value;
  menuTop.value = configStore.config.window.always_on_top;
  scheduleStore.viewRange = (configStore.config.view.range as any) || 'week';
  scheduleStore.weekStart = (configStore.config.view.week_start as any) || 'monday';
  if (configStore.config.startup.expand_today_on_launch) {
    expandedDate.value = toISO(new Date());
  }
  await weatherStore.init();

  // 同步开机自启状态：以 config 为准，确保注册表与配置一致
  try {
    const registered = await invoke<boolean>('is_autostart_enabled');
    const want = configStore.config.startup.auto_start;
    if (want && !registered) {
      await invoke<boolean>('set_autostart', { enabled: true });
    } else if (!want && registered) {
      await invoke<boolean>('set_autostart', { enabled: false });
    }
  } catch (e) { console.error('autostart sync failed', e); /* 忽略 */ }

  // 监听窗口移动/缩放，防抖保存几何
  unlistenMoved = await win.onMoved(() => debounceSaveGeometry());
  unlistenResized = await win.onResized(() => debounceSaveGeometry());

  // 监听导入日程事件，刷新显示
  unlistenImported = await listen('schedules-imported', () => {
    scheduleStore.refresh();
  });
});

onUnmounted(() => {
  unlistenMoved?.();
  unlistenResized?.();
  unlistenImported?.();
});
</script>

<template>
  <!-- 控制面板窗口（taskbar） -->
  <div v-if="isPanel" class="panel">
    <h2><Icon name="calendar" :size="20" /> 桌面日程</h2>
    <p class="panel-tip">桌面贴片始终显示，不受此窗口影响</p>
    <button class="panel-btn primary" @click="panelFocusWidget">
      <Icon name="calendar" :size="16" /> 找到桌面贴片
    </button>
    <button class="panel-btn" @click="panelToggleWidget">
      <Icon name="x" :size="16" /> 显示/隐藏贴片
    </button>
    <button class="panel-btn ghost" @click="showSettings = true">
      <Icon name="settings" :size="16" /> 设置
    </button>
    <SettingsPanel v-if="showSettings" @close="showSettings = false" />
  </div>

  <!-- 主贴片窗口（main） -->
  <div v-else class="widget" :style="rootStyle" :data-theme="theme">
    <!-- 背景层：透明度只作用于此层 -->
    <div class="bg-layer" :style="bgLayerStyle"></div>

    <!-- 文字底板层：磨砂玻璃，给文字柔和托底，保证低透明度下清晰 -->
    <div class="fg-veil"></div>

    <!-- 前景内容层（不透明） -->
    <div class="fg">
      <header class="topbar" @mousedown.left="onTitlebarDown">
        <div class="menu-wrap">
          <button class="icon-btn" @click="showMenu = !showMenu" title="菜单">
            <Icon name="menu" :size="18" />
          </button>
          <div v-if="showMenu" class="dropdown" @click.stop>
            <button @click="toggleLock">
              <Icon name="unlock" :size="15" v-if="menuLocked" />
              <Icon name="lock" :size="15" v-else />
              {{ menuLocked ? '解除锁定' : '锁定位置' }}
            </button>
            <button @click="toggleTop">
              <Icon name="pin" :size="15" />
              {{ menuTop ? '取消置顶' : '置顶' }}
            </button>
            <button @click="showAdd = true; showMenu = false; addPresetDate = expandedDate">
              <Icon name="plus" :size="15" />
              添加日程
            </button>
            <button @click="showSettings = true; showMenu = false">
              <Icon name="settings" :size="15" />
              设置
            </button>
          </div>
        </div>
        <span class="brand"><Icon name="calendar" :size="16" /> 桌面日程</span>
        <WeatherBadge />
        <div class="topbar-actions">
          <button class="icon-btn" @click="toggleLock" :title="menuLocked ? '解除锁定' : '锁定位置'">
            <Icon :name="menuLocked ? 'lock' : 'unlock'" :size="17" />
          </button>
          <button class="icon-btn" @click="toggleTop" :title="menuTop ? '取消置顶' : '置顶'"
            :class="{ active: menuTop }">
            <Icon name="pin" :size="17" />
          </button>
          <button class="icon-btn" @click="openAddFor(expandedDate)" title="添加">
            <Icon name="plus" :size="20" />
          </button>
        </div>
      </header>

      <div class="content" ref="contentRef">
        <CalendarGrid v-model:expanded-date="expandedDate" />
        <DayPanel
          v-if="expandedDate"
          :date-i-s-o="expandedDate"
          @add="openAddFor(expandedDate)"
        />
      </div>
    </div>

    <div class="resize-handle" @mousedown.left="onResizeDown" title="拖动调整大小"></div>

    <AddScheduleModal
      v-if="showAdd"
      :preset-date="addPresetDate"
      @close="showAdd = false"
      @added="onAdded"
    />
    <SettingsPanel v-if="showSettings" @close="showSettings = false" />
    <EncouragementToast />
  </div>
</template>

<style>
:root {
  font-family: 'Microsoft YaHei', 'Segoe UI', system-ui, sans-serif;
  -webkit-font-smoothing: antialiased;
}
html, body, #app {
  margin: 0;
  padding: 0;
  height: 100%;
  background: transparent;
}
* { box-sizing: border-box; }
</style>

<style scoped>
.widget {
  position: relative;
  height: 100vh;
  border-radius: 14px;
  overflow: hidden;
  border: 1px solid rgba(128, 128, 128, 0.25);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  user-select: none;
}
/* 背景层：独立透明，不拖累前景 */
.bg-layer {
  position: absolute;
  inset: 0;
  background-position: center;
  background-size: cover;
  background-repeat: no-repeat;
  z-index: 0;
}
/* 文字底板层：磨砂玻璃，托底保证可读性。z-index 在背景之上、前景之下 */
.fg-veil {
  position: absolute;
  inset: 0;
  background: var(--veil-bg);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  z-index: 1;
}
.fg {
  position: relative;
  z-index: 2;
  height: 100%;
  display: flex;
  flex-direction: column;
}
.topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.6em 0.7em;
  cursor: grab;
  flex-shrink: 0;
}
.topbar:active { cursor: grabbing; }
.brand {
  display: inline-flex;
  align-items: center;
  gap: 0.3em;
  font-size: 1em;
  font-weight: 600;
  flex: 1;
  margin-left: 0.3em;
}
.menu-wrap { position: relative; }
.topbar-actions { display: flex; align-items: center; gap: 0.2em; }
.icon-btn.active { background: rgba(108,140,255,0.3); color: #6c8cff; }
.icon-btn {
  background: transparent;
  border: none;
  color: inherit;
  opacity: 0.8;
  width: 1.8em;
  height: 1.8em;
  border-radius: 6px;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.icon-btn:hover { opacity: 1; background: rgba(128, 128, 128, 0.2); }
.dropdown {
  position: absolute;
  top: 2.2em;
  left: 0;
  background: #2a2c3a;
  color: #f0f0f5;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 8px;
  padding: 4px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  z-index: 50;
  min-width: 150px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
}
.dropdown button {
  background: transparent;
  border: none;
  color: #e0e0ec;
  padding: 0.55em 0.8em;
  text-align: left;
  border-radius: 5px;
  cursor: pointer;
  font-size: 0.85em;
  font-family: inherit;
  display: flex;
  align-items: center;
  gap: 0.5em;
}
.dropdown button:hover { background: rgba(255, 255, 255, 0.08); }
.content {
  flex: 1;
  overflow-y: auto;
  padding: 0.7em 0.8em 1em;
  display: flex;
  flex-direction: column;
  /* 隐藏滚动条但保留滚动功能（WebView2 基于 Chromium，两套前缀都加）*/
  scrollbar-width: none; /* Firefox */
  -ms-overflow-style: none; /* IE/旧 Edge */
}
.content::-webkit-scrollbar {
  display: none; /* Chrome/WebKit/WebView2 */
}
.resize-handle {
  position: absolute;
  right: 0;
  bottom: 0;
  width: 20px;
  height: 20px;
  cursor: nwse-resize;
  z-index: 70;
  background: linear-gradient(135deg, transparent 50%, rgba(128,128,128,0.45) 50%, rgba(128,128,128,0.45) 60%, transparent 60%, transparent 70%, rgba(128,128,128,0.45) 70%, rgba(128,128,128,0.45) 80%, transparent 80%);
}

/* ===== 控制面板（taskbar 窗口） ===== */
.panel {
  height: 100vh;
  background: #f5f5f7;
  color: #1a1a2e;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 24px 16px;
  gap: 12px;
  font-family: 'Microsoft YaHei', system-ui, sans-serif;
}
.panel h2 {
  margin: 0;
  font-size: 18px;
  display: inline-flex;
  align-items: center;
  gap: 0.4em;
}
.panel-tip {
  margin: 0 0 8px;
  font-size: 11px;
  opacity: 0.55;
  text-align: center;
}
.panel-btn {
  width: 100%;
  padding: 12px;
  border-radius: 8px;
  border: 1px solid rgba(0, 0, 0, 0.1);
  background: #fff;
  color: #1a1a2e;
  cursor: pointer;
  font-size: 14px;
  font-family: inherit;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.4em;
  transition: background 0.15s;
}
.panel-btn:hover { background: #ececef; }
.panel-btn.primary { background: #6c8cff; color: #fff; border-color: #6c8cff; }
.panel-btn.primary:hover { background: #5a7aee; }
.panel-btn.ghost { background: transparent; }
</style>
