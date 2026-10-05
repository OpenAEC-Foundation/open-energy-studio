import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { createDefaultProject } from '../context/EnergyContext';
import { KernelAuditPanel } from '../components/KernelAuditPanel/KernelAuditPanel';
import { renderWithProviders, userEvent } from './test-utils';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.mocked(isTauri).mockReturnValue(false);
  vi.mocked(invoke).mockReset();
});

describe('Rust kernel audit in the UI', () => {
  it('hides the old audit as soon as the project input changes', async () => {
    vi.stubGlobal('fetch', vi.fn()
      .mockResolvedValueOnce({ ok: true, json: async () => ({
        status: 'structurally_valid', targetNormVersion: 'NTA 8800:2025+C1:2026',
        kernelVersion: '0.1.0', inputFingerprint: 'sha256:first-project',
        calculationAvailable: false, issues: [],
        summary: { zoneCount: 1, surfaceCount: 0, systemCount: 0,
          heatPumpCount: 0, classifiedHeatPumpCount: 0, floorAreaM2: 100 },
      }) })
      .mockImplementation(() => new Promise(() => {})));
    const first = createDefaultProject();
    const { rerender } = renderWithProviders(<KernelAuditPanel project={first} />);
    expect(await screen.findByText('sha256:first-project')).toBeInTheDocument();
    rerender(<KernelAuditPanel project={{ ...first, id: 'next-project' }} />);
    expect(screen.queryByText('sha256:first-project')).not.toBeInTheDocument();
  });

  it('shows the actual API assessment and its limited scope', async () => {
    const user = userEvent.setup();
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        status: 'structurally_valid',
        targetNormVersion: 'NTA 8800:2025+C1:2026',
        kernelVersion: '0.1.0',
        inputFingerprint: 'sha256:abc123',
        calculationAvailable: false,
        summary: {
          zoneCount: 1, surfaceCount: 7, systemCount: 4,
          heatPumpCount: 2, classifiedHeatPumpCount: 0, floorAreaM2: 67,
          thermalBoundaries: { surfaceCount: 7, classifiedSurfaceCount: 7,
            bridgeCount: 3, classifiedBridgeCount: 3, complete: true },
          directOutdoorDiagnostic: {
            status: 'input_valid', scope: 'diagnostic_direct_outdoor_only',
            referenceVerified: false, bengCalculationAvailable: false,
            inputFingerprint: 'sha256:direct',
            elementConductanceWPerK: 3.8, linearBridgeConductanceWPerK: 0.15,
            pointBridgeConductanceWPerK: 0, totalDirectConductanceWPerK: 3.95,
          },
          dhwTestDiagnostics: [{ heatPumpId: 'hp-dhw', pointId: 'M', tapProfile: 'M',
            declarationNormVersion: 'NTA 8800:2020', declaredUsefulToInputRatio: 2.327,
            referenceVerified: false, annualPerformanceAvailable: false }],
          envelopeGeometry: {
            grossSurfaceAreaM2: 50, windowAreaM2: 10, remainingOpaqueAreaM2: 40,
            zones: [{ zoneId: 'zone-main', grossSurfaceAreaM2: 50,
              windowAreaM2: 10, remainingOpaqueAreaM2: 40 }],
          },
        },
        issues: [{
          severity: 'warning', code: 'heat_pump_route_unclassified',
          path: 'heatingSystems[0]', message: 'Legacy heat pump has only a scalar COP',
        }, {
          severity: 'error', code: 'heat_pump_metadata_invalid',
          path: 'heatingSystems[1].ntaHeatPump', message: 'Invalid metadata',
          detail: 'unknown field `performancePoint`',
        }, {
          severity: 'error', code: 'window_g_invalid',
          path: 'zones[0].surfaces[0].windows[0].gValue',
          message: 'Window g-value must be finite and between zero and one',
        }],
      }),
    });
    vi.stubGlobal('fetch', fetchMock);

    renderWithProviders(<KernelAuditPanel project={createDefaultProject()} />);

    expect(await screen.findByText('heatingSystems[0]')).toBeInTheDocument();
    expect(screen.getByText(/not an NTA 8800 calculation or attested energy label/i)).toBeInTheDocument();
    expect(screen.getByText(/Source, sink, drive and evidence are missing/i)).toBeInTheDocument();
    expect(screen.getByText('sha256:abc123')).toBeInTheDocument();
    expect(screen.getByText('unknown field `performancePoint`')).toBeInTheDocument();
    expect(screen.getByText('Window g must be between zero and one.')).toBeInTheDocument();
    await user.click(screen.getByText('Envelope geometry by zone'));
    expect(screen.getByText('zone-main')).toBeInTheDocument();
    expect(screen.getAllByText('40.00')).toHaveLength(2);
    expect(screen.getByText(/7\/7 surfaces/)).toBeInTheDocument();
    await user.click(screen.getByText('Direct outdoor transmission diagnostic'));
    expect(screen.getByText('3.95 W/K')).toBeInTheDocument();
    await user.click(screen.getByText('DHW test energy ratio'));
    expect(screen.getByText('2.327')).toBeInTheDocument();
    expect(fetchMock).toHaveBeenCalledWith('/api/v1/nta8800/validate', expect.objectContaining({ method: 'POST' }));
  });

  it('reports when the Rust service cannot be reached', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('connection refused')));
    renderWithProviders(<KernelAuditPanel project={createDefaultProject()} />);
    expect(await screen.findByRole('alert')).toHaveTextContent(/Rust input validation is currently unavailable/i);
    expect(screen.getByRole('alert')).toHaveTextContent('connection refused');
  });

  it('shows a point power ratio with a clear non-annual scope', async () => {
    const user = userEvent.setup();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        status: 'structurally_valid', targetNormVersion: 'NTA 8800:2025+C1:2026',
        kernelVersion: '0.1.0', inputFingerprint: 'sha256:point',
        calculationAvailable: false, issues: [],
        summary: { zoneCount: 1, surfaceCount: 1, systemCount: 1,
          heatPumpCount: 1, classifiedHeatPumpCount: 1, floorAreaM2: 100,
          performancePointDiagnostics: [{ heatPumpPath: 'ntaHeatPumps[0]',
            heatPumpId: 'hp-1', pointId: 'a7w35', service: 'space_heating',
            inputEnergyCarrier: 'electricity', instantaneousUsefulToInputRatio: 3.25,
            referenceVerified: false, annualPerformanceAvailable: false }],
        },
      }),
    }));
    renderWithProviders(<KernelAuditPanel project={createDefaultProject()} />);
    await user.click(await screen.findByText('Heat pump operating-point ratios'));
    expect(screen.getByText('3.25')).toBeInTheDocument();
    expect(screen.getByText(/not an annual COP\/EER/i)).toBeInTheDocument();
    expect(screen.getByText('a7w35')).toBeInTheDocument();
  });

  it('shows a malformed heat pump field returned by the Rust API', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: false, status: 400,
      json: async () => ({ error: 'invalid_project_shape', message: 'unknown field `performancePoint`' }),
    }));
    renderWithProviders(<KernelAuditPanel project={createDefaultProject()} />);
    expect(await screen.findByRole('alert')).toHaveTextContent('unknown field `performancePoint`');
  });

  it('uses the embedded Rust command inside the desktop app', async () => {
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(invoke).mockResolvedValue({
      status: 'structurally_valid', targetNormVersion: 'NTA 8800:2025+C1:2026',
      kernelVersion: '0.1.0', inputFingerprint: 'sha256:abc123',
      calculationAvailable: false, issues: [],
      summary: { zoneCount: 1, surfaceCount: 1, systemCount: 0,
        heatPumpCount: 0, classifiedHeatPumpCount: 0, floorAreaM2: 100 },
    });
    renderWithProviders(<KernelAuditPanel project={createDefaultProject()} />);
    expect(await screen.findByText('Structure checked')).toBeInTheDocument();
    expect(invoke).toHaveBeenCalledWith('validate_nta_project', expect.objectContaining({ project: expect.any(Object) }));
  });
});
