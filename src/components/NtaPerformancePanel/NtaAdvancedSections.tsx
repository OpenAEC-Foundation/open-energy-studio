import { useI18n } from '../../i18n/i18n';
import { airHeatersTemplate, bacsTemplate } from '../../core/nta/NtaSystemTemplates';
import {
  CheckField, NumberField, read, SelectField, TextField, TriStateField, type Draft, type Path,
} from './NtaFormFields';
import { ExternalSupplyFields } from './NtaExternalSupply';

// Less common NTA calculation inputs: area-weighted use functions (§5.3.1,
// Bbl art. 4.149), air-heater fans (9.23), §5.5.8 BACS evidence and the
// annex P external-supply routes (NtaExternalSupply).

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
    {rows.map((_, index) => <div key={index} className="nta-form-row" role="group"
      aria-label={`${t(`nta.form.${base}`)} ${index + 1}`}>
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

/** Table 9.12/9.13 type of one group of air heaters at `path` (its `kind`). */
function AirHeaterKindFields({ draft, change, path }: SectionProps & { path: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const kind = read(draft, [...path, 'kind']);
  const yesNo = { yes: t('nta.form.yes'), no: t('nta.form.no') };
  return <>
    <label>{t('nta.form.airHeaters.type')}
      <select value={typeof kind === 'string' ? kind : ''} onChange={(event) => change(path,
        (airHeatersTemplate(event.target.value as 'direct' | 'indirect') as { kind: unknown }).kind)}>
        <option value="direct">{t('nta.form.airHeaters.direct')}</option>
        <option value="indirect">{t('nta.form.airHeaters.indirect')}</option>
      </select>
    </label>
    {kind === 'direct' && <TriStateField {...field} {...yesNo} path={[...path, 'radialFan']} label={t('nta.form.airHeaters.radialFan')} />}
    {kind === 'indirect' && <>
      <TriStateField {...field} {...yesNo} path={[...path, 'roomHeightAbove8M']} label={t('nta.form.airHeaters.above8m')} />
      <TriStateField {...field} {...yesNo} path={[...path, 'warmAirReturn']} label={t('nta.form.airHeaters.warmAirReturn')} />
      <TriStateField {...field} {...yesNo} path={[...path, 'ecMotor']} label={t('nta.form.airHeaters.ecMotor')} />
    </>}
  </>;
}

/** 9.23 with tables 9.12/9.13: direct or indirect air heaters, of one or
 *  several types (n_H,aux counts every heater in the zone). */
export function AirHeatersFields({ draft, change }: SectionProps) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const base: Path = ['emission', 'airHeaters'];
  const kind = read(draft, [...base, 'kind', 'kind']);
  const yesNo = { yes: t('nta.form.yes'), no: t('nta.form.no') };
  const furtherValue = read(draft, [...base, 'further']);
  const further = Array.isArray(furtherValue) ? furtherValue : [];
  return <>
    <label>{t('nta.form.airHeaters')}
      <select value={typeof kind === 'string' ? kind : ''} onChange={(event) => {
        if (event.target.value === '') { change(base, undefined); return; }
        // Changing the type keeps the count, the other types, Q_h;b and the source.
        const kept: Record<string, unknown> = {
          designHeatLoadW: read(draft, [...base, 'designHeatLoadW']) ?? null,
          sourceReference: read(draft, [...base, 'sourceReference']) ?? '',
        };
        const count = read(draft, [...base, 'count']);
        if (count != null) kept.count = count;
        if (further.length > 0) kept.further = further;
        change(base, { ...airHeatersTemplate(event.target.value as 'direct' | 'indirect'), ...kept });
      }}>
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
      <NumberField {...field} path={[...base, 'count']} label={t('nta.form.airHeaters.count')} />
      {further.map((_, index) => <div key={index} className="nta-form-row">
        <AirHeaterKindFields draft={draft} change={change} path={[...base, 'further', index, 'kind']} />
        <NumberField {...field} path={[...base, 'further', index, 'count']} label={t('nta.form.airHeaters.count')} />
        <button type="button" className="nta-form-remove" onClick={() => {
          const rest = further.filter((__, item) => item !== index);
          change([...base, 'further'], rest.length > 0 ? rest : undefined);
        }}>{t('nta.form.remove')}</button>
      </div>)}
      <button type="button" className="btn" onClick={() => change([...base, 'further'],
        [...further, { kind: (airHeatersTemplate('direct') as { kind: unknown }).kind, count: 1 }])}>
        {t('nta.form.airHeaters.addType')}
      </button>
      <NumberField {...field} path={[...base, 'designHeatLoadW']} label={t('nta.form.airHeaters.designLoad')} />
      <TextField {...field} path={[...base, 'sourceReference']} label={t('nta.form.source')} />
    </>}
  </>;
}

const BACS_CLASSES: Array<[string, string]> = ['A', 'B', 'C', 'D'].map((key) => [key, key]);

/** NEN-EN 15232 evidence of one system or the whole building; omitted means none stated. */
function BacsEvidenceFields({ draft, change, base, label }: SectionProps & { base: Path; label: string }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const present = read(draft, base) != null;
  return <>
    <label className="nta-form-check">
      <input type="checkbox" checked={present}
        onChange={(event) => change(base, event.target.checked ? { present: false, sourceReference: '' } : undefined)} />
      {label}
    </label>
    {present && <>
      {/* Classes belong to a present BACS only (absent_bacs_has_classes): unticking clears them. */}
      <CheckField {...field} path={[...base, 'present']} label={t('nta.form.bacs.present')}
        onChange={(_, value) => change(base, value === true
          ? { ...(read(draft, base) as Draft), present: true }
          : { ...(read(draft, base) as Draft), present: false, automaticControlsClass: undefined, energyManagementClass: undefined })} />
      {read(draft, [...base, 'present']) === true && <>
        <SelectField {...field} path={[...base, 'automaticControlsClass']} label={t('nta.form.bacs.controlsClass')} options={BACS_CLASSES} />
        <SelectField {...field} path={[...base, 'energyManagementClass']} label={t('nta.form.bacs.managementClass')} options={BACS_CLASSES} />
      </>}
      <TextField {...field} path={[...base, 'sourceReference']} label={t('nta.form.source')} />
    </>}
  </>;
}

/** §5.5.8: the heating and cooling systems with their generator powers, and the BACS evidence. */
export function BacsFields({ draft, change }: SectionProps) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const systems = Array.isArray(read(draft, ['bacs', 'systems'])) ? read(draft, ['bacs', 'systems']) as Draft[] : [];
  return <div className="nta-form-subsection">
    <SelectField {...field} path={['bacs', 'buildingUse']} label={t('nta.form.bacs.use')} options={[
      ['residential', t('nta.form.bacs.use.residential')], ['utility', t('nta.form.bacs.use.utility')]]} />
    <CheckField {...field} path={['bacs', 'systemInventoryComplete']} label={t('nta.form.bacs.complete')} />
    <BacsEvidenceFields draft={draft} change={change} base={['bacs', 'bacs']} label={t('nta.form.bacs.building')} />
    {systems.map((system, index) => {
      const at = (...rest: Path): Path => ['bacs', 'systems', index, ...rest];
      const generators = Array.isArray(system.generators) ? system.generators as Draft[] : [];
      return <fieldset key={index} className="nta-form-row">
        <legend>{t('nta.form.bacs.system')} {index + 1}</legend>
        <TextField {...field} path={at('id')} label={t('nta.form.bacs.systemId')} />
        <SelectField {...field} path={at('service')} label={t('nta.form.bacs.service')} options={[
          ['heating', t('nta.form.bacs.service.heating')], ['cooling', t('nta.form.bacs.service.cooling')]]} />
        <TextField {...field} path={at('sourceReference')} label={t('nta.form.source')} />
        {generators.map((_, generator) => <div key={generator} className="nta-form-row" role="group"
          aria-label={`${t('nta.form.bacs.generator')} ${generator + 1}`}>
          <TextField {...field} path={at('generators', generator, 'id')} label={t('nta.form.bacs.generatorId')} />
          <NumberField {...field} path={at('generators', generator, 'nominalThermalCapacityKw')} label={t('nta.form.bacs.capacity')} />
          <TextField {...field} path={at('generators', generator, 'sourceReference')} label={t('nta.form.source')} />
          <button type="button" className="nta-form-remove"
            onClick={() => change(at('generators'), generators.filter((__, other) => other !== generator))}>
            {t('nta.form.bacs.removeGenerator')}
          </button>
        </div>)}
        <button type="button" onClick={() => change(at('generators'), [...generators,
          { id: `generator-${generators.length + 1}`, nominalThermalCapacityKw: null, sourceReference: '' }])}>
          {t('nta.form.bacs.addGenerator')}
        </button>
        <BacsEvidenceFields draft={draft} change={change} base={at('bacs')} label={t('nta.form.bacs.systemEvidence')} />
        <button type="button" className="nta-form-remove"
          onClick={() => change(['bacs', 'systems'], systems.filter((__, other) => other !== index))}>
          {t('nta.form.bacs.removeSystem')}
        </button>
      </fieldset>;
    })}
    <button type="button" onClick={() => change(['bacs', 'systems'], [...systems, {
      id: `system-${systems.length + 1}`, service: 'heating', sourceReference: '',
      generators: [{ id: 'generator-1', nominalThermalCapacityKw: null, sourceReference: '' }] }])}>
      {t('nta.form.bacs.addSystem')}
    </button>
  </div>;
}

/** §5.5.8 BACS evidence (replaces bacsFactor) and annex P external supply. */
export function BacsAndSupplyFields({ draft, change, residential }: SectionProps & { residential: boolean }) {
  const { t } = useI18n();
  const bacs = read(draft, ['bacs']) != null;
  return <>
    <label className="nta-form-check">
      <input type="checkbox" checked={bacs}
        onChange={(event) => change(['bacs'], event.target.checked ? bacsTemplate(residential) : undefined)} />
      {t('nta.form.bacsDraft')}
    </label>
    {bacs && <BacsFields draft={draft} change={change} />}
    <ExternalSupplyFields draft={draft} change={change} />
  </>;
}
