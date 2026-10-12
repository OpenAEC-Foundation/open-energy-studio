import { describe, expect, it } from 'vitest';
import { syncVerticalPipes, verticalPipesResidential } from '../core/nta/NtaFormModels';

// Review 9: a vertical pipe keeps only the members of its route (7.17a),
// decided by the zone's use function as in the kernel.
describe('vertical pipes follow the route of their zone', () => {
  it('drops storeys on the utility route and blanks a missing building height', () => {
    const block = {
      calculationScope: 'utility', usageFunction: 'office',
      verticalPipes: [{ id: 'leiding-1', storeys: null, insulated: false, sourceReference: '' }],
    };
    expect(syncVerticalPipes(block).verticalPipes).toEqual([
      { id: 'leiding-1', buildingHeightM: null, insulated: false, sourceReference: '' },
    ]);
  });

  it('drops building height and area share on the residential route', () => {
    const block = {
      calculationScope: 'residential', usageFunction: 'residential',
      verticalPipes: [{ id: 'leiding-1', buildingHeightM: 9, areaShare: 0.5, storeys: 3, insulated: true, sourceReference: 'x' }],
    };
    expect(syncVerticalPipes(block).verticalPipes).toEqual([
      { id: 'leiding-1', storeys: 3, insulated: true, sourceReference: 'x' },
    ]);
  });

  it('follows a zone’s own use function, not the scope', () => {
    const block = {
      calculationScope: 'utility', usageFunction: 'office',
      zoneData: [
        { zoneId: 'z1', verticalPipes: [{ id: 'a', storeys: 2, buildingHeightM: 7 }] },
        { zoneId: 'z2', usageFunction: 'residential', verticalPipes: [{ id: 'b', buildingHeightM: 7 }] },
      ],
    };
    expect(verticalPipesResidential(block, block.zoneData[1])).toBe(true);
    const synced = syncVerticalPipes(block).zoneData as Array<{ verticalPipes: unknown[] }>;
    expect(synced[0].verticalPipes).toEqual([{ id: 'a', buildingHeightM: 7 }]);
    expect(synced[1].verticalPipes).toEqual([{ id: 'b', storeys: null }]);
  });

  it('leaves an unknown or empty list alone', () => {
    expect(syncVerticalPipes({ usageFunction: 'office', verticalPipes: null })).toEqual({ usageFunction: 'office', verticalPipes: null });
    expect(syncVerticalPipes({ usageFunction: 'office', verticalPipes: [] }).verticalPipes).toEqual([]);
  });
});
