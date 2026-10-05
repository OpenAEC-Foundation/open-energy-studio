/**
 * Project items and their paths, shared by "Ga naar" (select the item a kernel
 * path points at) and the command palette (find an item by name).
 */
import type { IProject } from '../energy/types';
import { formatKernelPath, parseKernelPath } from '../nta/pathUtil';
import type { Route } from './routes';

export interface ProjectItemRef {
  id: string;
  itemType: string;
  name: string;
  path: string;
  route: Route;
  /** Translation key of the item kind, e.g. `browser.windows`. */
  kindKey: string;
}

export type SystemListKey = 'heatingSystems' | 'ventilationSystems' | 'coolingSystems' | 'hotWaterSystems' | 'solarPV' | 'solarThermal';

/** The systems of the project model, each with its Installaties sub page (UI redesign F5). */
const SYSTEM_LISTS: Array<{ key: SystemListKey; itemType: string; kindKey: string; sub: string }> = [
  { key: 'heatingSystems', itemType: 'heatingSystem', kindKey: 'browser.heating', sub: 'heating' },
  { key: 'ventilationSystems', itemType: 'ventilationSystem', kindKey: 'browser.ventilation', sub: 'ventilation' },
  { key: 'coolingSystems', itemType: 'coolingSystem', kindKey: 'browser.cooling', sub: 'cooling' },
  { key: 'hotWaterSystems', itemType: 'hotWaterSystem', kindKey: 'browser.hotWater', sub: 'hotWater' },
  { key: 'solarPV', itemType: 'solarPV', kindKey: 'browser.solarPV', sub: 'generation' },
  { key: 'solarThermal', itemType: 'solarThermal', kindKey: 'browser.solarThermal', sub: 'generation' },
];

export { SYSTEM_LISTS };

/** Every selectable item of the project with its path and page. */
export function projectItems(project: IProject): ProjectItemRef[] {
  const envelope: Route = { step: 'building', sub: 'envelope' };
  const zonesPage: Route = { step: 'building', sub: 'zones' };
  const bridges: Route = { step: 'building', sub: 'thermalBridges' };
  const constructionsPage: Route = { step: 'building', sub: 'constructions' };
  const items: ProjectItemRef[] = [];
  project.zones.forEach((zone, z) => {
    items.push({ id: zone.id, itemType: 'zone', name: zone.name, path: formatKernelPath(['zones', z]), route: zonesPage, kindKey: 'browser.zones' });
    zone.surfaces.forEach((surface, s) => {
      items.push({ id: surface.id, itemType: 'surface', name: surface.name, path: formatKernelPath(['zones', z, 'surfaces', s]), route: envelope, kindKey: 'browser.surfaces' });
      surface.windows.forEach((win, w) => items.push({
        id: win.id, itemType: 'window', name: win.name, path: formatKernelPath(['zones', z, 'surfaces', s, 'windows', w]), route: envelope, kindKey: 'browser.windows',
      }));
    });
    zone.thermalBridges.forEach((bridge, b) => items.push({
      id: bridge.id, itemType: 'thermalBridge', name: bridge.name, path: formatKernelPath(['zones', z, 'thermalBridges', b]), route: bridges, kindKey: 'browser.thermalBridges',
    }));
    (zone.pointThermalBridges ?? []).forEach((bridge, b) => items.push({
      id: bridge.id, itemType: 'pointBridge', name: bridge.name, path: formatKernelPath(['zones', z, 'pointThermalBridges', b]), route: bridges, kindKey: 'kernel.pointBridge.title',
    }));
  });
  project.constructions.forEach((construction, c) => items.push({
    id: construction.id, itemType: 'construction', name: construction.name, path: formatKernelPath(['constructions', c]), route: constructionsPage, kindKey: 'browser.constructions',
  }));
  for (const list of SYSTEM_LISTS) {
    (project[list.key] as Array<{ id: string; name: string }>).forEach((system, index) => items.push({
      id: system.id, itemType: list.itemType, name: system.name, path: formatKernelPath([list.key, index]), route: { step: 'installations', sub: list.sub }, kindKey: list.kindKey,
    }));
  }
  return items;
}

/**
 * The deepest project item a kernel path points into, e.g. the window for
 * `zones[0].surfaces[1].windows[0].gValue`; null when the path names no item.
 */
export function selectionForPath(project: IProject, path: string | null | undefined): { id: string; itemType: string } | null {
  const segments = parseKernelPath(path).filter((segment, index) => !(index === 0 && segment === 'derivedInput'));
  let best: ProjectItemRef | null = null;
  let bestLength = 0;
  for (const item of projectItems(project)) {
    const itemSegments = parseKernelPath(item.path);
    if (itemSegments.length > bestLength && itemSegments.every((segment, index) => segments[index] === segment)) {
      best = item;
      bestLength = itemSegments.length;
    }
  }
  return best ? { id: best.id, itemType: best.itemType } : null;
}
