/**
 * The NTA 8800 edition new calculations start in (Settings › Berekening).
 * Only editions the kernel implements are accepted; the current edition is
 * the default and the only registrable one.
 */
import { DEFAULT_NORM_VERSION, IMPLEMENTED_NORM_VERSIONS, type NormVersion } from './KernelClient';

export const DEFAULT_EDITION_KEY = 'oes-default-norm-version';

export function readDefaultEdition(): NormVersion {
  try {
    const stored = typeof localStorage === 'undefined' ? null : localStorage.getItem(DEFAULT_EDITION_KEY);
    return IMPLEMENTED_NORM_VERSIONS.includes(stored as NormVersion) ? stored as NormVersion : DEFAULT_NORM_VERSION;
  } catch {
    return DEFAULT_NORM_VERSION;
  }
}

export function writeDefaultEdition(edition: NormVersion): void {
  try {
    if (edition === DEFAULT_NORM_VERSION) localStorage.removeItem(DEFAULT_EDITION_KEY);
    else localStorage.setItem(DEFAULT_EDITION_KEY, edition);
  } catch { /* blocked storage: keep the current edition */ }
}
