import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import {
  checkReleasedSections,
  checkReleaseNotes,
  dutchDate,
  kernelVersion,
  parseReleaseNotes,
  releaseNotes,
} from '../../scripts/nta-kernel-version.mjs';

const root = join(__dirname, '../..');

const notes = (unreleased: string) => `# NTA 8800-kern — releasenotes

## Onuitgebracht
${unreleased}
## Rekenkern 0.2.0 — 8 oktober 2026
<!-- kernel-version: 0.2.0 -->

### 8 oktober 2026 — eerdere wijziging

#### Resultaten die veranderen

- iets

## Rekenkern 0.1.0 — 30 september 2026
<!-- kernel-version: 0.1.0 -->
`;

describe('NTA kernel version discipline (BRL 9501 §4.3, §5.2)', () => {
  it('accepts the committed release notes with the committed KERNEL_VERSION', () => {
    const text = readFileSync(join(root, 'docs/nta8800-releasenotes.md'), 'utf8');
    expect(checkReleaseNotes(text, kernelVersion(root))).toEqual([]);
  });

  it('requires a MINOR step for an unreleased change in results', () => {
    const changing = notes('\n### 9 oktober 2026 — nieuwe route\n\n- tekst\n');
    expect(checkReleaseNotes(changing, '0.2.0')[0]).toMatch(/Verhoog de MINOR-versie/);
    expect(checkReleaseNotes(changing, '0.2.1')[0]).toMatch(/PATCH-stap/);
    expect(checkReleaseNotes(changing, '0.3.0')).toEqual([]);
    expect(checkReleaseNotes(changing, '1.0.0')).toEqual([]);
  });

  it('counts an old-style ## entry above the first version as unreleased', () => {
    const oldStyle = notes('\n## 9 oktober 2026 — nieuwe route\n\n- tekst\n');
    expect(parseReleaseNotes(oldStyle).unreleased).toHaveLength(1);
    expect(checkReleaseNotes(oldStyle, '0.2.0')).toHaveLength(1);
  });

  it('lets entries without a change in results pass without a bump', () => {
    const text = notes('\n### 9 oktober 2026 — knoppen (geen rekenwijziging)\n');
    expect(checkReleaseNotes(text, '0.2.0')).toEqual([]);
    // A raised version still needs notes, and never goes down.
    expect(checkReleaseNotes(notes(''), '0.3.0')[0]).toMatch(/zonder onuitgebrachte releasenotes/);
    expect(checkReleaseNotes(notes(''), '0.1.9')[0]).toMatch(/lager dan de laatste uitgave/);
  });

  it('rejects a section without its marker and versions that do not descend', () => {
    const missing = notes('').replace('<!-- kernel-version: 0.2.0 -->\n', '');
    expect(checkReleaseNotes(missing, '0.2.0').join()).toMatch(/hoort <!-- kernel-version: 0.2.0 -->/);
    const swapped = notes('').replace('Rekenkern 0.1.0', 'Rekenkern 0.3.0').replace(
      'kernel-version: 0.1.0',
      'kernel-version: 0.3.0',
    );
    expect(checkReleaseNotes(swapped, '0.3.0').join()).toMatch(/moeten aflopen/);
  });

  it('moves unreleased entries under a new version section on release', () => {
    const text = notes('\n## 9 oktober 2026 — nieuwe route\n\n### Resultaten die veranderen\n\n- tekst\n');
    const released = releaseNotes(text, '0.3.0', dutchDate('2026-10-09'));
    expect(released).toContain(
      '## Rekenkern 0.3.0 — 9 oktober 2026\n<!-- kernel-version: 0.3.0 -->\n\n### 9 oktober 2026 — nieuwe route',
    );
    expect(released).toContain('#### Resultaten die veranderen\n\n- tekst');
    const parsed = parseReleaseNotes(released);
    expect(parsed.unreleased).toEqual([]);
    expect(parsed.sections.map((section) => section.version)).toEqual(['0.3.0', '0.2.0', '0.1.0']);
    expect(checkReleaseNotes(released, '0.3.0')).toEqual([]);
    expect(() => releaseNotes(notes(''), '0.2.0', '9 oktober 2026')).toThrow();
  });

  // Review 9 October 2026: headings that are almost, but not exactly, an
  // entry used to be ignored, so a change in results passed without a bump.
  it.each([
    ['en dash', '### 10 oktober 2026 – fix'],
    ['hyphen', '### 10 oktober 2026 - fix'],
    ['capital month', '### 10 Oktober 2026 — fix'],
    ['#### entry', '#### 10 oktober 2026 — fix'],
    ['abbreviated month', '### 10 okt 2026 — fix'],
    ['ISO date', '### 2026-10-10 — fix'],
    ['undated heading', '### Nieuwe route'],
  ])('refuses a malformed entry heading (%s) instead of ignoring it', (_name, heading) => {
    const errors = checkReleaseNotes(notes(`\n${heading}\n\n- tekst\n`), '0.2.0');
    expect(errors.join('\n')).toMatch(/geen geldige itemkop/);
  });

  it('refuses a date-like subheading hidden under an entry', () => {
    const text = notes('\n### 9 oktober 2026 — knoppen (geen rekenwijziging)\n\n#### 10 okt 2026 — fix\n');
    expect(checkReleaseNotes(text, '0.2.0').join()).toMatch(/lijkt een item met een datum/);
    const sub = notes('\n### 9 oktober 2026 — route\n\n#### Resultaten die veranderen\n\n- tekst\n');
    expect(checkReleaseNotes(sub, '0.3.0')).toEqual([]);
  });

  it('reads CRLF release notes like LF', () => {
    const crlf = notes('\n### 10 oktober 2026 — fix\n\n- tekst\n').replace(/\n/g, '\r\n');
    expect(parseReleaseNotes(crlf).unreleased).toHaveLength(1);
    expect(checkReleaseNotes(crlf, '0.2.0')[0]).toMatch(/Verhoog de MINOR-versie/);
    expect(checkReleaseNotes(crlf, '0.3.0')).toEqual([]);
  });

  it('only accepts the exact suffix "(geen rekenwijziging)"', () => {
    for (const title of ['knoppen Geen Rekenwijziging', 'geen rekenwijziging? nee, wel', 'knoppen (Geen rekenwijziging)']) {
      const text = notes(`\n### 9 oktober 2026 — ${title}\n`);
      const errors = checkReleaseNotes(text, '0.2.0').join('\n');
      expect(errors).toMatch(/exacte slot/);
      expect(errors).toMatch(/Verhoog de MINOR-versie/);
    }
    expect(checkReleaseNotes(notes('\n### 9 oktober 2026 — knoppen (geen rekenwijziging)\n'), '0.2.0')).toEqual([]);
  });

  it('keeps the structural headings and free text under "Afspraken"', () => {
    const text = notes('').replace(
      '## Onuitgebracht',
      '## Afspraken voor dit bestand\n\n### Voorbeeld\n\n- tekst\n\n## Onuitgebracht',
    );
    expect(checkReleaseNotes(text, '0.2.0')).toEqual([]);
  });

  it('detects an entry inserted into an already released section', () => {
    const previous = notes('');
    const next = previous.replace('- iets', '- iets\n- stilletjes toegevoegd');
    expect(checkReleasedSections(previous, next)[0]).toMatch(/uitgebrachte versiesectie is gewijzigd/);
    const released = releaseNotes(notes('\n### 9 oktober 2026 — nieuwe route\n'), '0.3.0', '9 oktober 2026');
    expect(checkReleasedSections(previous, released)).toEqual([]);
    expect(checkReleasedSections(previous, released.replace(/\n/g, '\r\n'))).toEqual([]);
    const dropped = previous.replace('## Rekenkern 0.2.0 — 8 oktober 2026', '## Rekenkern 0.2.0 — 9 oktober 2026');
    expect(checkReleasedSections(previous, dropped)[0]).toMatch(/ontbreekt/);
  });
});
