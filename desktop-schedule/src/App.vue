<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useScheduleStore } from './stores/schedules';
import { useConfigStore } from './stores/config';
import CalendarGrid from './components/CalendarGrid.vue';
import DayPanel from './components/DayPanel.vue';
import AddScheduleModal from './components/AddScheduleModal.vue';
import SettingsPanel from './components/SettingsPanel.vue';
import EncouragementToast from './components/EncouragementToast.vue';
import Icon from './components/Icon.vue';
import { toISO } from './utils/date';

const scheduleStore = useScheduleStore();
const configStore = useConfigStore();

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
  // veil：文字背后的磨砂底板。深色主题=深底，浅色=浅底，图片=半透深底
  const veilColor = light ? 'rgba(245,245,240,0.55)' : 'rgba(20,22,32,0.55)';
  return {
    '--app-font-size': `${configStore.config.window.font_size}px`,
    '--app-font-family': configStore.config.window.font_family,
    '--app-fg': light ? '#1a1a2e' : '#f0f0f5',
    '--app-fg-soft': light ? 'rgba(26,26,46,0.65)' : 'rgba(240,240,245,0.65)',
    '--modal-bg': light ? '#f5f5f0' : '#2a2c3a',
    '--veil-bg': veilColor,
    fontFamily: configStore.config.window.font_family,
    fontSize: `${configStore.config.window.font_size}px`,
    color: light ? '#1a1a2e' : '#f0f0f5',
  };
});

// 背景层样式：透明度只作用于背景，前景文字不受影响
const bgLayerStyle = computed(() => {
  const w = configStore.config.window;
  if (w.bg_mode === 'image' && w.bg_value) {
    return {
      backgroundImage: `url(${w.bg_value})`,
      opacity: w.opacity,
    };
  }
  return {
    background: w.bg_value,
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
  if (locked.value || menuLocked.value) return;
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

onMounted(async () => {
  await configStore.load();
  await scheduleStore.refresh();
  locked.value = configStore.config.window.locked;
  menuLocked.value = locked.value;
  menuTop.value = configStore.config.window.always_on_top;
  scheduleStore.viewRange = (configStore.config.view.range as any) || 'week';
  scheduleStore.weekStart = (configStore.config.view.week_start as any) || 'monday';
  if (configStore.config.startup.expand_today_on_launch) {
    expandedDate.value = toISO(new Date());
  }
});
</script>

<template>
  <div class="widget" :style="rootStyle" :data-theme="theme">
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
}
.resize-handle {
  position: absolute;
  right: 0;
  bottom: 0;
  width: 16px;
  height: 16px;
  cursor: nwse-resize;
  z-index: 60;
  background: linear-gradient(135deg, transparent 50%, rgba(128,128,128,0.4) 50%, rgba(128,128,128,0.4) 60%, transparent 60%, transparent 70%, rgba(128,128,128,0.4) 70%, rgba(128,128,128,0.4) 80%, transparent 80%);
}
</style>
