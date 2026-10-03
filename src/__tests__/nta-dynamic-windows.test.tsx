import { useState } from 'react';
import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import type { IProject } from '../core/energy/types';
import { DynamicWindowsFields } from '../components/NtaPerformancePanel/NtaDynamicWindows';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { renderWithProviders, userEvent } from './test-utils';

const project = {
  id: 'p', name: 'p', description: '', buildingFunction: 'residential', address: '', city: '',
  zones: [{ id: 'z1', surfaces: [
    { id: 's', name: 'Gevel Z', type: 'wall', thermalBoundary: 'outdoor', area: 30, orientation: 'S', constructionId: 'c', zoneId: 'z1',
      windows: [{ id: 'win-S', name: 'Raam Z', area: 8, uValue: 1.1, gValue: 0.6, orientation: 'S', surfaceId: 's' }] },
    { id: 'u', name: 'Wand berging', type: 'wall', thermalBoundary: 'unheated', area: 10, orientation: 'N', constructionId: 'c', zoneId: 'z1',
      windows: [{ id: 'win-U', name: 'Raam berging', area: 1, uValue: 1.1, gValue: 0.6, orientation: 'N', surfaceId: 'u' }] },
  ] }],
  heatingSystems: [], ventilationSystems: [], coolingSystems: [], hotWaterSystems: [], solarPV: [], solarThermal: [], constructions: [],
} as unknown as IProject;

function Harness() {
  const [draft, setDraft] = useState<Draft>({});
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  return <form aria-label="test">
    <DynamicWindowsFields draft={draft} change={change} project={project} />
    <output data-testid="draft">{JSON.stringify(draft)}</output>
  </form>;
}

const current = () => JSON.parse(screen.getByTestId('draft').textContent ?? '{}');

describe('Annex A dynamic windows form', () => {
  it('edits method B and method A with step-2 factors', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    const add = screen.getByLabelText('Add dynamic window');
    // Only outdoor windows can be dynamic (the project route uses them).
    expect([...(add as HTMLSelectElement).options].map((option) => option.value)).toEqual(['', 'win-S']);
    await user.selectOptions(add, 'win-S');
    await user.type(screen.getByLabelText('g_dyn;i (perpendicular)'), '0.6');
    await user.type(screen.getByLabelText('U_dyn;i, W/(m²·K)'), '1.1');
    expect(current().dynamicWindows).toEqual([{ windowId: 'win-S', dynamic: {
      method: 'single_state', state: { id: 'toestand-1', gPerpendicular: 0.6, uValueWPerM2k: 1.1 }, sourceReference: '' } }]);

    await user.selectOptions(screen.getByLabelText('Method'), 'weighted_states');
    let dynamic = current().dynamicWindows[0].dynamic;
    expect(dynamic.states).toHaveLength(2);
    expect(dynamic.states[0]).toEqual({ id: 'toestand-1', gPerpendicular: 0.6, uValueWPerM2k: 1.1 });
    expect(dynamic.solarWeights).toHaveLength(12);
    expect(dynamic.solarWeights[0]).toEqual([null, null]);
    await user.type(screen.getByLabelText('Share of Σ I_sol·Δt (A.2) 1 toestand-2'), '0.25');
    await user.click(screen.getByRole('button', { name: 'Add state' }));
    dynamic = current().dynamicWindows[0].dynamic;
    expect(dynamic.states).toHaveLength(3);
    expect(dynamic.solarWeights[0]).toEqual([null, 0.25, null]);
    expect(dynamic.temperatureWeights[11]).toEqual([null, null, null]);
    await user.click(screen.getAllByRole('button', { name: 'Remove state' })[0]);
    dynamic = current().dynamicWindows[0].dynamic;
    expect(dynamic.states.map((state: Draft) => state.id)).toEqual(['toestand-2', 'toestand-3']);
    expect(dynamic.solarWeights[0]).toEqual([0.25, null]);

    await user.click(screen.getByLabelText('Step 2 correction factors'));
    await user.type(screen.getByLabelText('U correction factor per month 1'), '1.1');
    expect(current().dynamicWindows[0].dynamic.correction.uFactors[0]).toBe(1.1);
    expect(current().dynamicWindows[0].dynamic.correction.gFactors).toHaveLength(12);

    await user.click(screen.getByRole('button', { name: 'Remove dynamic window' }));
    expect(current().dynamicWindows).toEqual([]);
  }, 60_000);
});
