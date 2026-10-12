import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { readIdentity } from '../../scripts/nta-leveringsdocument.mjs';
import { releaseNotes } from '../../scripts/nta-kernel-version.mjs';
import { githubUrl, releasedItems, renderGithubRelease, shortCommit } from '../../scripts/nta-github-release.mjs';

const root = join(__dirname, '../..');
const notes = readFileSync(join(root, 'docs/nta8800-releasenotes.md'), 'utf8');

describe('GitHub release body (scripts/nta-github-release.mjs)', () => {
  it('lists the entries of the released kernel section', () => {
    const { section, items } = releasedItems(notes, '0.3.0');
    expect(section).toBe('Rekenkern 0.3.0');
    expect(items.length).toBeGreaterThan(5);
    expect(items.every((item) => !item.startsWith('#'))).toBe(true);
  });

  it('takes the entries that a release moves under the new kernel version', () => {
    const identity = readIdentity(root);
    const released = releaseNotes(notes, identity.kernelVersion, '11 oktober 2026');
    const unreleased = releasedItems(notes, identity.kernelVersion);
    const moved = releasedItems(released, identity.kernelVersion);
    expect(unreleased.section).toBe('Onuitgebracht');
    expect(moved.section).toBe(`Rekenkern ${identity.kernelVersion}`);
    expect(moved.items).toEqual(unreleased.items);
  });

  it('fills versions, the attest status, the items and every asset with its SHA-256', () => {
    const identity = readIdentity(root);
    const text = renderGithubRelease({
      identity,
      tag: `oes-v${identity.programVersion}-kernel-v${identity.kernelVersion}`,
      commit: '0123456789abcdef0123',
      notesText: notes,
      repoUrl: 'https://github.com/owner/repo',
      assets: [
        { name: 'Open Energy Studio_0.2.0_amd64.deb', sha256: 'a'.repeat(64) },
        { name: 'leveringsdocument.md', sha256: 'b'.repeat(64) },
      ],
    });
    expect(text).toContain(`| Program | **${identity.programVersion}** |`);
    expect(text).toContain(`| NTA 8800 kernel (\`KERNEL_VERSION\`) | **${identity.kernelVersion}** |`);
    expect(text).toContain('commit `0123456789ab`');
    expect(text).toContain('| Attest number (BRL 9501) | nog niet toegekend |');
    expect(text).toContain('not yet attested');
    expect(text).toContain(`| \`Open Energy Studio_0.2.0_amd64.deb\` | \`${'a'.repeat(64)}\` |`);
    expect(text).toContain('https://github.com/owner/repo/blob/oes-v');
    expect(text).not.toMatch(/\{\{|undefined/);
  });

  it('keeps the local-changes marker of a dry run from a dirty worktree', () => {
    // release-nta.sh stamps the same marker in the leveringsdocument, testrapport and manifest.
    expect(shortCommit('0123456789abcdef0123')).toBe('`0123456789ab`');
    expect(shortCommit('0123456789abcdef0123 (met lokale wijzigingen)')).toBe(
      '`0123456789ab` (met lokale wijzigingen)',
    );
    const identity = readIdentity(root);
    const text = renderGithubRelease({
      identity, tag: 't', commit: '0123456789abcdef0123 (met lokale wijzigingen)',
      notesText: notes, repoUrl: '', assets: [],
    });
    expect(text).toContain('commit `0123456789ab` (met lokale wijzigingen) |');
  });

  it('reads the repository URL from an https or ssh remote', () => {
    expect(githubUrl('https://github.com/Erika98-be/open-energy-studio.git')).toBe(
      'https://github.com/Erika98-be/open-energy-studio',
    );
    expect(githubUrl('git@github.com:owner/repo.git')).toBe('https://github.com/owner/repo');
    expect(githubUrl('/srv/git/repo.git')).toBe('');
  });
});
