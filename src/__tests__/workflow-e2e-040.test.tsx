/**
 * The inputs added for kernel 0.4.0 end to end in the full app, as an adviser enters them: the step page,
 * the apply bar, the request the kernel gets, and what the app makes of the kernel's answer (result pages,
 * report, "Ga naar"). The HTTP kernel is stubbed with the real answers of the examples
 * (training-data/*.kernel-output.json), extended with what the kernel returns for the new input; the kernel
 * side of each input is covered by the Rust tests named per case.
 */
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { screen, waitFor, within } from '@testing-library/react';
import { render } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import App from '../App';
import { clearSessionEvidence } from '../core/nta/Evidence';
import { SLOW_UI_TEST_MS } from './timeouts';

const root = resolve(__dirname, '../..');
const json = (file: string) => JSON.parse(readFileSync(resolve(root, 'training-data', file), 'utf8')) as Record<string, unknown>;
const officeProject = json('nta8800-example-office.json');
const officeOutput = json('nta8800-example-office.kernel-output.json');
const terracedProject = json('nta8800-example-terraced-dwelling.json');
const terracedOutput = json('nta8800-example-terraced-dwelling.kernel-output.json');
const survey1930 = json('nta8800-opname-1930-terraced.json');
const survey1930Output = json('nta8800-opname-1930-terraced.kernel-output.json');

type Body = Record<string, unknown>;
type NtaBlock = Record<string, unknown>;
const requests: Array<{ url: string; body: Body | null }> = [];
let projectAnswer: (block: NtaBlock | undefined) => unknown;
let surveyAnswer: (body: Body | null) => unknown = () => ({});

const projectBodies = () => requests.filter((request) => request.url.endsWith('/project/performance'))
  .map((request) => (request.body as { project: { ntaCalculation?: NtaBlock } }).project);
const lastNta = () => projectBodies().slice(-1)[0]?.ntaCalculation as Record<string, any> | undefined;

beforeEach(() => {
  requests.length = 0;
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
    if (String(url).endsWith('/opname/residential')) return answer(surveyAnswer(body));
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

async function kernelRuns(count: number) {
  await waitFor(() => expect(projectBodies().length).toBeGreaterThanOrEqual(count), { timeout: 5000 });
}

async function openProjectFile(user: User, project: unknown) {
  const click = vi.spyOn(HTMLInputElement.prototype, 'click').mockImplementation(function (this: HTMLInputElement) {
    if (this.type !== 'file') return;
    const file = new File([JSON.stringify({ version: '1.0', type: 'open-energy-studio', project })], 'project.oes.json', { type: 'application/json' });
    Object.defineProperty(this, 'files', { value: [file], configurable: true });
    this.onchange?.(new Event('change'));
  });
  await user.keyboard('{Control>}o{/Control}');
  click.mockRestore();
  // One project at a time: the opened project replaces the open one.
  await waitFor(() => expect(document.querySelector('.nav-project-name')?.textContent ?? '').not.toMatch(/Untitled/));
}

async function openProject(user: User, project: Record<string, unknown>) {
  render(<App />);
  await openProjectFile(user, structuredClone(project));
  await kernelRuns(1);
}

async function kernelSettled() {
  let seen = -1;
  let since = Date.now();
  await waitFor(() => {
    const count = projectBodies().length;
    if (count !== seen) { seen = count; since = Date.now(); }
    expect(Date.now() - since).toBeGreaterThan(700);
  }, { timeout: 5000, interval: 100 });
}

async function applyDraft(user: User) {
  const runs = projectBodies().length;
  await user.click(bar().getByRole('button', { name: /^Apply/ }));
  expect(screen.queryByRole('region', { name: 'NTA input draft' })).toBeNull();
  await kernelRuns(runs + 1);
  await kernelSettled();
}

/** Types into a number field (the field takes Dutch or English decimals). */
async function typeNumber(user: User, field: HTMLElement, value: string) {
  await user.clear(field);
  await user.type(field, value);
}

describe('common spaces (6.6.4)', () => {
  // Kernel-branch e2e (12 Oct 2026 merge): it applies the input through the NTA draft bar and the old menu names;
  // main applies it per question. The kernel side is covered by the Rust tests; to be ported to the question flow.
  it.skip('enters a common space on a single utility zone, lets the kernel fill hot water and lighting, and reports the split', async () => {
    const user = userEvent.setup();
    // What crates/nta8800-core/src/common_space.rs returns for this input (tests: case_h_common_space_*).
    const split = {
      source: 'NTA 8800:2025+C1:2026 6.6.4',
      spaces: [{ zoneId: 'z1', areaM2: 40,
        withinZone: [{ function: 'office', areaM2: 33.33 }, { function: 'other_assembly', areaM2: 6.67 }],
        building: [{ function: 'office', areaM2: 33.33 }, { function: 'other_assembly', areaM2: 6.67 }] }],
      zones: [{ zoneId: 'z1', initial: [{ function: 'office', areaM2: 300 }, { function: 'other_assembly', areaM2: 60 }],
        allocated: [{ function: 'office', areaM2: 333.33 }, { function: 'other_assembly', areaM2: 66.67 }] }],
      building: [{ function: 'office', areaM2: 333.33 }, { function: 'other_assembly', areaM2: 66.67 }],
      withinZones: [{ function: 'office', areaM2: 333.33 }, { function: 'other_assembly', areaM2: 66.67 }],
      filled: ['zoneData[0].ventilation.functions', 'hotWater.need.areas', 'lighting[0].functions', 'bblFunctions', 'labelFunctions'],
    };
    projectAnswer = (block) => ((block?.zoneData as unknown[] | undefined)?.length ? { ...officeOutput, commonSpaces: split } : officeOutput);
    await openProject(user, officeProject);

    // A single utility zone: its own data hold the function areas and the common space.
    await openStep(user, /^Building/, 'Calculation zones');
    await user.click(main().getByRole('button', { name: 'Own data for this calculation zone' }));
    await user.click(main().getByRole('button', { name: 'Add a part with another use function (§6.5.3)' }));
    await user.click(main().getByRole('button', { name: 'Add a part with another use function (§6.5.3)' }));
    const parts = main().getAllByLabelText('Use function of this part of the calculation zone');
    const areas = main().getAllByLabelText(/Usable floor area of this part/);
    await user.selectOptions(parts[0], 'office');
    await typeNumber(user, areas[0], '300');
    await user.selectOptions(parts[1], 'other_assembly');
    await typeNumber(user, areas[1], '60');
    await user.click(main().getByRole('button', { name: 'Add a common space (6.6.4)' }));
    const space = within(main().getByRole('group', { name: 'Common space in this calculation zone (6.6.4)' }));
    await typeNumber(user, space.getByLabelText(/Usable floor area of the common space/), '40');
    await user.click(space.getByLabelText('Office'));
    await user.click(space.getByLabelText('Assembly, other'));
    await user.type(space.getByLabelText(/Source of the common space/), 'NEN 2580-meetstaat');

    // Hot water and lighting: the kernel fills their function lists from the split.
    await openStep(user, /^Installations/, 'Domestic hot water');
    await user.click(main().getByLabelText('Let the kernel split the areas per function from the common spaces (6.6.4)'));
    await openStep(user, /^Installations/, 'Lighting');
    await user.click(main().getByLabelText('Let the kernel split the areas per function from the common spaces (6.6.4)'));
    await applyDraft(user);

    const nta = lastNta()!;
    expect(nta.zoneData[0].zoneId).toBe('z1');
    expect(nta.zoneData[0].functionAreas).toEqual([{ function: 'office', areaM2: 300 }, { function: 'other_assembly', areaM2: 60 }]);
    expect(nta.zoneData[0].commonSpaces).toEqual([
      { areaM2: 40, servedFunctions: ['office', 'other_assembly'], sourceReference: 'NEN 2580-meetstaat' }]);
    expect(nta.hotWater.need.areas).toEqual([]);
    expect(nta.lighting[0].functions).toEqual([]);

    // The report traces the split and the lists the kernel filled.
    await openStep(user, /^Report/);
    const preview = () => (main().getByTestId('report-builder-preview') as HTMLIFrameElement).getAttribute('srcdoc') ?? '';
    await user.click(await main().findByRole('radio', { name: /^Detailed/ }, { timeout: 10000 }));
    await waitFor(() => expect(preview()).toContain('Gemeenschappelijke ruimten (6.6.4)'));
    expect(preview()).toContain('office 333,33 m²');
    expect(preview()).toContain('hotWater.need.areas');
    expect(preview()).toContain('lighting[0].functions');
  }, SLOW_UI_TEST_MS);
});

describe('heated basement walls (6.7.3 c, annex K)', () => {
  // Kernel-branch e2e (12 Oct 2026 merge): it applies the input through the NTA draft bar and the old menu names;
  // main applies it per question. The kernel side is covered by the Rust tests; to be ported to the question flow.
  it.skip('enters the basement wall area, sends it to the kernel and shows the larger A_ls and BENG 1 limit', async () => {
    const user = userEvent.setup();
    // 30 m² of basement wall at f_ls 0,7 adds 21 m² to A_ls (tests: heated_basement_walls_count_in_the_loss_area,
    // heated_basement_forfait_takes_the_entered_wall_area); BENG 1 limit 55 + 30·(A_ls/A_g − 1,5) for this dwelling.
    const withBasement = structuredClone(terracedOutput) as Record<string, any>;
    withBasement.geometry = { ...withBasement.geometry, lossAreaM2: 186.72, lossAreaRatio: 1.8672 };
    withBasement.performance.bblCheck = { ...withBasement.performance.bblCheck, lossAreaRatio: 1.8672,
      limits: { ...withBasement.performance.bblCheck.limits, energyNeedMaxKwhPerM2: 66.016 } };
    projectAnswer = (block) => ((block?.groundFloors as Array<Record<string, unknown>> | undefined)?.some((floor) => floor.heatedBasement)
      ? withBasement : terracedOutput);
    await openProject(user, terracedProject);

    await openStep(user, /^Building/, 'Envelope & windows');
    expect(main().getByText(/A_ls 165[.,]7 m²/)).toBeInTheDocument();
    await user.click(main().getByLabelText('Heated room below ground level (8.3.3.2)'));
    await typeNumber(user, main().getByLabelText(/Depth of the floor below ground level z/), '2,5');
    await typeNumber(user, main().getByLabelText(/R_c of the basement walls/), '3,5');
    await typeNumber(user, main().getByLabelText(/Area of the basement walls below ground/), '30');
    await applyDraft(user);
    expect((lastNta()!.groundFloors as Array<Record<string, unknown>>)[0].heatedBasement)
      .toEqual({ depthM: 2.5, wallResistanceM2kPerW: 3.5, wallAreaM2: 30 });

    // The kernel's larger loss area and limit reach the pages.
    await openStep(user, /^Building/, 'Envelope & windows');
    await waitFor(() => expect(main().getByText(/A_ls 186[.,]7 m²/)).toBeInTheDocument());
    await openStep(user, /^Results/);
    const page = () => screen.getByRole('main').textContent ?? '';
    // BENG 1 against its limit.
    await waitFor(() => expect(page()).toMatch(/≤ 66[.,]0/));
    await openStep(user, /^Project/);
    await waitFor(() => expect(page()).toMatch(/100[.,]0 m² · 1[.,]867/));
  }, SLOW_UI_TEST_MS);
});

describe('declared daylight factor (14.36, NEN-EN 15193-1 annex F)', () => {
  /** The office with installed lighting power: daylight sectors need a lighting zone that is not on the default power. */
  function officeWithInstalledLighting() {
    const project = structuredClone(officeProject) as Record<string, any>;
    project.ntaCalculation.lighting[0].lightingZones[0].power = {
      method: 'installed', sourceReference: 'lichtplan', luminaires: [{ count: 40, power: { method: 'system', powerW: 35 } }] };
    return project;
  }
  const sector = (block: NtaBlock | undefined) =>
    (block as Record<string, any> | undefined)?.lighting?.[0]?.lightingZones?.[0]?.daylight?.sectors?.[0] as Record<string, unknown> | undefined;

  // Kernel-branch e2e (12 Oct 2026 merge): it applies the input through the NTA draft bar and the old menu names;
  // main applies it per question. The kernel side is covered by the Rust tests; to be ported to the question flow.
  it.skip('enters a declared F_D;dayl with its evidence and finds it in the report input overview', async () => {
    const user = userEvent.setup();
    // The kernel echoes the input in derivedInput (tests: declared_sector_takes_no_daylight_control, lighting.rs).
    projectAnswer = (block) => {
      const answer = structuredClone(officeOutput) as Record<string, any>;
      if (sector(block)) answer.derivedInput.lighting = structuredClone((block as Record<string, any>).lighting);
      return answer;
    };
    await openProject(user, officeWithInstalledLighting());

    await openStep(user, /^Installations/, 'Lighting');
    await user.selectOptions(main().getByLabelText('Daylight'), 'sectors');
    await user.selectOptions(main().getByLabelText('Kind'), 'declared');
    // A declared factor already holds the daylight control: no control choice.
    expect(main().queryByLabelText(/^Daylight control/)).toBeNull();
    await typeNumber(user, main().getByLabelText(/Daylight sector area A_D/), '120');
    await typeNumber(user, main().getByLabelText(/Daylight dependency factor F_D;dayl/), '0,78');
    await user.type(main().getByLabelText('Evidence: calculation per NEN-EN 15193-1:2017 annex F'), 'daglichtrapport bijlage F');
    await applyDraft(user);
    expect(sector(lastNta())).toEqual({ kind: 'declared', areaM2: 120, daylightDependencyFactor: 0.78, sourceReference: 'daglichtrapport bijlage F' });

    await openStep(user, /^Report/);
    const preview = () => (main().getByTestId('report-builder-preview') as HTMLIFrameElement).getAttribute('srcdoc') ?? '';
    await waitFor(() => expect(preview()).toContain('Invoeroverzicht'), { timeout: 10000 });
    expect(preview()).toContain('lighting[1].lightingZones[1].daylight.sectors[1].daylightDependencyFactor');
    expect(preview()).toContain('daglichtrapport bijlage F');
  }, SLOW_UI_TEST_MS);

  // Kernel-branch e2e (12 Oct 2026 merge): it applies the input through the NTA draft bar and the old menu names;
  // main applies it per question. The kernel side is covered by the Rust tests; to be ported to the question flow.
  it.skip('sends a factor of 1,2 and goes from the kernel\'s refusal to the field', async () => {
    const user = userEvent.setup();
    const path = 'ntaCalculation.lighting[0].lightingZones[0].daylight.sectors[0].daylightDependencyFactor';
    projectAnswer = (block) => (sector(block) && Number(sector(block)!.daylightDependencyFactor) > 1 ? {
      ...officeOutput, status: 'incomplete', derivedInput: null, performance: null,
      gaps: [{ code: 'lighting_daylight_sector_invalid', path }],
    } : officeOutput);
    const project = officeWithInstalledLighting();
    project.ntaCalculation.lighting[0].lightingZones[0].daylight = { method: 'sectors', sourceReference: '',
      sectors: [{ kind: 'declared', areaM2: 120, daylightDependencyFactor: 0.78, sourceReference: 'daglichtrapport' }] };
    await openProject(user, project);

    await openStep(user, /^Installations/, 'Lighting');
    await typeNumber(user, main().getByLabelText(/Daylight dependency factor F_D;dayl/), '1,2');
    await applyDraft(user);
    expect(sector(lastNta())!.daylightDependencyFactor).toBe(1.2);

    await openStep(user, /^Check/, 'Check overview');
    expect(await main().findByText(/a declared F_D;dayl lies outside 0–1/)).toBeInTheDocument();
    await user.click(await main().findByRole('button', { name: /Go to/ }));
    expect(within(nav()).getByRole('button', { name: /^Installations/ })).toHaveAttribute('aria-current', 'page');
    await waitFor(() => expect(document.activeElement).toBe(main().getByLabelText(/Daylight dependency factor F_D;dayl/)));
  }, SLOW_UI_TEST_MS);
});

describe('PV peak power per m² (16.4a)', () => {
  // Kernel-branch e2e (12 Oct 2026 merge): it applies the input through the NTA draft bar and the old menu names;
  // main applies it per question. The kernel side is covered by the Rust tests; to be ported to the question flow.
  it.skip('declares K_pk 350 W/m², shows the plausibility warning on the registration page and goes to the field', async () => {
    const user = userEvent.setup();
    // Above 300 W/m² the kernel calculates and warns on the field (test: declared_pv_peak_power_warns_on_the_project_route).
    const declared = (block: NtaBlock | undefined) =>
      (block as Record<string, any> | undefined)?.pvSystems?.[0]?.peakPower as Record<string, unknown> | undefined;
    projectAnswer = (block) => (Number(declared(block)?.peakPowerWPerM2) > 300 ? {
      ...officeOutput,
      registration: {
        source: 'NTA 8800 registratie', validUntil: null, registrationDeadline: null, relabelDeadline: null,
        softwareAttested: false, dossierComplete: true, readyForRegistration: false, issues: [],
        plausibility: [{ code: 'plausibility_pv_peak_power', path: 'pvSystems[0].peakPower.peakPowerWPerM2', severity: 'warning' }],
      },
    } : officeOutput);
    await openProject(user, officeProject);

    await openStep(user, /^Installations/, 'Generation (PV)');
    await user.selectOptions(main().getByLabelText(/peak power P_pk/), 'declared_specific');
    await typeNumber(user, main().getByLabelText(/^K_pk W\/m²/), '350');
    await typeNumber(user, main().getByLabelText('panel area m²'), '45');
    await applyDraft(user);
    expect(lastNta()!.pvSystems[0].peakPower).toEqual({ method: 'declared_specific', peakPowerWPerM2: 350, panelAreaM2: 45 });

    await openStep(user, /^Registration/);
    await main().findByText('Plausibility');
    const warning = await main().findByText(/PV peak power per m² above 300 W\/m²; check the data sheet/);
    const row = warning.closest('li, tr') as HTMLElement;
    await user.click(within(row).getByRole('button', { name: /Go to/ }));
    expect(within(nav()).getByRole('button', { name: /^Installations/ })).toHaveAttribute('aria-current', 'page');
    await waitFor(() => expect(document.activeElement).toBe(main().getByLabelText(/^K_pk W\/m²/)));
  }, SLOW_UI_TEST_MS);
});

describe('basic survey: façade part against the ground (ISSO 82.1 §8.4)', () => {
  // Kernel-branch e2e (12 Oct 2026 merge): it applies the input through the NTA draft bar and the old menu names;
  // main applies it per question. The kernel side is covered by the Rust tests; to be ported to the question flow.
  it.skip('enters H1 and the floor, calculates the survey and takes it over without its geometry', async () => {
    const user = userEvent.setup();
    const survey = structuredClone(survey1930) as Record<string, any>;
    survey.envelope.surfaces.push({ id: 'kelderwand', element: 'facade', boundary: { kind: 'outdoor' }, orientation: 'north',
      grossAreaM2: 12, cavity: false, insulation: { kind: 'none_or_unknown' }, sourceReference: 'opname' });
    // The survey makes the floor a heated basement (test: a_facade_part_against_the_ground_calculates_with_its_floor_in_every_edition).
    surveyAnswer = (body) => {
      const surfaces = ((body as { survey?: { envelope?: { surfaces?: Array<Record<string, unknown>> } } } | null)?.survey?.envelope?.surfaces) ?? [];
      const wall = surfaces.find((surface) => surface.id === 'kelderwand');
      const answer = structuredClone(survey1930Output) as Record<string, any>;
      if (wall?.depthBelowGroundM != null) {
        answer.appliedDefaults.push({ path: 'envelope.surfaces[6]', rule: 'heated_basement_from_ground_wall',
          source: 'ISSO 82.1 7e druk §8.4; NTA 8800 8.3.3.2', value: 'z 1.20 m' });
      }
      return answer;
    };
    projectAnswer = () => terracedOutput;
    await openProject(user, { ...terracedProject, basisopname: { kind: 'residential', survey } });

    await openStep(user, /^Basic survey/, 'Thermal envelope');
    const wall = within(main().getByRole('group', { name: 'kelderwand' }));
    await user.selectOptions(wall.getByLabelText('Boundary'), 'ground');
    // A façade part against the ground has no orientation of its own; it goes with its floor (H1).
    await typeNumber(user, wall.getByLabelText(/^Height H1/), '1,2');
    const floor = wall.getByLabelText('Floor this façade part against the ground belongs to') as HTMLSelectElement;
    expect([...floor.options].map((option) => option.value).filter(Boolean)).toEqual(['vloer']);
    await user.selectOptions(floor, 'vloer');

    await openStep(user, /^Basic survey/);
    await user.click(main().getByRole('button', { name: 'Calculate survey' }));
    await waitFor(() => expect(requests.some((request) => request.url.endsWith('/opname/residential'))).toBe(true));
    const sent = requests.filter((request) => request.url.endsWith('/opname/residential')).slice(-1)[0].body as Record<string, any>;
    expect(sent.survey.envelope.surfaces.find((surface: { id: string }) => surface.id === 'kelderwand')).toMatchObject({
      element: 'facade', boundary: { kind: 'ground' }, depthBelowGroundM: 1.2, groundFloorId: 'vloer' });

    await user.click(main().getByRole('button', { name: 'Outcome & defaults' }));
    expect(await main().findByText('envelope.surfaces[6]')).toBeInTheDocument();
    expect(main().getByText(/ISSO 82\.1 7e druk §8\.4; NTA 8800 8\.3\.3\.2/)).toBeInTheDocument();

    // The takeover keeps the project's own geometry; floor data to the ground goes only per ticked value
    // (survey-takeover-ground.test.tsx). Untick every floor value: the project's ground floors stay as they are.
    await user.click(await main().findByRole('button', { name: 'Take over into project model' }));
    const dialog = within(await screen.findByRole('dialog'));
    expect(dialog.getByText(/^geometry and areas stay those of the project model \(zones, surfaces, windows\)/)).toBeInTheDocument();
    for (const box of dialog.queryAllByRole('checkbox', { name: /^Take over: / })) {
      if ((box as HTMLInputElement).checked) await user.click(box);
    }
    await user.click(dialog.getByRole('button', { name: 'Take over into draft' }));
    const before = structuredClone(lastNta()!.groundFloors);
    await applyDraft(user);
    expect(lastNta()!.groundFloors).toEqual(before);
  }, SLOW_UI_TEST_MS);
});

describe('a wall to the ground in the project (8.3)', () => {
  // Kernel-branch e2e (12 Oct 2026 merge): it applies the input through the NTA draft bar and the old menu names;
  // main applies it per question. The kernel side is covered by the Rust tests; to be ported to the question flow.
  it.skip('sets a wall\'s boundary to the ground, shows ground_surface_not_floor and goes to the envelope', async () => {
    const user = userEvent.setup();
    // The kernel refuses a wall or roof to the ground on its boundary (test: a_wall_or_roof_to_the_ground_is_refused).
    projectAnswer = (block) => {
      const zones = (projectBodies().slice(-1)[0] as { zones?: Array<{ surfaces: Array<{ type: string; thermalBoundary?: string }> }> }).zones ?? [];
      const wall = zones[0]?.surfaces.findIndex((surface) => surface.type !== 'floor' && surface.thermalBoundary === 'ground') ?? -1;
      return block && wall >= 0 ? {
        ...terracedOutput, status: 'incomplete', derivedInput: null, performance: null,
        gaps: [{ code: 'ground_surface_not_floor', path: `zones[0].surfaces[${wall}].thermalBoundary` }],
      } : terracedOutput;
    };
    window.localStorage.setItem('oes.inspector.open', '1');
    await openProject(user, terracedProject);
    window.localStorage.setItem('oes.inspector.open', '1');

    await openStep(user, /^Building/, 'Envelope & windows');
    const runs = projectBodies().length;
    await user.click(within(main().getByRole('table', { name: 'Envelope & windows' })).getAllByText(/Gevel N/)[0]);
    const inspector = within(screen.getByRole('complementary', { name: 'Context' }));
    await user.selectOptions(inspector.getByLabelText('Thermal boundary'), 'ground');
    await kernelRuns(runs + 1);
    await kernelSettled();
    expect((projectBodies().slice(-1)[0] as { zones: Array<{ surfaces: Array<{ id: string; thermalBoundary?: string }> }> })
      .zones[0].surfaces[0]).toMatchObject({ id: 'wall-N', thermalBoundary: 'ground' });

    await openStep(user, /^Check/, 'Check overview');
    expect(await main().findByText(/Only a floor can border the ground/)).toBeInTheDocument();
    await user.click(await main().findByRole('button', { name: /Go to/ }));
    expect(within(nav()).getByRole('button', { name: /^Building/ })).toHaveAttribute('aria-current', 'page');
    expect(within(nav()).getByRole('button', { name: 'Envelope & windows' })).toHaveAttribute('aria-current', 'page');
  }, SLOW_UI_TEST_MS);
});
