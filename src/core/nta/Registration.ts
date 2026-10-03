import { version } from '../../../package.json';
import type { NtaRegistration, NtaSoftwareIdentity } from './KernelClient';

/** Name of this program in the registration (Regeling art. 5 lid 1 onder b, p. 6). */
export const SOFTWARE_NAME = 'Open Energy Studio';

/**
 * BRL 9501 attest number of this program. Empty until the program is
 * attested; the kernel then reports `softwareAttested: false`, apart from
 * the dossier issues.
 */
export const SOFTWARE_ATTEST_NUMBER = '';

/** The program that makes the calculation, stored with the registration block. */
export function softwareIdentity(): NtaSoftwareIdentity {
  return SOFTWARE_ATTEST_NUMBER
    ? { name: SOFTWARE_NAME, version, attestNumber: SOFTWARE_ATTEST_NUMBER }
    : { name: SOFTWARE_NAME, version };
}

function cleanObject(value: object): Record<string, unknown> | undefined {
  const entries = Object.entries(value)
    .map(([key, item]) => [key, typeof item === 'string' ? item.trim() : item] as const)
    .filter(([, item]) => item !== undefined && item !== null && item !== '' && item !== false);
  return entries.length ? Object.fromEntries(entries) : undefined;
}

/** Drops empty values so an untouched registration block is not saved. */
export function cleanRegistration(registration: NtaRegistration): NtaRegistration | undefined {
  const result: Record<string, unknown> = {};
  for (const [key, raw] of Object.entries(registration)) {
    let value: unknown = typeof raw === 'string' ? raw.trim() : raw;
    if (key === 'software') continue;
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
  // Regeling art. 5 lid 1 onder b: the registration records the program used.
  // A relabel or a replacement keeps the program of the original
  // calculation (BRL 9500-W §4.2.4, p. 23–24).
  const messageType = registration.messageType ?? (registration.relabel ? 'relabel' : 'regular');
  result.software = messageType !== 'regular' && registration.software
    ? registration.software
    : softwareIdentity();
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
