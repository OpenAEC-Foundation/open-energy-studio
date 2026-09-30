import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { PreviewPanel } from '../components/PreviewPanel/PreviewPanel';
import { renderWithProviders, userEvent } from './test-utils';

function ChangeToOffice() {
  const { dispatch } = useEnergy();
  return (
    <button onClick={() => dispatch({ type: 'UPDATE_PROJECT_INFO', payload: { buildingFunction: 'office' } })}>
      Office
    </button>
  );
}

describe('preview label scope', () => {
  it('does not apply residential label thresholds to an office project', async () => {
    const user = userEvent.setup();
    renderWithProviders(<><ChangeToOffice /><PreviewPanel /></>);

    expect(document.querySelector('.preview-energy-label')).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Office' }));

    expect(document.querySelector('.preview-energy-label')).not.toBeInTheDocument();
    expect(screen.getByText(/Energy label classification is not yet implemented/)).toBeInTheDocument();
  });
});
