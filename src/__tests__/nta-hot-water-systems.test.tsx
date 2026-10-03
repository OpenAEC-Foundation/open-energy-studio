import { useState } from 'react';
import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { AdditionalHotWaterSystemsFields } from '../components/NtaPerformancePanel/NtaSystemSections';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { renderWithProviders, userEvent } from './test-utils';

function Harness({ initial, residential }: { initial: Draft; residential: boolean }) {
  const [draft, setDraft] = useState<Draft>(initial);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  return <form aria-label="test">
    <AdditionalHotWaterSystemsFields draft={draft} change={change} residential={residential} />
    <output data-testid="draft">{JSON.stringify(draft)}</output>
  </form>;
}

const current = () => JSON.parse(screen.getByTestId('draft').textContent ?? '{}');

describe('Further hot-water systems (§13.2.4)', () => {
  it('adds a kitchen system with 13.19a taps for a dwelling', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness residential initial={{
      hotWater: {
        need: { method: 'residential', dwellingCount: 1, sourceReference: 'survey' },
        generator: { kind: 'gas_appliance', appliance: 'combi_gaskeur', measuredClass: 'class4', kitchenOnly: false },
      },
    }} />);
    await user.click(screen.getByRole('button', { name: 'Add hot-water system' }));
    const draft = current();
    // The main system serves the bathroom, the new one the kitchen.
    expect(draft.hotWater.connectedTaps).toEqual({ bathrooms: 1, kitchens: 0 });
    expect(draft.additionalHotWaterSystems).toHaveLength(1);
    const added = draft.additionalHotWaterSystems[0];
    expect(added.need).toEqual({ method: 'residential', dwellingCount: 1, sourceReference: 'survey' });
    expect(added.connectedTaps).toEqual({ bathrooms: 0, kitchens: 1 });
    expect(added.emission).toMatchObject({ method: 'residential', served: 'kitchen_only' });
    expect(added.generator).toEqual({ kind: 'electric_instantaneous' });
    await user.click(screen.getByRole('button', { name: 'Remove' }));
    expect(current().additionalHotWaterSystems).toEqual([]);
  });

  it('gives utility systems their own served area (13.20)', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness residential={false} initial={{
      hotWater: { need: { method: 'utility', areas: [{ function: 'office', areaM2: 800 }], sourceReference: '' } },
    }} />);
    await user.click(screen.getByRole('button', { name: 'Add hot-water system' }));
    await user.type(screen.getByLabelText('Usable area served, m² (13.20)'), '200');
    const added = current().additionalHotWaterSystems[0];
    expect(added.need.areas[0].areaM2).toBe(200);
    expect(added.connectedTaps).toBeNull();
    expect(current().hotWater.connectedTaps).toBeUndefined();
  });
});
