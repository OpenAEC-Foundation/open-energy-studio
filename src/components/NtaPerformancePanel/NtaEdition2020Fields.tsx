import { useI18n } from '../../i18n/i18n';
import { NumberField, read, TextField, type Draft, type Path } from './NtaFormFields';

// Inputs that only NTA 8800:2020+A1 uses. Under a later edition a value left
// behind is offered for removal; the kernel accepts it there but it changes
// nothing.

type Change = (path: Path, value: unknown) => void;

export function hasEdition2020A1Routes(draft: Draft): boolean {
  return read(draft, ['normVersion']) === '2020+A1';
}

/** (9.85) of NTA 8800:2020+A1 (p. 336): heat pumps take the device forfait, A 13,0 kWh from build year 2015, otherwise 87,6 kWh. */
export function HeatPumpInstallationYear2020Field({ draft, change, base }: { draft: Draft; change: Change; base: Path }) {
  const { t } = useI18n();
  const year = [...base, 'installationYear'];
  const reference = [...base, 'installationYearReference'];
  const label = t('ntaStep.heatPumpInstallationYear');
  if (hasEdition2020A1Routes(draft)) {
    return <>
      <NumberField draft={draft} onChange={change} path={year} label={label} step="1" optional />
      <TextField draft={draft} onChange={change} path={reference} label={t('ntaStep.heatPumpInstallationYearReference')} />
    </>;
  }
  if (read(draft, year) == null && read(draft, reference) == null) return null;
  return <p className="nta-form-note nta-form-error" role="alert">
    {t('ntaStep.staleEdition2020A1', { field: label })}{' '}
    <button type="button" onClick={() => { change(year, undefined); change(reference, undefined); }}>{t('nta.form.remove')}</button>
  </p>;
}
