import { describe, expect, it } from 'vitest';
import { screen, within } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { UnheatedSpacesPanel } from '../components/UnheatedSpacesPanel/UnheatedSpacesPanel';
import { SurfaceEditorDialog } from '../components/dialogs/SurfaceEditorDialog/SurfaceEditorDialog';
import { PreviewPanel } from '../components/PreviewPanel/PreviewPanel';
import { calculateBENG } from '../core/energy/BENGCalculator';
import { calculateBENGMonthly } from '../core/energy/BENGCalculatorMonthly';
import { createDefaultProject } from '../context/EnergyContext';
import { renderWithProviders, userEvent } from './test-utils';

function ProjectEditor() {
  const { state } = useEnergy();
  return <><UnheatedSpacesPanel /><SurfaceEditorDialog editId="surf-wall-n" onClose={() => {}} /><PreviewPanel />
    <output data-testid="space-data">{JSON.stringify(state.project.unheatedSpaces ?? [])}</output>
    <output data-testid="surface-space">{state.project.zones[0].surfaces.find((item) => item.id === 'surf-wall-n')?.unheatedSpaceId ?? ''}</output>
  </>;
}

describe('unheated space project editor', () => {
  it('requires a factor source and links a surface to the recorded space', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ProjectEditor />);
    const panel = within(screen.getByRole('region', { name: 'Unheated spaces' }));
    await user.click(screen.getByRole('button', { name: 'Add space' }));
    await user.type(panel.getByLabelText('Name'), 'Garage');
    await user.type(screen.getByLabelText('Reduction factor b'), '0.5');
    await user.click(panel.getByRole('button', { name: 'Save' }));
    expect(screen.getByRole('alert')).toHaveTextContent('factor from 0 to 1');
    await user.type(screen.getByLabelText('Factor source'), 'drawing-b-1');
    await user.click(panel.getByRole('button', { name: 'Save' }));
    const spaces = JSON.parse(screen.getByTestId('space-data').textContent ?? '[]');
    expect(spaces).toMatchObject([{ name: 'Garage', reductionFactor: 0.5, factorSourceReference: 'drawing-b-1' }]);
    expect(screen.getByRole('alert')).toHaveTextContent('cannot include');
    expect(document.querySelector('.preview-energy-label')).not.toBeInTheDocument();
    await user.selectOptions(screen.getByLabelText('Thermal boundary'), 'unheated_space');
    await user.selectOptions(screen.getByLabelText('Unheated space'), spaces[0].id);
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByTestId('surface-space')).toHaveTextContent(spaces[0].id);
  });

  it('does not let either legacy calculator ignore an unheated boundary', () => {
    const project = createDefaultProject();
    project.zones[0].surfaces[0].thermalBoundary = 'unheated_space';
    expect(() => calculateBENG(project)).toThrow(/Unheated space transmission/);
    expect(() => calculateBENGMonthly(project)).toThrow(/Unheated space transmission/);
  });
});
