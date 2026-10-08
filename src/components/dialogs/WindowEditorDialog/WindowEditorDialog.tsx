import { useState, useMemo, useId } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { IWindow, Orientation } from '../../../core/energy/types';
import { DialogShell } from '../DialogShell';
import { resolveWindow } from '../../../core/energy/windowTypes';

interface WindowEditorDialogProps {
  editId?: string | null;
  onClose: () => void;
}

const orientations: Orientation[] = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW', 'horizontal'];

export function WindowEditorDialog({ editId, onClose }: WindowEditorDialogProps) {
  const fieldId = useId();
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
  const [typeId, setTypeId] = useState(existingWindow?.typeId ?? '');
  const [count, setCount] = useState(existingWindow?.count ?? 1);
  const windowTypes = project.windowTypes ?? [];
  const type = windowTypes.find((item) => item.id === typeId);
  const pickType = (id: string) => {
    setTypeId(id);
    const picked = windowTypes.find((item) => item.id === id);
    if (!picked) return;
    setUValue(picked.uValue);
    setGValue(picked.gValue);
    if (picked.unitArea != null) setArea(Number((count * picked.unitArea).toFixed(3)));
  };
  const pickCount = (value: number) => {
    const next = Math.max(1, Math.round(value) || 1);
    setCount(next);
    if (type?.unitArea != null) setArea(Number((next * type.unitArea).toFixed(3)));
  };
  const [zoneId, setZoneId] = useState(existingZoneId ?? (project.zones[0]?.id ?? ''));
  const [surfaceId, setSurfaceId] = useState(existingSurfaceId ?? '');
  const [errors, setErrors] = useState<Record<string, string>>({});

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
    const newErrors: Record<string, string> = {};
    if (!name.trim()) newErrors.name = t('dialog.validation.required');
    if (!zoneId) newErrors.zoneId = t('dialog.validation.required');
    if (!surfaceId) newErrors.surfaceId = t('dialog.validation.required');
    if (Object.keys(newErrors).length > 0) {
      setErrors(newErrors);
      return;
    }
    setErrors({});

    if (existingWindow && existingZoneId && existingSurfaceId) {
      dispatch({
        type: 'UPDATE_WINDOW',
        payload: {
          zoneId: existingZoneId,
          surfaceId: existingSurfaceId,
          windowId: existingWindow.id,
          data: { ...resolveWindow({ ...existingWindow, name, area, uValue, gValue, orientation, typeId: typeId || undefined, count: typeId ? count : undefined }, type),
            typeId: typeId || undefined, count: typeId ? count : undefined },
        },
      });
    } else {
      const window: IWindow = resolveWindow({
        id: crypto.randomUUID(),
        name,
        area,
        uValue,
        gValue,
        orientation,
        surfaceId,
        ...(typeId ? { typeId, count } : {}),
      }, type);
      dispatch({ type: 'ADD_WINDOW', payload: { zoneId, surfaceId, window } });
    }
    onClose();
  };

  return (
    <DialogShell variant="sheet"
      title={t('dialog.window.title')}
      onClose={onClose}
      onSubmit={handleSave}
      submitLabel={t('dialog.save')}
      cancelLabel={t('dialog.cancel')}
    >
        <div className="dialog-field">
          <label htmlFor={`${fieldId}-1`}>{t('dialog.window.name')}</label>
          <input id={`${fieldId}-1`}
            type="text"
            value={name}
            onChange={(e) => { setName(e.target.value); setErrors(prev => ({ ...prev, name: '' })); }}
            className={errors.name ? 'field-error' : ''}
          />
          {errors.name && <span className="field-error-text">{errors.name}</span>}
        </div>

        {windowTypes.length > 0 && <div className="dialog-field">
          <label htmlFor={`${fieldId}-type`}>{t('dialog.window.type')}</label>
          <select id={`${fieldId}-type`} value={typeId} onChange={(e) => pickType(e.target.value)}>
            <option value="">{t('dialog.window.typeNone')}</option>
            {windowTypes.map((item) => <option key={item.id} value={item.id}>{item.name} · U {item.uValue} · g {item.gValue}</option>)}
          </select>
          {type && <span className="dialog-hint">{t('dialog.window.typeHint')}</span>}
        </div>}

        {type && <div className="dialog-field">
          <label htmlFor={`${fieldId}-count`}>{t('dialog.window.count')}</label>
          <input id={`${fieldId}-count`} type="number" min={1} step={1} value={count} onChange={(e) => pickCount(parseFloat(e.target.value))} />
        </div>}

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-2`}>{t('dialog.window.area')}</label>
          <input id={`${fieldId}-2`}
            type="number"
            min={0}
            step={0.01}
            value={area}
            disabled={type?.unitArea != null}
            onChange={(e) => setArea(parseFloat(e.target.value) || 0)}
          />
        </div>

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-3`}>{t('dialog.window.uValue')}</label>
          <input id={`${fieldId}-3`}
            type="number"
            min={0}
            step={0.01}
            value={uValue}
            disabled={!!type}
            onChange={(e) => setUValue(parseFloat(e.target.value) || 0)}
          />
        </div>

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-4`}>{t('dialog.window.gValue')}</label>
          <input id={`${fieldId}-4`}
            type="number"
            min={0}
            max={1}
            step={0.01}
            value={gValue}
            disabled={!!type}
            onChange={(e) => setGValue(parseFloat(e.target.value) || 0)}
          />
        </div>

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-5`}>{t('dialog.window.orientation')}</label>
          <select id={`${fieldId}-5`} value={orientation} onChange={(e) => setOrientation(e.target.value as Orientation)}>
            {orientations.map((o) => (
              <option key={o} value={o}>
                {t(`orientation.${o}`)}
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-6`}>{t('dialog.surface.zone')}</label>
          <select id={`${fieldId}-6`}
            value={zoneId}
            onChange={(e) => { handleZoneChange(e.target.value); setErrors(prev => ({ ...prev, zoneId: '' })); }}
            disabled={!!existingWindow}
            className={errors.zoneId ? 'field-error' : ''}
          >
            <option value="">--</option>
            {project.zones.map((z) => (
              <option key={z.id} value={z.id}>
                {z.name}
              </option>
            ))}
          </select>
          {errors.zoneId && <span className="field-error-text">{errors.zoneId}</span>}
        </div>

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-7`}>{t('dialog.window.surface')}</label>
          <select id={`${fieldId}-7`}
            value={surfaceId}
            onChange={(e) => { setSurfaceId(e.target.value); setErrors(prev => ({ ...prev, surfaceId: '' })); }}
            disabled={!!existingWindow}
            className={errors.surfaceId ? 'field-error' : ''}
          >
            <option value="">--</option>
            {availableSurfaces.map((s) => (
              <option key={s.id} value={s.id}>
                {s.name}
              </option>
            ))}
          </select>
          {errors.surfaceId && <span className="field-error-text">{errors.surfaceId}</span>}
        </div>
    </DialogShell>
  );
}
