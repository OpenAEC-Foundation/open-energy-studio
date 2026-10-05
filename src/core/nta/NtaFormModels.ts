import type { IConstruction, IProject } from '../energy/types';
import type { EnvelopeInput, VentilationFunction, VentilationSystemVariant } from './KernelClient';

// Pure helpers behind the structured NTA input form. They only shape data;
// every norm check stays in the Rust kernel.

type Block = Record<string, unknown>;

/** Table 11.5 variants with their main principle (VENT_SYS_OP). */
export const VENTILATION_VARIANTS: Array<{ variant: VentilationSystemVariant; op: 'natural' | 'supply' | 'extract' | 'balanced' }> = [
  { variant: 'a1', op: 'natural' }, { variant: 'a2a', op: 'natural' }, { variant: 'a2b', op: 'natural' }, { variant: 'a2c', op: 'natural' },
  { variant: 'b1', op: 'supply' }, { variant: 'b2', op: 'supply' }, { variant: 'b3', op: 'supply' },
  { variant: 'c1', op: 'extract' }, { variant: 'c2a', op: 'extract' }, { variant: 'c2b', op: 'extract' }, { variant: 'c2c', op: 'extract' },
  { variant: 'c3a', op: 'extract' }, { variant: 'c3b', op: 'extract' }, { variant: 'c3c', op: 'extract' },
  { variant: 'c4a', op: 'extract' }, { variant: 'c4b', op: 'extract' }, { variant: 'c4c', op: 'extract' },
  { variant: 'c5a', op: 'extract' }, { variant: 'c5b', op: 'extract' },
  { variant: 'd1', op: 'balanced' }, { variant: 'd2', op: 'balanced' }, { variant: 'd3', op: 'balanced' },
  { variant: 'd4a', op: 'balanced' }, { variant: 'd4b', op: 'balanced' },
  { variant: 'd5a', op: 'balanced' }, { variant: 'd5b', op: 'balanced' }, { variant: 'd5c', op: 'balanced' },
];

export function ventilationOp(variant: unknown): 'natural' | 'supply' | 'extract' | 'balanced' | null {
  return VENTILATION_VARIANTS.find((item) => item.variant === variant)?.op ?? null;
}

/** Usage functions of tables 7.13–7.15 to the function list of table 11.8. */
export function ventilationFunction(usage: unknown): VentilationFunction | null {
  switch (usage) {
    case 'residential': return 'residential';
    case 'office': return 'office';
    case 'education': return 'education';
    case 'retail': return 'retail';
    case 'other_assembly': return 'other_assembly';
    case 'assembly_child_care': return 'assembly_child_care';
    case 'other_healthcare': return 'other_healthcare';
    case 'healthcare_with_beds': return 'healthcare_bed_area';
    case 'lodging': return 'lodging_building';
    case 'cell': return 'cell';
    case 'sport': return 'sport';
    default: return null;
  }
}

function defaultVariant(project: IProject): VentilationSystemVariant | null {
  switch (project.ventilationSystems[0]?.type) {
    case 'natural': return 'a1';
    case 'type_c': return 'c1';
    case 'type_d': return 'd2';
    default: return null;
  }
}

/** System unit with the fields the kernel needs for a variant. */
export function ventilationUnit(variant: VentilationSystemVariant | null, previous?: Block): Block {
  const op = ventilationOp(variant);
  return {
    variant,
    ducts: op === 'natural' ? 'no_ducts' : (previous?.ducts && previous.ducts !== 'no_ducts' ? previous.ducts : 'unknown'),
    ...(op === 'balanced' && previous?.heatRecovery ? { heatRecovery: previous.heatRecovery } : {}),
    ...((op === 'balanced' || op === 'supply') && previous?.airHandlingUnit ? { airHandlingUnit: previous.airHandlingUnit } : {}),
    equipmentReference: typeof previous?.equipmentReference === 'string' ? previous.equipmentReference : '',
  };
}

export function heatRecoveryTemplate(): Block {
  return {
    efficiency: { method: 'declared', value: null, standard: 'en13141_7', sourceReference: '' },
    bypass: { kind: 'full' },
    layout: 'central',
    constantVolumeControl: false,
    supplyDuctInsulation: { kind: 'unknown' },
    equipmentReference: '',
  };
}

/** Chapter 11 starting point for the block's single zone. */
export function buildVentilationDraft(block: Block, project: IProject): Block {
  const zone = project.zones[0];
  const variant = defaultVariant(project);
  const draft: Block = {
    zoneId: zone?.id ?? '',
    usableFloorAreaM2: zone?.floorArea ?? null,
    category: block.calculationScope === 'utility' ? 'utility' : 'residential',
    functions: [],
    buildingHeightM: null,
    constructionYear: null,
    floorAboveCrawlspace: false,
    heatingSetpointC: null,
    coolingSetpointC: null,
    system: { kind: 'single', unit: ventilationUnit(variant) },
    infiltration: zone && zone.airTightness?.qv10 > 0
      ? { method: 'measured', qv10DmPerSM2: zone.airTightness.qv10, sourceReference: '' }
      : { method: 'reference', buildingType: null },
    combustionAppliances: [],
    fans: { method: 'forfait', current: 'dc', manufactureYear: null },
    sourceReference: '',
  };
  return syncVentilation({ ...block, ventilation: draft }, project).ventilation as Block;
}

/**
 * Mirrors zone, area, setpoints, category and functions of the block into
 * `ventilation`, empties `ventilationFlows` because the kernel accepts only
 * one of both, and drops declared fan energy that chapter 11 replaces.
 */
export function syncVentilation(block: Block, project: IProject): Block {
  const ventilation = block.ventilation as Block | null | undefined;
  if (!ventilation) return block;
  const zone = project.zones[0];
  const area = zone?.floorArea ?? (ventilation.usableFloorAreaM2 as number | null);
  const setpoints = (block.setpoints as Block | undefined) ?? {};
  const residential = block.calculationScope !== 'utility';
  const fn = ventilationFunction(block.usageFunction);
  const gains = block.internalGains as Block | undefined;
  const synced: Block = {
    ...ventilation,
    zoneId: zone?.id ?? ventilation.zoneId,
    usableFloorAreaM2: area,
    category: residential ? 'residential' : 'utility',
    functions: fn ? [{ function: fn, areaM2: area }] : ventilation.functions ?? [],
    heatingSetpointC: setpoints.heatingC ?? null,
    coolingSetpointC: setpoints.coolingC ?? null,
    dwellingCount: residential ? (gains?.dwellingCount ?? ventilation.dwellingCount ?? 1) : 0,
    apartmentBuilding: block.dwellingType === 'apartment_building',
  };
  // Chapter 11 yields the fan energy itself; a declared fan use would count twice.
  const uses = Array.isArray(block.declaredUses)
    ? (block.declaredUses as Block[]).filter((use) => use.service !== 'ventilation_fans') : block.declaredUses;
  return { ...block, ventilation: synced, ventilationFlows: [], ...(uses !== undefined ? { declaredUses: uses } : {}) };
}

/** Back to explicit H_ve flows. */
export function removeVentilation(block: Block): Block {
  const { ventilation: _removed, ...rest } = block;
  void _removed;
  const flows = rest.ventilationFlows as unknown[] | undefined;
  return {
    ...rest,
    ventilationFlows: flows && flows.length > 0 ? flows : [{
      id: 'ventilation',
      sourceReference: '',
      months: Array.from({ length: 12 }, (_, index) => ({ month: index + 1, conductanceWPerK: null })),
    }],
  };
}

/** Utility internal gains per 7.25–7.29 with the chapter 14 lighting gain. */
export function utilityInternalGains(lighting: 'chapter14' | 'declared' = 'chapter14'): Block {
  return {
    method: 'utility',
    lighting: lighting === 'chapter14' ? { method: 'chapter14' } : { method: 'declared', annualKwh: null, recovery: 'forfait_power' },
    hotWaterRecoverableKwh: [],
    sourceReference: '',
  };
}

/** Distribution usage function of table 9.X for the usage function of 7.13. */
export function distributionUsage(usage: unknown): string {
  switch (usage) {
    case 'residential': return 'residential';
    case 'other_assembly': case 'assembly_child_care': return 'assembly';
    case 'cell': return 'cell';
    case 'healthcare_with_beds': return 'healthcare_with_beds';
    case 'other_healthcare': return 'healthcare_other';
    case 'office': return 'office';
    case 'lodging': return 'lodging';
    case 'education': return 'education';
    case 'sport': return 'sport';
    case 'retail': return 'retail';
    default: return 'residential';
  }
}

export function distributionSystemTemplate(block: Block): Block {
  return {
    designTemperatureClass: null,
    installation: 'individual',
    usageFunction: distributionUsage(block.usageFunction),
    connectedStoreys: 1,
    pipeTransmittance: { method: 'forfait', insulation: { state: 'unknown' } },
    valvesInsulated: false,
    pump: { method: 'included_in_generator_auxiliary' },
    sourceReference: '',
  };
}

/** Chapter 14 lighting for one utility zone with forfait values. */
export function lightingTemplate(project: IProject, labelFunction: unknown): Block {
  const zone = project.zones[0];
  const area = zone?.floorArea ?? null;
  return {
    zoneId: zone?.id ?? '',
    functions: [{ function: typeof labelFunction === 'string' ? labelFunction : 'office', areaM2: area }],
    lightingZones: [{
      id: 'lighting-1',
      areaM2: area,
      power: { method: 'forfait', ledFrom2017: false },
      parasitic: { method: 'forfait' },
      occupancy: { control: 'manual_or_unknown', centralOnControl: false, largeOfficeGroup: false },
      daylight: { method: 'none' },
      extractedLuminaires: false,
    }],
    sourceReference: '',
  };
}

// ---------------------------------------------------------------- constructions

export type NtaLayerDraft =
  | { kind: 'material'; name: string; thicknessM: number; lambda: number; sourceReference: string }
  | { kind: 'air_cavity'; name: string; thicknessMm: number; ventilation: 'unventilated' | 'weakly' | 'strongly' };

export interface NtaConstructionDraft {
  heatFlow: 'upward' | 'horizontal' | 'downward';
  exteriorAir: boolean;
  layers: NtaLayerDraft[];
}

export interface NtaForfaitDraft {
  element: 'facade' | 'floor' | 'roof';
  constructionYear: number;
  insulation: 'absent_or_unknown' | 'present_unknown_thickness' | 'known_thickness';
  thicknessMm: number;
  cavity: boolean;
}

/** Project layers (Σd/λ) as a starting point; λ is taken as λ_calc with a source to fill in. */
export function constructionDraft(construction: Pick<IConstruction, 'layers'>, heatFlow: NtaConstructionDraft['heatFlow'] = 'horizontal'): NtaConstructionDraft {
  return {
    heatFlow,
    exteriorAir: true,
    layers: construction.layers.map((layer) => ({
      kind: 'material', name: layer.material, thicknessM: layer.thickness, lambda: layer.lambda, sourceReference: '',
    })),
  };
}

export function envelopeFromLayers(id: string, draft: NtaConstructionDraft): EnvelopeInput {
  return {
    elements: [{
      id,
      element: {
        kind: 'opaque',
        construction: {
          heatFlow: draft.heatFlow,
          exteriorAir: draft.exteriorAir,
          build: {
            kind: 'homogeneous',
            layers: draft.layers.map((layer) => layer.kind === 'material'
              ? {
                kind: 'material',
                thicknessM: layer.thicknessM,
                conductivity: { method: 'calculated', lambdaCalc: layer.lambda, sourceReference: layer.sourceReference || layer.name },
              }
              : { kind: 'air_cavity', thicknessMm: layer.thicknessMm, ventilation: { kind: layer.ventilation } }),
          },
        },
      },
    }],
  };
}

export function envelopeFromForfait(id: string, draft: NtaForfaitDraft): EnvelopeInput {
  const insulation = draft.insulation === 'known_thickness'
    ? { kind: 'known_thickness', thicknessMm: draft.thicknessMm }
    : { kind: draft.insulation };
  return {
    elements: [{
      id,
      element: {
        kind: 'forfait_opaque',
        element: {
          element: draft.element,
          building: { kind: 'regular' },
          constructionYear: draft.constructionYear,
          insulation,
          cavity: draft.cavity,
        },
      },
    }],
  };
}
