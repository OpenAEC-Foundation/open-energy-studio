import { describe, expect, it } from 'vitest';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative, resolve } from 'node:path';
import ts from 'typescript';
import { en } from '../i18n/en';
import { nl } from '../i18n/nl';

// Placeholders in the labels match how each label is filled. i18next's t(key, vars)
// fills {{name}}; a label read with t(key) and filled with .replace('{name}', …)
// uses single braces. The Handleiding window showed "{manual}" and "{path}", and
// the relabel migration notice "{count}" and "{fields}": single braces given to t().

const DOUBLE = /\{\{\s*([\w.]+)[^}]*\}\}/g;
const SINGLE = /(?<!\{)\{([A-Za-z_][\w.]*)\}(?!\})/g;
const names = (label: string, pattern: RegExp) => [...new Set([...label.matchAll(pattern)].map((match) => match[1]))].sort();

/** i18next options that are no interpolation variables. `count` may select a plural without appearing. */
const OPTIONS = new Set(['count', 'defaultValue', 'context', 'ns', 'lng', 'lngs', 'returnObjects', 'interpolation', 'ordinal', 'joinArrays', 'postProcess', 'fallbackLng']);

const root = resolve(__dirname, '../..');
const LOCALES = ['fr', 'es', 'zh', 'it', 'de', 'pt', 'pl', 'tr', 'ja', 'ko', 'ar', 'fa'] as const;

function sourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((entry) => {
    const path = join(dir, entry);
    if (statSync(path).isDirectory()) return entry === '__tests__' ? [] : sourceFiles(path);
    return /\.tsx?$/.test(entry) && !entry.endsWith('.d.ts') ? [path] : [];
  });
}

function stringValue(node: ts.Node | undefined): string | null {
  return node && (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) ? node.text : null;
}

function isTranslateCall(node: ts.CallExpression): boolean {
  const callee = node.expression;
  return (ts.isIdentifier(callee) && callee.text === 't')
    || (ts.isPropertyAccessExpression(callee) && callee.name.text === 't');
}

/** The '{name}' strings of .replace(…) calls chained directly on this call. */
function chainedReplacements(call: ts.CallExpression): string[] {
  const out: string[] = [];
  let node: ts.Node = call;
  while (ts.isPropertyAccessExpression(node.parent) && node.parent.name.text === 'replace'
    && ts.isCallExpression(node.parent.parent)) {
    const value = stringValue(node.parent.parent.arguments[0]);
    if (value) out.push(value);
    node = node.parent.parent;
  }
  return out;
}

interface CallSite { where: string; key: string; vars: string[] | null; replaced: string[] }

function translateCalls(): CallSite[] {
  const calls: CallSite[] = [];
  for (const file of sourceFiles(join(root, 'src'))) {
    const text = readFileSync(file, 'utf8');
    if (!/\bt\(/.test(text)) continue;
    const source = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true, file.endsWith('x') ? ts.ScriptKind.TSX : ts.ScriptKind.TS);
    const visit = (node: ts.Node) => {
      if (ts.isCallExpression(node) && isTranslateCall(node)) {
        const key = stringValue(node.arguments[0]);
        if (key) {
          const second = node.arguments[1];
          let vars: string[] | null = [];
          if (second) {
            if (ts.isObjectLiteralExpression(second) && !second.properties.some(ts.isSpreadAssignment)) {
              vars = second.properties.flatMap((property) => {
                if (ts.isShorthandPropertyAssignment(property)) return [property.name.text];
                if ((ts.isPropertyAssignment(property) || ts.isMethodDeclaration(property)) && property.name
                  && (ts.isIdentifier(property.name) || ts.isStringLiteral(property.name))) return [property.name.text];
                return [];
              });
            } else {
              vars = null; // built elsewhere: its variables cannot be read here
            }
          }
          const { line } = source.getLineAndCharacterOfPosition(node.getStart());
          calls.push({ where: `${relative(root, file)}:${line + 1}`, key, vars, replaced: chainedReplacements(node) });
        }
      }
      ts.forEachChild(node, visit);
    };
    visit(source);
  }
  return calls;
}

describe('i18n placeholders', () => {
  it('Dutch and English share the placeholders of every key', () => {
    const problems: string[] = [];
    const nlLabels = nl as Record<string, string>;
    for (const [key, label] of Object.entries(en)) {
      if (!(key in nlLabels)) continue;
      for (const pattern of [DOUBLE, SINGLE]) {
        const a = names(label, pattern).join(','); const b = names(nlLabels[key], pattern).join(',');
        if (a !== b) problems.push(`${key}: en {${a}} nl {${b}}`);
      }
    }
    expect(problems).toEqual([]);
  });

  it('the other languages keep the English placeholders', async () => {
    const problems: string[] = [];
    for (const locale of LOCALES) {
      const labels = (await import(`../i18n/${locale}.ts`))[locale] as Record<string, string>;
      for (const [key, label] of Object.entries(labels)) {
        if (!(key in en)) continue;
        for (const pattern of [DOUBLE, SINGLE]) {
          const a = names(en[key], pattern).join(','); const b = names(label, pattern).join(',');
          if (a !== b) problems.push(`${locale} ${key}: en {${a}} ${locale} {${b}}`);
        }
      }
    }
    expect(problems).toEqual([]);
  });

  it('every t() call fills exactly the placeholders of its label', () => {
    const calls = translateCalls();
    expect(calls.length).toBeGreaterThan(1000);
    const problems: string[] = [];
    const nlLabels = nl as Record<string, string>;
    for (const call of calls) {
      const label = en[call.key];
      if (label === undefined) continue;
      const labels = [label, nlLabels[call.key] ?? label];
      const singles = [...new Set(labels.flatMap((value) => names(value, SINGLE)))];
      const doubles = [...new Set(labels.flatMap((value) => names(value, DOUBLE)))];
      // Single braces are only filled by a .replace('{name}', …) on the same call.
      for (const name of singles) {
        if (!call.replaced.includes(`{${name}}`)) problems.push(`${call.where} ${call.key}: {${name}} is not replaced${call.vars?.includes(name) ? ' (t() needs {{' + name + '}})' : ''}`);
      }
      for (const replaced of call.replaced) {
        if (/^\{\w+\}$/.test(replaced) && !singles.includes(replaced.slice(1, -1))) problems.push(`${call.where} ${call.key}: replaces ${replaced}, which the label does not hold`);
      }
      if (call.vars === null) continue;
      for (const name of doubles) {
        if (!call.vars.includes(name)) problems.push(`${call.where} ${call.key}: {{${name}}} is not passed`);
      }
      for (const name of call.vars) {
        if (!OPTIONS.has(name) && !doubles.includes(name)) problems.push(`${call.where} ${call.key}: passes ${name}, which the label does not use`);
      }
    }
    expect(problems).toEqual([]);
  });
});
