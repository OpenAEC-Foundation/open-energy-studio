import { useEffect } from 'react';
import { beforeEach, describe, expect, it } from 'vitest';
import { act, fireEvent, screen, waitFor } from '@testing-library/react';
import { strFromU8 } from 'fflate';
import { useEnergy } from '../context/EnergyContext';
import { useNtaDraft } from '../context/NtaDraftProvider';
import type { IProject } from '../core/energy/types';
import type { BuildingPerformanceInput, NtaEvidenceItem, OpnameAssessment } from '../core/nta/KernelClient';
import { clearSessionEvidence, rememberEvidenceBytes, sha256Hex } from '../core/nta/Evidence';
import {
  danglingEvidenceReferences, evidenceIdsIn, evidenceUsage, jsonPointer, linkEvidence, pointerToPath, unlinkEvidence, withEvidenceReference,
} from '../core/nta/EvidenceLinks';
import { canTakeOver, surveyTakeover } from '../core/nta/SurveyTakeover';
import { buildProjectDossier } from '../core/report/ProjectDossier';
import { EvidenceAttach, EvidenceReferencePicker } from '../components/EvidenceLink/EvidenceLink';
import { SurveyTakeoverAction } from '../components/BasisopnamePanel/SurveyTakeoverAction';
import { routeForPath } from '../core/nta/gapRoutes';
import { renderWithProviders } from './test-utils';

const item = (id: string, linkedPaths?: string[]): NtaEvidenceItem => ({
  id, kind: 'photo_detail', fileName: `${id}.jpg`, sha256: 'b'.repeat(64), ...(linkedPaths ? { linkedPaths } : {}),
});

function project(evidence: NtaEvidenceItem[], ntaCalculation?: Record<string, unknown>): IProject {
  return {
    id: 'p1', name: 'Woning', description: '', buildingFunction: 'residential', address: '', city: '',
    zones: [], heatingSystems: [], ventilationSystems: [], solarPV: [], solarThermal: [], constructions: [],
    registration: { evidence },
    ...(ntaCalculation ? { ntaCalculation } : {}),
  } as unknown as IProject;
}

const derived = {
  calculationScope: 'residential', totalUsableFloorAreaM2: 120, areaSourceReference: 'opname ISSO 82.1',
  bacsFactor: 1, bacsSourceReference: '', useInventoryComplete: true, declaredUses: [], productionInventoryComplete: true,
  onSiteProduction: [],
  hotWater: { kind: 'gas_appliance' },
  spaceHeating: {
    demand: {
      zoneId: 'z1', usableFloorAreaM2: 120, areaSourceReference: '', usageFunction: 'residential',
      setpoints: { heatingC: 20, coolingC: 24, sourceReference: 'NTA 8800 tabel 7.1' },
      thermalMass: { floor: 'heavy', wall: 'heavy', ceiling: 'closed_or_suspended', sourceReference: '' },
      internalGains: { method: 'residential', dwellingCount: 1, sourceReference: '' },
      ventilationFlows: [], transmission: { surfaces: [] },
    },
    emission: { system: 'radiators_or_convectors', balancing: 'none_or_unknown', control: 'main_room_thermostat', sourceReference: '' },
    distribution: { kind: 'unknown' },
    generator: { kind: 'boiler', efficiency: 0.9 },
  },
} as unknown as BuildingPerformanceInput;

const assessment = (input: BuildingPerformanceInput | null, status: OpnameAssessment['status'] = 'calculated_unverified') => ({
  status, scope: 'residential', source: 'ISSO 82.1', appliedDefaults: [], warnings: [], issues: [],
  derivedInput: input, performance: null, referenceVerified: false,
}) as OpnameAssessment;

describe('evidence links', () => {
  beforeEach(() => clearSessionEvidence());

  it('reads evidence ids from reference texts', () => {
    expect(evidenceIdsIn('evidence:ev-1')).toEqual(['ev-1']);
    expect(evidenceIdsIn('evidence:ev-1, evidence:ev-3; factuur')).toEqual(['ev-1', 'ev-3']);
    expect(evidenceIdsIn('factuur 2024')).toEqual([]);
    expect(evidenceIdsIn(undefined)).toEqual([]);
  });

  it('collects linked pointers and references per item, and finds dangling references', () => {
    const p = project([item('ev-1', ['/zones/0/surfaces/2']), item('ev-2')], {
      generator: { sourceReference: 'evidence:ev-1' },
      bacsSourceReference: 'evidence:ev-9',
      areaSourceReference: 'tekening',
    });
    const usage = evidenceUsage(p);
    expect(usage.get('ev-1')).toEqual(['/ntaCalculation/generator/sourceReference', '/zones/0/surfaces/2']);
    expect(usage.get('ev-2')).toBeUndefined();
    expect(danglingEvidenceReferences(p)).toEqual([{ pointer: '/ntaCalculation/bacsSourceReference', id: 'ev-9' }]);
  });

  it('adds an evidence reference to a source text without erasing its description', () => {
    expect(withEvidenceReference('', 'ev-1')).toBe('evidence:ev-1');
    expect(withEvidenceReference(undefined, 'ev-1')).toBe('evidence:ev-1');
    expect(withEvidenceReference('tekening A-101, gevel noord ', 'ev-2')).toBe('tekening A-101, gevel noord; evidence:ev-2');
    expect(withEvidenceReference('tekening A-101; evidence:ev-2', 'ev-2')).toBe('tekening A-101; evidence:ev-2');
    expect(evidenceIdsIn(withEvidenceReference('evidence:ev-1', 'ev-3'))).toEqual(['ev-1', 'ev-3']);
  });

  it('links and unlinks a pointer without duplicates', () => {
    let evidence = [item('ev-1')];
    evidence = linkEvidence(evidence, 'ev-1', '/zones/0');
    evidence = linkEvidence(evidence, 'ev-1', '/zones/0');
    expect(evidence[0].linkedPaths).toEqual(['/zones/0']);
    evidence = unlinkEvidence(evidence, 'ev-1', '/zones/0');
    expect(evidence[0]).not.toHaveProperty('linkedPaths');
  });

  it('escapes pointers and routes them to their page', () => {
    expect(jsonPointer(['a/b', 'c~d', 0])).toBe('/a~1b/c~0d/0');
    expect(pointerToPath('/zones/0/surfaces/2/windows/1')).toBe('zones[0].surfaces[2].windows[1]');
    expect(pointerToPath('/basisopname/survey/envelope/windows/0')).toBe('basisopname.envelope.windows[0]');
    expect(routeForPath(pointerToPath('/basisopname/survey/pv/0'))).toMatchObject({ step: 'survey', sub: 'zonnepanelen' });
    expect(routeForPath(pointerToPath('/zones/0/surfaces/1'))).toMatchObject({ step: 'building', sub: 'envelope' });
  });

  it('puts a table of contents with what each file supports in the dossier manifest', async () => {
    const bytes = new TextEncoder().encode('foto');
    const sha256 = await sha256Hex(bytes);
    rememberEvidenceBytes(sha256, bytes);
    const p = project([{ ...item('ev-1', ['/basisopname/survey/pv/0']), sha256 }, item('ev-2')]);
    const bundle = await buildProjectDossier({ project: p, assessment: null, generatedAt: '2026-10-07T00:00:00Z' } as never);
    const manifest = JSON.parse(strFromU8(bundle.files['manifest.json']));
    expect(manifest.evidence).toEqual([
      expect.objectContaining({ id: 'ev-1', sha256, archivePath: 'evidence/ev-1-ev-1.jpg', supports: ['/basisopname/survey/pv/0'] }),
      expect.objectContaining({ id: 'ev-2', archivePath: null, supports: [] }),
    ]);
    expect(bundle.files['evidence/ev-1-ev-1.jpg']).toBeDefined();
  });
});

describe('survey takeover', () => {
  it('takes the blocks with the same meaning and leaves the geometry', () => {
    const { next, keys, skipped } = surveyTakeover(derived, { normVersion: '2024', generator: { kind: 'heat_pump' } });
    expect(next.normVersion).toBe('2024');
    expect(next.generator).toEqual({ kind: 'boiler', efficiency: 0.9 });
    expect(next.setpoints).toEqual({ heatingC: 20, coolingC: 24, sourceReference: 'NTA 8800 tabel 7.1' });
    expect(next.hotWater).toEqual({ kind: 'gas_appliance' });
    expect(next).not.toHaveProperty('transmission');
    expect(next).not.toHaveProperty('totalUsableFloorAreaM2');
    expect(keys).toContain('emission');
    expect(skipped.map((part) => part.reason)).toEqual(['geometry', 'geometry']);
  });

  it('leaves the zone demand of a multi-zone survey to the user', () => {
    const multi = structuredClone(derived) as unknown as { spaceHeating: { additionalZones: unknown[] } };
    multi.spaceHeating.additionalZones = [{ demand: {} }];
    const { next, skipped } = surveyTakeover(multi as unknown as BuildingPerformanceInput, null);
    expect(next).not.toHaveProperty('setpoints');
    expect(next.generator).toEqual({ kind: 'boiler', efficiency: 0.9 });
    expect(skipped.map((part) => part.reason)).toContain('multi_zone');
  });

  it('only offers a calculated survey with a derived input', () => {
    expect(canTakeOver(assessment(derived))).toBe(true);
    expect(canTakeOver(assessment(derived, 'calculated_legacy_edition'))).toBe(true);
    expect(canTakeOver(assessment(null))).toBe(false);
    expect(canTakeOver(assessment(derived, 'derived_input_rejected'))).toBe(false);
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

function DraftProbe() {
  const shared = useNtaDraft();
  return <pre data-testid="draft">{JSON.stringify({ dirty: shared?.dirty, generator: (shared?.draft as Record<string, unknown> | null)?.generator })}</pre>;
}

const evidenceIn = () => JSON.parse(screen.getByTestId('evidence').textContent ?? '[]') as NtaEvidenceItem[];

describe('evidence link UI', () => {
  beforeEach(() => clearSessionEvidence());

  it('adds a survey photo to the register, linked to the item, and unlinks it', async () => {
    const seeded = project([item('ev-1')]);
    const { container } = renderWithProviders(<>
      <Seed value={seeded} />
      <EvidenceAttach photo pointer="/basisopname/survey/pv/0" />
      <EvidenceProbe />
    </>);
    const input = container.querySelector('input[type="file"]') as HTMLInputElement;
    expect(input.accept).toBe('image/*');
    const file = new File([new TextEncoder().encode('jpeg-bytes')], 'pv-typeplaatje.jpg', { type: 'image/jpeg' });
    await act(async () => { fireEvent.change(input, { target: { files: [file] } }); });
    await waitFor(() => expect(evidenceIn()).toHaveLength(2));
    const added = evidenceIn()[1];
    expect(added).toMatchObject({ id: 'ev-2', fileName: 'pv-typeplaatje.jpg', kind: 'photo_detail', linkedPaths: ['/basisopname/survey/pv/0'] });
    expect(added.sha256).toBe(await sha256Hex(new TextEncoder().encode('jpeg-bytes')));
    expect(screen.getAllByText('pv-typeplaatje.jpg', { exact: false }).length).toBeGreaterThan(0);

    // ev-1 is an image too, so it can be linked; then unlink the new photo.
    const select = container.querySelector('select') as HTMLSelectElement;
    fireEvent.change(select, { target: { value: 'ev-1' } });
    await waitFor(() => expect(evidenceIn()[0].linkedPaths).toEqual(['/basisopname/survey/pv/0']));
    fireEvent.click(screen.getAllByRole('button', { name: /pv-typeplaatje\.jpg/ })[0]);
    await waitFor(() => expect(evidenceIn()[1]).not.toHaveProperty('linkedPaths'));
  });

  it('fills a source field with the chosen evidence reference', async () => {
    let value = '';
    const seeded = project([item('ev-1'), item('ev-2')]);
    const { container, rerender } = renderWithProviders(<>
      <Seed value={seeded} />
      <EvidenceReferencePicker path="generator.sourceReference" value={value} onChange={(next) => { value = next; }} />
    </>);
    await waitFor(() => expect(container.querySelectorAll('option')).toHaveLength(3));
    fireEvent.change(container.querySelector('select') as HTMLSelectElement, { target: { value: 'ev-2' } });
    expect(value).toBe('evidence:ev-2');
    rerender(<>
      <Seed value={seeded} />
      <EvidenceReferencePicker path="generator.sourceReference" value="evidence:ev-9" onChange={() => undefined} />
    </>);
    expect(container.querySelector('.evidence-ref__file--missing')?.textContent).toContain('ev-9');
  });

  it('previews the survey takeover and writes it into the NTA draft', async () => {
    renderWithProviders(<>
      <SurveyTakeoverAction result={assessment(derived)} />
      <DraftProbe />
    </>);
    fireEvent.click(screen.getByRole('button', { name: /projectmodel|project model/i }));
    const dialog = await screen.findByRole('dialog');
    expect(dialog.textContent).toContain('generator');
    expect(dialog.textContent).toContain('boiler');
    fireEvent.click(screen.getByRole('button', { name: /in concept|into draft/i }));
    await waitFor(() => {
      const draft = JSON.parse(screen.getByTestId('draft').textContent ?? '{}');
      expect(draft.dirty).toBe(true);
      expect(draft.generator).toEqual({ kind: 'boiler', efficiency: 0.9 });
    });
    expect(screen.getByRole('status').textContent).toMatch(/concept|draft/);
  });

  it('cannot take over a survey without a derived input', () => {
    renderWithProviders(<SurveyTakeoverAction result={assessment(null)} />);
    expect((screen.getByRole('button', { name: /projectmodel|project model/i }) as HTMLButtonElement).disabled).toBe(true);
  });
});
