import { useState } from 'react';
import { describe, expect, it } from 'vitest';
import { fireEvent, screen, within } from '@testing-library/react';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { NTA_SECTIONS } from '../components/NtaPerformancePanel/NtaSections';
import { buildingOnlyWarnings } from '../core/nta/buildingWarnings';
import { routeForPath } from '../core/nta/gapRoutes';
import { kernelIssues } from '../core/nta/stepStatus';
import type { ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import type { IProject } from '../core/energy/types';
import terraced from '../../training-data/nta8800-example-terraced-dwelling.json';
import { renderWithProviders } from './test-utils';

// Review 53: inputs and warnings of reviews 40–52 reach the forms, the routes and the lists.

function Section({ id, project, initial }: { id: string; project: IProject; initial: Draft }) {
  const def = NTA_SECTIONS.find((section) => section.id === id)!;
  const [draft, setDraft] = useState<Draft>(initial);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  const update = (next: (current: Draft) => Draft) => setDraft((current) => next(current));
  const Component = def.Component;
  return <form aria-label="section">
    <Component draft={draft} change={change} update={update} project={project} />
    <output data-testid="draft">{JSON.stringify(draft)}</output>
  </form>;
}

const draft = () => JSON.parse(screen.getByTestId('draft').textContent ?? '{}') as Record<string, any>;
const dwelling = terraced as unknown as IProject;
const block = dwelling.ntaCalculation as unknown as Draft;

const room = (id: string) => ({ id, glazingGHeating: 0.6, glazingGCooling: 0.6, exteriorFrameFraction: 0.2,
  reductionFactor: 0.8, zoneConductanceWPerK: 10, distributionFactor: 0.5, surfaces: [], sourceReference: 'x' });

describe('review 53: building warnings in the workflow', () => {
  const assessment = {
    status: 'calculated_unverified', gaps: [],
    warnings: [
      { code: 'tojuli_sunroom_facade_left_out', path: 'ntaCalculation.sunrooms', detail: 'derivedInput.tojuli[0].sunrooms' },
      { code: 'generator_large_system_outside', path: 'ntaCalculation.generator.boiler.location',
        detail: 'derivedInput.spaceHeating.generator.boiler.location' },
    ],
    performance: { warnings: [
      { code: 'tojuli_sunroom_facade_left_out', path: 'tojuli[0].sunrooms' },
      { code: 'generator_large_system_outside', path: 'spaceHeating.generator.boiler.location' },
      { code: 'cooling_emission_loss_singular', path: 'cooling.months[3]' },
    ] },
  } as unknown as ProjectPerformanceAssessment;

  it('lists each lifted warning once and keeps the rest of the building warnings', () => {
    expect(buildingOnlyWarnings(assessment)).toEqual([{ code: 'cooling_emission_loss_singular', path: 'cooling.months[3]' }]);
    expect(buildingOnlyWarnings(null)).toEqual([]);
  });

  it('routes the lifted warnings to the step that holds their input', () => {
    const issues = kernelIssues(assessment);
    expect(issues.map((issue) => issue.code)).toEqual(['tojuli_sunroom_facade_left_out', 'generator_large_system_outside']);
    expect(routeForPath(issues[0].path)).toMatchObject({ step: 'building', sub: 'unheated' });
    expect(routeForPath(issues[1].path)).toMatchObject({ step: 'installations', sub: 'heating' });
    expect(routeForPath('ntaCalculation.constructionYear')).toMatchObject({ step: 'project' });
  });

  it('routes the gaps and warnings of reviews 40–52 to their field', () => {
    const cases: Array<[string, string, string]> = [
      ['ntaCalculation.thermalBridgeRoute', 'building', 'envelope'],
      ['ntaCalculation.groundFloors[0].edgeThermalBridges', 'building', 'envelope'],
      ['ntaCalculation.groundFloors[0].heatedBasement.wallDepths[1].depthM', 'building', 'envelope'],
      ['zones[0].thermalBridges[2].thermalBoundary', 'building', 'thermalBridges'],
      ['ntaCalculation.sunrooms[0]', 'building', 'unheated'],
      ['ntaCalculation.zoneData[1].sunrooms[0]', 'building', 'unheated'],
      ['ntaCalculation.humidifiers[0].humidification.humidifier.carrier', 'installations', 'humidification'],
    ];
    for (const [path, step, sub] of cases) expect(routeForPath(path), path).toMatchObject({ step, sub, focusPath: path });
  });
});

describe('review 53: shared sunroom', () => {
  const twoZones = {
    ...dwelling,
    zones: [{ ...dwelling.zones[0], id: 'z1', name: 'Woning' }, { ...dwelling.zones[0], id: 'z2', name: 'Praktijk' }],
    unheatedSpaces: [{ id: 'serre', name: 'Serre', reductionFactor: 0.8, factorSourceReference: 'x' }],
  } as unknown as IProject;
  const zoneData = [
    { zoneId: 'z1', sunrooms: [room('serre')] },
    { zoneId: 'z2', sunrooms: [room('serre')] },
  ];

  it('shows that two zones share one sunroom and picks the sunroom from the unheated spaces', () => {
    renderWithProviders(<Section id="zoneData" project={twoZones} initial={{ ...block, zoneData } as Draft} />);
    const notes = screen.getAllByTestId('sunroom-shared');
    expect(notes).toHaveLength(2);
    expect(notes[0].textContent).toContain('Praktijk');
    expect(notes[1].textContent).toContain('Woning');
    const select = screen.getAllByLabelText(/Unheated space of the sunroom/)[0] as HTMLSelectElement;
    expect([...select.options].map((option) => option.value)).toContain('serre');
  });

  it('gives a new sunroom an id no zone uses yet', () => {
    renderWithProviders(<Section id="zoneData" project={twoZones} initial={{ ...block, zoneData } as Draft} />);
    fireEvent.click(screen.getAllByRole('button', { name: 'Add sunroom' })[0]);
    expect(draft().zoneData[0].sunrooms.map((item: { id: string }) => item.id)).toEqual(['serre', 'serre-1']);
    fireEvent.click(screen.getAllByRole('button', { name: 'Add sunroom' })[1]);
    expect(draft().zoneData[1].sunrooms.map((item: { id: string }) => item.id)).toEqual(['serre', 'serre-2']);
  });
});

describe('review 53: basement depth per wall part (8.42/D.12)', () => {
  it('replaces the single depth by wall parts and back', () => {
    const floors = (block.groundFloors as Draft[]).map((floor, index) => index === 0
      ? { ...floor, heatedBasement: { depthM: 1.5, wallResistanceM2kPerW: 2.5 } } : floor);
    renderWithProviders(<Section id="groundFloors" project={dwelling} initial={{ ...block, groundFloors: floors } as Draft} />);
    fireEvent.click(screen.getByRole('button', { name: 'Enter the depth per wall part' }));
    // Review 54: the first part starts from the depth already entered.
    expect(draft().groundFloors[0].heatedBasement).toEqual({ wallResistanceM2kPerW: 2.5, wallDepths: [{ lengthM: null, depthM: 1.5 }] });
    expect(screen.queryByLabelText(/Depth of the floor below ground level z/)).toBeNull();
    const part = screen.getByRole('group', { name: 'Basement wall part 1' });
    fireEvent.change(within(part).getByLabelText(/Length ℓ_j/), { target: { value: '12' } });
    fireEvent.change(within(part).getByLabelText(/Depth z_j/), { target: { value: '1.2' } });
    expect(draft().groundFloors[0].heatedBasement.wallDepths).toEqual([{ lengthM: 12, depthM: 1.2 }]);
    fireEvent.click(within(part).getByRole('button', { name: /Remove/ }));
    // Review 54: the last part's depth becomes the single depth z again.
    expect(draft().groundFloors[0].heatedBasement).toEqual({ wallResistanceM2kPerW: 2.5, depthM: 1.2 });
    expect(screen.getByLabelText(/Depth of the floor below ground level z/)).toBeTruthy();
  });
});
