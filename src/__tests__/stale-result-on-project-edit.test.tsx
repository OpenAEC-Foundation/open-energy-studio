import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { calculateBENGMonthly } from '../core/energy/BENGCalculatorMonthly';
import { renderWithProviders, userEvent } from './test-utils';

function ResultLifecycle() {
  const { state, dispatch } = useEnergy();
  return <>
    <button onClick={() => dispatch({ type: 'SET_RESULT', payload: calculateBENGMonthly(state.project) })}>Calculate</button>
    <button onClick={() => dispatch({ type: 'TOGGLE_PREVIEW' })}>Toggle preview</button>
    <button onClick={() => dispatch({ type: 'UPDATE_HEATING_SYSTEM', payload: {
      id: state.project.heatingSystems[0].id, data: { cop: 4.1 },
    } })}>Change heating</button>
    <output data-testid="result-status">{state.result ? 'present' : 'absent'}</output>
  </>;
}

describe('result lifecycle', () => {
  it('clears an indicative result when project inputs change, but keeps it for UI-only actions', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ResultLifecycle />);
    await user.click(screen.getByRole('button', { name: 'Calculate' }));
    expect(screen.getByTestId('result-status')).toHaveTextContent('present');
    await user.click(screen.getByRole('button', { name: 'Toggle preview' }));
    expect(screen.getByTestId('result-status')).toHaveTextContent('present');
    await user.click(screen.getByRole('button', { name: 'Change heating' }));
    expect(screen.getByTestId('result-status')).toHaveTextContent('absent');
  });
});
