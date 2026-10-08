// Types for scripts/nta-manual-markdown.mjs, used by the in-app manual and its tests.
export type InlineNode =
  | { type: 'text'; value: string }
  | { type: 'code'; value: string }
  | { type: 'strong' | 'em' | 'sub' | 'sup'; children: InlineNode[] }
  | { type: 'link'; href: string; children: InlineNode[] }
  | { type: 'image'; alt: string; src: string };

export interface ListItem { children: InlineNode[]; blocks: BlockNode[] }

export type BlockNode =
  | { type: 'heading'; level: number; id: string; children: InlineNode[] }
  | { type: 'paragraph'; children: InlineNode[] }
  | { type: 'rule' }
  | { type: 'code'; lang: string; value: string }
  | { type: 'quote'; children: BlockNode[] }
  | { type: 'list'; ordered: boolean; start?: number; items: ListItem[] }
  | { type: 'table'; header: InlineNode[][]; align: Array<'center' | 'right' | undefined>; rows: InlineNode[][][] };

export interface ResolvedLink { href: string | null; title?: string }
export interface HtmlOptions {
  idPrefix?: string;
  resolveLink?: (href: string) => ResolvedLink | null;
  resolveImage?: (src: string) => string | null;
}
export interface ManualChapterSource { id: string; text: string }

export const MANUAL_DIR: string;
export function parseInline(text: string): InlineNode[];
export function inlineText(nodes: InlineNode[]): string;
export function slugify(text: string): string;
export function parseMarkdown(source: string): BlockNode[];
export function escapeHtml(text: string): string;
export function safeHref(href: string): string | null;
export function renderHtml(blocks: BlockNode[], options?: HtmlOptions): string;
export function chapterId(fileName: string): string;
export function manualStamp(indexText: string): { kernelVersion: string } | null;
export function orderChapters(fileNames: string[]): string[];
export function checkManualStamp(indexText: string, currentKernelVersion: string): { ok: boolean; message: string };
export function resolveManualLink(href: string, chapterIds: string[], currentChapter: string): ResolvedLink | null;
export function buildManualHtml(values: {
  chapters: ManualChapterSource[];
  images: Record<string, string>;
  programVersion: string;
  kernelVersion: string;
  generatedOn?: string;
}): string;
