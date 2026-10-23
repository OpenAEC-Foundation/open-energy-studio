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
 *     summary.json      - Empty until a verified calculation is available
 */

import { zipSync, strToU8, strFromU8 } from 'fflate';
import { safeUnzip } from './safeUnzip';
import { importUniec3Export, isUniec3Export } from './UNIEC3Import';
import {
  ImportFormatError, ImportIssues, droppedOnExport, lookup, ntaInputNotInFormat, sourceVersionNotCarried, unreadData, unusableRatio,
  withoutBom, type ImportResult, type ReadResult,
} from './importIssues';
import { downloadBytes, importFile, pickFile } from './importFile';
import { ASSUMED_QV10 } from '../energy/normalizeProject';
import { SURFACE_RESISTANCE } from '../energy/Constants';
import type {
  IProject,
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

} as const;

// ============================================================
// Value mapping helpers
// ============================================================

/**
 * A number as Uniec 3 writes it (decimal comma), at full precision: a
 * rounded qv10, kWp, tilt or Rc would change the calculation input of a
 * round trip.
 */
function nlNum(n: number): string {
  return Number.isFinite(n) ? String(n).replace('.', ',') : '';
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
// EXPORT: IProject input only → UNIEC3 ZIP
// ============================================================

export function exportToUNIEC3(project: IProject): Uint8Array {
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
    [PROP.GEB_FUNCTIE]: lookup(FUNC_TO_UNIEC3, project.buildingFunction) ?? 'GVL_FUNC_WOON',
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
        [PROP.BEGR_TYPE]: lookup(SURFACE_TO_UNIEC3, surface.type) ?? 'GVL_BEGR_GEVEL',
        [PROP.BEGR_OPP]: nlNum(surface.area),
        [PROP.BEGR_ORIENT]: lookup(ORIENT_TO_UNIEC3, surface.orientation) ?? 'GVL_BTNL_Z',
      });
      entities.push(begrEntity);
      relations.push(makeRelation(rzEntity.NTAEntityDataId, begrEntity.NTAEntityDataId, 'RZ_BEGR'));

      // Opaque construction linked to surface
      if (construction) {
        const constrdEntity = makeEntity(ENT.CONSTRD, {
          [PROP.CONSTRD_NAAM]: construction.name,
          [PROP.CONSTRD_RC]: nlNum(construction.rcValue),
          [PROP.CONSTRD_U]: nlNum(construction.uValue),
        });
        entities.push(constrdEntity);
        relations.push(makeRelation(begrEntity.NTAEntityDataId, constrdEntity.NTAEntityDataId, 'BEGR_CONSTRD'));

        // Library construction ref
        const libEntity = makeEntity(ENT.LIBCONSTRD, {
          [PROP.CONSTRD_NAAM]: construction.name,
          [PROP.CONSTRD_RC]: nlNum(construction.rcValue),
          [PROP.CONSTRD_U]: nlNum(construction.uValue),
        });
        entities.push(libEntity);
        relations.push(makeRelation(constrdEntity.NTAEntityDataId, libEntity.NTAEntityDataId, 'CONSTRD_LIB'));
      }

      // Windows (transparent constructions)
      for (const win of surface.windows) {
        const constrtEntity = makeEntity(ENT.CONSTRT, {
          [PROP.CONSTRT_NAAM]: win.name,
          [PROP.CONSTRT_OPP]: nlNum(win.area),
          [PROP.CONSTRT_UW]: nlNum(win.uValue),
          [PROP.CONSTRT_GWAARDE]: nlNum(win.gValue),
          [PROP.CONSTRT_ORIENT]: lookup(ORIENT_TO_UNIEC3, win.orientation) ?? 'GVL_BTNL_Z',
        });
        entities.push(constrtEntity);
        relations.push(makeRelation(begrEntity.NTAEntityDataId, constrtEntity.NTAEntityDataId, 'BEGR_CONSTRT'));

        // Library ref
        const libTEntity = makeEntity(ENT.LIBCONSTRT, {
          [PROP.CONSTRT_NAAM]: win.name,
          [PROP.CONSTRT_UW]: nlNum(win.uValue),
          [PROP.CONSTRT_GWAARDE]: nlNum(win.gValue),
        });
        entities.push(libTEntity);
        relations.push(makeRelation(constrtEntity.NTAEntityDataId, libTEntity.NTAEntityDataId, 'CONSTRT_LIB'));
      }
    }

    // Thermal bridges
    for (const tb of zone.thermalBridges) {
      const kbEntity = makeEntity(ENT.KOUDEBRUG, {
        [PROP.KB_NAAM]: tb.name,
        [PROP.KB_PSI]: nlNum(tb.psiValue),
        [PROP.KB_LENGTE]: nlNum(tb.length),
      });
      entities.push(kbEntity);
      relations.push(makeRelation(rzEntity.NTAEntityDataId, kbEntity.NTAEntityDataId, 'RZ_KOUDEBRUG'));
    }

    // Air tightness
    const luchtEntity = makeEntity(ENT.LUCHTDH, {
      [PROP.LUCHTDH_QV10]: nlNum(zone.airTightness.qv10),
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
        [PROP.VERW_TYPE]: lookup(HEATING_TO_UNIEC3, hs.type) ?? 'GVL_VERW_HR107',
        ...(unusableRatio(hs.cop) ? {} : { [PROP.VERW_COP]: nlNum(hs.cop) }),
        [PROP.VERW_DEKKING]: nlNum(hs.coverageFraction),
      });
      entities.push(verwOpwek);
      relations.push(makeRelation(verwGroup.NTAEntityDataId, verwOpwek.NTAEntityDataId, 'VERW_OPWEK'));
    }
  }

  // --- Ventilation systems ---
  for (const vs of project.ventilationSystems) {
    const ventEntity = makeEntity(ENT.VENT, {
      [PROP.VENT_NAAM]: vs.name,
      [PROP.VENT_TYPE]: lookup(VENT_TO_UNIEC3, vs.type) ?? 'GVL_VENT_NAT',
      [PROP.VENT_WTW]: nlNum(vs.heatRecoveryEfficiency),
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
        [PROP.KOEL_TYPE]: lookup(COOL_TO_UNIEC3, cs.type) ?? 'GVL_KOEL_GEEN',
        ...(unusableRatio(cs.eer, cs.type === 'none') ? {} : { [PROP.KOEL_EER]: nlNum(cs.eer) }),
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
        [PROP.TAPW_TYPE]: lookup(HW_TO_UNIEC3, hw.type) ?? 'GVL_TAPW_HRCOMBI',
        ...(unusableRatio(hw.efficiency) ? {} : { [PROP.TAPW_REND]: nlNum(hw.efficiency) }),
        [PROP.TAPW_ZONTHERM]: hw.hasSolarBoiler ? 'True' : 'False',
        [PROP.TAPW_ZONTHERM_FRAC]: nlNum(hw.solarBoilerFraction),
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
        [PROP.PV_WP]: nlNum(pv.peakPower),
        [PROP.PV_ORIENT]: lookup(ORIENT_TO_UNIEC3, pv.orientation) ?? 'GVL_BTNL_Z',
        [PROP.PV_HELLING]: nlNum(pv.tilt),
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
      [PROP.ZT_TYPE]: lookup(ST_TO_UNIEC3, st.type) ?? 'GVL_ZT_VLAK',
      [PROP.ZT_ORIENT]: lookup(ORIENT_TO_UNIEC3, st.orientation) ?? 'GVL_BTNL_Z',
      [PROP.ZT_HELLING]: nlNum(st.tilt),
    });
    entities.push(ztEntity);
    relations.push(makeRelation(installEntity.NTAEntityDataId, ztEntity.NTAEntityDataId, 'INST_ZONTHERM'));
  }

  // ============================================================
  // Build ZIP structure
  // ============================================================

  const meta: UNIEC3Meta = {
    VersionId: 311,
    NTAVersion: 'NTA8800 v3.4.0.3',
    ReleaseDate: '2024-07-01T00:00:00',
    ExportDate: now,
    Generator: 'Open Energy Studio input draft - unverified, no calculated results',
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

  // Create JSON files as UTF-8 encoded buffers
  const files: Record<string, Uint8Array> = {
    'meta.json': strToU8(JSON.stringify(meta, null, 2)),
    'folders.json': strToU8(JSON.stringify(folders, null, 2)),
    'projects.json': strToU8(JSON.stringify(projects, null, 2)),
    'buildings.json': strToU8(JSON.stringify(buildings, null, 2)),
    [`buildings/${buildingId}/entities.json`]: strToU8(JSON.stringify(entities)),
    [`buildings/${buildingId}/relations.json`]: strToU8(JSON.stringify(relations)),
    [`buildings/${buildingId}/deltas.json`]: strToU8('[]'),
    [`buildings/${buildingId}/summary.json`]: strToU8('{}'),
  };

  // fflate's TextEncoder may return a typed array from another JS realm.
  // Copy into this realm so zipSync treats each value as a file, not a directory.
  const zipFiles = Object.fromEntries(
    Object.entries(files).map(([path, data]) => [path, new Uint8Array(data)]),
  );
  return zipSync(zipFiles, { level: 6 });
}

// ============================================================
// IMPORT: UNIEC3 ZIP → IProject
// ============================================================

function parseJsonList(bytes: Uint8Array, file: string): unknown[] {
  let data: unknown;
  try {
    data = JSON.parse(withoutBom(strFromU8(bytes)));
  } catch {
    throw new ImportFormatError('uniec3_json_invalid', `${file} is geen geldige JSON.`, { file });
  }
  if (!Array.isArray(data)) {
    throw new ImportFormatError('uniec3_shape_invalid', `${file} moet een lijst zijn.`, { file });
  }
  return data;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value != null && typeof value === 'object' && !Array.isArray(value);
}

/** Checks the shape of entities.json; a missing property list counts as empty. */
function readEntities(list: unknown[], file: string): UNIEC3Entity[] {
  return list.map((item, index) => {
    if (!isRecord(item) || typeof item.NTAEntityDataId !== 'string' || typeof item.NTAEntityId !== 'string') {
      throw new ImportFormatError('uniec3_shape_invalid',
        `${file}: element ${index} heeft geen geldige NTAEntityDataId en NTAEntityId.`, { file });
    }
    const raw = item.NTAPropertyDatas ?? [];
    if (!Array.isArray(raw)) {
      throw new ImportFormatError('uniec3_shape_invalid',
        `${file}: NTAPropertyDatas van element ${index} moet een lijst zijn.`, { file });
    }
    const properties: UNIEC3Property[] = [];
    for (const property of raw) {
      if (!isRecord(property) || typeof property.NTAPropertyId !== 'string') continue;
      const value = property.Value;
      if (typeof value === 'string') properties.push({ NTAPropertyId: property.NTAPropertyId, Value: value });
      else if (typeof value === 'number' && Number.isFinite(value)) {
        properties.push({ NTAPropertyId: property.NTAPropertyId, Value: String(value) });
      }
    }
    return { NTAEntityDataId: item.NTAEntityDataId, NTAEntityId: item.NTAEntityId, NTAPropertyDatas: properties };
  });
}

function readRelations(list: unknown[], file: string): UNIEC3Relation[] {
  return list.map((item, index) => {
    if (!isRecord(item) || typeof item.NTAEntityRelationId !== 'string'
      || typeof item.ParentNTAEntityDataId !== 'string' || typeof item.ChildNTAEntityDataId !== 'string') {
      throw new ImportFormatError('uniec3_shape_invalid',
        `${file}: relatie ${index} heeft geen geldige ouder, kind en relatiesoort.`, { file });
    }
    return {
      NTAEntityDataId: typeof item.NTAEntityDataId === 'string' ? item.NTAEntityDataId : '',
      NTAEntityRelationId: item.NTAEntityRelationId,
      ParentNTAEntityDataId: item.ParentNTAEntityDataId,
      ChildNTAEntityDataId: item.ChildNTAEntityDataId,
    };
  });
}

/** A Uniec 3 code through its table; a missing or unknown code gives the fallback and an issue. */
function codeOf<T>(
  issues: ImportIssues, path: string, code: string | undefined, table: Record<string, T>, fallback: T, what: string,
): T {
  if (!code) {
    issues.add('defaulted', path, `${what} ontbreekt; ${String(fallback)} aangenomen.`, String(fallback));
    return fallback;
  }
  const value = lookup(table, code);
  if (value === undefined) {
    issues.add('invalid', path, `Onbekende code "${code.slice(0, 40)}" voor ${what.toLowerCase()}; ${String(fallback)} aangenomen.`, code.slice(0, 40));
    return fallback;
  }
  return value;
}

/** Property ids this importer reads; any other property of an entity it reads is reported. */
const READ_PROPERTIES = new Set<string>(Object.values(PROP));

/**
 * Entities the importer passes over on purpose: our own export writes a
 * library copy of every construction (the values are read from the
 * construction itself) and an empty settings entity.
 */
const LIBRARY_ENTITIES = new Set<string>([ENT.LIBCONSTRD, ENT.LIBCONSTRT]);

/**
 * The program and NTA 8800 version meta.json names ("NTA8800 v3.4.0.3,
 * VersionId 311, …"); undefined when the file has no readable meta.json.
 */
function metaVersion(bytes: Uint8Array | undefined): string | undefined {
  if (!bytes) return undefined;
  let meta: unknown;
  try {
    meta = JSON.parse(withoutBom(strFromU8(bytes)));
  } catch {
    return undefined;
  }
  if (!isRecord(meta)) return undefined;
  const parts = [
    typeof meta.NTAVersion === 'string' ? meta.NTAVersion.trim() : '',
    typeof meta.VersionId === 'number' || typeof meta.VersionId === 'string' ? `VersionId ${String(meta.VersionId).trim()}` : '',
    typeof meta.Generator === 'string' ? meta.Generator.trim() : '',
  ].filter((part) => part !== '').map((part) => part.slice(0, 80));
  return parts.length > 0 ? parts.join(', ') : undefined;
}

/** Imports a Uniec 3 archive, with the list of what the program filled in or left out. */
export function readUNIEC3(zipData: Uint8Array): ReadResult {
  // Only the building entities and relations and meta.json are read, with a
  // size cap, so a crafted archive cannot unpack gigabytes into memory.
  const files = safeUnzip(zipData, (name) => name === 'meta.json' || /^buildings\/\d+\/(entities|relations)\.json$/.test(name));
  // A ZIP written by Uniec3 itself (relations with ParentId/ChildId, library constructions): the
  // real-format importer of UNIEC3Import.ts; the rest is the app's own input-draft format.
  if (isUniec3Export(files)) {
    const issues = new ImportIssues();
    return { project: importUniec3Export(files, issues), issues, sourceVersion: metaVersion(files['meta.json']) };
  }
  const issues = new ImportIssues();
  const sourceVersion = metaVersion(files['meta.json']);

  const buildingIds = [...new Set(Object.keys(files)
    .map((path) => path.match(/^buildings\/(\d+)\/entities\.json$/)?.[1])
    .filter((id): id is string => !!id))]
    .sort((a, b) => Number(a) - Number(b));
  if (buildingIds.length === 0) {
    throw new ImportFormatError('uniec3_no_building', 'Geen gebouw gevonden in het UNIEC3-bestand.');
  }
  const buildingId = buildingIds[0];
  if (buildingIds.length > 1) {
    issues.add('multiple', 'buildings',
      `Het bestand bevat ${buildingIds.length} gebouwen (${buildingIds.join(', ')}); alleen gebouw ${buildingId} is ingelezen.`,
      buildingIds.length);
  }

  const entitiesFile = `buildings/${buildingId}/entities.json`;
  const relationsFile = `buildings/${buildingId}/relations.json`;
  const entitiesData = files[entitiesFile];
  const relationsData = files[relationsFile];
  if (!entitiesData || !relationsData) {
    throw new ImportFormatError('uniec3_files_missing', 'Ongeldig UNIEC3-bestand: entities.json of relations.json ontbreekt.');
  }
  const entities = readEntities(parseJsonList(entitiesData, entitiesFile), entitiesFile);
  const relations = readRelations(parseJsonList(relationsData, relationsFile), relationsFile);

  // Lookup by id; a repeated id keeps its first entity.
  const entityById = new Map<string, UNIEC3Entity>();
  let duplicates = 0;
  for (const entity of entities) {
    if (entityById.has(entity.NTAEntityDataId)) {
      duplicates += 1;
      continue;
    }
    entityById.set(entity.NTAEntityDataId, entity);
  }
  if (duplicates > 0) {
    issues.add('duplicate', 'entities',
      `${duplicates} entiteit(en) met een id dat al voorkwam; telkens de eerste is ingelezen.`, duplicates);
  }

  // Parent → children; relations to ids that are not in the file are reported.
  const childrenOf = new Map<string, { childId: string; relationId: string }[]>();
  let unknownReferences = 0;
  for (const relation of relations) {
    if (!entityById.has(relation.ParentNTAEntityDataId) || !entityById.has(relation.ChildNTAEntityDataId)) {
      unknownReferences += 1;
      continue;
    }
    const children = childrenOf.get(relation.ParentNTAEntityDataId) || [];
    children.push({ childId: relation.ChildNTAEntityDataId, relationId: relation.NTAEntityRelationId });
    childrenOf.set(relation.ParentNTAEntityDataId, children);
  }
  if (unknownReferences > 0) {
    issues.add('unknown_reference', 'relations',
      `${unknownReferences} relatie(s) verwijzen naar een entiteit die niet in het bestand staat en zijn overgeslagen.`,
      unknownReferences);
  }

  const unique = [...entityById.values()];
  // Every entity the importer reads is marked, so what it passes over can be reported.
  const read = new Set<UNIEC3Entity>();
  const mark = (entities: UNIEC3Entity[]) => {
    for (const entity of entities) read.add(entity);
    return entities;
  };
  const findByType = (type: string) => mark(unique.filter(e => e.NTAEntityId === type));
  const childrenBy = (parentId: string, relId: string) => (childrenOf.get(parentId) || [])
    .filter(c => c.relationId === relId)
    .map(c => entityById.get(c.childId))
    .filter((e): e is UNIEC3Entity => !!e);
  const getChildren = (parentId: string, relId: string) => mark(childrenBy(parentId, relId));
  /** The one child the importer reads; any further ones stay unread and are reported. */
  const firstChild = (parentId: string, relId: string): UNIEC3Entity | undefined => mark(childrenBy(parentId, relId).slice(0, 1))[0];

  // Building (GEB); further buildings and units are reported as 'multiple', not as unread.
  const gebEntities = findByType(ENT.GEB);
  const geb = gebEntities[0];
  if (!geb) {
    throw new ImportFormatError('uniec3_no_geb', 'Geen GEB-entiteit (gebouw) gevonden.');
  }
  if (gebEntities.length > 1) {
    issues.add('multiple', 'GEB',
      `Het bestand bevat ${gebEntities.length} gebouwentiteiten; alleen de eerste is ingelezen.`, gebEntities.length);
  }

  const projectName = getEntityProp(geb, PROP.GEB_NAAM) || 'Geïmporteerd project';
  const address = getEntityProp(geb, PROP.GEB_ADRES) || '';
  const city = getEntityProp(geb, PROP.GEB_PLAATS) || '';
  const description = getEntityProp(geb, PROP.GEB_OMSCHR) || '';
  const buildingFunction = codeOf<BuildingFunction>(issues, 'buildingFunction', getEntityProp(geb, PROP.GEB_FUNCTIE),
    FUNC_FROM_UNIEC3, 'residential', 'Gebruiksfunctie');

  // Unit → zones and installations
  const unitEntities = findByType(ENT.UNIT);
  if (unitEntities.length > 1) {
    issues.add('multiple', 'UNIT',
      `Het bestand bevat ${unitEntities.length} units; alleen de eerste is ingelezen.`, unitEntities.length);
  }
  const unit = unitEntities[0] || geb; // fallback to geb if no unit

  const zones: IZone[] = [];
  const constructions: IConstruction[] = [];
  const rzEntities = getChildren(unit.NTAEntityDataId, 'UNIT_RZ');
  if (rzEntities.length === 0) rzEntities.push(...findByType(ENT.RZ));

  rzEntities.forEach((rz, zoneIndex) => {
    const zp = `zones[${zoneIndex}]`;
    const zoneName = getEntityProp(rz, PROP.RZ_NAAM) || 'Zone';
    const floorArea = issues.number(`${zp}.floorArea`, getEntityProp(rz, PROP.RZ_AG), { fallback: 0, min: 0 });
    const volume = issues.number(`${zp}.volume`, getEntityProp(rz, PROP.RZ_VOLUME), { fallback: 0, min: 0 });
    const height = issues.number(`${zp}.height`, getEntityProp(rz, PROP.RZ_HOOGTE), { fallback: 2.6, min: 0 });

    const surfaces: ISurface[] = [];
    const thermalBridges: IThermalBridge[] = [];

    getChildren(rz.NTAEntityDataId, 'RZ_BEGR').forEach((begr, surfaceIndex) => {
      const sp = `${zp}.surfaces[${surfaceIndex}]`;
      const surfaceName = getEntityProp(begr, PROP.BEGR_NAAM) || 'Vlak';
      const area = issues.number(`${sp}.area`, getEntityProp(begr, PROP.BEGR_OPP), { fallback: 0, min: 0 });
      const surfaceType = codeOf<SurfaceType>(issues, `${sp}.type`, getEntityProp(begr, PROP.BEGR_TYPE),
        SURFACE_FROM_UNIEC3, 'wall', 'Vlaksoort');
      const orientation = codeOf<Orientation>(issues, `${sp}.orientation`, getEntityProp(begr, PROP.BEGR_ORIENT),
        ORIENT_FROM_UNIEC3, 'S', 'Oriëntatie');

      // Opaque construction: the U from the file, else from Rc with the R_si of the element type
      // (NTA 8800 table C.2); never a U of 0.
      let constructionId = '';
      const constrd = firstChild(begr.NTAEntityDataId, 'BEGR_CONSTRD');
      if (constrd) {
        const cName = getEntityProp(constrd, PROP.CONSTRD_NAAM) || surfaceName;
        const rc = issues.number(`${sp}.construction.rcValue`, getEntityProp(constrd, PROP.CONSTRD_RC),
          { fallback: 0, min: 0, silentWhenMissing: true });
        let u = issues.number(`${sp}.construction.uValue`, getEntityProp(constrd, PROP.CONSTRD_U),
          { fallback: 0, min: 0, minExclusive: true, silentWhenMissing: true });
        if (!(u > 0) && rc > 0) {
          const { rsi, rse } = SURFACE_RESISTANCE[surfaceType];
          u = 1 / (rc + rsi + rse);
          issues.add('defaulted', `${sp}.construction.uValue`,
            `U ontbreekt; berekend uit Rc ${rc} met R_si ${rsi} en R_se ${rse}.`, u);
        }
        if (u > 0) {
          let existing = constructions.find(c => c.name === cName && Math.abs(c.rcValue - rc) < 0.01
            && Math.abs(c.uValue - u) < 1e-9);
          if (!existing) {
            existing = { id: crypto.randomUUID(), name: cName, layers: [], rcValue: rc, uValue: u };
            constructions.push(existing);
          }
          constructionId = existing.id;
        } else {
          issues.add('dropped', `${sp}.constructionId`,
            `Constructie "${cName}" heeft geen Rc en geen U; niet gekoppeld, vul de constructie aan.`);
        }
      }

      const windows: IWindow[] = getChildren(begr.NTAEntityDataId, 'BEGR_CONSTRT').map((constrt, windowIndex) => {
        const wp = `${sp}.windows[${windowIndex}]`;
        const orientCode = getEntityProp(constrt, PROP.CONSTRT_ORIENT);
        return {
          id: crypto.randomUUID(),
          name: getEntityProp(constrt, PROP.CONSTRT_NAAM) || 'Raam',
          area: issues.number(`${wp}.area`, getEntityProp(constrt, PROP.CONSTRT_OPP), { fallback: 0, min: 0 }),
          uValue: issues.number(`${wp}.uValue`, getEntityProp(constrt, PROP.CONSTRT_UW),
            { fallback: 1.5, min: 0, minExclusive: true }),
          gValue: issues.number(`${wp}.gValue`, getEntityProp(constrt, PROP.CONSTRT_GWAARDE),
            { fallback: 0.5, min: 0, max: 1, clamp: true }),
          // A window without its own orientation faces the way its surface does.
          orientation: orientCode
            ? codeOf<Orientation>(issues, `${wp}.orientation`, orientCode, ORIENT_FROM_UNIEC3, orientation, 'Oriëntatie')
            : orientation,
          surfaceId: '', // set below
        };
      });

      const surfaceId = crypto.randomUUID();
      for (const w of windows) w.surfaceId = surfaceId;
      surfaces.push({ id: surfaceId, name: surfaceName, type: surfaceType, area, orientation, constructionId, zoneId: '', windows });
      // A UNIEC3 begrenzing has no tilt here: a pitched roof needs it in the NTA input, as on a VABI import.
      if (surfaceType === 'roof' && orientation !== 'horizontal') {
        issues.add('dropped', `${sp}.tilt`, `Hellingshoek van dak "${surfaceName}" ontbreekt; vul die in bij de NTA-invoer.`);
      }
    });

    getChildren(rz.NTAEntityDataId, 'RZ_KOUDEBRUG').forEach((kb, bridgeIndex) => {
      const bp = `${zp}.thermalBridges[${bridgeIndex}]`;
      thermalBridges.push({
        id: crypto.randomUUID(),
        name: getEntityProp(kb, PROP.KB_NAAM) || 'Koudebrug',
        // ψ may be negative (NTA 8800 §8.2.3); only its absence is reported.
        psiValue: issues.number(`${bp}.psiValue`, getEntityProp(kb, PROP.KB_PSI), { fallback: 0 }),
        length: issues.number(`${bp}.length`, getEntityProp(kb, PROP.KB_LENGTE), { fallback: 0, min: 0 }),
        zoneId: '',
      });
    });

    // Air tightness: a missing, zero or unreadable qv10 gets the importer default, marked assumed
    // (as normalizeProject does), so the envelope view asks the adviser to check it.
    const lucht = firstChild(rz.NTAEntityDataId, 'RZ_LUCHTDH');
    const before = issues.list.length;
    const qv10 = issues.number(`${zp}.airTightness.qv10`, lucht ? getEntityProp(lucht, PROP.LUCHTDH_QV10) : undefined, {
      fallback: ASSUMED_QV10, min: 0, minExclusive: true, fallbackReason: 'aangenomen waarde, te controleren',
    });
    const assumed = issues.list.length > before;

    const zoneId = crypto.randomUUID();
    for (const surface of surfaces) surface.zoneId = zoneId;
    for (const bridge of thermalBridges) bridge.zoneId = zoneId;
    zones.push({
      id: zoneId, name: zoneName, floorArea, volume, height, surfaces, thermalBridges,
      airTightness: assumed ? { qv10, assumed: true } : { qv10 },
    });
  });

  const heatingSystems: IHeatingSystem[] = [];
  const ventilationSystems: IVentilationSystem[] = [];
  const coolingSystems: ICoolingSystem[] = [];
  const hotWaterSystems: IHotWaterSystem[] = [];
  const solarPV: ISolarPV[] = [];
  const solarThermal: ISolarThermal[] = [];

  const installParent = firstChild(unit.NTAEntityDataId, 'UNIT_INSTALLATIE') || unit;

  const heating = (opwek: UNIEC3Entity) => {
    const hp = `heatingSystems[${heatingSystems.length}]`;
    heatingSystems.push({
      id: crypto.randomUUID(),
      name: getEntityProp(opwek, PROP.VERW_NAAM) || 'Verwarming',
      type: codeOf<HeatingSystemType>(issues, `${hp}.type`, getEntityProp(opwek, PROP.VERW_TYPE),
        HEATING_FROM_UNIEC3, 'hr107', 'Verwarmingssoort'),
      cop: issues.number(`${hp}.cop`, getEntityProp(opwek, PROP.VERW_COP), { fallback: 1, min: 0, minExclusive: true }),
      coverageFraction: issues.number(`${hp}.coverageFraction`, getEntityProp(opwek, PROP.VERW_DEKKING),
        { fallback: 1, min: 0, max: 1, clamp: true }),
    });
  };
  for (const verw of getChildren(installParent.NTAEntityDataId, 'INST_VERW')) {
    getChildren(verw.NTAEntityDataId, 'VERW_OPWEK').forEach(heating);
  }
  if (heatingSystems.length === 0) findByType(ENT['VERW-OPWEK']).forEach(heating);

  const ventilation = (vent: UNIEC3Entity) => {
    const vp = `ventilationSystems[${ventilationSystems.length}]`;
    ventilationSystems.push({
      id: crypto.randomUUID(),
      name: getEntityProp(vent, PROP.VENT_NAAM) || 'Ventilatie',
      type: codeOf<VentilationType>(issues, `${vp}.type`, getEntityProp(vent, PROP.VENT_TYPE),
        VENT_FROM_UNIEC3, 'natural', 'Ventilatiesoort'),
      heatRecoveryEfficiency: issues.number(`${vp}.heatRecoveryEfficiency`, getEntityProp(vent, PROP.VENT_WTW),
        { fallback: 0, min: 0, max: 1, clamp: true, silentWhenMissing: true }),
      sfp: issues.number(`${vp}.sfp`, getEntityProp(vent, PROP.VENT_SFP), { fallback: 0, min: 0, silentWhenMissing: true }),
    });
  };
  getChildren(installParent.NTAEntityDataId, 'INST_VENT').forEach(ventilation);
  if (ventilationSystems.length === 0) findByType(ENT.VENT).forEach(ventilation);

  for (const koel of getChildren(installParent.NTAEntityDataId, 'INST_KOEL')) {
    for (const opwek of getChildren(koel.NTAEntityDataId, 'KOEL_OPWEK')) {
      const cp = `coolingSystems[${coolingSystems.length}]`;
      const type = codeOf<CoolingSystemType>(issues, `${cp}.type`, getEntityProp(opwek, PROP.KOEL_TYPE),
        COOL_FROM_UNIEC3, 'none', 'Koelsoort');
      coolingSystems.push({
        id: crypto.randomUUID(),
        name: getEntityProp(opwek, PROP.KOEL_NAAM) || 'Koeling',
        type,
        // Without cooling the EER is not used, so 0 is kept as it is.
        eer: issues.number(`${cp}.eer`, getEntityProp(opwek, PROP.KOEL_EER),
          { fallback: 3, min: 0, minExclusive: type !== 'none' }),
      });
    }
  }

  for (const tapw of getChildren(installParent.NTAEntityDataId, 'INST_TAPW')) {
    for (const opwek of getChildren(tapw.NTAEntityDataId, 'TAPW_OPWEK')) {
      const wp = `hotWaterSystems[${hotWaterSystems.length}]`;
      hotWaterSystems.push({
        id: crypto.randomUUID(),
        name: getEntityProp(opwek, PROP.TAPW_NAAM) || 'Tapwater',
        type: codeOf<HotWaterSystemType>(issues, `${wp}.type`, getEntityProp(opwek, PROP.TAPW_TYPE),
          HW_FROM_UNIEC3, 'hr_combi', 'Tapwatersoort'),
        efficiency: issues.number(`${wp}.efficiency`, getEntityProp(opwek, PROP.TAPW_REND),
          { fallback: 1, min: 0, minExclusive: true }),
        hasSolarBoiler: getEntityProp(opwek, PROP.TAPW_ZONTHERM) === 'True',
        solarBoilerFraction: issues.number(`${wp}.solarBoilerFraction`, getEntityProp(opwek, PROP.TAPW_ZONTHERM_FRAC),
          { fallback: 0, min: 0, max: 1, clamp: true, silentWhenMissing: true }),
      });
    }
  }

  const pvField = (pvVeld: UNIEC3Entity) => {
    const pp = `solarPV[${solarPV.length}]`;
    solarPV.push({
      id: crypto.randomUUID(),
      name: getEntityProp(pvVeld, PROP.PV_NAAM) || 'PV-veld',
      peakPower: issues.number(`${pp}.peakPower`, getEntityProp(pvVeld, PROP.PV_WP), { fallback: 0, min: 0 }),
      orientation: codeOf<Orientation>(issues, `${pp}.orientation`, getEntityProp(pvVeld, PROP.PV_ORIENT),
        ORIENT_FROM_UNIEC3, 'S', 'Oriëntatie'),
      tilt: issues.number(`${pp}.tilt`, getEntityProp(pvVeld, PROP.PV_HELLING), { fallback: 30, min: 0, max: 90, clamp: true }),
      area: issues.number(`${pp}.area`, getEntityProp(pvVeld, PROP.PV_OPP), { fallback: 0, min: 0 }),
    });
  };
  for (const pvGroup of getChildren(installParent.NTAEntityDataId, 'INST_PV')) {
    getChildren(pvGroup.NTAEntityDataId, 'PV_VELD').forEach(pvField);
  }
  if (solarPV.length === 0) findByType(ENT['PV-VELD']).forEach(pvField);

  for (const zt of getChildren(installParent.NTAEntityDataId, 'INST_ZONTHERM')) {
    const tp = `solarThermal[${solarThermal.length}]`;
    solarThermal.push({
      id: crypto.randomUUID(),
      name: getEntityProp(zt, PROP.ZT_NAAM) || 'Zonnecollector',
      collectorArea: issues.number(`${tp}.collectorArea`, getEntityProp(zt, PROP.ZT_OPP), { fallback: 0, min: 0 }),
      type: codeOf<SolarThermalType>(issues, `${tp}.type`, getEntityProp(zt, PROP.ZT_TYPE),
        ST_FROM_UNIEC3, 'flat_plate', 'Collectorsoort'),
      orientation: codeOf<Orientation>(issues, `${tp}.orientation`, getEntityProp(zt, PROP.ZT_ORIENT),
        ORIENT_FROM_UNIEC3, 'S', 'Oriëntatie'),
      tilt: issues.number(`${tp}.tilt`, getEntityProp(zt, PROP.ZT_HELLING), { fallback: 45, min: 0, max: 90, clamp: true }),
    });
  }

  // What the file holds beyond this: entities nobody reads (another entity type, or one that is
  // not linked where the importer looks) and properties of read entities the importer has no
  // field for. Library copies and an empty settings entity hold nothing of their own.
  const unreadEntities = new Map<string, number>();
  const unreadProperties = new Map<string, Map<string, number>>();
  for (const entity of unique) {
    if (!read.has(entity)) {
      const empty = entity.NTAPropertyDatas.every((property) => property.Value.trim() === '');
      if (LIBRARY_ENTITIES.has(entity.NTAEntityId) || (entity.NTAEntityId === ENT.SETTINGS && empty)) continue;
      unreadEntities.set(entity.NTAEntityId, (unreadEntities.get(entity.NTAEntityId) ?? 0) + 1);
      continue;
    }
    for (const property of entity.NTAPropertyDatas) {
      if (READ_PROPERTIES.has(property.NTAPropertyId) || property.Value.trim() === '') continue;
      const names = unreadProperties.get(entity.NTAEntityId) ?? new Map<string, number>();
      names.set(property.NTAPropertyId, (names.get(property.NTAPropertyId) ?? 0) + 1);
      unreadProperties.set(entity.NTAEntityId, names);
    }
  }
  unreadData(issues, 'entities', 'Entiteiten', unreadEntities);
  for (const [type, names] of [...unreadProperties.entries()].sort(([a], [b]) => a.localeCompare(b))) {
    unreadData(issues, type, `Eigenschappen van ${type}`, names);
  }

  sourceVersionNotCarried(issues, 'UNIEC3', sourceVersion);
  ntaInputNotInFormat(issues, 'UNIEC3');

  return {
    project: {
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
    },
    issues,
    ...(sourceVersion ? { sourceVersion } : {}),
  };
}

/** The project of a Uniec 3 archive (see `readUNIEC3` for the import issues). */
export function importFromUNIEC3(zipData: Uint8Array): IProject {
  return readUNIEC3(zipData).project;
}

/** What a Uniec 3 export of this project leaves out (shown to the adviser). */
export function uniec3ExportLosses(project: IProject): string[] {
  return droppedOnExport(project, 'UNIEC3');
}

// ============================================================
// Download helper (export)
// ============================================================

/**
 * Saves the Uniec 3 archive and returns what the format left out (empty
 * when nothing was lost), so the adviser can be told.
 */
export function downloadUNIEC3(project: IProject): string[] {
  downloadBytes(exportToUNIEC3(project), `${project.name || 'project'}.input-draft.uniec3`);
  return uniec3ExportLosses(project);
}

// ============================================================
// File dialog helper (import)
// ============================================================

/** Picks and imports a Uniec 3 archive, with its name, SHA-256 and import issues. */
export async function openUNIEC3FileDialog(): Promise<ImportResult> {
  return importFile(await pickFile('.uniec3'), readUNIEC3);
}

