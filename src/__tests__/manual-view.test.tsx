import { describe, expect, it, vi } from 'vitest';
import { screen, within } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { version } from '../../package.json';
import { ManualView, manualLinkTarget } from '../components/ManualView/ManualView';
import { PageHeader } from '../components/shell/PageHeader';
import { ShellActionsProvider, type ShellActions } from '../components/shell/ShellActions';
import { MANUAL_CHAPTERS, MANUAL_KERNEL_VERSION, chapterForRoute, manualImage, splitChapterRef } from '../core/manual/manual';
import { normalizeRoute, TOOL_STEP } from '../core/navigation/routes';
import { kernelVersion } from '../../scripts/nta-kernel-version.mjs';
import { join } from 'node:path';

const root = join(__dirname, '../..');

describe('in-app manual (BRL 9501 §4.4)', () => {
  it('bundles every chapter of docs/handleiding-nta8800 with the index first', () => {
    expect(MANUAL_CHAPTERS[0].id).toBe('index');
    expect(MANUAL_CHAPTERS.map((chapter) => chapter.id)).toContain('10-normversies');
    expect(MANUAL_CHAPTERS.length).toBe(12);
    expect(MANUAL_KERNEL_VERSION).toBe(kernelVersion(root));
    expect(manualImage('img/welkom.png')).toBeTruthy();
    expect(manualImage('../elders.png')).toBeNull();
  });

  it('shows the index with the program and kernel versions', () => {
    renderWithProviders(<ManualView onOpen={vi.fn()} />);
    const article = screen.getByRole('article');
    expect(within(article).getByRole('heading', { level: 2, name: /Gebruikershandleiding NTA 8800/ })).toBeInTheDocument();
    const versions = screen.getByTestId('manual-versions');
    expect(versions).toHaveTextContent(version);
    expect(versions).toHaveTextContent(MANUAL_KERNEL_VERSION ?? 'x');
  });

  it('navigates through the table of contents and the links between chapters', async () => {
    const onOpen = vi.fn();
    const user = userEvent.setup();
    renderWithProviders(<ManualView onOpen={onOpen} />);
    await user.click(screen.getByRole('button', { name: /Normversies/ }));
    expect(onOpen).toHaveBeenLastCalledWith('10-normversies');
    // The index links to every chapter; following one stays in the viewer.
    await user.click(within(screen.getByRole('article')).getByRole('link', { name: 'Basisopname' }));
    expect(onOpen).toHaveBeenLastCalledWith('02-basisopname');
  });

  it('renders a chapter with its images and in-chapter anchors', async () => {
    const onOpen = vi.fn();
    const user = userEvent.setup();
    renderWithProviders(<ManualView chapterRef="00-werken-met-het-programma" onOpen={onOpen} />);
    const article = screen.getByRole('article');
    expect(article).toHaveAttribute('data-chapter', '00-werken-met-het-programma');
    expect(screen.getByRole('button', { name: /Werken met het programma/ })).toHaveAttribute('aria-current', 'page');
    const image = within(article).getAllByRole('img')[0];
    expect(image.getAttribute('src')).toBeTruthy();
    // A link to a section of another chapter keeps its anchor.
    renderWithProviders(<ManualView chapterRef="03-projectberekening" onOpen={onOpen} />);
    const links = screen.getAllByRole('link').filter((link) => link.getAttribute('data-chapter') === '00-werken-met-het-programma');
    expect(links.length).toBeGreaterThan(0);
    await user.click(links.find((link) => link.getAttribute('href')?.includes('#manual-')) ?? links[0]);
    expect(onOpen.mock.calls.at(-1)?.[0]).toMatch(/^00-werken-met-het-programma/);
  });

  it('classifies links: chapters, anchors, outside pages and project documents', () => {
    expect(manualLinkTarget('03-projectberekening.md#koeling', 'index')).toEqual({ kind: 'chapter', target: { chapter: '03-projectberekening', anchor: 'koeling' } });
    expect(manualLinkTarget('#koeling', '03-projectberekening')).toEqual({ kind: 'chapter', target: { chapter: '03-projectberekening', anchor: 'koeling' } });
    expect(manualLinkTarget('https://example.org', 'index')).toEqual({ kind: 'external', href: 'https://example.org' });
    expect(manualLinkTarget('../nta8800-api.md', 'index')).toEqual({ kind: 'document', path: 'docs/nta8800-api.md' });
    expect(manualLinkTarget('javascript:alert(1)', 'index').kind).toBe('document');
  });

  it('is a tool page whose route keeps the chapter', () => {
    expect(TOOL_STEP.subs.map((sub) => sub.id)).toContain('manual');
    expect(normalizeRoute({ step: 'tool', sub: 'manual', chapter: '04-uitvoer#label' })).toEqual({ step: 'tool', sub: 'manual', chapter: '04-uitvoer#label' });
    expect(normalizeRoute({ step: 'tool', sub: 'uvalue', chapter: '04-uitvoer' })).toEqual({ step: 'tool', sub: 'uvalue' });
    expect(splitChapterRef('04-uitvoer#label')).toEqual({ chapter: '04-uitvoer', anchor: 'label' });
    expect(splitChapterRef(undefined)).toEqual({ chapter: 'index' });
  });

  it('links every workflow page to the chapter that explains it', async () => {
    expect(chapterForRoute({ step: 'survey', sub: 'heating' })).toBe('02-basisopname');
    expect(chapterForRoute({ step: 'results' })).toBe('04-uitvoer');
    expect(chapterForRoute({ step: 'report', sub: 'checklist' })).toBe('07-herlabelen-registratie-dossier');
    const ids = new Set(MANUAL_CHAPTERS.map((chapter) => chapter.id));
    for (const step of ['project', 'building', 'installations', 'check', 'results', 'survey', 'advice', 'relabel', 'report', 'registration', 'tool'] as const) {
      expect(ids.has(chapterForRoute({ step }))).toBe(true);
    }
    const navigate = vi.fn();
    const user = userEvent.setup();
    renderWithProviders(
      <ShellActionsProvider value={{ navigate } as unknown as ShellActions}>
        <PageHeader route={{ step: 'check', sub: 'overview' }} />
      </ShellActionsProvider>,
    );
    await user.click(screen.getByRole('button', { name: 'Manual' }));
    expect(navigate).toHaveBeenCalledWith({ step: 'tool', sub: 'manual', chapter: '05-validatie' });
  });
});
