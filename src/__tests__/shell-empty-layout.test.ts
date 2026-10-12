import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

// The welcome screen (no open project) uses one grid column. In windows narrower
// than 1360 px a media rule set `.app-shell` back to the three work-area columns
// and, at the same specificity and later in the file, won: the welcome screen was
// squeezed into the 264 px navigation column (seen in the manual screenshot).
// jsdom applies no media queries, so the rule is checked in the stylesheet itself.

const css = readFileSync(resolve(__dirname, '../components/shell/shell.css'), 'utf8');

/** Selectors of the rules in `text` that set grid-template-columns. */
function columnRules(text: string): string[] {
  return [...text.matchAll(/([^{}]+)\{[^{}]*grid-template-columns[^{}]*\}/g)].map((match) => match[1].trim().split('\n').pop()!.trim());
}

describe('welcome screen layout', () => {
  it('keeps one column for the empty shell in every window width', () => {
    const empty = columnRules(css).filter((selector) => selector.includes('app-shell--empty'));
    expect(empty.length).toBeGreaterThan(0);
    // Every rule that sets .app-shell columns, also inside media queries, must lose to the empty rule.
    const shellRules = columnRules(css).filter((selector) => /(^|,\s*)\.app-shell(\s|$|,)/.test(selector));
    expect(shellRules.length).toBeGreaterThan(1);
    for (const selector of empty) expect(selector).toMatch(/\.app-shell\.app-shell--empty/);
  });
});
