/**
 * The live kernel assessment of the basisopname, shared by the navigation,
 * the question pages and the Controle page. The wizard recalculates after
 * every change (debounced); a fingerprint of the survey keeps a result from
 * an older draft from showing as current.
 */
import { useSyncExternalStore } from 'react';
import type { OpnameAssessment } from '../nta/KernelClient';
import { assessResidentialSurveyWithRust, assessUtilitySurveyWithRust } from '../nta/KernelClient';
import { asResidential, asUtility, missingSurveyAnswers, type StoredSurvey } from '../nta/SurveyTemplates';

export interface SurveyAssessmentState {
  /** JSON of the survey the result belongs to. */
  surveyJson: string | null;
  result: OpnameAssessment | null;
  busy: boolean;
  error: string | null;
}

let state: SurveyAssessmentState = { surveyJson: null, result: null, busy: false, error: null };
const listeners = new Set<() => void>();
let request = 0;

function set(next: Partial<SurveyAssessmentState>) {
  state = { ...state, ...next };
  listeners.forEach((listener) => listener());
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function useSurveyAssessment(): SurveyAssessmentState {
  return useSyncExternalStore(subscribe, () => state, () => state);
}

/** The result only when it belongs to this survey; null while it recalculates. */
export function currentResult(assessment: SurveyAssessmentState, stored: StoredSurvey | undefined | null): OpnameAssessment | null {
  if (!stored || assessment.surveyJson !== JSON.stringify(stored.survey)) return null;
  return assessment.result;
}

/** Runs the kernel on the survey; a newer call wins. */
export async function assessSurvey(stored: StoredSurvey): Promise<void> {
  const surveyJson = JSON.stringify(stored.survey);
  if (state.surveyJson === surveyJson && (state.result || state.busy)) return;
  const current = ++request;
  // Required answers still open: name them instead of a survey the kernel cannot read.
  const missing = missingSurveyAnswers(stored);
  if (missing.length > 0) {
    set({ surveyJson, busy: false, error: null, result: {
      status: 'derived_input_rejected', scope: stored.kind, source: 'app', appliedDefaults: [], warnings: [],
      issues: missing.map((path) => ({ code: 'survey_answer_required', path })),
      derivedInput: null, performance: null, referenceVerified: false,
    } });
    return;
  }
  set({ surveyJson, busy: true, error: null });
  try {
    const assessment = stored.kind === 'residential'
      ? await assessResidentialSurveyWithRust(asResidential(stored))
      : await assessUtilitySurveyWithRust(asUtility(stored));
    // A survey the kernel cannot read comes back as `{ error, message }` without issues.
    const refused = assessment as Partial<OpnameAssessment> & { error?: string; message?: string };
    if (!Array.isArray(refused.issues)) throw new Error(refused.message ?? refused.error ?? 'invalid response');
    if (request === current) set({ result: assessment, busy: false });
  } catch (failure) {
    if (request === current) set({ result: null, busy: false, error: failure instanceof Error ? failure.message : String(failure) });
  }
}

/** Forgets the result (another project, or the survey was discarded). */
export function resetSurveyAssessment() {
  request += 1;
  set({ surveyJson: null, result: null, busy: false, error: null });
}
