import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { createDefaultProject, useEnergy } from '../context/EnergyContext';
import { calculateBENGMonthly } from '../core/energy/BENGCalculatorMonthly';
import { calculateBENG } from '../core/energy/BENGCalculator';
import { calculateTOJuli } from '../core/energy/TOJuli';
import { PreviewPanel } from '../components/PreviewPanel/PreviewPanel';
import { renderWithProviders, userEvent } from './test-utils';

function ZeroAreaButton() {
  const { state, dispatch } = useEnergy();
  return <button onClick={() => dispatch({
    type: 'UPDATE_ZONE',
    payload: { id: state.project.zones[0].id, data: { floorArea: 0 } },
  })}>Zero area</button>;
}

describe('invalid floor area', () => {
  it('stops both legacy calculators instead of silently dividing by 1 m²', () => {
    const project = createDefaultProject();
    project.zones[0].floorArea = 0;
    expect(() => calculateBENGMonthly(project)).toThrow(/floor area greater than zero/);
    expect(() => calculateBENG(project)).toThrow(/floor area greater than zero/);
    expect(() => calculateTOJuli(project, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 0)).toThrow(/floor area greater than zero/);
  });

  it('shows an input error in the live preview', async () => {
    const user = userEvent.setup();
    renderWithProviders(<><ZeroAreaButton /><PreviewPanel /></>);
    await user.click(screen.getByRole('button', { name: 'Zero area' }));
    expect(screen.getByRole('alert')).toHaveTextContent(/floor area greater than zero/i);
    expect(document.querySelector('.preview-energy-label')).not.toBeInTheDocument();
  });
});
