// 日期工具：范围计算、格式化、ddl 着色

/** 'YYYY-MM-DD' */
export function toISO(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

export function parseISO(s: string): Date {
  const [y, m, d] = s.split('-').map(Number);
  return new Date(y, m - 1, d);
}

export function today(): string {
  return toISO(new Date());
}

export function addDays(d: Date, n: number): Date {
  const r = new Date(d);
  r.setDate(r.getDate() + n);
  return r;
}

/** 计算从某基准日期开始的范围 [start, end]（ISO 字符串） */
export function rangeFor(view: 'week' | 'biweek' | 'month', base: Date, weekStart: 'monday' | 'sunday'): [string, string] {
  if (view === 'month') {
    const start = new Date(base.getFullYear(), base.getMonth(), 1);
    const end = new Date(base.getFullYear(), base.getMonth() + 1, 0);
    return [toISO(start), toISO(end)];
  }
  // 周/双周：对齐到 weekStart
  const offset = weekStart === 'monday' ? 1 : 0;
  const dow = (base.getDay() + 7 - offset) % 7;
  const start = addDays(base, -dow);
  const weeks = view === 'biweek' ? 2 : 1;
  const end = addDays(start, weeks * 7 - 1);
  return [toISO(start), toISO(end)];
}

/** 生成 [start, end] 之间每一天的 Date 数组（含两端） */
export function eachDay(startISO: string, endISO: string): Date[] {
  const start = parseISO(startISO);
  const end = parseISO(endISO);
  const out: Date[] = [];
  for (let d = start; d <= end; d = addDays(d, 1)) out.push(new Date(d));
  return out;
}

/** 中文星期 */
export const WEEKDAY_CN = ['日', '一', '二', '三', '四', '五', '六'];

/** 判断是否今天 */
export function isToday(iso: string): boolean {
  return iso === today();
}

/** 月份导航：把基准日期移动若干月/周 */
export function navigate(base: Date, view: 'week' | 'biweek' | 'month', dir: 1 | -1): Date {
  if (view === 'month') {
    const r = new Date(base);
    r.setMonth(r.getMonth() + dir);
    return r;
  }
  const weeks = view === 'biweek' ? 2 : 1;
  return addDays(base, dir * weeks * 7);
}

// ============ ddl 着色 ============

export interface DdlStatus {
  color: string;
  label: string; // '还剩 3 天' / '今天' / '已超期 1 天' / ''
  level: 'overdue' | 'le1' | 'le3' | 'le7' | 'gt7' | 'none';
}

/** 根据 ddl 计算剩余天数与对应颜色级别（颜色由 CSS 变量驱动，此处仅算 level/label） */
export function ddlStatus(ddlAt: string | null, colors?: { overdue: string; le1: string; le3: string; le7: string; gt7: string }): DdlStatus {
  if (!ddlAt) return { color: '', label: '', level: 'none' };
  // ddl 可能是 'YYYY-MM-DD' 或 'YYYY-MM-DD HH:MM'
  const ddlDate = parseISO(ddlAt.slice(0, 10));
  const now = new Date();
  const today0 = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const diffDays = Math.round((ddlDate.getTime() - today0.getTime()) / 86400000);

  let level: DdlStatus['level'];
  let label: string;
  if (diffDays < 0) {
    level = 'overdue';
    label = `已超期 ${-diffDays} 天`;
  } else if (diffDays === 0) {
    level = 'le1';
    label = '今天';
  } else if (diffDays <= 1) {
    level = 'le1';
    label = `还剩 ${diffDays} 天`;
  } else if (diffDays <= 3) {
    level = 'le3';
    label = `还剩 ${diffDays} 天`;
  } else if (diffDays <= 7) {
    level = 'le7';
    label = `还剩 ${diffDays} 天`;
  } else {
    level = 'gt7';
    label = `还剩 ${diffDays} 天`;
  }
  const colorMap = colors
    ? {
        overdue: colors.overdue,
        le1: colors.le1,
        le3: colors.le3,
        le7: colors.le7,
        gt7: colors.gt7,
        none: '',
      }
    : { overdue: '', le1: '', le3: '', le7: '', gt7: '', none: '' };
  return { color: colorMap[level], label, level };
}
