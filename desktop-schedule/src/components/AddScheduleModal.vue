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
const sameAsStart = ref(true);
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

const ddlDate = computed(() => endDate.value);

const dayCount = computed(() => {
  const s = new Date(startDate.value);
  const e = new Date(sameAsStart.value ? startDate.value : endDate.value);
  return Math.round((e.getTime() - s.getTime()) / 86400000) + 1;
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
  const finalEnd = sameAsStart.value ? startDate.value : endDate.value;
  if (new Date(finalEnd) < new Date(startDate.value)) {
    error.value = '结束日期不能早于开始日期'; return;
  }
  saving.value = true;
  try {
    const ddlAt = autoDdl.value
      ? ddlHasTime.value && ddlTime.value ? `${ddlDate.value} ${ddlTime.value}` : ddlDate.value
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

      <div class="field">
        <span class="lab"><Icon name="calendar" :size="13" /> 开始日期</span>
        <DatePicker v-model="startDate" />
      </div>

      <div class="field">
        <label class="inline">
          <input type="checkbox" v-model="sameAsStart" />
          仅单日（结束=开始）
        </label>
      </div>

      <div class="field" v-if="!sameAsStart">
        <span class="lab"><Icon name="calendar" :size="13" /> 结束日期</span>
        <DatePicker v-model="endDate" />
        <span class="hint" v-if="dayCount > 1">将自动填充范围内的 {{ dayCount }} 天</span>
      </div>

      <div class="field">
        <label class="inline">
          <input type="checkbox" v-model="hasTime" />
          <Icon name="clock" :size="13" /> 每日时段
        </label>
        <input v-if="hasTime" type="time" v-model="timeOfDay" />
      </div>

      <div class="field">
        <span class="lab"><Icon name="star" :size="13" /> 优先级</span>
        <select v-model.number="priority">
          <option :value="0">普通</option>
          <option :value="1">重要</option>
          <option :value="2">紧急</option>
        </select>
      </div>

      <div class="field">
        <label class="inline">
          <input type="checkbox" v-model="autoDdl" />
          <Icon name="flag" :size="13" /> 设截止 (ddl) = {{ sameAsStart ? '当天' : '结束日期' }}
        </label>
        <template v-if="autoDdl">
          <span class="ddl-date">{{ ddlDate }}</span>
          <label class="inline">
            <input type="checkbox" v-model="ddlHasTime" />
            含时间
          </label>
          <input v-if="ddlHasTime" type="time" v-model="ddlTime" />
        </template>
      </div>

      <label class="field">
        <span class="lab"><Icon name="note" :size="13" /> 备注</span>
        <textarea v-model="note" rows="2" placeholder="可选"></textarea>
      </label>

      <div class="field">
        <span class="lab"><Icon name="image" :size="13" /> 关联文件 / 文件夹</span>
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
}
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
/* 表单内容区有独立 padding */
.modal :deep(.field),
.modal > .field,
.modal > p,
.modal > .actions {
  /* 由 field 自身 margin 控制 */
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
}
.hint {
  font-size: 11px;
  color: #f1c40f;
}
.ddl-date {
  font-size: 12px;
  color: #6c8cff;
  font-weight: 600;
}
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
input[type='checkbox'] {
  width: 15px;
  height: 15px;
  accent-color: #6c8cff;
}
.error {
  color: #e74c3c;
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
  background: #6c8cff;
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
  background: rgba(108,140,255,0.12);
  border: 1px solid rgba(108,140,255,0.35);
  border-radius: 6px;
  padding: 0.4em 0.6em;
}
.attach-name {
  display: inline-flex; align-items: center; gap: 0.3em;
  font-size: 0.8em; color: #6c8cff;
  flex: 1; min-width: 0;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.mini-btn {
  background: transparent; border: none; color: #ccc;
  cursor: pointer; padding: 2px; display: flex;
  border-radius: 4px;
}
.mini-btn:hover { background: rgba(255,255,255,0.1); color: #e74c3c; }
.attach-pick { display: flex; gap: 0.5em; }
.pick-btn {
  flex: 1;
  background: rgba(255,255,255,0.06);
  border: 1px dashed rgba(255,255,255,0.2);
  color: #ccc;
  padding: 0.5em; border-radius: 6px;
  font-size: 0.8em; cursor: pointer; font-family: inherit;
  display: flex; align-items: center; justify-content: center; gap: 0.3em;
}
.pick-btn:hover { background: rgba(255,255,255,0.12); }
</style>
