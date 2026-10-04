import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { nl } from '../i18n/nl';
import { en } from '../i18n/en';
import { KERNEL_CODE_PREFIXES } from '../i18n/format';

/** Every `.rs` file under the crates, without build output. */
function rustFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    if (name === 'target') return [];
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return rustFiles(path);
    return name.endsWith('.rs') ? [path] : [];
  });
}

/**
 * Code literals the kernel and the survey emit: the first string argument of an
 * `issue`/`gap`/`warning`-like call and every `code: "…"` field, outside test modules.
 */
function emittedCodes(): Map<string, string> {
  const call = /\b([A-Za-z_][A-Za-z0-9_]*)\s*\(\s*"([a-z][a-z0-9_]*)"/g;
  const field = /\bcode\s*:\s*"([a-z][a-z0-9_]*)"/g;
  const codes = new Map<string, string>();
  for (const file of rustFiles(join(__dirname, '..', '..', 'crates'))) {
    const source = readFileSync(file, 'utf8').split('#[cfg(test)]')[0];
    for (const match of source.matchAll(call)) {
      const [, fn, code] = match;
      if (fn !== 'issues' && /(issue|gap|warning|warn|finding)/i.test(fn)) codes.set(code, file);
    }
    for (const match of source.matchAll(field)) codes.set(match[1], file);
  }
  return codes;
}

const labelled = (table: Record<string, string>, code: string) =>
  KERNEL_CODE_PREFIXES.some((prefix) => typeof table[`${prefix}${code}`] === 'string');

describe('kernel code labels', () => {
  const codes = emittedCodes();

  it('finds the kernel codes', () => {
    expect(codes.size).toBeGreaterThan(700);
  });

  it('every emitted code has a Dutch and an English label', () => {
    const missingNl = [...codes].filter(([code]) => !labelled(nl, code)).map(([code, file]) => `${code} (${file})`);
    const missingEn = [...codes].filter(([code]) => !labelled(en, code)).map(([code, file]) => `${code} (${file})`);
    expect(missingNl).toEqual([]);
    expect(missingEn).toEqual([]);
  });
});
