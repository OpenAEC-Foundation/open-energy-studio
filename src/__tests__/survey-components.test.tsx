/**
 * "Aanklikken wat relevant is" (feedback 9 Oct 2026): optional parts of an
 * installation are asked only once ticked; data never disappears; unticking
 * clears after a question.
 */
import { useEffect } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen, within } from '@testing-library/react';
import { declare, renderWithProviders, userEvent } from './test-utils';
import { useEnergy } from '../context/EnergyContext';
import { BasisopnamePanel } from '../components/BasisopnamePanel/BasisopnamePanel';
import { surveyTemplate } from '../core/nta/SurveyTemplates';
import { componentActive, componentsOf, toggleComponent } from '../core/survey/surveyComponents';

afterEach(() => vi.restoreAllMocks());

describe('survey components', () => {
  it('lists the parts per question and kind', () => {
    expect(componentsOf('hotWaterRest', 'residential')).toEqual(['solarWater', 'showerWtw', 'hotWaterCollective']);
    expect(componentsOf('hotWaterRest', 'utility')).toEqual(['solarWater']);
    expect(componentsOf('ventilation', 'residential')).toEqual(['ventControls', 'ventCombined', 'ventStrips', 'passiveCooling']);
  });

  it('starts a part when ticked, keeps it shown while it has data and clears it when unticked', () => {
    const empty = surveyTemplate('residential');
    expect(componentActive(empty, 'solarWater')).toBe(false);
    const ticked = toggleComponent(empty, 'solarWater', true);
    expect((ticked.survey.hotWater as { solar: unknown[] }).solar).toHaveLength(1);
    expect(componentActive({ ...ticked, components: [] }, 'solarWater')).toBe(true);
    const cleared = toggleComponent(ticked, 'solarWater', false);
    expect((cleared.survey.hotWater as { solar: unknown[] }).solar).toEqual([]);
    expect(componentActive(cleared, 'solarWater')).toBe(false);
    expect((toggleComponent(empty, 'showerWtw', false).survey.hotWater as { showerHeatRecovery: string }).showerHeatRecovery).toBe('none');
  });
});

describe('component bar in the question flow', () => {
  function Harness({ part = 'hotWaterRest' }: { part?: 'hotWaterRest' | 'ventilation' }) {
    const { state, dispatch } = useEnergy();
    useEffect(() => { dispatch({ type: 'SET_BASISOPNAME', payload: surveyTemplate('residential') }); }, [dispatch]);
    return <>{state.project.basisopname && <BasisopnamePanel part={part} />}
      <output data-testid="survey">{JSON.stringify(state.project.basisopname ?? null)}</output></>;
  }
  const stored = () => JSON.parse(screen.getByTestId('survey').textContent ?? '{}');

  it('shows the solar water heater only once ticked and asks before clearing it', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    const bar = within(await screen.findByRole('group', { name: 'What does it have?' }));
    expect(screen.queryByRole('button', { name: /Add solar/i })).toBeNull();
    expect(screen.queryByRole('combobox', { name: /Shower heat recovery/i })).toBeNull();
    await user.click(bar.getByRole('button', { name: 'Solar water heater' }));
    expect(bar.getByRole('button', { name: 'Solar water heater' })).toHaveAttribute('aria-pressed', 'true');
    expect(stored().survey.hotWater.solar).toHaveLength(1);
    expect(stored().components).toEqual(['solarWater']);
    await user.click(bar.getByRole('button', { name: 'Shower heat recovery' }));
    expect(screen.getByRole('combobox', { name: /Shower heat recovery/i })).toBeInTheDocument();

    // Unticking a part with its own answers asks first.
    await user.type(screen.getAllByRole('spinbutton', { name: /area/i })[0], '5');
    const confirm = vi.spyOn(window, 'confirm').mockReturnValue(false);
    await user.click(bar.getByRole('button', { name: 'Solar water heater' }));
    expect(confirm).toHaveBeenCalledOnce();
    expect(stored().survey.hotWater.solar).toHaveLength(1);
    confirm.mockReturnValue(true);
    await user.click(bar.getByRole('button', { name: 'Solar water heater' }));
    expect(stored().survey.hotWater.solar).toEqual([]);
  }, 60000);


  it('asks a declaration only with Met verklaring and clears it with Standaardwaarde', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness part="ventilation" />);
    const bar = within(await screen.findByRole('group', { name: 'What does it have?' }));
    expect(screen.queryByRole('group', { name: 'Evidence for the controls' })).toBeNull();
    await user.click(bar.getByRole('button', { name: /Controls/ }));
    const group = within(screen.getByRole('group', { name: 'Evidence for the controls' }));
    expect(group.getByRole('button', { name: 'Standard value' })).toHaveAttribute('aria-pressed', 'true');
    await declare(user, 'Evidence for the controls', 'BCRG 77');
    expect(stored().survey.ventilation.controls.evidenceReference).toBe('BCRG 77');
    await user.click(group.getByRole('button', { name: 'Standard value' }));
    expect(stored().survey.ventilation.controls.evidenceReference).toBe('');
    expect(screen.queryByRole('textbox', { name: /Declaration number/ })).toBeNull();
  }, 60000);
});
