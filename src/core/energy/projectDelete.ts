import type { IProject } from './types';
import type { MwaMeasure } from '../nta/KernelClient';

/**
 * Deleting a zone, surface or window also removes the NTA input that refers
 * to it by id. Without this the kernel stops on dangling references
 * (`zone_data_without_zone`, `dynamic_window_without_window`,
 * `ground_floor_data_without_ground_surface`, `heating_system_zone_unknown`,
 * `cooling_zone_unknown`, `lighting_zone_unknown`).
 *
 * The NTA block is walked generically: an array element that is an object
 * with `zoneId`, `surfaceId` or `windowId` naming a removed item is dropped.
 * A `zoneIds` list is filtered; an element whose list becomes empty (a
 * heating or cooling system that only served removed zones) is dropped too.
 * A top-level object that carries such an id (no array around it) is kept.
 */
export interface RemovedIds {
  zoneIds: Set<string>;
  surfaceIds: Set<string>;
  windowIds: Set<string>;
}

export interface CascadeEntry {
  /** JSON path of the removed or changed element in the old NTA block. */
  path: string;
  /** Id that caused it. */
  id: string;
}

export interface DeleteResult {
  project: IProject;
  cascade: CascadeEntry[];
}

const REF_KEYS: Array<[keyof RemovedIds, string]> = [
  ['zoneIds', 'zoneId'], ['surfaceIds', 'surfaceId'], ['windowIds', 'windowId'],
];

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function matchedRef(element: Record<string, unknown>, removed: RemovedIds): string | null {
  for (const [set, key] of REF_KEYS) {
    const value = element[key];
    if (typeof value === 'string' && removed[set].has(value)) return value;
  }
  return null;
}

function cascadeValue(value: unknown, path: string, removed: RemovedIds, log: CascadeEntry[]): unknown {
  if (Array.isArray(value)) {
    let changed = false;
    const out: unknown[] = [];
    value.forEach((element, index) => {
      const elementPath = `${path}[${index}]`;
      if (isObject(element)) {
        const ref = matchedRef(element, removed);
        if (ref !== null) { log.push({ path: elementPath, id: ref }); changed = true; return; }
        const zoneIds = element.zoneIds;
        if (Array.isArray(zoneIds) && zoneIds.some((id) => typeof id === 'string' && removed.zoneIds.has(id))) {
          const kept = zoneIds.filter((id) => !(typeof id === 'string' && removed.zoneIds.has(id)));
          const id = zoneIds.find((z) => typeof z === 'string' && removed.zoneIds.has(z)) as string;
          log.push({ path: kept.length === 0 ? elementPath : `${elementPath}.zoneIds`, id });
          changed = true;
          if (kept.length === 0) return;
          out.push(cascadeValue({ ...element, zoneIds: kept }, elementPath, removed, log));
          return;
        }
      }
      const next = cascadeValue(element, elementPath, removed, log);
      if (next !== element) changed = true;
      out.push(next);
    });
    return changed ? out : value;
  }
  if (isObject(value)) {
    let changed = false;
    const out: Record<string, unknown> = {};
    for (const [key, child] of Object.entries(value)) {
      const next = cascadeValue(child, path ? `${path}.${key}` : key, removed, log);
      if (next !== child) changed = true;
      out[key] = next;
    }
    return changed ? out : value;
  }
  return value;
}

/** Removes NTA references to the removed ids; returns the same object when nothing refers to them. */
export function cascadeNtaReferences<T>(nta: T, removed: RemovedIds): { nta: T; cascade: CascadeEntry[] } {
  const cascade: CascadeEntry[] = [];
  const next = cascadeValue(nta, 'ntaCalculation', removed, cascade) as T;
  return { nta: next, cascade };
}

function emptyRemoved(): RemovedIds {
  return { zoneIds: new Set(), surfaceIds: new Set(), windowIds: new Set() };
}

function finish(project: IProject, zones: IProject['zones'], removed: RemovedIds): DeleteResult {
  if (!project.ntaCalculation) return { project: { ...project, zones }, cascade: [] };
  const { nta, cascade } = cascadeNtaReferences(project.ntaCalculation, removed);
  return { project: { ...project, zones, ntaCalculation: nta }, cascade };
}

export function deleteZoneFromProject(project: IProject, zoneId: string): DeleteResult {
  const removed = emptyRemoved();
  const zone = project.zones.find((z) => z.id === zoneId);
  removed.zoneIds.add(zoneId);
  for (const surface of zone?.surfaces ?? []) {
    removed.surfaceIds.add(surface.id);
    for (const win of surface.windows) removed.windowIds.add(win.id);
  }
  return finish(project, project.zones.filter((z) => z.id !== zoneId), removed);
}

export function deleteSurfaceFromProject(project: IProject, zoneId: string, surfaceId: string): DeleteResult {
  const removed = emptyRemoved();
  const zones = project.zones.map((z) => {
    if (z.id !== zoneId) return z;
    const surface = z.surfaces.find((s) => s.id === surfaceId);
    if (surface) {
      removed.surfaceIds.add(surface.id);
      for (const win of surface.windows) removed.windowIds.add(win.id);
    }
    return { ...z, surfaces: z.surfaces.filter((s) => s.id !== surfaceId) };
  });
  return finish(project, zones, removed);
}

export function deleteWindowFromProject(project: IProject, zoneId: string, surfaceId: string, windowId: string): DeleteResult {
  const removed = emptyRemoved();
  removed.windowIds.add(windowId);
  const zones = project.zones.map((z) => z.id !== zoneId ? z : {
    ...z,
    surfaces: z.surfaces.map((s) => s.id !== surfaceId ? s : {
      ...s, windows: s.windows.filter((w) => w.id !== windowId),
    }),
  });
  return finish(project, zones, removed);
}

function elementKey(element: unknown): string {
  if (isObject(element) && typeof element.id === 'string') return `id:${element.id}`;
  return `json:${JSON.stringify(element)}`;
}

/**
 * Whether a JSON-pointer path selects a different array element after the
 * change. Every array index on the way is compared by element id (or by
 * content when the element has no id), so a renumbered index is caught.
 */
function pathShifted(before: unknown, after: unknown, path: string): boolean {
  if (!path.startsWith('/')) return false;
  const keys = path.slice(1).split('/').map((key) => key.replace(/~1/g, '/').replace(/~0/g, '~'));
  let a: unknown = before;
  let b: unknown = after;
  for (const key of keys) {
    if (Array.isArray(a)) {
      if (!/^\d+$/.test(key)) return false;
      const index = Number(key);
      if (index >= a.length) return false;
      const nextB = Array.isArray(b) ? b[index] : undefined;
      if (nextB === undefined || elementKey(a[index]) !== elementKey(nextB)) return true;
      a = a[index];
      b = nextB;
    } else if (isObject(a)) {
      if (!(key in a)) return false;
      a = a[key];
      b = isObject(b) ? b[key] : undefined;
      if (b === undefined) return true;
    } else {
      return false;
    }
  }
  return false;
}

/**
 * Manual (non-template) maatwerkadvies measures address the project by array
 * index. Template measures are rebuilt by id before every calculation; manual
 * ones are not, so a delete that renumbers an index they use would make them
 * change a different element without any error.
 */
export function manualMeasuresShiftedBy(before: IProject, after: IProject, measures: MwaMeasure[] | undefined): string[] {
  const hits: string[] = [];
  for (const measure of measures ?? []) {
    if (measure.template || measure.target !== 'project') continue;
    if ((measure.patch ?? []).some((op) => pathShifted(before, after, op.path))) hits.push(measure.name || measure.id);
  }
  return hits;
}
