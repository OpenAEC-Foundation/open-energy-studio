/**
 * The live BRL 9500 dossier check and the project dossier export, shared by
 * the report view and the Rapport & dossier pages (UI redesign F9).
 */
import { useEffect, useMemo, useState } from 'react';
import type { IProject } from '../../core/energy/types';
import { labelInputSha256 } from '../../core/nta/Registration';
import { assessStoredSurvey } from '../../core/nta/SurveyTemplates';
import type { OpnameAssessment, ProjectPerformanceAssessment } from '../../core/nta/KernelClient';
import { DossierWriteError, downloadProjectDossier } from '../../core/report/ReportGenerator';
import {
  checkDossierCompleteness, DossierTooLargeError, MAX_DOSSIER_BYTES, openDossierItems, type DossierItem,
} from '../../core/report/ProjectDossier';
import { useI18n } from '../../i18n/i18n';

const GIB = 1024 * 1024 * 1024;

/** The message for a failed dossier export. */
export function dossierErrorText(reason: unknown, t: (key: string, options?: Record<string, unknown>) => string): string {
  if (reason instanceof DossierTooLargeError) {
    return t('dossier.error.tooLarge', { size: (reason.totalBytes / GIB).toFixed(1), max: MAX_DOSSIER_BYTES / GIB });
  }
  if (reason instanceof DossierWriteError) return t('dossier.error.writeFailed', { reason: reason.reason });
  return reason instanceof Error ? reason.message : String(reason);
}

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
  const { t } = useI18n();
  const [exported, setExported] = useState<{ project: IProject; checklist: DossierItem[]; missingEvidence: number } | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const exportDossier = () => {
    setError(null);
    setBusy(true);
    downloadProjectDossier(project)
      .then((manifest) => setExported({ project, checklist: manifest.checklist, missingEvidence: manifest.missingEvidence.length }))
      .catch((reason: unknown) => setError(dossierErrorText(reason, t)))
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
