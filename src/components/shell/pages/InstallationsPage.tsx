/**
 * Step 3 Installaties (UI redesign F5, mockup 03). The overview shows one
 * chain card per service with its key figures and the status from the kernel
 * findings; each service has its own sub page with the systems of the project
 * model and, since F6, the NTA 8800 sections of the service on the shared
 * draft (`NtaStepSections`). Click selects
 * (inspector), double click or Enter opens the editor as a side sheet.
 */
import type { ReactNode } from 'react';
import { ArrowRight, Plus } from 'lucide-react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { useKernel } from '../../../context/KernelProvider';
import { editAction } from '../../../core/energy/projectItems';
import { formatKernelPath, isPathWithin } from '../../../core/nta/pathUtil';
import { routeForPath } from '../../../core/nta/gapRoutes';
import { kernelIssues, type StepIssue } from '../../../core/nta/stepStatus';
import { SYSTEM_LISTS, type SystemListKey } from '../../../core/navigation/projectPaths';
import type {
  DialogType, ICoolingSystem, IHeatingSystem, IHotWaterSystem, IProject, ISolarPV, ISolarThermal, IVentilationSystem,
} from '../../../core/energy/types';
import { formatNumber } from '../../../i18n/format';
import { Banner, Button, Card, DataTable, IssueList, StatusPill, Tag, type Column } from '../../ui';
import { ItemActions } from '../../ItemActions/ItemActions';
import { useShellActions } from '../ShellActions';
import { routeLabel } from '../PageHeader';
import { HEATING_CHAIN, NtaStepSections, PART_KEYS, heatingPartFilled } from '../NtaStepPage';
import { useNtaDraft } from '../../../context/NtaDraftProvider';
import type { Draft } from '../../NtaPerformancePanel/NtaFormFields';

/**
 * Installed PV peak power: from the NTA input when it has PV systems (16.4a/16.4b),
 * otherwise from the simplified model. A table 16.1 system has no peak power of
 * its own in the input; the total is then marked as partial.
 */
export function projectPvPeak(project: IProject): { kwp: number; partial: boolean } {
  const systems = project.ntaCalculation?.pvSystems ?? [];
  if (systems.length === 0) return { kwp: project.solarPV.reduce((sum, pv) => sum + pv.peakPower, 0), partial: false };
  let watts = 0;
  let partial = false;
  for (const system of systems) {
    const peak = system.peakPower;
    if (peak.method === 'panels') watts += (peak.panelPeakPowerW ?? 0) * (peak.panelCount ?? 0);
    else if (peak.method === 'declared_specific') watts += (peak.peakPowerWPerM2 ?? 0) * (peak.panelAreaM2 ?? 0);
    else partial = true;
  }
  return { kwp: watts / 1000, partial };
}

export type ServiceId = 'heating' | 'hotWater' | 'ventilation' | 'cooling' | 'humidification' | 'lighting' | 'generation' | 'heatPumps' | 'bacs';

interface ServiceDef {
  id: ServiceId;
  /** NTA 8800 chapter or paragraph of the service. */
  ref: string;
  lists: SystemListKey[];
  /** NTA input blocks of the service (its chain card; the sections are edited on its sub page). */
  ntaKeys: string[];
  /** Kernel path prefixes whose findings count for this service. */
  paths: string[];
  add: Array<{ dialog: DialogType; labelKey: string }>;
}

export const SERVICES: ServiceDef[] = [
  {
    id: 'heating', ref: '§9', lists: ['heatingSystems'],
    ntaKeys: ['generator', 'emission', 'distribution', 'distributionSystem', 'additionalHeatingSystems', 'heatingSystems', 'spaceHeatingSolar'],
    paths: ['heatingSystems', 'ntaCalculation.generator', 'ntaCalculation.emission', 'ntaCalculation.distribution',
      'ntaCalculation.distributionSystem', 'ntaCalculation.additionalHeatingSystems', 'ntaCalculation.verticalPipes', 'ntaCalculation.heatingSystems',
      'ntaCalculation.identicalSystems', 'ntaCalculation.collectiveConnection', 'ntaCalculation.heatPumpRenewable', 'ntaCalculation.spaceHeatingSolar'],
    add: [{ dialog: 'heating-system', labelKey: 'ribbon.addHeating' }],
  },
  {
    id: 'hotWater', ref: '§13', lists: ['hotWaterSystems'], ntaKeys: ['hotWater', 'additionalHotWaterSystems'],
    paths: ['hotWaterSystems', 'ntaCalculation.hotWater', 'ntaCalculation.additionalHotWaterSystems'],
    add: [{ dialog: 'hot-water-system', labelKey: 'ribbon.addHotWater' }],
  },
  {
    id: 'ventilation', ref: '§11', lists: ['ventilationSystems'], ntaKeys: ['ventilation', 'ventilationFlows'],
    paths: ['ventilationSystems', 'ntaCalculation.ventilation', 'ntaCalculation.ventilationFlows', 'ntaCalculation.demandUsesFixedC1Ventilation'],
    add: [{ dialog: 'ventilation-system', labelKey: 'ribbon.addVentilation' }],
  },
  {
    id: 'cooling', ref: '§10', lists: ['coolingSystems'], ntaKeys: ['activeCooling', 'cooling', 'coolingSystems'],
    paths: ['coolingSystems', 'ntaCalculation.cooling', 'ntaCalculation.coolingSystems', 'ntaCalculation.activeCooling'],
    add: [{ dialog: 'cooling-system', labelKey: 'ribbon.addCooling' }],
  },
  { id: 'humidification', ref: '§12', lists: [], ntaKeys: ['humidifiers'], paths: ['ntaCalculation.humidifiers'], add: [] },
  { id: 'lighting', ref: '§14', lists: [], ntaKeys: ['lighting'], paths: ['ntaCalculation.lighting'], add: [] },
  {
    id: 'generation', ref: '§16', lists: ['solarPV', 'solarThermal'], ntaKeys: ['pvSystems', 'externalSupply', 'onSiteProduction', 'declaredUses', 'storage'],
    paths: ['solarPV', 'solarThermal', 'ntaCalculation.pvSystems', 'ntaCalculation.externalSupply', 'ntaCalculation.onSiteProduction',
      'ntaCalculation.declaredUses', 'ntaCalculation.declaredRenewableHeat', 'ntaCalculation.batteryStoragePresent', 'ntaCalculation.storage'],
    add: [{ dialog: 'solar-pv', labelKey: 'ribbon.addSolarPV' }, { dialog: 'solar-thermal', labelKey: 'ribbon.addSolarThermal' }],
  },
  { id: 'heatPumps', ref: '§9.6', lists: [], ntaKeys: [], paths: ['ntaHeatPumps'], add: [] },
  { id: 'bacs', ref: '§5.5.8', lists: [], ntaKeys: ['bacs', 'bacsFactor'], paths: ['ntaCalculation.bacs', 'ntaCalculation.bacsFactor', 'ntaCalculation.bacsSourceReference'], add: [] },
];

export const serviceDef = (id: string | undefined) => SERVICES.find((service) => service.id === id);

/** True when an NTA input block has content (a non-empty array or a set value). */
function hasBlock(value: unknown): boolean {
  if (Array.isArray(value)) return value.length > 0;
  return value != null;
}

export function ntaBlocks(project: IProject, service: ServiceDef): string[] {
  const nta = project.ntaCalculation as Record<string, unknown> | undefined;
  if (!nta) return [];
  return service.ntaKeys.filter((key) => hasBlock(nta[key]));
}

export function serviceIssues(issues: StepIssue[], service: ServiceDef): StepIssue[] {
  return issues.filter((issue) => service.paths.some((prefix) => isPathWithin(issue.path, prefix)));
}

/** The service has systems in the project model or input in the NTA calculation. */
export function servicePresent(project: IProject, service: ServiceDef): boolean {
  if (service.id === 'heatPumps') return (project.ntaHeatPumps ?? []).length > 0;
  return service.lists.some((key) => (project[key] as unknown[]).length > 0) || ntaBlocks(project, service).length > 0;
}

// ── Key figures per system ──────────────────────────────────────────

type SystemItem = { id: string; name: string };

function useSystemFigures() {
  const { t, locale } = useI18n();
  const n = (value: number | null | undefined, digits = 2) => formatNumber(value, locale, digits);
  const label = (key: string, fallback: string) => { const text = t(key); return text === key ? fallback : text; };
  return (key: SystemListKey, item: SystemItem): Array<[string, string]> => {
    switch (key) {
      case 'heatingSystems': {
        const system = item as IHeatingSystem;
        return [[t('installations.fig.type'), label(`report.heatingType.${system.type}`, system.type)],
          [system.type.startsWith('heat_pump') ? 'COP' : 'η', n(system.cop)],
          [t('installations.fig.coverage'), `${n(system.coverageFraction * 100, 0)} %`]];
      }
      case 'ventilationSystems': {
        const system = item as IVentilationSystem;
        return [[t('installations.fig.type'), label(`report.ventilationType.${system.type}`, system.type)],
          [t('installations.fig.heatRecovery'), `${n(system.heatRecoveryEfficiency * 100, 0)} %`],
          ['SFP', `${n(system.sfp)} W/(dm³/s)`]];
      }
      case 'coolingSystems': {
        const system = item as ICoolingSystem;
        return [[t('installations.fig.type'), label(`installations.coolingType.${system.type}`, system.type)], ['EER', n(system.eer)]];
      }
      case 'hotWaterSystems': {
        const system = item as IHotWaterSystem;
        return [[t('installations.fig.type'), label(`installations.hotWaterType.${system.type}`, system.type)],
          ['η', n(system.efficiency)],
          ...(system.hasSolarBoiler ? [[t('installations.fig.solarFraction'), `${n(system.solarBoilerFraction * 100, 0)} %`] as [string, string]] : [])];
      }
      case 'solarPV': {
        const system = item as ISolarPV;
        return [['P_pk', `${n(system.peakPower)} kWp`], [t('properties.orientation'), `${t(`orientation.${system.orientation}`)} · ${n(system.tilt, 0)}°`],
          [t('properties.area'), `${n(system.area, 1)} m²`]];
      }
      case 'solarThermal': {
        const system = item as ISolarThermal;
        return [[t('installations.fig.collector'), `${n(system.collectorArea, 1)} m²`],
          [t('properties.orientation'), `${t(`orientation.${system.orientation}`)} · ${n(system.tilt, 0)}°`]];
      }
    }
  };
}

function ServiceStatus({ issues, present }: { issues: StepIssue[]; present: boolean }) {
  const { t } = useI18n();
  const errors = issues.filter((issue) => issue.kind === 'error').length;
  const warnings = issues.length - errors;
  if (errors > 0) return <StatusPill tone="err">{t(errors === 1 ? 'installations.status.open.one' : 'installations.status.open.other').replace('{n}', String(errors))}</StatusPill>;
  if (warnings > 0) return <StatusPill tone="warn">{t(warnings === 1 ? 'installations.status.warn.one' : 'installations.status.warn.other').replace('{n}', String(warnings))}</StatusPill>;
  if (!present) return null;
  return <StatusPill tone="ok">{t('installations.status.ok')}</StatusPill>;
}

function useOpenDialog() {
  const actions = useShellActions();
  const { dispatch } = useEnergy();
  return (dialog: DialogType) => {
    if (actions) actions.openDialog(dialog);
    else dispatch({ type: 'OPEN_DIALOG', payload: { type: dialog } });
  };
}

/** The add buttons of a service (all services on the overview); names and titles are the former ribbon labels. */
export function InstallationAddBar({ sub }: { sub?: string }) {
  const { t } = useI18n();
  const open = useOpenDialog();
  const service = serviceDef(sub);
  const entries = service ? service.add : SERVICES.flatMap((item) => item.add);
  if (entries.length === 0) return null;
  return (
    <div className="add-bar" role="group" aria-label={t('installations.add')}>
      {entries.map((entry) => (
        <Button key={entry.dialog} size="sm" variant={service ? 'primary' : undefined} icon={<Plus aria-hidden="true" />} title={t(entry.labelKey)}
          onClick={() => open(entry.dialog)}>{t(entry.labelKey)}</Button>
      ))}
    </div>
  );
}

// ── Overview: chain cards ───────────────────────────────────────────

/**
 * The heating chain of mockup 03: opwekking › distributie › afgifte › regeling,
 * each phase marked filled or empty in the NTA input (the open draft, else the
 * applied block) and opening that part of the heating stepper.
 */
function HeatingChain({ project }: { project: IProject }) {
  const { t } = useI18n();
  const actions = useShellActions();
  const shared = useNtaDraft();
  const nta = shared?.draft ?? (project.ntaCalculation as unknown as Draft | undefined);
  return (
    <ol className="chain-phases" aria-label={t('installations.chain.label')}>
      {HEATING_CHAIN.map((part) => {
        const filled = heatingPartFilled(nta, part);
        return <li key={part}>
          <button type="button" className={filled ? 'chain-phase chain-phase--filled' : 'chain-phase'} data-part={part}
            onClick={() => { shared?.setHeatingPart(part); actions?.navigate({ step: 'installations', sub: 'heating' }); }}>
            <span className="chain-phase__dot" aria-hidden="true" />
            {t(PART_KEYS[part])}
            <span className="visually-hidden">, {t(filled ? 'installations.chain.filled' : 'installations.chain.empty')}</span>
          </button>
        </li>;
      })}
    </ol>
  );
}

export function InstallationsOverview() {
  const { t, locale } = useI18n();
  const { state, dispatch } = useEnergy();
  const kernel = useKernel();
  const actions = useShellActions();
  const { project } = state;
  const figures = useSystemFigures();
  const issues = kernelIssues(kernel?.settled);
  const pvPeak = projectPvPeak(project);
  const present = SERVICES.filter((service) => servicePresent(project, service));
  const absent = SERVICES.filter((service) => !servicePresent(project, service));

  return <>
    <div className="chain-toolbar"><InstallationAddBar /></div>
    {present.length === 0 && <p className="page-lead">{t('installations.empty')}</p>}
    <div className="chain-grid">
      {present.map((service) => {
        const own = serviceIssues(issues, service);
        const blocks = ntaBlocks(project, service);
        const systems = service.lists.flatMap((key) => (project[key] as SystemItem[]).map((item, index) => ({ key, item, index })));
        const subtitle = [
          service.ref,
          service.id === 'generation' && (project.solarPV.length > 0 || (project.ntaCalculation?.pvSystems ?? []).length > 0)
            ? `${formatNumber(pvPeak.kwp, locale, 2)}${pvPeak.partial ? '+' : ''} kWp` : null,
          service.id === 'heatPumps' ? `${(project.ntaHeatPumps ?? []).length} ${t('installations.heatPumpCount')}` : null,
        ].filter(Boolean).join(' · ');
        return (
          <Card key={service.id} level={2} className="chain-card" title={t(`nav.sub.installations.${service.id}`)} subtitle={subtitle}
            actions={<>
              <ServiceStatus issues={own} present />
              <Button size="sm" variant="ghost" onClick={() => actions?.navigate({ step: 'installations', sub: service.id })}
                aria-label={`${t('installations.open')}: ${t(`nav.sub.installations.${service.id}`)}`}>
                {t('installations.open')} <ArrowRight aria-hidden="true" />
              </Button>
            </>}>
            {service.id === 'heating' && <HeatingChain project={project} />}
            <ul className="chain-stages">
              {systems.map(({ key, item, index }) => (
                <li key={item.id} data-path={formatKernelPath([key, index])}
                  className={state.selectedItemId === item.id ? 'selected' : undefined}>
                  <button type="button" className="chain-stage" aria-pressed={state.selectedItemId === item.id}
                    onClick={() => dispatch({ type: 'SELECT_ITEM', payload: { id: item.id, itemType: SYSTEM_LISTS.find((list) => list.key === key)!.itemType } })}
                    onDoubleClick={() => { const action = editAction(SYSTEM_LISTS.find((list) => list.key === key)!.itemType, item.id); if (action) dispatch(action); }}>
                    <span className="chain-stage__kicker">{t(SYSTEM_LISTS.find((list) => list.key === key)!.kindKey)}</span>
                    <span className="chain-stage__name">{item.name}</span>
                    <span className="chain-figs">
                      {figures(key, item).map(([label, value]) => <span key={label}><small>{label}</small> <b className="ui-num">{value}</b></span>)}
                    </span>
                  </button>
                </li>
              ))}
              {blocks.map((key) => (
                <li key={key} data-path={`ntaCalculation.${key}`}>
                  <button type="button" className="chain-stage chain-stage--nta"
                    onClick={() => actions?.navigate(routeForPath(`ntaCalculation.${key}`))}>
                    <span className="chain-stage__kicker"><Tag>NTA 8800</Tag></span>
                    <span className="chain-stage__name">{t(`installations.block.${key}`)}</span>
                    <span className="chain-figs"><span>{t('installations.openNta')}</span></span>
                  </button>
                </li>
              ))}
            </ul>
            {own.length > 0 && <IssueList
              issues={own.slice(0, 3).map((issue) => ({ code: issue.code, severity: issue.kind, path: issue.path, detail: issue.detail, location: routeLabel(t, routeForPath(issue.path)) }))}
              onGoTo={actions ? (issue) => actions.navigate(routeForPath(issue.path)) : undefined} />}
          </Card>
        );
      })}
      {absent.length > 0 && <section className="chain-card chain-card--empty" aria-label={t('installations.absent')}>
        <h2 className="ui-card__title">{absent.map((service) => t(`nav.sub.installations.${service.id}`)).join(', ')}</h2>
        <p>{t('installations.absentText')}</p>
        <div className="chain-card__links">
          {absent.map((service) => (
            <Button key={service.id} size="sm" variant="ghost" onClick={() => actions?.navigate({ step: 'installations', sub: service.id })}>
              {t(`nav.sub.installations.${service.id}`)}
            </Button>
          ))}
        </div>
      </section>}
    </div>
    <Banner tone="info">{t('installations.ntaNote')}</Banner>
  </>;
}

// ── Sub page per service ────────────────────────────────────────────

export function ServicePage({ sub, extra, focusPath }: { sub: ServiceId; extra?: ReactNode; focusPath?: string }) {
  const { t, locale } = useI18n();
  const { state, dispatch } = useEnergy();
  const kernel = useKernel();
  const actions = useShellActions();
  const { project } = state;
  const service = serviceDef(sub)!;
  const figures = useSystemFigures();
  const issues = serviceIssues(kernelIssues(kernel?.settled), service);
  const pvPeak = projectPvPeak(project);

  return <>
    {issues.length > 0 && <Card level={2} title={t('installations.findings')} flush>
      <IssueList issues={issues.map((issue) => ({ code: issue.code, severity: issue.kind, path: issue.path, detail: issue.detail, location: routeLabel(t, routeForPath(issue.path)) }))}
        onGoTo={actions ? (issue) => actions.navigate(routeForPath(issue.path)) : undefined} />
    </Card>}
    {SYSTEM_LISTS.filter((list) => service.lists.includes(list.key)).map((list) => {
      const items = (project[list.key] as SystemItem[]).map((item, index) => ({ item, index }));
      const sample = items[0] ? figures(list.key, items[0].item) : [];
      const columns: Array<Column<{ item: SystemItem; index: number }>> = [
        { key: 'name', header: t('properties.name'), render: ({ item }) => item.name },
        ...sample.map(([label], column) => ({ key: `fig${column}`, header: <span className="ui-nocase">{label}</span>, render: ({ item }: { item: SystemItem }) => figures(list.key, item)[column]?.[1] ?? '' })),
        { key: 'actions', header: <span className="visually-hidden">{t('envelope.actions')}</span>, width: 84, render: ({ item }) => <ItemActions compact itemType={list.itemType} id={item.id} name={item.name} /> },
      ];
      return (
        <Card key={list.key} title={`${t(list.kindKey)} (${items.length})`} level={2} flush
          subtitle={list.key === 'solarPV' && items.length > 0 ? `${formatNumber(pvPeak.kwp, locale, 1)}${pvPeak.partial ? '+' : ''} kWp` : undefined}>
          <DataTable
            caption={t(list.kindKey)}
            columns={columns}
            rows={items}
            rowKey={({ item }) => item.id}
            selectedKey={state.selectedItemId}
            onSelect={({ item }) => dispatch({ type: 'SELECT_ITEM', payload: { id: item.id, itemType: list.itemType } })}
            onActivate={({ item }) => { const action = editAction(list.itemType, item.id); if (action) dispatch(action); }}
            rowProps={({ index }) => ({ 'data-path': formatKernelPath([list.key, index]) })}
            empty={t('installations.noSystems')}
          />
        </Card>
      );
    })}
    {extra}
    <NtaStepSections route={{ step: 'installations', sub, ...(focusPath ? { focusPath } : {}) }} />
  </>;
}

/** Kept for older imports: the overview. */
export const InstallationsPage = InstallationsOverview;
