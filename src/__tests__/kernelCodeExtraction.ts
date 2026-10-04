import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';

/** Every `.rs` file under the crates, without build output. */
export function rustFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    if (name === 'target') return [];
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return rustFiles(path);
    return name.endsWith('.rs') ? [path] : [];
  });
}

/** Callees whose string arguments are user-facing codes. */
const CODE_CALLEE = /(issue|gap|warning|warn|finding|push|add|err|refuse|reject|fail)/i;
const SNAKE = /^[a-z][a-z0-9]*(?:_[a-z0-9]+)+$/;
/** A JSON-pointer-like or camelCase path literal, as passed next to a code. */
const PATH_LITERAL = /^(?:[a-z][A-Za-z0-9]*[A-Z.\[][A-Za-z0-9.\[\]]*|)$/;

/** Source text without test modules and line comments. */
function productionSource(file: string): string {
  return readFileSync(file, 'utf8').split('#[cfg(test)]')[0].replace(/\/\/[^\n]*/g, '');
}

/**
 * Snake_case literals that are direct arguments of an issue-like call
 * (`issue(...)`, `push(...)`, `add(...)`, `Err(...)`, …), or members of a tuple
 * pushed by such a call (`push(("code", "path"))`). Recorder calls of the surveys
 * (`record("rule", …)`) carry internal rule ids, not user-facing codes.
 */
function callLiterals(source: string): Set<string> {
  const out = new Set<string>();
  const callee = /\b([A-Za-z_][A-Za-z0-9_]*)\s*(?:::<[^>]*>)?\(/g;
  for (const match of source.matchAll(callee)) {
    const name = match[1];
    if (name === 'issues' || !CODE_CALLEE.test(name)) continue;
    const start = (match.index ?? 0) + match[0].length;
    const tupleArgument = source[start] === '(';
    let depth = 1;
    let i = start;
    while (i < source.length && depth > 0) {
      const ch = source[i];
      if (ch === '"') {
        let j = i + 1;
        while (j < source.length && source[j] !== '"') j += source[j] === '\\' ? 2 : 1;
        const literal = source.slice(i + 1, j);
        if (SNAKE.test(literal) && (depth === 1 || (tupleArgument && depth === 2))) out.add(literal);
        i = j + 1;
        continue;
      }
      if (ch === '(') depth += 1;
      else if (ch === ')') depth -= 1;
      i += 1;
    }
  }
  return out;
}

/**
 * User-facing code literals the kernel, the surveys and the service emit, outside
 * test modules: issue-like call arguments, `code: "…"` fields, `unwrap_or("…")`
 * fallbacks and `("code", "path")` table rows.
 */
export function emittedCodes(root: string): Map<string, string> {
  const codes = new Map<string, string>();
  for (const file of rustFiles(root)) {
    const source = productionSource(file);
    const found = callLiterals(source);
    for (const match of source.matchAll(/\bcode\s*:\s*"([a-z][a-z0-9_]*)"/g)) found.add(match[1]);
    for (const match of source.matchAll(/unwrap_or\(\s*"([a-z][a-z0-9_]*)"\s*\)/g)) found.add(match[1]);
    // `("code", "path")` rows; a second literal followed by `:` is a JSON object key, not a path.
    for (const match of source.matchAll(/"([a-z][a-z0-9]*(?:_[a-z0-9]+)+)",\s*"([^"]*)"(?!\s*:)/g)) {
      const before = source.slice(Math.max(0, (match.index ?? 0) - 40), match.index);
      if (/\brecord\s*\(\s*$/.test(before)) continue;
      if (PATH_LITERAL.test(match[2])) found.add(match[1]);
    }
    for (const code of found) if (!codes.has(code)) codes.set(code, file);
  }
  return codes;
}
