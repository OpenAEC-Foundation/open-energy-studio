/**
 * Registratie of a basisopname project (feedback 8 Oct 2026). What the survey
 * already knows is taken over into the registration (purpose, survey type,
 * construction year, survey date, the registering adviser) instead of being
 * asked twice, and a checklist says what EP-Online and BRL 9500 still need.
 *
 * The kernel's registration check runs inside the project performance, which
 * a survey project does not have; the list below asks for the same required
 * fields as `assess_registration_with` (crates/nta8800-core/src/registration.rs).
 */
import type { IProject } from '../energy/types';
import type { NtaRegistration, OpnameAssessment } from '../nta/KernelClient';
import { softwareIdentity } from '../nta/Registration';
import type { StoredSurvey } from '../nta/SurveyTemplates';

type Stored = StoredSurvey & { surveyDate?: string };

/** House number and addition at the end of "Straat 85a" or "Straat 85-2". */
export function parseHouseNumber(address: string | undefined): { houseNumber?: string; houseNumberAddition?: string } {
  const match = /\s(\d+)\s*[-\s]?\s*([A-Za-z0-9]{0,4})\s*$/.exec(address ?? '');
  if (!match) return {};
  return { houseNumber: match[1], ...(match[2] ? { houseNumberAddition: match[2] } : {}) };
}

/** The registration fields the survey answers itself. */
export function surveyTakeovers(project: IProject): Partial<NtaRegistration> {
  const stored = project.basisopname as Stored | undefined;
  const year = (stored?.survey as { constructionYear?: unknown } | undefined)?.constructionYear;
  const registration = project.registration ?? {};
  const surveying = registration.surveyingAdvisor;
  return {
    purpose: 'existing_building',
    surveyType: 'basic',
    ...(typeof year === 'number' ? { constructionYear: year } : {}),
    ...(stored?.surveyDate ? { surveyDate: stored.surveyDate } : {}),
    // The surveying adviser registers, unless another one is named.
    ...(!registration.registeringAdvisor?.name && surveying?.name ? { registeringAdvisor: { ...surveying } } : {}),
  };
}

/** The registration with the survey's answers taken over. */
export function surveyRegistration(project: IProject): NtaRegistration {
  const registration = project.registration ?? {};
  const parsed = parseHouseNumber(project.address);
  return {
    ...registration,
    ...surveyTakeovers(project),
    ...(registration.houseNumber ? {} : parsed),
  };
}

/** True when the stored registration lacks or differs from a takeover. */
export function takeoversPending(project: IProject): boolean {
  const registration = project.registration ?? {};
  return Object.entries(surveyTakeovers(project)).some(([key, value]) =>
    JSON.stringify(registration[key as keyof NtaRegistration]) !== JSON.stringify(value));
}

export type ChecklistId = 'label' | 'address' | 'bag' | 'adviser' | 'certificate' | 'surveyDate'
  | 'client' | 'registrationDate' | 'reasons' | 'evidence' | 'attest';

export interface ChecklistItem {
  id: ChecklistId;
  state: 'done' | 'open' | 'waiting';
  /** Values for the label text (count, class, date). */
  values?: Record<string, string | number>;
}

const filled = (value: string | undefined | null) => Boolean(value && value.trim());

/** Applied defaults without a reason (BRL 9500-W §4.2.2); reasons are kept by path or by rule. */
export function defaultsWithoutReason(stored: StoredSurvey, result: OpnameAssessment | null): number {
  const reasons = ((stored.survey as { inklapRedenen?: Record<string, string> }).inklapRedenen) ?? {};
  return (result?.appliedDefaults ?? []).filter((item) =>
    !filled(item.inklapReden ?? reasons[item.path] ?? reasons[item.rule])).length;
}

/** What still stands between this survey and its registration. */
export function registrationChecklist(project: IProject, result: OpnameAssessment | null): ChecklistItem[] {
  const stored = project.basisopname as Stored | undefined;
  const registration = surveyRegistration(project);
  const item = (id: ChecklistId, done: boolean, values?: ChecklistItem['values']): ChecklistItem =>
    ({ id, state: done ? 'done' : 'open', ...(values ? { values } : {}) });
  const label = result?.performance?.indicativeLabelClass ?? null;
  const missingReasons = stored ? defaultsWithoutReason(stored, result) : 0;
  const defaults = result?.appliedDefaults.length ?? 0;
  const evidence = registration.evidence?.length ?? 0;
  const attest = softwareIdentity().attestNumber;
  const advisor = registration.surveyingAdvisor;
  return [
    item('label', label != null, label ? { label } : undefined),
    item('address', filled(project.address) && filled(registration.postcode) && filled(project.city)),
    item('bag', /^\d{16}$/.test(registration.bagObjectId?.trim() ?? '')),
    item('adviser', filled(advisor?.name) && filled(advisor?.competenceNumber)),
    item('certificate', filled(registration.certificateNumber)),
    item('surveyDate', filled(registration.surveyDate)),
    item('client', filled(registration.client)),
    item('registrationDate', filled(registration.registrationDate)),
    item('reasons', result != null && missingReasons === 0, { count: missingReasons, total: defaults }),
    item('evidence', evidence > 0, { count: evidence }),
    { id: 'attest', state: attest ? 'done' : 'waiting' },
  ];
}

/** Ten years after the survey date (Omgevingsregeling; kernel `valid_until`). */
export function validUntil(surveyDate: string | undefined): string | null {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(surveyDate ?? '');
  return match ? `${Number(match[1]) + 10}-${match[2]}-${match[3]}` : null;
}
