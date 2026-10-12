import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { useEffect, useRef } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { act, screen } from '@testing-library/react';
import { useDocumentManager, useEnergy } from '../context/EnergyContext';
import type { IProject } from '../core/energy/types';
import { emptyMaatwerkadvies, MaatwerkadviesPanel } from '../components/MaatwerkadviesPanel/MaatwerkadviesPanel';
import { generateMaatwerkadviesReportHTML } from '../core/report/MaatwerkadviesReport';
import type { MaatwerkadviesAssessment, MwaVariantResult } from '../core/nta/KernelClient';
import { renderWithProviders, userEvent } from './test-utils';
import { SLOW_UI_TEST_MS } from './timeouts';

// Review 38: a maatwerkadvies result is only shown for the input it was calculated from.

const dwelling = JSON.parse(readFileSync(resolve(process.cwd(), 'training-data/nta8800-example-terraced-dwelling.json'), 'utf8')) as IProject;
const project: IProject = { ...structuredClone(dwelling), maatwerkadvies: emptyMaatwerkadvies() };

const variant = (labelClass: string): MwaVariantResult => ({
  id: 'current', kind: 'current', name: '', valid: true,
  label: { labelClass, primaryFossilIndicatorKwhPerM2: 200 },
  actualUse: null, savings: null, investmentEur: 0, simplePaybackYears: null, netPresentValueEur: null,
  issues: [], systemChecks: [], phasing: [],
} as unknown as MwaVariantResult);

const assessment = (labelClass = 'Q') => ({
  status: 'calculated_unverified', attestStatus: 'unattested', kernelVersion: '0.4.0', targetNormVersion: '2025+C1',
  normVersion: '2025+C1', registrationEligible: true, inputFingerprint: 'x',
  current: variant(labelClass), measures: [], packages: [], advice: null, fitCheck: null,
  renovationPassport: null, interpretations: [], issues: [],
}) as unknown as MaatwerkadviesAssessment;

function Shell() {
  const { state, dispatch } = useEnergy();
  const { docState, docDispatch } = useDocumentManager();
  const loaded = useRef(false);
  useEffect(() => {
    if (loaded.current) return;
    loaded.current = true;
    dispatch({ type: 'SET_PROJECT', payload: structuredClone(project) });
  }, [dispatch]);
  if (!state.project.maatwerkadvies) return null;
  return <>
    <MaatwerkadviesPanel />
    <button type="button" onClick={() => dispatch({ type: 'SET_NTA_CALCULATION',
      payload: { ...(state.project.ntaCalculation ?? {}), normVersion: '2024' } as never })}>edition 2024</button>
    <button type="button" onClick={() => docDispatch({ type: 'DOC_NEW',
      payload: { id: 'doc-b', project: { ...state.project, id: 'project-b' } } })}>new document with the same advice</button>
    <button type="button" onClick={() => docDispatch({ type: 'DOC_SET_ACTIVE', payload: docState.documents[0].id })}>first document</button>
  </>;
}

/** The label class of the current situation in the result table. */
const resultShown = () => screen.queryByRole('cell', { name: 'Q' }) !== null;

describe('review 38: maatwerkadvies', () => {
  afterEach(() => { vi.unstubAllGlobals(); vi.unstubAllEnvs(); });

  it('drops the result on an edition change and never shows it for another document', async () => {
    vi.stubEnv('DEV', true);
    vi.stubGlobal('fetch', vi.fn(async () => new Response(JSON.stringify(assessment()))));
    const user = userEvent.setup();
    renderWithProviders(<Shell />);
    await user.click(await screen.findByRole('button', { name: 'Calculate' }));
    await screen.findByRole('cell', { name: 'Q' });

    // Before: the 2025+C1 result stayed after switching the base to 2024.
    await user.click(screen.getByRole('button', { name: 'edition 2024' }));
    expect(resultShown()).toBe(false);

    await user.click(screen.getByRole('button', { name: 'Calculate' }));
    await screen.findByRole('cell', { name: 'Q' });
    // Before: document B showed document A's result.
    await user.click(screen.getByRole('button', { name: 'new document with the same advice' }));
    expect(resultShown()).toBe(false);
    // One project at a time (main, 8 Oct 2026): the new document replaces the first, so there is no switching back.
  }, SLOW_UI_TEST_MS);

  it('does not show a result that arrives after the advice was edited', async () => {
    vi.stubEnv('DEV', true);
    let answer: (response: Response) => void = () => {};
    vi.stubGlobal('fetch', vi.fn(() => new Promise<Response>((done) => { answer = done; })));
    const user = userEvent.setup();
    renderWithProviders(<Shell />);
    await user.click(await screen.findByRole('button', { name: 'Calculate' }));
    // Edited while the kernel calculates.
    await user.click(screen.getByRole('button', { name: 'Add measure' }));
    await act(async () => { answer(new Response(JSON.stringify(assessment()))); });
    // Before: the result for the advice without the measure was shown.
    expect(resultShown()).toBe(false);
  }, SLOW_UI_TEST_MS);
});

describe('review 38: maatwerkadvies report', () => {
  it('names no package and no automatic choice when no package was advised', () => {
    const html = generateMaatwerkadviesReportHTML(project, project.maatwerkadvies!, assessment('C'));
    expect(html).toContain('<strong>Best passend pakket:</strong> geen pakket geadviseerd');
    expect(html).not.toContain('hoogste netto contante waarde, door adviseur te bevestigen');
  });

  it('lists the issues of a partly calculated advice, as the screen does', () => {
    const partial = { ...assessment('C'), status: 'partially_calculated',
      issues: [{ code: 'measure_patch_failed', path: 'measures[0].patch[0]', detail: 'path not found: /x' }] } as unknown as MaatwerkadviesAssessment;
    const html = generateMaatwerkadviesReportHTML(project, project.maatwerkadvies!, partial);
    expect(html).toContain('<h2>Meldingen</h2>');
    expect(html).toContain('measures[0].patch[0]');
    expect(html).toContain('path not found: /x');
  });
});
