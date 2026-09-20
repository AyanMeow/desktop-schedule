// 主题样式 composable：主窗口与抽屉窗口共用，
// 保证两个窗口的配色/字体/背景逐像素一致。
// 抽取自 App.vue 的 rootStyle/bgLayerStyle 原逻辑（行为不变）。
import { computed } from 'vue';
import { getPalette, getDdlScale } from '../themes';
import type { AppConfig } from '../types';

// hex → rgba（veil 半透明用）
export function hexToRgba(hex: string, alpha: number): string {
  const h = hex.replace('#', '');
  const r = parseInt(h.slice(0, 2), 16);
  const g = parseInt(h.slice(2, 4), 16);
  const b = parseInt(h.slice(4, 6), 16);
  return `rgba(${r},${g},${b},${alpha})`;
}

export function useThemeStyle(getWindow: () => AppConfig['window']) {
  const theme = computed(() => getWindow().bg_mode);
  const isLight = computed(() => theme.value === 'light');

  // 根容器样式：字体由 CSS 变量驱动，子组件用 em 继承
  const rootStyle = computed(() => {
    const light = isLight.value;
    const w = getWindow();
    const p = getPalette(w.theme_name);
    const fg = light ? p.lightFg : p.darkFg;
    const accent = light ? p.lightAccent : p.darkAccent;
    const warning = light ? p.lightWarning : p.darkWarning;
    const danger = light ? p.lightDanger : p.darkDanger;
    const ddl = getDdlScale(p, light);
    // veil：文字背后的磨砂底板
    const veilBg = light ? hexToRgba(p.lightBg, 0.55) : hexToRgba(p.darkBg, 0.55);
    return {
      '--app-font-size': `${w.font_size}px`,
      '--app-font-family': w.font_family,
      '--app-fg': fg,
      '--app-fg-soft': fg + 'a6', // 约 65% 不透明
      '--modal-bg': light ? p.lightBg : p.darkBg,
      '--veil-bg': veilBg,
      '--accent': accent,
      '--accent-soft': accent + '2e', // 约 18%
      '--warning': warning,
      '--danger': danger,
      '--ddl-overdue': ddl.overdue,
      '--ddl-le1': ddl.le1,
      '--ddl-le3': ddl.le3,
      '--ddl-le7': ddl.le7,
      '--ddl-gt7': ddl.gt7,
      fontFamily: w.font_family,
      fontSize: `${w.font_size}px`,
      color: fg,
    };
  });

  // 背景层样式：透明度只作用于背景，前景文字不受影响
  const bgLayerStyle = computed(() => {
    const w = getWindow();
    if (w.bg_mode === 'image' && w.bg_value) {
      return {
        backgroundImage: `url(${w.bg_value})`,
        opacity: w.opacity,
      };
    }
    // dark/light 模式：背景色由 palette 决定（切主题时即时生效）
    const p = getPalette(w.theme_name);
    const bg = isLight.value ? p.lightBg : p.darkBg;
    return {
      background: bg,
      opacity: w.opacity,
    };
  });

  return { theme, isLight, rootStyle, bgLayerStyle };
}
