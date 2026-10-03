import type { DialogType, IProject } from './types';
import type { EnergyAction } from '../../context/EnergyContext';
import {
  type CascadeEntry, type DeleteResult, deleteSurfaceFromProject, deleteWindowFromProject, deleteZoneFromProject,
  manualMeasuresShiftedBy,
} from './projectDelete';

/**
 * Edit and delete targets for project items shown in the browser tree,
 * the envelope view and the properties panel. Editing goes through the
 * existing dialogs with an `editId`, which update the item in place, so
 * ids stay stable for relabel comparisons and maatwerkadvies templates.
 */
export type ProjectItemType =
  | 'zone' | 'surface' | 'window' | 'thermalBridge' | 'pointBridge' | 'construction'
  | 'heatingSystem' | 'ventilationSystem' | 'coolingSystem' | 'hotWaterSystem'
  | 'solarPV' | 'solarThermal';

const EDIT_DIALOG: Record<ProjectItemType, DialogType> = {
  zone: 'zone-editor',
  surface: 'surface-editor',
  window: 'window-editor',
  thermalBridge: 'thermal-bridge',
  pointBridge: 'point-bridge',
  construction: 'construction-editor',
  heatingSystem: 'heating-system',
  ventilationSystem: 'ventilation-system',
  coolingSystem: 'cooling-system',
  hotWaterSystem: 'hot-water-system',
  solarPV: 'solar-pv',
  solarThermal: 'solar-thermal',
};

export function editDialogFor(itemType: string): DialogType {
  return EDIT_DIALOG[itemType as ProjectItemType] ?? null;
}

export function editAction(itemType: string, id: string): EnergyAction | null {
  const type = editDialogFor(itemType);
  return type ? { type: 'OPEN_DIALOG', payload: { type, editId: id } } : null;
}

/** Surfaces (zone/surface names) that use a construction. */
export function constructionUsage(project: IProject, constructionId: string): string[] {
  return project.zones.flatMap((zone) => zone.surfaces
    .filter((surface) => surface.constructionId === constructionId)
    .map((surface) => `${zone.name} › ${surface.name}`));
}

export type DeleteTarget =
  /** `cascade`: NTA input entries that the delete removes with it. */
  | { kind: 'action'; action: EnergyAction; cascade: CascadeEntry[] }
  | { kind: 'blocked'; reason: 'constructionInUse' | 'manualMeasures'; usedBy: string[] }
  | { kind: 'none' };

/** Geometry deletes cascade into the NTA block and are refused when they renumber a manual measure's path. */
function geometryDelete(project: IProject, action: EnergyAction, result: DeleteResult): DeleteTarget {
  const shifted = manualMeasuresShiftedBy(project, result.project, project.maatwerkadvies?.measures);
  return shifted.length > 0
    ? { kind: 'blocked', reason: 'manualMeasures', usedBy: shifted }
    : { kind: 'action', action, cascade: result.cascade };
}

export function deleteTarget(project: IProject, itemType: string, id: string): DeleteTarget {
  const action = (a: EnergyAction): DeleteTarget => ({ kind: 'action', action: a, cascade: [] });
  switch (itemType as ProjectItemType) {
    case 'zone':
      return project.zones.some((z) => z.id === id)
        ? geometryDelete(project, { type: 'DELETE_ZONE', payload: id }, deleteZoneFromProject(project, id))
        : { kind: 'none' };
    case 'surface':
      for (const zone of project.zones)
        if (zone.surfaces.some((s) => s.id === id))
          return geometryDelete(project, { type: 'DELETE_SURFACE', payload: { zoneId: zone.id, surfaceId: id } },
            deleteSurfaceFromProject(project, zone.id, id));
      return { kind: 'none' };
    case 'window':
      for (const zone of project.zones)
        for (const surface of zone.surfaces)
          if (surface.windows.some((w) => w.id === id))
            return geometryDelete(project,
              { type: 'DELETE_WINDOW', payload: { zoneId: zone.id, surfaceId: surface.id, windowId: id } },
              deleteWindowFromProject(project, zone.id, surface.id, id));
      return { kind: 'none' };
    case 'thermalBridge':
      for (const zone of project.zones)
        if (zone.thermalBridges.some((b) => b.id === id))
          return action({ type: 'DELETE_THERMAL_BRIDGE', payload: { zoneId: zone.id, bridgeId: id } });
      return { kind: 'none' };
    case 'pointBridge':
      for (const zone of project.zones)
        if ((zone.pointThermalBridges ?? []).some((b) => b.id === id))
          return action({ type: 'DELETE_POINT_BRIDGE', payload: { zoneId: zone.id, bridgeId: id } });
      return { kind: 'none' };
    case 'construction': {
      if (!project.constructions.some((c) => c.id === id)) return { kind: 'none' };
      const usedBy = constructionUsage(project, id);
      return usedBy.length > 0
        ? { kind: 'blocked', reason: 'constructionInUse', usedBy }
        : action({ type: 'DELETE_CONSTRUCTION', payload: id });
    }
    case 'heatingSystem': return action({ type: 'DELETE_HEATING_SYSTEM', payload: id });
    case 'ventilationSystem': return action({ type: 'DELETE_VENTILATION_SYSTEM', payload: id });
    case 'coolingSystem': return action({ type: 'DELETE_COOLING_SYSTEM', payload: id });
    case 'hotWaterSystem': return action({ type: 'DELETE_HOT_WATER_SYSTEM', payload: id });
    case 'solarPV': return action({ type: 'DELETE_SOLAR_PV', payload: id });
    case 'solarThermal': return action({ type: 'DELETE_SOLAR_THERMAL', payload: id });
    default: return { kind: 'none' };
  }
}
