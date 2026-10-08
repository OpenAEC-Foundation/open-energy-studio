/**
 * Window types (kozijntypen), the way Uniec3 and the NTA input treat them
 * (8 Oct 2026): one named type per kind of window, door or panel with U, g
 * and an area per unit; a window on a surface refers to a type with a count.
 * The window keeps the resolved area, U and g inline, so the kernel, the
 * reports, the exports and the 3D view read a window as before; a change of
 * the type is written through to every window that refers to it.
 */
import type { IProject, IWindow, IWindowType } from './types';

export const WINDOW_KINDS: IWindowType['kind'][] = ['window', 'door', 'panel'];

const round = (value: number) => Number(value.toFixed(3));

export function windowTypeOf(project: Pick<IProject, 'windowTypes'>, window: Pick<IWindow, 'typeId'>): IWindowType | undefined {
  return window.typeId ? project.windowTypes?.find((type) => type.id === window.typeId) : undefined;
}

/** The window with U and g of its type and, when it has a count and the type an area per unit, that area. */
export function resolveWindow(window: IWindow, type: IWindowType | undefined): IWindow {
  if (!type) return window;
  const area = window.count != null && type.unitArea != null ? round(window.count * type.unitArea) : window.area;
  if (window.uValue === type.uValue && window.gValue === type.gValue && window.area === area) return window;
  return { ...window, uValue: type.uValue, gValue: type.gValue, area };
}

/** Every window of `typeId` (all typed windows without it) resolved against its type; untouched objects keep their identity. */
export function applyWindowTypes(project: IProject, typeId?: string): IProject {
  let changed = false;
  const zones = project.zones.map((zone) => {
    const surfaces = zone.surfaces.map((surface) => {
      const windows = surface.windows.map((window) => {
        if (!window.typeId || (typeId && window.typeId !== typeId)) return window;
        const next = resolveWindow(window, windowTypeOf(project, window));
        if (next !== window) changed = true;
        return next;
      });
      return windows.some((window, index) => window !== surface.windows[index]) ? { ...surface, windows } : surface;
    });
    return surfaces.some((surface, index) => surface !== zone.surfaces[index]) ? { ...zone, surfaces } : zone;
  });
  return changed ? { ...project, zones } : project;
}

/** Windows of a removed type keep their values but lose the reference and count. */
export function detachWindowType(project: IProject, typeId: string): IProject {
  const zones = project.zones.map((zone) => ({
    ...zone,
    surfaces: zone.surfaces.map((surface) => ({
      ...surface,
      windows: surface.windows.map((window) => {
        if (window.typeId !== typeId) return window;
        const { typeId: _type, count: _count, ...rest } = window;
        void _type; void _count;
        return rest;
      }),
    })),
  }));
  return { ...project, zones };
}

export function windowTypeUsage(project: IProject, typeId: string): number {
  return project.zones.reduce((sum, zone) => sum + zone.surfaces.reduce((count, surface) =>
    count + surface.windows.filter((window) => window.typeId === typeId).length, 0), 0);
}

export function newWindowTypeId(project: Pick<IProject, 'windowTypes'>): string {
  const used = new Set((project.windowTypes ?? []).map((type) => type.id));
  let index = (project.windowTypes?.length ?? 0) + 1;
  while (used.has(`wt-${index}`)) index += 1;
  return `wt-${index}`;
}
