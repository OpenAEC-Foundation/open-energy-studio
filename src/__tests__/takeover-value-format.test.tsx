import { useEffect } from 'react';
import { describe, expect, it } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import type { IProject } from '../core/energy/types';
import type { BuildingPerformanceInput, OpnameAssessment } from '../core/nta/KernelClient';
import { formatTakeoverNumber, groundValueLines } from '../core/nta/SurveyTakeover';
import { SurveyTakeoverAction } from '../components/BasisopnamePanel/SurveyTakeoverAction';
import { evidenceLinkLabelsEn, evidenceLinkLabelsNl } from '../i18n/evidenceLinkLabels';
import terraced from '../../training-data/nta8800-example-terraced-dwelling.json';
import fixture from '../../training-data/nta8800-survey-takeover-ground.json';
import { renderWithProviders } from './test-utils';

// The manual screenshots showed R_c 2,986667 and raw JSON for a heated
// basement in "Vloeren op de grond". Values now round per member and objects
// read as one labelled line per member.

const nlLabel = (key: string) => evidenceLinkLabelsNl[`opname.takeover.ground.${key}`];

describe('takeover values for floors to the ground', () => {
  it('rounds R_c, P, areas and depths to two decimals and Ψ and U to three, in the UI language', () => {
    expect(formatTakeoverNumber(2.9866666666666664, 'nl', 'constructionResistanceM2kPerW')).toBe('2,99');
    expect(formatTakeoverNumber(38, 'nl', 'exposedPerimeterM')).toBe('38');
    expect(formatTakeoverNumber(1234.567, 'nl')).toBe('1.234,57');
    expect(formatTakeoverNumber(0.14007575, 'en', 'forfaitDeltaUWPerM2k')).toBe('0.14');
    expect(formatTakeoverNumber(0.0425, 'en', 'psiWPerMk')).toBe('0.043');
  });

  it('reads a heated basement as one labelled line per member', () => {
    const lines = groundValueLines(fixture.surveyFloor.heatedBasement, 'nl', nlLabel, 'heatedBasement');
    expect(lines).toEqual([
      { label: 'diepte z (m)', text: '1,2' },
      { label: 'ΔU_for (W/(m²·K))', text: '0,14' },
      { label: 'wandoppervlakte ΣA_T;bw (m²)', text: '24' },
      { label: 'R_c wand (m²K/W)', text: '4,5' },
    ]);
  });

  it('translates enum values, numbers list items and leaves the source reference out', () => {
    expect(groundValueLines({ method: 'forfait' }, 'nl', nlLabel)).toEqual([{ label: 'methode', text: 'forfaitair' }]);
    expect(groundValueLines([], 'nl', nlLabel)).toEqual([{ text: 'geen' }]);
    const edge = groundValueLines([{ kind: 'horizontal', resistanceM2kPerW: 2.5, thicknessM: 0.5, sourceReference: 'x' }], 'nl', nlLabel);
    expect(edge).toEqual([{ label: '1', text: 'soort horizontaal; R (m²K/W) 2,5; dikte (m) 0,5' }]);
    const below = groundValueLines({ kind: 'crawlspace', depthClass: 'on_sand', wallResistanceM2kPerW: 0.3, wallUValueWPerM2k: 2.1234 }, 'nl', nlLabel);
    expect(below.map((line) => line.text)).toEqual(['kruipruimte', 'op zand', '0,3', '2,123']);
  });

  it('has an nl and en label for every member and value it names', () => {
    const nl = Object.keys(evidenceLinkLabelsNl).filter((key) => key.startsWith('opname.takeover.ground.'));
    const en = Object.keys(evidenceLinkLabelsEn).filter((key) => key.startsWith('opname.takeover.ground.'));
    expect(new Set(nl)).toEqual(new Set(en));
  });
});

function Seed({ value }: { value: IProject }) {
  const { dispatch } = useEnergy();
  useEffect(() => { dispatch({ type: 'SET_PROJECT', payload: structuredClone(value) }); }, [dispatch, value]);
  return null;
}

describe('takeover dialog values', () => {
  it('shows rounded numbers and readable basement members, not JSON', async () => {
    const value = structuredClone(terraced) as unknown as IProject;
    for (const zone of value.zones) for (const surface of zone.surfaces) if (surface.id === 'floor') surface.area = fixture.surveyFloor.areaM2;
    const assessment = {
      status: 'calculated_unverified', scope: 'residential', source: 'ISSO 82.1', appliedDefaults: [], warnings: [], issues: [],
      derivedInput: { spaceHeating: { demand: { transmission: { groundFloors: [fixture.surveyFloor] } } } } as unknown as BuildingPerformanceInput,
      performance: null, referenceVerified: false,
    } as unknown as OpnameAssessment;
    renderWithProviders(<><Seed value={value} /><SurveyTakeoverAction result={assessment} /></>);
    await waitFor(() => expect(screen.getByRole('button', { name: /projectmodel|project model/i })).toBeTruthy());
    fireEvent.click(screen.getByRole('button', { name: /projectmodel|project model/i }));
    const ground = within(await screen.findByRole('dialog')).getByRole('region', { name: /grond|ground/i });
    const text = ground.textContent ?? '';
    expect(text).not.toMatch(/2[.,]98666/);
    expect(text).toMatch(/2[.,]99/);
    expect(text).not.toMatch(/\{"|depthM/);
    expect(text).toMatch(/(diepte|depth) z \(m\): 1[.,]2/);
    expect(within(ground).getAllByRole('rowheader').length).toBeGreaterThan(0);
  });
});
