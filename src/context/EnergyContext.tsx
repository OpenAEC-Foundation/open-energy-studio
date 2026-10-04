import React, { createContext, useContext, useReducer, useCallback, ReactNode } from 'react';
import {
  IProject,
  IZone,
  ISurface,
  IWindow,
  IThermalBridge,
  IPointThermalBridge,
  IAirTightness,
  IConstruction,
  IHeatingSystem,
  IVentilationSystem,
  ICoolingSystem,
  IHotWaterSystem,
  INtaHeatPumpInput,
  ISolarPV,
  ISolarThermal,
  IBENGResult,
  ViewMode,
  RibbonTab,
  DialogType,
  IDialogState,
} from '../core/energy/types';
import { deleteSurfaceFromProject, deleteWindowFromProject, deleteZoneFromProject } from '../core/energy/projectDelete';

// ============================================================
// State
// ============================================================

export interface EnergyState {
  project: IProject;
  result: IBENGResult | null;
  viewMode: ViewMode;
  activeRibbonTab: RibbonTab;
  dialog: IDialogState;
  selectedItemId: string | null;
  selectedItemType: string | null;
  isDirty: boolean;
  previewVisible: boolean;
}

// ============================================================
// Actions (discriminated union)
// ============================================================

export type EnergyAction =
  // Project-level
  | { type: 'SET_PROJECT'; payload: IProject }
  | { type: 'UPDATE_PROJECT_INFO'; payload: Partial<Pick<IProject, 'name' | 'description' | 'buildingFunction' | 'address' | 'city' | 'registration'>> }
  | { type: 'SET_UNHEATED_SPACES'; payload: NonNullable<IProject['unheatedSpaces']> }
  | { type: 'SET_NTA_CALCULATION'; payload: IProject['ntaCalculation'] }
  | { type: 'SET_MAATWERKADVIES'; payload: IProject['maatwerkadvies'] }
  | { type: 'SET_BASISOPNAME'; payload: IProject['basisopname'] }
  // Zones
  | { type: 'ADD_ZONE'; payload: IZone }
  | { type: 'UPDATE_ZONE'; payload: { id: string; data: Partial<IZone> } }
  | { type: 'DELETE_ZONE'; payload: string }
  // Surfaces (nested in zone)
  | { type: 'ADD_SURFACE'; payload: { zoneId: string; surface: ISurface } }
  | { type: 'UPDATE_SURFACE'; payload: { zoneId: string; surfaceId: string; data: Partial<ISurface> } }
  | { type: 'DELETE_SURFACE'; payload: { zoneId: string; surfaceId: string } }
  // Windows (nested in surface within zone)
  | { type: 'ADD_WINDOW'; payload: { zoneId: string; surfaceId: string; window: IWindow } }
  | { type: 'UPDATE_WINDOW'; payload: { zoneId: string; surfaceId: string; windowId: string; data: Partial<IWindow> } }
  | { type: 'DELETE_WINDOW'; payload: { zoneId: string; surfaceId: string; windowId: string } }
  // Thermal bridges (nested in zone)
  | { type: 'ADD_THERMAL_BRIDGE'; payload: { zoneId: string; bridge: IThermalBridge } }
  | { type: 'UPDATE_THERMAL_BRIDGE'; payload: { zoneId: string; bridgeId: string; data: Partial<IThermalBridge> } }
  | { type: 'DELETE_THERMAL_BRIDGE'; payload: { zoneId: string; bridgeId: string } }
  | { type: 'ADD_POINT_BRIDGE'; payload: { zoneId: string; bridge: IPointThermalBridge } }
  | { type: 'UPDATE_POINT_BRIDGE'; payload: { zoneId: string; bridgeId: string; data: Partial<IPointThermalBridge> } }
  | { type: 'DELETE_POINT_BRIDGE'; payload: { zoneId: string; bridgeId: string } }
  // Air tightness (per zone)
  | { type: 'UPDATE_AIR_TIGHTNESS'; payload: { zoneId: string; airTightness: IAirTightness } }
  // Constructions
  | { type: 'ADD_CONSTRUCTION'; payload: IConstruction }
  | { type: 'UPDATE_CONSTRUCTION'; payload: { id: string; data: Partial<IConstruction> } }
  | { type: 'DELETE_CONSTRUCTION'; payload: string }
  // Heating systems
  | { type: 'ADD_HEATING_SYSTEM'; payload: IHeatingSystem }
  | { type: 'UPDATE_HEATING_SYSTEM'; payload: { id: string; data: Partial<IHeatingSystem> } }
  | { type: 'DELETE_HEATING_SYSTEM'; payload: string }
  // Ventilation systems
  | { type: 'ADD_VENTILATION_SYSTEM'; payload: IVentilationSystem }
  | { type: 'UPDATE_VENTILATION_SYSTEM'; payload: { id: string; data: Partial<IVentilationSystem> } }
  | { type: 'DELETE_VENTILATION_SYSTEM'; payload: string }
  // Cooling systems
  | { type: 'ADD_COOLING_SYSTEM'; payload: ICoolingSystem }
  | { type: 'UPDATE_COOLING_SYSTEM'; payload: { id: string; data: Partial<ICoolingSystem> } }
  | { type: 'DELETE_COOLING_SYSTEM'; payload: string }
  // Hot water systems
  | { type: 'ADD_HOT_WATER_SYSTEM'; payload: IHotWaterSystem }
  | { type: 'UPDATE_HOT_WATER_SYSTEM'; payload: { id: string; data: Partial<IHotWaterSystem> } }
  | { type: 'DELETE_HOT_WATER_SYSTEM'; payload: string }
  | { type: 'ADD_NTA_HEAT_PUMP'; payload: INtaHeatPumpInput }
  | { type: 'UPDATE_NTA_HEAT_PUMP'; payload: INtaHeatPumpInput }
  | { type: 'DELETE_NTA_HEAT_PUMP'; payload: string }
  // Solar PV
  | { type: 'ADD_SOLAR_PV'; payload: ISolarPV }
  | { type: 'UPDATE_SOLAR_PV'; payload: { id: string; data: Partial<ISolarPV> } }
  | { type: 'DELETE_SOLAR_PV'; payload: string }
  // Solar thermal
  | { type: 'ADD_SOLAR_THERMAL'; payload: ISolarThermal }
  | { type: 'UPDATE_SOLAR_THERMAL'; payload: { id: string; data: Partial<ISolarThermal> } }
  | { type: 'DELETE_SOLAR_THERMAL'; payload: string }
  // Results
  | { type: 'SET_RESULT'; payload: IBENGResult | null }
  // UI state
  | { type: 'SET_VIEW_MODE'; payload: ViewMode }
  | { type: 'SET_RIBBON_TAB'; payload: RibbonTab }
  | { type: 'OPEN_DIALOG'; payload: { type: DialogType; editId?: string | null } }
  | { type: 'CLOSE_DIALOG' }
  | { type: 'SELECT_ITEM'; payload: { id: string; itemType: string } }
  | { type: 'DESELECT_ITEM' }
  | { type: 'SET_DIRTY'; payload: boolean }
  | { type: 'TOGGLE_PREVIEW' };

// ============================================================
// Default project factory
// ============================================================

export function createDefaultProject(): IProject {
  return {
    id: '2467-goejanverwelledijk',
    name: '2467 Goejanverwelledijk 85 Gouda',
    description: 'Nieuwbouw vrijstaande woning met kap, 2 bouwlagen + zolder',
    buildingFunction: 'residential',
    address: 'Goejanverwelledijk 85',
    city: 'Gouda',
    zones: [
      {
        id: 'zone-main',
        name: 'Woonfunctie',
        floorArea: 133.06,
        volume: 345.96,
        height: 2.6,
        surfaces: [
          {
            id: 'surf-wall-n',
            name: 'Gevel Noord',
            type: 'wall',
            area: 28.6,
            orientation: 'N',
            constructionId: 'con-wall',
            zoneId: 'zone-main',
            windows: [
              { id: 'win-n1', name: 'Raam Noord 1', area: 1.8, uValue: 1.1, gValue: 0.40, orientation: 'N', surfaceId: 'surf-wall-n' },
              { id: 'win-n2', name: 'Raam Noord 2', area: 1.2, uValue: 1.2, gValue: 0.40, orientation: 'N', surfaceId: 'surf-wall-n' },
            ],
          },
          {
            id: 'surf-wall-e',
            name: 'Gevel Oost',
            type: 'wall',
            area: 36.4,
            orientation: 'E',
            constructionId: 'con-wall',
            zoneId: 'zone-main',
            windows: [
              { id: 'win-e1', name: 'Raam Oost 1', area: 2.4, uValue: 1.1, gValue: 0.40, orientation: 'E', surfaceId: 'surf-wall-e' },
              { id: 'win-e2', name: 'Raam Oost 2', area: 1.6, uValue: 1.2, gValue: 0.40, orientation: 'E', surfaceId: 'surf-wall-e' },
            ],
          },
          {
            id: 'surf-wall-s',
            name: 'Gevel Zuid',
            type: 'wall',
            area: 28.6,
            orientation: 'S',
            constructionId: 'con-wall',
            zoneId: 'zone-main',
            windows: [
              { id: 'win-s1', name: 'Raam Zuid groot', area: 4.8, uValue: 1.1, gValue: 0.40, orientation: 'S', surfaceId: 'surf-wall-s' },
              { id: 'win-s2', name: 'Raam Zuid 2', area: 2.4, uValue: 1.1, gValue: 0.40, orientation: 'S', surfaceId: 'surf-wall-s' },
              { id: 'win-s3', name: 'Deur Zuid', area: 2.1, uValue: 2.0, gValue: 0.00, orientation: 'S', surfaceId: 'surf-wall-s' },
            ],
          },
          {
            id: 'surf-wall-w',
            name: 'Gevel West',
            type: 'wall',
            area: 36.4,
            orientation: 'W',
            constructionId: 'con-wall',
            zoneId: 'zone-main',
            windows: [
              { id: 'win-w1', name: 'Raam West 1', area: 2.4, uValue: 1.1, gValue: 0.40, orientation: 'W', surfaceId: 'surf-wall-w' },
              { id: 'win-w2', name: 'Deur West', area: 2.1, uValue: 2.0, gValue: 0.00, orientation: 'W', surfaceId: 'surf-wall-w' },
            ],
          },
          {
            id: 'surf-roof-e',
            name: 'Dak Oost',
            type: 'roof',
            area: 42,
            orientation: 'E',
            constructionId: 'con-roof',
            zoneId: 'zone-main',
            windows: [
              { id: 'win-dakraam-e', name: 'Dakraam Oost', area: 1.2, uValue: 1.2, gValue: 0.40, orientation: 'E', surfaceId: 'surf-roof-e' },
            ],
          },
          {
            id: 'surf-roof-w',
            name: 'Dak West',
            type: 'roof',
            area: 42,
            orientation: 'W',
            constructionId: 'con-roof',
            zoneId: 'zone-main',
            windows: [
              { id: 'win-dakraam-w', name: 'Dakraam West', area: 1.2, uValue: 1.2, gValue: 0.40, orientation: 'W', surfaceId: 'surf-roof-w' },
            ],
          },
          {
            id: 'surf-floor',
            name: 'Vloer begane grond',
            type: 'floor',
            area: 72,
            orientation: 'horizontal',
            constructionId: 'con-floor',
            zoneId: 'zone-main',
            windows: [],
          },
        ],
        thermalBridges: [
          { id: 'tb-1', name: 'Gevel-vloer', psiValue: 0.05, length: 34, zoneId: 'zone-main' },
          { id: 'tb-2', name: 'Gevel-dak', psiValue: 0.05, length: 34, zoneId: 'zone-main' },
          { id: 'tb-3', name: 'Raamkozijnen', psiValue: 0.03, length: 65, zoneId: 'zone-main' },
        ],
        pointThermalBridges: [],
        pointBridgeInventoryComplete: false,
        airTightness: { qv10: 0.98 },
      },
    ],
    heatingSystems: [
      { id: 'heat-1', name: 'Warmtepomp lucht (vloerverwarming)', type: 'heat_pump_air', cop: 3.00, coverageFraction: 1.0 },
    ],
    ventilationSystems: [
      { id: 'vent-1', name: 'Type D centraal', type: 'type_d', heatRecoveryEfficiency: 0.0, sfp: 0.8 },
    ],
    coolingSystems: [
      { id: 'cool-1', name: 'Compressiekoeling', type: 'split_unit', eer: 3.00 },
    ],
    hotWaterSystems: [
      { id: 'hw-1', name: 'Warmtepompboiler', type: 'heat_pump', efficiency: 1.40, hasSolarBoiler: false, solarBoilerFraction: 0 },
    ],
    ntaHeatPumps: [],
    solarPV: [
      { id: 'pv-west', name: 'PV West (18 panelen)', peakPower: 7.2, orientation: 'W', tilt: 30, area: 30.6 },
      { id: 'pv-east', name: 'PV Oost (3 panelen)', peakPower: 1.2, orientation: 'E', tilt: 30, area: 5.1 },
    ],
    solarThermal: [],
    constructions: [
      { id: 'con-wall', name: 'Gevel Rc=6.76', layers: [{ material: 'Isolatie PUR/PIR', thickness: 0.170, lambda: 0.025 }], rcValue: 6.76, uValue: 0.145 },
      { id: 'con-roof', name: 'Dak Rc=6.30', layers: [{ material: 'Isolatie', thickness: 0.189, lambda: 0.03 }], rcValue: 6.30, uValue: 0.155 },
      { id: 'con-floor', name: 'Vloer Rc=5.09', layers: [{ material: 'Isolatie EPS', thickness: 0.153, lambda: 0.03 }], rcValue: 5.09, uValue: 0.190 },
    ],
  };
}

// ============================================================
// Per-document state factory
// ============================================================

function createDocumentState(project: IProject): EnergyState {
  return {
    project,
    result: null,
    viewMode: 'project',
    activeRibbonTab: 'start',
    dialog: { type: null, editId: null },
    selectedItemId: null,
    selectedItemType: null,
    isDirty: false,
    previewVisible: true,
  };
}

// ============================================================
// Document manager (multi-document layer)
// ============================================================

export interface DocumentEntry {
  id: string;
  filePath: string | null;
  state: EnergyState;
}

export interface DocumentManagerState {
  documents: DocumentEntry[];
  activeDocumentId: string | null;
}

export type DocumentManagerAction =
  | { type: 'DOC_NEW'; payload: { id: string; project: IProject } }
  | { type: 'DOC_OPEN'; payload: { id: string; project: IProject; filePath: string | null } }
  | { type: 'DOC_CLOSE'; payload: string }
  | { type: 'DOC_SET_ACTIVE'; payload: string }
  | { type: 'DOC_SET_FILE_PATH'; payload: { id: string; filePath: string } }
  | { type: 'DOC_DISPATCH'; payload: { id: string; action: EnergyAction } };

export function documentManagerReducer(
  state: DocumentManagerState,
  action: DocumentManagerAction,
): DocumentManagerState {
  switch (action.type) {
    case 'DOC_NEW': {
      const newDoc: DocumentEntry = {
        id: action.payload.id,
        filePath: null,
        state: createDocumentState(action.payload.project),
      };
      return {
        documents: [...state.documents, newDoc],
        activeDocumentId: newDoc.id,
      };
    }
    case 'DOC_OPEN': {
      // Only a saved location identifies an open document; a browser open has none.
      const existing = action.payload.filePath != null
        ? state.documents.find(d => d.filePath === action.payload.filePath)
        : undefined;
      if (existing) {
        return { ...state, activeDocumentId: existing.id };
      }
      const newDoc: DocumentEntry = {
        id: action.payload.id,
        filePath: action.payload.filePath,
        state: createDocumentState(action.payload.project),
      };
      return {
        documents: [...state.documents, newDoc],
        activeDocumentId: newDoc.id,
      };
    }
    case 'DOC_CLOSE': {
      const remaining = state.documents.filter(d => d.id !== action.payload);
      let newActiveId: string | null = null;
      if (remaining.length > 0) {
        if (state.activeDocumentId === action.payload) {
          const closedIndex = state.documents.findIndex(d => d.id === action.payload);
          newActiveId = remaining[Math.min(closedIndex, remaining.length - 1)].id;
        } else {
          newActiveId = state.activeDocumentId;
        }
      }
      return { documents: remaining, activeDocumentId: newActiveId };
    }
    case 'DOC_SET_ACTIVE':
      return { ...state, activeDocumentId: action.payload };
    case 'DOC_SET_FILE_PATH':
      return {
        ...state,
        documents: state.documents.map(d =>
          d.id === action.payload.id
            ? { ...d, filePath: action.payload.filePath }
            : d
        ),
      };
    case 'DOC_DISPATCH':
      return {
        ...state,
        documents: state.documents.map(d =>
          d.id === action.payload.id
            ? { ...d, state: energyReducer(d.state, action.payload.action) }
            : d
        ),
      };
    default:
      return state;
  }
}

const initialDocManagerState: DocumentManagerState = {
  documents: [{
    id: 'default',
    filePath: null,
    state: createDocumentState(createDefaultProject()),
  }],
  activeDocumentId: 'default',
};

// ============================================================
// Helpers
// ============================================================

/** Return a new zones array with the zone matching `zoneId` replaced by `updater(zone)`. */
function mapZone(zones: IZone[], zoneId: string, updater: (zone: IZone) => IZone): IZone[] {
  return zones.map(z => (z.id === zoneId ? updater(z) : z));
}

/** Return a new surfaces array with the surface matching `surfaceId` replaced by `updater(surface)`. */
/** The object without keys whose value is `undefined`, so an editor round trip adds no empty keys. */
function withoutUndefined<T extends object>(value: T): T {
  return Object.fromEntries(Object.entries(value).filter(([, item]) => item !== undefined)) as T;
}

function mapSurface(surfaces: ISurface[], surfaceId: string, updater: (s: ISurface) => ISurface): ISurface[] {
  return surfaces.map(s => (s.id === surfaceId ? updater(s) : s));
}

// ============================================================
// Reducer
// ============================================================

function applyEnergyAction(state: EnergyState, action: EnergyAction): EnergyState {
  switch (action.type) {

    // ----------------------------------------------------------
    // Project-level
    // ----------------------------------------------------------

    case 'SET_PROJECT':
      return { ...state, project: action.payload, isDirty: false, result: null };

    case 'UPDATE_PROJECT_INFO':
      return {
        ...state,
        project: { ...state.project, ...action.payload },
        isDirty: true,
      };

    case 'SET_UNHEATED_SPACES':
      return { ...state, project: { ...state.project, unheatedSpaces: action.payload }, isDirty: true };

    case 'SET_NTA_CALCULATION':
      return { ...state, project: { ...state.project, ntaCalculation: action.payload }, isDirty: true };

    case 'SET_MAATWERKADVIES':
      return { ...state, project: { ...state.project, maatwerkadvies: action.payload }, isDirty: true };

    case 'SET_BASISOPNAME':
      return { ...state, project: { ...state.project, basisopname: action.payload }, isDirty: true };

    // ----------------------------------------------------------
    // Zones
    // ----------------------------------------------------------

    case 'ADD_ZONE':
      return {
        ...state,
        project: { ...state.project, zones: [...state.project.zones, action.payload] },
        isDirty: true,
      };

    case 'UPDATE_ZONE': {
      const { id, data } = action.payload;
      return {
        ...state,
        project: {
          ...state.project,
          zones: mapZone(state.project.zones, id, z => withoutUndefined({ ...z, ...data })),
        },
        isDirty: true,
      };
    }

    case 'DELETE_ZONE':
      return { ...state, project: deleteZoneFromProject(state.project, action.payload).project, isDirty: true };

    // ----------------------------------------------------------
    // Surfaces (nested in zone)
    // ----------------------------------------------------------

    case 'ADD_SURFACE': {
      const { zoneId, surface } = action.payload;
      return {
        ...state,
        project: {
          ...state.project,
          zones: mapZone(state.project.zones, zoneId, z => ({
            ...z,
            surfaces: [...z.surfaces, surface],
          })),
        },
        isDirty: true,
      };
    }

    case 'UPDATE_SURFACE': {
      const { zoneId, surfaceId, data } = action.payload;
      return {
        ...state,
        project: {
          ...state.project,
          zones: mapZone(state.project.zones, zoneId, z => ({
            ...z,
            // An `undefined` field in the update removes it (a cleared boundary), and is not stored as a key.
            surfaces: mapSurface(z.surfaces, surfaceId, s => withoutUndefined({ ...s, ...data })),
          })),
        },
        isDirty: true,
      };
    }

    case 'DELETE_SURFACE': {
      const { zoneId, surfaceId } = action.payload;
      return { ...state, project: deleteSurfaceFromProject(state.project, zoneId, surfaceId).project, isDirty: true };
    }

    // ----------------------------------------------------------
    // Windows (nested in surface within zone)
    // ----------------------------------------------------------

    case 'ADD_WINDOW': {
      const { zoneId, surfaceId, window: win } = action.payload;
      return {
        ...state,
        project: {
          ...state.project,
          zones: mapZone(state.project.zones, zoneId, z => ({
            ...z,
            surfaces: mapSurface(z.surfaces, surfaceId, s => ({
              ...s,
              windows: [...s.windows, win],
            })),
          })),
        },
        isDirty: true,
      };
    }

    case 'UPDATE_WINDOW': {
      const { zoneId, surfaceId, windowId, data } = action.payload;
      return {
        ...state,
        project: {
          ...state.project,
          zones: mapZone(state.project.zones, zoneId, z => ({
            ...z,
            surfaces: mapSurface(z.surfaces, surfaceId, s => ({
              ...s,
              windows: s.windows.map(w => (w.id === windowId ? withoutUndefined({ ...w, ...data }) : w)),
            })),
          })),
        },
        isDirty: true,
      };
    }

    case 'DELETE_WINDOW': {
      const { zoneId, surfaceId, windowId } = action.payload;
      return { ...state, project: deleteWindowFromProject(state.project, zoneId, surfaceId, windowId).project, isDirty: true };
    }

    // ----------------------------------------------------------
    // Thermal bridges (nested in zone)
    // ----------------------------------------------------------

    case 'ADD_THERMAL_BRIDGE': {
      const { zoneId, bridge } = action.payload;
      return {
        ...state,
        project: {
          ...state.project,
          zones: mapZone(state.project.zones, zoneId, z => ({
            ...z,
            thermalBridges: [...z.thermalBridges, bridge],
          })),
        },
        isDirty: true,
      };
    }

    case 'UPDATE_THERMAL_BRIDGE': {
      const { zoneId, bridgeId, data } = action.payload;
      return {
        ...state,
        project: {
          ...state.project,
          zones: mapZone(state.project.zones, zoneId, z => ({
            ...z,
            thermalBridges: z.thermalBridges.map(b => (b.id === bridgeId ? withoutUndefined({ ...b, ...data }) : b)),
          })),
        },
        isDirty: true,
      };
    }

    case 'DELETE_THERMAL_BRIDGE': {
      const { zoneId, bridgeId } = action.payload;
      return {
        ...state,
        project: {
          ...state.project,
          zones: mapZone(state.project.zones, zoneId, z => ({
            ...z,
            thermalBridges: z.thermalBridges.filter(b => b.id !== bridgeId),
          })),
        },
        isDirty: true,
      };
    }

    case 'ADD_POINT_BRIDGE': {
      const { zoneId, bridge } = action.payload;
      return { ...state, project: { ...state.project,
        zones: mapZone(state.project.zones, zoneId, z => ({ ...z,
          pointThermalBridges: [...(z.pointThermalBridges ?? []), bridge],
          pointBridgeInventoryComplete: false,
        })),
      }, isDirty: true };
    }

    case 'UPDATE_POINT_BRIDGE': {
      const { zoneId, bridgeId, data } = action.payload;
      return { ...state, project: { ...state.project,
        zones: mapZone(state.project.zones, zoneId, z => ({ ...z,
          pointThermalBridges: (z.pointThermalBridges ?? []).map(b => b.id === bridgeId ? withoutUndefined({ ...b, ...data }) : b),
        })),
      }, isDirty: true };
    }

    case 'DELETE_POINT_BRIDGE': {
      const { zoneId, bridgeId } = action.payload;
      return { ...state, project: { ...state.project,
        zones: mapZone(state.project.zones, zoneId, z => ({ ...z,
          pointThermalBridges: (z.pointThermalBridges ?? []).filter(b => b.id !== bridgeId),
          pointBridgeInventoryComplete: false,
        })),
      }, isDirty: true };
    }

    // ----------------------------------------------------------
    // Air tightness (per zone)
    // ----------------------------------------------------------

    case 'UPDATE_AIR_TIGHTNESS': {
      const { zoneId, airTightness } = action.payload;
      return {
        ...state,
        project: {
          ...state.project,
          zones: mapZone(state.project.zones, zoneId, z => ({
            ...z,
            airTightness,
          })),
        },
        isDirty: true,
      };
    }

    // ----------------------------------------------------------
    // Constructions
    // ----------------------------------------------------------

    case 'ADD_CONSTRUCTION':
      return {
        ...state,
        project: {
          ...state.project,
          constructions: [...state.project.constructions, action.payload],
        },
        isDirty: true,
      };

    case 'UPDATE_CONSTRUCTION': {
      const { id, data } = action.payload;
      return {
        ...state,
        project: {
          ...state.project,
          constructions: state.project.constructions.map(c => (c.id === id ? withoutUndefined({ ...c, ...data }) : c)),
        },
        isDirty: true,
      };
    }

    case 'DELETE_CONSTRUCTION':
      return {
        ...state,
        project: {
          ...state.project,
          constructions: state.project.constructions.filter(c => c.id !== action.payload),
        },
        isDirty: true,
      };

    // ----------------------------------------------------------
    // Heating systems
    // ----------------------------------------------------------

    case 'ADD_HEATING_SYSTEM':
      return {
        ...state,
        project: {
          ...state.project,
          heatingSystems: [...state.project.heatingSystems, action.payload],
        },
        isDirty: true,
      };

    case 'UPDATE_HEATING_SYSTEM': {
      const { id, data } = action.payload;
      return {
        ...state,
        project: {
          ...state.project,
          heatingSystems: state.project.heatingSystems.map(s => (s.id === id ? withoutUndefined({ ...s, ...data }) : s)),
        },
        isDirty: true,
      };
    }

    case 'DELETE_HEATING_SYSTEM':
      return {
        ...state,
        project: {
          ...state.project,
          heatingSystems: state.project.heatingSystems.filter(s => s.id !== action.payload),
        },
        isDirty: true,
      };

    // ----------------------------------------------------------
    // Ventilation systems
    // ----------------------------------------------------------

    case 'ADD_VENTILATION_SYSTEM':
      return {
        ...state,
        project: {
          ...state.project,
          ventilationSystems: [...state.project.ventilationSystems, action.payload],
        },
        isDirty: true,
      };

    case 'UPDATE_VENTILATION_SYSTEM': {
      const { id, data } = action.payload;
      return {
        ...state,
        project: {
          ...state.project,
          ventilationSystems: state.project.ventilationSystems.map(s => (s.id === id ? withoutUndefined({ ...s, ...data }) : s)),
        },
        isDirty: true,
      };
    }

    case 'DELETE_VENTILATION_SYSTEM':
      return {
        ...state,
        project: {
          ...state.project,
          ventilationSystems: state.project.ventilationSystems.filter(s => s.id !== action.payload),
        },
        isDirty: true,
      };

    // ----------------------------------------------------------
    // Cooling systems
    // ----------------------------------------------------------

    case 'ADD_COOLING_SYSTEM':
      return {
        ...state,
        project: {
          ...state.project,
          coolingSystems: [...state.project.coolingSystems, action.payload],
        },
        isDirty: true,
      };

    case 'UPDATE_COOLING_SYSTEM': {
      const { id, data } = action.payload;
      return {
        ...state,
        project: {
          ...state.project,
          coolingSystems: state.project.coolingSystems.map(s => (s.id === id ? withoutUndefined({ ...s, ...data }) : s)),
        },
        isDirty: true,
      };
    }

    case 'DELETE_COOLING_SYSTEM':
      return {
        ...state,
        project: {
          ...state.project,
          coolingSystems: state.project.coolingSystems.filter(s => s.id !== action.payload),
        },
        isDirty: true,
      };

    // ----------------------------------------------------------
    // Hot water systems
    // ----------------------------------------------------------

    case 'ADD_HOT_WATER_SYSTEM':
      return {
        ...state,
        project: {
          ...state.project,
          hotWaterSystems: [...state.project.hotWaterSystems, action.payload],
        },
        isDirty: true,
      };

    case 'UPDATE_HOT_WATER_SYSTEM': {
      const { id, data } = action.payload;
      return {
        ...state,
        project: {
          ...state.project,
          hotWaterSystems: state.project.hotWaterSystems.map(s => (s.id === id ? withoutUndefined({ ...s, ...data }) : s)),
        },
        isDirty: true,
      };
    }

    case 'DELETE_HOT_WATER_SYSTEM':
      return {
        ...state,
        project: {
          ...state.project,
          hotWaterSystems: state.project.hotWaterSystems.filter(s => s.id !== action.payload),
        },
        isDirty: true,
      };

    case 'ADD_NTA_HEAT_PUMP':
      return {
        ...state,
        project: {
          ...state.project,
          ntaHeatPumps: [...(state.project.ntaHeatPumps ?? []), action.payload],
        },
        result: null,
        isDirty: true,
      };

    case 'UPDATE_NTA_HEAT_PUMP':
      return {
        ...state,
        project: {
          ...state.project,
          ntaHeatPumps: (state.project.ntaHeatPumps ?? []).map((pump) =>
            pump.id === action.payload.id ? action.payload : pump),
        },
        result: null,
        isDirty: true,
      };

    case 'DELETE_NTA_HEAT_PUMP':
      return {
        ...state,
        project: {
          ...state.project,
          ntaHeatPumps: (state.project.ntaHeatPumps ?? []).filter((pump) => pump.id !== action.payload),
        },
        result: null,
        isDirty: true,
      };

    // ----------------------------------------------------------
    // Solar PV
    // ----------------------------------------------------------

    case 'ADD_SOLAR_PV':
      return {
        ...state,
        project: {
          ...state.project,
          solarPV: [...state.project.solarPV, action.payload],
        },
        isDirty: true,
      };

    case 'UPDATE_SOLAR_PV': {
      const { id, data } = action.payload;
      return {
        ...state,
        project: {
          ...state.project,
          solarPV: state.project.solarPV.map(s => (s.id === id ? withoutUndefined({ ...s, ...data }) : s)),
        },
        isDirty: true,
      };
    }

    case 'DELETE_SOLAR_PV':
      return {
        ...state,
        project: {
          ...state.project,
          solarPV: state.project.solarPV.filter(s => s.id !== action.payload),
        },
        isDirty: true,
      };

    // ----------------------------------------------------------
    // Solar thermal
    // ----------------------------------------------------------

    case 'ADD_SOLAR_THERMAL':
      return {
        ...state,
        project: {
          ...state.project,
          solarThermal: [...state.project.solarThermal, action.payload],
        },
        isDirty: true,
      };

    case 'UPDATE_SOLAR_THERMAL': {
      const { id, data } = action.payload;
      return {
        ...state,
        project: {
          ...state.project,
          solarThermal: state.project.solarThermal.map(s => (s.id === id ? withoutUndefined({ ...s, ...data }) : s)),
        },
        isDirty: true,
      };
    }

    case 'DELETE_SOLAR_THERMAL':
      return {
        ...state,
        project: {
          ...state.project,
          solarThermal: state.project.solarThermal.filter(s => s.id !== action.payload),
        },
        isDirty: true,
      };

    // ----------------------------------------------------------
    // Results
    // ----------------------------------------------------------

    case 'SET_RESULT':
      return { ...state, result: action.payload };

    // ----------------------------------------------------------
    // UI state
    // ----------------------------------------------------------

    case 'SET_VIEW_MODE':
      return { ...state, viewMode: action.payload };

    case 'SET_RIBBON_TAB':
      return { ...state, activeRibbonTab: action.payload };

    case 'OPEN_DIALOG':
      return {
        ...state,
        dialog: { type: action.payload.type, editId: action.payload.editId ?? null },
      };

    case 'CLOSE_DIALOG':
      return { ...state, dialog: { type: null, editId: null } };

    case 'SELECT_ITEM':
      return {
        ...state,
        selectedItemId: action.payload.id,
        selectedItemType: action.payload.itemType,
      };

    case 'DESELECT_ITEM':
      return { ...state, selectedItemId: null, selectedItemType: null };

    case 'SET_DIRTY':
      return { ...state, isDirty: action.payload };

    case 'TOGGLE_PREVIEW':
      return { ...state, previewVisible: !state.previewVisible };

    default:
      return state;
  }
}

/** The project after an action, without touching UI state; used to preview a delete. */
export function projectAfterAction(project: IProject, action: EnergyAction): IProject {
  const state: EnergyState = {
    project, result: null, viewMode: 'project', activeRibbonTab: 'start',
    dialog: { type: null, editId: null }, selectedItemId: null, selectedItemType: null, isDirty: false, previewVisible: false,
  };
  return applyEnergyAction(state, action).project;
}

function energyReducer(state: EnergyState, action: EnergyAction): EnergyState {
  const next = applyEnergyAction(state, action);
  return next.project !== state.project && next.result !== null
    ? { ...next, result: null }
    : next;
}

// ============================================================
// Contexts
// ============================================================

interface EnergyContextType {
  state: EnergyState;
  dispatch: React.Dispatch<EnergyAction>;
}

const EnergyContext = createContext<EnergyContextType | null>(null);

interface DocumentManagerContextType {
  docState: DocumentManagerState;
  docDispatch: React.Dispatch<DocumentManagerAction>;
}

const DocumentManagerContext = createContext<DocumentManagerContextType | null>(null);

// ============================================================
// Provider
// ============================================================

function ActiveDocumentBridge({ children }: { children: ReactNode }) {
  const { docState, docDispatch } = useDocumentManager();
  const activeDoc = docState.documents.find(d => d.id === docState.activeDocumentId);

  const dispatch = useCallback((action: EnergyAction) => {
    if (docState.activeDocumentId) {
      docDispatch({
        type: 'DOC_DISPATCH',
        payload: { id: docState.activeDocumentId, action },
      });
    }
  }, [docState.activeDocumentId, docDispatch]);

  const value = activeDoc
    ? { state: activeDoc.state, dispatch }
    : null;

  return (
    <EnergyContext.Provider value={value}>
      {children}
    </EnergyContext.Provider>
  );
}

export function EnergyProvider({ children }: { children: ReactNode }) {
  const [docState, docDispatch] = useReducer(documentManagerReducer, initialDocManagerState);

  return (
    <DocumentManagerContext.Provider value={{ docState, docDispatch }}>
      <ActiveDocumentBridge>
        {children}
      </ActiveDocumentBridge>
    </DocumentManagerContext.Provider>
  );
}

// ============================================================
// Hooks
// ============================================================

export function useEnergy() {
  const context = useContext(EnergyContext);
  if (!context) {
    throw new Error('useEnergy must be used within an EnergyProvider with an active document');
  }
  return context;
}

export function useDocumentManager() {
  const context = useContext(DocumentManagerContext);
  if (!context) {
    throw new Error('useDocumentManager must be used within an EnergyProvider');
  }
  return context;
}

export function useHasActiveDocument() {
  return useContext(EnergyContext) !== null;
}
