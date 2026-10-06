/**
 * Workflow navigation (docs/ui-redesign/ontwerp.md §3.2, §3.5).
 *
 * The app shell shows numbered workflow steps instead of a ribbon. A route is
 * `{ step, sub?, focusPath? }`; the old `viewMode` values stay available as an
 * alias so existing code that dispatches `SET_VIEW_MODE` keeps working.
 */
import type { IProject, ViewMode } from '../energy/types';
import { LEGACY_SURVEY_SUBS, SURVEY_STEP_IDS, surveySteps } from '../survey/surveyFlow';

export type StepId =
  | 'project' | 'building' | 'installations' | 'check' | 'results'
  | 'survey' | 'advice' | 'relabel' | 'report' | 'registration'
  | 'tool';

export type StepGroup = 'input' | 'calculation' | 'existing' | 'delivery';

export interface SubPage {
  id: string;
  labelKey: string;
}

export interface StepDefinition {
  id: StepId;
  /** Shown in the navigation circle; tools have no number. */
  number: number;
  group: StepGroup;
  labelKey: string;
  /** URL hash segment, Dutch as in the design (`#/installaties`). */
  slug: string;
  subs: SubPage[];
}

export interface Route {
  step: StepId;
  sub?: string;
  /** Kernel or project path of the field to focus after navigating ("Ga naar"). */
  focusPath?: string;
  /** The question of the basisopname step to show (question flow). */
  question?: string;
}

export const WORKFLOW_STEPS: StepDefinition[] = [
  { id: 'project', number: 1, group: 'input', labelKey: 'nav.step.project', slug: 'project', subs: [] },
  {
    id: 'building', number: 2, group: 'input', labelKey: 'nav.step.building', slug: 'gebouw', subs: [
      { id: 'envelope', labelKey: 'nav.sub.building.envelope' },
      { id: 'zones', labelKey: 'nav.sub.building.zones' },
      { id: 'constructions', labelKey: 'nav.sub.building.constructions' },
      { id: 'thermalBridges', labelKey: 'nav.sub.building.thermalBridges' },
      { id: 'airTightness', labelKey: 'nav.sub.building.airTightness' },
      { id: 'unheated', labelKey: 'nav.sub.building.unheated' },
      { id: 'model3d', labelKey: 'nav.sub.building.model3d' },
    ],
  },
  {
    id: 'installations', number: 3, group: 'input', labelKey: 'nav.step.installations', slug: 'installaties', subs: [
      { id: 'systems', labelKey: 'nav.sub.installations.systems' },
      { id: 'heating', labelKey: 'nav.sub.installations.heating' },
      { id: 'hotWater', labelKey: 'nav.sub.installations.hotWater' },
      { id: 'ventilation', labelKey: 'nav.sub.installations.ventilation' },
      { id: 'cooling', labelKey: 'nav.sub.installations.cooling' },
      { id: 'humidification', labelKey: 'nav.sub.installations.humidification' },
      { id: 'lighting', labelKey: 'nav.sub.installations.lighting' },
      { id: 'generation', labelKey: 'nav.sub.installations.generation' },
      { id: 'heatPumps', labelKey: 'nav.sub.installations.heatPumps' },
      { id: 'bacs', labelKey: 'nav.sub.installations.bacs' },
    ],
  },
  {
    id: 'check', number: 4, group: 'calculation', labelKey: 'nav.step.check', slug: 'controle', subs: [
      { id: 'overview', labelKey: 'nav.sub.check.overview' },
      { id: 'input', labelKey: 'nav.sub.check.input' },
    ],
  },
  {
    id: 'results', number: 5, group: 'calculation', labelKey: 'nav.step.results', slug: 'resultaten', subs: [
      { id: 'overview', labelKey: 'nav.sub.results.overview' },
      { id: 'services', labelKey: 'nav.sub.results.services' },
      { id: 'zones', labelKey: 'nav.sub.results.zones' },
      { id: 'monthly', labelKey: 'nav.sub.results.monthly' },
      { id: 'provenance', labelKey: 'nav.sub.results.provenance' },
    ],
  },
  {
    // The basisopname question flow (UI redesign 2026-10): one sub page per step of
    // src/core/survey/surveyFlow.ts; which steps show depends on the survey kind.
    id: 'survey', number: 6, group: 'existing', labelKey: 'nav.step.survey', slug: 'basisopname',
    subs: SURVEY_STEP_IDS.map((id) => ({ id, labelKey: `survey.step.${id}` })),
  },
  {
    id: 'advice', number: 7, group: 'existing', labelKey: 'nav.step.advice', slug: 'maatwerkadvies', subs: [
      { id: 'measures', labelKey: 'nav.sub.advice.measures' },
      { id: 'use', labelKey: 'nav.sub.advice.use' },
      { id: 'passport', labelKey: 'nav.sub.advice.passport' },
      { id: 'advice', labelKey: 'nav.sub.advice.advice' },
    ],
  },
  { id: 'relabel', number: 8, group: 'existing', labelKey: 'nav.step.relabel', slug: 'herlabelen', subs: [] },
  {
    id: 'report', number: 9, group: 'delivery', labelKey: 'nav.step.report', slug: 'rapport', subs: [
      { id: 'report', labelKey: 'nav.sub.report.report' },
      { id: 'input', labelKey: 'nav.sub.report.input' },
      { id: 'checklist', labelKey: 'nav.sub.report.checklist' },
      { id: 'exports', labelKey: 'nav.sub.report.exports' },
    ],
  },
  { id: 'registration', number: 10, group: 'delivery', labelKey: 'nav.step.registration', slug: 'registratie', subs: [] },
];

/** Tools live in the "Gereedschap" menu at the bottom of the navigation, not in a tab strip. */
export const TOOL_STEP: StepDefinition = {
  id: 'tool', number: 0, group: 'input', labelKey: 'nav.tools', slug: 'gereedschap', subs: [
    { id: 'uvalue', labelKey: 'ribbon.uvalueCalc' },
    { id: 'thermal-bridge', labelKey: 'ribbon.thermalBridgeCalc' },
    { id: 'heat-pump-sizing', labelKey: 'ribbon.heatPumpSizing' },
  ],
};

export const STEP_GROUPS: Array<{ id: StepGroup; labelKey: string }> = [
  { id: 'input', labelKey: 'nav.group.input' },
  { id: 'calculation', labelKey: 'nav.group.calculation' },
  { id: 'existing', labelKey: 'nav.group.existing' },
  { id: 'delivery', labelKey: 'nav.group.delivery' },
];

export function stepDefinition(step: StepId): StepDefinition {
  return step === 'tool' ? TOOL_STEP : WORKFLOW_STEPS.find((definition) => definition.id === step) ?? WORKFLOW_STEPS[0];
}

/** The sub pages shown for a project: the survey's Rekenzones only exist in a utility survey. */
export function visibleSubs(definition: StepDefinition, project: IProject | null | undefined): SubPage[] {
  if (definition.id !== 'survey') return definition.subs;
  const ids = surveySteps(project?.basisopname?.kind ?? 'residential').map((step) => step.id as string);
  return definition.subs.filter((sub) => ids.includes(sub.id));
}

/** The default sub page of a step (its first), or undefined for steps without sub pages. */
export function defaultSub(step: StepId): string | undefined {
  return stepDefinition(step).subs[0]?.id;
}

/** Normalise a route: an unknown sub falls back to the step's first sub page. */
export function normalizeRoute(route: Route): Route {
  const definition = stepDefinition(route.step);
  // Saved links to the survey sections before the question flow.
  if (definition.id === 'survey' && route.sub && LEGACY_SURVEY_SUBS[route.sub]) route = { ...route, sub: LEGACY_SURVEY_SUBS[route.sub] };
  const sub = route.sub && definition.subs.some((candidate) => candidate.id === route.sub) ? route.sub : defaultSub(route.step);
  return {
    step: definition.id, ...(sub ? { sub } : {}), ...(route.focusPath ? { focusPath: route.focusPath } : {}),
    ...(route.question && definition.id === 'survey' ? { question: route.question } : {}),
  };
}

/** Old view modes (ribbon era) as routes; `SET_VIEW_MODE` is an alias of navigating there. */
export function routeForViewMode(viewMode: ViewMode): Route {
  switch (viewMode) {
    case 'project': return { step: 'project' };
    case 'envelope': return { step: 'building', sub: 'envelope' };
    case 'installations':
    case 'renewables': return { step: 'installations', sub: 'systems' };
    case 'results': return { step: 'results' };
    case 'report': return { step: 'report' };
    case 'model3d': return { step: 'building', sub: 'model3d' };
    case 'uvalue-calc': return { step: 'tool', sub: 'uvalue' };
    case 'thermal-bridge-calc': return { step: 'tool', sub: 'thermal-bridge' };
    case 'heat-pump-sizing': return { step: 'tool', sub: 'heat-pump-sizing' };
    default: return { step: 'project' };
  }
}

/** The closest old view mode of a route, kept in state for code that still reads `viewMode`. */
export function viewModeForRoute(route: Route): ViewMode {
  switch (route.step) {
    case 'building': return route.sub === 'model3d' ? 'model3d' : 'envelope';
    case 'installations': return 'installations';
    case 'results': return 'results';
    case 'report': return 'report';
    case 'tool':
      if (route.sub === 'thermal-bridge') return 'thermal-bridge-calc';
      if (route.sub === 'heat-pump-sizing') return 'heat-pump-sizing';
      return 'uvalue-calc';
    default: return 'project';
  }
}

/** `#/installaties/systems` for the browser build and links from reports and notices. */
export function routeToHash(route: Route): string {
  const definition = stepDefinition(route.step);
  return `#/${definition.slug}${route.sub && route.sub !== defaultSub(route.step) ? `/${route.sub}` : ''}`;
}

/** Parse a hash written by `routeToHash`; null when it is not a route. */
export function routeFromHash(hash: string): Route | null {
  const match = /^#\/([^/?#]+)(?:\/([^/?#]+))?/.exec(hash);
  if (!match) return null;
  const definition = [...WORKFLOW_STEPS, TOOL_STEP].find((candidate) => candidate.slug === match[1]);
  if (!definition) return null;
  return normalizeRoute({ step: definition.id, sub: match[2] });
}

/** The previous or next workflow step (Alt ↑/↓); tools step back to the project. */
export function adjacentStep(step: StepId, direction: 1 | -1): StepId {
  const index = WORKFLOW_STEPS.findIndex((definition) => definition.id === step);
  if (index < 0) return 'project';
  const next = Math.min(WORKFLOW_STEPS.length - 1, Math.max(0, index + direction));
  return WORKFLOW_STEPS[next].id;
}
