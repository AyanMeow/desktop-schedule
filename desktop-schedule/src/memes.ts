// 表情包映射：按鼓励语分类选图
// 图片放 src/assets/memes/，import 后由 Vite 打包进 exe（离线可用，GIF 可动）
// 占位图用 SVG，你后续替换成真实表情包时：
//   1. 把图片放进 src/assets/memes/
//   2. import 进来加到对应数组
//   3. 不需要改其他代码

import hot1 from '../assets/memes/hot1.svg';
import hot2 from '../assets/memes/hot2.svg';
import playful1 from '../assets/memes/playful1.svg';
import playful2 from '../assets/memes/playful2.svg';
import warm1 from '../assets/memes/warm1.svg';
import warm2 from '../assets/memes/warm2.svg';

export const MEME_BY_CATEGORY: Record<string, string[]> = {
  '热血': [hot1, hot2],
  '调皮': [playful1, playful2],
  '暖心': [warm1, warm2],
};

const FALLBACK = [hot1, playful1, warm1];

/** 按分类随机取一张表情包（无分类则从全部随机） */
export function randomMeme(category?: string | null): string {
  const pool = (category && MEME_BY_CATEGORY[category]) || FALLBACK;
  return pool[Math.floor(Math.random() * pool.length)];
}
