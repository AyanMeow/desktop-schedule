import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { api } from '../api';
import type { AchievementOverview } from '../types';

export const useAchievementsStore = defineStore('achievements', () => {
  const overview = ref<AchievementOverview | null>(null);

  const unlockedCount = computed(
    () => overview.value?.items.filter((i) => i.unlocked).length ?? 0
  );
  const totalCount = computed(() => overview.value?.items.length ?? 0);

  async function load() {
    try {
      overview.value = await api.achievementOverview();
    } catch {
      /* 拉取失败不阻塞界面，下次打开面板/解锁事件时再拉 */
    }
  }

  return { overview, unlockedCount, totalCount, load };
});
