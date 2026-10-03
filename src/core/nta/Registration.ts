import { version } from '../../../package.json';
import type { NtaRegistration, NtaSoftwareIdentity } from './KernelClient';

/** Name of this program in the registration (Regeling art. 5 lid b, p. 6). */
export const SOFTWARE_NAME = 'Open Energy Studio';

/**
 * BRL 9501 attest number of this program. Empty until the program is
 * attested; the kernel then reports `software_attest_number_missing`.
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
  // Regeling art. 5 lid b: the registration records the program used.
  result.software = softwareIdentity();
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
}

const LEDGER_KEY = 'nta-bag-registrations';

interface LedgerStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
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

/** Records (or replaces) the registration of one project in the local ledger. */
export function recordBagRegistration(entry: BagLedgerEntry, store: LedgerStorage | undefined = storage()): void {
  if (!store) return;
  const others = readBagLedger(store).filter((item) => item.projectId !== entry.projectId);
  store.setItem(LEDGER_KEY, JSON.stringify([...others, entry]));
}

/**
 * Praktijkhandboek v2 p. 46: one residential label per addressable BAG
 * object. Returns the other projects in this installation that registered a
 * residential label on the same object. A relabel or a replacement of that
 * label is allowed, so those message types are not reported.
 */
export function bagConflicts(entry: BagLedgerEntry, ledger: BagLedgerEntry[]): BagLedgerEntry[] {
  if (!entry.residential || entry.messageType !== 'regular') return [];
  const id = entry.bagObjectId.trim();
  if (!id) return [];
  return ledger.filter((item) => item.projectId !== entry.projectId
    && item.residential && item.bagObjectId.trim() === id);
}
