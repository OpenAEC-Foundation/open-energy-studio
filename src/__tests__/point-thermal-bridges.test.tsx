import { describe, expect, it } from 'vitest';
import { fireEvent, screen, within } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { EnvelopeView } from '../components/EnvelopeView/EnvelopeView';
import { PointBridgeDialog } from '../components/dialogs/PointBridgeDialog/PointBridgeDialog';
import { renderWithProviders, userEvent } from './test-utils';

function PointBridgeEditor() {
  const { state, dispatch } = useEnergy();
  return <>
    <button type="button" onClick={() => dispatch({ type: 'OPEN_DIALOG', payload: { type: 'point-bridge' } })}>
      Open point bridge
    </button>
    <EnvelopeView />
    {state.dialog.type === 'point-bridge' && <PointBridgeDialog editId={state.dialog.editId}
      onClose={() => dispatch({ type: 'CLOSE_DIALOG' })} />}
    <output data-testid="point-bridges">{JSON.stringify(state.project.zones[0].pointThermalBridges ?? [])}</output>
    <output data-testid="inventory-complete">{String(state.project.zones[0].pointBridgeInventoryComplete ?? false)}</output>
  </>;
}

describe('point thermal bridge inventory', () => {
  it('requires explicit confirmation, saves a referenced χ, and resets confirmation when deleted', async () => {
    const user = userEvent.setup();
    renderWithProviders(<PointBridgeEditor />);
    expect(screen.getByTestId('inventory-complete')).toHaveTextContent('false');
    await user.click(screen.getByRole('button', { name: 'Confirm full point bridge inventory' }));
    expect(screen.getByTestId('inventory-complete')).toHaveTextContent('true');

    await user.click(screen.getByRole('button', { name: 'Open point bridge' }));
    fireEvent.change(screen.getByLabelText('Name'), { target: { value: 'Corner detail' } });
    fireEvent.change(screen.getByLabelText('Point transmittance χ (W/K)'), { target: { value: '0.04' } });
    await user.selectOptions(screen.getByLabelText('Thermal boundary'), 'outdoor');
    fireEvent.change(screen.getByLabelText('Source or detail reference'), { target: { value: 'Detail D1' } });
    await user.click(screen.getByRole('button', { name: 'Save' }));

    expect(JSON.parse(screen.getByTestId('point-bridges').textContent ?? '[]')).toMatchObject([{
      name: 'Corner detail', chiValue: 0.04, thermalBoundary: 'outdoor',
      sourceReference: 'Detail D1', zoneId: 'zone-main',
    }]);
    expect(screen.getByTestId('inventory-complete')).toHaveTextContent('false');
    await user.click(screen.getByRole('button', { name: 'Confirm full point bridge inventory' }));
    expect(screen.getByTestId('inventory-complete')).toHaveTextContent('true');
    await user.click(within(screen.getByRole('row', { name: /Corner detail/ }))
      .getByRole('button', { name: 'Delete: Corner detail' }));
    await user.click(screen.getByRole('button', { name: 'Yes, delete' }));
    expect(screen.getByTestId('point-bridges')).toHaveTextContent('[]');
    expect(screen.getByTestId('inventory-complete')).toHaveTextContent('false');
  }, 60000);
});
