<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { randomMeme } from '../memes';
import type { Encouragement } from '../types';

const visible = ref(false);
const text = ref('');
const category = ref('');
const meme = ref('');
let timer: number | undefined;
let unlisten: UnlistenFn | undefined;

function show(enc: Encouragement | null) {
  if (!enc) return;
  text.value = enc.text;
  category.value = enc.category || '';
  meme.value = randomMeme(enc.category);
  visible.value = true;
  if (timer) window.clearTimeout(timer);
  // 4 秒后和表情包一起淡出（比纯文字稍长，留时间看图）
  timer = window.setTimeout(() => (visible.value = false), 4000);
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
      <!-- 表情包：固定 96x96 容器，cover 裁剪保证任何尺寸都整齐 -->
      <div class="meme-box">
        <img :src="meme" alt="表情包" />
      </div>
      <!-- 鼓励语 -->
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
  padding: 12px 16px 14px;
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  z-index: 200;
  box-shadow: 0 8px 32px rgba(108, 140, 255, 0.5);
}
/* 表情包容器：固定尺寸 + cover 裁剪，任何图片都归一化 */
.meme-box {
  width: 96px;
  height: 96px;
  overflow: hidden;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.15);
}
.meme-box img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  object-position: center;
  display: block;
}
.text {
  font-size: 13px;
  font-weight: 500;
  text-align: center;
  max-width: 200px;
  line-height: 1.4;
}
/* 表情包和文字一起淡入淡出 */
.toast-enter-active,
.toast-leave-active {
  transition: all 0.4s cubic-bezier(0.2, 0.8, 0.2, 1);
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-16px);
}
</style>
