import type { IEnergyBreakdown } from '../energy/types';
import type { BuildingPerformanceAssessment, MonthlyDemandAssessment } from './KernelClient';
import { summarizeServiceEnergy } from './ServiceEnergy';

/**
 * The energy balance of the results chart, taken from the NTA kernel instead of the
 * simplified engine, so the chart never contradicts the kernel indicators.
 *
 * Losses and gains are the heating-balance terms of chapter 7 summed over zones and
 * months; infiltration is part of the ventilation term there and is reported as 0.
 * The gains total is the kernel's own Q_H;gn (7.4), so terms beyond solar and internal
 * gains (sunroom gains 7.37) appear as `otherGain` instead of being lost.
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

  const solarGain = sum(months.map((row) => row.windowSolarGainsKwh + row.opaqueSolarGainsKwh));
  const internalGain = sum(months.map((row) => row.internalGainsKwh));
  // Q_H;gn per month (7.4); without it in the output, nothing beyond solar and internal gains is known.
  const gainsKnown = months.length > 0 && months.every((row) => Number.isFinite(row.heating?.gainsKwh));
  const otherGain = gainsKnown ? sum(months.map((row) => row.heating.gainsKwh)) - solarGain - internalGain : 0;

  return {
    transmissionLoss: sum(months.map((row) => row.heating.transmissionKwh)),
    ventilationLoss: sum(months.map((row) => row.heating.ventilationKwh)),
    infiltrationLoss: 0,
    solarGain,
    internalGain,
    otherGain: Math.abs(otherGain) > 0.5 ? otherGain : 0,
    deliveredUnavailable: services == null,
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
