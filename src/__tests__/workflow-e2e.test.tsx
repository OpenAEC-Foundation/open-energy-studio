/**
 * The NTA 8800 workflow end to end, as a user goes through it in the full app:
 * File › New and File › Open, the workflow navigation, the step pages with the
 * shared NTA draft and its apply bar, the kernel answer, results, the report
 * builder, Bron & bewijs, the BRL 9500 checklist and the dossier. The HTTP
 * kernel is stubbed with answers the real kernel gave for the same requests
 * (training-data/*.kernel-output.json). Runs the default edition and
 * NTA 8800:2022.
 */
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { strFromU8, unzipSync } from 'fflate';
import App from '../App';
import { clearSessionEvidence } from '../core/nta/Evidence';

const root = resolve(__dirname, '../..');
const json = (file: string) => JSON.parse(readFileSync(resolve(root, 'training-data', file), 'utf8')) as Record<string, unknown>;
const terracedProject = json('nta8800-example-terraced-dwelling.json');
const terracedOutput = json('nta8800-example-terraced-dwelling.kernel-output.json');
const terraced2022Output = json('nta8800-example-terraced-dwelling.2022.kernel-output.json');
const survey1930 = json('nta8800-opname-1930-terraced.json');
const survey1930Output = json('nta8800-opname-1930-terraced.kernel-output.json');
const newProjectOutput = json('nta8800-new-project.kernel-output.json');

type Body = Record<string, unknown>;
type NtaBlock = Record<string, unknown> & { normVersion?: string };
const requests: Array<{ url: string; body: Body | null }> = [];
let projectAnswer: (block: NtaBlock | undefined) => unknown;

/** The kernel answer to a new, empty project: no NTA input and no zone. */
const MISSING_BLOCK = newProjectOutput;

/** The example's answer in its edition: the real outputs of the default edition and of 2022. */
function exampleAnswer(block: NtaBlock | undefined) {
  if (!block) return MISSING_BLOCK;
  return block.normVersion === '2022' ? terraced2022Output : terracedOutput;
}

const projectBodies = () => requests.filter((request) => request.url.endsWith('/project/performance'))
  .map((request) => (request.body as { project: { ntaCalculation?: NtaBlock } }).project);
const lastNta = () => projectBodies().slice(-1)[0]?.ntaCalculation;

beforeEach(() => {
  requests.length = 0;
  projectAnswer = exampleAnswer;
  clearSessionEvidence();
  try { window.localStorage.clear(); } catch { /* ignore */ }
  window.history.replaceState(null, '', '/');
  vi.stubGlobal('fetch', vi.fn((url: string, init?: RequestInit) => {
    const body = init?.body ? JSON.parse(String(init.body)) as Body : null;
    requests.push({ url: String(url), body });
    const answer = (value: unknown) => Promise.resolve({ ok: true, status: 200, json: async () => structuredClone(value) });
    if (String(url).endsWith('/project/performance')) {
      return answer(projectAnswer((body as { project: { ntaCalculation?: NtaBlock } }).project.ntaCalculation));
    }
    if (String(url).endsWith('/opname/residential')) return answer(survey1930Output);
    if (String(url).endsWith('/interpretations')) return answer([]);
    // Diagnostics: no answer, as with a kernel that is still busy.
    return new Promise(() => {});
  }));
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

type User = ReturnType<typeof userEvent.setup>;
const nav = () => screen.getByRole('navigation', { name: 'Workflow steps' });
const main = () => within(screen.getByRole('main'));
const bar = () => within(screen.getByRole('region', { name: 'NTA input draft' }));

async function openStep(user: User, step: RegExp, sub?: string) {
  await user.click(within(nav()).getByRole('button', { name: step }));
  if (sub) await user.click(within(nav()).getByRole('button', { name: sub }));
}

/** Waits for the kernel run of the current project state (debounced) to be asked. */
async function kernelRuns(count: number) {
  await waitFor(() => expect(projectBodies().length).toBeGreaterThanOrEqual(count), { timeout: 5000 });
}

/**
 * Opens a project file the way the browser build does: File › Open creates a file
 * input; the test hands it the file instead of the system dialog.
 */
async function openProjectFile(user: User, project: unknown, name = 'project.oes.json') {
  const click = vi.spyOn(HTMLInputElement.prototype, 'click').mockImplementation(function (this: HTMLInputElement) {
    if (this.type !== 'file') return;
    const file = new File([JSON.stringify({ version: '1.0', type: 'open-energy-studio', project })], name, { type: 'application/json' });
    Object.defineProperty(this, 'files', { value: [file], configurable: true });
    this.onchange?.(new Event('change'));
  });
  await user.keyboard('{Control>}o{/Control}');
  click.mockRestore();
  await waitFor(() => expect(screen.getAllByRole('button', { name: /^Close tab/ })).toHaveLength(2));
}

async function openExample(user: User, project: Record<string, unknown> = terracedProject) {
  render(<App />);
  await openProjectFile(user, structuredClone(project));
  await kernelRuns(1);
}

/** Waits until the debounced kernel runs have settled: no new project request for a while. */
async function kernelSettled() {
  let seen = -1;
  let since = Date.now();
  await waitFor(() => {
    const count = projectBodies().length;
    if (count !== seen) { seen = count; since = Date.now(); }
    expect(Date.now() - since).toBeGreaterThan(700);
  }, { timeout: 5000, interval: 100 });
}

async function applyDraft(user: User, changes?: RegExp) {
  if (changes) expect(screen.getByRole('region', { name: 'NTA input draft' })).toHaveTextContent(changes);
  const runs = projectBodies().length;
  await user.click(bar().getByRole('button', { name: /^Apply/ }));
  expect(screen.queryByRole('region', { name: 'NTA input draft' })).toBeNull();
  await kernelRuns(runs + 1);
  await kernelSettled();
}

describe('workflow end to end, default edition', () => {
  it('starts a new project without NTA input and starts the NTA input in the default edition', async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.keyboard('{Control>}n{/Control}');
    expect(screen.getByRole('button', { name: 'Close tab: Untitled 1' })).toBeInTheDocument();
    expect(within(nav()).getByRole('button', { name: /^Project/ })).toHaveAttribute('aria-current', 'page');
    await kernelRuns(1);
    expect(lastNta()).toBeUndefined();
    expect(await main().findByText('This project has no NTA input yet.')).toBeInTheDocument();
    // No kernel answer to show: no BENG values and no label class on the overview.
    await waitFor(() => expect(main().queryByText(/BENG 1 51/)).toBeNull());

    await user.click(main().getByRole('button', { name: 'Start NTA input' }));
    const edition = await main().findByRole('combobox', { name: 'NTA 8800 edition' });
    expect(edition).toHaveValue('2025+C1');
  }, 60000);

  it('takes a calculated basic survey over into the project model through the draft', async () => {
    const user = userEvent.setup();
    await openExample(user, { ...terracedProject, basisopname: { kind: 'residential', survey: structuredClone(survey1930) } });
    await openStep(user, /^Basic survey/);
    await user.click(main().getByRole('button', { name: 'Calculate survey' }));
    await waitFor(() => expect(requests.some((request) => request.url.endsWith('/opname/residential'))).toBe(true));
    await user.click(main().getByRole('button', { name: 'Outcome & defaults' }));

    await user.click(await main().findByRole('button', { name: 'Take over into project model' }));
    const dialog = within(await screen.findByRole('dialog'));
    expect(dialog.getByText('Take the survey over into the project model')).toBeInTheDocument();
    expect(dialog.getByText(/\d+ change\(s\)/)).toBeInTheDocument();
    await user.click(dialog.getByRole('button', { name: 'Take over into draft' }));
    expect(await screen.findByText('Survey taken over into the draft; apply it with the bar at the bottom.')).toBeInTheDocument();

    // Nothing is applied before the bar's Apply: the kernel still gets the example's own input.
    const before = structuredClone(lastNta());
    const derived = (survey1930Output as { derivedInput: { spaceHeating: { generator: unknown }; hotWater: unknown } }).derivedInput;
    await applyDraft(user, /change/);
    expect(lastNta()!.generator).toEqual(derived.spaceHeating.generator);
    // The kernel request leaves out empty (null) fields; what the survey derived arrives.
    expect(lastNta()!.hotWater).toMatchObject({ generator: { kind: 'gas_appliance', appliance: 'without_gaskeur' } });
    expect((derived.hotWater as { generator: { kind: string } }).generator.kind).toBe('gas_appliance');
    expect(lastNta()!.generator).not.toEqual(before!.generator);
  }, 60000);

  it('sets an obstruction for one window and applies it with the bar', async () => {
    const user = userEvent.setup();
    await openExample(user);
    await openStep(user, /^Building/);
    const page = main();
    await user.selectOptions(page.getByLabelText('Obstruction Raam N'), 'overhang');
    const group = within(page.getByRole('group', { name: 'Raam N' }));
    await user.type(group.getByLabelText('Relative height (h/a)'), '0,5');
    await user.type(group.getByLabelText('Source'), 'gevelaanzicht');
    expect((page.getByLabelText('Obstruction Raam S') as HTMLSelectElement).value).toBe('');
    await applyDraft(user, /change/);
    expect(lastNta()!.windowObstructions).toEqual([
      { windowId: 'win-N', obstruction: { method: 'overhang', relativeHeight: 0.5 }, sourceReference: 'gevelaanzicht' },
    ]);
  }, 60000);

  it('enters annex AA rooms with the project windows on the cooling page', async () => {
    const user = userEvent.setup();
    await openExample(user);
    await openStep(user, /^Installations/, 'Cooling');
    const page = main();
    await user.selectOptions(page.getByLabelText('Sufficient active cooling present (§5.7.1)'), 'compression_table10_29');
    await user.selectOptions(page.getByLabelText('Capacity evidence (§5.7.1)'), 'annex_aa');
    await user.click(page.getByRole('button', { name: 'Add annex AA calculation' }));
    const form = within(page.getByTestId('nta-annex-aa'));
    const rooms = form.getAllByRole('group', { name: /^Room / });
    const room = within(rooms[0]);
    await user.type(room.getByRole('spinbutton', { name: /Floor area/ }), '28');
    await user.click(room.getByRole('button', { name: 'Add window' }));
    const window = room.getByRole('combobox', { name: 'Window' }) as HTMLSelectElement;
    // Only the project's outdoor windows are offered.
    expect(Array.from(window.options).map((option) => option.value).filter(Boolean).sort()).toEqual(['win-N', 'win-S']);
    await applyDraft(user, /change/);
    const capacity = (lastNta()!.activeCooling as { capacity: { method: string; calculation: { rooms: Array<{ areaM2: number; windows: unknown[] }> } } }).capacity;
    expect(capacity.method).toBe('annex_aa');
    expect(capacity.calculation.rooms[0]).toMatchObject({ areaM2: 28, windows: [expect.objectContaining({ windowId: window.value })] });
  }, 60000);

  it('shows the kernel results and composes the report at each level', async () => {
    const user = userEvent.setup();
    await openExample(user);
    await openStep(user, /^Results/);
    const performance = (terracedOutput as { performance: { needIndicatorKwhPerM2Year: number; primaryFossilIndicatorKwhPerM2Year: number } }).performance;
    expect(await main().findByText(`${performance.needIndicatorKwhPerM2Year.toFixed(2)}`, { exact: false })).toBeInTheDocument();
    expect(main().getAllByText(`${performance.primaryFossilIndicatorKwhPerM2Year.toFixed(2)}`, { exact: false }).length).toBeGreaterThan(0);
    expect(main().getByText(/Unverified calculation/)).toBeInTheDocument();

    await openStep(user, /^Report/);
    const preview = () => (main().getByTestId('report-builder-preview') as HTMLIFrameElement).getAttribute('srcdoc') ?? '';
    await waitFor(() => expect(preview()).toContain('Bouwkundige uitgangspunten'));
    const balance = main().getByRole('checkbox', { name: /Heat and cold balance/ });
    expect(balance).toBeDisabled();
    expect(preview()).not.toContain('Berekening: Warmte- en koudebalans');
    await user.click(main().getByRole('radio', { name: /^Detailed/ }));
    expect(balance).toBeEnabled();
    expect(preview()).toContain('Berekening: Warmte- en koudebalans');
    await user.click(main().getByRole('radio', { name: /^Summary/ }));
    expect(balance).toBeDisabled();
    expect(preview()).not.toContain('Bouwkundige uitgangspunten');
  }, 60000);

  it('links an evidence file to a source field and lists it in the checklist and the dossier ZIP', async () => {
    const user = userEvent.setup();
    await openExample(user);
    await openStep(user, /^Building/);
    const input = main().getByLabelText('Add file: windowSolar.sourceReference') as HTMLInputElement;
    const photo = new File([new TextEncoder().encode('gevelfoto')], 'gevel-noord.jpg', { type: 'image/jpeg' });
    await act(async () => { fireEvent.change(input, { target: { files: [photo] } }); });
    await waitFor(() => expect(screen.getByRole('region', { name: 'NTA input draft' })).toBeInTheDocument());
    await applyDraft(user);
    // The file is added to the source text; the description that was there stays.
    expect((lastNta()!.windowSolar as { sourceReference: string }).sourceReference)
      .toMatch(/^synthetic: minimal obstruction .*; evidence:ev-\d+$/);

    await openStep(user, /^Report/, 'BRL 9500 checklist');
    expect(main().getByText('gevel-noord.jpg', { exact: false })).toBeInTheDocument();
    expect(main().queryByText('No evidence recorded yet.')).toBeNull();

    // The dossier ZIP (browser download) holds the file and a manifest that says what it supports.
    let zip: Blob | null = null;
    vi.stubGlobal('URL', Object.assign(URL, {
      createObjectURL: (blob: Blob) => { zip = blob; return 'blob:dossier'; },
      revokeObjectURL: () => undefined,
    }));
    await user.click(main().getByRole('button', { name: 'Export project dossier (ZIP)' }));
    await waitFor(() => expect(zip).not.toBeNull(), { timeout: 10000 });
    const archive = unzipSync(new Uint8Array(await zip!.arrayBuffer()));
    const manifest = JSON.parse(strFromU8(archive['manifest.json'])) as { evidence: Array<{ fileName: string; archivePath: string; supports: string[] }> };
    const entry = manifest.evidence.find((item) => item.fileName === 'gevel-noord.jpg');
    expect(entry).toBeDefined();
    expect(entry!.supports.join(' ')).toMatch(/windowSolar/);
    expect(strFromU8(archive[entry!.archivePath])).toBe('gevelfoto');
  }, 60000);
});

describe('inspector', () => {
  it('attaches a photo to a selected window and keeps the link by id when an earlier surface is removed', async () => {
    const user = userEvent.setup();
    window.localStorage.setItem('oes.inspector.open', '1');
    await openExample(user);
    window.localStorage.setItem('oes.inspector.open', '1');
    await openStep(user, /^Building/);
    const table = within(main().getByRole('table', { name: 'Envelope & windows' }));
    await user.click(table.getByText('Raam S'));
    const inspector = within(screen.getByRole('complementary', { name: 'Context' }));
    // Selecting shows the properties of the element, although the preview tab was open.
    expect(inspector.getByRole('tab', { name: 'Properties' })).toHaveAttribute('aria-selected', 'true');
    const evidence = within(inspector.getByRole('group', { name: /Source & evidence/ }));
    const input = evidence.getByLabelText(/Add file/) as HTMLInputElement;
    const photo = new File([new TextEncoder().encode('raam')], 'raam-zuid.jpg', { type: 'image/jpeg' });
    await act(async () => { fireEvent.change(input, { target: { files: [photo] } }); });
    await waitFor(() => expect(inspector.getByText('raam-zuid.jpg', { exact: false })).toBeInTheDocument());

    // Removing the first surface (with Raam N) moves Raam S up in the list; the link follows its id.
    await user.click(main().getByRole('button', { name: 'Delete: Gevel N' }));
    await user.click(main().getByRole('button', { name: 'Yes, delete' }));
    await waitFor(() => expect(within(main().getByRole('table', { name: 'Envelope & windows' })).queryByText('Raam N')).toBeNull());
    await openStep(user, /^Report/, 'BRL 9500 checklist');
    const register = main().getByText('raam-zuid.jpg', { exact: false }).closest('tr')!;
    expect(register).toHaveTextContent(/win-S/);
    expect(register).not.toHaveTextContent(/not in the project|no longer/i);
  }, 60000);
});

describe('workflow end to end, NTA 8800:2022', () => {
  it('switches the edition, shows the 2022-only fields and the legacy status, and offers to remove them again', async () => {
    const user = userEvent.setup();
    await openExample(user);
    const edition = await main().findByRole('combobox', { name: 'NTA 8800 edition' });
    await user.selectOptions(edition, '2022');
    expect(main().getByText('Older edition: the result is for comparison only and cannot be registered.')).toBeInTheDocument();
    await applyDraft(user, /change/);
    expect(lastNta()!.normVersion).toBe('2022');

    // 2022-only inputs appear on their pages.
    await openStep(user, /^Building/, 'Calculation zones');
    const mass = await main().findByLabelText('Mass per m² usable floor area (kg/m², NTA 8800:2022 only, table 7.10)');
    await user.type(mass, '400');
    // (8.47) of 2022 asks the real wall height of a crawlspace instead of the fixed 0,125 m.
    await openStep(user, /^Building/, 'Envelope & windows');
    expect(main().queryByLabelText(/Wall height above ground level/)).toBeNull();
    await user.selectOptions(main().getByLabelText('Space below the floor (8.3.4.2)'), 'crawlspace');
    await user.type(main().getByLabelText(/Wall height above ground level/), '0,4');
    await applyDraft(user, /change/);
    expect((lastNta()!.thermalMass as { massKgPerM2: number }).massKgPerM2).toBe(400);
    expect((lastNta()!.groundFloors as Array<{ below: { kind: string; wallHeightAboveGroundM: number } }>)[0].below)
      .toMatchObject({ kind: 'crawlspace', wallHeightAboveGroundM: 0.4 });

    // The kernel answers in 2022: comparison only, no registration.
    await openStep(user, /^Results/);
    const legacy = (await main().findByText('Older edition — not for registration.')).closest('[role="note"]');
    expect(legacy).toHaveTextContent('Older edition — not for registration. Calculated in NTA 8800:2022.');
    await openStep(user, /^Project/);
    expect(main().getAllByText('Rust kernel: calculated in an older edition (not for registration)').length).toBeGreaterThan(0);

    // Back to the default edition: the 2022 values are flagged with a remove action.
    await openStep(user, /^Project/);
    await user.selectOptions(await main().findByRole('combobox', { name: 'NTA 8800 edition' }), '2025+C1');
    await openStep(user, /^Building/, 'Calculation zones');
    expect(main().queryByLabelText(/Mass per m² usable floor area/)).toBeNull();
    const stale = main().getAllByRole('alert').find((alert) => /2022/.test(alert.textContent ?? ''));
    expect(stale).toBeDefined();
    await user.click(within(stale!).getByRole('button', { name: 'Remove' }));
    await applyDraft(user);
    expect(lastNta()!.normVersion ?? '2025+C1').toBe('2025+C1');
    expect(lastNta()!.thermalMass).not.toHaveProperty('massKgPerM2');
  }, 90000);
});

describe('open points', () => {
  it('goes from a kernel gap on the check page to the field', async () => {
    const user = userEvent.setup();
    projectAnswer = (block) => (block ? {
      ...terracedOutput, status: 'incomplete', derivedInput: null, performance: null,
      gaps: [{ code: 'setpoint_out_of_range', path: 'ntaCalculation.setpoints.heatingC' }],
    } : MISSING_BLOCK);
    await openExample(user);
    await openStep(user, /^Check/, 'Check overview');
    const goTo = await main().findByRole('button', { name: /Go to/ });
    await user.click(goTo);
    expect(within(nav()).getByRole('button', { name: /^Building/ })).toHaveAttribute('aria-current', 'page');
    await waitFor(() => expect(document.activeElement).toBe(main().getByLabelText('Heating setpoint °C')));
  }, 60000);
});
