import { useState } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, screen } from '@testing-library/react';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { NTA_SECTIONS } from '../components/NtaPerformancePanel/NtaSections';
import { calculateProjectPerformanceWithRust } from '../core/nta/KernelClient';
import { nullPaths } from '../core/nta/KernelInput';
import { declaredUseTemplate, hotWaterGeneratorTemplate } from '../core/nta/NtaSystemTemplates';
import type { IProject } from '../core/energy/types';
import office from '../../training-data/nta8800-example-office.json';
import terraced from '../../training-data/nta8800-example-terraced-dwelling.json';
import { renderWithProviders } from './test-utils';

// Bugs the project-form guard (nta-project-form-guard.test.tsx with
// crates/nta8800-core/tests/project_form_guard.rs) found, one test each.

function Section({ id, project, initial }: { id: string; project: IProject; initial?: Draft }) {
  const def = NTA_SECTIONS.find((section) => section.id === id)!;
  const [draft, setDraft] = useState<Draft>(initial ?? project.ntaCalculation as unknown as Draft);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  const update = (next: (current: Draft) => Draft) => setDraft((current) => next(current));
  const Component = def.Component;
  return <form aria-label="section">
    <Component draft={draft} change={change} update={update} project={project} />
    <output data-testid="draft">{JSON.stringify(draft)}</output>
  </form>;
}

const draft = () => JSON.parse(screen.getByTestId('draft').textContent ?? '{}') as Record<string, any>;
const optionsOf = (label: RegExp) => [...(screen.getByLabelText(label) as HTMLSelectElement).options].map((option) => option.value);
const officeProject = office as unknown as IProject;
const dwelling = terraced as unknown as IProject;

describe('project-form guard findings', () => {
  afterEach(() => { vi.unstubAllGlobals(); vi.unstubAllEnvs(); });

  it('sends the blanks of the NTA block as null to the project route', async () => {
    // Without the nulls a required member is absent and the kernel can only refuse the whole block
    // (nta_calculation_block_invalid); with them its blank search names each field (nta_value_missing).
    vi.stubEnv('DEV', true);
    const fetchMock = vi.fn(async () => new Response(JSON.stringify({ status: 'incomplete', gaps: [] })));
    vi.stubGlobal('fetch', fetchMock);
    const block = { ...dwelling.ntaCalculation!, generator: { kind: 'gas_boiler', boiler: { kind: null } } };
    await calculateProjectPerformanceWithRust({ ...dwelling, ntaCalculation: block as never });
    const [, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
    const body = JSON.parse(String(init.body)) as { project: Record<string, unknown> };
    expect(nullPaths(body.project.ntaCalculation)).toContain('generator.boiler.kind');
  });

  it('writes a blank hot-water generator kind as null, not as an empty string', () => {
    expect(hotWaterGeneratorTemplate('')).toEqual({ kind: null });
  });

  it('starts declared monthly values with twelve blank months', () => {
    expect(declaredUseTemplate(0).monthlyKwh).toEqual(Array(12).fill(null));
  });

  it('drops the dwelling type when the use function stops being residential', () => {
    renderWithProviders(<Section id="general" project={dwelling} />);
    expect(draft().dwellingType).toBeDefined();
    fireEvent.change(screen.getByLabelText(/Zone usage function/), { target: { value: 'office' } });
    expect(draft().usageFunction).toBe('office');
    expect(draft()).not.toHaveProperty('dwellingType');
  });

  it('keeps hydronic balancing consistent with the emission system (table 9.3 footnote a)', () => {
    const initial = { ...officeProject.ntaCalculation as unknown as Draft,
      emission: { system: 'radiators_or_convectors', balancing: 'static', control: 'main_room_thermostat', sourceReference: 'x' } };
    renderWithProviders(<Section id="emission" project={officeProject} initial={initial} />);
    fireEvent.change(screen.getByLabelText(/Emission system/), { target: { value: 'air_heating' } });
    expect(draft().emission.balancing).toBe('not_applicable');
    fireEvent.change(screen.getByLabelText(/Emission system/), { target: { value: 'floor_heating' } });
    expect(draft().emission.balancing).toBe('none_or_unknown');
  });

  it('offers only the balancing choices of the emission system', () => {
    const initial = { ...officeProject.ntaCalculation as unknown as Draft,
      emission: { system: 'local_heater', balancing: 'not_applicable', control: 'main_room_thermostat', sourceReference: 'x' } };
    renderWithProviders(<Section id="emissionControl" project={officeProject} initial={initial} />);
    expect(optionsOf(/Hydronic balancing/)).toEqual(['', 'not_applicable']);
  });

  it('offers no table 11.5 variant without an f_ctrl for a utility building', () => {
    renderWithProviders(<Section id="ventilationChapter11" project={officeProject} />);
    const variants = optionsOf(/System variant/);
    for (const variant of ['c4a', 'c5a', 'c5b', 'd5c']) expect(variants).not.toContain(variant);
    expect(variants).toContain('c4b');
  });

  it('offers no declared lighting use for a dwelling (§5.5)', () => {
    const initial = { ...dwelling.ntaCalculation as unknown as Draft, declaredUses: [declaredUseTemplate(0)] };
    renderWithProviders(<Section id="declaredFlows" project={dwelling} initial={initial} />);
    expect(optionsOf(/^Service/)).not.toContain('lighting');
  });

  it('clears the BACS classes when BACS present is unticked', () => {
    const initial = { ...officeProject.ntaCalculation as unknown as Draft, bacs: {
      buildingUse: 'utility', systemInventoryComplete: false, systems: [],
      bacs: { present: true, automaticControlsClass: 'b', energyManagementClass: 'c', sourceReference: '' } } };
    renderWithProviders(<Section id="bacs" project={officeProject} initial={initial} />);
    fireEvent.click(screen.getByLabelText(/BACS present/));
    expect(draft().bacs.bacs).toEqual({ present: false, sourceReference: '' });
    expect(screen.queryByLabelText(/Automatic controls class/)).toBeNull();
  });

  // Review 9: a cleared field with a kernel default drops its member (the default applies),
  // and a choice with a default has no blank option.
  it('drops a cleared shared-zones count instead of writing a blank', () => {
    renderWithProviders(<Section id="verticalPipes" project={officeProject} />);
    const input = screen.getByLabelText(/Zones or heated spaces along the pipe/);
    fireEvent.change(input, { target: { value: '2' } });
    expect(draft().verticalPipes[0].sharedZones).toBe(2);
    fireEvent.change(input, { target: { value: '' } });
    expect('sharedZones' in draft().verticalPipes[0]).toBe(false);
  });

  it('offers no blank PV mounting', () => {
    renderWithProviders(<Section id="pv" project={officeProject} />);
    const mountings = screen.queryAllByLabelText(/Mounting \(table 16.2\)/);
    expect(mountings.length).toBeGreaterThan(0);
    for (const select of mountings) {
      expect([...(select as HTMLSelectElement).options].map((option) => option.value)).not.toContain('');
    }
  });

  it('shows the building height, not the storeys, on the utility route', () => {
    renderWithProviders(<Section id="verticalPipes" project={officeProject} />);
    expect(screen.getByLabelText(/External building height H/)).toBeTruthy();
    expect(screen.queryByLabelText(/Storeys of the zone/)).toBeNull();
  });
});
