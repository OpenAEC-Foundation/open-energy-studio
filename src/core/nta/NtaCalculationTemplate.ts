import type { IProject } from '../energy/types';

/**
 * Starting point for the `ntaCalculation` block. Numbers the norm fixes for
 * the project's category are prefilled; project-specific values are `null`
 * and every source reference is empty, so the Rust kernel keeps reporting a
 * gap until the adviser has entered and substantiated each value.
 */
export function buildNtaCalculationTemplate(project: IProject): Record<string, unknown> {
  const residential = project.buildingFunction === 'residential';
  const surfaces = project.zones.flatMap((zone) => zone.surfaces);
  const heating = project.heatingSystems[0];
  const heatPump = heating?.type === 'heat_pump_air' || heating?.type === 'heat_pump_ground';
  const savedForfait = heating?.ntaHeatPump?.forfaitHeatPumpDraft;
  return {
    calculationScope: residential ? 'residential' : 'utility',
    areaSourceReference: '',
    setpoints: residential
      ? { heatingC: 20, coolingC: 24, sourceReference: '' }
      : { heatingC: null, coolingC: null, sourceReference: '' },
    thermalMass: {
      floor: null,
      wall: null,
      ceiling: residential ? 'open_or_none' : 'closed_or_suspended',
      sourceReference: '',
    },
    internalGains: residential
      ? { method: 'residential', dwellingCount: 1, sourceReference: '' }
      : { method: 'declared', heatFluxWPerM2: null, sourceReference: '' },
    surfaceTilts: surfaces
      .filter((surface) => surface.type === 'roof' && surface.orientation !== 'horizontal')
      .map((surface) => ({ surfaceId: surface.id, tiltDeg: null, sourceReference: '' })),
    windowSolar: { frameFraction: null, obstructionFactor: null, sourceReference: '' },
    groundFloors: surfaces
      .filter((surface) => surface.thermalBoundary === 'ground')
      .map((surface) => ({
        surfaceId: surface.id,
        exposedPerimeterM: null,
        constructionResistanceM2kPerW: null,
        sourceReference: '',
      })),
    ventilationFlows: [{
      id: 'ventilation',
      sourceReference: '',
      months: Array.from({ length: 12 }, (_, index) => ({ month: index + 1, conductanceWPerK: null })),
    }],
    emission: {
      system: 'other_or_unknown',
      balancing: 'none_or_unknown',
      control: 'other_or_unknown',
      sourceReference: '',
    },
    distribution: { method: 'heated_zone_only_space_heating', sourceReference: '' },
    generator: heatPump
      ? {
          kind: 'heat_pump_forfait',
          forfait: savedForfait ?? null,
          sourceSystem: 'individual',
          sourceSystemReference: '',
        }
      : {
          kind: 'gas_boiler',
          boiler: {
            generatorId: heating?.id ?? 'boiler',
            role: 'individual_main',
            location: null,
            kind: null,
            fuel: 'natural_gas',
            averageDesignEmissionTemperatureC: null,
            emissionCircuit: null,
            equipmentReference: '',
            locationReference: '',
            temperatureAndCircuitReference: '',
            pilotFlamePresent: false,
          },
        },
    ...(heatPump ? { heatPumpRenewable: { sourceBelow20C: null, exhaustAirSource: null, sourceReference: '' } } : {}),
    bacsFactor: 1,
    bacsSourceReference: '',
    useInventoryComplete: false,
    declaredUses: [],
    productionInventoryComplete: false,
    pvSystems: project.solarPV.map((pv) => ({
      id: pv.id,
      peakPowerKw: pv.peakPower,
      azimuthDeg: null,
      tiltDeg: pv.tilt,
      performanceFactor: null,
      shadingCorrection: null,
      obstructionFactor: null,
      sourceReference: '',
    })),
    hotWater: {
      need: residential
        ? { method: 'residential', dwellingCount: 1, sourceReference: '' }
        : { method: 'declared', specificNeedKwhPerM2Year: null, sourceReference: '' },
      emissionEfficiency: null,
      distributionEfficiency: null,
      generationEfficiency: null,
      carrier: null,
      renewableHeatPump: false,
      efficiencySourceReference: '',
    },
    labelFunction: residential ? 'residential' : null,
    bblFunction: null,
    activeCoolingPresent: false,
    demandUsesFixedC1Ventilation: false,
    batteryStoragePresent: false,
  };
}
