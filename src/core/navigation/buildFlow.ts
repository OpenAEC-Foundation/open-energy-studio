/**
 * The input question flow of a project without a basisopname (new build and
 * NTA input), in the style of the basisopname flow (UI redesign 2026-10):
 * one topic per screen, skip and come back later, Controle as the overview.
 *
 * The questions are the sub pages of the workflow steps, so the existing
 * pages, routes and "Ga naar" targets stay as they are. Progress lives in
 * `project.workflowProgress` and is not sent to the kernel.
 */
import type { IProject } from '../energy/types';
import type { Route, StepId } from './routes';
import { markProgress, questionState, type QuestionState, type StepState, type SurveyProgress } from '../survey/surveyFlow';

export type BuildFlowStepId = 'project' | 'building' | 'installations' | 'check';

export interface BuildFlowStep {
  id: BuildFlowStepId;
  /** Sub page ids; the project step has the single question `info`. */
  questions: string[];
}

/** The project step has no sub page; its one question is the project data. */
export const PROJECT_QUESTION = 'info';

export const BUILD_FLOW_STEP_IDS: BuildFlowStepId[] = ['project', 'building', 'installations', 'check'];

/** The steps of the flow for a project; lighting, humidification and BACS only for utility buildings. */
export function buildFlowSteps(project: Pick<IProject, 'buildingFunction'>): BuildFlowStep[] {
  const utility = project.buildingFunction !== 'residential';
  return [
    { id: 'project', questions: [PROJECT_QUESTION] },
    { id: 'building', questions: ['envelope', 'zones', 'constructions', 'thermalBridges', 'airTightness', 'unheated'] },
    {
      id: 'installations', questions: [
        'heating', 'hotWater', 'ventilation', 'cooling',
        ...(utility ? ['humidification', 'lighting'] : []),
        'generation',
        ...(utility ? ['bacs'] : []),
      ],
    },
    { id: 'check', questions: [] },
  ];
}

export function isBuildFlowStep(step: StepId): step is BuildFlowStepId {
  return (BUILD_FLOW_STEP_IDS as string[]).includes(step);
}

/** The question a route shows, or null when the route is outside the flow (3D model, overview, NTA input). */
export function buildQuestionOf(route: Route, steps: BuildFlowStep[]): string | null {
  if (route.step === 'project') return PROJECT_QUESTION;
  const step = steps.find((candidate) => candidate.id === route.step);
  if (!step || step.questions.length === 0) return null;
  return route.sub && step.questions.includes(route.sub) ? route.sub : null;
}

/** The route of a question. */
export function buildQuestionRoute(step: BuildFlowStepId, question?: string): Route {
  if (step === 'project') return { step: 'project' };
  if (step === 'check') return { step: 'check', sub: 'overview' };
  return { step, ...(question ? { sub: question } : {}) };
}

export const buildQuestionKey = (step: BuildFlowStepId, question: string) => `${step}.${question}`;

export function buildQuestionState(progress: SurveyProgress | undefined, step: BuildFlowStepId, question: string): QuestionState {
  return questionState(progress, buildQuestionKey(step, question));
}

/** Done when every question is done; skipped when the rest is done or skipped. */
export function buildStepState(step: BuildFlowStep, progress: SurveyProgress | undefined): StepState {
  if (step.questions.length === 0) return progress?.done?.includes(step.id) ? 'done' : 'todo';
  const states = step.questions.map((question) => buildQuestionState(progress, step.id, question));
  if (states.every((state) => state === 'done')) return 'done';
  if (states.every((state) => state !== 'todo')) return 'skipped';
  if (states.some((state) => state !== 'todo')) return 'partial';
  return 'todo';
}

export { markProgress };
