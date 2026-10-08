#!/usr/bin/env node
// Kernel-version discipline for the NTA 8800 kernel (BRL 9501 §4.3, §5.2, §8.2).
//
//   node scripts/nta-kernel-version.mjs check
//       Fails when docs/nta8800-releasenotes.md and KERNEL_VERSION
//       (crates/nta8800-core/Cargo.toml) disagree; see `checkReleaseNotes`.
//   node scripts/nta-kernel-version.mjs release <yyyy-mm-dd>
//       Moves the unreleased entries under a new version section for the
//       current KERNEL_VERSION (used by scripts/release-nta.sh).
//   node scripts/nta-kernel-version.mjs version
//       Prints KERNEL_VERSION.
//
// Release-notes convention (also at the top of the file):
// - a released version is `## Rekenkern X.Y.Z — <datum>` followed by the line
//   `<!-- kernel-version: X.Y.Z -->`; versions descend through the file;
// - every dated entry (`## ` or `### <d> <maand> <jjjj> — titel`) above the
//   first version section is unreleased;
// - an entry changes results unless its title says "geen rekenwijziging".
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const MONTHS = 'januari|februari|maart|april|mei|juni|juli|augustus|september|oktober|november|december';
const DATED = new RegExp(`^(#{2,3}) (\\d{1,2} (?:${MONTHS}) \\d{4}) — (.+)$`);
const SECTION = /^## Rekenkern (\d+\.\d+\.\d+) — (.+)$/;
const MARKER = /^<!-- kernel-version: (\d+\.\d+\.\d+) -->$/;
const NO_RESULT_CHANGE = /geen rekenwijziging/i;

export function parseVersion(text) {
  const match = /^(\d+)\.(\d+)\.(\d+)$/.exec(text);
  if (!match) throw new Error(`Geen versienummer MAJOR.MINOR.PATCH: ${text}`);
  return match.slice(1).map(Number);
}

export function compareVersions(a, b) {
  const [x, y] = [parseVersion(a), parseVersion(b)];
  for (let i = 0; i < 3; i += 1) if (x[i] !== y[i]) return x[i] - y[i];
  return 0;
}

/** Splits the release notes into unreleased entries and version sections. */
export function parseReleaseNotes(text) {
  const lines = text.split('\n');
  const sections = [];
  const unreleased = [];
  const problems = [];
  for (let i = 0; i < lines.length; i += 1) {
    const section = SECTION.exec(lines[i]);
    if (section) {
      const marker = MARKER.exec(lines[i + 1] ?? '');
      if (!marker) {
        problems.push(`Regel ${i + 2}: na "${lines[i]}" hoort <!-- kernel-version: ${section[1]} -->.`);
      } else if (marker[1] !== section[1]) {
        problems.push(`Regel ${i + 2}: de markering ${marker[1]} verschilt van de kop ${section[1]}.`);
      }
      sections.push({ version: section[1], date: section[2], line: i + 1 });
      continue;
    }
    if (MARKER.test(lines[i]) && !SECTION.test(lines[i - 1] ?? '')) {
      problems.push(`Regel ${i + 1}: een versiemarkering zonder versiekop.`);
    }
    const dated = DATED.exec(lines[i]);
    if (dated && sections.length === 0) {
      unreleased.push({
        line: i + 1,
        date: dated[2],
        title: dated[3],
        changesResults: !NO_RESULT_CHANGE.test(dated[3]),
      });
    }
  }
  for (let i = 1; i < sections.length; i += 1) {
    if (compareVersions(sections[i - 1].version, sections[i].version) <= 0) {
      problems.push(`Versie ${sections[i - 1].version} staat boven ${sections[i].version}: de versies moeten aflopen.`);
    }
  }
  return { sections, unreleased, problems };
}

/**
 * The rules, against the kernel version in Cargo.toml:
 * 1. the release notes have at least one version section, descending;
 * 2. KERNEL_VERSION is never below the latest released version;
 * 3. unreleased entries that change results need KERNEL_VERSION above the
 *    latest release by at least a MINOR step (policy: MINOR per change in
 *    results, MAJOR for a new norm version);
 * 4. a raised KERNEL_VERSION needs at least one unreleased entry.
 * Returns the list of violations (empty when consistent).
 */
export function checkReleaseNotes(text, kernelVersion) {
  const { sections, unreleased, problems } = parseReleaseNotes(text);
  const errors = [...problems];
  if (sections.length === 0) {
    errors.push('De releasenotes hebben geen versiesectie (## Rekenkern X.Y.Z — datum).');
    return errors;
  }
  const latest = sections[0].version;
  const order = compareVersions(kernelVersion, latest);
  const changing = unreleased.filter((entry) => entry.changesResults);
  if (order < 0) {
    errors.push(`KERNEL_VERSION ${kernelVersion} is lager dan de laatste uitgave ${latest}.`);
  } else if (order === 0 && changing.length > 0) {
    errors.push(
      `Onuitgebrachte wijzigingen veranderen uitkomsten, maar KERNEL_VERSION is nog ${latest}. ` +
        `Verhoog de MINOR-versie in crates/nta8800-core/Cargo.toml (en de Cargo.lock-bestanden). ` +
        `Betreft: ${changing.map((entry) => `"${entry.date} — ${entry.title}" (regel ${entry.line})`).join('; ')}.`,
    );
  } else if (order > 0) {
    const [major, minor] = parseVersion(kernelVersion);
    const [latestMajor, latestMinor] = parseVersion(latest);
    const minorStep = major > latestMajor || minor > latestMinor;
    if (changing.length > 0 && !minorStep) {
      errors.push(
        `Onuitgebrachte wijzigingen veranderen uitkomsten; ${kernelVersion} is alleen een PATCH-stap na ${latest}. ` +
          'Het versiebeleid vraagt minstens een MINOR-stap.',
      );
    }
    if (unreleased.length === 0) {
      errors.push(`KERNEL_VERSION ${kernelVersion} is verhoogd zonder onuitgebrachte releasenotes.`);
    }
  }
  return errors;
}

/** Moves the unreleased entries under a new version section. */
export function releaseNotes(text, kernelVersion, date) {
  const errors = checkReleaseNotes(text, kernelVersion);
  if (errors.length > 0) throw new Error(errors.join('\n'));
  const { sections, unreleased } = parseReleaseNotes(text);
  if (compareVersions(kernelVersion, sections[0].version) <= 0) {
    throw new Error(`KERNEL_VERSION ${kernelVersion} is al uitgebracht; verhoog de versie eerst.`);
  }
  const lines = text.split('\n');
  const first = unreleased[0].line - 1;
  const end = sections[0].line - 1;
  // Entries become ### under the section; their own ### become ####.
  const moved = lines.slice(first, end).map((line) => {
    if (DATED.test(line) && line.startsWith('## ')) return `#${line}`;
    if (/^### /.test(line) && !DATED.test(line)) return `#${line}`;
    return line;
  });
  const header = [`## Rekenkern ${kernelVersion} — ${date}`, `<!-- kernel-version: ${kernelVersion} -->`, ''];
  return [...lines.slice(0, first), ...header, ...moved, ...lines.slice(end)].join('\n');
}

export function kernelVersion(root) {
  const cargo = readFileSync(join(root, 'crates/nta8800-core/Cargo.toml'), 'utf8');
  const match = /^version = "([^"]+)"/m.exec(cargo);
  if (!match) throw new Error('Geen version in crates/nta8800-core/Cargo.toml');
  return match[1];
}

const DUTCH_MONTHS = MONTHS.split('|');

export function dutchDate(isoDate) {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(isoDate);
  if (!match) throw new Error(`Datum als jjjj-mm-dd verwacht: ${isoDate}`);
  return `${Number(match[3])} ${DUTCH_MONTHS[Number(match[2]) - 1]} ${match[1]}`;
}

function main(argv) {
  const root = join(dirname(fileURLToPath(import.meta.url)), '..');
  const notesPath = join(root, 'docs/nta8800-releasenotes.md');
  const [command, argument] = argv;
  const version = kernelVersion(root);
  if (command === 'version') {
    console.log(version);
    return 0;
  }
  const text = readFileSync(notesPath, 'utf8');
  if (command === 'check') {
    const errors = checkReleaseNotes(text, version);
    if (errors.length > 0) {
      console.error(`Releasenotes en KERNEL_VERSION ${version} passen niet bij elkaar:`);
      for (const error of errors) console.error(`- ${error}`);
      return 1;
    }
    const { unreleased } = parseReleaseNotes(text);
    console.log(`KERNEL_VERSION ${version} past bij de releasenotes (${unreleased.length} onuitgebrachte items).`);
    return 0;
  }
  if (command === 'release') {
    writeFileSync(notesPath, releaseNotes(text, version, dutchDate(argument ?? '')));
    console.log(`Releasenotes: onuitgebrachte items onder Rekenkern ${version}.`);
    return 0;
  }
  console.error('Gebruik: nta-kernel-version.mjs check | release <jjjj-mm-dd> | version');
  return 2;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  process.exit(main(process.argv.slice(2)));
}
