import { describe, expect, it } from 'vitest';
import { orderResults } from '../components/MaatwerkadviesPanel/MaatwerkadviesPanel';
import type { MwaVariantResult } from '../core/nta/KernelClient';

const result = (id: string, payback: number | null, npv: number | null) =>
  ({ id, simplePaybackYears: payback, netPresentValueEur: npv } as unknown as MwaVariantResult);

describe('maatwerkadvies result order', () => {
  const rows = [result('a', 12, 500), result('b', 6, -200), result('c', null, 900)];
  it('keeps the input order by default (ISSO 82.2 §6.2.4 prescribes no ranking)', () => {
    expect(orderResults(rows, 'input').map((item) => item.id)).toEqual(['a', 'b', 'c']);
  });
  it('sorts by payback time, shortest first, unknown last', () => {
    expect(orderResults(rows, 'payback').map((item) => item.id)).toEqual(['b', 'a', 'c']);
  });
  it('sorts by net present value, highest first', () => {
    expect(orderResults(rows, 'npv').map((item) => item.id)).toEqual(['c', 'a', 'b']);
  });
});
