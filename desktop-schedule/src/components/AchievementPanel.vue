<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { useAchievementsStore } from '../stores/achievements';
import Icon from './Icon.vue';

const emit = defineEmits<{ close: [] }>();
const store = useAchievementsStore();

onMounted(() => store.load());

const stats = computed(() => store.overview?.stats);

// 单列表：已解锁置顶（按解锁时间倒序），未解锁保持定义顺序
const rows = computed(() => {
  const items = [...(store.overview?.items ?? [])];
  const unlocked = items
    .filter((i) => i.unlocked)
    .sort((a, b) => (b.unlocked_at ?? '').localeCompare(a.unlocked_at ?? ''));
  const locked = items.filter((i) => !i.unlocked);
  return [...unlocked, ...locked];
});

const CATEGORY_ICON: Record<string, string> = {
  cumulative: 'trophy',
  streak: 'flame',
  daily: 'zap',
};

function pct(target: number, progress: number) {
  if (target <= 0) return 0;
  return Math.min(100, Math.round((progress / target) * 100));
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal">
      <header class="head">
        <span class="head-title"><Icon name="trophy" :size="18" /> 成就</span>
        <span class="head-count">{{ store.unlockedCount }} / {{ store.totalCount }}</span>
        <button class="close-btn" @click="emit('close')"><Icon name="x" :size="16" /></button>
      </header>

      <!-- 统计概览：累计 / 最大连续 / 单日最高 -->
      <div v-if="stats" class="stats">
        <div class="stat">
          <span class="stat-num">{{ stats.total }}</span>
          <span class="stat-label">累计完成日程数</span>
        </div>
        <div class="stat">
          <span class="stat-num">{{ stats.max_streak }}</span>
          <span class="stat-label">最大连续完成天数</span>
        </div>
        <div class="stat">
          <span class="stat-num">{{ stats.max_daily }}</span>
          <span class="stat-label">单日最高完成日程数</span>
        </div>
      </div>

      <!-- 成就墙：每行一条，已解锁置顶 -->
      <div class="list">
        <div v-for="it in rows" :key="it.id" class="row" :class="{ unlocked: it.unlocked }">
          <div class="row-icon">
            <Icon :name="CATEGORY_ICON[it.category] ?? 'trophy'" :size="16" />
          </div>
          <div class="row-body">
            <div class="row-line">
              <span class="row-title">{{ it.unlocked ? it.title : '？？？？？' }}</span>
              <span v-if="it.unlocked" class="row-date">{{ it.unlocked_at?.slice(0, 10) }}</span>
              <!-- 未解锁：眼睛图标，悬停显示解锁条件 -->
              <div v-else class="eye-wrap">
                <Icon name="eye" :size="15" />
                <span class="tooltip">解锁条件：{{ it.desc }}</span>
              </div>
            </div>
            <!-- 已解锁显示描述；未解锁显示进度条 -->
            <div v-if="it.unlocked" class="row-desc">{{ it.desc }}</div>
            <template v-else>
              <div class="bar"><div class="fill" :style="{ width: pct(it.target, it.progress) + '%' }"></div></div>
              <div class="num">{{ it.progress }} / {{ it.target }}</div>
            </template>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 颜色全部走主题变量：--app-fg / --app-fg-soft / --modal-bg 随深浅模式自适应 */
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.38);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 150;
}
.modal {
  width: min(560px, 92vw);
  max-height: 84vh;
  overflow-y: auto;
  scrollbar-width: none;
  background: var(--modal-bg, #232634);
  color: var(--app-fg, #e8eaf2);
  border-radius: 16px;
  padding: 18px 20px;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.35);
}
.modal::-webkit-scrollbar { display: none; }

.head {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 14px;
}
.head-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 16px;
  font-weight: 600;
  color: var(--accent, #f6b73c);
}
.head-count {
  font-size: 12px;
  color: var(--app-fg-soft, #9aa0b4);
  margin-left: auto;
}
.close-btn {
  border: none;
  background: transparent;
  color: inherit;
  cursor: pointer;
  padding: 4px;
  border-radius: 6px;
  opacity: 0.7;
}
.close-btn:hover { opacity: 1; background: rgba(128, 128, 128, 0.15); }

.stats {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
  margin-bottom: 16px;
}
.stat {
  background: rgba(128, 128, 128, 0.12);
  border-radius: 10px;
  padding: 10px 8px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
}
.stat-num {
  font-size: 20px;
  font-weight: 700;
  color: var(--accent, #f6b73c);
}
.stat-label {
  font-size: 11px;
  color: var(--app-fg-soft, #9aa0b4);
}

.list { display: flex; flex-direction: column; gap: 8px; }
.row {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  border: 1px solid rgba(128, 128, 128, 0.22);
  border-radius: 10px;
  padding: 10px 12px;
  opacity: 0.65;
}
.row.unlocked {
  opacity: 1;
  border-color: var(--accent, #f6b73c);
  background: var(--accent-soft, rgba(246, 183, 60, 0.12));
}
.row-icon {
  margin-top: 2px;
  color: var(--accent, #f6b73c);
  display: flex;
}
.row-body { flex: 1; min-width: 0; }
.row-line {
  display: flex;
  align-items: center;
  gap: 8px;
}
.row-title {
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.5px;
}
.row-date {
  font-size: 10px;
  color: var(--app-fg-soft, #9aa0b4);
  margin-left: auto;
  white-space: nowrap;
}
.row-desc {
  font-size: 11px;
  color: var(--app-fg-soft, #9aa0b4);
  line-height: 1.4;
  margin-top: 3px;
}

/* 眼睛 + 悬停提示（解锁条件） */
.eye-wrap {
  position: relative;
  margin-left: auto;
  color: var(--app-fg-soft, #9aa0b4);
  cursor: help;
  display: flex;
}
.tooltip {
  position: absolute;
  right: 0;
  top: calc(100% + 6px);
  white-space: nowrap;
  font-size: 11px;
  font-weight: 400;
  color: var(--app-fg, #e8eaf2);
  background: var(--modal-bg, #2a2d3a);
  border: 1px solid rgba(128, 128, 128, 0.35);
  border-radius: 8px;
  padding: 5px 10px;
  box-shadow: 0 6px 18px rgba(0, 0, 0, 0.3);
  opacity: 0;
  transform: translateY(-4px);
  pointer-events: none;
  transition: opacity 0.18s ease, transform 0.18s ease;
  z-index: 10;
}
.eye-wrap:hover .tooltip {
  opacity: 1;
  transform: translateY(0);
}

.bar {
  height: 5px;
  border-radius: 3px;
  background: rgba(128, 128, 128, 0.25);
  overflow: hidden;
  margin-top: 6px;
}
.fill {
  height: 100%;
  border-radius: 3px;
  background: var(--accent, #f6b73c);
  transition: width 0.4s ease;
}
.num {
  font-size: 10px;
  color: var(--app-fg-soft, #9aa0b4);
  text-align: right;
  margin-top: 3px;
}
</style>
