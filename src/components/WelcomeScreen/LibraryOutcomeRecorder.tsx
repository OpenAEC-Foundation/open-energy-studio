/**
 * Writes the calculated outcome of the open project to its library card
 * (src/core/io/projectLibrary.ts): the kernel's settled answer for a
 * new-build project, the survey outcome for a basisopname project.
 */
import { useEffect } from 'react';
import { useEnergy } from '../../context/EnergyContext';
import { useKernel } from '../../context/KernelProvider';
import { projectCalculated } from '../../core/nta/KernelClient';
import { summarizeForPreview } from '../../core/nta/PreviewSummary';
import { browserStore, rememberOutcome } from '../../core/io/projectLibrary';
import { currentResult, useSurveyAssessment } from '../../core/survey/surveyAssessment';
import { defaultsWithoutReason } from '../../core/survey/surveyRegistration';
import { isSurveyProject } from '../../core/nta/stepStatus';
import type { StoredSurvey } from '../../core/nta/SurveyTemplates';

export function LibraryOutcomeRecorder() {
  const { state } = useEnergy();
  const project = state.project;
  const settled = useKernel()?.settled ?? null;
  const survey = useSurveyAssessment();
  const stored = project.basisopname as StoredSurvey | undefined;
  const surveyResult = isSurveyProject(project) && stored ? currentResult(survey, stored) : null;

  useEffect(() => {
    const store = browserStore();
    if (!store) return;
    if (surveyResult?.performance) {
      const performance = surveyResult.performance;
      rememberOutcome(store, project, {
        label: performance.indicativeLabelClass ?? null,
        ep2: performance.primaryFossilIndicatorKwhPerM2Year ?? null,
        beng1: performance.needIndicatorKwhPerM2Year ?? null,
        beng3: performance.renewableSharePercent ?? null,
        tojuli: performance.tojuliMaxK ?? null,
        defaultsWithoutReason: stored ? defaultsWithoutReason(stored, surveyResult) : undefined,
      });
      return;
    }
    if (settled && !isSurveyProject(project)) {
      const summary = summarizeForPreview(settled);
      if (!projectCalculated(summary.status)) return;
      rememberOutcome(store, project, { label: summary.labelClass, ep2: summary.beng2, beng1: summary.beng1, beng3: summary.beng3, tojuli: summary.tojuliMaxK });
    }
    // The project reference changes on every edit; the outcome is re-recorded only when a result changes.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [settled, surveyResult]);
  return null;
}
