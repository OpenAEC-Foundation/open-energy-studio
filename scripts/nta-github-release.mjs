#!/usr/bin/env node
// Body of the GitHub release for a release tag (release/<tag>/github-release.md),
// written by scripts/release-nta.sh from the same identity as the
// leveringsdocument, the released notes and the SHA-256 of every asset.
//
//   node scripts/nta-github-release.mjs --out <file.md> --tag <tag> --commit <sha>
//        --notes <released-notes.md> [--repo-url <https://github.com/owner/repo>]
//        [--gate-summary <gate-samenvatting.json>]
//        [asset ...]
import { writeFileSync, readFileSync } from 'node:fs';
import { basename, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { normaliseNewlines } from './nta-kernel-version.mjs';
import { readIdentity, sha256File } from './nta-leveringsdocument.mjs';
import { renderGateSummaryEnglish } from './nta-gate-summary.mjs';

/**
 * The titles of the entries in the released section of `kernelVersion`
 * (`### <date> — <title>`), or of the unreleased entries when the kernel
 * version was not released again (a program-only release).
 */
export function releasedItems(notesText, kernelVersion) {
  const lines = normaliseNewlines(notesText).split('\n');
  const heading = new RegExp(`^## Rekenkern ${kernelVersion.replace(/\./g, '\\.')} — `);
  let start = lines.findIndex((line) => heading.test(line));
  let section = start >= 0 ? `Rekenkern ${kernelVersion}` : 'Onuitgebracht';
  if (start < 0) start = lines.findIndex((line) => line === '## Onuitgebracht');
  if (start < 0) return { section, items: [] };
  const items = [];
  for (let i = start + 1; i < lines.length && !lines[i].startsWith('## '); i += 1) {
    const item = /^### \d{1,2} \S+ \d{4} — (.+)$/.exec(lines[i]);
    if (item) items.push(item[1]);
  }
  return { section, items };
}

/**
 * The commit as shown in the release body: the hash shortened to 12 characters,
 * with any marker release-nta.sh adds (such as "(met lokale wijzigingen)" on a
 * dry run from a dirty worktree) kept after it.
 */
export function shortCommit(commit) {
  const [hash, ...rest] = commit.trim().split(/\s+/);
  const marker = rest.join(' ');
  return marker ? `\`${hash.slice(0, 12)}\` ${marker}` : `\`${hash.slice(0, 12)}\``;
}

/** Fills the release body. Every asset is listed with its SHA-256. */
export function renderGithubRelease(values) {
  const { identity, tag, commit, notesText, assets, repoUrl, gateSummary } = values;
  const link = (path) => (repoUrl ? `${repoUrl}/blob/${tag}/${path}` : path);
  const attest = identity.attestNumber || 'nog niet toegekend';
  const { section, items } = releasedItems(notesText, identity.kernelVersion);
  const lines = [
    `## Open Energy Studio ${identity.programVersion}, NTA 8800 kernel ${identity.kernelVersion}`,
    '',
    '| | Version |',
    '|---|---|',
    `| Program | **${identity.programVersion}** |`,
    `| NTA 8800 kernel (\`KERNEL_VERSION\`) | **${identity.kernelVersion}** |`,
    `| Norm edition (default, registrable) | ${identity.targetNormVersion} |`,
    `| Attest number (BRL 9501) | ${attest} |`,
    `| Source | tag \`${tag}\`, commit ${shortCommit(commit)} |`,
    '',
  ];
  if (!identity.attestNumber) {
    lines.push(
      '> **Status: pre-release.** This software is **not yet attested** under BRL 9501. ' +
        'Do not use it for registered energy labels until an attest has been issued. ' +
        `See [docs/nta8800-brl9501-gereedheid.md](${link('docs/nta8800-brl9501-gereedheid.md')}).`,
      '',
    );
  }
  lines.push(`### What's in this release (release notes, section "${section}")`, '');
  if (items.length === 0) {
    lines.push('No release-note entries.', '');
  } else {
    for (const item of items) lines.push(`- ${item}`);
    lines.push('');
  }
  lines.push(
    `Full change list: [docs/nta8800-releasenotes.md](${link('docs/nta8800-releasenotes.md')}).`,
    '',
    '### Verification',
    (gateSummary && !gateSummary.gate.passed
      ? 'The release gate `scripts/verify-nta.sh` did **not** pass: this text is no evidence for a release. '
      : gateSummary
        ? 'The release gate `scripts/verify-nta.sh` passed, including the kernel on Rust 1.77.2: ' +
          `${renderGateSummaryEnglish(gateSummary)} `
        : 'The release gate `scripts/verify-nta.sh` passed, including the kernel on Rust 1.77.2. ') +
      'The archive holds the gate log, the gate summary (`gate-samenvatting.json`), the reference-suite report and the test report. ' +
      `See the [test protocol](${link('docs/nta8800-testprotocol.md')}).`,
    '',
    '### Downloads',
    '| File | SHA-256 |',
    '|---|---|',
  );
  for (const asset of assets) lines.push(`| \`${asset.name}\` | \`${asset.sha256}\` |`);
  lines.push(
    '',
    'Windows and macOS installers are built by the release workflow from this tag.',
    '',
    `Known limitations and open questions: [docs/nta8800-brl9501-gereedheid.md](${link('docs/nta8800-brl9501-gereedheid.md')}) ` +
      `and [docs/nta8800-vragen-nen.md](${link('docs/nta8800-vragen-nen.md')}). Norm texts are not included, only page citations.`,
    '',
    'Licence: LGPL-3.0-or-later.',
    '',
  );
  return lines.join('\n');
}

/** https://github.com/owner/repo from an https or ssh GitHub remote, else ''. */
export function githubUrl(remote) {
  const match = /github\.com[:/]([^/\s]+)\/([^/\s]+?)(?:\.git)?\/?$/.exec(remote.trim());
  return match ? `https://github.com/${match[1]}/${match[2]}` : '';
}

function main(argv) {
  const root = join(dirname(fileURLToPath(import.meta.url)), '..');
  const options = { assets: [] };
  for (let i = 0; i < argv.length; i += 1) {
    const flag = argv[i];
    if (['--out', '--tag', '--commit', '--notes', '--repo-url', '--gate-summary'].includes(flag)) {
      options[flag.slice(2)] = argv[i + 1];
      i += 1;
    } else {
      options.assets.push(flag);
    }
  }
  for (const required of ['out', 'tag', 'commit', 'notes']) {
    if (!options[required]) {
      console.error(`--${required} ontbreekt`);
      return 2;
    }
  }
  const body = renderGithubRelease({
    identity: readIdentity(root),
    tag: options.tag,
    commit: options.commit,
    notesText: readFileSync(options.notes, 'utf8'),
    repoUrl: githubUrl(options['repo-url'] ?? ''),
    gateSummary: options['gate-summary'] ? JSON.parse(readFileSync(options['gate-summary'], 'utf8')) : null,
    assets: options.assets.map((path) => ({ name: basename(path), sha256: sha256File(path) })),
  });
  writeFileSync(options.out, body);
  return 0;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  process.exit(main(process.argv.slice(2)));
}
