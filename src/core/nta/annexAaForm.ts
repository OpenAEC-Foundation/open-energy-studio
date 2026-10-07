/**
 * Annex AA (simplified cooling need and minimum cooling capacity, §5.7.1) as
 * form data: the `activeCooling.capacity.calculation` block of the kernel's
 * `AnnexAaInput` (crates/nta8800-core/src/annex_aa.rs), with helpers for the
 * room table and a client-side check that mirrors the kernel's own checks, so
 * the form can point at a field before the kernel runs. The kernel stays the
 * validator; codes and paths are the kernel's.
 */

/** Draft path of the calculation inside `ntaCalculation`. */
export const ANNEX_AA_PATH = ['activeCooling', 'capacity', 'calculation'] as const;

export interface AnnexAaWindowDraft {
  windowId: string;
  /** U_w+shut (8.22) when shading per 7.6.6.1.4 is present; blank uses U_w of the window. */
  uWithShutterWPerM2k?: number | null;
}

export interface AnnexAaRoomDraft {
  id: string;
  areaM2: number | null;
  living: boolean;
  opaqueInnerAreaM2: number | null;
  windows: AnnexAaWindowDraft[];
  installedCapacityKw: number | null;
  /** NTA 8800:2024 only. */
  roofAreaM2?: number | null;
}

export interface AnnexAaDraft {
  constructionYear: number | null;
  postInsulated?: boolean;
  generatorCapacityKw?: number | null;
  rooms: AnnexAaRoomDraft[];
  /** NTA 8800:2024 only. */
  effectiveMassKgPerM2?: number | null;
}

export interface AnnexAaFormIssue {
  code: string;
  /** Draft path relative to `ntaCalculation`. */
  path: Array<string | number>;
}

export function newAnnexAaRoom(rooms: ReadonlyArray<{ id?: unknown }>, living = rooms.length === 0): AnnexAaRoomDraft {
  return {
    id: uniqueRoomId(rooms, living ? 'woonkamer' : 'vertrek'),
    areaM2: null,
    living,
    opaqueInnerAreaM2: null,
    windows: [],
    installedCapacityKw: null,
  };
}

export function newAnnexAaCalculation(constructionYear: unknown): AnnexAaDraft {
  return {
    constructionYear: typeof constructionYear === 'number' && Number.isFinite(constructionYear) ? constructionYear : null,
    postInsulated: false,
    rooms: [newAnnexAaRoom([], true)],
  };
}

/** `base`, `base-2`, `base-3`, … : the first id no room has yet. */
export function uniqueRoomId(rooms: ReadonlyArray<{ id?: unknown }>, base: string): string {
  const taken = new Set(rooms.map((room) => String(room?.id ?? '')));
  if (!taken.has(base)) return base;
  for (let n = 2; ; n += 1) if (!taken.has(`${base}-${n}`)) return `${base}-${n}`;
}

/**
 * A copy of room `index` placed after it with a new id. The windows are not
 * copied: the kernel refuses a window assigned to two rooms
 * (`annex_aa_window_assigned_twice`).
 */
export function duplicateAnnexAaRoom<T extends { id?: unknown; windows?: unknown }>(rooms: T[], index: number): T[] {
  const source = rooms[index];
  if (source == null) return rooms;
  const base = String(source.id ?? 'vertrek').replace(/-\d+$/, '');
  const copy = { ...structuredClone(source), id: uniqueRoomId(rooms, base), windows: [] } as T;
  return [...rooms.slice(0, index + 1), copy, ...rooms.slice(index + 1)];
}

/** Window ids already assigned to a room other than `roomIndex`. */
export function windowsAssignedElsewhere(rooms: ReadonlyArray<{ windows?: unknown }>, roomIndex: number): Set<string> {
  const ids = new Set<string>();
  rooms.forEach((room, index) => {
    if (index === roomIndex || !Array.isArray(room?.windows)) return;
    for (const window of room.windows as Array<{ windowId?: unknown }>) {
      if (typeof window?.windowId === 'string' && window.windowId !== '') ids.add(window.windowId);
    }
  });
  return ids;
}

const finite = (value: unknown): value is number => typeof value === 'number' && Number.isFinite(value);
/** The derived zone input names project windows `window:<id>`; the kernel resolves both. */
const bareWindowId = (id: string) => id.replace(/^window:/, '');

/**
 * The kernel's checks of `assess_annex_aa` that need no zone input (the
 * residential-only check and the calculation itself stay with the kernel),
 * plus the two members serde requires before it can check anything:
 * `constructionYear` and the room id.
 *
 * `edition2024` follows `AnnexAaVariant::Monthly2024`: SWM is required (50–100
 * kg/m²) and the roof area is allowed; in every other edition both are
 * `route_not_in_edition`. `windowIds` are the project's window ids; when
 * omitted, window ids are not checked.
 */
export function checkAnnexAa(calculation: unknown, options: { edition2024: boolean; windowIds?: ReadonlyArray<string> }): AnnexAaFormIssue[] {
  const base = [...ANNEX_AA_PATH];
  const issues: AnnexAaFormIssue[] = [];
  const push = (code: string, ...path: Array<string | number>) => issues.push({ code, path: [...base, ...path] });
  if (calculation == null || typeof calculation !== 'object') {
    issues.push({ code: 'annex_aa_calculation_required', path: base });
    return issues;
  }
  const aa = calculation as Record<string, unknown>;
  const year = aa.constructionYear;
  if (!(finite(year) && Number.isInteger(year) && year >= 1000 && year <= 2200)) push('annex_aa_construction_year_required', 'constructionYear');
  const rooms = Array.isArray(aa.rooms) ? aa.rooms as Array<Record<string, unknown>> : [];
  if (rooms.length === 0) push('annex_aa_room_required', 'rooms');

  const mass = aa.effectiveMassKgPerM2;
  if (options.edition2024) {
    if (mass == null) push('annex_aa_effective_mass_required', 'effectiveMassKgPerM2');
    else if (!(finite(mass) && mass >= 50 && mass <= 100)) push('annex_aa_effective_mass_invalid', 'effectiveMassKgPerM2');
  } else if (mass != null) {
    push('route_not_in_edition', 'effectiveMassKgPerM2');
  }
  const generator = aa.generatorCapacityKw;
  if (generator != null && !(finite(generator) && generator >= 0)) push('annex_aa_capacity_invalid', 'generatorCapacityKw');

  const known = options.windowIds == null ? null : new Set(options.windowIds.map(bareWindowId));
  const ids = new Set<string>();
  const assigned = new Set<string>();
  rooms.forEach((room, index) => {
    const at = (...path: Array<string | number>) => ['rooms', index, ...path];
    const id = typeof room?.id === 'string' ? room.id.trim() : '';
    if (id === '') push('annex_aa_room_id_required', ...at('id'));
    else if (ids.has(id)) push('annex_aa_room_id_duplicate', ...at('id'));
    else ids.add(id);
    if (!(finite(room?.areaM2) && room.areaM2 > 0)) push('annex_aa_area_invalid', ...at('areaM2'));
    if (!(finite(room?.opaqueInnerAreaM2) && room.opaqueInnerAreaM2 >= 0)) push('annex_aa_area_invalid', ...at('opaqueInnerAreaM2'));
    if (!(finite(room?.installedCapacityKw) && room.installedCapacityKw >= 0)) push('annex_aa_capacity_invalid', ...at('installedCapacityKw'));
    const roof = room?.roofAreaM2;
    if (roof != null) {
      if (!options.edition2024) push('route_not_in_edition', ...at('roofAreaM2'));
      else if (!(finite(roof) && roof >= 0)) push('annex_aa_area_invalid', ...at('roofAreaM2'));
    }
    const windows = Array.isArray(room?.windows) ? room.windows as Array<Record<string, unknown>> : [];
    windows.forEach((window, w) => {
      const windowId = typeof window?.windowId === 'string' ? bareWindowId(window.windowId) : '';
      if (windowId === '' || (known != null && !known.has(windowId))) {
        push('annex_aa_window_unknown', ...at('windows', w, 'windowId'));
      } else if (assigned.has(windowId)) {
        push('annex_aa_window_assigned_twice', ...at('windows', w, 'windowId'));
      } else {
        assigned.add(windowId);
      }
      const u = window?.uWithShutterWPerM2k;
      if (u != null && !(finite(u) && u > 0)) push('annex_aa_u_invalid', ...at('windows', w, 'uWithShutterWPerM2k'));
    });
  });
  return issues;
}

/** Σ floor area and Σ installed capacity of the rooms, for the table footer. */
export function annexAaTotals(calculation: unknown): { areaM2: number; capacityKw: number; rooms: number } {
  const rooms = Array.isArray((calculation as { rooms?: unknown } | null)?.rooms)
    ? (calculation as { rooms: Array<Record<string, unknown>> }).rooms : [];
  return rooms.reduce<{ areaM2: number; capacityKw: number; rooms: number }>((sum, room) => ({
    areaM2: sum.areaM2 + (finite(room?.areaM2) ? room.areaM2 : 0),
    capacityKw: sum.capacityKw + (finite(room?.installedCapacityKw) ? room.installedCapacityKw : 0),
    rooms: sum.rooms + 1,
  }), { areaM2: 0, capacityKw: 0, rooms: 0 });
}
