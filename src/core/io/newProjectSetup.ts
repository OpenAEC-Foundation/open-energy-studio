/**
 * Setting up a new project in one window (feedback 9 Oct 2026: "project
 * opzetten verbeteren", "aanklikken wat relevant is", woonboot as its own
 * choice): kind, address from the BAG, dwelling type and what the building
 * has. The answers fill the project and mark the survey questions they settle
 * as done, so the question flow starts at what is still open.
 */
import type { IProject } from '../energy/types';
import type { NtaRegistration } from '../nta/KernelClient';
import { heatingGeneratorTemplate, surveyTemplate } from '../nta/SurveyTemplates';
import { markProgress, questionKey, stepState, surveySteps, type SurveyProgress } from '../survey/surveyFlow';
import type { BagAddress } from './bagLookup';

export type NewProjectKind = 'residential' | 'utility' | 'existing_residential' | 'existing_utility';

/**
 * Dwelling types of the setup. Caravan and houseboat are their own choice, the
 * houseboat split by berth before or from 1 January 2018 (NTA tables I.5–I.7),
 * as in the new-calculation menu of Uniec3.
 */
export const SETUP_DWELLINGS = ['terraced', 'end_or_corner', 'detached', 'apartment', 'caravan', 'houseboat', 'houseboat_2018'] as const;
export type SetupDwelling = typeof SETUP_DWELLINGS[number];

export const SETUP_HEATING = ['boiler', 'heat_pump', 'district_heat', 'electric'] as const;
export const SETUP_VENTILATION = ['natural', 'mechanical_extract', 'balanced'] as const;

export interface SetupAdviser {
  name: string;
  competenceNumber: string;
  certificateNumber: string;
}

export interface NewProjectSetup {
  kind: NewProjectKind;
  /** From the BAG, or typed in (then without a BAG id). */
  address: BagAddress | null;
  dwelling?: SetupDwelling | null;
  roofType?: 'pitched' | 'partly_flat' | 'flat';
  apartmentFloor?: 'ground_or_intermediate' | 'top' | 'roof_and_floor';
  heating?: string | null;
  ventilation?: string | null;
  /** false: not present, the question is settled; true or null: asked in the flow. */
  cooling?: boolean | null;
  pv?: boolean | null;
  adviser?: SetupAdviser | null;
}

export const isSurveyKind = (kind: NewProjectKind) => kind === 'existing_residential' || kind === 'existing_utility';

/** The project name from an address: "Goejanverwelledijk 85a Gouda". */
export function nameFromAddress(address: BagAddress): string {
  const number = `${address.houseNumber}${address.addition ? (/^[a-z]$/i.test(address.addition) ? address.addition : `-${address.addition}`) : ''}`;
  return [address.street, number, address.city].filter(Boolean).join(' ').trim();
}

function dwellingAnswer(setup: NewProjectSetup): Record<string, unknown> | null {
  const roofType = setup.roofType ?? 'pitched';
  switch (setup.dwelling) {
    case 'apartment': return { kind: 'apartment', floor: setup.apartmentFloor ?? 'ground_or_intermediate', side: 'middle' };
    case 'houseboat': case 'houseboat_2018': case 'caravan': return { kind: 'single_family', position: 'detached', roofType };
    case 'terraced': case 'end_or_corner': case 'detached': return { kind: 'single_family', position: setup.dwelling, roofType };
    default: return null;
  }
}

function buildingKindAnswer(setup: NewProjectSetup): Record<string, unknown> | null {
  if (setup.dwelling === 'houseboat' || setup.dwelling === 'houseboat_2018') return { kind: 'floating', newBerthSince2018: setup.dwelling === 'houseboat_2018' };
  if (setup.dwelling === 'caravan') return { kind: 'caravan' };
  return null;
}

/** The new project with the setup answers in it. */
export function applySetup(base: IProject, setup: NewProjectSetup): IProject {
  const address = setup.address;
  const registration: NtaRegistration = { ...((base.registration ?? {}) as NtaRegistration) };
  if (address) {
    registration.postcode = address.postcode || undefined;
    registration.houseNumber = address.houseNumber || undefined;
    registration.houseNumberAddition = address.addition || undefined;
    registration.bagObjectId = address.bagObjectId || undefined;
  }
  if (setup.adviser?.name) {
    const adviser = { name: setup.adviser.name, competenceNumber: setup.adviser.competenceNumber };
    registration.surveyingAdvisor = adviser;
    registration.registeringAdvisor = { ...adviser };
    if (setup.adviser.certificateNumber) registration.certificateNumber = setup.adviser.certificateNumber;
  }
  const project: IProject = {
    ...base,
    ...(address ? { name: nameFromAddress(address) || base.name, address: address.street, city: address.city } : {}),
    registration,
  };
  if (!isSurveyKind(setup.kind)) return project;

  const surveyKind = setup.kind === 'existing_utility' ? 'utility' : 'residential';
  const template = surveyTemplate(surveyKind);
  const survey = template.survey as Record<string, unknown>;
  let progress: SurveyProgress = {};
  const done = (step: Parameters<typeof questionKey>[0], question: string) => { progress = markProgress(progress, questionKey(step, question), 'done'); };
  const first = surveyKind === 'utility' ? 'gebouw' : 'woning';

  if (address?.street) done(first, 'adres');
  // BAG values are a start: the general question stays open so the adviser checks them.
  if (address?.constructionYear) survey.constructionYear = address.constructionYear;
  if (surveyKind === 'residential' && address?.floorAreaM2) {
    survey.usableFloorAreaM2 = address.floorAreaM2;
    survey.areaSourceReference = 'BAG gebruiksoppervlakte (PDOK); controleren bij de opname';
  }
  if (surveyKind === 'residential') {
    const dwelling = dwellingAnswer(setup);
    if (dwelling) { survey.dwelling = dwelling; done('woning', 'soort'); }
    const buildingKind = buildingKindAnswer(setup);
    if (buildingKind) survey.envelope = { ...(survey.envelope as Record<string, unknown>), buildingKind };
  }
  if (setup.heating) {
    survey.heating = { ...(survey.heating as Record<string, unknown>), generator: heatingGeneratorTemplate(setup.heating) };
    done('verwarming', 'toestel');
  }
  if (setup.ventilation) {
    survey.ventilation = { ...(survey.ventilation as Record<string, unknown>), principle: setup.ventilation };
    done('ventilatie', 'systeem');
  }
  if (setup.cooling === false) done('koeling', 'koeling');
  if (setup.pv === false) done('zonnepanelen', 'pv');
  return { ...project, basisopname: { ...template, progress } } as IProject;
}

/** The first survey step with something still to answer, to open the flow at. */
export function firstOpenSurveyStep(project: IProject): string | null {
  const stored = project.basisopname as { kind: 'residential' | 'utility'; progress?: SurveyProgress } | undefined;
  if (!stored) return null;
  const steps = surveySteps(stored.kind);
  return (steps.find((step) => !step.special && stepState(step, stored.progress) !== 'done') ?? steps[0]).id;
}

const ADVISER_KEY = 'oes.adviser.v1';

/** The adviser of the previous project on this computer. */
export function rememberedAdviser(store: Pick<Storage, 'getItem'> | null = safeStorage()): SetupAdviser | null {
  try {
    const value = JSON.parse(store?.getItem(ADVISER_KEY) ?? 'null') as Partial<SetupAdviser> | null;
    return value && typeof value.name === 'string'
      ? { name: value.name, competenceNumber: value.competenceNumber ?? '', certificateNumber: value.certificateNumber ?? '' }
      : null;
  } catch {
    return null;
  }
}

export function rememberAdviser(adviser: SetupAdviser, store: Pick<Storage, 'setItem'> | null = safeStorage()): void {
  try { store?.setItem(ADVISER_KEY, JSON.stringify(adviser)); } catch { /* private mode: not remembered */ }
}

function safeStorage(): Storage | null {
  try { return typeof localStorage === 'undefined' ? null : localStorage; } catch { return null; }
}
