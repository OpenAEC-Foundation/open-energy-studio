/**
 * Recently opened or saved project files (desktop), for the welcome screen.
 * Kept per viewer in localStorage; browser documents have no path and are
 * not listed.
 */
import type { BuildingFunction } from '../energy/types';

export interface RecentProject {
  path: string;
  name: string;
  buildingFunction?: BuildingFunction;
  /** Energy label class of the last calculated result (e.g. 'A+'), when known. */
  labelClass?: string;
  /** ISO 8601 time of the last open or save. */
  at: string;
}

export const RECENT_PROJECTS_KEY = 'oes-recent-projects';
export const RECENT_PROJECTS_MAX = 8;

function storage(): Storage | null {
  try {
    return typeof localStorage === 'undefined' ? null : localStorage;
  } catch {
    return null;
  }
}

export function readRecentProjects(): RecentProject[] {
  try {
    const parsed: unknown = JSON.parse(storage()?.getItem(RECENT_PROJECTS_KEY) ?? '[]');
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((entry): entry is RecentProject =>
      entry != null && typeof entry.path === 'string' && entry.path !== ''
      && typeof entry.name === 'string' && typeof entry.at === 'string');
  } catch {
    return [];
  }
}

function write(entries: RecentProject[]): void {
  try {
    storage()?.setItem(RECENT_PROJECTS_KEY, JSON.stringify(entries));
  } catch { /* storage full or blocked: the list is a convenience */ }
}

/**
 * Put a project file first in the list (one entry per path, newest first, at most eight).
 * Without a label class (opened, not yet calculated) the label of the earlier entry is kept.
 */
export function recordRecentProject(entry: Omit<RecentProject, 'at'>, at = new Date()): RecentProject[] {
  const current = readRecentProjects();
  const labelClass = entry.labelClass ?? current.find((item) => item.path === entry.path)?.labelClass;
  const next = [{ ...entry, ...(labelClass ? { labelClass } : {}), at: at.toISOString() }, ...current.filter((item) => item.path !== entry.path)]
    .slice(0, RECENT_PROJECTS_MAX);
  write(next);
  return next;
}

/** Drop a path, e.g. when the file can no longer be read. */
export function forgetRecentProject(path: string): RecentProject[] {
  const next = readRecentProjects().filter((item) => item.path !== path);
  write(next);
  return next;
}
