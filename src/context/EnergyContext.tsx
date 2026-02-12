import React, { createContext, useContext, useReducer, ReactNode } from 'react';
import {
  IProject,
  IZone,
  ISurface,
  IWindow,
  IThermalBridge,
  IAirTightness,
  IConstruction,
  IHeatingSystem,
  IVentilationSystem,
  ICoolingSystem,
  IHotWaterSystem,
  ISolarPV,
  ISolarThermal,
  IBENGResult,
  ViewMode,
  RibbonTab,
  DialogType,
  IDialogState,
} from '../core/energy/types';

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
}

// ============================================================
// Actions (discriminated union)
// ============================================================

export type EnergyAction =
  // Project-level
  | { type: 'SET_PROJECT'; payload: IProject }
  | { type: 'UPDATE_PROJECT_INFO'; payload: Partial<Pick<IProject, 'name' | 'description' | 'buildingFunction' | 'address' | 'city'>> }
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
  | { type: 'SET_DIRTY'; payload: boolean };

// ============================================================
// Default project factory
// ============================================================

export function createDefaultProject(): IProject {
  return {
    id: crypto.randomUUID(),
    name: '',
    description: '',
    buildingFunction: 'residential',
    address: '',
    city: '',
    zones: [],
    heatingSystems: [],
    ventilationSystems: [],
    coolingSystems: [],
    hotWaterSystems: [],
    solarPV: [],
    solarThermal: [],
    constructions: [],
  };
}

// ============================================================
// Initial state
// ============================================================

const initialState: EnergyState = {
  project: createDefaultProject(),
  result: null,
  viewMode: 'project',
  activeRibbonTab: 'start',
  dialog: { type: null, editId: null },
  selectedItemId: null,
  selectedItemType: null,
  isDirty: false,
};

// ============================================================
// Helpers
// ============================================================

/** Return a new zones array with the zone matching `zoneId` replaced by `updater(zone)`. */
function mapZone(zones: IZone[], zoneId: string, updater: (zone: IZone) => IZone): IZone[] {
  return zones.map(z => (z.id === zoneId ? updater(z) : z));
}

/** Return a new surfaces array with the surface matching `surfaceId` replaced by `updater(surface)`. */
function mapSurface(surfaces: ISurface[], surfaceId: string, updater: (s: ISurface) => ISurface): ISurface[] {
  return surfaces.map(s => (s.id === surfaceId ? updater(s) : s));
}

// ============================================================
// Reducer
// ============================================================

function energyReducer(state: EnergyState, action: EnergyAction): EnergyState {
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
          zones: mapZone(state.project.zones, id, z => ({ ...z, ...data })),
        },
        isDirty: true,
      };
    }

    case 'DELETE_ZONE':
      return {
        ...state,
        project: {
          ...state.project,
          zones: state.project.zones.filter(z => z.id !== action.payload),
        },
        isDirty: true,
      };

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
            surfaces: mapSurface(z.surfaces, surfaceId, s => ({ ...s, ...data })),
          })),
        },
        isDirty: true,
      };
    }

    case 'DELETE_SURFACE': {
      const { zoneId, surfaceId } = action.payload;
      return {
        ...state,
        project: {
          ...state.project,
          zones: mapZone(state.project.zones, zoneId, z => ({
            ...z,
            surfaces: z.surfaces.filter(s => s.id !== surfaceId),
          })),
        },
        isDirty: true,
      };
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
              windows: s.windows.map(w => (w.id === windowId ? { ...w, ...data } : w)),
            })),
          })),
        },
        isDirty: true,
      };
    }

    case 'DELETE_WINDOW': {
      const { zoneId, surfaceId, windowId } = action.payload;
      return {
        ...state,
        project: {
          ...state.project,
          zones: mapZone(state.project.zones, zoneId, z => ({
            ...z,
            surfaces: mapSurface(z.surfaces, surfaceId, s => ({
              ...s,
              windows: s.windows.filter(w => w.id !== windowId),
            })),
          })),
        },
        isDirty: true,
      };
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
            thermalBridges: z.thermalBridges.map(b => (b.id === bridgeId ? { ...b, ...data } : b)),
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
          constructions: state.project.constructions.map(c => (c.id === id ? { ...c, ...data } : c)),
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
          heatingSystems: state.project.heatingSystems.map(s => (s.id === id ? { ...s, ...data } : s)),
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
          ventilationSystems: state.project.ventilationSystems.map(s => (s.id === id ? { ...s, ...data } : s)),
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
          coolingSystems: state.project.coolingSystems.map(s => (s.id === id ? { ...s, ...data } : s)),
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
          hotWaterSystems: state.project.hotWaterSystems.map(s => (s.id === id ? { ...s, ...data } : s)),
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
          solarPV: state.project.solarPV.map(s => (s.id === id ? { ...s, ...data } : s)),
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
          solarThermal: state.project.solarThermal.map(s => (s.id === id ? { ...s, ...data } : s)),
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

    default:
      return state;
  }
}

// ============================================================
// Context
// ============================================================

interface EnergyContextType {
  state: EnergyState;
  dispatch: React.Dispatch<EnergyAction>;
}

const EnergyContext = createContext<EnergyContextType | null>(null);

// ============================================================
// Provider
// ============================================================

export function EnergyProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(energyReducer, initialState);

  return (
    <EnergyContext.Provider value={{ state, dispatch }}>
      {children}
    </EnergyContext.Provider>
  );
}

// ============================================================
// Hook
// ============================================================

export function useEnergy() {
  const context = useContext(EnergyContext);
  if (!context) {
    throw new Error('useEnergy must be used within an EnergyProvider');
  }
  return context;
}
