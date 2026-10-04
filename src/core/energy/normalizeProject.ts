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
  let changed = false;
  const list = <T,>(value: T[] | undefined | null): T[] => {
    if (Array.isArray(value)) return value;
    changed = true;
    return [];
  };
  const text = (value: string | undefined | null): string => {
    if (typeof value === 'string') return value;
    changed = true;
    return '';
  };
  const raw = project as Partial<IProject>;
  const zones = list(raw.zones).map((zone: IZone) => {
    const surfaces = list(zone.surfaces).map((surface: ISurface) => {
      const windows = list(surface.windows);
      return windows === surface.windows ? surface : { ...surface, windows };
    });
    const thermalBridges = list(zone.thermalBridges);
    // A missing air tightness gets the importer default (UNIEC3/zone editor: 0,4 dm³/(s·m²)),
    // flagged as assumed so the envelope view asks the user to check it. Never 0: that would
    // claim a perfectly airtight building in the simplified engine, the heat-pump sizing and
    // the UNIEC3 export.
    let airTightness = zone.airTightness;
    if (airTightness == null || typeof airTightness.qv10 !== 'number' || !Number.isFinite(airTightness.qv10)) {
      changed = true;
      airTightness = { qv10: ASSUMED_QV10, assumed: true };
    }
    const same = surfaces.every((surface, index) => surface === zone.surfaces?.[index])
      && thermalBridges === zone.thermalBridges && airTightness === zone.airTightness;
    return same ? zone : { ...zone, surfaces, thermalBridges, airTightness };
  });
  const constructions = list(raw.constructions).map((construction: IConstruction) => {
    const layers = list(construction.layers);
    return layers === construction.layers ? construction : { ...construction, layers };
  });
  const normalized: IProject = {
    ...project,
    name: text(raw.name),
    description: text(raw.description),
    address: text(raw.address),
    city: text(raw.city),
    zones,
    heatingSystems: list(raw.heatingSystems),
    ventilationSystems: list(raw.ventilationSystems),
    coolingSystems: list(raw.coolingSystems),
    hotWaterSystems: list(raw.hotWaterSystems),
    solarPV: list(raw.solarPV),
    solarThermal: list(raw.solarThermal),
    constructions,
  };
  return changed ? normalized : project;
}
