/**
 * Herlabelen of a basisopname project (feedback 8 Oct 2026): the survey
 * changes per item in words with their 6a/6b verdict, evidence per change,
 * and the original label's data in the registration.
 */
import { useEffect } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { createDefaultProject, useEnergy } from '../context/EnergyContext';
import { SurveyRelabel } from '../components/SurveyWizard/SurveyRelabel';
import { relabelRows, relabelTakeovers, rowProofs } from '../core/survey/surveyRelabel';
import { surveyRegistration } from '../core/survey/surveyRegistration';
import { labelInputSha256 } from '../core/nta/Registration';
import { serializeProject } from '../core/io/ProjectSerializer';
import type { RelabelChange } from '../core/nta/KernelClient';
import type { IProject } from '../core/energy/types';

const change = (path: string, verdict: RelabelChange['verdict'], before: unknown, after: unknown): RelabelChange =>
  ({ path: `/basisopname/survey/${path}`, verdict, before, after, cluster: 'x' });

const CHANGES: RelabelChange[] = [
  change('envelope/surfaces/0/insulation/kind', 'allowed', 'none_or_unknown', 'thickness'),
  change('envelope/surfaces/0/insulation/thicknessMm', 'allowed', undefined, 120),
  change('heating/generator/kind', 'not_allowed', 'boiler', 'heat_pump'),
  change('heating/control', 'allowed', 'unknown', 'weather_compensated'),
  change('pv/0/panelAreaM2', 'review', 8, 16),
];

vi.mock('../core/nta/KernelClient', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../core/nta/KernelClient')>()),
  assessRelabelWithRust: vi.fn(async () => ({
    source: 'BRL 9500-W', scheme: 'w', allowed: false, needsReview: true, changes: CHANGES,
    originalLabelInputHash: 'a', currentLabelInputHash: 'b',
  })),
  assessResidentialSurveyWithRust: vi.fn(async () => ({ issues: [], appliedDefaults: [], derivedInput: null, performance: null })),
}));

function surveyProject(survey: Record<string, unknown>, registration: IProject['registration']): IProject {
  return {
    ...createDefaultProject(), zones: [],
    basisopname: { kind: 'residential', survey, progress: {}, surveyDate: '2026-09-01' },
    registration,
  } as IProject;
}

const SURVEY = {
  envelope: { surfaces: [{ id: 'dak', element: 'roof', insulation: { kind: 'thickness', thicknessMm: 120 } }], windows: [] },
  heating: { generator: { kind: 'heat_pump' }, control: 'weather_compensated' },
  pv: [{ id: 'pv1', panelAreaM2: 16 }],
};

describe('survey relabel', () => {
  it('takes the changes of one item together', () => {
    const rows = relabelRows(CHANGES);
    expect(rows.map((row) => [row.path, row.verdict, row.lines.map((line) => line.field)])).toEqual([
      ['envelope.surfaces[0]', 'allowed', ['insulation']],
      ['heating', 'not_allowed', ['generator', 'control']],
      ['pv[0]', 'review', ['panelAreaM2']],
    ]);
    const pv = rows[2];
    expect(rowProofs(pv, [{ id: 'e1', kind: 'invoice', fileName: 'f.pdf', sha256: 'x', relabelProof: 'specified_invoice', linkedPaths: [pv.pointer] }]).complete).toBe(false);
  });

  it('keeps the original survey date and names the improvement date', () => {
    const original = { registration: { surveyDate: '2021-04-24', epOnlineNumber: 'EP-1', certificateNumber: 'C-1' } };
    expect(relabelTakeovers(original, 'oud.json')).toMatchObject({
      messageType: 'relabel', surveyDate: '2021-04-24', originalEpOnlineNumber: 'EP-1', originalCertificateNumber: 'C-1',
      originalDossierReference: 'oud.json',
    });
    const project = surveyProject(SURVEY, { messageType: 'relabel', surveyDate: '2021-04-24' });
    expect(surveyRegistration(project)).toMatchObject({ surveyDate: '2021-04-24', improvementDate: '2026-09-01' });
  });

  it('hashes the survey of a survey project, without the reasons for defaults', async () => {
    const a = surveyProject(SURVEY, {});
    const b = surveyProject({ ...SURVEY, inklapRedenen: { x: 'y' } }, {});
    const c = surveyProject({ ...SURVEY, pv: [] }, {});
    expect(await labelInputSha256(a)).toBe(await labelInputSha256(b));
    expect(await labelInputSha256(a)).not.toBe(await labelInputSha256(c));
  });

  it('compares with the original file and shows the changes in words', async () => {
    const user = userEvent.setup();
    const original = surveyProject({ ...SURVEY, heating: { generator: { kind: 'boiler' }, control: 'unknown' },
      envelope: { surfaces: [{ id: 'dak', element: 'roof', insulation: { kind: 'none_or_unknown' } }], windows: [] } },
      { surveyDate: '2021-04-24', epOnlineNumber: 'EP-1' });
    function Harness() {
      const { state, dispatch } = useEnergy();
      useEffect(() => {
        const project = surveyProject(SURVEY, {});
        dispatch({ type: 'SET_BASISOPNAME', payload: project.basisopname! });
        dispatch({ type: 'UPDATE_PROJECT_INFO', payload: { registration: {} } });
      }, [dispatch]);
      return <>
        {state.project.basisopname && <SurveyRelabel actions={{ navigate: vi.fn() }} />}
        <output data-testid="registration">{JSON.stringify(state.project.registration ?? null)}</output>
      </>;
    }
    renderWithProviders(<Harness />);
    await screen.findByRole('heading', { name: 'Relabel' });
    const input = document.getElementById('srel-file') as HTMLInputElement;
    await user.upload(input, new File([serializeProject(original)], 'oud.json', { type: 'application/json' }));
    expect(await screen.findAllByText('Roof 1')).toHaveLength(2);
    expect(screen.getByText(/^Insulation: .+ → 120 mm$/)).toBeInTheDocument();
    expect(screen.getByText('Relabelling is not possible: there is a change from 6b. A new survey is needed.')).toBeInTheDocument();
    await waitFor(() => expect(JSON.parse(screen.getByTestId('registration').textContent ?? 'null'))
      .toMatchObject({ messageType: 'relabel', surveyDate: '2021-04-24', originalEpOnlineNumber: 'EP-1' }));
  }, 60000);
});
