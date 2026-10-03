import type { IEnergyBreakdown } from '../energy/types';
import type { BuildingPerformanceAssessment, MonthlyDemandAssessment } from './KernelClient';
import { summarizeServiceEnergy } from './ServiceEnergy';

/**
 * The energy balance of the results chart, taken from the NTA kernel instead of the
 * simplified engine, so the chart never contradicts the kernel indicators.
 *
 * Losses and gains are the heating-balance terms of chapter 7 summed over zones and
 * months; infiltration is part of the ventilation term there and is reported as 0.
 * Delivered energy is the §5.5.3 delivered energy per energy function; PV is the
 * chapter 16 production; solar thermal is the chapter 13 solar yield plus standalone
 * space-heating solar systems.
 */
export function kernelEnergyBreakdown(performance: BuildingPerformanceAssessment): IEnergyBreakdown {
  const heating = performance.spaceHeating;
  const zones: Array<Partial<MonthlyDemandAssessment>> = [heating.demand, ...(heating.additionalZoneDemands ?? [])];
  const months = zones.flatMap((zone) => zone.monthly ?? []);
  const sum = (values: number[]) => values.reduce((total, value) => total + (Number.isFinite(value) ? value : 0), 0);

  const services = summarizeServiceEnergy(performance.energyByService);
  const delivered = (service: string) => sum((services?.rows ?? [])
    .filter((row) => row.service === service).map((row) => row.deliveredKwh));

  const hotWater = performance.hotWater;
  const solarThermal = (hotWater ? hotWater.annualSolarRenewableKwh + hotWater.annualSolarSpaceHeatingKwh : 0)
    + sum(performance.standaloneSolar?.spaceHeatingKwh ?? []);

  return {
    transmissionLoss: sum(months.map((row) => row.heating.transmissionKwh)),
    ventilationLoss: sum(months.map((row) => row.heating.ventilationKwh)),
    infiltrationLoss: 0,
    solarGain: sum(months.map((row) => row.windowSolarGainsKwh + row.opaqueSolarGainsKwh)),
    internalGain: sum(months.map((row) => row.internalGainsKwh)),
    heatingDemand: sum(zones.map((zone) => zone.annualHeatingNeedKwh ?? 0)),
    coolingDemand: sum(zones.map((zone) => zone.annualCoolingNeedKwh ?? 0)),
    heatingEnergy: delivered('heating'),
    coolingEnergy: delivered('cooling'),
    ventilationEnergy: delivered('ventilation'),
    hotWaterEnergy: delivered('hotWater'),
    lightingEnergy: delivered('lighting'),
    auxiliaryEnergy: delivered('auxiliary') + delivered('humidification') + delivered('heatPumpSource'),
    totalPrimaryEnergy: performance.annualPrimaryFossilKwh ?? 0,
    renewableEnergy: performance.annualRenewablePrimaryKwh ?? 0,
    pvProduction: sum((performance.pvSystems ?? []).map((system) => system.annualKwh)),
    solarThermalProduction: solarThermal,
  };
}
