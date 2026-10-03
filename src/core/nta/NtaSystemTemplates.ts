import type { IProject } from '../energy/types';

// Kernel-shaped starting values for the system inputs of the NTA form.
// Unknown values start as null so the form shows them as open; KernelInput
// leaves nulls out, and the kernel reports a required one as a gap by path.

type Block = Record<string, unknown>;

const OTHER_AUX = () => ({ electricallyConnectedDevices: null, nominalPowerKw: null, sourceReference: '' });

/** Single space-heating generator kinds the form can create. */
export const SPACE_GENERATOR_KINDS = ['gas_boiler', 'external_heat', 'heat_pump_forfait', 'electric_resistance', 'biomass', 'chp',
  'gas_heat_pump', 'heat_pump_annex_q', 'product_boiler', 'local_heater', 'forfait_heater'] as const;

/** Table 9.31 class of a building CHP (method 2). */
export function chpClassTemplate(): Block {
  return { powerKw: null, builtAfter2006: true, hreDeclared: false, lowTemperature: false };
}

/** A space-heating generator of `kind` in the kernel's `Generator` shape. */
export function spaceGeneratorTemplate(kind: string, project?: IProject): Block | null {
  switch (kind) {
    case 'external_heat':
      return { kind, supplierReference: '', qualityDeclarationPresent: false, auxiliary: OTHER_AUX() };
    case 'gas_boiler':
      return { kind, boiler: forfaitBoilerTemplate() };
    case 'electric_resistance':
      return { kind, equipmentReference: '', auxiliary: OTHER_AUX() };
    case 'biomass':
      return { kind, appliance: null, location: 'inside_thermal_boundary', annexRCompliantAtMost500Kw: false,
        annexRReference: '', equipmentReference: '', soleHeatingInServedRooms: null, automaticFuelFeed: false,
        auxiliary: OTHER_AUX() };
    case 'heat_pump_forfait':
      return { kind, forfait: project?.heatingSystems[0]?.ntaHeatPump?.forfaitHeatPumpDraft ?? null,
        sourceSystem: 'individual', sourceSystemReference: '' };
    case 'chp':
      return { kind, chp: chpClassTemplate(), method1: null, auxiliary: OTHER_AUX(), equipmentReference: '' };
    case 'gas_heat_pump':
      return { kind, table: 'residential_at_most25_kw', source: 'outdoor_air', designSupplyTemperatureC: null,
        sourceCorrectionFactor: null, auxiliary: OTHER_AUX(), equipmentReference: '' };
    case 'heat_pump_annex_q':
      return { kind, heatPump: annexQHeatPumpTemplate(), designSupplyTemperatureC: null, backup: null, regeneration: null,
        equipmentReference: '' };
    case 'product_boiler':
      return { kind, boiler: productBoilerTemplate(), designTemperatureClass: null };
    case 'local_heater':
      return { kind, heater: localHeaterTemplate(), fuel: 'natural_gas' };
    case 'forfait_heater':
      return { kind, heaterKind: null, fuel: 'natural_gas', equipmentReference: '', pilotFlames: null, auxiliary: OTHER_AUX() };
    case 'multiple':
      return multipleGeneratorsTemplate(project);
    default:
      return null;
  }
}

/** One NEN-EN 14511/14825 measurement of annex Q (tables Q.11/Q.15). */
export function annexQPointTemplate(): Block {
  return { evaporatorInC: null, evaporatorOutC: null, condenserInC: null, condenserOutC: null, cop: null, heatingPowerKw: null };
}

/** Annex Q heat pump with product data; on/off until the part-load series are stated. */
export function annexQHeatPumpTemplate(): Block {
  return {
    source: 'outdoor_air_water', outdoorAirFraction: null,
    maximumPower: { condition1: annexQPointTemplate(), condition2: null, condition3: null, condition4: null },
    modulation: { method: 'on_off' }, switchOff: {}, sourcePump: null, evaporatorInlet: null, testReportReference: '',
  };
}

/** Table Q.15: the five part-load points of the low range (100/88/54/35/15 %). */
export function annexQModulatingTemplate(): Block {
  return { method: 'modulating', minimumPowerKw: null, lowRange: Array.from({ length: 5 }, annexQPointTemplate),
    highRange: null, condenserPumpModulating: false, sourcePumpModulating: false };
}

/** Forfait gas boiler of table 9.25 as the annex Q backup (F_H;gen < 1). */
export function forfaitBoilerTemplate(generatorId = 'boiler'): Block {
  return {
    generatorId, role: 'individual_main', location: null, kind: null, fuel: 'natural_gas',
    averageDesignEmissionTemperatureC: null, emissionCircuit: null, equipmentReference: '',
    locationReference: '', temperatureAndCircuitReference: '', pilotFlamePresent: false,
  };
}

/** Annex M boiler with product values (efficiencies as fractions). */
export function productBoilerTemplate(): Block {
  return {
    technology: null, fuel: 'natural_gas', placement: null, draught: null, control: null,
    product: {
      nominalPowerKw: null, intermediatePowerKw: null,
      fullLoad: { method: 'single', efficiency: null, testTemperatureC: null },
      partLoadEfficiency: null, partLoadTestTemperatureC: null,
      standbyLossFactor: null, standbyTestTemperatureC: null,
      auxiliaryStandbyW: null, auxiliaryIntermediateW: null, auxiliaryFullW: null, sourceReference: '',
    },
    equipmentReference: '',
  };
}

/** Annex N heater; empty product values take the N.6 defaults where they exist. */
export function localHeaterTemplate(): Block {
  return {
    heaterType: null, control: null, productionPeriod: null, condensing: false, pilotFlame: false,
    ventilation: null, location: null, fan: null, envelopeInsulation: null, stoveKind: null, roomHeightM: null,
    product: {}, sourceReference: '',
  };
}

/** 8.3.4.2: crawlspace or unheated basement below a ground floor. */
export function floorBelowTemplate(kind: 'crawlspace' | 'unheated_basement'): Block {
  const common = { floorResistanceM2kPerW: null, depthClass: 'other', wallResistanceM2kPerW: null, wallUValueWPerM2k: null };
  return kind === 'crawlspace'
    ? { kind, ...common, ventilationOpeningM2PerM: null }
    : { kind, ...common, volumeM3: null, airChangesPerHour: null };
}

/** Table D.1: one edge-insulation layer of a slab on ground. */
export function edgeInsulationTemplate(): Block {
  return { kind: 'horizontal', resistanceM2kPerW: null, thicknessM: null, sourceReference: '' };
}

/** BCRG product declaration table (interpolated by the kernel, not part of the project chain). */
export function declaredHeatingTableTemplate(): Block {
  return {
    declarationId: '', declarationNormVersion: 'NTA 8800:2025+C1:2026', sourceReference: '', tableScope: '',
    grossHeatDemandKwhPerYear: null, designSupplyTemperatureC: null, firstRowCoversLowerTemperatures: false,
    rows: [declaredHeatingRowTemplate()],
  };
}

export function declaredHeatingRowTemplate(): Block {
  return { supplyTemperatureC: null, points: [declaredHeatingPointTemplate()] };
}

export function declaredHeatingPointTemplate(): Block {
  return { grossHeatDemandKwhPerYear: null, generationEfficiency: null, preferredEnergyFraction: null,
    auxiliaryElectricityKwhPerYear: null };
}

/** Annex V regeneration of an individual ground source (table V.1). */
export function regenerationTemplate(): Block {
  return { freeCoolingFromSource: false, solar: [], sourceReference: '' };
}

/** Annex V: one solar collector field that regenerates the ground source. */
export function solarRegenerationTemplate(): Block {
  return { collectorAreaM2: null, azimuthDeg: 180, tiltDeg: 45, declaredEfficiency: null, sourceReference: '' };
}

/** §9.6.6.2 micro-CHP (method 1) measured at full load and CHP only. */
export function microChpTemplate(): Block {
  const point = () => ({ thermalPowerKw: null, electricPowerKw: null, thermalEfficiency: null, electricEfficiency: null,
    auxiliaryPowerKw: null });
  return { kind: 'stirling_engine', fuel: 'natural_gas', location: 'heated_space', hydraulics: null,
    fullLoad: point(), chpOnly: point(), testReportReference: '' };
}

/** 9.23 with tables 9.12/9.13: fans and controls of air heaters. */
export function airHeatersTemplate(kind: 'direct' | 'indirect' = 'direct'): Block {
  return {
    kind: kind === 'direct' ? { kind, radialFan: null } : { kind, roomHeightAbove8M: null, warmAirReturn: null, ecMotor: null },
    designHeatLoadW: null,
    sourceReference: '',
  };
}

/** §5.5.8: systems and BACS evidence that derive f_BACS. */
export function bacsTemplate(residential: boolean): Block {
  return {
    buildingUse: residential ? 'residential' : 'utility',
    systemInventoryComplete: false,
    systems: [{ id: 'heating-1', service: 'heating', sourceReference: '',
      generators: [{ id: 'generator-1', nominalThermalCapacityKw: null, sourceReference: '' }] }],
    bacs: { present: false, sourceReference: '' },
  };
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
  if (kind === 'heat_pump_en16147') return { kind, profile: 'l', deliveredKwhPerDay: 11.655, inputKwhPerDay: null,
    exhaustAirSource: false, storageWithoutLegionellaCycle: false, outdoorAirFraction: null, smartControlFactor: null,
    maxTestTemperatureC: null, designSetTemperatureC: null, sourceReference: '' };
  if (kind === 'chp') return { kind, chp: chpClassTemplate(), alsoSpaceHeating: false, equipmentReference: '' };
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

/** §13.6 hot-water vessel; the label is unknown until stated. */
export function hotWaterStorageTemplate(index = 0): Block {
  return { id: `vessel-${index + 1}`, volumeL: null, loss: { method: 'unknown_label', producedFrom2018: false },
    connectionFactor: null, inHeatedZone: true, unheatedAmbientC: null, notInApplianceTest: false, sourceReference: '' };
}

/** §13.6.2 storage loss by route (13.58–13.60). */
export function storageLossTemplate(method: string): Block {
  if (method === 'label') return { method, label: 'c' };
  if (method === 'measured') return { method, transmissionWPerK: null };
  if (method === 'measured_standby') return { method, standbyKwhPerDay: null, referenceStorageC: null, referenceAmbientC: null };
  return { method: 'unknown_label', producedFrom2018: false };
}

/** 9.6.6.2.2.8: storage outside the micro-CHP test configuration. */
export function microChpStorageTemplate(): Block {
  return { lossWPerK: null, setTemperatureC: null, chargingAuxiliaryW: null, sourceReference: '' };
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
