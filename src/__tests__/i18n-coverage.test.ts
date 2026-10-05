/**
 * F10 i18n check: every translation key used in src exists in Dutch and
 * English, the two languages have the same keys, and no Dutch text is
 * hard-coded where English is expected (and vice versa) in the shell.
 */
import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync, statSync } from 'fs';
import { join, resolve, relative } from 'path';
import { en } from '../i18n/en';
import { nl } from '../i18n/nl';

const SRC = resolve(__dirname, '..');

function sources(dir: string): string[] {
  const out: string[] = [];
  for (const name of readdirSync(dir)) {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) {
      if (name === '__tests__' || name === 'i18n' || name === 'wasm' || name === 'pkg') continue;
      out.push(...sources(path));
    } else if (/\.(ts|tsx)$/.test(name) && !/\.(test|spec)\.tsx?$/.test(name) && !name.endsWith('.d.ts')) {
      out.push(path);
    }
  }
  return out;
}

const files = sources(SRC).map((path) => ({ path: relative(SRC, path), text: readFileSync(path, 'utf-8') }));

/** Static keys: t('a.b') / t("a.b"); template keys: t(`a.${x}`) → prefix 'a.'. */
function usedKeys() {
  const literal = new Map<string, string>();
  const prefixes = new Map<string, string>();
  const call = /(?<![\w.])t\(\s*(['"`])([^'"`$]*)(\$\{)?[^'"`]*?\1/g;
  for (const { path, text } of files) {
    let m: RegExpExecArray | null;
    while ((m = call.exec(text))) {
      const key = m[2];
      if (!/^[a-zA-Z][\w-]*(\.[\w-]+)*\.?$/.test(key) || !key.includes('.')) continue;
      if (m[3]) prefixes.set(key, path);
      else if (!key.endsWith('.')) literal.set(key, path);
    }
  }
  return { literal, prefixes };
}

describe('i18n coverage (nl and en)', () => {
  const { literal, prefixes } = usedKeys();

  it('finds the keys the code uses', () => {
    expect(literal.size).toBeGreaterThan(500);
  });

  it('every literal t() key exists in Dutch and English', () => {
    const missing: string[] = [];
    for (const [key, path] of literal) {
      if (!(key in nl)) missing.push(`nl: ${key} (${path})`);
      if (!(key in en)) missing.push(`en: ${key} (${path})`);
    }
    expect(missing).toEqual([]);
  });

  it('every template key prefix has translations in both languages', () => {
    const missing: string[] = [];
    for (const [prefix, path] of prefixes) {
      if (!Object.keys(nl).some((k) => k.startsWith(prefix))) missing.push(`nl: ${prefix}… (${path})`);
      if (!Object.keys(en).some((k) => k.startsWith(prefix))) missing.push(`en: ${prefix}… (${path})`);
    }
    expect(missing).toEqual([]);
  });

  it('Dutch and English define the same keys', () => {
    const onlyNl = Object.keys(nl).filter((k) => !(k in en));
    const onlyEn = Object.keys(en).filter((k) => !(k in nl));
    expect({ onlyNl, onlyEn }).toEqual({ onlyNl: [], onlyEn: [] });
  });

  it('no Dutch value is left untranslated in English (heuristic)', () => {
    const dutch = /\b(het|een|niet|wordt|worden|zijn|voor|naar|gebouw|berekening|invoer|opslaan|bestand|toevoegen|verwijderen|controleer|waarde)\b/i;
    const english = /\b(the|and|with|from|building|calculation|save|file|add|remove|check|value|should|cannot)\b/i;
    const suspect = Object.keys(en).filter((k) => {
      const v = en[k];
      return v === nl[k] && v.split(/\s+/).length >= 3 && dutch.test(v) && !english.test(v);
    });
    expect(suspect).toEqual([]);
  });
});

describe('hard-coded text in the shell (heuristic)', () => {
  /**
   * JSX text and aria/title/placeholder attributes in the new shell and ui
   * building blocks must come from t(). A capitalised word of four or more
   * letters directly between tags, or as an attribute literal, is reported.
   */
  const shellFiles = files.filter((f) => /^components\/(shell|ui)\//.test(f.path) && f.path.endsWith('.tsx'));
  const ALLOWED = new Set(['NTA', 'BENG', 'EP-Online', 'UNIEC', 'VABI', 'IFC', 'TO-juli', 'PDF', 'Ctrl', 'Enter', 'Esc', 'OpenAEC', 'Open Energy Studio']);

  it('has no literal user-facing words in JSX', () => {
    const found: string[] = [];
    // Text between a closing '>' and an opening '<' that is not a type argument (Promise<T>).
    const jsxText = /(?<![=-])>\s*([A-Z][a-zà-ÿ]{3,}(?:\s+[a-zà-ÿ]+)*)\s*<(?![A-Z]\w*>)/g;
    const attr = /\b(aria-label|title|placeholder|alt)=["']([A-Z][a-zà-ÿ]{3,}[^"']*)["']/g;
    for (const { path, text } of shellFiles) {
      let m: RegExpExecArray | null;
      while ((m = jsxText.exec(text))) if (!ALLOWED.has(m[1].trim())) found.push(`${path}: ${m[1].trim()}`);
      while ((m = attr.exec(text))) if (!ALLOWED.has(m[2].trim())) found.push(`${path}: ${m[1]}="${m[2]}"`);
    }
    expect(found).toEqual([]);
  });
});
