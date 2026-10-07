import { afterEach, describe, expect, it, vi } from 'vitest';
import { invoke, isTauri } from '@tauri-apps/api/core';
import {
  calculateConstructionsWithRust, calculateSpaceHeatingChainWithRust, diagnoseForfaitHeatPumpDraftWithRust,
  diagnoseHybridHeatPumpMonthlyDraftWithRust, editionArgs,
  type EnvelopeInput, type ForfaitHeatPumpDraftInput, type HybridHeatPumpMonthlyDraftInput, type SpaceHeatingChainInput,
} from '../core/nta/KernelClient';

describe('edition of desktop diagnostic commands', () => {
  afterEach(() => {
    vi.mocked(isTauri).mockReturnValue(false);
    vi.mocked(invoke).mockReset();
  });

  it('sends an older edition and leaves the current one out', () => {
    expect(editionArgs('2022')).toEqual({ normVersion: '2022' });
    expect(editionArgs('2025+C1')).toEqual({});
    expect(editionArgs(null)).toEqual({});
    expect(editionArgs(undefined)).toEqual({});
  });

  it('passes normVersion to the Tauri commands', async () => {
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(invoke).mockResolvedValue({});
    const chain = {} as SpaceHeatingChainInput;
    await calculateSpaceHeatingChainWithRust(chain, '2020+A1');
    expect(invoke).toHaveBeenLastCalledWith('calculate_space_heating_chain', { input: chain, normVersion: '2020+A1' });
    await calculateSpaceHeatingChainWithRust(chain);
    expect(invoke).toHaveBeenLastCalledWith('calculate_space_heating_chain', { input: chain });
    const forfait = {} as ForfaitHeatPumpDraftInput;
    await diagnoseForfaitHeatPumpDraftWithRust(forfait, '2023');
    expect(invoke).toHaveBeenLastCalledWith('diagnose_forfait_heat_pump_draft', { input: forfait, normVersion: '2023' });
    const hybrid = {} as HybridHeatPumpMonthlyDraftInput;
    await diagnoseHybridHeatPumpMonthlyDraftWithRust(hybrid, '2024');
    expect(invoke).toHaveBeenLastCalledWith('diagnose_hybrid_heat_pump_monthly_draft', { input: hybrid, normVersion: '2024' });
    const envelope = {} as EnvelopeInput;
    await calculateConstructionsWithRust(envelope, '2025+C1');
    expect(invoke).toHaveBeenLastCalledWith('calculate_constructions', { input: envelope });
  });
});
