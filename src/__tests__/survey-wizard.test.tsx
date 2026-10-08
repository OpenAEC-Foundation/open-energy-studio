/**
 * The basisopname question flow (UI redesign 2026-10): start, choice cards,
 * skip and come back later, the navigation of the steps and questions, and
 * the Controle page with everything on one page.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen, within } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { useEnergy } from '../context/EnergyContext';
import { WorkflowNav } from '../components/shell/WorkflowNav';
import { StepRouter } from '../components/shell/StepRouter';
import type { ShellActions } from '../components/shell/ShellActions';
import { stepStatuses } from '../core/nta/stepStatus';
import {
  markProgress, questionForPath, stepForKind, stepState, surveyStep, surveySteps,
} from '../core/survey/surveyFlow';
import { missingSurveyAnswers, surveyExample, surveyTemplate, withSurveySources } from '../core/nta/SurveyTemplates';

afterEach(() => vi.unstubAllGlobals());

function actions(): ShellActions {
  return {
    newProject: vi.fn(), openProject: vi.fn(), saveProject: vi.fn(), saveAsProject: vi.fn(), calculate: vi.fn(),
    openDialog: vi.fn(), navigate: vi.fn(), exportReport: vi.fn(), printReport: vi.fn(), exportIFC: vi.fn(),
    exportModelIFC: vi.fn(), exportUNIEC3: vi.fn(), importUNIEC3: vi.fn(), exportVABI: vi.fn(), importVABI: vi.fn(),
    openSettings: vi.fn(), openFeedback: vi.fn(), openPalette: vi.fn(), toggleInspector: vi.fn(), togglePreview: vi.fn(),
  };
}

function Shell() {
  const { state, dispatch } = useEnergy();
  const wired: ShellActions = { ...actions(), navigate: (target) => dispatch({ type: 'NAVIGATE', payload: target }) };
  const statuses = stepStatuses(state.project, null);
  return <>
    <button type="button" onClick={() => wired.navigate({ step: 'survey' })}>Start the basic survey</button>
    <WorkflowNav project={state.project} route={state.route} statuses={statuses} actions={wired} />
    <main><StepRouter project={state.project} route={state.route} statuses={statuses} actions={wired} /></main>
    <output data-testid="state">{JSON.stringify({ route: state.route, survey: state.project.basisopname ?? null })}</output>
  </>;
}

const snapshot = () => JSON.parse(screen.getByTestId('state').textContent ?? '{}');

describe('survey flow definition', () => {
  it('orders the steps per kind and maps old section ids and kernel paths', () => {
    expect(surveySteps('residential').map((step) => step.id)).toEqual(
      ['woning', 'gevels', 'dak-vloer', 'verwarming', 'warm-water', 'ventilatie', 'koeling', 'zonnepanelen', 'controle', 'label']);
    expect(surveySteps('utility')[0].id).toBe('gebouw');
    expect(stepForKind('general', 'utility')).toBe('gebouw');
    expect(stepForKind('zones', 'residential')).toBe('woning');
    const stored = surveyExample('residential');
    const surfaces = (stored.survey.envelope as { surfaces: Array<{ element: string }> }).surfaces;
    const roof = surfaces.findIndex((surface) => surface.element === 'roof');
    expect(questionForPath(`basisopname.envelope.surfaces[${roof}].insulation`, stored).step).toBe('dak-vloer');
    expect(questionForPath('heating.generator.boilerType', stored)).toEqual({ step: 'verwarming', question: 'details' });
    expect(questionForPath('heating.emitters', stored)).toEqual({ step: 'verwarming', question: 'afgifte' });
  });

  it('marks a step skipped when its questions are skipped and done once answered', () => {
    const step = surveyStep('verwarming', 'residential');
    let progress = markProgress(undefined, 'verwarming.toestel', 'done');
    expect(stepState(step, progress)).toBe('partial');
    progress = markProgress(progress, 'verwarming.details', 'skipped');
    progress = markProgress(progress, 'verwarming.afgifte', 'skipped');
    expect(stepState(step, progress)).toBe('skipped');
    progress = markProgress(progress, 'verwarming.details', 'done');
    progress = markProgress(progress, 'verwarming.afgifte', 'done');
    expect(stepState(step, progress)).toBe('done');
    // An answered question stays answered when skipped later.
    expect(markProgress(progress, 'verwarming.toestel', 'skipped').skipped).not.toContain('verwarming.toestel');
  });
});

describe('empty dwelling survey', () => {
  it('starts without example surfaces or sources and names the answers still needed', () => {
    const stored = surveyTemplate('residential');
    const survey = stored.survey as { envelope: { surfaces: unknown[]; windows: unknown[] }; sourceReference: string };
    expect(survey.envelope.surfaces).toEqual([]);
    expect(survey.envelope.windows).toEqual([]);
    expect(survey.sourceReference).toBe('');
    expect(missingSurveyAnswers(stored)).toEqual(expect.arrayContaining([
      'dwelling', 'constructionYear', 'usableFloorAreaM2', 'buildingHeightM', 'construction.floor', 'construction.wall',
      'heating.generator', 'heating.emitters', 'hotWater.generator', 'ventilation.principle',
    ]));
    // The required answer leads to its question.
    expect(questionForPath('heating.generator', stored)).toEqual({ step: 'verwarming', question: 'toestel' });
    expect(questionForPath('ventilation.principle', stored)).toEqual({ step: 'ventilatie', question: 'systeem' });
  });

  it('gives items and answer blocks without a source the source of the building data', () => {
    const filled = withSurveySources({
      sourceReference: 'waarneming', areaSourceReference: '', construction: { floor: 'heavy', sourceReference: '' },
      heating: { sourceReference: 'typeplaatje' },
      envelope: { surfaces: [{ id: 'g1', sourceReference: '' }], windows: [{ id: 'r1' }], doors: [{ id: 'd1', sourceReference: 'tekening' }] },
      pv: [{ id: 'pv1', sourceReference: '' }],
    }) as Record<string, any>;
    expect(filled.areaSourceReference).toBe('waarneming');
    expect(filled.construction.sourceReference).toBe('waarneming');
    expect(filled.heating.sourceReference).toBe('typeplaatje');
    expect(filled.envelope.surfaces[0].sourceReference).toBe('waarneming');
    expect(filled.envelope.windows[0].sourceReference).toBe('waarneming');
    expect(filled.envelope.doors[0].sourceReference).toBe('tekening');
    expect(filled.pv[0].sourceReference).toBe('waarneming');
  });
});

describe('survey wizard', () => {
  it('starts a dwelling survey, answers with cards, skips and lists the skipped step', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, json: async () => ({ issues: [], appliedDefaults: [], warnings: [] }) }));
    const user = userEvent.setup();
    renderWithProviders(<Shell />);
    await user.click(screen.getByRole('button', { name: 'Start the basic survey' }));
    await user.click(screen.getByRole('button', { name: /Existing dwelling/ }));
    expect(snapshot().route).toMatchObject({ step: 'survey', sub: 'woning' });
    // The first question is the address of the project (ISSO opnameformulier §1).
    expect(screen.getByRole('heading', { level: 1, name: 'Address and project' })).toBeInTheDocument();
    await user.type(screen.getByRole('textbox', { name: 'Postcode' }), '3013 AL');
    await user.click(screen.getByRole('button', { name: 'Next question' }));
    expect(screen.getByRole('heading', { level: 1, name: 'What kind of dwelling is it?' })).toBeInTheDocument();

    // The navigation now shows the question flow.
    const flow = screen.getByRole('navigation', { name: 'Workflow steps' });
    expect(within(flow).getByText('Survey')).toBeInTheDocument();
    expect(within(flow).getByRole('button', { name: /Heating/ })).toBeInTheDocument();

    // A choice card writes the dwelling type.
    await user.click(screen.getByRole('button', { name: /Detached house/ }));
    expect(snapshot().survey.survey.dwelling).toMatchObject({ kind: 'single_family', position: 'detached' });
    await user.click(screen.getByRole('button', { name: 'Next question' }));
    expect(snapshot().survey.progress.done).toContain('woning.soort');
    expect(snapshot().route).toMatchObject({ sub: 'woning', question: 'basis' });

    // Skip the second question: the step moves on and is listed as still to fill in.
    await user.click(screen.getByRole('button', { name: 'Skip, fill in later' }));
    expect(snapshot().route).toMatchObject({ step: 'survey', sub: 'gevels' });
    expect(snapshot().survey.progress.skipped).toContain('woning.basis');
    const open = screen.getByRole('complementary', { name: 'Provisional outcome and open questions' });
    expect(within(open).getByRole('button', { name: /Dwelling.*skipped/ })).toBeInTheDocument();

    // Heating: a card replaces the generator with its template.
    await user.click(within(flow).getByRole('button', { name: /Heating/ }));
    await user.click(screen.getByRole('button', { name: /Heat pump.*Air, ground/ }));
    expect(snapshot().survey.survey.heating.generator).toEqual({ kind: 'heat_pump', source: 'outdoor_air' });
    // The questions of the current step are in the navigation.
    await user.click(within(flow).getByRole('button', { name: /Emission and pipes/ }));
    expect(snapshot().route).toMatchObject({ sub: 'verwarming', question: 'afgifte' });
    expect(screen.getByRole('heading', { level: 1, name: 'Emission, pipes and control' })).toBeInTheDocument();
  }, 60000);

  it('shows everything on the Controle page with an edit button per step', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, json: async () => ({ issues: [], appliedDefaults: [], warnings: [] }) }));
    const user = userEvent.setup();
    renderWithProviders(<Shell />);
    await user.click(screen.getByRole('button', { name: 'Start the basic survey' }));
    await user.click(screen.getByRole('button', { name: /Existing dwelling/ }));
    await user.click(document.querySelector<HTMLButtonElement>('[data-step="survey-check"]')!);
    expect(screen.getByRole('heading', { level: 1, name: 'Is everything right?' })).toBeInTheDocument();
    const heating = screen.getByRole('region', { name: 'Heating' });
    await user.click(within(heating).getByRole('button', { name: 'Edit' }));
    expect(snapshot().route).toMatchObject({ step: 'survey', sub: 'verwarming' });
  }, 60000);
});
