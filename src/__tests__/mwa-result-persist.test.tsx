import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { useEffect, useRef, useState } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen, within } from '@testing-library/react';
import { useDocumentManager, useEnergy } from '../context/EnergyContext';
import type { IProject } from '../core/energy/types';
import { emptyMaatwerkadvies, MaatwerkadviesPanel } from '../components/MaatwerkadviesPanel/MaatwerkadviesPanel';
import { generateMaatwerkadviesReportHTML } from '../core/report/MaatwerkadviesReport';
import type { MaatwerkadviesAssessment, MwaVariantResult } from '../core/nta/KernelClient';
import { renderWithProviders, userEvent } from './test-utils';
import { SLOW_UI_TEST_MS } from './timeouts';

// Follow-up of review 38: the maatwerkadvies result is kept per document when
// the step is left, and the screen table has the report's TO_juli and heat columns.

const dwelling = JSON.parse(readFileSync(resolve(process.cwd(), 'training-data/nta8800-example-terraced-dwelling.json'), 'utf8')) as IProject;
const project: IProject = { ...structuredClone(dwelling), maatwerkadvies: emptyMaatwerkadvies() };

const current = {
  id: 'current', kind: 'current', name: '', valid: true,
  label: { labelClass: 'Q', primaryFossilIndicatorKwhPerM2: 200, tojuliMaxK: 1.234 },
  actualUse: { gasM3: 1200, electricityImportKwh: 3000, electricityExportKwh: 500, districtHeatKwh: 4321.4, co2Kg: 2500, energyCostEur: 2100 },
  savings: null, investmentEur: 0, simplePaybackYears: null, netPresentValueEur: null,
  issues: [], systemChecks: [], phasing: [],
} as unknown as MwaVariantResult;

const assessment = {
  status: 'calculated_unverified', attestStatus: 'unattested', kernelVersion: '0.4.0', targetNormVersion: '2025+C1',
  normVersion: '2025+C1', registrationEligible: true, inputFingerprint: 'x',
  current, measures: [], packages: [], advice: null, fitCheck: null,
  renovationPassport: null, interpretations: [], issues: [],
} as unknown as MaatwerkadviesAssessment;

/** The panel as a step: "leave step" unmounts it, as the shell does. */
function Shell() {
  const { state, dispatch } = useEnergy();
  const { docState, docDispatch } = useDocumentManager();
  const [inStep, setInStep] = useState(true);
  const loaded = useRef(false);
  useEffect(() => {
    if (loaded.current) return;
    loaded.current = true;
    dispatch({ type: 'SET_PROJECT', payload: structuredClone(project) });
  }, [dispatch]);
  if (!state.project.maatwerkadvies) return null;
  return <>
    {inStep && <MaatwerkadviesPanel />}
    <button type="button" onClick={() => setInStep((value) => !value)}>toggle step</button>
    <button type="button" onClick={() => dispatch({ type: 'SET_NTA_CALCULATION',
      payload: { ...(state.project.ntaCalculation ?? {}), normVersion: '2024' } as never })}>edition 2024</button>
    <button type="button" onClick={() => docDispatch({ type: 'DOC_NEW',
      payload: { id: 'doc-b', project: { ...state.project, id: 'project-b' } } })}>new document with the same advice</button>
    <button type="button" onClick={() => docDispatch({ type: 'DOC_SET_ACTIVE', payload: docState.documents[0].id })}>first document</button>
  </>;
}

const resultShown = () => screen.queryByRole('cell', { name: 'Q' }) !== null;

async function calculated() {
  vi.stubEnv('DEV', true);
  vi.stubGlobal('fetch', vi.fn(async () => new Response(JSON.stringify(assessment))));
  const user = userEvent.setup();
  renderWithProviders(<Shell />);
  await user.click(await screen.findByRole('button', { name: 'Calculate' }));
  await screen.findByRole('cell', { name: 'Q' });
  return user;
}

describe('maatwerkadvies result per document', () => {
  afterEach(() => { vi.unstubAllGlobals(); vi.unstubAllEnvs(); });

  it('keeps the result when the step is left and opened again', async () => {
    const user = await calculated();
    await user.click(screen.getByRole('button', { name: 'toggle step' }));
    expect(resultShown()).toBe(false);
    // Before: the panel kept the result itself, so it was gone here.
    await user.click(screen.getByRole('button', { name: 'toggle step' }));
    expect(resultShown()).toBe(true);
  }, SLOW_UI_TEST_MS);

  it('does not show the kept result after the input changed while the step was closed', async () => {
    const user = await calculated();
    await user.click(screen.getByRole('button', { name: 'toggle step' }));
    await user.click(screen.getByRole('button', { name: 'edition 2024' }));
    await user.click(screen.getByRole('button', { name: 'toggle step' }));
    expect(resultShown()).toBe(false);
  }, SLOW_UI_TEST_MS);

  it('shows another document none of it', async () => {
    const user = await calculated();
    await user.click(screen.getByRole('button', { name: 'toggle step' }));
    await user.click(screen.getByRole('button', { name: 'new document with the same advice' }));
    await user.click(screen.getByRole('button', { name: 'toggle step' }));
    expect(resultShown()).toBe(false);
    // One project at a time (main, 8 Oct 2026): the new document replaces the first, so there is no switching back.
  }, SLOW_UI_TEST_MS);
});

describe('maatwerkadvies screen table', () => {
  afterEach(() => { vi.unstubAllGlobals(); vi.unstubAllEnvs(); });

  it('has the TO_juli and heat columns of the report, rounded the same', async () => {
    await calculated();
    const row = screen.getByRole('cell', { name: 'Q' }).closest('tr')!;
    const cells = within(row).getAllByRole('cell').map((cell) => cell.textContent);
    const headers = screen.getAllByRole('columnheader').map((cell) => cell.textContent);
    expect(headers).toHaveLength(13);
    expect(cells[headers.indexOf('TOjuli [K]') - 1]).toBe('1.23');
    expect(cells[headers.indexOf('Heat [kWh]') - 1]).toBe('4,321');

    // The report: same columns, same rounding (Dutch notation).
    const html = generateMaatwerkadviesReportHTML(project, project.maatwerkadvies!, assessment);
    const head = html.slice(html.indexOf('<th>Variant</th>'), html.indexOf('</tr></thead>', html.indexOf('<th>Variant</th>')));
    expect(head.match(/<th>/g)).toHaveLength(13);
    expect(html).toContain('<td class="n">1,23</td>');
    expect(html).toContain('<td class="n">4.321</td>');
  }, SLOW_UI_TEST_MS);
});
