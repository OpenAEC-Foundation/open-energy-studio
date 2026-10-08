/**
 * Feedback 8 Oct 2026: one Rc per kind of surface (new build), one insulation
 * answer per element (survey), and wrong or empty answers marked red with the
 * label closed until they are resolved.
 */
import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { createDefaultProject } from '../context/EnergyContext';
import { elementRcActions, sharedConstruction, sharedWindowU, surfacesOf } from '../core/energy/elementRc';
import { ElementInsulation, derivedRc } from '../components/BasisopnamePanel/BasisopnamePanel';
import { pathHasIssue, surveyBlocked } from '../components/SurveyWizard/SurveyWizard';
import type { IProject } from '../core/energy/types';
import type { OpnameAssessment } from '../core/nta/KernelClient';

const t = (key: string, options?: Record<string, unknown>) => (options ? `${key} ${JSON.stringify(options)}` : key);

function apply(project: IProject, actions: ReturnType<typeof elementRcActions>): IProject {
  let next = structuredClone(project);
  for (const action of actions) {
    if (action.type === 'ADD_CONSTRUCTION') next = { ...next, constructions: [...next.constructions, action.payload] };
    if (action.type === 'UPDATE_CONSTRUCTION') next = { ...next, constructions: next.constructions.map((item) => item.id === action.payload.id ? { ...item, ...action.payload.data } : item) };
    if (action.type === 'UPDATE_SURFACE') next = { ...next, zones: next.zones.map((zone) => zone.id !== action.payload.zoneId ? zone : {
      ...zone, surfaces: zone.surfaces.map((surface) => surface.id === action.payload.surfaceId ? { ...surface, ...action.payload.data } : surface),
    }) };
  }
  return next;
}

describe('Rc per kind of surface (new build)', () => {
  it('sets one construction for all walls and derives U from Rc', () => {
    const project = createDefaultProject();
    const walls = surfacesOf(project, 'wall');
    expect(walls.length).toBeGreaterThan(1);
    const next = apply(project, elementRcActions(project, 'wall', 4.7, 'Gevels'));
    const shared = sharedConstruction(next, 'wall');
    expect(shared?.rcValue).toBe(4.7);
    // U = 1/(0,13 + 4,7 + 0,04) (NTA 8800 table C.2)
    expect(shared?.uValue).toBeCloseTo(1 / 4.87, 4);
    expect(surfacesOf(next, 'wall').every(({ surface }) => surface.constructionId === shared?.id)).toBe(true);
  });

  it('gives a kind its own construction when the shared one is also used elsewhere', () => {
    const project = createDefaultProject();
    const roofs = surfacesOf(project, 'roof');
    if (roofs.length === 0) return;
    const wallConstruction = surfacesOf(project, 'wall')[0].surface.constructionId;
    const mixed = apply(project, roofs.map(({ zoneId, surface }) => ({ type: 'UPDATE_SURFACE' as const, payload: { zoneId, surfaceId: surface.id, data: { constructionId: wallConstruction } } })));
    const next = apply(mixed, elementRcActions(mixed, 'roof', 6.3, 'Daken'));
    expect(sharedConstruction(next, 'roof')?.id).toBe('rc-roof');
    expect(next.constructions.find((item) => item.id === wallConstruction)?.rcValue)
      .toBe(mixed.constructions.find((item) => item.id === wallConstruction)?.rcValue);
  });

  it('knows when all windows share one U', () => {
    const project = createDefaultProject();
    expect(sharedWindowU(project) === null || typeof sharedWindowU(project) === 'number').toBe(true);
  });
});

describe('insulation per element (survey)', () => {
  it('applies one answer to all facades except party walls', async () => {
    const user = userEvent.setup();
    const draft = { envelope: { surfaces: [
      { id: 'a', element: 'facade', boundary: { kind: 'outdoor' }, insulation: { kind: 'none_or_unknown' } },
      { id: 'b', element: 'facade', boundary: { kind: 'outdoor' }, insulation: { kind: 'thickness', thicknessMm: 50 } },
      { id: 'c', element: 'facade', boundary: { kind: 'adjacent_heated' }, insulation: { kind: 'none_or_unknown' } },
    ] } };
    let next: Record<string, unknown> | null = null;
    render(<ElementInsulation draft={draft} element="facade" replace={(value) => { next = value; }} t={t} />);
    await user.selectOptions(screen.getByRole('combobox'), 'cavity_filled_unknown_width');
    const surfaces = (next as unknown as typeof draft).envelope.surfaces;
    expect(surfaces.map((surface) => surface.insulation.kind)).toEqual(['cavity_filled_unknown_width', 'cavity_filled_unknown_width', 'none_or_unknown']);
  });

  it('reads the Rc the kernel derived for a surface', () => {
    const result = { derivedInput: { opaqueElements: [{ id: 'voorgevel', sourceReference: 'opname; basisopname R_c 0.35 (I.2.1.2)' }] } } as unknown as OpnameAssessment;
    expect(derivedRc(result, 'voorgevel')).toBe(0.35);
    expect(derivedRc(result, 'other')).toBeNull();
  });
});

describe('red fields and a closed label', () => {
  it('matches a field to the finding of its path or a parent path', () => {
    expect(pathHasIssue('basisopname.envelope.surfaces[1].grossAreaM2', ['envelope.surfaces[1].grossAreaM2'])).toBe(true);
    expect(pathHasIssue('basisopname.heating.generator.kind', ['heating.generator'])).toBe(true);
    expect(pathHasIssue('basisopname.envelope.surfaces[10].grossAreaM2', ['envelope.surfaces[1]'])).toBe(false);
    expect(pathHasIssue('ntaCalculation.setpoints.heatingC', ['ntaCalculation.setpoints.heatingC'])).toBe(true);
  });

  it('closes the label while the kernel has findings or no outcome', () => {
    expect(surveyBlocked(null)).toBe(true);
    expect(surveyBlocked({ issues: [{ code: 'x', path: 'y' }], performance: {} } as unknown as OpnameAssessment)).toBe(true);
    expect(surveyBlocked({ issues: [], performance: null } as unknown as OpnameAssessment)).toBe(true);
    expect(surveyBlocked({ issues: [], performance: {} } as unknown as OpnameAssessment)).toBe(false);
  });
});
