import { describe, expect, it } from 'vitest';
import { createDefaultProject } from '../context/EnergyContext';
import { generateNtaCalculationReportHTML } from '../core/report/NtaCalculationReport';
import type { ProjectPerformanceAssessment } from '../core/nta/KernelClient';

const zone = {
  climateSource: 'NTA 8800 tables 17.1/17.2',
  omittedCorrections: ['7.9.2 intermittent heating reduction a_H;red'],
  specificHeatCapacityKjPerM2k: 180,
  annualHeatingNeedKwh: 3790,
  annualCoolingNeedKwh: 420,
  transmission: { method: 'components', conductanceWPerK: 43.6, groundConductanceWPerK: 9.4 },
  monthly: Array.from({ length: 12 }, () => ({ cooling: { needKwh: 35 } })),
};

function calculated(): ProjectPerformanceAssessment {
  return {
    status: 'calculated_unverified', targetNormVersion: 'NTA 8800:2025+C1:2026', kernelVersion: '0.1.0',
    inputFingerprint: 'sha256:feed', attestStatus: 'unattested', gaps: [], derivedInput: null,
    performance: {
      status: 'calculated_unverified', chapter5Source: 'consultation ch5', issues: [],
      needIndicatorKwhPerM2Year: null, primaryFossilIndicatorKwhPerM2Year: 11.09, renewableSharePercent: 77.7,
      indicativeLabelClass: 'A+++', labelSource: 'Omgevingsregeling bijlage IX',
      annualPrimaryFossilKwh: 1109, annualRenewablePrimaryKwh: 3870, annualHeatPumpAmbientHeatKwh: 0,
      annualHeatingAndCoolingNeedKwh: 4210,
      carriers: Array.from({ length: 12 }, (_, index) => ({ carrier: 'gas', month: index + 1, usedKwh: 300, deliveredKwh: 300 })),
      electricityBalance: Array.from({ length: 12 }, (_, index) => ({ month: index + 1, usedKwh: 30, producedKwh: 200, selfUsedKwh: 30, exportedKwh: 170 })),
      bblCheck: { source: 'Bbl art. 4.149', function: 'other_residential', lossAreaRatio: 2.12,
        limits: { energyNeedMaxKwhPerM2: 78.6, primaryFossilMaxKwhPerM2: 30, renewableShareMinPercent: 50, lightConstructionAllowanceApplied: true },
        energyNeedMeets: null, primaryFossilMeets: true, renewableShareMeets: true },
      tojuli: [{ zoneId: 'z1', status: 'calculated_unverified', activeCooling: false, maxTojuliK: 0.84, meetsBblLimit: true, issues: [],
        orientations: [{ orientation: 'south', areaM2: 82, share: 0.5, assessed: true, conductanceWPerK: 45, coolingNeedJulyKwh: 28, tojuliK: 0.84 }] }],
      tojuliMaxK: 0.84, tojuliMeetsBblLimit: true,
      spaceHeating: {
        chapter9Source: 'consultation ch9', omittedTerms: ['9.2.3 node losses and node gains (including solar thermal)'],
        monthly: Array.from({ length: 12 }, () => ({ heatingNeedKwh: 300 })),
        demand: zone, additionalZoneDemands: [],
      },
    },
  } as unknown as ProjectPerformanceAssessment;
}

describe('NTA calculation report', () => {
  it('states the unverified status and repeats indicators, checks and sources', () => {
    const project = { ...createDefaultProject(), name: 'Woning <test>' };
    const html = generateNtaCalculationReportHTML(project, calculated());
    expect(html).toContain('Onverifieerde berekening — geen officieel energielabel, niet geattesteerd.');
    expect(html).toContain('Woning &lt;test&gt;');
    expect(html).not.toContain('Woning <test>');
    expect(html).toContain('sha256:feed');
    expect(html).toContain('11,09');
    expect(html).toContain('77,7');
    expect(html).toContain('A+++');
    expect(html).toContain('vereist vast ventilatiesysteem C1');
    expect(html).toContain('voldoet (onverifieerd)');
    expect(html).toContain('0,84');
    expect(html).toContain('7.9.2 intermittent heating reduction a_H;red');
    expect(html).toContain('NTA 8800 tables 17.1/17.2');
    expect(html.match(/<tr><th>(jan|feb|mrt|apr|mei|jun|jul|aug|sep|okt|nov|dec)<\/th>/g)).toHaveLength(12);
    expect(html).not.toContain('Ventilatie (hoofdstuk 11)');
  });

  it('shows the EMGforf label RER without falling back to the declaration RER', () => {
    const assessment = calculated();
    const performance = assessment.performance as unknown as Record<string, unknown>;
    performance.labelPrimaryFossilIndicatorKwhPerM2Year = 14.5;
    performance.labelRenewableSharePercent = null;
    const html = generateNtaCalculationReportHTML(createDefaultProject(), assessment);
    const row = html.match(/<tr><th>Labelgegevens \(Reg\. art\. 4\)<\/th>.*?<\/tr>/)?.[0] ?? '';
    expect(row).toContain('EP2 14,50');
    expect(row).toContain('hernieuwbaar — %');
    expect(row).not.toContain('77,7');
  });

  it('reports chapter 11 ventilation and the fixed C1 basis of BENG 1', () => {
    const assessment = calculated();
    const performance = assessment.performance as unknown as Record<string, unknown> & { spaceHeating: { demand: Record<string, unknown> } };
    performance.needIndicatorKwhPerM2Year = 48.12;
    performance.spaceHeating.demand = {
      ...zone,
      recoverableLossesApplied: true,
      fixedC1: { status: 'calculated_unverified', annualHeatingNeedKwh: 4500, annualCoolingNeedKwh: 300 },
      ventilation: {
        zoneId: 'z1', annualFanElectricityKwh: 212, annualFrostProtectionElectricityKwh: 3,
        annualGrillePreheatingElectricityKwh: 0,
        months: [{ heating: { requiredOutdoorAirM3PerH: 191, infiltrationM3PerH: 38, conductanceWPerK: 90.1 } }],
      },
    };
    const html = generateNtaCalculationReportHTML(createDefaultProject(), assessment);
    expect(html).toContain('Ventilatie (hoofdstuk 11)');
    expect(html).toContain('90,1');
    expect(html).toContain('212');
    expect(html).toContain('aparte run met vast ventilatiesysteem C1 (§5.4.2)');
    expect(html).toContain('4800');
    expect(html).toContain('verrekend');
  });

  it('reports the chapter 5 indicators', () => {
    const assessment = calculated();
    const performance = assessment.performance as unknown as Record<string, unknown>;
    performance.chapter5 = {
      heatingNeedKwhPerM2: 61.27, coolingNeedKwhPerM2: 4.5, heatingAndCoolingNeedKwhPerM2: 65.77,
      standardInsulationKwhPerM2: 63, meetsStandardInsulation: true,
      renewableIndicatorKwhPerM2: 38.7, finalEnergyKwhPerM2: 120.41, finalEnergyEedKwhPerM2: 131.02,
      deliveredElectricityKwh: 2405, deliveredElectricityKwhPerM2: 24.05,
      deliveredExternalGj: 0, deliveredExternalGjPerM2: 0,
      deliveredOtherM3Aeq: 1068, deliveredOtherM3AeqPerM2: 10.68,
      renewableByCarrier: { electricity: 3870, heatPumpHeat: 0, solarHeat: 0, cold: 0, biomass: 0, externalHeat: 0, externalCold: 0 },
      locallyCarbonFree: false, renovationStandardKwhPerM2: null, meetsRenovationStandard: null,
    };
    const html = generateNtaCalculationReportHTML(createDefaultProject(), assessment);
    expect(html).toContain('Indicatoren hoofdstuk 5');
    expect(html).toContain('61,27');
    expect(html).toContain('63 kWh/m²·jr — voldoet (onverifieerd)');
    expect(html).toContain('1068 m³ aeq');
    expect(html).toContain('Lokaal koolstofemissievrij');
    expect(html).toContain('<td>nee</td>');
  });

  it('reports the kernel warnings next to the project plausibility warnings', () => {
    const assessment = calculated();
    assessment.warnings = [{ code: 'declared_hot_water_efficiency_above_one', path: 'ntaCalculation.declaredUses[0].monthlyKwh' }];
    (assessment.performance as unknown as Record<string, unknown>).warnings = [
      { code: 'cooling_emission_loss_singular', path: 'spaceCooling.months[9]' },
    ];
    const html = generateNtaCalculationReportHTML(createDefaultProject(), assessment);
    expect(html).toContain('Plausibiliteit');
    expect(html).toContain('declared_hot_water_efficiency_above_one');
    expect(html).toContain('cooling_emission_loss_singular');
  });

  it('lists gaps instead of numbers when the kernel has no result', () => {
    const html = generateNtaCalculationReportHTML(createDefaultProject(), {
      status: 'incomplete', targetNormVersion: 'NTA 8800:2025+C1:2026', kernelVersion: '0.1.0',
      inputFingerprint: 'sha256:x', attestStatus: 'unattested',
      gaps: [{ code: 'nta_calculation_block_missing', path: 'ntaCalculation' }],
      geometry: { usableFloorAreaM2: 96, lossAreaM2: 232.8, envelopeAreaM2: 247.2, lossAreaRatio: 2.425, unclassifiedSurfaceCount: 0 },
      derivedInput: null, performance: null,
    });
    expect(html).toContain('Geen uitkomst');
    expect(html).toContain('nta_calculation_block_missing');
    expect(html).not.toContain('BENG 2');
    expect(html).toContain('232,8');
  });
});
