import { useState, type ReactElement } from 'react';
import { describe, expect, it } from 'vitest';
import { fireEvent, screen } from '@testing-library/react';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import {
  PvMountingFields, SurfaceGeometryFields, SurveyGeneralCoverageFields, SurveyHeatingBalanceFields, SurveyHotWaterServedFields,
  SurveyPanelsFields, SurveyShowerHeatRecoveryField, UtilityCoolingCoverageFields, UtilityHotWaterStorageFields,
  VentilationBypassField, WindowDeclaredObstructionFields,
} from '../components/BasisopnamePanel/SurveyCoverageFields';
import { en } from '../i18n/en';
import { renderWithProviders } from './test-utils';

// Survey answers that only the JSON view reached before (docs/ui-redesign/formulierdekking.md).
// The kernel reads the same shapes in crates/nta8800-core/tests/survey_form_shapes.rs
// (`form_coverage_answers_are_read_by_both_surveys`).

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
const select = (label: RegExp | string, value: string) => fireEvent.change(screen.getByLabelText(label), { target: { value } });
const type = (label: RegExp | string, value: string) => fireEvent.change(screen.getByLabelText(label), { target: { value } });
const list = (label: RegExp | string, value: string) => {
  const input = screen.getByLabelText(label);
  fireEvent.change(input, { target: { value } });
  fireEvent.blur(input);
};

describe('survey answers that had no form', () => {
  it('writes the area source and offers storage only with PV (storage_requires_pv)', () => {
    const { unmount } = renderWithProviders(<Harness initial={{}}>
      {(d, change) => <SurveyGeneralCoverageFields draft={d} change={change} t={t} />}</Harness>);
    type(/Source of the usable floor area/, 'meetrapport NEN 2580');
    expect(draft().areaSourceReference).toBe('meetrapport NEN 2580');
    expect(screen.queryByLabelText(/Building-bound storage/)).toBeNull();
    unmount();

    renderWithProviders(<Harness initial={{ pv: [{ id: 'pv1' }] }}>
      {(d, change) => <SurveyGeneralCoverageFields draft={d} change={change} t={t} />}</Harness>);
    fireEvent.click(screen.getByLabelText(/Building-bound storage/));
    expect(draft().storage).toEqual({ electricalKwh: 0, thermalKwh: 0, sourceReference: '' });
    fireEvent.click(screen.getByLabelText(/Building-bound storage/));
    expect(draft().storage).toBeUndefined();
  });

  it('writes orientation, tilt, cavity and the exposed perimeter of a ground floor', () => {
    const base: Path = ['envelope', 'surfaces', 0];
    renderWithProviders(<Harness initial={{ envelope: { surfaces: [
      { id: 'gevel', element: 'facade', boundary: { kind: 'outdoor' }, cavity: false },
      { id: 'vloer', element: 'floor', boundary: { kind: 'ground' }, cavity: false }] } }}>
      {(d, change) => <>
        <SurfaceGeometryFields draft={d} change={change} t={t} base={base} />
        <SurfaceGeometryFields draft={d} change={change} t={t} base={['envelope', 'surfaces', 1]} />
      </>}</Harness>);
    select('Orientation of the surface', 'south_west');
    fireEvent.click(screen.getAllByLabelText('Cavity in the surface')[0]);
    type(/Perimeter of the floor/, '16');
    const [facade, floor] = draft().envelope.surfaces;
    expect(facade).toMatchObject({ orientation: 'south_west', cavity: true });
    expect(floor.exposedPerimeterM).toBe(16);
    // A floor has no orientation question; an empty choice leaves the field out.
    expect(screen.getAllByLabelText('Orientation of the surface')).toHaveLength(1);
    select('Orientation of the surface', '');
    expect(draft().envelope.surfaces[0].orientation).toBeUndefined();
  });

  it('asks H1 and the floor for a façade part against the ground (ISSO 82.1 §8.4)', () => {
    renderWithProviders(<Harness initial={{ envelope: { surfaces: [
      { id: 'kelderwand', element: 'facade', boundary: { kind: 'ground' }, cavity: false },
      { id: 'keldervloer', element: 'floor', boundary: { kind: 'ground' }, cavity: false },
      { id: 'dak', element: 'roof', boundary: { kind: 'outdoor' }, cavity: false }] } }}>
      {(d, change) => <SurfaceGeometryFields draft={d} change={change} t={t} base={['envelope', 'surfaces', 0]} />}</Harness>);
    type(/Height H1/, '2.4');
    select('Floor this façade part against the ground belongs to', 'keldervloer');
    expect(draft().envelope.surfaces[0]).toMatchObject({ depthBelowGroundM: 2.4, groundFloorId: 'keldervloer' });
    // Only floors to the ground, a crawlspace or an unheated cellar are offered.
    const options = [...(screen.getByLabelText('Floor this façade part against the ground belongs to') as HTMLSelectElement).options]
      .map((option) => option.value);
    expect(options).toEqual(['', 'keldervloer']);
    select('Floor this façade part against the ground belongs to', '');
    expect(draft().envelope.surfaces[0].groundFloorId).toBeUndefined();
  });

  it('adds panels with fresh ids and the kernel insulation shape', () => {
    renderWithProviders(<Harness initial={{ envelope: { surfaces: [{ id: 'gevel' }], panels: [] } }}>
      {(d, change) => <SurveyPanelsFields draft={d} change={change} t={t} />}</Harness>);
    fireEvent.click(screen.getByText('Add panel'));
    fireEvent.click(screen.getByText('Add panel'));
    fireEvent.change(screen.getAllByLabelText('Insulation of the panel')[0], { target: { value: 'known_thickness' } });
    const panels = draft().envelope.panels;
    expect(panels.map((panel: Draft) => panel.id)).toEqual(['paneel-1', 'paneel-2']);
    expect(panels[0]).toMatchObject({ surfaceId: 'gevel', insulation: { kind: 'known_thickness', thicknessMm: 50 }, cavity: false });
  });

  it('declares twelve obstruction factors and drops the situation they exclude', () => {
    const base: Path = ['envelope', 'windows', 0];
    renderWithProviders(<Harness initial={{ envelope: { windows: [{ id: 'raam', shading: { situation: 'minimal' } }] } }}>
      {(d, change) => <WindowDeclaredObstructionFields draft={d} change={change} t={t} base={base} />}</Harness>);
    fireEvent.click(screen.getByLabelText(/Declared obstruction factors/));
    let window = draft().envelope.windows[0];
    expect(window.shading).toBeUndefined();
    expect(window.obstruction.heating).toHaveLength(12);
    list(/Obstruction factors for heating/, '0,9; 0,9; 0,9; 0,9; 0,9; 0,9; 0,9; 0,9; 0,9; 0,9; 0,9; 0,9');
    window = draft().envelope.windows[0];
    expect(window.obstruction.heating).toEqual(Array(12).fill(0.9));
  });

  it('writes the heating balance and storeys, leaving empty storeys to the default', () => {
    renderWithProviders(<Harness initial={{ heating: {} }}>
      {(d, change) => <SurveyHeatingBalanceFields draft={d} change={change} t={t} />}</Harness>);
    select('Heating hydronically balanced', 'true');
    type(/Storeys served by the heating distribution/, '2');
    expect(draft().heating).toEqual({ balanced: true, storeys: 2 });
    type(/Storeys served by the heating distribution/, '');
    expect(draft().heating.storeys).toBeUndefined();
  });

  it('writes the PV mounting and one declared obstruction factor instead of a situation', () => {
    const base: Path = ['pv', 0];
    renderWithProviders(<Harness initial={{ pv: [{ id: 'pv1', mounting: 'moderately_ventilated', shading: { method: 'minimal' } }] }}>
      {(d, change) => <PvMountingFields draft={d} change={change} t={t} base={base} />}</Harness>);
    select(/Ventilation behind the PV modules/, 'strongly_ventilated');
    fireEvent.click(screen.getByLabelText(/Declared obstruction factors F_sh;obst/));
    list('Obstruction factors F_sh;obst', '0,85');
    expect(draft().pv[0]).toEqual({ id: 'pv1', mounting: 'strongly_ventilated', obstructionFactors: [0.85] });
  });

  it('asks the bypass only for balanced ventilation', () => {
    const { unmount } = renderWithProviders(<Harness initial={{ ventilation: { principle: 'natural' } }}>
      {(d, change) => <VentilationBypassField draft={d} change={change} t={t} />}</Harness>);
    expect(screen.queryByLabelText(/Bypass/)).toBeNull();
    unmount();
    renderWithProviders(<Harness initial={{ ventilation: { principle: 'balanced' } }}>
      {(d, change) => <VentilationBypassField draft={d} change={change} t={t} />}</Harness>);
    select(/Bypass on the heat recovery/, 'false');
    expect(draft().ventilation.bypassPresent).toBe(false);
  });

  it('writes the taps served, their pipe lengths and the shower heat recovery', () => {
    renderWithProviders(<Harness initial={{ hotWater: { showerHeatRecovery: 'none' } }}>
      {(d, change) => <>
        <SurveyHotWaterServedFields draft={d} change={change} t={t} />
        <SurveyShowerHeatRecoveryField draft={d} change={change} t={t} />
      </>}</Harness>);
    select('Taps the generator serves', 'bathroom_only');
    expect(screen.queryByLabelText(/to the kitchen/)).toBeNull();
    type(/to the bathroom/, '6');
    select('Shower heat recovery (table 13.16)', 'horizontal');
    expect(draft().hotWater).toEqual({ showerHeatRecovery: 'horizontal', served: 'bathroom_only', bathroomLengthM: 6 });
  });

  it('writes the utility cooling answers in the kernel enum codes', () => {
    // The design temperature and pipe insulation belong to a water distribution (review 8).
    renderWithProviders(<Harness initial={{ cooling: { generator: 'compression', waterBased: true } }}>
      {(d, change) => <UtilityCoolingCoverageFields draft={d} change={change} t={t} />}</Harness>);
    select('Design temperature of the cooling emission', 't12_to18');
    select('Control of the cooling', 'central_with_room_control');
    select('Cooling pipes insulated', 'true');
    expect(draft().cooling).toEqual({ generator: 'compression', waterBased: true, designTemperature: 't12_to18',
      control: 'central_with_room_control', pipesInsulated: true });
  });

  it('writes the utility circulation and vessels with fresh ids', () => {
    renderWithProviders(<Harness initial={{ hotWater: { storage: [] } }}>
      {(d, change) => <UtilityHotWaterStorageFields draft={d} change={change} t={t} />}</Harness>);
    select('Circulation pipe present', 'true');
    fireEvent.click(screen.getByText('Add vessel'));
    select(/Energy label of the vessel/, 'b');
    select('Connection pipes of the vessel', 'fully_insulated');
    select('Vessel manufactured from 2018', 'true');
    expect(draft().hotWater).toEqual({ circulation: true, storage: [{ id: 'vat-1', volumeL: 150, sourceReference: '', label: 'b',
      connection: 'fully_insulated', producedFrom2018: true }] });
  });
});
