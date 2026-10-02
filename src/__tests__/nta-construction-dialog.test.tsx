import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { ConstructionEditorDialog } from '../components/dialogs/ConstructionEditorDialog/ConstructionEditorDialog';
import { renderWithProviders, userEvent } from './test-utils';

afterEach(() => {
  vi.unstubAllGlobals();
});

function Harness() {
  const { state } = useEnergy();
  return <>
    <ConstructionEditorDialog onClose={() => undefined} />
    <output data-testid="constructions">{JSON.stringify(state.project.constructions.map((item) => [item.name, item.rcValue, item.uValue]))}</output>
  </>;
}

describe('NTA construction section', () => {
  it('sends the layers to the kernel and applies the kernel U-value', async () => {
    const user = userEvent.setup();
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        status: 'calculated_unverified', scope: 'x', issues: [], bridges: [], deltaUForfait: null, interpretations: [],
        referenceVerified: false,
        elements: [{ id: 'construction', route: 'opaque', uValue: 0.2123, uRounded: 0.212, rC: 4.5, rCRounded: 4.5,
          opaque: null, window: null, forfait: null }],
      }),
    });
    vi.stubGlobal('fetch', fetchMock);
    renderWithProviders(<Harness />);
    const before = JSON.parse(screen.getByTestId('constructions').textContent ?? '[]').length;
    await user.click(screen.getByRole('button', { name: 'Calculate with the kernel' }));
    expect(fetchMock).toHaveBeenCalledWith('/api/v1/nta8800/constructions/calculate', expect.anything());
    const body = JSON.parse(fetchMock.mock.calls[0][1].body);
    expect(body.input.elements[0].element.construction.build.layers[0]).toMatchObject({
      kind: 'material', thicknessM: 0.1, conductivity: { method: 'calculated', lambdaCalc: 0.04 } });
    await user.click(await screen.findByRole('button', { name: 'Apply to this construction' }));
    expect(screen.getByText(/Applied from the NTA kernel: U = 0\.212/)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Save' }));
    const after = JSON.parse(screen.getByTestId('constructions').textContent ?? '[]');
    expect(after).toHaveLength(before + 1);
    expect(after[after.length - 1].slice(1)).toEqual([4.5, 0.212]);
  });
});
