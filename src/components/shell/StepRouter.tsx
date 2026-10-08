/**
 * The work area of the shell: one page per workflow step (formerly MainView).
 * During F4 the steps show the existing views and panels (ontwerp §F4
 * "Tussenstand"); F5–F9 replace them page by page.
 */
import { useEffect, useRef } from 'react';
import { Box, Download, FileDown, Printer } from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import { useKernel } from '../../context/KernelProvider';
import { routeForPath } from '../../core/nta/gapRoutes';
import { kernelIssues, type StepStatus } from '../../core/nta/stepStatus';
import type { Route, StepId } from '../../core/navigation/routes';
import type { IProject } from '../../core/energy/types';
import { Banner, Button, Card, IssueList } from '../ui';
import { ErrorBoundary } from '../ErrorBoundary/ErrorBoundary';
import { UnheatedSpacesPanel } from '../UnheatedSpacesPanel/UnheatedSpacesPanel';
import { HeatPumpInventoryPanel } from '../HeatPumpInventoryPanel/HeatPumpInventoryPanel';
import { GasChainReferencePanel } from '../GasChainReferencePanel/GasChainReferencePanel';
import { KernelAuditPanel } from '../KernelAuditPanel/KernelAuditPanel';
import { NtaPerformancePanel } from '../NtaPerformancePanel/NtaPerformancePanel';
import { ResultsDashboard } from './pages/results/ResultsDashboard';
import { SurveyWizard } from '../SurveyWizard/SurveyWizard';
import { SurveyReport } from '../SurveyWizard/SurveyReport';
import { SurveyMwa } from '../SurveyWizard/SurveyMwa';
import { MaatwerkadviesPanel, type MwaTab } from '../MaatwerkadviesPanel/MaatwerkadviesPanel';
import { RelabelPanel } from '../MaatwerkadviesPanel/RelabelPanel';
import { SurveyRelabel } from '../SurveyWizard/SurveyRelabel';
import { Building3DView, HeatPumpSizingCalculator, LazyPage, ManualView, ThermalBridgeCalculator, UValueCalculator } from './lazyPages';
import { PageHeader, SubTabs, routeLabel } from './PageHeader';
import { ProjectOverview } from './pages/ProjectOverview';
import { InstallationAddBar, InstallationsOverview, ServicePage, type ServiceId } from './pages/InstallationsPage';
import {
  AirTightnessPage, BuildingAddBar, BuildingLead, ConstructionsPage, EnvelopePage, ThermalBridgesPage, ZonesPage,
} from './pages/BuildingPages';
import { RegistrationPage } from './pages/RegistrationPage';
import { DossierPage, ExportsPage, InputDossierPage, ReportPage } from './pages/DeliveryPages';
import { ShellActionsProvider, type ShellActions } from './ShellActions';
import { focusPathIn } from './focusPath';
import { NtaApplyBar, NtaDraftNotice, NtaStepSections } from './NtaStepPage';
import { BuildCheckPage, BuildFlowFrame, ProjectInfoQuestion } from '../BuildFlow/BuildFlow';
import { buildFlowSteps, buildQuestionOf } from '../../core/navigation/buildFlow';
import { isSurveyProject } from '../../core/nta/stepStatus';

export { focusPathIn } from './focusPath';

interface StepRouterProps {
  project: IProject;
  route: Route;
  statuses: Record<StepId, StepStatus>;
  actions: ShellActions;
}

export function StepRouter({ project, route, statuses, actions }: StepRouterProps) {
  const { t } = useI18n();
  const { state } = useEnergy();
  const kernel = useKernel();
  const bodyRef = useRef<HTMLDivElement>(null);
  const status = statuses[route.step];

  // Focus after navigation: the field of "Ga naar", else the page title (unless the
  // user is moving through the navigation itself).
  useEffect(() => {
    const container = bodyRef.current;
    if (!container) return;
    const frame = window.requestAnimationFrame(() => {
      if (route.focusPath && focusPathIn(container, route.focusPath)) return;
      const active = document.activeElement;
      if (active && active.closest('.workflow-nav, .page-subtabs, [role="dialog"]')) return;
      if (route.focusPath || active === document.body || active == null || !container.contains(active)) {
        container.querySelector<HTMLElement>('#page-title')?.focus({ preventScroll: true });
      }
    });
    return () => window.cancelAnimationFrame(frame);
  }, [route]);

  const navigateSub = (sub: string) => actions.navigate({ step: route.step, sub });
  const header = (extra?: { title?: React.ReactNode; actions?: React.ReactNode; lead?: React.ReactNode }) => <>
    <PageHeader route={route} title={extra?.title} lead={extra?.lead ?? t(`page.${route.step}.lead`)} actions={extra?.actions} />
    <SubTabs route={route} onSelect={navigateSub} />
  </>;
  // Projects without a basisopname enter their input as a question flow (UI redesign 2026-10).
  const flow = !isSurveyProject(project);
  const flowQuestion = flow ? buildQuestionOf(route, buildFlowSteps(project)) : null;
  const dimmedBanner = status?.dimmed ? <Banner tone="info">{t('page.notApplicable')}</Banner> : null;

  let page: React.ReactNode;
  switch (route.step) {
    case 'project':
      if (flow) {
        page = <div className="page-body"><BuildFlowFrame route={route}>
          <ProjectInfoQuestion />
          <NtaStepSections route={route} />
        </BuildFlowFrame></div>;
        break;
      }
      page = <>
        <ProjectOverview project={project} statuses={statuses} actions={actions} />
        <div className="page-body page-body--nta"><NtaStepSections route={route} /></div>
      </>;
      break;
    case 'building':
      if (flowQuestion) {
        page = <div className="page-body"><BuildFlowFrame route={route} toolbar={<>
          <Button size="sm" variant="ghost" icon={<Box aria-hidden="true" />}
            onClick={() => actions.navigate({ step: 'building', sub: 'model3d' })}>{t('building.open3d')}</Button>
          <BuildingAddBar sub={route.sub} />
        </>}>
          {route.sub === 'envelope' && <><p className="survey-muted"><BuildingLead sub="envelope" /></p><EnvelopePage /></>}
          {route.sub === 'zones' && <ZonesPage />}
          {route.sub === 'constructions' && <ConstructionsPage />}
          {route.sub === 'thermalBridges' && <ThermalBridgesPage />}
          {route.sub === 'airTightness' && <AirTightnessPage />}
          {route.sub === 'unheated' && <UnheatedSpacesPanel />}
          <NtaStepSections route={route} />
        </BuildFlowFrame></div>;
        break;
      }
      page = <>
        {header({
          lead: <BuildingLead sub={route.sub} />,
          actions: route.sub === 'model3d'
            ? <Button icon={<Box aria-hidden="true" />} onClick={actions.exportModelIFC}>{t('ribbon.exportModelIFC')}</Button>
            : <>
              <Button size="sm" variant="ghost" icon={<Box aria-hidden="true" />}
                onClick={() => actions.navigate({ step: 'building', sub: 'model3d' })}>{t('building.open3d')}</Button>
              <BuildingAddBar sub={route.sub} />
            </>,
        })}
        <div className={route.sub === 'model3d' ? 'page-body page-body--flush' : 'page-body'}>
          {route.sub === 'envelope' && <EnvelopePage />}
          {route.sub === 'zones' && <ZonesPage />}
          {route.sub === 'constructions' && <ConstructionsPage />}
          {route.sub === 'thermalBridges' && <ThermalBridgesPage />}
          {route.sub === 'airTightness' && <AirTightnessPage />}
          {route.sub === 'unheated' && <UnheatedSpacesPanel />}
          {route.sub === 'model3d' && <LazyPage><Building3DView /></LazyPage>}
          {route.sub !== 'model3d' && <NtaStepSections route={route} />}
        </div>
      </>;
      break;
    case 'installations':
      if (flowQuestion) {
        page = <div className="page-body"><BuildFlowFrame route={route} toolbar={<InstallationAddBar sub={route.sub} />}>
          <ServicePage key={route.sub} sub={route.sub as ServiceId} focusPath={route.focusPath} />
        </BuildFlowFrame></div>;
        break;
      }
      page = <>
        {header({
          lead: route.sub && route.sub !== 'systems' ? t(`lead.installations.${route.sub}`) : t('page.installations.lead'),
          actions: route.sub && route.sub !== 'systems' ? <InstallationAddBar sub={route.sub} /> : undefined,
        })}
        <div className="page-body">
          {(route.sub === 'systems' || !route.sub) && <InstallationsOverview />}
          {route.sub && route.sub !== 'systems' && route.sub !== 'heatPumps' && <ServicePage key={route.sub} sub={route.sub as ServiceId}
            focusPath={route.focusPath} />}
          {route.sub === 'heatPumps' && <ServicePage sub="heatPumps" extra={<>
            <HeatPumpInventoryPanel />
            <details className="diagnostic-card">
              <summary>{t('installations.gasReference')}</summary>
              <GasChainReferencePanel />
            </details>
          </>} />}
        </div>
      </>;
      break;
    case 'check': {
      if (flow && route.sub !== 'input') {
        page = <div className="page-body"><BuildCheckPage statuses={statuses} /></div>;
        break;
      }
      const issues = kernelIssues(kernel?.settled);
      page = <>
        {header()}
        <div className="page-body">
          {route.sub === 'overview' && <>
            <Card title={t('overview.openPoints')} subtitle={t('overview.openPointsHint')} level={2} flush>
              <IssueList empty={t('overview.noIssues')}
                issues={issues.map((issue) => ({
                  code: issue.code, severity: issue.kind, path: issue.path, detail: issue.detail,
                  location: routeLabel(t, routeForPath(issue.path)),
                }))}
                onGoTo={(issue) => actions.navigate(routeForPath(issue.path))} />
            </Card>
            <KernelAuditPanel project={project} />
          </>}
          {route.sub === 'input' && <>
            <NtaDraftNotice />
            <NtaStepSections route={route} />
            <NtaPerformancePanel />
          </>}
        </div>
      </>;
      break;
    }
    case 'results':
      page = <>
        {header()}
        <div className="page-body"><ResultsDashboard sub={route.sub} onNavigate={actions.navigate} /></div>
      </>;
      break;
    case 'survey':
      // The basisopname question flow: the steps are in the navigation, the page shows one question.
      page = <>
        {/* The question page has its own h1 (the question) and step line. */}
        <div className="page-body">{dimmedBanner}
          <SurveyWizard route={route} navigate={actions.navigate} />
        </div>
      </>;
      break;
    case 'advice':
      // A basisopname project takes its measures as changes to the survey.
      if (!flow) {
        page = <div className="page-body">{dimmedBanner}<SurveyMwa route={route} navigate={actions.navigate} /></div>;
        break;
      }
      page = <>{header()}<div className="page-body">{dimmedBanner}<MaatwerkadviesPanel tab={route.sub as MwaTab | undefined} /></div></>;
      break;
    case 'relabel':
      // A basisopname project compares its survey with the original label's survey.
      if (!flow) {
        page = <div className="page-body"><SurveyRelabel actions={actions} /></div>;
        break;
      }
      page = <>{header()}<div className="page-body">{dimmedBanner}<RelabelPanel /></div></>;
      break;
    case 'report':
      // A basisopname project reports its survey outcome (the new-build calculation is empty there).
      if (!flow) {
        page = <div className="page-body"><SurveyReport actions={actions} /></div>;
        break;
      }
      page = <>
        {header({
          actions: <>
            <Button icon={<FileDown aria-hidden="true" />} onClick={actions.exportReport}>{t('report.export')}</Button>
            <Button icon={<Printer aria-hidden="true" />} onClick={actions.printReport}>{t('report.print')}</Button>
            {route.sub !== 'exports' && (
              <Button variant="ghost" icon={<Download aria-hidden="true" />}
                onClick={() => actions.navigate({ step: 'report', sub: 'exports' })}>{t('report.page.exports')}</Button>
            )}
          </>,
        })}
        <div className="page-body page-body--delivery">
          {(route.sub === 'report' || !route.sub) && <ReportPage />}
          {route.sub === 'input' && <InputDossierPage />}
          {route.sub === 'checklist' && <DossierPage actions={actions} />}
          {route.sub === 'exports' && <ExportsPage actions={actions} />}
        </div>
      </>;
      break;
    case 'registration':
      page = <>
        {header()}
        <div className="page-body"><RegistrationPage project={project} actions={actions} /></div>
      </>;
      break;
    case 'tool':
      page = <>
        {header({ lead: t(route.sub === 'manual' ? 'manual.lead' : 'page.tool.lead') })}
        <div className="page-body">
          <LazyPage>
            {route.sub === 'manual' && (
              <ManualView chapterRef={route.chapter} onOpen={(chapter) => actions.navigate({ step: 'tool', sub: 'manual', chapter })} />
            )}
            {route.sub === 'uvalue' && <UValueCalculator />}
            {route.sub === 'thermal-bridge' && <ThermalBridgeCalculator />}
            {route.sub === 'heat-pump-sizing' && <HeatPumpSizingCalculator />}
          </LazyPage>
        </div>
      </>;
      break;
    default:
      page = null;
  }

  return (
    <div className="step-page" ref={bodyRef} data-step={route.step} style={{ display: 'contents' }}>
      <ShellActionsProvider value={actions}>
        <ErrorBoundary resetKey={state.project}>{page}</ErrorBoundary>
        {!flowQuestion && <NtaApplyBar route={route} navigate={actions.navigate} />}
      </ShellActionsProvider>
    </div>
  );
}
