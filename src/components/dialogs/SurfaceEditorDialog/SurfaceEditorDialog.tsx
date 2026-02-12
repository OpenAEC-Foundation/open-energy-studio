import { useState } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { ISurface, SurfaceType, Orientation } from '../../../core/energy/types';

interface SurfaceEditorDialogProps {
  editId?: string | null;
  onClose: () => void;
}

const surfaceTypes: SurfaceType[] = ['wall', 'roof', 'floor'];
const orientations: Orientation[] = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW', 'horizontal'];

export function SurfaceEditorDialog({ editId, onClose }: SurfaceEditorDialogProps) {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();
  const { project } = state;

  // Find existing surface across all zones
  let existingSurface: ISurface | null = null;
  let existingZoneId: string | null = null;
  if (editId) {
    for (const zone of project.zones) {
      const found = zone.surfaces.find((s) => s.id === editId);
      if (found) {
        existingSurface = found;
        existingZoneId = zone.id;
        break;
      }
    }
  }

  const [name, setName] = useState(existingSurface?.name ?? '');
  const [type, setType] = useState<SurfaceType>(existingSurface?.type ?? 'wall');
  const [area, setArea] = useState(existingSurface?.area ?? 0);
  const [orientation, setOrientation] = useState<Orientation>(existingSurface?.orientation ?? 'N');
  const [constructionId, setConstructionId] = useState(existingSurface?.constructionId ?? '');
  const [zoneId, setZoneId] = useState(existingZoneId ?? (project.zones[0]?.id ?? ''));

  const handleSave = () => {
    if (!zoneId) return;

    if (existingSurface && existingZoneId) {
      dispatch({
        type: 'UPDATE_SURFACE',
        payload: {
          zoneId: existingZoneId,
          surfaceId: existingSurface.id,
          data: { name, type, area, orientation, constructionId },
        },
      });
    } else {
      const surface: ISurface = {
        id: crypto.randomUUID(),
        name,
        type,
        area,
        orientation,
        constructionId,
        zoneId,
        windows: [],
      };
      dispatch({ type: 'ADD_SURFACE', payload: { zoneId, surface } });
    }
    onClose();
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-title">{t('dialog.surface.title')}</div>

        <div className="dialog-field">
          <label>{t('dialog.surface.name')}</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.surface.type')}</label>
          <select value={type} onChange={(e) => setType(e.target.value as SurfaceType)}>
            {surfaceTypes.map((st) => (
              <option key={st} value={st}>
                {t(`dialog.surface.${st}`)}
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-field">
          <label>{t('dialog.surface.area')}</label>
          <input
            type="number"
            min={0}
            step={0.1}
            value={area}
            onChange={(e) => setArea(parseFloat(e.target.value) || 0)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.surface.orientation')}</label>
          <select value={orientation} onChange={(e) => setOrientation(e.target.value as Orientation)}>
            {orientations.map((o) => (
              <option key={o} value={o}>
                {t(`orientation.${o}`)}
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-field">
          <label>{t('dialog.surface.construction')}</label>
          <select value={constructionId} onChange={(e) => setConstructionId(e.target.value)}>
            <option value="">--</option>
            {project.constructions.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name} (U={c.uValue.toFixed(3)})
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-field">
          <label>{t('dialog.surface.zone')}</label>
          <select
            value={zoneId}
            onChange={(e) => setZoneId(e.target.value)}
            disabled={!!existingSurface}
          >
            <option value="">--</option>
            {project.zones.map((z) => (
              <option key={z.id} value={z.id}>
                {z.name}
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-actions">
          <button className="btn" onClick={onClose}>
            {t('dialog.cancel')}
          </button>
          <button className="btn btn-primary" onClick={handleSave}>
            {t('dialog.save')}
          </button>
        </div>
      </div>
    </div>
  );
}
