/**
 * The live BRL 9500 dossier check and the project dossier export, shared by
 * the report view and the Rapport & dossier pages (UI redesign F9).
 */
import { useEffect, useMemo, useState } from 'react';
import type { IProject } from '../../core/energy/types';
import { labelInputSha256 } from '../../core/nta/Registration';
import { assessStoredSurvey } from '../../core/nta/SurveyTemplates';
import type { OpnameAssessment, ProjectPerformanceAssessment } from '../../core/nta/KernelClient';
import { downloadProjectDossier } from '../../core/report/ReportGenerator';
import { checkDossierCompleteness, openDossierItems, type DossierItem } from '../../core/report/ProjectDossier';

export interface DossierState {
  checklist: DossierItem[];
  open: DossierItem[];
  /** Export the project dossier (ZIP); the checklist then shows what was exported. */
  exportDossier: () => void;
  busy: boolean;
  error: string | null;
  /** Evidence files of the last export that were unavailable or changed. */
  missingEvidence: number | null;
}

export function useDossier(
  project: IProject,
  assessment: ProjectPerformanceAssessment | null,
  kernelPending: boolean,
): DossierState {
  const [exported, setExported] = useState<{ project: IProject; checklist: DossierItem[]; missingEvidence: number } | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const exportDossier = () => {
    setError(null);
    setBusy(true);
    downloadProjectDossier(project)
      .then((manifest) => setExported({ project, checklist: manifest.checklist, missingEvidence: manifest.missingEvidence.length }))
      .catch((reason: unknown) => setError(reason instanceof Error ? reason.message : String(reason)))
      .finally(() => setBusy(false));
  };
  // The live check uses the same inputs as the dossier export: the kernel
  // assessment (with its relabel re-run) and the canonical label-input hash.
  const [labelSha, setLabelSha] = useState<{ project: IProject; sha: string | null } | null>(null);
  useEffect(() => {
    let cancelled = false;
    labelInputSha256(project).then((sha) => { if (!cancelled) setLabelSha({ project, sha }); })
      .catch(() => { if (!cancelled) setLabelSha({ project, sha: null }); });
    return () => { cancelled = true; };
  }, [project]);
  const currentSha = labelSha?.project === project ? labelSha.sha : null;
  // The survey assessment, as the export passes it, for the collapse reasons.
  const survey = project.basisopname;
  const edition = project.ntaCalculation?.normVersion ?? null;
  const [opname, setOpname] = useState<{ survey: typeof survey; result: OpnameAssessment | null } | null>(null);
  useEffect(() => {
    let cancelled = false;
    assessStoredSurvey(survey, edition).then((result) => { if (!cancelled) setOpname({ survey, result }); });
    return () => { cancelled = true; };
  }, [survey, edition]);
  const opnameDone = opname?.survey === survey;
  const checklist = useMemo(() => {
    if (exported && exported.project === project) return exported.checklist;
    return checkDossierCompleteness({
      project, assessment, opname: opnameDone ? opname?.result ?? null : null, labelInputSha256: currentSha,
      pending: kernelPending || !opnameDone,
    });
  }, [exported, project, assessment, currentSha, opname, opnameDone, kernelPending]);
  return {
    checklist,
    open: openDossierItems(checklist),
    exportDossier,
    busy,
    error,
    missingEvidence: exported ? exported.missingEvidence : null,
  };
}
