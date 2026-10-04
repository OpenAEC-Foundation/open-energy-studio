import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { nl } from '../i18n/nl';
import { en } from '../i18n/en';
import { KERNEL_CODE_PREFIXES } from '../i18n/format';
import { emittedCodes } from './kernelCodeExtraction';

const labelled = (table: Record<string, string>, code: string) =>
  KERNEL_CODE_PREFIXES.some((prefix) => typeof table[`${prefix}${code}`] === 'string');

describe('kernel code labels', () => {
  const codes = emittedCodes(join(__dirname, '..', '..', 'crates'));

  it('finds the kernel codes, including the forms the first extractor missed', () => {
    expect(codes.size).toBeGreaterThan(1000);
    // push helpers, multi-line `issue(Severity, "…")`, tuple pushes and `unwrap_or("…")`.
    for (const code of ['pv_tilt_invalid', 'pv_peak_power_invalid', 'system_record_invalid', 'project_name_required',
      'boiler_efficiency_invalid', 'booster_test_invalid', 'regeneration_efficiency_invalid', 'micro_chp_efficiency_invalid',
      'obstruction_geometry_invalid', 'hot_water_mixed_air_invalid', 'ground_floor_invalid', 'annex_u_run_invalid',
      'zone_multiple_heating_systems', 'residential_capacity_above25']) {
      expect(codes.has(code), code).toBe(true);
    }
    // Internal survey rule ids (`recorder.record("rule", …)`) and JSON enum values are not codes.
    for (const notCode of ['bypass_table_11_12', 'gas_boiler', 'table16_1', 'electric_resistance']) {
      expect(codes.has(notCode), notCode).toBe(false);
    }
  });

  it('every emitted code has a Dutch and an English label', () => {
    const missingNl = [...codes].filter(([code]) => !labelled(nl, code)).map(([code, file]) => `${code} (${file})`);
    const missingEn = [...codes].filter(([code]) => !labelled(en, code)).map(([code, file]) => `${code} (${file})`);
    expect(missingNl).toEqual([]);
    expect(missingEn).toEqual([]);
  });
});
