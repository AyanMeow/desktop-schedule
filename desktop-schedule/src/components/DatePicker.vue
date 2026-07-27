<script setup lang="ts">
// 轻量日历日期选择器：弹出月历，单击选日期
import { ref, computed } from 'vue';
import { toISO, parseISO } from '../utils/date';

const props = withDefaults(defineProps<{
  modelValue: string; // 'YYYY-MM-DD'
  label?: string;
}>(), { label: '' });
const emit = defineEmits<{ 'update:modelValue': [string] }>();

const open = ref(false);
// 游标月份
const cursor = ref(parseISO(props.modelValue || toISO(new Date())));

const weekdays = ['一', '二', '三', '四', '五', '六', '日'];

const grid = computed(() => {
  const y = cursor.value.getFullYear();
  const m = cursor.value.getMonth();
  const first = new Date(y, m, 1);
  // 周一为首
  const lead = (first.getDay() + 6) % 7;
  const daysInMonth = new Date(y, m + 1, 0).getDate();
  const cells: (string | null)[] = [];
  for (let i = 0; i < lead; i++) cells.push(null);
  for (let d = 1; d <= daysInMonth; d++) {
    cells.push(toISO(new Date(y, m, d)));
  }
  return cells;
});

const monthLabel = computed(() => `${cursor.value.getFullYear()}年${cursor.value.getMonth() + 1}月`);

const display = computed(() => {
  if (!props.modelValue) return '选择日期';
  const d = parseISO(props.modelValue);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
});

function prevMonth() {
  cursor.value = new Date(cursor.value.getFullYear(), cursor.value.getMonth() - 1, 1);
}
function nextMonth() {
  cursor.value = new Date(cursor.value.getFullYear(), cursor.value.getMonth() + 1, 1);
}
function pick(iso: string) {
  emit('update:modelValue', iso);
  open.value = false;
}
function toggle() {
  open.value = !open.value;
  if (open.value && props.modelValue) cursor.value = parseISO(props.modelValue);
}
</script>

<template>
  <div class="dp">
    <span v-if="label" class="lab">{{ label }}</span>
    <button class="trigger" @click="toggle" type="button">{{ display }}</button>
    <div v-if="open" class="popup" @click.stop>
      <div class="head">
        <button type="button" @click="prevMonth">‹</button>
        <span>{{ monthLabel }}</span>
        <button type="button" @click="nextMonth">›</button>
      </div>
      <div class="wd">
        <span v-for="w in weekdays" :key="w">{{ w }}</span>
      </div>
      <div class="cells">
        <button
          v-for="(c, i) in grid"
          :key="i"
          type="button"
          :class="{ empty: !c, sel: c === modelValue }"
          :disabled="!c"
          @click="c && pick(c)"
        >{{ c ? Number(c.slice(8)) : '' }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dp {
  display: flex;
  flex-direction: column;
  gap: 4px;
  position: relative;
}
.lab {
  font-size: 12px;
  opacity: 0.8;
}
.trigger {
  background: rgba(128, 128, 128, 0.15);
  border: 1px solid rgba(128, 128, 128, 0.25);
  border-radius: 6px;
  padding: 7px 9px;
  color: inherit;
  font-size: 13px;
  font-family: inherit;
  cursor: pointer;
  text-align: left;
}
.popup {
  position: absolute;
  top: 100%;
  left: 0;
  margin-top: 4px;
  background: var(--modal-bg, #2a2c3a);
  color: inherit;
  border: 1px solid rgba(128, 128, 128, 0.25);
  border-radius: 8px;
  padding: 8px;
  z-index: 80;
  width: 220px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
}
.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
  font-size: 12px;
  font-weight: 600;
}
.head button {
  background: rgba(128, 128, 128, 0.15);
  border: none;
  color: inherit;
  width: 22px;
  height: 22px;
  border-radius: 5px;
  cursor: pointer;
  display: flex; align-items: center; justify-content: center;
}
.head button:hover { background: rgba(128, 128, 128, 0.3); }
.wd {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  text-align: center;
  font-size: 10px;
  opacity: 0.5;
  margin-bottom: 3px;
}
.cells {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 2px;
}
.cells button {
  background: transparent;
  border: none;
  color: inherit;
  padding: 5px 0;
  border-radius: 5px;
  cursor: pointer;
  font-size: 12px;
  font-family: inherit;
}
.cells button:hover:not(:disabled) {
  background: rgba(128, 128, 128, 0.2);
}
.cells button.sel {
  background: #6c8cff;
  color: #fff;
  font-weight: 600;
}
.cells button.empty,
.cells button:disabled {
  cursor: default;
  color: transparent;
}
</style>
