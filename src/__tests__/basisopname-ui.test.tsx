import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { BasisopnamePanel } from '../components/BasisopnamePanel/BasisopnamePanel';
import { renderWithProviders, userEvent } from './test-utils';

function Editor() {
  const { state } = useEnergy();
  return <><BasisopnamePanel />
    <output data-testid="survey">{JSON.stringify(state.project.basisopname ?? null)}</output></>;
}

function stored() {
  return JSON.parse(screen.getByTestId('survey').textContent ?? 'null') as {
    kind: string; survey: Record<string, any>;
  } | null;
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('basisopname panel', () => {
  it('starts a dwelling survey and edits generators, solar and PV in the kernel shape', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Editor />);
    expect(stored()).toBeNull();
    await user.click(screen.getByRole('button', { name: 'Start dwelling survey' }));
    expect(stored()?.kind).toBe('residential');

    // Heating: CHP with a diesel engine and a peak boiler.
    const generators = screen.getAllByRole('combobox', { name: 'Generator' });
    await user.selectOptions(generators[0], 'chp');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Prime mover' }), 'diesel_engine');
    await user.click(screen.getAllByRole('button', { name: 'Add generator' })[0]);
    let survey = stored()!.survey;
    expect(survey.heating.generator).toEqual({ kind: 'chp', electricalPowerKw: 20, engine: 'diesel_engine' });
    expect(survey.heating.additionalGenerators).toEqual([
      { generator: { kind: 'boiler', boilerType: 'hr107', insideThermalBoundary: true }, nominalPowerKw: 20 },
    ]);

    // Hot water: collective, solar water heater and PV with a roof edge.
    await user.click(screen.getByRole('checkbox', { name: 'Collective hot-water system' }));
    await user.click(screen.getByRole('button', { name: 'Add solar water heater' }));
    await user.click(screen.getByRole('button', { name: 'Add PV system' }));
    await user.selectOptions(screen.getByRole('combobox', { name: 'Obstruction (§15.4.7)' }), 'roof_edge');
    survey = stored()!.survey;
    expect(survey.hotWater.collective).toEqual({});
    expect(survey.hotWater.solar[0]).toMatchObject({ id: 'zb1', collector: 'glazed', orientation: 'south' });
    expect(survey.pv[0].shading).toEqual({ method: 'roof_edge', heightM: 1, distanceM: 0.5 });

    // Vertical pipes: explicitly none.
    await user.selectOptions(screen.getByRole('combobox', { name: 'Vertical pipes (§7.2.4)' }), 'none');
    expect(stored()!.survey.verticalPipes).toEqual([]);
  });

  it('adds a dwelling cooling system in the ISSO 82.1 chapter 10 shape', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Editor />);
    await user.click(screen.getByRole('button', { name: 'Start dwelling survey' }));
    await user.click(screen.getByRole('checkbox', { name: 'Building-bound cooling system present' }));
    let survey = stored()!.survey;
    expect(survey.coolingPresent).toBe(true);
    expect(survey.cooling).toMatchObject({ generator: 'room_air_conditioner', emitter: 'split_indoor_units_on_wall', waterBased: false });
    await user.selectOptions(screen.getByRole('combobox', { name: 'Cooling generator' }), 'closed_ground_loop');
    await user.click(screen.getByRole('checkbox', { name: 'Collective cooling generator (several dwellings)' }));
    survey = stored()!.survey;
    expect(survey.cooling.generator).toBe('closed_ground_loop');
    expect(survey.coolingCollective).toBe(true);
    await user.click(screen.getByRole('checkbox', { name: 'Building-bound cooling system present' }));
    survey = stored()!.survey;
    expect(survey.cooling).toBeNull();
    expect(survey.coolingPresent).toBe(false);
  });

  it('runs the kernel route and records a reason per applied default', async () => {
    const assessment = {
      status: 'calculated_unverified', scope: 'x', source: 'ISSO 82.1',
      appliedDefaults: [{ rule: 'chp_engine_unknown_gas', path: 'heating.generator.engine', value: 'gas_engine', source: 'ISSO 82.1 p. 113 (table 9.7)' }],
      warnings: [], issues: [], derivedInput: null,
      performance: { indicativeLabelClass: 'C', needIndicatorKwhPerM2Year: 120, primaryFossilIndicatorKwhPerM2Year: 210.5, renewableSharePercent: 4 },
      referenceVerified: false,
    };
    const fetchMock = vi.fn(async () => ({ json: async () => assessment }));
    vi.stubGlobal('fetch', fetchMock);
    const user = userEvent.setup();
    renderWithProviders(<Editor />);
    await user.click(screen.getByRole('button', { name: 'Start utility survey' }));
    await user.click(screen.getByRole('button', { name: 'Calculate survey' }));
    expect(await screen.findByText('ISSO 82.1 p. 113 (table 9.7)')).toBeInTheDocument();
    expect(fetchMock).toHaveBeenCalledWith('/api/v1/nta8800/opname/utility', expect.anything());
    expect(screen.getByText('C')).toBeInTheDocument();
    await user.type(screen.getByRole('textbox', { name: 'Reason for the forfait heating.generator.engine' }), 'plate missing');
    expect(stored()!.survey.inklapRedenen).toEqual({ 'heating.generator.engine': 'plate missing' });
    // The result stays while the reason is typed.
    expect(screen.getByText('ISSO 82.1 p. 113 (table 9.7)')).toBeInTheDocument();
  });
});
