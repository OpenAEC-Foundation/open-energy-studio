import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { SurfaceEditorDialog } from '../components/dialogs/SurfaceEditorDialog/SurfaceEditorDialog';
import { ThermalBridgeDialog } from '../components/dialogs/ThermalBridgeDialog/ThermalBridgeDialog';
import { renderWithProviders, userEvent } from './test-utils';

function SurfaceEditor() {
  const { state } = useEnergy();
  const surface = state.project.zones[0].surfaces.find((item) => item.id === 'surf-wall-n');
  return <><SurfaceEditorDialog editId="surf-wall-n" onClose={() => {}} />
    <output data-testid="surface-boundary">{surface?.thermalBoundary ?? 'unknown'}</output>
  </>;
}

function BridgeEditor() {
  const { state } = useEnergy();
  const bridge = state.project.zones[0].thermalBridges.find((item) => item.id === 'tb-1');
  return <><ThermalBridgeDialog editId="tb-1" onClose={() => {}} />
    <output data-testid="bridge-boundary">{bridge?.thermalBoundary ?? 'unknown'}</output>
    <output data-testid="bridge-orientations">{(bridge?.orientations ?? []).join(',') || 'pro rata'}</output>
  </>;
}

describe('explicit thermal boundary editors', () => {
  it('persists an outdoor boundary on an existing surface', async () => {
    const user = userEvent.setup();
    renderWithProviders(<SurfaceEditor />);
    expect(screen.getByTestId('surface-boundary')).toHaveTextContent('unknown');
    await user.selectOptions(screen.getByLabelText('Thermal boundary'), 'outdoor');
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByTestId('surface-boundary')).toHaveTextContent('outdoor');
  });

  it('persists a ground boundary on an existing thermal bridge', async () => {
    const user = userEvent.setup();
    renderWithProviders(<BridgeEditor />);
    await user.selectOptions(screen.getByLabelText('Thermal boundary'), 'ground');
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByTestId('bridge-boundary')).toHaveTextContent('ground');
  });

  // NTA 8800 §5.7.2 step 2: the parts a bridge lies on decide its TOjuli orientation.
  it('persists the orientations a thermal bridge lies on', async () => {
    const user = userEvent.setup();
    renderWithProviders(<BridgeEditor />);
    expect(screen.getByTestId('bridge-orientations')).toHaveTextContent('pro rata');
    await user.click(screen.getByRole('checkbox', { name: 'Northwest' }));
    await user.click(screen.getByRole('checkbox', { name: 'South' }));
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByTestId('bridge-orientations')).toHaveTextContent('S,NW');
  });
});
