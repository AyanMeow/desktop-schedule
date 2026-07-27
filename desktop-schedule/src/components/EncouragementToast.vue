<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import Icon from './Icon.vue';
import type { Encouragement } from '../types';

const visible = ref(false);
const text = ref('');
const category = ref('');
let timer: number | undefined;
let unlisten: UnlistenFn | undefined;

function show(enc: Encouragement | null) {
  if (!enc) return;
  text.value = enc.text;
  category.value = enc.category || '';
  visible.value = true;
  if (timer) window.clearTimeout(timer);
  timer = window.setTimeout(() => (visible.value = false), 2800);
}

onMounted(async () => {
  unlisten = await listen<Encouragement | null>('encouragement', (e) => show(e.payload));
});

onUnmounted(() => {
  unlisten?.();
  if (timer) window.clearTimeout(timer);
});
</script>

<template>
  <Transition name="toast">
    <div v-if="visible" class="enc-toast">
      <Icon name="star" :size="16" />
      <span class="text">{{ text }}</span>
    </div>
  </Transition>
</template>

<style scoped>
.enc-toast {
  position: fixed;
  top: 18px;
  left: 50%;
  transform: translateX(-50%);
  background: linear-gradient(135deg, #6c8cff, #8e6cff);
  color: #fff;
  padding: 10px 18px;
  border-radius: 22px;
  font-size: 13px;
  font-weight: 500;
  box-shadow: 0 6px 24px rgba(108, 140, 255, 0.45);
  display: flex;
  align-items: center;
  gap: 8px;
  z-index: 200;
  max-width: 90%;
  text-align: center;
}
.toast-enter-active,
.toast-leave-active {
  transition: all 0.35s cubic-bezier(0.2, 0.8, 0.2, 1);
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-12px);
}
</style>
