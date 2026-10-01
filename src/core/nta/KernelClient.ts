import { invoke, isTauri } from '@tauri-apps/api/core';
import type { IProject, INtaHeatPumpInput } from '../energy/types';

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
export type NtaObstruction =
  | { method: 'minimal' }
  | { method: 'declared'; heating: number[]; cooling: number[]; sourceReference: string };
export interface NtaMovableShading {
  reductionFactor: number;
  control: 'manual_residential' | 'automatic';
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
        adjustedConductanceWPerK: number;
        heatingKwh: number[];
        coolingKwh: number[];
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
          reductionFactor: number;
          factorSourceReference: string;
          boundary: NtaDirectTransmissionInput;
        }>;
      };
      groundFloors: Array<{
        id: string;
        areaM2: number;
        exposedPerimeterM: number;
        constructionResistanceM2kPerW: number;
        sourceReference: string;
      }>;
      groundInventoryConfirmed: boolean;
    };

export interface MonthlyDemandInput {
  zoneId: string;
  usableFloorAreaM2: number;
  areaSourceReference: string;
  setpoints: { heatingC: number; coolingC: number; sourceReference: string };
  transmission: MonthlyDemandTransmission;
  ventilationFlows: Array<{
    id: string;
    sourceReference: string;
    months: Array<{ month: number; conductanceWPerK: number; supplyTemperatureC?: number | null }>;
  }>;
  thermalMass: {
    floor: NtaMassClass;
    wall: NtaMassClass;
    ceiling: 'closed_or_suspended' | 'open_or_none';
    sourceReference: string;
  };
  internalGains:
    | { method: 'residential'; dwellingCount: number; sourceReference: string }
    | { method: 'declared'; heatFluxWPerM2: number; sourceReference: string };
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
    groundConductanceWPerK: number;
    annualMeanOutdoorTemperatureC: number | null;
  };
  monthly: Array<{
    month: number;
    hours: number;
    outdoorTemperatureC: number;
    timeConstantH: number;
    a: number;
    internalGainsKwh: number;
    windowSolarGainsKwh: number;
    windowSolarCoolingKwh: number;
    opaqueSolarGainsKwh: number;
    heating: MonthlyDemandBalanceTerms;
    cooling: MonthlyDemandBalanceTerms;
  }>;
  annualHeatingNeedKwh: number | null;
  annualCoolingNeedKwh: number | null;
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
  demand: MonthlyDemandInput;
  additionalZones?: SpaceHeatingChainZone[];
  emission: {
    system: 'radiators_or_convectors' | 'floor_heating' | 'fan_assisted_radiators_or_convectors' | 'air_heating' | 'other_or_unknown';
    balancing: 'none_or_unknown' | 'static' | 'dynamic' | 'not_applicable';
    control: 'main_room_thermostat' | 'central_with_room_valves' | 'individual_room_thermostats' | 'other_or_unknown';
    sourceReference: string;
  };
  distribution:
    | { method: 'heated_zone_only_space_heating'; sourceReference: string }
    | { method: 'declared'; monthlyLossKwh: number[]; sourceReference: string };
  generator:
    | { kind: 'gas_boiler'; boiler: BoilerForfaitDraftInput }
    | {
        kind: 'heat_pump_forfait';
        forfait: ForfaitHeatPumpDraftInput;
        sourceSystem: 'individual' | 'collective_ground' | 'collective_groundwater_surface_or_at_least15_c';
        sourceSystemReference: string;
        auxiliaryMeasurements?: Record<string, unknown> | null;
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
    | { kind: 'external_heat'; supplierReference: string; qualityDeclarationPresent: boolean }
    | { kind: 'electric_resistance'; equipmentReference: string }
    | {
        kind: 'biomass';
        appliance: 'freestanding_wood_stove' | 'insert_stove' | 'pellet_stove' | 'accumulating_stove' | 'central_boiler';
        location: 'inside_thermal_boundary' | 'outside_thermal_boundary';
        annexRCompliantAtMost500Kw: boolean;
        annexRReference: string;
        equipmentReference: string;
      };
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
    generatorOutputKwh: number;
    heatPumpOutputKwh: number;
    naturalGasKwh: number;
    districtHeatKwh: number;
    biomassKwh: number;
    generatorElectricityKwh: number;
    auxiliaryElectricityKwh: number | null;
    collectiveSourceHeatKwh: number;
  }>;
  annualNaturalGasKwh: number | null;
  annualGeneratorElectricityKwh: number | null;
  annualAuxiliaryElectricityKwh: number | null;
  annualCollectiveSourceHeatKwh: number | null;
  annualDistrictHeatKwh: number | null;
  annualBiomassKwh: number | null;
  demand: MonthlyDemandAssessment;
  additionalZoneDemands: MonthlyDemandAssessment[];
  issues: Array<{ code: string; path: string }>;
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

export interface BuildingPerformanceInput {
  calculationScope: 'residential' | 'utility';
  totalUsableFloorAreaM2: number;
  areaSourceReference: string;
  spaceHeating: SpaceHeatingChainInput;
  heatPumpRenewable?: { sourceBelow20C: boolean; exhaustAirSource: boolean; sourceReference: string } | null;
  bacsFactor: 1 | 1.05;
  bacsSourceReference: string;
  useInventoryComplete: boolean;
  declaredUses: Array<{
    id: string;
    service:
      | 'domestic_hot_water' | 'domestic_hot_water_auxiliary' | 'ventilation_fans'
      | 'space_cooling' | 'space_cooling_auxiliary' | 'lighting' | 'pv_auxiliary';
    carrier: 'el' | 'gas' | 'oil';
    monthlyKwh: number[];
    sourceReference: string;
  }>;
  declaredRenewableHeat?: Array<{ id: string; monthlyKwh: number[]; sourceReference: string }>;
  productionInventoryComplete: boolean;
  onSiteProduction: Array<{ id: string; kind: 'pv' | 'pvt' | 'wind'; monthlyKwh: number[]; sourceReference: string }>;
  pvSystems?: Array<{
    id: string;
    peakPowerKw: number;
    azimuthDeg: number;
    tiltDeg: number;
    performanceFactor: 0.76 | 0.8 | 0.82;
    shadingCorrection: number;
    obstructionFactor: number;
    sourceReference: string;
  }>;
  hotWater?: NtaHotWaterSystem | null;
  demandUsesFixedC1Ventilation: boolean;
  batteryStoragePresent: boolean;
}

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
  function: NtaBblFunction;
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
    tojuliK: number | null;
  }>;
  maxTojuliK: number | null;
  meetsBblLimit: boolean | null;
  issues: Array<{ code: string; path: string }>;
}

export interface NtaCoolingSystem {
  emissionEfficiency: number;
  distributionEfficiency: number;
  controlFactor: number;
  efficiencySourceReference: string;
  generator:
    | { kind: 'compression' }
    | { kind: 'gas_absorption' }
    | {
        kind: 'free_cooling';
        source: 'aquifer_dwellings_from2013' | 'aquifer_utility_before2013' | 'aquifer_dwellings_before2013'
          | 'surface_water' | 'closed_ground_loop' | 'dew_point_cooling';
        freeCoolingFraction: number;
        fractionSourceReference: string;
      };
  equipmentReference: string;
}

export interface NtaHotWaterSystem {
  need:
    | { method: 'residential'; dwellingCount: number; sourceReference: string }
    | { method: 'declared'; specificNeedKwhPerM2Year: number; sourceReference: string };
  emissionEfficiency: number;
  distributionEfficiency: number;
  generationEfficiency: number;
  carrier: 'el' | 'gas' | 'oil';
  showerHeatRecoveryKwh?: number[];
  auxiliaryElectricityKwh?: number[];
  renewableHeatPump: boolean;
  efficiencySourceReference: string;
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
  carriers: Array<{ carrier: 'el' | 'gas' | 'oil' | 'dh' | 'bm'; month: number; usedKwh: number; deliveredKwh: number }>;
  electricityBalance: Array<{ month: number; usedKwh: number; producedKwh: number; selfUsedKwh: number; exportedKwh: number }>;
  annualPrimaryFossilKwh: number | null;
  annualRenewablePrimaryKwh: number | null;
  annualHeatPumpAmbientHeatKwh: number | null;
  annualHeatingAndCoolingNeedKwh: number | null;
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
    sourceReference: string;
  }>;
  ventilationFlows: MonthlyDemandInput['ventilationFlows'];
  zoneData?: Array<{
    zoneId: string;
    ventilationFlows: MonthlyDemandInput['ventilationFlows'];
    internalGains: MonthlyDemandInput['internalGains'];
    setpoints?: MonthlyDemandInput['setpoints'];
    thermalMass?: MonthlyDemandInput['thermalMass'];
    emission?: SpaceHeatingChainInput['emission'];
    distribution?: SpaceHeatingChainInput['distribution'];
  }>;
  emission: SpaceHeatingChainInput['emission'];
  distribution: SpaceHeatingChainInput['distribution'];
  generator: SpaceHeatingChainInput['generator'];
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
  cooling?: NtaCoolingSystem | null;
  labelFunction?: NtaLabelFunction | null;
  bblFunction?: NtaBblFunction | null;
  activeCoolingPresent?: boolean;
  permitApplicationAfter20260529?: boolean;
  demandUsesFixedC1Ventilation: boolean;
  batteryStoragePresent: boolean;
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
    lossAreaM2: number;
    lossAreaRatio: number | null;
    unclassifiedSurfaceCount: number;
  };
  derivedInput: BuildingPerformanceInput | null;
  performance: BuildingPerformanceAssessment | null;
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
