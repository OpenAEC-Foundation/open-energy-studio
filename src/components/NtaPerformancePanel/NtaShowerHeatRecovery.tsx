import { useI18n } from '../../i18n/i18n';
import { CheckField, NumberField, read, SelectField, TextField, type Draft, type Path } from './NtaFormFields';

// §13.5.3: shower heat recovery (douche-WTW) of a hot-water system, one unit
// per shower (13.53) with the connection factor of table 13.22.

interface Props {
  draft: Draft;
  change: (path: Path, value: unknown) => void;
  /** The hot-water system (`['hotWater']` or `['additionalHotWaterSystems', i]`). */
  base: Path;
  residential: boolean;
}

const UNITS = ['none', 'vertical', 'horizontal', 'unknown', 'declared'] as const;
const CONNECTIONS = ['mixer_and_heater', 'mixer_only', 'heater_only', 'shared_units', 'unknown'] as const;

/** One shower's unit in the kernel's `ShowerUnit` shape. */
export function showerUnitTemplate(unit: string): Draft {
  return unit === 'declared' ? { unit, efficiency: null, sourceReference: '' } : { unit };
}

export function showerHeatRecoveryTemplate(): Draft {
  return { showers: [showerUnitTemplate('vertical')], connection: 'mixer_and_heater', sourceReference: '' };
}

export function ShowerHeatRecoveryFields({ draft, change, base, residential }: Props) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const recovery: Path = [...base, 'showerHeatRecovery'];
  const present = read(draft, recovery) != null;
  const showers = (read(draft, [...recovery, 'showers']) as Draft[] | undefined) ?? [];
  return <>
    <label className="nta-form-check">
      <input type="checkbox" checked={present}
        onChange={(event) => change(recovery, event.target.checked ? showerHeatRecoveryTemplate() : null)} />
      {t('nta.form.shower')}
    </label>
    {present && <div className="nta-form-subsection">
      <SelectField {...field} path={[...recovery, 'connection']} label={t('nta.form.shower.connection')}
        options={CONNECTIONS.map((key) => [key, t(`nta.form.shower.connection.${key}`)])} />
      {!residential && <CheckField {...field} path={[...recovery, 'assignmentUnknown']} label={t('nta.form.shower.assignmentUnknown')} />}
      {showers.map((shower, index) => {
        const at: Path = [...recovery, 'showers', index];
        return <fieldset key={index} className="nta-form-row">
          <legend>{t('nta.form.shower.unit')} {index + 1}</legend>
          <SelectField required {...field} path={[...at, 'unit']} label={t('nta.form.shower.unitKind')}
            options={UNITS.map((key) => [key, t(`nta.form.shower.unit.${key}`)])}
            onChange={(_, value) => change(at, showerUnitTemplate(String(value ?? 'none')))} />
          {shower.unit === 'declared' && <>
            <NumberField {...field} path={[...at, 'efficiency']} label={t('nta.form.shower.efficiency')} />
            <TextField {...field} path={[...at, 'sourceReference']} label={t('nta.form.shower.unitSource')} />
          </>}
          <button type="button" onClick={() => change([...recovery, 'showers'], showers.filter((__, other) => other !== index))}>
            {t('nta.form.remove')}</button>
        </fieldset>;
      })}
      <button type="button" onClick={() => change([...recovery, 'showers'], [...showers, showerUnitTemplate('vertical')])}>
        {t('nta.form.shower.add')}</button>
      <TextField {...field} path={[...recovery, 'sourceReference']} label={t('nta.form.shower.source')} />
    </div>}
  </>;
}
