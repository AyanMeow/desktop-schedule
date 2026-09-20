<script setup lang="ts">
// 左侧抽屉：收纳 设置/临时待办/成就 三个图标入口。
// 独立窗口（#drawer），由主贴片左缘热区触发展开：自定位到主窗左缘外侧，
// 内容自贴片边缘向左揭示（clip-path）。收起：鼠标离开 300ms 宽限 /
// 点击入口 / 主窗拖动缩放 / 鼠标移入主窗正文 / 主窗被隐藏。
import { ref, onMounted, onUnmounted } from 'vue';
import { getCurrentWindow, Window } from '@tauri-apps/api/window';
import { PhysicalPosition, PhysicalSize } from '@tauri-apps/api/dpi';
import { listen, emit } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { useConfigStore } from '../stores/config';
import { api } from '../api';
import { useThemeStyle } from '../composables/themeStyle';
import Icon from './Icon.vue';

const DRAWER_W = 56; // 逻辑宽度（纯图标纵排），显示时按主窗缩放因子换算物理像素

const win = getCurrentWindow();
const configStore = useConfigStore();
const { theme, rootStyle, bgLayerStyle } = useThemeStyle(() => configStore.config.window);

// 展开动画开关：窗口先显示，卡片再自贴片边缘向左揭示
const opened = ref(false);

let mainWin: Window | null = null;
async function getMain(): Promise<Window | null> {
  mainWin = mainWin || (await Window.getByLabel('main'));
  return mainWin;
}

let hideTimer: number | undefined;
let pollTimer: number | undefined;
let unlistenOpen: UnlistenFn | undefined;
let unlistenHide: UnlistenFn | undefined;
let unlistenCfg: UnlistenFn | undefined;

function cancelHide() {
  if (hideTimer) {
    window.clearTimeout(hideTimer);
    hideTimer = undefined;
  }
}
function scheduleHide(delay = 300) {
  cancelHide();
  hideTimer = window.setTimeout(() => void hide(), delay);
}

async function open() {
  const m = await getMain();
  if (!m) return;
  try {
    if (!(await m.isVisible())) return;
    const pos = await m.outerPosition();
    const size = await m.outerSize();
    const sf = await m.scaleFactor();
    const w = Math.round(DRAWER_W * sf);
    await win.setSize(new PhysicalSize(w, size.height));
    await win.setPosition(new PhysicalPosition(pos.x - w, pos.y));
    // 跟随主窗置顶态，避免被其它窗口盖住
    await win.setAlwaysOnTop(await m.isAlwaysOnTop());
    cancelHide();
    await win.show();
    // 取得焦点：WebView2 对前台窗口的鼠标事件派发最可靠（离开事件是收起的命脉）
    await win.setFocus();
    requestAnimationFrame(() => (opened.value = true));
  } catch {
    /* 主窗不存在等异常静默 */
  }
}

async function hide() {
  cancelHide();
  opened.value = false;
  try {
    await win.hide();
  } catch {
    /* 忽略 */
  }
}

// 入口点击：通知主窗执行原逻辑，随后收起
async function onAction(type: 'settings' | 'memo' | 'achievements') {
  await emit('drawer-action', { type });
  void hide();
}

// 鼠标在抽屉内活动即视为"仍在使用"
function onDocOver() {
  if (opened.value) cancelHide();
}
// 鼠标离开抽屉窗口：宽限后收起
function onDocLeave() {
  if (opened.value) scheduleHide();
}
// 兜底：mouseout 且无 relatedTarget 同样意味着离开窗口（个别环境下 mouseleave 不可靠）
function onDocOut(e: MouseEvent) {
  if (opened.value && !e.relatedTarget) scheduleHide();
}

onMounted(async () => {
  // 主题配置加载（带与主窗相同的重试）
  await configStore.load();

  // 主窗热区请求展开；悬停心跳重复触发时仅重置收起计时
  unlistenOpen = await listen('open-drawer', () => {
    if (opened.value) cancelHide();
    else void open();
  });
  // 主窗拖动/缩放前、鼠标移入主窗正文时请求收起
  unlistenHide = await listen('drawer-hide', () => void hide());
  // 主窗配置变更广播：刷新主题跟随
  unlistenCfg = await listen('config-changed', async () => {
    try {
      configStore.config = await api.getConfig();
    } catch {
      /* 忽略，保持现状 */
    }
  });

  document.addEventListener('mouseover', onDocOver);
  document.addEventListener('mouseleave', onDocLeave);
  document.addEventListener('mouseout', onDocOut);

  // 轮询主窗可见性：面板/托盘隐藏贴片时抽屉同步收起
  pollTimer = window.setInterval(async () => {
    if (!opened.value) return;
    const m = await getMain();
    const vis = m ? await m.isVisible().catch(() => false) : false;
    if (!vis) void hide();
  }, 800);
});

onUnmounted(() => {
  cancelHide();
  if (pollTimer) window.clearInterval(pollTimer);
  unlistenOpen?.();
  unlistenHide?.();
  unlistenCfg?.();
  document.removeEventListener('mouseover', onDocOver);
  document.removeEventListener('mouseleave', onDocLeave);
  document.removeEventListener('mouseout', onDocOut);
});
</script>

<template>
  <div class="drawer" :class="{ open: opened }" :style="rootStyle" :data-theme="theme">
    <div class="bg" :style="bgLayerStyle"></div>
    <div class="veil"></div>
    <div class="inner">
      <button class="d-item" @click="onAction('settings')" title="设置">
        <Icon name="settings" :size="18" />
      </button>
      <button class="d-item" @click="onAction('memo')" title="临时待办">
        <Icon name="note" :size="18" />
      </button>
      <button class="d-item" @click="onAction('achievements')" title="成就">
        <Icon name="trophy" :size="18" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.drawer {
  position: relative;
  height: 100vh;
  overflow: hidden;
  border-radius: 12px;
  border: 1px solid rgba(128, 128, 128, 0.25);
  user-select: none;
  /* 自贴片边缘向左揭示：初始整体裁掉，open 时左边界展开到 0 */
  clip-path: inset(0 0 0 100%);
  transition: clip-path 0.18s ease-out;
  /* clip-path 会裁掉 box-shadow，改用 drop-shadow 跟随裁剪形状 */
  filter: drop-shadow(-6px 0 16px rgba(0, 0, 0, 0.35));
}
.drawer.open {
  clip-path: inset(0 0 0 0);
}
.bg {
  position: absolute;
  inset: 0;
  background-position: center;
  background-size: cover;
  background-repeat: no-repeat;
  z-index: 0;
}
.veil {
  position: absolute;
  inset: 0;
  background: var(--veil-bg);
  backdrop-filter: blur(20px) saturate(1.2);
  -webkit-backdrop-filter: blur(20px) saturate(1.2);
  z-index: 1;
}
.inner {
  position: relative;
  z-index: 2;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 12px 0;
}
.d-item {
  width: 38px;
  height: 38px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid transparent;
  border-radius: 10px;
  background: rgba(128, 128, 128, 0.12);
  color: inherit;
  cursor: pointer;
  transition: background 0.12s;
}
.d-item:hover {
  background: var(--accent-soft);
  border-color: var(--accent);
}
.d-item:active {
  background: var(--accent);
  color: #fff;
}
</style>
