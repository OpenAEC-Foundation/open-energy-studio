// Markdown parser and HTML renderer of the NTA 8800 user manual, without any
// Node imports, so the app bundles it for the in-app viewer and
// scripts/nta-manual.mjs uses it for the release (BRL 9501 §4.4, task B7).
//
// The parser covers the Markdown the manual uses: headings, paragraphs,
// (nested) lists, tables, block quotes, rules, fenced code, and inline code,
// bold, italic, links, images and <sub>/<sup>. Everything else is text; the
// HTML renderer escapes all text, so the manual cannot inject markup.

export const MANUAL_DIR = 'docs/handleiding-nta8800';
const STAMP = /<!--\s*handleiding:\s*rekenkern\s+(\d+\.\d+\.\d+)\s*-->/;

// ---------------------------------------------------------------- inline

const INLINE_HTML = /^<(\/?)(sub|sup)>/i;

/** Inline Markdown to nodes: text, code, strong, em, link, image, sub, sup. */
export function parseInline(text) {
  const nodes = [];
  let buffer = '';
  const flush = () => { if (buffer) { nodes.push({ type: 'text', value: buffer }); buffer = ''; } };
  let i = 0;
  while (i < text.length) {
    const rest = text.slice(i);
    const ch = text[i];
    if (ch === '\\' && i + 1 < text.length && /[\\`*_[\]()#+\-.!<>|]/.test(text[i + 1])) {
      buffer += text[i + 1]; i += 2; continue;
    }
    if (ch === '`') {
      const ticks = /^`+/.exec(rest)[0];
      const end = text.indexOf(ticks, i + ticks.length);
      if (end > 0) {
        flush();
        nodes.push({ type: 'code', value: text.slice(i + ticks.length, end).trim() || text.slice(i + ticks.length, end) });
        i = end + ticks.length; continue;
      }
    }
    if (ch === '!' && text[i + 1] === '[') {
      const link = matchLink(text, i + 1);
      if (link) { flush(); nodes.push({ type: 'image', alt: link.label, src: link.href }); i = link.end; continue; }
    }
    if (ch === '[') {
      const link = matchLink(text, i);
      if (link) { flush(); nodes.push({ type: 'link', href: link.href, children: parseInline(link.label) }); i = link.end; continue; }
    }
    if ((ch === '*' || ch === '_') && text[i + 1] === ch) {
      const end = text.indexOf(ch + ch, i + 2);
      if (end > i + 2) { flush(); nodes.push({ type: 'strong', children: parseInline(text.slice(i + 2, end)) }); i = end + 2; continue; }
    }
    if (ch === '*' || (ch === '_' && !/\w/.test(text[i - 1] ?? ''))) {
      const end = findEmphasisEnd(text, i + 1, ch);
      if (end > i + 1) { flush(); nodes.push({ type: 'em', children: parseInline(text.slice(i + 1, end)) }); i = end + 1; continue; }
    }
    const html = INLINE_HTML.exec(rest);
    if (html && !html[1]) {
      const tag = html[2].toLowerCase();
      const close = text.toLowerCase().indexOf(`</${tag}>`, i + html[0].length);
      if (close > 0) {
        flush();
        nodes.push({ type: tag, children: parseInline(text.slice(i + html[0].length, close)) });
        i = close + tag.length + 3; continue;
      }
    }
    buffer += ch; i += 1;
  }
  flush();
  return nodes;
}

function findEmphasisEnd(text, from, marker) {
  for (let j = from; j < text.length; j += 1) {
    if (text[j] === '`') { const end = text.indexOf('`', j + 1); if (end < 0) return -1; j = end; continue; }
    if (text[j] === marker && text[j + 1] !== marker && text[j - 1] !== ' ' && (marker === '*' || !/\w/.test(text[j + 1] ?? ''))) return j;
  }
  return -1;
}

function matchLink(text, start) {
  let depth = 0;
  let j = start;
  for (; j < text.length; j += 1) {
    if (text[j] === '[') depth += 1;
    else if (text[j] === ']') { depth -= 1; if (depth === 0) break; }
  }
  if (depth !== 0 || text[j + 1] !== '(') return null;
  const close = text.indexOf(')', j + 2);
  if (close < 0) return null;
  return { label: text.slice(start + 1, j), href: text.slice(j + 2, close).trim(), end: close + 1 };
}

/** Plain text of inline nodes (for anchors and alt texts). */
export function inlineText(nodes) {
  return nodes.map((node) => (node.type === 'text' || node.type === 'code' ? node.value
    : node.type === 'image' ? node.alt : inlineText(node.children ?? []))).join('');
}

/** GitHub-style heading anchor: lower case, punctuation dropped, spaces to "-". */
export function slugify(text) {
  return text.toLowerCase().trim().replace(/[^\p{L}\p{N}\s_-]/gu, '').replace(/\s/g, '-');
}

// ---------------------------------------------------------------- blocks

const LIST_ITEM = /^(\s*)([-*+]|\d+[.)])\s+(.*)$/;

/** Block Markdown to nodes: heading, paragraph, list, table, quote, rule, code. */
export function parseMarkdown(source) {
  const lines = source.replace(/\r\n?/g, '\n').split('\n');
  const blocks = [];
  const slugs = new Map();
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    if (!line.trim() || /^\s*<!--.*-->\s*$/.test(line)) { i += 1; continue; }
    const fence = /^\s*(```|~~~)(.*)$/.exec(line);
    if (fence) {
      const body = [];
      i += 1;
      while (i < lines.length && !lines[i].trim().startsWith(fence[1])) { body.push(lines[i]); i += 1; }
      blocks.push({ type: 'code', lang: fence[2].trim(), value: body.join('\n') });
      i += 1; continue;
    }
    const heading = /^(#{1,6})\s+(.*?)\s*#*\s*$/.exec(line);
    if (heading) {
      const children = parseInline(heading[2]);
      const base = slugify(inlineText(children));
      const count = slugs.get(base) ?? 0;
      slugs.set(base, count + 1);
      blocks.push({ type: 'heading', level: heading[1].length, id: count ? `${base}-${count}` : base, children });
      i += 1; continue;
    }
    if (/^\s*([-*_])(\s*\1){2,}\s*$/.test(line)) { blocks.push({ type: 'rule' }); i += 1; continue; }
    if (/^\s*>/.test(line)) {
      const body = [];
      while (i < lines.length && /^\s*>/.test(lines[i])) { body.push(lines[i].replace(/^\s*>\s?/, '')); i += 1; }
      blocks.push({ type: 'quote', children: parseMarkdown(body.join('\n')) });
      continue;
    }
    if (/^\s*\|/.test(line) && i + 1 < lines.length && /^\s*\|?\s*:?-{3,}/.test(lines[i + 1])) {
      const header = splitRow(line);
      const align = splitRow(lines[i + 1]).map((cell) => (/^:-+:$/.test(cell) ? 'center' : /-:$/.test(cell) ? 'right' : undefined));
      const rows = [];
      i += 2;
      while (i < lines.length && /^\s*\|/.test(lines[i])) { rows.push(splitRow(lines[i]).map(parseInline)); i += 1; }
      blocks.push({ type: 'table', header: header.map(parseInline), align, rows });
      continue;
    }
    if (LIST_ITEM.test(line)) {
      const { list, next } = parseList(lines, i);
      blocks.push(list);
      i = next; continue;
    }
    const body = [line.trim()];
    i += 1;
    while (i < lines.length && lines[i].trim() && !/^(#{1,6}\s|\s*>|\s*\||\s*```)/.test(lines[i]) && !LIST_ITEM.test(lines[i])) {
      body.push(lines[i].trim()); i += 1;
    }
    blocks.push({ type: 'paragraph', children: parseInline(body.join(' ')) });
  }
  return blocks;
}

function splitRow(line) {
  const trimmed = line.trim().replace(/^\|/, '').replace(/\|$/, '');
  const cells = [];
  let cell = '';
  let code = false;
  for (let j = 0; j < trimmed.length; j += 1) {
    const ch = trimmed[j];
    if (ch === '\\' && trimmed[j + 1] === '|') { cell += '|'; j += 1; continue; }
    if (ch === '`') code = !code;
    if (ch === '|' && !code) { cells.push(cell.trim()); cell = ''; continue; }
    cell += ch;
  }
  cells.push(cell.trim());
  return cells;
}

function parseList(lines, start) {
  const first = LIST_ITEM.exec(lines[start]);
  const indent = first[1].length;
  const ordered = /\d/.test(first[2]);
  const list = { type: 'list', ordered, start: ordered ? Number.parseInt(first[2], 10) : undefined, items: [] };
  let i = start;
  while (i < lines.length) {
    const match = LIST_ITEM.exec(lines[i]);
    if (!match || match[1].length < indent) break;
    if (match[1].length > indent) break; // handled as a child below
    const text = [match[3]];
    const children = [];
    i += 1;
    while (i < lines.length) {
      const line = lines[i];
      if (!line.trim()) {
        const next = lines[i + 1];
        if (next && (LIST_ITEM.exec(next)?.[1].length ?? -1) > indent) { i += 1; continue; }
        break;
      }
      const child = LIST_ITEM.exec(line);
      if (child && child[1].length > indent) {
        const nested = parseList(lines, i);
        children.push(nested.list);
        i = nested.next; continue;
      }
      if (child) break;
      if (/^\s+\S/.test(line) && !/^(#{1,6}\s|\s*\|)/.test(line)) { text.push(line.trim()); i += 1; continue; }
      break;
    }
    list.items.push({ children: parseInline(text.join(' ')), blocks: children });
  }
  return { list, next: i };
}

// ---------------------------------------------------------------- HTML

const ESCAPES = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' };
export function escapeHtml(text) {
  return String(text).replace(/[&<>"']/g, (ch) => ESCAPES[ch]);
}

/**
 * Only http(s), mailto, in-page and relative links; anything else is dropped.
 * Browsers remove tabs, newlines and other control characters from a URL, so
 * `java\tscript:` is still a script link: a link with a control character
 * is refused, and the scheme is tested with whitespace removed.
 */
export function safeHref(href) {
  const value = String(href).trim();
  // eslint-disable-next-line no-control-regex
  if (/[\u0000-\u001f\u007f]/.test(value)) return null;
  const probe = value.replace(/\s+/g, '');
  if (/^(https?:|mailto:)/i.test(probe)) return value;
  if (/^[a-z][a-z0-9+.-]*:/i.test(probe)) return null;
  return value;
}

function inlineHtml(nodes, options) {
  return nodes.map((node) => {
    switch (node.type) {
      case 'text': return escapeHtml(node.value);
      case 'code': return `<code>${escapeHtml(node.value)}</code>`;
      case 'strong': return `<strong>${inlineHtml(node.children, options)}</strong>`;
      case 'em': return `<em>${inlineHtml(node.children, options)}</em>`;
      case 'sub': case 'sup': return `<${node.type}>${inlineHtml(node.children, options)}</${node.type}>`;
      case 'image': {
        const src = options.resolveImage ? options.resolveImage(node.src) : safeHref(node.src);
        return src ? `<img src="${escapeHtml(src)}" alt="${escapeHtml(node.alt)}">` : escapeHtml(node.alt);
      }
      case 'link': {
        const label = inlineHtml(node.children, options);
        const resolved = options.resolveLink ? options.resolveLink(node.href) : { href: safeHref(node.href) };
        if (!resolved?.href) return `<span class="manual-doc-ref"${resolved?.title ? ` title="${escapeHtml(resolved.title)}"` : ''}>${label}</span>`;
        const external = /^https?:/i.test(resolved.href);
        return `<a href="${escapeHtml(resolved.href)}"${external ? ' rel="noopener noreferrer" target="_blank"' : ''}>${label}</a>`;
      }
      default: return '';
    }
  }).join('');
}

/** Blocks to an HTML string; every text is escaped. */
export function renderHtml(blocks, options = {}) {
  return blocks.map((block) => {
    switch (block.type) {
      case 'heading': {
        const id = options.idPrefix ? `${options.idPrefix}${block.id}` : block.id;
        return `<h${block.level} id="${escapeHtml(id)}">${inlineHtml(block.children, options)}</h${block.level}>`;
      }
      case 'paragraph': return `<p>${inlineHtml(block.children, options)}</p>`;
      case 'rule': return '<hr>';
      case 'code': return `<pre><code>${escapeHtml(block.value)}</code></pre>`;
      case 'quote': return `<blockquote>${renderHtml(block.children, options)}</blockquote>`;
      case 'list': {
        const tag = block.ordered ? 'ol' : 'ul';
        const start = block.ordered && block.start !== 1 ? ` start="${block.start}"` : '';
        return `<${tag}${start}>${block.items.map((item) => `<li>${inlineHtml(item.children, options)}${renderHtml(item.blocks, options)}</li>`).join('')}</${tag}>`;
      }
      case 'table': {
        const cell = (tag, nodes, index) => `<${tag}${block.align[index] ? ` style="text-align:${block.align[index]}"` : ''}>${inlineHtml(nodes, options)}</${tag}>`;
        return `<table><thead><tr>${block.header.map((nodes, index) => cell('th', nodes, index)).join('')}</tr></thead>`
          + `<tbody>${block.rows.map((row) => `<tr>${row.map((nodes, index) => cell('td', nodes, index)).join('')}</tr>`).join('')}</tbody></table>`;
      }
      default: return '';
    }
  }).join('\n');
}

// ---------------------------------------------------------------- manual

/** The chapter id of a manual file name ("03-projectberekening.md" → "03-projectberekening"). */
export function chapterId(fileName) {
  return fileName.replace(/\.md$/, '');
}

/** The manual's version stamp in index.md, or null. */
export function manualStamp(indexText) {
  const match = STAMP.exec(indexText);
  return match ? { kernelVersion: match[1] } : null;
}

/** Chapters in reading order: the index first, then 00…10 by name. */
export function orderChapters(fileNames) {
  const chapters = fileNames.filter((name) => name.endsWith('.md') && name !== 'index.md').sort();
  return fileNames.includes('index.md') ? ['index.md', ...chapters] : chapters;
}

/** The version check of the gate: the stamp must name the current KERNEL_VERSION. */
export function checkManualStamp(indexText, currentKernelVersion) {
  const stamp = manualStamp(indexText);
  if (!stamp) {
    return { ok: false, message: `${MANUAL_DIR}/index.md heeft geen versiestempel <!-- handleiding: rekenkern X.Y.Z -->.` };
  }
  if (stamp.kernelVersion !== currentKernelVersion) {
    return {
      ok: false,
      message: `De handleiding is gestempeld voor rekenkern ${stamp.kernelVersion}, KERNEL_VERSION is ${currentKernelVersion}. `
        + 'Loop de handleiding na voor deze kernversie en pas de stempel in index.md aan.',
    };
  }
  return { ok: true, message: `Handleiding gestempeld voor rekenkern ${stamp.kernelVersion}, gelijk aan KERNEL_VERSION.` };
}

/** Resolves a link inside the single-file manual: chapters become in-page anchors. */
export function resolveManualLink(href, chapterIds, currentChapter) {
  const safe = safeHref(href);
  if (!safe) return null;
  if (/^(https?:|mailto:)/i.test(safe)) return { href: safe };
  const [path, hash] = safe.split('#');
  if (!path) return { href: `#${currentChapter}--${hash}` };
  const file = path.replace(/^\.\//, '');
  const id = chapterId(file);
  if (!file.includes('/') && chapterIds.includes(id)) return { href: hash ? `#${id}--${hash}` : `#${id}` };
  return { href: null, title: `Zie ${file.replace(/^\.\.\//, 'docs/')} in de documentatie bij het programma.` };
}

/** The whole manual as one self-contained HTML document. */
export function buildManualHtml({ chapters, images, programVersion, kernelVersion: kernel, generatedOn }) {
  const ids = chapters.map((chapter) => chapter.id);
  const body = chapters.map((chapter) => {
    const blocks = parseMarkdown(chapter.text);
    const html = renderHtml(blocks, {
      idPrefix: `${chapter.id}--`,
      resolveLink: (href) => resolveManualLink(href, ids, chapter.id),
      resolveImage: (src) => images[src.replace(/^\.\//, '')] ?? null,
    });
    return `<section class="chapter" id="${escapeHtml(chapter.id)}">\n${html}\n</section>`;
  }).join('\n');
  const stamp = `Programmaversie ${programVersion} · rekenkern ${kernel}${generatedOn ? ` · ${generatedOn}` : ''}`;
  return `<!doctype html>
<html lang="nl">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="program-version" content="${escapeHtml(programVersion)}">
<meta name="kernel-version" content="${escapeHtml(kernel)}">
<title>Handleiding NTA 8800 — Open Energy Studio ${escapeHtml(programVersion)}</title>
<style>
body{font:15px/1.55 system-ui,sans-serif;max-width:52rem;margin:2rem auto;padding:0 1rem;color:#1d232b;background:#fff}
h1{font-size:1.7rem}h2{font-size:1.3rem;margin-top:2rem}h3{font-size:1.1rem}
.chapter{border-top:2px solid #d5dbe3;margin-top:3rem;padding-top:1rem;page-break-before:always}
.chapter:first-of-type{border-top:0;page-break-before:auto}
.stamp{color:#5a6573;font-size:.9rem}
code{font-family:ui-monospace,monospace;background:#f1f3f6;padding:0 .2em;border-radius:3px}
table{border-collapse:collapse;margin:1rem 0}th,td{border:1px solid #d5dbe3;padding:.3rem .5rem;vertical-align:top}
img{max-width:100%;border:1px solid #d5dbe3}
.manual-doc-ref{text-decoration:underline dotted}
</style>
</head>
<body>
<p class="stamp" id="manual-stamp">${escapeHtml(stamp)}</p>
${body}
</body>
</html>
`;
}
