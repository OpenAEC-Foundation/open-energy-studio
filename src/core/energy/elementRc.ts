/**
 * One Rc per kind of surface at the start of the new-build input (feedback
 * 8 Oct 2026: "geef de Rc-waarde van het totaal in het begin: wanden, vloeren").
 * The surfaces of a kind share one construction; an Rc entered for the kind
 * sets that construction (U = 1/(R_si + Rc + R_se), NTA 8800 table C.2), and
 * one U for all windows. A surface can still get its own construction after.
 */
import type { IConstruction, IProject, ISurface, SurfaceType } from './types';
import { insulationValues, type InsulationPart } from '../nta/MwaTemplates';

export type ElementKind = 'wall' | 'roof' | 'floor';
export const ELEMENT_KINDS: ElementKind[] = ['wall', 'roof', 'floor'];
const PART: Record<ElementKind, InsulationPart> = { wall: 'facade', roof: 'roof', floor: 'floor' };

/** Id of the construction the app makes for a kind. */
export const elementConstructionId = (kind: ElementKind) => `rc-${kind}`;

type Located = { zoneId: string; surface: ISurface };

/** The surfaces of a kind on the thermal envelope (not internal walls, not towards a heated neighbour). */
export function surfacesOf(project: IProject, kind: ElementKind): Located[] {
  return project.zones.flatMap((zone) => zone.surfaces
    .filter((surface) => surface.type === (kind as SurfaceType)
      && surface.thermalBoundary !== 'internal' && surface.thermalBoundary !== 'adjacent_conditioned')
    .map((surface) => ({ zoneId: zone.id, surface })));
}

/** The construction all surfaces of a kind share, or null when they differ or there are none. */
export function sharedConstruction(project: IProject, kind: ElementKind): IConstruction | null {
  const ids = new Set(surfacesOf(project, kind).map(({ surface }) => surface.constructionId));
  if (ids.size !== 1) return null;
  const [id] = [...ids];
  return project.constructions.find((construction) => construction.id === id) ?? null;
}

/** True when a construction is also used by surfaces of another kind (then it is not changed in place). */
function usedElsewhere(project: IProject, id: string, kind: ElementKind): boolean {
  return project.zones.some((zone) => zone.surfaces.some((surface) => surface.constructionId === id
    && !surfacesOf(project, kind).some((item) => item.surface === surface)));
}

export type ElementRcAction =
  | { type: 'ADD_CONSTRUCTION'; payload: IConstruction }
  | { type: 'UPDATE_CONSTRUCTION'; payload: { id: string; data: Partial<IConstruction> } }
  | { type: 'UPDATE_SURFACE'; payload: { zoneId: string; surfaceId: string; data: Partial<ISurface> } };

/**
 * The project changes for an Rc of a kind: the shared construction is set in
 * place when only this kind uses it; otherwise the kind's own construction is
 * made or set and every surface of the kind points to it.
 */
export function elementRcActions(project: IProject, kind: ElementKind, rc: number, name: string): ElementRcAction[] {
  const values = insulationValues(PART[kind], rc, null);
  if (!values) return [];
  const data = { rcValue: values.rc, uValue: Number(values.u.toFixed(4)), layers: [] };
  const shared = sharedConstruction(project, kind);
  if (shared && !usedElsewhere(project, shared.id, kind)) {
    return [{ type: 'UPDATE_CONSTRUCTION', payload: { id: shared.id, data } }];
  }
  const id = elementConstructionId(kind);
  const actions: ElementRcAction[] = project.constructions.some((construction) => construction.id === id)
    ? [{ type: 'UPDATE_CONSTRUCTION', payload: { id, data } }]
    : [{ type: 'ADD_CONSTRUCTION', payload: { id, name, ...data } }];
  for (const { zoneId, surface } of surfacesOf(project, kind)) {
    if (surface.constructionId !== id) actions.push({ type: 'UPDATE_SURFACE', payload: { zoneId, surfaceId: surface.id, data: { constructionId: id } } });
  }
  return actions;
}

/** The U all windows share, or null when they differ or there are none. */
export function sharedWindowU(project: IProject): number | null {
  const values = new Set(project.zones.flatMap((zone) => zone.surfaces.flatMap((surface) => surface.windows.map((window) => window.uValue))));
  return values.size === 1 ? [...values][0] : null;
}
