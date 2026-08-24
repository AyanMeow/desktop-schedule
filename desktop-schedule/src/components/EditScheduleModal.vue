<script setup lang="ts">
// 编辑已有日程：标题/时段/优先级/DDL/备注/附件（日期不可改——多天日程组的
// 日期结构由创建时的范围填充决定，单改一条日期会破坏组一致性）
import { ref, computed } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { useScheduleStore } from '../stores/schedules';
import Icon from './Icon.vue';
import type { Schedule, UpdateSchedule } from '../types';

const props = defineProps<{ schedule: Schedule }>();
const emit = defineEmits<{ close: []; saved: [] }>();
const store = useScheduleStore();

// 预填：从现有日程解析初始值
const title = ref(props.schedule.title);
const hasTime = ref(!!props.schedule.time_of_day);
const timeOfDay = ref(props.schedule.time_of_day || '');
const note = ref(props.schedule.note || '');
const priority = ref(props.schedule.priority);
const autoDdl = ref(props.schedule.has_ddl);
// ddl_at 形如 'YYYY-MM-DD HH:MM' 或 'YYYY-MM-DD'
const ddlDate = ref(props.schedule.ddl_at ? props.schedule.ddl_at.slice(0, 10) : props.schedule.date);
const ddlHasTime = ref(!!(props.schedule.ddl_at && props.schedule.ddl_at.includes(' ')));
const ddlTime = ref(
  props.schedule.ddl_at && props.schedule.ddl_at.includes(' ')
    ? props.schedule.ddl_at.split(' ')[1]
    : '23:59'
);
const attachment = ref<string | null>(props.schedule.attachment);
const saving = ref(false);
const error = ref('');

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
  saving.value = true;
  try {
    const ddlAt = autoDdl.value
      ? ddlHasTime.value && ddlTime.value ? `${ddlDate.value} ${ddlTime.value}` : ddlDate.value
      : null;
    const update: UpdateSchedule = {
      title: title.value.trim(),
      time_of_day: hasTime.value && timeOfDay.value ? timeOfDay.value : null,
      note: note.value.trim() || null,
      priority: priority.value,
      ddl_at: ddlAt,
      attachment: attachment.value,
    };
    await store.update(props.schedule.id, update);
    emit('saved');
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
        <h3><Icon name="note" :size="18" /> 编辑日程</h3>
        <button class="close-btn" @click="emit('close')" title="关闭"><Icon name="x" :size="18" /></button>
      </div>

      <label class="field">
        <span class="lab"><Icon name="note" :size="13" /> 标题 *</span>
        <input v-model="title" placeholder="例如：晨跑" @keyup.enter="submit" />
      </label>

      <div class="field">
        <span class="lab"><Icon name="calendar" :size="13" /> 日期</span>
        <span class="fixed-date">{{ schedule.date }}</span>
        <span v-if="schedule.group_id" class="hint">该日程属于多天日程组，本次修改仅影响这一条（其余日期不变）</span>
        <span v-else class="hint">如需变更日期请删除后重新创建</span>
      </div>

      <div class="field">
        <label class="inline">
          <input type="checkbox" v-model="hasTime" />
          <Icon name="clock" :size="13" /> 时段
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
          <Icon name="flag" :size="13" /> 设截止 (ddl)
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
          {{ saving ? '保存中…' : '保存' }}
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
.fixed-date {
  font-size: 13px;
  color: var(--accent);
  font-weight: 600;
}
.hint {
  font-size: 11px;
  opacity: 0.6;
}
.ddl-date {
  font-size: 12px;
  color: var(--accent);
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
  accent-color: var(--accent);
}
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
