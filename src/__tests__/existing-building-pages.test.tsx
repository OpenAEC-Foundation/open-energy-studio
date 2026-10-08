import { useEffect, useState } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen, within } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import {
  BasisopnamePanel, SURVEY_SECTIONS, surveySectionForPath, type SurveySection,
} from '../components/BasisopnamePanel/BasisopnamePanel';
import { MaatwerkadviesPanel, type MwaTab } from '../components/MaatwerkadviesPanel/MaatwerkadviesPanel';
import { RelabelPanel, relabelProofCounts } from '../components/MaatwerkadviesPanel/RelabelPanel';
import { labelPathBars } from '../components/shell/pages/existing/MwaViews';
import { WORKFLOW_STEPS, normalizeRoute } from '../core/navigation/routes';
import { SURVEY_STEP_IDS } from '../core/survey/surveyFlow';
import type { MaatwerkadviesAssessment, MwaVariantResult } from '../core/nta/KernelClient';
import { renderWithProviders, userEvent } from './test-utils';

afterEach(() => vi.unstubAllGlobals());

function variant(id: string, name: string, kind: MwaVariantResult['kind'], labelClass: string, ep2: number,
  extra: Partial<MwaVariantResult> = {}): MwaVariantResult {
  return {
    id, name, kind, measureIds: [], valid: true,
    label: { labelClass, needIndicatorKwhPerM2: null, primaryFossilIndicatorKwhPerM2: ep2, renewableSharePercent: 0, tojuliMaxK: null },
    actualUse: null,
    savings: kind === 'current' ? null : { gasM3: 400, electricityKwh: -200, heatKwh: 0, primaryFossilKwh: 3000, co2Kg: 700, energyCostEur: 640 },
    investmentEur: kind === 'current' ? 0 : 12000, maintenanceEurPerYear: 0,
    simplePaybackYears: kind === 'current' ? null : 18.5, netPresentValueEur: kind === 'current' ? null : 2300,
    horizonYears: 30, phasing: [], systemChecks: [], issues: [], ...extra,
  };
}

const ASSESSMENT: MaatwerkadviesAssessment = {
  status: 'calculated_unverified', scope: 's', targetNormVersion: 'NTA', kernelVersion: 'k', inputFingerprint: 'f',
  attestStatus: 'unattested', current: variant('current', 'Huidig', 'current', 'E', 268),
  measures: [variant('m1', 'Spouwmuur', 'measure', 'C', 210)],
  packages: [variant('p1', 'Schil', 'package', 'B', 176), variant('p2', 'Schil + WP', 'package', 'A', 118, { netPresentValueEur: 7850 })],
  fitCheck: null,
  advice: { packageId: 'p2', chosenBy: 'adviser', motivation: '', warnings: [], specialistNotes: [], notes: [] },
  renovationPassport: null, interpretations: [], issues: [],
} as unknown as MaatwerkadviesAssessment;

describe('survey wizard (F8)', () => {
  it('maps kernel paths of the survey to its sections and lists them as sub pages', () => {
    expect(surveySectionForPath('basisopname.envelope.surfaces[0].uValue')).toBe('envelope');
    expect(surveySectionForPath('additionalHotWaterSystems[1]')).toBe('hotWater');
    expect(surveySectionForPath('coolingPresent')).toBe('cooling');
    expect(surveySectionForPath('lighting.zones[0]')).toBe('zones');
    expect(surveySectionForPath('derivedInput')).toBe('result');
    expect(surveySectionForPath('constructionYear')).toBe('general');
    // The survey route lists the steps of the question flow; old section ids still open the matching step.
    const survey = WORKFLOW_STEPS.find((step) => step.id === 'survey')!;
    expect(survey.subs.map((sub) => sub.id)).toEqual(SURVEY_STEP_IDS);
    expect(SURVEY_SECTIONS.filter((section) => section !== 'result').length).toBeGreaterThan(0);
    expect(normalizeRoute({ step: 'survey', sub: 'envelope' }).sub).toBe('gevels');
    expect(normalizeRoute({ step: 'survey', sub: 'result' }).sub).toBe('controle');
  });

  it('shows one section at a time, moves with the progress list and opens the failing section with "Go to"', async () => {
    const assessment = {
      status: 'calculated_unverified', scope: 'x', source: 'ISSO 75.1',
      appliedDefaults: [{ rule: 'r', path: 'heating.generator.engine', value: 'gas_engine', source: 'ISSO 75.1 table 9.7' }],
      warnings: [], issues: [{ code: 'value_required', path: 'envelope.surfaces[0].areaM2' }], derivedInput: null,
      performance: { indicativeLabelClass: 'C', needIndicatorKwhPerM2Year: 120, primaryFossilIndicatorKwhPerM2Year: 214, renewableSharePercent: 4 },
      referenceVerified: false,
    };
    vi.stubGlobal('fetch', vi.fn(async () => ({ json: async () => assessment })));
    const calls: Array<[SurveySection, string | undefined]> = [];
    function Wizard() {
      const [section, setSection] = useState<SurveySection>('general');
      return <BasisopnamePanel section={section} onSection={(next, focus) => { calls.push([next, focus]); setSection(next); }} />;
    }
    const user = userEvent.setup();
    renderWithProviders(<Wizard />);
    await user.click(screen.getByRole('button', { name: 'Start utility survey' }));
    const progress = screen.getByRole('navigation', { name: 'Survey progress' });
    expect(within(progress).getByRole('button', { name: 'General' })).toHaveAttribute('aria-current', 'step');
    expect(screen.queryByLabelText('Vertical pipes (§7.2.4)')).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Next' }));
    expect(within(progress).getByRole('button', { name: 'Calculation zones' })).toHaveAttribute('aria-current', 'step');
    expect(screen.queryByLabelText('Vertical pipes (§7.2.4)')).not.toBeInTheDocument();

    // The outcome card stays beside every section and is marked indicative.
    const card = screen.getByRole('complementary', { name: 'Survey outcome' });
    await user.click(within(card).getByRole('button', { name: 'Calculate survey' }));
    expect(await within(card).findByText('C')).toBeInTheDocument();
    expect(within(card).getByText('Indicative')).toBeInTheDocument();
    expect(within(card).getByText('1 default value(s)')).toBeInTheDocument();
    expect(within(progress).getByRole('button', { name: /Thermal envelope.*1 error/ })).toBeInTheDocument();

    await user.click(within(card).getByRole('button', { name: 'Details and default values' }));
    await user.click(screen.getByRole('button', { name: 'Go to Thermal envelope' }));
    expect(calls[calls.length - 1]).toEqual(['envelope', 'basisopname.envelope.surfaces[0].areaM2']);
    // The kernel result is kept while moving between sections.
    expect(within(card).getByText('C')).toBeInTheDocument();
  }, 60000);
});

describe('maatwerkadvies tabs (F8)', () => {
  it('builds the label path from the current state and the packages, advised package marked', () => {
    const bars = labelPathBars(ASSESSMENT, 'p2', 'Current');
    expect(bars.map((bar) => [bar.name, bar.labelClass, bar.ep2, bar.advised])).toEqual([
      ['Current', 'E', 268, false], ['1 · Schil', 'B', 176, false], ['2 · Schil + WP', 'A', 118, true],
    ]);
    // Without packages the measures form the path.
    expect(labelPathBars({ ...ASSESSMENT, packages: [] }, null, 'Current').map((bar) => bar.name)).toEqual(['Current', '1 · Spouwmuur']);
  });

  it('shows measure cards with template picker and incomplete state, package chips, and the comparison after calculating', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({ json: async () => ASSESSMENT })));
    function Page() {
      const [tab, setTab] = useState<MwaTab>('measures');
      const { state } = useEnergy();
      return <>
        <button type="button" onClick={() => setTab('advice')}>to advice</button>
        <MaatwerkadviesPanel tab={tab} />
        <output data-testid="mwa">{JSON.stringify(state.project.maatwerkadvies ?? null)}</output>
      </>;
    }
    const user = userEvent.setup();
    renderWithProviders(<Page />);
    await user.click(screen.getByRole('button', { name: 'Start tailored advice' }));
    // Adding a measure opens it in the side sheet.
    await user.click(screen.getByRole('button', { name: 'Add measure' }));
    const sheet = await screen.findByRole('dialog');
    await user.type(within(sheet).getByLabelText('Name'), 'Spouwmuur');
    await user.click(within(sheet).getByRole('button', { name: 'Done' }));
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();

    const card = screen.getByTestId('mwa-card-m1');
    // A fresh insulation template has no value yet: incomplete before the kernel runs.
    expect(within(card).getByText('incomplete')).toBeInTheDocument();
    await user.selectOptions(within(card).getByLabelText('Measure type'), 'manual');
    expect(JSON.parse(screen.getByTestId('mwa').textContent ?? 'null').measures[0].template).toBeFalsy();

    await user.click(screen.getByRole('button', { name: 'Add package' }));
    await user.click(within(screen.getByTestId('mwa-card-m1')).getByRole('button', { name: 'p1' }));
    expect(JSON.parse(screen.getByTestId('mwa').textContent ?? 'null').packages[0].measureIds).toEqual(['m1']);

    await user.click(screen.getAllByRole('button', { name: 'Calculate' })[0]);
    const comparison = await screen.findByRole('region', { name: 'Package comparison' });
    expect(within(comparison).getAllByRole('row').map((row) => row.querySelector('th')?.textContent))
      .toEqual(['Variant', 'Schil', 'Schil + WP advice']);
    await user.click(within(comparison).getByRole('radio', { name: 'NPV' }));
    expect(within(comparison).getAllByRole('row')[1].querySelector('th')?.textContent).toBe('Schil + WP advice');
    expect(screen.getByRole('img', { name: /Current situation: E 268.*Schil \+ WP: A 118/ })).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'to advice' }));
    const inspector = screen.getByRole('region', { name: 'Chosen package Schil + WP' });
    expect(within(inspector).getByLabelText('Label from E to A')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Advice report (pdf)' })).toBeInTheDocument();
  }, 60000);
});

describe('relabel stepper (F8)', () => {
  it('counts the evidence roles', () => {
    expect(relabelProofCounts([{ relabelProof: 'review' }, { relabelProof: 'specified_invoice' }, {}])).toEqual({
      quote_with_order: 0, specified_invoice: 1, production_photo: 0, review: 1,
    });
  });

  it('walks original → comparison → changes → evidence → readiness with 6a/6b/review classification', async () => {
    const assessment = {
      allowed: false, needsReview: true, scheme: 'w', source: 'BRL 9500',
      changes: [
        { path: '/zones/0/surfaces/0/uValue', before: 2, after: 0.4, verdict: 'allowed', cluster: 'insulationProperties' },
        { path: '/zones/0/floorArea', before: 80, after: 95, verdict: 'not_allowed', cluster: 'area' },
        { path: '/name', before: 'a', after: 'b', verdict: 'review', cluster: 'notClassified' },
      ],
    };
    function Seed() {
      const { dispatch } = useEnergy();
      useEffect(() => {
        dispatch({ type: 'UPDATE_PROJECT_INFO', payload: { registration: {
          relabelComparison: { originalFileName: 'origineel.oes', originalSha256: 'ab'.repeat(32), assessment },
          evidence: [
            { id: 'e1', kind: 'invoice', fileName: 'factuur.pdf', sha256: 'cd'.repeat(32), relabelProof: 'review' },
            { id: 'e2', kind: 'photo_detail', fileName: 'pv.jpg', sha256: 'ef'.repeat(32), relabelProof: 'production_photo' },
          ],
        } as never } });
      }, [dispatch]);
      return null;
    }
    const user = userEvent.setup();
    renderWithProviders(<><Seed /><RelabelPanel /></>);
    const steps = await screen.findByRole('list', { name: 'Relabel steps' });
    expect(within(steps).getByRole('button', { name: /Original file.*done/ })).toBeInTheDocument();
    expect(within(steps).getByRole('button', { name: /Changes.*blocks the relabel/ })).toBeInTheDocument();
    expect(within(steps).getByRole('button', { name: /Evidence roles.*needs attention/ })).toBeInTheDocument();
    expect(within(steps).getByRole('button', { name: /Readiness.*to do/ })).toBeInTheDocument();

    expect(screen.getByText('Relabelling not allowed')).toBeInTheDocument();
    expect(screen.getByText('origineel.oes')).toBeInTheDocument();
    // The migration of old invoices is a banner with the count.
    expect(screen.getByText(/1 invoice\(s\) from before the relabel roles/)).toBeInTheDocument();

    const table = screen.getByRole('table');
    expect(within(table).getAllByRole('row')).toHaveLength(4);
    await user.click(screen.getByRole('radio', { name: '6b 1' }));
    const rows = within(table).getAllByRole('row');
    expect(rows).toHaveLength(2);
    expect(within(rows[1]).getByText('not allowed (6b)')).toBeInTheDocument();
    expect(rows[1]).toHaveTextContent('/zones/0/floorArea');

    const evidence = screen.getByRole('region', { name: /4 · Evidence roles/ });
    expect(within(evidence).getByText('Photo of PV or solar thermal with shading').parentElement).toHaveTextContent('1');
    expect(screen.getByText('No registration check from the calculation core yet. Choose Recalculate.')).toBeInTheDocument();
  }, 60000);
});
