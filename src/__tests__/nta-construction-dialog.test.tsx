import { afterEach, describe, expect, it, vi } from 'vitest';
import { act, screen, waitFor } from '@testing-library/react';
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
        elements: [{ id: 'construction', route: 'opaque', uValue: 0.21, uRounded: 0.21, uUnrounded: 0.2123, rC: 4.5, rCRounded: 4.5,
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
    expect(screen.getByText(/Applied from the NTA kernel: U = 0\.21\b/)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Save' }));
    const after = JSON.parse(screen.getByTestId('constructions').textContent ?? '[]');
    expect(after).toHaveLength(before + 1);
    expect(after[after.length - 1].slice(1)).toEqual([4.5, 0.21]);
  });

  it('hides stale construction results when the input changes during or after calculation', async () => {
    const assessment = { status: 'calculated_unverified', issues: [], elements: [{
      id: 'construction', route: 'opaque', uRounded: 0.21, rCRounded: 4.5,
    }] };
    let resolveFirst!: (value: { ok: boolean; json: () => Promise<unknown> }) => void;
    const fetchMock = vi.fn()
      .mockImplementationOnce(() => new Promise((resolve) => { resolveFirst = resolve; }))
      .mockResolvedValueOnce({ ok: true, json: async () => assessment });
    vi.stubGlobal('fetch', fetchMock);
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    await user.click(screen.getByRole('button', { name: 'Calculate with the kernel' }));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledOnce());
    await user.selectOptions(screen.getByLabelText('Heat flow'), 'upward');
    await act(async () => resolveFirst({ ok: true, json: async () => assessment }));
    expect(screen.queryByRole('button', { name: 'Apply to this construction' })).not.toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Calculate with the kernel' }));
    expect(await screen.findByRole('button', { name: 'Apply to this construction' })).toBeInTheDocument();
    await user.selectOptions(screen.getByLabelText('Heat flow'), 'horizontal');
    expect(screen.queryByRole('button', { name: 'Apply to this construction' })).not.toBeInTheDocument();
  });
});
