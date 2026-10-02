import { invoke, isTauri } from '@tauri-apps/api/core';
import type { IProject, INtaHeatPumpInput } from '../energy/types';

/** Annex T Gaskeur test report (gas water heaters and combi appliances). */
export type NtaAnnexTTest =
  | {
      method: 'water_heater';
      usefulMj: number;
      fuelInputMj: number;
      electricityKwh: number;
      fuel: 'natural_gas' | 'propane' | 'butane';
      sourceReference: string;
    }
  | {
      method: 'combi_forfait';
      usefulMj: number;
      fuelInputMj: number;
      electricityKwh: number;
      fullLoadEfficiency: number;
      fuel: 'natural_gas' | 'propane' | 'butane';
      sourceReference: string;
    }
  | {
      method: 'combi_measured';
      summerUsefulMjPerDay: number;
      summerFuelMjPerDay: number;
      winterUsefulMjPerDay: number;
      winterFuelMjPerDay: number;
      heatingFuelMjPerDay: number;
      fullLoadEfficiency: number;
      summerElectricityKwhPerDay: number;
      electronicsKwhPerDay: number;
      fuel: 'natural_gas' | 'propane' | 'butane';
      sourceReference: string;
    };

/** Annex U shower heat recovery test: three runs at one class. */
export interface NtaAnnexUTest {
  class: 'class2' | 'class3' | 'class4';
  runs: Array<
    | { method: 'energies'; recoveredKj: number; showerKj: number }
    | {
        method: 'samples';
        sampleTimeS: number;
        samples: Array<{
          coldFlowM3PerS: number;
          coldInC: number;
          coldOutC: number;
          showerFlowM3PerS: number;
          showerC: number;
        }>;
      }
  >;
  sourceReference: string;
}

export interface KernelIssue {
  severity: 'error' | 'warning';
  code: string;
  path: string;
  message: string;
  detail?: string;
}

export interface KernelAssessment {
  status: 'invalid' | 'structurally_valid';
  targetNormVersion: string;
  kernelVersion: string;
  inputFingerprint: string;
  calculationAvailable: boolean;
  summary: {
    zoneCount: number;
    surfaceCount: number;
    systemCount: number;
    heatPumpCount: number;
    classifiedHeatPumpCount: number;
    auxiliaryComponentCount?: number;
    systemLinkCount?: number;
    performancePointDiagnostics?: Array<{
      heatPumpPath: string;
      heatPumpId: string;
      pointId: string;
      service: 'space_heating' | 'domestic_hot_water' | 'space_cooling';
      inputEnergyCarrier: 'electricity' | 'gas' | 'district_heat' | 'other';
      instantaneousUsefulToInputRatio: number;
      referenceVerified: boolean;
      annualPerformanceAvailable: boolean;
    }>;
    dhwTestDiagnostics?: Array<{
      heatPumpId: string;
      pointId: string;
      tapProfile: string;
      declarationNormVersion: string;
      declaredUsefulToInputRatio: number;
      referenceVerified: boolean;
      annualPerformanceAvailable: boolean;
    }>;
    floorAreaM2: number;
    thermalBoundaries?: {
      surfaceCount: number;
      classifiedSurfaceCount: number;
      bridgeCount: number;
      classifiedBridgeCount: number;
      pointBridgeCount?: number;
      classifiedPointBridgeCount?: number;
      pointInventoryComplete?: boolean;
      complete: boolean;
    };
    directOutdoorDiagnostic?: {
      status: 'input_valid' | 'invalid';
      scope: string;
      referenceVerified: boolean;
      bengCalculationAvailable: boolean;
      inputFingerprint: string;
      elementConductanceWPerK: number | null;
      linearBridgeConductanceWPerK: number | null;
      pointBridgeConductanceWPerK: number | null;
      totalDirectConductanceWPerK: number | null;
    } | null;
    unheatedTransmissionDiagnostic?: {
      status: 'input_valid' | 'invalid';
      referenceVerified: boolean;
      bengCalculationAvailable: boolean;
      totalReducedConductanceWPerK: number | null;
      spaces: Array<{ id: string; unreducedConductanceWPerK: number; reductionFactor: number; reducedConductanceWPerK: number }>;
    } | null;
    envelopeGeometry?: {
      grossSurfaceAreaM2: number;
      windowAreaM2: number;
      remainingOpaqueAreaM2: number;
      zones: Array<{
        zoneId: string;
        grossSurfaceAreaM2: number;
        windowAreaM2: number;
        remainingOpaqueAreaM2: number;
      }>;
    } | null;
  };
  issues: KernelIssue[];
}

export interface DeclaredHeatingPoint {
  grossHeatDemandKwhPerYear: number;
  generationEfficiency: number;
  preferredEnergyFraction: number;
  auxiliaryElectricityKwhPerYear: number;
}

export interface DeclaredHeatingTableInput {
  declarationId: string;
  declarationNormVersion: string;
  sourceReference: string;
  tableScope: string;
  grossHeatDemandKwhPerYear: number;
  designSupplyTemperatureC: number;
  firstRowCoversLowerTemperatures?: boolean;
  rows: Array<{ supplyTemperatureC: number; points: DeclaredHeatingPoint[] }>;
}

export interface DeclaredHeatingTableAssessment {
  status: 'input_valid' | 'invalid';
  scope: string;
  inputFingerprint: string;
  declarationId: string;
  declarationNormVersion: string;
  sourceReference: string;
  tableScope: string;
  declarationEditionMatchesTarget: boolean;
  referenceVerified: false;
  annualPerformanceAvailable: false;
  bengCalculationAvailable: false;
  interpolation: {
    demandLowerKwhPerYear: number;
    demandUpperKwhPerYear: number;
    demandWeight: number;
    temperatureLowerC: number;
    temperatureUpperC: number;
    temperatureWeight: number;
  } | null;
  generationEfficiency: number | null;
  preferredEnergyFraction: number | null;
  auxiliaryElectricityKwhPerYear: number | null;
  issues: Array<{ code: string; path: string }>;
}

export async function diagnoseDeclaredHeatingTableWithRust(input: DeclaredHeatingTableInput): Promise<DeclaredHeatingTableAssessment> {
  if (isTauri()) {
    return invoke<DeclaredHeatingTableAssessment>('diagnose_declared_heating_table', { input });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/heat-pumps/declared-heating-table/diagnose', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ input }),
    });
    return response.json() as Promise<DeclaredHeatingTableAssessment>;
  }
  throw new Error('Rust table diagnostics are available in the desktop app and local development server.');
}

export interface DeclaredDhwAssessment {
  status: 'input_valid' | 'invalid';
  scope: 'declared_dhw_test_input_only';
  inputFingerprint: string;
  heatPumpId: string;
  referenceVerified: false;
  annualPerformanceAvailable: false;
  bengCalculationAvailable: false;
  profiles: Array<{
    pointId: string;
    tapProfile: string;
    declarationNormVersion: string;
    declarationEditionMatchesTarget: boolean;
    sourceReference: string;
    usefulEnergyKwhPerDay: number;
    inputEnergyKwhPerDay: number;
    rawUsefulToInputRatio: number;
    practiceFactorInput: number;
    nominalCapacityKw: number;
  }>;
  issues: Array<{ code: string; path: string }>;
}

export async function diagnoseDeclaredDhwWithRust(input: INtaHeatPumpInput): Promise<DeclaredDhwAssessment> {
  if (isTauri()) {
    return invoke<DeclaredDhwAssessment>('diagnose_declared_dhw', { input });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/heat-pumps/declared-dhw/diagnose', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ input }),
    });
    return response.json() as Promise<DeclaredDhwAssessment>;
  }
  throw new Error('Rust declaration diagnostics are available in the desktop app and local development server.');
}

export interface FinalEnergyDraftInput {
  carrierInventoryComplete: boolean;
  solarThermalInventoryComplete: boolean;
  totalUsableFloorAreaM2?: number;
  areaSourceReference?: string;
  carriers: Array<{
    carrierCode: string;
    sourceReference: string;
    monthlyEpusKwh: Array<{ month: number; energyKwh: number }>;
  }>;
  solarThermal?: Array<{
    systemId: string;
    service: 'space_heating' | 'domestic_hot_water';
    sourceReference: string;
    monthlyRenewablePracticeKwh: Array<{ month: number; energyKwh: number }>;
  }>;
}

export interface FinalEnergyDraftAssessment {
  status: 'input_valid' | 'incomplete' | 'invalid';
  scope: 'public_chapter_5_draft_arithmetic_only';
  draftSource: string;
  inputFingerprint: string;
  finalEditionVerified: false;
  referenceVerified: false;
  bengCalculationAvailable: false;
  annualByCarrierKwh: Array<{ carrierCode: string; energyKwhPerYear: number }>;
  finalEnergyKwhPerYear: number | null;
  solarThermalKwhPerYear: number | null;
  eedFinalEnergyKwhPerYear: number | null;
  finalEnergyIndicatorKwhPerM2Year: number | null;
  eedFinalEnergyIndicatorKwhPerM2Year: number | null;
  issues: Array<{ code: string; path: string }>;
}

export async function diagnoseFinalEnergyDraftWithRust(input: FinalEnergyDraftInput): Promise<FinalEnergyDraftAssessment> {
  if (isTauri()) {
    return invoke<FinalEnergyDraftAssessment>('diagnose_final_energy_draft', { input });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/energy/final-draft/diagnose', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ input }),
    });
    return response.json() as Promise<FinalEnergyDraftAssessment>;
  }
  throw new Error('Rust draft energy diagnostics are available in the desktop app and local development server.');
}

export interface EpusDraftMonthlyUse {
  month: number;
  spaceHeatingKwh: number;
  humidificationKwh: number;
  ventilationKwh: number;
  lightingKwh: number;
  spaceCoolingKwh: number;
  dehumidificationKwh: number;
  domesticHotWaterKwh: number;
  collectiveSourceHeatKwh: number;
  auxiliary: {
    spaceHeatingKwh: number;
    humidificationKwh: number;
    spaceCoolingKwh: number;
    dehumidificationKwh: number;
    domesticHotWaterKwh: number;
    solarPvKwh: number;
  };
}

export interface EpusDraftInput extends Omit<FinalEnergyDraftInput, 'carriers'> {
  bacsFactor: 1 | 1.05;
  bacsSourceReference: string;
  bacsEvidence?: BacsDraftInput;
  carriers: Array<{
    carrierCode: 'el' | 'gas' | 'bm' | 'dh' | 'dw' | 'dc' | 'oil';
    sourceReference: string;
    months: EpusDraftMonthlyUse[];
  }>;
  /**
   * Optional collective gas heat-pump source evidence. When valid, the kernel
   * derives the monthly `dh` source heat (draft eq. 5.20) itself; the supplied
   * `dh` rows must then keep `collectiveSourceHeatKwh` at 0 to avoid double counting.
   */
  gasCollectiveSourceEvidence?: GasCollectiveSourceDraftInput;
}

export interface EpusDraftAssessment {
  status: 'input_valid' | 'incomplete' | 'invalid';
  scope: 'public_chapter_5_draft_service_composition_only';
  draftSource: string;
  inputFingerprint: string;
  finalEditionVerified: false;
  referenceVerified: false;
  bengCalculationAvailable: false;
  monthlyByCarrierKwh: Array<{ carrierCode: string; month: number; energyKwh: number }>;
  gasCollectiveSourceDerived: boolean;
  gasCollectiveSourceFingerprint: string | null;
  finalEnergy: FinalEnergyDraftAssessment | null;
  issues: Array<{ code: string; path: string }>;
}

export async function diagnoseEpusDraftWithRust(input: EpusDraftInput): Promise<EpusDraftAssessment> {
  if (isTauri()) {
    return invoke<EpusDraftAssessment>('diagnose_epus_draft', { input });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/energy/epus-draft/diagnose', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ input }),
    });
    return response.json() as Promise<EpusDraftAssessment>;
  }
  throw new Error('Rust draft energy diagnostics are available in the desktop app and local development server.');
}

export type NtaOrientation =
  | 'north' | 'north_east' | 'east' | 'south_east'
  | 'south' | 'south_west' | 'west' | 'north_west';
export type NtaMassClass = 'light' | 'heavy' | 'very_heavy';
export type NtaObstructionSide = 'left' | 'right' | 'both';
/** §17.3.2 situations a–g (tables 17.4–17.14); relative heights perpendicular to the window. */
export type NtaObstruction =
  | { method: 'minimal' }
  | { method: 'parallel_obstruction'; relativeHeight: number }
  | { method: 'overhang'; relativeHeight: number }
  | { method: 'side_obstruction'; side: NtaObstructionSide; relativeWidth: number; coolingHeightCondition?: boolean }
  | { method: 'full'; coolingConditionsMet?: boolean }
  | { method: 'other'; overhangRelativeHeight?: number | null }
  | { method: 'declared'; heating: number[]; cooling: number[]; sourceReference: string };
/** Collectors and PV (x = P): tables 17.6, 17.12 and 17.15. */
export type NtaCollectorObstruction =
  | { method: 'minimal' }
  | { method: 'side_obstruction'; side: NtaObstructionSide; relativeWidth: number }
  | { method: 'full' }
  | { method: 'roof_edge'; heightM: number; distanceM: number }
  | { method: 'other' }
  | { method: 'declared'; factors: number[]; sourceReference: string };
export type NtaShadingControl =
  | 'manual_residential'
  | 'automatic_residential_iso52016'
  | 'automatic'
  | 'manual_utility_with_glare_protection'
  | 'manual_utility_without_glare_protection';
export interface NtaMovableShading {
  reductionFactor: number;
  control: NtaShadingControl;
  sourceReference: string;
}

/** Usage functions of NTA 8800 tables 7.13–7.15. */
export type NtaUsageFunction =
  | 'assembly_child_care' | 'other_assembly' | 'cell' | 'healthcare_with_beds'
  | 'other_healthcare' | 'office' | 'lodging' | 'education' | 'sport' | 'retail'
  | 'residential';
/** 7.78 f_mod;sp: 0,5 apartment buildings, 0,6 other dwellings. */
export type NtaDwellingType = 'apartment_building' | 'other';

export type NtaGroundEdgeThermalBridges =
  | { method: 'detailed'; bridges: Array<{ lengthM: number; psiWPerMk: number; sourceReference: string }> }
  | { method: 'forfait' };
export interface NtaGroundEdgeInsulation {
  kind: 'horizontal' | 'vertical';
  resistanceM2kPerW: number;
  thicknessM: number;
  sourceReference: string;
}

export interface NtaDirectTransmissionInput {
  elements: Array<{ id: string; areaM2: number; uValueWPerM2k: number; sourceReference: string }>;
  linearBridges?: Array<{ id: string; lengthM: number; psiWPerMk: number; sourceReference: string }>;
  pointBridges?: Array<{ id: string; chiWPerK: number; sourceReference: string }>;
}

export type MonthlyDemandTransmission =
  | {
      method: 'explicit';
      conductanceWPerK: number;
      sourceReference: string;
      ground: null | {
        monthlyConductanceWPerK: number[];
        heatingAdjustedConductanceWPerK: number;
        coolingAdjustedConductanceWPerK: number;
        sourceReference: string;
      };
      groundInventoryConfirmed: boolean;
    }
  | {
      method: 'components';
      direct: NtaDirectTransmissionInput;
      unheated: null | {
        spaces: Array<{
          id: string;
          /** Declared b_U (basic survey, annex I.2.4); exclusive with `outside`. */
          reductionFactor?: number;
          factorSourceReference?: string;
          boundary: NtaDirectTransmissionInput;
          /** Derive b_U from the space's losses to outside (8.53–8.59). */
          outside?: {
            transmission: NtaDirectTransmissionInput;
            ventilation:
              | { method: 'flow'; airflowM3PerH: number; sourceReference: string }
              | { method: 'half_of_transmission' };
            otherZonesConductanceWPerK?: number;
          };
        }>;
      };
      groundFloors: Array<{
        id: string;
        areaM2: number;
        exposedPerimeterM: number;
        constructionResistanceM2kPerW: number;
        edgeThermalBridges: NtaGroundEdgeThermalBridges;
        edgeInsulation?: NtaGroundEdgeInsulation[];
        below?: NtaFloorBelow | null;
        heatedBasement?: NtaHeatedBasement | null;
        sourceReference: string;
      }>;
      groundInventoryConfirmed: boolean;
      /** 7.3.3 vertical pipes through the envelope (H_p, table 7.1). */
      verticalPipes?: Array<{ id: string; storeys: number; insulated: boolean; sharedZones?: number; sourceReference: string }>;
    };

/** 8.3.4.2: crawlspace or unheated basement below a ground floor. */
export type NtaFloorBelow =
  | {
      kind: 'crawlspace';
      floorResistanceM2kPerW?: number;
      depthClass: 'on_sand' | 'other';
      wallResistanceM2kPerW: number;
      wallUValueWPerM2k: number;
      ventilationOpeningM2PerM?: number | null;
    }
  | {
      kind: 'unheated_basement';
      floorResistanceM2kPerW?: number;
      depthClass: 'on_sand' | 'other';
      wallResistanceM2kPerW: number;
      wallUValueWPerM2k: number;
      volumeM3: number;
      airChangesPerHour?: number | null;
    };

/** 8.3.3.2: heated room with its floor below ground level. */
export interface NtaHeatedBasement {
  depthM: number;
  wallResistanceM2kPerW: number;
  forfaitDeltaUWPerM2k?: number | null;
}

/** Chapter 12 humidifier of one zone (space-heating chain input). */
export interface NtaZoneHumidifier {
  zoneId: string;
  humidification: {
    humidifier: { kind: 'atomising' } | { kind: 'steam'; carrier: 'electricity' | 'gas_or_oil' };
    rotaryWheel: boolean;
    equipmentReference: string;
  };
}

/** 7.30b adjacent unheated sunroom (AOS). */
export interface NtaSunroom {
  id: string;
  glazingGHeating: number;
  glazingGCooling: number;
  exteriorFrameFraction: number;
  reductionFactor: number;
  zoneConductanceWPerK: number;
  distributionFactor?: number;
  surfaces: Array<{ areaM2: number; absorptance: number; azimuthDeg: number; tiltDeg: number }>;
  sourceReference: string;
}

export interface MonthlyDemandInput {
  zoneId: string;
  usableFloorAreaM2: number;
  areaSourceReference: string;
  usageFunction: NtaUsageFunction;
  /** §6.5.3: functions with areas in a mixed zone (area-weighted values). */
  functionAreas?: Array<{ function: NtaUsageFunction; areaM2: number }>;
  dwellingType?: NtaDwellingType | null;
  setpoints: { heatingC: number; coolingC: number; sourceReference: string };
  transmission: MonthlyDemandTransmission;
  ventilationFlows: Array<{
    id: string;
    sourceReference: string;
    months: Array<{
      month: number;
      conductanceWPerK: number;
      supplyTemperatureC?: number | null;
      coolingConductanceWPerK?: number | null;
      coolingSupplyTemperatureC?: number | null;
    }>;
  }>;
  /** Chapter 11 input; leave `ventilationFlows` empty when given. */
  ventilation?: VentilationInput | null;
  thermalMass: {
    floor: NtaMassClass;
    wall: NtaMassClass;
    ceiling: 'closed_or_suspended' | 'open_or_none';
    /** Annex B: elements whose effective heat capacity replaces table 7.10. */
    annexBElements?: NtaMassElement[];
    sourceReference: string;
  };
  internalGains:
    | { method: 'residential'; dwellingCount: number; sourceReference: string }
    | {
        method: 'utility';
        /** Φ_int;L (7.28): from chapter 14 lighting, declared W_t, or resolved. */
        lighting:
          | { method: 'chapter14' }
          | { method: 'declared'; annualKwh: number; recovery: 'forfait_power' | 'extracted_luminaires' | 'other' }
          | { method: 'resolved'; gainW: number };
        hotWaterRecoverableKwh?: number[];
        sourceReference: string;
      }
    | { method: 'declared'; heatFluxWPerM2: number; sourceReference: string };
  /** Adjacent unheated sunrooms (7.30b). */
  sunrooms?: NtaSunroom[];
  windowInventoryComplete: boolean;
  windows: Array<{
    id: string;
    areaM2: number;
    orientation: NtaOrientation;
    tiltDeg: number;
    gPerpendicular: number;
    frameFraction: number;
    uValueWPerM2k: number;
    obstruction: NtaObstruction;
    movableShading?: NtaMovableShading | null;
    /** Annex A: dynamic g and U per month. */
    dynamic?: NtaDynamicTransparent | null;
    sourceReference: string;
  }>;
  opaqueInventoryComplete: boolean;
  opaqueElements: Array<{
    id: string;
    areaM2: number;
    orientation: NtaOrientation;
    tiltDeg: number;
    uValueWPerM2k: number;
    sourceReference: string;
  }>;
}

export interface MonthlyDemandBalanceTerms {
  setpointC: number;
  reductionFactor: number;
  calculationTemperatureC: number;
  ventilationConductanceWPerK: number;
  timeConstantH: number;
  a: number;
  transmissionKwh: number;
  ventilationKwh: number;
  heatTransferKwh: number;
  gainsKwh: number;
  gamma: number | null;
  utilization: number;
  needKwh: number;
}

export interface MonthlyDemandAssessment {
  status: 'calculated_unverified' | 'invalid';
  scope: string;
  climateSource: string;
  inputFingerprint: string;
  finalEditionVerified: false;
  referenceVerified: false;
  bengCalculationAvailable: false;
  omittedCorrections: string[];
  specificHeatCapacityKjPerM2k: number | null;
  transmission: null | {
    method: 'explicit' | 'components';
    conductanceWPerK: number;
    directConductanceWPerK: number | null;
    unheatedConductanceWPerK: number | null;
    verticalPipeConductanceWPerK: number | null;
    groundSteadyConductanceWPerK: number | null;
    groundMonthlyConductanceWPerK: number[];
    groundHeatingAdjustedWPerK: number;
    groundCoolingAdjustedWPerK: number;
    annualMeanOutdoorTemperatureC: number;
  };
  monthly: Array<{
    month: number;
    hours: number;
    outdoorTemperatureC: number;
    internalGainsKwh: number;
    windowSolarGainsKwh: number;
    windowSolarCoolingKwh: number;
    opaqueSolarGainsKwh: number;
    groundConductanceWPerK: number;
    heating: MonthlyDemandBalanceTerms;
    cooling: MonthlyDemandBalanceTerms;
  }>;
  annualHeatingNeedKwh: number | null;
  annualCoolingNeedKwh: number | null;
  recoverableLossesApplied: boolean;
  annualHeatingNeedWithoutRecoverableKwh: number | null;
  annualCoolingNeedWithoutRecoverableKwh: number | null;
  ventilation: VentilationResult | null;
  /** 9.28/9.29 heating-limit need per month (chapter 11 input only). */
  heatingLimitNeedKwh: number[];
  /** §5.4.2 need with the fixed C1 system (BENG 1). */
  fixedC1: {
    status: 'calculated_unverified' | 'invalid' | 'unavailable';
    monthlyHeatingNeedKwh: number[];
    monthlyCoolingNeedKwh: number[];
    annualHeatingNeedKwh: number | null;
    annualCoolingNeedKwh: number | null;
    issues: Array<{ code: string; path: string }>;
  } | null;
  issues: Array<{ code: string; path: string }>;
}

export async function calculateMonthlyDemandWithRust(input: MonthlyDemandInput): Promise<MonthlyDemandAssessment> {
  if (isTauri()) {
    return invoke<MonthlyDemandAssessment>('calculate_monthly_demand', { input });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/demand/monthly/calculate', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ input }),
    });
    return response.json() as Promise<MonthlyDemandAssessment>;
  }
  throw new Error('Rust demand calculation is available in the desktop app and local development server.');
}

export interface BacsDraftInput {
  buildingUse: 'residential' | 'utility';
  systemInventoryComplete: boolean;
  systems: Array<{
    id: string;
    service: 'heating' | 'cooling';
    sourceReference: string;
    bacs?: {
      present: boolean;
      automaticControlsClass?: 'A' | 'B' | 'C' | 'D';
      energyManagementClass?: 'A' | 'B' | 'C' | 'D';
      sourceReference: string;
    };
    generators: Array<{
      id: string;
      nominalThermalCapacityKw: number | null;
      sourceReference: string;
    }>;
  }>;
  bacs?: {
    present: boolean;
    automaticControlsClass?: 'A' | 'B' | 'C' | 'D';
    energyManagementClass?: 'A' | 'B' | 'C' | 'D';
    sourceReference: string;
  };
}

export interface BacsDraftAssessment {
  status: 'input_valid' | 'incomplete' | 'invalid';
  scope: 'public_chapter_5_draft_bacs_factor_only';
  draftSource: string;
  targetNormVersion: string;
  kernelVersion: string;
  inputFingerprint: string;
  finalEditionVerified: false;
  referenceVerified: false;
  bengCalculationAvailable: false;
  factor: 1 | 1.05 | null;
  applicability: 'undetermined' | 'residential_exempt' | 'below_or_equal_threshold' | 'utility_triggered';
  triggeringSystemIds: string[];
  systems: Array<{
    id: string;
    nominalThermalCapacityKw: number | null;
    capacityUndeterminable: boolean;
    above290Kw: boolean;
  }>;
  issues: Array<{ code: string; path: string }>;
}

export async function diagnoseBacsDraftWithRust(input: BacsDraftInput): Promise<BacsDraftAssessment> {
  if (isTauri()) {
    return invoke<BacsDraftAssessment>('diagnose_bacs_draft', { input });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/energy/bacs-draft/diagnose', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ input }),
    });
    return response.json() as Promise<BacsDraftAssessment>;
  }
  throw new Error('Rust draft BACS diagnostics are available in the desktop app and local development server.');
}

export interface IndicatorsDraftInput {
  calculationScope: 'residential' | 'utility';
  totalUsableFloorAreaM2: number;
  areaSourceReference: string;
  annualNeedC1Kwh: number;
  needSourceReference: string;
  scenarios: Array<{
    kind: 'ordinary' | 'emg_declaration' | 'emg_forfait';
    annualPrimaryFossilKwh: number;
    annualRenewableKwh: number;
    sourceReference: string;
  }>;
}

export interface IndicatorsDraftAssessment {
  status: 'input_valid' | 'invalid';
  scope: 'public_chapter_5_draft_indicators_from_supplied_annual_totals_only';
  draftSource: string;
  targetNormVersion: string;
  kernelVersion: string;
  inputFingerprint: string;
  finalEditionVerified: false;
  referenceVerified: false;
  bengCalculationAvailable: false;
  labelAvailable: false;
  needIndicatorKwhPerM2Year: number | null;
  scenarios: Array<{
    kind: IndicatorsDraftInput['scenarios'][number]['kind'];
    primaryFossilIndicatorKwhPerM2Year: number;
    renewableSharePercent: number;
    renewableIndicatorKwhPerM2Year: number;
  }>;
  issues: Array<{ code: string; path: string }>;
}

export async function diagnoseIndicatorsDraftWithRust(input: IndicatorsDraftInput): Promise<IndicatorsDraftAssessment> {
  if (isTauri()) {
    return invoke<IndicatorsDraftAssessment>('diagnose_indicators_draft', { input });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/energy/indicators-draft/diagnose', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ input }),
    });
    return response.json() as Promise<IndicatorsDraftAssessment>;
  }
  throw new Error('Rust draft indicator diagnostics are available in the desktop app and local development server.');
}

export interface HeatingAuxDraftInput {
  generatorId: string;
  generatorSourceReference: string;
  coefficients: {
    aAnnualKwh: number;
    bKw: number;
    cDimensionless: number;
    nominalElectricDriveKw: number;
    sourceReference: string;
  };
  inputEnergySourceReference: string;
  months: Array<{ month: number; generatorInputElectricityKwh: number }>;
}

export interface HeatingAuxDraftAssessment {
  status: 'input_valid' | 'invalid';
  scope: 'public_chapter_9_draft_9_85_measured_coefficients_one_individual_generator_only';
  draftSource: string;
  targetNormVersion: string;
  kernelVersion: string;
  inputFingerprint: string;
  finalEditionVerified: false;
  referenceVerified: false;
  bengCalculationAvailable: false;
  annualHeatPumpPerformanceAvailable: false;
  sourcePumpAndFanIncluded: false;
  forfaitUsed: false;
  monthlyAuxiliaryElectricityKwh: Array<{ month: number; electricityKwh: number }>;
  annualAuxiliaryElectricityKwh: number | null;
  issues: Array<{ code: string; path: string }>;
}

export async function diagnoseHeatingAuxDraftWithRust(input: HeatingAuxDraftInput): Promise<HeatingAuxDraftAssessment> {
  if (isTauri()) {
    return invoke<HeatingAuxDraftAssessment>('diagnose_heating_aux_draft', { input });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/heat-pumps/heating-aux-draft/diagnose', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ input }),
    });
    return response.json() as Promise<HeatingAuxDraftAssessment>;
  }
  throw new Error('Rust draft heat-pump auxiliary diagnostics are available in the desktop app and local development server.');
}

export interface HeatingAuxMeasuredDraftInput extends Omit<HeatingAuxDraftInput, 'coefficients'> {
  measurements: {
    standbyElectronicsW: number;
    deliveryPumpDuringCompressorW: number;
    deliveryPumpPrePostW: number;
    pumpPreRunSeconds: number;
    pumpPostRunSeconds: number;
    averageCompressorOnSeconds: number;
    meanCompressorModulation: number;
    nominalElectricDriveKw: number;
    measurementSourceReference: string;
    timingSourceReference: string;
  };
}

export interface ForfaitHeatPumpDraftInput {
  generatorId: string;
  classificationSourceReference: string;
  scope: 'residential_at_most25_kw' | 'utility_collective_or_over25_kw';
  source: 'ground' | 'ground_or_groundwater_unknown' | 'groundwater_below15_c' | 'outdoor_air' | 'exhaust_air'
    | 'surface_water' | 'collective15_to20_c' | 'collective20_to40_c' | 'collective_at_least40_c';
  sink: 'hydronic' | 'indoor_air';
  designSupplyTemperatureC: number | null;
  sourceCorrectionFactor: number | null;
  sourceCorrectionReference: string | null;
  thermalCapacityKw?: number | null;
  capacitySourceReference?: string | null;
  collectiveBuildingInstallation?: boolean | null;
  rowVariant?: 'base' | 'table_9_28_high_efficiency';
  highEfficiencyEvidence?: {
    productReference: string;
    testReportReference: string;
    testStandardEdition: string;
    points: Array<{ condition: 'b0_w45' | 'b0_w35' | 'w10_w45' | 'w10_w35'
      | 'a7_wet6_w45' | 'a7_wet6_w35' | 'a_minus7_wet_minus8_w45'; measuredCop: number }>;
  } | null;
  sourceTemperatureC?: number | null;
  sourceTemperatureEvidenceReference?: string | null;
  sourceQualityDeclarationReference?: string | null;
}

export interface ForfaitHeatPumpDraftAssessment {
  status: 'input_valid' | 'invalid';
  scope: 'public_chapter_9_draft_electric_heat_pump_forfait_lookup_only';
  consultationSource: string;
  targetNormVersion: string;
  kernelVersion: string;
  inputFingerprint: string;
  table: '9.27' | '9.29';
  rowVariant: 'base' | 'table_9_28_high_efficiency';
  selectedSource: ForfaitHeatPumpDraftInput['source'];
  sourceFallbackApplied: boolean;
  sourceFallbackReason: 'ground_or_groundwater_unknown' | 'source_quality_declaration_missing' | null;
  temperatureBand: string | null;
  tableCop: number | null;
  correctedCop: number | null;
  finalEditionVerified: false;
  applicabilityVerified: false;
  annualPerformanceAvailable: false;
  bengCalculationAvailable: false;
  issues: Array<{ code: string; path: string }>;
}

export async function diagnoseForfaitHeatPumpDraftWithRust(input: ForfaitHeatPumpDraftInput): Promise<ForfaitHeatPumpDraftAssessment> {
  if (isTauri()) {
    return invoke<ForfaitHeatPumpDraftAssessment>('diagnose_forfait_heat_pump_draft', { input });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/heat-pumps/forfait-cop-draft/diagnose', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ input }),
    });
    return response.json() as Promise<ForfaitHeatPumpDraftAssessment>;
  }
  throw new Error('Rust draft heat-pump forfait diagnostics are available in the desktop app and local development server.');
}

export interface GasHeatPumpForfaitDraftInput {
  generatorId: string;
  drive: 'gas_engine' | 'absorption';
  application: 'residential_collective_at_most25_kw' | 'utility' | 'collective_building' | 'over25_kw';
  applicationReference: string;
  collectiveBuildingInstallation: boolean;
  externalHeatSupply: boolean;
  thermalCapacityKw: number;
  capacityReference: string;
  source: 'ground' | 'outdoor_air' | 'exhaust_air' | 'groundwater_aquifer' | 'surface_water';
  sourceReference: string;
  designSupplyTemperatureC: number;
  designSupplyReference: string;
  sourceCorrectionFactor?: number | null;
  sourceCorrectionReference?: string | null;
}

export interface GasHeatPumpForfaitDraftAssessment {
  status: 'diagnostic_valid' | 'invalid';
  scope: string;
  consultationSource: string;
  targetNormVersion: string;
  kernelVersion: string;
  inputFingerprint: string;
  table: '9.27' | '9.29';
  temperatureBandUpperC: number | null;
  forfaitCop: number | null;
  correctedCop: number | null;
  gasInputEnergyAvailable: false;
  auxiliaryEnergyAvailable: false;
  finalEditionVerified: false;
  bengCalculationAvailable: false;
  issues: Array<{ code: string; path: string }>;
}

export async function diagnoseGasHeatPumpForfaitDraftWithRust(input: GasHeatPumpForfaitDraftInput): Promise<GasHeatPumpForfaitDraftAssessment> {
  if (isTauri()) return invoke<GasHeatPumpForfaitDraftAssessment>('diagnose_gas_heat_pump_forfait_draft', { input });
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/heat-pumps/gas-forfait-cop-draft/diagnose', {
      method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ input }),
    });
    if (!response.ok) throw new Error(`Gas heat pump diagnostic: HTTP ${response.status}`);
    return response.json() as Promise<GasHeatPumpForfaitDraftAssessment>;
  }
  throw new Error('Rust gas heat-pump diagnostics are available in the desktop app and local development server.');
}

export interface GasHeatPumpAuxDraftInput {
  generatorId: string;
  drive: 'gas_engine' | 'absorption';
  nominalThermalCapacityKw: number;
  capacityReference: string;
  standbyElectronicsW: number;
  burnerAuxiliaryWPerKw: number;
  solutionPumpWPerKw: number;
  coefficientsReference: string;
  meanModulation: number;
  modulationReference: string;
  buildingShare: number;
  buildingShareReference: string;
  forfaitCopUsed: boolean;
  monthHoursReference: string;
  generatorOutputReference: string;
  months: Array<{ month: number; hours: number; generatorOutputKwh: number }>;
}

export interface GasHeatPumpAuxDraftAssessment {
  status: 'diagnostic_valid' | 'invalid';
  scope: string;
  draftSource: string;
  targetNormVersion: string;
  kernelVersion: string;
  inputFingerprint: string;
  monthly: Array<{ month: number; cappedOnHours: number; auxiliaryElectricityKwh: number }>;
  annualAuxiliaryElectricityKwh: number | null;
  gasInputEnergyAvailable: false;
  sourcePumpOrFanIncluded: false;
  finalEditionVerified: false;
  referenceVerified: false;
  bengCalculationAvailable: false;
  issues: Array<{ code: string; path: string }>;
}

export async function diagnoseGasHeatPumpAuxDraftWithRust(input: GasHeatPumpAuxDraftInput): Promise<GasHeatPumpAuxDraftAssessment> {
  if (isTauri()) return invoke<GasHeatPumpAuxDraftAssessment>('diagnose_gas_heat_pump_aux_draft', { input });
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/heat-pumps/gas-aux-draft/diagnose', {
      method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ input }),
    });
    const assessment = await response.json() as GasHeatPumpAuxDraftAssessment;
    if (!response.ok && assessment.status !== 'invalid') throw new Error(`Gas auxiliary diagnostic: HTTP ${response.status}`);
    return assessment;
  }
  throw new Error('Rust gas auxiliary diagnostics are available in the desktop app and local development server.');
}

export interface GasHeatPumpMonthlyDraftInput {
  forfait: GasHeatPumpForfaitDraftInput;
  generatorOutputKwh: Array<{ month: number; energyKwh: number }>;
  generatorOutputReference: string;
  sourceSystem: 'individual' | 'collective_ground' | 'collective_groundwater_surface_or_at_least15_c';
  sourceSystemReference: string;
}

export interface GasHeatPumpMonthlyDraftAssessment {
  status: 'diagnostic_valid' | 'invalid';
  scope: string;
  draftSource: string;
  targetNormVersion: string;
  kernelVersion: string;
  inputFingerprint: string;
  correctedCop: number | null;
  collectiveSourceCorrectionFactor: number | null;
  monthly: Array<{
    month: number;
    generatorOutputKwh: number;
    collectiveSourceHeatKwh: number;
    uncorrectedInputTermKwh: number;
    collectiveSourceCorrectionTermKwh: number;
    equation962InputTermKwh: number;
  }>;
  generatorOutputDerived: false;
  collectiveSourceHeatDerived: boolean;
  carrierAllocationAvailable: false;
  gasInputEnergyAvailable: false;
  auxiliariesIncluded: false;
  finalEditionVerified: false;
  referenceVerified: false;
  bengCalculationAvailable: false;
  issues: Array<{ code: string; path: string }>;
}

export async function diagnoseGasHeatPumpMonthlyDraftWithRust(input: GasHeatPumpMonthlyDraftInput): Promise<GasHeatPumpMonthlyDraftAssessment> {
  if (isTauri()) return invoke<GasHeatPumpMonthlyDraftAssessment>('diagnose_gas_heat_pump_monthly_draft', { input });
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/heat-pumps/gas-forfait-monthly-draft/diagnose', {
      method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ input }),
    });
    const assessment = await response.json() as GasHeatPumpMonthlyDraftAssessment;
    if (!response.ok && assessment.status !== 'invalid') throw new Error(`Gas monthly diagnostic: HTTP ${response.status}`);
    return assessment;
  }
  throw new Error('Rust gas monthly diagnostics are available in the desktop app and local development server.');
}

export interface GasHeatPumpChainDraftInput {
  monthly: GasHeatPumpMonthlyDraftInput;
  auxiliary: GasHeatPumpAuxDraftInput;
}

export interface GasHeatPumpChainDraftAssessment {
  status: 'diagnostic_valid' | 'invalid';
  scope: string;
  targetNormVersion: string;
  kernelVersion: string;
  inputFingerprint: string;
  monthly: Array<{
    month: number;
    suppliedGeneratorOutputKwh: number;
    collectiveSourceHeatKwh: number;
    equation962UnallocatedInputTermKwh: number;
    cappedAuxiliaryOnHours: number;
    equipmentAuxiliaryElectricityKwh: number;
  }>;
  annualEquipmentAuxiliaryElectricityKwh: number | null;
  generatorOutputDerived: false;
  carrierAllocationAvailable: false;
  gasInputEnergyAvailable: false;
  sourcePumpOrFanIncluded: false;
  finalEditionVerified: false;
  referenceVerified: false;
  bengCalculationAvailable: false;
  issues: Array<{ code: string; path: string }>;
}

export async function diagnoseGasHeatPumpChainDraftWithRust(input: GasHeatPumpChainDraftInput): Promise<GasHeatPumpChainDraftAssessment> {
  if (isTauri()) return invoke<GasHeatPumpChainDraftAssessment>('diagnose_gas_heat_pump_chain_draft', { input });
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/heat-pumps/gas-chain-draft/diagnose', {
      method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ input }),
    });
    const assessment = await response.json() as GasHeatPumpChainDraftAssessment;
    if (!response.ok && assessment.status !== 'invalid') throw new Error(`Gas chain diagnostic: HTTP ${response.status}`);
    return assessment;
  }
  throw new Error('Rust gas chain diagnostics are available in the desktop app and local development server.');
}

export type GasCollectiveSourceTemperatureClass = 'below20_c' | 'at_least20_c' | 'unknown';

export interface GasCollectiveSourceDraftInput {
  chain: GasHeatPumpChainDraftInput;
  sourceTemperatureClass: GasCollectiveSourceTemperatureClass;
  sourceTemperatureReference: string;
  noQualityDeclarationConfirmed: boolean;
  noQualityDeclarationReference: string;
}

export interface GasCollectiveSourceDraftAssessment {
  status: 'diagnostic_valid' | 'invalid';
  scope: string;
  inputFingerprint: string;
  sourceEnergyCarrier: 'dh';
  primaryFossilFactor: number | null;
  primaryRenewableFactor: number | null;
  factorBasis: string | null;
  monthly: Array<{ month: number; deliveredSourceHeatDhKwh: number; draftPrimaryFossilKwh: number; draftPrimaryRenewableKwh: number }>;
  annualDeliveredSourceHeatKwh: number | null;
  annualDraftPrimaryFossilKwh: number | null;
  annualDraftPrimaryRenewableKwh: number | null;
  gasInputEnergyAvailable: false;
  sourcePumpOrFanIncluded: false;
  finalEditionVerified: false;
  referenceVerified: false;
  bengCalculationAvailable: false;
  issues: Array<{ code: string; path: string }>;
}

export async function diagnoseGasCollectiveSourceDraftWithRust(input: GasCollectiveSourceDraftInput): Promise<GasCollectiveSourceDraftAssessment> {
  if (isTauri()) return invoke<GasCollectiveSourceDraftAssessment>('diagnose_gas_collective_source_draft', { input });
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/heat-pumps/gas-collective-source-draft/diagnose', {
      method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ input }),
    });
    const assessment = await response.json() as GasCollectiveSourceDraftAssessment;
    if (!response.ok && assessment.status !== 'invalid') throw new Error(`Gas collective source diagnostic: HTTP ${response.status}`);
    return assessment;
  }
  throw new Error('Rust gas collective source diagnostics are available in the desktop app and local development server.');
}

export type GasChainDiagnosticMetric = 'equation962_unallocated_input_term'
  | 'equipment_auxiliary_electricity' | 'annual_equipment_auxiliary_electricity';

export interface GasChainDiagnosticCase {
  caseId: string;
  normVersion: string;
  source: { publisher: string; documentId: string; edition: string; usePermission: string; independentReviewer: string };
  input: GasHeatPumpChainDraftInput;
  expected: Array<{
    metric: GasChainDiagnosticMetric;
    month: number | null;
    valueKwh: number;
    absoluteToleranceKwh: number;
    calculationBasis: string;
  }>;
}

export interface GasChainDiagnosticComparison {
  caseId: string;
  status: 'invalid_case' | 'compared_pass' | 'compared_fail';
  targetNormVersion: string;
  kernelVersion: string;
  inputFingerprint: string;
  caseFingerprint: string;
  referenceVerified: false;
  carrierAllocationAvailable: false;
  bengCalculationAvailable: false;
  allMetricsWithinTolerance: boolean;
  metrics: Array<{
    metric: GasChainDiagnosticMetric;
    month: number | null;
    expectedKwh: number;
    actualKwh: number;
    absoluteDifferenceKwh: number;
    absoluteToleranceKwh: number;
    withinTolerance: boolean;
  }>;
  issues: Array<{ code: string; path: string }>;
}

export async function compareGasHeatPumpChainDiagnosticWithRust(
  caseInput: GasChainDiagnosticCase,
): Promise<GasChainDiagnosticComparison> {
  if (isTauri()) return invoke<GasChainDiagnosticComparison>('compare_gas_heat_pump_chain_diagnostic', { case: caseInput });
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/reference/gas-chain-diagnostic/compare', {
      method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ case: caseInput }),
    });
    const body = await response.text();
    let parsed: unknown;
    try { parsed = JSON.parse(body); } catch { throw new Error(`Gas reference comparison: HTTP ${response.status}: ${body}`); }
    if (!parsed || typeof parsed !== 'object' || !('status' in parsed)) {
      const problem = parsed as { error?: string; message?: string } | null;
      throw new Error(problem?.message ?? problem?.error ?? `Gas reference comparison: HTTP ${response.status}`);
    }
    const assessment = parsed as GasChainDiagnosticComparison;
    if (!response.ok && assessment.status !== 'invalid_case') throw new Error(`Gas reference comparison: HTTP ${response.status}`);
    return assessment;
  }
  throw new Error('Rust gas reference comparisons are available in the desktop app and local development server.');
}

export interface ForfaitHeatPumpMonthlyDraftInput {
  forfait: ForfaitHeatPumpDraftInput;
  generatorOutputKwh: Array<{ month: number; energyKwh: number }>;
  generatorOutputReference: string;
  sourceSystem: 'individual' | 'collective_ground' | 'collective_groundwater_surface_or_at_least15_c';
  sourceSystemReference: string;
}

export interface ForfaitHeatPumpMonthlyDraftAssessment {
  status: 'diagnostic_valid' | 'invalid';
  scope: 'public_chapter_9_draft_equation_9_62_supplied_monthly_flows_only';
  consultationSource: string;
  targetNormVersion: string;
  kernelVersion: string;
  inputFingerprint: string;
  correctedCop: number | null;
  collectiveSourceCorrectionFactor: number | null;
  monthly: Array<{
    month: number;
    generatorOutputKwh: number;
    collectiveSourceHeatKwh: number;
    uncorrectedInputElectricityKwh: number;
    collectiveSourceCorrectionKwh: number;
    generatorInputElectricityKwh: number;
  }>;
  generatorOutputDerived: false;
  collectiveSourceHeatDerived: boolean;
  auxiliariesIncluded: false;
  annualPerformanceAvailable: false;
  bengCalculationAvailable: false;
  finalEditionVerified: false;
  issues: Array<{ code: string; path: string }>;
}

export async function diagnoseForfaitHeatPumpMonthlyDraftWithRust(
  input: ForfaitHeatPumpMonthlyDraftInput,
): Promise<ForfaitHeatPumpMonthlyDraftAssessment> {
  if (isTauri()) {
    return invoke<ForfaitHeatPumpMonthlyDraftAssessment>('diagnose_forfait_heat_pump_monthly_draft', { input });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/heat-pumps/forfait-monthly-draft/diagnose', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ input }),
    });
    return response.json() as Promise<ForfaitHeatPumpMonthlyDraftAssessment>;
  }
  throw new Error('Rust draft monthly heat-pump diagnostics are available in the desktop app and local development server.');
}

export interface GeneratorDispatchDraftInput {
  nodeInputKwh: Array<{ month: number; energyKwh: number }>;
  nodeInputReference: string;
  designContext: 'new_build';
  generators: Array<{
    id: string;
    class: 'exhaust_air_heat_pump_without_overventilation' | 'heat_pump' | 'biomass_boiler' | 'combined_heat_power' | 'other_boiler';
    classificationReference: string;
    nominalThermalPowerKw: number;
    powerReference: string;
    priorityEfficiency: number;
    efficiencyReference: string;
  }>;
}

export interface GeneratorDispatchDraftAssessment {
  status: 'diagnostic_valid' | 'invalid';
  scope: 'public_chapter_9_draft_new_build_installed_power_dispatch_only';
  consultationSource: string;
  targetNormVersion: string;
  kernelVersion: string;
  inputFingerprint: string;
  monthly: Array<{
    month: number;
    nodeInputKwh: number;
    generators: Array<{
      generatorId: string;
      betaCumulative: number;
      energyFraction: number;
      requiredHeatKwh: number;
      maximumHeatKwh: number;
      deliveredHeatKwh: number;
    }>;
    unallocatedHeatKwh: number;
  }>;
  nodeInputDerived: false;
  finalEditionVerified: false;
  bengCalculationAvailable: false;
  issues: Array<{ code: string; path: string }>;
}

export async function diagnoseGeneratorDispatchDraftWithRust(
  input: GeneratorDispatchDraftInput,
): Promise<GeneratorDispatchDraftAssessment> {
  if (isTauri()) {
    return invoke<GeneratorDispatchDraftAssessment>('diagnose_generator_dispatch_draft', { input });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/heating/generator-dispatch-draft/diagnose', {
      method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ input }),
    });
    return response.json() as Promise<GeneratorDispatchDraftAssessment>;
  }
  throw new Error('Rust draft generator dispatch is available in the desktop app and local development server.');
}

export interface BoilerForfaitDraftInput {
  generatorId: string;
  role: 'individual_main' | 'individual_supplementary' | 'collective';
  location: 'inside_thermal_boundary' | 'outside_thermal_boundary';
  kind: 'conventional' | 'vr' | 'hr100' | 'hr104' | 'hr107';
  fuel: 'natural_gas';
  averageDesignEmissionTemperatureC: number;
  emissionCircuit: 'direct' | 'mixing_with_return_limit' | 'mixing_without_return_limit';
  equipmentReference: string;
  locationReference: string;
  temperatureAndCircuitReference: string;
  pilotFlamePresent: boolean;
  installationYear?: number | null;
  installationYearReference?: string | null;
}

export interface BoilerForfaitDraftAssessment {
  status: 'diagnostic_valid' | 'invalid';
  inputFingerprint: string;
  temperatureClass: 'lt' | 'ht' | null;
  generationEfficiency: number | null;
  auxiliaryYearClass: 'before_2015_or_unknown' | 'from_2015' | null;
  issues: Array<{ code: string; path: string }>;
}

export interface BoilerForfaitMonthlyDraftAssessment {
  status: 'diagnostic_valid' | 'invalid';
  generationEfficiency: number | null;
  monthly: Array<{ month: number; generatorOutputKwh: number; inputNaturalGasKwh: number; auxiliaryElectricityKwh: number | null }>;
  auxiliaryYearClass: 'before_2015_or_unknown' | 'from_2015' | null;
  annualAuxiliaryElectricityKwh: number | null;
  issues: Array<{ code: string; path: string }>;
}

export async function diagnoseBoilerForfaitDraftWithRust(input: BoilerForfaitDraftInput): Promise<BoilerForfaitDraftAssessment> {
  if (isTauri()) return invoke<BoilerForfaitDraftAssessment>('diagnose_boiler_forfait_draft', { input });
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/boilers/forfait-draft/diagnose', {
      method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ input }),
    });
    return response.json() as Promise<BoilerForfaitDraftAssessment>;
  }
  throw new Error('Rust draft boiler diagnostics are available in the desktop app and local development server.');
}

export interface HybridHeatPumpMonthlyDraftInput {
  dispatch: GeneratorDispatchDraftInput;
  forfait: ForfaitHeatPumpDraftInput;
  boiler: BoilerForfaitDraftInput;
  sourceSystem: ForfaitHeatPumpMonthlyDraftInput['sourceSystem'];
  sourceSystemReference: string;
  declaredOperatingLimitsPresent: boolean;
  heatPumpAuxiliaryMeasurements?: Pick<HeatingAuxMeasuredDraftInput, 'generatorId' | 'generatorSourceReference' | 'measurements'> | null;
}

export interface HybridHeatPumpMonthlyDraftAssessment {
  status: 'diagnostic_valid' | 'invalid';
  scope: 'public_chapter_9_draft_new_build_dispatch_electric_heat_pump_gas_boiler_and_optional_measured_aux';
  kernelVersion: string;
  targetNormVersion: string;
  inputFingerprint: string;
  dispatch: GeneratorDispatchDraftAssessment | null;
  heatPump: ForfaitHeatPumpMonthlyDraftAssessment | null;
  boiler: BoilerForfaitMonthlyDraftAssessment | null;
  heatPumpAuxiliary: HeatingAuxMeasuredDraftAssessment | null;
  generatorOutputDerived: boolean;
  boilerInputEnergyAvailable: boolean;
  boilerAuxiliaryElectricityAvailable: boolean;
  heatPumpAuxiliaryElectricityAvailable: boolean;
  bengCalculationAvailable: false;
  finalEditionVerified: false;
  issues: Array<{ code: string; path: string }>;
}

export async function diagnoseHybridHeatPumpMonthlyDraftWithRust(
  input: HybridHeatPumpMonthlyDraftInput,
): Promise<HybridHeatPumpMonthlyDraftAssessment> {
  if (isTauri()) {
    return invoke<HybridHeatPumpMonthlyDraftAssessment>('diagnose_hybrid_heat_pump_monthly_draft', { input });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/heat-pumps/hybrid-monthly-draft/diagnose', {
      method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ input }),
    });
    return response.json() as Promise<HybridHeatPumpMonthlyDraftAssessment>;
  }
  throw new Error('Rust draft hybrid heat-pump diagnostics are available in the desktop app and local development server.');
}

export interface HeatingAuxMeasuredDraftAssessment {
  status: 'input_valid' | 'invalid';
  scope: 'public_chapter_9_draft_9_86_9_88_measured_electric_heat_pump_only';
  draftSource: string;
  targetNormVersion: string;
  kernelVersion: string;
  inputFingerprint: string;
  finalEditionVerified: false;
  referenceVerified: false;
  bengCalculationAvailable: false;
  primaryElectricityFactorUsed: 1.45;
  usefulPumpFractionUsed: 0.5;
  derivedCoefficients: HeatingAuxDraftInput['coefficients'] | null;
  auxiliary: HeatingAuxDraftAssessment | null;
  issues: Array<{ code: string; path: string }>;
}

export async function diagnoseHeatingAuxMeasuredDraftWithRust(input: HeatingAuxMeasuredDraftInput): Promise<HeatingAuxMeasuredDraftAssessment> {
  if (isTauri()) {
    return invoke<HeatingAuxMeasuredDraftAssessment>('diagnose_heating_aux_measured_draft', { input });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/heat-pumps/heating-aux-measured-draft/diagnose', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ input }),
    });
    return response.json() as Promise<HeatingAuxMeasuredDraftAssessment>;
  }
  throw new Error('Rust draft heat-pump measurement diagnostics are available in the desktop app and local development server.');
}

export async function assessProjectWithRust(project: IProject): Promise<KernelAssessment> {
  if (isTauri()) {
    return invoke<KernelAssessment>('validate_nta_project', { project });
  }

  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/validate', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ project }),
    });
    if (!response.ok) {
      const body: unknown = await response.json().catch(() => null);
      const detail = body && typeof body === 'object' && 'message' in body
        && typeof body.message === 'string' ? `: ${body.message}` : '';
      throw new Error(`Rust API: HTTP ${response.status}${detail}`);
    }
    return response.json() as Promise<KernelAssessment>;
  }

  throw new Error('Rust validation is available in the desktop app and local development server.');
}

export interface SpaceHeatingChainZone {
  demand: MonthlyDemandInput;
  emission: SpaceHeatingChainInput['emission'];
  distribution: SpaceHeatingChainInput['distribution'];
}

export interface SpaceHeatingChainInput {
  humidifiers?: NtaZoneHumidifier[];
  /** 9.2.3.4 Q_H;ren;prac of solar combi systems per month, kWh. */
  solarHeatingKwh?: number[];
  demand: MonthlyDemandInput;
  additionalZones?: SpaceHeatingChainZone[];
  emission: {
    system: 'radiators_or_convectors' | 'floor_heating' | 'fan_assisted_radiators_or_convectors' | 'air_heating' | 'local_heater' | 'other_or_unknown';
    balancing: 'none_or_unknown' | 'static' | 'dynamic' | 'not_applicable';
    control: 'main_room_thermostat' | 'central_with_room_valves' | 'individual_room_thermostats' | 'other_or_unknown';
    sourceReference: string;
    /** Room fans (9.21/9.22, table 9.11); required for fan-assisted emitters. */
    fans?: {
      kind: 'fan_convector' | 'electric_heating' | 'dynamic_storage' | 'unknown';
      count: number;
      /** NEN-EN 16430 tested power per fan, W. */
      testedPowerW?: number;
      sourceReference: string;
    };
  };
  distribution:
    | { method: 'heated_zone_only_space_heating'; sourceReference: string }
    | { method: 'declared'; monthlyLossKwh: number[]; sourceReference: string }
    | { method: 'calculated'; heatingLimitExtraKwh?: number[] | null; sourceReference: string };
  /** §9.4 hydraulic data: calculated distribution loss and pump energy. */
  distributionSystem?: NtaDistributionSystem | null;
  /** Part of a building on a collective installation (`f_gebouw;si;H`). */
  collectiveConnection?: { connectedUsableAreaM2: number; sourceReference: string } | null;
  generator:
    | {
        kind: 'gas_boiler';
        boiler: BoilerForfaitDraftInput;
        auxiliary?: NtaOtherGeneratorAuxiliary | null;
        /** Annex O component measurements for the 9.85 constants (individual boilers). */
        auxiliaryMeasurements?: NtaAppliancePowerMeasurements | null;
      }
    | {
        kind: 'heat_pump_forfait';
        forfait: ForfaitHeatPumpDraftInput;
        sourceSystem: 'individual' | 'collective_ground' | 'collective_groundwater_surface_or_at_least15_c';
        sourceSystemReference: string;
        auxiliaryMeasurements?: Record<string, unknown> | null;
        auxiliary?: NtaOtherGeneratorAuxiliary | null;
      }
    | {
        kind: 'hybrid_heat_pump';
        designContext: 'new_build';
        generators: Array<{
          id: string;
          class: 'heat_pump' | 'exhaust_air_heat_pump_without_overventilation' | 'other_boiler';
          classificationReference: string;
          nominalThermalPowerKw: number;
          powerReference: string;
          priorityEfficiency: number;
          efficiencyReference: string;
        }>;
        forfait: ForfaitHeatPumpDraftInput;
        boiler: BoilerForfaitDraftInput;
        sourceSystem: 'individual' | 'collective_ground' | 'collective_groundwater_surface_or_at_least15_c';
        sourceSystemReference: string;
        declaredOperatingLimitsPresent?: boolean;
      }
    | {
        kind: 'external_heat';
        supplierReference: string;
        qualityDeclarationPresent: boolean;
        auxiliary?: NtaOtherGeneratorAuxiliary | null;
      }
    | { kind: 'electric_resistance'; equipmentReference: string; auxiliary?: NtaOtherGeneratorAuxiliary | null }
    | {
        kind: 'heat_pump_annex_q';
        heatPump: NtaAnnexQHeatPump;
        /** θ_sup for tables Q.5/Q.7, up to 75 °C. */
        designSupplyTemperatureC: number;
        /** Required when F_H;gen < 1. */
        backup?: { kind: 'electric_resistance' } | { kind: 'gas_boiler'; boiler: BoilerForfaitDraftInput } | null;
        regeneration?: NtaRegenerationInput | null;
        equipmentReference: string;
      }
    | {
        kind: 'biomass';
        appliance: 'freestanding_wood_stove' | 'insert_stove' | 'pellet_stove' | 'accumulating_stove' | 'central_boiler';
        location: 'inside_thermal_boundary' | 'outside_thermal_boundary';
        annexRCompliantAtMost500Kw: boolean;
        annexRReference: string;
        equipmentReference: string;
        soleHeatingInServedRooms?: boolean | null;
        automaticFuelFeed?: boolean;
        auxiliary?: NtaOtherGeneratorAuxiliary | null;
      }
    | {
        /** Annex M: boiler with product values. */
        kind: 'product_boiler';
        boiler: NtaProductBoiler;
        designTemperatureClass?: NtaDesignTemperatureClass | null;
        annexRCompliantAtMost500Kw?: boolean | null;
        annexRReference?: string | null;
      }
    | {
        /** Annex N: local, air or radiant heater or stove. */
        kind: 'local_heater';
        heater: NtaLocalHeater;
        fuel: 'natural_gas' | 'oil' | 'biomass';
        annexRCompliantAtMost500Kw?: boolean | null;
        annexRReference?: string | null;
        soleHeatingInServedRooms?: boolean | null;
      }
    | {
        /** Table 9.25 "overige systemen". */
        kind: 'forfait_heater';
        heaterKind:
          | 'local_with_flue' | 'local_without_flue'
          | 'air_heater_conventional' | 'air_heater_vr' | 'air_heater_hr100' | 'air_heater_hr104' | 'air_heater_hr107';
        fuel: 'natural_gas' | 'oil';
        equipmentReference: string;
        /** Pilot flames of gas air heaters (695 kWh/year each, §9.6.2.1). */
        pilotFlames?: number;
        auxiliary?: NtaOtherGeneratorAuxiliary | null;
      }
    | {
        /** 9.6.1: several unequal generators split by preference (9.56–9.60, table 9.23). */
        kind: 'multiple';
        generators: Array<{
          preference: number;
          nominalPowerKw: number;
          /** Any single generator of this union except `multiple` and `hybrid_heat_pump`. */
          generator: { kind: string } & Record<string, unknown>;
        }>;
        /** 9.58/9.59: renovation with an added preferred generator. */
        addedPreferredGenerator?: boolean;
        sourceReference: string;
      };
}

/** Annex M product values (efficiencies as fractions, auxiliary powers in W). */
export interface NtaProductBoiler {
  technology: 'solid_fuel_standard' | 'gas_oil_standard' | 'low_temperature' | 'condensing_gas' | 'condensing_oil';
  fuel: 'natural_gas' | 'oil' | 'wood';
  placement: 'outdoors' | 'installation_room' | 'under_roof' | 'heated_space';
  draught: 'atmospheric' | 'fan_assisted';
  control: 'floor_standing_outdoor_compensated' | 'wall_hung_outdoor_compensated' | 'wall_hung_room_temperature';
  product: {
    nominalPowerKw: number;
    intermediatePowerKw?: number | null;
    fullLoad:
      | {
          method: 'single';
          efficiency: number;
          testTemperatureC?: number | null;
          additionalTest?: { efficiency: number; testTemperatureC: number } | null;
        }
      | { method: 'condensing'; efficiencyAt60: number; efficiencyAt30: number };
    partLoadEfficiency: number;
    partLoadTestTemperatureC?: number | null;
    partLoadAdditionalTest?: { efficiency: number; testTemperatureC: number } | null;
    standbyLossFactor: number;
    standbyTestTemperatureC: number;
    auxiliaryStandbyW: number;
    auxiliaryIntermediateW: number;
    auxiliaryFullW: number;
    sourceReference: string;
  };
  equipmentReference: string;
}

/** Annex N heater; omitted product values take the N.6 defaults where they exist. */
export interface NtaLocalHeater {
  heaterType:
    | 'high_temperature_radiant' | 'radiant_tube_without_flue' | 'radiant_tube_with_flue'
    | 'air_heater_atmospheric' | 'air_heater_fan_burner' | 'air_heater_modulating_combustion_air'
    | 'air_heater_modulating_no_combustion_air' | 'air_heater_modulating_evaporative'
    | 'condensing_air_heater' | 'stove';
  control: 'on_off' | 'high_low' | 'modulating';
  productionPeriod: 'after2005' | 'from1990_to2005' | 'before1990';
  condensing: boolean;
  pilotFlame: boolean;
  ventilation: 'required' | 'interlocked' | 'none';
  location:
    | 'heated_space_free' | 'heated_space_against_wall_or_roof' | 'boiler_room'
    | 'under_roof_outside_heated_space' | 'outdoors';
  fan?: 'centrifugal' | 'axial' | null;
  envelopeInsulation?:
    | 'well_insulated_new_high_efficiency' | 'well_insulated_maintained' | 'old_average' | 'old_poor' | 'none'
    | null;
  stoveKind?: 'solid_fuel_room_heater' | 'inset_or_open_fire' | 'pellet' | 'accumulating' | 'gas_or_oil' | null;
  waterConnection?: { outputToAirKw: number; outputToWaterKw: number } | null;
  roomHeightM?: number | null;
  product: Partial<{
    inputFullKw: number;
    outputFullKw: number;
    combustionEfficiencyPercent: number;
    chimneyLossPercent: number;
    chimneyCorrectionFactor: number;
    testAirTemperatureC: number;
    loadExponent: number;
    auxBurnerKw: number;
    auxAfterBurnerKw: number;
    auxStandbyKw: number;
    envelopeLossPercent: number;
    pilotLossPercent: number;
    inputMinKw: number;
    chimneyLossMinPercent: number;
    combustionEfficiencyMinPercent: number;
    auxBurnerMinKw: number;
    auxAfterBurnerMinKw: number;
  }>;
  sourceReference: string;
}

/** One NEN-EN 14511 / 14825 measurement (annex Q tables Q.11/Q.15). */
export interface NtaAnnexQPoint {
  evaporatorInC: number;
  evaporatorOutC: number;
  condenserInC: number;
  condenserOutC: number;
  cop: number;
  heatingPowerKw: number;
}

/** Annex Q heat pump product data. */
export interface NtaAnnexQHeatPump {
  source: 'brine_water' | 'water_water' | 'outdoor_air_water' | 'exhaust_air_water' | 'combined_air_water' | 'air_air';
  /** f_buitenlucht (11.24) for combined_air_water. */
  outdoorAirFraction?: number | null;
  maximumPower: {
    condition1: NtaAnnexQPoint;
    condition2?: NtaAnnexQPoint | null;
    condition3?: NtaAnnexQPoint | null;
    condition4?: NtaAnnexQPoint | null;
  };
  modulation:
    | { method: 'on_off' }
    | {
        method: 'modulating';
        minimumPowerKw: number;
        lowRange: NtaAnnexQPoint[];
        highRange?: NtaAnnexQPoint[] | null;
        condenserPumpModulating: boolean;
        sourcePumpModulating: boolean;
      };
  switchOff?: {
    minEvaporatorInC?: number | null;
    minEvaporatorOutC?: number | null;
    maxCondenserInC?: number | null;
    maxCondenserOutC?: number | null;
    minCop?: number | null;
  };
  sourcePump?: { nominalPowerW?: number | null; overrunS: number } | null;
  evaporatorInlet?: { method: 'forfait' } | { method: 'declared'; temperaturesC: number[]; sourceReference: string } | null;
  testReportReference: string;
}

/** Annex V regeneration of an individual ground source. */
export interface NtaRegenerationInput {
  freeCoolingFromSource?: boolean;
  solar?: Array<{
    collectorAreaM2: number;
    azimuthDeg: number;
    tiltDeg: number;
    obstructionFactors?: number[];
    declaredEfficiency?: number | null;
    sourceReference: string;
  }>;
  sourceReference: string;
}

/** Annex O component measurements (powers in W, times in s). */
export interface NtaAppliancePowerMeasurements {
  nominalLoadKw: number;
  standbyElectronicsW: number;
  gasValveW?: number;
  fan:
    | { method: 'none' }
    | { method: 'single_speed'; powerW: number }
    | { method: 'modulating'; points: Array<{ modulation: number; powerW: number }> };
  pump:
    | { method: 'none' }
    | { method: 'staged'; operationW: number; prePostRunW: number }
    | {
        method: 'modulating';
        points: Array<{ modulation: number; powerW: number }>;
        prePostRunModulation: number;
      };
  pumpPreRunS?: number;
  pumpPostRunS?: number;
  fanPreRunS?: number;
  fanPostRunS?: number;
  loadCurve?: Array<{ timeS: number; meanLoad: number; meanPumpModulation?: number | null }>;
  bivalent?: boolean;
  bivalentMeanLoad?: number | null;
  sourceReference: string;
}

/** Annex W booster heat pump. */
export interface NtaBoosterHeatPump {
  lowTest: { sourceTemperatureC: number; cop: number };
  highTest: { sourceTemperatureC: number; cop: number };
  measuredClass: 'class1' | 'class2' | 'class3' | 'class4';
  standingLossKw: number;
  sourceTemperaturesC: number[];
  coolingExtractionKwh?: number[] | null;
  heatSource:
    | { kind: 'external_heat' }
    | { kind: 'collective_generator'; generationEfficiency: number; carrier: 'gas' | 'oil' | 'electricity'; sourceReference: string };
  testReportReference: string;
}

/** 9.91/9.92 inputs for generators outside 9.85. */
export interface NtaOtherGeneratorAuxiliary {
  electricallyConnectedDevices: number;
  nominalPowerKw?: number | null;
  sourceReference: string;
}

export type NtaDesignTemperatureClass =
  | '30_27' | '35_30' | '40_35' | '45_40' | '50_42' | '55_47'
  | '60_50' | '65_55' | '70_60' | '75_65' | '80_60' | '90_70';

export type NtaPipeTransmittance =
  | {
      method: 'forfait';
      insulation:
        | { state: 'insulated'; period: 'from1995' | 'from1980_to1995' | 'before1980_or_unknown' }
        | { state: 'uninsulated' }
        | { state: 'unknown' };
    }
  | {
      method: 'insulated_in_air';
      pipeOuterDiameterM: number;
      insulatedDiameterM: number;
      insulationLambdaWPerMK: number;
      surfaceCoefficientWPerM2K?: number | null;
    }
  | {
      method: 'insulated_embedded';
      pipeOuterDiameterM: number;
      insulatedDiameterM: number;
      insulationLambdaWPerMK: number;
      embeddingLambdaWPerMK: number;
      depthM: number;
    }
  | {
      method: 'uninsulated';
      innerDiameterM: number;
      outerDiameterM: number;
      pipeLambdaWPerMK: number;
      surfaceCoefficientWPerM2K?: number | null;
    };

export interface NtaDistributionSystem {
  designTemperatureClass?: NtaDesignTemperatureClass | null;
  installation: 'individual' | 'collective';
  usageFunction:
    | 'residential' | 'assembly' | 'cell' | 'healthcare_with_beds' | 'healthcare_other'
    | 'office' | 'lodging' | 'education' | 'sport' | 'retail';
  pipesAlsoForHotWater?: boolean;
  collectiveHotWaterDeliverySet?: boolean;
  connectedStoreys: number;
  pipeTransmittance: NtaPipeTransmittance;
  unheatedPipeTransmittance?: NtaPipeTransmittance | null;
  valvesInsulated: boolean;
  actualPipeLengthM?: number | null;
  unheatedPipeLengthM?: number | null;
  unheatedAmbientC?: number[] | null;
  bufferVessel?: {
    volumeL: number;
    standingLossW?: number | null;
    label?: 'a_plus' | 'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | null;
    producedFrom2018: boolean;
    inHeatedSpace: boolean;
    constantTemperature?: boolean;
    sourceReference: string;
  } | null;
  pump:
    | { method: 'included_in_generator_auxiliary' }
    | { method: 'none_on_site'; sourceReference: string }
    | {
        method: 'calculated';
        heatMeterPresent: boolean;
        maxPipeLengthM?: number | null;
        designFlowM3PerH?: number | null;
        energyEfficiencyIndex?: number | null;
        electricPowerKw?: number | null;
        sourceReference: string;
      };
  sourceReference: string;
}

export interface SpaceHeatingChainAssessment {
  status: 'calculated_unverified' | 'invalid';
  scope: string;
  chapter9Source: string;
  inputFingerprint: string;
  finalEditionVerified: false;
  referenceVerified: false;
  bengCalculationAvailable: false;
  omittedTerms: string[];
  emissionTemperatureIncrementK: number | null;
  generationEfficiency: number | null;
  monthly: Array<{
    month: number;
    heatingNeedKwh: number;
    emissionLossKwh: number;
    emissionInputKwh: number;
    distributionLossKwh: number;
    distributionAuxiliaryToMediumKwh: number;
    nodeLossKwh: number;
    /** 9.2.3.4 node gain of solar combi systems. */
    solarGainKwh: number;
    generatorOutputKwh: number;
    heatPumpOutputKwh: number;
    naturalGasKwh: number;
    districtHeatKwh: number;
    biomassKwh: number;
    oilKwh: number;
    generatorRecoverableLossKwh: number;
    generatorElectricityKwh: number;
    auxiliaryElectricityKwh: number | null;
    distributionAuxiliaryElectricityKwh: number;
    recoverableLossKwh: number;
    collectiveSourceHeatKwh: number;
  }>;
  annualNaturalGasKwh: number | null;
  annualGeneratorElectricityKwh: number | null;
  annualAuxiliaryElectricityKwh: number | null;
  annualCollectiveSourceHeatKwh: number | null;
  annualDistrictHeatKwh: number | null;
  annualBiomassKwh: number | null;
  distribution: {
    designTemperatureClass: NtaDesignTemperatureClass;
    buildingFraction: number;
    pipeLengthM: number;
    unheatedPipeLengthM: number;
    psiZoneWPerMk: number;
    psiUnheatedWPerMk: number;
    zones: Array<{
      zoneId: string;
      heatingLimitC: number;
      operatingHours: number[];
      meanMediumTemperatureC: number[];
    }>;
    pump: {
      maxPipeLengthM: number;
      pressureKpa: number;
      designFlowM3PerH: number;
      hydraulicPowerKw: number;
      energyFactor: number;
      balancingFactor: number;
    } | null;
  } | null;
  zoneRecoverableLosses: Array<{ zoneId: string; monthlyKwh: number[] }>;
  /** Annex Q (and annex V) details of an annex Q heat pump. */
  annexQ: {
    annexQ: {
      energyFraction: number;
      generationEfficiency: number;
      deliveredKwh: number;
      electricityKwh: number;
      sourcePumpKwh: number;
      bins: Array<{
        outdoorC: number;
        hours: number;
        demandKw: number;
        maximumPowerKw: number;
        deliveredKw: number;
        cop: number;
        switchOffFactor: number;
        onHours: number;
        onFraction: number;
      }>;
      monthlyOnFraction: number[];
      interpretations: string[];
    };
    demandClass: 'residential_low' | 'residential_high' | 'utility_low' | 'utility_high';
    regenerationDegree: number | null;
    /** Always 1: 9.63 (method 1) has no c_source. */
    sourceCorrection: number;
    /** Q.5.3 f_H;t;hp-on;mi(3) used for chapter 11, when derived by the chain. */
    exhaustAirHeatingTimeFraction?: number[] | null;
    /** 9.63 f_prac (0,95). */
    practiceFactor: number;
    /** COP · f_prac. */
    correctedEfficiency: number;
  } | null;
  demand: MonthlyDemandAssessment;
  additionalZoneDemands: MonthlyDemandAssessment[];
  issues: Array<{ code: string; path: string }>;
}

export type HeatFlowDirection = 'upward' | 'horizontal' | 'downward';
export type FrameGroup = 'wood_or_plastic' | 'metal_with_thermal_break' | 'metal_without_thermal_break';

/** Design conductivity routes of NTA 8800 annex E/H; see crates/nta8800-core/src/materials.rs. */
export type MaterialConductivity =
  | { method: 'calculated'; lambdaCalc: number; sourceReference: string }
  | {
      method: 'declared_insulation';
      lambdaDeclared: number;
      moisture: string;
      ageing: { kind: 'factory_made' } | { kind: 'in_situ'; product: string; situation: 'a' | 'b'; practiceTested?: boolean };
      temperature?: { meanTemperatureC: number; conversionCoefficient: number; sourceReference: string };
      convectionFactor?: number;
      sourceReference: string;
    }
  | {
      method: 'forfait_insulation';
      material: string;
      moisture: string;
      ageing: { kind: 'factory_made' } | { kind: 'in_situ'; product: string; situation: 'a' | 'b'; practiceTested?: boolean };
    }
  | {
      method: 'masonry_table';
      kind: 'brick' | 'concrete' | 'calcium_silicate' | 'aerated_concrete';
      densityKgM3: number;
      environment: 'dry_indoor' | 'other';
      joints: 'glued' | 'mortar';
    }
  | {
      method: 'masonry_declared';
      kind: 'brick' | 'concrete' | 'calcium_silicate' | 'aerated_concrete';
      unitLambdaDeclared: number;
      mortarLambdaDeclared: number;
      jointAreaFraction: number;
      environment: 'dry_indoor' | 'other';
      sourceReference: string;
    }
  | { method: 'declared_other'; lambdaDeclared: number; class: 'inorganic' | 'glass' | 'organic' | 'plastic'; densityKgM3: number; sourceReference: string }
  | { method: 'window_material'; material: string };

export type ConstructionLayer =
  | { kind: 'material'; thicknessM: number; conductivity: MaterialConductivity }
  | { kind: 'resistance'; resistanceM2KPerW: number; thicknessM?: number; sourceReference: string }
  | {
      kind: 'declared_resistance';
      resistanceDeclared: number;
      moisture: string;
      ageing: { kind: 'factory_made' } | { kind: 'in_situ'; product: string; situation: 'a' | 'b'; practiceTested?: boolean };
      temperature?: { meanTemperatureC: number; conversionCoefficient: number; sourceReference: string };
      convectionFactor?: number;
      thicknessM?: number;
      sourceReference: string;
    }
  | { kind: 'reflective_foil'; system: { kind: 'foil_layers'; thicknessM: number } | { kind: 'facing' | 'two_foils_with_air_layer' | 'three_foils_with_air_layers' } }
  | {
      kind: 'air_cavity';
      thicknessMm: number;
      ventilation: { kind: 'unventilated' } | { kind: 'weakly'; openingMm2?: number } | { kind: 'strongly' };
      reflectiveSurface?: boolean;
      /** Reflective layer facing up: no bracket value unless hermetically sealed (table C.4 note b). */
      reflectiveFacingUp?: boolean;
      hermeticallySealed?: boolean;
    }
  | { kind: 'narrow_cavity'; thicknessMm: number; widthMm: number }
  | { kind: 'tubular_cavity'; thicknessMm: number; widthMm: number; orientation: 'horizontal' | 'vertical' }
  | { kind: 'attic'; roof: 'tiles_without_underlay' | 'tiles_or_slates_with_underlay' | 'tiles_with_underlay_and_reflective_foil' | 'boarding_and_felt' }
  | {
      kind: 'ventilated_unheated_space';
      separationAreaM2: number;
      envelopeParts: Array<{ areaM2: number; uValueWPerM2k?: number }>;
      airChangeRate?: number;
      volumeM3: number;
    };

export interface OpaqueConstructionInput {
  heatFlow: HeatFlowDirection;
  exteriorAir?: boolean;
  build:
    | { kind: 'homogeneous'; layers: ConstructionLayer[] }
    | {
        kind: 'composite';
        sections: Array<{ id: string; area: number; layers: ConstructionLayer[] }>;
        interruption: 'stony_unshielded' | 'woody_unshielded' | 'metal_one_side_shielded' | 'other';
        /** Section id for R_1/R_T of 8.9/8.11/8.13; default: highest C.3 R_T. */
        insulationSection?: string;
      };
  corrections?: {
    airVoids?: { level: 'none' | 'weak' | 'strong'; insulationLayer: number };
    fasteners?: {
      fasteners:
        | { method: 'point_bridge'; countPerM2: number; chiWPerK: number; sourceReference: string }
        | {
            method: 'formula';
            countPerM2: number;
            lambdaWPerMK: number;
            crossSectionM2: number;
            penetrationDepthM: number;
            insulationThicknessM: number;
          };
      insulationLayer: number;
    };
    invertedRoof?: {
      drainage: 'xps_green_roof' | 'xps_rebated_edges' | 'xps_straight_edges' | 'xps_with_waterproof_vapour_open_layer' | 'other_insulation';
      insulationLayer: number;
    };
  };
  unheatedReductionFactor?: number;
}

/** Element kinds of crates/nta8800-core/src/envelope_elements.rs; window and forfait shapes follow window_u.rs and forfait_envelope.rs. */
export type EnvelopeElementKind =
  | { kind: 'opaque'; construction: OpaqueConstructionInput }
  | { kind: 'tapered_roof'; roof: Record<string, unknown> }
  | { kind: 'window'; window: { method: Record<string, unknown>; shutter?: Record<string, unknown> } }
  | { kind: 'forfait_opaque'; element: Record<string, unknown> }
  | { kind: 'forfait_window'; glass: string; frame: FrameGroup; exterior: boolean }
  | { kind: 'forfait_door'; insulated: boolean; exterior: boolean; glassFraction?: number; glass?: string; frame?: FrameGroup }
  | { kind: 'forfait_panel'; insulation: Record<string, unknown>; cavity: boolean; frame: FrameGroup; exterior: boolean }
  | { kind: 'rooflight'; uRcWPerM2K: number; areaWithUpstandM2: number; sourceReference: string }
  | { kind: 'ventilation_grille' }
  | { kind: 'numerical'; couplingWPerK: number; constructionAreaM2: number; deltaUWPerM2K?: number; sourceReference: string };

export interface EnvelopeInput {
  elements: Array<{ id: string; projectedAreaM2?: number; inForfaitSupplement?: boolean; element: EnvelopeElementKind }>;
  forfaitBridges?: Array<{
    id: string;
    position?: number;
    variant?: number;
    column: 'a' | 'b';
    lengthM: number;
    shared?: boolean;
    description: string;
  }>;
  forfaitSupplement?: boolean;
}

export interface EnvelopeAssessment {
  status: 'calculated_unverified' | 'invalid';
  scope: string;
  issues: Array<{ code: string; path: string }>;
  elements: Array<{
    id: string;
    route: string;
    /** Rounded per 8.2.2.1; the value used in H_D and ΔU_for. */
    uValue: number;
    uRounded: number;
    uUnrounded: number;
    rC: number | null;
    rCRounded: number | null;
    opaque: Record<string, number | null> | null;
    window: { uW: number; uWShut: number | null; uEffective: number; uRounded: number } | null;
    forfait: { rC: number; uC: number; route: string } | null;
  }>;
  bridges: Array<{ id: string; psiWPerMk: number; coefficientWPerK: number }>;
  deltaUForfait: number | null;
  interpretations: string[];
  referenceVerified: false;
}

export async function calculateConstructionsWithRust(input: EnvelopeInput): Promise<EnvelopeAssessment> {
  if (isTauri()) {
    return invoke<EnvelopeAssessment>('calculate_constructions', { input });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/constructions/calculate', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ input }),
    });
    return response.json() as Promise<EnvelopeAssessment>;
  }
  throw new Error('Rust construction calculation is available in the desktop app and local development server.');
}

export type VentilationSystemVariant =
  | 'a1' | 'a2a' | 'a2b' | 'a2c' | 'b1' | 'b2' | 'b3'
  | 'c1' | 'c2a' | 'c2b' | 'c2c' | 'c3a' | 'c3b' | 'c3c' | 'c4a' | 'c4b' | 'c4c' | 'c5a' | 'c5b'
  | 'd1' | 'd2' | 'd3' | 'd4a' | 'd4b' | 'd5a' | 'd5b' | 'd5c';

export type VentilationFunction =
  | 'residential' | 'assembly_child_care' | 'other_assembly' | 'cell' | 'healthcare_bed_area'
  | 'other_healthcare' | 'office' | 'lodging_building' | 'education' | 'sport' | 'retail';

export interface VentilationSystemUnit {
  variant: VentilationSystemVariant;
  heatRecovery?: {
    efficiency:
      | { method: 'declared'; value: number; standard: 'en13141_7' | 'en13141_8' | 'en13142' | 'en13053'; sourceReference: string }
      | { method: 'table'; exchanger: string };
    bypass:
      | { kind: 'none' }
      | { kind: 'full'; coldRecoveryEvidence?: string }
      | { kind: 'partial'; fraction: number }
      | { kind: 'unknown'; bypassPresent: boolean };
    layout: 'central' | 'decentral';
    constantVolumeControl?: boolean;
    supplyDuctLengthM?: number;
    supplyDuctInsulation:
      | { kind: 'uninsulated' | 'insulated' | 'unknown' }
      | { kind: 'specified'; thicknessM: number; conductivityWPerMK: number };
    manufactureYear?: number;
    equipmentReference: string;
  };
  ducts: 'unknown' | 'luka_a_b_c' | 'luka_d' | 'no_ducts';
  airHandlingUnit?: {
    insideThermalZone: boolean;
    supplyDuctsOutside: 'none' | 'situation1' | 'situation2' | 'situation3';
    /** Reheating coil (11.118–11.121, table 11.15). */
    heatingCoil?: boolean;
    /** Cooling coil (11.114–11.117, table 11.15). */
    coolingCoil?: boolean;
  };
  equipmentReference: string;
}

/** Chapter 11 input for one zone; see crates/nta8800-core/src/ventilation.rs. */
export interface VentilationInput {
  zoneId: string;
  usableFloorAreaM2: number;
  category: 'residential' | 'utility';
  functions: Array<{ function: VentilationFunction; areaM2: number; swimmingPool?: boolean }>;
  dwellingCount?: number;
  wholeDwellingAreaM2?: number;
  apartmentBuilding?: boolean;
  buildingHeightM: number;
  constructionYear: number;
  floorAboveCrawlspace?: boolean;
  heatingSetpointC: number;
  coolingSetpointC: number;
  system:
    | { kind: 'single'; unit: VentilationSystemUnit }
    | {
        kind: 'combined';
        decentralAreaM2: number;
        totalResidenceAreaM2: number;
        decentral: VentilationSystemUnit;
        other: VentilationSystemUnit;
      };
  maximumCapacityForCooling?: string;
  installedCapacity?: { totalDm3PerS: number; naturalSupplyDm3PerS?: number; sourceReference: string };
  flowReduction?: { collective?: boolean; recirculationPercent?: number; flowControlPercent?: number };
  infiltration:
    | { method: 'measured'; qv10DmPerSM2: number; sourceReference: string }
    | { method: 'reference'; buildingType: string; renovationYear?: number };
  combustionAppliances?: Array<{
    id: string;
    kind: string;
    class: 'kitchen_stove' | 'gas_type_a' | 'open_fireplace' | 'gas_type_b' | 'specific_gas_appliance' | 'room_sealed';
    nominalInputKw?: number;
    sourceReference: string;
  }>;
  overventilation?: {
    /** Q.5.3 f_H;t;hp-on per month; leave out to let an annex Q heat pump chain derive it. */
    heatingTimeFraction?: number[];
    hotWaterTimeFraction: number[];
    heatingFlowM3PerH?: number;
    heatingAreaShare?: number;
    hotWaterFlowM3PerH: number[];
    sourceReference: string;
  };
  ventilativeCooling?: {
    openings: Array<{
      id: string;
      area:
        | { method: 'declared'; netAreaM2: number }
        | { method: 'discharge'; grossAreaM2: number; dischargeCoefficient: number; entryLossCoefficient: number }
        | { method: 'opening_angle'; maxNetAreaM2: number; maxAngleDeg: number };
      centreHeightM: number;
      openingHeightM: number;
      azimuthDeg: number;
      tiltDeg: number;
    }>;
    operation: 'manual' | 'automatic' | 'automatic_with_temperature';
    conditionsEvidence: string;
  };
  grillePreheating?: {
    control:
      | { method: 'fallback' }
      | {
          method: 'specified';
          maxPowerWPerDm3PerS: number;
          maxTemperatureRiseK: number;
          switchOnBelowC: number;
          maxSupplyTemperatureC: number;
        };
    preheatedDesignFlowM3PerH?: number;
    sourceReference: string;
  };
  fans:
    | { method: 'forfait'; current: 'ac' | 'dc'; manufactureYear?: number }
    | {
        method: 'declared';
        fans: Array<{
          id: string;
          power:
            | { method: 'nominal'; nominalPowerW: number }
            | { method: 'motor'; motorPowerW: number; manufactureYear?: number; electricalInputW?: number };
        }>;
        control:
          | { method: 'residential_table' }
          | { method: 'declared'; monthly: number[]; sourceReference: string }
          | { method: 'flow_control'; control: 'throttle' | 'inlet_vane_or_blade_pitch' | 'speed_control' | 'other' };
        buildingShare?: number;
        sourceReference: string;
      };
  sourceReference: string;
}

export interface VentilationBalanceFlows {
  requiredOutdoorAirM3PerH: number;
  effectiveOutdoorAirM3PerH: number;
  infiltrationM3PerH: number;
  naturalSupplyM3PerH: number;
  purgeM3PerH: number;
  ventilativeCoolingM3PerH: number;
  combustionM3PerH: number;
  mechanicalSupplyM3PerH: number;
  mechanicalExtractM3PerH: number;
  naturalSupplyTemperatureC: number;
  mechanicalSupplyTemperatureC: number;
  ventilativeCoolingTemperatureC: number;
  conductanceWPerK: number;
  heatFlowPerSetpointW: number;
}

export interface VentilationResult {
  zoneId: string;
  ventilationSystemOp: Array<'natural' | 'supply' | 'extract' | 'balanced'>;
  infiltrationQV1M3PerH: number;
  months: Array<{
    month: number;
    heating: VentilationBalanceFlows;
    cooling: VentilationBalanceFlows;
    heatingReferencePressurePa: number[];
    coolingReferencePressurePa: number[];
    fanElectricityKwh: number;
    frostProtectionElectricityKwh: number;
    grillePreheatingElectricityKwh: number;
    /** 9.29 Q_H;ϑHstook;in;air, kWh. */
    heatingLimitAirKwh: number;
    /** 10.20 Q_C;ϑkoelgrens;in;air for the cooling limit, kWh. */
    coolingLimitAirKwh: number;
    ahuHeatingKwh: number;
    ahuCoolingKwh: number;
    outdoorAirFraction: number | null;
  }>;
  demandFlows: Array<{
    id: string;
    months: Array<{
      month: number;
      heatingConductanceWPerK: number;
      heatingSupplyTemperatureC: number;
      coolingConductanceWPerK: number;
      coolingSupplyTemperatureC: number;
      heatingLimitSupplyTemperatureC: number;
    }>;
  }>;
  annualFanElectricityKwh: number;
  annualFrostProtectionElectricityKwh: number;
  annualGrillePreheatingElectricityKwh: number;
  interpretations: string[];
}

export interface VentilationAssessment {
  status: 'calculated_unverified' | 'invalid';
  scope: string;
  issues: Array<{ code: string; path: string }>;
  actual: VentilationResult | null;
  fixedC1: VentilationResult | null;
  referenceVerified: false;
}

export async function calculateVentilationWithRust(input: VentilationInput): Promise<VentilationAssessment> {
  if (isTauri()) {
    return invoke<VentilationAssessment>('calculate_ventilation', { input });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/ventilation/calculate', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ input }),
    });
    return response.json() as Promise<VentilationAssessment>;
  }
  throw new Error('Rust ventilation calculation is available in the desktop app and local development server.');
}

/** ISSO 82.1 basic survey of an existing dwelling; see docs/nta8800-basisopname.md. */
export type OpnameInsulation =
  | { kind: 'none_or_unknown' }
  | { kind: 'present_unknown_thickness' }
  | { kind: 'cavity_filled_unknown_width' }
  | { kind: 'thickness'; thicknessMm: number; proven?: boolean };

export type OpnameGlass =
  | 'triple_hr' | 'hr_plus_plus' | 'hr_plus' | 'hr' | 'coated_type_unknown' | 'double'
  | 'double_with_coating' | 'double_with_secondary' | 'hr_with_secondary'
  | 'hr_plus_plus_with_secondary' | 'secondary_window' | 'single' | 'leaded_light' | 'glass_blocks';

export type OpnameFrame = 'wood_or_plastic' | 'metal_with_thermal_break' | 'metal' | 'none';

export interface ResidentialSurvey {
  id: string;
  constructionYear: number;
  renovation?: {
    envelopePostInsulated: boolean;
    glazingReplacedWithDraughtStrips: boolean;
    framesReplaced: boolean;
    frameJointsSealed: boolean;
    year?: number | null;
    evidenceReference: string;
  } | null;
  dwelling:
    | { kind: 'single_family'; position: 'terraced' | 'end_or_corner' | 'detached' | 'unknown'; roofType: 'pitched' | 'partly_flat' | 'flat' }
    | { kind: 'apartment'; floor: 'ground_or_intermediate' | 'top'; side: 'middle' | 'end_or_corner' | 'unknown' };
  usableFloorAreaM2: number;
  areaSourceReference: string;
  buildingHeightM: number;
  construction: {
    floor: 'light' | 'heavy' | 'very_heavy';
    wall: 'light' | 'heavy' | 'very_heavy';
    lighterCeiling?: boolean;
    /** Closed or suspended ceiling, any floor type (table 7.5 first column). */
    closedOrSuspendedCeiling?: boolean;
    sourceReference: string;
  };
  measuredInfiltration?: { qv10Dm3PerSM2: number; sourceReference: string } | null;
  /** Storeys of the dwelling; absent: the storeys served by the heating. */
  storeys?: number | null;
  /** §7.2.4 vertical pipes; absent: one uninsulated pipe per storey, empty: none. */
  verticalPipes?: OpnameVerticalPipe[] | null;
  envelope: {
    surfaces: Array<{
      id: string;
      element: 'facade' | 'roof' | 'floor';
      boundary:
        | { kind: 'outdoor' | 'ground' | 'crawlspace' | 'adjacent_heated' | 'unheated_cellar' | 'strongly_ventilated' }
        | { kind: 'unheated_space'; spaceId: string };
      grossAreaM2: number;
      orientation?: NtaOrientation;
      tiltDeg?: number;
      cavity: boolean;
      insulation: OpnameInsulation;
      thermalCushions?: boolean;
      /** Reed thatch measured at the underside, mm (ISSO 82.1 afb. 8.16). */
      reedThicknessMm?: number | null;
      exposedPerimeterM?: number;
      crawlspaceBottomInsulated?: boolean | null;
      /** §8.7.2.1: insulated at a renovation or extension (with present_unknown_thickness). */
      renovation?: { year?: number | null; meetsRequirementsOfYear?: boolean } | null;
      sourceReference: string;
    }>;
    windows?: Array<{
      id: string;
      surfaceId: string;
      areaM2: number;
      glass: OpnameGlass;
      frame: OpnameFrame;
      obstruction?: { heating: number[]; cooling: number[]; sourceReference: string };
      /** ISSO 82.1/75.1 tables 8.24/8.25; exclusive with `obstruction`. */
      shading?:
        | { situation: 'minimal' }
        | { situation: 'constant_height_obstruction'; relativeHeight: number }
        | { situation: 'constant_overhang'; relativeHeight: number }
        | { situation: 'full'; coolingConditionsMet?: boolean }
        | { situation: 'overhang_with_obstructions'; overhangRelativeHeight: number }
        | { situation: 'other' }
        | { situation: 'side_obstruction'; side: NtaObstructionSide; relativeWidth: number };
      sourceReference: string;
    }>;
    doors?: Array<{
      id: string;
      surfaceId: string;
      areaM2: number;
      insulated?: boolean | null;
      glassFraction?: number;
      glass?: OpnameGlass;
      frame: OpnameFrame;
      sourceReference: string;
    }>;
    panels?: Array<{
      id: string;
      surfaceId: string;
      areaM2: number;
      insulation:
        | { kind: 'absent_or_unknown' | 'present_unknown_thickness' }
        | { kind: 'known_thickness'; thicknessMm: number };
      cavity: boolean;
      frame: OpnameFrame;
      sourceReference: string;
    }>;
    unheatedSpaces?: Array<{ id: string; description: string }>;
  };
  heating: {
    generator:
      | { kind: 'boiler'; boilerType: 'conventional' | 'vr' | 'hr100' | 'hr104' | 'hr107' | 'hydrogen'; pilotFlame?: boolean | null; insideThermalBoundary: boolean; manufactureYear?: number; installationYear?: number }
      | { kind: 'heat_pump'; source: 'outdoor_air' | 'exhaust_air' | 'outdoor_and_exhaust_air' | 'ground' | 'groundwater' | 'surface_water' | 'water_based_unknown'; airSink?: boolean; highTemperature?: boolean; capacityKw?: number; sourceRegenerationFactor?: number | null; highEfficiencyEvidence?: unknown }
      | { kind: 'district_heat' }
      | { kind: 'electric'; connectedDevices: number }
      | { kind: 'biomass'; appliance: 'freestanding_wood_stove' | 'insert_stove' | 'pellet_stove' | 'accumulating_stove' | 'central_boiler'; insideThermalBoundary: boolean; soleHeatingInServedRooms: boolean; annexRCompliant?: boolean | null }
      | { kind: 'none_present' };
    emitters: 'radiators' | 'low_temperature_radiators' | 'floor_heating' | 'floor_heating_and_radiators' | 'air_heating' | 'local_heaters';
    designClass?: 'c45_40' | 'c55_47' | 'c70_50' | 'c90_70' | null;
    balanced?: boolean | null;
    control: 'room_thermostat' | 'central_with_radiator_valves' | 'individual_room_control' | 'unknown';
    /** Afb. 9.1; absent: present with the forfait length when unheated spaces exist. */
    unheatedPipes?: { kind: 'absent' } | { kind: 'present'; lengthM?: number | null } | null;
    storeys?: number;
    sourceReference: string;
  };
  hotWater: {
    generator:
      | { kind: 'none' | 'electric_boiler' | 'electric_instantaneous' | 'district_heat' }
      | { kind: 'gas_appliance'; applianceType: 'bath_geyser' | 'combi' | 'kitchen_geyser' | 'unknown'; gaskeur: 'none' | 'gaskeur' | 'gaskeur_cw' | 'gaskeur_hr_cw' | 'unknown'; burnerLoadKw?: number; cwClass?: 'cw1' | 'cw2' | 'cw3' | 'cw4_to6' | 'unknown' | null }
      | { kind: 'heat_pump'; exhaustAirSource: boolean };
    served: 'kitchen_and_bathroom' | 'bathroom_only' | 'kitchen_only';
    kitchenLengthM?: number;
    bathroomLengthM?: number;
    showers?: number;
    showerHeatRecovery: 'none' | 'vertical' | 'horizontal' | 'unknown';
    /** Vessel of an electric boiler (§13.4). */
    boilerVessel?: {
      volumeL?: number | null;
      kitchenCabinet?: boolean;
      label?: string | null;
      manufactureYear?: number | null;
      inHeatedZone?: boolean | null;
    } | null;
    sourceReference: string;
  };
  ventilation: {
    principle: 'natural' | 'mechanical_supply' | 'mechanical_extract' | 'balanced';
    declaredVariant?: VentilationSystemVariant | null;
    selfRegulatingVents?: boolean | null;
    pressureClass?: 'at_most1_pa' | 'from1_to5_pa' | 'from5_to10_pa' | null;
    installationYear?: number | null;
    heatRecovery?: 'counter_flow_aluminium' | 'counter_flow_plastic' | 'counter_flow_unknown_material' | 'cross_flow' | 'plate_or_tube' | 'rotary' | 'enthalpy' | 'heat_pipe' | 'two_element' | 'unknown' | null;
    bypassPresent?: boolean | null;
    unitManufactureYear?: number | null;
    motor?: 'ac' | 'dc' | 'unknown' | null;
    sourceReference: string;
  };
  pv?: Array<{
    id: string;
    panelAreaM2: number;
    moduleType: 'monocrystalline' | 'polycrystalline' | 'amorphous_single_junction' | 'amorphous_multi_junction' | 'amorphous_unknown' | 'cigs' | 'cd_te' | 'unknown';
    installationYear?: number | null;
    azimuthDeg: number;
    tiltDeg: number;
    mounting: 'not_ventilated' | 'moderately_ventilated' | 'strongly_ventilated' | 'unknown';
    obstructionFactors?: number[];
    sourceReference: string;
  }>;
  /** Building-bound storage (§15.5); requires PV. */
  storage?: OpnameStorage | null;
  coolingPresent?: boolean;
  sourceReference: string;
  /** Adviser's reason per applied default (path or rule) for the forfait ("inklappen", BRL 9500 §4.2.2). */
  inklapRedenen?: Record<string, string>;
}

export interface OpnameVerticalPipe {
  insulated?: boolean | null;
  sharedZones?: number | null;
}

export interface OpnameStorage {
  electricalKwh?: number;
  thermalKwh?: number;
  sourceReference: string;
}

export interface OpnameAssessment {
  status: 'calculated_unverified' | 'derived_input_rejected' | 'invalid';
  scope: string;
  source: string;
  appliedDefaults: Array<{ rule: string; path: string; value: string; source: string; inklapReden?: string }>;
  warnings: Array<{ code: string; path: string; note: string }>;
  issues: Array<{ code: string; path: string }>;
  derivedInput: BuildingPerformanceInput | null;
  performance: BuildingPerformanceAssessment | null;
  /** §6.4/§6.5.2 schematisation warnings per zone. */
  schematisation?: Array<{ code: string; zoneId: string }>;
  referenceVerified: false;
}

export async function assessResidentialSurveyWithRust(survey: ResidentialSurvey): Promise<OpnameAssessment> {
  if (isTauri()) {
    return invoke<OpnameAssessment>('assess_residential_survey', { survey });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/opname/residential', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ survey }),
    });
    return response.json() as Promise<OpnameAssessment>;
  }
  throw new Error('Rust basic survey is available in the desktop app and local development server.');
}

/** ISSO 75.1 basic survey of an existing utility building (one calculation zone). */
export interface UtilitySurvey {
  id: string;
  constructionYear: number;
  renovation?: ResidentialSurvey['renovation'];
  buildingType:
    | { kind: 'single_layer'; position: 'detached' | 'end_or_corner' | 'terraced'; roof: 'pitched' | 'partly_flat' | 'flat' }
    | { kind: 'multi_layer_whole' }
    | { kind: 'multi_layer_part'; level: 'bottom' | 'intermediate' | 'top'; position: 'end_or_corner' | 'middle' | 'whole_storey' };
  /** Other functions up to 25 % of A_g are merged into the largest one. */
  functions: Array<{ function: Exclude<NtaLabelFunction, 'residential'>; areaM2: number }>;
  areaSourceReference: string;
  buildingHeightM: number;
  storeys?: number;
  construction: ResidentialSurvey['construction'];
  measuredInfiltration?: ResidentialSurvey['measuredInfiltration'];
  /** Toilet groups, stacked groups once (ISSO 75.1 table 7.8). */
  toiletStacks?: number | null;
  verticalPipes?: OpnameVerticalPipe[] | null;
  envelope: ResidentialSurvey['envelope'];
  solarControlWindowIds?: string[];
  heating: ResidentialSurvey['heating'];
  heatingInstallation: { collective: boolean; capacityKw?: number | null };
  cooling?: {
    generator:
      | 'compression' | 'room_air_conditioner' | 'gas_absorption' | 'external_cold' | 'unknown_collective'
      | 'aquifer_before2013' | 'aquifer_from2013' | 'aquifer_year_unknown' | 'surface_water'
      | 'closed_ground_loop' | 'dew_point_cooling';
    capacityKw?: number | null;
    emitter:
      | 'floor_cooling' | 'concrete_core_activation' | 'wall_cooling' | 'ceiling_cooling'
      | 'fan_coil_on_outer_wall' | 'fan_coil_on_ceiling' | 'split_indoor_units_on_wall'
      | 'split_indoor_units_on_ceiling' | 'other';
    fanCoilCount?: number;
    waterBased: boolean;
    designTemperature?: 't6_to12' | 't12_to16' | 't12_to18' | 't17_to21' | null;
    balanced?: boolean | null;
    control?: 'standalone' | 'central_with_room_control' | 'other_or_unknown' | null;
    pipesInsulated?: boolean | null;
    pipeInsulationYear?: number | null;
    aquiferPermitYear?: number | null;
    sourceReference: string;
  } | null;
  ventilation: Omit<ResidentialSurvey['ventilation'], 'sourceReference'> & {
    ductsLukaAbc?: boolean | null;
    ahu?: {
      insideThermalZone?: boolean | null;
      ductsOutsideThermalZone?: boolean | null;
      ductLength?: 'at_most20_m' | 'from20_to40_m' | 'at_least40_m' | null;
      ductsInsulated?: boolean | null;
    } | null;
    recirculation?: 'none' | 'present_percent_unknown' | 'unknown' | null;
    recirculationPercent?: number | null;
    flowControl?: {
      method: 'throttle' | 'inlet_vane' | 'blade_pitch' | 'speed_control' | 'other';
      minimumPercent?: number | null;
    } | null;
    sourceReference: string;
  };
  humidification?: {
    humidifier: 'electric_steam' | 'non_electric_steam' | 'adiabatic';
    absorptionWheel?: boolean;
    sourceReference: string;
  } | null;
  hotWater: {
    generator:
      | { kind: 'none' | 'electric_boiler' | 'electric_instantaneous' | 'district_heat' | 'collective_unknown' }
      | { kind: 'gas_appliance'; applianceType: 'bath_geyser' | 'combi' | 'kitchen_geyser' | 'unknown'; gaskeur: 'none' | 'gaskeur' | 'gaskeur_cw' | 'gaskeur_hr_cw' | 'unknown'; burnerLoadKw?: number }
      | { kind: 'heat_pump'; exhaustAirSource: boolean }
      | { kind: 'gas_storage_heater'; volumeL: number; before1985?: boolean | null; inHeatedZone?: boolean | null };
    meanDrawOffLengthM?: number | null;
    circulation?: boolean | null;
    showers?: number;
    showerHeatRecovery: 'none' | 'vertical' | 'horizontal' | 'unknown';
    storage?: Array<{
      id: string;
      volumeL: number;
      label?: 'a_plus' | 'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | null;
      producedFrom2018?: boolean | null;
      connection?: 'fully_insulated' | 'straight_only_four' | 'straight_only_more_than_four' | 'uninsulated' | null;
      inHeatedZone?: boolean | null;
      sourceReference: string;
    }>;
    sourceReference: string;
  };
  lighting: Array<{
    id: string;
    areaM2: number;
    power:
      | { method: 'luminaires'; luminaires: Array<{ count: number; powerW: number }> }
      | {
          method: 'lamps';
          lamps: Array<{
            count: number;
            lampPowerW: number;
            lampType:
              | 't12' | 't8_conventional' | 't8_high_frequency' | 't5' | 'led_in_luminaire'
              | 'cfl_plug_in' | 'cfl_integrated' | 'incandescent_or_halogen' | 'led_lamp' | 'unknown';
          }>;
        }
      | { method: 'unknown'; ledFrom2017?: boolean };
    presence:
      | 'none_or_central_on' | 'room_switch' | 'room_switch_with_sweep' | 'sensors_type_unknown'
      | 'auto_on_dimmed' | 'auto_on_auto_off' | 'manual_on_dimmed' | 'manual_on_auto_off';
    daylight?: 'none' | 'switching' | 'dimming' | 'unknown';
    largeOfficeGroup?: boolean;
    extractedLuminaires?: boolean;
    sourceReference: string;
  }>;
  pv?: ResidentialSurvey['pv'];
  bacs?: {
    systemPowerKw?: number | null;
    servedAreaM2?: number | null;
    present?: boolean | null;
    automationClassCOrBetter?: boolean | null;
    managementClassBOrBetter?: boolean | null;
    evidenceReference?: string | null;
  };
  storage?: OpnameStorage | null;
  sourceReference: string;
  /** Adviser's reason per applied default (path or rule) for the forfait (BRL 9500 §4.2.2). */
  inklapRedenen?: Record<string, string>;
}

export async function assessUtilitySurveyWithRust(survey: UtilitySurvey): Promise<OpnameAssessment> {
  if (isTauri()) {
    return invoke<OpnameAssessment>('assess_utility_survey', { survey });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/opname/utility', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ survey }),
    });
    return response.json() as Promise<OpnameAssessment>;
  }
  throw new Error('Rust basic survey is available in the desktop app and local development server.');
}

export async function calculateSpaceHeatingChainWithRust(input: SpaceHeatingChainInput): Promise<SpaceHeatingChainAssessment> {
  if (isTauri()) {
    return invoke<SpaceHeatingChainAssessment>('calculate_space_heating_chain', { input });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/heating/space-heating-chain/calculate', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ input }),
    });
    return response.json() as Promise<SpaceHeatingChainAssessment>;
  }
  throw new Error('Rust heating chain calculation is available in the desktop app and local development server.');
}

/** Table 5.2/5.5 carriers delivered to the generators of a collective system. */
export type NtaAnnexPCarrier =
  | { kind: 'natural_gas' }
  | { kind: 'oil' }
  | { kind: 'electricity'; directRenewableShare?: number }
  | { kind: 'biogas' }
  | { kind: 'biomass_above500_kw' }
  | { kind: 'waste_incineration' }
  | { kind: 'biofuel_mix'; biofuelShare: number };

export type NtaTableP5Source =
  | 'electric_ground' | 'electric_outdoor_air' | 'electric_groundwater_below15_c'
  | 'electric_surface_water' | 'electric_source15_to20_c' | 'electric_source20_to40_c'
  | 'electric_source_at_least40_c' | 'gas_ground_or_outdoor_air' | 'gas_groundwater' | 'gas_surface_water';

export type NtaAnnexPGenerator =
  | { kind: 'combustion'; carrier: NtaAnnexPCarrier; efficiency: number; efficiencyReference: string }
  | {
      kind: 'heat_pump';
      efficiency:
        | { method: 'declared'; value: number; sourceReference: string }
        | { method: 'table_p5'; source: NtaTableP5Source; supplyTemperatureC: number };
      drive: NtaAnnexPCarrier;
    }
  | {
      kind: 'chp_without_loss';
      carrier: NtaAnnexPCarrier;
      thermalEfficiency: number;
      electricalEfficiency: number;
      efficiencyReference: string;
    }
  | { kind: 'chp_with_loss'; carrier: NtaAnnexPCarrier; lossRatio?: number; lossRatioReference?: string }
  | { kind: 'residual_heat'; auxiliarySpecific?: number; auxiliaryReference?: string }
  | { kind: 'geothermal'; sourceTemperatureC: number; returnTemperatureC: number }
  | { kind: 'declared'; primaryFactor: number; co2KgPerKwh: number; renewableFactor: number; sourceReference: string };

export type NtaAnnexPFunction = 'heating' | 'hot_water' | 'cooling';

/** NTA 8800 §5.8 / annex P: external heat, hot-water or cold supply values. */
export type NtaAnnexPRoute =
  | {
      method: 'declared';
      primaryFactor: number;
      renewableFactor: number;
      co2KgPerKwh: number;
      declarationReference: string;
    }
  | {
      method: 'calculated';
      function: NtaAnnexPFunction;
      deliveredKwh: number;
      distribution:
        | { method: 'flows'; inputKwh?: number; lossKwh?: number; sourceReference: string }
        | {
            method: 'small_system_forfait';
            connections: number;
            connectionType: 'ground_bound' | 'within_building';
            designTemperature?: 't90_to60' | 't90_to50' | 't70_to40' | 't50_to40' | 't35_to25';
            otherLossKwh?: number;
          }
        | { method: 'small_cold_forfait'; supplyBelow10C: boolean };
      generators: Array<{ id: string; energyFraction: number; kind: NtaAnnexPGenerator }>;
      auxiliaryElectricityKwh: number;
      auxiliaryRenewableShare?: number;
      /** P.34/P.35 η_WD;gen;sto; required for hot water (WD). */
      hotWaterStorage?:
        | { method: 'losses'; storageLossKwh: number; pipeLossKwh: number; sourceReference: string }
        | { method: 'forfait'; insulation: 'at_least20_mm' | 'at_least10_mm' | 'none' };
      sourceReference: string;
    }
  | {
      method: 'measured';
      function: NtaAnnexPFunction;
      deliveredKwh: number;
      inputs: Array<{ carrier: NtaAnnexPCarrier; kwh: number; chpLossElectrical?: number }>;
      exportedElectricityKwh?: number;
      renewableFactor: number;
      sourceReference: string;
    };

export interface NtaExternalSupply {
  heating?: NtaAnnexPRoute | null;
  hotWater?: NtaAnnexPRoute | null;
  cooling?: NtaAnnexPRoute | null;
  collectiveHeatPumpSource?: {
    temperatureClass: 'below20_c' | 'at_least20_c_or_surface_water_or_unknown';
    supplierReference: string;
    annexP?: NtaAnnexPRoute | null;
  } | null;
}

export interface NtaSupplyFactors {
  primaryFactor: number;
  renewableFactor: number;
  co2KgPerKwh: number;
}

export interface NtaCarrierFactors {
  districtHeat: NtaSupplyFactors;
  districtHotWater: NtaSupplyFactors;
  districtCold: NtaSupplyFactors;
  heatPumpSource: NtaSupplyFactors | null;
}

export interface NtaAnnexPSystemResult {
  factors: NtaSupplyFactors;
  distributionEfficiency: number | null;
  generationPrimaryFactor: number | null;
  storageEfficiency?: number;
  generators: Array<{ id: string; primaryFactor: number; co2KgPerKwh: number; renewableFactor: number; heatKwh: number }>;
}

export interface NtaExternalSupplyResult {
  declared: NtaCarrierFactors;
  forfait: NtaCarrierFactors;
  qualityDeclarationUsed: boolean;
  heating: NtaAnnexPSystemResult | null;
  hotWater: NtaAnnexPSystemResult | null;
  cooling: NtaAnnexPSystemResult | null;
  heatPumpSource: NtaAnnexPSystemResult | null;
  forfaitPrimaryFossilKwh: number | null;
  forfaitRenewablePrimaryKwh: number | null;
  forfaitCo2Kg: number | null;
}

export interface BuildingPerformanceInput {
  calculationScope: 'residential' | 'utility';
  totalUsableFloorAreaM2: number;
  areaSourceReference: string;
  spaceHeating: SpaceHeatingChainInput;
  heatPumpRenewable?: {
    sourceBelow20C: boolean;
    exhaustAirSource: boolean;
    sourceReference: string;
    combinedOutdoorAndExhaustAir?: boolean;
    outdoorAirHeatFraction?: number | null;
    outdoorAirFractionReference?: string | null;
  } | null;
  bacsFactor: 1 | 1.05;
  bacsSourceReference: string;
  useInventoryComplete: boolean;
  declaredUses: Array<{
    id: string;
    service:
      | 'domestic_hot_water' | 'domestic_hot_water_auxiliary' | 'ventilation_fans'
      | 'space_cooling' | 'space_cooling_auxiliary' | 'lighting' | 'pv_auxiliary' | 'humidification';
    carrier: 'el' | 'gas' | 'oil';
    monthlyKwh: number[];
    sourceReference: string;
  }>;
  declaredRenewableHeat?: Array<{ id: string; monthlyKwh: number[]; sourceReference: string }>;
  productionInventoryComplete: boolean;
  onSiteProduction: Array<{ id: string; kind: 'pv' | 'pvt' | 'wind'; monthlyKwh: number[]; sourceReference: string }>;
  pvSystems?: Array<{
    id: string;
    peakPower: NtaPvPeakPower;
    azimuthDeg: number;
    tiltDeg: number;
    mounting?: 'not_ventilated' | 'moderately_ventilated' | 'strongly_ventilated' | 'unknown';
    /** F_sh;obst;mi: one value or twelve monthly values (§17.3). */
    obstructionFactors: number[];
    collective?: { buildingUsableFloorAreaM2: number; sourceReference: string } | null;
    sourceReference: string;
  }>;
  hotWater?: NtaHotWaterSystem | null;
  /** Utility lighting per calculation zone (NTA 8800 chapter 14). */
  lighting?: NtaZoneLighting[];
  demandUsesFixedC1Ventilation: boolean;
  batteryStoragePresent: boolean;
  storage?: NtaEnergyStorage | null;
  /** NTA 8800 §5.8 / annex P values for external supply. */
  externalSupply?: NtaExternalSupply;
}

/** 5.14a: building-bound storage capacities for `f_BAT;cor`. */
export interface NtaEnergyStorage {
  buildingBoundElectricalKwh: number;
  buildingBoundThermalKwh: number;
  sourceReference: string;
}

/** Chapter 14 lighting of one calculation zone. */
export interface NtaZoneLighting {
  zoneId: string;
  functions: Array<{ function: NtaLabelFunction; areaM2: number }>;
  lightingZones: Array<{
    id: string;
    areaM2: number;
    power:
      | { method: 'forfait'; ledFrom2017: boolean }
      | { method: 'installed'; dynamicFactor?: number | null; sourceReference: string;
          luminaires: Array<{ count: number; power: { method: 'system'; powerW: number }
            | { method: 'lamps'; lampPowerW: number; lampCount: number; technology: string } }> };
    parasitic: { method: 'forfait' } | { method: 'installed'; emergencyChargingW: number; controlStandbyW: number; sourceReference: string };
    occupancy: {
      control: 'manual_or_unknown' | 'manual_with_sweep' | 'auto_on_dimmed' | 'auto_on_auto_off' | 'manual_on_dimmed' | 'manual_on_auto_off';
      centralOnControl: boolean;
      largeOfficeGroup?: boolean;
    };
    daylight: { method: 'none' } | { method: 'forfait'; daylightControl: boolean } | { method: 'sectors'; sectors: unknown[]; sourceReference: string };
    extractedLuminaires?: boolean;
  }>;
  sourceReference: string;
}

/** Active cooling with demonstrated capacity (NTA 8800 §5.7.1). */
export interface NtaActiveCoolingEvidence {
  system:
    | 'compression_table10_29' | 'absorption_table10_30' | 'free_cooling_table10_34'
    | 'dew_point_cooling_humidified_exhaust' | 'heat_pump_with_cooling_emitter'
    | 'external_cold_with_cooling_emitter' | 'split_units_in_every_habitable_room' | 'other_utility';
  capacity:
    | { method: 'dynamic_cooling_load'; sourceReference: string }
    | { method: 'annex_aa'; calculation?: NtaAnnexAaInput | null; sourceReference: string }
    | { method: 'solar_limitation'; criterion: 'small_window_area' | 'shaded_glazing'; sourceReference: string };
  sourceReference: string;
}

/** P_pk route of NTA 8800 16.4a/16.4b. */
export type NtaPvPeakPower =
  | { method: 'table16_1'; moduleType: string; panelAreaM2: number }
  | { method: 'declared_specific'; peakPowerWPerM2: number; panelAreaM2: number }
  | { method: 'panels'; panelPeakPowerW: number; panelCount: number };

export type NtaLabelFunction =
  | 'residential' | 'office' | 'assembly_without_day_care' | 'assembly_with_day_care'
  | 'education' | 'healthcare_without_beds' | 'healthcare_with_beds' | 'retail'
  | 'sport' | 'lodging' | 'cell';

export type NtaBblFunction =
  | 'residential_building' | 'caravan' | 'floating_building_after2018_berth'
  | 'floating_building_other_berth' | 'other_residential' | 'assembly_child_care'
  | 'other_assembly' | 'cell' | 'healthcare_with_beds' | 'other_healthcare' | 'office'
  | 'lodging_in_lodging_building' | 'other_lodging' | 'education' | 'sport' | 'retail';

export interface NtaBblCheck {
  source: string;
  /** The function, or the largest one for mixed functions. */
  function: NtaBblFunction;
  functions: Array<{ function: NtaBblFunction; areaM2: number }>;
  lossAreaRatio: number;
  limits: {
    energyNeedMaxKwhPerM2: number;
    primaryFossilMaxKwhPerM2: number;
    renewableShareMinPercent: number;
    lightConstructionAllowanceApplied: boolean;
  };
  energyNeedMeets: boolean | null;
  primaryFossilMeets: boolean | null;
  renewableShareMeets: boolean | null;
}

export interface NtaTojuliAssessment {
  zoneId: string;
  status: 'calculated_unverified' | 'invalid';
  activeCooling: boolean;
  orientations: Array<{
    orientation: NtaOrientation;
    areaM2: number;
    share: number;
    assessed: boolean;
    conductanceWPerK: number;
    coolingNeedJulyKwh: number;
    boosterHeatPumpJulyKwh: number;
    tojuliK: number | null;
  }>;
  maxTojuliK: number | null;
  meetsBblLimit: boolean | null;
  annexAa: NtaAnnexAaResult | null;
  issues: Array<{ code: string; path: string }>;
}

/** Annex AA input per calculation zone (dwellings). */
export interface NtaAnnexAaInput {
  constructionYear: number;
  postInsulated?: boolean;
  /** B_C;inst;zi in kW; omit when every room has its own generator. */
  generatorCapacityKw?: number | null;
  rooms: Array<{
    id: string;
    areaM2: number;
    living: boolean;
    opaqueInnerAreaM2: number;
    windows?: Array<{ windowId: string; uWithShutterWPerM2k?: number | null }>;
    installedCapacityKw: number;
  }>;
}

interface NtaAnnexAaLoads {
  peakHour: number;
  internalW: number;
  outdoorAirW: number;
  opaqueW: number;
  solarW: number;
  glazingW: number;
  needWPerM2: number;
  requiredKw: number;
}

export interface NtaAnnexAaResult extends NtaAnnexAaLoads {
  generatorKw: number | null;
  rooms: Array<NtaAnnexAaLoads & { id: string; installedKw: number; sufficient: boolean }>;
  sufficient: boolean;
}

/** Annex A dynamic transparent element. */
export type NtaDynamicTransparent =
  | {
      method: 'weighted_states';
      states: Array<{ id: string; gPerpendicular: number; uValueWPerM2k: number }>;
      /** 12 rows, one weight per state, each row summing to 1. */
      solarWeights: number[][];
      temperatureWeights: number[][];
      sourceReference: string;
    }
  | {
      method: 'single_state';
      state: { id: string; gPerpendicular: number; uValueWPerM2k: number };
      sourceReference: string;
    };

/** Annex B construction element; layers from the zone side outwards. */
export interface NtaMassElement {
  id: string;
  areaM2: number;
  layers: Array<{
    thicknessM: number;
    conductivityWPerMk: number;
    densityKgPerM3: number;
    specificHeatJPerKgk: number;
    openSuspendedCeiling?: boolean;
  }>;
  bothSides?: boolean;
  sourceReference: string;
}

type NtaHeatRejection =
  | 'air_cooled' | 'closed_cooling_tower' | 'open_cooling_tower' | 'dry_cooler' | 'ground_storage' | 'surface_water';
type NtaDeclaredEfficiency = { value: number; sourceReference: string };
type NtaChpClass = { powerKw: number; builtAfter2006: boolean; hreDeclared?: boolean; lowTemperature?: boolean };
type NtaEn14825Point = { partLoadPercent: number; eer: number; evaporatorOutletC: number; condenserInletC: number };
/** §10.5.4 (method 1, NEN-EN 14825) or §10.5.5 (method 2, NEN-EN 14511) instead of table 10.29. */
export type NtaCompressionPerformance =
  | {
      method: 'en14825';
      nominalEer: number;
      nominalCapacityKw: number;
      minimumCapacityKw: number;
      /** Conditions A, B, C and D. */
      testPoints: NtaEn14825Point[];
      fifthPoint?: NtaEn14825Point | null;
      condenserInletLimitC?: number | null;
      requiredOutletC?: number | null;
      sourceReference: string;
    }
  | {
      method: 'en14511';
      nominalEer: number;
      nominalCapacityKw: number;
      nominalEvaporatorOutletC: number;
      nominalCondenserInletC: number;
      /** Table 10.19; required for a room air conditioner. */
      roomUnitType?: 'split' | 'multi_split_staged' | 'split_inverter' | 'multi_split_inverter' | null;
      heatRejectionToExhaustAir?: boolean;
      axialFansWithoutSilencer?: boolean;
      requiredOutletC?: number | null;
      sourceReference: string;
    };

/** Chapter 10 cooling system (method 3, or methods 1/2 for rated compression generators). */
export interface NtaCoolingSystem {
  emission: {
    emitter: 'floor_cooling' | 'wall_cooling' | 'fan_coil_or_rac_on_outer_wall' | 'ceiling_cooling'
      | 'fan_coil_or_rac_on_ceiling' | 'other_or_unknown';
    balancing: 'none_or_unknown' | 'static' | 'dynamic' | 'not_applicable';
    control: 'unknown_or_other' | 'standalone_per_room' | 'central_with_room_control';
    fanCoilCount?: number;
    sourceReference: string;
  };
  distribution?: {
    designTemperature: 't6_to12_or_unknown' | 't12_to16' | 't12_to18' | 't17_to21';
    pipe:
      | { kind: 'insulated_from1995' | 'insulated1980_to1995' | 'insulated_before1980_or_unknown_age' | 'uninsulated' }
      | { kind: 'declared'; psiWPerMK: number; sourceReference: string };
    fittingsInsulated: boolean;
    pipeLengthM?: number | null;
    unconditionedPipeLengthM?: number | null;
    unconditionedAmbientC?: number | null;
    pump?: {
      hydraulicallyBalanced: boolean;
      floorCount: number;
      heatMeter: boolean;
      individualDwellingInstallation: boolean;
      labelPowerKw?: number | null;
      energyEfficiencyIndex?: number | null;
      sourceReference: string;
    } | null;
    sourceReference: string;
  } | null;
  generators: Array<{
    id: string;
    generator:
      | {
          kind: 'compression';
          heatRejection?: NtaHeatRejection | null;
          declared?: NtaDeclaredEfficiency | null;
          performance?: NtaCompressionPerformance | null;
        }
      | { kind: 'room_air_conditioner'; declared?: NtaDeclaredEfficiency | null; performance?: NtaCompressionPerformance | null }
      | { kind: 'unknown_collective' }
      | { kind: 'gas_engine_compression'; gasEngine: NtaChpClass; heatRejection?: NtaHeatRejection | null }
      | { kind: 'gas_absorption'; heatRejection?: NtaHeatRejection | null; declared?: NtaDeclaredEfficiency | null }
      | { kind: 'absorption_external_heat'; heatRejection?: NtaHeatRejection | null }
      | { kind: 'absorption_chp'; chp: NtaChpClass; heatRejection?: NtaHeatRejection | null }
      | { kind: 'external_cold' }
      | {
          kind: 'free_cooling';
          source: 'aquifer_from2013' | 'aquifer_utility_before2013' | 'aquifer_dwellings_before2013'
            | 'surface_water' | 'closed_ground_loop' | 'dew_point_cooling';
          heatPumpSource?: boolean;
          groundAboveZeroDemonstrated?: boolean;
        };
    capacityKw?: number | null;
    equipmentReference: string;
  }>;
  /** Q_C;HP;si per month (annex W), kWh. */
  boosterHeatPumpExtractionKwh?: number[];
  collective?: { buildingUsableFloorAreaM2: number; sourceReference: string } | null;
}

type NtaApplicationClass = 'class1' | 'class2' | 'class3' | 'class4';
type NtaDhwDeclared = { value: number; sourceReference: string };

/** 13.144a limits of an exhaust-air heat pump among several generators. */
export interface NtaExhaustAirUse {
  /** Ventilation system C or D without heat recovery. */
  ventilationSuitable: boolean;
  /** f_combi per month; empty means hot water only. */
  heatingTimeFraction?: number[];
}

/** Collectors use the §17.3 collector tables (17.6/17.12/17.15). */
type NtaSolarObstruction = NtaCollectorObstruction;

/** §13.7 solar water heater; see crates/nta8800-core/src/solar_thermal.rs. */
export interface NtaSolarWaterHeater {
  id: string;
  solarUse: 'water_heating' | 'combi';
  /** N_soli identical physical systems. */
  count?: number;
  method:
    | {
        method: 'calculated';
        solarType: 'preheater' | 'integrated_backup';
        collectors: {
          moduleAreaM2: number;
          moduleCount: number;
          orientation: 'north' | 'north_east' | 'east' | 'south_east' | 'south' | 'south_west' | 'west' | 'north_west';
          tiltDeg: number;
          obstruction: NtaSolarObstruction;
          efficiency:
            | { method: 'forfait'; collector: 'unglazed_or_unknown' | 'glazed' | 'evacuated_tube' }
            | { method: 'declared'; eta0: number; a1WPerM2K: number; a2WPerM2K2: number; incidenceAngleModifier: number; sourceReference: string };
          heatExchangerWPerK?: number | null;
          loopPipes:
            | { method: 'forfait' }
            | { method: 'pipes'; pipes: Array<{ lengthM: number; psiWPerMk: number }>; sourceReference: string }
            | { method: 'declared'; heatLossWPerK: number; sourceReference: string };
          pumpPowerW?: number | null;
        };
        storage: {
          totalVolumeL: number;
          backupVolumeL?: number | null;
          loss:
            | { method: 'label'; label: 'a_plus' | 'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' }
            | { method: 'unknown_label'; producedFrom2018: boolean }
            | { method: 'measured'; transmissionWPerK: number };
          backupLossInGeneratorEfficiency?: boolean;
        };
      }
    | {
        method: 'tested';
        solarType: 'preheater' | 'integrated_backup';
        orientation: 'north' | 'north_east' | 'east' | 'south_east' | 'south' | 'south_west' | 'west' | 'north_west';
        tiltDeg: number;
        obstruction: NtaSolarObstruction;
        totalVolumeL: number;
        testPoints: Array<{ annualDemandKwh: number; solarOutputKwh?: number | null; backupOutputKwh?: number | null; auxiliaryKwh: number }>;
        backupLossInGeneratorEfficiency?: boolean;
        sourceReference: string;
      };
  pvt?: 'unglazed' | 'single_glazed' | 'tested_iso9806' | null;
  sourceReference: string;
}

export type NtaHotWaterGenerator =
    | { kind: 'gas_appliance'; appliance: 'without_gaskeur' | 'water_heater_gaskeur' | 'water_heater_gaskeur_cw' | 'kitchen_geyser'
        | 'combi_gaskeur' | 'combi_gaskeur_hr_cw' | 'unknown'; measuredClass?: NtaApplicationClass | null; kitchenOnly?: boolean;
        declared?: NtaDhwDeclared | null; annexT?: NtaAnnexTTest | null;
        /** §13.8.4.3 conditions; required with annexT. */
        annexTConditions?: { typeSuppliedBefore2021: boolean; applianceIndoors: boolean } | null }
    | { kind: 'heat_pump'; exhaustAirSource: boolean; sourceCorrection?: number | null; measuredClass?: NtaApplicationClass | null;
        outdoorAirFraction?: number | null }
    | { kind: 'heat_pump_en16147'; profile: 's' | 'm' | 'l' | 'xl'; deliveredKwhPerDay: number; inputKwhPerDay: number;
        exhaustAirSource: boolean; storageWithoutLegionellaCycle: boolean; outdoorAirFraction?: number | null; sourceReference: string }
    | { kind: 'electric_instantaneous' }
    | { kind: 'electric_boiler' }
    | { kind: 'gas_storage_heater'; volumeL: number; measuredStandbyKwhPerDay?: number | null; before1985: boolean; inHeatedZone: boolean }
    | { kind: 'large_direct_storage'; gasFired: boolean }
    | { kind: 'indirect_boiler'; boiler: 'conventional_or_unknown' | 'vr' | 'hr100_or104' | 'hr107'; oil: boolean;
        insideBoundary: boolean; alsoSpaceHeating: boolean; declared?: NtaDhwDeclared | null }
    | { kind: 'indirect_heat_pump'; alsoSpaceHeating: boolean }
    | { kind: 'external_heat' }
    | ({ kind: 'booster_heat_pump' } & NtaBoosterHeatPump);

/** Chapter 13 hot-water system (several generators, solar systems). */
export interface NtaHotWaterSystem {
  need:
    | { method: 'residential'; dwellingCount: number; sourceReference: string }
    | { method: 'utility'; areas: Array<{ function: NtaLabelFunction; areaM2: number }>; sourceReference: string };
  emission:
    | { method: 'residential'; served: 'kitchen_and_bathroom' | 'bathroom_only' | 'kitchen_only';
        kitchenLengthM?: number | null; bathroomLengthM?: number | null; sourceReference: string }
    | { method: 'utility'; meanLengthM: number; sourceReference: string };
  showerHeatRecovery?: {
    showers: Array<
      | { unit: 'none' | 'vertical' | 'horizontal' | 'unknown' }
      | { unit: 'declared'; efficiency: number; testClass?: 'class2' | 'class3' | 'class4'; sourceReference: string }
      | { unit: 'annex_u'; test: NtaAnnexUTest }
    >;
    connection: 'mixer_and_heater' | 'mixer_only' | 'heater_only' | 'shared_units' | 'unknown';
    sourceReference: string;
  } | null;
  circulation?: {
    outerDiameterMm?: number | null;
    insulation: 'none' | 'unknown' | 'mm10' | 'mm15' | 'mm20' | 'mm25';
    declaredPsiWPerMK?: number | null;
    fittingsInsulated: boolean;
    lengthM?: number | null;
    unheatedLengthM?: number | null;
    unheatedAmbientC?: number | null;
    floorCount: number;
    sportHallAreaM2?: number;
    connectedDwellings?: number | null;
    pump: { control: 'uncontrolled_or_unknown' | 'constant_pressure'; labelPowerKw?: number | null; energyEfficiencyIndex?: number | null };
    sourceReference: string;
  } | null;
  storage?: Array<{
    id: string;
    volumeL: number;
    loss:
      | { method: 'label'; label: 'a_plus' | 'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' }
      | { method: 'unknown_label'; producedFrom2018: boolean }
      | { method: 'measured'; transmissionWPerK: number };
    connectionFactor: 1 | 2 | 3 | 4 | 5;
    inHeatedZone: boolean;
    unheatedAmbientC?: number | null;
    sourceReference: string;
  }>;
  deliverySets?: { count: number; sourceReference: string } | null;
  boilingWaterTap?: boolean;
  generator: NtaHotWaterGenerator;
  /** P_nom of the main generator (13.141), kW. */
  nominalPowerKw?: number | null;
  /** 13.144a when the main generator is an exhaust-air heat pump. */
  exhaustAir?: NtaExhaustAirUse | null;
  /** Further generators (13.8.2). */
  additionalGenerators?: Array<{
    generator: NtaHotWaterGenerator;
    nominalPowerKw?: number | null;
    exhaustAir?: NtaExhaustAirUse | null;
    equipmentReference: string;
  }>;
  /** 13.141a–d: main generator first, the single additional one second. */
  series?: { kind: 'hotfill_electric_boiler' } | { kind: 'collective_first_also_heating'; maximumSupplyC: number[] } | null;
  /** §13.7 solar water heaters and solar combi systems. */
  solar?: NtaSolarWaterHeater[];
  collective?: { buildingUsableFloorAreaM2: number; sourceReference: string } | null;
  equipmentReference: string;
}

export interface BuildingPerformanceAssessment {
  status: 'calculated_unverified' | 'invalid';
  scope: string;
  chapter5Source: string;
  inputFingerprint: string;
  finalEditionVerified: false;
  referenceVerified: false;
  attestStatus: 'unattested';
  labelAvailable: false;
  carriers: Array<{
    carrier: 'el' | 'gas' | 'oil' | 'dh' | 'dw' | 'dc' | 'dh_hp_source' | 'bm';
    month: number;
    usedKwh: number;
    deliveredKwh: number;
  }>;
  electricityBalance: Array<{ month: number; usedKwh: number; producedKwh: number; selfUsedKwh: number; exportedKwh: number }>;
  annualPrimaryFossilKwh: number | null;
  annualRenewablePrimaryKwh: number | null;
  annualHeatPumpAmbientHeatKwh: number | null;
  annualHeatingAndCoolingNeedKwh: number | null;
  annualStorageCorrectionKwh: number | null;
  annualCo2Kg: number | null;
  co2KgPerM2: number | null;
  needIndicatorKwhPerM2Year: number | null;
  primaryFossilIndicatorKwhPerM2Year: number | null;
  renewableSharePercent: number | null;
  indicativeLabelClass: string | null;
  labelSource: string;
  bblCheck: NtaBblCheck | null;
  a0Check: null | {
    source: string;
    primaryFossilMaxKwhPerM2: number;
    energyNeedMeets: boolean | null;
    primaryFossilMeets: boolean | null;
    renewableShareMeets: boolean | null;
    noOnSiteFossilCombustion: boolean;
    eligible: boolean | null;
  };
  tojuli: NtaTojuliAssessment[];
  tojuliMaxK: number | null;
  tojuliMeetsBblLimit: boolean | null;
  spaceHeating: SpaceHeatingChainAssessment;
  /** Chapter 13 when calculated. */
  hotWater?: NtaHotWaterAssessment;
  lighting?: Array<{ zoneId: string; annualKwh: number; monthlyKwh: number[]; internalGainW: number }>;
  /** Factors of the external supply and the EMGforf totals (§5.8). */
  externalSupply?: NtaExternalSupplyResult | null;
  issues: Array<{ code: string; path: string }>;
}

export async function calculateBuildingPerformanceWithRust(input: BuildingPerformanceInput): Promise<BuildingPerformanceAssessment> {
  if (isTauri()) {
    return invoke<BuildingPerformanceAssessment>('calculate_building_performance', { input });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/performance/calculate', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ input }),
    });
    return response.json() as Promise<BuildingPerformanceAssessment>;
  }
  throw new Error('Rust performance calculation is available in the desktop app and local development server.');
}

export interface NtaCalculationInput {
  calculationScope: 'residential' | 'utility';
  areaSourceReference: string;
  usageFunction: NtaUsageFunction;
  dwellingType?: NtaDwellingType | null;
  setpoints: MonthlyDemandInput['setpoints'];
  thermalMass: MonthlyDemandInput['thermalMass'];
  internalGains: MonthlyDemandInput['internalGains'];
  surfaceTilts: Array<{ surfaceId: string; tiltDeg: number; sourceReference: string }>;
  windowSolar: {
    frameFraction: number;
    obstruction: NtaObstruction;
    movableShading?: NtaMovableShading | null;
    sourceReference: string;
  };
  groundFloors: Array<{
    surfaceId: string;
    exposedPerimeterM: number;
    constructionResistanceM2kPerW: number;
    edgeThermalBridges: NtaGroundEdgeThermalBridges;
    edgeInsulation?: NtaGroundEdgeInsulation[];
    below?: NtaFloorBelow | null;
    heatedBasement?: NtaHeatedBasement | null;
    sourceReference: string;
  }>;
  ventilationFlows: MonthlyDemandInput['ventilationFlows'];
  ventilation?: VentilationInput | null;
  /** 7.3.3 vertical pipes (single-zone projects; per zone in `zoneData`). */
  verticalPipes?: Array<{ id: string; storeys: number; insulated: boolean; sharedZones?: number; sourceReference: string }>;
  zoneData?: Array<{
    zoneId: string;
    verticalPipes?: Array<{ id: string; storeys: number; insulated: boolean; sharedZones?: number; sourceReference: string }>;
    functionAreas?: Array<{ function: NtaUsageFunction; areaM2: number }>;
    ventilationFlows: MonthlyDemandInput['ventilationFlows'];
    ventilation?: VentilationInput | null;
    internalGains: MonthlyDemandInput['internalGains'];
    usageFunction?: NtaUsageFunction | null;
    dwellingType?: NtaDwellingType | null;
    setpoints?: MonthlyDemandInput['setpoints'];
    thermalMass?: MonthlyDemandInput['thermalMass'];
    emission?: SpaceHeatingChainInput['emission'];
    distribution?: SpaceHeatingChainInput['distribution'];
  }>;
  emission: SpaceHeatingChainInput['emission'];
  distribution: SpaceHeatingChainInput['distribution'];
  generator: SpaceHeatingChainInput['generator'];
  distributionSystem?: NtaDistributionSystem | null;
  collectiveConnection?: SpaceHeatingChainInput['collectiveConnection'];
  heatPumpRenewable?: BuildingPerformanceInput['heatPumpRenewable'];
  bacsFactor: 1 | 1.05;
  bacsSourceReference: string;
  useInventoryComplete: boolean;
  declaredUses: BuildingPerformanceInput['declaredUses'];
  declaredRenewableHeat?: BuildingPerformanceInput['declaredRenewableHeat'];
  productionInventoryComplete: boolean;
  onSiteProduction?: BuildingPerformanceInput['onSiteProduction'];
  pvSystems?: BuildingPerformanceInput['pvSystems'];
  hotWater?: NtaHotWaterSystem | null;
  /** Utility lighting per calculation zone (NTA 8800 chapter 14). */
  lighting?: NtaZoneLighting[];
  cooling?: NtaCoolingSystem | null;
  labelFunction?: NtaLabelFunction | null;
  bblFunction?: NtaBblFunction | null;
  /** Bbl art. 4.149 lid 2: several use functions, limits weighted by area. */
  bblFunctions?: Array<{ function: NtaBblFunction; areaM2: number }>;
  activeCooling?: NtaActiveCoolingEvidence | null;
  permitApplicationAfter20260529?: boolean;
  demandUsesFixedC1Ventilation: boolean;
  batteryStoragePresent: boolean;
  storage?: NtaEnergyStorage | null;
  /** NTA 8800 §5.8 / annex P values for external supply. */
  externalSupply?: NtaExternalSupply;
}

export interface ProjectPerformanceAssessment {
  status: 'calculated_unverified' | 'incomplete' | 'invalid';
  targetNormVersion: string;
  kernelVersion: string;
  inputFingerprint: string;
  attestStatus: 'unattested';
  gaps: Array<{ code: string; path: string; detail?: string }>;
  geometry: null | {
    usableFloorAreaM2: number;
    /** A_ls with f_ls (6.7.3): ground and crawlspace weighted 0,7. */
    lossAreaM2: number;
    /** Unweighted envelope A_o to outdoor air, ground and unheated spaces. */
    envelopeAreaM2: number;
    lossAreaRatio: number | null;
    unclassifiedSurfaceCount: number;
  };
  derivedInput: BuildingPerformanceInput | null;
  performance: BuildingPerformanceAssessment | null;
  /** BRL 9500 §4.2.3–4.2.5 checks; null without a registration block. */
  registration?: RegistrationAssessment | null;
  /** Regeling energieprestatie gebouwen art. 4 label data. */
  labelData?: LabelData | null;
}

/** Registration data of the EP report (crates/nta8800-core/src/registration.rs); all optional. */
export interface NtaRegistration {
  purpose?: 'existing_building' | 'delivery' | 'bbl_check';
  surveyType?: 'basic' | 'detailed';
  representation?: 'unique' | 'reference' | 'similar';
  referenceObjectId?: string;
  bagObjectId?: string;
  postcode?: string;
  houseNumber?: string;
  houseNumberAddition?: string;
  constructionYear?: number;
  buildingType?: string;
  client?: string;
  certificateNumber?: string;
  surveyingAdvisor?: { name: string; competenceNumber: string };
  registeringAdvisor?: { name: string; competenceNumber: string };
  /** YYYY-MM-DD; for a relabel the original survey date. */
  surveyDate?: string;
  /** YYYY-MM-DD. */
  registrationDate?: string;
  serialProject?: boolean;
  relabel?: boolean;
  originalKernelVersion?: string;
  epOnlineNumber?: string;
  /** Opleverdatum, YYYY-MM-DD (§3.1: completed after 1-1-2021 needs a detailed survey). */
  completionDate?: string;
  /** BRL 9500 §3.1 situations that make the detailed survey mandatory. */
  detailSurveyTriggers?: NtaDetailSurveyTriggers;
  /** Evidence register of the project dossier (BRL 9500 Bijlage 3). */
  evidence?: NtaEvidenceItem[];
}

export interface NtaDetailSurveyTriggers {
  rebuiltAfterDemolition?: boolean;
  fullRenovationWithNewBuildRequirements?: boolean;
  energyPerformanceFee?: boolean;
  bengRequirementProof?: boolean;
  previousDetailedRegistration?: boolean;
  addedAfter2021?: boolean;
}

export type NtaEvidenceKind =
  | 'photo_overview' | 'photo_detail' | 'invoice' | 'drawing' | 'datasheet'
  | 'declaration_of_performance' | 'quality_declaration' | 'client_statement' | 'other';

/** One evidence file; `evidence:<id>` in a `…Reference` field links input to it. */
export interface NtaEvidenceItem {
  id: string;
  kind: NtaEvidenceKind;
  fileName: string;
  /** SHA-256, lowercase hex. */
  sha256: string;
  /** YYYY-MM-DD. */
  date?: string;
  gps?: { latitude: number; longitude: number };
  sourceParty?: string;
  checkedBy?: string;
  description?: string;
  /** JSON pointers into the project, e.g. `/zones/0/surfaces/2`. */
  linkedPaths?: string[];
  /** Local copy in the desktop app's data folder. */
  storedPath?: string;
}

export interface RelabelChange {
  path: string;
  before: unknown;
  after: unknown;
  verdict: 'allowed' | 'not_allowed' | 'review';
  cluster: string;
  note?: string;
}

/** BRL 9500 Bijlage 6a/6b classification of the changes since the original label. */
export interface RelabelAssessment {
  source: string;
  allowed: boolean;
  needsReview: boolean;
  changes: RelabelChange[];
}

export async function assessRelabelWithRust(original: unknown, current: unknown): Promise<RelabelAssessment> {
  if (isTauri()) {
    return invoke<RelabelAssessment>('assess_relabel', { original, current });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/relabel/assess', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ original, current }),
    });
    if (!response.ok) throw new Error(`Rust API: HTTP ${response.status}`);
    return response.json() as Promise<RelabelAssessment>;
  }
  throw new Error('Relabel classification is available in the desktop app and local development server.');
}

export interface RegistrationAssessment {
  source: string;
  validUntil: string | null;
  registrationDeadline: string | null;
  relabelDeadline: string | null;
  readyForRegistration: boolean;
  issues: Array<{ code: string; path: string; severity: 'error' | 'missing' }>;
}

export interface LabelData {
  source: string;
  envelope: Array<{
    category: 'facade' | 'roof' | 'floor' | 'glazing';
    areaM2: number;
    meanUWPerM2k: number | null;
    minRcM2kPerW: number | null;
    maxRcM2kPerW: number | null;
  }>;
  installations: {
    heatingGenerator: string | null;
    hotWaterGenerator: string | null;
    ventilationSystems: string[];
    coolingGenerators: string[];
    pvSystemCount: number;
    lightingZoneCount: number;
  };
}

export async function calculateProjectPerformanceWithRust(project: IProject): Promise<ProjectPerformanceAssessment> {
  if (isTauri()) {
    return invoke<ProjectPerformanceAssessment>('calculate_project_performance', { project });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/project/performance', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ project }),
    });
    return response.json() as Promise<ProjectPerformanceAssessment>;
  }
  throw new Error('Rust project calculation is available in the desktop app and local development server.');
}

/** Chapter 13 result; see crates/nta8800-core/src/domestic_hot_water.rs. */
export interface NtaHotWaterAssessment {
  annualNetNeedKwh: number;
  emissionEfficiency: number;
  annualGeneratorOutputKwh: number;
  annualSolarRenewableKwh: number;
  annualSolarSpaceHeatingKwh: number;
  months: Array<{
    month: number;
    netNeedKwh: number;
    generatorOutputKwh: number;
    generationEfficiency: number;
    carrierInputKwh: number;
    electricityKwh: number;
    naturalGasKwh: number;
    oilKwh: number;
    districtHeatKwh: number;
    auxiliaryElectricityKwh: number;
    ambientHeatKwh: number;
    recoverableLossKwh: number;
    solarRenewableKwh: number;
    solarSpaceHeatingKwh: number;
    solarAuxiliaryKwh: number;
    solarBackupStorageLossKwh: number;
    solarRecoverableKwh: number;
    extraElectricOutputKwh: number;
  }>;
  generators: Array<{ index: number; order: number; monthlyOutputKwh: number[]; monthlyShare: number[] }>;
}
