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
import { Building3DView } from '../Building3DView/Building3DView';
import { HeatPumpInventoryPanel } from '../HeatPumpInventoryPanel/HeatPumpInventoryPanel';
import { GasChainReferencePanel } from '../GasChainReferencePanel/GasChainReferencePanel';
import { KernelAuditPanel } from '../KernelAuditPanel/KernelAuditPanel';
import { NtaPerformancePanel } from '../NtaPerformancePanel/NtaPerformancePanel';
import { ResultsDashboard } from './pages/results/ResultsDashboard';
import { BasisopnamePanel } from '../BasisopnamePanel/BasisopnamePanel';
import { MaatwerkadviesPanel } from '../MaatwerkadviesPanel/MaatwerkadviesPanel';
import { RelabelPanel } from '../MaatwerkadviesPanel/RelabelPanel';
import { ReportView } from '../ReportView/ReportView';
import { UValueCalculator } from '../UValueCalculator/UValueCalculator';
import { ThermalBridgeCalculator } from '../ThermalBridgeCalculator/ThermalBridgeCalculator';
import { HeatPumpSizingCalculator } from '../HeatPumpSizingCalculator/HeatPumpSizingCalculator';
import { PageHeader, SubTabs, routeLabel } from './PageHeader';
import { ProjectOverview } from './pages/ProjectOverview';
import { InstallationAddBar, InstallationsOverview, ServicePage, type ServiceId } from './pages/InstallationsPage';
import {
  AirTightnessPage, BuildingAddBar, BuildingLead, ConstructionsPage, EnvelopePage, ThermalBridgesPage, ZonesPage,
} from './pages/BuildingPages';
import { RegistrationEditButton, RegistrationPage } from './pages/RegistrationPage';
import { ShellActionsProvider, type ShellActions } from './ShellActions';
import { focusPathIn } from './focusPath';
import { NtaApplyBar, NtaDraftNotice, NtaStepSections } from './NtaStepPage';

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
  const dimmedBanner = status?.dimmed ? <Banner tone="info">{t('page.notApplicable')}</Banner> : null;

  let page: React.ReactNode;
  switch (route.step) {
    case 'project':
      page = <>
        <ProjectOverview project={project} statuses={statuses} actions={actions} />
        <div className="page-body page-body--nta"><NtaStepSections route={route} /></div>
      </>;
      break;
    case 'building':
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
          {route.sub === 'model3d' && <Building3DView />}
          {route.sub !== 'model3d' && <NtaStepSections route={route} />}
        </div>
      </>;
      break;
    case 'installations':
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
      page = <>{header()}<div className="page-body">{dimmedBanner}<BasisopnamePanel /></div></>;
      break;
    case 'advice':
      page = <>{header()}<div className="page-body">{dimmedBanner}<MaatwerkadviesPanel /></div></>;
      break;
    case 'relabel':
      page = <>{header()}<div className="page-body">{dimmedBanner}<RelabelPanel /></div></>;
      break;
    case 'report':
      page = <>
        {header({
          actions: <>
            <Button icon={<FileDown aria-hidden="true" />} onClick={actions.exportReport}>{t('report.export')}</Button>
            <Button icon={<Printer aria-hidden="true" />} onClick={actions.printReport}>{t('report.print')}</Button>
            <Button icon={<Download aria-hidden="true" />} onClick={actions.exportIFC}>{t('report.page.ifc')}</Button>
            <Button icon={<Download aria-hidden="true" />} onClick={actions.exportUNIEC3}>{t('ribbon.exportUNIEC3Draft')}</Button>
            <Button icon={<Download aria-hidden="true" />} onClick={actions.exportVABI}>{t('ribbon.exportVABI')}</Button>
          </>,
        })}
        <div className="page-body"><ReportView /></div>
      </>;
      break;
    case 'registration':
      page = <>
        {header({ actions: <RegistrationEditButton onOpen={() => actions.openDialog('project-info')} /> })}
        <div className="page-body"><RegistrationPage project={project} actions={actions} /></div>
      </>;
      break;
    case 'tool':
      page = <>
        {header({ lead: t('page.tool.lead') })}
        <div className="page-body">
          {route.sub === 'uvalue' && <UValueCalculator />}
          {route.sub === 'thermal-bridge' && <ThermalBridgeCalculator />}
          {route.sub === 'heat-pump-sizing' && <HeatPumpSizingCalculator />}
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
        <NtaApplyBar route={route} navigate={actions.navigate} />
      </ShellActionsProvider>
    </div>
  );
}
