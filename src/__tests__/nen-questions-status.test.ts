/**
 * Issue #8: the public list of interpretation questions to NEN carries, per
 * question, its status at NEN and the reading the kernel follows. Every
 * question in docs/nta8800-vragen-nen.md has a row in "Stand per uitgave",
 * and the table names the current KERNEL_VERSION.
 */
import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(__dirname, '../..');
const read = (path: string) => readFileSync(resolve(root, path), 'utf8').replace(/\r\n/g, '\n');

describe('status of the NEN questions per release', () => {
  const text = read('docs/nta8800-vragen-nen.md');
  const questions = [...text.matchAll(/^## (\d+)\. /gm)].map((match) => match[1]);
  const status = text.split('\n## Stand per uitgave\n')[1]?.split('\n## ')[0] ?? '';
  const rows = [...status.matchAll(/^\| (\d+) \| [^|]+ \| ([^|]+) \| (.+) \|$/gm)];

  it('has a row with a status and a reading for every question', () => {
    expect(questions.length).toBeGreaterThan(0);
    expect(rows.map((row) => row[1])).toEqual(questions);
    for (const row of rows) {
      expect(row[2].trim(), `vraag ${row[1]}`).not.toBe('');
      expect(row[3].trim(), `vraag ${row[1]}`).not.toBe('');
    }
  });

  it('is brought up to date for the current kernel version', () => {
    const kernel = /^version = "([^"]+)"/m.exec(read('crates/nta8800-core/Cargo.toml'))?.[1];
    expect(status).toContain(`Stand bij rekenkern ${kernel} `);
  });
});
