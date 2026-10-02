import type { NtaRegistration } from './KernelClient';

/** Drops empty values so an untouched registration block is not saved. */
export function cleanRegistration(registration: NtaRegistration): NtaRegistration | undefined {
  const result: Record<string, unknown> = {};
  for (const [key, raw] of Object.entries(registration)) {
    let value: unknown = typeof raw === 'string' ? raw.trim() : raw;
    if (typeof value === 'object' && value !== null) {
      const advisor = value as { name: string; competenceNumber: string };
      value = { name: advisor.name.trim(), competenceNumber: advisor.competenceNumber.trim() };
      if (!advisor.name.trim() && !advisor.competenceNumber.trim()) continue;
    }
    if (value === undefined || value === '' || value === false) continue;
    result[key] = value;
  }
  return Object.keys(result).length ? result as NtaRegistration : undefined;
}
