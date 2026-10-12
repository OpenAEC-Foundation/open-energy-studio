import { describe, expect, it } from 'vitest';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';

// Keys defined in en.ts and the spread label files (nl.ts mirrors them, which
// i18n-coverage.test.ts checks) must be used somewhere in src/. A key counts
// as used when it appears as a string literal, or when a literal or template
// prefix builds it at run time:
// - `kernel.issue.${code}` and `nta.gap.${code}` (a template prefix);
// - options(t, 'nta.annexP.carrier', CARRIERS) or const p = 'ntaStep.em23'
//   (a literal that is itself a key prefix, completed with '.' + value);
// - 'nta.warning.' + code (a literal ending in a dot).
// Keys that are only built by data outside src/ go in this list with a reason.
const ALLOWED_UNUSED: Record<string, string> = {};

const I18N = 'src/i18n';

function sourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return name === '__tests__' ? [] : sourceFiles(path);
    return /\.tsx?$/.test(name) ? [path] : [];
  });
}

function definedKeys(): string[] {
  const files = ['en.ts', ...readdirSync(I18N).filter((name) => name.endsWith('Labels.ts'))];
  const keys = new Set<string>();
  for (const file of files) {
    for (const match of readFileSync(join(I18N, file), 'utf8').matchAll(/^\s*['"]([A-Za-z0-9_.-]+)['"]\s*:/gm)) {
      keys.add(match[1]);
    }
  }
  return [...keys];
}

describe('i18n keys', () => {
  it('defines no key that src/ never uses', () => {
    const keys = definedKeys();
    // Code outside the translation tables, plus the i18n helpers that build keys.
    const helpers = new Set(['format.ts', 'i18n.ts', 'kernelDetail.ts', 'I18nProvider.tsx']);
    const code = sourceFiles('src')
      .filter((path) => !path.startsWith(`${I18N}/`) || helpers.has(path.slice(I18N.length + 1)))
      .map((path) => readFileSync(path, 'utf8'))
      .join('\n');
    const literals = new Set([...code.matchAll(/['"`]([A-Za-z0-9_.-]+)['"`]/g)].map((match) => match[1]));
    const prefixes = new Set([
      ...[...code.matchAll(/`([A-Za-z0-9_.-]+)\$\{/g)].map((match) => match[1]),
      ...[...code.matchAll(/['"]([A-Za-z][A-Za-z0-9_-]*(?:\.[A-Za-z0-9_-]+)*\.)['"]/g)].map((match) => match[1]),
    ]);
    const keyPrefixes = new Set(keys.flatMap((key) => {
      const parts = key.split('.');
      return parts.slice(1).map((_, index) => parts.slice(0, index + 1).join('.'));
    }));
    // A literal counts as a key prefix only when it has a dot ("nta.annexP.carrier"):
    // a one-word literal such as 'nta' or 'overview' would cover a whole namespace.
    for (const literal of literals) if (literal.includes('.') && keyPrefixes.has(literal)) prefixes.add(`${literal}.`);
    // A plural form (`key_zero`, `key_one`, `key_other`) is used when its base key is: i18next
    // picks the form from `count` (t('nav.state.errors', { count })).
    const base = (key: string) => key.replace(/_(zero|one|two|few|many|other)$/, '');
    const unused = keys.filter((key) => !literals.has(key) && !literals.has(base(key))
      && ![...prefixes].some((prefix) => key.startsWith(prefix))
      && !(key in ALLOWED_UNUSED));
    expect(unused).toEqual([]);
  });

  it('keeps the allow list honest', () => {
    const keys = new Set(definedKeys());
    expect(Object.keys(ALLOWED_UNUSED).filter((key) => !keys.has(key))).toEqual([]);
  });
});
