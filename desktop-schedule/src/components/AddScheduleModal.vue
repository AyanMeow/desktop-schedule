<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { useScheduleStore } from '../stores/schedules';
import { today } from '../utils/date';
import DatePicker from './DatePicker.vue';
import Icon from './Icon.vue';
import type { NewSchedule } from '../types';

const props = withDefaults(defineProps<{
  presetDate?: string | null;
}>(), { presetDate: null });
const store = useScheduleStore();
const emit = defineEmits<{ close: []; added: [dateISO: string] }>();

const title = ref('');
const startDate = ref(props.presetDate || today());
const endDate = ref(props.presetDate || today());
const mode = ref<'single' | 'multi'>('single');
const repeat = ref<'daily' | 'interval'>('daily');
const intervalDays = ref(2);
const timeOfDay = ref('');
const hasTime = ref(false);
const note = ref('');
const priority = ref(0);
const autoDdl = ref(true);
const ddlHasTime = ref(false);
const ddlTime = ref('23:59');
const attachment = ref<string | null>(null);
const saving = ref(false);
const error = ref('');

watch(() => props.presetDate, (v) => {
  if (v) { startDate.value = v; endDate.value = v; }
});

// 切到多天时若结束早于开始，先把结束拉齐到开始
watch(mode, (m) => {
  if (m === 'multi' && new Date(endDate.value) < new Date(startDate.value)) {
    endDate.value = startDate.value;
  }
});

function setIntervalDays(v: number | null) {
  const n = Math.round(Number(v));
  if (!Number.isFinite(n)) return;
  intervalDays.value = Math.min(30, Math.max(2, n));
}

const dayCount = computed(() => {
  const s = new Date(startDate.value);
  const e = new Date(endDate.value);
  return Math.round((e.getTime() - s.getTime()) / 86400000) + 1;
});

// 间隔模式下实际出现的日期列表（含首尾判断）
const occurrenceDays = computed(() => {
  if (mode.value !== 'multi' || repeat.value !== 'interval') return [];
  const e = new Date(endDate.value).getTime();
  const out: Date[] = [];
  for (let t = new Date(startDate.value).getTime(); t <= e; t += intervalDays.value * 86400000) {
    out.push(new Date(t));
  }
  return out;
});

function fmtShort(d: Date): string {
  const m = `${d.getMonth() + 1}`.padStart(2, '0');
  const day = `${d.getDate()}`.padStart(2, '0');
  return `${m}-${day}`;
}

const fillPreview = computed(() => {
  if (mode.value !== 'multi') return '';
  if (repeat.value === 'daily') {
    return `范围内共 ${dayCount.value} 天，每天都有一条`;
  }
  const occ = occurrenceDays.value;
  if (occ.length === 0) return '';
  const head = occ.slice(0, 3).map(fmtShort).join('、');
  const tail = occ.length > 3 ? '…' : '';
  return `每 ${intervalDays.value} 天一次，共 ${occ.length} 次：${head}${tail}`;
});

// 截止日期指向：单日=当天，多天每天=结束日期，多天间隔=各次出现当天
const ddlTargetText = computed(() => {
  if (mode.value === 'single') return '当天';
  return repeat.value === 'daily' ? '结束日期' : '各次当天';
});

const attachmentName = computed(() => {
  if (!attachment.value) return '';
  const p = attachment.value.replace(/\\/g, '/');
  return p.split('/').filter(Boolean).pop() || attachment.value;
});

async function pickFile() {
  const sel = await open({ multiple: false });
  if (typeof sel === 'string') attachment.value = sel;
}
async function pickDir() {
  const sel = await open({ directory: true, multiple: false });
  if (typeof sel === 'string') attachment.value = sel;
}
function clearAttachment() { attachment.value = null; }

async function submit() {
  error.value = '';
  if (!title.value.trim()) { error.value = '请填写标题'; return; }
  if (new Date(endDate.value) < new Date(startDate.value)) {
    error.value = '结束日期不能早于开始日期'; return;
  }
  saving.value = true;
  try {
    const finalEnd = mode.value === 'single' ? startDate.value : endDate.value;
    // 间隔模式下 ddl_at 传结束日期占位，后端按各次出现日期替换（时间部分保留）
    const ddlBase = mode.value === 'single' ? startDate.value : finalEnd;
    const ddlAt = autoDdl.value
      ? ddlHasTime.value && ddlTime.value ? `${ddlBase} ${ddlTime.value}` : ddlBase
      : null;
    const input: NewSchedule = {
      title: title.value.trim(),
      start_date: startDate.value,
      end_date: finalEnd,
      time_of_day: hasTime.value && timeOfDay.value ? timeOfDay.value : null,
      note: note.value.trim() || null,
      priority: priority.value,
      ddl_at: ddlAt,
      attachment: attachment.value,
      interval_days: mode.value === 'multi' && repeat.value === 'interval' ? intervalDays.value : null,
    };
    await store.create(input);
    emit('added', finalEnd);
    emit('close');
  } catch (e) {
    error.value = String(e);
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal">
      <div class="modal-head">
        <h3><Icon name="plus" :size="18" /> 添加日程</h3>
        <button class="close-btn" @click="emit('close')" title="关闭"><Icon name="x" :size="18" /></button>
      </div>

      <label class="field">
        <span class="lab"><Icon name="note" :size="13" /> 标题 *</span>
        <input v-model="title" placeholder="例如：晨跑" @keyup.enter="submit" />
      </label>

      <h4 class="sec"><Icon name="calendar" :size="12" /> 日期与重复</h4>
      <div class="seg">
        <button type="button" :class="{ sel: mode === 'single' }" @click="mode = 'single'">单日</button>
        <button type="button" :class="{ sel: mode === 'multi' }" @click="mode = 'multi'">多天</button>
      </div>

      <div v-if="mode === 'single'" class="field">
        <span class="lab">日期</span>
        <DatePicker v-model="startDate" />
      </div>

      <template v-else>
        <div class="grid2">
          <div class="field">
            <span class="lab">开始</span>
            <DatePicker v-model="startDate" />
          </div>
          <div class="field">
            <span class="lab">结束</span>
            <DatePicker v-model="endDate" />
          </div>
        </div>
        <div class="row">
          <div class="seg repeat">
            <button type="button" :class="{ sel: repeat === 'daily' }" @click="repeat = 'daily'">每天</button>
            <button type="button" :class="{ sel: repeat === 'interval' }" @click="repeat = 'interval'">每隔…天</button>
          </div>
          <div class="stepper" v-if="repeat === 'interval'">
            <button type="button" title="减" @click="setIntervalDays(intervalDays - 1)">−</button>
            <input
              type="number" min="2" max="30" :value="intervalDays"
              @change="(e) => setIntervalDays(+(e.target as HTMLInputElement).value)"
            />
            <button type="button" title="加" @click="setIntervalDays(intervalDays + 1)">+</button>
            <span class="unit">天</span>
          </div>
        </div>
        <p class="preview" v-if="fillPreview">{{ fillPreview }}</p>
      </template>

      <h4 class="sec"><Icon name="star" :size="12" /> 属性</h4>
      <div class="row">
        <label class="inline">
          <input type="checkbox" class="switch" v-model="hasTime" />
          <Icon name="clock" :size="13" /> 每日时段
        </label>
        <input v-if="hasTime" type="time" v-model="timeOfDay" class="time-input" />
      </div>
      <div class="field">
        <span class="lab">优先级</span>
        <div class="seg prio">
          <button type="button" class="p0" :class="{ sel: priority === 0 }" @click="priority = 0">普通</button>
          <button type="button" class="p1" :class="{ sel: priority === 1 }" @click="priority = 1">重要</button>
          <button type="button" class="p2" :class="{ sel: priority === 2 }" @click="priority = 2">紧急</button>
        </div>
      </div>
      <div class="row">
        <label class="inline">
          <input type="checkbox" class="switch" v-model="autoDdl" />
          <Icon name="flag" :size="13" /> 设截止
        </label>
        <span class="ddl-date" v-if="autoDdl">= {{ ddlTargetText }}</span>
        <template v-if="autoDdl">
          <label class="inline">
            <input type="checkbox" class="switch" v-model="ddlHasTime" />
            含时间
          </label>
          <input v-if="ddlHasTime" type="time" v-model="ddlTime" class="time-input" />
        </template>
      </div>

      <h4 class="sec"><Icon name="image" :size="12" /> 备注 / 附件</h4>
      <label class="field">
        <textarea v-model="note" rows="2" placeholder="备注（可选）"></textarea>
      </label>
      <div class="field">
        <div v-if="attachment" class="attach-row">
          <span class="attach-name" :title="attachment"><Icon name="image" :size="13" /> {{ attachmentName }}</span>
          <button type="button" class="mini-btn" @click="clearAttachment"><Icon name="x" :size="13" /></button>
        </div>
        <div v-else class="attach-pick">
          <button type="button" class="pick-btn" @click="pickFile"><Icon name="note" :size="14" /> 选文件</button>
          <button type="button" class="pick-btn" @click="pickDir"><Icon name="image" :size="14" /> 选文件夹</button>
        </div>
      </div>

      <p class="error" v-if="error">{{ error }}</p>

      <div class="actions">
        <button class="btn ghost" @click="emit('close')">取消</button>
        <button class="btn primary" :disabled="saving" @click="submit">
          {{ saving ? '保存中…' : '添加' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  backdrop-filter: blur(2px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  padding: 12px;
}
/* 模态背景跟随主题：半透明 + 毛玻璃，文字用 currentColor 继承 */
.modal {
  background: color-mix(in srgb, var(--modal-bg, #2a2c3a) 92%, transparent);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid rgba(128, 128, 128, 0.25);
  border-radius: 14px;
  padding: 0;
  width: 80vw;
  max-width: 480px;
  max-height: 80vh;
  overflow-y: auto;
  color: inherit;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.6);
  scrollbar-width: none;
  -ms-overflow-style: none;
}
.modal::-webkit-scrollbar { display: none; }
.modal-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px 10px;
  border-bottom: 1px solid rgba(128, 128, 128, 0.2);
  position: sticky;
  top: 0;
  background: inherit;
  z-index: 2;
}
h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  display: inline-flex;
  align-items: center;
  gap: 0.4em;
}
.close-btn {
  background: transparent;
  border: none;
  color: inherit;
  opacity: 0.5;
  cursor: pointer;
  padding: 4px;
  border-radius: 6px;
  display: flex;
}
.close-btn:hover { opacity: 1; background: rgba(128, 128, 128, 0.2); }
/* 分区标题 */
.sec {
  margin: 14px 16px 8px;
  font-size: 11px;
  font-weight: 600;
  opacity: 0.65;
  letter-spacing: 0.5px;
  display: flex;
  align-items: center;
  gap: 0.35em;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin: 0 16px 12px;
}
.modal-head + .field { margin-top: 12px; }
.lab {
  font-size: 12px;
  opacity: 0.75;
  display: inline-flex;
  align-items: center;
  gap: 0.3em;
}
.inline {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  cursor: pointer;
  white-space: nowrap;
}
.ddl-date {
  font-size: 12px;
  color: var(--accent);
  font-weight: 600;
  white-space: nowrap;
}
/* 横向紧凑行（分段控件/勾选项组合） */
.row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  margin: 0 16px 12px;
}
/* 分段控件 */
.seg {
  display: flex;
  gap: 2px;
  background: rgba(128, 128, 128, 0.12);
  border: 1px solid rgba(128, 128, 128, 0.2);
  border-radius: 7px;
  padding: 2px;
}
.seg button {
  flex: 1;
  background: transparent;
  border: none;
  color: inherit;
  opacity: 0.75;
  font-size: 12px;
  padding: 5px 12px;
  border-radius: 5px;
  cursor: pointer;
  font-family: inherit;
  white-space: nowrap;
}
.seg button:hover { opacity: 1; background: rgba(128, 128, 128, 0.15); }
.seg button.sel { background: var(--accent); color: #fff; opacity: 1; }
/* 优先级三档配色：中性 / 主题色 / 危险色 */
.seg.prio button.p0.sel { background: rgba(128, 128, 128, 0.55); }
.seg.prio button.p2.sel { background: var(--danger, #d0342c); }
/* 重复方式分段不占满整行，与步进器同行 */
.seg.repeat { flex: 0 1 auto; }
.seg.repeat button { padding: 5px 10px; }
.grid2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  margin: 0 16px 12px;
}
.grid2 .field { margin: 0; }
/* 间隔天数步进器 */
.stepper { display: inline-flex; align-items: center; gap: 4px; }
.stepper button {
  width: 24px;
  height: 24px;
  border-radius: 5px;
  border: 1px solid rgba(128, 128, 128, 0.25);
  background: rgba(128, 128, 128, 0.12);
  color: inherit;
  cursor: pointer;
  font-size: 13px;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.stepper button:hover { background: rgba(128, 128, 128, 0.25); }
.stepper input { width: 46px; text-align: center; padding: 4px 2px; }
.stepper input::-webkit-outer-spin-button,
.stepper input::-webkit-inner-spin-button { -webkit-appearance: none; margin: 0; }
.stepper input[type='number'] { -moz-appearance: textfield; appearance: textfield; }
.stepper .unit { font-size: 12px; opacity: 0.7; }
/* 填充预览行 */
.preview {
  margin: -4px 16px 12px;
  font-size: 11.5px;
  color: var(--accent);
  background: var(--accent-soft);
  border-radius: 6px;
  padding: 6px 9px;
}
.time-input { width: 100px; }
input[type='text'],
input:not([type]),
input[type='time'],
select,
textarea {
  background: rgba(128, 128, 128, 0.15);
  border: 1px solid rgba(128, 128, 128, 0.25);
  border-radius: 6px;
  padding: 7px 9px;
  color: inherit;
  font-size: 13px;
  font-family: inherit;
  width: 100%;
}
input::placeholder, textarea::placeholder { color: currentColor; opacity: 0.4; }
textarea { resize: vertical; }
/* 切换开关（与设置面板同款）：滑块左移右移，选中态主题色 */
input[type='checkbox'].switch {
  -webkit-appearance: none;
  appearance: none;
  width: 34px; height: 20px;
  border-radius: 10px;
  background: rgba(128, 128, 128, 0.35);
  position: relative;
  cursor: pointer;
  transition: background 0.18s;
  flex-shrink: 0;
  margin: 0;
}
input[type='checkbox'].switch::after {
  content: '';
  position: absolute;
  top: 2px; left: 2px;
  width: 16px; height: 16px;
  border-radius: 50%;
  background: #fff;
  transition: left 0.18s;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.35);
}
input[type='checkbox'].switch:checked { background: var(--accent); }
input[type='checkbox'].switch:checked::after { left: 16px; }
input[type='checkbox'].switch:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
.error {
  color: var(--danger);
  font-size: 12px;
  margin: 4px 16px;
}
.actions {
  display: flex;
  gap: 10px;
  justify-content: flex-end;
  margin: 6px 16px 16px;
  padding-top: 10px;
  border-top: 1px solid rgba(128, 128, 128, 0.15);
  position: sticky;
  bottom: 0;
  background: inherit;
}
.btn {
  padding: 8px 18px;
  border-radius: 6px;
  border: none;
  cursor: pointer;
  font-size: 13px;
  font-family: inherit;
}
.btn.primary {
  background: var(--accent);
  color: #fff;
  display: inline-flex;
  align-items: center;
  gap: 0.3em;
}
.btn.ghost {
  background: transparent;
  color: inherit;
  opacity: 0.8;
  border: 1px solid rgba(128, 128, 128, 0.3);
}
.btn:disabled { opacity: 0.5; }
.attach-row {
  display: flex; align-items: center; gap: 0.5em;
  background: var(--accent-soft);
  border: 1px solid var(--accent);
  border-radius: 6px;
  padding: 0.4em 0.6em;
}
.attach-name {
  display: inline-flex; align-items: center; gap: 0.3em;
  font-size: 0.8em; color: var(--accent);
  flex: 1; min-width: 0;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.mini-btn {
  background: transparent; border: none; color: currentColor;
  cursor: pointer; padding: 2px; display: flex;
  border-radius: 4px; opacity: 0.7;
}
.mini-btn:hover { background: var(--accent-soft); color: var(--danger); opacity: 1; }
.attach-pick { display: flex; gap: 0.5em; }
.pick-btn {
  flex: 1;
  background: var(--accent-soft);
  border: 1px dashed var(--accent);
  color: var(--accent);
  padding: 0.5em; border-radius: 6px;
  font-size: 0.8em; cursor: pointer; font-family: inherit;
  display: flex; align-items: center; justify-content: center; gap: 0.3em;
}
.pick-btn:hover { background: var(--accent); color: #fff; }
</style>
