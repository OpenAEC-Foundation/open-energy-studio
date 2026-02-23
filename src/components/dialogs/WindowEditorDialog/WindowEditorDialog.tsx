import { useState, useMemo } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { IWindow, Orientation } from '../../../core/energy/types';

interface WindowEditorDialogProps {
  editId?: string | null;
  onClose: () => void;
}

const orientations: Orientation[] = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW', 'horizontal'];

export function WindowEditorDialog({ editId, onClose }: WindowEditorDialogProps) {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();
  const { project } = state;

  // Find existing window across all zones/surfaces
  let existingWindow: IWindow | null = null;
  let existingZoneId: string | null = null;
  let existingSurfaceId: string | null = null;
  if (editId) {
    for (const zone of project.zones) {
      for (const surface of zone.surfaces) {
        const found = surface.windows.find((w) => w.id === editId);
        if (found) {
          existingWindow = found;
          existingZoneId = zone.id;
          existingSurfaceId = surface.id;
          break;
        }
      }
      if (existingWindow) break;
    }
  }

  const [name, setName] = useState(existingWindow?.name ?? '');
  const [area, setArea] = useState(existingWindow?.area ?? 0);
  const [uValue, setUValue] = useState(existingWindow?.uValue ?? 1.0);
  const [gValue, setGValue] = useState(existingWindow?.gValue ?? 0.5);
  const [orientation, setOrientation] = useState<Orientation>(existingWindow?.orientation ?? 'S');
  const [zoneId, setZoneId] = useState(existingZoneId ?? (project.zones[0]?.id ?? ''));
  const [surfaceId, setSurfaceId] = useState(existingSurfaceId ?? '');

  // Filter surfaces by selected zone
  const availableSurfaces = useMemo(() => {
    const zone = project.zones.find((z) => z.id === zoneId);
    return zone?.surfaces ?? [];
  }, [project.zones, zoneId]);

  // Reset surface when zone changes (unless editing)
  const handleZoneChange = (newZoneId: string) => {
    setZoneId(newZoneId);
    if (!existingWindow) {
      setSurfaceId('');
    }
  };

  const handleSave = () => {
    if (!zoneId || !surfaceId) return;

    if (existingWindow && existingZoneId && existingSurfaceId) {
      dispatch({
        type: 'UPDATE_WINDOW',
        payload: {
          zoneId: existingZoneId,
          surfaceId: existingSurfaceId,
          windowId: existingWindow.id,
          data: { name, area, uValue, gValue, orientation },
        },
      });
    } else {
      const window: IWindow = {
        id: crypto.randomUUID(),
        name,
        area,
        uValue,
        gValue,
        orientation,
        surfaceId,
      };
      dispatch({ type: 'ADD_WINDOW', payload: { zoneId, surfaceId, window } });
    }
    onClose();
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <span className="dialog-header-title">{t('dialog.window.title')}</span>
          <button className="dialog-close-btn" onClick={onClose}>&times;</button>
        </div>
        <div className="dialog-body">

        <div className="dialog-field">
          <label>{t('dialog.window.name')}</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.window.area')}</label>
          <input
            type="number"
            min={0}
            step={0.01}
            value={area}
            onChange={(e) => setArea(parseFloat(e.target.value) || 0)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.window.uValue')}</label>
          <input
            type="number"
            min={0}
            step={0.01}
            value={uValue}
            onChange={(e) => setUValue(parseFloat(e.target.value) || 0)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.window.gValue')}</label>
          <input
            type="number"
            min={0}
            max={1}
            step={0.01}
            value={gValue}
            onChange={(e) => setGValue(parseFloat(e.target.value) || 0)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.window.orientation')}</label>
          <select value={orientation} onChange={(e) => setOrientation(e.target.value as Orientation)}>
            {orientations.map((o) => (
              <option key={o} value={o}>
                {t(`orientation.${o}`)}
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-field">
          <label>{t('dialog.surface.zone')}</label>
          <select
            value={zoneId}
            onChange={(e) => handleZoneChange(e.target.value)}
            disabled={!!existingWindow}
          >
            <option value="">--</option>
            {project.zones.map((z) => (
              <option key={z.id} value={z.id}>
                {z.name}
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-field">
          <label>{t('dialog.window.surface')}</label>
          <select
            value={surfaceId}
            onChange={(e) => setSurfaceId(e.target.value)}
            disabled={!!existingWindow}
          >
            <option value="">--</option>
            {availableSurfaces.map((s) => (
              <option key={s.id} value={s.id}>
                {s.name}
              </option>
            ))}
          </select>
        </div>

        </div>
        <div className="dialog-footer">
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
