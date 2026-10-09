/**
 * Several opaque parts in one surface (planner, 9 Oct 2026: "meerdere dichte
 * delen per vlak"), e.g. a facade of brickwork with a timber part. The kernel
 * takes one construction per survey surface (`SurveySurface`, no extra
 * fields), so every part is a surface of its own; the app keeps which surface
 * it belongs to beside the survey (`StoredSurvey.surfaceParts`, part id →
 * surface id). A part follows its surface in element, boundary, orientation,
 * tilt and zone; it has its own area and construction. Openings stay on the
 * surface itself.
 */
import type { StoredSurvey } from '../nta/SurveyTemplates';

/** What a part takes over from its surface. */
export const PART_SYNC_KEYS = ['element', 'boundary', 'orientation', 'tiltDeg', 'zoneId'] as const;

type Surface = Record<string, unknown>;
const surfacesOf = (stored: StoredSurvey): Surface[] =>
  ((stored.survey.envelope as { surfaces?: Surface[] } | undefined)?.surfaces ?? []);
const withSurfaces = (stored: StoredSurvey, surfaces: Surface[]): StoredSurvey => ({
  ...stored,
  survey: { ...stored.survey, envelope: { ...(stored.survey.envelope as Record<string, unknown>), surfaces } },
});

/** The surface a part belongs to, when that surface still exists. */
export function parentOf(stored: StoredSurvey, id: string): string | null {
  const parent = stored.surfaceParts?.[id];
  return parent && surfacesOf(stored).some((surface) => surface.id === parent) ? parent : null;
}

/** The ids of the parts of a surface, in survey order. */
export function partsOf(stored: StoredSurvey, surfaceId: string): string[] {
  return surfacesOf(stored).map((surface) => String(surface.id ?? '')).filter((id) => stored.surfaceParts?.[id] === surfaceId);
}

/** Adds an opaque part to a surface: its own area and construction, the rest taken over. */
export function addPart(stored: StoredSurvey, surfaceId: string): StoredSurvey {
  const surfaces = surfacesOf(stored);
  const parent = surfaces.find((surface) => surface.id === surfaceId);
  if (!parent) return stored;
  // Numbered per surface like its name (part 2, 3, …), unique among all surfaces.
  const used = new Set(surfaces.map((surface) => String(surface.id ?? '')));
  let n = partsOf(stored, surfaceId).length + 2;
  while (used.has(`${surfaceId}-deel-${n}`)) n += 1;
  const id = `${surfaceId}-deel-${n}`;
  const part: Surface = { id, grossAreaM2: 0, cavity: Boolean(parent.cavity), insulation: { kind: 'none_or_unknown' }, sourceReference: '' };
  for (const key of PART_SYNC_KEYS) if (parent[key] !== undefined) part[key] = parent[key];
  return { ...withSurfaces(stored, [...surfaces, part]), surfaceParts: { ...(stored.surfaceParts ?? {}), [id]: surfaceId } };
}

/** Takes the shared answers of every surface over into its parts. */
export function syncParts(stored: StoredSurvey): StoredSurvey {
  const surfaces = surfacesOf(stored);
  const byId = new Map(surfaces.map((surface) => [String(surface.id ?? ''), surface]));
  let changed = false;
  const next = surfaces.map((surface) => {
    const parent = byId.get(stored.surfaceParts?.[String(surface.id ?? '')] ?? '');
    if (!parent) return surface;
    const synced: Surface = { ...surface };
    for (const key of PART_SYNC_KEYS) {
      if (JSON.stringify(synced[key]) === JSON.stringify(parent[key])) continue;
      if (parent[key] === undefined) delete synced[key]; else synced[key] = parent[key];
      changed = true;
    }
    return synced;
  });
  return changed ? withSurfaces(stored, next) : stored;
}

/** Removes a surface with its parts, or one part. */
export function removeSurface(stored: StoredSurvey, id: string): { stored: StoredSurvey; removed: string[] } {
  const removed = [id, ...partsOf(stored, id)];
  const parts = { ...(stored.surfaceParts ?? {}) };
  for (const item of removed) delete parts[item];
  return { stored: { ...withSurfaces(stored, surfacesOf(stored).filter((surface) => !removed.includes(String(surface.id ?? '')))), surfaceParts: parts }, removed };
}
