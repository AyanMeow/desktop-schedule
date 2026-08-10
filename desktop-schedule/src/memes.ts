// 表情包映射：按鼓励语分类选图
// 图片放 src/assets/memes/，import 后由 Vite 打包进 exe（离线可用，GIF 可动）
// 替换表情包：把新图放进 src/assets/memes/，在下方 import 并加到对应分类数组即可

import hot1 from './assets/memes/hot1.jpg?url';
import hot2 from './assets/memes/hot2.jpg?url';
import hot3 from './assets/memes/hot3.gif?url';
import hot4 from './assets/memes/hot4.gif?url';
import playful1 from './assets/memes/playful1.gif?url';
import playful2 from './assets/memes/playful2.jpg?url';
import playful3 from './assets/memes/playful3.gif?url';
import warm1 from './assets/memes/warm1.jpg?url';
import warm2 from './assets/memes/warm2.jpg?url';

export const MEME_BY_CATEGORY: Record<string, string[]> = {
  '热血': [hot1, hot2, hot3, hot4],
  '调皮': [playful1, playful2, playful3],
  '暖心': [warm1, warm2],
};

const FALLBACK = [hot1, playful1, warm1];

/** 按分类随机取一张表情包（无分类则从全部随机） */
export function randomMeme(category?: string | null): string {
  const pool = (category && MEME_BY_CATEGORY[category]) || FALLBACK;
  return pool[Math.floor(Math.random() * pool.length)];
}
