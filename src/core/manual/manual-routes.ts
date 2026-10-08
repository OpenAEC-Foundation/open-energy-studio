/**
 * Which manual chapter explains which page. Kept apart from manual.ts so the
 * page header can link to the manual without bundling the manual itself.
 */
import type { Route, StepId } from '../navigation/routes';

export const MANUAL_INDEX = 'index';

/** The chapter that explains a workflow step (the "Handleiding" link on every page). */
const STEP_CHAPTERS: Record<StepId, string> = {
  project: '03-projectberekening',
  building: '03-projectberekening',
  installations: '03-projectberekening',
  check: '05-validatie',
  results: '04-uitvoer',
  survey: '02-basisopname',
  advice: '06-maatwerkadvies',
  relabel: '07-herlabelen-registratie-dossier',
  report: '04-uitvoer',
  registration: '07-herlabelen-registratie-dossier',
  tool: '00-werken-met-het-programma',
};

export function chapterForRoute(route: Route): string {
  if (route.step === 'report' && (route.sub === 'checklist' || route.sub === 'exports')) return '07-herlabelen-registratie-dossier';
  return STEP_CHAPTERS[route.step] ?? MANUAL_INDEX;
}

/** "03-projectberekening#koelvermogen" → chapter and anchor. */
export function splitChapterRef(ref: string | undefined): { chapter: string; anchor?: string } {
  if (!ref) return { chapter: MANUAL_INDEX };
  const [chapter, anchor] = ref.split('#');
  return { chapter: chapter || MANUAL_INDEX, ...(anchor ? { anchor } : {}) };
}
