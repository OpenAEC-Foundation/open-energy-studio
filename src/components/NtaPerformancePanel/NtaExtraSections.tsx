import { DistributionShell2023Field } from './NtaEdition2023Fields';
import { NumberInput } from '../ui/Field';
import { PipeGeometryFields, pipeGeometryTemplate } from './NtaPipeGeometry';
import { ConstantIlluminance2022Field } from './NtaEdition2022Fields';
import { CommonSpaceFillField, LightingZoneField } from './NtaZoneData';
import { useI18n } from '../../i18n/i18n';
import type { IProject } from '../../core/energy/types';
import { distributionSystemTemplate, lightingTemplate } from '../../core/nta/NtaFormModels';
import {
  DaylightSectorsFields, daylightSectorTemplate, LuminaireGroupsFields, luminaireGroupTemplate, ParasiticPowerFields,
} from './NtaLightingDetails';
import {
  CheckField, NumberField, read, Section, SelectField, TextField, type Draft, type Path,
} from './NtaFormFields';

interface SectionProps {
  draft: Draft;
  change: (path: Path, value: unknown) => void;
}

const DESIGN_CLASSES = ['30_27', '35_30', '40_35', '45_40', '50_42', '55_47', '60_50', '65_55', '70_60', '75_65', '80_60', '90_70'];

/** Utility internal gains 7.25–7.29: lighting from chapter 14 or declared W_t. */
export function NtaUtilityGainsFields({ draft, change }: SectionProps) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const lighting = read(draft, ['internalGains', 'lighting', 'method']);
  return <>
    <SelectField required {...field} path={['internalGains', 'lighting', 'method']} label={t('nta.gains.lighting')}
      options={[['chapter14', t('nta.gains.lighting.chapter14')], ['declared', t('nta.gains.lighting.declared')]]}
      onChange={(_, value) => change(['internalGains', 'lighting'], value === 'declared'
        ? { method: 'declared', annualKwh: null, recovery: 'forfait_power' } : { method: 'chapter14' })} />
    {lighting === 'declared' && <>
      <NumberField {...field} path={['internalGains', 'lighting', 'annualKwh']} label={t('nta.gains.lightingKwh')} />
      <SelectField {...field} path={['internalGains', 'lighting', 'recovery']} label={t('nta.gains.recovery')} options={[
        ['forfait_power', t('nta.gains.recovery.forfait')], ['extracted_luminaires', t('nta.gains.recovery.extracted')],
        ['other', t('nta.gains.recovery.other')]]} />
    </>}
    <label>{t('nta.gains.hotWaterRbl')}
      <NumberInput value={(() => {
        const values = read(draft, ['internalGains', 'hotWaterRecoverableKwh']) as number[] | undefined;
        return values && values.length === 12 && values.every((value) => value === values[0]) ? values[0] : null;
      })()} onChange={(next) => change(['internalGains', 'hotWaterRecoverableKwh'],
        next == null ? [] : Array(12).fill(next))} />
    </label>
    <p className="nta-form-note">{t('nta.gains.note')}</p>
  </>;
}

/** A free `lighting-N` id for a new lighting zone. */
function nextLightingZoneId(zones: Draft[]): string {
  const taken = new Set(zones.map((zone) => String(zone.id)));
  let index = zones.length + 1;
  while (taken.has(`lighting-${index}`)) index += 1;
  return `lighting-${index}`;
}

/**
 * Chapter 14 lighting for the single-zone block. With `measureOnly` (a
 * maatwerkadvies measure) only the changeable installation fields are open:
 * functions, areas and the list of lighting zones belong to the building.
 */
export function NtaLightingSection({ draft, change, project, measureOnly = false }:
  SectionProps & { project: IProject; measureOnly?: boolean }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const lighting = (read(draft, ['lighting']) as Draft[] | undefined) ?? [];
  const zones = (read(draft, ['lighting', 0, 'lightingZones']) as Draft[] | undefined) ?? [];
  return <Section title={t('nta.light.title')}>
    {measureOnly && <p className="nta-form-note">{t('nta.light.measureOnlyNote')}</p>}
    {!measureOnly && <label className="nta-form-check">
      <input type="checkbox" checked={lighting.length > 0} onChange={(event) => change(['lighting'],
        event.target.checked ? [lightingTemplate(project, read(draft, ['labelFunction']))] : [])} />
      {t('nta.light.present')}
    </label>}
    {lighting.length > 0 && <>
      {!measureOnly && <LightingZoneField draft={draft} change={change} project={project} index={0} />}
      <CommonSpaceFillField draft={draft} change={change} path={['lighting', 0, 'functions']}>
      <SelectField {...field} path={['lighting', 0, 'functions', 0, 'function']} label={t('nta.form.usageFunction')} options={[
        ['office', t('nta.form.label.office')], ['education', t('nta.form.label.education')], ['retail', t('nta.form.label.retail')],
        ['sport', t('nta.form.label.sport')], ['lodging', t('nta.form.label.lodging')], ['cell', t('nta.form.label.cell')],
        ['assembly_without_day_care', t('nta.form.label.assembly')], ['assembly_with_day_care', t('nta.form.label.dayCare')],
        ['healthcare_without_beds', t('nta.form.label.healthcare')], ['healthcare_with_beds', t('nta.form.label.healthcareBeds')]]}
        disabled={measureOnly} />
      <NumberField {...field} path={['lighting', 0, 'functions', 0, 'areaM2']} label={t('nta.form.area')} disabled={measureOnly} />
      </CommonSpaceFillField>
      {zones.map((zone, index) => <div key={String(zone.id)} className="nta-form-row" role="group" aria-label={String(zone.id)}>
        <NumberField {...field} path={['lighting', 0, 'lightingZones', index, 'areaM2']} label={`${String(zone.id)} — ${t('nta.form.area')}`}
          disabled={measureOnly} />
        <SelectField required {...field} path={['lighting', 0, 'lightingZones', index, 'power', 'method']} label={t('nta.light.power')}
          options={[['forfait', t('nta.light.power.forfait')], ['installed', t('nta.light.power.installed')]]}
          onChange={(_, value) => change(['lighting', 0, 'lightingZones', index, 'power'], value === 'installed'
            ? { method: 'installed', sourceReference: '', luminaires: [luminaireGroupTemplate()] }
            : { method: 'forfait', ledFrom2017: false })} />
        {read(draft, ['lighting', 0, 'lightingZones', index, 'power', 'method']) === 'installed' ? <>
          <LuminaireGroupsFields draft={draft} change={change} base={['lighting', 0, 'lightingZones', index, 'power']} />
          <TextField {...field} path={['lighting', 0, 'lightingZones', index, 'power', 'sourceReference']} label={t('nta.form.source')} />
        </> : <CheckField {...field} path={['lighting', 0, 'lightingZones', index, 'power', 'ledFrom2017']} label={t('nta.light.led')} />}
        <ParasiticPowerFields draft={draft} change={change} base={['lighting', 0, 'lightingZones', index, 'parasitic']} />
        <SelectField {...field} path={['lighting', 0, 'lightingZones', index, 'occupancy', 'control']} label={t('nta.light.control')} options={[
          ['manual_or_unknown', t('nta.light.control.manual')], ['manual_with_sweep', t('nta.light.control.sweep')],
          ['auto_on_dimmed', t('nta.light.control.autoDimmed')], ['auto_on_auto_off', t('nta.light.control.autoOff')],
          ['manual_on_dimmed', t('nta.light.control.manualDimmed')], ['manual_on_auto_off', t('nta.light.control.manualAutoOff')]]} />
        <CheckField {...field} path={['lighting', 0, 'lightingZones', index, 'occupancy', 'centralOnControl']} label={t('nta.light.central')} />
        {read(draft, ['lighting', 0, 'lightingZones', index, 'occupancy', 'centralOnControl']) === true &&
          <p className="nta-form-note">{t('nta.light.centralNote')}</p>}
        {read(draft, ['lighting', 0, 'lightingZones', index, 'power', 'method']) === 'forfait' &&
          <p className="nta-form-note">{t('nta.light.forfaitDaylightNote')}</p>}
        <SelectField required {...field} path={['lighting', 0, 'lightingZones', index, 'daylight', 'method']} label={t('nta.light.daylight')}
          disabled={read(draft, ['lighting', 0, 'lightingZones', index, 'power', 'method']) === 'forfait'}
          options={[['none', t('nta.light.daylight.none')], ['forfait', t('nta.light.daylight.forfait')],
            ['sectors', t('nta.light.daylight.sectors')]]}
          onChange={(_, value) => change(['lighting', 0, 'lightingZones', index, 'daylight'], value === 'forfait'
            ? { method: 'forfait', daylightControl: false }
            : value === 'sectors' ? { method: 'sectors', sectors: [daylightSectorTemplate('vertical_windows')], sourceReference: '' }
              : { method: 'none' })} />
        {read(draft, ['lighting', 0, 'lightingZones', index, 'daylight', 'method']) === 'forfait' &&
          <CheckField {...field} path={['lighting', 0, 'lightingZones', index, 'daylight', 'daylightControl']} label={t('nta.light.daylightControl')}
            disabled={read(draft, ['lighting', 0, 'lightingZones', index, 'power', 'method']) === 'forfait'} />}
        {read(draft, ['lighting', 0, 'lightingZones', index, 'daylight', 'method']) === 'sectors' &&
          <DaylightSectorsFields draft={draft} change={change} base={['lighting', 0, 'lightingZones', index, 'daylight']} />}
        <CheckField {...field} path={['lighting', 0, 'lightingZones', index, 'extractedLuminaires']} label={t('nta.light.extracted')} />
        <ConstantIlluminance2022Field draft={draft} change={change} path={['lighting', 0, 'lightingZones', index, 'constantIlluminance']} />
        {!measureOnly && zones.length > 1 && <button type="button" className="nta-form-remove"
          onClick={() => change(['lighting', 0, 'lightingZones'], zones.filter((__, other) => other !== index))}>{t('nta.form.remove')}</button>}
      </div>)}
      {!measureOnly && <button type="button" onClick={() => change(['lighting', 0, 'lightingZones'], [...zones, {
        id: nextLightingZoneId(zones), areaM2: null, power: { method: 'forfait', ledFrom2017: false }, parasitic: { method: 'forfait' },
        occupancy: { control: 'manual_or_unknown', centralOnControl: false, largeOfficeGroup: false }, daylight: { method: 'none' },
        extractedLuminaires: false,
      }])}>{t('nta.light.addZone')}</button>}
      {!measureOnly && <TextField {...field} path={['lighting', 0, 'sourceReference']} label={t('nta.form.source')} />}
      {project.zones.length > 1 && <p className="nta-form-note">{t('nta.light.note')}</p>}
    </>}
  </Section>;
}

/** Calculated distribution 9.26–9.51 (`distributionSystem`). */
/** §9.5 buffer vessel in the distribution system: volume, loss (measured or by label) and location. */
function BufferVesselFields({ draft, change }: SectionProps) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const base: Path = ['distributionSystem', 'bufferVessel'];
  const vessel = read(draft, base);
  return <>
    <label className="nta-form-check">
      <input type="checkbox" checked={vessel != null} onChange={(event) => change(base, event.target.checked
        ? { volumeL: null, standingLossW: null, label: null, producedFrom2018: false, inHeatedSpace: true, sourceReference: '' }
        : null)} />
      {t('nta.dist.buffer')}
    </label>
    {vessel != null && <div className="nta-form-subsection">
      <NumberField {...field} path={[...base, 'volumeL']} label={t('nta.dist.buffer.volume')} unit="l" />
      <NumberField {...field} path={[...base, 'standingLossW']} label={t('nta.dist.buffer.loss')} optional unit="W" />
      <SelectField {...field} path={[...base, 'label']} label={t('nta.dist.buffer.label')}
        options={(['a_plus', 'a', 'b', 'c', 'd', 'e', 'f', 'g'] as const)
          .map((key): [string, string] => [key, key === 'a_plus' ? 'A+' : key.toUpperCase()])} />
      <CheckField {...field} path={[...base, 'producedFrom2018']} label={t('nta.dist.buffer.from2018')} />
      <CheckField {...field} path={[...base, 'inHeatedSpace']} label={t('nta.dist.buffer.inHeated')} />
      <CheckField {...field} path={[...base, 'constantTemperature']} label={t('nta.dist.buffer.constant')} />
      <TextField {...field} path={[...base, 'sourceReference']} label={t('nta.dist.buffer.source')} />
    </div>}
  </>;
}

export function NtaDistributionFields({ draft, change }: SectionProps) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const system = read(draft, ['distributionSystem']) as Draft | undefined | null;
  const route = read(draft, ['distribution', 'method']);
  const pump = read(draft, ['distributionSystem', 'pump', 'method']);
  const pipe = read(draft, ['distributionSystem', 'pipeTransmittance', 'insulation', 'state']);
  const psiMethod = read(draft, ['distributionSystem', 'pipeTransmittance', 'method']);
  const psiCalculated = psiMethod != null && psiMethod !== 'forfait';
  const collective = read(draft, ['distributionSystem', 'installation']) === 'collective';
  return <>
    <SelectField required {...field} path={['distribution', 'method']} label={t('nta.dist.route')} options={[
      ['heated_zone_only_space_heating', t('nta.dist.route.heatedZone')], ['calculated', t('nta.dist.route.calculated')]]}
      onChange={(_, value) => {
        const source = read(draft, ['distribution', 'sourceReference']) ?? '';
        if (value === 'calculated') {
          change(['distribution'], { method: 'calculated', sourceReference: source });
          if (!system) change(['distributionSystem'], distributionSystemTemplate(draft));
        } else {
          change(['distribution'], { method: 'heated_zone_only_space_heating', sourceReference: source });
        }
      }} />
    <label className="nta-form-check">
      <input type="checkbox" checked={system != null} onChange={(event) => change(['distributionSystem'],
        event.target.checked ? distributionSystemTemplate(draft) : null)} />
      {t('nta.dist.system')}
    </label>
    {system != null && <>
      {route === 'calculated' && <SelectField {...field} path={['distributionSystem', 'designTemperatureClass']} label={t('nta.dist.design')}
        options={DESIGN_CLASSES.map((key) => [key, key.replace('_', '/') + ' °C'])} />}
      <SelectField {...field} path={['distributionSystem', 'installation']} label={t('nta.dist.installation')}
        options={[['individual', t('nta.dist.individual')], ['collective', t('nta.dist.collective')]]} />
      <NumberField {...field} path={['distributionSystem', 'connectedStoreys']} label={t('nta.dist.storeys')} step="1" />
      {route === 'calculated' && <>
        <label data-path="ntaCalculation.distributionSystem.pipeTransmittance.method">{t('nta.dist.psiRoute')}
          <select value={psiCalculated ? 'calculated' : 'forfait'}
            onChange={(event) => change(['distributionSystem', 'pipeTransmittance'], event.target.value === 'calculated'
              ? pipeGeometryTemplate('insulated_in_air')
              : { method: 'forfait', insulation: { state: 'insulated', period: 'before1980_or_unknown' } })}>
            <option value="forfait">{t('nta.dist.psiRoute.forfait')}</option>
            <option value="calculated">{t('nta.dist.psiRoute.calculated')}</option>
          </select>
        </label>
        {psiCalculated
          ? <PipeGeometryFields draft={draft} change={change} base={['distributionSystem', 'pipeTransmittance']} />
          : <>
        <SelectField required {...field} path={['distributionSystem', 'pipeTransmittance', 'insulation', 'state']} label={t('nta.dist.pipes')}
          options={[['insulated', t('nta.vent.insulated')], ['uninsulated', t('nta.vent.uninsulated')], ['unknown', t('nta.form.unknown')]]}
          onChange={(_, value) => change(['distributionSystem', 'pipeTransmittance'], { method: 'forfait', insulation: value === 'insulated'
            ? { state: 'insulated', period: 'before1980_or_unknown' } : { state: value ?? 'unknown' } })} />
        {pipe === 'insulated' && <SelectField {...field} path={['distributionSystem', 'pipeTransmittance', 'insulation', 'period']}
          label={t('nta.dist.pipePeriod')} options={[['from1995', '≥ 1995'], ['from1980_to1995', '1980–1995'], ['before1980_or_unknown', t('nta.dist.before1980')]]} />}
          </>}
        <CheckField {...field} path={['distributionSystem', 'valvesInsulated']} label={t('nta.dist.valves')} />
        <DistributionShell2023Field draft={draft} change={change} />
        <NumberField {...field} path={['distributionSystem', 'actualPipeLengthM']} label={t('nta.dist.pipeLength')} />
        <NumberField {...field} path={['distributionSystem', 'unheatedPipeLengthM']} label={t('nta.dist.unheatedLength')} />
        <NumberField {...field} path={['distributionSystem', 'unheatedReductionFactor']} label={t('nta.dist.unheatedReduction')} optional />
        <CheckField {...field} path={['distributionSystem', 'pipesAlsoForHotWater']} label={t('nta.dist.alsoHotWater')} />
        {collective && <CheckField {...field} path={['distributionSystem', 'collectiveHotWaterDeliverySet']} label={t('nta.dist.deliverySet')} />}
        {/* Table 9.16 note 2: a forfait Ψ already includes buffer losses; a
            buffer vessel is only entered for a collective installation. */}
        {collective && psiCalculated && <BufferVesselFields draft={draft} change={change} />}
      </>}
      <SelectField required {...field} path={['distributionSystem', 'pump', 'method']} label={t('nta.dist.pump')} options={[
        ['included_in_generator_auxiliary', t('nta.dist.pump.included')], ['calculated', t('nta.dist.pump.calculated')],
        ['none_on_site', t('nta.dist.pump.none')]]}
        onChange={(_, value) => change(['distributionSystem', 'pump'], value === 'calculated'
          ? { method: 'calculated', heatMeterPresent: false, sourceReference: '' }
          : value === 'none_on_site' ? { method: 'none_on_site', sourceReference: '' } : { method: 'included_in_generator_auxiliary' })} />
      {pump === 'calculated' && <>
        <CheckField {...field} path={['distributionSystem', 'pump', 'heatMeterPresent']} label={t('nta.dist.heatMeter')} />
        <NumberField {...field} path={['distributionSystem', 'pump', 'maxPipeLengthM']} label={t('nta.dist.pump.maxLength')} optional unit="m" />
        <NumberField {...field} path={['distributionSystem', 'pump', 'designFlowM3PerH']} label={t('nta.dist.pump.designFlow')} optional unit="m³/h" />
        <NumberField {...field} path={['distributionSystem', 'pump', 'energyEfficiencyIndex']} label={t('nta.dist.pump.eei')} optional />
        <NumberField {...field} path={['distributionSystem', 'pump', 'electricPowerKw']} label={t('nta.dist.pump.power')} optional unit="kW" />
        <NumberField {...field} path={['distributionSystem', 'pump', 'onePipeEmitterCount']} label={t('nta.dist.pump.onePipe')} optional step="1" />
      </>}
      {pump && pump !== 'included_in_generator_auxiliary' &&
        <TextField {...field} path={['distributionSystem', 'pump', 'sourceReference']} label={t('nta.form.source')} />}
      <TextField {...field} path={['distributionSystem', 'sourceReference']} label={t('nta.form.source')} />
    </>}
  </>;
}
