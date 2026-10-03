import { describe, expect, it } from 'vitest';
import { createDefaultProject } from '../context/EnergyContext';
import { generateNtaCalculationReportHTML } from '../core/report/NtaCalculationReport';
import { summarizeServiceEnergy } from '../core/nta/ServiceEnergy';
import type { NtaServiceEnergyBreakdown, ProjectPerformanceAssessment } from '../core/nta/KernelClient';

const months = Array.from({ length: 12 }, (_, index) => index + 1);

/** Gas heating, electric hot water and fans; 10 kWh exported per month. */
function breakdown(): NtaServiceEnergyBreakdown {
  return {
    months: months.flatMap((month) => [
      { service: 'heating' as const, carrier: 'gas', month, usedKwh: 300, deliveredKwh: 300, primaryFossilKwh: 300 },
      { service: 'hotWater' as const, carrier: 'el', month, usedKwh: 40, deliveredKwh: 20, primaryFossilKwh: 29 },
      { service: 'ventilation' as const, carrier: 'el', month, usedKwh: 20, deliveredKwh: 10, primaryFossilKwh: 14.5 },
    ]),
    renewable: months.map((month) => ({ service: 'hotWater' as const, month, renewablePrimaryKwh: 5 })),
    adjustments: months.map((month) => ({ month, exportedElectricityCreditKwh: 14.5, storageCorrectionKwh: 0, renewableElectricityKwh: 101.5 })),
    annual: [
      { service: 'heating', carrier: 'gas', usedKwh: 3600, deliveredKwh: 3600, primaryFossilKwh: 3600 },
      { service: 'hotWater', carrier: 'el', usedKwh: 480, deliveredKwh: 240, primaryFossilKwh: 348 },
      { service: 'ventilation', carrier: 'el', usedKwh: 240, deliveredKwh: 120, primaryFossilKwh: 174 },
    ],
  };
}

function assessment(): ProjectPerformanceAssessment {
  const zone = {
    climateSource: 'NTA 8800 tables 17.1/17.2', omittedCorrections: [], specificHeatCapacityKjPerM2k: 180,
    annualHeatingNeedKwh: 3000, annualCoolingNeedKwh: 0,
    transmission: { method: 'components', conductanceWPerK: 40 },
    monthly: Array.from({ length: 12 }, () => ({ cooling: { needKwh: 0 } })),
  };
  return {
    status: 'calculated_unverified', targetNormVersion: 'NTA 8800:2025+C1:2026', kernelVersion: '0.1.0',
    inputFingerprint: 'sha256:feed', attestStatus: 'unattested', gaps: [], derivedInput: null,
    performance: {
      status: 'calculated_unverified', chapter5Source: 'ch5', issues: [], needIndicatorKwhPerM2Year: null,
      primaryFossilIndicatorKwhPerM2Year: 39.4, renewableSharePercent: 30, indicativeLabelClass: 'A',
      labelSource: 'bijlage IX', annualPrimaryFossilKwh: 3948, annualRenewablePrimaryKwh: 1278,
      annualZebPrimaryTotalKwh: 5100, zebPrimaryTotalIndicatorKwhPerM2: 51.01, annualZebCo2Kg: 820,
      carriers: [], electricityBalance: [], tojuli: [], tojuliMaxK: null, tojuliMeetsBblLimit: null,
      energyByService: breakdown(),
      hotWater: {
        annualNetNeedKwh: 2000, emissionEfficiency: 0.95, annualGeneratorOutputKwh: 2400, annualSolarRenewableKwh: 0,
        annualSolarSpaceHeatingKwh: 0, generators: [{ index: 0, order: 1, monthlyOutputKwh: [2400], monthlyShare: [1] }],
        months: months.map((month) => ({ month, netNeedKwh: 2000 / 12, emissionInputKwh: 175, circulationLossKwh: 12.5,
          storageLossKwh: 7.5, generatorOutputKwh: 200, carrierInputKwh: 40, auxiliaryElectricityKwh: 1,
          ambientHeatKwh: 160, recoverableLossKwh: 4 })),
      },
      cooling: {
        coolingLimitC: 14, internalTemperatureShiftK: 1.2, interpretations: [],
        generatorShares: [{ id: 'chiller-1', priority: 1, beta: 1, shareJulyToSeptember: 1, shareOtherMonths: 1, method: 2,
          monthlyEer: [0, 0, 0, 0, 4, 4, 4, 4, 4, 0, 0, 0] }],
        months: months.map((month) => ({ month, needKwh: 50, emissionLossKwh: 2, distributionLossKwh: 3, generatorColdKwh: 55,
          electricityKwh: 13.75, naturalGasKwh: 0, districtHeatKwh: 0, districtColdKwh: 0, auxiliaryElectricityKwh: 1, ambientColdKwh: 0 })),
      },
      pvSystems: [{ id: 'pv-zuid', annualKwh: 1200, monthlyKwh: months.map(() => 100) }],
      spaceHeating: {
        chapter9Source: 'ch9', omittedTerms: [], monthly: Array.from({ length: 12 }, () => ({ heatingNeedKwh: 250 })),
        demand: zone, additionalZoneDemands: [],
      },
    },
  } as unknown as ProjectPerformanceAssessment;
}

describe('§5.5.3 energy per energy function', () => {
  it('sums the functions and the building-level terms to EPTot and EPrenTot', () => {
    const summary = summarizeServiceEnergy(breakdown())!;
    expect(summary.rows.map((row) => row.service)).toEqual(['heating', 'hotWater', 'ventilation']);
    expect(summary.carriers).toEqual(['el', 'gas']);
    expect(summary.rows[1].usedKwh.el).toBe(480);
    expect(summary.rows[1].renewablePrimaryKwh).toBe(60);
    expect(summary.primaryFossilTotalKwh).toBeCloseTo(3600 + 348 + 174 - 174, 6);
    expect(summary.renewableTotalKwh).toBeCloseTo(60 + 1218, 6);
    expect(summarizeServiceEnergy({ months: [], renewable: [], adjustments: [], annual: [] })).toBeNull();
  });

  it('reports the breakdown, chapters 13, 10 and 16, the ZEB indicator and the interpretations', () => {
    const html = generateNtaCalculationReportHTML(createDefaultProject(), assessment(), [
      { part: 'bijlage Q', module: 'annex_q', items: ['Q.48: θevap;nom −10 °C for air/water <test>'] },
    ]);
    expect(html).toContain('Energie per energiefunctie (§5.5.3, 5.20)');
    expect(html).toContain('Warm tapwater (E<sub>W</sub>)');
    expect(html).toContain('3600');
    expect(html).toContain('Warm tapwater (hoofdstuk 13)');
    expect(html).toContain('150'); // circulation loss 12,5 × 12
    expect(html).toContain('Koeling (hoofdstuk 10)');
    expect(html).toContain('chiller-1 (methode 2');
    expect(html).toContain('Zonnestroom (hoofdstuk 16)');
    expect(html).toContain('pv-zuid');
    expect(html).toContain('ZEB-indicator (bijlage AB, informatief)');
    expect(html).toContain('51.01');
    expect(html).toContain('Bijlage: interpretaties van de rekenkern');
    expect(html).toContain('θevap;nom −10 °C for air/water &lt;test&gt;');
  });

  it('leaves the new sections out when the kernel did not produce them', () => {
    const plain = assessment();
    const performance = plain.performance as unknown as Record<string, unknown>;
    delete performance.energyByService;
    delete performance.hotWater;
    delete performance.cooling;
    delete performance.pvSystems;
    performance.annualZebPrimaryTotalKwh = null;
    performance.zebPrimaryTotalIndicatorKwhPerM2 = null;
    const html = generateNtaCalculationReportHTML(createDefaultProject(), plain);
    expect(html).not.toContain('Energie per energiefunctie');
    expect(html).not.toContain('Warm tapwater (hoofdstuk 13)');
    expect(html).not.toContain('Koeling (hoofdstuk 10)');
    expect(html).not.toContain('Zonnestroom (hoofdstuk 16)');
    expect(html).not.toContain('ZEB-indicator');
    expect(html).not.toContain('Bijlage: interpretaties');
  });
});
