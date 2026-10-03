import { useI18n } from '../../i18n/i18n';
import { CheckField, NumberField, read, SelectField, TextField, type Draft, type Path } from './NtaFormFields';

// Chapter 14 details of one lighting zone: several luminaire groups (14.8/
// 14.9 with table 14.2), the parasitic power of 14.10–14.12 and the daylight
// sectors of 14.25–14.42 (vertical windows, rooflights and annex Y tilted
// windows). Field names follow lighting.rs.

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

const LAMP_TECHNOLOGIES = ['led', 'no_separate_ballast', 'fluorescent_t5', 'fluorescent_t8_electronic', 'fluorescent_t8_conventional',
  'fluorescent_t12', 'compact_fluorescent_not_integrated', 'unknown_or_other'] as const;

export function luminaireGroupTemplate(): Draft {
  return { count: null, power: { method: 'system', powerW: null } };
}

export function daylightSectorTemplate(kind: string): Draft {
  const control = 'automatic_switching_or_unknown';
  switch (kind) {
    case 'rooflights':
      return { kind, rooflightDepthM: null, rooflightWidthM: null, clearHeightM: null, wallDistancesM: [null, null, null, null],
        openingAreaM2: null, roomLengthM: null, roomWidthM: null, luminaireHeightM: null, control };
    case 'tilted_window':
      return { kind, tiltDeg: null, windowLengthM: null, slopeOffsetM: null, slopeStartHeightM: null, ceilingHeightM: null,
        windowWidthM: null, control };
    default:
      return { kind: 'vertical_windows', sectorWidthM: null, lintelHeightM: null, actualDepthM: null, openingAreaM2: null,
        heavilyShaded: false, control };
  }
}

/** 14.8/14.9: the luminaire groups of `power` (method installed). */
export function LuminaireGroupsFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const groups = list(draft, [...base, 'luminaires']);
  return <>
    {groups.map((_, index) => {
      const at = (...rest: Path): Path => [...base, 'luminaires', index, ...rest];
      const method = read(draft, at('power', 'method'));
      return <div key={index} className="nta-form-row">
        <NumberField {...field} path={at('count')} label={`${t('nta.light.group')} ${index + 1} — ${t('nta.light.count')}`} step="1" />
        <SelectField {...field} path={at('power', 'method')} label={t('nta.light.groupPower')} options={[
          ['system', t('nta.light.groupPower.system')], ['lamps', t('nta.light.groupPower.lamps')]]}
          onChange={(_, value) => change(at('power'), value === 'lamps'
            ? { method: 'lamps', lampPowerW: null, lampCount: null, technology: null }
            : { method: 'system', powerW: null })} />
        {method === 'lamps' ? <>
          <NumberField {...field} path={at('power', 'lampPowerW')} label={t('nta.light.lampPower')} />
          <NumberField {...field} path={at('power', 'lampCount')} label={t('nta.light.lampCount')} step="1" />
          <SelectField {...field} path={at('power', 'technology')} label={t('nta.light.technology')}
            options={LAMP_TECHNOLOGIES.map((key) => [key, t(`nta.light.technology.${key}`)])} />
        </> : <NumberField {...field} path={at('power', 'powerW')} label={t('nta.light.systemPower')} />}
        <RemoveButton label={t('nta.form.remove')}
          onClick={() => change([...base, 'luminaires'], groups.filter((__, other) => other !== index))} />
      </div>;
    })}
    <button type="button" onClick={() => change([...base, 'luminaires'], [...groups, luminaireGroupTemplate()])}>
      {t('nta.light.addGroup')}
    </button>
    <NumberField {...field} path={[...base, 'dynamicFactor']} label={t('nta.light.dynamicFactor')} />
  </>;
}

/** 14.14 forfait or 14.10–14.12 installed parasitic power at `base`. */
export function ParasiticPowerFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const method = read(draft, [...base, 'method']);
  return <>
    <SelectField {...field} path={[...base, 'method']} label={t('nta.light.parasitic')} options={[
      ['forfait', t('nta.light.parasitic.forfait')], ['installed', t('nta.light.parasitic.installed')]]}
      onChange={(_, value) => change(base, value === 'installed'
        ? { method: 'installed', emergencyChargingW: null, controlStandbyW: null, sourceReference: '' } : { method: 'forfait' })} />
    {method === 'installed' && <>
      <NumberField {...field} path={[...base, 'emergencyChargingW']} label={t('nta.light.parasitic.emergency')} />
      <NumberField {...field} path={[...base, 'controlStandbyW']} label={t('nta.light.parasitic.control')} />
      <TextField {...field} path={[...base, 'sourceReference']} label={t('nta.form.source')} />
    </>}
  </>;
}

const CONTROL_OPTIONS = (t: (key: string) => string): Array<[string, string]> => [
  ['none_or_manual', t('nta.light.sector.control.manual')],
  ['automatic_switching_or_unknown', t('nta.light.sector.control.switching')],
  ['automatic_dimming', t('nta.light.sector.control.dimming')]];

/** Four wall distances of a rooflight projection (depth sides, width sides). */
function WallDistances({ draft, change, path }: SectionProps & { path: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  return <>
    {[0, 1, 2, 3].map((side) => <NumberField key={side} {...field} path={[...path, side]}
      label={`${t('nta.light.sector.wallDistance')} ${side + 1}`} />)}
  </>;
}

/** One daylight sector at `base` (vertical windows, rooflights or an annex Y tilted window). */
function DaylightSectorFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const kind = read(draft, [...base, 'kind']);
  const at = (...rest: Path): Path => [...base, ...rest];
  const vertical = read(draft, at('vertical')) != null;
  const rooflight = read(draft, at('rooflight')) != null;
  return <>
    <SelectField {...field} path={at('kind')} label={t('nta.light.sector.kind')} options={[
      ['vertical_windows', t('nta.light.sector.vertical')], ['rooflights', t('nta.light.sector.rooflights')],
      ['tilted_window', t('nta.light.sector.tilted')]]}
      onChange={(_, value) => change(base, daylightSectorTemplate(String(value ?? 'vertical_windows')))} />
    {kind === 'vertical_windows' && <>
      <NumberField {...field} path={at('sectorWidthM')} label={t('nta.light.sector.width')} />
      <NumberField {...field} path={at('lintelHeightM')} label={t('nta.light.sector.lintel')} />
      <NumberField {...field} path={at('actualDepthM')} label={t('nta.light.sector.depth')} />
      <CheckField {...field} path={at('useActualDepthAlternative')} label={t('nta.light.sector.actualDepthAlternative')} />
      <NumberField {...field} path={at('openingAreaM2')} label={t('nta.light.sector.opening')} />
      <CheckField {...field} path={at('heavilyShaded')} label={t('nta.light.sector.shaded')} />
    </>}
    {kind === 'rooflights' && <>
      <NumberField {...field} path={at('rooflightDepthM')} label={t('nta.light.sector.rooflightDepth')} />
      <NumberField {...field} path={at('rooflightWidthM')} label={t('nta.light.sector.rooflightWidth')} />
      <NumberField {...field} path={at('clearHeightM')} label={t('nta.light.sector.clearHeight')} />
      <WallDistances draft={draft} change={change} path={at('wallDistancesM')} />
      <NumberField {...field} path={at('openingAreaM2')} label={t('nta.light.sector.opening')} />
      <NumberField {...field} path={at('roomLengthM')} label={t('nta.light.sector.roomLength')} />
      <NumberField {...field} path={at('roomWidthM')} label={t('nta.light.sector.roomWidth')} />
      <NumberField {...field} path={at('luminaireHeightM')} label={t('nta.light.sector.luminaireHeight')} />
    </>}
    {kind === 'tilted_window' && <>
      <NumberField {...field} path={at('tiltDeg')} label={t('nta.light.sector.tilt')} />
      <NumberField {...field} path={at('windowLengthM')} label={t('nta.light.sector.windowLength')} />
      <NumberField {...field} path={at('windowWidthM')} label={t('nta.light.sector.windowWidth')} />
      <NumberField {...field} path={at('slopeOffsetM')} label={t('nta.light.sector.slopeOffset')} />
      <NumberField {...field} path={at('slopeStartHeightM')} label={t('nta.light.sector.slopeStart')} />
      <NumberField {...field} path={at('ceilingHeightM')} label={t('nta.light.sector.ceiling')} />
      <label className="nta-form-check">
        <input type="checkbox" checked={vertical} onChange={(event) => change(at('vertical'), event.target.checked
          ? { sectorWidthM: null, actualDepthM: null, heavilyShaded: false } : null)} />
        {t('nta.light.sector.verticalProjection')}
      </label>
      {vertical && <>
        <NumberField {...field} path={at('vertical', 'sectorWidthM')} label={t('nta.light.sector.width')} />
        <NumberField {...field} path={at('vertical', 'actualDepthM')} label={t('nta.light.sector.depth')} />
        <CheckField {...field} path={at('vertical', 'useActualDepthAlternative')} label={t('nta.light.sector.actualDepthAlternative')} />
        <CheckField {...field} path={at('vertical', 'heavilyShaded')} label={t('nta.light.sector.shaded')} />
      </>}
      <label className="nta-form-check">
        <input type="checkbox" checked={rooflight} onChange={(event) => change(at('rooflight'), event.target.checked
          ? { wallDistancesM: [null, null, null, null], roomLengthM: null, roomWidthM: null, luminaireHeightM: null } : null)} />
        {t('nta.light.sector.rooflightProjection')}
      </label>
      {rooflight && <>
        <WallDistances draft={draft} change={change} path={at('rooflight', 'wallDistancesM')} />
        <NumberField {...field} path={at('rooflight', 'roomLengthM')} label={t('nta.light.sector.roomLength')} />
        <NumberField {...field} path={at('rooflight', 'roomWidthM')} label={t('nta.light.sector.roomWidth')} />
        <NumberField {...field} path={at('rooflight', 'luminaireHeightM')} label={t('nta.light.sector.luminaireHeight')} />
      </>}
    </>}
    <SelectField {...field} path={at('control')} label={t('nta.light.sector.control')} options={CONTROL_OPTIONS(t)} />
  </>;
}

/** 14.25–14.42: the daylight sectors of `daylight` (method sectors). */
export function DaylightSectorsFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const sectors = list(draft, [...base, 'sectors']);
  return <>
    {sectors.map((_, index) => <fieldset key={index} className="nta-form-row">
      <legend>{t('nta.light.sector')} {index + 1}</legend>
      <DaylightSectorFields draft={draft} change={change} base={[...base, 'sectors', index]} />
      <RemoveButton label={t('nta.form.remove')}
        onClick={() => change([...base, 'sectors'], sectors.filter((__, other) => other !== index))} />
    </fieldset>)}
    <button type="button" onClick={() => change([...base, 'sectors'], [...sectors, daylightSectorTemplate('vertical_windows')])}>
      {t('nta.light.addSector')}
    </button>
    <TextField {...field} path={[...base, 'sourceReference']} label={t('nta.form.source')} />
  </>;
}
