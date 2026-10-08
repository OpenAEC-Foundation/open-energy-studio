/**
 * Annex AA rooms and windows as a form (replacing the raw JSON): the client
 * check mirrors the kernel's codes and paths, the room helpers, and a round
 * trip from the form to the kernel's `AnnexAaInput` JSON.
 */
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { createDefaultProject } from '../context/EnergyContext';
import type { IProject } from '../core/energy/types';
import {
  annexAaCalculations, annexAaEditionRules, annexAaTotals, checkAnnexAa, duplicateAnnexAaRoom, newAnnexAaCalculation,
  newAnnexAaRoom, uniqueRoomId,
} from '../core/nta/annexAaForm';
import { NtaCalculationForm } from '../components/NtaPerformancePanel/NtaCalculationForm';
import { renderWithProviders, userEvent } from './test-utils';

const codes = (issues: ReturnType<typeof checkAnnexAa>) => issues.map((issue) => `${issue.code}@${issue.path.join('.')}`);
const P = 'activeCooling.capacity.calculation';

/** A project with two outdoor windows, w-south and w-west. */
function projectWithWindows(): IProject {
  const project = createDefaultProject();
  const zone = project.zones[0];
  zone.surfaces = [{
    id: 'gevel-z', name: 'Gevel zuid', type: 'wall', thermalBoundary: 'outdoor', area: 20, orientation: 'S',
    windows: [
      { id: 'w-south', name: 'Pui zuid', area: 4.2, uValue: 1.4, gValue: 0.6, orientation: 'S', surfaceId: 'gevel-z' },
      { id: 'w-west', name: 'Raam west', area: 1.5, uValue: 1.4, gValue: 0.6, orientation: 'W', surfaceId: 'gevel-z' },
    ],
  } as unknown as IProject['zones'][number]['surfaces'][number]];
  return project;
}

function block(calculation?: unknown, normVersion = '2025+C1') {
  return {
    normVersion,
    activeCooling: {
      system: 'compression_table10_29', sourceReference: '',
      capacity: { method: 'annex_aa', sourceReference: '', ...(calculation === undefined ? {} : { calculation }) },
    },
  };
}

describe('annex AA client check', () => {
  const valid = {
    constructionYear: 2010, postInsulated: false,
    rooms: [{ id: 'woonkamer', areaM2: 30, living: true, opaqueInnerAreaM2: 18, installedCapacityKw: 2.5,
      windows: [{ windowId: 'w-south' }, { windowId: 'window:w-west', uWithShutterWPerM2k: 1.1 }] }],
  };

  it('accepts a complete calculation, including the window:<id> spelling', () => {
    expect(checkAnnexAa(valid, { edition2024: false, windowIds: ['w-south', 'w-west'] })).toEqual([]);
  });

  it('reports the kernel codes at the kernel paths', () => {
    const broken = {
      constructionYear: null, generatorCapacityKw: -1, effectiveMassKgPerM2: 70,
      rooms: [
        { id: 'a', areaM2: 0, living: false, opaqueInnerAreaM2: null, installedCapacityKw: null, roofAreaM2: 5,
          windows: [{ windowId: 'w-south', uWithShutterWPerM2k: 0 }, { windowId: 'nope' }] },
        { id: 'a', areaM2: 12, living: false, opaqueInnerAreaM2: 4, installedCapacityKw: 1, windows: [{ windowId: 'w-south' }] },
      ],
    };
    expect(codes(checkAnnexAa(broken, { edition2024: false, windowIds: ['w-south'] }))).toEqual([
      `annex_aa_construction_year_required@${P}.constructionYear`,
      `route_not_in_edition@${P}.effectiveMassKgPerM2`,
      `annex_aa_capacity_invalid@${P}.generatorCapacityKw`,
      `annex_aa_area_invalid@${P}.rooms.0.areaM2`,
      `annex_aa_area_invalid@${P}.rooms.0.opaqueInnerAreaM2`,
      `annex_aa_capacity_invalid@${P}.rooms.0.installedCapacityKw`,
      `route_not_in_edition@${P}.rooms.0.roofAreaM2`,
      `annex_aa_u_invalid@${P}.rooms.0.windows.0.uWithShutterWPerM2k`,
      `annex_aa_window_unknown@${P}.rooms.0.windows.1.windowId`,
      `annex_aa_room_id_duplicate@${P}.rooms.1.id`,
      `annex_aa_window_assigned_twice@${P}.rooms.1.windows.0.windowId`,
    ]);
  });

  it('requires SWM 50–100 kg/m² under 2024 and at least one room', () => {
    expect(codes(checkAnnexAa({ constructionYear: 2000, rooms: [] }, { edition2024: true }))).toEqual([
      `annex_aa_room_required@${P}.rooms`, `annex_aa_effective_mass_required@${P}.effectiveMassKgPerM2`,
    ]);
    expect(codes(checkAnnexAa({ ...valid, effectiveMassKgPerM2: 165 }, { edition2024: true }))).toEqual([
      `annex_aa_effective_mass_invalid@${P}.effectiveMassKgPerM2`,
    ]);
    expect(codes(checkAnnexAa(null, { edition2024: false }))).toEqual([`annex_aa_calculation_required@${P}`]);
  });
});

describe('annex AA room helpers', () => {
  it('gives new and duplicated rooms a unique id and does not copy windows', () => {
    expect(uniqueRoomId([{ id: 'vertrek' }, { id: 'vertrek-2' }], 'vertrek')).toBe('vertrek-3');
    const first = newAnnexAaCalculation(1998);
    expect(first).toMatchObject({ constructionYear: 1998, rooms: [{ id: 'woonkamer', living: true }] });
    expect(newAnnexAaCalculation(undefined).constructionYear).toBeNull();
    expect(newAnnexAaRoom(first.rooms)).toMatchObject({ id: 'vertrek', living: false, windows: [] });
    const rooms = [{ id: 'slaapkamer', areaM2: 12, windows: [{ windowId: 'w-west' }] }, { id: 'badkamer', windows: [] }];
    const next = duplicateAnnexAaRoom(rooms, 0);
    expect(next.map((room) => room.id)).toEqual(['slaapkamer', 'slaapkamer-2', 'badkamer']);
    expect(next[1]).toMatchObject({ areaM2: 12, windows: [] });
    expect(rooms[0].windows).toHaveLength(1);
    expect(annexAaTotals({ rooms: [{ areaM2: 12, installedCapacityKw: 1 }, { areaM2: null }] }))
      .toEqual({ areaM2: 12, capacityKw: 1, rooms: 2 });
  });
});

describe('annex AA form', () => {
  it('builds the kernel JSON from the form: rooms, windows, duplicate and remove', async () => {
    const user = userEvent.setup();
    let saved: Record<string, unknown> | null = null;
    renderWithProviders(<NtaCalculationForm project={projectWithWindows()} initial={block()}
      onSave={(next) => { saved = next; }} onCancel={() => undefined} />);

    await user.click(screen.getByRole('button', { name: 'Add annex AA calculation' }));
    const form = within(screen.getByTestId('nta-annex-aa'));
    await user.clear(form.getByLabelText(/Construction year/));
    await user.type(form.getByLabelText(/Construction year/), '2008');
    await user.click(form.getByLabelText('More than 50 % of A_in demonstrably post-insulated'));

    const living = within(form.getByRole('group', { name: 'Room woonkamer' }));
    await user.type(living.getByLabelText(/Floor area/), '32,5');
    await user.type(living.getByLabelText(/Opaque outer area/), '14');
    await user.type(living.getByLabelText(/Installed cooling capacity/), '3');
    await user.click(living.getByRole('button', { name: 'Add window' }));
    await user.type(living.getByLabelText(/U_w\+shut/), '1,2');

    // A second room: duplicate the first (windows are not copied), rename it, give it the west window.
    await user.click(living.getByRole('button', { name: 'Duplicate' }));
    const copy = within(form.getByRole('group', { name: 'Room woonkamer-2' }));
    expect(copy.queryByLabelText('Window')).toBeNull();
    await user.clear(copy.getByLabelText('Name / id'));
    await user.type(copy.getByLabelText('Name / id'), 'slaapkamer');
    const bedroom = within(form.getByRole('group', { name: 'Room slaapkamer' }));
    await user.click(bedroom.getByLabelText('Living room, kitchen or dining room (double internal load)'));
    await user.click(bedroom.getByRole('button', { name: 'Add window' }));
    expect(bedroom.getByLabelText('Window')).toHaveValue('w-west');
    // Both windows are assigned now.
    expect(bedroom.getByText('Every project window is already assigned to a room.')).toBeInTheDocument();

    // A third room, removed again.
    await user.click(form.getByRole('button', { name: 'Add room' }));
    await user.click(within(form.getByRole('group', { name: 'Room vertrek' })).getByRole('button', { name: 'Remove room' }));

    await user.click(screen.getByRole('button', { name: 'Save' }));
    const calculation = (saved as unknown as ReturnType<typeof block> & {
      activeCooling: { capacity: { calculation: unknown } };
    }).activeCooling.capacity.calculation;
    expect(calculation).toEqual({
      constructionYear: 2008,
      postInsulated: true,
      rooms: [
        { id: 'woonkamer', areaM2: 32.5, living: true, opaqueInnerAreaM2: 14, installedCapacityKw: 3,
          windows: [{ windowId: 'w-south', uWithShutterWPerM2k: 1.2 }] },
        { id: 'slaapkamer', areaM2: 32.5, living: false, opaqueInnerAreaM2: 14, installedCapacityKw: 3,
          windows: [{ windowId: 'w-west' }] },
      ],
    });
    expect(checkAnnexAa(calculation, { edition2024: false, windowIds: ['w-south', 'w-west'] })).toEqual([]);
  }, 60000);

  it('shows the kernel messages at the fields and keeps unknown windows visible', async () => {
    renderWithProviders(<NtaCalculationForm project={projectWithWindows()} initial={block({
      constructionYear: 2010,
      rooms: [{ id: 'woonkamer', areaM2: 0, living: true, opaqueInnerAreaM2: 10, installedCapacityKw: 1,
        windows: [{ windowId: 'oud-raam' }] }],
    })} onSave={() => undefined} onCancel={() => undefined} />);
    const form = within(screen.getByTestId('nta-annex-aa'));
    const alerts = form.getAllByRole('alert').map((alert) => alert.getAttribute('data-code'));
    expect(alerts).toEqual(['annex_aa_area_invalid', 'annex_aa_window_unknown']);
    expect(form.getByLabelText('Window')).toHaveDisplayValue('oud-raam (not in the project)');
  });

  it('applies edited JSON and refuses invalid JSON', async () => {
    const user = userEvent.setup();
    let saved: Record<string, unknown> | null = null;
    renderWithProviders(<NtaCalculationForm project={projectWithWindows()} initial={block(newAnnexAaCalculation(2000))}
      onSave={(next) => { saved = next; }} onCancel={() => undefined} />);
    const form = within(screen.getByTestId('nta-annex-aa'));
    await user.click(form.getByRole('button', { name: 'View JSON' }));
    const area = form.getByLabelText('Annex AA as JSON (advanced)');
    await user.clear(area);
    await user.click(area);
    await user.paste('{ "constructionYear": 1960');
    await user.click(form.getByRole('button', { name: 'Apply JSON' }));
    expect(form.getByText('Not valid JSON; the calculation is unchanged.')).toBeInTheDocument();
    await user.clear(area);
    await user.paste('{ "constructionYear": 1960, "rooms": [] }');
    await user.click(form.getByRole('button', { name: 'Apply JSON' }));
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect((saved as unknown as { activeCooling: { capacity: { calculation: unknown } } }).activeCooling.capacity.calculation)
      .toEqual({ constructionYear: 1960, rooms: [] });
  }, 60000);

  it('notes that editions before 2024 have no annex AA route', () => {
    renderWithProviders(<NtaCalculationForm project={projectWithWindows()} initial={block(undefined, '2023')}
      onSave={() => undefined} onCancel={() => undefined} />);
    expect(within(screen.getByTestId('nta-annex-aa')).getByText(/route_not_in_edition/)).toBeInTheDocument();
  });
});

/** Two zones, each with one outdoor window. */
function projectWithTwoZones(): IProject {
  const project = projectWithWindows();
  const first = project.zones[0];
  first.surfaces[0].windows = [first.surfaces[0].windows[0]];
  const second = structuredClone(first);
  second.id = 'zone-2';
  second.name = 'Zolder';
  second.surfaces = [{
    ...structuredClone(first.surfaces[0]), id: 'gevel-n', name: 'Gevel noord', orientation: 'N',
    windows: [{ id: 'w-north', name: 'Raam noord', area: 2, uValue: 1.4, gValue: 0.6, orientation: 'N', surfaceId: 'gevel-n' }],
  } as unknown as IProject['zones'][number]['surfaces'][number]];
  project.zones = [first, second];
  return project;
}

describe('annex AA per rekenzone', () => {
  it('follows the kernel edition profile in one helper', () => {
    expect(annexAaEditionRules('2025+C1')).toEqual({ route: true, monthly2024: false });
    expect(annexAaEditionRules(undefined)).toEqual({ route: true, monthly2024: false });
    expect(annexAaEditionRules('2024')).toEqual({ route: true, monthly2024: true });
    // 2023, 2022 and 2020+A1 build on the 2024 method but have no annex AA route.
    for (const edition of ['2023', '2022', '2020+A1']) expect(annexAaEditionRules(edition)).toEqual({ route: false, monthly2024: true });
  });

  it('lists the single and the per-zone calculations with their paths', () => {
    const found = annexAaCalculations({
      calculation: { constructionYear: 2000, rooms: [] },
      zoneCalculations: [{ zoneId: 'zone-2', constructionYear: 2000, rooms: [] }, null],
    });
    expect(found.map((item) => [item.path.join('.'), item.zoneId])).toEqual([
      ['activeCooling.capacity.calculation', null],
      ['activeCooling.capacity.zoneCalculations.0', 'zone-2'],
    ]);
    expect(codes(checkAnnexAa({ constructionYear: null, rooms: [] }, {
      edition2024: false, base: ['activeCooling', 'capacity', 'zoneCalculations', 1],
    }))).toEqual([
      'annex_aa_construction_year_required@activeCooling.capacity.zoneCalculations.1.constructionYear',
      'annex_aa_room_required@activeCooling.capacity.zoneCalculations.1.rooms',
    ]);
  });

  it('gives each zone its own calculation and only its own windows', async () => {
    const user = userEvent.setup();
    const project = projectWithTwoZones();
    let saved: Record<string, unknown> | null = null;
    renderWithProviders(<NtaCalculationForm project={project} initial={block(undefined)}
      onSave={(next) => { saved = next; }} onCancel={() => undefined} />);
    const zones = within(screen.getByTestId('nta-annex-aa-zones'));
    const second = within(zones.getByTestId('nta-annex-aa-zone-zone-2'));
    await user.click(second.getByRole('button', { name: 'Add annex AA calculation for Zolder' }));
    await user.click(second.getByRole('button', { name: 'Add window' }));
    // Only the north window of zone 2 is offered.
    const options = second.getAllByRole('option').map((option) => option.textContent);
    expect(options).toHaveLength(1);
    expect(options[0]).toMatch(/Raam noord/);
    await user.click(screen.getByRole('button', { name: 'Save' }));
    const capacity = (saved as unknown as { activeCooling: { capacity: Record<string, unknown> } }).activeCooling.capacity;
    expect(capacity.calculation).toBeUndefined();
    expect(capacity.zoneCalculations).toEqual([
      expect.objectContaining({ zoneId: 'zone-2', rooms: [expect.objectContaining({ windows: [{ windowId: 'w-north' }] })] }),
    ]);
  }, 60000);

  it('moves a calculation without zone to the chosen zone', async () => {
    const user = userEvent.setup();
    let saved: Record<string, unknown> | null = null;
    const single = { constructionYear: 2010, rooms: [] };
    renderWithProviders(<NtaCalculationForm project={projectWithTwoZones()} initial={block(single)}
      onSave={(next) => { saved = next; }} onCancel={() => undefined} />);
    const zones = within(screen.getByTestId('nta-annex-aa-zones'));
    expect(zones.getByText(/without a calculation zone/)).toBeInTheDocument();
    await user.click(zones.getByRole('button', { name: 'Assign to Zolder' }));
    await user.click(screen.getByRole('button', { name: 'Save' }));
    const capacity = (saved as unknown as { activeCooling: { capacity: Record<string, unknown> } }).activeCooling.capacity;
    expect(capacity.calculation).toBeUndefined();
    expect(capacity.zoneCalculations).toEqual([{ zoneId: 'zone-2', constructionYear: 2010, rooms: [] }]);
  }, 60000);
});
