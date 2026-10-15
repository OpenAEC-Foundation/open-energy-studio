/**
 * The full app against the real Rust kernel, without the API server: every
 * `/api/…` call is answered in-process by the `kernel_once` example
 * (crates/nta8800-service/examples/kernel_once.rs). Each flow checks that what
 * the app shows equals the kernel's own answer to the request the app sent:
 * the project result in the status bar, an edit that marks it outdated, the
 * basic survey and its takeover, the tailored advice table, the edition
 * switch review, the report and EP-Online overview, and a gap with "Ga naar".
 * Ported to the shell of the 12 Oct 2026 merge: a project without a survey is
 * entered as a question flow whose "Next" applies the shared NTA draft; a
 * survey project keeps the draft bar (workflow-e2e-040.test.tsx).
 *
 * Opt-in: runs only when OES_KERNEL_ONCE names the built example binary.
 * scripts/verify-nta.sh builds it and sets the variable; a plain `vitest run`
 * skips this file.
 */
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { strFromU8, unzipSync } from 'fflate';
import App from '../App';
import { clearSessionEvidence } from '../core/nta/Evidence';
import type { EditionSwitchReport, MaatwerkadviesAssessment, MwaVariantResult, ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import { summarizeForPreview } from '../core/nta/PreviewSummary';
import { dutchNumber } from '../core/report/DutchReportText';
import { formatNumber } from '../i18n/format';
import { SLOW_UI_TEST_MS } from './timeouts';

const binary = process.env.OES_KERNEL_ONCE;
const root = resolve(__dirname, '../..');
const example = (file: string) => JSON.parse(readFileSync(resolve(root, 'training-data', file), 'utf8')) as Record<string, unknown>;

/** One kernel operation through `kernel_once`: the same answer the API server would give. */
function kernel(method: string, path: string, body?: unknown): { status: number; body: unknown } {
  const run = spawnSync(binary!, [method, path], { input: body == null ? '' : JSON.stringify(body), encoding: 'utf8', maxBuffer: 256 * 1024 * 1024 });
  if (run.status !== 0) throw new Error(`kernel_once ${method} ${path} failed: ${run.error?.message ?? run.stderr}`);
  return JSON.parse(run.stdout) as { status: number; body: unknown };
}

/** Every kernel request the app sent, in order. */
const calls: Array<{ path: string; body: unknown }> = [];
const sent = (suffix: string) => calls.filter((call) => call.path.endsWith(suffix)).map((call) => call.body);
const projectBodies = () => sent('/project/performance');
/** The kernel's answer to the last request the app sent to `suffix`. */
const answerToLast = <T,>(suffix: string): T => {
  const bodies = sent(suffix);
  expect(bodies.length).toBeGreaterThan(0);
  const path = calls.filter((call) => call.path.endsWith(suffix)).slice(-1)[0].path;
  return kernel('POST', path, bodies[bodies.length - 1]).body as T;
};

beforeEach(() => {
  calls.length = 0;
  clearSessionEvidence();
  try { window.localStorage.clear(); } catch { /* ignore */ }
  window.history.replaceState(null, '', '/');
  vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
    const path = String(url).replace(/^\/api/, '');
    const body = init?.body ? JSON.parse(String(init.body)) as unknown : undefined;
    calls.push({ path, body });
    const outcome = kernel(init?.method ?? (body === undefined ? 'GET' : 'POST'), path, body);
    return {
      ok: outcome.status < 400,
      status: outcome.status,
      json: async () => outcome.body,
      text: async () => JSON.stringify(outcome.body),
    };
  }));
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

type User = ReturnType<typeof userEvent.setup>;
const statusBar = () => document.querySelector('footer.status-bar') as HTMLElement;
const nav = () => screen.getByRole('navigation', { name: 'Workflow steps' });
const main = () => within(screen.getByRole('main'));
const en = (value: number | null | undefined, digits = 0) => formatNumber(value, 'en', digits);
const escape = (text: string) => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

/**
 * A workflow step by its label ("2 Building, to do") and optionally one of its pages ("· Calculation zones").
 * Results and report are one step with two pages; Tailored advice is under "More" in a project without a survey.
 */
async function openStep(user: User, step: string, sub?: string) {
  if (step === 'Results' || step === 'Report & dossier') {
    await user.click(within(nav()).getByRole('button', { name: /^Results and report, / }));
    await user.click(within(nav()).getByRole('button', { name: step }));
  } else if (step === 'Tailored advice') {
    await user.click(within(nav()).getByRole('button', { name: step }));
  } else {
    await user.click(within(nav()).getByRole('button', { name: new RegExp(`${escape(step)}, `) }));
  }
  if (!sub) return;
  // Flow steps list their questions in the navigation; the other steps have their pages as tabs on the page.
  const name = new RegExp(`${escape(sub)}$`);
  await user.click(within(nav()).queryByRole('button', { name }) ?? main().getByRole('button', { name }));
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

async function openExample(user: User, project: Record<string, unknown>) {
  render(<App />);
  await openProjectFile(user, project);
}

/** The primary button of the question flow ("Next question", "Next: …"), which applies the shared NTA draft. */
const nextButton = () => {
  const button = screen.getByRole('main').querySelector<HTMLButtonElement>('.survey-actions .btn-primary');
  if (!button) throw new Error('no question-flow next button on this page');
  return button;
};

/** Applies the draft the way the question flow does (by moving on) and waits for the kernel run of the applied input. */
async function applyDraft(user: User) {
  const runs = projectBodies().length;
  await user.click(nextButton());
  await waitFor(() => expect(projectBodies().length).toBeGreaterThan(runs), { timeout: 10_000 });
}

/** Applies the draft with the bar, where a page is not a question of the flow (a survey project). */
async function applyDraftWithBar(user: User) {
  const runs = projectBodies().length;
  const bar = within(screen.getByRole('region', { name: 'NTA input draft' }));
  await user.click(bar.getByRole('button', { name: /^Apply/ }));
  expect(screen.queryByRole('region', { name: 'NTA input draft' })).toBeNull();
  await waitFor(() => expect(projectBodies().length).toBeGreaterThan(runs), { timeout: 10_000 });
}

/** The status bar of the settled answer agrees with the kernel's answer to the last request the app sent. */
async function expectKernelFigures() {
  await waitFor(() => expect(statusBar()).toHaveTextContent('Result current'), { timeout: 30_000 });
  const assessment = answerToLast<ProjectPerformanceAssessment>('/project/performance');
  const summary = summarizeForPreview(assessment);
  expect(summary.beng1).not.toBeNull();
  const health = kernel('GET', '/health').body as { kernelVersion: string };
  const status = statusBar();
  expect(status).toHaveTextContent(`Kernel ${health.kernelVersion}`);
  expect(status).toHaveTextContent(`BENG 1 ${en(summary.beng1, 1)}`);
  expect(status).toHaveTextContent(`BENG 2 ${en(summary.beng2, 1)}`);
  return { summary, assessment };
}

describe.skipIf(!binary)('full app against the real kernel (kernel_once)', () => {
  it('calculates the example terraced dwelling and recalculates after an edit', async () => {
    const user = userEvent.setup();
    await openExample(user, example('nta8800-example-terraced-dwelling.json'));
    const before = await expectKernelFigures();

    // An overhang on the south window: "Next" applies the draft, the result is outdated until the new answer.
    await openStep(user, 'Building');
    await user.selectOptions(main().getByLabelText('Obstruction Raam S'), 'overhang');
    const group = within(main().getByRole('group', { name: 'Raam S' }));
    await user.type(group.getByLabelText('Relative height (h/a)'), '0,9');
    await user.type(group.getByLabelText('Source'), 'gevelaanzicht');
    const runs = projectBodies().length;
    await user.click(nextButton());
    await waitFor(() => expect(statusBar()).toHaveTextContent('Result outdated'), { timeout: 10_000 });
    await waitFor(() => expect(projectBodies().length).toBeGreaterThan(runs), { timeout: 10_000 });
    const after = await expectKernelFigures();
    expect(after.summary.beng1).not.toBe(before.summary.beng1);
  }, SLOW_UI_TEST_MS);

  it('calculates the example office', async () => {
    const user = userEvent.setup();
    await openExample(user, example('nta8800-example-office.json'));
    await expectKernelFigures();
  }, SLOW_UI_TEST_MS);

  it('calculates the basic survey, takes it over and calculates the project with it', async () => {
    const user = userEvent.setup();
    const survey = example('nta8800-opname-1930-terraced.json');
    await openExample(user, { ...example('nta8800-example-terraced-dwelling.json'), basisopname: { kind: 'residential', survey } });
    await expectKernelFigures();

    // The survey's check page holds the full form with the calculation, its outcome and the takeover.
    await openStep(user, 'Check');
    await user.click(main().getByText('All data as a form (for experts)'));
    await user.click(main().getByRole('button', { name: 'Calculate survey' }));
    await waitFor(() => expect(sent('/opname/residential').length).toBeGreaterThan(0));
    const result = await main().findByLabelText('Survey result');
    const answer = answerToLast<{ performance: ProjectPerformanceAssessment['performance']; derivedInput: { spaceHeating: { generator: unknown } } }>('/opname/residential');
    const performance = answer.performance!;
    expect(result).toHaveTextContent(`Indicative label class: ${performance.indicativeLabelClass}`);
    expect(result).toHaveTextContent(`BENG 1: ${en(performance.needIndicatorKwhPerM2Year, 2)} kWh/m²`);
    expect(result).toHaveTextContent(`BENG 2: ${en(performance.primaryFossilIndicatorKwhPerM2Year, 2)} kWh/m²`);
    expect(result).toHaveTextContent(`BENG 3: ${en(performance.renewableSharePercent, 1)} %`);

    await user.click(within(result).getByRole('button', { name: 'Take over into project model' }));
    const dialog = within(await screen.findByRole('dialog'));
    await user.click(dialog.getByRole('button', { name: 'Take over into draft' }));
    // A survey project has no NTA question flow: the takeover goes into the draft that the bar applies.
    await applyDraftWithBar(user);
    const project = await expectKernelFigures();
    // The project the kernel calculated has the generator the survey derived.
    const request = projectBodies().slice(-1)[0] as { project: { ntaCalculation: { generator: unknown } } };
    expect(request.project.ntaCalculation.generator).toEqual(answer.derivedInput.spaceHeating.generator);
    expect(project.assessment.status).toBe('calculated_unverified');
  }, SLOW_UI_TEST_MS);

  it('calculates the tailored advice and shows the kernel\'s figures in the variant table', async () => {
    const user = userEvent.setup();
    const dwelling = example('nta8800-example-terraced-dwelling.json');
    const maatwerkadvies = {
      currentUse: { profile: 'nta', sourceReference: 'NTA-gebruiksprofiel' },
      measures: [{
        id: 'm1', name: 'Twaalf panelen', category: 'pv', target: 'project',
        patch: [{ op: 'replace', path: '/ntaCalculation/pvSystems/0/peakPower/panelCount', value: 12 }],
        investmentEur: 3000, costSource: 'offerte', lifetimeYears: 25,
      }],
      packages: [{ id: 'p1', name: 'Pakket zon', measureIds: ['m1'] }],
      tariffs: { gasEurPerM3: 1.4, electricityEurPerKwh: 0.3, electricityExportEurPerKwh: 0.05, sourceReference: 'tarieven 2026' },
      economics: { discountRate: 0.03, energyPriceChange: 0, sourceReference: 'standaard' },
    };
    await openExample(user, { ...dwelling, maatwerkadvies });
    await expectKernelFigures();

    await openStep(user, 'Tailored advice', 'Advice & report');
    await user.click(main().getAllByRole('button', { name: 'Calculate' })[0]);
    await waitFor(() => expect(sent('/maatwerkadvies').length).toBeGreaterThan(0));
    const table = (await main().findByRole('rowheader', { name: /^Pakket zon/ }, { timeout: 30_000 })).closest('table')!;
    const assessment = answerToLast<MaatwerkadviesAssessment>('/maatwerkadvies');
    const headers = within(table).getAllByRole('columnheader').map((cell) => cell.textContent);
    expect(headers).toHaveLength(13);
    const expected = (result: MwaVariantResult) => {
      const use = result.actualUse;
      return [
        result.label.labelClass ?? '–', en(result.label.primaryFossilIndicatorKwhPerM2, 1), en(result.label.tojuliMaxK, 2),
        en(use?.gasM3), en(use ? use.electricityImportKwh - use.electricityExportKwh : null), en(use?.districtHeatKwh),
        en(use?.co2Kg), en(use?.energyCostEur), en(result.savings?.energyCostEur), en(result.investmentEur),
        en(result.simplePaybackYears, 1), en(result.netPresentValueEur),
      ];
    };
    const rowOf = (name: string) => within(within(table).getByRole('rowheader', { name: new RegExp(`^${name}`) }).closest('tr')!)
      .getAllByRole('cell').map((cell) => cell.textContent);
    expect(rowOf('Current situation')).toEqual(expected(assessment.current!));
    const pkg = assessment.packages.find((item) => item.id === 'p1')!;
    expect(rowOf('Pakket zon')).toEqual(expected(pkg));
    // The extra panels produce: the package's EP2 differs from the current situation.
    expect(pkg.label.primaryFossilIndicatorKwhPerM2).not.toBe(assessment.current!.label.primaryFossilIndicatorKwhPerM2);
  }, SLOW_UI_TEST_MS);

  it('reviews the switch to 2024 and back with the kernel\'s comparison and calculates in each edition', async () => {
    const user = userEvent.setup();
    await openExample(user, example('nta8800-example-terraced-dwelling.json'));
    await expectKernelFigures();

    const reviewSwitch = async (edition: string) => {
      await openStep(user, 'Project');
      await user.selectOptions(await main().findByLabelText('NTA 8800 edition'), edition);
      const dialog = within(await screen.findByRole('dialog', { name: 'Change the NTA 8800 edition' }));
      await waitFor(() => expect(dialog.queryByText('Calculating the project in both editions…')).toBeNull(), { timeout: 30_000 });
      const report = answerToLast<EditionSwitchReport>('/project/editions/compare');
      expect(report.toNormVersion).toBe(edition);
      const status = dialog.getAllByRole('status').map((item) => item.textContent).join(' ');
      if (report.items.length === 0) expect(status).toContain('No input changes');
      else expect(status).toMatch(new RegExp(`^${report.items.length} items?;`));
      for (const item of report.items) expect(dialog.getByText(item.path)).toBeInTheDocument();
      const valuesOnly = report.switchPoints.filter((point) => !point.hasInputs).length;
      if (valuesOnly > 0) expect(dialog.getByText(new RegExp(`^${valuesOnly} switch points?`))).toBeInTheDocument();
      await user.click(dialog.getByRole('button', { name: 'Change the edition only' }));
      await applyDraft(user);
      return expectKernelFigures();
    };

    const older = await reviewSwitch('2024');
    expect(older.assessment.targetNormVersion).toMatch(/2024/);
    const back = await reviewSwitch('2025+C1');
    expect(back.assessment.targetNormVersion).toMatch(/2025\+C1/);
  }, SLOW_UI_TEST_MS);

  it('carries the kernel\'s figures, kernel version and edition into the report and the EP-Online overview', async () => {
    const user = userEvent.setup();
    await openExample(user, example('nta8800-example-terraced-dwelling.json'));
    const { assessment } = await expectKernelFigures();
    const performance = assessment.performance!;

    await openStep(user, 'Report & dossier');
    const preview = () => (main().getByTestId('report-builder-preview') as HTMLIFrameElement).getAttribute('srcdoc') ?? '';
    await waitFor(() => expect(preview()).toContain(`kern ${assessment.kernelVersion}`), { timeout: 10_000 });
    expect(preview()).toContain(assessment.targetNormVersion!);
    expect(preview()).toContain(`<th>Indicatieve labelklasse</th><td>${performance.indicativeLabelClass}</td>`);
    const labelEp2 = performance.labelPrimaryFossilIndicatorKwhPerM2Year ?? performance.primaryFossilIndicatorKwhPerM2Year;
    expect(preview()).toContain(`${dutchNumber(labelEp2, 2)} kWh/m²·jr`);

    await openStep(user, 'Report & dossier', 'BRL 9500 checklist');
    const overview = within(main().getByTestId('ep-online-overview'));
    const value = (field: string) => overview.getByText(field).closest('tr')!.querySelectorAll('td')[1].textContent;
    const round = (number: number | null | undefined, digits = 2) => String(Math.round(number! * 10 ** digits) / 10 ** digits);
    expect(value('Energieklasse')).toBe(performance.indicativeLabelClass);
    expect(value('Energiebehoefte')).toBe(round(performance.needIndicatorKwhPerM2Year));
    expect(value('Primaire_fossiele_energie')).toBe(round(performance.primaryFossilIndicatorKwhPerM2Year));
    expect(value('Aandeel_hernieuwbare_energie')).toBe(round(performance.renewableSharePercent, 1));

    // The dossier ZIP's overview names the kernel version and edition of the answer.
    let zip: Blob | null = null;
    vi.stubGlobal('URL', Object.assign(URL, {
      createObjectURL: (blob: Blob) => { zip = blob; return 'blob:dossier'; },
      revokeObjectURL: () => undefined,
    }));
    await user.click(main().getByRole('button', { name: 'Export project dossier (ZIP)' }));
    await waitFor(() => expect(zip).not.toBeNull(), { timeout: 20_000 });
    const archive = unzipSync(new Uint8Array(await zip!.arrayBuffer()));
    const saved = JSON.parse(strFromU8(archive['ep-online-gegevensoverzicht.json'])) as {
      pandcertificaat: Record<string, unknown>; berekening: { rekenkernversie: string; uitgave: string };
    };
    expect(saved.berekening.rekenkernversie).toBe(assessment.kernelVersion);
    expect(saved.berekening.uitgave).toContain(assessment.targetNormVersion);
    expect(saved.pandcertificaat.Primaire_fossiele_energie).toBe(Number(round(performance.primaryFossilIndicatorKwhPerM2Year)));
  }, SLOW_UI_TEST_MS);

  it('shows the kernel\'s gap for a blanked field and goes to that field', async () => {
    const user = userEvent.setup();
    await openExample(user, example('nta8800-example-terraced-dwelling.json'));
    await expectKernelFigures();

    await openStep(user, 'Building', 'Calculation zones');
    await user.clear(main().getByLabelText('Heating setpoint °C'));
    await applyDraft(user);
    const answer = answerToLast<ProjectPerformanceAssessment>('/project/performance');
    expect(answer.status).toBe('incomplete');
    const gap = answer.gaps.find((item) => item.path === 'ntaCalculation.setpoints.heatingC');
    expect(gap?.code).toBe('nta_value_missing');
    // The form copies the setpoint into the ventilation; one cleared field gives one gap.
    expect(answer.gaps.filter((item) => item.code === 'nta_value_missing').map((item) => item.path))
      .toEqual(['ntaCalculation.setpoints.heatingC']);

    // The check page of the question flow lists the kernel's open points.
    await openStep(user, 'Check');
    const buttons = await main().findAllByRole('button', { name: /Go to/ }, { timeout: 30_000 });
    const itemOf = (button: HTMLElement) => (button.closest('li, tr') ?? button.parentElement!) as HTMLElement;
    // One open point per kernel gap, each with the kernel's code and detail.
    const items = buttons.map((button) => itemOf(button).textContent ?? '');
    for (const kernelGap of answer.gaps) {
      expect(items.some((text) => text.includes(kernelGap.code) && text.includes(kernelGap.detail ?? '')), items.join(' | ')).toBe(true);
    }
    const goTo = buttons.find((button) => /Building › Calculation zones/.test(itemOf(button).textContent ?? ''));
    expect(goTo, items.join(' | ')).toBeDefined();
    await user.click(goTo!);
    expect(within(nav()).getByRole('button', { name: /^Building/ })).toHaveAttribute('aria-current', 'page');
    await waitFor(() => expect(document.activeElement).toBe(main().getByLabelText('Heating setpoint °C')));
  }, SLOW_UI_TEST_MS);
});
