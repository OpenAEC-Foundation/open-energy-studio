import { describe, expect, it } from 'vitest';
import { previewValue } from '../core/nta/SurveyTakeover';
import { withoutStaleGroundWall } from '../components/BasisopnamePanel/BasisopnamePanel';

// Review 28: cross-feature frontend checks of the changes since release 0.3.0.
describe('review 28', () => {
  it('shows a number in the takeover preview in the UI language', () => {
    expect(previewValue(38.5, 'nl')).toBe('38,5');
    expect(previewValue(1234.25, 'nl')).toBe('1.234,25');
    expect(previewValue(38.5, 'en')).toBe('38.5');
    // Objects and text stay as they were; without a locale a number stays JSON.
    expect(previewValue({ depthM: 1.2 }, 'nl')).toBe('{"depthM":1.2}');
    expect(previewValue('abc', 'nl')).toBe('abc');
    expect(previewValue(38.5)).toBe('38.5');
  });

  it('drops H1 and the floor of a ground wall once the surface is no longer a façade against the ground', () => {
    const wall = { id: 'kelderwand', element: 'facade', boundary: { kind: 'ground' }, depthBelowGroundM: 1.2, groundFloorId: 'vloer' };
    expect(withoutStaleGroundWall(wall)).toEqual(wall);
    const outdoor = withoutStaleGroundWall({ ...wall, boundary: { kind: 'outdoor' } });
    expect('depthBelowGroundM' in outdoor || 'groundFloorId' in outdoor).toBe(false);
    const floor = withoutStaleGroundWall({ ...wall, element: 'floor' });
    expect('depthBelowGroundM' in floor || 'groundFloorId' in floor).toBe(false);
    expect(floor.boundary).toEqual({ kind: 'ground' });
  });
});
