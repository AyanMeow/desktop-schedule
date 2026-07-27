<script setup lang="ts">
import { computed } from 'vue';
import { useScheduleStore } from '../stores/schedules';
import { useConfigStore } from '../stores/config';
import { eachDay, toISO, isToday, WEEKDAY_CN, ddlStatus } from '../utils/date';
import Icon from './Icon.vue';
import type { Schedule } from '../types';

const scheduleStore = useScheduleStore();
const configStore = useConfigStore();

// 当前展开的日期（null=未展开）；外部 v-model
const props = defineProps<{ expandedDate: string | null }>();
const emit = defineEmits<{ 'update:expandedDate': [string | null] }>();

const colors = computed(() => configStore.config.ddl_colors);

// 网格的所有日期
const days = computed(() => {
  const [start, end] = scheduleStore.currentRange;
  return eachDay(start, end);
});

// 表头星期（按 weekStart 排列）
const weekdays = computed(() => {
  const offset = scheduleStore.weekStart === 'monday' ? 1 : 0;
  const arr: string[] = [];
  for (let i = 0; i < 7; i++) {
    arr.push(WEEKDAY_CN[(i + offset) % 7]);
  }
  return arr;
});

// 范围标题：2026年7月 / 7月20日-7月26日
const rangeTitle = computed(() => {
  const [start, end] = scheduleStore.currentRange;
  const s = new Date(start);
  const e = new Date(end);
  if (scheduleStore.viewRange === 'month') {
    return `${s.getFullYear()}年${s.getMonth() + 1}月`;
  }
  const fmt = (d: Date) => `${d.getMonth() + 1}月${d.getDate()}日`;
  return `${fmt(s)} - ${fmt(e)}`;
});

function schedulesOn(dateISO: string): Schedule[] {
  return scheduleStore.byDate.get(dateISO) || [];
}

function ddlDotColor(s: Schedule): string {
  const st = ddlStatus(s.ddl_at, colors.value);
  return st.color;
}

function pendingCount(dateISO: string): number {
  return schedulesOn(dateISO).filter((s) => !s.completed).length;
}

function isExpanded(dateISO: string): boolean {
  return props.expandedDate === dateISO;
}

function onCellClick(dateISO: string) {
  emit('update:expandedDate', isExpanded(dateISO) ? null : dateISO);
}

// 月份视图时，月初可能不在周首对齐，用 padding 补齐网格
const gridDays = computed(() => {
  const ds = days.value;
  if (scheduleStore.viewRange === 'month') {
    // 月视图：补齐前面空白使第一天对齐正确星期列
    const first = new Date(ds[0]);
    const offset = scheduleStore.weekStart === 'monday' ? 1 : 0;
    const lead = (first.getDay() - offset + 7) % 7;
    const padded: (Date | null)[] = [];
    for (let i = 0; i < lead; i++) padded.push(null);
    for (const d of ds) padded.push(d);
    return padded;
  }
  return ds as (Date | null)[];
});
</script>

<template>
  <div class="calendar">
    <!-- 月份导航 -->
    <div class="navbar">
      <button class="nav-btn" @click="scheduleStore.navigate(-1)" title="上一个">
        <Icon name="chevron-left" :size="16" />
      </button>
      <span class="title">{{ rangeTitle }}</span>
      <button class="nav-btn" @click="scheduleStore.navigate(1)" title="下一个">
        <Icon name="chevron-right" :size="16" />
      </button>
    </div>

    <!-- 范围切换 -->
    <div class="range-switch">
      <button
        v-for="r in ['week', 'biweek', 'month']"
        :key="r"
        :class="{ active: scheduleStore.viewRange === r }"
        @click="scheduleStore.setRange(r as any)"
      >
        {{ r === 'week' ? '周' : r === 'biweek' ? '双周' : '月' }}
      </button>
      <button class="today-btn" @click="scheduleStore.goToday()">今天</button>
    </div>

    <!-- 星期表头 -->
    <div class="weekdays">
      <div v-for="w in weekdays" :key="w" class="weekday">{{ w }}</div>
    </div>

    <!-- 日期网格 -->
    <div class="grid" :class="scheduleStore.viewRange">
      <div
        v-for="(d, i) in gridDays"
        :key="i"
        class="cell"
        :class="{
          empty: !d,
          today: d && isToday(toISO(d)),
          expanded: d && isExpanded(toISO(d)),
          hasmore: d && pendingCount(toISO(d)) > 0,
        }"
        @click="d && onCellClick(toISO(d))"
      >
        <template v-if="d">
          <div class="date-num">{{ d.getDate() }}</div>
          <div class="dots" v-if="schedulesOn(toISO(d)).length">
            <span
              v-for="s in schedulesOn(toISO(d)).slice(0, 4)"
              :key="s.id"
              class="dot"
              :class="{ done: s.completed }"
              :style="!s.completed && s.has_ddl ? { background: ddlDotColor(s) } : {}"
            ></span>
          </div>
          <div class="count" v-if="pendingCount(toISO(d)) > 0">{{ pendingCount(toISO(d)) }}</div>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.calendar {
  display: flex;
  flex-direction: column;
  gap: 0.5em;
}
.navbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 2px;
}
.title {
  font-size: 0.88em;
  font-weight: 600;
}
.nav-btn {
  background: rgba(128, 128, 128, 0.15);
  border: none;
  color: inherit;
  width: 1.7em;
  height: 1.7em;
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}
.nav-btn:hover { background: rgba(128, 128, 128, 0.3); }
.range-switch {
  display: flex;
  gap: 0.3em;
  align-items: center;
}
.range-switch button {
  background: rgba(128, 128, 128, 0.12);
  border: none;
  color: inherit;
  opacity: 0.7;
  padding: 0.2em 0.6em;
  border-radius: 5px;
  font-size: 0.78em;
  cursor: pointer;
  font-family: inherit;
}
.range-switch button.active {
  background: #6c8cff;
  color: #fff;
  opacity: 1;
}
.today-btn { margin-left: auto !important; }
.weekdays {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 0.2em;
}
.weekday {
  text-align: center;
  font-size: 0.72em;
  opacity: 0.5;
  padding: 0.15em 0;
}
.grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 0.2em;
}
.cell {
  aspect-ratio: 1 / 0.8;
  border-radius: 6px;
  background: rgba(128, 128, 128, 0.1);
  padding: 0.2em 0.3em;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.15em;
  transition: background 0.15s;
  position: relative;
}
/* 月视图：格更紧凑，能容纳更多行 */
.grid.month .cell {
  aspect-ratio: 1 / 0.7;
  min-height: 2.2em;
}
.cell:hover { background: rgba(128, 128, 128, 0.22); }
.cell.empty { background: transparent; cursor: default; }
.cell.empty:hover { background: transparent; }
.cell.today {
  background: rgba(108, 140, 255, 0.25);
  outline: 1px solid rgba(108, 140, 255, 0.6);
}
.cell.expanded {
  background: rgba(108, 140, 255, 0.35);
  outline: 1px solid #6c8cff;
}
.date-num {
  font-size: 0.8em;
  font-weight: 500;
}
.cell.today .date-num { font-weight: 700; }
.dots {
  display: flex;
  gap: 0.15em;
  flex-wrap: wrap;
  justify-content: center;
}
.dot {
  width: 0.35em;
  height: 0.35em;
  border-radius: 50%;
  background: #6c8cff;
}
.dot.done { background: rgba(128, 128, 128, 0.4); }
.count {
  font-size: 0.6em;
  color: #f1c40f;
  font-weight: 600;
}
</style>
