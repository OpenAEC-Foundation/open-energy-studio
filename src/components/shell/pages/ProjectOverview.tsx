/**
 * Step 1 Project: overview after mockup 01 — progress per step from the
 * kernel check, the open points with "Ga naar", the outcome and the project data.
 */
import { projectCalculated } from '../../../core/nta/KernelClient';
import type { ReactNode } from 'react';
import { AlertTriangle, ArrowRight, Check, ChevronRight, Pencil, Upload, X } from 'lucide-react';
import { useI18n } from '../../../i18n/i18n';
import { formatNumber } from '../../../i18n/format';
import { Button, Card, IssueList, Pill, type IssueItem } from '../../ui';
import { WORKFLOW_STEPS, type StepId } from '../../../core/navigation/routes';
import { routeForPath } from '../../../core/nta/gapRoutes';
import { kernelIssues, type StepStatus } from '../../../core/nta/stepStatus';
import { summarizeForPreview } from '../../../core/nta/PreviewSummary';
import { useKernel } from '../../../context/KernelProvider';
import type { IProject } from '../../../core/energy/types';
import type { ShellActions } from '../ShellActions';
import { PageHeader, routeLabel } from '../PageHeader';
import { stepStateText } from '../WorkflowNav';

const MAX_ISSUES = 12;

/** CSS class of the energy label colour. */
export function labelClassName(label: string | null | undefined): string {
  if (!label) return '';
  const plus = (label.match(/\+/g) ?? []).length;
  if (label.startsWith('A') && plus > 0) return plus === 1 ? 'lbl-ap' : `lbl-app${plus === 2 ? '2' : plus}`;
  return `lbl-${label.toLowerCase()}`;
}

function progressOf(status: StepStatus): number {
  switch (status.state) {
    case 'complete': return 100;
    case 'warnings': return status.hasInput ? 100 : 50;
    case 'errors': return 50;
    default: return 0;
  }
}

function BengRow({ name, value, limit, meets, unit, higherIsBetter = false, digits = 1 }: {
  name: ReactNode; value: number | null; limit: number | null; meets: boolean | null; unit: string; higherIsBetter?: boolean; digits?: number;
}) {
  const { t, locale } = useI18n();
  if (value == null) return null;
  const scale = Math.max(value, limit ?? 0, 1) * 1.15;
  const ok = meets ?? (limit == null ? null : higherIsBetter ? value >= limit : value <= limit);
  return (
    <div className="beng-bullet-row">
      <span className="name">{name}</span>
      <div className="bullet" aria-hidden="true">
        <span className={`fill${ok == null ? '' : ok ? ' ok' : ' fail'}`} style={{ width: `${Math.min(100, (value / scale) * 100)}%` }} />
        {limit != null && <span className="lim" style={{ left: `calc(${(limit / scale) * 100}% - 1px)` }} />}
      </div>
      <span className="value">
        <b>{formatNumber(value, locale, digits)}</b> <small>{unit}</small>{' '}
        {ok != null && <span className={ok ? 'ok' : 'fail'} title={t(ok ? 'overview.meets' : 'overview.failsLimit')}>
          {ok ? <Check aria-label={t('overview.meets')} /> : <X aria-label={t('overview.failsLimit')} />}
        </span>}
      </span>
    </div>
  );
}

export function ProjectOverview({ project, statuses, actions }: {
  project: IProject;
  statuses: Record<StepId, StepStatus>;
  actions: Pick<ShellActions, 'navigate' | 'openDialog' | 'importUNIEC3' | 'importVABI'>;
}) {
  const { t, locale } = useI18n();
  const kernel = useKernel();
  const assessment = kernel?.settled ?? null;
  const summary = assessment ? summarizeForPreview(assessment) : null;
  const calculated = projectCalculated(summary?.status);
  const issues = kernelIssues(assessment);
  const errors = issues.filter((issue) => issue.kind === 'error').length;
  const warnings = issues.length - errors;
  const geometry = assessment?.geometry ?? null;

  const surfaces = project.zones.reduce((sum, zone) => sum + zone.surfaces.length, 0);
  const windows = project.zones.reduce((sum, zone) => zone.surfaces.reduce((count, surface) => count + surface.windows.length, sum), 0);
  const bridges = project.zones.reduce((sum, zone) => sum + zone.thermalBridges.length, 0);
  const systemNames = [...project.heatingSystems, ...project.hotWaterSystems, ...project.ventilationSystems,
    ...project.coolingSystems, ...project.solarPV, ...project.solarThermal].map((system) => system.name);
  const summaries: Partial<Record<StepId, string>> = {
    project: [t(`function.${project.buildingFunction}`), project.address, project.city].filter(Boolean).join(' · '),
    building: `${t('browser.zones')} ${project.zones.length} · ${t('browser.surfaces')} ${surfaces} · ${t('browser.windows')} ${windows} · ${t('browser.thermalBridges')} ${bridges}`,
    installations: systemNames.length ? systemNames.slice(0, 4).join(' · ') + (systemNames.length > 4 ? ' …' : '') : t('installations.empty'),
    check: t(`overview.kernelStatus.${assessment?.status ?? 'none'}`),
    results: calculated
      ? [summary?.beng1 != null && `BENG 1 ${formatNumber(summary.beng1, locale, 1)}`, summary?.beng2 != null && `BENG 2 ${formatNumber(summary.beng2, locale, 1)}`,
        summary?.beng3 != null && `BENG 3 ${formatNumber(summary.beng3, locale, 1)} %`].filter(Boolean).join(' · ')
      : t('overview.noOutcome'),
  };

  const issueItems: IssueItem[] = issues.slice(0, MAX_ISSUES).map((issue) => ({
    code: issue.code,
    severity: issue.kind,
    path: issue.path,
    detail: issue.detail,
    location: routeLabel(t, routeForPath(issue.path)),
  }));

  return <>
    <PageHeader route={{ step: 'project' }} title={project.name || t('app.untitledProject')}
      lead={project.description || t('page.project.lead')}
      actions={<>
        <Button icon={<Pencil aria-hidden="true" />} onClick={() => actions.openDialog('project-info')}>{t('ribbon.projectInfo')}</Button>
        <Button icon={<Upload aria-hidden="true" />} onClick={actions.importUNIEC3}>{t('ribbon.importUNIEC3')}</Button>
        <Button icon={<Upload aria-hidden="true" />} onClick={actions.importVABI}>{t('ribbon.importVABI')}</Button>
      </>} />
    <div className="page-body">
      <div className="overview-grid">
        <Card className="overview-progress" title={t('overview.progress')} level={2}
          subtitle={kernel?.phase === 'stale' ? t('status.kernel.stale') : t('overview.fromKernel')}
          actions={<>
            {warnings > 0 && <Pill tone="warn" icon={<AlertTriangle aria-hidden="true" />}>{t('nav.state.warnings', { count: String(warnings) })}</Pill>}
            <Pill tone={errors > 0 ? 'err' : 'neutral'}>{t('nav.state.errors', { count: String(errors) })}</Pill>
          </>} flush>
          <table className="overview-table">
            <tbody>
              {WORKFLOW_STEPS.map((step) => {
                const status = statuses[step.id];
                const kind = status.state === 'complete' ? 'complete' : status.state === 'errors' ? 'errors' : status.state === 'warnings' ? 'warnings' : 'todo';
                return (
                  <tr key={step.id} className={status.dimmed ? 'is-dimmed' : undefined}>
                    <td style={{ width: 34 }}>
                      <span className={`nav-step-no ${kind}`} aria-hidden="true">{kind === 'complete' ? <Check /> : step.number}</span>
                    </td>
                    <td>
                      <div className="step-name">{t(step.labelKey)}</div>
                      <div className="step-sum">{summaries[step.id] ?? t(`page.${step.id}.lead`)}</div>
                    </td>
                    <td style={{ width: 180 }}>
                      <div className="overview-progress-bar" role="img" aria-label={`${progressOf(status)} %`}>
                        <i className={kind} style={{ width: `${progressOf(status)}%` }} />
                      </div>
                    </td>
                    <td className="step-state" style={{ width: 150 }}>
                      {status.errors > 0 ? <Pill tone="err">{stepStateText(t, status)}</Pill>
                        : status.warnings > 0 ? <Pill tone="warn">{stepStateText(t, status)}</Pill>
                          : stepStateText(t, status)}
                    </td>
                    <td style={{ width: 44 }}>
                      <button type="button" className="ui-iconbtn ui-iconbtn--sm" aria-label={`${t('overview.open')} ${t(step.labelKey)}`}
                        onClick={() => actions.navigate({ step: step.id })}><ChevronRight aria-hidden="true" /></button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
          <div className="overview-sub"><h3>{t('overview.openPoints')}</h3><span>{t('overview.openPointsHint')}</span></div>
          <IssueList issues={issueItems} empty={t('overview.noIssues')}
            onGoTo={(issue) => actions.navigate(routeForPath(issue.path))} />
          {issues.length > MAX_ISSUES && (
            <div className="overview-sub">
              <Button variant="ghost" size="sm" onClick={() => actions.navigate({ step: 'check', sub: 'overview' })}>
                {t('overview.moreIssues', { count: String(issues.length - MAX_ISSUES) })} <ArrowRight aria-hidden="true" />
              </Button>
            </div>
          )}
        </Card>

        <Card title={t('overview.outcome')} level={2}
          subtitle={<Pill tone="unv">{t('nta.performance.unverified')}</Pill>}
          actions={<Button variant="ghost" size="sm" onClick={() => actions.navigate({ step: 'results' })}>
            {t('nav.step.results')} <ArrowRight aria-hidden="true" /></Button>}>
          {calculated && summary ? (
            <div style={{ display: 'grid', gap: 14 }}>
              {summary.labelClass && (
                <div className="overview-label">
                  <span className={`label-badge ${labelClassName(summary.labelClass)}`}>{summary.labelClass}</span>
                  <div>
                    <div className="overview-overline">{t('overview.labelClass')}</div>
                    <div className="page-lead">{t('overview.notRegistered')}</div>
                  </div>
                </div>
              )}
              <div className="beng-bullets">
                <BengRow name="BENG 1" value={summary.beng1} limit={summary.beng1Limit} meets={assessment?.performance?.bblCheck?.energyNeedMeets ?? null} unit="kWh/m²" />
                <BengRow name="BENG 2" value={summary.beng2} limit={summary.beng2Limit} meets={assessment?.performance?.bblCheck?.primaryFossilMeets ?? null} unit="kWh/m²" />
                <BengRow name="BENG 3" value={summary.beng3} limit={summary.beng3Limit} meets={assessment?.performance?.bblCheck?.renewableShareMeets ?? null} unit="%" higherIsBetter />
                {summary.tojuliApplies && <BengRow name={<>TO<sub>juli</sub></>} value={summary.tojuliMaxK} limit={1.2} meets={summary.tojuliMeetsLimit} unit="K" digits={2} />}
              </div>
            </div>
          ) : (
            <p className="page-lead">{kernel?.phase === 'loading' ? t('status.kernel.loading')
              : assessment ? t(`overview.kernelStatus.${assessment.status}`) : t('overview.noOutcome')}</p>
          )}
        </Card>

        <Card title={t('overview.projectData')} level={2}
          actions={<Button variant="ghost" size="sm" icon={<Pencil aria-hidden="true" />} onClick={() => actions.openDialog('project-info')}>{t('overview.edit')}</Button>}>
          <table className="kv-table"><tbody>
            <tr><td>{t('overview.function')}</td><td>{t(`function.${project.buildingFunction}`)}</td></tr>
            <tr><td>{t('dialog.projectInfo.address')}</td><td>{[project.address, project.city].filter(Boolean).join(', ') || '—'}</td></tr>
            <tr><td>A<sub>g</sub> · A<sub>ls</sub>/A<sub>g</sub></td><td>{geometry
              ? `${formatNumber(geometry.usableFloorAreaM2, locale, 1)} m² · ${formatNumber(geometry.lossAreaRatio, locale, 3)}` : '—'}</td></tr>
            <tr><td>{t('overview.registrationType')}</td><td>{t(`overview.purpose.${project.registration?.purpose ?? 'none'}`)}</td></tr>
            {assessment && <tr><td>{t('overview.fingerprint')}</td><td className="mono" title={assessment.inputFingerprint}>
              {assessment.inputFingerprint.length > 20 ? `${assessment.inputFingerprint.slice(0, 15)}…${assessment.inputFingerprint.slice(-2)}` : assessment.inputFingerprint}</td></tr>}
          </tbody></table>
        </Card>
      </div>
    </div>
  </>;
}
