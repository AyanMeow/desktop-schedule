import { defineStore } from 'pinia';
import { ref } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { api } from '../api';
import type { UpdateInfo, UpdateProgress } from '../types';

export const useUpdateStore = defineStore('update', () => {
  const info = ref<UpdateInfo | null>(null);
  const checking = ref(false);
  const downloading = ref(false);
  const progress = ref<UpdateProgress | null>(null);
  const ready = ref(false);
  const error = ref('');

  /** 手动检查；发现新版本自动开始下载 */
  async function manualCheck() {
    if (checking.value || downloading.value) return;
    checking.value = true;
    error.value = '';
    ready.value = false;
    info.value = null;
    try {
      const r = await api.checkUpdate();
      info.value = r;
      if (r.has_update) {
        void startDownload();
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      checking.value = false;
    }
  }

  async function startDownload() {
    if (downloading.value) return;
    downloading.value = true;
    error.value = '';
    try {
      await api.downloadUpdate();
      ready.value = true;
    } catch (e) {
      error.value = String(e);
    } finally {
      downloading.value = false;
    }
  }

  /** 用已下载的新版本重启（覆盖换壳） */
  async function restart() {
    try {
      await api.applyUpdate();
    } catch (e) {
      error.value = String(e);
    }
  }

  let initialized = false;
  /** 挂载后台更新事件（仅主窗口调用一次） */
  async function init() {
    if (initialized) return;
    initialized = true;
    const unlistens: UnlistenFn[] = [];
    unlistens.push(
      await listen<UpdateInfo>('update-available', (e) => {
        info.value = e.payload;
      })
    );
    unlistens.push(
      await listen<UpdateProgress>('update-progress', (e) => {
        progress.value = e.payload;
        downloading.value = true;
      })
    );
    unlistens.push(
      await listen<UpdateInfo>('update-ready', (e) => {
        ready.value = true;
        downloading.value = false;
        if (e.payload) info.value = e.payload;
      })
    );
    // 应用生命周期内常驻，无需解绑
  }

  return { info, checking, downloading, progress, ready, error, manualCheck, startDownload, restart, init };
});
