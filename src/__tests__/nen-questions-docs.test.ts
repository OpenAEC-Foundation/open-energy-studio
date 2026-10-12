/**
 * The interpretation questions to NEN (docs/nta8800-vragen-nen.md) and the letter that
 * carries them (docs/nta8800-vragen-nen-brief.md) stay in step: the same questions,
 * numbered 1..n without gaps, with the same titles, and both state the count in words.
 */
import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(__dirname, '../..');
const read = (path: string) => readFileSync(resolve(root, path), 'utf8');

const WORDS = ['', 'één', 'twee', 'drie', 'vier', 'vijf', 'zes', 'zeven', 'acht', 'negen', 'tien',
  'elf', 'twaalf', 'dertien', 'veertien', 'vijftien', 'zestien', 'zeventien', 'achttien', 'negentien', 'twintig'];

const questions = (text: string, heading: RegExp) =>
  text.split('\n').flatMap((line) => {
    const match = heading.exec(line);
    return match ? [{ number: Number(match[1]), title: match[2].trim() }] : [];
  });

describe('NEN interpretation questions', () => {
  const list = questions(read('docs/nta8800-vragen-nen.md'), /^## (\d+)\. (.+)$/);
  const letter = questions(read('docs/nta8800-vragen-nen-brief.md'), /^### Vraag (\d+)\. (.+)$/);

  it('numbers the questions 1..n in both documents', () => {
    expect(list.length).toBeGreaterThan(0);
    expect(list.map((q) => q.number)).toEqual(list.map((_, i) => i + 1));
    expect(letter.map((q) => q.number)).toEqual(list.map((q) => q.number));
  });

  it('cites the same place in the norm for each question', () => {
    // The letter may word the title a little differently; the citation in brackets must match.
    const citation = (title: string) => /\(([^()]*(?:\([^()]*\)[^()]*)*)\)\s*$/.exec(title)?.[1];
    for (const [i, q] of list.entries()) {
      expect(citation(letter[i].title), `vraag ${q.number}`).toBe(citation(q.title));
    }
  });

  it('states the number of questions in words in both documents', () => {
    const word = WORDS[list.length];
    expect(read('docs/nta8800-vragen-nen.md')).toContain(`Alle ${word} vragen`);
    expect(read('docs/nta8800-vragen-nen-brief.md')).toContain(`${word} punten`);
  });
});
