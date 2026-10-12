import { useState } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { EditionSwitchField } from '../components/NtaPerformancePanel/NtaEditionSwitch';
import type { EditionSwitchReport } from '../core/nta/KernelClient';
import type { IProject } from '../core/energy/types';
import office from '../../training-data/nta8800-example-office.json';
import { renderWithProviders } from './test-utils';
import { auditAccessibility } from './a11yAudit';
import { ShellActionsProvider, type ShellActions } from '../components/shell/ShellActions';

// Edition-switch guidance: a change of edition shows what the kernel then
// refuses or requires (compare_project_editions) before anything is written.

const project = office as unknown as IProject;
const base = project.ntaCalculation as unknown as Draft;

function adjusted(): Draft {
  const next = structuredClone(base) as Record<string, any>;
  next.normVersion = '2020+A1';
  delete next.pvSystems[0].peakPower.panelPeakPowerW;
  return next;
}

function report(): EditionSwitchReport {
  return {
    fromNormVersion: '2025+C1',
    toNormVersion: '2020+A1',
    fromLabel: 'NTA 8800:2025+C1:2026',
    toLabel: 'NTA 8800:2020+A1:2020',
    switchPoints: [
      { points: ['68'], subject: 'piekvermogen PV (16.4)', fields: ['pv_kpk_per_m2_floor'], hasInputs: true },
      { points: ['3'], subject: 'K_CO2 tabel 5.3', fields: ['k_co2_gas'], hasInputs: false },
    ],
    items: [
      { category: 'refused', code: 'route_not_in_edition', path: 'ntaCalculation.lighting[0].lightingZones[0].power.ledFrom2017',
        points: ['33'], subject: 'tabel 14.3 led vanaf 2017', fix: 'switchOff', value: false, resolvedByAutomaticChanges: true },
      { category: 'refused', code: 'route_not_in_edition', path: 'ntaCalculation.pvSystems[0].peakPower.panelPeakPowerW',
        points: ['68'], subject: 'piekvermogen PV (16.4)', fix: 'choose', resolvedByAutomaticChanges: false },
      { category: 'required', code: 'ground_floor_wall_height_required', path: 'ntaCalculation.groundFloors[0].below.wallHeightAboveGroundM',
        points: ['59'], subject: 'hoogte h boven maaiveld (8.47)', fix: 'enter', resolvedByAutomaticChanges: false },
    ],
    adjustedProject: { ...project, ntaCalculation: adjusted() } as never,
    statusAfterAutomaticChanges: 'invalid',
    remainingAfterAutomaticChanges: [
      { code: 'route_not_in_edition', path: 'ntaCalculation.pvSystems[0].peakPower.panelPeakPowerW' },
      { code: 'ground_floor_wall_height_required', path: 'ntaCalculation.groundFloors[0].below.wallHeightAboveGroundM' },
    ],
  };
}

function Host() {
  const [draft, setDraft] = useState<Draft>(base);
  const [history, setHistory] = useState<Draft[]>([]);
  const change = (path: Path, value: unknown) => { setHistory((h) => [...h, draft]); setDraft(write(draft, path, value)); };
  const update = (next: (current: Draft) => Draft) => { setHistory((h) => [...h, draft]); setDraft(next(draft)); };
  const undo = () => { const previous = history[history.length - 1]; if (previous) { setDraft(previous); setHistory(history.slice(0, -1)); } };
  return <form aria-label="section">
    <EditionSwitchField draft={draft} project={project} change={change} update={update} />
    <button type="button" onClick={undo}>undo</button>
    <output data-testid="draft">{JSON.stringify(draft)}</output>
  </form>;
}

const draft = () => JSON.parse(screen.getByTestId('draft').textContent ?? '{}') as Record<string, any>;

function stubKernel(make: () => EditionSwitchReport = report) {
  vi.stubEnv('DEV', true);
  const fetchMock = vi.fn(async () => new Response(JSON.stringify(make())));
  vi.stubGlobal('fetch', fetchMock);
  return fetchMock;
}

describe('edition switch', () => {
  afterEach(() => { vi.unstubAllGlobals(); vi.unstubAllEnvs(); });

  it('asks the kernel for the comparison with the draft and the target, and writes nothing yet', async () => {
    const fetchMock = stubKernel();
    renderWithProviders(<Host />);
    fireEvent.change(screen.getByLabelText(/NTA 8800 edition/), { target: { value: '2020+A1' } });
    const dialog = await screen.findByRole('dialog');
    await within(dialog).findByText(/3 items;/);
    const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe('/api/v1/nta8800/project/editions/compare');
    const body = JSON.parse(String(init.body));
    expect(body.normVersion).toBe('2020+A1');
    expect(body.project.ntaCalculation.normVersion ?? '2025+C1').toBe('2025+C1');
    expect(draft().normVersion ?? '2025+C1').toBe('2025+C1');
    // Grouped, with the switch point per item.
    expect(within(dialog).getByRole('heading', { name: 'Refused in the new edition' })).toBeTruthy();
    expect(within(dialog).getByRole('heading', { name: 'Required in the new edition' })).toBeTruthy();
    expect(within(dialog).getByText(/Point 59/)).toBeTruthy();
    expect(within(dialog).getByText(/Adjusting automatically switches this choice off/)).toBeTruthy();
    expect(auditAccessibility(dialog)).toEqual([]);
  });

  it('changes only the edition, or applies the safe changes too, in one undoable step', async () => {
    stubKernel();
    renderWithProviders(<Host />);
    fireEvent.change(screen.getByLabelText(/NTA 8800 edition/), { target: { value: '2020+A1' } });
    let dialog = await screen.findByRole('dialog');
    fireEvent.click(await within(dialog).findByRole('button', { name: 'Change the edition only' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(draft().normVersion).toBe('2020+A1');
    expect(draft().pvSystems[0].peakPower.panelPeakPowerW).toBe(400);
    fireEvent.click(screen.getByRole('button', { name: 'undo' }));
    expect(draft().normVersion ?? '2025+C1').toBe('2025+C1');

    fireEvent.change(screen.getByLabelText(/NTA 8800 edition/), { target: { value: '2020+A1' } });
    dialog = await screen.findByRole('dialog');
    fireEvent.click(await within(dialog).findByRole('button', { name: 'Adjust automatically and change' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(draft()).toEqual(adjusted());
    fireEvent.click(screen.getByRole('button', { name: 'undo' }));
    expect(draft()).toEqual(base);
  });

  it('cancelling keeps the edition, and an unreachable kernel still allows the change', async () => {
    vi.stubEnv('DEV', true);
    vi.stubGlobal('fetch', vi.fn(async () => { throw new Error('offline'); }));
    renderWithProviders(<Host />);
    fireEvent.change(screen.getByLabelText(/NTA 8800 edition/), { target: { value: '2023' } });
    let dialog = await screen.findByRole('dialog');
    await within(dialog).findByText(/kernel cannot be reached/);
    expect(within(dialog).queryByRole('button', { name: 'Adjust automatically and change' })).toBeNull();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(draft().normVersion ?? '2025+C1').toBe('2025+C1');
    fireEvent.change(screen.getByLabelText(/NTA 8800 edition/), { target: { value: '2023' } });
    dialog = await screen.findByRole('dialog');
    fireEvent.click(await within(dialog).findByRole('button', { name: 'Change the edition only' }));
    await waitFor(() => expect(draft().normVersion).toBe('2023'));
  });

  // Review 29.
  it('names the test standard the new edition asks for but leaves the evidence to the adviser', async () => {
    stubKernel(() => ({
      ...report(),
      items: [{ category: 'refused', code: 'high_test_standard_invalid',
        path: 'ntaCalculation.generator.forfait.highEfficiencyEvidence.testStandardEdition',
        points: ['67a'], subject: 'beproeving hoge-COP-rij tabel 9.28', fix: 'choose', value: 'NEN-EN 14511-2:2007',
        resolvedByAutomaticChanges: false }],
      adjustedProject: { ...project, ntaCalculation: { ...base, normVersion: '2020+A1' } } as never,
      remainingAfterAutomaticChanges: [{ code: 'high_test_standard_invalid',
        path: 'ntaCalculation.generator.forfait.highEfficiencyEvidence.testStandardEdition' }],
    }));
    renderWithProviders(<Host />);
    fireEvent.change(screen.getByLabelText(/NTA 8800 edition/), { target: { value: '2020+A1' } });
    const dialog = await screen.findByRole('dialog');
    expect(await within(dialog).findByText(/asks for NEN-EN 14511-2:2007\. This is evidence/)).toBeTruthy();
    expect(within(dialog).queryByRole('button', { name: 'Adjust automatically and change' })).toBeNull();
  });

  it('goes to a field the new edition requires only after changing the edition', async () => {
    stubKernel();
    const navigate = vi.fn();
    const actions = { navigate } as unknown as ShellActions;
    renderWithProviders(<ShellActionsProvider value={actions}><Host /></ShellActionsProvider>);
    fireEvent.change(screen.getByLabelText(/NTA 8800 edition/), { target: { value: '2020+A1' } });
    const dialog = await screen.findByRole('dialog');
    // A refused input is still there in the current edition: going there keeps it.
    fireEvent.click(await within(dialog).findByRole('button', { name: /Go to ntaCalculation\.pvSystems\[0\]/ }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(draft().normVersion ?? '2025+C1').toBe('2025+C1');
    expect(navigate).toHaveBeenLastCalledWith(expect.objectContaining({ focusPath: expect.stringContaining('panelPeakPowerW') }));
    // The 8.47 wall height is only shown in 2022 and earlier: changing the edition first.
    fireEvent.change(screen.getByLabelText(/NTA 8800 edition/), { target: { value: '2020+A1' } });
    const again = await screen.findByRole('dialog');
    fireEvent.click(await within(again).findByRole('button', { name: /Change only the edition and go to ntaCalculation\.groundFloors\[0\]/ }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(draft().normVersion).toBe('2020+A1');
    expect(draft().pvSystems[0].peakPower.panelPeakPowerW).toBe(400);
    expect(navigate).toHaveBeenLastCalledWith(expect.objectContaining({ focusPath: expect.stringContaining('wallHeightAboveGroundM') }));
  });
});
