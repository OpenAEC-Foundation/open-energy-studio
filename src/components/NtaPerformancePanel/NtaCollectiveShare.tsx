import { useI18n } from '../../i18n/i18n';
import { NumberField, read, TextField, type Draft, type Path } from './NtaFormFields';

// The share of a collective installation in the building (form coverage,
// docs/ui-redesign/formulierdekking.md): collective cooling (`cooling.collective`)
// and a shared PV system (`pvSystems[i].collective`, 6.6.7), each by the
// building's usable floor area.

type Change = (path: Path, value: unknown) => void;
interface SectionProps { draft: Draft; change: Change }

/** 6.6.7: the share of a collective installation (`base`: its `collective`) by the building's usable area. */
export function CollectiveShareFields({ draft, change, base, label }: SectionProps & { base: Path; label: string }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const present = read(draft, base) != null;
  return <>
    <label className="nta-form-check">
      <input type="checkbox" checked={present} onChange={(event) => change(base,
        event.target.checked ? { buildingUsableFloorAreaM2: null, sourceReference: '' } : undefined)} />
      {label}
    </label>
    {present && <>
      <NumberField {...field} path={[...base, 'buildingUsableFloorAreaM2']} label={t('nta.form.collectiveShare.area')} unit="m²" />
      <TextField {...field} path={[...base, 'sourceReference']} label={t('nta.form.collectiveShare.source')} />
    </>}
  </>;
}

/** Cooling installed collectively for the building (`cooling.collective`). */
export function CollectiveCoolingField({ draft, change, system = ['cooling'] }: SectionProps & { system?: Path }) {
  const { t } = useI18n();
  if (read(draft, system) == null) return null;
  return <CollectiveShareFields draft={draft} change={change} base={[...system, 'collective']} label={t('nta.form.cooling.collective')} />;
}

/** A PV system shared by the building (`pvSystems[i].collective`, 6.6.7). */
export function CollectivePvField({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  return <CollectiveShareFields draft={draft} change={change} base={[...base, 'collective']} label={t('nta.form.pv.collective')} />;
}
