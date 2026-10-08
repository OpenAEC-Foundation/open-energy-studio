/**
 * The NTA 8800 user manual (docs/handleiding-nta8800/) bundled into the app
 * at build time (BRL 9501 §4.4, task B7). The chapters and images come from
 * the same folder the release exports, so the app and its release always
 * carry the same manual; nothing is fetched at run time, which keeps the
 * Tauri app working offline.
 */
import { version as programVersion } from '../../../package.json';
import { chapterId, inlineText, orderChapters, parseMarkdown, type BlockNode } from '../../../scripts/nta-manual-markdown.mjs';

const SOURCES = import.meta.glob('../../../docs/handleiding-nta8800/*.md', { query: '?raw', import: 'default', eager: true }) as Record<string, string>;
const IMAGES = import.meta.glob('../../../docs/handleiding-nta8800/img/*', { query: '?url', import: 'default', eager: true }) as Record<string, string>;

export interface ManualChapter {
  id: string;
  title: string;
  blocks: BlockNode[];
}

const fileName = (path: string) => path.slice(path.lastIndexOf('/') + 1);
const byName = new Map(Object.entries(SOURCES).map(([path, text]) => [fileName(path), text]));

/** The chapters in reading order: the index first, then 00…10. */
export const MANUAL_CHAPTERS: ManualChapter[] = orderChapters([...byName.keys()]).map((name) => {
  const blocks = parseMarkdown(byName.get(name) ?? '');
  const heading = blocks.find((block) => block.type === 'heading');
  return { id: chapterId(name), title: heading && heading.type === 'heading' ? inlineText(heading.children) : name, blocks };
});

export { MANUAL_INDEX } from './manual-routes';

export { MANUAL_KERNEL_VERSION } from './manual-version';

export const MANUAL_PROGRAM_VERSION: string = programVersion;

export function manualChapter(id: string | undefined): ManualChapter {
  return MANUAL_CHAPTERS.find((chapter) => chapter.id === id) ?? MANUAL_CHAPTERS[0];
}

/** A bundled image of the manual ("img/welkom.png" → its URL in the build), or null. */
export function manualImage(src: string): string | null {
  const name = src.replace(/^\.\//, '');
  if (!name.startsWith('img/')) return null;
  const entry = Object.entries(IMAGES).find(([path]) => path.endsWith(`/${name}`));
  return entry ? entry[1] : null;
}

export { chapterForRoute, splitChapterRef } from './manual-routes';
