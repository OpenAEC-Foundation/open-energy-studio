import { useEffect, useId, useMemo, useState } from 'react';
import { useI18n } from '../../i18n/i18n';
import type { IProject } from '../../core/energy/types';
import type { NtaInterpretationGroup, ProjectPerformanceAssessment } from '../../core/nta/KernelClient';
import { fetchKernelInterpretations } from '../../core/nta/KernelClient';
import {
  DETAIL_SECTIONS, DETAIL_SECTION_TITLES, generateEnergyPerformanceReportHTML,
  type DetailSection, type ReportLevel, type ReportOptions,
} from '../../core/report/EnergyPerformanceReport';
import { downloadEnergyPerformanceReportHTML, printEnergyPerformanceReport } from '../../core/report/ReportGenerator';

const STORAGE_KEY = 'oes-energy-report-options';
const LEVELS: ReportLevel[] = ['summary', 'standard', 'detailed'];

interface StoredChoice {
  level: ReportLevel;
  details: Record<DetailSection, boolean>;
}

const allDetails = (on: boolean) => Object.fromEntries(DETAIL_SECTIONS.map((section) => [section, on])) as Record<DetailSection, boolean>;

/** The user's last report choice from localStorage; the standard level when nothing is stored. */
export function loadReportChoice(): StoredChoice {
  try {
    const parsed = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? 'null') as Partial<StoredChoice> | null;
    if (parsed && LEVELS.includes(parsed.level as ReportLevel)) {
      return { level: parsed.level as ReportLevel, details: { ...allDetails(true), ...(parsed.details ?? {}) } };
    }
  } catch { /* fall back to the default */ }
  return { level: 'standard', details: allDetails(true) };
}

function storeReportChoice(choice: StoredChoice) {
  try { localStorage.setItem(STORAGE_KEY, JSON.stringify(choice)); } catch { /* storage unavailable */ }
}

/**
 * The "Rapportage Energieprestatie (NTA 8800)" builder: report level, the optional detail
 * chapters, a live preview and export/print. The preview uses the same kernel run as the
 * rest of the report tab; exports run the kernel again on the current project.
 */
export function ReportBuilder({ project, assessment, pending }: {
  project: IProject;
  assessment: ProjectPerformanceAssessment | null;
  pending: boolean;
}) {
  const { t } = useI18n();
  const id = useId();
  const [choice, setChoice] = useState<StoredChoice>(loadReportChoice);
  const [interpretations, setInterpretations] = useState<NtaInterpretationGroup[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [previewOpen, setPreviewOpen] = useState(true);

  useEffect(() => {
    let cancelled = false;
    fetchKernelInterpretations().then((groups) => { if (!cancelled) setInterpretations(groups); }).catch(() => undefined);
    return () => { cancelled = true; };
  }, []);

  const update = (next: StoredChoice) => {
    setChoice(next);
    storeReportChoice(next);
  };
  const options: ReportOptions = { level: choice.level, details: choice.details, interpretations };
  const preview = useMemo(
    () => (assessment ? generateEnergyPerformanceReportHTML(project, assessment, { level: choice.level, details: choice.details, interpretations }) : null),
    [project, assessment, choice, interpretations],
  );
  const run = (action: () => Promise<void>) => {
    setError(null);
    setBusy(true);
    action().catch((reason: unknown) => setError(reason instanceof Error ? reason.message : String(reason))).finally(() => setBusy(false));
  };
  const detailed = choice.level === 'detailed';

  return (
    <section className="report-builder" aria-labelledby={`${id}-title`}>
      <h2 id={`${id}-title`}>{t('report.builder.title')}</h2>
      <p>{t('report.builder.intro')}</p>
      <fieldset className="report-builder-levels">
        <legend>{t('report.builder.level')}</legend>
        {LEVELS.map((level) => (
          <label key={level} className="report-builder-level">
            <input type="radio" name={`${id}-level`} value={level} checked={choice.level === level}
              onChange={() => update({ ...choice, level })} />
            <span><strong>{t(`report.builder.level.${level}`)}</strong> — {t(`report.builder.level.${level}.description`)}</span>
          </label>
        ))}
      </fieldset>
      <fieldset className="report-builder-details" disabled={!detailed} aria-describedby={`${id}-details-note`}>
        <legend>{t('report.builder.details')}</legend>
        <p id={`${id}-details-note`} className="report-builder-note">{t(detailed ? 'report.builder.detailsNote' : 'report.builder.detailsDisabled')}</p>
        <div className="report-builder-detail-grid">
          {DETAIL_SECTIONS.map((section) => (
            <label key={section}>
              <input type="checkbox" checked={choice.details[section]}
                onChange={(event) => update({ ...choice, details: { ...choice.details, [section]: event.target.checked } })} />
              {t(`report.builder.section.${section}`, { defaultValue: DETAIL_SECTION_TITLES[section] })}
            </label>
          ))}
        </div>
        <div className="report-builder-detail-actions">
          <button type="button" className="btn" onClick={() => update({ ...choice, details: allDetails(true) })}>{t('report.builder.selectAll')}</button>
          <button type="button" className="btn" onClick={() => update({ ...choice, details: allDetails(false) })}>{t('report.builder.selectNone')}</button>
        </div>
      </fieldset>
      <div className="report-builder-actions">
        <button type="button" className="btn btn-primary" disabled={busy} onClick={() => run(() => downloadEnergyPerformanceReportHTML(project, options))}>
          {t('report.builder.export')}
        </button>
        <button type="button" className="btn" disabled={busy} onClick={() => run(() => printEnergyPerformanceReport(project, options))}>
          {t('report.builder.print')}
        </button>
        <button type="button" className="btn" aria-expanded={previewOpen} onClick={() => setPreviewOpen(!previewOpen)}>
          {t(previewOpen ? 'report.builder.hidePreview' : 'report.builder.showPreview')}
        </button>
      </div>
      {error && <p role="alert">{error}</p>}
      {previewOpen && (
        pending && !preview
          ? <p role="status">{t('report.builder.pending')}</p>
          : preview
            ? <iframe className="report-builder-preview" title={t('report.builder.previewTitle')} srcDoc={preview} data-testid="report-builder-preview" />
            : <p role="status">{t('report.builder.noKernel')}</p>
      )}
    </section>
  );
}
