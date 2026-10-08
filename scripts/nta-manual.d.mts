// Types for scripts/nta-manual.mjs, used by its Vitest test.
export * from './nta-manual-markdown.mjs';
import type { ManualChapterSource } from './nta-manual-markdown.mjs';

export function readManual(root: string): { chapters: ManualChapterSource[]; images: Record<string, string> };
