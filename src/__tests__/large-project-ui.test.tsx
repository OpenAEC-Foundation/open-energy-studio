/**
 * Large projects in the interface: the input pages of a utility building of
 * 40 rekenzones and 360 windows (public case H twenty times) and the results
 * of a 40-zone kernel answer render within a generous time budget, so a big
 * building stays workable. The budgets catch pathological slowness (for
 * example a quadratic lookup per row), not small regressions.
 */
import { useEffect, useState } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import caseH from '../../training-data/nta8800-public-comparison-h.json';
import office from '../../training-data/nta8800-example-office.kernel-output.json';
import { renderWithProviders } from './test-utils';
import { useEnergy } from '../context/EnergyContext';
import { StepRouter } from '../components/shell/StepRouter';
import type { ShellActions } from '../components/shell/ShellActions';
import { stepStatuses } from '../core/nta/stepStatus';
import type { Route } from '../core/navigation/routes';
import type { IProject } from '../core/energy/types';
import type { ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import type { KernelState } from '../context/KernelProvider';
import { kernelVerdict } from '../core/nta/KernelVerdict';
import { ResultsDashboard } from '../components/shell/pages/results/ResultsDashboard';

const kernel = vi.hoisted(() => ({ state: null as KernelState | null }));
vi.mock('../context/KernelProvider', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../context/KernelProvider')>()),
  useKernel: () => kernel.state,
}));

afterEach(() => {
  kernel.state = null;
  vi.unstubAllGlobals();
});

/** Generous: the full Vitest run shares the CPU with the Rust gate. */
const PAGE_BUDGET_MS = 15000;
const COPIES = 20;
const PER_ZONE = ['zoneData', 'groundFloors', 'surfaceTilts', 'windowObstructions', 'lighting'] as const;

type Json = unknown;

function elementIds(project: Record<string, any>): Set<string> {
  const ids = new Set<string>();
  for (const zone of project.zones) {
    ids.add(zone.id);
    for (const surface of zone.surfaces) {
      ids.add(surface.id);
      for (const window of surface.windows ?? []) ids.add(window.id);
    }
  }
  for (const entry of project.ntaCalculation.zoneData ?? []) {
    for (const pipe of entry.verticalPipes ?? []) ids.add(pipe.id);
  }
  return ids;
}

function rename(value: Json, ids: Set<string>, copy: number): Json {
  if (typeof value === 'string') return ids.has(value) ? `${value}~${copy}` : value;
  if (Array.isArray(value)) return value.map((item) => rename(item, ids, copy));
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, rename(item, ids, copy)]));
  }
  return value;
}

/** Case H with its zones (and the per-zone NTA entries) repeated `copies` times. */
function largeProject(copies: number): IProject {
  const base = structuredClone(caseH) as Record<string, any>;
  const ids = elementIds(base);
  const project = structuredClone(base);
  for (let copy = 1; copy < copies; copy += 1) {
    project.zones.push(...base.zones.map((zone: Json) => rename(zone, ids, copy)));
    for (const key of PER_ZONE) {
      const entries = base.ntaCalculation[key];
      if (Array.isArray(entries)) project.ntaCalculation[key].push(...entries.map((entry: Json) => rename(entry, ids, copy)));
    }
  }
  return { ...project, id: `large-${copies}` } as IProject;
}

const noop = () => undefined;
const actions: ShellActions = {
  newProject: noop, openProject: noop, saveProject: noop, saveAsProject: noop, calculate: noop,
  openDialog: noop, navigate: noop, exportReport: noop, printReport: noop, exportIFC: noop,
  exportModelIFC: noop, exportUNIEC3: noop, importUNIEC3: noop, exportVABI: noop, importVABI: noop,
  openSettings: noop, openFeedback: noop, openPalette: noop, toggleInspector: noop, togglePreview: noop,
};

function Shell({ route, project }: { route: Route; project: IProject }) {
  const { state, dispatch } = useEnergy();
  const [loaded, setLoaded] = useState(false);
  useEffect(() => { dispatch({ type: 'SET_PROJECT', payload: project }); setLoaded(true); }, [dispatch, project]);
  if (!loaded) return null;
  const statuses = stepStatuses(state.project, null);
  return <main><StepRouter project={state.project} route={route} statuses={statuses} actions={actions} /></main>;
}

describe('a large project in the interface', () => {
  const project = largeProject(COPIES);

  it('has 40 zones and 360 windows', () => {
    expect(project.zones).toHaveLength(2 * COPIES);
    const windows = project.zones.flatMap((zone) => zone.surfaces).reduce((sum, surface) => sum + (surface.windows?.length ?? 0), 0);
    expect(windows).toBe(18 * COPIES);
  });

  it.each([
    ['building/envelope', { step: 'building', sub: 'envelope' }],
    ['building/zones', { step: 'building', sub: 'zones' }],
    ['installations/heating', { step: 'installations', sub: 'heating' }],
    ['check/overview', { step: 'check', sub: 'overview' }],
  ] as Array<[string, Route]>)('renders %s within the budget', async (_name, route) => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve({ ok: true, json: async () => [] })));
    const started = performance.now();
    const { unmount } = renderWithProviders(<Shell route={route} project={project} />);
    await screen.findByRole('heading', { level: 1 }, { timeout: PAGE_BUDGET_MS });
    const elapsed = performance.now() - started;
    expect(elapsed).toBeLessThan(PAGE_BUDGET_MS);
    unmount();
  }, 60000);

  it('renders every results tab of a 40-zone answer within the budget', async () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve({ ok: true, json: async () => [] })));
    const assessment = structuredClone(office) as unknown as ProjectPerformanceAssessment;
    const heating = assessment.performance!.spaceHeating;
    heating.additionalZoneDemands = Array.from({ length: 2 * COPIES - 1 }, () => structuredClone(heating.demand));
    kernel.state = {
      project: {} as KernelState['project'],
      query: { kind: 'done', assessment } as KernelState['query'],
      settled: assessment,
      verdict: kernelVerdict(assessment),
      phase: 'current',
      error: null,
      refresh: () => {},
    };
    const started = performance.now();
    const { rerender } = renderWithProviders(<ResultsDashboard sub="overview" onNavigate={() => {}} />);
    for (const sub of ['services', 'zones', 'monthly', 'provenance'] as const) {
      rerender(<ResultsDashboard sub={sub} onNavigate={() => {}} />);
    }
    await waitFor(() => expect(screen.getByTestId('results-provenance')).toBeInTheDocument());
    expect(performance.now() - started).toBeLessThan(PAGE_BUDGET_MS);
  }, 60000);
});
