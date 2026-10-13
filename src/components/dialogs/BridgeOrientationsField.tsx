import type { Orientation } from '../../core/energy/types';
import { useI18n } from '../../i18n/i18n';

const ORIENTATIONS: Orientation[] = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW', 'horizontal'];

/**
 * The construction parts a thermal bridge lies on, for the TOjuli split of
 * NTA 8800 §5.7.2 step 2: a bridge on parts of one orientation counts there,
 * on parts of several orientations in equal shares; none ticked (or only
 * horizontal parts) distributes it pro rata like the other elements.
 */
export function BridgeOrientationsField({ value, onChange }: {
  value: Orientation[]; onChange: (next: Orientation[]) => void;
}) {
  const { t } = useI18n();
  return <fieldset className="dialog-field nta-form-group">
    <legend>{t('kernel.bridge.orientations')}</legend>
    {ORIENTATIONS.map((orientation) => <label key={orientation} className="nta-form-check">
      <input type="checkbox" checked={value.includes(orientation)}
        onChange={(event) => onChange(event.target.checked
          ? ORIENTATIONS.filter((item) => item === orientation || value.includes(item))
          : value.filter((item) => item !== orientation))} />
      {t(`orientation.${orientation}`)}
    </label>)}
    <small>{t('kernel.bridge.orientationsNote')}</small>
  </fieldset>;
}
