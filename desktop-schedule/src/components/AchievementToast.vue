<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import Icon from './Icon.vue';
import type { AchievementUnlockedPayload } from '../types';
import { useAchievementsStore } from '../stores/achievements';

const achievementsStore = useAchievementsStore();
const visible = ref(false);
const firstTitle = ref('');
const extraCount = ref(0);
let timer: number | undefined;
let unlisten: UnlistenFn | undefined;

function show(payload: AchievementUnlockedPayload) {
  const list = payload?.achievements || [];
  if (!list.length) return;
  firstTitle.value = list[0].title;
  extraCount.value = list.length - 1;
  visible.value = true;
  // 同步刷新面板数据（面板若开着立即翻面）
  void achievementsStore.load();
  if (timer) window.clearTimeout(timer);
  timer = window.setTimeout(() => (visible.value = false), 4000);
}

onMounted(async () => {
  unlisten = await listen<AchievementUnlockedPayload>(
    'achievement-unlocked',
    (e) => show(e.payload)
  );
});

onUnmounted(() => {
  unlisten?.();
  if (timer) window.clearTimeout(timer);
});
</script>

<template>
  <Transition name="ach-toast">
    <div v-if="visible" class="ach-toast">
      <div class="trophy-badge"><Icon name="trophy" :size="26" /></div>
      <span class="label">成就解锁</span>
      <span class="title">{{ firstTitle }}</span>
      <span v-if="extraCount > 0" class="more">等 {{ extraCount }} 项成就同时解锁</span>
    </div>
  </Transition>
</template>

<style scoped>
/* 右上角金色 Toast，与顶部居中的鼓励语气泡错开 */
.ach-toast {
  position: fixed;
  top: 18px;
  right: 18px;
  background: linear-gradient(135deg, #f6b73c, #e08600);
  color: #fff;
  padding: 12px 18px;
  border-radius: 14px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  z-index: 201;
  box-shadow: 0 8px 28px rgba(224, 134, 0, 0.45);
}
.trophy-badge {
  width: 44px;
  height: 44px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.22);
  display: flex;
  align-items: center;
  justify-content: center;
}
.label {
  font-size: 11px;
  opacity: 0.9;
  letter-spacing: 2px;
}
.title {
  font-size: 15px;
  font-weight: 600;
}
.more {
  font-size: 11px;
  opacity: 0.85;
}
.ach-toast-enter-active,
.ach-toast-leave-active {
  transition: all 0.4s cubic-bezier(0.2, 0.8, 0.2, 1);
}
.ach-toast-enter-from,
.ach-toast-leave-to {
  opacity: 0;
  transform: translateY(-16px);
}
</style>
