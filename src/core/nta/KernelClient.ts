import { invoke, isTauri } from '@tauri-apps/api/core';
import type { IProject, INtaHeatPumpInput } from '../energy/types';
import { kernelProject } from './KernelInput';

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
/** ISSO 82.1/75.1 survey heating generator (opname/heating.rs). */
export type OpnameHeatingGenerator =
  /** `oil`: oil-fired central boiler, conventional per NTA table 9.25 (no pilot flame). */
  | { kind: 'boiler'; boilerType: 'conventional' | 'vr' | 'hr100' | 'hr104' | 'hr107' | 'hydrogen' | 'oil'; pilotFlame?: boolean | null; insideThermalBoundary: boolean; manufactureYear?: number; installationYear?: number }
  /**
   * Table 9.6: `heat_pump_panel` takes the outdoor-air row (NTA p. 336); groundwater or a collective
   * source without `sourceTemperatureC` takes the ground row (NTA p. 335); `high_temperature` is a
   * collective source only. Gas-driven heat pumps use the GWP rows of tables 9.27/9.29.
   */
  | {
    kind: 'heat_pump';
    source: 'outdoor_air' | 'exhaust_air' | 'outdoor_and_exhaust_air' | 'ground' | 'groundwater' | 'surface_water'
      | 'water_based_unknown' | 'heat_pump_panel' | 'high_temperature';
    airSink?: boolean; highTemperature?: boolean; capacityKw?: number | null; sourceRegenerationFactor?: number | null;
    highEfficiencyEvidence?: unknown;
    drive?: 'electric' | 'gas_engine' | 'gas_absorption';
    /** Table 9.6; unknown → recirculation (doublet: c_source 1,04 for a collective source, table V.3). */
    groundwaterSystem?: 'doublet' | 'recirculation' | null;
    /** Invoices or design data of a collective water-based source; absent: individual source. */
    collectiveSourceReference?: string | null;
    sourceTemperatureC?: number | null;
    sourceTemperatureReference?: string | null;
    /** Quality declaration of a source of 20 °C or more (otherwise the groundwater row). */
    sourceQualityDeclarationReference?: string | null;
  }
  /** Table 9.3 / NTA table 9.25: local gas heating incl. pilot, oil heating or a steam boiler (0,65 with flue, 0,10 without). */
  | { kind: 'local_fired'; appliance: 'gas_heater' | 'oil_heater' | 'steam_boiler'; fuel?: 'natural_gas' | 'oil' | null; flueGasExhaust: boolean; electricityConnected?: boolean | null }
  /** Table 9.3: direct-fired gas air heaters; pilot flame unknown → present; count unknown → the table 9.16 count, else 1. */
  | { kind: 'gas_air_heater'; heaterType: 'conventional' | 'vr' | 'hr100' | 'hr104' | 'hr107'; pilotFlame?: boolean | null; count?: number | null }
  | { kind: 'district_heat' }
  | { kind: 'electric'; connectedDevices: number }
  | { kind: 'biomass'; appliance: 'freestanding_wood_stove' | 'insert_stove' | 'pellet_stove' | 'accumulating_stove' | 'central_boiler'; insideThermalBoundary: boolean; soleHeatingInServedRooms: boolean; annexRCompliant?: boolean | null }
  /** Building CHP (NTA table 9.31); thermal power unknown → table 9.7 rule by engine (gas 1,5, diesel 1,2, micro turbine 2,5). */
  | { kind: 'chp'; electricalPowerKw: number; thermalPowerKw?: number | null; engine?: 'gas_engine' | 'diesel_engine' | 'micro_turbine' | null; manufactureYear?: number | null; hreDeclared?: boolean; lowTemperature?: boolean }
  | { kind: 'none_present' };

/** ISSO 82.1 survey hot-water generator (opname/hot_water.rs). */
export type OpnameHotWaterGenerator =
  | { kind: 'none' | 'electric_boiler' | 'electric_instantaneous' | 'district_heat' | 'collective_unknown' | 'delivery_set_from_heating' }
  | { kind: 'gas_appliance'; applianceType: 'bath_geyser' | 'combi' | 'kitchen_geyser' | 'unknown'; gaskeur: 'none' | 'gaskeur' | 'gaskeur_cw' | 'gaskeur_hr_cw' | 'unknown'; burnerLoadKw?: number; cwClass?: 'cw1' | 'cw2' | 'cw3' | 'cw4_to6' | 'unknown' | null }
  | { kind: 'heat_pump'; exhaustAirSource: boolean };

/** Solar water heater in the survey (§15.3–15.4, tables 15.4/15.5/15.8). */
export interface OpnameSolarWaterHeater {
  id: string;
  collector: 'unglazed' | 'glazed' | 'evacuated_tube' | 'unknown';
  collectorAreaM2: number;
  /** Gross area: evacuated tubes count 60 % (p. 192). */
  grossArea?: boolean;
  collectorCount?: number;
  orientation: 'north' | 'north_east' | 'east' | 'south_east' | 'south' | 'south_west' | 'west' | 'north_west';
  tiltDeg: number;
  shading?: NtaCollectorObstruction | null;
  backup: 'separate_heater' | 'integrated_gas' | 'integrated_electric' | 'unknown';
  storageVolumeL: number;
  backupVolumeL?: number | null;
  storageLabel?: 'a_plus' | 'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | null;
  storageManufactureYear?: number | null;
  alsoSpaceHeating?: boolean;
  pvt?: 'unglazed' | 'single_glazed' | 'tested_iso9806' | null;
  sourceReference: string;
}

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
export type NtaShadeColour = 'dark' | 'other' | 'white' | 'unknown';
/** Table 7.5/7.6 devices; F_c follows from the table instead of `reductionFactor`. */
export type NtaShadingDevice =
  | { kind: 'external_screen'; colour: NtaShadeColour }
  | { kind: 'external_venetian_blind'; colour: NtaShadeColour }
  | { kind: 'external_roller_shutter'; colour: NtaShadeColour }
  | { kind: 'internal_metallised_fabric' }
  | { kind: 'drop_arm_awning' }
  | { kind: 'folding_arm_awning' };
export interface NtaMovableShading {
  /** Explicit F_c; omit when `device` gives the table value. */
  reductionFactor?: number;
  device?: NtaShadingDevice | null;
  control: NtaShadingControl;
  sourceReference: string;
}
/** 7.41/7.41a/7.41b: table 7.4 glazing, fixed louvres (7.4a) or diffusing glazing (7.4b). */
export interface NtaGlazingSolar {
  glazingType?: 'single' | 'double' | 'single_with_secondary_pane' | 'double_low_e' | 'triple_one_coating' | 'triple_two_coatings' | 'solar_control' | null;
  fixedLouvres?: { kind: 'horizontal90' } | { kind: 'horizontal_angled' } | { kind: 'rotatable'; control: NtaShadingControl } | null;
  diffusing?: { gAltitude45: number; gDiffuse: number; sourceReference: string } | null;
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
      verticalPipes?: Array<{ id: string; storeys?: number; buildingHeightM?: number | null; areaShare?: number | null; insulated: boolean; sharedZones?: number; sourceReference: string }>;
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
  /** Omit when `wallDepths` gives z_j per wall part (8.42/D.12). */
  depthM?: number;
  wallDepths?: Array<{ lengthM: number; depthM: number }>;
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
  /** 12.2.1 (p. 521): A_g served by the humidification generator for the 500 m² limit; absent: the heating-system area. */
  servedAreaM2?: number | null;
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
    /** Omit when `glazing.glazingType` gives the table 7.4 value. */
    gPerpendicular?: number;
    glazing?: NtaGlazingSolar | null;
    frameFraction: number;
    uValueWPerM2k: number;
    /** ΔU_for (8.3) carried by H_D only; the solar terms keep U_c. */
    forfaitDeltaUWPerM2k?: number | null;
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
    /** ΔU_for (8.3) carried by H_D only; the solar terms keep U_c. */
    forfaitDeltaUWPerM2k?: number | null;
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
  /** θ_int;op;H/C (7.9.6): θ_int;calc;H for heating, 7.80/7.81 for cooling; null when H_C;ht is undefined. */
  operativeTemperatureC: number | null;
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

export type NtaDispatchDesignContext = 'new_build' | 'existing' | 'existing_added_preferred';

export interface GeneratorDispatchDraftInput {
  nodeInputKwh: Array<{ month: number; energyKwh: number }>;
  nodeInputReference: string;
  /** new_build/existing: 9.56/9.57; existing_added_preferred: 9.58/9.59. */
  designContext: NtaDispatchDesignContext;
  /** f_gebouw;si;H for 9.58 (default 1). */
  buildingFraction?: number | null;
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
  /** `unknown`: collective only (table 9.25 a), W_aux = 0 per 9.6.8.2.2). */
  kind: 'conventional' | 'vr' | 'hr100' | 'hr104' | 'hr107' | 'unknown';
  /** Oil boilers are `conventional`. */
  fuel: 'natural_gas' | 'oil';
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

export async function assessProjectWithRust(input: IProject): Promise<KernelAssessment> {
  const project = kernelProject(input);
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
  /** 13.68 Q_H;sol;ls;rbl of solar systems for space heating only, kWh per month (into 7.3). */
  solarRecoverableKwh?: number[];
  /** 13.185 hot water made with heat from this system (§13.8.4.9.3), kWh per month. */
  hotWaterLoadKwh?: number[];
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
    /** 9.23 with tables 9.12/9.13: fans and controls of air heaters. */
    airHeaters?: {
      kind:
        | { kind: 'direct'; radialFan?: boolean | null }
        | { kind: 'indirect'; roomHeightAbove8M?: boolean | null; warmAirReturn?: boolean | null; ecMotor?: boolean | null };
      /** Q_h;b per EN 12831-1, W; absent means the table 9.12 estimate. */
      designHeatLoadW?: number | null;
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
  /** §9.1: number of identical physical generators modelled as one system. */
  identicalSystems?: number | null;
  /** Annex V V.1: hot water of a heat pump on the same ground source. */
  regenerationHotWater?: { annualKwh: number; generationEfficiency: number } | null;
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
        /** Annex V regeneration: c_source of table 9.27 footnote a from table V.1. */
        regeneration?: { freeCoolingFromSource?: boolean; solar?: Array<Record<string, unknown>>; sourceReference: string } | null;
        auxiliaryMeasurements?: Record<string, unknown> | null;
        auxiliary?: NtaOtherGeneratorAuxiliary | null;
      }
    | {
        /** Gas-engine or gas-absorption heat pump, tables 9.27/9.29 (≤ 55 °C). */
        kind: 'gas_heat_pump';
        table: 'residential_at_most25_kw' | 'utility_collective_or_above25_kw';
        source: 'ground' | 'groundwater' | 'outdoor_air' | 'exhaust_air' | 'surface_water';
        designSupplyTemperatureC: number;
        sourceCorrectionFactor?: number | null;
        auxiliary?: NtaOtherGeneratorAuxiliary | null;
        equipmentReference: string;
      }
    | {
        kind: 'hybrid_heat_pump';
        designContext: NtaDispatchDesignContext;
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
        /** Biomass above 500 kW per installation: bmA (table 5.2). */
        biomassAbove500Kw?: boolean;
      }
    | {
        /** Annex N: local, air or radiant heater or stove. */
        kind: 'local_heater';
        heater: NtaLocalHeater;
        fuel: 'natural_gas' | 'oil' | 'biomass';
        annexRCompliantAtMost500Kw?: boolean | null;
        annexRReference?: string | null;
        biomassAbove500Kw?: boolean;
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
        /** 9.91; without `nominalPowerKw` 9.92 runs the burner the whole month (upper bound). */
        auxiliary?: NtaOtherGeneratorAuxiliary | null;
      }
    | {
        /** 9.6.6.1 building CHP, method 2 (table 9.31), gas; electricity per 16.12. */
        kind: 'chp';
        /** Method 2 (9.6.6.1, table 9.31); exclusive with `method1`. */
        chp?: { powerKw: number; builtAfter2006: boolean; hreDeclared?: boolean; lowTemperature?: boolean } | null;
        /** Method 1 (9.6.6.2): micro-CHP with NEN-EN 50465 test values. */
        method1?: NtaMicroChp | null;
        auxiliary?: NtaOtherGeneratorAuxiliary | null;
        equipmentReference: string;
      }
    | {
        /** 9.6.1: several unequal generators split by preference (9.56–9.60, table 9.23). */
        kind: 'multiple';
        generators: Array<{
          preference: number;
          /** May be omitted with `estimatedBeta`. */
          nominalPowerKw?: number;
          /** Any single generator of this union except `multiple` and `hybrid_heat_pump`. */
          generator: { kind: string } & Record<string, unknown>;
        }>;
        /** 9.58/9.59: renovation with an added preferred generator. */
        addedPreferredGenerator?: boolean;
        /** 9.6.1 note 1: estimated cumulative β for preferences 1 … n−1 when powers are unknown. */
        estimatedBeta?: number[];
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
    /** 9.4: the W.2 heat loads the node of the building's space-heating chain. */
    | { kind: 'heating_system' }
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
  /** b_U of the unheated space with the pipes: ϑ_ztu per 7.82 when no ϑ_ztu is entered. */
  unheatedReductionFactor?: number | null;
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
        /** One-pipe loop: emitters in series, each adding its table 9.21 resistance (p. 317). */
        onePipeEmitterCount?: number | null;
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
      /** E.8/E.9 F_M from the moisture content; replaces table E.2. */
      moistureConversion?: { basis: 'volume' | 'mass'; conversionCoefficient: number; moistureContent: number; sourceReference: string } | null;
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
  flowReduction?: { collective?: boolean; recirculationPercent?: number; flowControlPercent?: number; evidenceReference?: string };
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
    /** 13.149 f_W;t;hp-on per month; leave out to derive it from the hot-water system. */
    hotWaterTimeFraction?: number[];
    heatingFlowM3PerH?: number;
    heatingAreaShare?: number;
    /** 13.148/13.148a q_ve;hp;W per month; leave out to derive it from the hot-water system. */
    hotWaterFlowM3PerH?: number[];
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
  /** ρ·c·Σq without b_v (the supply temperatures carry b_v); not H_ve of 7.19. */
  conductanceWPerK: number;
  /** H_ve of 7.19 with b_v: heat flow per setpoint / (θ_set − θ_e). Absent in older payloads. */
  weightedConductanceWPerK?: number | null;
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

export type OpnameCo2Measurement = 'none' | 'living_room' | 'living_room_and_main_bedroom' | 'every_habitable_room';
export type OpnameControlTarget = 'none' | 'supply' | 'extract' | 'supply_and_extract';

/** ISSO 82.1 tables 11.4–11.6; unknown answers count as none. */
export interface OpnameVentilationControls {
  co2Measurement?: OpnameCo2Measurement | null;
  co2Control?: OpnameControlTarget | null;
  timeControl?: OpnameControlTarget | null;
  zoning?: boolean | null;
  /** System C: separate extract points in every habitable room (C.5b). */
  extractPerHabitableRoom?: boolean | null;
  evidenceReference: string;
}

/** ISSO 82.1 §11.3.7 (NTA 11.123); all four settings or the 11.124 fallback. */
export interface OpnameGrilleHeatingStrips {
  maxPowerWPerDm3PerS?: number | null;
  maxTemperatureRiseK?: number | null;
  switchOnBelowC?: number | null;
  maxSupplyTemperatureC?: number | null;
  sourceReference: string;
}

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
    // floor: positions 1/2/5/6, 3/7 or 4/8 ("dak + vloer") of ISSO 82.1 afb. 7.1.
    | { kind: 'apartment'; floor: 'ground_or_intermediate' | 'top' | 'roof_and_floor'; side: 'middle' | 'end_or_corner' | 'unknown' };
  usableFloorAreaM2: number;
  areaSourceReference: string;
  buildingHeightM: number;
  construction: {
    floor: 'light' | 'heavy' | 'very_heavy';
    wall: 'light' | 'heavy' | 'very_heavy';
    lighterCeiling?: boolean;
    /** Closed or suspended ceiling, any floor type (ISSO 75.1 table 7.5 first column). Utility only: the dwelling survey rejects it (82.1 table 7.4). */
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
        // sunroom: AOS as outdoor air (ISSO 82.1 §6.3.4); water: houseboat hull.
        | { kind: 'outdoor' | 'ground' | 'crawlspace' | 'adjacent_heated' | 'unheated_cellar' | 'strongly_ventilated' | 'sunroom' | 'water' }
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
      /** Utility survey with `zones`: the calculation zone; absent: split over the zones by A_g. */
      zoneId?: string | null;
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
      /** Solar-control glass or film: g from the product data or quality declaration (82.1 p. 94). */
      solarControl?: { gValue: number | null; sourceReference: string } | null;
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
    /** Rooflights with a BCRG quality declaration (ISSO 82.1 p. 68): A_rc and U_rc. */
    rooflights?: Array<{
      id: string;
      surfaceId: string;
      areaM2: number;
      uValue: number;
      glass: OpnameGlass;
      qualityDeclarationReference: string;
    }>;
    /** Caravan or houseboat (ISSO 82.1 p. 49; NTA tables I.5–I.7); absent: regular. */
    buildingKind?: { kind: 'regular' | 'caravan' } | { kind: 'floating'; newBerthSince2018: boolean } | null;
  };
  heating: {
    generator: OpnameHeatingGenerator;
    emitters: 'radiators' | 'low_temperature_radiators' | 'floor_heating' | 'floor_heating_and_radiators' | 'air_heating' | 'local_heaters';
    designClass?: 'c45_40' | 'c55_47' | 'c70_50' | 'c90_70' | null;
    /** Controlled declaration for a heat pump above 70 °C (table 9.9, erratum §4). */
    heatPumpAbove70Declaration?: string | null;
    balanced?: boolean | null;
    control: 'room_thermostat' | 'central_with_radiator_valves' | 'individual_room_control' | 'unknown';
    /** Afb. 9.1; absent: present with the forfait length when unheated spaces exist. */
    unheatedPipes?: { kind: 'absent' } | { kind: 'present'; lengthM?: number | null } | null;
    storeys?: number;
    /** Table 9.7 nominal power of the main generator, kW; required with additionalGenerators. */
    nominalPowerKw?: number | null;
    /** §9.3.2: further unequal generators (kernel `multiple`, preference per p. 112). */
    additionalGenerators?: Array<{ generator: OpnameHeatingGenerator; nominalPowerKw?: number | null }>;
    /** §9.3.6: a preferred generator added after delivery. */
    addedPreferredGenerator?: boolean;
    /** Collective installation (p. 106, 121–122). */
    collective?: {
      connectedUsableAreaM2?: number | null;
      connectedDwellings?: number | null;
      connectedStoreys?: number | null;
      heatMetersPresent?: boolean | null;
    } | null;
    /** ISSO table 9.16 with emitters `air_heating`; absent: type unknown (no fans). */
    airHeating?: OpnameAirHeating | null;
    /** §9.4.2; absent: two-pipe. A one-pipe loop sums its emitter resistances (NTA table 9.21). */
    distributionType?: { kind: 'two_pipe' | 'renovated_one_pipe' } | { kind: 'one_pipe'; emitterCount: number } | null;
    /** Table 9.12; absent: not insulated. Year unknown → construction year; fittings unknown → not insulated. */
    pipeInsulation?: { insulated: boolean; insulationYear?: number | null; fittingsInsulated?: boolean | null } | null;
    sourceReference: string;
  };
  hotWater: {
    generator: OpnameHotWaterGenerator;
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
    /** 13.141 nominal power of the main generator, kW. */
    nominalPowerKw?: number | null;
    /** NTA 13.8.2 further generators. */
    additionalGenerators?: Array<{ generator: OpnameHotWaterGenerator; nominalPowerKw?: number | null }>;
    /** Collective hot-water system (p. 164, 176). */
    collective?: { buildingUsableAreaM2?: number | null; connectedDwellings?: number | null } | null;
    /** Solar water heaters (§15.3–15.4). */
    solar?: OpnameSolarWaterHeater[];
    /** p. 164 / NTA 13.19a; unknown follows `served`. */
    connectedBathrooms?: number | null;
    connectedKitchens?: number | null;
    sourceReference: string;
  };
  /** Further hot-water systems, e.g. a kitchen geyser (ISSO 82.1 p. 164). */
  additionalHotWaterSystems?: Array<ResidentialSurvey['hotWater']>;
  ventilation: {
    principle: 'natural' | 'mechanical_supply' | 'mechanical_extract' | 'balanced';
    declaredVariant?: VentilationSystemVariant | null;
    selfRegulatingVents?: boolean | null;
    pressureClass?: 'at_most1_pa' | 'from1_to5_pa' | 'from5_to10_pa' | null;
    installationYear?: number | null;
    heatRecovery?: 'counter_flow_aluminium' | 'counter_flow_plastic' | 'counter_flow_unknown_material' | 'cross_flow' | 'plate_or_tube' | 'rotary' | 'enthalpy' | 'heat_pipe' | 'two_element' | 'cold_storage_with_ahu' | 'unknown' | null;
    bypassPresent?: boolean | null;
    unitManufactureYear?: number | null;
    motor?: 'ac' | 'dc' | 'unknown' | null;
    passiveCooling?: OpnamePassiveCooling | null;
    /** Table 11.6: central or decentral heat recovery; absent: central. */
    heatRecoveryLayout?: 'central' | 'decentral' | null;
    /** Tables 11.4–11.6 controls; absent: no control. `declaredVariant` overrides them. */
    controls?: OpnameVentilationControls | null;
    /** System E (§11.3.6): decentral D.5b part; `principle` is the other part. */
    combined?: { decentralAreaM2: number; totalResidenceAreaM2: number } | null;
    /** §11.3.7 grilles with electric heating strips; missing settings: NTA 11.124 fallback. */
    grilleHeatingStrips?: OpnameGrilleHeatingStrips | null;
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
    /** ISSO §15.4.7 / table 16.1 situation; exclusive with `obstructionFactors`. Absent: minimal. */
    shading?: NtaCollectorObstruction;
    sourceReference: string;
  }>;
  /** Building-bound storage (§15.5); requires PV. */
  storage?: OpnameStorage | null;
  coolingPresent?: boolean;
  /** ISSO 82.1 chapter 10: the dwelling's main cooling system (same fields as the utility survey). */
  cooling?: UtilitySurvey['cooling'];
  /** The cooling generator serves several dwellings. */
  coolingCollective?: boolean;
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
      | 'closed_ground_loop' | 'dew_point_cooling' | 'gas_engine_compression';
    capacityKw?: number | null;
    emitter:
      | 'floor_cooling' | 'concrete_core_activation' | 'wall_cooling' | 'ceiling_cooling'
      | 'fan_coil_on_outer_wall' | 'fan_coil_on_ceiling' | 'split_indoor_units_on_wall'
      | 'split_indoor_units_on_ceiling' | 'other';
    fanCoilCount?: number;
    waterBased: boolean;
    designTemperature?: 't6_to12' | 't12_to16' | 't12_to18' | 't17_to21' | null;
    /** Table 10.6; `true`/`false` of older surveys mean static / none. */
    balanced?: 'none' | 'static' | 'dynamic' | boolean | null;
    /** NTA table 10.11 footnote a: NEN-EN 14336 balancing declaration; absent: balancing counts as none. */
    balancingEvidenceReference?: string | null;
    control?: 'standalone' | 'central_with_room_control' | 'other_or_unknown' | null;
    pipesInsulated?: boolean | null;
    pipeInsulationYear?: number | null;
    aquiferPermitYear?: number | null;
    /** NTA 10.84: a heat pump uses this ground storage; unknown follows the heating heat pump. */
    heatPumpSource?: boolean | null;
    /** ISSO 82.1 p. 129: ground source demonstrably always above 0 °C. */
    groundAboveZeroDemonstrated?: boolean;
    /** ISSO 75.1 table 10.2: gas engine of a gas-driven chiller (power required, year unknown: up to 2006). */
    gasEngine?: OpnameGasEngine | null;
    /** §10.4.1: direct expansion (not water-based) in the room or in the AHU; absent: room. */
    directExpansion?: 'room' | 'air_handling_unit' | null;
    /** Table 10.9: valves and brackets fully insulated; unknown: not insulated. */
    fittingsInsulated?: boolean | null;
    /** Table 10.11: cold meters; unknown: present. */
    coldMeters?: boolean | null;
    /** Table 10.10: actual pipe length L and the length through uncooled spaces, m; absent: forfait. */
    pipeLengthM?: number | null;
    uncooledPipeLengthM?: number | null;
    /** Table 10.10: actual maximum supply-pipe length L_max, m; absent: forfait (10.27). */
    maxPipeLengthM?: number | null;
    /** §10.3.2: further generators on the same distribution, each with its nominal power. */
    additionalGenerators?: Array<{
      generator: NonNullable<UtilitySurvey['cooling']>['generator'];
      capacityKw?: number | null;
      aquiferPermitYear?: number | null;
      gasEngine?: OpnameGasEngine | null;
    }>;
    sourceReference: string;
  } | null;
  ventilation: Omit<ResidentialSurvey['ventilation'], 'sourceReference' | 'controls'> & {
    ductsLukaAbc?: boolean | null;
    /** ISSO 75.1 table 11.13; overrides `ductsLukaAbc`. */
    ductAirtightness?: 'luka_abc' | 'luka_d' | 'no_ducts' | 'unknown' | null;
    /** §11.4.1 installed capacity of the zone, dm³/s; absent: regulatory flow. */
    installedCapacityDm3PerS?: number | null;
    /** Table 11.10: the outside connection of the heat-recovery unit; absent: not insulated. */
    supplyDuctInsulation?:
      | { kind: 'uninsulated' | 'insulated' }
      | { kind: 'specified'; thicknessM: number; conductivityWPerMK: number }
      | null;
    supplyDuctLengthM?: number | null;
    /** Table 11.11: constant-volume control at all flows; unknown: none. */
    constantVolumeControl?: boolean | null;
    /** Table 11.12: partial bypass percentage (rounded down to tens); absent: unknown. */
    bypassPercent?: number | null;
    ahu?: {
      insideThermalZone?: boolean | null;
      ductsOutsideThermalZone?: boolean | null;
      ductLength?: 'at_most20_m' | 'from20_to40_m' | 'at_least40_m' | null;
      ductsInsulated?: boolean | null;
      /** p. 149: reheating coil; null: not determinable (not connected). */
      heatingConnected?: boolean | null;
      /** p. 149: cooling coil; requires a cooling system. */
      coolingConnected?: boolean | null;
    } | null;
    recirculation?: 'none' | 'present_percent_unknown' | 'unknown' | null;
    recirculationPercent?: number | null;
    flowControl?: {
      method: 'throttle' | 'inlet_vane' | 'blade_pitch' | 'speed_control' | 'other';
      minimumPercent?: number | null;
    } | null;
    passiveCooling?: OpnamePassiveCooling | null;
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
    /** Solar water heaters (ISSO 75.1 §15.3–15.4). */
    solar?: OpnameSolarWaterHeater[];
    /** 13.141 nominal power of the main generator, kW. */
    nominalPowerKw?: number | null;
    /** NTA 13.8.2 further generators with the utility generator types. */
    additionalGenerators?: Array<{ generator: UtilitySurvey['hotWater']['generator']; nominalPowerKw?: number | null }>;
    /** Areas served by an additional system (13.20a); ignored on the main system. */
    servedAreas?: UtilitySurvey['functions'];
    sourceReference: string;
  };
  /** Further hot-water systems, each with its served areas; the main system serves the rest. */
  additionalHotWaterSystems?: Array<UtilitySurvey['hotWater']>;
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
    /** Calculation zone of this lighting zone; required with `zones`. */
    zoneId?: string | null;
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
  /** ISSO 75.1 §7.1.7: fossil-fuel installations on the plot; absent: not established. */
  fossilFuelOnPlot?: boolean | null;
  /** p. 65: A_g of the sport and swimming halls (sport function, building ≥ 1 000 m²). */
  sportHallAreaM2?: number | null;
  /** p. 65: A_g of the room with a swimming pool (part of the sport function). */
  swimmingPoolAreaM2?: number | null;
  /** Afb. 6.6: residence areas openly connected (no split on ventilation capacity). */
  openlyConnectedResidenceAreas?: boolean;
  /**
   * ISSO 75.1 §6.5 (p. 52–54): two or more calculation zones, each with its use
   * functions; per function the zones add up to `functions`. Empty: one zone.
   */
  zones?: Array<{
    id: string;
    functions: UtilitySurvey['functions'];
    /** p. 147: installed ventilation capacity of the zone, dm³/s; empty: share of the building's by A_g. */
    installedCapacityDm3PerS?: number | null;
    /** p. 65: swimming-pool room in this zone, m²; the zones add up to `swimmingPoolAreaM2`. */
    swimmingPoolAreaM2?: number | null;
    /** p. 145: system E areas of this zone; zones without it have no decentral part. */
    combined?: { decentralAreaM2: number; totalResidenceAreaM2: number } | null;
  }>;
  sourceReference: string;
  /** Adviser's reason per applied default (path or rule) for the forfait (BRL 9500 §4.2.2). */
  inklapRedenen?: Record<string, string>;
}

/** ISSO 75.1 table 10.2: the gas engine of a gas-driven chiller. */
export interface OpnameGasEngine {
  /** Manufactured from 2007; unknown: up to 2006. */
  from2007?: boolean | null;
  electricPowerKw?: number | null;
  hreDeclared?: boolean;
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

export type NtaHeatPumpEfficiency =
  /** `sourcePumpIncluded`: P.6.8.4.3 (p. 1009), the source pump or fan is part of the declared efficiency (0 W/kW). */
  | { method: 'declared'; value: number; sourceReference: string; sourcePumpIncluded?: boolean }
  | { method: 'table_p5'; source: NtaTableP5Source; supplyTemperatureC: number };

export type NtaTemperatureLevel = 'low' | 'high';

/** Table P.6: CHP conversion numbers by electrical power and build year. */
export interface NtaTableP6 {
  electricalPowerKw: number;
  installedAfter2006: boolean;
  /** Table P.4; absent means HT. */
  temperatureLevel?: NtaTemperatureLevel;
}

type NtaSolarCalculatedMethod = Extract<NtaSolarWaterHeater['method'], { method: 'calculated' }>;

export type NtaAnnexPGenerator =
  | { kind: 'combustion'; carrier: NtaAnnexPCarrier; efficiency: number; efficiencyReference: string }
  | { kind: 'heat_pump'; efficiency: NtaHeatPumpEfficiency; drive: NtaAnnexPCarrier }
  | {
      kind: 'chp_without_loss';
      carrier: NtaAnnexPCarrier;
      /** Declared values override table P.6. */
      thermalEfficiency?: number;
      electricalEfficiency?: number;
      efficiencyReference?: string;
      tableP6?: NtaTableP6;
    }
  | { kind: 'chp_with_loss'; carrier: NtaAnnexPCarrier; lossRatio?: number; lossRatioReference?: string }
  | { kind: 'residual_heat'; auxiliarySpecific?: number; auxiliaryReference?: string }
  | { kind: 'geothermal'; sourceTemperatureC: number; returnTemperatureC: number }
  | { kind: 'declared'; primaryFactor: number; co2KgPerKwh: number; renewableFactor: number; sourceReference: string }
  /** P.6.5.4.2/P.6.6.5.2 with tables P.3/P.4. */
  | {
      kind: 'boiler';
      carrier: NtaAnnexPCarrier;
      efficiency:
        | {
            method: 'table_p3';
            boiler: 'conventional' | 'vr' | 'hr100' | 'hr104' | 'hr107';
            temperatureLevel?: NtaTemperatureLevel;
            emission?: {
              averageDesignTemperatureC: number;
              system: 'mixing_without_return_limit' | 'mixing_with_return_limit' | 'direct';
            };
          }
        | { method: 'full_load'; value: number; outdoorInstallation?: boolean; sourceReference: string };
    }
  /** P.6.5.4.3: net efficiency per NEN-EN 303-5. */
  | { kind: 'solid_biomass_boiler'; carrier: NtaAnnexPCarrier; netEfficiency: number; sourceReference: string }
  /** P.6.5.4.10: always preferred, F = ΣQ_sol;mi / Q_in;tot. */
  | {
      kind: 'collective_solar';
      contribution:
        | { method: 'declared'; monthlyKwh?: number[]; annualKwh?: number; sourceReference: string }
        | {
            method: 'calculated';
            solarType: 'preheater' | 'integrated_backup';
            collectors: NtaSolarCalculatedMethod['collectors'];
            storage: NtaSolarCalculatedMethod['storage'];
            networkSupplyC: number;
            networkReturnC: number;
            storageAmbientC: number;
          };
    }
  /** P.6.5.4.11 with tables 5.5/5.6; the thermal power is the generator's `nominalPowerKw`. */
  | {
      kind: 'electric_flex';
      generator:
        | { kind: 'electrode_boiler'; efficiency?: number; efficiencyReference?: string }
        | { kind: 'heat_pump'; efficiency: NtaHeatPumpEfficiency };
      flexHeatKwh?: number;
      flexReference?: string;
      /** Whole-network heat (heating and hot water) for the 15 % cap. */
      networkProductionKwh?: number;
      connections: number;
      heatBuffer: boolean;
      registrationReference: string;
    }
  /** Table P.9. */
  | {
      kind: 'compression_chiller';
      variant:
        | 'unspecified' | 'high_temperature_emission' | 'wet_cooling'
        | 'high_temperature_emission_and_wet_cooling' | 'low_temperature_source'
        | 'high_temperature_emission_and_low_temperature_source';
      drive: NtaAnnexPCarrier;
      engineEfficiency?:
        | { method: 'declared'; value: number; sourceReference: string }
        | ({ method: 'table_p6' } & NtaTableP6);
    }
  | {
      kind: 'free_cooling';
      source:
        | 'aquifer_storage_before2013' | 'aquifer_storage_from2013' | 'aquifer_recirculation'
        | 'aquifer_without_heat_use' | 'other_low_temperature_source';
      drive: NtaAnnexPCarrier;
    }
  /** Table P.10. */
  | {
      kind: 'sorption_chiller';
      heat:
        | { source: 'collective_heat'; primaryFactor: number; co2KgPerKwh: number; sourceReference: string }
        | {
            source: 'chp';
            carrier: NtaAnnexPCarrier;
            thermalEfficiency?: number;
            electricalEfficiency?: number;
            efficiencyReference?: string;
            tableP6?: NtaTableP6;
          };
    };

export interface NtaAnnexPSystemGenerator {
  id: string;
  /** Absent for all generators: derived per P.6.5.3/P.6.6.3/P.6.7.3. */
  energyFraction?: number;
  /** Nominal thermal (cold: cooling) power, kW. */
  nominalPowerKw?: number;
  /** Cold: P.53/P.55. */
  coolingPower?:
    | { method: 'compressor_shaft'; shaftPowerKw: number }
    | { method: 'aquifer'; flowM3PerS: number; supplyC: number; returnC: number };
  /** Fixed order of preference, 1 first. */
  priority?: number;
  auxiliary?: {
    standbyW?: number;
    burnerWPerKw?: number;
    sourceWPerKw?: number;
    solutionPumpWPerKw?: number;
    heatRejection?: 'closed_cooling_tower' | 'open_cooling_tower' | 'dry_cooler';
    heatRejectionWPerKw?: number;
    sourceReference?: string;
    /** Hot water: heating carries the standby (P.6.9.4.3). */
    alsoServesHeating?: boolean;
    /** Hot water: no auxiliary energy, e.g. a traditional gas boiler. */
    withoutAuxiliaryEnergy?: boolean;
  };
  kind: NtaAnnexPGenerator;
}

export interface NtaAnnexPPlotFlow {
  annualKwh?: number;
  monthlyKwh?: number[];
}

/** P.8: one connected plot; supplied flows override the forfait tables P.14/P.15. */
export interface NtaAnnexPPlot {
  id: string;
  usableAreaM2?: number;
  heating?: NtaAnnexPPlotFlow;
  heatingForfait?: 'apartment' | 'terraced_or_utility' | 'corner_or_semi_detached' | 'detached';
  sorptionCooling?: NtaAnnexPPlotFlow;
  hotWater?: NtaAnnexPPlotFlow;
  hotWaterForfait?:
    | 'dwelling_low_temperature' | 'dwelling_high_temperature' | 'assembly_with_alcohol' | 'assembly'
    | 'cell' | 'healthcare_clinical' | 'healthcare_non_clinical' | 'office' | 'lodging' | 'education'
    | 'sport' | 'retail';
  hotWaterViaDeliverySet?: boolean;
  cooling?: NtaAnnexPPlotFlow;
  dehumidification?: NtaAnnexPPlotFlow;
  sourceReference: string;
}

export type NtaPipeAmbient = { kind: 'outdoor' } | { kind: 'crawlspace' } | { kind: 'indoor'; temperatureC: number };

/** P.43/P.44 vessel or buffer. */
export interface NtaStorageVessel {
  volumeL?: number;
  surfaceM2?: number;
  diameterAtLeast50Cm?: boolean;
  insulation?: 'none' | 'at_least10_mm' | 'at_least20_mm' | 'at_least30_mm';
  lossFactorWPerM2k?: number;
  standbyLossKwhPerDay?: number;
  standbyTestDifferenceK?: number;
  waterTemperatureC?: number;
  ambientTemperatureC?: number;
}

export type NtaAnnexPFunction = 'heating' | 'hot_water' | 'cooling';

/** NTA 8800 §5.8 / annex P: external heat, hot-water or cold supply values. */
export type NtaAnnexPRoute =
  | {
      method: 'declared';
      primaryFactor: number;
      renewableFactor: number;
      co2KgPerKwh: number;
      declarationReference: string;
      /** Measured values only: f_prac 1 (9.84, 13.152, 10.78); otherwise 0,95. */
      measuredOnly?: boolean;
    }
  | {
      method: 'calculated';
      function: NtaAnnexPFunction;
      /** Q_XD;out;tot; overrides `areaDemand`. */
      deliveredKwh?: number;
      /** P.72–P.83. */
      areaDemand?: { plots: NtaAnnexPPlot[] };
      distribution:
        | { method: 'flows'; inputKwh?: number; lossKwh?: number; sourceReference: string }
        | {
            method: 'small_system_forfait';
            connections: number;
            connectionType: 'ground_bound' | 'within_building';
            designTemperature?: 't90_to60' | 't90_to50' | 't70_to40' | 't50_to40' | 't35_to25';
            otherLossKwh?: number;
          }
        | { method: 'small_cold_forfait'; supplyBelow10C: boolean }
        /** P.13–P.18 with tables P.1 and P.16. */
        | {
            method: 'pipes';
            segments: Array<{
              lengthM: number;
              layers?: Array<{ conductivityWPerMk: number; innerDiameterM: number; outerDiameterM: number }>;
              placement:
                | { kind: 'buried'; coverDepthM: number; groundConductivity?: number; ambient: NtaPipeAmbient }
                | { kind: 'in_air'; ambient: NtaPipeAmbient; surfaceCoefficient?: number };
              correction?:
                | 'two_pipes_in_trench' | 'two_pipes_in_trench_rigid' | 'old_sliding_system'
                | 'surface_or_recessed' | 'single_pipe_in_trench';
              correctionFactor?: number;
              resistanceKmPerW?: number;
            }>;
            /** Absent for hot water: 65 °C. */
            waterTemperature?:
              | { method: 'constant'; temperatureC: number }
              | { method: 'monthly'; temperaturesC: Array<number | null> }
              | { method: 'outdoor_bins'; curve: Array<{ outdoorC: number; waterC: number }>; offAboveOutdoorC?: number };
            buffers?: NtaStorageVessel[];
            /** Cold: required; the loss is 0 at 10 °C or more. */
            supplyBelow10C?: boolean;
            otherLossKwh?: number;
            sourceReference: string;
          };
      generators: NtaAnnexPSystemGenerator[];
      /** P.25 from historical peaks, kW. */
      referencePowerKw?: number;
      /** W_XD;aux;tot; overrides `auxiliary`. */
      auxiliaryElectricityKwh?: number;
      /** P.56–P.70. */
      auxiliary?: {
        distribution:
          | { method: 'pumps'; pumpPowersW: number[]; operatingHours?: number; sourceReference: string }
          | { method: 'pumps_monthly'; pumpPowersW: number[]; sourceReference: string }
          | {
              method: 'forfait';
              network?: 'primary_and_secondary' | 'primary' | 'secondary' | 'small_system';
              farthestDistanceKm?: number;
            };
        solarKwh?: number;
      };
      auxiliaryRenewableShare?: number;
      /** P.34/P.35 η_WD;gen;sto; required for hot water (WD). */
      hotWaterStorage?:
        | { method: 'losses'; storageLossKwh: number; pipeLossKwh: number; sourceReference: string }
        | { method: 'forfait'; insulation: 'at_least20_mm' | 'at_least10_mm' | 'none' }
        | {
            method: 'calculated';
            vessels?: NtaStorageVessel[];
            pipes?: Array<{
              lengthM: number;
              uValueWPerMk?: number;
              outerDiameterMm?: number;
              insulationMm?: number;
              ambientTemperatureC?: number;
            }>;
            exchanger?: { nominalPowerKw: number; insulated?: boolean; specificLossWPerKw?: number };
            circulationTemperatureC?: number;
            correctionFactor?: number;
            sourceReference: string;
          };
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
  /** P.7/P.71: electricity produced in the area with a direct physical connection. */
  areaElectricity?: Array<
    | ({ kind: 'pv' } & NonNullable<BuildingPerformanceInput['pvSystems']>[number])
    | { kind: 'declared'; id: string; annualKwh: number; sourceReference: string }
  >;
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
  generators: Array<{
    id: string;
    primaryFactor: number;
    co2KgPerKwh: number;
    renewableFactor: number;
    heatKwh: number;
    energyFraction?: number;
    efficiency?: number;
    auxiliaryKwh?: number;
  }>;
  /** Intermediate values of the calculated route. */
  calculation?: {
    deliveredKwh: number;
    inputKwh: number;
    distributionLossKwh: number;
    monthlyInputKwh: number[];
    auxiliaryElectricityKwh: number;
    auxiliary?: { distributionKwh: number; solarKwh: number; generatorsKwh: number; totalKwh: number };
    beta?: number;
    referencePowerKw?: number;
    preferred: string[];
    fractionsDerived: boolean;
  };
}

export interface NtaExternalSupplyResult {
  declared: NtaCarrierFactors;
  forfait: NtaCarrierFactors;
  qualityDeclarationUsed: boolean;
  heating: NtaAnnexPSystemResult | null;
  hotWater: NtaAnnexPSystemResult | null;
  cooling: NtaAnnexPSystemResult | null;
  heatPumpSource: NtaAnnexPSystemResult | null;
  /** P.71. */
  areaElectricity?: { totalKwh: number; generators: Array<{ id: string; annualKwh: number }> } | null;
  forfaitPrimaryFossilKwh: number | null;
  forfaitRenewablePrimaryKwh: number | null;
  forfaitCo2Kg: number | null;
}

export interface BuildingPerformanceInput {
  calculationScope: 'residential' | 'utility';
  totalUsableFloorAreaM2: number;
  areaSourceReference: string;
  spaceHeating: SpaceHeatingChainInput;
  /** §9.2: further heating systems, each with its own zones (zone ids unique across systems). */
  additionalHeatingSystems?: SpaceHeatingChainInput[];
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
  /** `wind` must be all zeros: 16.17 sets E_el;wind = 0. */
  onSiteProduction: Array<{ id: string; kind: 'pv' | 'pvt' | 'wind'; monthlyKwh: number[]; sourceReference: string }>;
  pvSystems?: Array<{
    id: string;
    peakPower: NtaPvPeakPower;
    azimuthDeg: number;
    tiltDeg: number;
    mounting?: 'not_ventilated' | 'moderately_ventilated' | 'strongly_ventilated' | 'unknown';
    /** F_sh;obst;mi: one value or twelve monthly values (§17.3); exclusive with `obstruction`. */
    obstructionFactors?: number[];
    /** §17.3 collector situation (tables 17.6/17.12/17.15). */
    obstruction?: NtaCollectorObstruction;
    collective?: { buildingUsableFloorAreaM2: number; sourceReference: string } | null;
    /** 16.10, table 16.4: f_PVT;PV of a PVT panel. */
    pvt?: { kind: 'unglazed' } | { kind: 'glazed'; collectorAreaM2: number; storageVolumeL: number } | null;
    sourceReference: string;
  }>;
  /** Chapter 10: one cooling system serving every calculation zone. */
  cooling?: NtaCoolingSystem | null;
  /** §10.2: several cooling systems with the calculation zones they serve; exclusive with `cooling`. */
  coolingSystems?: Array<{ zoneIds: string[]; system: NtaCoolingSystem }>;
  hotWater?: NtaHotWaterSystem | null;
  /** §13.2.4: further hot-water systems (dwellings: with `connectedTaps`, 13.19a). */
  additionalHotWaterSystems?: NtaHotWaterSystem[];
  /** §13.7 solar systems for space heating only (SHS), without a hot-water system. */
  spaceHeatingSolar?: NtaSolarWaterHeater[];
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
  /** Non-blocking findings, e.g. insufficient annex AA capacity (TOjuli then calculated). */
  warnings?: Array<{ code: string; path: string }>;
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

/** ISSO 82.1/75.1 table 9.16 air-heating type. */
export type OpnameAirHeating =
  | { kind: 'direct'; radialFan?: boolean | null; count?: number | null }
  | {
      kind: 'indirect';
      roomHeightAbove8M?: boolean | null;
      warmAirReturn?: boolean | null;
      ecMotor?: boolean | null;
      count?: number | null;
    }
  | { kind: 'via_air_handling_unit' };

/** ISSO §11.4.1/§11.5.6: passive cooling proven by a supplier project document. */
export interface OpnamePassiveCooling {
  evidenceReference: string;
  /** Commissioning report, including the extra capacity; absent: regulatory flow. */
  installedCapacityDm3PerS?: number | null;
}

/**
 * Annex A state. A blank form value is `null`: the kernel reports it as
 * `dynamic_value_missing` at its own path. τ_sol/τ_vis (A.3/A.4) are kept
 * for the record only; chapter 14 has no input for them (14.38, 14.41).
 */
export interface NtaDynamicState {
  id: string;
  gPerpendicular: number | null;
  uValueWPerM2k: number | null;
  tauSolar?: number | null;
  tauVisual?: number | null;
}

/** Annex A step 2: twelve declared factors per property (p. 770). */
export interface NtaDynamicCorrection {
  uFactors: Array<number | null>;
  gFactors: Array<number | null>;
  sourceReference: string;
}

/** Annex A dynamic transparent element. */
export type NtaDynamicTransparent =
  | {
      method: 'weighted_states';
      states: NtaDynamicState[];
      /** 12 rows, one weight per state, each row summing to 1. */
      solarWeights: Array<Array<number | null>>;
      temperatureWeights: Array<Array<number | null>>;
      sourceReference: string;
      correction?: NtaDynamicCorrection | null;
    }
  | {
      method: 'single_state';
      state: NtaDynamicState;
      sourceReference: string;
      correction?: NtaDynamicCorrection | null;
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
/** Method 2 (10.66) of an absorption chiller: ζ_n at the NEN-EN 14511 rating conditions; PLV 0,95. */
export interface NtaAbsorptionRating {
  nominalHeatRatio: number;
  sourceReference: string;
}

/**
 * NEN-EN 14825 part-load point (10.63). `evaporatorOutletC` is the temperature leaving the
 * evaporator (for air-to-air units the leaving supply air, not the indoor test temperature);
 * `condenserInletC` the temperature entering the condenser (outdoor air for air-cooled units).
 * The kernel requires condenserInletC > evaporatorOutletC.
 */
type NtaEn14825Point = { partLoadPercent: number; eer: number; evaporatorOutletC: number; condenserInletC: number };
/** §10.5.4 (method 1, NEN-EN 14825) or §10.5.5 (method 2, NEN-EN 14511) instead of table 10.29. */
export type NtaCompressionPerformance =
  | {
      method: 'en14825';
      nominalEer: number;
      nominalCapacityKw: number;
      /** Below `nominalCapacityKw`: method 1 applies to modulating units only (§10.5.4). */
      minimumCapacityKw: number;
      /** Conditions A, B, C and D, in that order. */
      testPoints: NtaEn14825Point[];
      /** Part load of C at the condenser inlet of A (10.63); omitted: 10.64, which needs equal evaporator outlets at A and C. */
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
      /** §10.4.2.3: actual L_max, m; absent: forfait 10.27. */
      maxPipeLengthM?: number | null;
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
      | { kind: 'gas_absorption'; heatRejection?: NtaHeatRejection | null; declared?: NtaDeclaredEfficiency | null; rating?: NtaAbsorptionRating }
      | { kind: 'absorption_external_heat'; heatRejection?: NtaHeatRejection | null; rating?: NtaAbsorptionRating }
      /** Table 10.30 with 9.65: CHP fuel booked as gas, its electricity (ε_chp;el) credited in chapter 16. */
      | { kind: 'absorption_chp'; chp: NtaChpClass; heatRejection?: NtaHeatRejection | null; rating?: NtaAbsorptionRating }
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
  /** q_ve;hp;W from a quality declaration (13.148a), m³/h. */
  declaredFlowM3PerH?: number | null;
}

/** 13.146: F_W;gen;gi from a quality declaration, interpolated over Q_W;dis;nren;an. */
export interface NtaDeclaredGeneratorShare {
  points: Array<{ annualKwh: number; share: number }>;
  sourceReference: string;
}

/** §13.8.4.2 tapping profile test (NEN-EN 13203-2 / NEN-EN 16147). */
export interface NtaProfileTest {
  profile: 's' | 'm' | 'l' | 'xl' | 'xxl' | '3xl' | '4xl';
  deliveredKwhPerDay: number;
  /** Gas: Q_gas;p on the net calorific value; heat pump: Q_elec. */
  inputKwhPerDay: number;
  auxiliaryKwhPerDay?: number | null;
  maxTestTemperatureC?: number | null;
}

export interface NtaTwoProfileTest {
  standard: 'en13203_gas' | 'en16147_heat_pump';
  storageAppliance: boolean;
  low: NtaProfileTest;
  high: NtaProfileTest;
  combi?: boolean;
  integratedVessel?: boolean;
  exhaustAirSource?: boolean;
  outdoorAirFraction?: number | null;
  smartControlFactor?: number | null;
  designSetTemperatureC?: number | null;
  legionellaCycleTested?: boolean;
  /** 13.153b/d–i: combi heat pump on a mix of outdoor and return air. */
  mixedAir?:
    | { method: 'declared'; monthlyFactors: number[]; sourceReference: string }
    | { method: 'en14511'; copCondition2: number; condenserOutC: number; evaporatorInC: number;
        evaporatorOutC: number; minimumAirFlowM3PerH: number; sourceReference: string }
    | null;
  /** 13.156a/b: PFHRD of a gas combi (prEN 13203-7, net values, kWh/day). */
  pfhrd?: { indirectGasKwhPerDay: number; heatingGasKwhPerDay: number; sourceReference: string } | null;
  sourceReference: string;
}

/** Collectors use the §17.3 collector tables (17.6/17.12/17.15). */
type NtaSolarObstruction = NtaCollectorObstruction;

/** §13.7 solar water heater; see crates/nta8800-core/src/solar_thermal.rs. */
export interface NtaSolarWaterHeater {
  id: string;
  solarUse: 'water_heating' | 'combi' | 'space_heating';
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
            | { method: 'measured'; transmissionWPerK: number }
            | { method: 'measured_standby'; standbyKwhPerDay: number; referenceStorageC: number; referenceAmbientC: number };
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
        outdoorAirFraction?: number | null;
        /** Annex V: on the same regenerated ground source as the space-heating heat pump (V.1, table V.1). */
        sameGroundSource?: boolean }
    | { kind: 'heat_pump_en16147'; profile: 's' | 'm' | 'l' | 'xl'; deliveredKwhPerDay: number; inputKwhPerDay: number;
        exhaustAirSource: boolean; storageWithoutLegionellaCycle: boolean; outdoorAirFraction?: number | null;
        /** 13.153b SCF (smart = 1 from 0,07) and 13.153c temperatures; omitted means no correction. */
        smartControlFactor?: number | null; maxTestTemperatureC?: number | null; designSetTemperatureC?: number | null;
        sourceReference: string }
    | { kind: 'electric_instantaneous' }
    | { kind: 'electric_boiler' }
    | { kind: 'gas_storage_heater'; volumeL: number; measuredStandbyKwhPerDay?: number | null; before1985: boolean; inHeatedZone: boolean }
    | { kind: 'large_direct_storage'; gasFired: boolean }
    | { kind: 'indirect_boiler'; boiler: 'conventional_or_unknown' | 'vr' | 'hr100_or104' | 'hr107'; oil: boolean;
        insideBoundary: boolean; alsoSpaceHeating: boolean; declared?: NtaDhwDeclared | null;
        /** §13.8.4.7.4 gas pilot flame, counted when the boiler does not also heat the building. */
        pilotFlame?: boolean }
    /** §13.8.4.6 table 13.22 solid-biomass combi appliance with an annex R vessel. */
    | { kind: 'biomass_combi'; insulation: 'at_least20_mm' | 'at_least10_mm' | 'none'; insideBoundary: boolean }
    /** §13.8.4.10 electric heat pumps in series as one notional device (table 9.29, 65–70 °C). */
    | { kind: 'heat_pump_series'; lastSource: ForfaitHeatPumpDraftInput['source'] }
    | { kind: 'indirect_heat_pump'; alsoSpaceHeating: boolean }
    | { kind: 'external_heat' }
    | ({ kind: 'booster_heat_pump' } & NtaBoosterHeatPump)
    | ({ kind: 'measured_two_profiles' } & NtaTwoProfileTest)
    | { kind: 'heating_system' }
    /** §13.8.4.7.4/§13.8.4.8 building CHP: method 2 (`chp`) or method 1 (`method1`), exclusive. */
    | { kind: 'chp'; chp?: NtaChpClass | null; method1?: NtaMicroChp | null; alsoSpaceHeating?: boolean;
        /** Method 1 without NEN-EN 50465 auxiliary powers: 9.6.8 (9.91/9.92), per 9.6.6.2.2.3. */
        auxiliary?: NtaOtherGeneratorAuxiliary | null; equipmentReference: string };

/** Chapter 13 hot-water system (several generators, solar systems). */
export interface NtaHotWaterSystem {
  /** 13.19a: bathrooms and kitchens on this system when a dwelling has several systems. */
  connectedTaps?: { bathrooms: number; kitchens: number } | null;
  /** 7.82: b_U of the unheated space with pipes or vessels; ϑ_ztu = ϑ_set − b_U·(ϑ_set − ϑ_e). */
  unheatedReductionFactor?: number | null;
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
    /** Utility (p. 564): assignment of showers to units unknown; above 80 % connected the minimum applies. */
    assignmentUnknown?: boolean;
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
      | { method: 'measured'; transmissionWPerK: number }
      /** 13.60 from the standby test; H_sto;ls rounded up per annex X. */
      | { method: 'measured_standby'; standbyKwhPerDay: number; referenceStorageC: number; referenceAmbientC: number };
    connectionFactor: 1 | 2 | 3 | 4 | 5;
    inHeatedZone: boolean;
    unheatedAmbientC?: number | null;
    /** Note 1 of §13.6.2: the 13.8.4.2/13.8.4.3 appliance was tested without this vessel. */
    notInApplianceTest?: boolean;
    sourceReference: string;
  }>;
  deliverySets?: { count: number; sourceReference: string } | null;
  boilingWaterTap?: boolean;
  generator: NtaHotWaterGenerator;
  /** P_nom of the main generator (13.141), kW. */
  nominalPowerKw?: number | null;
  /** 13.144a when the main generator is an exhaust-air heat pump. */
  exhaustAir?: NtaExhaustAirUse | null;
  /** 13.146 for the main generator. */
  declaredShare?: NtaDeclaredGeneratorShare | null;
  /** Further generators (13.8.2). */
  additionalGenerators?: Array<{
    generator: NtaHotWaterGenerator;
    nominalPowerKw?: number | null;
    exhaustAir?: NtaExhaustAirUse | null;
    declaredShare?: NtaDeclaredGeneratorShare | null;
    equipmentReference: string;
  }>;
  /** 13.141a–d: main generator first, the single additional one second. */
  series?: { kind: 'hotfill_electric_boiler' } | { kind: 'collective_first_also_heating'; maximumSupplyC: number[] } | null;
  /** §13.7 solar water heaters and solar combi systems. */
  solar?: NtaSolarWaterHeater[];
  collective?: { buildingUsableFloorAreaM2: number; sourceReference: string } | null;
  equipmentReference: string;
}

/** Chapter 10 result; see crates/nta8800-core/src/space_cooling.rs. */
export interface NtaCoolingResult {
  coolingLimitC: number;
  internalTemperatureShiftK: number;
  generatorShares: Array<{ id: string; priority: number; beta: number; shareJulyToSeptember: number;
    shareOtherMonths: number; method: number; monthlyEer: number[] }>;
  months: Array<{ month: number; needKwh: number; emissionLossKwh: number; distributionLossKwh: number;
    generatorColdKwh: number; electricityKwh: number; naturalGasKwh: number; districtHeatKwh: number;
    districtColdKwh: number; auxiliaryElectricityKwh: number; ambientColdKwh: number }>;
  interpretations: string[];
  systems?: Array<{ zoneIndexes: number[]; assessment: NtaCoolingResult }>;
  warnings?: Array<{ code: string; path: string }>;
}

/** Energy functions of 5.20 (E_dhum is part of cooling). */
export type NtaEnergyFunction = 'heating' | 'humidification' | 'ventilation' | 'lighting' | 'cooling'
  | 'hotWater' | 'auxiliary' | 'heatPumpSource';

/**
 * §5.5.3 energy per energy function and carrier; see building_performance.rs.
 * Σ months per carrier equals `carriers`; Σ primaryFossilKwh minus the
 * adjustments equals EPTot; Σ renewable plus renewableElectricityKwh equals EPrenTot.
 */
export interface NtaServiceEnergyBreakdown {
  months: Array<{ service: NtaEnergyFunction; carrier: string; month: number; usedKwh: number;
    deliveredKwh: number; primaryFossilKwh: number }>;
  renewable: Array<{ service: NtaEnergyFunction; month: number; renewablePrimaryKwh: number }>;
  adjustments: Array<{ month: number; exportedElectricityCreditKwh: number; storageCorrectionKwh: number;
    renewableElectricityKwh: number }>;
  annual: Array<{ service: NtaEnergyFunction; carrier: string; usedKwh: number; deliveredKwh: number;
    primaryFossilKwh: number }>;
}

/** One norm part with the kernel's interpretation choices. */
export interface NtaInterpretationGroup {
  part: string;
  module: string;
  items: string[];
}

/** The kernel's interpretation lists (crates/nta8800-core/src/interpretations.rs). */
export async function fetchKernelInterpretations(): Promise<NtaInterpretationGroup[]> {
  if (isTauri()) {
    return invoke<NtaInterpretationGroup[]>('kernel_interpretations');
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/interpretations');
    return response.json() as Promise<NtaInterpretationGroup[]>;
  }
  return [];
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
    carrier: 'el' | 'gas' | 'oil' | 'dh' | 'dw' | 'dc' | 'bm';
    month: number;
    usedKwh: number;
    deliveredKwh: number;
  }>;
  /** §5.5.3/5.20: the carriers split per energy function. */
  energyByService?: NtaServiceEnergyBreakdown;
  electricityBalance: Array<{ month: number; usedKwh: number; producedKwh: number; selfUsedKwh: number; exportedKwh: number }>;
  annualPrimaryFossilKwh: number | null;
  annualRenewablePrimaryKwh: number | null;
  annualHeatPumpAmbientHeatKwh: number | null;
  annualHeatingAndCoolingNeedKwh: number | null;
  annualStorageCorrectionKwh: number | null;
  annualCo2Kg: number | null;
  /** Annex AB (informative) E_P,ZEB;Tot;an, kWh. */
  annualZebPrimaryTotalKwh?: number | null;
  /** AB.1 EweP,ZEB;Tot, kWh/m² (rounded up to 0,01). */
  zebPrimaryTotalIndicatorKwhPerM2?: number | null;
  /** AB.3 operational CO2eq, kg per year. */
  annualZebCo2Kg?: number | null;
  /** 5.57/5.58 final energy per carrier, kWh. */
  finalEnergyByCarrier: Array<{ carrier: string; annualKwh: number }>;
  /** 5.57 E_Final, kWh (own production not netted). */
  annualFinalEnergyKwh: number | null;
  /** 5.60 E_Final;EED = E_Final + solar thermal yields, kWh. */
  annualFinalEnergyEedKwh: number | null;
  co2KgPerM2: number | null;
  /** Chapter 5 label and record indicators (5.3a–i, 5.17–5.19, 5.39a–h, 5.5.7, table 5.7). */
  chapter5?: NtaChapterFiveIndicators | null;
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
  /** 13.184: part of the space-heating carriers used for hot water (reported only). */
  hotWaterFromHeating?: Array<{ month: number; share: number; naturalGasKwh: number; oilKwh: number;
    biomassKwh: number; districtHeatKwh: number; electricityKwh: number }>;
  /** Standalone space-heating solar systems. */
  standaloneSolar?: { spaceHeatingKwh: number[]; auxiliaryKwh: number[]; recoverableKwh: number[] };
  lighting?: Array<{ zoneId: string; annualKwh: number; monthlyKwh: number[]; internalGainW: number }>;
  /** Chapter 10 when cooled; `systems` per cooling system with several (§10.2). */
  cooling?: NtaCoolingResult;
  /** Chapter 16 E_pr;el per PV system, kWh. */
  pvSystems?: Array<{ id: string; monthlyKwh: number[]; annualKwh: number }>;
  /** Factors of the external supply and the EMGforf totals (§5.8). */
  externalSupply?: NtaExternalSupplyResult | null;
  issues: Array<{ code: string; path: string }>;
  /** Non-blocking findings (e.g. cooling_emission_loss_singular, hot_water_circulation_defaults_low_efficiency). */
  warnings?: Array<{ code: string; path: string }>;
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

export interface NtaProjectHeatingSystem {
  zoneIds: string[];
  generator: SpaceHeatingChainInput['generator'];
  distributionSystem?: NtaDistributionSystem | null;
  collectiveConnection?: SpaceHeatingChainInput['collectiveConnection'];
  identicalSystems?: number | null;
  humidifiers?: NtaZoneHumidifier[];
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
  /** Annex A dynamic transparent elements per project window id. */
  dynamicWindows?: Array<{ windowId: string; dynamic: NtaDynamicTransparent }>;
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
  /** 7.3.3 vertical pipes (single-zone projects; per zone in `zoneData`); `[]` is none, absent is unknown (a gap). */
  verticalPipes?: Array<{ id: string; storeys?: number; buildingHeightM?: number | null; areaShare?: number | null; insulated: boolean; sharedZones?: number; sourceReference: string }> | null;
  zoneData?: Array<{
    zoneId: string;
    verticalPipes?: Array<{ id: string; storeys?: number; buildingHeightM?: number | null; areaShare?: number | null; insulated: boolean; sharedZones?: number; sourceReference: string }> | null;
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
  identicalSystems?: number | null;
  /** §9.2: further heating systems; the main system serves the zones not listed here. */
  additionalHeatingSystems?: NtaProjectHeatingSystem[];
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
  /** §13.2.4: further hot-water systems (dwellings: with `connectedTaps`, 13.19a). */
  additionalHotWaterSystems?: NtaHotWaterSystem[];
  /** §13.7 solar systems for space heating only (SHS), without a hot-water system. */
  spaceHeatingSolar?: NtaSolarWaterHeater[];
  /** Utility lighting per calculation zone (NTA 8800 chapter 14). */
  lighting?: NtaZoneLighting[];
  cooling?: NtaCoolingSystem | null;
  /** §10.2: several cooling systems with the calculation zones they serve; exclusive with `cooling`. */
  coolingSystems?: Array<{ zoneIds: string[]; system: NtaCoolingSystem }>;
  labelFunction?: NtaLabelFunction | null;
  /** §5.3.1: use functions of an existing utility building (label bounds and table 5.7 weighted by area). */
  labelFunctions?: Array<{ function: NtaLabelFunction; areaM2: number }>;
  /** For the Standaard voor woningisolatie; falls back to registration.constructionYear. */
  constructionYear?: number;
  /** §5.5.7: fossil-fuelled building-bound appliances left out of the calculation. */
  fossilAppliancesOutsideCalculation?: boolean;
  /** §5.5.8: systems and BACS evidence; derives f_BACS and replaces bacsFactor. */
  bacs?: {
    buildingUse: 'residential' | 'utility';
    systemInventoryComplete: boolean;
    systems: Array<{
      id: string;
      service: 'heating' | 'cooling';
      sourceReference: string;
      generators: Array<{ id: string; nominalThermalCapacityKw: number | null; sourceReference: string }>;
      bacs?: NtaBacsEvidence;
    }>;
    bacs?: NtaBacsEvidence;
  } | null;
  bblFunction?: NtaBblFunction | null;
  /** Annex AB footnote g: delivery temperature of external heat (unknown = ≥ 60 °C). */
  zebHeatDeliveryTemperature?: 'at_least60' | 'from40_to60' | 'from20_to40';
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

/** NTA 8800 chapter 5 indicators; see crates/nta8800-core/src/building_performance.rs. */
export interface NtaChapterFiveIndicators {
  /** 5.3a E_H;nd, kWh/m², rounded up to 0,01. */
  heatingNeedKwhPerM2: number;
  /** 5.3d E_C;nd. */
  coolingNeedKwhPerM2: number;
  /** 5.3g E_H+C;nd. */
  heatingAndCoolingNeedKwhPerM2: number;
  /** §5.3.2 Standaard voor woningisolatie (dwellings), whole kWh/m². */
  standardInsulationKwhPerM2: number | null;
  meetsStandardInsulation: boolean | null;
  /** §5.3.1.3 EwePrenTot, rounded down to 0,01. */
  renewableIndicatorKwhPerM2: number;
    /** §5.3.1.3 EwePrenTot;EMGforf with a quality declaration. */
    renewableIndicatorForfaitKwhPerM2?: number | null;
  /** 5.3h / 5.3i. */
  finalEnergyKwhPerM2: number;
  finalEnergyEedKwhPerM2: number;
  /** 5.17a/b. */
  deliveredElectricityKwh: number;
  deliveredElectricityKwhPerM2: number;
  /** 5.18a/b, GJ. */
  deliveredExternalGj: number;
  deliveredExternalGjPerM2: number;
  /** 5.19a/b, m³ natural gas equivalent. */
  deliveredOtherM3Aeq: number;
  deliveredOtherM3AeqPerM2: number;
  /** 5.39a–h, kWh primary renewable. */
  renewableByCarrier: {
    electricity: number;
    heatPumpHeat: number;
    solarHeat: number;
    cold: number;
    biomass: number;
    externalHeat: number;
    externalCold: number;
  };
  /** §5.5.7; null when fossil appliances outside the calculation are not stated. */
  locallyCarbonFree: boolean | null;
  /** Table 5.7 (utility), rounded to 0,01. */
  renovationStandardKwhPerM2: number | null;
  meetsRenovationStandard: boolean | null;
}

export interface ProjectPerformanceAssessment {
  status: 'calculated_unverified' | 'incomplete' | 'invalid';
  targetNormVersion: string;
  kernelVersion: string;
  inputFingerprint: string;
  attestStatus: 'unattested';
  gaps: Array<{ code: string; path: string; detail?: string }>;
  /** Plausibility findings that leave the calculation running. */
  warnings?: Array<{ code: string; path: string; detail?: string }>;
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
  /** §5.5.8 f_BACS assessment when ntaCalculation.bacs is given. */
  bacs?: {
    status: string;
    factor: number | null;
    applicability: string;
    triggeringSystemIds: string[];
    issues: Array<{ code: string; path: string }>;
  } | null;
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
  /** Kept for saved projects; `messageType` supersedes it. */
  relabel?: boolean;
  /** BRL 9500-W §4.2.5 opmerking 4/5: regular, relabel or replacement of an incorrect label. */
  messageType?: NtaMessageType;
  /** Replacement: EP-Online number of the replaced label. */
  replacedEpOnlineNumber?: string;
  /** Program that made the calculation (Regeling art. 5 lid 1 onder b); filled in by the app. */
  software?: NtaSoftwareIdentity;
  /** WLC-GWP result, required for new buildings > 1000 m² checked against the Bbl from 2028. */
  wlcGwp?: { valueKgCo2EqPerM2Year?: number; reportReference?: string };
  /** Delivery: date of the toets Bbl it follows (YYYY-MM-DD); the WLC-GWP duty follows that check. */
  bblCheckDate?: string;
  /** A_g of the whole building when the calculation covers one dwelling (WLC-GWP threshold per building). */
  buildingUsableFloorAreaM2?: number;
  /** Class of the label registered before, for the class-jump plausibility check. */
  previousLabelClass?: string;
  /** Relabel: date of the improvement (quote with order or invoice), YYYY-MM-DD; within 24 months of the survey (BRL 9500 §4.2.3). */
  improvementDate?: string;
  originalKernelVersion?: string;
  epOnlineNumber?: string;
  /** Opleverdatum, YYYY-MM-DD (§3.1: completed after 1-1-2021 needs a detailed survey). */
  completionDate?: string;
  /** BRL 9500 §3.1 situations that make the detailed survey mandatory. */
  detailSurveyTriggers?: NtaDetailSurveyTriggers;
  /** Evidence register of the project dossier (BRL 9500 Bijlage 3). */
  evidence?: NtaEvidenceItem[];
}

export type NtaMessageType = 'regular' | 'relabel' | 'replacement';

export interface NtaSoftwareIdentity {
  name: string;
  version: string;
  /** BRL 9501 attest number; empty until the program is attested. */
  attestNumber?: string;
  /** Calculation core of the calculation; a relabel must keep it (BRL 9500-W §4.2.4). */
  kernelVersion?: string;
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
  /** BRL 9500-W (dwellings) or 9500-U (utility), from `buildingFunction`. */
  scheme: 'w' | 'u';
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

// ------------------------------------------------------------------
// Maatwerkadvies (BRL 9500-MWA-W/U, ISSO 82.2/75.2)
// ------------------------------------------------------------------

export type MwaUserProfile = 'nta' | 'energy_conscious' | 'average' | 'not_energy_conscious';

/** ISSO 82.2/75.2 table 2.2: user-dependent parameters (profile + free input). */
export interface MwaUsageProfile {
  profile: MwaUserProfile;
  heatingSetpointC?: number;
  coolingSetpointC?: number;
  reducedSetpointC?: number;
  dayReductionH?: number;
  weekendReductionH?: number;
  spatialFraction?: number;
  occupants?: number;
  internalGainPerPersonW?: number;
  occupancyApplianceWPerM2?: number;
  hotWaterNeedPerPersonKwh?: number;
  annualHotWaterNeedKwh?: number;
  /** ISSO 82.2 table 2.7 / 75.2 table 2.8; standard values for the standard profiles (A 0,25, C 0,5, D 0,75; purge and infiltration 0,5), none under `nta` unless entered. */
  ventilationPractice?: { system?: number; purge?: number; leakage?: number };
  /** Utility (75.2 table 2.6): N_p of the building, split over the zones by area. */
  persons?: number;
  /** Utility: q_oc;p;usi in W per person (standard 80). */
  heatPerPersonW?: number;
  /** Utility: occupancy time fraction f_t (standard NTA table 7.2). */
  occupancyTimeFraction?: number;
  /** Utility: q_A in W/m² (standard NTA table 7.3). */
  applianceWPerM2?: number;
  /** Utility (75.2 table 2.7): factor on the table 14.1 burning hours (0,8 / 1,0 / 1,2 by profile). */
  lightingHoursFactor?: number;
  sourceReference: string;
}

export type MwaPatchOperation =
  | { op: 'replace'; path: string; value: unknown }
  | { op: 'add'; path: string; value: unknown }
  | { op: 'remove'; path: string };

export type MwaMeasureCategory =
  | 'insulation' | 'glazing' | 'airtightness' | 'ventilation' | 'heat_recovery' | 'heating'
  | 'heat_pump' | 'hot_water' | 'cooling' | 'pv' | 'solar_thermal' | 'lighting' | 'control' | 'other';

export interface MwaMeasure {
  id: string;
  name: string;
  category: MwaMeasureCategory;
  /** `project`: JSON patch on the project; `building`: on the derived building input. */
  target: 'project' | 'building';
  patch: MwaPatchOperation[];
  investmentEur: number;
  costSource: string;
  lifetimeYears: number;
  maintenanceEurPerYear?: number;
  phaseYear?: number;
  specialistNote?: string;
}

export interface MwaPackage {
  id: string;
  name: string;
  measureIds: string[];
  partialExecutionWarning?: string;
}

export interface MwaTariffs {
  gasEurPerM3: number;
  electricityEurPerKwh: number;
  electricityExportEurPerKwh?: number;
  districtHeatEurPerKwh?: number;
  districtColdEurPerKwh?: number;
  oilEurPerKwh?: number;
  biomassEurPerKwh?: number;
  gasFixedEurPerYear?: number;
  heatFixedEurPerYear?: number;
  sourceReference: string;
}

export interface MwaEconomics {
  discountRate?: number;
  energyPriceChange?: number;
  horizonYears?: number;
  /** Calendar year of t = 0; with it a measure's phaseYear delays its investment in the NCW. */
  baseYear?: number;
  /** Annual change of maintenance costs (fraction). */
  maintenancePriceChange?: number;
  sourceReference?: string;
}

export interface MwaMeasuredUse {
  annualGasM3?: number;
  annualElectricityKwh?: number;
  annualHeatKwh?: number;
  monthlyGasM3?: Array<number | null>;
  /** Net monthly electricity on the main meter (delivered minus exported), kWh. */
  monthlyElectricityKwh?: Array<number | null>;
  monthlyHeatKwh?: Array<number | null>;
  /** Local monthly mean outdoor temperature of the metered period, °C. */
  monthlyOutdoorTemperatureC?: Array<number | null>;
  sourceReference?: string;
}

/** ISSO 82.2 §1.10.2 / §4.4: the three stacked steps of a renovation passport. */
export interface MwaRenovationPassportInput {
  demandPackageId: string;
  systemsPackageId: string;
  productionPackageId: string;
  prewarStandard?: boolean;
  prewarMotivation?: string;
  insulationStandardMet?: boolean;
  insulationStandardMaxNeedKwhPerM2?: number;
  overheatingMeasureIds?: string[];
  storageConsidered?: boolean;
}

/** The maatwerkadvies definition stored with a project (the base is the project itself). */
export interface NtaMaatwerkadvies {
  currentUse?: MwaUsageProfile;
  futureUse?: MwaUsageProfile;
  measures: MwaMeasure[];
  packages: MwaPackage[];
  tariffs: MwaTariffs;
  economics?: MwaEconomics;
  measured?: MwaMeasuredUse;
  advisedPackageId?: string;
  adviceMotivation?: string;
  notes?: Array<{ text: string; packageId?: string }>;
  renovationPassport?: MwaRenovationPassportInput;
}

export interface MwaEnergyUse {
  gasKwh: number;
  gasM3: number;
  electricityImportKwh: number;
  electricityExportKwh: number;
  electricityProducedKwh: number;
  districtHeatKwh: number;
  districtColdKwh: number;
  oilKwh: number;
  biomassKwh: number;
  primaryFossilKwh: number;
  co2Kg: number;
  energyCostEur: number;
  monthlyGasM3: number[];
  monthlyElectricityImportKwh: number[];
  monthlyHeatKwh: number[];
  monthlyElectricityExportKwh: number[];
}

/** Bbl art. 4.248 with Omgevingsregeling bijlage VIII (ISSO 82.2 §5.2). */
export interface MwaSystemCheck {
  system: 'space_heating' | 'space_cooling' | 'hot_water' | 'ventilation' | 'lighting';
  value: number | null;
  limit: number | null;
  unit: string;
  meets: boolean | null;
  note: string | null;
}

export interface MwaIssue { code: string; path: string; detail?: string }

export interface MwaVariantResult {
  id: string;
  name: string;
  kind: 'current' | 'measure' | 'package' | 'passport_step';
  measureIds: string[];
  valid: boolean;
  label: {
    labelClass: string | null;
    needIndicatorKwhPerM2: number | null;
    primaryFossilIndicatorKwhPerM2: number | null;
    renewableSharePercent: number | null;
    tojuliMaxK: number | null;
  };
  actualUse: MwaEnergyUse | null;
  savings: {
    gasM3: number;
    electricityKwh: number;
    heatKwh: number;
    primaryFossilKwh: number;
    co2Kg: number;
    energyCostEur: number;
  } | null;
  investmentEur: number;
  maintenanceEurPerYear: number;
  simplePaybackYears: number | null;
  netPresentValueEur: number | null;
  horizonYears: number;
  phasing: Array<{ year: number | null; measureIds: string[] }>;
  systemChecks: MwaSystemCheck[];
  issues: MwaIssue[];
}

export interface MwaRegressionLine {
  slopeM3PerK: number;
  interceptM3: number;
  heatingLimitC: number | null;
  points: number;
}

export interface MaatwerkadviesAssessment {
  status: 'calculated_unverified' | 'partially_calculated' | 'invalid';
  scope: string;
  targetNormVersion: string;
  kernelVersion: string;
  inputFingerprint: string;
  attestStatus: string;
  current: MwaVariantResult | null;
  measures: MwaVariantResult[];
  packages: MwaVariantResult[];
  fitCheck: {
    calculated: MwaEnergyUse;
    gasDeviationPercent: number | null;
    electricityDeviationPercent: number | null;
    heatDeviationPercent: number | null;
    measuredGasLine: MwaRegressionLine | null;
    calculatedGasLine: MwaRegressionLine | null;
    measuredHeatLine: MwaRegressionLine | null;
    calculatedHeatLine: MwaRegressionLine | null;
    measuredGasBaseLoad: number | null;
    calculatedGasBaseLoad: number | null;
    measuredHeatBaseLoad: number | null;
    calculatedHeatBaseLoad: number | null;
    monthlyElectricityDeviationPercent: Array<number | null>;
    measuredElectricityMonthlyMeanKwh: number | null;
    calculatedElectricityMonthlyMeanKwh: number | null;
    measuredElectricityLine: MwaRegressionLine | null;
    calculatedElectricityLine: MwaRegressionLine | null;
    measuredElectricityBaseLoad: number | null;
    calculatedElectricityBaseLoad: number | null;
    /** ISSO 82.2 Bijlage C.1: annual ±5 %, slope ±5 %, heating limit ±1 °C, base line ±5 %. */
    criteria: {
      annualGas: boolean | null;
      annualElectricity: boolean | null;
      annualHeat: boolean | null;
      gasSlope: boolean | null;
      gasHeatingLimit: boolean | null;
      gasBaseLine: boolean | null;
      heatSlope: boolean | null;
      heatHeatingLimit: boolean | null;
      heatBaseLine: boolean | null;
      electricitySlope: boolean | null;
      electricityHeatingLimit: boolean | null;
      electricityBaseLine: boolean | null;
      withinCriteria: boolean | null;
    };
  } | null;
  advice: {
    packageId: string | null;
    chosenBy: 'adviser' | 'automatic_highest_npv';
    motivation: string | null;
    warnings: string[];
    specialistNotes: string[];
    notes: string[];
  } | null;
  renovationPassport: {
    steps: MwaVariantResult[];
    requirements: Array<{ code: string; met: boolean | null; detail: string | null }>;
    eligible: boolean | null;
    requiredStatements: string[];
  } | null;
  interpretations: string[];
  issues: MwaIssue[];
}

/** Builds the kernel input: the project (without its own MWA block) is the base. */
export function buildMaatwerkadviesInput(project: IProject, definition: NtaMaatwerkadvies): Record<string, unknown> {
  const { maatwerkadvies: _omit, ...base } = project as IProject & { maatwerkadvies?: unknown };
  void _omit;
  return { base: { kind: 'project', project: base }, ...definition };
}

export async function assessMaatwerkadviesWithRust(project: IProject, definition: NtaMaatwerkadvies): Promise<MaatwerkadviesAssessment> {
  const input = buildMaatwerkadviesInput(project, definition);
  if (isTauri()) {
    return invoke<MaatwerkadviesAssessment>('assess_maatwerkadvies', { input });
  }
  if (import.meta.env.DEV) {
    const response = await fetch('/api/v1/nta8800/maatwerkadvies', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ input }),
    });
    const body = await response.json() as MaatwerkadviesAssessment & { error?: string; message?: string };
    if (body.error) throw new Error(`${body.error}: ${body.message ?? ''}`);
    return body;
  }
  throw new Error('Maatwerkadvies is available in the desktop app and local development server.');
}

export interface RegistrationAssessment {
  source: string;
  /** Effective message type (`messageType`, else `relabel`). */
  messageType?: NtaMessageType;
  validUntil: string | null;
  registrationDeadline: string | null;
  relabelDeadline: string | null;
  replacementDeadline?: string | null;
  /** WLC-GWP required; null when A_g or the date is unknown. */
  wlcGwpRequired?: boolean | null;
  /** Program of the registration; this program for projects saved without one. */
  software?: NtaSoftwareIdentity;
  /** Whether that program has a BRL 9501 attest number. */
  softwareAttested?: boolean;
  /** The dossier holds everything the registration needs (no `issues`). */
  dossierComplete?: boolean;
  /** Dossier complete and program attested (Regeling art. 2/3, p. 4–5). */
  readyForRegistration: boolean;
  issues: Array<{ code: string; path: string; severity: 'error' | 'missing' }>;
  /** BRL 9500 §7.2.2-style plausibility warnings; never block registration. */
  plausibility?: Array<{ code: string; path: string; severity: 'warning' }>;
}

export interface NtaBacsEvidence {
  present: boolean;
  automaticControlsClass?: 'A' | 'B' | 'C' | 'D';
  energyManagementClass?: 'A' | 'B' | 'C' | 'D';
  sourceReference: string;
}

export interface LabelData {
  source: string;
  /** Regeling art. 4 a. */
  general: {
    useFunction: string | null;
    constructionYear: number | null;
    usableFloorAreaM2: number | null;
    dwellingType: string | null;
  };
  envelope: Array<{
    category: 'facade' | 'roof' | 'floor' | 'glazing';
    areaM2: number;
    meanUWPerM2k: number | null;
    minRcM2kPerW: number | null;
    maxRcM2kPerW: number | null;
  }>;
  installations: {
    heatingGenerator: string | null;
    /** §9.2: generator kinds of every heating system, main first. */
    heatingGenerators: string[];
    hotWaterGenerator: string | null;
    ventilationSystems: string[];
    coolingGenerators: string[];
    pvSystemCount: number;
    lightingZoneCount: number;
    solarWaterHeaterCount: number;
    solarWaterHeaterUses: string[];
  };
  /** Regeling art. 4 indicators from the calculation. */
  indicators?: {
    primaryFossilKwhPerM2: number | null;
    renewableSharePercent: number | null;
    tojuliMaxK: number | null;
    heatingNeedKwhPerM2: number | null;
    standardInsulationKwhPerM2: number | null;
    energyNeedKwhPerM2: number | null;
    renovationStandardKwhPerM2: number | null;
    indicativeLabelClass: string | null;
  } | null;
}

export async function calculateProjectPerformanceWithRust(input: IProject): Promise<ProjectPerformanceAssessment> {
  const project = kernelProject(input);
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
    /** 13.17 emission input Q_W;em;in, kWh. */
    emissionInputKwh?: number;
    /** 13.26 circulation loss, kWh. */
    circulationLossKwh?: number;
    /** 13.58 storage loss, kWh. */
    storageLossKwh?: number;
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
    /** 13.185 output supplied by the space-heating system (§13.8.4.9.3). */
    heatingSystemLoadKwh: number;
    /** 16.13/16.16 electricity of a hot-water CHP, kWh. */
    chpElectricityKwh: number;
    /** 9.66: hot-water heat above the CHP full load, booked without electricity, kWh. */
    chpExcessKwh?: number;
  }>;
  generators: Array<{ index: number; order: number; monthlyOutputKwh: number[]; monthlyShare: number[] }>;
  /** 13.148/13.149 data of an exhaust-air heat pump. */
  exhaustAir?: { timeFraction: number[]; hotWaterOnly: boolean; declaredFlowM3PerH: number | null };
  /** §13.8.4.8 (p. 650): the heating share of a combi micro-CHP after one joint 9.6.6.2 evaluation. */
  combiChpHeating?: Array<{ month: number; inputKwh: number; electricityKwh: number; auxiliaryKwh: number | null }>;
}

/** 9.6.6.2 test point (table 9.33); omitted efficiencies take table 9.37. */
export interface NtaChpTestPoint {
  thermalPowerKw: number;
  electricPowerKw?: number | null;
  thermalEfficiency?: number | null;
  electricEfficiency?: number | null;
  auxiliaryPowerKw?: number | null;
}

/** §9.6.6.2 micro-CHP (method 1, NEN-EN 15316-4-4); see crates/nta8800-core/src/micro_chp.rs. */
export interface NtaMicroChp {
  kind: 'stirling_engine' | 'pem_fuel_cell' | 'solid_oxide_fuel_cell' | 'gas_engine' | 'diesel_engine' | 'micro_turbine' | 'organic_rankine_cycle';
  fuel: 'natural_gas' | 'oil';
  location: 'heated_space' | 'unheated_space' | 'installation_room' | 'outdoors';
  hydraulics?: 'direct' | 'decoupled' | 'condensation_pump' | 'heat_exchanger' | null;
  /** CHP_100 %+Sup_100 %. */
  fullLoad: NtaChpTestPoint;
  /** CHP_100 %+Sup_0 %. */
  chpOnly: NtaChpTestPoint;
  standbyLossKw?: number | null;
  pilotKw?: number | null;
  standbyElectricKw?: number | null;
  standbyAuxiliaryKw?: number | null;
  /** Only the net power production was measured (9.72). */
  netProductionMeasured?: boolean;
  /** 9.6.6.2.2.8 storage outside the test configuration (space heating only). */
  storage?: { lossWPerK: number; setTemperatureC: number; chargingAuxiliaryW?: number | null; sourceReference: string } | null;
  testReportReference: string;
}
