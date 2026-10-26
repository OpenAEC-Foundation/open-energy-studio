/**
 * Update notice of the desktop app (26 Oct 2026): it asks GitHub for the
 * releases and says when one is newer than the running version, so nobody
 * keeps working on an old version without knowing.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import {
  RELEASES_API, checkForUpdate, compareVersions, dismissUpdate, newestRelease, setUpdateCheckEnabled, updateCheckEnabled,
  updateDismissed, versionOfTag,
} from '../core/io/desktopUpdate';

afterEach(() => {
  localStorage.removeItem('oes.updateCheck.v1');
  localStorage.removeItem('oes.updateDismissed.v1');
});

const releases = [
  { tag_name: 'v2026.45.1', draft: true },
  { tag_name: 'v2026.45.0', draft: false, prerelease: true },
  { tag_name: 'v2026.44.0', draft: false },
  { tag_name: 'oes-v0.3.0-kernel-v0.4.0', draft: false },
  { tag_name: 'v0.1.7-alpha', draft: false },
];

describe('desktop update notice', () => {
  it('reads the program version of a release tag', () => {
    expect(versionOfTag('v2026.45.0')).toEqual([2026, 45, 0]);
    expect(versionOfTag('oes-v0.3.0-kernel-v0.4.0')).toEqual([0, 3, 0]);
    expect(versionOfTag('v0.1.7-alpha')).toBeNull();
    // Week 9 is older than week 10, also as text.
    expect(compareVersions([2026, 9, 3], [2026, 10, 0])).toBeLessThan(0);
  });

  it('names the newest published release above the running version, pre-releases included', () => {
    expect(newestRelease(releases, '2026.44.0')).toEqual({ version: '2026.45.0', tag: 'v2026.45.0' });
    // The draft does not count; nothing newer than the running version.
    expect(newestRelease(releases, '2026.45.0')).toBeNull();
    expect(newestRelease({ message: 'API rate limit exceeded' }, '2026.44.0')).toBeNull();
  });

  it('asks the releases of this repository and stays quiet on any failure', async () => {
    const ok = vi.fn(async () => new Response(JSON.stringify(releases), { status: 200 }));
    expect(await checkForUpdate('2026.44.0', ok as unknown as typeof fetch)).toEqual({ version: '2026.45.0', tag: 'v2026.45.0' });
    expect((ok.mock.calls[0] as unknown[])[0]).toBe(RELEASES_API);
    const limited = vi.fn(async () => new Response('{}', { status: 403 }));
    expect(await checkForUpdate('2026.44.0', limited as unknown as typeof fetch)).toBeNull();
    const offline = vi.fn(async () => { throw new TypeError('Failed to fetch'); });
    expect(await checkForUpdate('2026.44.0', offline as unknown as typeof fetch)).toBeNull();
  });

  it('can be turned off, and "Later" hides one version only', () => {
    expect(updateCheckEnabled()).toBe(true);
    setUpdateCheckEnabled(false);
    expect(updateCheckEnabled()).toBe(false);
    setUpdateCheckEnabled(true);
    expect(updateCheckEnabled()).toBe(true);
    dismissUpdate('2026.45.0');
    expect(updateDismissed('2026.45.0')).toBe(true);
    expect(updateDismissed('2026.46.0')).toBe(false);
  });

  it('opens the download page through a desktop command that is registered', () => {
    const banner = readFileSync(resolve(__dirname, '../components/shell/NewVersionBanner.tsx'), 'utf8');
    expect(banner).toContain("invoke('open_release_page'");
    const lib = readFileSync(resolve(__dirname, '../../src-tauri/src/lib.rs'), 'utf8');
    expect(lib).toMatch(/generate_handler!\[[\s\S]*open_release_page[\s\S]*\]\)/);
  });
});
