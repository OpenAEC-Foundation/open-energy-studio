import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { isTauri } from '@tauri-apps/api/core';
import { createDefaultProject, useEnergy } from '../context/EnergyContext';
import { calculateBENGMonthly } from '../core/energy/BENGCalculatorMonthly';
import { normalizeProject, ASSUMED_QV10 } from '../core/energy/normalizeProject';
import { generateReportHTML } from '../core/report/ReportTemplate';
import { bengIfcModel, exportBENGToIFC } from '../core/ifc/IFCEnergyExporter';
import { kernelVerdict, kernelWithheld, indicativeAllowed } from '../core/nta/KernelVerdict';
import { kernelReportModel } from '../core/report/KernelReportModel';
import { kernelDetailText } from '../i18n/kernelDetail';
import { ResultsView } from '../components/ResultsView/ResultsView';
import { ReportView } from '../components/ReportView/ReportView';
import type { IProject } from '../core/energy/types';
import type { ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import { renderWithProviders, userEvent } from './test-utils';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.mocked(isTauri).mockReturnValue(false);
});

const invalid = { status: 'invalid', gaps: [{ code: 'u_value_out_of_range', path: 'constructions[0].uValue' }],
  derivedInput: null, performance: null } as unknown as ProjectPerformanceAssessment;
const incomplete = { status: 'incomplete', gaps: [{ code: 'ventilation_flow_required', path: 'zones[0]' }],
  derivedInput: null, performance: null } as unknown as ProjectPerformanceAssessment;
const noNtaInput = { status: 'incomplete', gaps: [{ code: 'nta_calculation_block_missing', path: 'ntaCalculation' }],
  derivedInput: null, performance: null } as unknown as ProjectPerformanceAssessment;

const demandMonth = (index: number) => ({
  month: index + 1, windowSolarGainsKwh: 50, opaqueSolarGainsKwh: 10, internalGainsKwh: 100,
  heating: { transmissionKwh: 400, ventilationKwh: 200, needKwh: 300, gainsKwh: 160 }, cooling: { needKwh: 0 },
});
const calculated = {
  status: 'calculated_unverified', gaps: [], derivedInput: { calculationScope: 'residential' },
  geometry: { usableFloorAreaM2: 120 },
  performance: {
    needIndicatorKwhPerM2Year: 51.37, primaryFossilIndicatorKwhPerM2Year: 80.56, renewableSharePercent: 22.3,
    indicativeLabelClass: 'A+', tojuliMaxK: 0.4, tojuliMeetsBblLimit: true, pvSystems: [],
    bblCheck: { limits: { energyNeedMaxKwhPerM2: 59.72, primaryFossilMaxKwhPerM2: 30, renewableShareMinPercent: 50 },
      energyNeedMeets: true, primaryFossilMeets: false, renewableShareMeets: false },
    energyByService: { months: [], annual: [], renewable: [], adjustments: [] },
    spaceHeating: { additionalZoneDemands: [], monthly: [],
      demand: { monthly: Array.from({ length: 12 }, (_, index) => demandMonth(index)) } },
  },
} as unknown as ProjectPerformanceAssessment;

describe('one shared rule for simplified numbers (KernelVerdict)', () => {
  it('classifies kernel answers', () => {
    expect(kernelVerdict(null)).toBe('unknown');
    expect(kernelVerdict(calculated)).toBe('calculated');
    expect(kernelVerdict(invalid)).toBe('withheld');
    expect(kernelVerdict(incomplete)).toBe('withheld');
    expect(kernelVerdict(noNtaInput)).toBe('noNtaInputYet');
    expect(kernelWithheld(invalid)).toBe(true);
    expect(indicativeAllowed(noNtaInput)).toBe(true);
    expect(indicativeAllowed(calculated)).toBe(false);
  });
});

describe('the BENG report (HTML export, print and print preview) withholds simplified numbers', () => {
  const project = createDefaultProject();
  const result = calculateBENGMonthly(project);
  const beng1 = result.beng1.toLocaleString('nl-NL', { minimumFractionDigits: 1, maximumFractionDigits: 1 });

  for (const [name, assessment] of [['invalid', invalid], ['incomplete', incomplete]] as const) {
    it(`prints no simplified BENG values when the kernel result is ${name}`, () => {
      const html = generateReportHTML(project, result, { kernel: assessment, locale: 'nl' });
      expect(html).toContain('BENG-resultaten achtergehouden');
      expect(html).not.toContain('Indicatieve BENG-waarden');
      expect(html).not.toContain(`<td>${beng1}</td>`);
    });
  }

  it('still shows the indicative estimate without any NTA input yet', () => {
    const html = generateReportHTML(project, result, { kernel: noNtaInput, locale: 'nl' });
    expect(html).toContain('Indicatieve BENG-waarden');
  });
});

describe('the IFC export follows the same rule', () => {
  const project = createDefaultProject();
  const result = calculateBENGMonthly(project);
  const texts = (model: ReturnType<typeof exportBENGToIFC>) => [...model.entities.values()]
    .filter((entity) => entity.type === 'IFCTEXT').map((entity) => entity.attributes[0]);
  const reals = (model: ReturnType<typeof exportBENGToIFC>) => [...model.entities.values()]
    .filter((entity) => entity.type === 'IFCREAL').map((entity) => entity.attributes[0]);

  it('refuses when the kernel withholds its result', () => {
    expect(bengIfcModel(project, result, invalid)).toBe('withheld');
    expect(bengIfcModel(project, result, incomplete)).toBe('withheld');
  });

  it('writes the kernel figures when the kernel calculated', () => {
    const model = bengIfcModel(project, result, calculated);
    expect(model).not.toBe('withheld');
    const ifc = model as ReturnType<typeof exportBENGToIFC>;
    expect(reals(ifc)).toContain(80.56);
    expect(reals(ifc)).not.toContain(result.beng2);
    expect(texts(ifc)).toContain('DOES_NOT_MEET');
    expect(texts(ifc)).not.toContain('INDICATIVE');
    expect(kernelReportModel(calculated)).not.toBeNull();
  });

  it('writes the indicative result only without a kernel verdict', () => {
    const model = bengIfcModel(project, result, null) as ReturnType<typeof exportBENGToIFC>;
    expect(texts(model).filter((value) => value === 'INDICATIVE')).toHaveLength(3);
  });
});

/** A kernel stub whose answers are handed out in order; past the list it never answers (a pending run). */
function stubKernelSequence(bodies: unknown[]) {
  let call = 0;
  vi.stubGlobal('fetch', vi.fn((url: string) => {
    if (!String(url).includes('project/performance')) return new Promise(() => {});
    const body = bodies[call++];
    return body === undefined ? new Promise(() => {}) : Promise.resolve({ ok: true, json: async () => body });
  }));
}

function Harness({ view }: { view: 'results' | 'report' }) {
  const { state, dispatch } = useEnergy();
  return <>
    <button type="button" onClick={() => dispatch({ type: 'SET_RESULT', payload: calculateBENGMonthly(state.project) })}>Calculate</button>
    <button type="button" onClick={() => dispatch({ type: 'UPDATE_HEATING_SYSTEM', payload: {
      id: state.project.heatingSystems[0].id, data: { cop: 4.1 },
    } })}>Change heating</button>
    {view === 'results' ? <ResultsView /> : <ReportView />}
  </>;
}

describe('views keep simplified numbers back', () => {
  it('the results view does not flash simplified numbers while the next run is pending', async () => {
    stubKernelSequence([invalid]);
    const user = userEvent.setup();
    renderWithProviders(<Harness view="results" />);
    await user.click(screen.getByRole('button', { name: 'Calculate' }));
    expect(await screen.findByTestId('results-withheld')).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Change heating' }));
    await user.click(screen.getByRole('button', { name: 'Calculate' }));
    // The second run never answers: the withheld verdict of the last run must hold.
    expect(screen.getByTestId('results-withheld')).toBeInTheDocument();
    expect(screen.queryByTestId('beng-cards-indicative')).not.toBeInTheDocument();
    expect(screen.queryByTestId('results-indicative')).not.toBeInTheDocument();
  }, 60000);

  it('the report tab shows no indicative BENG table when the kernel result is invalid', async () => {
    stubKernelSequence([invalid]);
    const user = userEvent.setup();
    renderWithProviders(<Harness view="report" />);
    await user.click(screen.getByRole('button', { name: 'Calculate' }));
    expect(await screen.findByText('Results withheld')).toBeInTheDocument();
    expect(screen.queryByTestId('report-beng-indicative')).not.toBeInTheDocument();
  }, 60000);
});

describe('opening a file without air tightness', () => {
  it('fills in the importer default 0,4, flagged as assumed, never 0', () => {
    const project = createDefaultProject();
    const broken = { ...project, zones: project.zones.map((zone) => ({ ...zone, airTightness: undefined })) } as unknown as IProject;
    const fixed = normalizeProject(broken);
    expect(ASSUMED_QV10).toBe(0.4);
    for (const zone of fixed.zones) expect(zone.airTightness).toEqual({ qv10: 0.4, assumed: true });
    expect(normalizeProject(project)).toBe(project);
  });

  it('refuses an explicitly invalid qv10 instead of replacing the recorded value with an assumption', () => {
    const project = createDefaultProject();
    for (const invalid of ['unknown', 0, -0.1]) {
      const broken = { ...project, zones: [{ ...project.zones[0], airTightness: { qv10: invalid } }] } as unknown as IProject;
      expect(() => normalizeProject(broken)).toThrow('airTightness.qv10 must be a positive finite number');
    }
    const missing = { ...project, zones: [{ ...project.zones[0], airTightness: {} }] } as unknown as IProject;
    expect(normalizeProject(missing).zones[0].airTightness).toEqual({ qv10: ASSUMED_QV10, assumed: true });
  });
});

describe('kernel detail numbers', () => {
  it('shows at most 2 decimals', () => {
    const t = (key: string) => (key === 'kernel.detail.lossAreaRatio' ? 'A_ls/A_g = {{p1}}' : key);
    expect(kernelDetailText('A_ls/A_g 5000.123456789', t, 'en')).toEqual({ text: 'A_ls/A_g = 5,000.12', translated: true });
    expect(kernelDetailText('A_ls/A_g 200001.4', t, 'nl')).toEqual({ text: 'A_ls/A_g = 200.001,4', translated: true });
  });
});
