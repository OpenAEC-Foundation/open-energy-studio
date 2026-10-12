import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import catalogue from '../../training-data/nta8800-input-options.json';
import {
  FORM_BUILDERS, FORM_COVERAGE_EXEMPTIONS, FORM_SOURCES, type CoverageRoute,
} from '../core/nta/formCoverage';

interface CatalogueEntry { route: CoverageRoute; kind: 'variant' | 'field'; path: string; options: string[] }

function sourceFiles(path: string): string[] {
  if (!statSync(path).isDirectory()) return [path];
  return readdirSync(path).flatMap((name) => {
    const child = join(path, name);
    if (statSync(child).isDirectory()) return sourceFiles(child);
    return /\.tsx?$/.test(name) && !name.includes('.test.') ? [child] : [];
  });
}

/** Block and line comments out, so a name in a comment never counts. */
function stripComments(text: string): string {
  return text
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/(^|[^:'"`\\])\/\/.*$/gm, '$1');
}

interface FormFile { file: string; text: string }

function routeFiles(route: CoverageRoute): FormFile[] {
  return FORM_SOURCES[route].flatMap((source) => sourceFiles(source))
    .map((file) => ({ file, text: stripComments(readFileSync(file, 'utf8')) }));
}

const files: Record<CoverageRoute, FormFile[]> = {
  project: routeFiles('project'),
  residentialSurvey: routeFiles('residentialSurvey'),
  utilitySurvey: routeFiles('utilitySurvey'),
};

function escape(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * The name at a write site on one line: an option value, a path element or a
 * template key. Lines that read (`read(`), compare (`=== 'x'`) or test
 * membership (`'x' in`) do not count.
 */
function writesName(line: string, option: string): boolean {
  if (/\bread\(/.test(line)) return false;
  const name = escape(option);
  for (const match of line.matchAll(new RegExp(`['"\`]${name}['"\`]`, 'g'))) {
    const before = line.slice(0, match.index).trimEnd();
    const after = line.slice((match.index ?? 0) + match[0].length).trimStart();
    if (/(===|!==|==|!=|\bcase)$/.test(before) || /^(===|!==|==|!=|in\b)/.test(after)) continue;
    return true;
  }
  return new RegExp(`(^|[{,\\s])${name}\\s*\\??:`).test(line);
}

function namesKey(text: string, key: string): boolean {
  const name = escape(key);
  return new RegExp(`['"\`]${name}['"\`]`).test(text) || new RegExp(`\\b${name}\\s*\\??:`).test(text)
    || new RegExp(`\\.${name}\\b`).test(text);
}

/** The catalogue path's last named segment: the object or field the option belongs to. */
function parentKey(path: string): string | null {
  const segments = path.split('/').filter((segment) => segment !== '' && segment !== '[]');
  const last = segments[segments.length - 1];
  return last == null || last === 'ntaCalculation' ? null : last;
}

/** Lines around a write site in which a field's parent key must be named. */
const PARENT_WINDOW = 12;

/**
 * A variant counts when its value is written in a file that names the field
 * it belongs to (option lists often sit in a constant at the top of the
 * file). A field counts only when its parent key is named near the write
 * site, so `glass` of a window does not cover `glass` of a door in the same
 * file; components that write under a `base` path passed in by their caller
 * are exempt from that, since the caller names the parent.
 */
function covered(route: CoverageRoute, path: string, option: string, kind: 'variant' | 'field'): boolean {
  const parent = parentKey(path);
  return files[route].some(({ file, text }) => {
    const lines = text.split('\n');
    const builds = parent != null && builders(route).some((builder) => file.endsWith(builder.file) && builder.parents.includes(parent));
    if (kind === 'variant' || parent == null || builds) {
      return (parent == null || builds || namesKey(text, parent)) && lines.some((line) => writesName(line, option));
    }
    return lines.some((line, index) => writesName(line, option)
      && (pathVariableHasParent(lines, index, parent)
        || namesKey(lines.slice(Math.max(0, index - PARENT_WINDOW), index + PARENT_WINDOW + 1).join('\n'), parent)));
  });
}

/**
 * The line writes under a path variable (`[...base, 'x']`, `[...path, 'x']`
 * or `at('x')`, which spreads `base`) whose parent is accounted for: either
 * the function receives it as a prop, so its caller names the parent, or the
 * function builds it itself and names the parent where it does (e.g. `const
 * base = ['envelope', 'doors', index]` names `doors`).
 */
function pathVariableHasParent(lines: string[], index: number, parent: string): boolean {
  const variables = [...lines[index].matchAll(/\[\.\.\.(\w+)/g)].map((match) => match[1]);
  if (/\bat\(/.test(lines[index])) variables.push('base');
  return variables.some((variable) => {
    const defined = new RegExp(`\\b(const|let) ${variable}\\b`);
    for (let row = index; row >= 0; row -= 1) {
      if (defined.test(lines[row])) return namesKey(lines.slice(row, row + 3).join('\n'), parent);
      if (/^(export )?function \w+|^const \w+ = \(/.test(lines[row])) return new RegExp(`\\b${variable}\\b`).test(lines[row]);
    }
    return false;
  });
}

/** Files that build a whole object of the kernel input in one literal, with the keys of the objects they build. */
const builders = (route: CoverageRoute) => FORM_BUILDERS.filter((builder) => builder.route === route);

const entries = catalogue as CatalogueEntry[];
const exempt = (route: string, path: string, option: string) => FORM_COVERAGE_EXEMPTIONS.find((item) =>
  item.route === route && item.path === path && item.options.includes(option));

describe('form coverage of the kernel input (docs/ui-redesign/formulierdekking.md)', () => {
  it('has a catalogue of the kernel input options', () => {
    const options = entries.reduce((sum, entry) => sum + entry.options.length, 0);
    expect(options).toBeGreaterThan(1500);
    expect(new Set(entries.map((entry) => entry.route))).toEqual(new Set(['project', 'residentialSurvey', 'utilitySurvey']));
  });

  it('counts only writes, not reads, comparisons or comments', () => {
    expect(writesName("const pipe = read(draft, ['distributionSystem', 'bufferVessel']);", 'bufferVessel')).toBe(false);
    expect(writesName("{kind === 'booster_heat_pump' && <Fields />}", 'booster_heat_pump')).toBe(false);
    expect(writesName("<NumberField path={at('bufferVessel', 'volumeL')} />", 'volumeL')).toBe(true);
    expect(writesName("options={[['collective', t('x')]]}", 'collective')).toBe(true);
    expect(writesName("return { volumeL: null, label: null };", 'volumeL')).toBe(true);
    expect(stripComments("// the 'glass' of a door\nconst a = 1; /* 'panels' */")).not.toMatch(/glass|panels/);
  });

  it('offers every option in a form of its own route or lists it with a reason', () => {
    const missing = entries.flatMap((entry) => entry.options
      .filter((option) => !covered(entry.route, entry.path, option, entry.kind) && !exempt(entry.route, entry.path, option))
      .map((option) => `${entry.route} ${entry.kind} ${entry.path}: ${option}`));
    expect(missing, 'add a form field, or an entry in FORM_COVERAGE_EXEMPTIONS with the reason').toEqual([]);
  });

  it('keeps no stale exemptions', () => {
    const stale = FORM_COVERAGE_EXEMPTIONS.flatMap((item) => item.options
      .filter((option) => {
        const entry = entries.find((candidate) => candidate.route === item.route && candidate.path === item.path);
        return entry == null || !entry.options.includes(option) || covered(item.route, item.path, option, entry.kind);
      })
      .map((option) => `${item.route} ${item.path}: ${option}`));
    expect(stale).toEqual([]);
    for (const item of FORM_COVERAGE_EXEMPTIONS) expect(item.reason.length).toBeGreaterThan(20);
  });
});
