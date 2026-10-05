// ============================================================
// Open Energy Studio – Monthly BENG Calculator (Orchestrator)
// ============================================================

import type {
  IProject,
  IBENGResultMonthly,
  IEnergyBreakdown,
  IMonthlyBreakdown,
  MonthlyValues,
} from './types';
import { assertLegacyHeatPumpInputs, hasUnmodelledHeatPumpDetails, hasUnmodelledUnheatedTransmission, validProjectFloorArea } from './ProjectArea';
import {
  BENG_LIMITS,
  getHotWaterDemand,
  LIGHTING_ENERGY,
  USAGE_HOURS_PER_YEAR,
  VENTILATION_DESIGN_FLOW_RATE_RESIDENTIAL,
  VENTILATION_FAN_TIME_FACTOR,
  QV10_CORRECTION_FACTOR,
} from './Constants';
import { MONTHLY_AVERAGE_TEMPERATURE, HOURS_PER_MONTH } from './ClimateData';
import {
  calculateMonthlyTransmissionLoss,
  calculateTransmissionLossCoefficient,
} from './MonthlyTransmissionLoss';
import { calculateMonthlyVentilationLoss } from './MonthlyVentilationLoss';
import { calculateMonthlySolarGain } from './MonthlySolarGain';
import { calculateMonthlyInternalGain } from './MonthlyInternalGain';
import { calculateMonthlyHeatingDemand } from './MonthlyHeatingDemand';
import { calculateMonthlyCoolingDemand } from './MonthlyCoolingDemand';
import { calculateMonthlyPVProduction } from './MonthlyPVProduction';
import { calculateMonthlySolarThermalProduction } from './MonthlySolarThermalProduction';
import { calculateTOJuli } from './TOJuli';
import { calculatePrimaryEnergy } from './PrimaryEnergy';
import { calculateRenewableShare } from './RenewableShare';

/**
 * Sum 12 monthly values to get an annual total.
 */
function sumMonthly(values: MonthlyValues): number {
  let total = 0;
  for (let m = 0; m < 12; m++) {
    total += values[m];
  }
  return total;
}

/**
 * Calculate ventilation fan energy based on SFP, airflow and operating hours.
 * NTA 8800: uses time-averaged flow rate (demand-controlled factor ≈ 0.67)
 */
function calculateVentilationFanEnergy(project: IProject, totalFloorArea: number): number {
  if (project.ventilationSystems.length === 0) return 0;

  const designAirflow = VENTILATION_DESIGN_FLOW_RATE_RESIDENTIAL * totalFloorArea; // dm³/s
  const avgSfp = project.ventilationSystems.reduce((s, v) => s + v.sfp, 0) /
    project.ventilationSystems.length;

  return avgSfp * designAirflow * VENTILATION_FAN_TIME_FACTOR * USAGE_HOURS_PER_YEAR / 1000;
}

/**
 * Calculate heat pump renewable energy (energy extracted from environment).
 */
function calculateHeatPumpRenewable(project: IProject, heatingDemand: number): number {
  let hpRenewable = 0;
  for (const hs of project.heatingSystems) {
    if (
      (hs.type === 'heat_pump_air' || hs.type === 'heat_pump_ground') &&
      hs.cop > 1
    ) {
      hpRenewable += heatingDemand * hs.coverageFraction * (1 - 1 / hs.cop);
    }
  }
  return hpRenewable;
}

/**
 * Main monthly BENG calculation function.
 *
 * Orchestrates all monthly sub-calculations, sums to annual values,
 * and produces complete BENG 1/2/3 + TO-juli results with monthly breakdown.
 *
 * @param project - Complete project definition
 * @returns Full monthly BENG result with pass/fail, monthly breakdown, and TO-juli
 */
export function calculateBENGMonthly(project: IProject): IBENGResultMonthly {
  if (hasUnmodelledUnheatedTransmission(project)) {
    throw new Error('Unheated space transmission is not included in the legacy indicative calculator.');
  }
  if (project.ntaHeatPumps?.length) {
    throw new Error('Standalone NTA heat pumps are not included in the legacy indicative calculator.');
  }
  if (hasUnmodelledHeatPumpDetails(project)) {
    throw new Error('Classified heat pump details are not included in the legacy indicative calculator.');
  }
  assertLegacyHeatPumpInputs(project);
  // --- Total floor area ---
  const totalFloorArea = validProjectFloorArea(project);
  if (totalFloorArea === null) throw new Error('Every calculation zone needs a finite floor area greater than zero.');

  // --- Monthly calculations ---
  const monthlyTransmissionLoss = calculateMonthlyTransmissionLoss(
    project.zones,
    project.constructions
  );

  const { ventilationLoss: monthlyVentilationLoss, infiltrationLoss: monthlyInfiltrationLoss } =
    calculateMonthlyVentilationLoss(project.zones, project.ventilationSystems);

  const monthlySolarGain = calculateMonthlySolarGain(project.zones);

  const monthlyInternalGain = calculateMonthlyInternalGain(
    totalFloorArea,
    project.buildingFunction
  );

  // --- Monthly heating demand ---
  const { heatingDemand: monthlyHeatingDemand, utilizationFactor: monthlyUtilHeating } =
    calculateMonthlyHeatingDemand(
      monthlyTransmissionLoss,
      monthlyVentilationLoss,
      monthlyInfiltrationLoss,
      monthlySolarGain,
      monthlyInternalGain
    );

  // --- Cooling-mode losses at θ_int_cool = 25°C ---
  // NTA 8800 §8.3: Cooling uses losses at cooling setpoint, NOT heating setpoint.
  // WTW is bypassed in cooling mode (summer bypass).
  const coolingSetpoint = 25; // °C

  // Transmission losses at 25°C (same Ht coefficient, different setpoint)
  const ht = calculateTransmissionLossCoefficient(project.zones, project.constructions);
  const monthlyCoolingTransLoss = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] as MonthlyValues;

  // Ventilation + infiltration losses at 25°C (WTW bypassed)
  const monthlyCoolingVentLoss = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] as MonthlyValues;
  const monthlyCoolingInfLoss = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] as MonthlyValues;

  let totalHv = 0;
  let totalHi = 0;
  for (const zone of project.zones) {
    const designFlow = VENTILATION_DESIGN_FLOW_RATE_RESIDENTIAL * zone.floorArea;
    totalHv += 0.34 * designFlow * 3.6; // W/K
    const qv10Corr = QV10_CORRECTION_FACTOR * zone.airTightness.qv10 * zone.volume;
    totalHi += 0.34 * qv10Corr * 3.6; // W/K
  }

  for (let m = 0; m < 12; m++) {
    const deltaT = Math.max(0, coolingSetpoint - MONTHLY_AVERAGE_TEMPERATURE[m]);
    const factor = deltaT * HOURS_PER_MONTH[m] / 1000;
    monthlyCoolingTransLoss[m] = ht * factor;
    monthlyCoolingVentLoss[m] = totalHv * factor;  // NO WTW in cooling mode (summer bypass)
    monthlyCoolingInfLoss[m] = totalHi * factor;
  }

  // --- Monthly cooling demand (summer balance at 25°C) ---
  const { coolingDemand: monthlyCoolingDemand, utilizationFactor: monthlyUtilCooling } =
    calculateMonthlyCoolingDemand(
      monthlyCoolingTransLoss,
      monthlyCoolingVentLoss,
      monthlyCoolingInfLoss,
      monthlySolarGain,
      monthlyInternalGain
    );

  // --- Monthly PV production ---
  const monthlyPVProduction = calculateMonthlyPVProduction(project);

  // --- Monthly solar thermal production ---
  const monthlySolarThermalProduction = calculateMonthlySolarThermalProduction(project);

  // --- TO-juli (summer comfort) ---
  const toJuli = calculateTOJuli(project, monthlySolarGain, totalFloorArea);

  // --- Annual sums ---
  const transmissionLoss = sumMonthly(monthlyTransmissionLoss);
  const ventilationLoss = sumMonthly(monthlyVentilationLoss);
  const infiltrationLoss = sumMonthly(monthlyInfiltrationLoss);
  const solarGain = sumMonthly(monthlySolarGain);
  const internalGain = sumMonthly(monthlyInternalGain);
  const heatingDemand = sumMonthly(monthlyHeatingDemand);
  const coolingDemand = sumMonthly(monthlyCoolingDemand);
  const pvProduction = sumMonthly(monthlyPVProduction);
  const solarThermalProduction = sumMonthly(monthlySolarThermalProduction);

  // --- Hot water demand (NTA 8800: scaled with Ag for residential) ---
  const hotWaterDemand = getHotWaterDemand(project.buildingFunction, totalFloorArea);

  // --- Ventilation fan energy ---
  const ventilationEnergy = calculateVentilationFanEnergy(project, totalFloorArea);

  // --- Lighting energy ---
  const lightingSpecific = LIGHTING_ENERGY[project.buildingFunction]
    ?? LIGHTING_ENERGY.residential;
  const lightingEnergy = lightingSpecific * totalFloorArea;

  // --- Primary energy ---
  const primaryResult = calculatePrimaryEnergy(
    heatingDemand,
    coolingDemand,
    hotWaterDemand,
    ventilationEnergy,
    lightingEnergy,
    project.heatingSystems,
    project.coolingSystems,
    project.hotWaterSystems,
    project.ventilationSystems
  );

  // --- Auxiliary energy (NTA 8800 §12) ---
  // Includes source-side pump/fan, defrost, circulation pump, controls/standby.
  // Scales with heating delivered energy (system operation) and floor area (distribution).
  const auxiliaryEnergy = 0.08 * primaryResult.heatingEnergy + 0.5 * totalFloorArea;
  const auxiliaryPrimary = auxiliaryEnergy * 1.45; // PRIMARY_ENERGY_FACTOR_ELECTRICITY_FOSSIL

  // Gross fossil primary including auxiliary
  const grossPrimaryEnergy = primaryResult.totalPrimaryEnergy + auxiliaryPrimary;

  // --- Renewable energy ---
  const heatPumpRenewable = calculateHeatPumpRenewable(project, heatingDemand);
  const totalRenewableEnergy = pvProduction + solarThermalProduction + heatPumpRenewable;

  // --- BENG indicators ---
  const beng1 = (heatingDemand + coolingDemand) / totalFloorArea;

  // BENG2: net fossil primary = gross primary - PV credit (in primary energy terms)
  // NTA 8800 §16.3: Full PV production is credited (no self-consumption cap).
  const pvPrimaryCredit = pvProduction * 1.45; // PRIMARY_ENERGY_FACTOR_ELECTRICITY_FOSSIL
  const netPrimaryEnergy = grossPrimaryEnergy - pvPrimaryCredit;
  const beng2 = Math.max(0, netPrimaryEnergy) / totalFloorArea;

  // BENG3: renewable share uses NET fossil primary (after PV credit)
  // NTA 8800: E_P;ren;tot / (E_P;ren;tot + E_P;nren;tot)
  // where E_P;nren;tot = net fossil primary (= BENG2 × Ag)
  const renewableShare = calculateRenewableShare(
    pvProduction,
    solarThermalProduction,
    heatPumpRenewable,
    Math.max(0, netPrimaryEnergy)
  );
  const beng3 = renewableShare;

  // --- Limits ---
  const limits = BENG_LIMITS[project.buildingFunction] ?? BENG_LIMITS.residential;

  // --- Energy breakdown ---
  const breakdown: IEnergyBreakdown = {
    transmissionLoss,
    ventilationLoss,
    infiltrationLoss,
    solarGain,
    internalGain,
    heatingDemand,
    coolingDemand,
    heatingEnergy: primaryResult.heatingEnergy,
    coolingEnergy: primaryResult.coolingEnergy,
    ventilationEnergy: primaryResult.ventilationEnergy,
    hotWaterEnergy: primaryResult.hotWaterEnergy,
    lightingEnergy: primaryResult.lightingEnergy,
    auxiliaryEnergy,
    totalPrimaryEnergy: grossPrimaryEnergy,
    renewableEnergy: totalRenewableEnergy,
    pvProduction,
    solarThermalProduction,
  };

  // --- Monthly breakdown ---
  const monthly: IMonthlyBreakdown[] = [];
  for (let m = 0; m < 12; m++) {
    monthly.push({
      month: m,
      transmissionLoss: monthlyTransmissionLoss[m],
      ventilationLoss: monthlyVentilationLoss[m],
      infiltrationLoss: monthlyInfiltrationLoss[m],
      solarGain: monthlySolarGain[m],
      internalGain: monthlyInternalGain[m],
      heatingDemand: monthlyHeatingDemand[m],
      coolingDemand: monthlyCoolingDemand[m],
      utilizationFactorHeating: monthlyUtilHeating[m],
      utilizationFactorCooling: monthlyUtilCooling[m],
    });
  }

  return {
    beng1,
    beng2,
    beng3,
    beng1Limit: limits.beng1,
    beng2Limit: limits.beng2,
    beng3Limit: limits.beng3,
    beng1Pass: beng1 <= limits.beng1,
    beng2Pass: beng2 <= limits.beng2,
    beng3Pass: beng3 >= limits.beng3,
    breakdown,
    totalFloorArea,
    monthly,
    toJuli,
    monthlyPVProduction,
    monthlySolarThermalProduction,
  };
}
