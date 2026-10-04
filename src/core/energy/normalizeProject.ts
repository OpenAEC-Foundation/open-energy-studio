import type { IConstruction, IProject, ISurface, IZone } from './types';

/**
 * Fills the arrays the editor relies on when a project file left them out
 * (older versions, hand-edited or foreign files). A project that already has
 * them is returned unchanged, so a normal open never alters the input
 * fingerprint. Optional fields whose absence means something (such as the
 * point-bridge inventory) are left alone.
 */
/** Importer default for a missing air tightness, dm³/(s·m²) at 10 Pa (same as UNIEC3Exporter and the zone editor). */
export const ASSUMED_QV10 = 0.4;

export function normalizeProject(project: IProject): IProject {
  if (project == null || typeof project !== 'object' || Array.isArray(project)) {
    throw new Error('Invalid project: expected an object');
  }
  if (typeof project.id !== 'string' || project.id.trim() === '') {
    throw new Error('Invalid project: id is missing');
  }
  if (typeof project.buildingFunction !== 'string' || project.buildingFunction.trim() === '') {
    throw new Error('Invalid project: buildingFunction is missing');
  }
  let changed = false;
  const list = <T,>(value: T[] | undefined | null, path: string): T[] => {
    if (Array.isArray(value)) {
      if (value.some((item) => item == null || typeof item !== 'object' || Array.isArray(item))) {
        throw new Error(`Invalid project: ${path} contains a non-object item`);
      }
      return value;
    }
    if (value != null) throw new Error(`Invalid project: ${path} must be an array`);
    changed = true;
    return [];
  };
  const text = (value: string | undefined | null, path: string): string => {
    if (typeof value === 'string') return value;
    if (value != null) throw new Error(`Invalid project: ${path} must be text`);
    changed = true;
    return '';
  };
  const raw = project as Partial<IProject>;
  const zones = list(raw.zones, 'zones').map((zone: IZone, zoneIndex) => {
    const surfaces = list(zone.surfaces, `zones[${zoneIndex}].surfaces`).map((surface: ISurface, surfaceIndex) => {
      const windows = list(surface.windows, `zones[${zoneIndex}].surfaces[${surfaceIndex}].windows`);
      return windows === surface.windows ? surface : { ...surface, windows };
    });
    const thermalBridges = list(zone.thermalBridges, `zones[${zoneIndex}].thermalBridges`);
    // A missing air tightness gets the importer default (UNIEC3/zone editor: 0,4 dm³/(s·m²)),
    // flagged as assumed so the envelope view asks the user to check it. Never 0: that would
    // claim a perfectly airtight building in the simplified engine, the heat-pump sizing and
    // the UNIEC3 export.
    let airTightness = zone.airTightness;
    if (airTightness != null && (typeof airTightness !== 'object' || Array.isArray(airTightness))) {
      throw new Error(`Invalid project: zones[${zoneIndex}].airTightness must be an object`);
    }
    if (airTightness == null || typeof airTightness.qv10 !== 'number' || !Number.isFinite(airTightness.qv10)) {
      changed = true;
      airTightness = { qv10: ASSUMED_QV10, assumed: true };
    }
    const same = surfaces.every((surface, index) => surface === zone.surfaces?.[index])
      && thermalBridges === zone.thermalBridges && airTightness === zone.airTightness;
    return same ? zone : { ...zone, surfaces, thermalBridges, airTightness };
  });
  const constructions = list(raw.constructions, 'constructions').map((construction: IConstruction, index) => {
    const layers = list(construction.layers, `constructions[${index}].layers`);
    return layers === construction.layers ? construction : { ...construction, layers };
  });
  const normalized: IProject = {
    ...project,
    name: text(raw.name, 'name'),
    description: text(raw.description, 'description'),
    address: text(raw.address, 'address'),
    city: text(raw.city, 'city'),
    zones,
    heatingSystems: list(raw.heatingSystems, 'heatingSystems'),
    ventilationSystems: list(raw.ventilationSystems, 'ventilationSystems'),
    coolingSystems: list(raw.coolingSystems, 'coolingSystems'),
    hotWaterSystems: list(raw.hotWaterSystems, 'hotWaterSystems'),
    solarPV: list(raw.solarPV, 'solarPV'),
    solarThermal: list(raw.solarThermal, 'solarThermal'),
    constructions,
  };
  return changed ? normalized : project;
}
