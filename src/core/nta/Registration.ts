import { version } from '../../../package.json';
import attest from './attest.json';
import type { NtaRegistration, NtaSoftwareIdentity } from './KernelClient';
import { sha256Hex } from './Evidence';

/**
 * Name of this program in the registration (Omgevingsregeling art. 5.14 lid 1
 * onder b). It, the attest number and the identification code come from
 * `attest.json`, which the leveringsdocument of a release reads too
 * (scripts/nta-leveringsdocument.mjs, BRL 9501 §6.1).
 */
export const SOFTWARE_NAME: string = attest.softwareName;

/**
 * BRL 9501 attest number of this program. Empty until the program is
 * attested; the kernel then reports `softwareAttested: false`, apart from
 * the dossier issues.
 */
export const SOFTWARE_ATTEST_NUMBER: string = attest.attestNumber;

/**
 * The program that makes the calculation, stored with the registration block.
 * `kernelVersion` is the calculation core of the kernel stamp; it is left out
 * when the kernel is unavailable.
 */
export function softwareIdentity(kernelVersion?: string): NtaSoftwareIdentity {
  return {
    name: SOFTWARE_NAME,
    version,
    ...(SOFTWARE_ATTEST_NUMBER ? { attestNumber: SOFTWARE_ATTEST_NUMBER } : {}),
    ...(kernelVersion ? { kernelVersion } : {}),
  };
}

/**
 * The program identity to store: a relabel keeps the stored identity and,
 * when that has no calculation core, names the stated original core (BRL
 * 9500-W §4.2.4, p. 24); otherwise this program with the current core.
 */
export function registrationSoftware(registration: NtaRegistration, kernelVersion?: string): NtaSoftwareIdentity {
  if (keepsSoftwareIdentity(registration) && registration.software) {
    const kept = registration.software;
    const original = registration.originalKernelVersion?.trim();
    return kept.kernelVersion || !original ? kept : { ...kept, kernelVersion: original };
  }
  return softwareIdentity(kernelVersion);
}

function cleanObject(value: object): Record<string, unknown> | undefined {
  const entries = Object.entries(value)
    .map(([key, item]) => [key, typeof item === 'string' ? item.trim() : item] as const)
    .filter(([, item]) => item !== undefined && item !== null && item !== '' && item !== false);
  return entries.length ? Object.fromEntries(entries) : undefined;
}

/** Only a relabel keeps the stored program identity (BRL 9500-W §4.2.4, p. 24). */
export function keepsSoftwareIdentity(registration: NtaRegistration): boolean {
  return (registration.messageType ?? (registration.relabel ? 'relabel' : 'regular')) === 'relabel';
}

/** Drops empty values so an untouched registration block is not saved. */
export function cleanRegistration(registration: NtaRegistration, kernelVersion?: string): NtaRegistration | undefined {
  const result: Record<string, unknown> = {};
  for (const [key, raw] of Object.entries(registration)) {
    let value: unknown = typeof raw === 'string' ? raw.trim() : raw;
    if (key === 'software') continue;
    if (key === 'relabelComparison') {
      if (value && typeof value === 'object') result[key] = value;
      continue;
    }
    // A stated "no" is an answer the kernel checks (BRL 9500 §4.2.3).
    if ((key === 'productionPhysicallyConnected' || key === 'noExcludedChangesConfirmed') && typeof value === 'boolean') {
      result[key] = value;
      continue;
    }
    if (Array.isArray(value)) {
      if (value.length === 0) continue;
    } else if (key === 'detailSurveyTriggers' && typeof value === 'object' && value !== null) {
      const set = Object.entries(value).filter(([, flag]) => flag === true);
      if (set.length === 0) continue;
      value = Object.fromEntries(set);
    } else if (key === 'wlcGwp' && typeof value === 'object' && value !== null) {
      value = cleanObject(value);
    } else if (typeof value === 'object' && value !== null) {
      const advisor = value as { name: string; competenceNumber: string };
      value = { name: advisor.name.trim(), competenceNumber: advisor.competenceNumber.trim() };
      if (!advisor.name.trim() && !advisor.competenceNumber.trim()) continue;
    }
    if (value === undefined || value === '' || value === false) continue;
    result[key] = value;
  }
  if (!Object.keys(result).length) return undefined;
  // Omgevingsregeling art. 5.14 lid 1 onder b: the registration records the program used.
  // A relabel keeps the program of the original calculation (BRL 9500-W
  // §4.2.4, p. 24); a replacement is a new calculation with the current
  // attested version (p. 23), so it records this program.
  result.software = registrationSoftware(registration, kernelVersion);
  return result as NtaRegistration;
}

/** One registration of a residential label in the local ledger. */
export interface BagLedgerEntry {
  bagObjectId: string;
  projectId: string;
  projectName: string;
  residential: boolean;
  messageType: NonNullable<NtaRegistration['messageType']>;
  epOnlineNumber?: string;
  registrationDate?: string;
  /** Survey date: the label is valid ten years from it (Bep art. 2.1 lid 7). */
  surveyDate?: string;
}

const LEDGER_KEY = 'nta-bag-registrations';

interface LedgerStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

/** Registrations only: a draft or scenario copy without an EP-Online number is not a label. */
function registered(entry: BagLedgerEntry): boolean {
  return Boolean(entry.bagObjectId.trim() && entry.epOnlineNumber?.trim());
}

function storage(): LedgerStorage | undefined {
  return typeof localStorage === 'undefined' ? undefined : localStorage;
}

export function readBagLedger(store: LedgerStorage | undefined = storage()): BagLedgerEntry[] {
  try {
    const parsed = JSON.parse(store?.getItem(LEDGER_KEY) ?? '[]');
    return Array.isArray(parsed) ? parsed as BagLedgerEntry[] : [];
  } catch {
    return [];
  }
}

/**
 * Records (or replaces) the registration of one project in the local ledger.
 * Only a registered label (with an EP-Online number) is kept; otherwise the
 * project's entry is removed, e.g. after the BAG id was cleared.
 */
export function recordBagRegistration(entry: BagLedgerEntry, store: LedgerStorage | undefined = storage()): void {
  if (!store) return;
  const others = readBagLedger(store).filter((item) => item.projectId !== entry.projectId);
  store.setItem(LEDGER_KEY, JSON.stringify(registered(entry) ? [...others, entry] : others));
}

/** Label validity: ten years from the survey date (Bep art. 2.1 lid 7). */
function expired(item: BagLedgerEntry, reference: string): boolean {
  if (!item.surveyDate || !reference) return false;
  const [year, month, day] = item.surveyDate.split('-').map(Number);
  if (!year || !month || !day) return false;
  const end = `${String(year + 10).padStart(4, '0')}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
  return end < reference;
}

/**
 * Praktijkhandboek v2 p. 46: one residential label per addressable BAG
 * object at a time. Returns the other projects in this installation that
 * registered a still valid residential label on the same object. A relabel
 * or a replacement of that label is allowed, so those message types are not
 * reported; an expired label (survey date + 10 years before this survey or
 * registration) is no conflict either.
 */
export function bagConflicts(entry: BagLedgerEntry, ledger: BagLedgerEntry[]): BagLedgerEntry[] {
  if (!entry.residential || entry.messageType !== 'regular') return [];
  const id = entry.bagObjectId.trim();
  if (!id) return [];
  const reference = entry.registrationDate || entry.surveyDate || new Date().toISOString().slice(0, 10);
  return ledger.filter((item) => item.projectId !== entry.projectId
    && item.residential && registered(item) && item.bagObjectId.trim() === id
    && !expired(item, reference));
}

/**
 * A number as `<digits>e<exponent>` with integer digits and no leading or
 * trailing zeros (`0` for zero): the kernel's `canonical_number`, so both
 * notations of one value (`1e-7`, `0.0000001`, `1e+21`) hash alike.
 */
export function canonicalNumber(text: string): string {
  const negative = text.startsWith('-');
  const unsigned = negative ? text.slice(1) : text;
  const at = unsigned.search(/[eE]/);
  const mantissa = at < 0 ? unsigned : unsigned.slice(0, at);
  let exponent = at < 0 ? 0 : Number.parseInt(unsigned.slice(at + 1), 10) || 0;
  const [whole, fraction = ''] = mantissa.split('.');
  exponent -= fraction.length;
  const digits = `${whole}${fraction}`.replace(/^0+/, '');
  if (digits === '') return '0';
  const trimmed = digits.replace(/0+$/, '');
  exponent += digits.length - trimmed.length;
  return `${negative ? '-' : ''}${trimmed}e${exponent}`;
}

const utf8 = new TextEncoder();

function compareUtf8(a: string, b: string): number {
  const x = utf8.encode(a);
  const y = utf8.encode(b);
  for (let index = 0; index < Math.min(x.length, y.length); index += 1) {
    if (x[index] !== y[index]) return x[index] - y[index];
  }
  return x.length - y.length;
}

/**
 * The kernel's canonical label-input text: object members that are null or
 * undefined left out, keys sorted by UTF-8 bytes, numbers in canonical form.
 */
function canonicalJson(value: unknown): string {
  if (value === null || value === undefined) return 'null';
  if (typeof value === 'number') return canonicalNumber(String(value));
  if (typeof value === 'string' || typeof value === 'boolean') return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(',')}]`;
  const entries = Object.entries(value as Record<string, unknown>)
    .filter(([, item]) => item !== null && item !== undefined)
    .sort(([a], [b]) => compareUtf8(a, b));
  return `{${entries.map(([key, item]) => `${JSON.stringify(key)}:${canonicalJson(item)}`).join(',')}}`;
}

/**
 * SHA-256 of the project's label input: everything except the registration,
 * the maatwerkadvies, the basic survey and the import log, which the relabel comparison
 * skips, in the kernel's canonical form (`label_input_hash` in relabel.rs),
 * so it equals the kernel's `currentLabelInputHash`. A stored comparison
 * whose hash differs is out of date.
 */
export async function labelInputSha256(project: object): Promise<string> {
  const rest: Record<string, unknown> = { ...(project as Record<string, unknown>) };
  delete rest.registration;
  delete rest.maatwerkadvies;
  delete rest.basisopname;
  delete rest.importLog;
  return sha256Hex(utf8.encode(canonicalJson(rest)));
}

/** Relabel fields required since 3 October 2026 (BRL 9500-W §4.2.3–4.2.4, p. 23–24). */
export type RelabelMigrationField = 'originalCertificateNumber' | 'originalEpOnlineNumber' | 'relabelComparison' | 'relabelProof';

/**
 * Whether a relabel project was saved before the relabel rules of
 * 3 October 2026: none of the fields those rules added is present (no
 * evidence role, no original certificate or EP-Online number, no stored
 * original project).
 */
function predatesRelabelRules(registration: NtaRegistration): boolean {
  return !(registration.evidence ?? []).some((item) => item.relabelProof !== undefined)
    && registration.originalCertificateNumber === undefined
    && registration.originalEpOnlineNumber === undefined
    && !registration.relabelComparison?.originalProjectText;
}

/**
 * Brings a relabel project saved before the new relabel rules up to date.
 * Only such files are touched: each invoice without a relabel role is
 * marked `review`, which does not count as proof (BRL 9500-W §4.2.3,
 * p. 23) until the adviser picks its role. Returns the number of invoices
 * marked and the fields still to fill in; both are empty for a file that
 * already follows the rules, so running it again changes nothing.
 */
export function migrateLegacyRelabel<T extends { registration?: NtaRegistration }>(project: T): {
  project: T; missing: RelabelMigrationField[]; markedForReview: number;
} {
  const registration = project.registration;
  if (!registration || (registration.messageType ?? (registration.relabel ? 'relabel' : 'regular')) !== 'relabel'
    || !predatesRelabelRules(registration)) {
    return { project, missing: [], markedForReview: 0 };
  }
  let markedForReview = 0;
  const evidence = (registration.evidence ?? []).map((item) => {
    if (item.kind === 'invoice' && !item.relabelProof) {
      markedForReview += 1;
      return { ...item, relabelProof: 'review' as const };
    }
    return item;
  });
  const missing: RelabelMigrationField[] = [];
  if (!registration.originalCertificateNumber?.trim()) missing.push('originalCertificateNumber');
  if (!registration.originalEpOnlineNumber?.trim()) missing.push('originalEpOnlineNumber');
  if (!registration.relabelComparison?.originalProjectText) missing.push('relabelComparison');
  missing.push('relabelProof');
  const migrated = markedForReview > 0 ? { ...project, registration: { ...registration, evidence } } : project;
  return { project: migrated, missing, markedForReview };
}

/**
 * Key for the one-time relabel notice: the project id, else the file path,
 * else a hash of the file text, so projects without an id never share one.
 */
export async function relabelNoticeKey(projectId: string | undefined, filePath: string | undefined, text: string): Promise<string> {
  const id = projectId?.trim() || filePath?.trim() || await sha256Hex(utf8.encode(text));
  return `oes-relabel-migration-notice:${id}`;
}

/**
 * Prepares the original project file for storage with the comparison: a
 * relabel comparison inside it (with its own original file) is left out,
 * so originals do not nest round after round. Other files stay as read.
 */
export function originalProjectTextForStorage(text: string): string {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch {
    return text;
  }
  const root = parsed as Record<string, unknown> | null;
  const project = root && typeof root === 'object'
    ? (root.type === 'open-energy-studio' && root.project && typeof root.project === 'object' ? root.project : root) as Record<string, unknown>
    : null;
  const registration = project?.registration as NtaRegistration | undefined;
  if (!registration?.relabelComparison?.originalProjectText) return text;
  const comparison = { ...registration.relabelComparison };
  delete comparison.originalProjectText;
  project!.registration = { ...registration, relabelComparison: comparison };
  return JSON.stringify(parsed, null, 2);
}

/** Last day an improvement may be counted: 24 months after the survey (BRL 9500-W §4.2.3, p. 23). */
export function relabelDeadline(surveyDate: string | undefined): string | undefined {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(surveyDate ?? '');
  if (!match) return undefined;
  const year = Number(match[1]) + 2;
  const month = Number(match[2]);
  const lastDay = new Date(Date.UTC(year, month, 0)).getUTCDate();
  const day = Math.min(Number(match[3]), lastDay);
  return `${year}-${match[2]}-${String(day).padStart(2, '0')}`;
}
