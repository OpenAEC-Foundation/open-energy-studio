/**
 * The question flow of the basisopname (UI redesign 2026-10, mockup B/C):
 * one question per screen, grouped in steps the adviser can skip and revisit.
 * The flow is the single definition the navigation, the wizard and the
 * Controle page read; the survey fields themselves stay in BasisopnamePanel.
 */
import type { StoredSurvey, SurveyKind } from '../nta/SurveyTemplates';

/** Which part of the survey a question shows. */
export type SurveyPart =
  | 'address' | 'dwellingType' | 'general' | 'zones' | 'walls' | 'roofFloor'
  | 'heatingKind' | 'heatingGenerator' | 'heatingRest'
  | 'hotWaterKind' | 'hotWaterGenerator' | 'hotWaterRest'
  | 'ventilationPrinciple' | 'ventilation' | 'cooling' | 'pv';

export interface SurveyQuestion {
  /** Unique within its step; the progress key is `<step>.<question>`. */
  id: string;
  part: SurveyPart;
  titleKey: string;
  helpKey: string;
}

export type SurveyStepId =
  | 'woning' | 'gebouw' | 'zones' | 'gevels' | 'dak-vloer' | 'verwarming' | 'warm-water'
  | 'ventilatie' | 'koeling' | 'zonnepanelen' | 'controle' | 'label';

export interface SurveyFlowStep {
  id: SurveyStepId;
  labelKey: string;
  questions: SurveyQuestion[];
  /** Controle and Label show a page of their own instead of questions. */
  special?: 'check' | 'label';
}

const q = (step: string, id: string, part: SurveyPart): SurveyQuestion => ({
  id, part, titleKey: `survey.q.${step}.${id}`, helpKey: `survey.q.${step}.${id}.help`,
});

const STEPS: Record<SurveyStepId, SurveyFlowStep> = {
  woning: { id: 'woning', labelKey: 'survey.step.woning', questions: [q('woning', 'adres', 'address'), q('woning', 'soort', 'dwellingType'), q('woning', 'basis', 'general')] },
  gebouw: { id: 'gebouw', labelKey: 'survey.step.gebouw', questions: [q('gebouw', 'adres', 'address'), q('gebouw', 'basis', 'general')] },
  zones: { id: 'zones', labelKey: 'survey.step.zones', questions: [q('zones', 'zones', 'zones')] },
  gevels: { id: 'gevels', labelKey: 'survey.step.gevels', questions: [q('gevels', 'gevels', 'walls')] },
  'dak-vloer': { id: 'dak-vloer', labelKey: 'survey.step.dak-vloer', questions: [q('dak-vloer', 'dak-vloer', 'roofFloor')] },
  verwarming: {
    id: 'verwarming', labelKey: 'survey.step.verwarming',
    questions: [q('verwarming', 'toestel', 'heatingKind'), q('verwarming', 'details', 'heatingGenerator'), q('verwarming', 'afgifte', 'heatingRest')],
  },
  'warm-water': {
    id: 'warm-water', labelKey: 'survey.step.warm-water',
    questions: [q('warm-water', 'toestel', 'hotWaterKind'), q('warm-water', 'details', 'hotWaterGenerator'), q('warm-water', 'overig', 'hotWaterRest')],
  },
  ventilatie: {
    id: 'ventilatie', labelKey: 'survey.step.ventilatie',
    questions: [q('ventilatie', 'systeem', 'ventilationPrinciple'), q('ventilatie', 'details', 'ventilation')],
  },
  koeling: { id: 'koeling', labelKey: 'survey.step.koeling', questions: [q('koeling', 'koeling', 'cooling')] },
  zonnepanelen: { id: 'zonnepanelen', labelKey: 'survey.step.zonnepanelen', questions: [q('zonnepanelen', 'pv', 'pv')] },
  controle: { id: 'controle', labelKey: 'survey.step.controle', questions: [], special: 'check' },
  label: { id: 'label', labelKey: 'survey.step.label', questions: [], special: 'label' },
};

const ORDER: Record<SurveyKind, SurveyStepId[]> = {
  residential: ['woning', 'gevels', 'dak-vloer', 'verwarming', 'warm-water', 'ventilatie', 'koeling', 'zonnepanelen', 'controle', 'label'],
  utility: ['gebouw', 'zones', 'gevels', 'dak-vloer', 'verwarming', 'warm-water', 'ventilatie', 'koeling', 'zonnepanelen', 'controle', 'label'],
};

/** Every step id of either kind, in a stable order: the sub pages of the survey route. */
export const SURVEY_STEP_IDS: SurveyStepId[] = [
  'woning', 'gebouw', 'zones', 'gevels', 'dak-vloer', 'verwarming', 'warm-water', 'ventilatie', 'koeling', 'zonnepanelen', 'controle', 'label',
];

/** The steps of a survey kind, in order. */
export function surveySteps(kind: SurveyKind): SurveyFlowStep[] {
  return ORDER[kind].map((id) => STEPS[id]);
}

export function surveyStep(id: string | undefined, kind: SurveyKind): SurveyFlowStep {
  const steps = surveySteps(kind);
  return steps.find((step) => step.id === id) ?? steps[0];
}

/** Sub pages of the survey route before the redesign, kept for saved links and hashes. */
export const LEGACY_SURVEY_SUBS: Record<string, SurveyStepId> = {
  general: 'woning', zones: 'zones', envelope: 'gevels', heating: 'verwarming', hotWater: 'warm-water',
  ventilation: 'ventilatie', cooling: 'koeling', pv: 'zonnepanelen', result: 'controle',
};

/** A step id that exists for the kind: the general step differs (woning/gebouw), zones are utility only. */
export function stepForKind(id: string | undefined, kind: SurveyKind): SurveyStepId {
  const mapped = (id && LEGACY_SURVEY_SUBS[id]) ?? id;
  if (mapped === 'woning' && kind === 'utility') return 'gebouw';
  if (mapped === 'gebouw' && kind === 'residential') return 'woning';
  const steps = ORDER[kind];
  return steps.includes(mapped as SurveyStepId) ? mapped as SurveyStepId : steps[0];
}

/** Progress of the question flow, kept next to the survey (not sent to the kernel). */
export interface SurveyProgress {
  done?: string[];
  skipped?: string[];
}

export type QuestionState = 'done' | 'skipped' | 'todo';
export type StepState = 'done' | 'skipped' | 'partial' | 'todo';

export const questionKey = (step: SurveyStepId, question: string) => `${step}.${question}`;

export function questionState(progress: SurveyProgress | undefined, key: string): QuestionState {
  if (progress?.done?.includes(key)) return 'done';
  if (progress?.skipped?.includes(key)) return 'skipped';
  return 'todo';
}

/** A step is done when every question is done; skipped when the rest is done or skipped. */
export function stepState(step: SurveyFlowStep, progress: SurveyProgress | undefined): StepState {
  if (step.special) return progress?.done?.includes(step.id) ? 'done' : 'todo';
  const states = step.questions.map((question) => questionState(progress, questionKey(step.id, question.id)));
  if (states.every((state) => state === 'done')) return 'done';
  if (states.every((state) => state !== 'todo')) return 'skipped';
  if (states.some((state) => state !== 'todo')) return 'partial';
  return 'todo';
}

/** Marks a question (or a special step) done or skipped; done wins over an earlier skip. */
export function markProgress(progress: SurveyProgress | undefined, key: string, mark: 'done' | 'skipped'): SurveyProgress {
  const done = new Set(progress?.done ?? []);
  const skipped = new Set(progress?.skipped ?? []);
  if (mark === 'done') {
    done.add(key);
    skipped.delete(key);
  } else if (!done.has(key)) {
    skipped.add(key);
  }
  return { done: [...done], skipped: [...skipped] };
}

/** The stored survey with its flow progress. */
export type SurveyWithProgress = StoredSurvey & { progress?: SurveyProgress };

/**
 * The step and question of a kernel or survey path (`basisopname.heating.generator.kind`),
 * for "Ga naar vraag". Surfaces go to Gevels or Dak en vloer by their element.
 */
export function questionForPath(path: string, stored: StoredSurvey): { step: SurveyStepId; question?: string } {
  const kind = stored.kind;
  const bare = path.replace(/^basisopname\./, '');
  const head = bare.split(/[.[]/)[0];
  const general = kind === 'utility' ? 'gebouw' : 'woning';
  switch (head) {
    case 'zones': case 'lighting': return { step: kind === 'utility' ? 'zones' : general };
    case 'envelope': {
      if (/^envelope\.rooflights/.test(bare)) return { step: 'dak-vloer' };
      const surface = /^envelope\.surfaces\[(\d+)\]/.exec(bare);
      if (surface) {
        const surfaces = ((stored.survey.envelope as { surfaces?: Array<{ element?: string }> } | undefined)?.surfaces) ?? [];
        const element = surfaces[Number(surface[1])]?.element;
        return { step: element === 'roof' || element === 'floor' ? 'dak-vloer' : 'gevels' };
      }
      const window = /^envelope\.windows\[(\d+)\]/.exec(bare);
      if (window) {
        const envelope = stored.survey.envelope as { surfaces?: Array<{ id?: string; element?: string }>; windows?: Array<{ surfaceId?: string }> } | undefined;
        const surfaceId = envelope?.windows?.[Number(window[1])]?.surfaceId;
        if (envelope?.surfaces?.find((item) => item.id === surfaceId)?.element === 'roof') return { step: 'dak-vloer' };
      }
      return { step: 'gevels' };
    }
    case 'heating': {
      if (bare === 'heating.generator') return { step: 'verwarming', question: 'toestel' };
      const generator = /^heating\.(generator|nominalPowerKw|additionalGenerators)/.test(bare);
      return { step: 'verwarming', question: generator ? 'details' : 'afgifte' };
    }
    case 'hotWater': case 'additionalHotWaterSystems': {
      if (bare === 'hotWater.generator') return { step: 'warm-water', question: 'toestel' };
      const generator = /^hotWater\.(generator|nominalPowerKw|additionalGenerators)/.test(bare);
      return { step: 'warm-water', question: generator ? 'details' : 'overig' };
    }
    case 'ventilation': return { step: 'ventilatie', question: bare === 'ventilation.principle' ? 'systeem' : 'details' };
    case 'cooling': case 'coolingPresent': case 'coolingCollective': return { step: 'koeling' };
    case 'pv': case 'storage': return { step: 'zonnepanelen' };
    case 'derivedInput': case 'inklapRedenen': case '': return { step: 'controle' };
    case 'dwelling': return { step: general, question: kind === 'residential' ? 'soort' : undefined };
    default: return { step: general, question: kind === 'residential' ? 'basis' : undefined };
  }
}
