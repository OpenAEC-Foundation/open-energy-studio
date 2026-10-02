import { describe, expect, it } from 'vitest';
import type { IProject } from '../core/energy/types';
import {
  buildVentilationDraft, constructionDraft, distributionSystemTemplate, envelopeFromForfait, envelopeFromLayers,
  removeVentilation, syncVentilation, utilityInternalGains, ventilationFunction, ventilationOp, ventilationUnit,
} from '../core/nta/NtaFormModels';
import { buildNtaCalculationTemplate } from '../core/nta/NtaCalculationTemplate';

function project(overrides: Partial<IProject> = {}): IProject {
  return {
    id: 'p', name: 'p', description: '', buildingFunction: 'residential', address: '', city: '',
    zones: [{ id: 'z1', name: 'Woning', floorArea: 124, volume: 330, height: 2.6, surfaces: [], thermalBridges: [], airTightness: { qv10: 0.4 } }],
    heatingSystems: [], ventilationSystems: [{ id: 'v', name: 'WTW', type: 'type_d', heatRecoveryEfficiency: 0.9, sfp: 0.5 }],
    coolingSystems: [], hotWaterSystems: [], solarPV: [], solarThermal: [], constructions: [],
    ...overrides,
  };
}

describe('NTA form models', () => {
  it('builds a chapter 11 draft that mirrors the demand and empties the explicit flows', () => {
    const p = project();
    const block = buildNtaCalculationTemplate(p);
    const ventilation = buildVentilationDraft(block, p);
    expect(ventilation).toMatchObject({
      zoneId: 'z1', usableFloorAreaM2: 124, category: 'residential',
      functions: [{ function: 'residential', areaM2: 124 }],
      heatingSetpointC: 20, coolingSetpointC: 24, dwellingCount: 1,
      infiltration: { method: 'measured', qv10DmPerSM2: 0.4, sourceReference: '' },
      system: { kind: 'single', unit: { variant: 'd2', ducts: 'unknown' } },
    });
    const synced = syncVentilation({ ...block, ventilation, setpoints: { heatingC: 21, coolingC: 24 } }, p);
    expect(synced.ventilationFlows).toEqual([]);
    expect((synced.ventilation as Record<string, unknown>).heatingSetpointC).toBe(21);
    const back = removeVentilation(synced);
    expect(back.ventilation).toBeUndefined();
    expect((back.ventilationFlows as unknown[]).length).toBe(1);
  });

  it('maps usage functions and table 11.5 principles', () => {
    expect(ventilationFunction('healthcare_with_beds')).toBe('healthcare_bed_area');
    expect(ventilationFunction('lodging')).toBe('lodging_building');
    expect(ventilationOp('c4a')).toBe('extract');
    expect(ventilationUnit('a1')).toMatchObject({ variant: 'a1', ducts: 'no_ducts' });
    // Heat recovery survives only on balanced variants.
    const unit = { ...ventilationUnit('d2'), heatRecovery: { layout: 'central' } };
    expect(ventilationUnit('d5c', unit).heatRecovery).toEqual({ layout: 'central' });
    expect(ventilationUnit('c1', unit).heatRecovery).toBeUndefined();
  });

  it('creates utility gains and a distribution system with kernel field names', () => {
    expect(utilityInternalGains()).toEqual({ method: 'utility', lighting: { method: 'chapter14' }, hotWaterRecoverableKwh: [], sourceReference: '' });
    expect(distributionSystemTemplate({ usageFunction: 'other_assembly' })).toMatchObject({
      usageFunction: 'assembly', pump: { method: 'included_in_generator_auxiliary' },
      pipeTransmittance: { method: 'forfait', insulation: { state: 'unknown' } },
    });
  });

  it('turns project layers and annex I data into envelope input', () => {
    const draft = constructionDraft({ layers: [{ material: 'kalkzandsteen', thickness: 0.1, lambda: 1 }] });
    draft.layers.push({ kind: 'air_cavity', name: 'spouw', thicknessMm: 40, ventilation: 'weakly' });
    const input = envelopeFromLayers('wall', draft);
    const element = input.elements[0].element;
    expect(element.kind).toBe('opaque');
    if (element.kind !== 'opaque' || element.construction.build.kind !== 'homogeneous') throw new Error('shape');
    expect(element.construction.build.layers).toEqual([
      { kind: 'material', thicknessM: 0.1, conductivity: { method: 'calculated', lambdaCalc: 1, sourceReference: 'kalkzandsteen' } },
      { kind: 'air_cavity', thicknessMm: 40, ventilation: { kind: 'weakly' } },
    ]);
    const forfait = envelopeFromForfait('roof', { element: 'roof', constructionYear: 1975, insulation: 'known_thickness', thicknessMm: 60, cavity: false });
    expect(forfait.elements[0].element).toEqual({ kind: 'forfait_opaque', element: {
      element: 'roof', building: { kind: 'regular' }, constructionYear: 1975,
      insulation: { kind: 'known_thickness', thicknessMm: 60 }, cavity: false } });
  });
});
