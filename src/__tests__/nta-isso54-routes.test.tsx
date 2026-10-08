import { useState, type ReactElement } from 'react';
import { describe, expect, it } from 'vitest';
import { screen, within } from '@testing-library/react';
import type { IProject } from '../core/energy/types';
import {
  WindowGlazingsFields, withFixedLouvres, withGlazingType, withWindowGlazing,
} from '../components/NtaPerformancePanel/NtaWindowGlazings';
import {
  CirculationPsiFields, DeclaredAuxiliaryConstantsFields, DeclaredChpEfficienciesFields, PipeGeometryFields,
  pipeGeometryTemplate,
} from '../components/NtaPerformancePanel/NtaPipeGeometry';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { hasExplicitRoute, routeForPath } from '../core/nta/gapRoutes';
import { renderWithProviders, userEvent } from './test-utils';

const project = {
  id: 'p', name: 'p', description: '', buildingFunction: 'residential', address: '', city: '',
  zones: [{ id: 'z1', surfaces: [
    { id: 'ag', name: 'Achtergevel', type: 'wall', thermalBoundary: 'outdoor', area: 30, orientation: 'S', constructionId: 'c', zoneId: 'z1',
      windows: [{ id: 'win-S', name: 'Pui', area: 7.6, uValue: 1.1, gValue: 0.6, orientation: 'S', surfaceId: 'ag' }] },
  ] }],
  heatingSystems: [], ventilationSystems: [], coolingSystems: [], hotWaterSystems: [], solarPV: [], solarThermal: [], constructions: [],
} as unknown as IProject;

function Harness({ initial = {}, children }: { initial?: Draft; children: (draft: Draft, change: (path: Path, value: unknown) => void) => ReactElement }) {
  const [draft, setDraft] = useState<Draft>(initial);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  return <form aria-label="test">
    {children(draft, change)}
    <output data-testid="draft">{JSON.stringify(draft)}</output>
  </form>;
}

const current = () => JSON.parse(screen.getByTestId('draft').textContent ?? '{}');

describe('Glazing per window (7.41, table 7.4, 7.41a)', () => {
  it('builds the windowGlazings entries', () => {
    const diffusing = withWindowGlazing([], 'win-S', 'diffusing');
    expect(diffusing).toEqual([{ windowId: 'win-S', glazing: { diffusing: { gAltitude45: null, gDiffuse: null, sourceReference: '' } }, sourceReference: '' }]);
    expect(withWindowGlazing(diffusing, 'win-S', '')).toEqual([]);
    expect(withGlazingType({}, 'double_low_e')).toEqual({ glazingType: 'double_low_e' });
    expect(withGlazingType({ glazingType: 'double' }, '')).toEqual({});
    expect(withFixedLouvres({ glazingType: 'double' }, 'horizontal90')).toEqual({ glazingType: 'double', fixedLouvres: { kind: 'horizontal90' } });
    expect(withFixedLouvres({ fixedLouvres: { kind: 'horizontal90' } }, '')).toEqual({});
    expect(hasExplicitRoute('ntaCalculation.windowGlazings[0].glazing.diffusing')).toBe(true);
    expect(routeForPath('ntaCalculation.windowGlazings[0].glazing.diffusing')).toMatchObject({ step: 'building', sub: 'envelope' });
  });

  it('enters the ISSO 54 EP-W011a louvre values for one window', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness>{(draft, change) => <WindowGlazingsFields draft={draft} change={change} project={project} />}</Harness>);
    await user.selectOptions(screen.getByLabelText('Glazing Pui'), 'diffusing');
    const group = screen.getByRole('group', { name: 'Pui' });
    await user.type(within(group).getByLabelText('g at 45° solar altitude (g_gl,alt)'), '0.045');
    await user.type(within(group).getByLabelText('g for diffuse radiation (g_gl,dif)'), '0.2');
    expect(current().windowGlazings[0].glazing.diffusing).toMatchObject({ gAltitude45: 0.045, gDiffuse: 0.2 });
  });
});

describe('Calculated pipe Ψ, declared 9.85 constants and CHP factors', () => {
  it('writes a complete geometry when a method is chosen', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ pipe: { kind: 'calculated' } }}>
      {(draft, change) => <PipeGeometryFields draft={draft} change={change} base={['pipe', 'geometry']} />}
    </Harness>);
    expect(screen.queryByLabelText('Depth in the construction (m)')).toBeNull();
    await user.selectOptions(screen.getByLabelText('Pipe'), 'insulated_embedded');
    expect(current().pipe.geometry).toEqual(pipeGeometryTemplate('insulated_embedded'));
    await user.type(screen.getByLabelText('Depth in the construction (m)'), '0.03');
    expect(current().pipe.geometry.depthM).toBe(0.03);
  });

  it('offers the circulation Ψ only with a circulation', async () => {
    const user = userEvent.setup();
    const { unmount } = renderWithProviders(<Harness>{(draft, change) => <CirculationPsiFields draft={draft} change={change} />}</Harness>);
    expect(screen.queryByLabelText(/circulation pipe ψ/)).toBeNull();
    unmount();
    renderWithProviders(<Harness initial={{ hotWater: { circulation: { floorCount: 1 } } }}>
      {(draft, change) => <CirculationPsiFields draft={draft} change={change} />}
    </Harness>);
    await user.click(screen.getByLabelText(/circulation pipe ψ/));
    expect(current().hotWater.circulation.calculatedPsi.method).toBe('insulated_embedded');
  });

  it('enters the EP-W203j boiler declaration and the EP-W204h CHP declaration', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness>{(draft, change) => <>
      <DeclaredAuxiliaryConstantsFields draft={draft} change={change} base={['generator', 'declaredAuxiliaryConstants']} />
      <DeclaredChpEfficienciesFields draft={draft} change={change} base={['chp', 'declaredEfficiencies']} />
    </>}</Harness>);
    await user.click(screen.getByLabelText(/Auxiliary energy from a quality declaration/));
    await user.type(screen.getByLabelText('A (kWh)'), '10');
    await user.type(screen.getByLabelText('C'), '0.3');
    expect(current().generator.declaredAuxiliaryConstants).toMatchObject({ aKwh: 10, c: 0.3 });
    await user.click(screen.getByLabelText(/Conversion factors from a quality declaration/));
    await user.type(screen.getByLabelText('ε_chp;th'), '0.8');
    expect(current().chp.declaredEfficiencies).toMatchObject({ thermal: 0.8 });
  });
});
