import type { NtaEvidenceItem } from './KernelClient';
import type { IProject } from '../energy/types';
import { EVIDENCE_REFERENCE_PREFIX } from './Evidence';

/**
 * Links between the evidence register (BRL 9500 Bijlage 3) and the project.
 * An input refers to a file in two ways: a `…Reference` text holding
 * `evidence:<id>`, or a JSON pointer in the item's `linkedPaths` (an element,
 * a survey item). Both count as "onderbouwt" for the register and the dossier.
 */

/** JSON pointer (RFC 6901) of path segments. */
export function jsonPointer(segments: Array<string | number>): string {
  return segments.map((segment) => `/${String(segment).replace(/~/g, '~0').replace(/\//g, '~1')}`).join('');
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

/** Per evidence id: the inputs it supports (references and linked pointers), sorted and unique. */
export function evidenceUsage(project: IProject): Map<string, string[]> {
  const usage = new Map<string, Set<string>>();
  const add = (id: string, pointer: string) => {
    if (!usage.has(id)) usage.set(id, new Set());
    usage.get(id)!.add(pointer);
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

/** The items linked to a pointer (by `linkedPaths`). */
export function evidenceForPointer(evidence: NtaEvidenceItem[], pointer: string): NtaEvidenceItem[] {
  return evidence.filter((item) => (item.linkedPaths ?? []).includes(pointer));
}

/** Adds a pointer to an item's `linkedPaths`. */
export function linkEvidence(evidence: NtaEvidenceItem[], id: string, pointer: string): NtaEvidenceItem[] {
  return evidence.map((item) => item.id !== id || (item.linkedPaths ?? []).includes(pointer)
    ? item : { ...item, linkedPaths: [...(item.linkedPaths ?? []), pointer] });
}

/** Removes a pointer from an item's `linkedPaths`. */
export function unlinkEvidence(evidence: NtaEvidenceItem[], id: string, pointer: string): NtaEvidenceItem[] {
  return evidence.map((item) => {
    if (item.id !== id || !(item.linkedPaths ?? []).includes(pointer)) return item;
    const linkedPaths = (item.linkedPaths ?? []).filter((path) => path !== pointer);
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
 * the `survey` member: the survey routes are `basisopname.<section>`.
 */
export function pointerToPath(pointer: string): string {
  const segments: Array<string | number> = pointer.split('/').slice(1)
    .map((part) => part.replace(/~1/g, '/').replace(/~0/g, '~'))
    .map((part) => (/^\d+$/.test(part) ? Number(part) : part));
  if (segments[0] === 'basisopname' && segments[1] === 'survey') segments.splice(1, 1);
  return segments
    .map((segment, index) => (typeof segment === 'number' ? `[${segment}]` : `${index === 0 ? '' : '.'}${segment}`))
    .join('');
}
