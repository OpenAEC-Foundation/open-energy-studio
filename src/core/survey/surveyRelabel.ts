/**
 * Herlabelen of a basisopname project (feedback 8 Oct 2026). The kernel
 * compares the survey answers of the original label with the current ones
 * (`survey_label_input` in crates/nta8800-core/src/relabel.rs) and classifies
 * every difference per BRL 9500 Bijlage 6a/6b; the app takes the changes of
 * one surface, window or installation together, so a measure is one row.
 */
import type { NtaEvidenceItem, NtaRegistration, RelabelChange } from '../nta/KernelClient';

export const SURVEY_POINTER = '/basisopname/survey';

type Verdict = RelabelChange['verdict'];
const RANK: Record<Verdict, number> = { allowed: 0, review: 1, not_allowed: 2 };
export const worst = (verdicts: Verdict[]): Verdict =>
  verdicts.reduce<Verdict>((top, verdict) => (RANK[verdict] > RANK[top] ? verdict : top), 'allowed');

const ITEM_LISTS = ['surfaces', 'windows', 'doors', 'rooflights'];

export interface RelabelLine {
  /** Field of the item that changed ("insulation", "glass", "generator"); empty when the item itself was added or removed. */
  field: string;
  /** Survey path of that field (dotted, `envelope.surfaces[0].insulation`). */
  path: string;
  verdict: Verdict;
  changes: RelabelChange[];
}

export interface RelabelRow {
  /** Survey path of the item or section (`envelope.surfaces[0]`, `heating`). */
  path: string;
  /** JSON pointer of the item in the project, for evidence links. */
  pointer: string;
  verdict: Verdict;
  lines: RelabelLine[];
  /** Building-bound production (PV): needs a photo with shading too. */
  production: boolean;
}

const dotted = (segments: string[]) => segments
  .map((segment, index) => (/^\d+$/.test(segment) ? `[${segment}]` : `${index ? '.' : ''}${segment}`)).join('');

/** Survey segments of a change pointer, or null outside the survey. */
export function surveySegments(pointer: string): string[] | null {
  if (!pointer.startsWith(`${SURVEY_POINTER}/`)) return null;
  return pointer.slice(SURVEY_POINTER.length + 1).split('/').map((part) => part.replace(/~1/g, '/').replace(/~0/g, '~'));
}

/** The survey changes taken together per item (surface, window, PV system) or section (heating, ventilation). */
export function relabelRows(changes: RelabelChange[]): RelabelRow[] {
  const rows = new Map<string, RelabelRow>();
  for (const change of changes) {
    const segments = surveySegments(change.path);
    if (!segments) continue;
    let item: string[];
    if (segments[0] === 'envelope' && ITEM_LISTS.includes(segments[1]) && segments[2] != null) item = segments.slice(0, 3);
    else if (segments[0] === 'pv' && segments[1] != null) item = segments.slice(0, 2);
    else item = segments.slice(0, 1);
    const field = segments[item.length] ?? '';
    const key = dotted(item);
    const row = rows.get(key) ?? {
      path: key, pointer: `${SURVEY_POINTER}/${item.join('/')}`, verdict: 'allowed' as Verdict, lines: [],
      production: item[0] === 'pv',
    };
    const fieldPath = field ? `${key}.${field}` : key;
    const line = row.lines.find((candidate) => candidate.path === fieldPath);
    if (line) { line.changes.push(change); line.verdict = worst([line.verdict, change.verdict]); }
    else row.lines.push({ field, path: fieldPath, verdict: change.verdict, changes: [change] });
    row.verdict = worst([row.verdict, change.verdict]);
    rows.set(key, row);
  }
  return [...rows.values()];
}

/** Evidence linked to a row, by role. */
export function rowProofs(row: RelabelRow, evidence: NtaEvidenceItem[] | undefined) {
  const linked = (evidence ?? []).filter((item) => item.linkedPaths?.some((path) => path === row.pointer || path.startsWith(`${row.pointer}/`)));
  const count = (role: NtaEvidenceItem['relabelProof']) => linked.filter((item) => item.relabelProof === role).length;
  const proof = count('quote_with_order') + count('specified_invoice');
  const photo = count('production_photo');
  // BRL 9500-W §4.2.3 (p. 23): an order with a quote or a specified invoice; PV also a photo with shading.
  return { linked, proof, photo, complete: proof > 0 && (!row.production || photo > 0) };
}

/** The original label's survey date: the registration's, else the survey's own. */
export function originalSurveyDate(original: { registration?: NtaRegistration; basisopname?: unknown }): string | undefined {
  return original.registration?.surveyDate ?? (original.basisopname as { surveyDate?: string } | undefined)?.surveyDate;
}

/** The relabel fields of the registration that the original label answers (BRL 9500-W §4.2.3–4.2.4). */
export function relabelTakeovers(original: { registration?: NtaRegistration; basisopname?: unknown }, fileName: string): Partial<NtaRegistration> {
  const from = original.registration ?? {};
  const surveyDate = originalSurveyDate(original);
  return {
    messageType: 'relabel',
    relabel: undefined,
    ...(surveyDate ? { surveyDate } : {}),
    ...(from.certificateNumber ? { originalCertificateNumber: from.certificateNumber } : {}),
    ...(from.epOnlineNumber ? { originalEpOnlineNumber: from.epOnlineNumber } : {}),
    ...(from.software?.kernelVersion ? { originalKernelVersion: from.software.kernelVersion } : {}),
    originalDossierReference: fileName,
  };
}
