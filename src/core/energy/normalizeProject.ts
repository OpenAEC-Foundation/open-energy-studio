import type { IConstruction, IProject, ISurface, IZone } from './types';
import { migrateEvidenceLinks, withSurveyItemIds } from '../nta/EvidenceLinks';

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
  // Project-model numbers the NTA calculation does not read still feed the editor, the simplified
  // engine and the exports; a negative or non-finite value there is a broken file, not an input.
  const nonNegative = (value: unknown, path: string) => {
    if (value == null) return;
    if (typeof value !== 'number' || !Number.isFinite(value) || value < 0) {
      throw new Error(`Invalid project: ${path} must be a non-negative finite number`);
    }
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
    if (airTightness == null || airTightness.qv10 == null) {
      changed = true;
      airTightness = { qv10: ASSUMED_QV10, assumed: true };
    } else if (typeof airTightness.qv10 !== 'number' || !Number.isFinite(airTightness.qv10)
      || airTightness.qv10 <= 0) {
      throw new Error(`Invalid project: zones[${zoneIndex}].airTightness.qv10 must be a positive finite number`);
    }
    for (const field of ['floorArea', 'volume', 'height'] as const) {
      nonNegative(zone[field], `zones[${zoneIndex}].${field}`);
    }
    const same = surfaces.every((surface, index) => surface === zone.surfaces?.[index])
      && thermalBridges === zone.thermalBridges && airTightness === zone.airTightness;
    return same ? zone : { ...zone, surfaces, thermalBridges, airTightness };
  });
  const constructions = list(raw.constructions, 'constructions').map((construction: IConstruction, index) => {
    nonNegative(construction.rcValue, `constructions[${index}].rcValue`);
    nonNegative(construction.uValue, `constructions[${index}].uValue`);
    const layers = list(construction.layers, `constructions[${index}].layers`);
    return layers === construction.layers ? construction : { ...construction, layers };
  });
  const windowTypes = raw.windowTypes === undefined ? undefined : list(raw.windowTypes, 'windowTypes').map((type, index) => {
    nonNegative(type.uValue, `windowTypes[${index}].uValue`);
    nonNegative(type.gValue, `windowTypes[${index}].gValue`);
    nonNegative(type.unitArea, `windowTypes[${index}].unitArea`);
    return type;
  });
  const normalized: IProject = {
    ...project,
    ...(windowTypes !== undefined && windowTypes !== raw.windowTypes ? { windowTypes } : {}),
    name: text(raw.name, 'name'),
    description: text(raw.description, 'description'),
    address: text(raw.address, 'address'),
    city: text(raw.city, 'city'),
    zones,
    heatingSystems: list(raw.heatingSystems, 'heatingSystems').map((system, index) => {
      nonNegative(system.cop, `heatingSystems[${index}].cop`);
      if (typeof system.coverageFraction === 'number' && system.coverageFraction > 1) {
        throw new Error(`Invalid project: heatingSystems[${index}].coverageFraction must lie between 0 and 1`);
      }
      nonNegative(system.coverageFraction, `heatingSystems[${index}].coverageFraction`);
      return system;
    }),
    ventilationSystems: list(raw.ventilationSystems, 'ventilationSystems'),
    coolingSystems: list(raw.coolingSystems, 'coolingSystems'),
    hotWaterSystems: list(raw.hotWaterSystems, 'hotWaterSystems'),
    solarPV: list(raw.solarPV, 'solarPV'),
    solarThermal: list(raw.solarThermal, 'solarThermal'),
    constructions,
  };
  // Evidence links of older projects named elements by position; they become id-based. Survey
  // lists that had no ids get them first, so their links can follow too.
  return migrateEvidenceLinks(withSurveyItemIds(changed ? normalized : project));
}
