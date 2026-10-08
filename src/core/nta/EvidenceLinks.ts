import type { NtaEvidenceItem } from './KernelClient';
import type { IProject } from '../energy/types';
import { EVIDENCE_REFERENCE_PREFIX } from './Evidence';

/**
 * Links between the evidence register (BRL 9500 Bijlage 3) and the project.
 * An input refers to a file in two ways: a `…Reference` text holding
 * `evidence:<id>`, or a JSON pointer in the item's `linkedPaths` (an element,
 * a survey item). Both count as "onderbouwt" for the register and the dossier.
 *
 * A linked pointer names an array element by its `id` where it has one
 * (`/zones/@woonzone/surfaces/@gevel-noord`), so a link stays on its element
 * when other elements are removed or reordered. The survey's further
 * generators and hot-water systems get an id when added or when an older
 * project is opened (`withSurveyItemIds`). An element without a unique id
 * keeps its position; removing one through `linksAfterRemoval` moves the
 * links of the elements after it.
 */

/** Prefix of a pointer segment that names an array element by its `id` (`/zones/@woonzone`). */
export const ID_SEGMENT = '@';

/** JSON pointer (RFC 6901) of path segments. */
export function jsonPointer(segments: Array<string | number>): string {
  return segments.map((segment) => `/${String(segment).replace(/~/g, '~0').replace(/\//g, '~1')}`).join('');
}

function pointerSegments(pointer: string): string[] | null {
  if (pointer === '') return [];
  if (!pointer.startsWith('/')) return null;
  return pointer.split('/').slice(1).map((part) => part.replace(/~1/g, '/').replace(/~0/g, '~'));
}

function member(value: unknown, key: string): unknown {
  return value != null && typeof value === 'object' ? (value as Record<string, unknown>)[key] : undefined;
}

function elementId(value: unknown): string | null {
  const id = member(value, 'id');
  return typeof id === 'string' && id !== '' ? id : null;
}

/** The id of `list[index]` when it names that element alone (a unique, non-empty `id`). */
function uniqueId(list: unknown[], index: number): string | null {
  const id = elementId(list[index]);
  if (id == null) return null;
  return list.filter((item) => elementId(item) === id).length === 1 ? id : null;
}

/**
 * The position form of a pointer in `root` (`/zones/0/surfaces/2`): `@id`
 * segments become indexes. Null when an array element it names is not there
 * (a removed element, an index out of range): a dangling link.
 */
export function resolvePointer(root: unknown, pointer: string): string | null {
  const segments = pointerSegments(pointer);
  if (segments == null) return null;
  const resolved: Array<string | number> = [];
  let value: unknown = root;
  for (const segment of segments) {
    if (Array.isArray(value)) {
      let index: number;
      if (segment.startsWith(ID_SEGMENT)) {
        const id = segment.slice(ID_SEGMENT.length);
        const found = value.flatMap((item, position) => (elementId(item) === id ? [position] : []));
        if (found.length !== 1) return null;
        index = found[0];
      } else if (/^\d+$/.test(segment)) {
        index = Number(segment);
        if (index >= value.length) return null;
      } else {
        return null;
      }
      resolved.push(index);
      value = value[index];
    } else {
      if (segment.startsWith(ID_SEGMENT)) return null;
      resolved.push(segment);
      value = member(value, segment);
    }
  }
  return jsonPointer(resolved);
}

/**
 * The stable form of a pointer in `root`: every array element with a unique
 * id is named by `@id`, others keep their index. A pointer that does not
 * resolve is returned unchanged. Applying it twice gives the same pointer.
 */
export function stablePointer(root: unknown, pointer: string): string {
  const resolved = resolvePointer(root, pointer);
  const segments = resolved == null ? null : pointerSegments(resolved);
  if (segments == null) return pointer;
  const stable: string[] = [];
  let value: unknown = root;
  for (const segment of segments) {
    if (Array.isArray(value)) {
      const index = Number(segment);
      const id = uniqueId(value, index);
      stable.push(id != null ? `${ID_SEGMENT}${id}` : segment);
      value = value[index];
    } else {
      stable.push(segment);
      value = member(value, segment);
    }
  }
  return jsonPointer(stable);
}

/** Whether two pointers name the same place in `root` (equal text when they do not resolve). */
export function samePlace(root: unknown, a: string, b: string): boolean {
  if (a === b) return true;
  const left = resolvePointer(root, a);
  return left != null && left === resolvePointer(root, b);
}

/** The evidence ids in a reference text (`evidence:ev-1`, also several separated by commas). */
export function evidenceIdsIn(value: unknown): string[] {
  if (typeof value !== 'string') return [];
  return value.split(/[,;]\s*/).map((part) => part.trim())
    .filter((part) => part.startsWith(EVIDENCE_REFERENCE_PREFIX))
    .map((part) => part.slice(EVIDENCE_REFERENCE_PREFIX.length))
    .filter(Boolean);
}

/** The reference text for an evidence id. */
export function evidenceReference(id: string): string {
  return `${EVIDENCE_REFERENCE_PREFIX}${id}`;
}

/**
 * A source text with the evidence id added: an empty text becomes the reference, a text that
 * already names the id stays as it is, and any other text keeps its description with the
 * reference after it (`tekening A-101; evidence:ev-2`), so attaching a file does not erase it.
 */
export function withEvidenceReference(value: unknown, id: string): string {
  const text = typeof value === 'string' ? value.trim() : '';
  if (!text) return evidenceReference(id);
  if (evidenceIdsIn(text).includes(id)) return text;
  return `${text}; ${evidenceReference(id)}`;
}

/** Every `…Reference` in the project that names an evidence id, as [pointer, id]. */
export function referencesInProject(project: IProject): Array<{ pointer: string; id: string }> {
  const found: Array<{ pointer: string; id: string }> = [];
  const visit = (value: unknown, segments: Array<string | number>) => {
    const last = segments[segments.length - 1];
    if (typeof value === 'string') {
      if (typeof last === 'string' && /Reference$/.test(last)) {
        for (const id of evidenceIdsIn(value)) found.push({ pointer: jsonPointer(segments), id });
      }
      return;
    }
    if (Array.isArray(value)) value.forEach((item, index) => visit(item, [...segments, index]));
    else if (value != null && typeof value === 'object') {
      for (const [key, item] of Object.entries(value as Record<string, unknown>)) {
        // The register itself is not input.
        if (segments.length === 1 && segments[0] === 'registration' && key === 'evidence') continue;
        visit(item, [...segments, key]);
      }
    }
  };
  visit(project, []);
  return found;
}

/** Per evidence id: the inputs it supports (references and linked pointers, stable form), sorted and unique. */
export function evidenceUsage(project: IProject): Map<string, string[]> {
  const usage = new Map<string, Set<string>>();
  const add = (id: string, pointer: string) => {
    if (!usage.has(id)) usage.set(id, new Set());
    usage.get(id)!.add(stablePointer(project, pointer));
  };
  for (const item of project.registration?.evidence ?? []) {
    for (const pointer of item.linkedPaths ?? []) add(item.id, pointer);
  }
  for (const { pointer, id } of referencesInProject(project)) add(id, pointer);
  return new Map([...usage].map(([id, pointers]) => [id, [...pointers].sort()]));
}

/** References to evidence ids that are not in the register (a removed file). */
export function danglingEvidenceReferences(project: IProject): Array<{ pointer: string; id: string }> {
  const ids = new Set((project.registration?.evidence ?? []).map((item) => item.id));
  return referencesInProject(project).filter((reference) => !ids.has(reference.id));
}

/** Linked pointers that no longer name an element in the project (the element was removed). */
export function unresolvedEvidenceLinks(project: IProject): Array<{ pointer: string; id: string }> {
  const found: Array<{ pointer: string; id: string }> = [];
  for (const item of project.registration?.evidence ?? []) {
    for (const pointer of item.linkedPaths ?? []) {
      if (resolvePointer(project, pointer) == null) found.push({ pointer, id: item.id });
    }
  }
  return found;
}

/**
 * Project load: index-based links of older projects (`/zones/0/surfaces/2`)
 * become id-based (`/zones/@woonzone/surfaces/@gevel-noord`). Idempotent; a
 * project without such links is returned as it is.
 */
/** The first `<prefix>-<n>` that no element of `list` uses as its id, for an element being added. */
export function freshItemId(list: unknown[], prefix: string): string {
  const used = new Set(list.map(elementId).filter((id): id is string => id != null));
  let n = list.length + 1;
  while (used.has(`${prefix}-${n}`)) n += 1;
  return `${prefix}-${n}`;
}

/** Gives every element of `list` without an id a fresh one; the same array when all have one. */
function withIds(list: unknown, prefix: string): unknown {
  if (!Array.isArray(list) || list.every((item) => item == null || typeof item !== 'object' || elementId(item) != null)) return list;
  const next: unknown[] = [];
  list.forEach((item, index) => {
    if (item == null || typeof item !== 'object' || elementId(item) != null) {
      next.push(item);
      return;
    }
    // Ids of the elements still to come stay reserved, so a fresh id never takes one of them.
    next.push({ ...(item as Record<string, unknown>), id: freshItemId([...next, ...list.slice(index + 1)], prefix) });
  });
  return next;
}

/**
 * Survey lists that older projects stored without ids (further heating and hot-water generators,
 * further hot-water systems) get one, so links to their items can be id-based (`/…/@id`).
 * The kernel ignores the ids in the calculation. Returns the same project when nothing is added.
 */
export function withSurveyItemIds(project: IProject): IProject {
  const survey = project.basisopname?.survey;
  if (survey == null || typeof survey !== 'object') return project;
  let changed = false;
  const set = (value: Record<string, unknown>, key: string, prefix: string) => {
    const next = withIds(value[key], prefix);
    if (next === value[key]) return value;
    changed = true;
    return { ...value, [key]: next };
  };
  let next: Record<string, unknown> = survey;
  for (const [block, prefix] of [['heating', 'opwekker'], ['hotWater', 'tapwateropwekker']] as const) {
    const value = next[block];
    if (value != null && typeof value === 'object' && !Array.isArray(value)) {
      const updated = set(value as Record<string, unknown>, 'additionalGenerators', prefix);
      if (updated !== value) next = { ...next, [block]: updated };
    }
  }
  next = set(next, 'additionalHotWaterSystems', 'tapwatersysteem');
  const systems = next.additionalHotWaterSystems;
  if (Array.isArray(systems)) {
    const updated = systems.map((system) => (system != null && typeof system === 'object' && !Array.isArray(system)
      ? set(system as Record<string, unknown>, 'additionalGenerators', 'tapwateropwekker') : system));
    if (updated.some((system, index) => system !== systems[index])) next = { ...next, additionalHotWaterSystems: updated };
  }
  return changed ? { ...project, basisopname: { ...project.basisopname!, survey: next } } : project;
}

export function migrateEvidenceLinks(project: IProject): IProject {
  const evidence = project.registration?.evidence;
  if (!Array.isArray(evidence)) return project;
  let changed = false;
  const next = evidence.map((item) => {
    if (item == null || !Array.isArray(item.linkedPaths)) return item;
    const before = item.linkedPaths;
    const linkedPaths = [...new Set(before.map((pointer) =>
      (typeof pointer === 'string' ? stablePointer(project, pointer) : pointer)))];
    if (linkedPaths.length === before.length && linkedPaths.every((pointer, index) => pointer === before[index])) return item;
    changed = true;
    return { ...item, linkedPaths };
  });
  return changed ? { ...project, registration: { ...project.registration, evidence: next } } : project;
}

/**
 * The register after removing element `index` of the array at `list`
 * (position form) from `before`, giving `after`: links to the removed element
 * are dropped (its files stay in the register), and links to elements after
 * it that are kept by position move one place up.
 */
export function linksAfterRemoval(
  evidence: NtaEvidenceItem[], before: unknown, after: unknown, list: string, index: number,
): NtaEvidenceItem[] {
  const removed = `${list}/${index}`;
  return evidence.map((item) => {
    if (!item.linkedPaths?.length) return item;
    let changed = false;
    const linkedPaths: string[] = [];
    for (const pointer of item.linkedPaths) {
      const resolved = resolvePointer(before, pointer);
      if (resolved === removed || resolved?.startsWith(`${removed}/`)) {
        changed = true;
        continue;
      }
      const rest = resolved?.startsWith(`${list}/`) ? resolved.slice(list.length + 1) : null;
      const position = rest == null ? NaN : Number(rest.split('/')[0]);
      if (rest != null && Number.isInteger(position) && position > index) {
        const moved = stablePointer(after, `${list}/${position - 1}${rest.slice(String(position).length)}`);
        if (moved !== pointer) changed = true;
        linkedPaths.push(moved);
      } else {
        linkedPaths.push(pointer);
      }
    }
    if (!changed) return item;
    if (linkedPaths.length > 0) return { ...item, linkedPaths };
    const { linkedPaths: _removed, ...rest } = item;
    return rest;
  });
}

/** The items linked to a pointer (by `linkedPaths`; with `root`, any pointer naming the same place). */
export function evidenceForPointer(evidence: NtaEvidenceItem[], pointer: string, root?: unknown): NtaEvidenceItem[] {
  return evidence.filter((item) => (item.linkedPaths ?? []).some((path) =>
    (root === undefined ? path === pointer : samePlace(root, path, pointer))));
}

/** Adds a pointer to an item's `linkedPaths`. */
export function linkEvidence(evidence: NtaEvidenceItem[], id: string, pointer: string): NtaEvidenceItem[] {
  return evidence.map((item) => item.id !== id || (item.linkedPaths ?? []).includes(pointer)
    ? item : { ...item, linkedPaths: [...(item.linkedPaths ?? []), pointer] });
}

/** Removes a pointer from an item's `linkedPaths` (with `root`, every pointer naming the same place). */
export function unlinkEvidence(evidence: NtaEvidenceItem[], id: string, pointer: string, root?: unknown): NtaEvidenceItem[] {
  const matches = (path: string) => (root === undefined ? path === pointer : samePlace(root, path, pointer));
  return evidence.map((item) => {
    if (item.id !== id || !(item.linkedPaths ?? []).some(matches)) return item;
    const linkedPaths = (item.linkedPaths ?? []).filter((path) => !matches(path));
    if (linkedPaths.length > 0) return { ...item, linkedPaths };
    const { linkedPaths: _removed, ...rest } = item;
    return rest;
  });
}

/** Whether a file is an image (shown as a thumbnail; a survey photo). */
export function isImageFile(fileName: string): boolean {
  return /\.(jpe?g|png|webp|gif|heic)$/i.test(fileName);
}

/**
 * The project path of a pointer in kernel notation, for "Ga naar"
 * (`/zones/0/surfaces/2` → `zones[0].surfaces[2]`). Survey pointers drop
 * the `survey` member: the survey routes are `basisopname.<section>`. With
 * `root`, an id-based pointer is first turned into positions; without it an
 * `@id` segment reads as `[id]` (`zones[woonzone].surfaces[gevel-noord]`).
 */
export function pointerToPath(pointer: string, root?: unknown): string {
  const source = root === undefined ? pointer : resolvePointer(root, pointer) ?? pointer;
  const segments: Array<string | number> = (pointerSegments(source) ?? [])
    .map((part) => (/^\d+$/.test(part) ? Number(part) : part));
  if (segments[0] === 'basisopname' && segments[1] === 'survey') segments.splice(1, 1);
  return segments
    .map((segment, index) => (typeof segment === 'number' ? `[${segment}]`
      : index > 0 && segment.startsWith(ID_SEGMENT) ? `[${segment.slice(ID_SEGMENT.length)}]`
        : `${index === 0 ? '' : '.'}${segment}`))
    .join('');
}
