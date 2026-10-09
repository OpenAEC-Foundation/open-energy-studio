/**
 * "Aanklikken wat relevant is" (feedback 9 Oct 2026, for every system where it
 * applies): the optional parts of an installation are asked only when they are
 * present. Each question shows the parts it holds as toggles; a part is shown
 * when it is ticked or already has data, so nothing filled in ever disappears.
 * Ticking a part starts its data (a list with one item, an object with its
 * template); unticking clears it.
 */
import { freshItemId } from '../nta/EvidenceLinks';
import { heatingGeneratorTemplate, hotWaterGeneratorTemplate, solarTemplate, type StoredSurvey } from '../nta/SurveyTemplates';
import type { SurveyPart } from './surveyFlow';

export type SurveyComponentId =
  | 'heatingExtra' | 'heatingCollective'
  | 'hotWaterExtra' | 'solarWater' | 'showerWtw' | 'hotWaterCollective'
  | 'ventControls' | 'ventCombined' | 'ventStrips' | 'passiveCooling';

interface ComponentDef {
  id: SurveyComponentId;
  part: SurveyPart;
  /** Only in a dwelling survey (the utility survey asks these elsewhere or not at all). */
  residentialOnly?: boolean;
  path: string[];
  /** Whether the value at the path counts as data of the part. */
  present: (value: unknown) => boolean;
  /** The value when ticked without data; undefined: ticking only shows the fields. */
  start?: () => unknown;
  /** The value when unticked. */
  off: unknown;
}

const listed = (value: unknown) => Array.isArray(value) && value.length > 0;
const set = (value: unknown) => value != null;

const COMPONENTS: ComponentDef[] = [
  { id: 'heatingExtra', part: 'heatingGenerator', path: ['heating', 'additionalGenerators'], present: listed, off: [],
    start: () => [{ id: freshItemId([], 'opwekker'), generator: heatingGeneratorTemplate('boiler'), nominalPowerKw: 20 }] },
  { id: 'heatingCollective', part: 'heatingRest', residentialOnly: true, path: ['heating', 'collective'], present: set, off: null, start: () => ({}) },
  { id: 'hotWaterExtra', part: 'hotWaterGenerator', path: ['hotWater', 'additionalGenerators'], present: listed, off: [],
    start: () => [{ id: freshItemId([], 'tapwateropwekker'), generator: hotWaterGeneratorTemplate('electric_instantaneous'), nominalPowerKw: 10 }] },
  { id: 'solarWater', part: 'hotWaterRest', path: ['hotWater', 'solar'], present: listed, off: [], start: () => [solarTemplate(0)] },
  { id: 'showerWtw', part: 'hotWaterRest', residentialOnly: true, path: ['hotWater', 'showerHeatRecovery'],
    present: (value) => value === 'vertical' || value === 'horizontal', off: 'none', start: () => 'unknown' },
  { id: 'hotWaterCollective', part: 'hotWaterRest', residentialOnly: true, path: ['hotWater', 'collective'], present: set, off: null, start: () => ({}) },
  // ISSO 82.1 §11.3: controls of tables 11.4–11.6, a combined (decentral) system, grilles with heating strips.
  { id: 'ventControls', part: 'ventilation', residentialOnly: true, path: ['ventilation', 'controls'], present: set, off: null,
    start: () => ({ evidenceReference: '' }) },
  { id: 'ventCombined', part: 'ventilation', path: ['ventilation', 'combined'], present: set, off: null,
    start: () => ({ decentralAreaM2: 0, totalResidenceAreaM2: 0 }) },
  { id: 'ventStrips', part: 'ventilation', path: ['ventilation', 'grilleHeatingStrips'], present: set, off: null,
    start: () => ({ sourceReference: '' }) },
  // ISSO §11.4.1/§11.5.6: passive cooling proven by a supplier project document.
  { id: 'passiveCooling', part: 'ventilation', path: ['ventilation', 'passiveCooling'], present: set, off: null,
    start: () => ({ evidenceReference: '', installedCapacityDm3PerS: null }) },
];
const BY_ID = new Map(COMPONENTS.map((item) => [item.id, item]));

/** The parts a question asks about, in order. */
export function componentsOf(part: SurveyPart | undefined, kind: StoredSurvey['kind']): SurveyComponentId[] {
  return COMPONENTS.filter((item) => item.part === part && (kind === 'residential' || !item.residentialOnly)).map((item) => item.id);
}

type Survey = Record<string, unknown>;
const at = (survey: Survey, path: string[]): unknown =>
  path.reduce<unknown>((value, key) => (value != null && typeof value === 'object' ? (value as Survey)[key] : undefined), survey);
const writeAt = (survey: Survey, [head, ...rest]: string[], value: unknown): Survey => ({
  ...survey,
  [head]: rest.length === 0 ? value : writeAt(((survey[head] as Survey | undefined) ?? {}), rest, value),
});

/** Whether the survey already holds data of the part. */
export function componentHasData(id: SurveyComponentId, survey: Survey): boolean {
  const def = BY_ID.get(id)!;
  return def.present(at(survey, def.path));
}

/** Holds more than what ticking it starts with (unticking then asks first). */
export function componentFilledIn(id: SurveyComponentId, survey: Survey): boolean {
  const def = BY_ID.get(id)!;
  const value = at(survey, def.path);
  return def.present(value) && JSON.stringify(value) !== JSON.stringify(def.start?.());
}

/** Shown: ticked, or holding data. */
export function componentActive(stored: StoredSurvey, id: SurveyComponentId): boolean {
  return Boolean(stored.components?.includes(id)) || componentHasData(id, stored.survey);
}

/** Ticks or unticks a part: ticking starts its data, unticking clears it. */
export function toggleComponent(stored: StoredSurvey, id: SurveyComponentId, on: boolean): StoredSurvey {
  const def = BY_ID.get(id)!;
  const ticked = new Set(stored.components ?? []);
  let survey = stored.survey as Survey;
  if (on) {
    ticked.add(id);
    if (!def.present(at(survey, def.path)) && def.start) survey = writeAt(survey, def.path, def.start());
  } else {
    ticked.delete(id);
    survey = writeAt(survey, def.path, def.off);
  }
  return { ...stored, survey, components: [...ticked] };
}
