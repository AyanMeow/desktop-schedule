<script setup lang="ts">
// 临时待办便签：可编辑清单 + 右下工具栏 + 每行倒计时。
// 纯内存数据，不持久化；关闭（X）前确认并清空；顶栏按钮收起/唤起保留数据。
import { ref, computed, watch, onUnmounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import Icon from './Icon.vue';

interface MemoLine {
  id: number;
  text: string;
  done: boolean;
  end: number | null; // 倒计时截止 epoch ms
}

const win = getCurrentWindow();
const lines = ref<MemoLine[]>([]);
const input = ref('');
let nextId = 1;

// 便签级样式
const fontSize = ref(15);
const bold = ref(false);
const color = ref('#1a1a1a');
const COLORS = ['#1a1a1a', '#d0342c', '#2563eb', '#16a34a', '#ca8a04'];

// 活动行（倒计时的设置目标）：点击行选中
const activeId = ref<number | null>(null);
const showTimerPicker = ref(false);
const customMinutes = ref('');

// 到时提醒横幅
const expiredText = ref<string | null>(null);
// 确认关闭
const confirming = ref(false);

const noteStyle = computed(() => ({
  fontSize: fontSize.value + 'px',
  fontWeight: bold.value ? '700' : '400',
  color: color.value,
}));

// ---- 倒计时：有活动倒计时才跑表（1s），到时强制弹出 + 提醒 ----
const now = ref(Date.now());
let tickTimer: number | undefined;
const hasCountdown = computed(() => lines.value.some((l) => l.end && !l.done));

function startTick() {
  if (tickTimer) return;
  tickTimer = window.setInterval(() => {
    now.value = Date.now();
    checkExpired();
  }, 1000);
}
function stopTick() {
  if (tickTimer) {
    window.clearInterval(tickTimer);
    tickTimer = undefined;
  }
}
watch(hasCountdown, (v) => (v ? startTick() : stopTick()), { immediate: true });
onUnmounted(stopTick);

function checkExpired() {
  for (const l of lines.value) {
    if (l.end && !l.done && now.value >= l.end) {
      l.end = null;
      expiredText.value = l.text;
      void forceShow();
      beep();
      const t = l.text;
      window.setTimeout(() => {
        if (expiredText.value === t) expiredText.value = null;
      }, 8000);
    }
  }
}

async function forceShow() {
  try {
    await win.show();
    await win.setFocus();
  } catch {
    /* 忽略 */
  }
}

let audioCtx: AudioContext | null = null;
function beep() {
  try {
    audioCtx = audioCtx || new AudioContext();
    if (audioCtx.state === 'suspended') void audioCtx.resume();
    const o = audioCtx.createOscillator();
    const g = audioCtx.createGain();
    o.connect(g);
    g.connect(audioCtx.destination);
    o.frequency.value = 880;
    g.gain.setValueAtTime(0.12, audioCtx.currentTime);
    g.gain.exponentialRampToValueAtTime(0.001, audioCtx.currentTime + 0.6);
    o.start();
    o.stop(audioCtx.currentTime + 0.6);
  } catch {
    /* 无声环境忽略 */
  }
}

// ---- 行操作 ----
function addLine() {
  const t = input.value.trim();
  if (!t) return;
  lines.value.push({ id: nextId++, text: t, done: false, end: null });
  input.value = '';
}

function toggleLine(l: MemoLine) {
  l.done = !l.done;
  if (l.done) l.end = null; // 完成即取消倒计时
}

function removeLine(id: number) {
  lines.value = lines.value.filter((l) => l.id !== id);
}

function fmt(end: number): string {
  const s = Math.max(0, Math.ceil((end - now.value) / 1000));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  const two = (n: number) => String(n).padStart(2, '0');
  return h > 0 ? `${h}:${two(m)}:${two(sec)}` : `${two(m)}:${two(sec)}`;
}

function setCountdown(minutes: number) {
  const l = lines.value.find((x) => x.id === activeId.value);
  if (l && !l.done) {
    l.end = Date.now() + minutes * 60_000;
    now.value = Date.now();
    startTick();
  }
  showTimerPicker.value = false;
  customMinutes.value = '';
}

function onCustom() {
  const m = parseFloat(customMinutes.value);
  if (m > 0) setCountdown(m);
}

// ---- 窗口：拖动 / 缩放 / 关闭 ----
async function onDragDown(e: MouseEvent) {
  if ((e.target as HTMLElement).closest('button')) return;
  await win.startDragging();
}
async function onResizeDown(e: MouseEvent) {
  e.stopPropagation();
  e.preventDefault();
  await win.startResizeDragging('SouthEast');
}

async function requestClose() {
  if (lines.value.length === 0) {
    await win.hide();
    return;
  }
  confirming.value = true;
}
async function doClose() {
  confirming.value = false;
  lines.value = [];
  input.value = '';
  activeId.value = null;
  expiredText.value = null;
  await win.hide();
}
</script>

<template>
  <div class="memo">
    <!-- 顶部拖动条 -->
    <div class="memo-bar" @mousedown.left="onDragDown">
      <span class="memo-title">临时待办</span>
      <button class="bar-close" @click="requestClose" title="关闭（内容将清空）">
        <Icon name="x" :size="14" />
      </button>
    </div>

    <!-- 到时提醒横幅 -->
    <div v-if="expiredText" class="banner">
      ⏰ 到时：{{ expiredText }}
    </div>

    <!-- 清单 -->
    <div class="memo-body" :style="noteStyle">
      <div
        v-for="l in lines"
        :key="l.id"
        class="line"
        :class="{ done: l.done, active: l.id === activeId }"
        @click="activeId = l.id"
      >
        <button class="check" :class="{ on: l.done }" @click.stop="toggleLine(l)" title="勾选完成">
          <Icon v-if="l.done" name="check" :size="12" />
        </button>
        <span class="line-text">{{ l.text }}</span>
        <span v-if="l.end" class="cd">{{ fmt(l.end) }}</span>
        <button class="line-del" @click.stop="removeLine(l.id)" title="删除此行">
          <Icon name="x" :size="12" />
        </button>
      </div>
      <div v-if="lines.length === 0" class="empty">输入一行内容，回车添加待办</div>
    </div>

    <!-- 输入行 -->
    <div class="input-row">
      <input
        v-model="input"
        placeholder="输入后回车添加…"
        @keyup.enter="addLine"
      />
    </div>

    <!-- 右下工具栏：仅五组 -->
    <div class="toolbar" @mousedown.stop>
      <button class="tb" @click="fontSize = Math.max(12, fontSize - 2)" title="减小字号">A−</button>
      <button class="tb" @click="fontSize = Math.min(28, fontSize + 2)" title="增大字号">A+</button>
      <button class="tb" :class="{ on: bold }" @click="bold = !bold" title="加粗">B</button>
      <div class="colors">
        <button
          v-for="c in COLORS"
          :key="c"
          class="dot"
          :class="{ sel: color === c }"
          :style="{ background: c }"
          @click="color = c"
          :title="'文字颜色 ' + c"
        ></button>
      </div>
      <div class="timer-wrap">
        <button class="tb" @click="showTimerPicker = !showTimerPicker" title="为选中行设置倒计时">
          <Icon name="clock" :size="14" />
        </button>
        <div v-if="showTimerPicker" class="picker" @click.stop>
          <p class="picker-tip">{{ activeId == null ? '先点击选择一行' : '为选中行设置倒计时（分钟）' }}</p>
          <div class="picker-quick">
            <button v-for="m in [5, 10, 15, 30, 60]" :key="m" class="pk" @click="setCountdown(m)">{{ m }}</button>
          </div>
          <div class="picker-custom">
            <input v-model="customMinutes" type="number" min="1" placeholder="自定义" @keyup.enter="onCustom" />
            <button class="pk" @click="onCustom">设定</button>
          </div>
        </div>
      </div>
    </div>

    <!-- 缩放手柄 -->
    <div class="resize-handle" @mousedown.left="onResizeDown" title="拖动调整大小"></div>

    <!-- 关闭确认 -->
    <div v-if="confirming" class="cf-overlay" @click.self="confirming = false">
      <div class="cf-card">
        <p>关闭后内容将清空，确定关闭？</p>
        <div class="cf-btns">
          <button class="cf ghost" @click="confirming = false">取消</button>
          <button class="cf danger" @click="doClose">清空并关闭</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.memo {
  position: relative;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #faf9f4;
  border: 1px solid rgba(0, 0, 0, 0.12);
  border-radius: 10px;
  overflow: hidden;
  font-family: 'Microsoft YaHei', 'Segoe UI', system-ui, sans-serif;
}
.memo-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px 6px 12px;
  background: rgba(0, 0, 0, 0.05);
  cursor: grab;
  user-select: none;
  flex-shrink: 0;
}
.memo-bar:active { cursor: grabbing; }
.memo-title { font-size: 12px; font-weight: 600; color: #555; letter-spacing: 1px; }
.bar-close {
  border: none; background: transparent; color: #777;
  cursor: pointer; padding: 3px; border-radius: 5px; display: flex;
}
.bar-close:hover { color: #d0342c; background: rgba(0, 0, 0, 0.07); }

.banner {
  flex-shrink: 0;
  background: #d0342c;
  color: #fff;
  font-size: 12px;
  padding: 5px 10px;
  animation: flash 0.8s ease-in-out 6;
}
@keyframes flash { 50% { opacity: 0.55; } }

.memo-body {
  flex: 1;
  overflow-y: auto;
  scrollbar-width: thin;
  padding: 6px 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.line {
  display: flex;
  align-items: baseline;
  gap: 6px;
  padding: 3px 6px;
  border-radius: 6px;
  cursor: pointer;
}
.line:hover { background: rgba(0, 0, 0, 0.045); }
.line.active { background: rgba(37, 99, 235, 0.1); outline: 1px dashed rgba(37, 99, 235, 0.4); }
.line.done .line-text {
  text-decoration: line-through;
  opacity: 0.4;
}
.line.done .cd { opacity: 0.4; }
.check {
  width: 1.05em; height: 1.05em;
  flex-shrink: 0;
  align-self: center;
  border: 1.5px solid currentColor;
  border-radius: 4px;
  background: transparent;
  color: inherit;
  cursor: pointer;
  display: flex; align-items: center; justify-content: center;
  padding: 0;
}
.check.on { background: currentColor; color: #faf9f4; }
.line-text { flex: 1; min-width: 0; word-break: break-all; line-height: 1.45; }
.cd {
  font-size: 0.72em;
  font-variant-numeric: tabular-nums;
  opacity: 0.65;
  white-space: nowrap;
  margin-left: auto;
}
.line-del {
  border: none; background: transparent; color: #999;
  cursor: pointer; padding: 1px; border-radius: 4px;
  align-self: center; display: flex; opacity: 0;
}
.line:hover .line-del { opacity: 1; }
.line-del:hover { color: #d0342c; }
.empty { opacity: 0.45; font-size: 0.85em; padding: 10px 6px; }

.input-row { flex-shrink: 0; padding: 6px 8px; }
.input-row input {
  width: 100%;
  border: 1px dashed rgba(0, 0, 0, 0.25);
  border-radius: 7px;
  background: #fff;
  padding: 6px 9px;
  font-size: 0.9em;
  font-family: inherit;
  color: inherit;
  outline: none;
}
.input-row input:focus { border-style: solid; border-color: #2563eb; }

.toolbar {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 4px;
  padding: 6px 8px;
  border-top: 1px solid rgba(0, 0, 0, 0.08);
  position: relative;
}
.tb {
  border: 1px solid rgba(0, 0, 0, 0.15);
  background: #fff;
  color: #444;
  border-radius: 6px;
  min-width: 26px;
  height: 24px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-family: inherit;
}
.tb:hover { background: #eef1f6; }
.tb.on { background: #2563eb; color: #fff; border-color: #2563eb; }
.colors { display: flex; gap: 3px; margin: 0 2px; }
.dot {
  width: 15px; height: 15px;
  border-radius: 50%;
  border: 1.5px solid rgba(0, 0, 0, 0.2);
  cursor: pointer; padding: 0;
}
.dot.sel { border-color: #2563eb; box-shadow: 0 0 0 2px rgba(37, 99, 235, 0.3); }

.timer-wrap { position: relative; }
.picker {
  position: absolute;
  bottom: calc(100% + 6px);
  right: 0;
  background: #fff;
  border: 1px solid rgba(0, 0, 0, 0.15);
  border-radius: 9px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.18);
  padding: 8px 10px;
  z-index: 20;
  width: 170px;
}
.picker-tip { margin: 0 0 6px; font-size: 11px; color: #888; }
.picker-quick { display: flex; gap: 4px; flex-wrap: wrap; }
.pk {
  border: 1px solid rgba(0, 0, 0, 0.15);
  background: #f3f4f8;
  border-radius: 6px;
  padding: 3px 9px;
  font-size: 12px;
  cursor: pointer;
  font-family: inherit;
}
.pk:hover { background: #2563eb; color: #fff; }
.picker-custom { display: flex; gap: 4px; margin-top: 6px; }
.picker-custom input {
  flex: 1;
  border: 1px solid rgba(0, 0, 0, 0.15);
  border-radius: 6px;
  padding: 3px 6px;
  font-size: 12px;
  width: 60px;
}

.resize-handle {
  position: absolute;
  right: 0;
  bottom: 0;
  width: 16px;
  height: 16px;
  cursor: nwse-resize;
  z-index: 5;
}
.resize-handle::after {
  content: '';
  position: absolute;
  right: 3px;
  bottom: 3px;
  width: 8px;
  height: 8px;
  border-right: 2px solid rgba(0, 0, 0, 0.25);
  border-bottom: 2px solid rgba(0, 0, 0, 0.25);
  border-radius: 0 0 3px 0;
}

.cf-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 50;
}
.cf-card {
  background: #fff;
  border-radius: 10px;
  padding: 14px 16px;
  width: 82%;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
}
.cf-card p { margin: 0 0 10px; font-size: 13px; color: #333; }
.cf-btns { display: flex; gap: 8px; justify-content: flex-end; }
.cf {
  border: none;
  border-radius: 7px;
  padding: 6px 14px;
  font-size: 12px;
  cursor: pointer;
  font-family: inherit;
}
.cf.ghost { background: transparent; border: 1px solid rgba(0, 0, 0, 0.2); color: #555; }
.cf.danger { background: #d0342c; color: #fff; }
</style>
