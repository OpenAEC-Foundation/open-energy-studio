import { useI18n } from '../../i18n/i18n';
import { airHeatersTemplate, bacsTemplate } from '../../core/nta/NtaSystemTemplates';
import {
  JsonField, NumberField, read, SelectField, TextField, TriStateField, type Draft, type Path,
} from './NtaFormFields';

// Less common NTA calculation inputs: area-weighted use functions (§5.3.1,
// Bbl art. 4.149), air-heater fans (9.23), §5.5.8 BACS evidence and the
// annex P external-supply routes (edited as JSON).

interface SectionProps {
  draft: Draft;
  change: (path: Path, value: unknown) => void;
}

export const LABEL_FUNCTIONS = ['residential', 'office', 'assembly_without_day_care', 'assembly_with_day_care', 'education',
  'healthcare_without_beds', 'healthcare_with_beds', 'retail', 'sport', 'lodging', 'cell'] as const;
export const BBL_FUNCTIONS = ['residential_building', 'caravan', 'floating_building_after2018_berth',
  'floating_building_other_berth', 'other_residential', 'assembly_child_care', 'other_assembly', 'cell',
  'healthcare_with_beds', 'other_healthcare', 'office', 'lodging_in_lodging_building', 'other_lodging', 'education',
  'sport', 'retail'] as const;

function list(draft: Draft, path: Path): Draft[] {
  const value = read(draft, path);
  return Array.isArray(value) ? value as Draft[] : [];
}

/** `labelFunctions` or `bblFunctions`: use functions with their usable area. */
export function FunctionAreasFields({ draft, change, base, functions, prefix }: SectionProps & {
  base: 'labelFunctions' | 'bblFunctions'; functions: readonly string[]; prefix: string;
}) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const rows = list(draft, [base]);
  return <>
    {rows.map((_, index) => <div key={index} className="nta-form-row">
      <SelectField {...field} path={[base, index, 'function']} label={t(`nta.form.${base}`)}
        options={functions.map((key) => [key, t(`${prefix}.${key}`)])} />
      <NumberField {...field} path={[base, index, 'areaM2']} label={t('nta.form.functionArea')} />
      <button type="button" className="nta-form-remove"
        onClick={() => change([base], rows.filter((_, item) => item !== index))}>{t('nta.form.remove')}</button>
    </div>)}
    <button type="button" className="btn" onClick={() => change([base], [...rows, { function: functions[0], areaM2: null }])}>
      {t(`nta.form.${base}.add`)}
    </button>
  </>;
}

/** 9.23 with tables 9.12/9.13: direct or indirect air heaters. */
export function AirHeatersFields({ draft, change }: SectionProps) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const base: Path = ['emission', 'airHeaters'];
  const kind = read(draft, [...base, 'kind', 'kind']);
  const yesNo = { yes: t('nta.form.yes'), no: t('nta.form.no') };
  return <>
    <label>{t('nta.form.airHeaters')}
      <select value={typeof kind === 'string' ? kind : ''} onChange={(event) => change(base, event.target.value === ''
        ? undefined
        : { ...airHeatersTemplate(event.target.value as 'direct' | 'indirect'),
          designHeatLoadW: read(draft, [...base, 'designHeatLoadW']) ?? null,
          sourceReference: read(draft, [...base, 'sourceReference']) ?? '' })}>
        <option value="">—</option>
        <option value="direct">{t('nta.form.airHeaters.direct')}</option>
        <option value="indirect">{t('nta.form.airHeaters.indirect')}</option>
      </select>
    </label>
    {kind === 'direct' && <TriStateField {...field} {...yesNo} path={[...base, 'kind', 'radialFan']} label={t('nta.form.airHeaters.radialFan')} />}
    {kind === 'indirect' && <>
      <TriStateField {...field} {...yesNo} path={[...base, 'kind', 'roomHeightAbove8M']} label={t('nta.form.airHeaters.above8m')} />
      <TriStateField {...field} {...yesNo} path={[...base, 'kind', 'warmAirReturn']} label={t('nta.form.airHeaters.warmAirReturn')} />
      <TriStateField {...field} {...yesNo} path={[...base, 'kind', 'ecMotor']} label={t('nta.form.airHeaters.ecMotor')} />
    </>}
    {kind != null && <>
      <NumberField {...field} path={[...base, 'designHeatLoadW']} label={t('nta.form.airHeaters.designLoad')} />
      <TextField {...field} path={[...base, 'sourceReference']} label={t('nta.form.source')} />
    </>}
  </>;
}

/** §5.5.8 BACS evidence (replaces bacsFactor) and annex P external supply. */
export function BacsAndSupplyFields({ draft, change, residential }: SectionProps & { residential: boolean }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const bacs = read(draft, ['bacs']) != null;
  return <>
    <label className="nta-form-check">
      <input type="checkbox" checked={bacs}
        onChange={(event) => change(['bacs'], event.target.checked ? bacsTemplate(residential) : undefined)} />
      {t('nta.form.bacsDraft')}
    </label>
    {bacs && <JsonField key="bacs" {...field} path={['bacs']} label={t('nta.form.bacsDraftJson')} invalid={t('nta.form.jsonInvalid')} />}
    <JsonField key="externalSupply" {...field} path={['externalSupply']} label={t('nta.form.externalSupplyJson')}
      invalid={t('nta.form.jsonInvalid')} />
    <p className="nta-form-note">{t('nta.form.externalSupplyNote')}</p>
  </>;
}
