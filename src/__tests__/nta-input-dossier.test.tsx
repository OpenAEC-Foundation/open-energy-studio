import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { createDefaultProject } from '../context/EnergyContext';
import { ReportView } from '../components/ReportView/ReportView';
import { generateNtaInputDossierHTML } from '../core/report/NtaInputDossier';
import { renderWithProviders, userEvent } from './test-utils';

afterEach(() => { vi.restoreAllMocks(); vi.unstubAllGlobals(); });

describe('NTA input and evidence dossier', () => {
  it('lists saved gas table evidence as input and escapes its references', () => {
    const project = createDefaultProject();
    project.ntaHeatPumps = [{
      id: 'gwp-1', source: 'exhaust_air', sink: 'hydronic', drive: 'absorption',
      reversible: false, hybrid: false, booster: false,
      performanceEvidence: { kind: 'normative_default', reference: null },
      gasHeatPumpForfaitDraft: {
        generatorId: 'gwp-1', drive: 'absorption', application: 'collective_building',
        applicationReference: '<building schedule>', collectiveBuildingInstallation: true,
        externalHeatSupply: false, thermalCapacityKw: 40, capacityReference: 'plate',
        source: 'exhaust_air', sourceReference: 'product sheet',
        designSupplyTemperatureC: 40, designSupplyReference: 'heating design',
      },
    }];
    const html = generateNtaInputDossierHTML(project);
    expect(html).toContain('Gasmotor-/gasabsorptie-tabelinvoer');
    expect(html).toContain('&lt;building schedule&gt;');
    expect(html).toContain('collective_building');
    expect(html).toContain('geen gasgebruik of geverifieerde COP');
    expect(html).not.toContain('<building schedule>');
    expect(html).not.toContain('Indicatieve BENG-waarden');
  });

  it('exports saved measured auxiliary evidence without a calculated result', () => {
    const project = createDefaultProject();
    project.ntaHeatPumps = [{
      id: 'hp-aux', source: 'outdoor_air', sink: 'hydronic', drive: 'electric_compression',
      reversible: false, hybrid: false, booster: false,
      performanceEvidence: { kind: 'normative_default', reference: null },
      heatingAuxMeasuredDraft: {
        generatorId: 'hp-aux', generatorSourceReference: 'schedule',
        measurements: { standbyElectronicsW: 10, deliveryPumpDuringCompressorW: 200,
          deliveryPumpPrePostW: 90, pumpPreRunSeconds: 300, pumpPostRunSeconds: 300,
          averageCompressorOnSeconds: 600, meanCompressorModulation: 0.5,
          nominalElectricDriveKw: 2, measurementSourceReference: '<meter>',
          timingSourceReference: 'cycle sheet' },
        inputEnergySourceReference: 'monthly meter',
        months: Array.from({ length: 12 }, (_, index) => ({ month: index + 1, generatorInputElectricityKwh: 100 })),
      },
      forfaitHeatPumpDraft: {
        generatorId: 'hp-aux', classificationSourceReference: '<design>',
        scope: 'residential_at_most25_kw', source: 'outdoor_air', sink: 'hydronic',
        designSupplyTemperatureC: 35, sourceCorrectionFactor: null, sourceCorrectionReference: null,
        thermalCapacityKw: 8, capacitySourceReference: '<capacity sheet>',
        collectiveBuildingInstallation: false,
        rowVariant: 'table_9_28_high_efficiency',
        highEfficiencyEvidence: { productReference: '<model A>', testReportReference: '<lab report>',
          testStandardEdition: 'NEN-EN 14511-2:2022', points: [
            { condition: 'a7_wet6_w45', measuredCop: 2.76 },
            { condition: 'a7_wet6_w35', measuredCop: 2.86 },
            { condition: 'a_minus7_wet_minus8_w45', measuredCop: 1.91 },
          ] },
      },
    }];
    const html = generateNtaInputDossierHTML(project);
    expect(html).toContain('&lt;meter&gt;');
    expect(html).toContain('&lt;design&gt;');
    expect(html).toContain('&lt;capacity sheet&gt;');
    expect(html).toContain('&lt;lab report&gt;');
    expect(html).toContain('a7_wet6_w45: 2.76');
    expect(html).toContain('Thermisch vermogen kW');
    expect(html).toContain('residential_at_most25_kw');
    expect(html).toContain('12: 100');
    expect(html).toContain('geen geverifieerde uitkomst');
    expect(html).not.toContain('<meter>');
    expect(html).not.toContain('<design>');
    expect(html).not.toContain('<capacity sheet>');
    expect(html).not.toContain('<lab report>');
    expect(html).not.toContain('315,6 kWh');
  });

  it('shows exact heat pump provenance without implying a BENG result or rendering entered HTML', () => {
    const project = createDefaultProject();
    project.name = '<script>alert(1)</script>';
    project.ntaHeatPumps = [{
      id: 'hp-1', servedZoneIds: [project.zones[0].id],
      source: 'outdoor_air', sink: 'combined_hydronic_and_hot_water', drive: 'electric_compression',
      reversible: false, hybrid: false, booster: false,
      performanceEvidence: {
        kind: 'controlled_quality_declaration', reference: '20260214GK',
        registryRecord: {
          registrationNumber: '20260214GK', productName: 'Model A + tank B',
          manufacturer: 'Supplier', sourceUrl: 'https://bcrg.nl/declaration/example',
        },
      },
      performancePoints: [{ id: 'A7W35', service: 'space_heating', sourceTemperatureC: 7,
        sinkTemperatureC: 35, usefulCapacityKw: 5, inputPowerKw: 1.5,
        inputEnergyCarrier: 'electricity', testReference: 'Lab report 7' }],
      dhwTestPoints: [{ id: 'M', tapProfile: 'M', usefulEnergyKwhPerDay: 5.865,
        inputEnergyKwhPerDay: 2.52, nominalCapacityKw: 4.71, practiceFactor: 0.9,
        testSetpointC: 49.1, designSetpointC: 55,
        declarationNormVersion: 'NTA 8800:2020', sourceReference: 'BCRG p3 <table>' }],
      declaredOperatingLimits: { minimumOperatingCop: 2, maximumSupplyTemperatureC: 55,
        declarationNormVersion: 'NTA 8800:2024', sourceReference: 'BCRG p2 <limit>' },
      auxiliaryComponents: [{ id: 'fan', kind: 'source_fan', service: 'space_heating',
        nominalPowerW: 80, energyCarrier: 'electricity', measurementBoundary: 'additional',
        evidenceReference: 'Sheet 9' }],
      systemLinks: [],
    }];
    const html = generateNtaInputDossierHTML(project);
    expect(html).toContain('Model A + tank B');
    expect(html).toContain('20260214GK');
    expect(html).toContain('Lab report 7');
    expect(html).toContain('Sheet 9');
    expect(html).toContain('5.865');
    expect(html).toContain('NTA 8800:2020');
    expect(html).toContain('BCRG p3 &lt;table&gt;');
    expect(html).toContain('BCRG p2 &lt;limit&gt;');
    expect(html).toContain('Maximale aanvoer °C');
    expect(html).toContain('niet geattesteerd');
    expect(html).not.toContain('<script>alert(1)</script>');
    expect(html).toContain('&lt;script&gt;alert(1)&lt;/script&gt;');
    expect(html).not.toContain('Indicatieve BENG-waarden');
  });

  it('offers the input dossier from the report view before a calculation exists', async () => {
    const user = userEvent.setup();
    const createObjectURL = vi.fn(() => 'blob:oes-dossier');
    const revokeObjectURL = vi.fn();
    vi.stubGlobal('URL', { ...URL, createObjectURL, revokeObjectURL });
    const click = vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => {});
    renderWithProviders(<ReportView />);
    await user.click(screen.getByRole('button', { name: 'Export NTA input dossier' }));
    expect(createObjectURL).toHaveBeenCalledOnce();
    expect(click).toHaveBeenCalledOnce();
    expect(revokeObjectURL).toHaveBeenCalledWith('blob:oes-dossier');
    expect(screen.queryByText('BENG 1')).not.toBeInTheDocument();
  });
  it('lists every NTA input source and flags missing ones', () => {
    const project = createDefaultProject();
    project.ntaCalculation = {
      calculationScope: 'residential', areaSourceReference: 'plan A-01',
      setpoints: { heatingC: 20, coolingC: 24, sourceReference: '' },
    } as unknown as NonNullable<typeof project.ntaCalculation>;
    const html = generateNtaInputDossierHTML(project);
    expect(html).toContain('NTA-rekeninvoer en bronnen');
    expect(html).toContain('ntaCalculation.areaSourceReference');
    expect(html).toContain('plan A-01');
    expect(html).toContain('ntaCalculation.setpoints.sourceReference');
    expect(html).toContain('BRON ONTBREEKT');
    expect(html).toContain('heatingC: 20');
  });
});
