import { useI18n } from '../../i18n/i18n';
import { NumberField, read, SelectField, TextField, type Draft, type Path } from './NtaFormFields';

// §16 PV system fields (16.2–16.4b, tables 16.1/16.2), shared by the
// building's own PV systems and annex P area electricity (P.7/P.71).

const MODULE_TYPES: Array<[string, string]> = [
  ['monocrystalline_from2018', 'mono ≥ 2018'], ['monocrystalline2015_to2017', 'mono 2015–2017'],
  ['monocrystalline2011_to2014', 'mono 2011–2014'], ['monocrystalline2001_to2010', 'mono 2001–2010'],
  ['monocrystalline_before2001', 'mono < 2001'], ['multicrystalline_from2018', 'multi ≥ 2018'],
  ['multicrystalline2015_to2017', 'multi 2015–2017'], ['multicrystalline2011_to2014', 'multi 2011–2014'],
  ['multicrystalline2001_to2010', 'multi 2001–2010'], ['multicrystalline_before2001', 'multi < 2001'],
  ['amorphous_single_junction', 'a-Si'], ['amorphous_multi_junction', 'a-Si multi'], ['cigs', 'CIGS'], ['cd_te', 'CdTe']];

/** The `peakPower` block for a 16.4a/16.4b route; switching drops the other route's fields. */
export function peakPowerTemplate(method: string): Draft {
  switch (method) {
    case 'table16_1': return { method, moduleType: null, panelAreaM2: null };
    case 'declared_specific': return { method, peakPowerWPerM2: null, panelAreaM2: null };
    default: return { method: 'panels', panelPeakPowerW: null, panelCount: null };
  }
}

/** One PV system at `base`; `label` prefixes the first field (the system id). */
export function PvSystemFields({ draft, change, base, label }: {
  draft: Draft; change: (path: Path, value: unknown) => void; base: Path; label: string;
}) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const at = (...rest: Path): Path => [...base, ...rest];
  const method = read(draft, at('peakPower', 'method'));
  return <>
    <SelectField {...field} path={at('peakPower', 'method')} label={`${label} — ${t('nta.form.pvPeak')}`}
      options={[['panels', t('nta.form.pvPeak.panels')], ['declared_specific', t('nta.form.pvPeak.declared')], ['table16_1', t('nta.form.pvPeak.table')]]}
      onChange={(_, value) => change(at('peakPower'), peakPowerTemplate(String(value ?? 'panels')))} />
    {method === 'panels' ? <>
      <NumberField {...field} path={at('peakPower', 'panelPeakPowerW')} label={t('nta.form.pvPanelPower')} />
      <NumberField {...field} path={at('peakPower', 'panelCount')} label={t('nta.form.pvPanelCount')} step="1" />
    </> : <>
      {method === 'table16_1'
        ? <SelectField {...field} path={at('peakPower', 'moduleType')} label={t('nta.form.pvModuleType')} options={MODULE_TYPES} />
        : <NumberField {...field} path={at('peakPower', 'peakPowerWPerM2')} label={t('nta.form.pvSpecificPeak')} />}
      <NumberField {...field} path={at('peakPower', 'panelAreaM2')} label={t('nta.form.pvArea')} />
    </>}
    <NumberField {...field} path={at('azimuthDeg')} label={t('nta.form.pvAzimuth')} />
    <NumberField {...field} path={at('tiltDeg')} label={t('nta.form.tilt')} />
    <SelectField {...field} path={at('mounting')} label={t('nta.form.pvPerformance')}
      options={[['unknown', t('nta.form.pvMount.unknown')], ['not_ventilated', '0,76'], ['moderately_ventilated', '0,80'], ['strongly_ventilated', '0,82']]} />
    <NumberField {...field} path={at('obstructionFactors', 0)} label={t('nta.form.obstruction')} />
    <TextField {...field} path={at('sourceReference')} label={t('nta.form.source')} />
  </>;
}
