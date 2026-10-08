import { useEffect } from 'react';
import { beforeEach, describe, expect, it } from 'vitest';
import { act, fireEvent, screen, waitFor } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import type { IProject } from '../core/energy/types';
import type { NtaEvidenceItem } from '../core/nta/KernelClient';
import { clearSessionEvidence } from '../core/nta/Evidence';
import { normalizeProject } from '../core/energy/normalizeProject';
import {
  danglingEvidenceReferences, evidenceUsage, freshItemId, linksAfterRemoval, migrateEvidenceLinks, pointerToPath,
  resolvePointer, stablePointer, unresolvedEvidenceLinks, withSurveyItemIds,
} from '../core/nta/EvidenceLinks';
import { routeForPath } from '../core/nta/gapRoutes';
import { EvidenceAttach } from '../components/EvidenceLink/EvidenceLink';
import { renderWithProviders } from './test-utils';

const item = (id: string, linkedPaths?: string[]): NtaEvidenceItem => ({
  id, kind: 'photo_detail', fileName: `${id}.jpg`, sha256: 'b'.repeat(64), ...(linkedPaths ? { linkedPaths } : {}),
});

const zone = (id: string, surfaces: string[]) => ({
  id, name: id, surfaces: surfaces.map((surface) => ({ id: surface, name: surface, windows: [{ id: `${surface}-raam` }] })),
  thermalBridges: [], airTightness: { qv10: 0.4 },
});

function project(evidence: NtaEvidenceItem[], extra: Record<string, unknown> = {}): IProject {
  return {
    id: 'p1', name: 'Woning', description: '', buildingFunction: 'residential', address: '', city: '',
    zones: [zone('woonzone', ['gevel-noord', 'gevel-zuid']), zone('berging', ['dak'])],
    heatingSystems: [], ventilationSystems: [], coolingSystems: [], hotWaterSystems: [], solarPV: [], solarThermal: [],
    constructions: [],
    registration: { evidence },
    ...extra,
  } as unknown as IProject;
}

const survey = (generators: number, pv: string[]) => ({
  kind: 'residential' as const,
  survey: {
    heating: { additionalGenerators: Array.from({ length: generators }, (_, index) => ({ generator: { kind: 'boiler' }, nominalPowerKw: 10 + index })) },
    pv: pv.map((id) => ({ id })),
  },
});

describe('id-based evidence links', () => {
  beforeEach(() => clearSessionEvidence());

  it('names elements by id, resolves back to positions and is idempotent', () => {
    const p = project([]);
    const stable = stablePointer(p, '/zones/1/surfaces/0/windows/0');
    expect(stable).toBe('/zones/@berging/surfaces/@dak/windows/@dak-raam');
    expect(stablePointer(p, stable)).toBe(stable);
    expect(resolvePointer(p, stable)).toBe('/zones/1/surfaces/0/windows/0');
    // Members that are not array elements stay as they are; an unknown position does not resolve.
    expect(stablePointer(p, '/zones/0/airTightness')).toBe('/zones/@woonzone/airTightness');
    expect(resolvePointer(p, '/zones/5')).toBeNull();
    expect(stablePointer(p, '/zones/5')).toBe('/zones/5');
  });

  it('keeps an element without a unique id by position', () => {
    const p = project([], { basisopname: survey(2, ['pv-1', 'pv-1']) });
    expect(stablePointer(p, '/basisopname/survey/heating/additionalGenerators/1')).toBe('/basisopname/survey/heating/additionalGenerators/1');
    // Two PV items with the same id: neither can be named by it.
    expect(stablePointer(p, '/basisopname/survey/pv/1')).toBe('/basisopname/survey/pv/1');
    expect(resolvePointer(p, '/basisopname/survey/pv/@pv-1')).toBeNull();
  });

  it('keeps a link on its element when other elements are removed or reordered', () => {
    const p = project([item('ev-1', ['/zones/@berging/surfaces/@dak']), item('ev-2', ['/zones/@woonzone/surfaces/@gevel-zuid'])]);
    const removed = { ...p, zones: p.zones.slice(1) } as IProject;
    expect(resolvePointer(removed, '/zones/@berging/surfaces/@dak')).toBe('/zones/0/surfaces/0');
    const reordered = { ...p, zones: [p.zones[1], { ...p.zones[0], surfaces: [...p.zones[0].surfaces].reverse() }] } as IProject;
    expect(resolvePointer(reordered, '/zones/@woonzone/surfaces/@gevel-zuid')).toBe('/zones/1/surfaces/0');
    expect(evidenceUsage(reordered).get('ev-2')).toEqual(['/zones/@woonzone/surfaces/@gevel-zuid']);
    // Shown by id, navigated by position.
    expect(pointerToPath('/zones/@woonzone/surfaces/@gevel-zuid')).toBe('zones[woonzone].surfaces[gevel-zuid]');
    expect(pointerToPath('/zones/@woonzone/surfaces/@gevel-zuid', reordered)).toBe('zones[1].surfaces[0]');
    expect(routeForPath(pointerToPath('/zones/@woonzone/surfaces/@gevel-zuid', reordered))).toMatchObject({ step: 'building', sub: 'envelope' });
  });

  it('reports links to removed elements and still reports references to removed files', () => {
    const p = project([item('ev-1', ['/zones/@berging/surfaces/@dak', '/zones/@woonzone'])], {
      ntaCalculation: { bacsSourceReference: 'evidence:ev-9' },
    });
    expect(unresolvedEvidenceLinks(p)).toEqual([]);
    const removed = { ...p, zones: p.zones.slice(0, 1) } as IProject;
    expect(unresolvedEvidenceLinks(removed)).toEqual([{ pointer: '/zones/@berging/surfaces/@dak', id: 'ev-1' }]);
    expect(danglingEvidenceReferences(removed)).toEqual([{ pointer: '/ntaCalculation/bacsSourceReference', id: 'ev-9' }]);
  });

  it('migrates the index-based links of an older project on load, once', () => {
    const old = project([
      item('ev-1', ['/zones/1/surfaces/0', '/zones/@berging/surfaces/@dak']),
      item('ev-2', ['/basisopname/survey/pv/1', '/basisopname/survey/heating/additionalGenerators/0', '/zones/9']),
      item('ev-3'),
    ], { basisopname: survey(1, ['pv-1', 'pv-2']) });
    const opened = normalizeProject(old);
    const evidence = opened.registration!.evidence!;
    // Duplicates that name the same element collapse into one link.
    expect(evidence[0].linkedPaths).toEqual(['/zones/@berging/surfaces/@dak']);
    // The survey's further generator gets an id on load, and its link follows it.
    expect((opened.basisopname!.survey.heating as { additionalGenerators: Array<{ id?: string }> }).additionalGenerators[0].id).toBe('opwekker-1');
    expect(evidence[1].linkedPaths).toEqual([
      '/basisopname/survey/pv/@pv-2', '/basisopname/survey/heating/additionalGenerators/@opwekker-1', '/zones/9',
    ]);
    expect(evidence[2]).toBe(old.registration!.evidence![2]);
    expect(normalizeProject(opened)).toBe(opened);
    expect(migrateEvidenceLinks(opened)).toBe(opened);
  });

  it('gives the survey lists without ids an id on load and keeps the ids already there', () => {
    const old = project([], {
      basisopname: {
        kind: 'utility',
        survey: {
          heating: { additionalGenerators: [{ id: 'opwekker-2', generator: { kind: 'boiler' } }, { generator: { kind: 'boiler' } }] },
          hotWater: { additionalGenerators: [{ generator: { kind: 'electric_instantaneous' } }] },
          additionalHotWaterSystems: [{ generator: { kind: 'electric_boiler' }, additionalGenerators: [{ generator: { kind: 'none' } }] }],
        },
      },
    });
    const opened = withSurveyItemIds(old);
    const s = opened.basisopname!.survey as Record<string, any>;
    // The fresh id skips the one the first generator already has.
    expect(s.heating.additionalGenerators.map((item: { id: string }) => item.id)).toEqual(['opwekker-2', 'opwekker-3']);
    expect(s.hotWater.additionalGenerators[0].id).toBe('tapwateropwekker-1');
    expect(s.additionalHotWaterSystems[0].id).toBe('tapwatersysteem-1');
    expect(s.additionalHotWaterSystems[0].additionalGenerators[0].id).toBe('tapwateropwekker-1');
    expect(withSurveyItemIds(opened)).toBe(opened);
    expect(withSurveyItemIds(project([]))).toEqual(project([]));
    expect(freshItemId([{ id: 'opwekker-2' }, {}], 'opwekker')).toBe('opwekker-3');
    expect(freshItemId([], 'opwekker')).toBe('opwekker-1');
  });

  it('moves the position links of a removed survey item up and drops the links to it', () => {
    const before = project([
      item('ev-1', ['/basisopname/survey/heating/additionalGenerators/0']),
      item('ev-2', ['/basisopname/survey/heating/additionalGenerators/1']),
      item('ev-3', ['/basisopname/survey/heating/additionalGenerators/2/generator', '/basisopname/survey/pv/@pv-2']),
    ], { basisopname: survey(3, ['pv-1', 'pv-2']) });
    const after = { ...before, basisopname: survey(2, ['pv-1', 'pv-2']) } as IProject;
    const list = '/basisopname/survey/heating/additionalGenerators';
    const moved = linksAfterRemoval(before.registration!.evidence!, before, after, list, 1);
    expect(moved[0]).toBe(before.registration!.evidence![0]);
    expect(moved[1]).not.toHaveProperty('linkedPaths');
    expect(moved[2].linkedPaths).toEqual([`${list}/1/generator`, '/basisopname/survey/pv/@pv-2']);

    // A PV item with an id: removing the one before it changes nothing in the link.
    const pvAfter = { ...before, basisopname: survey(3, ['pv-2']) } as IProject;
    const pvMoved = linksAfterRemoval(before.registration!.evidence!, before, pvAfter, '/basisopname/survey/pv', 0);
    expect(pvMoved[2].linkedPaths).toEqual(['/basisopname/survey/heating/additionalGenerators/2/generator', '/basisopname/survey/pv/@pv-2']);
  });
});

function Seed({ value }: { value: IProject }) {
  const { dispatch } = useEnergy();
  useEffect(() => { dispatch({ type: 'SET_PROJECT', payload: structuredClone(value) }); }, [dispatch, value]);
  return null;
}

function EvidenceProbe() {
  const { state } = useEnergy();
  return <pre data-testid="evidence">{JSON.stringify(state.project.registration?.evidence ?? [])}</pre>;
}

const evidenceIn = () => JSON.parse(screen.getByTestId('evidence').textContent ?? '[]') as NtaEvidenceItem[];

describe('id-based evidence link UI', () => {
  beforeEach(() => clearSessionEvidence());

  it('stores the link by id and shows a link stored by position', async () => {
    // ev-1 still has the old position form; it names the same window.
    const seeded = project([item('ev-1', ['/zones/0/surfaces/1/windows/0']), item('ev-2')]);
    const { container } = renderWithProviders(<>
      <Seed value={seeded} />
      <EvidenceAttach pointer="/zones/0/surfaces/1/windows/0" />
      <EvidenceProbe />
    </>);
    await waitFor(() => expect(screen.getAllByText('ev-1.jpg', { exact: false }).length).toBeGreaterThan(0));
    await act(async () => {
      fireEvent.change(container.querySelector('select') as HTMLSelectElement, { target: { value: 'ev-2' } });
    });
    await waitFor(() => expect(evidenceIn()[1].linkedPaths).toEqual(['/zones/@woonzone/surfaces/@gevel-zuid/windows/@gevel-zuid-raam']));
    // Unlinking ev-1 removes its position-form link too.
    fireEvent.click(screen.getByRole('button', { name: /ev-1\.jpg/ }));
    await waitFor(() => expect(evidenceIn()[0]).not.toHaveProperty('linkedPaths'));
  });
});
