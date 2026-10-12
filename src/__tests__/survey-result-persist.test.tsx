import { useState } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { useDocumentManager, useEnergy } from '../context/EnergyContext';
import { BasisopnamePanel } from '../components/BasisopnamePanel/BasisopnamePanel';
import { resultMatches } from '../context/SurveyResultProvider';
import type { StoredSurvey } from '../core/nta/SurveyTemplates';
import type { OpnameAssessment } from '../core/nta/KernelClient';
import { renderWithProviders, userEvent } from './test-utils';
import { SLOW_UI_TEST_MS } from './timeouts';

// Follow-up of review 36: the survey result is kept per document, with the survey
// and edition it was calculated from, so leaving the step and coming back keeps it.

const SOURCE = 'ISSO 82.1 p. 113 (table 9.7)';
const assessment = {
  status: 'calculated_unverified', scope: 'x', source: 'ISSO 82.1',
  appliedDefaults: [{ rule: 'chp_engine_unknown_gas', path: 'heating.generator.engine', value: 'gas_engine', source: SOURCE }],
  warnings: [], issues: [], derivedInput: null,
  performance: { indicativeLabelClass: 'C', needIndicatorKwhPerM2Year: 120, primaryFossilIndicatorKwhPerM2Year: 210.5, renewableSharePercent: 4 },
  referenceVerified: false,
};

/** The survey step can be left (another step shown) and the document switched, as in the shell. */
function Shell() {
  const [step, setStep] = useState<'survey' | 'other'>('survey');
  const { state, dispatch } = useEnergy();
  const { docState, docDispatch } = useDocumentManager();
  return <>
    {step === 'survey' ? <BasisopnamePanel /> : <p>other step</p>}
    <button type="button" onClick={() => setStep(step === 'survey' ? 'other' : 'survey')}>toggle step</button>
    <button type="button" onClick={() => dispatch({ type: 'SET_NTA_CALCULATION',
      payload: { ...(state.project.ntaCalculation ?? {}), normVersion: '2024' } as never })}>edition 2024</button>
    <button type="button" onClick={() => docDispatch({ type: 'DOC_NEW',
      payload: { id: 'doc-b', project: { ...state.project, id: 'project-b' } } })}>new document with the same survey</button>
    <button type="button" onClick={() => docDispatch({ type: 'DOC_SET_ACTIVE', payload: docState.documents[0].id })}>first document</button>
  </>;
}

const resultShown = () => screen.queryByText(SOURCE) !== null;

describe('survey result per document', () => {
  afterEach(() => { vi.unstubAllGlobals(); });

  it('keeps the result when the survey step is left and opened again', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({ json: async () => assessment })));
    const user = userEvent.setup();
    renderWithProviders(<Shell />);
    await user.click(screen.getByRole('button', { name: 'Start utility survey' }));
    await user.click(screen.getByRole('button', { name: 'Calculate survey' }));
    expect(await screen.findByText(SOURCE)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'toggle step' }));
    expect(resultShown()).toBe(false);
    await user.click(screen.getByRole('button', { name: 'toggle step' }));
    // Before: "not calculated" until Opname doorrekenen was run again.
    expect(resultShown()).toBe(true);
    // A reason for an applied default is documentation: the result stays.
    await user.type(screen.getByLabelText('Reason for the forfait heating.generator.engine'), 'plate missing');
    expect(resultShown()).toBe(true);
  }, SLOW_UI_TEST_MS);

  it('drops the result when the survey or the edition changes', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({ json: async () => assessment })));
    const user = userEvent.setup();
    renderWithProviders(<Shell />);
    await user.click(screen.getByRole('button', { name: 'Start utility survey' }));
    await user.click(screen.getByRole('button', { name: 'Calculate survey' }));
    expect(await screen.findByText(SOURCE)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Add calculation zone' }));
    expect(resultShown()).toBe(false);

    await user.click(screen.getByRole('button', { name: 'Calculate survey' }));
    expect(await screen.findByText(SOURCE)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'toggle step' }));
    await user.click(screen.getByRole('button', { name: 'edition 2024' }));
    await user.click(screen.getByRole('button', { name: 'toggle step' }));
    expect(resultShown()).toBe(false);
  }, SLOW_UI_TEST_MS);

  it('shows no result of another document', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({ json: async () => assessment })));
    const user = userEvent.setup();
    renderWithProviders(<Shell />);
    await user.click(screen.getByRole('button', { name: 'Start utility survey' }));
    await user.click(screen.getByRole('button', { name: 'Calculate survey' }));
    expect(await screen.findByText(SOURCE)).toBeInTheDocument();
    // The other document has the very same survey, but was not calculated.
    await user.click(screen.getByRole('button', { name: 'new document with the same survey' }));
    expect(resultShown()).toBe(false);
    expect(screen.getByRole('button', { name: 'Calculate survey' })).toBeInTheDocument();
    // One project at a time (main, 8 Oct 2026): the new document replaces the first, so there is no switching back.
  }, SLOW_UI_TEST_MS);

  it('matches a result only to the survey and edition it was calculated from', () => {
    const survey: StoredSurvey = { kind: 'utility', survey: { zones: [], inklapRedenen: {} } } as never;
    const entry = { survey, edition: null, result: assessment as unknown as OpnameAssessment };
    expect(resultMatches(entry, survey, null)).toBe(true);
    expect(resultMatches(entry, { ...survey, survey: { zones: [], inklapRedenen: { a: 'b' } } } as never, null)).toBe(true);
    expect(resultMatches(entry, { ...survey, survey: { zones: [{ id: 'z' }] } } as never, null)).toBe(false);
    expect(resultMatches(entry, { ...survey, kind: 'residential' } as never, null)).toBe(false);
    expect(resultMatches(entry, survey, '2024')).toBe(false);
    expect(resultMatches(entry, undefined, null)).toBe(false);
    expect(resultMatches(undefined, survey, null)).toBe(false);
  });
});
