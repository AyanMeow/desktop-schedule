<script setup lang="ts">
import { computed } from 'vue';
import { useScheduleStore } from '../stores/schedules';
import ScheduleItem from './ScheduleItem.vue';
import Icon from './Icon.vue';
import { isToday, parseISO } from '../utils/date';
import type { Schedule } from '../types';

const props = defineProps<{ dateISO: string }>();
const emit = defineEmits<{ add: [] }>();
const scheduleStore = useScheduleStore();

const items = computed<Schedule[]>(() => {
  const list = scheduleStore.byDate.get(props.dateISO) || [];
  return [...list].sort((a, b) => {
    if (a.completed !== b.completed) return a.completed ? 1 : -1;
    if (a.time_of_day && b.time_of_day) return a.time_of_day.localeCompare(b.time_of_day);
    if (a.time_of_day) return -1;
    if (b.time_of_day) return 1;
    return a.created_at.localeCompare(b.created_at);
  });
});

const dayLabel = computed(() => {
  const d = parseISO(props.dateISO);
  const wd = '日一二三四五六'[d.getDay()];
  return `${d.getMonth() + 1}月${d.getDate()}日 周${wd}${isToday(props.dateISO) ? ' · 今天' : ''}`;
});
</script>

<template>
  <div class="day-panel">
    <div class="panel-head">
      <span>{{ dayLabel }}</span>
      <button class="add-btn" @click="emit('add')" title="添加日程">
        <Icon name="plus" :size="16" />
      </button>
    </div>
    <div class="list">
      <ScheduleItem v-for="s in items" :key="s.id" :schedule="s" />
      <div v-if="items.length === 0" class="empty">
        <span>这一天还没有日程</span>
        <button class="add-link" @click="emit('add')">
          <Icon name="plus" :size="12" /> 添加一个
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.day-panel {
  display: flex; flex-direction: column; gap: 0.2em;
  margin-top: 0.4em;
}
.panel-head {
  display: flex; align-items: center; justify-content: space-between;
  padding: 0.4em 0.2em;
  border-bottom: 1px solid rgba(128,128,128,0.2);
  position: sticky; top: 0;
  background: inherit; z-index: 1;
}
.panel-head span { font-size: 0.82em; font-weight: 600; }
.add-btn {
  background: var(--accent); border: none; color: #fff;
  width: 1.3em; height: 1.3em; border-radius: 5px;
  cursor: pointer; display: flex; align-items: center; justify-content: center;
}
.list { display: flex; flex-direction: column; gap: 0.05em; }
.empty {
  display: flex; flex-direction: column; align-items: center; gap: 0.5em;
  padding: 1em 0; font-size: 0.8em; opacity: 0.5;
}
.add-link {
  background: none; border: none; color: var(--accent); cursor: pointer;
  font-size: 1em; font-family: inherit;
  display: inline-flex; align-items: center; gap: 0.3em;
}
</style>
