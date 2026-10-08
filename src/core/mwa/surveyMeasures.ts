/**
 * Maatwerkadvies on a basisopname (feedback 8 Oct 2026). A measure is a
 * change to the survey answers ("dak 100 → 200 mm", "PV 8 → 16 m²"). The app
 * runs the survey again with the change, and the difference between the two
 * derived building inputs is the measure's patch on the kernel's `building`
 * base (MwaBase::Building, crates/nta8800-core/src/maatwerkadvies.rs). The
 * kernel then runs label, savings and economics per ISSO 82.2 as for any
 * other base, so packages combine the measures' patches.
 */
import type {
  MaatwerkadviesAssessment, MwaMeasure, MwaMeasureCategory, MwaPatchOperation, NtaMaatwerkadvies, OpnameAssessment,
} from '../nta/KernelClient';
import { assessMaatwerkadviesInputWithRust, assessResidentialSurveyWithRust, assessUtilitySurveyWithRust } from '../nta/KernelClient';
import { asResidential, asUtility, type StoredSurvey } from '../nta/SurveyTemplates';

export type SurveyMeasureKind = 'roof' | 'facade' | 'floor' | 'windows' | 'heating' | 'hotWater' | 'ventilation' | 'pv';

export const SURVEY_MEASURE_KINDS: SurveyMeasureKind[] = ['roof', 'facade', 'floor', 'windows', 'heating', 'hotWater', 'ventilation', 'pv'];

/** The change a measure makes to the survey; stored as the measure's template. */
export type SurveyMeasureChange =
  | { kind: 'roof' | 'facade' | 'floor'; surfaceIds: string[]; thicknessMm: number }
  | { kind: 'windows'; surfaceIds: string[]; glass: string; frame?: string }
  | { kind: 'heating'; generator: Record<string, unknown>; emitters?: string; designClass?: string }
  | { kind: 'hotWater'; generator: Record<string, unknown> }
  | { kind: 'ventilation'; principle: string; heatRecovery?: string | null }
  | { kind: 'pv'; panelAreaM2: number; azimuthDeg: number; tiltDeg: number; moduleType: string };

export interface SurveyMeasureTemplate { source: 'survey'; change: SurveyMeasureChange }

export function surveyTemplateOf(measure: MwaMeasure): SurveyMeasureTemplate | null {
  const template = measure.template as unknown as SurveyMeasureTemplate | null | undefined;
  return template && template.source === 'survey' ? template : null;
}

export const CATEGORY: Record<SurveyMeasureKind, MwaMeasureCategory> = {
  roof: 'insulation', facade: 'insulation', floor: 'insulation', windows: 'glazing', heating: 'heating',
  hotWater: 'hot_water', ventilation: 'ventilation', pv: 'pv',
};

const ELEMENT: Partial<Record<SurveyMeasureKind, string>> = { roof: 'roof', facade: 'facade', floor: 'floor' };

type Json = Record<string, unknown>;
const clone = <T>(value: T): T => structuredClone(value);

/** The survey with one change applied. */
export function applySurveyChange(survey: Json, change: SurveyMeasureChange): Json {
  const next = clone(survey);
  const envelope = (next.envelope ?? {}) as Json;
  const surfaces = (envelope.surfaces as Json[] | undefined) ?? [];
  switch (change.kind) {
    case 'roof': case 'facade': case 'floor': {
      const element = ELEMENT[change.kind];
      for (const surface of surfaces) {
        if (surface.element !== element) continue;
        if (change.surfaceIds.length > 0 && !change.surfaceIds.includes(String(surface.id))) continue;
        surface.insulation = { kind: 'thickness', thicknessMm: change.thicknessMm };
      }
      break;
    }
    case 'windows': {
      for (const window of (envelope.windows as Json[] | undefined) ?? []) {
        if (change.surfaceIds.length > 0 && !change.surfaceIds.includes(String(window.surfaceId))) continue;
        window.glass = change.glass;
        if (change.frame) window.frame = change.frame;
      }
      break;
    }
    case 'heating': {
      const heating: Json = { ...((next.heating ?? {}) as Json), generator: clone(change.generator) };
      if (change.emitters) heating.emitters = change.emitters;
      if (change.designClass) heating.designClass = change.designClass;
      next.heating = heating;
      break;
    }
    case 'hotWater':
      next.hotWater = { ...((next.hotWater ?? {}) as Json), generator: clone(change.generator) };
      break;
    case 'ventilation': {
      const ventilation: Json = { ...((next.ventilation ?? {}) as Json), principle: change.principle };
      if (change.heatRecovery) ventilation.heatRecovery = change.heatRecovery; else delete ventilation.heatRecovery;
      next.ventilation = ventilation;
      break;
    }
    case 'pv': {
      const pv = ((next.pv as Json[] | undefined) ?? []).map(clone);
      const first = pv[0] ?? { id: 'pv1', mounting: 'moderately_ventilated', sourceReference: '' };
      // One PV system with the new total area; the others are folded into it.
      next.pv = [{ ...first, panelAreaM2: change.panelAreaM2, azimuthDeg: change.azimuthDeg, tiltDeg: change.tiltDeg, moduleType: change.moduleType }];
      break;
    }
  }
  return next;
}

const pointer = (segments: Array<string | number>) => `/${segments.map((part) => String(part).replace(/~/g, '~0').replace(/\//g, '~1')).join('/')}`;
const isObject = (value: unknown): value is Json => value != null && typeof value === 'object' && !Array.isArray(value);

/** RFC 6902 operations that turn `from` into `to` (never on the root). */
export function jsonDiff(from: unknown, to: unknown, path: Array<string | number> = []): MwaPatchOperation[] {
  if (JSON.stringify(from) === JSON.stringify(to)) return [];
  if (isObject(from) && isObject(to)) {
    const operations: MwaPatchOperation[] = [];
    for (const key of Object.keys(to)) {
      if (!(key in from)) operations.push({ op: 'add', path: pointer([...path, key]), value: to[key] } as MwaPatchOperation);
      else operations.push(...jsonDiff(from[key], to[key], [...path, key]));
    }
    for (const key of Object.keys(from)) {
      if (!(key in to)) operations.push({ op: 'remove', path: pointer([...path, key]) } as MwaPatchOperation);
    }
    return operations;
  }
  if (Array.isArray(from) && Array.isArray(to) && from.length === to.length) {
    return from.flatMap((item, index) => jsonDiff(item, to[index], [...path, index]));
  }
  if (path.length === 0) throw new Error('survey measure changes the whole input');
  return [{ op: 'replace', path: pointer(path), value: to } as MwaPatchOperation];
}

/** True when two patches touch the same part of the input (one path within the other). */
export function patchesOverlap(a: MwaPatchOperation[], b: MwaPatchOperation[]): boolean {
  const within = (x: string, y: string) => x === y || x.startsWith(`${y}/`) || y.startsWith(`${x}/`);
  return a.some((first) => b.some((second) => within(first.path, second.path)));
}

async function deriveInput(stored: StoredSurvey, survey: Json): Promise<{ input: unknown; assessment: OpnameAssessment }> {
  const variant: StoredSurvey = { ...stored, survey };
  const assessment = stored.kind === 'residential'
    ? await assessResidentialSurveyWithRust(asResidential(variant))
    : await assessUtilitySurveyWithRust(asUtility(variant));
  if (!assessment.derivedInput) throw new Error(`survey_not_calculated: ${assessment.issues.map((issue) => issue.code).join(', ')}`);
  return { input: assessment.derivedInput, assessment };
}

export interface SurveyMwaRun {
  assessment: MaatwerkadviesAssessment;
  /** Per measure: its patch, for the overlap check of packages. */
  patches: Record<string, MwaPatchOperation[]>;
}

/** Runs the maatwerkadvies of a survey project: the survey's derived input is the base. */
export async function assessSurveyMaatwerkadvies(stored: StoredSurvey, definition: NtaMaatwerkadvies): Promise<SurveyMwaRun> {
  const base = await deriveInput(stored, stored.survey);
  const patches: Record<string, MwaPatchOperation[]> = {};
  const measures: MwaMeasure[] = [];
  for (const measure of definition.measures) {
    const template = surveyTemplateOf(measure);
    if (!template) continue;
    const variant = await deriveInput(stored, applySurveyChange(stored.survey as Json, template.change));
    const patch = jsonDiff(base.input, variant.input);
    patches[measure.id] = patch;
    // The kernel keeps no unknown fields: the template stays in the app.
    const { template: _template, incomplete: _incomplete, ...rest } = measure;
    void _template; void _incomplete;
    measures.push({ ...rest, target: 'building', patch });
  }
  const known = new Set(measures.map((measure) => measure.id));
  const input = {
    base: { kind: 'building', input: base.input },
    ...(definition.currentUse ? { currentUse: definition.currentUse } : {}),
    ...(definition.futureUse ? { futureUse: definition.futureUse } : {}),
    measures,
    packages: definition.packages.map((pack) => ({ ...pack, measureIds: pack.measureIds.filter((id) => known.has(id)) })),
    tariffs: definition.tariffs,
    ...(definition.economics ? { economics: definition.economics } : {}),
    ...(definition.measured ? { measured: definition.measured } : {}),
    ...(definition.advisedPackageId && definition.packages.some((pack) => pack.id === definition.advisedPackageId)
      ? { advisedPackageId: definition.advisedPackageId } : {}),
    ...(definition.adviceMotivation ? { adviceMotivation: definition.adviceMotivation } : {}),
    ...(definition.notes ? { notes: definition.notes } : {}),
  };
  const assessment = await assessMaatwerkadviesInputWithRust(input);
  return { assessment, patches };
}
