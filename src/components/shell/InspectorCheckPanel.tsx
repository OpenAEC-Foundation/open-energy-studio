/**
 * Inspector › Controle (ontwerp §6.4, mockup 04): the kernel errors, points of
 * attention and omitted corrections, the findings of the open page first and
 * the rest grouped per step, each with "Ga naar" to its field.
 */
import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import { useKernel } from '../../context/KernelProvider';
import { projectCalculated } from '../../core/nta/KernelClient';
import { kernelNote } from '../../core/nta/KernelNoteText';
import { routeForPath } from '../../core/nta/gapRoutes';
import { kernelIssues, type StepIssue } from '../../core/nta/stepStatus';
import type { Route } from '../../core/navigation/routes';
import { IssueList, type IssueItem } from '../ui';
import { useShellActions } from './ShellActions';
import { routeLabel } from './PageHeader';

interface Routed extends StepIssue { route: Route }

/** Findings and omitted corrections of a kernel answer, the open page first. */
export function checkSummary(issues: StepIssue[], current: Route, omitted: string[]) {
  const routed: Routed[] = issues.map((issue) => ({ ...issue, route: routeForPath(issue.path) }));
  const onPage = (item: Routed) => item.route.step === current.step && (item.route.sub ?? null) === (current.sub ?? null);
  return {
    errors: issues.filter((issue) => issue.kind === 'error').length,
    warnings: issues.filter((issue) => issue.kind === 'warning').length,
    omitted: [...new Set(omitted)],
    here: routed.filter(onPage),
    elsewhere: routed.filter((item) => !onPage(item)),
  };
}

export function InspectorCheckPanel() {
  const { t, locale } = useI18n();
  const { state } = useEnergy();
  const kernel = useKernel();
  const actions = useShellActions();
  const assessment = kernel?.settled ?? null;
  const heating = projectCalculated(assessment?.status) ? assessment?.performance?.spaceHeating ?? null : null;
  const summary = checkSummary(kernelIssues(assessment), state.route,
    heating ? [...heating.demand.omittedCorrections, ...heating.omittedTerms] : []);
  const item = (issue: Routed): IssueItem => ({
    code: issue.code, severity: issue.kind, path: issue.path, detail: issue.detail, location: routeLabel(t, issue.route),
  });
  const goTo = actions ? (issue: IssueItem) => actions.navigate(routeForPath(issue.path)) : undefined;
  const groups = new Map<string, Routed[]>();
  for (const issue of summary.elsewhere) {
    const label = routeLabel(t, issue.route);
    groups.set(label, [...(groups.get(label) ?? []), issue]);
  }

  return (
    <section className="inspector-check" aria-labelledby="inspector-check-title">
      <h3 id="inspector-check-title" className="visually-hidden">{t('shell.inspector.check')}</h3>
      <dl className="inspector-check__tiles">
        <div className={summary.errors > 0 ? 'err' : undefined}><dt>{t('inspectorCheck.errors')}</dt><dd className="ui-num">{summary.errors}</dd></div>
        <div className={summary.warnings > 0 ? 'warn' : undefined}><dt>{t('inspectorCheck.warnings')}</dt><dd className="ui-num">{summary.warnings}</dd></div>
        <div><dt>{t('inspectorCheck.omitted')}</dt><dd className="ui-num">{summary.omitted.length}</dd></div>
      </dl>
      {!assessment
        ? <p className="inspector-check__hint">{t('inspectorCheck.noAnswer')}</p>
        : <p className="inspector-check__hint">{t('inspectorCheck.hint')}</p>}
      {assessment && summary.here.length + summary.elsewhere.length === 0 && <p className="inspector-check__none">{t('inspectorCheck.none')}</p>}
      {summary.here.length > 0 && <>
        <h4 className="inspector-check__group">{t('inspectorCheck.thisPage')} · {routeLabel(t, state.route)}</h4>
        <IssueList issues={summary.here.map(item)} onGoTo={goTo} />
      </>}
      {[...groups.entries()].map(([label, issues]) => <div key={label}>
        <h4 className="inspector-check__group">{label}</h4>
        <IssueList issues={issues.map(item)} onGoTo={goTo} />
      </div>)}
      {summary.omitted.length > 0 && <>
        <h4 className="inspector-check__group">{t('inspectorCheck.omittedTitle')}</h4>
        <ul className="inspector-check__omitted">
          {summary.omitted.map((note) => <li key={note}>{kernelNote(note, locale)}</li>)}
        </ul>
      </>}
    </section>
  );
}
