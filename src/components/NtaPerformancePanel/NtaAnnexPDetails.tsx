import { useI18n } from '../../i18n/i18n';
import { CheckField, NumberField, read, SelectField, TextField, type Draft, type Path } from './NtaFormFields';
import { MonthlyValues } from './NtaSystemSections';

// Annex P details of the calculated route: the pipe segments of P.13–P.18
// (with the network water temperature and buffer vessels), and the
// calculated η_WD;gen;sto of P.43–P.46 (vessels, charging pipes and the
// external exchanger). Field names follow annex_p.rs.

interface SectionProps {
  draft: Draft;
  change: (path: Path, value: unknown) => void;
}

function list(draft: Draft, path: Path): Draft[] {
  const value = read(draft, path);
  return Array.isArray(value) ? value as Draft[] : [];
}

function RemoveButton({ label, onClick }: { label: string; onClick: () => void }) {
  return <button type="button" className="nta-form-remove" onClick={onClick}>{label}</button>;
}

export function pipeSegmentTemplate(): Draft {
  return { lengthM: null, layers: [], placement: { kind: 'buried', coverDepthM: null, ambient: { kind: 'outdoor' } } };
}

export function storageVesselTemplate(): Draft {
  return { volumeL: null, insulation: null };
}

export function chargingPipeTemplate(): Draft {
  return { lengthM: null };
}

/** Surroundings of a pipe (`θ_ext;avg;mi,j`): outdoor, crawl space (P.16) or an indoor space. */
function PipeAmbientFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const kind = read(draft, [...base, 'kind']);
  return <>
    <SelectField {...field} path={[...base, 'kind']} label={t('nta.annexP.pipe.ambient')} options={[
      ['outdoor', t('nta.annexP.pipe.ambient.outdoor')], ['crawlspace', t('nta.annexP.pipe.ambient.crawlspace')],
      ['indoor', t('nta.annexP.pipe.ambient.indoor')]]}
      onChange={(_, value) => change(base, value === 'indoor' ? { kind: 'indoor', temperatureC: null }
        : { kind: value ?? 'outdoor' })} />
    {kind === 'indoor' && <NumberField {...field} path={[...base, 'temperatureC']} label={t('nta.annexP.pipe.indoorTemperature')} />}
  </>;
}

/** P.17 buried or P.18 in-air placement of a pipe segment. */
function PipePlacementFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const kind = read(draft, [...base, 'kind']);
  return <>
    <SelectField {...field} path={[...base, 'kind']} label={t('nta.annexP.pipe.placement')} options={[
      ['buried', t('nta.annexP.pipe.placement.buried')], ['in_air', t('nta.annexP.pipe.placement.inAir')]]}
      onChange={(_, value) => change(base, value === 'in_air'
        ? { kind: 'in_air', ambient: { kind: 'indoor', temperatureC: null } }
        : { kind: 'buried', coverDepthM: null, ambient: { kind: 'outdoor' } })} />
    {kind === 'buried' ? <>
      <NumberField {...field} path={[...base, 'coverDepthM']} label={t('nta.annexP.pipe.coverDepth')} />
      <NumberField {...field} path={[...base, 'groundConductivity']} label={t('nta.annexP.pipe.groundConductivity')} />
    </> : <NumberField {...field} path={[...base, 'surfaceCoefficient']} label={t('nta.annexP.pipe.surfaceCoefficient')} />}
    <PipeAmbientFields draft={draft} change={change} base={[...base, 'ambient']} />
  </>;
}

/** One P.13–P.18 pipe segment: length, layers inside out, placement and table P.1 correction. */
function PipeSegmentFields({ draft, change, base, index }: SectionProps & { base: Path; index: number }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const layers = list(draft, [...base, 'layers']);
  return <>
    <legend>{t('nta.annexP.pipe.segment')} {index + 1}</legend>
    <NumberField {...field} path={[...base, 'lengthM']} label={t('nta.annexP.pipe.length')} />
    {layers.map((_, layer) => <div key={layer} className="nta-form-row">
      <NumberField {...field} path={[...base, 'layers', layer, 'conductivityWPerMk']} label={`${t('nta.annexP.pipe.layer')} ${layer + 1} — λ W/(m·K)`} />
      <NumberField {...field} path={[...base, 'layers', layer, 'innerDiameterM']} label={t('nta.annexP.pipe.innerDiameter')} />
      <NumberField {...field} path={[...base, 'layers', layer, 'outerDiameterM']} label={t('nta.annexP.pipe.outerDiameter')} />
      <RemoveButton label={t('nta.form.remove')}
        onClick={() => change([...base, 'layers'], layers.filter((__, other) => other !== layer))} />
    </div>)}
    <button type="button" onClick={() => change([...base, 'layers'],
      [...layers, { conductivityWPerMk: null, innerDiameterM: null, outerDiameterM: null }])}>
      {t('nta.annexP.pipe.addLayer')}
    </button>
    <PipePlacementFields draft={draft} change={change} base={[...base, 'placement']} />
    <SelectField {...field} path={[...base, 'correction']} label={t('nta.annexP.pipe.correction')} options={[
      ['two_pipes_in_trench', t('nta.annexP.pipe.correction.twoPipes')],
      ['two_pipes_in_trench_rigid', t('nta.annexP.pipe.correction.twoPipesRigid')],
      ['old_sliding_system', t('nta.annexP.pipe.correction.oldSliding')],
      ['surface_or_recessed', t('nta.annexP.pipe.correction.surface')],
      ['single_pipe_in_trench', t('nta.annexP.pipe.correction.single')]]} />
    <NumberField {...field} path={[...base, 'correctionFactor']} label={t('nta.annexP.pipe.correctionFactor')} />
    <NumberField {...field} path={[...base, 'resistanceKmPerW']} label={t('nta.annexP.pipe.resistance')} />
  </>;
}

/** Mean network water temperature: constant, twelve months (P.14) or a heating curve over the bins (P.15). */
function WaterTemperatureFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const method = read(draft, [...base, 'method']);
  const curve = list(draft, [...base, 'curve']);
  return <>
    <label>{t('nta.annexP.pipe.waterTemperature')}
      <select value={typeof method === 'string' ? method : ''} onChange={(event) => {
        const value = event.target.value;
        change(base, value === 'constant' ? { method: value, temperatureC: null }
          : value === 'monthly' ? { method: value, temperaturesC: Array(12).fill(null) }
            : value === 'outdoor_bins' ? { method: value, curve: [] } : null);
      }}>
        <option value="">{t('nta.annexP.pipe.waterTemperature.default')}</option>
        <option value="constant">{t('nta.annexP.pipe.waterTemperature.constant')}</option>
        <option value="monthly">{t('nta.annexP.pipe.waterTemperature.monthly')}</option>
        <option value="outdoor_bins">{t('nta.annexP.pipe.waterTemperature.bins')}</option>
      </select>
    </label>
    {method === 'constant' && <NumberField {...field} path={[...base, 'temperatureC']} label={t('nta.annexP.pipe.temperature')} />}
    {method === 'monthly' && <MonthlyValues draft={draft} change={change} path={[...base, 'temperaturesC']}
      label={t('nta.annexP.pipe.monthlyTemperatures')} />}
    {method === 'outdoor_bins' && <>
      {curve.map((_, point) => <div key={point} className="nta-form-row">
        <NumberField {...field} path={[...base, 'curve', point, 'outdoorC']} label={`${t('nta.annexP.pipe.curveOutdoor')} ${point + 1}`} />
        <NumberField {...field} path={[...base, 'curve', point, 'waterC']} label={t('nta.annexP.pipe.curveWater')} />
        <RemoveButton label={t('nta.form.remove')}
          onClick={() => change([...base, 'curve'], curve.filter((__, other) => other !== point))} />
      </div>)}
      <button type="button" onClick={() => change([...base, 'curve'], [...curve, { outdoorC: null, waterC: null }])}>
        {t('nta.annexP.pipe.addCurvePoint')}
      </button>
      <NumberField {...field} path={[...base, 'offAboveOutdoorC']} label={t('nta.annexP.pipe.offAbove')} />
    </>}
  </>;
}

/** Indirectly heated vessels or buffers (P.43/P.44) at `base`. */
export function StorageVesselsFields({ draft, change, base, label, addLabel }: SectionProps & {
  base: Path; label: string; addLabel: string;
}) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const vessels = list(draft, base);
  return <>
    {vessels.map((_, index) => {
      const at = (...rest: Path): Path => [...base, index, ...rest];
      return <fieldset key={index} className="nta-form-row">
        <legend>{label} {index + 1}</legend>
        <NumberField {...field} path={at('volumeL')} label={t('nta.annexP.vessel.volume')} />
        <NumberField {...field} path={at('surfaceM2')} label={t('nta.annexP.vessel.surface')} />
        <CheckField {...field} path={at('diameterAtLeast50Cm')} label={t('nta.annexP.vessel.diameter50')} />
        <SelectField {...field} path={at('insulation')} label={t('nta.annexP.vessel.insulation')} options={[
          ['none', t('nta.annexP.insulation.none')], ['at_least10_mm', '≥ 10 mm'], ['at_least20_mm', '≥ 20 mm'], ['at_least30_mm', '≥ 30 mm']]} />
        <NumberField {...field} path={at('lossFactorWPerM2k')} label={t('nta.annexP.vessel.lossFactor')} />
        <NumberField {...field} path={at('standbyLossKwhPerDay')} label={t('nta.annexP.vessel.standbyLoss')} />
        <NumberField {...field} path={at('standbyTestDifferenceK')} label={t('nta.annexP.vessel.standbyDifference')} />
        <NumberField {...field} path={at('waterTemperatureC')} label={t('nta.annexP.vessel.waterTemperature')} />
        <NumberField {...field} path={at('ambientTemperatureC')} label={t('nta.annexP.vessel.ambientTemperature')} />
        <RemoveButton label={t('nta.form.remove')} onClick={() => change(base, vessels.filter((__, other) => other !== index))} />
      </fieldset>;
    })}
    <button type="button" onClick={() => change(base, [...vessels, storageVesselTemplate()])}>{addLabel}</button>
  </>;
}

/** P.13–P.18 distribution by pipe segments (`distribution.method = pipes`). */
export function PipeDistributionFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const segments = list(draft, [...base, 'segments']);
  return <>
    {segments.map((_, index) => <fieldset key={index} className="nta-form-row">
      <PipeSegmentFields draft={draft} change={change} base={[...base, 'segments', index]} index={index} />
      <RemoveButton label={t('nta.form.remove')}
        onClick={() => change([...base, 'segments'], segments.filter((__, other) => other !== index))} />
    </fieldset>)}
    <button type="button" onClick={() => change([...base, 'segments'], [...segments, pipeSegmentTemplate()])}>
      {t('nta.annexP.pipe.addSegment')}
    </button>
    <WaterTemperatureFields draft={draft} change={change} base={[...base, 'waterTemperature']} />
    <StorageVesselsFields draft={draft} change={change} base={[...base, 'buffers']} label={t('nta.annexP.buffer')}
      addLabel={t('nta.annexP.addBuffer')} />
    <label>{t('nta.annexP.pipe.coldSupply')}
      <select value={read(draft, [...base, 'supplyBelow10C']) == null ? '' : String(read(draft, [...base, 'supplyBelow10C']))}
        onChange={(event) => change([...base, 'supplyBelow10C'], event.target.value === '' ? null : event.target.value === 'true')}>
        <option value="">{t('nta.annexP.pipe.coldSupply.notCold')}</option>
        <option value="true">{t('nta.annexP.pipe.coldSupply.below10')}</option>
        <option value="false">{t('nta.annexP.pipe.coldSupply.atLeast10')}</option>
      </select>
    </label>
    <NumberField {...field} path={[...base, 'otherLossKwh']} label={t('nta.annexP.otherLoss')} optional />
    <TextField {...field} path={[...base, 'sourceReference']} label={t('nta.form.source')} />
  </>;
}

/** P.35 with P.43–P.46: calculated η_WD;gen;sto (`hotWaterStorage.method = calculated`). */
export function CalculatedHotWaterStorageFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const pipes = list(draft, [...base, 'pipes']);
  const exchanger = read(draft, [...base, 'exchanger']) != null;
  return <>
    <StorageVesselsFields draft={draft} change={change} base={[...base, 'vessels']} label={t('nta.annexP.vessel')}
      addLabel={t('nta.annexP.addVessel')} />
    {pipes.map((_, index) => {
      const at = (...rest: Path): Path => [...base, 'pipes', index, ...rest];
      return <fieldset key={index} className="nta-form-row">
        <legend>{t('nta.annexP.chargingPipe')} {index + 1}</legend>
        <NumberField {...field} path={at('lengthM')} label={t('nta.annexP.pipe.length')} />
        <NumberField {...field} path={at('uValueWPerMk')} label={t('nta.annexP.chargingPipe.u')} />
        <NumberField {...field} path={at('outerDiameterMm')} label={t('nta.annexP.chargingPipe.outerDiameter')} />
        <NumberField {...field} path={at('insulationMm')} label={t('nta.annexP.chargingPipe.insulation')} />
        <NumberField {...field} path={at('ambientTemperatureC')} label={t('nta.annexP.vessel.ambientTemperature')} />
        <RemoveButton label={t('nta.form.remove')}
          onClick={() => change([...base, 'pipes'], pipes.filter((__, other) => other !== index))} />
      </fieldset>;
    })}
    <button type="button" onClick={() => change([...base, 'pipes'], [...pipes, chargingPipeTemplate()])}>
      {t('nta.annexP.addChargingPipe')}
    </button>
    <label className="nta-form-check">
      <input type="checkbox" checked={exchanger} onChange={(event) => change([...base, 'exchanger'], event.target.checked
        ? { nominalPowerKw: null, insulated: false } : null)} />
      {t('nta.annexP.exchanger')}
    </label>
    {exchanger && <>
      <NumberField {...field} path={[...base, 'exchanger', 'nominalPowerKw']} label={t('nta.annexP.nominalPower')} />
      <CheckField {...field} path={[...base, 'exchanger', 'insulated']} label={t('nta.annexP.exchanger.insulated')} />
      <NumberField {...field} path={[...base, 'exchanger', 'specificLossWPerKw']} label={t('nta.annexP.exchanger.specificLoss')} />
    </>}
    <NumberField {...field} path={[...base, 'circulationTemperatureC']} label={t('nta.annexP.circulationTemperature')} />
    <NumberField {...field} path={[...base, 'correctionFactor']} label={t('nta.annexP.storageCorrection')} />
    <TextField {...field} path={[...base, 'sourceReference']} label={t('nta.form.source')} />
  </>;
}
