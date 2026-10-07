import { useEffect } from 'react';
import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { createDefaultProject, useEnergy } from '../context/EnergyContext';
import { StatusBar } from '../components/StatusBar/StatusBar';
import { NtaCalculationForm } from '../components/NtaPerformancePanel/NtaCalculationForm';
import { generateNtaCalculationReportHTML } from '../core/report/NtaCalculationReport';
import { kernelVerdict } from '../core/nta/KernelVerdict';
import { calculatedAssessment } from '../core/nta/useProjectPerformance';
import {
  legacyEdition, projectCalculated, type NtaCalculationInput, type ProjectPerformanceAssessment,
} from '../core/nta/KernelClient';
import { renderWithProviders } from './test-utils';

function assessment(status: ProjectPerformanceAssessment['status'], edition: '2024' | '2025+C1'): ProjectPerformanceAssessment {
  const legacy = edition === '2024';
  return {
    status, normVersion: edition, registrationEligible: !legacy,
    targetNormVersion: legacy ? 'NTA 8800:2024 met INT-V1:2024' : 'NTA 8800:2025+C1:2026',
    kernelVersion: '0.1.0', inputFingerprint: 'sha256:feed', attestStatus: 'unattested', gaps: [], derivedInput: null,
    geometry: null, performance: null,
  } as unknown as ProjectPerformanceAssessment;
}

describe('NTA 8800 editions', () => {
  it('treats a legacy-edition result as calculated but never registrable', () => {
    const legacy = { ...assessment('calculated_legacy_edition', '2024'), performance: {} } as ProjectPerformanceAssessment;
    expect(projectCalculated('calculated_legacy_edition')).toBe(true);
    expect(projectCalculated('incomplete')).toBe(false);
    expect(kernelVerdict(legacy)).toBe('calculated');
    expect(calculatedAssessment({ kind: 'done', project: createDefaultProject(), assessment: legacy } as never)).toBe(legacy);
    expect(legacyEdition(legacy)).toBe(true);
    expect(legacyEdition(assessment('calculated_unverified', '2025+C1'))).toBe(false);
  });

  it('puts a not-for-registration notice in the calculation report', () => {
    const project = createDefaultProject();
    const legacyHtml = generateNtaCalculationReportHTML(project, assessment('calculated_legacy_edition', '2024'));
    expect(legacyHtml).toContain('Oudere uitgave — niet voor registratie.');
    expect(legacyHtml).toContain('NTA 8800:2024 met INT-V1:2024');
    const currentHtml = generateNtaCalculationReportHTML(project, assessment('calculated_unverified', '2025+C1'));
    expect(currentHtml).not.toContain('Oudere uitgave');
  });

  it('shows the chosen older edition in the status bar', async () => {
    function LegacyStatusBar() {
      const { dispatch } = useEnergy();
      useEffect(() => {
        dispatch({ type: 'SET_NTA_CALCULATION', payload: { normVersion: '2024' } as NtaCalculationInput });
      }, [dispatch]);
      return <StatusBar />;
    }
    renderWithProviders(<LegacyStatusBar />);
    expect(await screen.findByText('NTA 8800:2024 with INT-V1:2024 — not for registration')).toBeInTheDocument();
  });

  it('offers the implemented editions in the NTA form and warns for an older one', () => {
    const project = createDefaultProject();
    renderWithProviders(<NtaCalculationForm project={project} initial={{ normVersion: '2024' }}
      onSave={() => undefined} onCancel={() => undefined} />);
    const select = screen.getByLabelText('NTA 8800 edition') as HTMLSelectElement;
    expect([...select.options].map((option) => option.value)).toEqual(['', '2025+C1', '2024', '2023', '2022', '2020+A1']);
    expect(select.value).toBe('2024');
    expect(screen.getByText('Older edition: the result is for comparison only and cannot be registered.')).toBeInTheDocument();
  });

  it('shows the table 13.2 kitchen pipe diameter only for the 2023 edition', () => {
    const project = createDefaultProject();
    const hotWater = { emission: { method: 'residential', served: 'kitchen_and_bathroom', kitchenLengthM: 7, kitchenPipeDiameter: 'up_to_10_mm' } };
    const label = 'Inner diameter of the kitchen draw-off pipe over at least two thirds of its length (table 13.2, 2023 edition)';
    const { unmount } = renderWithProviders(<NtaCalculationForm project={project}
      initial={{ normVersion: '2023', calculationScope: 'residential', hotWater } as never}
      onSave={() => undefined} onCancel={() => undefined} />);
    expect((screen.getByLabelText(label) as HTMLSelectElement).value).toBe('up_to_10_mm');
    unmount();
    renderWithProviders(<NtaCalculationForm project={project}
      initial={{ normVersion: '2024', calculationScope: 'residential', hotWater } as never}
      onSave={() => undefined} onCancel={() => undefined} />);
    expect(screen.queryByLabelText(label)).toBeNull();
    expect(screen.getByText(/only the 2020\+A1, 2022 and 2023 editions have this input/)).toBeInTheDocument();
  });

  it('offers the 2023 heating-emission description only for the 2023 edition', () => {
    const project = createDefaultProject();
    const emission = { system: 'floor_heating', balancing: 'none_or_unknown', control: 'individual_room_thermostats', sourceReference: '',
      edition2023: { kind: { type: 'surface', control: 'room', system: 'floor_dry', insulation: 'unknown' } } };
    const label = 'Emission per tables 9.2–9.10 of NTA 8800:2023';
    const { unmount } = renderWithProviders(<NtaCalculationForm project={project}
      initial={{ normVersion: '2023', calculationScope: 'residential', emission } as never}
      onSave={() => undefined} onCancel={() => undefined} />);
    expect((screen.getByLabelText(label) as HTMLInputElement).checked).toBe(true);
    expect((screen.getByLabelText('System') as HTMLSelectElement).value).toBe('floor_dry');
    unmount();
    renderWithProviders(<NtaCalculationForm project={project}
      initial={{ normVersion: '2024', calculationScope: 'residential', emission } as never}
      onSave={() => undefined} onCancel={() => undefined} />);
    expect(screen.queryByLabelText(label)).toBeNull();
    expect(screen.getByText(/only the 2020\+A1, 2022 and 2023 editions have this input/)).toBeInTheDocument();
  });

  it('offers the 2022-only inputs only for the 2022 edition and flags values left behind', () => {
    const project = createDefaultProject();
    const initial = {
      calculationScope: 'utility',
      groundFloors: [{ below: { kind: 'crawlspace', floorResistanceM2kPerW: 3.5, depthClass: 'other', wallResistanceM2kPerW: 2,
        wallUValueWPerM2k: 0.4, wallHeightAboveGroundM: 0.3 } }],
      hotWater: { storage: [{ id: 'v1', volumeL: 80, loss: { method: 'forfait' }, connectionFactor: 2, electricBoilerInsulatedPipe: true,
        inHeatedZone: true, sourceReference: '' }] },
      lighting: [{ lightingZones: [{ id: 'z1', areaM2: 100, power: { method: 'forfait', ledFrom2017: false }, parasitic: { method: 'forfait' },
        occupancy: { control: 'manual_or_unknown', centralOnControl: false }, daylight: { method: 'none' }, extractedLuminaires: false,
        constantIlluminance: 'led_l80' }] }],
    };
    const height = 'Wall height above ground level to the top of the ground floor h (m, NTA 8800:2022 formula 8.47)';
    const boiler = 'Electric boiler with insulated hot-water pipes (f_sto;dis;ls = 1.5, NTA 8800:2022 § 13.6.3)';
    const illuminance = 'Constant-illuminance control (NTA 8800:2022 table 14.4)';
    const { unmount } = renderWithProviders(<NtaCalculationForm project={project}
      initial={{ ...initial, normVersion: '2022' } as never} onSave={() => undefined} onCancel={() => undefined} />);
    expect((screen.getByLabelText(height) as HTMLInputElement).value).toBe('0.3');
    expect((screen.getByLabelText(boiler) as HTMLInputElement).checked).toBe(true);
    expect((screen.getByLabelText(illuminance) as HTMLSelectElement).value).toBe('led_l80');
    unmount();
    renderWithProviders(<NtaCalculationForm project={project}
      initial={{ ...initial, normVersion: '2023' } as never} onSave={() => undefined} onCancel={() => undefined} />);
    expect(screen.queryByLabelText(height)).toBeNull();
    expect(screen.queryByLabelText(boiler)).toBeNull();
    expect(screen.queryByLabelText(illuminance)).toBeNull();
    expect(screen.getAllByText(/only the 2022 and 2020\+A1 editions have this input/)).toHaveLength(3);
  });

  it('offers the heat-pump build year only for the 2020+A1 edition and flags a value left behind', () => {
    const project = createDefaultProject();
    const initial = {
      calculationScope: 'residential',
      generator: { kind: 'heat_pump_forfait', forfait: { generatorId: 'hp', installationYear: 2016, installationYearReference: 'type plate' } },
    };
    const year = 'Heat-pump build year (NTA 8800:2020+A1 formula 9.85: A = 13.0 kWh from 2015, otherwise 87.6 kWh)';
    const { unmount } = renderWithProviders(<NtaCalculationForm project={project}
      initial={{ ...initial, normVersion: '2020+A1' } as never} onSave={() => undefined} onCancel={() => undefined} />);
    expect((screen.getByLabelText(year) as HTMLInputElement).value).toBe('2016');
    unmount();
    renderWithProviders(<NtaCalculationForm project={project}
      initial={{ ...initial, normVersion: '2022' } as never} onSave={() => undefined} onCancel={() => undefined} />);
    expect(screen.queryByLabelText(year)).toBeNull();
    expect(screen.getByText(/only the 2020\+A1 edition uses this input/)).toBeInTheDocument();
  });
});
