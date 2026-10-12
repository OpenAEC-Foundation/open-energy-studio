import { join } from 'node:path';
import { useState, type ReactElement } from 'react';
import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { HotWaterGeneratorFields } from '../components/NtaPerformancePanel/NtaSystemSections';
import { hotWaterGeneratorTemplate } from '../core/nta/NtaSystemTemplates';
import { routeForPath } from '../core/nta/gapRoutes';
import { kernelCodeLabel } from '../i18n/format';
import { nl } from '../i18n/nl';
import { en } from '../i18n/en';
import { emittedCodes } from './kernelCodeExtraction';
import { renderWithProviders, userEvent } from './test-utils';

function Harness({ children }: { children: (draft: Draft, change: (path: Path, value: unknown) => void) => ReactElement }) {
  const [draft, setDraft] = useState<Draft>({ hotWater: { generator: { kind: 'electric_boiler' } } });
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  return <form aria-label="test">
    {children(draft, change)}
    <output data-testid="draft">{JSON.stringify(draft)}</output>
  </form>;
}

const generator = () => JSON.parse(screen.getByTestId('draft').textContent ?? '{}').hotWater.generator;

describe('forfait booster heat pump (13.162/13.163)', () => {
  it('starts from the kernel shape with an unknown supply temperature and external heat', () => {
    expect(hotWaterGeneratorTemplate('booster_heat_pump_forfait')).toEqual({
      kind: 'booster_heat_pump_forfait', designSupplyTemperatureC: null,
      heatSource: { kind: 'external_heat' }, sourceReference: '',
    });
  });

  it('enters the supply temperature and a collective generator as heat source', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness>{(draft, change) =>
      <HotWaterGeneratorFields draft={draft} change={change} base={['hotWater', 'generator']} />}</Harness>);
    await user.selectOptions(screen.getByLabelText('Hot-water generator'), 'booster_heat_pump_forfait');
    expect(generator()).toEqual(hotWaterGeneratorTemplate('booster_heat_pump_forfait'));

    await user.type(screen.getByLabelText(/Design supply temperature of the feeding heating system/), '35');
    await user.selectOptions(screen.getByLabelText('Source of the heat (figure W.1)'), 'collective_generator');
    await user.type(screen.getByLabelText('Generation efficiency of the collective generator (chapter 9)'), '0.9');
    await user.selectOptions(screen.getByLabelText('Energy carrier of the collective generator'), 'oil');
    await user.type(screen.getByLabelText('Source of the collective generator'), 'ketel');
    await user.type(screen.getByLabelText('Source (product sheet or survey of the booster heat pump)'), 'opname');
    expect(generator()).toEqual({
      kind: 'booster_heat_pump_forfait', designSupplyTemperatureC: 35,
      heatSource: { kind: 'collective_generator', generationEfficiency: 0.9, carrier: 'oil', sourceReference: 'ketel' },
      sourceReference: 'opname',
    });

    // Back to the building's heating system: the collective fields go.
    await user.selectOptions(screen.getByLabelText('Source of the heat (figure W.1)'), 'heating_system');
    expect(generator().heatSource).toEqual({ kind: 'heating_system' });
  });

  it('routes its gaps to the hot-water installation', () => {
    for (const path of ['ntaCalculation.hotWater.generator.designSupplyTemperatureC',
      'ntaCalculation.hotWater.generator.heatSource.generationEfficiency',
      'ntaCalculation.hotWater.additionalGenerators[0].generator.sourceReference']) {
      expect(routeForPath(path), path).toMatchObject({ step: 'installations', sub: 'hotWater' });
    }
  });
});

describe('gap texts', () => {
  const codes = emittedCodes(join(__dirname, '..', '..', 'crates'));
  const lookup = (table: Record<string, string>) => (key: string) => table[key] ?? key;

  it('every code the kernel can emit shows a human text where gaps are listed, in Dutch and English', () => {
    // Gap lists ask for `nta.gap.` first; codes labelled only under `kernel.issue.` (or another
    // kernel prefix) must still resolve through the fallback, never as the bare key.
    for (const [name, table] of [['nl', nl], ['en', en]] as const) {
      const raw = [...codes.keys()].filter((code) => !kernelCodeLabel(lookup(table), code, ['nta.gap.']).known);
      expect(raw, name).toEqual([]);
    }
  });

  it('the booster supply-temperature code reads as text', () => {
    const label = kernelCodeLabel(lookup(en), 'booster_supply_temperature_invalid', ['nta.gap.']);
    expect(label.known).toBe(true);
    expect(label.text).not.toContain('booster_supply_temperature_invalid');
  });
});
