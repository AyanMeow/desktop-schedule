import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { api } from '../api';
import { useConfigStore } from './config';
import type { Schedule, NewSchedule, UpdateSchedule, ViewRange } from '../types';
import { rangeFor } from '../utils/date';

export const useScheduleStore = defineStore('schedules', () => {
  const configStore = useConfigStore();
  // 当前视图基准日期（决定显示哪一周/月）
  const baseDate = ref(new Date());
  const viewRange = ref<ViewRange>('week');
  const weekStart = ref<'monday' | 'sunday'>('monday');

  // 当前范围的所有日程，按日期分组
  const schedules = ref<Schedule[]>([]);
  const loading = ref(false);

  // 计算当前范围的 [start, end]
  const currentRange = computed(() =>
    rangeFor(viewRange.value, baseDate.value, weekStart.value)
  );

  // 按日期分组：Map<'YYYY-MM-DD', Schedule[]>
  const byDate = computed(() => {
    const map = new Map<string, Schedule[]>();
    for (const s of schedules.value) {
      if (!map.has(s.date)) map.set(s.date, []);
      map.get(s.date)!.push(s);
    }
    return map;
  });

  async function refresh() {
    loading.value = true;
    try {
      const [start, end] = currentRange.value;
      schedules.value = await api.listSchedules(start, end);
    } finally {
      loading.value = false;
    }
  }

  function setRange(r: ViewRange) {
    viewRange.value = r;
    // 持久化视图范围
    configStore.config.view.range = r;
    configStore.save();
    refresh();
  }

  function navigate(dir: 1 | -1) {
    if (viewRange.value === 'month') {
      baseDate.value = new Date(baseDate.value.getFullYear(), baseDate.value.getMonth() + dir, 1);
    } else {
      const weeks = viewRange.value === 'biweek' ? 2 : 1;
      const d = new Date(baseDate.value);
      d.setDate(d.getDate() + dir * weeks * 7);
      baseDate.value = d;
    }
    refresh();
  }

  function goToday() {
    baseDate.value = new Date();
    refresh();
  }

  async function create(input: NewSchedule) {
    await api.createSchedule(input);
    await refresh();
  }

  async function toggle(id: number) {
    const updated = await api.toggleComplete(id);
    const idx = schedules.value.findIndex((s) => s.id === id);
    if (idx >= 0) schedules.value[idx] = updated;
    return updated;
  }

  async function update(id: number, update: UpdateSchedule) {
    const updated = await api.updateSchedule(id, update);
    const idx = schedules.value.findIndex((s) => s.id === id);
    if (idx >= 0) schedules.value[idx] = updated;
    return updated;
  }

  async function remove(id: number) {
    await api.deleteSchedule(id);
    schedules.value = schedules.value.filter((s) => s.id !== id);
  }

  async function removeGroup(groupId: string) {
    await api.deleteGroup(groupId);
    schedules.value = schedules.value.filter((s) => s.group_id !== groupId);
  }

  return {
    baseDate,
    viewRange,
    weekStart,
    schedules,
    loading,
    currentRange,
    byDate,
    refresh,
    setRange,
    navigate,
    goToday,
    create,
    toggle,
    update,
    remove,
    removeGroup,
  };
});
