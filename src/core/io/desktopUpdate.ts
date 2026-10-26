/**
 * Update notice of the desktop app (26 Oct 2026): the web app loads a newer
 * build by itself (newVersion.ts), the desktop app cannot. At start and once a
 * day it asks GitHub for the releases of this program and says when one is
 * newer than the running version, so nobody keeps working on an old version
 * without knowing. It installs nothing; the adviser downloads the new version.
 * Pre-releases count, since every release so far is one. The adviser can turn
 * the check off (settings); it then sends no request.
 */

export const RELEASES_API = 'https://api.github.com/repos/OpenAEC-Foundation/open-energy-studio/releases?per_page=20';
export const UPDATE_CHECK_EVERY_MS = 24 * 60 * 60 * 1000;
const CHECK_OFF_KEY = 'oes.updateCheck.v1';
const DISMISSED_KEY = 'oes.updateDismissed.v1';

export interface AvailableUpdate {
  version: string;
  /** The release tag, e.g. `v2026.45.0`; the desktop command opens its page. */
  tag: string;
}

interface GitHubRelease {
  tag_name?: unknown;
  draft?: unknown;
}

/** The program version of a release tag (`v2026.45.0`, or the older `oes-v0.3.0-kernel-v0.4.0`), or null. */
export function versionOfTag(tag: string): number[] | null {
  const match = /^(?:oes-)?v(\d+)\.(\d+)\.(\d+)(?:-kernel-v\d+\.\d+\.\d+)?$/.exec(tag.trim());
  return match ? match.slice(1, 4).map(Number) : null;
}

export function compareVersions(a: number[], b: number[]): number {
  for (let index = 0; index < 3; index += 1) {
    if (a[index] !== b[index]) return a[index] - b[index];
  }
  return 0;
}

/** The newest published release that is newer than `current`, or null. */
export function newestRelease(releases: unknown, current: string): AvailableUpdate | null {
  const running = versionOfTag(`v${current}`);
  if (!running || !Array.isArray(releases)) return null;
  let best: { parts: number[]; tag: string } | null = null;
  for (const release of releases as GitHubRelease[]) {
    if (release?.draft === true || typeof release?.tag_name !== 'string') continue;
    // Only the plain program tags vJJJJ.W.N open a page through the desktop command.
    if (!/^v\d+\.\d+\.\d+$/.test(release.tag_name)) continue;
    const parts = versionOfTag(release.tag_name);
    if (!parts || compareVersions(parts, running) <= 0) continue;
    if (!best || compareVersions(parts, best.parts) > 0) best = { parts, tag: release.tag_name };
  }
  return best ? { version: best.parts.join('.'), tag: best.tag } : null;
}

/** Asks GitHub; null on any failure (offline, rate limit), so the app never complains about the check itself. */
export async function checkForUpdate(current: string, fetcher: typeof fetch = fetch): Promise<AvailableUpdate | null> {
  try {
    const response = await fetcher(RELEASES_API, { headers: { Accept: 'application/vnd.github+json' }, cache: 'no-store' });
    if (!response.ok) return null;
    return newestRelease(await response.json(), current);
  } catch {
    return null;
  }
}

export function updateCheckEnabled(): boolean {
  try { return localStorage.getItem(CHECK_OFF_KEY) !== 'off'; } catch { return true; }
}

export function setUpdateCheckEnabled(enabled: boolean): void {
  try {
    if (enabled) localStorage.removeItem(CHECK_OFF_KEY);
    else localStorage.setItem(CHECK_OFF_KEY, 'off');
  } catch { /* storage unavailable: the default (on) stays */ }
}

/** "Later" hides the notice for this version only; a newer one shows it again. */
export function updateDismissed(version: string): boolean {
  try { return localStorage.getItem(DISMISSED_KEY) === version; } catch { return false; }
}

export function dismissUpdate(version: string): void {
  try { localStorage.setItem(DISMISSED_KEY, version); } catch { /* shown again next start */ }
}
