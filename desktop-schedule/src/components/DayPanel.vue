<script setup lang="ts">
import { ref, computed } from 'vue';
import { useScheduleStore } from '../stores/schedules';
import ScheduleItem from './ScheduleItem.vue';
import EditScheduleModal from './EditScheduleModal.vue';
import Icon from './Icon.vue';
import { isToday, parseISO } from '../utils/date';
import type { Schedule } from '../types';

const props = defineProps<{ dateISO: string }>();
const emit = defineEmits<{ add: [] }>();
const scheduleStore = useScheduleStore();

// 正在编辑的日程（弹窗状态）
const editing = ref<Schedule | null>(null);

// ---- 排序：ddl / priority，激活后再按一次切换升降序；均默认升序 ----
type SortKey = 'none' | 'ddl' | 'priority';
const sortBy = ref<SortKey>('none');
const sortDesc = ref(false);

function toggleSort(key: Exclude<SortKey, 'none'>) {
  if (sortBy.value !== key) {
    sortBy.value = key;
    sortDesc.value = false;
  } else {
    sortDesc.value = !sortDesc.value;
  }
}

const items = computed<Schedule[]>(() => {
  const list = [...(scheduleStore.byDate.get(props.dateISO) || [])];
  // 排序键之前的默认顺序：未完成在前，再按时段
  const baseCompare = (a: Schedule, b: Schedule) => {
    if (a.completed !== b.completed) return a.completed ? 1 : -1;
    if (a.time_of_day && b.time_of_day) return a.time_of_day.localeCompare(b.time_of_day);
    if (a.time_of_day) return -1;
    if (b.time_of_day) return 1;
    return a.created_at.localeCompare(b.created_at);
  };
  const dir = sortDesc.value ? -1 : 1;
  if (sortBy.value === 'ddl') {
    // 有截止的按时间排（字符串比较即可：YYYY-MM-DD[ HH:MM] 字典序=时间序），无截止恒排末尾
    list.sort((a, b) => {
      if (a.completed !== b.completed) return a.completed ? 1 : -1;
      if (!a.ddl_at && !b.ddl_at) return baseCompare(a, b);
      if (!a.ddl_at) return 1;
      if (!b.ddl_at) return -1;
      const c = a.ddl_at.localeCompare(b.ddl_at) * dir;
      return c !== 0 ? c : baseCompare(a, b);
    });
  } else if (sortBy.value === 'priority') {
    // 升序=普通→重要→紧急；降序反之
    list.sort((a, b) => {
      if (a.completed !== b.completed) return a.completed ? 1 : -1;
      const c = (a.priority - b.priority) * dir;
      return c !== 0 ? c : baseCompare(a, b);
    });
  } else {
    list.sort(baseCompare);
  }
  return list;
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
      <div class="head-actions">
        <button
          class="sort-btn"
          :class="{ active: sortBy === 'ddl' }"
          :title="sortBy === 'ddl' ? (sortDesc ? '按截止排序（降序）' : '按截止排序（升序）') : '按截止排序'"
          @click="toggleSort('ddl')"
        >
          <Icon name="clock" :size="14" />
          <span v-if="sortBy === 'ddl'" class="dir">{{ sortDesc ? '↓' : '↑' }}</span>
        </button>
        <button
          class="sort-btn"
          :class="{ active: sortBy === 'priority' }"
          :title="sortBy === 'priority' ? (sortDesc ? '按重要程度排序（降序）' : '按重要程度排序（升序）') : '按重要程度排序'"
          @click="toggleSort('priority')"
        >
          <Icon name="alert" :size="14" />
          <span v-if="sortBy === 'priority'" class="dir">{{ sortDesc ? '↓' : '↑' }}</span>
        </button>
        <button class="add-btn" @click="emit('add')" title="添加日程">
          <Icon name="plus" :size="16" />
        </button>
      </div>
    </div>
    <div class="list">
      <ScheduleItem
        v-for="s in items"
        :key="s.id"
        :schedule="s"
        @edit="editing = $event"
      />
      <div v-if="items.length === 0" class="empty">
        <span>这一天还没有日程</span>
        <button class="add-link" @click="emit('add')">
          <Icon name="plus" :size="12" /> 添加一个
        </button>
      </div>
    </div>

    <!-- 编辑日程弹窗 -->
    <EditScheduleModal
      v-if="editing"
      :schedule="editing"
      @close="editing = null"
      @saved="editing = null"
    />
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
/* 排序按钮：与添加按钮同行，激活高亮 + 方向箭头 */
.head-actions { display: flex; align-items: center; gap: 0.35em; }
.sort-btn {
  background: rgba(128, 128, 128, 0.15); border: none; color: inherit;
  width: 1.3em; height: 1.3em; border-radius: 5px; cursor: pointer;
  display: inline-flex; align-items: center; justify-content: center;
  position: relative; opacity: 0.75;
}
.sort-btn:hover { opacity: 1; background: rgba(128, 128, 128, 0.3); }
.sort-btn.active {
  background: var(--accent-soft);
  border: 1px solid var(--accent);
  color: var(--accent);
  opacity: 1;
}
.sort-btn .dir {
  position: absolute;
  right: -1px;
  bottom: -2px;
  font-size: 0.55em;
  font-weight: 700;
  line-height: 1;
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
