/**
 * UNIEC3 Exporter & Importer
 * Real UNIEC3 format: ZIP archive containing JSON files
 * Based on analysis of actual UNIEC3 files (NTA 8800 v3.4)
 *
 * ZIP structure:
 *   meta.json           - Version info
 *   folders.json        - Folder structure
 *   projects.json       - Project metadata
 *   buildings.json      - Building list
 *   buildings/{id}/
 *     entities.json     - All NTA entities with properties
 *     relations.json    - Parent-child relationships
 *     deltas.json       - Change tracking (empty on export)
 *     summary.json      - Calculation results summary
 */

import { zipSync, unzipSync, strToU8, strFromU8 } from 'fflate';
import type {
  IProject, IBENGResult, IBENGResultMonthly,
  Orientation, SurfaceType, HeatingSystemType,
  VentilationType, CoolingSystemType, HotWaterSystemType,
  SolarThermalType, BuildingFunction,
  IZone, ISurface, IWindow, IThermalBridge,
  IHeatingSystem, IVentilationSystem, ICoolingSystem,
  IHotWaterSystem, ISolarPV, ISolarThermal, IConstruction,
} from '../energy/types';

// ============================================================
// UNIEC3 JSON types
// ============================================================

interface UNIEC3Property {
  NTAPropertyId: string;
  Value: string;
  IsHidden?: boolean;
}

interface UNIEC3Entity {
  NTAEntityDataId: string;
  NTAEntityId: string;
  NTAPropertyDatas: UNIEC3Property[];
}

interface UNIEC3Relation {
  NTAEntityDataId: string;
  NTAEntityRelationId: string;
  ParentNTAEntityDataId: string;
  ChildNTAEntityDataId: string;
}

interface UNIEC3Meta {
  VersionId: number;
  NTAVersion: string;
  ReleaseDate: string;
  ExportDate: string;
  Generator: string;
}

interface UNIEC3Project {
  ProjectId: number;
  Name: string;
  Description: string;
  CreateDate: string;
  ChangeDate: string;
}

interface UNIEC3Building {
  BuildingId: number;
  ProjectId: number;
  NTAVersionId: number;
  Locked: boolean;
  Afgemeld: boolean;
  Afmeldstatus: number;
  CreateDate: string;
  ChangeDate: string;
}

interface UNIEC3Summary {
  BENG1?: number;
  BENG2?: number;
  BENG3?: number;
  TOJuli?: number;
  Energielabel?: string;
}

// ============================================================
// GUID generator
// ============================================================

function newGuid(): string {
  return crypto.randomUUID();
}

// ============================================================
// NTA Entity & Property ID constants
// ============================================================

// Entity type IDs (NTAEntityId values from real UNIEC3 files)
const ENT = {
  GEB: 'GEB',             // Gebouw (building)
  UNIT: 'UNIT',           // Unit/dwelling
  RZ: 'RZ',               // Rekenzone (calculation zone)
  BEGR: 'BEGR',           // Begrenzingsvlak (boundary surface)
  CONSTRD: 'CONSTRD',     // Constructie dicht (opaque construction)
  CONSTRT: 'CONSTRT',     // Constructie transparant (window/glass)
  LIBCONSTRD: 'LIBCONSTRD', // Library: opaque construction
  LIBCONSTRT: 'LIBCONSTRT', // Library: transparent construction
  KOUDEBRUG: 'KOUDEBRUG', // Thermal bridge
  LUCHTDH: 'LUCHTDH',     // Air tightness
  INSTALLATIE: 'INSTALLATIE', // Installation group
  VERW: 'VERW',           // Heating group
  'VERW-OPWEK': 'VERW-OPWEK', // Heating generator
  VENT: 'VENT',           // Ventilation system
  KOEL: 'KOEL',           // Cooling group
  'KOEL-OPWEK': 'KOEL-OPWEK', // Cooling generator
  TAPW: 'TAPW',           // Hot water group
  'TAPW-OPWEK': 'TAPW-OPWEK', // Hot water generator
  PV: 'PV',               // PV system group
  'PV-VELD': 'PV-VELD',   // PV field/array
  ZONTHERM: 'ZONTHERM',   // Solar thermal
  PRESTATIE: 'PRESTATIE',  // Performance/results
  SETTINGS: 'SETTINGS',    // Project settings
} as const;

// Property IDs used in real UNIEC3 files
const PROP = {
  // Building
  GEB_NAAM: 'GEB_NAAM',
  GEB_ADRES: 'GEB_ADRES',
  GEB_PLAATS: 'GEB_PLAATS',
  GEB_FUNCTIE: 'GEB_FUNCTIE',
  GEB_OMSCHR: 'GEB_OMSCHR',

  // Zone
  RZ_NAAM: 'RZ_NAAM',
  RZ_AG: 'RZ_AG',           // Floor area m²
  RZ_VOLUME: 'RZ_VOLUME',   // Volume m³
  RZ_HOOGTE: 'RZ_HOOGTE',   // Height m

  // Surface (BEGR)
  BEGR_NAAM: 'BEGR_NAAM',
  BEGR_TYPE: 'BEGR_TYPE',       // GVL_BEGR (surface type code)
  BEGR_OPP: 'BEGR_OPP',         // Area m²
  BEGR_ORIENT: 'BEGR_ORIENT',   // Orientation code

  // Opaque construction
  CONSTRD_NAAM: 'CONSTRD_NAAM',
  CONSTRD_RC: 'CONSTRD_RC',     // Rc m²K/W
  CONSTRD_U: 'CONSTRD_U',       // U W/(m²K)

  // Transparent construction (window)
  CONSTRT_NAAM: 'CONSTRT_NAAM',
  CONSTRT_OPP: 'CONSTRT_OPP',   // Area m²
  CONSTRT_UW: 'CONSTRT_UW',     // Uw W/(m²K)
  CONSTRT_GWAARDE: 'CONSTRT_GWAARDE', // g-value
  CONSTRT_ORIENT: 'CONSTRT_ORIENT',   // Orientation

  // Thermal bridge
  KB_PSI: 'KB_PSI',             // Psi W/(mK)
  KB_LENGTE: 'KB_LENGTE',       // Length m
  KB_NAAM: 'KB_NAAM',

  // Air tightness
  LUCHTDH_QV10: 'LUCHTDH_QV10', // qv10 dm³/(s·m²)

  // Heating
  VERW_TYPE: 'VERW_TYPE',
  VERW_COP: 'VERW_COP',
  VERW_DEKKING: 'VERW_DEKKING',
  VERW_NAAM: 'VERW_NAAM',

  // Ventilation
  VENT_TYPE: 'VENT_TYPE',
  VENT_WTW: 'VENT_WTW',         // Heat recovery efficiency
  VENT_SFP: 'VENT_SFP',
  VENT_NAAM: 'VENT_NAAM',

  // Cooling
  KOEL_TYPE: 'KOEL_TYPE',
  KOEL_EER: 'KOEL_EER',
  KOEL_NAAM: 'KOEL_NAAM',

  // Hot water
  TAPW_TYPE: 'TAPW_TYPE',
  TAPW_REND: 'TAPW_REND',       // Efficiency
  TAPW_NAAM: 'TAPW_NAAM',
  TAPW_ZONTHERM: 'TAPW_ZONTHERM', // Has solar boiler
  TAPW_ZONTHERM_FRAC: 'TAPW_ZONTHERM_FRAC',

  // PV
  PV_NAAM: 'PV_NAAM',
  PV_WP: 'PV_WP',               // Peak power kWp
  PV_ORIENT: 'PV_ORIENT',
  PV_HELLING: 'PV_HELLING',     // Tilt degrees
  PV_OPP: 'PV_OPP',             // Area m²

  // Solar thermal
  ZT_NAAM: 'ZT_NAAM',
  ZT_OPP: 'ZT_OPP',             // Collector area
  ZT_TYPE: 'ZT_TYPE',
  ZT_ORIENT: 'ZT_ORIENT',
  ZT_HELLING: 'ZT_HELLING',

  // Performance/results
  PREST_BENG1: 'PREST_BENG1',
  PREST_BENG2: 'PREST_BENG2',
  PREST_BENG3: 'PREST_BENG3',
  PREST_TOJULI: 'PREST_TOJULI',
  PREST_LABEL: 'PREST_LABEL',
} as const;

// ============================================================
// Value mapping helpers
// ============================================================

/** Format number as Dutch locale string (comma decimal separator) */
function nlNum(n: number, decimals = 2): string {
  return n.toFixed(decimals).replace('.', ',');
}

function parseNlNum(s: string): number {
  return parseFloat(s.replace(',', '.'));
}

// Orientation mapping: our codes → UNIEC3 orientation codes
const ORIENT_TO_UNIEC3: Record<Orientation, string> = {
  'N': 'GVL_BTNL_N',
  'NE': 'GVL_BTNL_NO',
  'E': 'GVL_BTNL_O',
  'SE': 'GVL_BTNL_ZO',
  'S': 'GVL_BTNL_Z',
  'SW': 'GVL_BTNL_ZW',
  'W': 'GVL_BTNL_W',
  'NW': 'GVL_BTNL_NW',
  'horizontal': 'GVL_BTNL_HOR',
};

const ORIENT_FROM_UNIEC3: Record<string, Orientation> = Object.fromEntries(
  Object.entries(ORIENT_TO_UNIEC3).map(([k, v]) => [v, k as Orientation])
) as Record<string, Orientation>;

// Surface type mapping
const SURFACE_TO_UNIEC3: Record<SurfaceType, string> = {
  'wall': 'GVL_BEGR_GEVEL',
  'roof': 'GVL_BEGR_DAK',
  'floor': 'GVL_BEGR_VLOER',
  'internal': 'GVL_BEGR_INTERN',
};

const SURFACE_FROM_UNIEC3: Record<string, SurfaceType> = Object.fromEntries(
  Object.entries(SURFACE_TO_UNIEC3).map(([k, v]) => [v, k as SurfaceType])
) as Record<string, SurfaceType>;

// Heating type mapping
const HEATING_TO_UNIEC3: Record<HeatingSystemType, string> = {
  'hr107': 'GVL_VERW_HR107',
  'hr_combi': 'GVL_VERW_HRCOMBI',
  'heat_pump_air': 'GVL_VERW_WP_LUCHT',
  'heat_pump_ground': 'GVL_VERW_WP_BODEM',
  'district_heating': 'GVL_VERW_STADSVW',
  'electric': 'GVL_VERW_ELEK',
  'biomass': 'GVL_VERW_BIOM',
};

const HEATING_FROM_UNIEC3: Record<string, HeatingSystemType> = Object.fromEntries(
  Object.entries(HEATING_TO_UNIEC3).map(([k, v]) => [v, k as HeatingSystemType])
) as Record<string, HeatingSystemType>;

// Ventilation type mapping
const VENT_TO_UNIEC3: Record<VentilationType, string> = {
  'natural': 'GVL_VENT_NAT',
  'type_c': 'GVL_VENT_MECH_C',
  'type_d': 'GVL_VENT_BAL_D',
};

const VENT_FROM_UNIEC3: Record<string, VentilationType> = Object.fromEntries(
  Object.entries(VENT_TO_UNIEC3).map(([k, v]) => [v, k as VentilationType])
) as Record<string, VentilationType>;

// Cooling type mapping
const COOL_TO_UNIEC3: Record<CoolingSystemType, string> = {
  'none': 'GVL_KOEL_GEEN',
  'split_unit': 'GVL_KOEL_SPLIT',
  'central_chiller': 'GVL_KOEL_CENTRAAL',
  'heat_pump_reversible': 'GVL_KOEL_WP_REV',
};

const COOL_FROM_UNIEC3: Record<string, CoolingSystemType> = Object.fromEntries(
  Object.entries(COOL_TO_UNIEC3).map(([k, v]) => [v, k as CoolingSystemType])
) as Record<string, CoolingSystemType>;

// Hot water type mapping
const HW_TO_UNIEC3: Record<HotWaterSystemType, string> = {
  'hr_combi': 'GVL_TAPW_HRCOMBI',
  'heat_pump': 'GVL_TAPW_WP',
  'electric_boiler': 'GVL_TAPW_ELEK',
  'solar_boiler': 'GVL_TAPW_ZON',
  'district_heating': 'GVL_TAPW_STADSVW',
};

const HW_FROM_UNIEC3: Record<string, HotWaterSystemType> = Object.fromEntries(
  Object.entries(HW_TO_UNIEC3).map(([k, v]) => [v, k as HotWaterSystemType])
) as Record<string, HotWaterSystemType>;

// Building function mapping
const FUNC_TO_UNIEC3: Record<BuildingFunction, string> = {
  'residential': 'GVL_FUNC_WOON',
  'office': 'GVL_FUNC_KANTOOR',
  'education': 'GVL_FUNC_ONDERWIJS',
  'healthcare': 'GVL_FUNC_GEZONDHEID',
  'retail': 'GVL_FUNC_WINKEL',
  'industrial': 'GVL_FUNC_INDUSTRIE',
  'other': 'GVL_FUNC_OVERIG',
};

const FUNC_FROM_UNIEC3: Record<string, BuildingFunction> = Object.fromEntries(
  Object.entries(FUNC_TO_UNIEC3).map(([k, v]) => [v, k as BuildingFunction])
) as Record<string, BuildingFunction>;

// Solar thermal type mapping
const ST_TO_UNIEC3: Record<SolarThermalType, string> = {
  'flat_plate': 'GVL_ZT_VLAK',
  'vacuum_tube': 'GVL_ZT_VACUUM',
};

const ST_FROM_UNIEC3: Record<string, SolarThermalType> = Object.fromEntries(
  Object.entries(ST_TO_UNIEC3).map(([k, v]) => [v, k as SolarThermalType])
) as Record<string, SolarThermalType>;

// ============================================================
// Entity builder helpers
// ============================================================

function makeEntity(entityType: string, props: Record<string, string>): UNIEC3Entity {
  return {
    NTAEntityDataId: newGuid(),
    NTAEntityId: entityType,
    NTAPropertyDatas: Object.entries(props).map(([id, value]) => ({
      NTAPropertyId: id,
      Value: value,
    })),
  };
}

function makeRelation(parentId: string, childId: string, relationId: string): UNIEC3Relation {
  return {
    NTAEntityDataId: newGuid(),
    NTAEntityRelationId: relationId,
    ParentNTAEntityDataId: parentId,
    ChildNTAEntityDataId: childId,
  };
}

function getEntityProp(entity: UNIEC3Entity, propId: string): string | undefined {
  return entity.NTAPropertyDatas.find(p => p.NTAPropertyId === propId)?.Value;
}

// ============================================================
// EXPORT: IProject + IBENGResult → UNIEC3 ZIP
// ============================================================

export function exportToUNIEC3(project: IProject, result: IBENGResult): Uint8Array {
  const entities: UNIEC3Entity[] = [];
  const relations: UNIEC3Relation[] = [];

  const now = new Date().toISOString();
  const projectId = Math.floor(Math.random() * 900000) + 100000;
  const buildingId = Math.floor(Math.random() * 9000000) + 1000000;

  // --- Settings entity ---
  const settingsEntity = makeEntity(ENT.SETTINGS, {});
  entities.push(settingsEntity);

  // --- Building (GEB) entity ---
  const gebEntity = makeEntity(ENT.GEB, {
    [PROP.GEB_NAAM]: project.name || 'Naamloos',
    [PROP.GEB_ADRES]: project.address || '',
    [PROP.GEB_PLAATS]: project.city || '',
    [PROP.GEB_FUNCTIE]: FUNC_TO_UNIEC3[project.buildingFunction] || 'GVL_FUNC_WOON',
    [PROP.GEB_OMSCHR]: project.description || '',
  });
  entities.push(gebEntity);

  // --- Unit entity ---
  const unitEntity = makeEntity(ENT.UNIT, {});
  entities.push(unitEntity);
  relations.push(makeRelation(gebEntity.NTAEntityDataId, unitEntity.NTAEntityDataId, 'GEB_UNITS'));

  // --- Installation group entity ---
  const installEntity = makeEntity(ENT.INSTALLATIE, {});
  entities.push(installEntity);
  relations.push(makeRelation(unitEntity.NTAEntityDataId, installEntity.NTAEntityDataId, 'UNIT_INSTALLATIE'));

  // --- Zones ---
  for (const zone of project.zones) {
    const rzEntity = makeEntity(ENT.RZ, {
      [PROP.RZ_NAAM]: zone.name,
      [PROP.RZ_AG]: nlNum(zone.floorArea),
      [PROP.RZ_VOLUME]: nlNum(zone.volume),
      [PROP.RZ_HOOGTE]: nlNum(zone.height),
    });
    entities.push(rzEntity);
    relations.push(makeRelation(unitEntity.NTAEntityDataId, rzEntity.NTAEntityDataId, 'UNIT_RZ'));

    // Surfaces
    for (const surface of zone.surfaces) {
      const construction = project.constructions.find(c => c.id === surface.constructionId);

      const begrEntity = makeEntity(ENT.BEGR, {
        [PROP.BEGR_NAAM]: surface.name,
        [PROP.BEGR_TYPE]: SURFACE_TO_UNIEC3[surface.type] || 'GVL_BEGR_GEVEL',
        [PROP.BEGR_OPP]: nlNum(surface.area),
        [PROP.BEGR_ORIENT]: ORIENT_TO_UNIEC3[surface.orientation] || 'GVL_BTNL_Z',
      });
      entities.push(begrEntity);
      relations.push(makeRelation(rzEntity.NTAEntityDataId, begrEntity.NTAEntityDataId, 'RZ_BEGR'));

      // Opaque construction linked to surface
      if (construction) {
        const constrdEntity = makeEntity(ENT.CONSTRD, {
          [PROP.CONSTRD_NAAM]: construction.name,
          [PROP.CONSTRD_RC]: nlNum(construction.rcValue),
          [PROP.CONSTRD_U]: nlNum(construction.uValue, 3),
        });
        entities.push(constrdEntity);
        relations.push(makeRelation(begrEntity.NTAEntityDataId, constrdEntity.NTAEntityDataId, 'BEGR_CONSTRD'));

        // Library construction ref
        const libEntity = makeEntity(ENT.LIBCONSTRD, {
          [PROP.CONSTRD_NAAM]: construction.name,
          [PROP.CONSTRD_RC]: nlNum(construction.rcValue),
          [PROP.CONSTRD_U]: nlNum(construction.uValue, 3),
        });
        entities.push(libEntity);
        relations.push(makeRelation(constrdEntity.NTAEntityDataId, libEntity.NTAEntityDataId, 'CONSTRD_LIB'));
      }

      // Windows (transparent constructions)
      for (const win of surface.windows) {
        const constrtEntity = makeEntity(ENT.CONSTRT, {
          [PROP.CONSTRT_NAAM]: win.name,
          [PROP.CONSTRT_OPP]: nlNum(win.area),
          [PROP.CONSTRT_UW]: nlNum(win.uValue, 3),
          [PROP.CONSTRT_GWAARDE]: nlNum(win.gValue, 2),
          [PROP.CONSTRT_ORIENT]: ORIENT_TO_UNIEC3[win.orientation] || 'GVL_BTNL_Z',
        });
        entities.push(constrtEntity);
        relations.push(makeRelation(begrEntity.NTAEntityDataId, constrtEntity.NTAEntityDataId, 'BEGR_CONSTRT'));

        // Library ref
        const libTEntity = makeEntity(ENT.LIBCONSTRT, {
          [PROP.CONSTRT_NAAM]: win.name,
          [PROP.CONSTRT_UW]: nlNum(win.uValue, 3),
          [PROP.CONSTRT_GWAARDE]: nlNum(win.gValue, 2),
        });
        entities.push(libTEntity);
        relations.push(makeRelation(constrtEntity.NTAEntityDataId, libTEntity.NTAEntityDataId, 'CONSTRT_LIB'));
      }
    }

    // Thermal bridges
    for (const tb of zone.thermalBridges) {
      const kbEntity = makeEntity(ENT.KOUDEBRUG, {
        [PROP.KB_NAAM]: tb.name,
        [PROP.KB_PSI]: nlNum(tb.psiValue, 3),
        [PROP.KB_LENGTE]: nlNum(tb.length),
      });
      entities.push(kbEntity);
      relations.push(makeRelation(rzEntity.NTAEntityDataId, kbEntity.NTAEntityDataId, 'RZ_KOUDEBRUG'));
    }

    // Air tightness
    const luchtEntity = makeEntity(ENT.LUCHTDH, {
      [PROP.LUCHTDH_QV10]: nlNum(zone.airTightness.qv10, 1),
    });
    entities.push(luchtEntity);
    relations.push(makeRelation(rzEntity.NTAEntityDataId, luchtEntity.NTAEntityDataId, 'RZ_LUCHTDH'));
  }

  // --- Heating systems ---
  if (project.heatingSystems.length > 0) {
    const verwGroup = makeEntity(ENT.VERW, {});
    entities.push(verwGroup);
    relations.push(makeRelation(installEntity.NTAEntityDataId, verwGroup.NTAEntityDataId, 'INST_VERW'));

    for (const hs of project.heatingSystems) {
      const verwOpwek = makeEntity(ENT['VERW-OPWEK'], {
        [PROP.VERW_NAAM]: hs.name,
        [PROP.VERW_TYPE]: HEATING_TO_UNIEC3[hs.type] || 'GVL_VERW_HR107',
        [PROP.VERW_COP]: nlNum(hs.cop),
        [PROP.VERW_DEKKING]: nlNum(hs.coverageFraction, 2),
      });
      entities.push(verwOpwek);
      relations.push(makeRelation(verwGroup.NTAEntityDataId, verwOpwek.NTAEntityDataId, 'VERW_OPWEK'));
    }
  }

  // --- Ventilation systems ---
  for (const vs of project.ventilationSystems) {
    const ventEntity = makeEntity(ENT.VENT, {
      [PROP.VENT_NAAM]: vs.name,
      [PROP.VENT_TYPE]: VENT_TO_UNIEC3[vs.type] || 'GVL_VENT_NAT',
      [PROP.VENT_WTW]: nlNum(vs.heatRecoveryEfficiency, 2),
      [PROP.VENT_SFP]: nlNum(vs.sfp),
    });
    entities.push(ventEntity);
    relations.push(makeRelation(installEntity.NTAEntityDataId, ventEntity.NTAEntityDataId, 'INST_VENT'));
  }

  // --- Cooling systems ---
  if (project.coolingSystems.length > 0) {
    const koelGroup = makeEntity(ENT.KOEL, {});
    entities.push(koelGroup);
    relations.push(makeRelation(installEntity.NTAEntityDataId, koelGroup.NTAEntityDataId, 'INST_KOEL'));

    for (const cs of project.coolingSystems) {
      const koelOpwek = makeEntity(ENT['KOEL-OPWEK'], {
        [PROP.KOEL_NAAM]: cs.name,
        [PROP.KOEL_TYPE]: COOL_TO_UNIEC3[cs.type] || 'GVL_KOEL_GEEN',
        [PROP.KOEL_EER]: nlNum(cs.eer),
      });
      entities.push(koelOpwek);
      relations.push(makeRelation(koelGroup.NTAEntityDataId, koelOpwek.NTAEntityDataId, 'KOEL_OPWEK'));
    }
  }

  // --- Hot water systems ---
  if (project.hotWaterSystems.length > 0) {
    const tapwGroup = makeEntity(ENT.TAPW, {});
    entities.push(tapwGroup);
    relations.push(makeRelation(installEntity.NTAEntityDataId, tapwGroup.NTAEntityDataId, 'INST_TAPW'));

    for (const hw of project.hotWaterSystems) {
      const tapwOpwek = makeEntity(ENT['TAPW-OPWEK'], {
        [PROP.TAPW_NAAM]: hw.name,
        [PROP.TAPW_TYPE]: HW_TO_UNIEC3[hw.type] || 'GVL_TAPW_HRCOMBI',
        [PROP.TAPW_REND]: nlNum(hw.efficiency, 2),
        [PROP.TAPW_ZONTHERM]: hw.hasSolarBoiler ? 'True' : 'False',
        [PROP.TAPW_ZONTHERM_FRAC]: nlNum(hw.solarBoilerFraction, 2),
      });
      entities.push(tapwOpwek);
      relations.push(makeRelation(tapwGroup.NTAEntityDataId, tapwOpwek.NTAEntityDataId, 'TAPW_OPWEK'));
    }
  }

  // --- Solar PV ---
  if (project.solarPV.length > 0) {
    const pvGroup = makeEntity(ENT.PV, {
      [PROP.PV_NAAM]: 'PV-installatie',
    });
    entities.push(pvGroup);
    relations.push(makeRelation(installEntity.NTAEntityDataId, pvGroup.NTAEntityDataId, 'INST_PV'));

    for (const pv of project.solarPV) {
      const pvVeld = makeEntity(ENT['PV-VELD'], {
        [PROP.PV_NAAM]: pv.name,
        [PROP.PV_WP]: nlNum(pv.peakPower, 1),
        [PROP.PV_ORIENT]: ORIENT_TO_UNIEC3[pv.orientation] || 'GVL_BTNL_Z',
        [PROP.PV_HELLING]: nlNum(pv.tilt, 0),
        [PROP.PV_OPP]: nlNum(pv.area),
      });
      entities.push(pvVeld);
      relations.push(makeRelation(pvGroup.NTAEntityDataId, pvVeld.NTAEntityDataId, 'PV_VELD'));
    }
  }

  // --- Solar thermal ---
  for (const st of project.solarThermal) {
    const ztEntity = makeEntity(ENT.ZONTHERM, {
      [PROP.ZT_NAAM]: st.name,
      [PROP.ZT_OPP]: nlNum(st.collectorArea),
      [PROP.ZT_TYPE]: ST_TO_UNIEC3[st.type] || 'GVL_ZT_VLAK',
      [PROP.ZT_ORIENT]: ORIENT_TO_UNIEC3[st.orientation] || 'GVL_BTNL_Z',
      [PROP.ZT_HELLING]: nlNum(st.tilt, 0),
    });
    entities.push(ztEntity);
    relations.push(makeRelation(installEntity.NTAEntityDataId, ztEntity.NTAEntityDataId, 'INST_ZONTHERM'));
  }

  // --- Performance results ---
  const monthly = result as IBENGResultMonthly;
  const prestEntity = makeEntity(ENT.PRESTATIE, {
    [PROP.PREST_BENG1]: nlNum(result.beng1),
    [PROP.PREST_BENG2]: nlNum(result.beng2),
    [PROP.PREST_BENG3]: nlNum(result.beng3, 1),
    ...(monthly.toJuli ? { [PROP.PREST_TOJULI]: nlNum(monthly.toJuli.gto) } : {}),
  });
  entities.push(prestEntity);
  relations.push(makeRelation(gebEntity.NTAEntityDataId, prestEntity.NTAEntityDataId, 'GEB_PRESTATIE'));

  // ============================================================
  // Build ZIP structure
  // ============================================================

  const meta: UNIEC3Meta = {
    VersionId: 311,
    NTAVersion: 'NTA8800 v3.4.0.3',
    ReleaseDate: '2024-07-01T00:00:00',
    ExportDate: now,
    Generator: 'Open Energy Studio',
  };

  const folders = [{ FolderId: 1, Name: 'Default', ParentFolderId: null }];

  const projects: UNIEC3Project[] = [{
    ProjectId: projectId,
    Name: project.name || 'Naamloos',
    Description: project.description || '',
    CreateDate: now,
    ChangeDate: now,
  }];

  const buildings: UNIEC3Building[] = [{
    BuildingId: buildingId,
    ProjectId: projectId,
    NTAVersionId: 311,
    Locked: false,
    Afgemeld: false,
    Afmeldstatus: 0,
    CreateDate: now,
    ChangeDate: now,
  }];

  const summary: UNIEC3Summary = {
    BENG1: result.beng1,
    BENG2: result.beng2,
    BENG3: result.beng3,
    ...(monthly.toJuli ? { TOJuli: monthly.toJuli.gto } : {}),
  };

  // Create JSON files as UTF-8 encoded buffers
  const files: Record<string, Uint8Array> = {
    'meta.json': strToU8(JSON.stringify(meta, null, 2)),
    'folders.json': strToU8(JSON.stringify(folders, null, 2)),
    'projects.json': strToU8(JSON.stringify(projects, null, 2)),
    'buildings.json': strToU8(JSON.stringify(buildings, null, 2)),
    [`buildings/${buildingId}/entities.json`]: strToU8(JSON.stringify(entities)),
    [`buildings/${buildingId}/relations.json`]: strToU8(JSON.stringify(relations)),
    [`buildings/${buildingId}/deltas.json`]: strToU8('[]'),
    [`buildings/${buildingId}/summary.json`]: strToU8(JSON.stringify(summary, null, 2)),
  };

  return zipSync(files, { level: 6 });
}

// ============================================================
// IMPORT: UNIEC3 ZIP → IProject
// ============================================================

export function importFromUNIEC3(zipData: Uint8Array): IProject {
  const files = unzipSync(zipData);

  // Find the building directory
  let buildingId = '';
  for (const path of Object.keys(files)) {
    const match = path.match(/^buildings\/(\d+)\/entities\.json$/);
    if (match) {
      buildingId = match[1];
      break;
    }
  }

  if (!buildingId) {
    throw new Error('Geen gebouw gevonden in UNIEC3 bestand');
  }

  const entitiesData = files[`buildings/${buildingId}/entities.json`];
  const relationsData = files[`buildings/${buildingId}/relations.json`];

  if (!entitiesData || !relationsData) {
    throw new Error('Ongeldig UNIEC3 bestand: entities.json of relations.json ontbreekt');
  }

  const entities: UNIEC3Entity[] = JSON.parse(strFromU8(entitiesData));
  const relations: UNIEC3Relation[] = JSON.parse(strFromU8(relationsData));

  // Build lookup maps
  const entityById = new Map<string, UNIEC3Entity>();
  for (const e of entities) {
    entityById.set(e.NTAEntityDataId, e);
  }

  // Build parent→children map
  const childrenOf = new Map<string, { childId: string; relationId: string }[]>();
  for (const r of relations) {
    const children = childrenOf.get(r.ParentNTAEntityDataId) || [];
    children.push({ childId: r.ChildNTAEntityDataId, relationId: r.NTAEntityRelationId });
    childrenOf.set(r.ParentNTAEntityDataId, children);
  }

  // Find entities by type
  const findByType = (type: string) => entities.filter(e => e.NTAEntityId === type);
  const getChildren = (parentId: string, relId?: string) => {
    const children = childrenOf.get(parentId) || [];
    return (relId ? children.filter(c => c.relationId === relId) : children)
      .map(c => entityById.get(c.childId))
      .filter((e): e is UNIEC3Entity => !!e);
  };

  // Find GEB (building) entity
  const gebEntities = findByType(ENT.GEB);
  const geb = gebEntities[0];
  if (!geb) {
    throw new Error('Geen GEB (gebouw) entiteit gevonden');
  }

  // Extract project info
  const projectName = getEntityProp(geb, PROP.GEB_NAAM) || 'Geïmporteerd project';
  const address = getEntityProp(geb, PROP.GEB_ADRES) || '';
  const city = getEntityProp(geb, PROP.GEB_PLAATS) || '';
  const funcCode = getEntityProp(geb, PROP.GEB_FUNCTIE) || '';
  const description = getEntityProp(geb, PROP.GEB_OMSCHR) || '';
  const buildingFunction = FUNC_FROM_UNIEC3[funcCode] || 'residential';

  // Find UNIT → zones, installations
  const unitEntities = findByType(ENT.UNIT);
  const unit = unitEntities[0] || geb; // fallback to geb if no unit

  // Parse zones
  const zones: IZone[] = [];
  const constructions: IConstruction[] = [];
  const rzEntities = getChildren(unit.NTAEntityDataId, 'UNIT_RZ');
  if (rzEntities.length === 0) {
    // Try direct RZ entities
    rzEntities.push(...findByType(ENT.RZ));
  }

  for (const rz of rzEntities) {
    const zoneName = getEntityProp(rz, PROP.RZ_NAAM) || 'Zone';
    const floorArea = parseNlNum(getEntityProp(rz, PROP.RZ_AG) || '0');
    const volume = parseNlNum(getEntityProp(rz, PROP.RZ_VOLUME) || '0');
    const height = parseNlNum(getEntityProp(rz, PROP.RZ_HOOGTE) || '2,6');

    const surfaces: ISurface[] = [];
    const thermalBridges: IThermalBridge[] = [];
    let qv10 = 0.4;

    // Parse surfaces (BEGR)
    const begrEntities = getChildren(rz.NTAEntityDataId, 'RZ_BEGR');
    for (const begr of begrEntities) {
      const surfaceName = getEntityProp(begr, PROP.BEGR_NAAM) || 'Vlak';
      const surfaceTypeCode = getEntityProp(begr, PROP.BEGR_TYPE) || '';
      const area = parseNlNum(getEntityProp(begr, PROP.BEGR_OPP) || '0');
      const orientCode = getEntityProp(begr, PROP.BEGR_ORIENT) || '';

      const surfaceType = SURFACE_FROM_UNIEC3[surfaceTypeCode] || 'wall';
      const orientation = ORIENT_FROM_UNIEC3[orientCode] || 'S';

      // Find opaque construction
      let constructionId = '';
      const constrdEntities = getChildren(begr.NTAEntityDataId, 'BEGR_CONSTRD');
      if (constrdEntities.length > 0) {
        const constrd = constrdEntities[0];
        const cName = getEntityProp(constrd, PROP.CONSTRD_NAAM) || surfaceName;
        const rc = parseNlNum(getEntityProp(constrd, PROP.CONSTRD_RC) || '0');
        const u = parseNlNum(getEntityProp(constrd, PROP.CONSTRD_U) || '0');

        // Check if construction already exists
        let existing = constructions.find(c => c.name === cName && Math.abs(c.rcValue - rc) < 0.01);
        if (!existing) {
          existing = {
            id: crypto.randomUUID(),
            name: cName,
            layers: [],
            rcValue: rc,
            uValue: u || (rc > 0 ? 1 / (rc + 0.17 + 0.04) : 0),
          };
          constructions.push(existing);
        }
        constructionId = existing.id;
      }

      // Find windows (transparent constructions)
      const windows: IWindow[] = [];
      const constrtEntities = getChildren(begr.NTAEntityDataId, 'BEGR_CONSTRT');
      for (const constrt of constrtEntities) {
        windows.push({
          id: crypto.randomUUID(),
          name: getEntityProp(constrt, PROP.CONSTRT_NAAM) || 'Raam',
          area: parseNlNum(getEntityProp(constrt, PROP.CONSTRT_OPP) || '0'),
          uValue: parseNlNum(getEntityProp(constrt, PROP.CONSTRT_UW) || '1,5'),
          gValue: parseNlNum(getEntityProp(constrt, PROP.CONSTRT_GWAARDE) || '0,5'),
          orientation: ORIENT_FROM_UNIEC3[getEntityProp(constrt, PROP.CONSTRT_ORIENT) || ''] || orientation,
          surfaceId: '', // will be set below
        });
      }

      const surfaceId = crypto.randomUUID();
      for (const w of windows) w.surfaceId = surfaceId;

      surfaces.push({
        id: surfaceId,
        name: surfaceName,
        type: surfaceType,
        area,
        orientation,
        constructionId,
        zoneId: '', // will be set below
        windows,
      });
    }

    // Parse thermal bridges
    const kbEntities = getChildren(rz.NTAEntityDataId, 'RZ_KOUDEBRUG');
    for (const kb of kbEntities) {
      thermalBridges.push({
        id: crypto.randomUUID(),
        name: getEntityProp(kb, PROP.KB_NAAM) || 'Koudebrug',
        psiValue: parseNlNum(getEntityProp(kb, PROP.KB_PSI) || '0'),
        length: parseNlNum(getEntityProp(kb, PROP.KB_LENGTE) || '0'),
        zoneId: '', // will be set below
      });
    }

    // Parse air tightness
    const luchtEntities = getChildren(rz.NTAEntityDataId, 'RZ_LUCHTDH');
    if (luchtEntities.length > 0) {
      qv10 = parseNlNum(getEntityProp(luchtEntities[0], PROP.LUCHTDH_QV10) || '0,4');
    }

    const zoneId = crypto.randomUUID();
    for (const s of surfaces) s.zoneId = zoneId;
    for (const tb of thermalBridges) tb.zoneId = zoneId;

    zones.push({
      id: zoneId,
      name: zoneName,
      floorArea,
      volume,
      height,
      surfaces,
      thermalBridges,
      airTightness: { qv10 },
    });
  }

  // Parse installations from INSTALLATIE group
  const heatingSystems: IHeatingSystem[] = [];
  const ventilationSystems: IVentilationSystem[] = [];
  const coolingSystems: ICoolingSystem[] = [];
  const hotWaterSystems: IHotWaterSystem[] = [];
  const solarPV: ISolarPV[] = [];
  const solarThermal: ISolarThermal[] = [];

  const installEntities = getChildren(unit.NTAEntityDataId, 'UNIT_INSTALLATIE');
  const installParent = installEntities[0] || unit;

  // Heating
  const verwEntities = getChildren(installParent.NTAEntityDataId, 'INST_VERW');
  for (const verw of verwEntities) {
    const opwekEntities = getChildren(verw.NTAEntityDataId, 'VERW_OPWEK');
    for (const opwek of opwekEntities) {
      const typeCode = getEntityProp(opwek, PROP.VERW_TYPE) || '';
      heatingSystems.push({
        id: crypto.randomUUID(),
        name: getEntityProp(opwek, PROP.VERW_NAAM) || 'Verwarming',
        type: HEATING_FROM_UNIEC3[typeCode] || 'hr107',
        cop: parseNlNum(getEntityProp(opwek, PROP.VERW_COP) || '1'),
        coverageFraction: parseNlNum(getEntityProp(opwek, PROP.VERW_DEKKING) || '1'),
      });
    }
  }

  // Also check for VERW-OPWEK directly under install
  if (heatingSystems.length === 0) {
    for (const opwek of findByType(ENT['VERW-OPWEK'])) {
      const typeCode = getEntityProp(opwek, PROP.VERW_TYPE) || '';
      heatingSystems.push({
        id: crypto.randomUUID(),
        name: getEntityProp(opwek, PROP.VERW_NAAM) || 'Verwarming',
        type: HEATING_FROM_UNIEC3[typeCode] || 'hr107',
        cop: parseNlNum(getEntityProp(opwek, PROP.VERW_COP) || '1'),
        coverageFraction: parseNlNum(getEntityProp(opwek, PROP.VERW_DEKKING) || '1'),
      });
    }
  }

  // Ventilation
  const ventEntities = getChildren(installParent.NTAEntityDataId, 'INST_VENT');
  for (const vent of ventEntities) {
    const typeCode = getEntityProp(vent, PROP.VENT_TYPE) || '';
    ventilationSystems.push({
      id: crypto.randomUUID(),
      name: getEntityProp(vent, PROP.VENT_NAAM) || 'Ventilatie',
      type: VENT_FROM_UNIEC3[typeCode] || 'natural',
      heatRecoveryEfficiency: parseNlNum(getEntityProp(vent, PROP.VENT_WTW) || '0'),
      sfp: parseNlNum(getEntityProp(vent, PROP.VENT_SFP) || '0'),
    });
  }

  // Also check VENT directly
  if (ventilationSystems.length === 0) {
    for (const vent of findByType(ENT.VENT)) {
      const typeCode = getEntityProp(vent, PROP.VENT_TYPE) || '';
      ventilationSystems.push({
        id: crypto.randomUUID(),
        name: getEntityProp(vent, PROP.VENT_NAAM) || 'Ventilatie',
        type: VENT_FROM_UNIEC3[typeCode] || 'natural',
        heatRecoveryEfficiency: parseNlNum(getEntityProp(vent, PROP.VENT_WTW) || '0'),
        sfp: parseNlNum(getEntityProp(vent, PROP.VENT_SFP) || '0'),
      });
    }
  }

  // Cooling
  const koelEntities = getChildren(installParent.NTAEntityDataId, 'INST_KOEL');
  for (const koel of koelEntities) {
    const opwekEntities = getChildren(koel.NTAEntityDataId, 'KOEL_OPWEK');
    for (const opwek of opwekEntities) {
      const typeCode = getEntityProp(opwek, PROP.KOEL_TYPE) || '';
      coolingSystems.push({
        id: crypto.randomUUID(),
        name: getEntityProp(opwek, PROP.KOEL_NAAM) || 'Koeling',
        type: COOL_FROM_UNIEC3[typeCode] || 'none',
        eer: parseNlNum(getEntityProp(opwek, PROP.KOEL_EER) || '3'),
      });
    }
  }

  // Hot water
  const tapwEntities = getChildren(installParent.NTAEntityDataId, 'INST_TAPW');
  for (const tapw of tapwEntities) {
    const opwekEntities = getChildren(tapw.NTAEntityDataId, 'TAPW_OPWEK');
    for (const opwek of opwekEntities) {
      const typeCode = getEntityProp(opwek, PROP.TAPW_TYPE) || '';
      hotWaterSystems.push({
        id: crypto.randomUUID(),
        name: getEntityProp(opwek, PROP.TAPW_NAAM) || 'Tapwater',
        type: HW_FROM_UNIEC3[typeCode] || 'hr_combi',
        efficiency: parseNlNum(getEntityProp(opwek, PROP.TAPW_REND) || '1'),
        hasSolarBoiler: getEntityProp(opwek, PROP.TAPW_ZONTHERM) === 'True',
        solarBoilerFraction: parseNlNum(getEntityProp(opwek, PROP.TAPW_ZONTHERM_FRAC) || '0'),
      });
    }
  }

  // Solar PV
  const pvGroupEntities = getChildren(installParent.NTAEntityDataId, 'INST_PV');
  for (const pvGroup of pvGroupEntities) {
    const pvVeldEntities = getChildren(pvGroup.NTAEntityDataId, 'PV_VELD');
    for (const pvVeld of pvVeldEntities) {
      solarPV.push({
        id: crypto.randomUUID(),
        name: getEntityProp(pvVeld, PROP.PV_NAAM) || 'PV-veld',
        peakPower: parseNlNum(getEntityProp(pvVeld, PROP.PV_WP) || '0'),
        orientation: ORIENT_FROM_UNIEC3[getEntityProp(pvVeld, PROP.PV_ORIENT) || ''] || 'S',
        tilt: parseNlNum(getEntityProp(pvVeld, PROP.PV_HELLING) || '30'),
        area: parseNlNum(getEntityProp(pvVeld, PROP.PV_OPP) || '0'),
      });
    }
  }

  // Also check PV-VELD directly
  if (solarPV.length === 0) {
    for (const pvVeld of findByType(ENT['PV-VELD'])) {
      solarPV.push({
        id: crypto.randomUUID(),
        name: getEntityProp(pvVeld, PROP.PV_NAAM) || 'PV-veld',
        peakPower: parseNlNum(getEntityProp(pvVeld, PROP.PV_WP) || '0'),
        orientation: ORIENT_FROM_UNIEC3[getEntityProp(pvVeld, PROP.PV_ORIENT) || ''] || 'S',
        tilt: parseNlNum(getEntityProp(pvVeld, PROP.PV_HELLING) || '30'),
        area: parseNlNum(getEntityProp(pvVeld, PROP.PV_OPP) || '0'),
      });
    }
  }

  // Solar thermal
  const ztEntities = getChildren(installParent.NTAEntityDataId, 'INST_ZONTHERM');
  for (const zt of ztEntities) {
    solarThermal.push({
      id: crypto.randomUUID(),
      name: getEntityProp(zt, PROP.ZT_NAAM) || 'Zonnecollector',
      collectorArea: parseNlNum(getEntityProp(zt, PROP.ZT_OPP) || '0'),
      type: ST_FROM_UNIEC3[getEntityProp(zt, PROP.ZT_TYPE) || ''] || 'flat_plate',
      orientation: ORIENT_FROM_UNIEC3[getEntityProp(zt, PROP.ZT_ORIENT) || ''] || 'S',
      tilt: parseNlNum(getEntityProp(zt, PROP.ZT_HELLING) || '45'),
    });
  }

  return {
    id: crypto.randomUUID(),
    name: projectName,
    description,
    buildingFunction,
    address,
    city,
    zones,
    heatingSystems,
    ventilationSystems,
    coolingSystems,
    hotWaterSystems,
    solarPV,
    solarThermal,
    constructions,
  };
}

// ============================================================
// Download helper (export)
// ============================================================

export function downloadUNIEC3(project: IProject, result: IBENGResult): void {
  const zipData = exportToUNIEC3(project, result);
  const blob = new Blob([zipData.buffer as ArrayBuffer], { type: 'application/zip' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${project.name || 'project'}.uniec3`;
  a.click();
  URL.revokeObjectURL(url);
}

// ============================================================
// File dialog helper (import)
// ============================================================

export function openUNIEC3FileDialog(): Promise<IProject> {
  return new Promise((resolve, reject) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.uniec3';
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return reject(new Error('Geen bestand geselecteerd'));
      const reader = new FileReader();
      reader.onload = () => {
        try {
          const data = new Uint8Array(reader.result as ArrayBuffer);
          const project = importFromUNIEC3(data);
          resolve(project);
        } catch (err) {
          reject(err);
        }
      };
      reader.onerror = () => reject(new Error('Bestand kon niet gelezen worden'));
      reader.readAsArrayBuffer(file);
    };
    input.click();
  });
}
