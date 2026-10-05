import { useEffect, useRef, useState } from 'react';
import { ArrowRight, FolderOpen } from 'lucide-react';
import { useEnergy } from '../../context/EnergyContext';
import { useKernel } from '../../context/KernelProvider';
import { useI18n } from '../../i18n/i18n';
import { Banner, Button, Card, FileButton, IconButton, IssueList, Pill, Segmented, Stepper, StatusPill, type StepStatus } from '../ui';
import { useShellActions } from '../shell/ShellActions';
import { routeLabel } from '../shell/PageHeader';
import { assessRelabelWithRust, type NtaRelabelComparison, type NtaRelabelProof, type RelabelChange } from '../../core/nta/KernelClient';
import { deserializeProject } from '../../core/io/ProjectSerializer';
import { sha256Hex } from '../../core/nta/Evidence';
import { labelInputSha256, originalProjectTextForStorage } from '../../core/nta/Registration';
import { relabelCluster, relabelElementName, relabelNote, relabelValue } from '../../core/nta/RelabelText';
import { dutchSource } from '../../core/nta/OpnameValueText';
import { hasExplicitRoute, routeForPath } from '../../core/nta/gapRoutes';
import '../NtaPerformancePanel/NtaPerformancePanel.css';
import './MaatwerkadviesPanel.css';
import '../shell/pages/existing/existing.css';

/** The steps of a relabel (F8): original file → comparison → changes → evidence roles → readiness. */
export const RELABEL_STEPS = ['original', 'comparison', 'changes', 'evidence', 'readiness'] as const;
export type RelabelStep = typeof RELABEL_STEPS[number];

type VerdictFilter = 'all' | RelabelChange['verdict'];

const PROOF_ROLES: NtaRelabelProof[] = ['quote_with_order', 'specified_invoice', 'production_photo', 'review'];

const VERDICT_TONE = { allowed: 'ok', not_allowed: 'err', review: 'warn' } as const;

/** Evidence files per relabel role (BRL 9500-W §4.2.3). */
export function relabelProofCounts(evidence: Array<{ relabelProof?: NtaRelabelProof }> | undefined): Record<NtaRelabelProof, number> {
  const counts: Record<NtaRelabelProof, number> = { quote_with_order: 0, specified_invoice: 0, production_photo: 0, review: 0 };
  for (const item of evidence ?? []) if (item.relabelProof) counts[item.relabelProof] += 1;
  return counts;
}

/**
 * BRL 9500 Bijlage 6a/6b: compares the project with the original label's project file.
 * The comparison is kept with the registration, so the project dossier carries the
 * overview of later changes (Bijlage 3).
 */
export function RelabelPanel() {
  const { state, dispatch } = useEnergy();
  const { t, locale } = useI18n();
  const actions = useShellActions();
  const registration = useKernel()?.settled?.registration ?? null;
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [currentSha, setCurrentSha] = useState<string | null>(null);
  const [filter, setFilter] = useState<VerdictFilter>('all');
  const requestId = useRef(0);
  const fileInput = useRef<HTMLInputElement>(null);
  const stored: NtaRelabelComparison | undefined = state.project.registration?.relabelComparison;
  const result = stored?.assessment;

  useEffect(() => {
    // A comparison still running belongs to the project as it was; drop it.
    requestId.current += 1;
    setBusy(false);
    let active = true;
    void labelInputSha256(state.project).then((sha) => { if (active) setCurrentSha(sha); }).catch(() => undefined);
    return () => { active = false; };
  }, [state.project]);

  const store = (relabelComparison: NtaRelabelComparison | undefined) => {
    const registrationData = { ...(state.project.registration ?? {}) };
    if (relabelComparison) registrationData.relabelComparison = relabelComparison;
    else delete registrationData.relabelComparison;
    dispatch({ type: 'UPDATE_PROJECT_INFO', payload: { registration: registrationData } });
  };

  const compare = async (file: File) => {
    const current = ++requestId.current;
    setError(null);
    setBusy(true);
    try {
      // A relabel comparison inside the original (with its own original
      // file) is left out, so originals do not nest.
      const text = originalProjectTextForStorage(await file.text());
      const original = deserializeProject(text);
      const assessment = await assessRelabelWithRust(original, state.project);
      const [originalSha256, currentSha256] = await Promise.all([
        sha256Hex(new TextEncoder().encode(text)),
        assessment.currentLabelInputHash ?? labelInputSha256(state.project),
      ]);
      if (requestId.current === current) {
        store({ originalFileName: file.name, originalSha256, currentSha256, comparedAt: new Date().toISOString(),
          originalProjectText: text, assessment });
      }
    } catch (reason) {
      if (requestId.current === current) setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      if (requestId.current === current) setBusy(false);
      if (fileInput.current) fileInput.current.value = '';
    }
  };

  const outdated = Boolean(stored?.currentSha256 && currentSha && stored.currentSha256 !== currentSha);
  const changes = result?.changes ?? [];
  const count = (verdict: RelabelChange['verdict']) => changes.filter((change) => change.verdict === verdict).length;
  const proofs = relabelProofCounts(state.project.registration?.evidence);
  const relabelIssues = registration?.issues.filter((issue) => issue.code.startsWith('relabel_')) ?? [];
  const recheck = registration?.relabelAssessment;
  const shown = filter === 'all' ? changes : changes.filter((change) => change.verdict === filter);

  const status: Record<RelabelStep, StepStatus> = {
    original: stored ? 'done' : 'current',
    comparison: !result ? 'todo' : outdated ? 'warn' : 'done',
    changes: !result ? 'todo' : count('not_allowed') > 0 ? 'error' : count('review') > 0 ? 'warn' : 'done',
    evidence: proofs.review > 0 ? 'warn' : proofs.quote_with_order + proofs.specified_invoice > 0 ? 'done' : result ? 'warn' : 'todo',
    readiness: !registration || !result ? 'todo' : relabelIssues.length > 0 ? 'error' : registration.readyForRegistration ? 'done' : 'warn',
  };
  const statusText = (step: RelabelStep) => t(`relabel.page.status.${status[step]}`);
  const goToStep = (step: RelabelStep) => {
    const target = document.getElementById(`relabel-step-${step}`);
    target?.scrollIntoView?.({ block: 'start', behavior: 'smooth' });
    target?.focus({ preventScroll: true });
  };
  const stepTitle = (step: RelabelStep) => `${RELABEL_STEPS.indexOf(step) + 1} · ${t(`relabel.page.step.${step}`)}`;
  const openEvidence = () => actions?.openDialog('project-info');

  return (
    <section className="relabel-page" aria-label={t('relabel.title')}>
      <p className="relabel-page__intro">{t('relabel.intro')}</p>
      <Stepper aria-label={t('relabel.page.steps')} onSelect={goToStep} className="relabel-stepper"
        steps={RELABEL_STEPS.map((step) => ({ id: step, label: t(`relabel.page.step.${step}`), status: status[step], statusText: statusText(step) }))} />
      {proofs.review > 0 && <Banner tone="warn" title={t('app.toast.relabelMigrated')}
        action={actions && <Button size="sm" onClick={openEvidence}>{t('relabel.page.openEvidence')}</Button>}>
        {t('relabel.page.reviewInvoices', { count: proofs.review })}</Banner>}
      {error && <Banner tone="err">{error}</Banner>}

      <div id="relabel-step-original" className="relabel-step" tabIndex={-1}>
        <Card title={stepTitle('original')} aria-label={stepTitle('original')} subtitle={t('relabel.page.originalSub')} level={2}>
          <div className="relabel-file">
            <label htmlFor="relabel-original-file">{t('relabel.choose')}</label>
            <FileButton id="relabel-original-file" inputRef={fileInput} accept=".json,.oes,application/json" disabled={busy}
              icon={<FolderOpen aria-hidden="true" />} label={t('relabel.page.chooseFile')}
              onChange={(event) => { const file = event.target.files?.[0]; if (file) void compare(file); }} />
            {busy && <span className="relabel-busy" role="status">{t('relabel.page.comparing')}</span>}
          </div>
          {stored && <dl className="relabel-facts">
            <div><dt>{t('relabel.page.fileName')}</dt><dd>{stored.originalFileName}</dd></div>
            {stored.comparedAt && <div><dt>{t('relabel.page.comparedAt')}</dt>
              <dd><time dateTime={stored.comparedAt}>{new Date(stored.comparedAt).toLocaleString(locale)}</time></dd></div>}
            {stored.originalSha256 && <div><dt>SHA-256</dt><dd className="mono" title={stored.originalSha256}>{stored.originalSha256.slice(0, 16)}…</dd></div>}
          </dl>}
        </Card>
      </div>

      <div id="relabel-step-comparison" className="relabel-step" tabIndex={-1}>
        <Card title={stepTitle('comparison')} aria-label={stepTitle('comparison')} level={2}
          actions={stored && <Button size="sm" variant="ghost" onClick={() => store(undefined)}>{t('relabel.clear')}</Button>}>
          {!result ? <p className="nta-form-note">{t('relabel.page.noComparison')}</p> : <>
            <p className="relabel-verdict">
              <StatusPill tone={result.allowed ? 'ok' : 'err'}>{result.allowed ? t('relabel.allowed') : t('relabel.notAllowed')}</StatusPill>
              {result.needsReview && <StatusPill tone="warn">{t('relabel.review')}</StatusPill>}
              <Pill>{result.scheme === 'u' ? 'BRL 9500-U' : 'BRL 9500-W'}</Pill>
              <span className="relabel-source">{locale.toLowerCase().startsWith('nl') ? dutchSource(result.source) : result.source}</span>
            </p>
            <p className="dialog-hint">{t('relabel.stored')}</p>
            {outdated && <Banner tone="warn" role="status" className="relabel-review">
              <span data-testid="relabel-outdated">{t('relabel.outdated')}</span></Banner>}
          </>}
        </Card>
      </div>

      <div id="relabel-step-changes" className="relabel-step" tabIndex={-1}>
        <Card title={stepTitle('changes')} aria-label={stepTitle('changes')} subtitle={t('relabel.page.changesSub')} level={2} flush
          actions={changes.length > 0 && <Segmented<VerdictFilter> size="sm" aria-label={t('relabel.page.filter')} value={filter} onChange={setFilter}
            options={[
              { value: 'all', label: `${t('relabel.page.filter.all')} ${changes.length}` },
              { value: 'allowed', label: `6a ${count('allowed')}` },
              { value: 'not_allowed', label: `6b ${count('not_allowed')}` },
              { value: 'review', label: `${t('relabel.verdict.review')} ${count('review')}` },
            ]} />}>
          {!result ? <p className="nta-form-note relabel-pad">{t('relabel.page.noComparison')}</p>
            : changes.length === 0 ? <p className="relabel-pad">{t('relabel.noChanges')}</p> : (
              <div className="ui-table-wrap">
                <table className="ui-table relabel-table">
                  <thead><tr><th scope="col">{t('relabel.col.verdict')} · {t('relabel.col.cluster')}</th><th scope="col">{t('relabel.col.path')}</th>
                    <th scope="col">{t('relabel.col.before')}</th>
                    <th scope="col">{t('relabel.col.after')}</th><th scope="col">{t('relabel.col.note')}</th>
                    {actions && <th scope="col"><span className="visually-hidden">{t('relabel.page.goTo')}</span></th>}</tr></thead>
                  <tbody>
                    {shown.map((change, index) => (
                      <tr key={index} className={`relabel-row relabel-row--${change.verdict}`}>
                        <td className={`relabel-${change.verdict}`}>
                          <StatusPill tone={VERDICT_TONE[change.verdict]}>{t(`relabel.verdict.${change.verdict}`)}</StatusPill>
                          <small className="relabel-cluster">{relabelCluster(t, change.cluster)}</small></td>
                        <td title={change.path}><strong>{relabelElementName(state.project, change.path)}</strong>
                          <small className="relabel-path mono">{change.path}</small></td>
                        <td>{relabelValue(t, locale, change.before)}</td>
                        <td>{relabelValue(t, locale, change.after)}</td>
                        <td className="relabel-note">{relabelNote(t, change.note)}</td>
                        {actions && <td>{hasExplicitRoute(change.path) && (
                          <IconButton size="sm" icon={<ArrowRight aria-hidden="true" />}
                            aria-label={t('relabel.page.goToItem', { place: routeLabel(t, routeForPath(change.path)) })}
                            onClick={() => actions.navigate(routeForPath(change.path))} />)}</td>}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
        </Card>
      </div>

      <div id="relabel-step-evidence" className="relabel-step" tabIndex={-1}>
        <Card title={stepTitle('evidence')} aria-label={stepTitle('evidence')} subtitle={t('relabel.page.evidenceSub')} level={2}
          actions={actions && <Button size="sm" onClick={openEvidence}>{t('relabel.page.openEvidence')}</Button>}>
          <ul className="relabel-proofs">
            {PROOF_ROLES.map((role) => (
              <li key={role}>
                <span>{t(`evidence.relabelProof.${role}`)}</span>
                {role === 'review'
                  ? proofs.review > 0 ? <StatusPill tone="warn">{proofs.review}</StatusPill> : <Pill>0</Pill>
                  : proofs[role] > 0 ? <StatusPill tone="ok">{proofs[role]}</StatusPill> : <Pill>0</Pill>}
              </li>
            ))}
          </ul>
        </Card>
      </div>

      <div id="relabel-step-readiness" className="relabel-step" tabIndex={-1}>
        <Card title={stepTitle('readiness')} aria-label={stepTitle('readiness')} subtitle={t('relabel.page.readinessSub')} level={2}>
          {!registration ? <p className="nta-form-note">{t('relabel.page.noKernel')}</p> : <>
            <table className="kv-table"><tbody>
              <tr><td>{t('relabel.page.recheck')}</td><td>{recheck == null
                ? <Pill tone="unv">{t('relabel.page.recheckMissing')}</Pill>
                : <StatusPill tone={recheck.allowed ? 'ok' : 'err'}>
                  {t(recheck.allowed ? 'relabel.page.recheckAllowed' : 'relabel.page.recheckNotAllowed')}</StatusPill>}
                {recheck && result && recheck.allowed !== result.allowed && <> <StatusPill tone="warn">{t('relabel.page.recheckDiffers')}</StatusPill></>}
              </td></tr>
              <tr><td>{t('registration.page.ready')}</td><td>{registration.readyForRegistration
                ? <StatusPill tone="ok">{t('registration.page.yes')}</StatusPill>
                : <StatusPill tone="warn">{t('registration.page.no')}</StatusPill>}</td></tr>
            </tbody></table>
            <IssueList prefixes={['registration.issue.']} empty={t('relabel.page.noIssues')}
              issues={relabelIssues.map((issue) => ({
                code: issue.code, severity: 'error' as const, path: issue.path, location: routeLabel(t, routeForPath(issue.path)),
              }))}
              onGoTo={actions ? (issue) => {
                const route = routeForPath(issue.path);
                if (route.step === 'registration' || route.step === 'project') actions.openDialog('project-info');
                else if (route.step === 'relabel') goToStep('original');
                else actions.navigate(route);
              } : undefined} />
          </>}
        </Card>
      </div>
    </section>
  );
}
