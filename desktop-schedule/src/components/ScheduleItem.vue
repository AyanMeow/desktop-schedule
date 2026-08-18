<script setup lang="ts">
import { computed, ref } from 'vue';
import { useScheduleStore } from '../stores/schedules';
import { api } from '../api';
import { ddlStatus, isToday } from '../utils/date';
import Icon from './Icon.vue';
import type { Schedule } from '../types';

const props = defineProps<{ schedule: Schedule }>();
const emit = defineEmits<{ deleted: [] }>();

const scheduleStore = useScheduleStore();

const showNote = ref(false);
const confirming = ref(false); // 删除多天确认态

const ddl = computed(() => ddlStatus(props.schedule.ddl_at));

const priorityColor = computed(() => {
  switch (props.schedule.priority) {
    case 2: return 'var(--danger)';
    case 1: return 'var(--warning)';
    default: return 'transparent';
  }
});

// 附件显示名（取路径最后一段）
const attachmentName = computed(() => {
  if (!props.schedule.attachment) return '';
  const p = props.schedule.attachment.replace(/\\/g, '/');
  return p.split('/').filter(Boolean).pop() || p;
});

async function onToggle() { await scheduleStore.toggle(props.schedule.id); }

async function onOpenAttachment() {
  if (!props.schedule.attachment) return;
  try { await api.openAttachment(props.schedule.attachment); }
  catch (e) { alert('打开失败：' + e); }
}

async function onRemove() {
  // 若属于多天日程组，弹确认是否删除全部
  if (props.schedule.group_id) {
    try {
      const count = await api.countGroup(props.schedule.group_id);
      if (count > 1) {
        confirming.value = true;
        return;
      }
    } catch { /* 查询失败按单条删 */ }
  }
  await doDeleteOne();
}

async function doDeleteOne() {
  await scheduleStore.remove(props.schedule.id);
  confirming.value = false;
  emit('deleted');
}

async function doDeleteAll() {
  if (props.schedule.group_id) {
    await scheduleStore.removeGroup(props.schedule.group_id);
  }
  confirming.value = false;
  emit('deleted');
}
</script>

<template>
  <div class="item" :class="{ done: schedule.completed }">
    <div class="pribar" :style="{ background: priorityColor }"></div>

    <button class="check" @click="onToggle" :title="schedule.completed ? '标记未完成' : '标记完成'">
      <Icon v-if="schedule.completed" name="check" :size="14" />
    </button>

    <div class="body" @click="showNote = !showNote">
      <div class="head">
        <span class="title">{{ schedule.title }}</span>
        <span v-if="schedule.time_of_day" class="time">
          <Icon name="clock" :size="12" /> {{ schedule.time_of_day }}
        </span>
      </div>
      <div class="meta">
        <!-- 倒计时按今天计算，仅在查看当日日程时显示，避免其他日期下产生误解 -->
        <span v-if="schedule.has_ddl && !schedule.completed && isToday(schedule.date)" class="ddl" :class="'ddl-lv-' + ddl.level">
          <Icon name="flag" :size="11" /> {{ ddl.label }}
        </span>
        <span v-if="schedule.priority >= 1" class="pri-icon" :style="{ color: priorityColor }">
          <Icon name="star" :size="11" />
        </span>
        <button
          v-if="schedule.attachment"
          class="attach-btn"
          @click.stop="onOpenAttachment"
          :title="'打开：' + schedule.attachment"
        >
          <Icon name="image" :size="11" /> {{ attachmentName }}
        </button>
      </div>
      <div v-if="showNote && schedule.note" class="note">{{ schedule.note }}</div>

      <!-- 删除多天确认条 -->
      <div v-if="confirming" class="confirm" @click.stop>
        <span>该日程涉及多天，删除范围？</span>
        <div class="confirm-btns">
          <button class="cbtn all" @click="doDeleteAll">删除全部</button>
          <button class="cbtn one" @click="doDeleteOne">仅此天</button>
          <button class="cbtn cancel" @click="confirming = false">取消</button>
        </div>
      </div>
    </div>

    <button class="del" @click="onRemove" title="删除" v-if="!confirming">
      <Icon name="trash" :size="14" />
    </button>
  </div>
</template>

<style scoped>
.item {
  display: flex; align-items: flex-start; gap: 0.5em;
  padding: 0.5em 0.5em 0.5em 0;
  border-radius: 7px; transition: background 0.15s;
}
.item:hover { background: rgba(128, 128, 128, 0.1); }
.item.done .title { text-decoration: line-through; opacity: 0.5; }
.item.done .time, .item.done .meta { opacity: 0.4; }
.pribar {
  width: 3px; align-self: stretch;
  border-radius: 2px; min-height: 1.2em;
}
.check {
  flex-shrink: 0;
  width: 1.1em; height: 1.1em;
  border-radius: 5px;
  border: 1.5px solid currentColor;
  background: transparent;
  color: inherit; cursor: pointer;
  display: flex; align-items: center; justify-content: center;
  margin-top: 0.1em;
}
.item.done .check { background: var(--accent); border-color: var(--accent); color: #fff; }
.body { flex: 1; min-width: 0; cursor: pointer; }
.head { display: flex; align-items: baseline; gap: 0.5em; flex-wrap: wrap; }
.title { font-size: 0.9em; word-break: break-word; }
.time {
  display: inline-flex; align-items: center; gap: 0.2em;
  font-size: 0.75em; opacity: 0.65; flex-shrink: 0;
}
.meta { display: flex; gap: 0.6em; align-items: center; margin-top: 0.15em; flex-wrap: wrap; }
.ddl {
  display: inline-flex; align-items: center; gap: 0.2em;
  font-size: 0.72em; font-weight: 600;
}
.ddl-lv-overdue { color: var(--ddl-overdue); }
.ddl-lv-le1 { color: var(--ddl-le1); }
.ddl-lv-le3 { color: var(--ddl-le3); }
.ddl-lv-le7 { color: var(--ddl-le7); }
.ddl-lv-gt7 { color: var(--ddl-gt7); opacity: 0.8; }
.pri-icon { display: inline-flex; }
.attach-btn {
  display: inline-flex; align-items: center; gap: 0.2em;
  background: var(--accent-soft);
  border: 1px solid var(--accent);
  color: var(--accent);
  padding: 0.1em 0.4em; border-radius: 4px;
  font-size: 0.68em; cursor: pointer; font-family: inherit;
  max-width: 12em; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.attach-btn:hover { background: var(--accent); color: #fff; }
.note {
  font-size: 0.78em; opacity: 0.7;
  margin-top: 0.3em; padding: 0.35em 0.45em;
  background: rgba(0,0,0,0.2); border-radius: 5px;
  line-height: 1.5; word-break: break-word;
}
.confirm {
  margin-top: 0.4em; padding: 0.4em 0.5em;
  background: rgba(231, 76, 60, 0.12);
  border: 1px solid rgba(231, 76, 60, 0.4);
  border-radius: 6px;
  font-size: 0.72em;
}
.confirm span { display: block; margin-bottom: 0.3em; opacity: 0.9; }
.confirm-btns { display: flex; gap: 0.4em; }
.cbtn {
  border: none; border-radius: 4px;
  padding: 0.25em 0.6em; cursor: pointer;
  font-size: 1em; font-family: inherit;
}
.cbtn.all { background: var(--danger); color: #fff; }
.cbtn.one { background: var(--warning); color: #fff; }
.cbtn.cancel { background: rgba(128,128,128,0.25); color: inherit; }
.del {
  flex-shrink: 0; background: transparent; border: none;
  color: currentColor; opacity: 0.35; cursor: pointer;
  width: 1.3em; height: 1.3em; border-radius: 5px;
  display: flex; align-items: center; justify-content: center;
}
.del:hover { opacity: 1; color: var(--danger); background: var(--accent-soft); }
</style>
