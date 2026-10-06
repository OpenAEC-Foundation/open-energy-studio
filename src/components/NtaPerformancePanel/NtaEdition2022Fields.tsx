import type { ReactNode } from 'react';
import { useI18n } from '../../i18n/i18n';
import { CheckField, NumberField, read, SelectField, type Draft, type Path } from './NtaFormFields';

// Inputs that only NTA 8800:2022 has (and 2020+A1, whose kernel profile is
// cumulative on 2022). Under a later edition a value left behind is offered
// for removal; the kernel reports it as route_not_in_edition.

type Change = (path: Path, value: unknown) => void;

/** Editions that have the 2022-only routes. */
export const EDITIONS_WITH_2022_ROUTES: readonly string[] = ['2022', '2020+A1'];

export function hasEdition2022Routes(draft: Draft): boolean {
  return EDITIONS_WITH_2022_ROUTES.includes(read(draft, ['normVersion']) as string);
}

function Only2022({ draft, change, path, label, children }:
  { draft: Draft; change: Change; path: Path; label: string; children: ReactNode }) {
  const { t } = useI18n();
  if (hasEdition2022Routes(draft)) return <>{children}</>;
  const left = read(draft, path);
  if (left == null || left === false) return null;
  return <p className="nta-form-note nta-form-error" role="alert">
    {t('ntaStep.staleEdition2022', { field: label })}{' '}
    <button type="button" onClick={() => change(path, undefined)}>{t('nta.form.remove')}</button>
  </p>;
}

/** (8.47) of NTA 8800:2022 (p. 236): the real wall height h above ground level; required under 2022. */
export function WallHeightAboveGround2022Field({ draft, change, path }: { draft: Draft; change: Change; path: Path }) {
  const { t } = useI18n();
  const label = t('ntaStep.wallHeightAboveGround');
  return <Only2022 draft={draft} change={change} path={path} label={label}>
    <NumberField draft={draft} onChange={change} path={path} label={label} />
  </Only2022>;
}

/** §13.6.3 of NTA 8800:2022 (p. 550): electric boiler with insulated pipes, f_sto;dis;ls = 1,5. */
export function ElectricBoilerInsulatedPipe2022Field({ draft, change, path }: { draft: Draft; change: Change; path: Path }) {
  const { t } = useI18n();
  const label = t('ntaStep.electricBoilerInsulatedPipe');
  return <Only2022 draft={draft} change={change} path={path} label={label}>
    <CheckField draft={draft} onChange={(target, value) => change(target, value === true ? true : undefined)} path={path} label={label} />
  </Only2022>;
}

/** Table 14.4 of NTA 8800:2022 (p. 639): constant-illuminance compensation of a lighting zone. */
export function ConstantIlluminance2022Field({ draft, change, path }: { draft: Draft; change: Change; path: Path }) {
  const { t } = useI18n();
  const label = t('ntaStep.constantIlluminance');
  return <Only2022 draft={draft} change={change} path={path} label={label}>
    <SelectField draft={draft} onChange={(target, value) => change(target, value == null || value === '' ? undefined : value)}
      path={path} label={label}
      options={(['linear_fluorescent', 'led_l80'] as const).map((key) => [key, t(`ntaStep.constantIlluminance.${key}`)])} />
  </Only2022>;
}
