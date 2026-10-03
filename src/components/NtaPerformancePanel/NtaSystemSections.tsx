import { useI18n } from '../../i18n/i18n';
import type { IProject } from '../../core/energy/types';
import {
  additionalHotWaterGeneratorTemplate, calculatedSolarMethod, collectorObstructionTemplate, coolingPerformanceTemplate,
  chpClassTemplate, declaredShareTemplate, en14825PointTemplate, exhaustAirUseTemplate, hotWaterGeneratorTemplate,
  hotWaterStorageTemplate, microChpStorageTemplate, microChpTemplate, preferredGeneratorTemplate, regenerationTemplate, solarRegenerationTemplate, solarWaterHeaterTemplate,
  spaceGeneratorTemplate, storageLossTemplate, testedSolarMethod, windowObstructionTemplate,
} from '../../core/nta/NtaSystemTemplates';
import {
  CheckField, NumberField, read, SelectField, TextField, type Draft, type Path,
} from './NtaFormFields';

// System inputs of the NTA form that share a base path: space-heating
// generators (also nested in `multiple`), hot-water generators, solar water
// heaters, cooling ratings and window obstruction.

interface SectionProps {
  draft: Draft;
  change: (path: Path, value: unknown) => void;
}

type Translate = (key: string) => string;

const ORIENTATIONS = ['north', 'north_east', 'east', 'south_east', 'south', 'south_west', 'west', 'north_west'];

function list(draft: Draft, path: Path): Draft[] {
  const value = read(draft, path);
  return Array.isArray(value) ? value as Draft[] : [];
}

function RemoveButton({ label, onClick }: { label: string; onClick: () => void }) {
  return <button type="button" className="nta-form-remove" onClick={onClick}>{label}</button>;
}

/** One space-heating generator at `base`; `allowMultiple` offers the 9.6.1 split. */
export function SpaceGeneratorFields({ draft, change, base, project, allowMultiple = false }: SectionProps & {
  base: Path; project: IProject; allowMultiple?: boolean;
}) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const at = (...rest: Path): Path => [...base, ...rest];
  const kind = read(draft, at('kind'));
  const otherAux = ['external_heat', 'electric_resistance', 'biomass', 'chp', 'gas_heat_pump'].includes(String(kind));
  return <>
    <label>{t('nta.form.generatorKind')}
      <select value={typeof kind === 'string' ? kind : ''} onChange={(event) => {
        const template = spaceGeneratorTemplate(event.target.value, project);
        if (template) change(base, template);
      }}>
        <option value="gas_boiler">{t('nta.form.generator.boiler')}</option>
        <option value="external_heat">{t('nta.form.generator.external')}</option>
        <option value="heat_pump_forfait">{t('nta.form.generator.heatPump')}</option>
        <option value="electric_resistance">{t('nta.form.generator.electric')}</option>
        <option value="biomass">{t('nta.form.generator.biomass')}</option>
        <option value="chp">{t('nta.form.generator.chp')}</option>
        <option value="gas_heat_pump">{t('nta.form.generator.gasHeatPump')}</option>
        {allowMultiple && <option value="multiple">{t('nta.form.generator.multiple')}</option>}
        {kind === 'hybrid_heat_pump' && <option value="hybrid_heat_pump">{t('nta.form.generator.hybrid')}</option>}
      </select>
    </label>
    {kind === 'external_heat' && <>
      <TextField {...field} path={at('supplierReference')} label={t('nta.form.externalSource')} />
      <p className="nta-form-note">{t('nta.form.externalNote')}</p>
    </>}
    {kind === 'electric_resistance' && <TextField {...field} path={at('equipmentReference')} label={t('nta.form.source')} />}
    {kind === 'chp' && <ChpClassFields draft={draft} change={change} base={base} lowTemperature />}
    {kind === 'gas_heat_pump' && <>
      <SelectField {...field} path={at('table')} label={t('nta.form.gasHp.table')} options={[
        ['residential_at_most25_kw', t('nta.form.gasHp.table.residential')],
        ['utility_collective_or_above25_kw', t('nta.form.gasHp.table.utility')]]} />
      <SelectField {...field} path={at('source')} label={t('nta.form.gasHp.source')} options={[
        ['outdoor_air', t('nta.form.gasHp.source.outdoorAir')], ['exhaust_air', t('nta.form.gasHp.source.exhaustAir')],
        ['ground', t('nta.form.gasHp.source.ground')], ['groundwater', t('nta.form.gasHp.source.groundwater')],
        ['surface_water', t('nta.form.gasHp.source.surfaceWater')]]} />
      <NumberField {...field} path={at('designSupplyTemperatureC')} label={t('nta.form.gasHp.supplyTemperature')} />
      <NumberField {...field} path={at('sourceCorrectionFactor')} label={t('nta.form.gasHp.sourceCorrection')} step="0.01" />
      <TextField {...field} path={at('equipmentReference')} label={t('nta.form.boilerEquipmentSource')} />
    </>}
    {kind === 'biomass' && <>
      <SelectField {...field} path={at('appliance')} label={t('nta.form.biomassAppliance')} options={[
        ['freestanding_wood_stove', t('nta.form.biomass.stove')], ['insert_stove', t('nta.form.biomass.insert')],
        ['pellet_stove', t('nta.form.biomass.pellet')], ['accumulating_stove', t('nta.form.biomass.accumulating')],
        ['central_boiler', t('nta.form.biomass.boiler')]]} />
      <SelectField {...field} path={at('location')} label={t('nta.form.boilerLocation')} options={[
        ['inside_thermal_boundary', t('nta.form.boiler.inside')], ['outside_thermal_boundary', t('nta.form.boiler.outside')]]} />
      <CheckField {...field} path={at('annexRCompliantAtMost500Kw')} label={t('nta.form.biomassAnnexR')} />
      <TextField {...field} path={at('annexRReference')} label={t('nta.form.biomassAnnexRSource')} />
      <TextField {...field} path={at('equipmentReference')} label={t('nta.form.boilerEquipmentSource')} />
      <CheckField {...field} path={at('soleHeatingInServedRooms')} label={t('nta.form.biomassSoleHeating')} />
      <CheckField {...field} path={at('automaticFuelFeed')} label={t('nta.form.biomassAutomaticFeed')} />
    </>}
    {otherAux && <>
      <NumberField {...field} path={at('auxiliary', 'electricallyConnectedDevices')} label={t('nta.form.auxDevices')} step="1" />
      <NumberField {...field} path={at('auxiliary', 'nominalPowerKw')} label={t('nta.form.auxNominalPower')} />
      <TextField {...field} path={at('auxiliary', 'sourceReference')} label={t('nta.form.auxSource')} />
      <p className="nta-form-note">{t('nta.form.distributionSystemNote')}</p>
    </>}
    {kind === 'gas_boiler' && <>
      <SelectField {...field} path={at('boiler', 'kind')} label={t('nta.form.boilerKind')} options={[
        ['hr107', 'HR107'], ['hr104', 'HR104'], ['hr100', 'HR100'], ['vr', 'VR'], ['conventional', t('nta.form.boiler.conventional')]]} />
      <SelectField {...field} path={at('boiler', 'location')} label={t('nta.form.boilerLocation')} options={[
        ['inside_thermal_boundary', t('nta.form.boiler.inside')], ['outside_thermal_boundary', t('nta.form.boiler.outside')]]} />
      <NumberField {...field} path={at('boiler', 'averageDesignEmissionTemperatureC')} label={t('nta.form.boilerTemperature')} />
      <SelectField {...field} path={at('boiler', 'emissionCircuit')} label={t('nta.form.boilerCircuit')} options={[
        ['direct', t('nta.form.boiler.direct')], ['mixing_with_return_limit', t('nta.form.boiler.mixingLimit')],
        ['mixing_without_return_limit', t('nta.form.boiler.mixingNoLimit')]]} />
      <TextField {...field} path={at('boiler', 'equipmentReference')} label={t('nta.form.boilerEquipmentSource')} />
      <TextField {...field} path={at('boiler', 'locationReference')} label={t('nta.form.boilerLocationSource')} />
      <TextField {...field} path={at('boiler', 'temperatureAndCircuitReference')} label={t('nta.form.boilerTemperatureSource')} />
    </>}
    {(kind === 'heat_pump_forfait' || kind === 'hybrid_heat_pump') && <p className="nta-form-note">{t('nta.form.heatPumpNote')}</p>}
    {kind === 'heat_pump_forfait' && <RegenerationFields draft={draft} change={change} base={at('regeneration')} />}
    {kind === 'multiple' && <MultipleGeneratorFields draft={draft} change={change} base={base} project={project} />}
  </>;
}

/** Annex V regeneration of an individual ground source (free cooling, solar collectors). */
function RegenerationFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const present = read(draft, base) != null;
  const solar = list(draft, [...base, 'solar']);
  return <>
    <label className="nta-form-check">
      <input type="checkbox" checked={present}
        onChange={(event) => change(base, event.target.checked ? regenerationTemplate() : undefined)} />
      {t('nta.form.regeneration')}
    </label>
    {present && <>
      <CheckField {...field} path={[...base, 'freeCoolingFromSource']} label={t('nta.form.regeneration.freeCooling')} />
      <TextField {...field} path={[...base, 'sourceReference']} label={t('nta.form.source')} />
      {solar.map((_, index) => <div key={index} className="nta-form-row">
        <strong>{t('nta.form.regeneration.solar')} {index + 1}</strong>
        <NumberField {...field} path={[...base, 'solar', index, 'collectorAreaM2']} label={t('nta.form.regeneration.area')} />
        <NumberField {...field} path={[...base, 'solar', index, 'azimuthDeg']} label={t('nta.form.regeneration.azimuth')} />
        <NumberField {...field} path={[...base, 'solar', index, 'tiltDeg']} label={t('nta.form.regeneration.tilt')} />
        <NumberField {...field} path={[...base, 'solar', index, 'declaredEfficiency']} label={t('nta.form.regeneration.efficiency')} />
        <TextField {...field} path={[...base, 'solar', index, 'sourceReference']} label={t('nta.form.source')} />
        <RemoveButton label={t('nta.form.remove')} onClick={() => change([...base, 'solar'], solar.filter((_, item) => item !== index))} />
      </div>)}
      <button type="button" className="btn" onClick={() => change([...base, 'solar'], [...solar, solarRegenerationTemplate()])}>
        {t('nta.form.regeneration.addSolar')}
      </button>
    </>}
  </>;
}

/** Building CHP: table 9.31 class (method 2) or measured micro-CHP (method 1, 9.6.6.2). */
function ChpClassFields({ draft, change, base, lowTemperature = false }: SectionProps & { base: Path; lowTemperature?: boolean }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const method1 = read(draft, [...base, 'method1']) != null;
  const setMethod = (measured: boolean) => change(base, {
    ...(read(draft, base) as Draft),
    chp: measured ? null : chpClassTemplate(),
    method1: measured ? microChpTemplate() : null,
  });
  return <>
    <label>{t('nta.form.chp.method')}
      <select value={method1 ? 'method1' : 'method2'} onChange={(event) => setMethod(event.target.value === 'method1')}>
        <option value="method2">{t('nta.form.chp.method2')}</option>
        <option value="method1">{t('nta.form.chp.method1')}</option>
      </select>
    </label>
    {method1
      ? <MicroChpFields draft={draft} change={change} base={[...base, 'method1']} />
      : <ChpTableFields draft={draft} change={change} base={base} lowTemperature={lowTemperature} />}
    <TextField {...field} path={[...base, 'equipmentReference']} label={t('nta.form.boilerEquipmentSource')} />
    <p className="nta-form-note">{t('nta.form.chp.note')}</p>
  </>;
}

/** 9.6.6.2 micro-CHP test values (NEN-EN 50465), method 1. */
function MicroChpFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const at = (...rest: Path): Path => [...base, ...rest];
  const storage = read(draft, at('storage'));
  const point = (which: 'fullLoad' | 'chpOnly') => <fieldset key={which} className="nta-form-row">
    <legend>{t(`nta.form.microChp.${which}`)}</legend>
    <NumberField {...field} path={at(which, 'thermalPowerKw')} label={t('nta.form.microChp.thermalPower')} />
    <NumberField {...field} path={at(which, 'electricPowerKw')} label={t('nta.form.microChp.electricPower')} />
    <NumberField {...field} path={at(which, 'thermalEfficiency')} label={t('nta.form.microChp.thermalEfficiency')} />
    <NumberField {...field} path={at(which, 'electricEfficiency')} label={t('nta.form.microChp.electricEfficiency')} />
    <NumberField {...field} path={at(which, 'auxiliaryPowerKw')} label={t('nta.form.microChp.auxiliaryPower')} />
  </fieldset>;
  return <div className="nta-form-subsection">
    <SelectField {...field} path={at('kind')} label={t('nta.form.microChp.kind')} options={[
      ['stirling_engine', t('nta.form.microChp.kind.stirling')], ['pem_fuel_cell', t('nta.form.microChp.kind.pem')],
      ['solid_oxide_fuel_cell', t('nta.form.microChp.kind.sofc')], ['gas_engine', t('nta.form.microChp.kind.gasEngine')],
      ['diesel_engine', t('nta.form.microChp.kind.diesel')], ['micro_turbine', t('nta.form.microChp.kind.turbine')],
      ['organic_rankine_cycle', t('nta.form.microChp.kind.orc')]]} />
    <SelectField {...field} path={at('fuel')} label={t('nta.form.microChp.fuel')} options={[
      ['natural_gas', t('nta.form.microChp.fuel.gas')], ['oil', t('nta.form.microChp.fuel.oil')]]} />
    <SelectField {...field} path={at('location')} label={t('nta.form.microChp.location')} options={[
      ['heated_space', t('nta.form.microChp.location.heated')], ['unheated_space', t('nta.form.microChp.location.unheated')],
      ['installation_room', t('nta.form.microChp.location.room')], ['outdoors', t('nta.form.microChp.location.outdoors')]]} />
    <SelectField {...field} path={at('hydraulics')} label={t('nta.form.microChp.hydraulics')} options={[
      ['direct', t('nta.form.microChp.hydraulics.direct')], ['decoupled', t('nta.form.microChp.hydraulics.decoupled')],
      ['condensation_pump', t('nta.form.microChp.hydraulics.condensationPump')],
      ['heat_exchanger', t('nta.form.microChp.hydraulics.heatExchanger')]]} />
    {point('fullLoad')}
    {point('chpOnly')}
    <NumberField {...field} path={at('standbyLossKw')} label={t('nta.form.microChp.standbyLoss')} />
    <NumberField {...field} path={at('pilotKw')} label={t('nta.form.microChp.pilot')} />
    <NumberField {...field} path={at('standbyElectricKw')} label={t('nta.form.microChp.standbyElectric')} />
    <NumberField {...field} path={at('standbyAuxiliaryKw')} label={t('nta.form.microChp.standbyAuxiliary')} />
    <CheckField {...field} path={at('netProductionMeasured')} label={t('nta.form.microChp.netProduction')} />
    <label className="nta-form-check">
      <input type="checkbox" checked={storage != null}
        onChange={(event) => change(at('storage'), event.target.checked ? microChpStorageTemplate() : null)} />
      {t('nta.form.microChp.storage')}
    </label>
    {storage != null && <>
      <NumberField {...field} path={at('storage', 'lossWPerK')} label={t('nta.form.microChp.storageLoss')} />
      <NumberField {...field} path={at('storage', 'setTemperatureC')} label={t('nta.form.microChp.storageSet')} />
      <NumberField {...field} path={at('storage', 'chargingAuxiliaryW')} label={t('nta.form.microChp.storageCharging')} />
      <TextField {...field} path={at('storage', 'sourceReference')} label={t('nta.form.source')} />
    </>}
    <TextField {...field} path={at('testReportReference')} label={t('nta.form.microChp.testReport')} />
  </div>;
}

function ChpTableFields({ draft, change, base, lowTemperature }: SectionProps & { base: Path; lowTemperature: boolean }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  return <>
    <NumberField {...field} path={[...base, 'chp', 'powerKw']} label={t('nta.form.chp.power')} />
    <CheckField {...field} path={[...base, 'chp', 'builtAfter2006']} label={t('nta.form.chp.after2006')} />
    <CheckField {...field} path={[...base, 'chp', 'hreDeclared']} label={t('nta.form.chp.hre')} />
    {lowTemperature && <CheckField {...field} path={[...base, 'chp', 'lowTemperature']} label={t('nta.form.chp.lowTemperature')} />}
  </>;
}

/** 9.6.1: generators with preference and nominal power (9.56–9.60, table 9.23). */
function MultipleGeneratorFields({ draft, change, base, project }: SectionProps & { base: Path; project: IProject }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const parts = list(draft, [...base, 'generators']);
  return <div className="nta-form-subsection">
    <p className="nta-form-note">{t('nta.form.multiple.note')}</p>
    {parts.map((_, index) => <fieldset key={index} className="nta-form-row">
      <legend>{t('nta.form.multiple.generator')} {index + 1}</legend>
      <NumberField {...field} path={[...base, 'generators', index, 'preference']} label={t('nta.form.multiple.preference')} step="1" />
      <NumberField {...field} path={[...base, 'generators', index, 'nominalPowerKw']} label={t('nta.form.multiple.power')} />
      <SpaceGeneratorFields draft={draft} change={change} base={[...base, 'generators', index, 'generator']} project={project} />
      {parts.length > 2 && <RemoveButton label={t('nta.form.remove')}
        onClick={() => change([...base, 'generators'], parts.filter((__, other) => other !== index))} />}
    </fieldset>)}
    <button type="button" onClick={() => change([...base, 'generators'], [...parts, preferredGeneratorTemplate(parts, project)])}>
      {t('nta.form.multiple.add')}
    </button>
    <CheckField {...field} path={[...base, 'addedPreferredGenerator']} label={t('nta.form.multiple.added')} />
    <TextField {...field} path={[...base, 'sourceReference']} label={t('nta.form.source')} />
  </div>;
}

function hotWaterGeneratorOptions(t: Translate): Array<[string, string]> {
  return [['gas_appliance', t('nta.form.dhwGen.gas')], ['heat_pump', t('nta.form.dhwGen.heatPump')],
    ['electric_instantaneous', t('nta.form.dhwGen.instantaneous')], ['electric_boiler', t('nta.form.dhwGen.electricBoiler')],
    ['indirect_boiler', t('nta.form.dhwGen.indirectBoiler')], ['external_heat', t('nta.form.dhwGen.external')],
    ['measured_two_profiles', t('nta.form.dhwGen.twoProfiles')], ['heat_pump_en16147', t('nta.form.dhwGen.en16147')], ['heating_system', t('nta.form.dhwGen.heatingSystem')],
    ['chp', t('nta.form.generator.chp')]];
}

const PROFILES: Array<[string, string]> = ['s', 'm', 'l', 'xl', 'xxl', '3xl', '4xl'].map((key) => [key, key.toUpperCase()]);

/** §13.8.4.2 test results at two tapping profiles (13.153a–13.160a). */
function TwoProfileFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const at = (...rest: Path): Path => [...base, ...rest];
  const electric = read(draft, at('standard')) === 'en16147_heat_pump';
  const pfhrd = read(draft, at('pfhrd'));
  const mixed = read(draft, at('mixedAir', 'method'));
  return <div className="nta-form-subsection">
    <SelectField {...field} path={at('standard')} label={t('nta.form.twoProfile.standard')} options={[
      ['en13203_gas', t('nta.form.twoProfile.gas')], ['en16147_heat_pump', t('nta.form.twoProfile.heatPump')]]} />
    <CheckField {...field} path={at('storageAppliance')} label={t('nta.form.twoProfile.storage')} />
    {(['low', 'high'] as const).map((which) => <fieldset key={which} className="nta-form-row">
      <legend>{t(`nta.form.twoProfile.${which}`)}</legend>
      <SelectField {...field} path={at(which, 'profile')} label={t('nta.form.twoProfile.profile')} options={PROFILES} />
      <NumberField {...field} path={at(which, 'deliveredKwhPerDay')} label={t('nta.form.twoProfile.delivered')} />
      <NumberField {...field} path={at(which, 'inputKwhPerDay')} label={t(electric ? 'nta.form.twoProfile.inputElectric' : 'nta.form.twoProfile.inputGas')} />
      {!electric && <NumberField {...field} path={at(which, 'auxiliaryKwhPerDay')} label={t('nta.form.twoProfile.auxiliary')} />}
      {electric && <NumberField {...field} path={at(which, 'maxTestTemperatureC')} label={t('nta.form.twoProfile.maxTemperature')} />}
    </fieldset>)}
    <CheckField {...field} path={at('combi')} label={t('nta.form.twoProfile.combi')} />
    <CheckField {...field} path={at('integratedVessel')} label={t('nta.form.twoProfile.vessel')} />
    {electric && <>
      <CheckField {...field} path={at('exhaustAirSource')} label={t('nta.form.dhwExhaustAir')} />
      <NumberField {...field} path={at('outdoorAirFraction')} label={t('nta.form.twoProfile.outdoorShare')} />
      <NumberField {...field} path={at('smartControlFactor')} label={t('nta.form.twoProfile.scf')} />
      <NumberField {...field} path={at('designSetTemperatureC')} label={t('nta.form.twoProfile.designSet')} />
      <CheckField {...field} path={at('legionellaCycleTested')} label={t('nta.form.twoProfile.legionella')} />
      <label>{t('nta.form.twoProfile.mixedAir')}
        <select value={typeof mixed === 'string' ? mixed : ''} onChange={(event) => change(at('mixedAir'),
          event.target.value === 'declared' ? { method: 'declared', monthlyFactors: Array(12).fill(null), sourceReference: '' }
            : event.target.value === 'en14511' ? { method: 'en14511', copCondition2: null, condenserOutC: 55, evaporatorInC: 7,
              evaporatorOutC: null, minimumAirFlowM3PerH: null, sourceReference: '' } : null)}>
          <option value="">{t('nta.form.twoProfile.mixedAir.none')}</option>
          <option value="declared">{t('nta.form.twoProfile.mixedAir.declared')}</option>
          <option value="en14511">{t('nta.form.twoProfile.mixedAir.en14511')}</option>
        </select>
      </label>
      {mixed === 'declared' && <MonthlyValues draft={draft} change={change} path={at('mixedAir', 'monthlyFactors')}
        label={t('nta.form.twoProfile.mixedAir.factors')} />}
      {mixed === 'en14511' && <>
        <NumberField {...field} path={at('mixedAir', 'copCondition2')} label={t('nta.form.twoProfile.mixedAir.cop')} />
        <NumberField {...field} path={at('mixedAir', 'condenserOutC')} label={t('nta.form.twoProfile.mixedAir.condenserOut')} />
        <NumberField {...field} path={at('mixedAir', 'evaporatorInC')} label={t('nta.form.twoProfile.mixedAir.evaporatorIn')} />
        <NumberField {...field} path={at('mixedAir', 'evaporatorOutC')} label={t('nta.form.twoProfile.mixedAir.evaporatorOut')} />
        <NumberField {...field} path={at('mixedAir', 'minimumAirFlowM3PerH')} label={t('nta.form.twoProfile.mixedAir.flow')} />
      </>}
      {mixed && <TextField {...field} path={at('mixedAir', 'sourceReference')} label={t('nta.form.source')} />}
    </>}
    {!electric && <label className="nta-form-check">
      <input type="checkbox" checked={pfhrd != null} onChange={(event) => change(at('pfhrd'), event.target.checked
        ? { indirectGasKwhPerDay: null, heatingGasKwhPerDay: null, sourceReference: '' } : null)} />
      {t('nta.form.twoProfile.pfhrd')}
    </label>}
    {!electric && pfhrd != null && <>
      <NumberField {...field} path={at('pfhrd', 'indirectGasKwhPerDay')} label={t('nta.form.twoProfile.pfhrdIndirect')} />
      <NumberField {...field} path={at('pfhrd', 'heatingGasKwhPerDay')} label={t('nta.form.twoProfile.pfhrdHeating')} />
      <TextField {...field} path={at('pfhrd', 'sourceReference')} label={t('nta.form.source')} />
    </>}
    <TextField {...field} path={at('sourceReference')} label={t('nta.form.source')} />
  </div>;
}

/** 13.160b: heat pump tested at one NEN-EN 16147 profile, with the 13.153b/13.153c corrections. */
function En16147Fields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const at = (...rest: Path): Path => [...base, ...rest];
  return <div className="nta-form-subsection">
    <SelectField {...field} path={at('profile')} label={t('nta.form.twoProfile.profile')} options={PROFILES.slice(0, 4)} />
    <NumberField {...field} path={at('deliveredKwhPerDay')} label={t('nta.form.twoProfile.delivered')} />
    <NumberField {...field} path={at('inputKwhPerDay')} label={t('nta.form.twoProfile.inputElectric')} />
    <CheckField {...field} path={at('exhaustAirSource')} label={t('nta.form.dhwExhaustAir')} />
    <NumberField {...field} path={at('outdoorAirFraction')} label={t('nta.form.twoProfile.outdoorShare')} />
    <CheckField {...field} path={at('storageWithoutLegionellaCycle')} label={t('nta.form.en16147.noLegionella')} />
    <NumberField {...field} path={at('smartControlFactor')} label={t('nta.form.twoProfile.scf')} />
    <NumberField {...field} path={at('maxTestTemperatureC')} label={t('nta.form.twoProfile.maxTemperature')} />
    <NumberField {...field} path={at('designSetTemperatureC')} label={t('nta.form.twoProfile.designSet')} />
    <p className="nta-form-note">{t('nta.form.en16147.note')}</p>
    <TextField {...field} path={at('sourceReference')} label={t('nta.form.source')} />
  </div>;
}

/** §13.6.2 loss of a vessel: label (13.59), measured H_sto;ls or the standby test (13.60). */
function StorageLossFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const method = read(draft, [...base, 'method']);
  return <>
    <SelectField {...field} path={[...base, 'method']} label={t('nta.form.solar.storageLoss')} options={[
      ['label', t('nta.form.solar.storageLoss.label')], ['unknown_label', t('nta.form.solar.storageLoss.unknown')],
      ['measured', t('nta.form.solar.storageLoss.measured')], ['measured_standby', t('nta.form.storageLoss.standby')]]}
      onChange={(_, value) => change(base, storageLossTemplate(String(value ?? '')))} />
    {method === 'label' && <SelectField {...field} path={[...base, 'label']} label={t('nta.form.solar.energyLabel')}
      options={['a_plus', 'a', 'b', 'c', 'd', 'e', 'f', 'g'].map((key) => [key, key === 'a_plus' ? 'A+' : key.toUpperCase()])} />}
    {method === 'unknown_label' && <CheckField {...field} path={[...base, 'producedFrom2018']} label={t('nta.form.solar.from2018')} />}
    {method === 'measured' && <NumberField {...field} path={[...base, 'transmissionWPerK']} label={t('nta.form.solar.transmission')} />}
    {method === 'measured_standby' && <>
      <NumberField {...field} path={[...base, 'standbyKwhPerDay']} label={t('nta.form.storageLoss.standbyKwh')} />
      <NumberField {...field} path={[...base, 'referenceStorageC']} label={t('nta.form.storageLoss.referenceStorage')} />
      <NumberField {...field} path={[...base, 'referenceAmbientC']} label={t('nta.form.storageLoss.referenceAmbient')} />
    </>}
    {(method === 'measured' || method === 'measured_standby') && <p className="nta-form-note">{t('nta.form.storageLoss.roundingNote')}</p>}
  </>;
}

/** §13.6 hot-water vessels of the system (`hotWater.storage`). */
export function HotWaterStorageFields({ draft, change, base = ['hotWater'] }: SectionProps & { base?: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const vessels = list(draft, [...base, 'storage']);
  return <div className="nta-form-subsection">
    {vessels.map((_, index) => {
      const at = (...rest: Path): Path => [...base, 'storage', index, ...rest];
      const heated = read(draft, at('inHeatedZone')) === true;
      return <fieldset key={index} className="nta-form-row">
        <legend>{t('nta.form.vessel')} {index + 1}</legend>
        <TextField {...field} path={at('id')} label={t('nta.form.vessel.id')} />
        <NumberField {...field} path={at('volumeL')} label={t('nta.form.vessel.volume')} />
        <StorageLossFields draft={draft} change={change} base={at('loss')} />
        <SelectField {...field} path={at('connectionFactor')} label={t('nta.form.vessel.connection')}
          options={['1', '2', '3', '4', '5'].map((key) => [key, key])}
          onChange={(path, value) => change(path, value == null ? null : Number(value))} />
        <CheckField {...field} path={at('inHeatedZone')} label={t('nta.form.vessel.heated')} />
        {!heated && <NumberField {...field} path={at('unheatedAmbientC')} label={t('nta.form.vessel.ambient')} />}
        <CheckField {...field} path={at('notInApplianceTest')} label={t('nta.form.vessel.notInTest')} />
        <TextField {...field} path={at('sourceReference')} label={t('nta.form.source')} />
        <RemoveButton label={t('nta.form.remove')}
          onClick={() => change([...base, 'storage'], vessels.filter((__, other) => other !== index))} />
      </fieldset>;
    })}
    <button type="button" onClick={() => change([...base, 'storage'], [...vessels, hotWaterStorageTemplate(vessels.length)])}>
      {t('nta.form.vessel.add')}
    </button>
  </div>;
}

/** Per generator: 13.144a/13.148 exhaust-air use and the 13.146 declared share; `unit` holds `generator`. */
export function HotWaterUnitFields({ draft, change, unit }: SectionProps & { unit: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const generator = read(draft, [...unit, 'generator']) as Draft | undefined;
  const exhaustSource = generator != null && (generator.exhaustAirSource === true);
  const exhaust = read(draft, [...unit, 'exhaustAir']);
  const share = read(draft, [...unit, 'declaredShare']);
  const points = list(draft, [...unit, 'declaredShare', 'points']);
  return <>
    {exhaustSource && <label className="nta-form-check">
      <input type="checkbox" checked={exhaust != null}
        onChange={(event) => change([...unit, 'exhaustAir'], event.target.checked ? exhaustAirUseTemplate() : null)} />
      {t('nta.form.dhwExhaust.use')}
    </label>}
    {exhaustSource && exhaust != null && <>
      <CheckField {...field} path={[...unit, 'exhaustAir', 'ventilationSuitable']} label={t('nta.form.dhwExhaust.suitable')} />
      <NumberField {...field} path={[...unit, 'exhaustAir', 'declaredFlowM3PerH']} label={t('nta.form.dhwExhaust.flow')} />
      <MonthlyValues draft={draft} change={change} path={[...unit, 'exhaustAir', 'heatingTimeFraction']}
        label={t('nta.form.dhwExhaust.combi')} />
    </>}
    <label className="nta-form-check">
      <input type="checkbox" checked={share != null}
        onChange={(event) => change([...unit, 'declaredShare'], event.target.checked ? declaredShareTemplate() : null)} />
      {t('nta.form.dhwShare')}
    </label>
    {share != null && <>
      {points.map((_, index) => <fieldset key={index} className="nta-form-row">
        <NumberField {...field} path={[...unit, 'declaredShare', 'points', index, 'annualKwh']} label={t('nta.form.dhwShare.annual')} />
        <NumberField {...field} path={[...unit, 'declaredShare', 'points', index, 'share']} label={t('nta.form.dhwShare.share')} />
        {points.length > 1 && <RemoveButton label={t('nta.form.remove')}
          onClick={() => change([...unit, 'declaredShare', 'points'], points.filter((__, other) => other !== index))} />}
      </fieldset>)}
      <button type="button" onClick={() => change([...unit, 'declaredShare', 'points'], [...points, { annualKwh: null, share: null }])}>
        {t('nta.form.dhwShare.add')}
      </button>
      <TextField {...field} path={[...unit, 'declaredShare', 'sourceReference']} label={t('nta.form.source')} />
    </>}
  </>;
}

/** One hot-water generator at `base` (13.8). */
export function HotWaterGeneratorFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const kind = read(draft, [...base, 'kind']);
  const classes: Array<[string, string]> = [['class1', '1 (CW-1+)'], ['class2', '2 (CW-2)'], ['class3', '3 (CW-3)'], ['class4', '4 (CW-4/5/6)']];
  return <>
    <SelectField {...field} path={[...base, 'kind']} label={t('nta.form.hotWaterGenerator')} options={hotWaterGeneratorOptions(t)}
      onChange={(_, value) => change(base, hotWaterGeneratorTemplate(String(value ?? '')))} />
    {kind === 'gas_appliance' && <>
      <SelectField {...field} path={[...base, 'appliance']} label={t('nta.form.dhwGas')} options={[
        ['combi_gaskeur_hr_cw', t('nta.form.dhwGas.combiHrCw')], ['combi_gaskeur', t('nta.form.dhwGas.combi')],
        ['water_heater_gaskeur_cw', t('nta.form.dhwGas.heaterCw')], ['water_heater_gaskeur', t('nta.form.dhwGas.heater')],
        ['kitchen_geyser', t('nta.form.dhwGas.geyser')], ['without_gaskeur', t('nta.form.dhwGas.none')], ['unknown', t('nta.form.dhwGas.unknown')]]} />
      <SelectField {...field} path={[...base, 'measuredClass']} label={t('nta.form.dhwClass')} options={classes} />
    </>}
    {kind === 'heat_pump' && <>
      <CheckField {...field} path={[...base, 'exhaustAirSource']} label={t('nta.form.dhwExhaustAir')} />
      <SelectField {...field} path={[...base, 'measuredClass']} label={t('nta.form.dhwClass')} options={classes} />
      <CheckField {...field} path={[...base, 'sameGroundSource']} label={t('nta.form.dhwSameGroundSource')} />
    </>}
    {kind === 'indirect_boiler' && <SelectField {...field} path={[...base, 'boiler']} label={t('nta.form.dhwBoiler')} options={[
      ['hr107', 'HR 107'], ['hr100_or104', 'HR 100/104'], ['vr', 'VR'], ['conventional_or_unknown', t('nta.form.dhwGas.unknown')]]} />}
    {kind === 'measured_two_profiles' && <TwoProfileFields draft={draft} change={change} base={base} />}
    {kind === 'heat_pump_en16147' && <En16147Fields draft={draft} change={change} base={base} />}
    {kind === 'heating_system' && <p className="nta-form-note">{t('nta.form.dhwGen.heatingSystemNote')}</p>}
    {kind === 'chp' && <>
      <ChpClassFields draft={draft} change={change} base={base} />
      <CheckField {...field} path={[...base, 'alsoSpaceHeating']} label={t('nta.form.chp.alsoHeating')} />
    </>}
  </>;
}

/** 13.8.2: nominal power of the main generator, further generators and the series arrangement. */
export function HotWaterGeneratorsFields({ draft, change }: SectionProps) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const extra = list(draft, ['hotWater', 'additionalGenerators']);
  const series = read(draft, ['hotWater', 'series', 'kind']);
  return <div className="nta-form-subsection">
    <NumberField {...field} path={['hotWater', 'nominalPowerKw']} label={t('nta.form.dhwNominalPower')} />
    <HotWaterUnitFields draft={draft} change={change} unit={['hotWater']} />
    {extra.map((_, index) => <fieldset key={index} className="nta-form-row">
      <legend>{t('nta.form.dhwExtra')} {index + 1}</legend>
      <HotWaterGeneratorFields draft={draft} change={change} base={['hotWater', 'additionalGenerators', index, 'generator']} />
      <NumberField {...field} path={['hotWater', 'additionalGenerators', index, 'nominalPowerKw']} label={t('nta.form.dhwNominalPower')} />
      <HotWaterUnitFields draft={draft} change={change} unit={['hotWater', 'additionalGenerators', index]} />
      <TextField {...field} path={['hotWater', 'additionalGenerators', index, 'equipmentReference']} label={t('nta.form.boilerEquipmentSource')} />
      <RemoveButton label={t('nta.form.remove')}
        onClick={() => change(['hotWater', 'additionalGenerators'], extra.filter((__, other) => other !== index))} />
    </fieldset>)}
    <button type="button" onClick={() => change(['hotWater', 'additionalGenerators'], [...extra, additionalHotWaterGeneratorTemplate()])}>
      {t('nta.form.dhwExtraAdd')}
    </button>
    {extra.length === 1 && <label>{t('nta.form.dhwSeries')}
      <select value={typeof series === 'string' ? series : ''} onChange={(event) => change(['hotWater', 'series'],
        event.target.value === 'hotfill_electric_boiler' ? { kind: 'hotfill_electric_boiler' }
          : event.target.value === 'collective_first_also_heating'
            ? { kind: 'collective_first_also_heating', maximumSupplyC: Array(12).fill(null) } : null)}>
        <option value="">{t('nta.form.dhwSeries.none')}</option>
        <option value="hotfill_electric_boiler">{t('nta.form.dhwSeries.hotfill')}</option>
        <option value="collective_first_also_heating">{t('nta.form.dhwSeries.collective')}</option>
      </select>
    </label>}
    {series === 'collective_first_also_heating' && <MonthlyValues draft={draft} change={change}
      path={['hotWater', 'series', 'maximumSupplyC']} label={t('nta.form.dhwSeries.maxSupply')} />}
  </div>;
}

/** A further hot-water system (§13.2.4), started from the main system. */
function additionalHotWaterSystem(main: Draft | null, residential: boolean): Draft {
  const need = (main?.need as Draft | undefined) ?? (residential
    ? { method: 'residential', dwellingCount: 1, sourceReference: '' }
    : { method: 'utility', areas: [{ function: 'office', areaM2: null }], sourceReference: '' });
  return {
    need: residential ? need : { method: 'utility', areas: [{ function: 'office', areaM2: null }], sourceReference: '' },
    emission: residential
      ? { method: 'residential', served: 'kitchen_only', kitchenLengthM: null, bathroomLengthM: null, sourceReference: '' }
      : { method: 'utility', meanLengthM: null, sourceReference: '' },
    connectedTaps: residential ? { bathrooms: 0, kitchens: 1 } : null,
    storage: [],
    boilingWaterTap: false,
    generator: hotWaterGeneratorTemplate('electric_instantaneous'),
    equipmentReference: '',
  };
}

/**
 * §13.2.4: further hot-water systems. Dwellings split the need by the
 * connected bathrooms and kitchens (13.19a); utility systems by the areas
 * of their own need (13.20).
 */
export function AdditionalHotWaterSystemsFields({ draft, change, residential }: SectionProps & { residential: boolean }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const systems = list(draft, ['additionalHotWaterSystems']);
  return <div className="nta-form-subsection">
    {systems.length > 0 && residential && <fieldset className="nta-form-row">
      <legend>{t('nta.form.dhwSystems.mainTaps')}</legend>
      <NumberField {...field} path={['hotWater', 'connectedTaps', 'bathrooms']} label={t('nta.form.dhwSystems.bathrooms')} step="1" />
      <NumberField {...field} path={['hotWater', 'connectedTaps', 'kitchens']} label={t('nta.form.dhwSystems.kitchens')} step="1" />
    </fieldset>}
    {systems.map((_, index) => {
      const base: Path = ['additionalHotWaterSystems', index];
      return <fieldset key={index} className="nta-form-row">
        <legend>{t('nta.form.dhwSystems.system')} {index + 2}</legend>
        {residential ? <>
          <SelectField {...field} path={[...base, 'emission', 'served']} label={t('nta.form.hotWaterTaps')} options={[
            ['kitchen_and_bathroom', t('nta.form.dhwTaps.both')], ['bathroom_only', t('nta.form.dhwTaps.bathroom')],
            ['kitchen_only', t('nta.form.dhwTaps.kitchen')]]} />
          <NumberField {...field} path={[...base, 'emission', 'kitchenLengthM']} label={t('nta.form.hotWaterKitchenLength')} />
          <NumberField {...field} path={[...base, 'emission', 'bathroomLengthM']} label={t('nta.form.hotWaterBathroomLength')} />
          <NumberField {...field} path={[...base, 'connectedTaps', 'bathrooms']} label={t('nta.form.dhwSystems.bathrooms')} step="1" />
          <NumberField {...field} path={[...base, 'connectedTaps', 'kitchens']} label={t('nta.form.dhwSystems.kitchens')} step="1" />
        </> : <>
          <NumberField {...field} path={[...base, 'need', 'areas', 0, 'areaM2']} label={t('nta.form.dhwSystems.servedArea')} />
          <NumberField {...field} path={[...base, 'emission', 'meanLengthM']} label={t('nta.form.hotWaterMeanLength')} />
        </>}
        <TextField {...field} path={[...base, 'emission', 'sourceReference']} label={t('nta.form.source')} />
        <HotWaterGeneratorFields draft={draft} change={change} base={[...base, 'generator']} />
        <HotWaterStorageFields draft={draft} change={change} base={base} />
        <TextField {...field} path={[...base, 'equipmentReference']} label={t('nta.form.boilerEquipmentSource')} />
        <RemoveButton label={t('nta.form.remove')}
          onClick={() => change(['additionalHotWaterSystems'], systems.filter((__, other) => other !== index))} />
      </fieldset>;
    })}
    <button type="button" className="btn" onClick={() => {
      if (systems.length === 0 && residential && read(draft, ['hotWater', 'connectedTaps']) == null) {
        change(['hotWater', 'connectedTaps'], { bathrooms: 1, kitchens: 0 });
      }
      change(['additionalHotWaterSystems'],
        [...systems, additionalHotWaterSystem(read(draft, ['hotWater']) as Draft | null, residential)]);
    }}>{t('nta.form.dhwSystems.add')}</button>
  </div>;
}

/**
 * §9.2: further heating systems, each with the calculation zones it serves
 * and its own generator. The main system serves the remaining zones.
 */
export function AdditionalHeatingSystemsFields({ draft, change, project }: SectionProps & { project: IProject }) {
  const { t } = useI18n();
  const systems = list(draft, ['additionalHeatingSystems']);
  const taken = (except: number) => new Set(systems.flatMap((system, index) =>
    index === except ? [] : (Array.isArray(system.zoneIds) ? system.zoneIds as string[] : [])));
  return <div className="nta-form-subsection">
    {systems.map((system, index) => {
      const base: Path = ['additionalHeatingSystems', index];
      const zoneIds = Array.isArray(system.zoneIds) ? system.zoneIds as string[] : [];
      const elsewhere = taken(index);
      return <fieldset key={index} className="nta-form-row">
        <legend>{t('nta.form.heatingSystems.system')} {index + 2}</legend>
        {project.zones.map((zone) => <label key={zone.id} className="nta-form-check">
          <input type="checkbox" checked={zoneIds.includes(zone.id)} disabled={elsewhere.has(zone.id)}
            onChange={(event) => change([...base, 'zoneIds'], event.target.checked
              ? [...zoneIds, zone.id]
              : zoneIds.filter((id) => id !== zone.id))} />
          {zone.name || zone.id}
        </label>)}
        <SpaceGeneratorFields draft={draft} change={change} base={[...base, 'generator']} project={project} allowMultiple />
        <RemoveButton label={t('nta.form.remove')}
          onClick={() => change(['additionalHeatingSystems'], systems.filter((__, other) => other !== index))} />
      </fieldset>;
    })}
    <button type="button" className="btn" onClick={() => change(['additionalHeatingSystems'],
      [...systems, { zoneIds: [], generator: spaceGeneratorTemplate('gas_boiler', project) }])}>
      {t('nta.form.heatingSystems.add')}
    </button>
  </div>;
}

/** Twelve monthly numbers in one row. */
function MonthlyValues({ draft, change, path, label }: SectionProps & { path: Path; label: string }) {
  const values = list(draft, path) as unknown as Array<number | null>;
  return <fieldset className="nta-form-months"><legend>{label}</legend>
    {Array.from({ length: 12 }, (_, index) => <input key={index} type="number" step="any" aria-label={`${label} ${index + 1}`}
      value={typeof values[index] === 'number' ? values[index] as number : ''}
      onChange={(event) => change([...path, index], event.target.value === '' ? null : Number(event.target.value))} />)}
  </fieldset>;
}

/** §17.3 collector obstruction (tables 17.6/17.12/17.15). */
function CollectorObstructionFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const method = read(draft, [...base, 'method']);
  return <>
    <SelectField {...field} path={[...base, 'method']} label={t('nta.form.obstruction')} options={[
      ['minimal', t('nta.form.obstruction.minimal')], ['side_obstruction', t('nta.form.obstruction.side')],
      ['full', t('nta.form.obstruction.full')], ['roof_edge', t('nta.form.obstruction.roofEdge')],
      ['other', t('nta.form.obstruction.other')], ['declared', t('nta.form.obstruction.declared')]]}
      onChange={(_, value) => change(base, collectorObstructionTemplate(String(value ?? 'minimal')))} />
    {method === 'side_obstruction' && <>
      <SelectField {...field} path={[...base, 'side']} label={t('nta.form.obstruction.sideWhich')} options={[
        ['left', t('nta.form.obstruction.left')], ['right', t('nta.form.obstruction.right')], ['both', t('nta.form.obstruction.both')]]} />
      <NumberField {...field} path={[...base, 'relativeWidth']} label={t('nta.form.obstruction.relativeWidth')} />
    </>}
    {method === 'roof_edge' && <>
      <NumberField {...field} path={[...base, 'heightM']} label={t('nta.form.obstruction.roofEdgeHeight')} />
      <NumberField {...field} path={[...base, 'distanceM']} label={t('nta.form.obstruction.roofEdgeDistance')} />
    </>}
    {method === 'declared' && <>
      <MonthlyValues draft={draft} change={change} path={[...base, 'factors']} label={t('nta.form.obstruction.factors')} />
      <TextField {...field} path={[...base, 'sourceReference']} label={t('nta.form.source')} />
    </>}
  </>;
}

/** §13.7 solar water heaters and solar combi systems (`hotWater.solar`). */
export function SolarWaterHeaterFields({ draft, change }: SectionProps) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const systems = list(draft, ['hotWater', 'solar']);
  const orientations: Array<[string, string]> = ORIENTATIONS.map((key) => [key, t(`nta.form.orientation.${key}`)]);
  return <div className="nta-form-subsection">
    {systems.map((_, index) => {
      const at = (...rest: Path): Path => ['hotWater', 'solar', index, ...rest];
      const method = read(draft, at('method', 'method'));
      const points = list(draft, at('method', 'testPoints'));
      return <fieldset key={index} className="nta-form-row">
        <legend>{t('nta.form.solar.system')} {index + 1}</legend>
        <TextField {...field} path={at('id')} label={t('nta.form.solar.id')} />
        <SelectField {...field} path={at('solarUse')} label={t('nta.form.solar.use')} options={[
          ['water_heating', t('nta.form.solar.use.water')], ['combi', t('nta.form.solar.use.combi')],
          ['space_heating', t('nta.form.solar.use.space')]]} />
        <NumberField {...field} path={at('count')} label={t('nta.form.solar.count')} step="1" />
        <SelectField {...field} path={at('method', 'method')} label={t('nta.form.solar.method')} options={[
          ['calculated', t('nta.form.solar.method.calculated')], ['tested', t('nta.form.solar.method.tested')]]}
          onChange={(_, value) => change(at('method'), value === 'tested' ? testedSolarMethod() : calculatedSolarMethod())} />
        <SelectField {...field} path={at('method', 'solarType')} label={t('nta.form.solar.type')} options={[
          ['preheater', t('nta.form.solar.type.preheater')], ['integrated_backup', t('nta.form.solar.type.integrated')]]} />
        {method === 'calculated' && <>
          <NumberField {...field} path={at('method', 'collectors', 'moduleAreaM2')} label={t('nta.form.solar.moduleArea')} />
          <NumberField {...field} path={at('method', 'collectors', 'moduleCount')} label={t('nta.form.solar.moduleCount')} step="1" />
          <SelectField {...field} path={at('method', 'collectors', 'orientation')} label={t('nta.form.solar.orientation')} options={orientations} />
          <NumberField {...field} path={at('method', 'collectors', 'tiltDeg')} label={t('nta.form.tilt')} />
          <CollectorObstructionFields draft={draft} change={change} base={at('method', 'collectors', 'obstruction')} />
          <SelectField {...field} path={at('method', 'collectors', 'efficiency', 'method')} label={t('nta.form.solar.efficiency')} options={[
            ['forfait', t('nta.form.solar.efficiency.forfait')], ['declared', t('nta.form.solar.efficiency.declared')]]}
            onChange={(_, value) => change(at('method', 'collectors', 'efficiency'), value === 'declared'
              ? { method: 'declared', eta0: null, a1WPerM2K: null, a2WPerM2K2: null, incidenceAngleModifier: null, sourceReference: '' }
              : { method: 'forfait', collector: 'glazed' })} />
          {read(draft, at('method', 'collectors', 'efficiency', 'method')) === 'declared' ? <>
            <NumberField {...field} path={at('method', 'collectors', 'efficiency', 'eta0')} label="η0" />
            <NumberField {...field} path={at('method', 'collectors', 'efficiency', 'a1WPerM2K')} label="a1 W/(m²K)" />
            <NumberField {...field} path={at('method', 'collectors', 'efficiency', 'a2WPerM2K2')} label="a2 W/(m²K²)" />
            <NumberField {...field} path={at('method', 'collectors', 'efficiency', 'incidenceAngleModifier')} label="IAM" />
            <TextField {...field} path={at('method', 'collectors', 'efficiency', 'sourceReference')} label={t('nta.form.source')} />
          </> : <SelectField {...field} path={at('method', 'collectors', 'efficiency', 'collector')} label={t('nta.form.solar.collector')} options={[
            ['unglazed_or_unknown', t('nta.form.solar.collector.unglazed')], ['glazed', t('nta.form.solar.collector.glazed')],
            ['evacuated_tube', t('nta.form.solar.collector.tube')]]} />}
          <NumberField {...field} path={at('method', 'collectors', 'heatExchangerWPerK')} label={t('nta.form.solar.heatExchanger')} />
          <SelectField {...field} path={at('method', 'collectors', 'loopPipes', 'method')} label={t('nta.form.solar.loop')} options={[
            ['forfait', t('nta.form.solar.loop.forfait')], ['declared', t('nta.form.solar.loop.declared')]]}
            onChange={(_, value) => change(at('method', 'collectors', 'loopPipes'), value === 'declared'
              ? { method: 'declared', heatLossWPerK: null, sourceReference: '' } : { method: 'forfait' })} />
          {read(draft, at('method', 'collectors', 'loopPipes', 'method')) === 'declared' && <>
            <NumberField {...field} path={at('method', 'collectors', 'loopPipes', 'heatLossWPerK')} label={t('nta.form.solar.loopLoss')} />
            <TextField {...field} path={at('method', 'collectors', 'loopPipes', 'sourceReference')} label={t('nta.form.source')} />
          </>}
          <NumberField {...field} path={at('method', 'collectors', 'pumpPowerW')} label={t('nta.form.solar.pump')} />
          <NumberField {...field} path={at('method', 'storage', 'totalVolumeL')} label={t('nta.form.solar.volume')} />
          <NumberField {...field} path={at('method', 'storage', 'backupVolumeL')} label={t('nta.form.solar.backupVolume')} />
          <StorageLossFields draft={draft} change={change} base={at('method', 'storage', 'loss')} />
          <CheckField {...field} path={at('method', 'storage', 'backupLossInGeneratorEfficiency')} label={t('nta.form.solar.backupInGenerator')} />
        </>}
        {method === 'tested' && <>
          <SelectField {...field} path={at('method', 'orientation')} label={t('nta.form.solar.orientation')} options={orientations} />
          <NumberField {...field} path={at('method', 'tiltDeg')} label={t('nta.form.tilt')} />
          <CollectorObstructionFields draft={draft} change={change} base={at('method', 'obstruction')} />
          <NumberField {...field} path={at('method', 'totalVolumeL')} label={t('nta.form.solar.volume')} />
          {points.map((__, point) => <div key={point} className="nta-form-row">
            <NumberField {...field} path={at('method', 'testPoints', point, 'annualDemandKwh')} label={t('nta.form.solar.testDemand')} />
            <NumberField {...field} path={at('method', 'testPoints', point, 'solarOutputKwh')} label={t('nta.form.solar.testSolar')} />
            <NumberField {...field} path={at('method', 'testPoints', point, 'backupOutputKwh')} label={t('nta.form.solar.testBackup')} />
            <NumberField {...field} path={at('method', 'testPoints', point, 'auxiliaryKwh')} label={t('nta.form.solar.testAuxiliary')} />
          </div>)}
          <button type="button" onClick={() => change(at('method', 'testPoints'),
            [...points, { annualDemandKwh: null, solarOutputKwh: null, auxiliaryKwh: null }])}>{t('nta.form.solar.testAdd')}</button>
          <CheckField {...field} path={at('method', 'backupLossInGeneratorEfficiency')} label={t('nta.form.solar.backupInGenerator')} />
          <TextField {...field} path={at('method', 'sourceReference')} label={t('nta.form.source')} />
        </>}
        <SelectField {...field} path={at('pvt')} label={t('nta.form.solar.pvt')} options={[
          ['unglazed', t('nta.form.solar.pvt.unglazed')], ['single_glazed', t('nta.form.solar.pvt.glazed')],
          ['tested_iso9806', t('nta.form.solar.pvt.tested')]]} />
        <TextField {...field} path={at('sourceReference')} label={t('nta.form.source')} />
        <RemoveButton label={t('nta.form.remove')} onClick={() => change(['hotWater', 'solar'], systems.filter((__, other) => other !== index))} />
      </fieldset>;
    })}
    <button type="button" onClick={() => change(['hotWater', 'solar'], [...systems, solarWaterHeaterTemplate(systems.length)])}>
      {t('nta.form.solar.add')}
    </button>
  </div>;
}

/** §10.5.4/§10.5.5 rating of a compression generator, and its heat rejection. */
export function CoolingPerformanceFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const kind = read(draft, [...base, 'kind']);
  const roomUnit = kind === 'room_air_conditioner';
  const method = read(draft, [...base, 'performance', 'method']);
  const at = (...rest: Path): Path => [...base, 'performance', ...rest];
  const points = list(draft, at('testPoints'));
  const point = (path: Path, label: string) => <div className="nta-form-row" key={label}>
    <NumberField {...field} path={[...path, 'partLoadPercent']} label={`${label} — ${t('nta.form.cool.partLoad')}`} />
    <NumberField {...field} path={[...path, 'eer']} label="EER" />
    <NumberField {...field} path={[...path, 'evaporatorOutletC']} label={t('nta.form.cool.evapOut')} />
    <NumberField {...field} path={[...path, 'condenserInletC']} label={t('nta.form.cool.condIn')} />
  </div>;
  return <>
    {kind === 'compression' && <SelectField {...field} path={[...base, 'heatRejection']} label={t('nta.form.cool.heatRejection')} options={[
      ['air_cooled', t('nta.form.cool.hr.air')], ['closed_cooling_tower', t('nta.form.cool.hr.closedTower')],
      ['open_cooling_tower', t('nta.form.cool.hr.openTower')], ['dry_cooler', t('nta.form.cool.hr.dry')],
      ['ground_storage', t('nta.form.cool.hr.ground')], ['surface_water', t('nta.form.cool.hr.surface')]]} />}
    <SelectField {...field} path={at('method')} label={t('nta.form.cool.method')} options={[
      ['en14825', t('nta.form.cool.method.en14825')], ['en14511', t('nta.form.cool.method.en14511')]]}
      onChange={(_, value) => change([...base, 'performance'], coolingPerformanceTemplate(String(value ?? ''), roomUnit))} />
    {method != null && <>
      <NumberField {...field} path={at('nominalEer')} label={t('nta.form.cool.nominalEer')} />
      <NumberField {...field} path={at('nominalCapacityKw')} label={t('nta.form.cool.nominalCapacity')} />
    </>}
    {method === 'en14825' && <>
      <NumberField {...field} path={at('minimumCapacityKw')} label={t('nta.form.cool.minimumCapacity')} />
      {points.map((_, index) => point(at('testPoints', index), ['A', 'B', 'C', 'D'][index] ?? String(index + 1)))}
      <label className="nta-form-check">
        <input type="checkbox" checked={read(draft, at('fifthPoint')) != null}
          onChange={(event) => change(at('fifthPoint'), event.target.checked ? en14825PointTemplate() : null)} />
        {t('nta.form.cool.fifthPoint')}
      </label>
      {read(draft, at('fifthPoint')) != null && point(at('fifthPoint'), '5')}
      <NumberField {...field} path={at('condenserInletLimitC')} label={t('nta.form.cool.condLimit')} />
    </>}
    {method === 'en14511' && <>
      <NumberField {...field} path={at('nominalEvaporatorOutletC')} label={t('nta.form.cool.evapOut')} />
      <NumberField {...field} path={at('nominalCondenserInletC')} label={t('nta.form.cool.condIn')} />
      {roomUnit && <SelectField {...field} path={at('roomUnitType')} label={t('nta.form.cool.roomUnit')} options={[
        ['split', t('nta.form.cool.ru.split')], ['multi_split_staged', t('nta.form.cool.ru.multiStaged')],
        ['split_inverter', t('nta.form.cool.ru.inverter')], ['multi_split_inverter', t('nta.form.cool.ru.multiInverter')]]} />}
      <CheckField {...field} path={at('heatRejectionToExhaustAir')} label={t('nta.form.cool.exhaustAir')} />
      <CheckField {...field} path={at('axialFansWithoutSilencer')} label={t('nta.form.cool.axialFans')} />
    </>}
    {method != null && <>
      <NumberField {...field} path={at('requiredOutletC')} label={t('nta.form.cool.requiredOutlet')} />
      <TextField {...field} path={at('sourceReference')} label={t('nta.form.source')} />
    </>}
  </>;
}

/** §17.3 window obstruction situations a–g (`windowSolar.obstruction`). */
export function WindowObstructionFields({ draft, change }: SectionProps) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const base: Path = ['windowSolar', 'obstruction'];
  const method = read(draft, [...base, 'method']);
  return <>
    <SelectField {...field} path={[...base, 'method']} label={t('nta.form.obstruction')} options={[
      ['minimal', t('nta.form.obstruction.minimal')], ['parallel_obstruction', t('nta.form.obstruction.parallel')],
      ['overhang', t('nta.form.obstruction.overhang')], ['side_obstruction', t('nta.form.obstruction.side')],
      ['full', t('nta.form.obstruction.full')], ['other', t('nta.form.obstruction.other')],
      ['declared', t('nta.form.obstruction.declared')]]}
      onChange={(_, value) => change(base, windowObstructionTemplate(String(value ?? 'minimal')))} />
    {(method === 'parallel_obstruction' || method === 'overhang') &&
      <NumberField {...field} path={[...base, 'relativeHeight']} label={t('nta.form.obstruction.relativeHeight')} />}
    {method === 'side_obstruction' && <>
      <SelectField {...field} path={[...base, 'side']} label={t('nta.form.obstruction.sideWhich')} options={[
        ['left', t('nta.form.obstruction.left')], ['right', t('nta.form.obstruction.right')], ['both', t('nta.form.obstruction.both')]]} />
      <NumberField {...field} path={[...base, 'relativeWidth']} label={t('nta.form.obstruction.relativeWidth')} />
      <CheckField {...field} path={[...base, 'coolingHeightCondition']} label={t('nta.form.obstruction.coolingHeight')} />
    </>}
    {method === 'full' && <CheckField {...field} path={[...base, 'coolingConditionsMet']} label={t('nta.form.obstruction.coolingConditions')} />}
    {method === 'other' && <NumberField {...field} path={[...base, 'overhangRelativeHeight']} label={t('nta.form.obstruction.overhangHeight')} />}
    {method === 'declared' && <>
      <MonthlyValues draft={draft} change={change} path={[...base, 'heating']} label={t('nta.form.obstruction.heating')} />
      <MonthlyValues draft={draft} change={change} path={[...base, 'cooling']} label={t('nta.form.obstruction.cooling')} />
      <TextField {...field} path={[...base, 'sourceReference']} label={t('nta.form.source')} />
    </>}
  </>;
}

