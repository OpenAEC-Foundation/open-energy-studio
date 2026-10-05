/**
 * Status bar of the app shell (ontwerp §3.1, mockup 01): kernel state
 * (actueel / verouderd / bezig / achtergehouden), kernel version, BENG 1–3,
 * TOjuli and the indicative label from the shared kernel run, the
 * "onverifieerd · geen attest" status and the save state. A click on a figure
 * opens Resultaten. Without a kernel answer the indicative engine's status
 * stays visible as before.
 */
import { Check, X } from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import { formatNumber } from '../../i18n/format';
import { useEnergy } from '../../context/EnergyContext';
import { DEFAULT_NORM_VERSION, legacyEdition, projectCalculated } from '../../core/nta/KernelClient';
import { useKernel } from '../../context/KernelProvider';
import { summarizeForPreview } from '../../core/nta/PreviewSummary';
import { Pill } from '../ui';
import { useShellActions } from '../shell/ShellActions';
import './StatusBar.css';

function Verdict({ ok }: { ok: boolean | null }) {
  const { t } = useI18n();
  if (ok == null) return null;
  return ok
    ? <span className="ok"><Check aria-label={t('overview.meets')} /></span>
    : <span className="fail"><X aria-label={t('overview.failsLimit')} /></span>;
}

/** A figure in the status bar; a button to Resultaten inside the shell. */
function StatusChip({ onClick, title, children }: { onClick?: () => void; title: string; children: React.ReactNode }) {
  return onClick
    ? <button type="button" className="status-chip" onClick={onClick} title={title}>{children}</button>
    : <span className="status-chip">{children}</span>;
}

export function StatusBar() {
  const { t, locale } = useI18n();
  const { state } = useEnergy();
  const kernel = useKernel();
  const actions = useShellActions();
  const { result, isDirty } = state;

  const assessment = kernel?.settled ?? null;
  const summary = assessment ? summarizeForPreview(assessment) : null;
  const calculated = projectCalculated(summary?.status);
  // An older NTA 8800 edition: shown until the kernel answers, then from its result.
  const edition = assessment?.normVersion ?? state.project.ntaCalculation?.normVersion;
  const legacy = assessment ? legacyEdition(assessment) : edition != null && edition !== DEFAULT_NORM_VERSION;
  const bbl = assessment?.performance?.bblCheck ?? null;
  const phase = kernel?.phase ?? 'idle';
  const phaseKey = phase === 'current' && kernel?.verdict === 'withheld' ? 'withheld' : phase;
  const phaseTone = phaseKey === 'current' ? 'ok' : phaseKey === 'stale' || phaseKey === 'withheld' ? 'warn'
    : phaseKey === 'error' ? 'err' : 'neutral';
  const openResults = actions ? () => actions.navigate({ step: 'results' }) : undefined;
  const Chip = ({ children }: { children: React.ReactNode }) => (
    <StatusChip onClick={openResults} title={t('status.openResults')}>{children}</StatusChip>);
  const n = (value: number | null, digits = 1) => formatNumber(value, locale, digits);

  return (
    <footer className="status-bar">
      <div className="status-section" role="status" aria-live="polite">
        {kernel
          ? <Pill tone={phaseTone}>{t(`status.kernel.${phaseKey}`)}</Pill>
          : <span className="status-hint">{result ? t('status.calculated') : t('status.ready')}</span>}
      </div>
      {assessment && (
        <span className="status-chip status-optional">{t('status.kernelName')} <b>{assessment.kernelVersion}</b> · {assessment.targetNormVersion}</span>
      )}
      {legacy && edition && (
        <Pill tone="warn"><span className="status-legacy-edition" title={t('nta.edition.legacyTitle')}>
          {t('status.legacyEdition', { edition: t(`nta.edition.${edition}`) })}
        </span></Pill>
      )}
      <span className="status-sep" aria-hidden="true" />
      <div className="status-section">
        {calculated && summary ? <>
          {summary.beng1 != null && <Chip>BENG 1 <b>{n(summary.beng1)}</b> <Verdict ok={bbl?.energyNeedMeets ?? null} /></Chip>}
          {summary.beng2 != null && <Chip>BENG 2 <b>{n(summary.beng2)}</b> <Verdict ok={bbl?.primaryFossilMeets ?? null} /></Chip>}
          {summary.beng3 != null && <Chip>BENG 3 <b>{n(summary.beng3)} %</b> <Verdict ok={bbl?.renewableShareMeets ?? null} /></Chip>}
          {summary.tojuliApplies && summary.tojuliMaxK != null && (
            <Chip>TO<sub>juli</sub> <b>{n(summary.tojuliMaxK, 2)}</b> <Verdict ok={summary.tojuliMeetsLimit} /></Chip>
          )}
          {summary.labelClass && <Chip>{t('status.label')} <b>{summary.labelClass}</b></Chip>}
        </> : result && kernel?.verdict !== 'withheld' && (
          <span className="status-indicative">BENG: {t('results.indicative')}</span>
        )}
      </div>
      <div className="status-section status-right">
        <Pill tone="unv">{t('status.unattested')}</Pill>
        <span>{t(isDirty ? 'status.unsaved' : 'status.saved')}</span>
        <span className="status-optional" aria-label={t('settings.language')}>{locale.toUpperCase()}</span>
      </div>
    </footer>
  );
}
