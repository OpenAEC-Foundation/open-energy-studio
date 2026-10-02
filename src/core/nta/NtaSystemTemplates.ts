import type { IProject } from '../energy/types';

// Kernel-shaped starting values for the system inputs of the NTA form.
// Unknown numbers start as null so the kernel reports them as missing.

type Block = Record<string, unknown>;

const OTHER_AUX = () => ({ electricallyConnectedDevices: null, nominalPowerKw: null, sourceReference: '' });

/** Single space-heating generator kinds the form can create. */
export const SPACE_GENERATOR_KINDS = ['gas_boiler', 'external_heat', 'heat_pump_forfait', 'electric_resistance', 'biomass'] as const;

/** A space-heating generator of `kind` in the kernel's `Generator` shape. */
export function spaceGeneratorTemplate(kind: string, project?: IProject): Block | null {
  switch (kind) {
    case 'external_heat':
      return { kind, supplierReference: '', qualityDeclarationPresent: false, auxiliary: OTHER_AUX() };
    case 'gas_boiler':
      return { kind, boiler: {
        generatorId: 'boiler', role: 'individual_main', location: null, kind: null, fuel: 'natural_gas',
        averageDesignEmissionTemperatureC: null, emissionCircuit: null, equipmentReference: '',
        locationReference: '', temperatureAndCircuitReference: '', pilotFlamePresent: false } };
    case 'electric_resistance':
      return { kind, equipmentReference: '', auxiliary: OTHER_AUX() };
    case 'biomass':
      return { kind, appliance: null, location: 'inside_thermal_boundary', annexRCompliantAtMost500Kw: false,
        annexRReference: '', equipmentReference: '', soleHeatingInServedRooms: null, automaticFuelFeed: false,
        auxiliary: OTHER_AUX() };
    case 'heat_pump_forfait':
      return { kind, forfait: project?.heatingSystems[0]?.ntaHeatPump?.forfaitHeatPumpDraft ?? null,
        sourceSystem: 'individual', sourceSystemReference: '' };
    case 'multiple':
      return multipleGeneratorsTemplate(project);
    default:
      return null;
  }
}

/** 9.6.1: a preferred heat pump and a boiler as the starting split. */
export function multipleGeneratorsTemplate(project?: IProject): Block {
  return {
    kind: 'multiple',
    generators: [
      { preference: 1, nominalPowerKw: null, generator: spaceGeneratorTemplate('heat_pump_forfait', project) },
      { preference: 2, nominalPowerKw: null, generator: spaceGeneratorTemplate('gas_boiler', project) },
    ],
    addedPreferredGenerator: false,
    sourceReference: '',
  };
}

/** Next part of a `multiple` generator: one preference below the lowest. */
export function preferredGeneratorTemplate(parts: Block[], project?: IProject): Block {
  const lowest = parts.reduce((max, part) => Math.max(max, Number(part.preference) || 0), 0);
  return { preference: lowest + 1, nominalPowerKw: null, generator: spaceGeneratorTemplate('gas_boiler', project) };
}

/** A hot-water generator of `kind` in the kernel's `HotWaterGenerator` shape. */
export function hotWaterGeneratorTemplate(kind: string): Block {
  if (kind === 'gas_appliance') return { kind, appliance: null, measuredClass: 'class4', kitchenOnly: false };
  if (kind === 'heat_pump') return { kind, exhaustAirSource: false, measuredClass: 'class4' };
  if (kind === 'indirect_boiler') return { kind, boiler: null, oil: false, insideBoundary: true, alsoSpaceHeating: true };
  if (kind === 'measured_two_profiles') return twoProfileTemplate();
  return { kind };
}

/** §13.8.4.2: appliance tested at two tapping profiles (i1 = M, i2 = L). */
export function twoProfileTemplate(): Block {
  return {
    kind: 'measured_two_profiles',
    standard: 'en13203_gas',
    storageAppliance: false,
    low: { profile: 'm', deliveredKwhPerDay: 5.845, inputKwhPerDay: null, auxiliaryKwhPerDay: null },
    high: { profile: 'l', deliveredKwhPerDay: 11.655, inputKwhPerDay: null, auxiliaryKwhPerDay: null },
    combi: false,
    integratedVessel: false,
    exhaustAirSource: false,
    legionellaCycleTested: false,
    sourceReference: '',
  };
}

/** 13.144a/13.148: exhaust-air use of a hot-water heat pump. */
export function exhaustAirUseTemplate(): Block {
  return { ventilationSuitable: true, heatingTimeFraction: [], declaredFlowM3PerH: null };
}

/** 13.146: declared energy share of a generator. */
export function declaredShareTemplate(): Block {
  return { points: [{ annualKwh: null, share: null }], sourceReference: '' };
}

/** 13.8.2: a further hot-water generator. */
export function additionalHotWaterGeneratorTemplate(): Block {
  return { generator: hotWaterGeneratorTemplate('electric_boiler'), nominalPowerKw: null, equipmentReference: '' };
}

/** §13.7 calculated solar water heater with forfait collector values. */
export function solarWaterHeaterTemplate(index = 0): Block {
  return {
    id: `solar-${index + 1}`,
    solarUse: 'water_heating',
    count: 1,
    method: calculatedSolarMethod(),
    pvt: null,
    sourceReference: '',
  };
}

export function calculatedSolarMethod(): Block {
  return {
    method: 'calculated',
    solarType: 'preheater',
    collectors: {
      moduleAreaM2: null, moduleCount: 1, orientation: 'south', tiltDeg: 45,
      obstruction: { method: 'minimal' },
      efficiency: { method: 'forfait', collector: 'glazed' },
      loopPipes: { method: 'forfait' },
    },
    storage: { totalVolumeL: null, loss: { method: 'unknown_label', producedFrom2018: false } },
  };
}

/** §13.7.2.3: system tested as a whole (hot water only). */
export function testedSolarMethod(): Block {
  return {
    method: 'tested',
    solarType: 'preheater',
    orientation: 'south',
    tiltDeg: 45,
    obstruction: { method: 'minimal' },
    totalVolumeL: null,
    testPoints: [{ annualDemandKwh: null, solarOutputKwh: null, auxiliaryKwh: null }],
    sourceReference: '',
  };
}

/** §17.3 collector obstruction (tables 17.6/17.12/17.15). */
export function collectorObstructionTemplate(method: string): Block {
  switch (method) {
    case 'side_obstruction': return { method, side: 'both', relativeWidth: null };
    case 'roof_edge': return { method, heightM: null, distanceM: null };
    case 'declared': return { method, factors: Array(12).fill(null), sourceReference: '' };
    case 'full':
    case 'other': return { method };
    default: return { method: 'minimal' };
  }
}

/** §17.3 window obstruction situations a–g (tables 17.4–17.14). */
export function windowObstructionTemplate(method: string): Block {
  switch (method) {
    case 'parallel_obstruction':
    case 'overhang': return { method, relativeHeight: null };
    case 'side_obstruction': return { method, side: 'both', relativeWidth: null, coolingHeightCondition: false };
    case 'full': return { method, coolingConditionsMet: false };
    case 'other': return { method, overhangRelativeHeight: null };
    case 'declared': return { method, heating: Array(12).fill(null), cooling: Array(12).fill(null), sourceReference: '' };
    default: return { method: 'minimal' };
  }
}

const EN14825_POINT = (partLoadPercent: number) =>
  ({ partLoadPercent, eer: null, evaporatorOutletC: null, condenserInletC: null });

/** §10.5.4 (NEN-EN 14825) or §10.5.5 (NEN-EN 14511) rating of a compression generator. */
export function coolingPerformanceTemplate(method: string, roomUnit: boolean): Block | null {
  if (method === 'en14825') {
    return {
      method, nominalEer: null, nominalCapacityKw: null, minimumCapacityKw: null,
      testPoints: [EN14825_POINT(100), EN14825_POINT(74), EN14825_POINT(47), EN14825_POINT(21)],
      sourceReference: '',
    };
  }
  if (method === 'en14511') {
    return {
      method, nominalEer: null, nominalCapacityKw: null, nominalEvaporatorOutletC: 7, nominalCondenserInletC: 35,
      ...(roomUnit ? { roomUnitType: 'split_inverter' } : {}),
      sourceReference: '',
    };
  }
  return null;
}

export function en14825PointTemplate(partLoadPercent = 47): Block {
  return EN14825_POINT(partLoadPercent);
}
