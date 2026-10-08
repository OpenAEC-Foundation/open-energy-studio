/**
 * The user manual inside the app (Gereedschap › Handleiding, BRL 9501 §4.4).
 * Chapters render as React elements from the shared Markdown parser, so the
 * manual text can never inject markup; links between chapters stay in the
 * viewer, links to other project documents show their path, and images come
 * from the build, so everything works offline.
 */
import { Fragment, useEffect, useRef, type ReactNode } from 'react';
import { useI18n } from '../../i18n/i18n';
import { useKernel } from '../../context/KernelProvider';
import { safeHref, type BlockNode, type InlineNode } from '../../../scripts/nta-manual-markdown.mjs';
import {
  MANUAL_CHAPTERS, MANUAL_KERNEL_VERSION, MANUAL_PROGRAM_VERSION, manualChapter, manualImage, splitChapterRef,
} from '../../core/manual/manual';
import { Banner } from '../ui';
import './ManualView.css';

const HEADING_PREFIX = 'manual-';

interface LinkTarget { chapter: string; anchor?: string }

/** Where a link in chapter `current` points: another chapter, an anchor, an outside page or a project document. */
export function manualLinkTarget(href: string, current: string):
  | { kind: 'chapter'; target: LinkTarget }
  | { kind: 'external'; href: string }
  | { kind: 'document'; path: string } {
  const safe = safeHref(href);
  // An unsafe scheme (javascript:, data:, …) is shown as text, never followed.
  if (safe === null) return { kind: 'document', path: href };
  if (/^(https?:|mailto:)/i.test(safe)) return { kind: 'external', href: safe };
  const [path, anchor] = safe.split('#');
  if (!path) return { kind: 'chapter', target: { chapter: current, ...(anchor ? { anchor } : {}) } };
  const file = path.replace(/^\.\//, '');
  const id = file.replace(/\.md$/, '');
  if (!file.includes('/') && MANUAL_CHAPTERS.some((chapter) => chapter.id === id)) {
    return { kind: 'chapter', target: { chapter: id, ...(anchor ? { anchor } : {}) } };
  }
  return { kind: 'document', path: file.replace(/^\.\.\//, 'docs/') };
}

interface RenderContext {
  chapter: string;
  open: (target: LinkTarget) => void;
  documentTitle: (path: string) => string;
}

function Inline({ nodes, ctx }: { nodes: InlineNode[]; ctx: RenderContext }): ReactNode {
  return nodes.map((node, index) => {
    switch (node.type) {
      case 'text': return <Fragment key={index}>{node.value}</Fragment>;
      case 'code': return <code key={index}>{node.value}</code>;
      case 'strong': return <strong key={index}><Inline nodes={node.children} ctx={ctx} /></strong>;
      case 'em': return <em key={index}><Inline nodes={node.children} ctx={ctx} /></em>;
      case 'sub': return <sub key={index}><Inline nodes={node.children} ctx={ctx} /></sub>;
      case 'sup': return <sup key={index}><Inline nodes={node.children} ctx={ctx} /></sup>;
      case 'image': {
        const src = manualImage(node.src);
        return src ? <img key={index} src={src} alt={node.alt} className="manual-image" /> : <Fragment key={index}>{node.alt}</Fragment>;
      }
      case 'link': {
        const label = <Inline nodes={node.children} ctx={ctx} />;
        const target = manualLinkTarget(node.href, ctx.chapter);
        if (target.kind === 'external') {
          return <a key={index} href={target.href} target="_blank" rel="noopener noreferrer">{label}</a>;
        }
        if (target.kind === 'document') {
          return <span key={index} className="manual-doc-ref" title={ctx.documentTitle(target.path)}>{label}</span>;
        }
        const anchor = target.target.anchor ? `#${HEADING_PREFIX}${target.target.anchor}` : '';
        return (
          <a key={index} href={`#/gereedschap/manual${anchor}`} data-chapter={target.target.chapter}
            onClick={(event) => { event.preventDefault(); ctx.open(target.target); }}>{label}</a>
        );
      }
      default: return null;
    }
  });
}

function Blocks({ blocks, ctx }: { blocks: BlockNode[]; ctx: RenderContext }): ReactNode {
  return blocks.map((block, index) => {
    switch (block.type) {
      case 'heading': {
        // The chapter title is the page's h2 (the page head holds the h1); sections follow.
        const Tag = `h${Math.min(6, block.level + 1)}` as 'h2';
        return <Tag key={index} id={`${HEADING_PREFIX}${block.id}`} tabIndex={-1}><Inline nodes={block.children} ctx={ctx} /></Tag>;
      }
      case 'paragraph': return <p key={index}><Inline nodes={block.children} ctx={ctx} /></p>;
      case 'rule': return <hr key={index} />;
      case 'code': return <pre key={index}><code>{block.value}</code></pre>;
      case 'quote': return <blockquote key={index}><Blocks blocks={block.children} ctx={ctx} /></blockquote>;
      case 'list': {
        const items = block.items.map((item, itemIndex) => (
          <li key={itemIndex}><Inline nodes={item.children} ctx={ctx} /><Blocks blocks={item.blocks} ctx={ctx} /></li>
        ));
        return block.ordered
          ? <ol key={index} start={block.start}>{items}</ol>
          : <ul key={index}>{items}</ul>;
      }
      case 'table':
        return (
          <div key={index} className="manual-table-wrap">
            <table className="manual-table">
              <thead><tr>{block.header.map((cell, cellIndex) => (
                <th key={cellIndex} scope="col" style={{ textAlign: block.align[cellIndex] }}><Inline nodes={cell} ctx={ctx} /></th>
              ))}</tr></thead>
              <tbody>{block.rows.map((row, rowIndex) => (
                <tr key={rowIndex}>{row.map((cell, cellIndex) => (
                  <td key={cellIndex} style={{ textAlign: block.align[cellIndex] }}><Inline nodes={cell} ctx={ctx} /></td>
                ))}</tr>
              ))}</tbody>
            </table>
          </div>
        );
      default: return null;
    }
  });
}

export interface ManualViewProps {
  /** "03-projectberekening" or "03-projectberekening#anchor"; the index when empty. */
  chapterRef?: string;
  onOpen: (chapterRef: string) => void;
}

export function ManualView({ chapterRef, onOpen }: ManualViewProps) {
  const { t } = useI18n();
  const kernel = useKernel();
  const { chapter: chapterId, anchor } = splitChapterRef(chapterRef);
  const chapter = manualChapter(chapterId);
  const articleRef = useRef<HTMLElement>(null);
  const runningKernel = kernel?.settled?.kernelVersion ?? null;

  useEffect(() => {
    const article = articleRef.current;
    if (!article) return;
    const target = anchor ? article.querySelector<HTMLElement>(`#${CSS.escape(`${HEADING_PREFIX}${anchor}`)}`) : null;
    if (target) {
      target.scrollIntoView?.({ block: 'start' });
      target.focus({ preventScroll: true });
    } else {
      article.scrollTop = 0;
    }
  }, [chapter.id, anchor]);

  const ctx: RenderContext = {
    chapter: chapter.id,
    open: (target) => onOpen(target.anchor ? `${target.chapter}#${target.anchor}` : target.chapter),
    documentTitle: (path) => t('manual.documentRef', { path }),
  };

  return (
    <div className="manual-view">
      <nav className="manual-toc" aria-label={t('manual.contents')}>
        <ol>
          {MANUAL_CHAPTERS.map((item) => (
            <li key={item.id}>
              <button type="button" className="manual-toc-item" aria-current={item.id === chapter.id ? 'page' : undefined}
                onClick={() => onOpen(item.id)}>{item.title}</button>
            </li>
          ))}
        </ol>
        <dl className="manual-versions" data-testid="manual-versions">
          <dt>{t('manual.programVersion')}</dt><dd>{MANUAL_PROGRAM_VERSION}</dd>
          <dt>{t('manual.manualKernel')}</dt><dd>{MANUAL_KERNEL_VERSION ?? '–'}</dd>
          {runningKernel && <><dt>{t('manual.runningKernel')}</dt><dd>{runningKernel}</dd></>}
        </dl>
      </nav>
      <article className="manual-article" ref={articleRef} aria-label={chapter.title} data-chapter={chapter.id}>
        {runningKernel && MANUAL_KERNEL_VERSION && runningKernel !== MANUAL_KERNEL_VERSION && (
          <Banner tone="warn">{t('manual.kernelMismatch', { manual: MANUAL_KERNEL_VERSION, kernel: runningKernel })}</Banner>
        )}
        <Blocks blocks={chapter.blocks} ctx={ctx} />
      </article>
    </div>
  );
}
