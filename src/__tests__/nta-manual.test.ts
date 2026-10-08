import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import {
  buildManualHtml, checkManualStamp, inlineText, manualStamp, orderChapters, parseInline, parseMarkdown, readManual,
  renderHtml, resolveManualLink, slugify,
} from '../../scripts/nta-manual.mjs';
import { kernelVersion } from '../../scripts/nta-kernel-version.mjs';
import { readIdentity, renderLeveringsdocument } from '../../scripts/nta-leveringsdocument.mjs';

const root = join(__dirname, '../..');

describe('manual Markdown parser (BRL 9501 §4.4)', () => {
  it('parses inline code, emphasis, links, images and sub/sup', () => {
    const nodes = parseInline('**Opslaan** met `<projectnaam>.oes.json`, *zie* [hfst 7](07-x.md#a) en TO<sub>juli</sub> ![alt](img/a.png)');
    expect(nodes.map((node) => node.type)).toEqual(['strong', 'text', 'code', 'text', 'em', 'text', 'link', 'text', 'sub', 'text', 'image']);
    expect(nodes[2]).toEqual({ type: 'code', value: '<projectnaam>.oes.json' });
    expect(nodes[6]).toMatchObject({ type: 'link', href: '07-x.md#a' });
    expect(inlineText(nodes)).toContain('TOjuli');
  });

  it('parses nested lists, tables and unique heading anchors', () => {
    const blocks = parseMarkdown([
      '# Titel', '', '## Stap', '', '1. een', '   - a', '   - b', '2. twee', '', '| A | B |', '|---|--:|', '| `x\\|y` | 2 |', '', '## Stap',
    ].join('\n'));
    expect(blocks.map((block) => block.type)).toEqual(['heading', 'heading', 'list', 'table', 'heading']);
    const list = blocks[2];
    expect(list.type === 'list' && list.ordered && list.items.length).toBe(2);
    expect(list.type === 'list' && list.items[0].blocks[0].type).toBe('list');
    const table = blocks[3];
    expect(table.type === 'table' && table.align).toEqual([undefined, 'right']);
    expect(table.type === 'table' && table.rows[0][0]).toEqual([{ type: 'code', value: 'x|y' }]);
    expect(blocks[1].type === 'heading' && blocks[1].id).toBe('stap');
    expect(blocks[4].type === 'heading' && blocks[4].id).toBe('stap-1');
  });

  it('makes GitHub-style anchors', () => {
    expect(slugify('Koelvermogen volgens bijlage AA')).toBe('koelvermogen-volgens-bijlage-aa');
    expect(slugify('NTA-invoer in de stappen')).toBe('nta-invoer-in-de-stappen');
    expect(slugify('Bron & bewijs')).toBe('bron--bewijs');
  });

  it('escapes all text and drops unsafe links in the HTML export', () => {
    const html = renderHtml(parseMarkdown('<script>alert(1)</script> en [klik](javascript:alert(1)) en ![x](javascript:y)'));
    expect(html).not.toContain('<script>');
    expect(html).toContain('&lt;script&gt;');
    expect(html).not.toMatch(/href="javascript/);
    expect(html).not.toMatch(/src="javascript/);
  });
});

describe('manual version stamp', () => {
  it('reads the stamp and compares it with KERNEL_VERSION', () => {
    expect(manualStamp('<!-- handleiding: rekenkern 1.2.3 -->')).toEqual({ kernelVersion: '1.2.3' });
    expect(checkManualStamp('<!-- handleiding: rekenkern 1.2.3 -->', '1.2.3').ok).toBe(true);
    expect(checkManualStamp('<!-- handleiding: rekenkern 1.2.3 -->', '1.3.0')).toMatchObject({ ok: false });
    expect(checkManualStamp('# zonder stempel', '1.2.3').ok).toBe(false);
  });

  it('the manual in the repository is stamped for the current kernel version', () => {
    const index = readFileSync(join(root, 'docs/handleiding-nta8800/index.md'), 'utf8');
    expect(checkManualStamp(index, kernelVersion(root))).toMatchObject({ ok: true });
  });
});

describe('manual release export', () => {
  it('orders the index first, then the chapters by number', () => {
    expect(orderChapters(['02-b.md', 'index.md', '00-a.md', 'img'])).toEqual(['index.md', '00-a.md', '02-b.md']);
  });

  it('turns chapter links into in-page anchors and other documents into references', () => {
    const ids = ['index', '03-projectberekening'];
    expect(resolveManualLink('03-projectberekening.md#koeling', ids, 'index')).toEqual({ href: '#03-projectberekening--koeling' });
    expect(resolveManualLink('#koeling', ids, '03-projectberekening')).toEqual({ href: '#03-projectberekening--koeling' });
    expect(resolveManualLink('https://example.org', ids, 'index')).toEqual({ href: 'https://example.org' });
    expect(resolveManualLink('../nta8800-api.md', ids, 'index')).toMatchObject({ href: null, title: expect.stringContaining('docs/nta8800-api.md') });
  });

  it('exports the whole manual as one file: every chapter, every image embedded, every anchor resolvable', () => {
    const manual = readManual(root);
    const html = buildManualHtml({ ...manual, programVersion: '9.9.9', kernelVersion: '0.2.0', generatedOn: '2026-10-09' });
    expect(manual.chapters[0].id).toBe('index');
    for (const chapter of manual.chapters) expect(html).toContain(`<section class="chapter" id="${chapter.id}">`);
    expect(html).toContain('Programmaversie 9.9.9 · rekenkern 0.2.0 · 2026-10-09');
    expect(html).toContain('<meta name="kernel-version" content="0.2.0">');
    const ids = new Set([...html.matchAll(/ id="([^"]+)"/g)].map((match) => match[1]));
    const anchors = [...html.matchAll(/href="#([^"]+)"/g)].map((match) => match[1]);
    expect(anchors.length).toBeGreaterThan(10);
    expect(anchors.filter((anchor) => !ids.has(anchor))).toEqual([]);
    const images = [...html.matchAll(/<img src="([^"]+)"/g)].map((match) => match[1]);
    expect(images.length).toBe(Object.keys(manual.images).length);
    expect(images.every((src) => src.startsWith('data:image/'))).toBe(true);
  });

  it('lists the manual with its SHA-256 in the leveringsdocument', () => {
    const template = readFileSync(join(root, 'docs/templates/nta8800-leveringsdocument.md'), 'utf8');
    const values = { ...readIdentity(root), releaseDate: '9 oktober 2026', packages: [] };
    expect(renderLeveringsdocument(template, values)).toContain('| (geen handleiding meegeleverd) | – |');
    const text = renderLeveringsdocument(template, { ...values, manual: [{ name: 'handleiding-nta8800-1.0.html', sha256: 'a'.repeat(64) }] });
    expect(text).toContain(`| \`handleiding-nta8800-1.0.html\` | \`${'a'.repeat(64)}\` |`);
    expect(text).not.toMatch(/\{\{\w+\}\}/);
  });
});
