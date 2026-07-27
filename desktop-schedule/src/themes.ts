// 主题配色预设：每套含深/浅两种模式的完整色板
// 设计原则：低饱和度、对比度足够、扁平现代

export interface Palette {
  name: string;        // 预设标识
  label: string;       // 显示名
  // 深色模式（bg_mode=dark）
  darkBg: string;
  darkFg: string;
  darkAccent: string;    // 强调色（链接/按钮/激活态）
  darkWarning: string;   // 警告/紧迫（替代刺眼黄）
  darkDanger: string;    // 危险/超期
  // 浅色模式（bg_mode=light）
  lightBg: string;
  lightFg: string;
  lightAccent: string;
  lightWarning: string;
  lightDanger: string;
}

// ddl 色阶（5 级），每套主题独立
export interface DdlScale {
  overdue: string;
  le1: string;
  le3: string;
  le7: string;
  gt7: string;
}

export const PALETTES: Palette[] = [
  {
    name: 'slate',
    label: '岩灰',
    darkBg: '#2b2d3a', darkFg: '#eef0f5',
    darkAccent: '#7c9cff', darkWarning: '#e8b04d', darkDanger: '#e57373',
    lightBg: '#f2f3f5', lightFg: '#2b2d3a',
    lightAccent: '#4a6cf7', lightWarning: '#c8861a', lightDanger: '#c0392b',
  },
  {
    name: 'forest',
    label: '森绿',
    darkBg: '#1f2a26', darkFg: '#e8f0ea',
    darkAccent: '#6fbf8e', darkWarning: '#d9b450', darkDanger: '#e07a6e',
    lightBg: '#eef3ee', lightFg: '#1f2a26',
    lightAccent: '#3d9468', lightWarning: '#b8881a', lightDanger: '#b9483a',
  },
  {
    name: 'ocean',
    label: '深海',
    darkBg: '#1a2632', darkFg: '#e4ecf2',
    darkAccent: '#5db4d9', darkWarning: '#e0b94d', darkDanger: '#e06b6b',
    lightBg: '#eaf2f6', lightFg: '#1a2632',
    lightAccent: '#2b8fb8', lightWarning: '#b8861a', lightDanger: '#b0392b',
  },
  {
    name: 'rose',
    label: '蔷薇',
    darkBg: '#2e2126', darkFg: '#f2e8ea',
    darkAccent: '#d98ca8', darkWarning: '#e0b94d', darkDanger: '#e57878',
    lightBg: '#f5edef', lightFg: '#2e2126',
    lightAccent: '#b8567a', lightWarning: '#b8861a', lightDanger: '#b9394a',
  },
  {
    name: 'sand',
    label: '沙金',
    darkBg: '#2d2820', darkFg: '#f0ece0',
    darkAccent: '#d4a857', darkWarning: '#e0c050', darkDanger: '#e07a6e',
    lightBg: '#f5f0e6', lightFg: '#2d2820',
    lightAccent: '#a8842e', lightWarning: '#9a7510', lightDanger: '#b9483a',
  },
  {
    name: 'ink',
    label: '墨黑',
    darkBg: '#1a1a1e', darkFg: '#ececee',
    darkAccent: '#9a9aa5', darkWarning: '#c8a040', darkDanger: '#cc7070',
    lightBg: '#ededf0', lightFg: '#1a1a1e',
    lightAccent: '#555560', lightWarning: '#8a6a10', lightDanger: '#a04040',
  },
];

// 取指定主题预设（不存在则回退 slate）
export function getPalette(name: string): Palette {
  return PALETTES.find((p) => p.name === name) || PALETTES[0];
}

// 由主题推导 ddl 5 级色阶（基于 danger，向 warning 渐变）
export function getDdlScale(p: Palette, isLight: boolean): DdlScale {
  const danger = isLight ? p.lightDanger : p.darkDanger;
  const warning = isLight ? p.lightWarning : p.darkWarning;
  return {
    overdue: danger,
    le1: danger,
    le3: '#e8821c',   // 橙（固定，深浅都可见）
    le7: warning,
    gt7: isLight ? '#7a8290' : '#8a92a0',
  };
}
