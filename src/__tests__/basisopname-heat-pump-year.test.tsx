/**
 * The survey's heat-pump device years (ISSO 82.1 p. 28), which NTA 8800:2020+A1
 * uses for the auxiliary energy of formula 9.85.
 */
import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { BasisopnamePanel } from '../components/BasisopnamePanel/BasisopnamePanel';
import { renderWithProviders, userEvent } from './test-utils';

function Editor() {
  const { state } = useEnergy();
  return <><BasisopnamePanel />
    <output data-testid="survey">{JSON.stringify(state.project.basisopname ?? null)}</output></>;
}

const generatorOf = () =>
  JSON.parse(screen.getByTestId('survey').textContent ?? 'null')?.survey.heating.generator as Record<string, unknown>;

describe('survey heat-pump years', () => {
  it('records the manufacture and installation year of a heat pump in the kernel shape', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Editor />);
    await user.click(screen.getByRole('button', { name: 'Start dwelling survey' }));
    // The boiler has its manufacture year, but no installation year field.
    expect(screen.getAllByLabelText('Manufacture year')).toHaveLength(1);
    expect(screen.queryByLabelText('Installation year')).toBeNull();

    await user.selectOptions(screen.getAllByLabelText('Generator')[0], 'heat_pump');
    expect(screen.getByText(/Only NTA 8800:2020\+A1 uses the device year of a heat pump/)).toBeInTheDocument();
    await user.type(screen.getByLabelText('Manufacture year'), '2016');
    await user.type(screen.getByLabelText('Installation year'), '2017');
    expect(generatorOf()).toMatchObject({ kind: 'heat_pump', manufactureYear: 2016, installationYear: 2017 });

    // Another generator kind drops the heat-pump fields.
    await user.selectOptions(screen.getAllByLabelText('Generator')[0], 'gas_air_heater');
    expect(generatorOf()).not.toHaveProperty('installationYear');
  }, 60000);
});
