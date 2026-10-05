import type { INtaHeatPumpInput, IProject } from './types';

/** Legacy calculation guard; this does not establish normative Ag measurement. */
export function validProjectFloorArea(project: IProject): number | null {
  if (project.zones.length === 0) return null;
  let total = 0;
  for (const zone of project.zones) {
    if (!Number.isFinite(zone.floorArea) || zone.floorArea <= 0) return null;
    total += zone.floorArea;
  }
  return Number.isFinite(total) && total > 0 ? total : null;
}

/** Reject malformed legacy heat-pump inputs before producing even indicative results. */
export function legacyHeatPumpInputIssue(project: IProject): 'cop' | 'coverage' | null {
  for (const system of project.heatingSystems) {
    if (system.type !== 'heat_pump_air' && system.type !== 'heat_pump_ground') continue;
    if (!Number.isFinite(system.cop) || system.cop <= 0) {
      return 'cop';
    }
    if (!Number.isFinite(system.coverageFraction)
      || system.coverageFraction < 0 || system.coverageFraction > 1) {
      return 'coverage';
    }
  }
  return null;
}

export function assertLegacyHeatPumpInputs(project: IProject): void {
  const issue = legacyHeatPumpInputIssue(project);
  if (issue === 'cop') {
    throw new Error('Legacy heat pump COP must be finite and greater than zero.');
  }
  if (issue === 'coverage') {
    throw new Error('Legacy heat pump coverage fraction must be between zero and one.');
  }
}

/** The legacy model has no route from declared equipment details to seasonal use. */
export function hasUnmodelledHeatPumpDetails(project: IProject): boolean {
  const hasDetails = (pump?: INtaHeatPumpInput) =>
    Boolean(pump?.performancePoints?.length || pump?.dhwTestPoints?.length || pump?.declaredOperatingLimits
      || pump?.auxiliaryComponents?.length || pump?.systemLinks?.length);
  return project.heatingSystems.some((system) => hasDetails(system.ntaHeatPump))
    || project.hotWaterSystems.some((system) => hasDetails(system.ntaHeatPump));
}

/** The legacy indicative route has no named unheated-space reduction path. */
export function hasUnmodelledUnheatedTransmission(project: IProject): boolean {
  return Boolean(project.unheatedSpaces?.length) || project.zones.some((zone) =>
    zone.surfaces.some((item) => item.thermalBoundary === 'unheated_space')
    || zone.thermalBridges.some((item) => item.thermalBoundary === 'unheated_space')
    || zone.pointThermalBridges?.some((item) => item.thermalBoundary === 'unheated_space'));
}
