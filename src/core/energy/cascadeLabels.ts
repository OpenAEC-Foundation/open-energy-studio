import type { IProject } from './types';
import type { CascadeEntry } from './projectDelete';

type Translate = (key: string, params?: Record<string, string | number>) => string;

/** Name of the zone, surface or window with this id, or the id itself. */
function itemName(project: IProject, id: string): string {
  for (const zone of project.zones) {
    if (zone.id === id) return zone.name || id;
    for (const surface of zone.surfaces) {
      if (surface.id === id) return surface.name || id;
      for (const window of surface.windows) {
        if (window.id === id) return window.name || id;
      }
    }
  }
  return id;
}

const SYSTEM_LISTS = ['heatingSystems', 'hotWaterSystems', 'ntaHeatPumps'] as const;

/**
 * One readable line per cascade entry: what kind of NTA input goes and which
 * element it referred to. The JSON path stays available as a tooltip.
 */
export function describeCascade(entry: CascadeEntry, project: IProject, t: Translate): string {
  const name = itemName(project, entry.id);
  for (const list of SYSTEM_LISTS) {
    const match = new RegExp(`^${list}\\[(\\d+)\\]`).exec(entry.path);
    if (match) {
      const system = (project[list] as Array<{ name?: string; id: string }> | undefined)?.[Number(match[1])];
      return t('item.cascade.servedZone', { system: system?.name || system?.id || list, name });
    }
  }
  const section = /^ntaCalculation\.([A-Za-z]+)/.exec(entry.path)?.[1] ?? 'other';
  const key = `item.cascadeKind.${section}`;
  const translated = t(key);
  const kind = translated === key ? t('item.cascadeKind.other') : translated;
  const zoneIdsOnly = entry.path.endsWith('.zoneIds');
  return t(zoneIdsOnly ? 'item.cascade.zoneFromList' : 'item.cascade.removed', { kind, name });
}

/** Surfaces and windows that disappear together with a zone or surface. */
export function containedItems(project: IProject, itemType: string, id: string): { surfaces: number; windows: number } {
  if (itemType === 'zone') {
    const zone = project.zones.find((z) => z.id === id);
    if (!zone) return { surfaces: 0, windows: 0 };
    return {
      surfaces: zone.surfaces.length,
      windows: zone.surfaces.reduce((count, surface) => count + surface.windows.length, 0),
    };
  }
  if (itemType === 'surface') {
    for (const zone of project.zones) {
      const surface = zone.surfaces.find((s) => s.id === id);
      if (surface) return { surfaces: 0, windows: surface.windows.length };
    }
  }
  return { surfaces: 0, windows: 0 };
}
