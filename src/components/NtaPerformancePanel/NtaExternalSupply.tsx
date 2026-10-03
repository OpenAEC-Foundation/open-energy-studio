import { useI18n } from '../../i18n/i18n';
import {
  annexPGeneratorKindTemplate, annexPGeneratorTemplate, annexPPlotTemplate, annexPRouteTemplate,
} from '../../core/nta/NtaSystemTemplates';
import {
  CheckField, NumberField, read, SelectField, TextField, type Draft, type Path,
} from './NtaFormFields';
import { CalculatedHotWaterStorageFields, PipeDistributionFields } from './NtaAnnexPDetails';
import { PvSystemFields, peakPowerTemplate } from './NtaPvFields';
import { MonthlyValues, SolarCalculatedFields } from './NtaSystemSections';
import { calculatedSolarMethod } from '../../core/nta/NtaSystemTemplates';

// §5.8 / annex P: values for external heat, hot water and cold, a
// collective heat-pump source and area electricity. Each service is either
// forfait (absent), a registered declaration, measured flows (P.6) or the
// calculated route (P.7, P.9), with pipe segments, calculated storage and
// every generator kind of P.6.5.4/P.6.7.4 as structured fields.

interface SectionProps {
  draft: Draft;
  change: (path: Path, value: unknown) => void;
}

type Translate = (key: string) => string;

export type AnnexPFunction = 'heating' | 'hot_water' | 'cooling';

const CARRIERS = ['natural_gas', 'oil', 'electricity', 'biogas', 'biomass_above500_kw', 'waste_incineration', 'biofuel_mix'] as const;

const GENERATOR_KINDS = ['combustion', 'boiler', 'heat_pump', 'chp_without_loss', 'chp_with_loss', 'residual_heat',
  'geothermal', 'solid_biomass_boiler', 'declared', 'compression_chiller', 'free_cooling', 'collective_solar', 'electric_flex',
  'sorption_chiller'] as const;

const TABLE_P5_SOURCES = ['electric_ground', 'electric_outdoor_air', 'electric_groundwater_below15_c', 'electric_surface_water',
  'electric_source15_to20_c', 'electric_source20_to40_c', 'electric_source_at_least40_c', 'gas_ground_or_outdoor_air',
  'gas_groundwater', 'gas_surface_water'] as const;

function list(draft: Draft, path: Path): Draft[] {
  const value = read(draft, path);
  return Array.isArray(value) ? value as Draft[] : [];
}

function RemoveButton({ label, onClick }: { label: string; onClick: () => void }) {
  return <button type="button" className="nta-form-remove" onClick={onClick}>{label}</button>;
}

const options = (t: Translate, prefix: string, keys: readonly string[]): Array<[string, string]> =>
  keys.map((key) => [key, t(`${prefix}.${key}`)]);

/** Table 5.2/5.5 carrier of a generator or measured input. */
function CarrierFields({ draft, change, base, label }: SectionProps & { base: Path; label: string }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const kind = read(draft, [...base, 'kind']);
  return <>
    <SelectField {...field} path={[...base, 'kind']} label={label} options={options(t, 'nta.annexP.carrier', CARRIERS)}
      onChange={(_, value) => change(base, value === 'biofuel_mix' ? { kind: value, biofuelShare: null }
        : value == null ? null : { kind: value })} />
    {kind === 'electricity' &&
      <NumberField {...field} path={[...base, 'directRenewableShare']} label={t('nta.annexP.directRenewableShare')} />}
    {kind === 'biofuel_mix' && <NumberField {...field} path={[...base, 'biofuelShare']} label={t('nta.annexP.biofuelShare')} />}
  </>;
}

/** Table P.6 conversion numbers of a CHP. */
function TableP6Fields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  return <>
    <NumberField {...field} path={[...base, 'electricalPowerKw']} label={t('nta.annexP.p6.power')} />
    <CheckField {...field} path={[...base, 'installedAfter2006']} label={t('nta.annexP.p6.after2006')} />
    <SelectField {...field} path={[...base, 'temperatureLevel']} label={t('nta.annexP.temperatureLevel')} options={[
      ['low', t('nta.annexP.temperatureLevel.low')], ['high', t('nta.annexP.temperatureLevel.high')]]} />
  </>;
}

/** Table P.5 or a declared efficiency of a heat pump. */
function HeatPumpEfficiencyFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const method = read(draft, [...base, 'method']);
  return <>
    <SelectField {...field} path={[...base, 'method']} label={t('nta.annexP.hpEfficiency')} options={[
      ['table_p5', t('nta.annexP.hpEfficiency.tableP5')], ['declared', t('nta.annexP.hpEfficiency.declared')]]}
      onChange={(_, value) => change(base, value === 'declared'
        ? { method: 'declared', value: null, sourceReference: '' }
        : { method: 'table_p5', source: null, supplyTemperatureC: null })} />
    {method === 'declared' ? <>
      <NumberField {...field} path={[...base, 'value']} label={t('nta.annexP.efficiency')} />
      <CheckField {...field} path={[...base, 'sourcePumpIncluded']} label={t('nta.annexP.sourcePumpIncluded')} />
      <TextField {...field} path={[...base, 'sourceReference']} label={t('nta.form.source')} />
    </> : <>
      <SelectField {...field} path={[...base, 'source']} label={t('nta.annexP.p5Source')} options={options(t, 'nta.annexP.p5', TABLE_P5_SOURCES)} />
      <NumberField {...field} path={[...base, 'supplyTemperatureC']} label={t('nta.annexP.supplyTemperature')} />
    </>}
  </>;
}

/** The kind-specific inputs of one annex P generator at `base` (`generators[i].kind`). */
function GeneratorKindFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const kind = read(draft, [...base, 'kind']);
  const at = (...rest: Path): Path => [...base, ...rest];
  switch (kind) {
    case 'combustion':
      return <>
        <CarrierFields draft={draft} change={change} base={at('carrier')} label={t('nta.annexP.carrier')} />
        <NumberField {...field} path={at('efficiency')} label={t('nta.annexP.efficiency')} />
        <TextField {...field} path={at('efficiencyReference')} label={t('nta.form.source')} />
      </>;
    case 'boiler': {
      const method = read(draft, at('efficiency', 'method'));
      return <>
        <CarrierFields draft={draft} change={change} base={at('carrier')} label={t('nta.annexP.carrier')} />
        <SelectField {...field} path={at('efficiency', 'method')} label={t('nta.annexP.boilerEfficiency')} options={[
          ['table_p3', t('nta.annexP.boilerEfficiency.tableP3')], ['full_load', t('nta.annexP.boilerEfficiency.fullLoad')]]}
          onChange={(_, value) => change(at('efficiency'), value === 'full_load'
            ? { method: 'full_load', value: null, outdoorInstallation: false, sourceReference: '' }
            : { method: 'table_p3', boiler: null })} />
        {method === 'full_load' ? <>
          <NumberField {...field} path={at('efficiency', 'value')} label={t('nta.annexP.efficiency')} />
          <CheckField {...field} path={at('efficiency', 'outdoorInstallation')} label={t('nta.annexP.outdoorInstallation')} />
          <TextField {...field} path={at('efficiency', 'sourceReference')} label={t('nta.form.source')} />
        </> : <>
          <SelectField {...field} path={at('efficiency', 'boiler')} label={t('nta.annexP.boilerType')} options={[
            ['conventional', t('nta.annexP.boiler.conventional')], ['vr', 'VR'], ['hr100', 'HR100'], ['hr104', 'HR104'], ['hr107', 'HR107']]} />
          <SelectField {...field} path={at('efficiency', 'temperatureLevel')} label={t('nta.annexP.temperatureLevel')} options={[
            ['low', t('nta.annexP.temperatureLevel.low')], ['high', t('nta.annexP.temperatureLevel.high')]]} />
        </>}
      </>;
    }
    case 'heat_pump':
      return <>
        <HeatPumpEfficiencyFields draft={draft} change={change} base={at('efficiency')} />
        <CarrierFields draft={draft} change={change} base={at('drive')} label={t('nta.annexP.drive')} />
      </>;
    case 'chp_without_loss':
      return <>
        <CarrierFields draft={draft} change={change} base={at('carrier')} label={t('nta.annexP.carrier')} />
        <NumberField {...field} path={at('thermalEfficiency')} label={t('nta.annexP.thermalEfficiency')} />
        <NumberField {...field} path={at('electricalEfficiency')} label={t('nta.annexP.electricalEfficiency')} />
        <TextField {...field} path={at('efficiencyReference')} label={t('nta.form.source')} />
        <label className="nta-form-check">
          <input type="checkbox" checked={read(draft, at('tableP6')) != null} onChange={(event) => change(at('tableP6'),
            event.target.checked ? { electricalPowerKw: null, installedAfter2006: true } : null)} />
          {t('nta.annexP.p6')}
        </label>
        {read(draft, at('tableP6')) != null && <TableP6Fields draft={draft} change={change} base={at('tableP6')} />}
      </>;
    case 'chp_with_loss':
      return <>
        <CarrierFields draft={draft} change={change} base={at('carrier')} label={t('nta.annexP.carrier')} />
        <NumberField {...field} path={at('lossRatio')} label={t('nta.annexP.lossRatio')} />
        <TextField {...field} path={at('lossRatioReference')} label={t('nta.form.source')} />
      </>;
    case 'residual_heat':
      return <>
        <NumberField {...field} path={at('auxiliarySpecific')} label={t('nta.annexP.residualAuxiliary')} />
        <TextField {...field} path={at('auxiliaryReference')} label={t('nta.form.source')} />
      </>;
    case 'geothermal':
      return <>
        <NumberField {...field} path={at('sourceTemperatureC')} label={t('nta.annexP.geoSource')} />
        <NumberField {...field} path={at('returnTemperatureC')} label={t('nta.annexP.geoReturn')} />
      </>;
    case 'solid_biomass_boiler':
      return <>
        <CarrierFields draft={draft} change={change} base={at('carrier')} label={t('nta.annexP.carrier')} />
        <NumberField {...field} path={at('netEfficiency')} label={t('nta.annexP.netEfficiency')} />
        <TextField {...field} path={at('sourceReference')} label={t('nta.form.source')} />
      </>;
    case 'declared':
      return <>
        <NumberField {...field} path={at('primaryFactor')} label={t('nta.annexP.primaryFactor')} />
        <NumberField {...field} path={at('renewableFactor')} label={t('nta.annexP.renewableFactor')} />
        <NumberField {...field} path={at('co2KgPerKwh')} label={t('nta.annexP.co2')} />
        <TextField {...field} path={at('sourceReference')} label={t('nta.form.source')} />
      </>;
    case 'compression_chiller':
      return <>
        <SelectField {...field} path={at('variant')} label={t('nta.annexP.chillerVariant')} options={options(t, 'nta.annexP.chiller', [
          'unspecified', 'high_temperature_emission', 'wet_cooling', 'high_temperature_emission_and_wet_cooling',
          'low_temperature_source', 'high_temperature_emission_and_low_temperature_source'])} />
        <CarrierFields draft={draft} change={change} base={at('drive')} label={t('nta.annexP.drive')} />
      </>;
    case 'free_cooling':
      return <>
        <SelectField {...field} path={at('source')} label={t('nta.annexP.freeSource')} options={options(t, 'nta.annexP.free', [
          'aquifer_storage_before2013', 'aquifer_storage_from2013', 'aquifer_recirculation', 'aquifer_without_heat_use',
          'other_low_temperature_source'])} />
        <CarrierFields draft={draft} change={change} base={at('drive')} label={t('nta.annexP.drive')} />
      </>;
    case 'collective_solar':
      return <CollectiveSolarFields draft={draft} change={change} base={at('contribution')} />;
    case 'electric_flex':
      return <ElectricFlexFields draft={draft} change={change} base={base} />;
    case 'sorption_chiller':
      return <SorptionHeatFields draft={draft} change={change} base={at('heat')} />;
    default:
      return null;
  }
}

/** P.6.5.4.10: the solar contribution of collective collectors, declared or by 13.7.2.2 (P.33). */
function CollectiveSolarFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const method = read(draft, [...base, 'method']);
  const monthly = Array.isArray(read(draft, [...base, 'monthlyKwh']));
  const reference = read(draft, [...base, 'sourceReference']) ?? '';
  return <>
    <SelectField {...field} path={[...base, 'method']} label={t('nta.annexP.solar.method')} options={[
      ['declared', t('nta.annexP.solar.declared')], ['calculated', t('nta.annexP.solar.calculated')]]}
      onChange={(_, value) => {
        if (value === 'calculated') {
          const { collectors, storage, solarType } = calculatedSolarMethod();
          change(base, { method: 'calculated', solarType, collectors, storage, networkSupplyC: null, networkReturnC: null,
            storageAmbientC: null });
        } else {
          change(base, { method: 'declared', annualKwh: null, sourceReference: '' });
        }
      }} />
    {method === 'declared' && <>
      <label className="nta-form-check">
        <input type="checkbox" checked={monthly} onChange={(event) => change(base, event.target.checked
          ? { method: 'declared', monthlyKwh: Array(12).fill(null), sourceReference: reference }
          : { method: 'declared', annualKwh: null, sourceReference: reference })} />
        {t('nta.annexP.solar.monthly')}
      </label>
      {monthly
        ? <MonthlyValues draft={draft} change={change} path={[...base, 'monthlyKwh']} label={t('nta.annexP.solar.monthlyKwh')} />
        : <NumberField {...field} path={[...base, 'annualKwh']} label={t('nta.annexP.solar.annualKwh')} />}
      <TextField {...field} path={[...base, 'sourceReference']} label={t('nta.form.source')} />
    </>}
    {method === 'calculated' && <>
      <SelectField {...field} path={[...base, 'solarType']} label={t('nta.form.solar.type')} options={[
        ['preheater', t('nta.form.solar.type.preheater')], ['integrated_backup', t('nta.form.solar.type.integrated')]]} />
      <SolarCalculatedFields draft={draft} change={change} base={base} />
      <NumberField {...field} path={[...base, 'networkSupplyC']} label={t('nta.annexP.solar.networkSupply')} />
      <NumberField {...field} path={[...base, 'networkReturnC']} label={t('nta.annexP.solar.networkReturn')} />
      <NumberField {...field} path={[...base, 'storageAmbientC']} label={t('nta.annexP.solar.storageAmbient')} />
    </>}
  </>;
}

/** P.6.5.4.11: an electric generator in flex mode (5.8, tables 5.5/5.6). */
function ElectricFlexFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const at = (...rest: Path): Path => [...base, ...rest];
  const generator = read(draft, at('generator', 'kind'));
  return <>
    <SelectField {...field} path={at('generator', 'kind')} label={t('nta.annexP.flex.generator')} options={[
      ['electrode_boiler', t('nta.annexP.flex.electrodeBoiler')], ['heat_pump', t('nta.annexP.flex.heatPump')]]}
      onChange={(_, value) => change(at('generator'), value === 'heat_pump'
        ? { kind: 'heat_pump', efficiency: { method: 'table_p5', source: null, supplyTemperatureC: null } }
        : { kind: 'electrode_boiler' })} />
    {generator === 'heat_pump' ? <HeatPumpEfficiencyFields draft={draft} change={change} base={at('generator', 'efficiency')} /> : <>
      <NumberField {...field} path={at('generator', 'efficiency')} label={t('nta.annexP.flex.electrodeEfficiency')} />
      <TextField {...field} path={at('generator', 'efficiencyReference')} label={t('nta.form.source')} />
    </>}
    <NumberField {...field} path={at('connections')} label={t('nta.annexP.flex.connections')} step="1" />
    <CheckField {...field} path={at('heatBuffer')} label={t('nta.annexP.flex.heatBuffer')} />
    <TextField {...field} path={at('registrationReference')} label={t('nta.annexP.flex.registration')} />
    <NumberField {...field} path={at('flexHeatKwh')} label={t('nta.annexP.flex.heat')} />
    <TextField {...field} path={at('flexReference')} label={t('nta.form.source')} />
    <NumberField {...field} path={at('networkProductionKwh')} label={t('nta.annexP.flex.networkProduction')} />
  </>;
}

/** Table P.10: the heat source of a sorption chiller, collective heat or a CHP (P.26/P.27). */
function SorptionHeatFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const source = read(draft, [...base, 'source']);
  const at = (...rest: Path): Path => [...base, ...rest];
  return <>
    <SelectField {...field} path={at('source')} label={t('nta.annexP.sorption.source')} options={[
      ['collective_heat', t('nta.annexP.sorption.collectiveHeat')], ['chp', t('nta.annexP.sorption.chp')]]}
      onChange={(_, value) => change(base, value === 'chp'
        ? { source: 'chp', carrier: { kind: 'natural_gas' }, tableP6: { electricalPowerKw: null, installedAfter2006: true } }
        : { source: 'collective_heat', primaryFactor: null, co2KgPerKwh: null, sourceReference: '' })} />
    {source === 'chp' ? <>
      <CarrierFields draft={draft} change={change} base={at('carrier')} label={t('nta.annexP.carrier')} />
      <NumberField {...field} path={at('thermalEfficiency')} label={t('nta.annexP.thermalEfficiency')} />
      <NumberField {...field} path={at('electricalEfficiency')} label={t('nta.annexP.electricalEfficiency')} />
      <TextField {...field} path={at('efficiencyReference')} label={t('nta.form.source')} />
      <label className="nta-form-check">
        <input type="checkbox" checked={read(draft, at('tableP6')) != null} onChange={(event) => change(at('tableP6'),
          event.target.checked ? { electricalPowerKw: null, installedAfter2006: true } : null)} />
        {t('nta.annexP.p6')}
      </label>
      {read(draft, at('tableP6')) != null && <TableP6Fields draft={draft} change={change} base={at('tableP6')} />}
    </> : <>
      <NumberField {...field} path={at('primaryFactor')} label={t('nta.annexP.primaryFactor')} />
      <NumberField {...field} path={at('co2KgPerKwh')} label={t('nta.annexP.co2')} />
      <TextField {...field} path={at('sourceReference')} label={t('nta.form.source')} />
    </>}
  </>;
}

/** P.6.5–P.6.7 generators of the calculated route. */
function GeneratorsFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const generators = list(draft, base);
  const kindOptions: Array<[string, string]> = GENERATOR_KINDS.map((key) => [key, t(`nta.annexP.kind.${key}`)]);
  return <>
    {generators.map((_, index) => {
      const at = (...rest: Path): Path => [...base, index, ...rest];
      return <fieldset key={index} className="nta-form-row">
        <legend>{t('nta.annexP.generator')} {index + 1}</legend>
        <TextField {...field} path={at('id')} label={t('nta.annexP.generatorId')} />
        <SelectField {...field} path={at('kind', 'kind')} label={t('nta.annexP.generatorKind')} options={kindOptions}
          onChange={(_, value) => change(at('kind'), annexPGeneratorKindTemplate(String(value ?? 'combustion')))} />
        <GeneratorKindFields draft={draft} change={change} base={at('kind')} />
        <NumberField {...field} path={at('nominalPowerKw')} label={t('nta.annexP.nominalPower')} />
        <NumberField {...field} path={at('energyFraction')} label={t('nta.annexP.energyFraction')} />
        <NumberField {...field} path={at('priority')} label={t('nta.annexP.priority')} step="1" />
        <RemoveButton label={t('nta.form.remove')} onClick={() => change(base, generators.filter((__, other) => other !== index))} />
      </fieldset>;
    })}
    <button type="button" onClick={() => change(base, [...generators, annexPGeneratorTemplate(generators.length)])}>
      {t('nta.annexP.addGenerator')}
    </button>
  </>;
}

const HOT_WATER_FORFAIT = ['dwelling_low_temperature', 'dwelling_high_temperature', 'assembly_with_alcohol', 'assembly', 'cell',
  'healthcare_clinical', 'healthcare_non_clinical', 'office', 'lodging', 'education', 'sport', 'retail'] as const;

/** P.72–P.83: connected plots with supplied flows or the forfait tables P.14/P.15. */
function PlotsFields({ draft, change, base, fn }: SectionProps & { base: Path; fn: AnnexPFunction }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const plots = list(draft, [...base, 'plots']);
  const flow = fn === 'hot_water' ? 'hotWater' : fn;
  return <>
    {plots.map((_, index) => {
      const at = (...rest: Path): Path => [...base, 'plots', index, ...rest];
      return <fieldset key={index} className="nta-form-row">
        <legend>{t('nta.annexP.plot')} {index + 1}</legend>
        <TextField {...field} path={at('id')} label={t('nta.annexP.plotId')} />
        <NumberField {...field} path={at('usableAreaM2')} label={t('nta.form.area')} />
        <NumberField {...field} path={at(flow, 'annualKwh')} label={t('nta.annexP.plotAnnual')} />
        {fn === 'heating' && <SelectField {...field} path={at('heatingForfait')} label={t('nta.annexP.heatingForfait')}
          options={options(t, 'nta.annexP.dwelling', ['apartment', 'terraced_or_utility', 'corner_or_semi_detached', 'detached'])} />}
        {fn === 'hot_water' && <>
          <SelectField {...field} path={at('hotWaterForfait')} label={t('nta.annexP.hotWaterForfait')}
            options={options(t, 'nta.annexP.hw', HOT_WATER_FORFAIT)} />
          <CheckField {...field} path={at('hotWaterViaDeliverySet')} label={t('nta.annexP.deliverySet')} />
        </>}
        <TextField {...field} path={at('sourceReference')} label={t('nta.form.source')} />
        <RemoveButton label={t('nta.form.remove')}
          onClick={() => change([...base, 'plots'], plots.filter((__, other) => other !== index))} />
      </fieldset>;
    })}
    <button type="button" onClick={() => change([...base, 'plots'], [...plots, annexPPlotTemplate(plots.length)])}>
      {t('nta.annexP.addPlot')}
    </button>
  </>;
}

/** P.6.4 / P.10–P.18 distribution of the calculated route. */
function DistributionFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const method = read(draft, [...base, 'method']);
  return <>
    <SelectField {...field} path={[...base, 'method']} label={t('nta.annexP.distribution')}
      options={options(t, 'nta.annexP.distribution', ['flows', 'small_system_forfait', 'small_cold_forfait', 'pipes'])}
      onChange={(_, value) => change(base,
        value === 'small_system_forfait' ? { method: value, connections: null, connectionType: 'ground_bound' }
          : value === 'small_cold_forfait' ? { method: value, supplyBelow10C: false }
            : value === 'pipes' ? { method: value, segments: [], sourceReference: '' }
              : { method: 'flows', inputKwh: null, lossKwh: null, sourceReference: '' })} />
    {method === 'flows' && <>
      <NumberField {...field} path={[...base, 'inputKwh']} label={t('nta.annexP.distributionInput')} />
      <NumberField {...field} path={[...base, 'lossKwh']} label={t('nta.annexP.distributionLoss')} />
      <TextField {...field} path={[...base, 'sourceReference']} label={t('nta.form.source')} />
    </>}
    {method === 'small_system_forfait' && <>
      <NumberField {...field} path={[...base, 'connections']} label={t('nta.annexP.connections')} step="1" />
      <SelectField {...field} path={[...base, 'connectionType']} label={t('nta.annexP.connectionType')} options={[
        ['ground_bound', t('nta.annexP.connectionType.ground')], ['within_building', t('nta.annexP.connectionType.building')]]} />
      <SelectField {...field} path={[...base, 'designTemperature']} label={t('nta.annexP.designTemperature')} options={[
        ['t90_to60', '90/60'], ['t90_to50', '90/50'], ['t70_to40', '70/40'], ['t50_to40', '50/40'], ['t35_to25', '35/25']]} />
      <NumberField {...field} path={[...base, 'otherLossKwh']} label={t('nta.annexP.otherLoss')} />
    </>}
    {method === 'small_cold_forfait' &&
      <CheckField {...field} path={[...base, 'supplyBelow10C']} label={t('nta.annexP.supplyBelow10')} />}
    {method === 'pipes' && <PipeDistributionFields draft={draft} change={change} base={base} />}
  </>;
}

/** P.56–P.70: auxiliary energy of the calculated route. */
function AuxiliaryFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const present = read(draft, [...base, 'auxiliary']) != null;
  const method = read(draft, [...base, 'auxiliary', 'distribution', 'method']);
  const pumps = (read(draft, [...base, 'auxiliary', 'distribution', 'pumpPowersW']) as Array<number | null> | undefined) ?? [];
  return <>
    <NumberField {...field} path={[...base, 'auxiliaryElectricityKwh']} label={t('nta.annexP.auxiliaryTotal')} />
    <label className="nta-form-check">
      <input type="checkbox" checked={present} onChange={(event) => change([...base, 'auxiliary'], event.target.checked
        ? { distribution: { method: 'forfait' } } : null)} />
      {t('nta.annexP.auxiliary')}
    </label>
    {present && <>
      <SelectField {...field} path={[...base, 'auxiliary', 'distribution', 'method']} label={t('nta.annexP.auxiliaryMethod')}
        options={options(t, 'nta.annexP.aux', ['forfait', 'pumps', 'pumps_monthly'])}
        onChange={(_, value) => change([...base, 'auxiliary', 'distribution'], value === 'pumps' || value === 'pumps_monthly'
          ? { method: value, pumpPowersW: [], sourceReference: '' } : { method: 'forfait' })} />
      {method === 'forfait' && <>
        <SelectField {...field} path={[...base, 'auxiliary', 'distribution', 'network']} label={t('nta.annexP.network')}
          options={options(t, 'nta.annexP.network', ['primary_and_secondary', 'primary', 'secondary', 'small_system'])} />
        <NumberField {...field} path={[...base, 'auxiliary', 'distribution', 'farthestDistanceKm']} label={t('nta.annexP.farthest')} />
      </>}
      {(method === 'pumps' || method === 'pumps_monthly') && <>
        {pumps.map((_, index) => <NumberField key={index} {...field}
          path={[...base, 'auxiliary', 'distribution', 'pumpPowersW', index]} label={`${t('nta.annexP.pumpPower')} ${index + 1}`} />)}
        <button type="button" onClick={() => change([...base, 'auxiliary', 'distribution', 'pumpPowersW'], [...pumps, null])}>
          {t('nta.annexP.addPump')}
        </button>
        {method === 'pumps' && <NumberField {...field} path={[...base, 'auxiliary', 'distribution', 'operatingHours']}
          label={t('nta.annexP.operatingHours')} />}
        <TextField {...field} path={[...base, 'auxiliary', 'distribution', 'sourceReference']} label={t('nta.form.source')} />
      </>}
      <NumberField {...field} path={[...base, 'auxiliary', 'solarKwh']} label={t('nta.annexP.solarAuxiliary')} />
    </>}
    <NumberField {...field} path={[...base, 'auxiliaryRenewableShare']} label={t('nta.annexP.auxiliaryRenewable')} />
  </>;
}

/** P.34/P.35 η_WD;gen;sto of a hot-water system. */
function HotWaterStorageFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const method = read(draft, [...base, 'method']);
  return <>
    <SelectField {...field} path={[...base, 'method']} label={t('nta.annexP.hwStorage')}
      options={options(t, 'nta.annexP.hwStorage', ['forfait', 'losses', 'calculated'])}
      onChange={(_, value) => change(base, value === 'losses'
        ? { method: value, storageLossKwh: null, pipeLossKwh: null, sourceReference: '' }
        : value === 'calculated' ? { method: value, vessels: [], sourceReference: '' }
          : { method: 'forfait', insulation: null })} />
    {method === 'forfait' && <SelectField {...field} path={[...base, 'insulation']} label={t('nta.annexP.insulation')} options={[
      ['at_least20_mm', '≥ 20 mm'], ['at_least10_mm', '≥ 10 mm'], ['none', t('nta.annexP.insulation.none')]]} />}
    {method === 'losses' && <>
      <NumberField {...field} path={[...base, 'storageLossKwh']} label={t('nta.annexP.storageLoss')} />
      <NumberField {...field} path={[...base, 'pipeLossKwh']} label={t('nta.annexP.pipeLoss')} />
      <TextField {...field} path={[...base, 'sourceReference']} label={t('nta.form.source')} />
    </>}
    {method === 'calculated' && <CalculatedHotWaterStorageFields draft={draft} change={change} base={base} />}
  </>;
}

/** One annex P route at `base`: forfait (absent), declared, measured or calculated. */
export function AnnexPRouteFields({ draft, change, base, fn, label }: SectionProps & {
  base: Path; fn: AnnexPFunction; label: string;
}) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const method = read(draft, [...base, 'method']);
  const at = (...rest: Path): Path => [...base, ...rest];
  const inputs = list(draft, at('inputs'));
  const areaDemand = read(draft, at('areaDemand')) != null;
  return <>
    <label>{label}
      <select value={typeof method === 'string' ? method : ''}
        onChange={(event) => change(base, event.target.value === '' ? null : annexPRouteTemplate(event.target.value, fn))}>
        <option value="">{t('nta.annexP.route.forfait')}</option>
        <option value="declared">{t('nta.annexP.route.declared')}</option>
        <option value="measured">{t('nta.annexP.route.measured')}</option>
        <option value="calculated">{t('nta.annexP.route.calculated')}</option>
      </select>
    </label>
    {method === 'declared' && <>
      <NumberField {...field} path={at('primaryFactor')} label={t('nta.annexP.primaryFactor')} />
      <NumberField {...field} path={at('renewableFactor')} label={t('nta.annexP.renewableFactor')} />
      <NumberField {...field} path={at('co2KgPerKwh')} label={t('nta.annexP.co2')} />
      <TextField {...field} path={at('declarationReference')} label={t('nta.annexP.declarationReference')} />
      <CheckField {...field} path={at('measuredOnly')} label={t('nta.annexP.measuredOnly')} />
    </>}
    {method === 'measured' && <>
      <NumberField {...field} path={at('deliveredKwh')} label={t('nta.annexP.delivered')} />
      {inputs.map((_, index) => <div key={index} className="nta-form-row">
        <CarrierFields draft={draft} change={change} base={at('inputs', index, 'carrier')} label={t('nta.annexP.carrier')} />
        <NumberField {...field} path={at('inputs', index, 'kwh')} label={t('nta.annexP.inputKwh')} />
        <NumberField {...field} path={at('inputs', index, 'chpLossElectrical')} label={t('nta.annexP.chpLoss')} />
        <RemoveButton label={t('nta.form.remove')} onClick={() => change(at('inputs'), inputs.filter((__, other) => other !== index))} />
      </div>)}
      <button type="button" onClick={() => change(at('inputs'), [...inputs, { carrier: { kind: 'natural_gas' }, kwh: null }])}>
        {t('nta.annexP.addInput')}
      </button>
      <NumberField {...field} path={at('exportedElectricityKwh')} label={t('nta.annexP.exported')} />
      <NumberField {...field} path={at('renewableFactor')} label={t('nta.annexP.renewableFactor')} />
      <TextField {...field} path={at('sourceReference')} label={t('nta.form.source')} />
    </>}
    {method === 'calculated' && <>
      <NumberField {...field} path={at('deliveredKwh')} label={t('nta.annexP.delivered')} />
      <label className="nta-form-check">
        <input type="checkbox" checked={areaDemand} onChange={(event) => change(at('areaDemand'), event.target.checked
          ? { plots: [annexPPlotTemplate(0)] } : null)} />
        {t('nta.annexP.areaDemand')}
      </label>
      {areaDemand && <PlotsFields draft={draft} change={change} base={at('areaDemand')} fn={fn} />}
      <DistributionFields draft={draft} change={change} base={at('distribution')} />
      <GeneratorsFields draft={draft} change={change} base={at('generators')} />
      <NumberField {...field} path={at('referencePowerKw')} label={t('nta.annexP.referencePower')} />
      <AuxiliaryFields draft={draft} change={change} base={base} />
      {fn === 'hot_water' && <HotWaterStorageFields draft={draft} change={change} base={at('hotWaterStorage')} />}
      <TextField {...field} path={at('sourceReference')} label={t('nta.form.source')} />
    </>}
  </>;
}

/** §5.8 / annex P: the external-supply block of the NTA input. */
export function ExternalSupplyFields({ draft, change }: SectionProps) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const base: Path = ['externalSupply'];
  const source = read(draft, [...base, 'collectiveHeatPumpSource']) != null;
  const area = list(draft, [...base, 'areaElectricity']);
  return <div className="nta-form-subsection">
    <AnnexPRouteFields draft={draft} change={change} base={[...base, 'heating']} fn="heating" label={t('nta.annexP.heating')} />
    <AnnexPRouteFields draft={draft} change={change} base={[...base, 'hotWater']} fn="hot_water" label={t('nta.annexP.hotWater')} />
    <AnnexPRouteFields draft={draft} change={change} base={[...base, 'cooling']} fn="cooling" label={t('nta.annexP.cooling')} />
    <label className="nta-form-check">
      <input type="checkbox" checked={source} onChange={(event) => change([...base, 'collectiveHeatPumpSource'], event.target.checked
        ? { temperatureClass: null, supplierReference: '' } : null)} />
      {t('nta.annexP.collectiveSource')}
    </label>
    {source && <>
      <SelectField {...field} path={[...base, 'collectiveHeatPumpSource', 'temperatureClass']} label={t('nta.annexP.sourceClass')} options={[
        ['below20_c', t('nta.annexP.sourceClass.below20')],
        ['at_least20_c_or_surface_water_or_unknown', t('nta.annexP.sourceClass.atLeast20')]]} />
      <TextField {...field} path={[...base, 'collectiveHeatPumpSource', 'supplierReference']} label={t('nta.annexP.supplierReference')} />
      <AnnexPRouteFields draft={draft} change={change} base={[...base, 'collectiveHeatPumpSource', 'annexP']} fn="heating"
        label={t('nta.annexP.sourceRoute')} />
    </>}
    {area.map((item, index) => <fieldset key={index} className="nta-form-row">
      <legend>{t('nta.annexP.areaElectricity')} {index + 1}</legend>
      {item.kind === 'declared' ? <>
        <TextField {...field} path={[...base, 'areaElectricity', index, 'id']} label={t('nta.annexP.generatorId')} />
        <NumberField {...field} path={[...base, 'areaElectricity', index, 'annualKwh']} label={t('nta.annexP.annualKwh')} />
        <TextField {...field} path={[...base, 'areaElectricity', index, 'sourceReference']} label={t('nta.form.source')} />
      </> : <PvSystemFields draft={draft} change={change} base={[...base, 'areaElectricity', index]}
        label={String(item.id ?? '')} />}
      <RemoveButton label={t('nta.form.remove')}
        onClick={() => change([...base, 'areaElectricity'], area.filter((__, other) => other !== index))} />
    </fieldset>)}
    <button type="button" onClick={() => change([...base, 'areaElectricity'],
      [...area, { kind: 'declared', id: `gebied-${area.length + 1}`, annualKwh: null, sourceReference: '' }])}>
      {t('nta.annexP.addAreaDeclared')}
    </button>
    <button type="button" onClick={() => change([...base, 'areaElectricity'],
      [...area, {
        kind: 'pv', id: `gebied-pv-${area.length + 1}`, peakPower: peakPowerTemplate('panels'), azimuthDeg: null, tiltDeg: null,
        mounting: 'unknown', obstructionFactors: [null], sourceReference: '',
      }])}>
      {t('nta.annexP.addAreaPv')}
    </button>
    <p className="nta-form-note">{t('nta.form.externalSupplyNote')}</p>
  </div>;
}
