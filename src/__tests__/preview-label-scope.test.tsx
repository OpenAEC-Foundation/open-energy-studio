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
  it('never classifies a label outside the Rust kernel', async () => {
    const user = userEvent.setup();
    renderWithProviders(<><ChangeToOffice /><PreviewPanel /></>);

    expect(document.querySelector('.preview-energy-label')).not.toBeInTheDocument();
    expect(screen.getByText(/label class comes only from the NTA 8800 kernel/)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Office' }));

    expect(document.querySelector('.preview-energy-label')).not.toBeInTheDocument();
    expect(screen.getByText(/label class comes only from the NTA 8800 kernel/)).toBeInTheDocument();
  });
});
