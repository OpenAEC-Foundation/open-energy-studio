import { useState, type ReactElement } from 'react';
import { describe, expect, it } from 'vitest';
import { fireEvent, screen } from '@testing-library/react';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import {
  ConstructionExtrasFields, emptyRequiredSurveyFields, SolarControlWindowsFields, SurveyDoorsFields, SurveyUnheatedSpacesFields,
  SYSTEM_VARIANTS, UtilityInstallationExtrasFields, UtilityLightingFields, VentilationBasicsFields,
} from '../components/BasisopnamePanel/SurveyBasics';
import { SurveyShowerHeatRecoveryField } from '../components/BasisopnamePanel/SurveyCoverageFields';
import { en } from '../i18n/en';
import { renderWithProviders } from './test-utils';

// Survey form results the kernel used to refuse (review 7). The same shapes are read by
// the kernel in crates/nta8800-core/tests/survey_form_shapes.rs.

function Harness({ initial, children }: {
  initial: Draft;
  children: (draft: Draft, change: (path: Path, value: unknown) => void) => ReactElement;
}) {
  const [draft, setDraft] = useState<Draft>(initial);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  return <form aria-label="test">
    {children(draft, change)}
    <output data-testid="draft">{JSON.stringify(draft)}</output>
  </form>;
}

const draft = () => JSON.parse(screen.getByTestId('draft').textContent ?? '{}');
const t = (key: string) => (en as Record<string, string>)[key] ?? key;

describe('survey basics write what the kernel reads', () => {
  it('records whether a known qv10 was measured or laid down in quality assurance (§11.2.5)', () => {
    renderWithProviders(<Harness initial={{}}>
      {(d, change) => <ConstructionExtrasFields draft={d} change={change} t={t} utility={false} />}</Harness>);
    fireEvent.click(screen.getByLabelText('Air permeability q_v10 known (§11.2.5)'));
    expect(draft().measuredInfiltration.basis).toBe('measurement');
    fireEvent.change(screen.getByLabelText('Origin of q_v10'), { target: { value: 'quality_assurance' } });
    fireEvent.change(screen.getByLabelText(/^q_v10/), { target: { value: '0.62' } });
    expect(draft().measuredInfiltration).toMatchObject({ basis: 'quality_assurance', qv10Dm3PerSM2: 0.62 });
  });

  it('asks the connection of a shower heat recovery unit only when there is one (table 13.8)', () => {
    renderWithProviders(<Harness initial={{ hotWater: { showerHeatRecovery: 'none' } }}>
      {(d, change) => <SurveyShowerHeatRecoveryField draft={d} change={change} t={t} />}</Harness>);
    expect(screen.queryByLabelText('Shower heat recovery connection (table 13.8)')).toBeNull();
    fireEvent.change(screen.getByLabelText('Shower heat recovery (table 13.16)'), { target: { value: 'vertical' } });
    fireEvent.change(screen.getByLabelText('Shower heat recovery connection (table 13.8)'), { target: { value: 'heater_only' } });
    expect(draft().hotWater).toMatchObject({ showerHeatRecovery: 'vertical', showerHeatRecoveryConnection: 'heater_only' });
  });

  it('unchecking the last solar-control window leaves an empty list, not null', () => {
    renderWithProviders(<Harness initial={{ envelope: { windows: [{ id: 'raam1' }] }, solarControlWindowIds: ['raam1'] }}>
      {(d, change) => <SolarControlWindowsFields draft={d} change={change} t={t} />}</Harness>);
    fireEvent.click(screen.getByLabelText('raam1'));
    expect(draft().solarControlWindowIds).toEqual([]);
  });

  it('adds unheated spaces with fresh ids and resets a surface that bordered a removed one', () => {
    renderWithProviders(<Harness initial={{ envelope: { surfaces: [{ id: 'gevel', boundary: { kind: 'unheated_space', spaceId: 'onverwarmd-1' } }],
      unheatedSpaces: [{ id: 'onverwarmd-1', description: 'berging' }] } }}>
      {(d, change) => <SurveyUnheatedSpacesFields draft={d} change={change} t={t} />}</Harness>);
    fireEvent.click(screen.getByText('Add unheated space'));
    expect(draft().envelope.unheatedSpaces.map((space: Draft) => space.id)).toEqual(['onverwarmd-1', 'onverwarmd-2']);
    fireEvent.click(screen.getAllByText('Remove')[0]);
    expect(draft().envelope.unheatedSpaces).toEqual([{ id: 'onverwarmd-2', description: '' }]);
    expect(draft().envelope.surfaces[0].boundary).toEqual({ kind: 'outdoor' });
  });

  it('asks the glass type of a door with a glass share, with its own area label and fresh ids', () => {
    renderWithProviders(<Harness initial={{ envelope: { surfaces: [{ id: 'gevel' }], doors: [] } }}>
      {(d, change) => <SurveyDoorsFields draft={d} change={change} t={t} />}</Harness>);
    fireEvent.click(screen.getByText('Add door'));
    fireEvent.click(screen.getByText('Add door'));
    fireEvent.click(screen.getAllByText('Remove')[0]);
    fireEvent.click(screen.getByText('Add door'));
    expect(draft().envelope.doors.map((door: Draft) => door.id)).toEqual(['deur-2', 'deur-3']);
    expect(screen.getAllByLabelText(/^Door area/)).toHaveLength(2);
    expect(screen.queryByLabelText('Glass type in the door')).toBeNull();
    fireEvent.change(screen.getAllByLabelText('Glass share of the door')[0], { target: { value: '0.4' } });
    fireEvent.change(screen.getByLabelText('Glass type in the door'), { target: { value: 'hr_plus_plus' } });
    expect(draft().envelope.doors[0]).toMatchObject({ glassFraction: 0.4, glass: 'hr_plus_plus' });
  });

  it('offers the table 11.5 variants as a choice and leaves the field out when cleared', () => {
    renderWithProviders(<Harness initial={{ ventilation: { principle: 'balanced' } }}>
      {(d, change) => <VentilationBasicsFields draft={d} change={change} t={t} />}</Harness>);
    const variant = screen.getByLabelText(/table 11.5 variant/) as HTMLSelectElement;
    expect([...variant.options].map((option) => option.value).slice(1)).toEqual(SYSTEM_VARIANTS);
    fireEvent.change(variant, { target: { value: 'c4a' } });
    expect(draft().ventilation.declaredVariant).toBe('c4a');
    fireEvent.change(variant, { target: { value: '' } });
    expect('declaredVariant' in draft().ventilation).toBe(false);
  });

  it('gives new lighting zones fresh ids and leaves the daylight default out', () => {
    renderWithProviders(<Harness initial={{ lighting: [] }}>
      {(d, change) => <UtilityLightingFields draft={d} change={change} t={t} />}</Harness>);
    fireEvent.click(screen.getByText('Add lighting zone'));
    fireEvent.click(screen.getByText('Add lighting zone'));
    fireEvent.click(screen.getAllByText('Remove')[0]);
    fireEvent.click(screen.getByText('Add lighting zone'));
    const zones = draft().lighting as Draft[];
    expect(zones.map((zone) => zone.id)).toEqual(['verlichting-2', 'verlichting-3']);
    expect((screen.getAllByLabelText('Daylight control')[0] as HTMLSelectElement).value).toBe('none');
    expect(zones.every((zone) => !('daylight' in zone))).toBe(true);
  });

  it('only asks the recirculation percentage while recirculation is present, and clears it otherwise', () => {
    renderWithProviders(<Harness initial={{ ventilation: { principle: 'balanced', recirculation: 'present_percent_unknown',
      recirculationPercent: 30 }, heatingInstallation: { collective: false } }}>
      {(d, change) => <UtilityInstallationExtrasFields draft={d} change={change} t={t} />}</Harness>);
    expect(screen.getByLabelText(/Proven recirculation percentage/)).toBeTruthy();
    fireEvent.change(screen.getByLabelText('Recirculation'), { target: { value: 'none' } });
    expect(draft().ventilation.recirculation).toBe('none');
    expect('recirculationPercent' in draft().ventilation).toBe(false);
    expect(screen.queryByLabelText(/Proven recirculation percentage/)).toBeNull();
  });

  it('names empty required numbers instead of sending them to the kernel', () => {
    expect(emptyRequiredSurveyFields({
      measuredInfiltration: { qv10Dm3PerSM2: null, sourceReference: '' },
      lighting: [{ id: 'a', areaM2: null, power: { method: 'lamps', lamps: [{ count: 1, lampPowerW: null }] } }],
      envelope: { surfaces: [{ id: 'g', boundary: { kind: 'unheated_space', spaceId: '' } }] },
    })).toEqual(['measuredInfiltration.qv10Dm3PerSM2', 'lighting[0].areaM2', 'lighting[0].power.lamps[0].lampPowerW',
      'envelope.surfaces[0].boundary.spaceId']);
    expect(emptyRequiredSurveyFields({ lighting: [{ id: 'a', areaM2: 10, power: { method: 'unknown' } }] })).toEqual([]);
  });
});
