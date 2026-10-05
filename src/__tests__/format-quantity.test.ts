import { describe, expect, it } from 'vitest';
import { formatQuantity, parseDecimal, QUANTITIES, NBSP, MINUS } from '../i18n/format';
import { formatKernelPath, isPathWithin, parseKernelPath } from '../core/nta/pathUtil';

describe('formatQuantity (ontwerp.md §5.3)', () => {
  it('formats each quantity with its unit and decimals in Dutch', () => {
    expect(formatQuantity(51.37, 'beng', 'nl')).toBe(`51,37${NBSP}kWh/m²·jr`);
    expect(formatQuantity(51.37, 'bengStatus', 'nl')).toBe(`51,4${NBSP}kWh/m²·jr`);
    expect(formatQuantity(22.31, 'share', 'nl')).toBe(`22,3${NBSP}%`);
    expect(formatQuantity(0.851, 'tojuli', 'nl')).toBe(`0,85${NBSP}K`);
    expect(formatQuantity(10232.4, 'energy', 'nl')).toBe(`10.232${NBSP}kWh`);
    expect(formatQuantity(1015, 'gas', 'nl')).toBe(`1.015${NBSP}m³`);
    expect(formatQuantity(30, 'area', 'nl')).toBe(`30,00${NBSP}m²`);
    expect(formatQuantity(100, 'usableArea', 'nl')).toBe(`100,0${NBSP}m²`);
    expect(formatQuantity(0.21, 'uValue', 'nl')).toBe(`0,210${NBSP}W/m²K`);
    expect(formatQuantity(4.7, 'rc', 'nl')).toBe(`4,70${NBSP}m²K/W`);
    expect(formatQuantity(0.045, 'psi', 'nl')).toBe(`0,045${NBSP}W/mK`);
    expect(formatQuantity(0.4, 'qv10', 'nl')).toBe(`0,40${NBSP}dm³/(s·m²)`);
    expect(formatQuantity(45, 'temperature', 'nl')).toBe(`45${NBSP}°C`);
    expect(formatQuantity(26400, 'money', 'nl')).toBe(`€${NBSP}26.400`);
    expect(formatQuantity(16.06, 'payback', 'nl')).toBe(`16,1${NBSP}jr`);
  });

  it('uses English separators and unit spelling in English', () => {
    expect(formatQuantity(10232.4, 'energy', 'en')).toBe(`10,232${NBSP}kWh`);
    expect(formatQuantity(51.37, 'beng', 'en')).toBe(`51.37${NBSP}kWh/m²·yr`);
    expect(formatQuantity(16.06, 'payback', 'en')).toBe(`16.1${NBSP}yr`);
  });

  it('writes negatives with a typographic minus and never "−0"', () => {
    expect(formatQuantity(-1600, 'energy', 'nl')).toBe(`${MINUS}1.600${NBSP}kWh`);
    expect(formatQuantity(-0.0001, 'beng', 'nl')).toBe(`0,00${NBSP}kWh/m²·jr`);
  });

  it('renders missing values as a dash and can drop the unit', () => {
    expect(formatQuantity(null, 'beng', 'nl')).toBe('–');
    expect(formatQuantity(Number.NaN, 'beng', 'nl')).toBe('–');
    expect(formatQuantity(51.37, 'beng', 'nl', { unit: false })).toBe('51,37');
    expect(formatQuantity(51.37, 'beng', 'nl', { digits: 0 })).toBe(`51${NBSP}kWh/m²·jr`);
  });

  it('defines a unit and decimals for every kind', () => {
    for (const spec of Object.values(QUANTITIES)) {
      expect(Number.isInteger(spec.digits)).toBe(true);
    }
  });
});

describe('parseDecimal', () => {
  it('accepts a comma or a point as decimal separator', () => {
    expect(parseDecimal('0,25')).toBe(0.25);
    expect(parseDecimal('0.25')).toBe(0.25);
    expect(parseDecimal(' 12 ')).toBe(12);
    expect(parseDecimal('-3,5')).toBe(-3.5);
    expect(parseDecimal(`${MINUS}3,5`)).toBe(-3.5);
  });

  it('reads Dutch and English thousands separators by the last separator', () => {
    expect(parseDecimal('1.234,5')).toBe(1234.5);
    expect(parseDecimal('1,234.5')).toBe(1234.5);
  });

  it('returns null for empty input and NaN for text', () => {
    expect(parseDecimal('')).toBeNull();
    expect(parseDecimal('   ')).toBeNull();
    expect(parseDecimal('abc')).toBeNaN();
    expect(parseDecimal('1,2,3')).toBeNaN();
  });
});

describe('parseKernelPath', () => {
  it('splits dot/bracket paths', () => {
    expect(parseKernelPath('ntaCalculation.zoneData[0].ventilation.months[3]'))
      .toEqual(['ntaCalculation', 'zoneData', 0, 'ventilation', 'months', 3]);
  });

  it('splits JSON pointers', () => {
    expect(parseKernelPath('/zones/0/surfaces/2/windows/1/uValue'))
      .toEqual(['zones', 0, 'surfaces', 2, 'windows', 1, 'uValue']);
    expect(parseKernelPath('/a~1b/c~0d')).toEqual(['a/b', 'c~d']);
  });

  it('handles empty and root paths', () => {
    expect(parseKernelPath('')).toEqual([]);
    expect(parseKernelPath('.')).toEqual([]);
    expect(parseKernelPath(undefined)).toEqual([]);
  });

  it('formats back and compares by segment', () => {
    expect(formatKernelPath(['a', 0, 'b'])).toBe('a[0].b');
    expect(isPathWithin('ntaCalculation.zoneData[0].ventilation', 'ntaCalculation.zoneData')).toBe(true);
    expect(isPathWithin('ntaCalculation.zoneDataX', 'ntaCalculation.zoneData')).toBe(false);
  });
});
