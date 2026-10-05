import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { ConstructionEditorDialog } from '../components/dialogs/ConstructionEditorDialog/ConstructionEditorDialog';
import { SURFACE_RESISTANCE } from '../core/energy/Constants';
import { renderWithProviders, userEvent } from './test-utils';

afterEach(() => {
  vi.unstubAllGlobals();
});

function Harness() {
  const { state } = useEnergy();
  return <>
    <ConstructionEditorDialog onClose={() => undefined} />
    <output data-testid="constructions">{JSON.stringify(state.project.constructions)}</output>
  </>;
}

const assessment = (exteriorSurfaceResistance: number) => ({
  status: 'calculated_unverified', scope: 'x', issues: [], bridges: [], deltaUForfait: null, interpretations: [],
  referenceVerified: false,
  elements: [{ id: 'construction', route: 'opaque', uValue: 0.4, uRounded: 0.4, uUnrounded: 0.4, rC: 2.33, rCRounded: 2.33,
    opaque: { exteriorSurfaceResistance }, window: null, forfait: null }],
});

describe('exterior surface resistance of a project construction (NTA 8800 table C.2, 8.4.2.1)', () => {
  it('stores R_se 0 from a kernel construction without exterior air', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, json: async () => assessment(0) }));
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    await user.click(screen.getByRole('button', { name: 'Calculate with the kernel' }));
    await user.click(await screen.findByRole('button', { name: 'Apply to this construction' }));
    await user.click(screen.getByRole('button', { name: 'Save' }));
    const saved = JSON.parse(screen.getByTestId('constructions').textContent ?? '[]');
    expect(saved[saved.length - 1]).toMatchObject({ uValue: 0.4, exteriorSurfaceResistance: 0 });
  }, 60000);

  it('leaves the default R_se 0,04 implicit', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, json: async () => assessment(0.04) }));
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    await user.click(screen.getByRole('button', { name: 'Calculate with the kernel' }));
    await user.click(await screen.findByRole('button', { name: 'Apply to this construction' }));
    await user.click(screen.getByRole('button', { name: 'Save' }));
    const saved = JSON.parse(screen.getByTestId('constructions').textContent ?? '[]');
    expect(saved[saved.length - 1]).not.toHaveProperty('exteriorSurfaceResistance');
  }, 60000);

  it('keeps R_se 0,04 for internal surfaces', () => {
    expect(SURFACE_RESISTANCE.internal.rse).toBe(0.04);
  });

});
