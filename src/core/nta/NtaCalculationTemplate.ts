import type { IProject } from '../energy/types';
import { DEFAULT_NORM_VERSION } from './KernelClient';
import { readDefaultEdition } from './defaultEdition';

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
  // Settings › Berekening: an older default edition is written explicitly.
  const edition = readDefaultEdition();
  return {
    ...(edition !== DEFAULT_NORM_VERSION ? { normVersion: edition } : {}),
    calculationScope: residential ? 'residential' : 'utility',
    areaSourceReference: '',
    // Tables 7.13–7.15 follow the usage function; 7.78 needs the dwelling type.
    usageFunction: residential ? 'residential' : null,
    dwellingType: null,
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
    windowSolar: { frameFraction: null, obstruction: { method: 'minimal' }, sourceReference: '' },
    groundFloors: surfaces
      .filter((surface) => surface.thermalBoundary === 'ground')
      .map((surface) => ({
        surfaceId: surface.id,
        exposedPerimeterM: null,
        constructionResistanceM2kPerW: null,
        edgeThermalBridges: null,
        sourceReference: '',
      })),
    // 7.3.3: unknown until the adviser lists the pipes or states none.
    ...(project.zones.length > 1 ? {} : { verticalPipes: null }),
    ventilationFlows: [{
      id: 'ventilation',
      sourceReference: '',
      months: Array.from({ length: 12 }, (_, index) => ({ month: index + 1, conductanceWPerK: null })),
    }],
    ...(project.zones.length > 1 ? {
      zoneData: project.zones.map((zone) => ({
        zoneId: zone.id,
        verticalPipes: null,
        ventilationFlows: [{
          id: `ventilation-${zone.id}`,
          sourceReference: '',
          months: Array.from({ length: 12 }, (_, index) => ({ month: index + 1, conductanceWPerK: null })),
        }],
        internalGains: residential
          ? { method: 'residential', dwellingCount: null, sourceReference: '' }
          : { method: 'declared', heatFluxWPerM2: null, sourceReference: '' },
      })),
    } : {}),
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
      peakPower: pv.area > 0
        ? { method: 'declared_specific', peakPowerWPerM2: Math.round((pv.peakPower * 1000 / pv.area) * 100) / 100, panelAreaM2: pv.area }
        : { method: 'panels', panelPeakPowerW: null, panelCount: null },
      azimuthDeg: null,
      tiltDeg: pv.tilt,
      mounting: 'unknown',
      obstructionFactors: [null],
      sourceReference: '',
    })),
    hotWater: {
      need: residential
        ? { method: 'residential', dwellingCount: 1, sourceReference: '' }
        : { method: 'utility', areas: [{ function: null, areaM2: null }], sourceReference: '' },
      emission: residential
        ? { method: 'residential', served: 'kitchen_and_bathroom', kitchenLengthM: null, bathroomLengthM: null, sourceReference: '' }
        : { method: 'utility', meanLengthM: null, sourceReference: '' },
      generator: { kind: null },
      equipmentReference: '',
    },
    labelFunction: residential ? 'residential' : null,
    bblFunction: null,
    activeCooling: null,
    demandUsesFixedC1Ventilation: false,
    batteryStoragePresent: false,
  };
}
