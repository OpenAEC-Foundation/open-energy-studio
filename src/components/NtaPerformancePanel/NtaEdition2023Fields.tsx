import type { ReactNode } from 'react';
import { useI18n } from '../../i18n/i18n';
import { CheckField, NumberField, read, SelectField, type Draft, type Path } from './NtaFormFields';

// Inputs that only NTA 8800:2023 has. Under another edition a value left
// behind is offered for removal (the kernel reports route_not_in_edition),
// as for the table 13.2 kitchen pipe diameter.

type Change = (path: Path, value: unknown) => void;

function Only2023({ draft, change, path, label, children }:
  { draft: Draft; change: Change; path: Path; label: string; children: ReactNode }) {
  const { t } = useI18n();
  if (read(draft, ['normVersion']) === '2023') return <>{children}</>;
  const left = read(draft, path);
  if (left == null || left === false) return null;
  return <p className="nta-form-note nta-form-error" role="alert">
    {t('ntaStep.staleEdition2023', { field: label })}{' '}
    <button type="button" onClick={() => change(path, undefined)}>{t('nta.form.remove')}</button>
  </p>;
}

const options = (t: (key: string) => string, prefix: string, keys: readonly string[]): Array<[string, string]> =>
  keys.map((key) => [key, t(`${prefix}.${key}`)]);

const KINDS = ['radiators', 'surface', 'dwelling_air', 'electric_air', 'ventilation_air', 'high_room'] as const;
const KIND_TEMPLATES: Record<string, Record<string, unknown>> = {
  radiators: { type: 'radiators', control: 'room', overTemperature: 'unknown', position: 'unknown' },
  surface: { type: 'surface', control: 'room', system: 'floor_wet_or_unknown', insulation: 'unknown' },
  dwelling_air: { type: 'dwelling_air', control: 'room' },
  electric_air: { type: 'electric_air', wall: 'unknown', control: 'unknown' },
  ventilation_air: { type: 'ventilation_air', configuration: 'reheat_unknown' },
  high_room: { type: 'high_room', heightM: 6, emitter: 'warm_air_from_ceiling', control: 'controlled' },
};

/** Heating emission per tables 9.2–9.10 of NTA 8800:2023 (`emission.edition2023`). */
export function HeatingEmission2023Fields({ draft, change }: { draft: Draft; change: Change }) {
  const { t } = useI18n();
  const base: Path = ['emission', 'edition2023'];
  const value = read(draft, base);
  const kind = read(draft, [...base, 'kind', 'type']) as string | undefined;
  const field = { draft, onChange: change };
  const at = (...rest: Array<string | number>): Path => [...base, ...rest];
  const p = 'ntaStep.em23';
  return <Only2023 draft={draft} change={change} path={base} label={t(`${p}.title`)}>
    <label className="nta-form-check">
      <input type="checkbox" checked={value != null} onChange={(event) => change(base, event.target.checked
        ? { kind: KIND_TEMPLATES.radiators, pipeSystem: 'unknown', balancing: 'none_or_unknown', roomAutomation: 'unknown' } : undefined)} />
      {t(`${p}.title`)}
    </label>
    {value != null && <>
      <SelectField {...field} path={at('kind', 'type')} label={t(`${p}.kind`)} options={options(t, `${p}.kind`, KINDS)}
        onChange={(_, next) => change(at('kind'), KIND_TEMPLATES[String(next)] ?? KIND_TEMPLATES.radiators)} />
      {(kind === 'radiators' || kind === 'surface' || kind === 'dwelling_air') &&
        <SelectField {...field} path={at('kind', 'control')} label={t(`${p}.control`)} options={options(t, `${p}.control`, ['central', 'room'])} />}
      {kind === 'radiators' && <>
        <SelectField {...field} path={at('kind', 'overTemperature')} label={t(`${p}.overTemperature`)} options={options(t, `${p}.overTemperature`,
          ['mechanical_ventilation', 'fan_assisted', 'local_heater', 'two_pipe60_k_or_unknown', 'two_pipe42_k', 'two_pipe30_k', 'two_pipe20_k',
            'one_pipe60_k_or_unknown', 'one_pipe42_k', 'unknown'])} />
        <SelectField {...field} path={at('kind', 'position')} label={t(`${p}.position`)} options={options(t, `${p}.position`,
          ['inner_wall', 'outer_wall_glass_without_protection', 'outer_wall_glass_with_protection', 'outer_wall', 'unknown'])} />
      </>}
      {kind === 'surface' && <>
        <SelectField {...field} path={at('kind', 'system')} label={t(`${p}.surface`)} options={options(t, `${p}.surface`,
          ['mechanical_ventilation', 'floor_wet_or_unknown', 'floor_dry', 'floor_thin_screed', 'wall', 'ceiling', 'unknown'])} />
        <SelectField {...field} path={at('kind', 'insulation')} label={t(`${p}.insulation`)} options={options(t, `${p}.insulation`,
          ['without_insulation', 'minimal_insulation', 'double_insulation', 'unknown'])} />
      </>}
      {kind === 'electric_air' && <>
        <SelectField {...field} path={at('kind', 'wall')} label={t(`${p}.wall`)} options={options(t, `${p}.wall`, ['outer_wall', 'inner_wall', 'unknown'])} />
        <SelectField {...field} path={at('kind', 'control')} label={t(`${p}.control`)} options={options(t, `${p}.electric`,
          ['p_per_zone', 'central_with_local_p', 'p_per_room', 'pi_per_room', 'unknown'])} />
      </>}
      {kind === 'ventilation_air' && <SelectField {...field} path={at('kind', 'configuration')} label={t(`${p}.configuration`)}
        options={options(t, `${p}.configuration`, ['reheat_room_air', 'reheat_cascade', 'reheat_extract_air', 'reheat_unknown', 'recirculation'])} />}
      {kind === 'high_room' && <>
        <NumberField {...field} path={at('kind', 'heightM')} label={t(`${p}.height`)} unit="m" />
        <SelectField {...field} path={at('kind', 'emitter')} label={t(`${p}.emitter`)} options={options(t, `${p}.emitter`,
          ['warm_air_horizontal', 'warm_air_horizontal_low_temperature', 'warm_air_from_ceiling', 'warm_air_from_ceiling_low_temperature',
            'recirculation_two_step', 'recirculation_pi', 'dark_radiators', 'high_temperature_radiators', 'ceiling_panels',
            'floor_uninsulated_spacing_up_to20_cm', 'floor_uninsulated_spacing_above20_cm', 'floor_minimal_insulation_up_to10_cm',
            'floor_minimal_insulation_above10_cm', 'floor_thermally_decoupled', 'floor_unknown'])} />
        <SelectField {...field} path={at('kind', 'control')} label={t(`${p}.control`)} options={options(t, `${p}.highControl`, ['not_controlled', 'controlled'])} />
        {(read(draft, at('kind', 'emitter')) === 'dark_radiators' || read(draft, at('kind', 'emitter')) === 'high_temperature_radiators') && <>
          <NumberField {...field} path={at('kind', 'radiant', 'specificPowerWPerM2')} label={t(`${p}.specificPower`)} unit="W/m²" />
          <NumberField {...field} path={at('kind', 'radiant', 'radiationFactor')} label={t(`${p}.radiationFactor`)} optional />
        </>}
      </>}
      <CheckField {...field} path={at('certifiedControl')} label={t(`${p}.certified`)} />
      <SelectField {...field} path={at('roomAutomation')} label={t(`${p}.roomAutomation`)} options={options(t, `${p}.roomAutomation`,
        ['unknown', 'individual_per_room', 'individual_with_manual_override', 'network_with_override_and_adaptive'])} />
      <SelectField {...field} path={at('pipeSystem')} label={t(`${p}.pipeSystem`)} options={options(t, `${p}.pipeSystem`,
        ['one_pipe', 'two_pipe', 'unknown', 'not_hydronic'])} />
      <SelectField {...field} path={at('balancing')} label={t(`${p}.balancing`)} options={options(t, `${p}.balancing`,
        ['none_or_unknown', 'static', 'static_or_dynamic_with_groups', 'dynamic_with_load_control', 'dynamic_full'])} />
    </>}
  </Only2023>;
}

/** Cooling emission per tables 10.2–10.5 of NTA 8800:2023 (`<cooling>.emission.edition2023`). */
export function CoolingEmission2023Fields({ draft, change, base }: { draft: Draft; change: Change; base: Path }) {
  const { t } = useI18n();
  const path: Path = [...base, 'edition2023'];
  const value = read(draft, path);
  const field = { draft, onChange: change };
  const p = 'ntaStep.cool23';
  return <Only2023 draft={draft} change={change} path={path} label={t(`${p}.title`)}>
    <label className="nta-form-check">
      <input type="checkbox" checked={value != null} onChange={(event) => change(path, event.target.checked
        ? { control: 'room', balancing: 'none_or_unknown', roomAutomation: 'unknown' } : undefined)} />
      {t(`${p}.title`)}
    </label>
    {value != null && <>
      <SelectField {...field} path={[...path, 'control']} label={t(`${p}.control`)} options={options(t, `${p}.control`, ['central', 'p_before1988', 'room'])} />
      <CheckField {...field} path={[...path, 'certifiedControl']} label={t('ntaStep.em23.certified')} />
      <SelectField {...field} path={[...path, 'balancing']} label={t(`${p}.balancing`)} options={options(t, `${p}.balancing`,
        ['none_or_unknown', 'static_per_emitter', 'static_with_group_balancing', 'static_with_dynamic_groups', 'dynamic_or_direct_expansion'])} />
      <SelectField {...field} path={[...path, 'roomAutomation']} label={t(`${p}.roomAutomation`)} options={options(t, `${p}.roomAutomation`,
        ['unknown', 'standalone', 'standalone_with_manual_override', 'network_with_override_and_adaptive'])} />
    </>}
  </Only2023>;
}

/** 9.4.3 of NTA 8800:2023: f_H;dis;rbl 0,5 for pipes in an uninsulated shell. */
export function DistributionShell2023Field({ draft, change }: { draft: Draft; change: Change }) {
  const { t } = useI18n();
  const path: Path = ['distributionSystem', 'uninsulatedPipesInUninsulatedShell'];
  return <Only2023 draft={draft} change={change} path={path} label={t('ntaStep.dist23.shell')}>
    <CheckField draft={draft} onChange={change} path={path} label={t('ntaStep.dist23.shell')} />
  </Only2023>;
}
