import { useState, useId } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { ISurface, SurfaceType, Orientation, ThermalBoundary } from '../../../core/energy/types';
import { DialogShell } from '../DialogShell';

interface SurfaceEditorDialogProps {
  editId?: string | null;
  onClose: () => void;
}

const surfaceTypes: SurfaceType[] = ['wall', 'roof', 'floor'];
const orientations: Orientation[] = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW', 'horizontal'];
const thermalBoundaries: ThermalBoundary[] = ['outdoor', 'ground', 'unheated_space', 'adjacent_conditioned', 'internal'];

export function SurfaceEditorDialog({ editId, onClose }: SurfaceEditorDialogProps) {
  const fieldId = useId();
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
  const [thermalBoundary, setThermalBoundary] = useState<ThermalBoundary | ''>(existingSurface?.thermalBoundary ?? '');
  const [unheatedSpaceId, setUnheatedSpaceId] = useState(existingSurface?.unheatedSpaceId ?? '');
  const [area, setArea] = useState(existingSurface?.area ?? 0);
  const [orientation, setOrientation] = useState<Orientation>(existingSurface?.orientation ?? 'N');
  const [constructionId, setConstructionId] = useState(existingSurface?.constructionId ?? '');
  const [zoneId, setZoneId] = useState(existingZoneId ?? (project.zones[0]?.id ?? ''));
  const [errors, setErrors] = useState<Record<string, string>>({});

  const handleSave = () => {
    const newErrors: Record<string, string> = {};
    if (!zoneId) newErrors.zoneId = t('dialog.validation.required');
    if (!name.trim()) newErrors.name = t('dialog.validation.required');
    if (thermalBoundary === 'unheated_space' && !unheatedSpaceId) newErrors.unheatedSpaceId = t('dialog.validation.required');
    if (Object.keys(newErrors).length > 0) {
      setErrors(newErrors);
      return;
    }
    setErrors({});

    if (existingSurface && existingZoneId) {
      dispatch({
        type: 'UPDATE_SURFACE',
        payload: {
          zoneId: existingZoneId,
          surfaceId: existingSurface.id,
          data: { name, type, thermalBoundary: thermalBoundary || undefined,
            unheatedSpaceId: thermalBoundary === 'unheated_space' ? unheatedSpaceId : undefined,
            area, orientation, constructionId },
        },
      });
    } else {
      const surface: ISurface = {
        id: crypto.randomUUID(),
        name,
        type,
        thermalBoundary: thermalBoundary || undefined,
        unheatedSpaceId: thermalBoundary === 'unheated_space' ? unheatedSpaceId : undefined,
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
    <DialogShell
      title={t('dialog.surface.title')}
      onClose={onClose}
      onSubmit={handleSave}
      submitLabel={t('dialog.save')}
      cancelLabel={t('dialog.cancel')}
    >
        <div className="dialog-field">
          <label htmlFor={`${fieldId}-1`}>{t('dialog.surface.name')}</label>
          <input id={`${fieldId}-1`}
            type="text"
            value={name}
            onChange={(e) => { setName(e.target.value); setErrors(prev => ({ ...prev, name: '' })); }}
            className={errors.name ? 'field-error' : ''}
          />
          {errors.name && <span className="field-error-text">{errors.name}</span>}
        </div>

        {thermalBoundary === 'unheated_space' && <div className="dialog-field">
          <label htmlFor="surface-unheated-space">{t('kernel.unheated.space')}</label>
          <select id="surface-unheated-space" value={unheatedSpaceId} onChange={(event) => setUnheatedSpaceId(event.target.value)}>
            <option value="">--</option>
            {(project.unheatedSpaces ?? []).map((space) => <option key={space.id} value={space.id}>{space.name}</option>)}
          </select>
          {errors.unheatedSpaceId && <span className="field-error-text">{errors.unheatedSpaceId}</span>}
        </div>}

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-2`}>{t('dialog.surface.type')}</label>
          <select id={`${fieldId}-2`} value={type} onChange={(e) => setType(e.target.value as SurfaceType)}>
            {surfaceTypes.map((st) => (
              <option key={st} value={st}>
                {t(`dialog.surface.${st}`)}
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-field">
          <label htmlFor="surface-thermal-boundary">{t('kernel.boundary.label')}</label>
          <select id="surface-thermal-boundary" value={thermalBoundary}
            onChange={(event) => setThermalBoundary(event.target.value as ThermalBoundary | '')}>
            <option value="">{t('kernel.boundary.unknown')}</option>
            {thermalBoundaries.map((boundary) => <option key={boundary} value={boundary}>
              {t(`kernel.boundary.${boundary}`)}
            </option>)}
          </select>
          <small>{t('kernel.boundary.scope')}</small>
        </div>

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-3`}>{t('dialog.surface.area')}</label>
          <input id={`${fieldId}-3`}
            type="number"
            min={0}
            step={0.1}
            value={area}
            onChange={(e) => setArea(parseFloat(e.target.value) || 0)}
          />
        </div>

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-4`}>{t('dialog.surface.orientation')}</label>
          <select id={`${fieldId}-4`} value={orientation} onChange={(e) => setOrientation(e.target.value as Orientation)}>
            {orientations.map((o) => (
              <option key={o} value={o}>
                {t(`orientation.${o}`)}
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-5`}>{t('dialog.surface.construction')}</label>
          <select id={`${fieldId}-5`} value={constructionId} onChange={(e) => setConstructionId(e.target.value)}>
            <option value="">--</option>
            {project.constructions.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name} (U={c.uValue.toFixed(3)})
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-6`}>{t('dialog.surface.zone')}</label>
          <select id={`${fieldId}-6`}
            value={zoneId}
            onChange={(e) => { setZoneId(e.target.value); setErrors(prev => ({ ...prev, zoneId: '' })); }}
            disabled={!!existingSurface}
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
    </DialogShell>
  );
}
