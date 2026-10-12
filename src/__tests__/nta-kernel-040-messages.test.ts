import { describe, expect, it } from 'vitest';
import { nl } from '../i18n/nl';
import { en } from '../i18n/en';
import { kernelCodeLabel } from '../i18n/format';
import { kernelCodeLabelsEn, kernelCodeLabelsNl } from '../i18n/kernelCodeLabels';
import { hasExplicitRoute, routeForPath } from '../core/nta/gapRoutes';

// Messages and "Ga naar" routes of what kernel 0.4.0 added: the PV K_pk bounds
// (16.4 sets no maximum) and the declared F_D;dayl daylight sector.
const translate = (table: Record<string, string>) => (key: string) => table[key] ?? key;
const nlAll = { ...nl, ...kernelCodeLabelsNl };
const enAll = { ...en, ...kernelCodeLabelsEn };

describe('kernel 0.4.0 messages', () => {
  it('names the PV plausibility warning and tells the adviser what to check', () => {
    for (const [table, word] of [[nlAll, 'datablad'], [enAll, 'data sheet']] as const) {
      const label = kernelCodeLabel(translate(table), 'plausibility_pv_peak_power', ['registration.issue.']);
      expect(label.known).toBe(true);
      expect(label.text).toContain('300 W/m²');
      expect(label.text).toContain(word);
    }
  });

  it('states the 400 W/m² refusal, not the old 300', () => {
    for (const table of [nlAll, enAll]) {
      const label = kernelCodeLabel(translate(table), 'pv_peak_power_invalid');
      expect(label.text).toContain('400 W/m²');
      expect(label.text).not.toContain('300');
    }
    expect(nl['nta.form.pvSpecificPeak']).toMatch(/300.*400/);
    expect(en['nta.form.pvSpecificPeak']).toMatch(/300.*400/);
  });

  it('covers a declared F_D;dayl in the daylight-sector refusal', () => {
    expect(kernelCodeLabel(translate(nlAll), 'lighting_daylight_sector_invalid').text).toContain('F_D;dayl');
    expect(kernelCodeLabel(translate(enAll), 'lighting_daylight_sector_invalid').text).toContain('F_D;dayl');
    // The declared factor includes the daylight control, so the form offers none.
    expect(nl['nta.light.sector.declaredFactor']).toContain('daglichtregeling');
    expect(en['nta.light.sector.declaredFactor']).toContain('daylight control');
  });

  it('routes the PV warning and a declared sector to their form sections', () => {
    expect(hasExplicitRoute('pvSystems[0].peakPower.peakPowerWPerM2')).toBe(true);
    expect(routeForPath('pvSystems[0].peakPower.peakPowerWPerM2').focusPath).toContain('pvSystems[0]');
    expect(hasExplicitRoute('lighting[0].lightingZones[0].daylight.sectors[0].sourceReference')).toBe(true);
  });
});
