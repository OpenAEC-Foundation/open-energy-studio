/**
 * F10 contrast audit: resolves the colour tokens of every theme
 * (src/styles/tokens.css plus the legacy variables in src/index.css)
 * and checks WCAG 2.x contrast for the pairs the UI actually uses.
 *
 * Text: at least 4.5:1. UI components and status marks: at least 3:1.
 */
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'fs';
import { resolve } from 'path';

type Rgba = [number, number, number, number];
type Vars = Record<string, string>;

const tokensCss = readFileSync(resolve(__dirname, '../styles/tokens.css'), 'utf-8');
const legacyCss = readFileSync(resolve(__dirname, '../index.css'), 'utf-8');

/** Collect custom properties declared in blocks whose selector matches. */
function collect(css: string, matches: (selector: string) => boolean): Vars {
  const vars: Vars = {};
  const block = /([^{}]+)\{([^{}]*)\}/g;
  let m: RegExpExecArray | null;
  while ((m = block.exec(css))) {
    const selector = m[1].replace(/\/\*[\s\S]*?\*\//g, '').trim();
    if (!matches(selector)) continue;
    const decl = /(--[a-z0-9-]+)\s*:\s*([^;]+);/gi;
    let d: RegExpExecArray | null;
    while ((d = decl.exec(m[2]))) vars[d[1]] = d[2].replace(/\/\*[\s\S]*?\*\//g, '').trim();
  }
  return vars;
}

function themeVars(theme: 'dark' | 'light' | 'highContrast'): Vars {
  const isRoot = (s: string) => s.split(',').map((x) => x.trim()).includes(':root');
  const isTheme = (s: string) => s.includes(`[data-theme="${theme}"]`);
  const base = { ...collect(tokensCss, (s) => s === ':root'), ...collect(legacyCss, (s) => isRoot(s) && !s.includes('[data-theme')) };
  const darkDefault = theme === 'dark' ? {} : {};
  return {
    ...base,
    ...collect(tokensCss, (s) => (theme === 'dark' ? isRoot(s) || isTheme(s) : isTheme(s))),
    ...collect(legacyCss, (s) => (theme === 'dark' ? isRoot(s) || isTheme(s) : isTheme(s))),
    ...darkDefault,
  };
}

function parseColor(value: string, vars: Vars, depth = 0): Rgba {
  const v = value.trim();
  const ref = /^var\((--[a-z0-9-]+)(?:\s*,\s*(.+))?\)$/i.exec(v);
  if (ref) {
    if (depth > 10) throw new Error(`cycle at ${v}`);
    const target = vars[ref[1]] ?? ref[2];
    if (!target) throw new Error(`undefined ${ref[1]}`);
    return parseColor(target, vars, depth + 1);
  }
  const hex = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(v);
  if (hex) {
    const h = hex[1].length === 3 ? hex[1].split('').map((c) => c + c).join('') : hex[1];
    return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16), 1];
  }
  const rgba = /^rgba?\(\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)\s*(?:,\s*([\d.]+)\s*)?\)$/i.exec(v);
  if (rgba) return [Number(rgba[1]), Number(rgba[2]), Number(rgba[3]), rgba[4] === undefined ? 1 : Number(rgba[4])];
  throw new Error(`unparsed colour ${v}`);
}

function over(top: Rgba, bottom: Rgba): Rgba {
  const a = top[3];
  return [top[0] * a + bottom[0] * (1 - a), top[1] * a + bottom[1] * (1 - a), top[2] * a + bottom[2] * (1 - a), 1];
}

function luminance([r, g, b]: Rgba): number {
  const lin = (c: number) => {
    const s = c / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
}

function contrast(fg: string, bg: string, vars: Vars, canvas = '--surface-canvas'): number {
  const ground = parseColor(`var(${canvas})`, vars);
  const back = over(parseColor(`var(${bg})`, vars), ground);
  const front = over(parseColor(`var(${fg})`, vars), back);
  const [l1, l2] = [luminance(front), luminance(back)].sort((a, b) => b - a);
  return (l1 + 0.05) / (l2 + 0.05);
}

const TEXT_SURFACES = ['--surface-card', '--surface-canvas', '--surface-chrome', '--surface-raised', '--surface-sunken'];
const TEXT_TOKENS = ['--fg-1', '--fg-2', '--fg-3', '--accent-text', '--ok-text', '--warn-text', '--error-text', '--info-text', '--unverified'];
/** Status text on its own subtle background (pills, banners). */
const PILL_PAIRS: Array<[string, string]> = [
  ['--ok-text', '--ok-subtle'], ['--warn-text', '--warn-subtle'], ['--error-text', '--error-subtle'],
  ['--info-text', '--info-subtle'], ['--unverified', '--unverified-subtle'], ['--accent-text', '--accent-subtle'],
  ['--fg-1', '--surface-selected'], ['--fg-1', '--surface-hover'],
];
/** Non-text UI: status marks, accent fills, focus colour. */
/**
 * The brand accent fill (Construction Amber) is exempt: it always carries
 * --fg-on-accent text (checked above) and --accent-text is used for amber
 * text on surfaces. Decorative separators (--line, --line-strong) are not
 * component boundaries; input borders use --line-control.
 */
const UI_TOKENS = ['--ok', '--warn', '--error', '--info', '--line-control'];
/** Legacy variables still used by older panels. */
const LEGACY_TEXT: Array<[string, string]> = [
  ['--text-primary', '--bg-primary'], ['--text-secondary', '--bg-primary'], ['--text-muted', '--bg-primary'],
  ['--text-primary', '--bg-secondary'], ['--text-secondary', '--bg-secondary'], ['--text-muted', '--bg-secondary'],
  ['--text-muted', '--bg-dark'],
];

describe.each(['dark', 'light', 'highContrast'] as const)('contrast in the %s theme', (theme) => {
  const vars = themeVars(theme);
  const failures: string[] = [];

  it('text tokens reach 4.5:1 on every surface', () => {
    for (const surface of TEXT_SURFACES) {
      for (const text of TEXT_TOKENS) {
        if (!vars[text] || !vars[surface]) continue;
        const ratio = contrast(text, surface, vars);
        if (ratio < 4.5) failures.push(`${text} on ${surface}: ${ratio.toFixed(2)}`);
      }
    }
    expect(failures).toEqual([]);
  });

  it('status text reaches 4.5:1 on its subtle background', () => {
    const bad: string[] = [];
    for (const [text, back] of PILL_PAIRS) {
      if (!vars[text] || !vars[back]) continue;
      for (const ground of ['--surface-card', '--surface-canvas']) {
        const ratio = contrast(text, back, vars, ground);
        if (ratio < 4.5) bad.push(`${text} on ${back} over ${ground}: ${ratio.toFixed(2)}`);
      }
    }
    expect(bad).toEqual([]);
  });

  it('text on an accent fill reaches 4.5:1', () => {
    expect(contrast('--fg-on-accent', '--accent', vars)).toBeGreaterThanOrEqual(4.5);
  });

  it('UI marks reach 3:1 against cards and canvas', () => {
    const bad: string[] = [];
    for (const token of UI_TOKENS) {
      if (!vars[token]) continue;
      for (const surface of ['--surface-card', '--surface-canvas']) {
        const ratio = contrast(token, surface, vars);
        if (ratio < 3) bad.push(`${token} on ${surface}: ${ratio.toFixed(2)}`);
      }
    }
    expect(bad).toEqual([]);
  });

  it('legacy text variables reach 4.5:1', () => {
    const bad: string[] = [];
    for (const [text, back] of LEGACY_TEXT) {
      if (!vars[text] || !vars[back]) continue;
      const ratio = contrast(text, back, vars);
      if (ratio < 4.5) bad.push(`${text} on ${back}: ${ratio.toFixed(2)}`);
    }
    expect(bad).toEqual([]);
  });
});
