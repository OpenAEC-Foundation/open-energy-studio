// The simplified (indicative) BENG calculation behind "Bereken", with the checks that refuse a project it cannot
// calculate. The calculate button of the app and the assistant tools of the site (core/site/siteAppTools.ts) use it.
import type { IBENGResultMonthly, IProject } from './types';
import { calculateBENGMonthly } from './BENGCalculatorMonthly';
import { hasUnmodelledHeatPumpDetails, legacyHeatPumpInputIssue, validProjectFloorArea } from './ProjectArea';

export type IndicativeOutcome =
  | { result: IBENGResultMonthly; errorKey: null; error: null }
  /** `errorKey` is an i18n key of the app; `error` the message of a calculation that failed. */
  | { result: null; errorKey: string | null; error: string | null };

export function indicativeCalculation(project: IProject): IndicativeOutcome {
  const refused = (errorKey: string): IndicativeOutcome => ({ result: null, errorKey, error: null });
  if (project.ntaHeatPumps?.length) return refused('calculation.standaloneHeatPumps');
  if (hasUnmodelledHeatPumpDetails(project)) return refused('calculation.performancePointsUnsupported');
  if (validProjectFloorArea(project) === null) return refused('calculation.invalidFloorArea');
  const heatPumpIssue = legacyHeatPumpInputIssue(project);
  if (heatPumpIssue) return refused(heatPumpIssue === 'cop' ? 'calculation.invalidHeatPumpCop' : 'calculation.invalidHeatPumpCoverage');
  try {
    return { result: calculateBENGMonthly(project), errorKey: null, error: null };
  } catch (error) {
    return { result: null, errorKey: null, error: error instanceof Error ? error.message : String(error) };
  }
}
