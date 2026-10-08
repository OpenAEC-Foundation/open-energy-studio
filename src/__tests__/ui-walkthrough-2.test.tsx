import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen, within } from '@testing-library/react';
import { isTauri } from '@tauri-apps/api/core';
import { createDefaultProject } from '../context/EnergyContext';
import { calculateBENGMonthly } from '../core/energy/BENGCalculatorMonthly';
import { generateReportHTML, reportLanguage } from '../core/report/ReportTemplate';
import { kernelReportModel } from '../core/report/KernelReportModel';
import { dutchCodeHtml, dutchNumber } from '../core/report/DutchReportText';
import { kernelEnergyBreakdown } from '../core/nta/KernelBreakdown';
import { adviceText } from '../core/nta/MwaAdviceText';
import { relabelCluster, relabelElementName, relabelNote, relabelValue } from '../core/nta/RelabelText';
import { calculateProjectPerformanceShared } from '../core/nta/useProjectPerformance';
import { ResultRow } from '../components/MaatwerkadviesPanel/MaatwerkadviesPanel';
import { defaultValueLabel } from '../components/BasisopnamePanel/BasisopnamePanel';
import { setpointChecks, setpointWriteBack, TABLE_713_SOURCE } from '../components/NtaPerformancePanel/NtaCalculationForm';
import { ProjectInfoDialog } from '../components/dialogs/ProjectInfoDialog/ProjectInfoDialog';
import { RegistrationForm } from '../components/shell/pages/RegistrationForm';
import { ReportView } from '../components/ReportView/ReportView';
import type { BuildingPerformanceAssessment, MwaVariantResult, ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import { renderWithProviders, userEvent } from './test-utils';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.mocked(isTauri).mockReturnValue(false);
});

const demandMonth = (index: number) => ({
  month: index + 1, hours: 720, outdoorTemperatureC: 5, internalGainsKwh: 100, windowSolarGainsKwh: 50,
  windowSolarCoolingKwh: 0, opaqueSolarGainsKwh: 10, groundConductanceWPerK: 0,
  heating: { transmissionKwh: 400, ventilationKwh: 200, needKwh: 300, gainsKwh: 180 },
  cooling: { needKwh: index === 6 ? 25 : 0 },
});

const performance = {
  status: 'calculated_unverified', issues: [], warnings: [],
  needIndicatorKwhPerM2Year: 51.37, primaryFossilIndicatorKwhPerM2Year: 48.33, renewableSharePercent: 49.6,
  indicativeLabelClass: 'A+++', labelSource: 'annex IX',
  bblCheck: { source: 'Bbl', function: 'residential', lossAreaRatio: 1.2,
    limits: { energyNeedMaxKwhPerM2: 65, primaryFossilMaxKwhPerM2: 50, renewableShareMinPercent: 40,
      lightConstructionAllowanceApplied: false },
    energyNeedMeets: true, primaryFossilMeets: true, renewableShareMeets: true },
  tojuliMaxK: 0.4, tojuliMeetsBblLimit: true, tojuli: [],
  annualPrimaryFossilKwh: 6000, annualRenewablePrimaryKwh: 1500,
  pvSystems: [{ id: 'pv-zuid', monthlyKwh: [], annualKwh: 13166 }],
  energyByService: {
    months: [{ service: 'heating', carrier: 'gas', month: 1, usedKwh: 3600, deliveredKwh: 3600, primaryFossilKwh: 3600 }],
    renewable: [], adjustments: [],
    annual: [{ service: 'heating', carrier: 'gas', usedKwh: 3600, deliveredKwh: 3600, primaryFossilKwh: 3600 }],
  },
  spaceHeating: {
    omittedTerms: [], additionalZoneDemands: [], monthly: [],
    demand: {
      annualHeatingNeedKwh: 3600, annualCoolingNeedKwh: 25, omittedCorrections: [],
      monthly: Array.from({ length: 12 }, (_, index) => demandMonth(index)),
    },
  },
} as unknown as BuildingPerformanceAssessment;

const assessment = {
  status: 'calculated_unverified', gaps: [], warnings: [], performance,
  derivedInput: { calculationScope: 'residential' }, geometry: { usableFloorAreaM2: 124.5, lossAreaM2: 200, lossAreaRatio: 1.6 },
} as unknown as ProjectPerformanceAssessment;

describe('BENG report from the NTA kernel', () => {
  it('builds the report figures from the kernel assessment', () => {
    const model = kernelReportModel(assessment)!;
    expect(model.indicators.map((row) => row.value)).toEqual([51.37, 48.33, 49.6]);
    expect(model.indicators[1].limit).toBe(50);
    expect(model.tojuli).toEqual({ value: 0.4, meets: true });
    expect(model.monthly[6]).toEqual({ month: 7, heatingNeedKwh: 300, coolingNeedKwh: 25, solarGainKwh: 60, transmissionKwh: 400 });
    expect(model.pvSystems).toEqual([{ id: 'pv-zuid', annualKwh: 13166 }]);
    expect(kernelReportModel({ ...assessment, status: 'incomplete' } as ProjectPerformanceAssessment)).toBeNull();
  });

  it('prints the kernel values in the exported report, in the UI language', () => {
    const project = createDefaultProject();
    const result = calculateBENGMonthly(project);
    const nl = generateReportHTML(project, result, { kernel: assessment, locale: 'nl' });
    expect(nl).toContain('<html lang="nl">');
    expect(nl).toContain('BENG-indicatoren (NTA 8800-kern)');
    expect(nl).toContain('48,33');
    expect(nl).toContain('13.166 kWh/jaar');
    expect(nl).not.toContain('Indicatieve BENG-waarden');
    const en = generateReportHTML(project, result, { kernel: assessment, locale: 'en' });
    expect(en).toContain('<html lang="en">');
    expect(en).toContain('BENG indicators (NTA 8800 kernel)');
    expect(en).toContain('13,166 kWh/yr');
    expect(en).toContain('Monthly overview');
    expect(reportLanguage('fr')).toBe('en');
  });

  it('falls back to the simplified engine, labelled as not NTA 8800', () => {
    const project = createDefaultProject();
    const html = generateReportHTML(project, calculateBENGMonthly(project), { kernel: null, locale: 'nl' });
    expect(html).toContain('Indicatief, niet volgens NTA 8800');
  });

  it('shows the kernel figures on the report tab', async () => {
    vi.stubGlobal('fetch', vi.fn((url: string) => (String(url).includes('project/performance')
      ? Promise.resolve({ ok: true, json: async () => assessment })
      : new Promise(() => {}))));
    renderWithProviders(<ReportView />);
    const section = within(await screen.findByTestId('report-beng-kernel', {}, { timeout: 3000 }));
    expect(section.getByText(/48\.33/)).toBeInTheDocument();
    expect(section.getByText('≤ 50.00')).toBeInTheDocument();
    expect(screen.queryByTestId('report-beng-indicative')).not.toBeInTheDocument();
  });
});

describe('kernel energy balance', () => {
  it('shows the sunroom and other gains and flags missing delivered energy', () => {
    const breakdown = kernelEnergyBreakdown({ ...performance, energyByService: undefined } as unknown as BuildingPerformanceAssessment);
    expect(breakdown.solarGain).toBe(720);
    expect(breakdown.internalGain).toBe(1200);
    // Q_H;gn 12 × 180 = 2160; 2160 − 720 − 1200 = 240 kWh other gains.
    expect(breakdown.otherGain).toBe(240);
    expect(breakdown.deliveredUnavailable).toBe(true);
  });
});

describe('tailored advice', () => {
  it('shows why a variant could not be calculated', () => {
    const variant = {
      id: 'm1', name: 'Isolatie', kind: 'measure', measureIds: [], valid: false,
      label: { labelClass: null, needIndicatorKwhPerM2: null, primaryFossilIndicatorKwhPerM2: null, renewableSharePercent: null, tojuliMaxK: null },
      actualUse: null, savings: null, investmentEur: 1000, maintenanceEurPerYear: 0, simplePaybackYears: null,
      netPresentValueEur: null, horizonYears: 30, phasing: [], systemChecks: [],
      issues: [{ code: 'measure_patch_failed', path: 'measures[0].changes[0]', detail: '/zones/0/surfaces/9' }],
    } as unknown as MwaVariantResult;
    const t = (key: string) => key;
    renderWithProviders(<table><tbody><ResultRow result={variant} t={t} /></tbody></table>);
    const row = screen.getByTestId('mwa-issues-m1');
    expect(row).toHaveTextContent('measure_patch_failed');
    expect(row).toHaveTextContent('/zones/0/surfaces/9');
  });

  it('translates the kernel advice sentences for English', () => {
    expect(adviceText('Pakket A: de energiekosten nemen toe', 'en')).toBe('Pakket A: the energy costs increase');
    expect(adviceText('ISSO 82.2/75.2 §4.2.2: combineer de maatregelen in minimaal twee pakketten', 'en'))
      .toBe('ISSO 82.2/75.2 §4.2.2: combine the measures into at least two packages');
    expect(adviceText('Pakket A: de energiekosten nemen toe', 'nl')).toBe('Pakket A: de energiekosten nemen toe');
    expect(adviceText('eigen notitie', 'en')).toBe('eigen notitie');
  });
});

describe('Dutch records and translated values', () => {
  it('writes BRL documents with a decimal comma and translated codes', () => {
    expect(dutchNumber(11.094, 2)).toBe('11,09');
    expect(dutchNumber(4800)).toBe('4800');
    expect(dutchCodeHtml('client_required')).toBe('Opdrachtgever ontbreekt <code>client_required</code>');
    expect(dutchCodeHtml('no_such_code')).toBe('<code>no_such_code</code>');
  });

  it('translates relabel clusters, notes, values and element names', () => {
    const t = (key: string) => ({
      'relabel.cluster.notClassified': 'niet ingedeeld',
      'relabel.note.decide': 'beslis met Bijlage 6a/6b en ISSO 82.1',
      'common.yes': 'ja',
    } as Record<string, string>)[key] ?? key;
    expect(relabelCluster(t, 'not classified')).toBe('niet ingedeeld');
    expect(relabelCluster(t, 'something new')).toBe('something new');
    expect(relabelNote(t, 'decide with Bijlage 6a/6b and ISSO 82.1')).toBe('beslis met Bijlage 6a/6b en ISSO 82.1');
    expect(relabelValue(t, 'nl', 2.8)).toBe('2,8');
    expect(relabelValue(t, 'nl', true)).toBe('ja');
    const project = { zones: [{ name: 'Woning', surfaces: [{ name: 'Voorgevel', windows: [{ name: 'Raam woonkamer', uValue: 1.1 }] }] }] };
    expect(relabelElementName(project, '/zones/0/surfaces/0/windows/0/uValue')).toBe('Woning › Voorgevel › Raam woonkamer › uValue');
    expect(relabelElementName(project, '/zones/0/surfaces/3/area')).toBe('Woning › surfaces › #4 › area');
  });

  it('labels survey default values in Dutch', () => {
    const t = (key: string) => ({ 'common.yes': 'ja', 'common.no': 'nee' } as Record<string, string>)[key] ?? key;
    renderWithProviders(<p>{defaultValueLabel(t, 'uninsulated', 'nl')} / {defaultValueLabel(t, 'false', 'nl')} / {defaultValueLabel(t, 'HrCoatedDouble', 'nl')} / {defaultValueLabel(t, '2.5', 'nl')}</p>);
    expect(screen.getByText(/ongeïsoleerd/)).toBeInTheDocument();
    expect(screen.getByText(/nee/)).toBeInTheDocument();
    expect(screen.getByText(/hr coated double/)).toBeInTheDocument();
    expect(screen.getByText('HrCoatedDouble')).toHaveClass('kernel-code-ref');
    expect(screen.getByText(/2,5/)).toBeInTheDocument();
  });
});

describe('project information dialog', () => {
  it('is a labelled dialog; the relabel fields live on the Registratie step', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProjectInfoDialog onClose={() => undefined} />);
    expect(screen.getByRole('dialog', { name: 'Project Information' })).toHaveAttribute('aria-modal', 'true');
    expect(screen.getByLabelText('Project Name')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Close' })).toBeInTheDocument();
    renderWithProviders(<RegistrationForm />);
    expect(screen.queryByLabelText('Kernel version of original survey (relabel)')).not.toBeInTheDocument();
    await user.selectOptions(screen.getByLabelText('Message type (BRL 9500 §4.2.5)'), 'relabel');
    expect(screen.getByLabelText('Kernel version of original survey (relabel)')).toBeInTheDocument();
  }, 60000);
});

describe('table 7.13 setpoint check', () => {
  it('mirrors the kernel: area-weighted functions, per project zone', () => {
    const block = { usageFunction: 'office', setpoints: { heatingC: 21, coolingC: 24 } };
    expect(setpointChecks(block, [])).toEqual([expect.objectContaining({ zoneId: null, expected: { heatingC: 21, coolingC: 24 } })]);
    const zones = { ...block, zoneData: [
      { zoneId: 'a', functionAreas: [{ function: 'office', areaM2: 300 }, { function: 'sport', areaM2: 100 }] },
      { zoneId: 'b', usageFunction: 'residential', setpoints: { heatingC: 20, coolingC: 24, sourceReference: '' } },
      { zoneId: 'orphan', usageFunction: 'sport' },
    ] };
    // The kernel walks the project zones: zone c has no zoneData entry, the orphan entry is not a zone.
    const rows = setpointChecks(zones, ['a', 'b', 'c']);
    // (21·300 + 16·100) / 400 = 19.75 °C; zone a uses the block setpoints.
    expect(rows[0]).toMatchObject({ zoneId: 'a', zoneIndex: 0, expected: { heatingC: 19.75, coolingC: 24 }, path: ['setpoints'], ownSetpoints: false });
    expect(rows[1]).toMatchObject({ zoneId: 'b', zoneIndex: 1, expected: { heatingC: 20 }, path: ['zoneData', 1, 'setpoints'], ownSetpoints: true });
    expect(rows[2]).toMatchObject({ zoneId: 'c', zoneIndex: null, expected: { heatingC: 21 }, path: ['setpoints'], ownSetpoints: false });
    expect(rows.map((row) => row.zoneId)).not.toContain('orphan');
  });

  it('writes table values with a source reference the kernel accepts', () => {
    const draft = { usageFunction: 'office', setpoints: { heatingC: 20, coolingC: 24, sourceReference: 'bestek' },
      zoneData: [{ zoneId: 'a', usageFunction: 'sport' }] };
    const [rowA, rowB] = setpointChecks(draft, ['a', 'b']);
    // Shared block setpoints, different targets: each zone gets its own setpoints.
    expect(setpointWriteBack(draft, rowA, true, 'bestek')).toEqual([['zoneData', 0, 'setpoints'],
      { sourceReference: 'bestek', heatingC: 16, coolingC: 24 }]);
    expect(setpointWriteBack(draft, rowB, true, 'bestek')).toEqual([['zoneData'],
      [...draft.zoneData, { zoneId: 'b', setpoints: { sourceReference: 'bestek', heatingC: 21, coolingC: 24 } }]]);
    const bare = { usageFunction: 'office', setpoints: { heatingC: 20, coolingC: 24, sourceReference: '' } };
    const [row] = setpointChecks(bare, []);
    expect(setpointWriteBack(bare, row, false, TABLE_713_SOURCE)[1]).toMatchObject({ sourceReference: 'NTA 8800 tabel 7.13', heatingC: 21 });
  });
});

describe('shared kernel runs', () => {
  it('lets concurrent requests for one project state share one kernel call', async () => {
    const fetchMock = vi.fn(() => Promise.resolve({ ok: true, json: async () => assessment }));
    vi.stubGlobal('fetch', fetchMock);
    const project = createDefaultProject();
    const [first, second] = await Promise.all([calculateProjectPerformanceShared(project), calculateProjectPerformanceShared(project)]);
    expect(first).toBe(second);
    expect(fetchMock).toHaveBeenCalledTimes(1);
    await calculateProjectPerformanceShared(project);
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });
});
