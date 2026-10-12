import { useState, type ReactElement } from 'react';
import { describe, expect, it } from 'vitest';
import { act, fireEvent, screen, within } from '@testing-library/react';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import {
  parseNumberList, PvMountingFields, SurfaceGeometryFields, SurveyGeneralCoverageFields, SurveyHeatingBalanceFields,
  SurveyHotWaterServedFields, SurveyPanelsFields, SurveyShowerHeatRecoveryField, UtilityCoolingCoverageFields,
  UtilityHotWaterStorageFields, VentilationBypassField, WindowDeclaredObstructionFields,
} from '../components/BasisopnamePanel/SurveyCoverageFields';
import { emptyRequiredSurveyFields } from '../components/BasisopnamePanel/SurveyBasics';
import { en } from '../i18n/en';
import nullTolerant from '../../training-data/nta8800-survey-null-tolerant-paths.json';
import { renderWithProviders } from './test-utils';

// Review 8: an emptied required number or a "—" choice wrote null, and the kernel refused the whole
// survey with a bare serde message. This test empties every number field and picks every empty
// choice in each survey coverage section, and requires that every null left in the survey is either
// named by emptyRequiredSurveyFields (so "Opname doorrekenen" names the field first) or is an answer
// the kernel reads as "not answered" (training-data/nta8800-survey-null-tolerant-paths.json, checked
// against the kernel in crates/nta8800-core/tests/survey_form_shapes.rs).

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

const t = (key: string) => (en as Record<string, string>)[key] ?? key;
const draft = () => JSON.parse(screen.getByTestId('draft').textContent ?? '{}') as Draft;

/** Every path that holds null, as `a.b[0].c`. */
function nullPaths(value: unknown, prefix = ''): string[] {
  if (value === null) return [prefix];
  if (Array.isArray(value)) return value.flatMap((item, index) => nullPaths(item, `${prefix}[${index}]`));
  if (typeof value === 'object') {
    return Object.entries(value as Record<string, unknown>)
      .flatMap(([key, item]) => nullPaths(item, prefix === '' ? key : `${prefix}.${key}`));
  }
  return [];
}

const pattern = (path: string) => path.replace(/\[\d+\]/g, '[]');

/** Empties every number field and picks every empty choice in the form. */
function emptyEverything() {
  const form = screen.getByRole('form', { name: 'test' });
  for (const input of within(form).queryAllByRole('spinbutton')) {
    fireEvent.change(input, { target: { value: '' } });
    fireEvent.blur(input);
  }
  for (const selectElement of form.querySelectorAll('select')) {
    if (Array.from(selectElement.options).some((option) => option.value === '')) {
      act(() => { fireEvent.change(selectElement, { target: { value: '' } }); });
    }
  }
}

function check(route: 'residentialSurvey' | 'utilitySurvey') {
  const survey = draft();
  const reported = new Set(emptyRequiredSurveyFields(survey));
  const tolerant = new Set((nullTolerant as Record<string, unknown>)[route] as string[]);
  const unexplained = nullPaths(survey).filter((path) => !reported.has(path) && !tolerant.has(pattern(path)));
  expect(unexplained).toEqual([]);
}

const sections: Array<{ name: string; route: 'residentialSurvey' | 'utilitySurvey'; initial: Draft; render: (d: Draft, c: (path: Path, value: unknown) => void) => ReactElement }> = [
  {
    name: 'area source and storage', route: 'residentialSurvey',
    initial: { pv: [{ id: 'pv-1' }], storage: { electricalKwh: 5, thermalKwh: 2, sourceReference: '' } },
    render: (d, c) => <SurveyGeneralCoverageFields draft={d} change={c} t={t} />,
  },
  {
    name: 'surface geometry', route: 'residentialSurvey',
    initial: { envelope: { surfaces: [{ id: 'vloer', element: 'floor', boundary: { kind: 'ground' }, tiltDeg: 0, exposedPerimeterM: 20 }] } },
    render: (d, c) => <SurfaceGeometryFields draft={d} change={c} t={t} base={['envelope', 'surfaces', 0]} />,
  },
  {
    name: 'façade part against the ground', route: 'residentialSurvey',
    initial: { envelope: { surfaces: [
      { id: 'kelderwand', element: 'facade', boundary: { kind: 'ground' }, depthBelowGroundM: 2.5, groundFloorId: 'keldervloer' },
      { id: 'keldervloer', element: 'floor', boundary: { kind: 'ground' }, exposedPerimeterM: 20 },
    ] } },
    render: (d, c) => <SurfaceGeometryFields draft={d} change={c} t={t} base={['envelope', 'surfaces', 0]} />,
  },
  {
    name: 'panels', route: 'residentialSurvey',
    initial: { envelope: { surfaces: [{ id: 'gevel' }], panels: [{
      id: 'paneel-1', surfaceId: 'gevel', areaM2: 2, insulation: { kind: 'known_thickness', thicknessMm: 40 },
      cavity: false, frame: 'metal', sourceReference: '',
    }] } },
    render: (d, c) => <SurveyPanelsFields draft={d} change={c} t={t} />,
  },
  {
    name: 'declared window obstruction', route: 'residentialSurvey',
    initial: { envelope: { windows: [{ id: 'raam', obstruction: { heating: [1], cooling: [1], sourceReference: '' } }] } },
    render: (d, c) => <WindowDeclaredObstructionFields draft={d} change={c} t={t} base={['envelope', 'windows', 0]} />,
  },
  {
    name: 'heating balance', route: 'residentialSurvey', initial: { heating: { balanced: true, storeys: 2 } },
    render: (d, c) => <SurveyHeatingBalanceFields draft={d} change={c} t={t} />,
  },
  {
    name: 'PV mounting', route: 'residentialSurvey',
    initial: { pv: [{ id: 'pv-1', mounting: 'not_ventilated', obstructionFactors: [0.9] }] },
    render: (d, c) => <PvMountingFields draft={d} change={c} t={t} base={['pv', 0]} />,
  },
  {
    name: 'bypass (system E)', route: 'residentialSurvey',
    initial: { ventilation: { principle: 'natural_supply_mechanical_extract', combined: {}, bypassPresent: true } },
    render: (d, c) => <VentilationBypassField draft={d} change={c} t={t} />,
  },
  {
    name: 'taps served', route: 'residentialSurvey', initial: { hotWater: { served: 'bathroom_only', bathroomLengthM: 4 } },
    render: (d, c) => <SurveyHotWaterServedFields draft={d} change={c} t={t} />,
  },
  {
    name: 'shower heat recovery', route: 'residentialSurvey', initial: { hotWater: { showerHeatRecovery: 'vertical' } },
    render: (d, c) => <SurveyShowerHeatRecoveryField draft={d} change={c} t={t} />,
  },
  {
    name: 'utility cooling', route: 'utilitySurvey',
    initial: { cooling: { waterBased: true, designTemperature: 't12_to16', control: 'standalone', pipesInsulated: true } },
    render: (d, c) => <UtilityCoolingCoverageFields draft={d} change={c} t={t} />,
  },
  {
    name: 'utility hot-water vessels', route: 'utilitySurvey',
    initial: { hotWater: { circulation: true, storage: [{
      id: 'vat-1', volumeL: 150, label: 'b', producedFrom2018: true, connection: 'uninsulated', inHeatedZone: true, sourceReference: '',
    }] } },
    render: (d, c) => <UtilityHotWaterStorageFields draft={d} change={c} t={t} />,
  },
];

describe('survey forms never leave an unexplained null (review 8)', () => {
  for (const section of sections) {
    it(`empties every field of ${section.name}`, () => {
      renderWithProviders(<Harness initial={section.initial}>{section.render}</Harness>);
      emptyEverything();
      check(section.route);
    });
  }

  it('names the required numbers it found empty', () => {
    expect(emptyRequiredSurveyFields({
      envelope: { panels: [{ id: 'p', surfaceId: 'g', areaM2: null, frame: 'metal', insulation: { kind: 'known_thickness', thicknessMm: null } }] },
      hotWater: { storage: [{ id: 'v', volumeL: null }] },
    })).toEqual([
      'envelope.panels[0].areaM2', 'envelope.panels[0].insulation.thicknessMm', 'hotWater.storage[0].volumeL',
    ]);
  });
});

describe('number lists (review 8)', () => {
  it('drops empty parts instead of reading them as 0, and leaves an empty field out', () => {
    expect(parseNumberList('')).toBeUndefined();
    expect(parseNumberList('  ')).toBeUndefined();
    expect(parseNumberList('1; 0,9;')).toEqual([1, 0.9]);
    expect(parseNumberList('1;;0,8')).toEqual([1, 0.8]);
  });

  it('removes the PV obstruction factors when the field is emptied', () => {
    renderWithProviders(<Harness initial={{ pv: [{ id: 'pv-1', obstructionFactors: [0.9] }] }}>
      {(d, c) => <PvMountingFields draft={d} change={c} t={t} base={['pv', 0]} />}
    </Harness>);
    const input = screen.getByLabelText(en['opname.pv.obstructionFactors.values']);
    fireEvent.change(input, { target: { value: '' } });
    fireEvent.blur(input);
    expect((draft().pv as Draft[])[0].obstructionFactors).toBeUndefined();
  });
});

describe('fields shown only where the kernel reads them (review 8)', () => {
  it('shows the bypass for a combined system (E) and for balanced ventilation only', () => {
    const { unmount } = renderWithProviders(<Harness initial={{ ventilation: { principle: 'natural_supply_mechanical_extract', combined: {} } }}>
      {(d, c) => <VentilationBypassField draft={d} change={c} t={t} />}
    </Harness>);
    expect(screen.getByLabelText(en['opname.ventilation.bypass'])).toBeTruthy();
    unmount();
    renderWithProviders(<Harness initial={{ ventilation: { principle: 'natural_supply_mechanical_extract' } }}>
      {(d, c) => <VentilationBypassField draft={d} change={c} t={t} />}
    </Harness>);
    expect(screen.queryByLabelText(en['opname.ventilation.bypass'])).toBeNull();
  });

  it('shows the design temperature and pipe insulation only for water-based cooling', () => {
    renderWithProviders(<Harness initial={{ cooling: { waterBased: false } }}>
      {(d, c) => <UtilityCoolingCoverageFields draft={d} change={c} t={t} />}
    </Harness>);
    expect(screen.queryByLabelText(en['opname.cooling.designTemperature'])).toBeNull();
    expect(screen.queryByLabelText(en['opname.cooling.pipesInsulated'])).toBeNull();
    expect(screen.getByLabelText(en['opname.cooling.control'])).toBeTruthy();
  });
});
