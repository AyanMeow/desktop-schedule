<script setup lang="ts">
// 右下角更新状态条：下载中显示进度，就绪后显示重启按钮
import { computed } from 'vue';
import { useUpdateStore } from '../stores/update';
import Icon from './Icon.vue';

const store = useUpdateStore();
const visible = computed(() => store.downloading || store.ready);
</script>

<template>
  <Transition name="ubar">
    <div v-if="visible" class="ubar">
      <template v-if="store.downloading">
        <Icon name="cloud" :size="15" />
        <div class="ubar-body">
          <span class="ubar-text">正在下载 v{{ store.info?.latest }} {{ store.progress?.percent ?? 0 }}%</span>
          <div class="ubar-bar">
            <div class="ubar-fill" :style="{ width: (store.progress?.percent ?? 0) + '%' }"></div>
          </div>
        </div>
      </template>
      <template v-else-if="store.ready">
        <Icon name="check" :size="15" />
        <span class="ubar-text">新版本 v{{ store.info?.latest }} 已就绪</span>
        <button class="ubar-btn" @click="store.restart()">立即重启</button>
      </template>
    </div>
  </Transition>
</template>

<style scoped>
.ubar {
  position: fixed;
  right: 18px;
  bottom: 18px;
  background: var(--modal-bg, #232634);
  color: var(--app-fg, #e8eaf2);
  border: 1px solid var(--accent, #6c8cff);
  border-radius: 12px;
  padding: 10px 14px;
  display: flex;
  align-items: center;
  gap: 10px;
  z-index: 210;
  box-shadow: 0 8px 28px rgba(0, 0, 0, 0.35);
  font-size: 12px;
}
.ubar-body { display: flex; flex-direction: column; gap: 5px; min-width: 170px; }
.ubar-text { white-space: nowrap; }
.ubar-bar {
  height: 5px;
  border-radius: 3px;
  background: rgba(128, 128, 128, 0.25);
  overflow: hidden;
}
.ubar-fill {
  height: 100%;
  background: var(--accent, #6c8cff);
  border-radius: 3px;
  transition: width 0.3s ease;
}
.ubar-btn {
  border: none;
  cursor: pointer;
  padding: 5px 12px;
  border-radius: 7px;
  font-size: 12px;
  color: #fff;
  background: var(--accent, #6c8cff);
}
.ubar-btn:hover { filter: brightness(1.1); }
.ubar-enter-active,
.ubar-leave-active { transition: all 0.3s ease; }
.ubar-enter-from,
.ubar-leave-to { opacity: 0; transform: translateY(12px); }
</style>
