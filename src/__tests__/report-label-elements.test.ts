import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { LABEL_ELEMENT, LABEL_TOKEN, labelElementValue } from '../core/report/EnergyPerformanceReport';

const kernel = (file: string) => readFileSync(resolve(__dirname, '../../crates/nta8800-core/src', file), 'utf8');
const camel = (snake: string) => snake.replace(/_([a-z0-9])/g, (_, letter: string) => letter.toUpperCase());

// The fields of `LabelElements` (label_data.rs), serialized in camelCase.
function labelElementKeys(): string[] {
  const source = kernel('label_data.rs');
  const body = source.slice(source.indexOf('pub struct LabelElements {'));
  const struct = body.slice(0, body.indexOf('\n}'));
  return [...struct.matchAll(/pub ([a-z0-9_]+):/g)].map((match) => camel(match[1]));
}

// Carrier and renewable-source tokens the kernel can put in the label elements.
function labelTokens(): string[] {
  const label = kernel('label_data.rs');
  const sources = [...label.matchAll(/\("([a-z_]+)", by\./g)].map((match) => match[1]);
  const building = kernel('building_performance.rs');
  const codes = building.slice(building.indexOf('fn code(self) -> &\'static str'));
  const carrierCodes = [...codes.slice(0, codes.indexOf('\n    }\n')).matchAll(/=> "([a-z_]+)"/g)].map((match) => match[1]);
  const literal = [...building.matchAll(/carrier: "([a-z_]+)"/g)].map((match) => match[1]);
  return [...new Set([...sources, ...carrierCodes, ...literal])];
}

describe('label elements in the report (Omgevingsregeling art. 5.13a)', () => {
  it('names every label element the kernel emits', () => {
    const statements = new Set(['respondsToExternalSignals', 'lowTemperatureHeating']);
    const keys = labelElementKeys().filter((key) => !statements.has(key));
    expect(keys.length).toBeGreaterThan(8);
    for (const key of keys) expect(LABEL_ELEMENT[key], key).toBeTruthy();
    for (const label of Object.values(LABEL_ELEMENT)) expect(label).not.toMatch(/^[a-z]+[A-Z]/);
  });

  it('translates every carrier and source token', () => {
    const tokens = labelTokens();
    expect(tokens).toEqual(expect.arrayContaining(['gas', 'electricity', 'el']));
    for (const token of tokens) {
      expect(LABEL_TOKEN[token], token).toBeTruthy();
      expect(labelElementValue('mainEnergyCarrier', token)).not.toMatch(/_|^[a-z]{2}$/);
    }
  });

  it('shows annual kWh as whole kWh and other numbers with one decimal', () => {
    expect(labelElementValue('renewableProductionKwh', 1600.24)).toBe('1600');
    expect(labelElementValue('operationalCo2KgPerM2', 12.345)).toBe('12,3');
  });
});
