/**
 * The NTA input on the workflow steps (UI redesign F6, ontwerp §6): sections per
 * step, the heating stepper, Basis / Alle velden, the shared draft with its one
 * apply bar, "Ga naar" into a collapsed section, decimal commas and the inputs
 * that only the 2024 edition has.
 */
import { useEffect, useState } from 'react';
import { act, screen, within } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';
import { createDefaultProject, useEnergy } from '../context/EnergyContext';
import { changedPaths, useNtaDraft } from '../context/NtaDraftProvider';
import type { Route } from '../core/navigation/routes';
import type { NtaCalculationInput } from '../core/nta/KernelClient';
import { buildNtaCalculationTemplate } from '../core/nta/NtaCalculationTemplate';
import { NtaCalculationForm } from '../components/NtaPerformancePanel/NtaCalculationForm';
import { NtaApplyBar, NtaStepSections, heatingPartForPath, sectionEvidence } from '../components/shell/NtaStepPage';
import { NTA_SECTIONS } from '../components/NtaPerformancePanel/NtaSections';
import { focusPathIn } from '../components/shell/focusPath';
import { checkSummary } from '../components/shell/InspectorCheckPanel';
import { renderWithProviders, userEvent } from './test-utils';

let navigateTo: (route: Route) => void = () => undefined;

/** A document with an applied NTA block, one step page and the apply bar. */
function Harness({ initial, block }: { initial: Route; block?: Record<string, unknown> }) {
  const { state, dispatch } = useEnergy();
  const [route, setRoute] = useState<Route>(initial);
  navigateTo = setRoute;
  useEffect(() => {
    dispatch({ type: 'SET_NTA_CALCULATION', payload: (block ?? buildNtaCalculationTemplate(state.project)) as unknown as NtaCalculationInput });
    // Once, on mount.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
  return <>
    <div data-testid="page"><NtaStepSections key={`${route.step}/${route.sub}`} route={route} /></div>
    <NtaApplyBar route={route} navigate={setRoute} />
    <output data-testid="applied">{JSON.stringify(state.project.ntaCalculation ?? null)}</output>
    <output data-testid="route">{`${route.step}/${route.sub ?? ''}`}</output>
  </>;
}

const applied = () => JSON.parse(screen.getByTestId('applied').textContent ?? 'null');

beforeEach(() => {
  try { window.localStorage.removeItem('oes.nta.fieldMode'); } catch { /* ignore */ }
});

describe('NTA sections on the workflow steps', () => {
  it('shows each section on its own step and sub page', async () => {
    renderWithProviders(<Harness initial={{ step: 'project' }} />);
    const page = within(await screen.findByTestId('page'));
    expect(await page.findByRole('combobox', { name: 'NTA 8800 edition' })).toBeInTheDocument();
    expect(page.getByLabelText('Calculation scope')).toBeInTheDocument();
    expect(page.queryByLabelText('Heating setpoint °C')).toBeNull();

    act(() => navigateTo({ step: 'building', sub: 'zones' }));
    expect(await page.findByLabelText('Heating setpoint °C')).toBeInTheDocument();
    expect(page.getByLabelText('Floors')).toBeInTheDocument();
    expect(page.queryByLabelText('NTA 8800 edition')).toBeNull();

    act(() => navigateTo({ step: 'installations', sub: 'ventilation' }));
    expect(await page.findByLabelText('Ventilation input')).toBeInTheDocument();

    act(() => navigateTo({ step: 'installations', sub: 'bacs' }));
    expect(await page.findByLabelText('f_BACS (1 or 1.05)')).toBeInTheDocument();

    act(() => navigateTo({ step: 'check', sub: 'input' }));
    expect(await page.findByLabelText('All energy uses are included')).toBeInTheDocument();
  });

  it('steps through the parts of the heating page', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ step: 'installations', sub: 'heating' }} />);
    const page = within(await screen.findByTestId('page'));
    const parts = within(await page.findByRole('list', { name: 'Parts of the heating system' }));
    expect(parts.getByRole('button', { name: /Generation/ })).toHaveAttribute('aria-current', 'step');
    expect(page.getAllByLabelText('Generator type').length).toBeGreaterThan(0);
    expect(page.queryByLabelText('Emission system')).toBeNull();
    await user.click(parts.getByRole('button', { name: /Emission/ }));
    expect(page.getByLabelText('Emission system')).toBeInTheDocument();
    expect(page.queryByLabelText('Generator type')).toBeNull();
    await user.click(parts.getByRole('button', { name: /Control & BCRG/ }));
    expect(page.getByLabelText('Hydronic balancing')).toBeInTheDocument();
  });

  it('keeps advanced sections behind "Advanced" in the Basic view', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ step: 'installations', sub: 'heating' }} />);
    const page = within(await screen.findByTestId('page'));
    await user.click(await page.findByRole('button', { name: /Distribution/ }));
    const advanced = page.getByText(/Advanced · 1 sections/).closest('details')!;
    expect(advanced.open).toBe(false);
    expect(within(advanced).getByText(/all at default/)).toBeInTheDocument();
    await user.click(page.getByRole('radio', { name: 'All fields' }));
    expect(page.queryByText(/Advanced · /)).toBeNull();
    expect(page.getByLabelText('Pipes through the envelope')).toBeVisible();
    expect(window.localStorage.getItem('oes.nta.fieldMode')).toBe('all');
  });

  it('keeps one draft over the steps and applies it with the bar', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ step: 'building', sub: 'zones' }} />);
    const page = within(await screen.findByTestId('page'));
    const heating = await page.findByLabelText('Heating setpoint °C');
    await user.clear(heating);
    await user.type(heating, '19,5');
    expect(screen.getByRole('region', { name: 'NTA input draft' })).toHaveTextContent('1 change');
    expect(applied().setpoints.heatingC).toBe(20);

    // Another step and back: the draft is still there.
    act(() => navigateTo({ step: 'installations', sub: 'cooling' }));
    await page.findByLabelText('Sufficient active cooling present (§5.7.1)');
    act(() => navigateTo({ step: 'building', sub: 'zones' }));
    expect(await page.findByLabelText('Heating setpoint °C')).toHaveValue('19.5');

    // Undo goes back one edit at a time; apply writes the draft like the former Save.
    const bar = within(screen.getByRole('region', { name: 'NTA input draft' }));
    await user.click(bar.getByRole('button', { name: 'Undo' }));
    expect(page.getByLabelText('Heating setpoint °C')).toHaveValue('19');
    await user.type(page.getByLabelText('Heating setpoint °C'), ',5');
    await user.click(within(screen.getByRole('region', { name: 'NTA input draft' })).getByRole('button', { name: 'Apply and continue' }));
    expect(applied().setpoints.heatingC).toBe(19.5);
    expect(screen.queryByRole('region', { name: 'NTA input draft' })).toBeNull();
    // "Apply and continue" opens the next page with NTA input.
    expect(screen.getByTestId('route')).toHaveTextContent('building/unheated');
  });

  it('goes back to the previous NTA page from the bar', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ step: 'building', sub: 'zones' }} />);
    const page = within(await screen.findByTestId('page'));
    await user.selectOptions(await page.findByLabelText('Floors'), 'very_heavy');
    await user.click(within(screen.getByRole('region', { name: 'NTA input draft' })).getByRole('button', { name: 'Previous step' }));
    expect(screen.getByTestId('route')).toHaveTextContent('building/envelope');
  });

  it('opens the heating part and the collapsed section of a "Ga naar" path', async () => {
    expect(heatingPartForPath('ntaCalculation.emission.system')).toBe('emission');
    expect(heatingPartForPath('ntaCalculation.emission.fans.count')).toBe('auxiliary');
    expect(heatingPartForPath('ntaCalculation.zoneData[1].verticalPipes[0]')).toBe('distribution');
    expect(heatingPartForPath('ntaCalculation.hotWater')).toBeNull();

    renderWithProviders(<Harness initial={{ step: 'installations', sub: 'heating', focusPath: 'ntaCalculation.verticalPipes' }} />);
    const page = await screen.findByTestId('page');
    const select = await within(page).findByLabelText('Pipes through the envelope');
    // The advanced block opens by itself for the field of "Ga naar".
    expect(select.closest('details')!.open).toBe(true);
    expect(focusPathIn(page, 'ntaCalculation.verticalPipes')).toBe(select);
    expect(document.activeElement).toBe(select);
  });

  it('lists the source references of a section under "Source & evidence"', async () => {
    const section = NTA_SECTIONS.find((def) => def.id === 'mass')!;
    expect(sectionEvidence({ thermalMass: { floor: 'heavy', sourceReference: '' } }, section))
      .toEqual([{ path: 'thermalMass.sourceReference', filled: false }]);
    renderWithProviders(<Harness initial={{ step: 'building', sub: 'zones' }} />);
    const evidence = await screen.findByRole('group', { name: 'Source & evidence: Thermal mass (table 7.10)' });
    expect(evidence).toHaveTextContent('thermalMass.sourceReference');
  });
});

describe('field input', () => {
  it('accepts a decimal comma or point in NTA number fields', async () => {
    const user = userEvent.setup();
    let saved: Record<string, unknown> | null = null;
    renderWithProviders(<NtaCalculationForm project={createDefaultProject()} initial={{ setpoints: { heatingC: 20, coolingC: 24, sourceReference: 'x' } }}
      onSave={(block) => { saved = block; }} onCancel={() => undefined} />);
    const heating = screen.getByRole('spinbutton', { name: 'Heating setpoint °C' });
    await user.clear(heating);
    await user.type(heating, '20,5');
    const cooling = screen.getByRole('spinbutton', { name: 'Cooling setpoint °C' });
    await user.clear(cooling);
    await user.type(cooling, '24.5');
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect((saved as unknown as { setpoints: { heatingC: number; coolingC: number } }).setpoints).toMatchObject({ heatingC: 20.5, coolingC: 24.5 });
    expect(heating.closest('[data-path]')).toHaveAttribute('data-path', 'ntaCalculation.setpoints.heatingC');
  });

  it('counts changed leaves of the draft', () => {
    expect(changedPaths({ a: 1, b: { c: [1, 2] } }, { a: 1, b: { c: [1, 3] }, d: null })).toEqual(['b.c[1]', 'd']);
    expect(changedPaths({ a: undefined }, {})).toEqual([]);
  });
});

describe('2024-only inputs', () => {
  const block2024 = {
    normVersion: '2024',
    activeCooling: {
      system: 'compression_table10_29', sourceReference: '',
      capacity: { method: 'annex_aa', sourceReference: '', calculation: { constructionYear: 2020, rooms: [{ id: 'woonkamer', areaM2: 30 }] } },
    },
    externalSupply: { collectiveHeatPumpSource: { temperatureClass: null, supplierReference: '' } },
  };

  it('shows annex AA mass and roof area and the 2013 source only for the 2024 edition', async () => {
    const user = userEvent.setup();
    let saved: Record<string, unknown> | null = null;
    renderWithProviders(<NtaCalculationForm project={createDefaultProject()} initial={structuredClone(block2024)}
      onSave={(block) => { saved = block; }} onCancel={() => undefined} />);
    await user.type(screen.getByRole('spinbutton', { name: 'Effective mass (annex AA, 2024 edition), kg/m²' }), '165');
    await user.type(screen.getByRole('spinbutton', { name: 'Roof area of room woonkamer (annex AA, 2024 edition), m²' }), '12,5');
    await user.selectOptions(screen.getByLabelText('Collective source realised from 2013 (2023 or 2024 edition)'), 'true');
    await user.click(screen.getByRole('button', { name: 'Save' }));
    const block = saved as unknown as typeof block2024 & {
      activeCooling: { capacity: { calculation: { effectiveMassKgPerM2: number; rooms: Array<{ roofAreaM2: number }> } } };
      externalSupply: { collectiveHeatPumpSource: { realisedFrom2013: boolean } };
    };
    expect(block.activeCooling.capacity.calculation.effectiveMassKgPerM2).toBe(165);
    expect(block.activeCooling.capacity.calculation.rooms[0].roofAreaM2).toBe(12.5);
    expect(block.externalSupply.collectiveHeatPumpSource.realisedFrom2013).toBe(true);
  });

  it('hides them under another edition and offers to remove values left behind', async () => {
    const user = userEvent.setup();
    let saved: Record<string, unknown> | null = null;
    const stale = structuredClone(block2024) as Record<string, unknown>;
    (stale.activeCooling as { capacity: { calculation: Record<string, unknown> } }).capacity.calculation.effectiveMassKgPerM2 = 165;
    (stale.externalSupply as { collectiveHeatPumpSource: Record<string, unknown> }).collectiveHeatPumpSource.realisedFrom2013 = true;
    stale.normVersion = '2025+C1';
    renderWithProviders(<NtaCalculationForm project={createDefaultProject()} initial={stale}
      onSave={(block) => { saved = block; }} onCancel={() => undefined} />);
    expect(screen.queryByLabelText(/Effective mass/)).toBeNull();
    expect(screen.queryByLabelText('Collective source realised from 2013 (2023 or 2024 edition)')).toBeNull();
    const alerts = screen.getAllByRole('alert').filter((alert) => /only the 2024 edition|only the 2023 and 2024 editions/.test(alert.textContent ?? ''));
    expect(alerts).toHaveLength(2);
    for (const alert of alerts) await user.click(within(alert).getByRole('button', { name: 'Remove' }));
    await user.click(screen.getByRole('button', { name: 'Save' }));
    const block = saved as unknown as { activeCooling: { capacity: { calculation: Record<string, unknown> } }; externalSupply: { collectiveHeatPumpSource: Record<string, unknown> } };
    expect(block.activeCooling.capacity.calculation).not.toHaveProperty('effectiveMassKgPerM2');
    expect(block.externalSupply.collectiveHeatPumpSource.realisedFrom2013).toBeUndefined();
  });

  it('shows the edition-only fields on the cooling and generation steps', async () => {
    renderWithProviders(<Harness initial={{ step: 'installations', sub: 'cooling' }} block={block2024} />);
    expect(await screen.findByTestId('nta-annex-aa-2024')).toBeInTheDocument();
    act(() => navigateTo({ step: 'installations', sub: 'generation' }));
    expect(await screen.findByLabelText('Collective source realised from 2013 (2023 or 2024 edition)')).toBeInTheDocument();
  });
});

describe('Inspector › Check', () => {
  it('counts errors, points of attention and omitted corrections, this page first', () => {
    const summary = checkSummary([
      { kind: 'error', code: 'missing_generator', path: 'ntaCalculation.generator.kind' },
      { kind: 'warning', code: 'x', path: 'ntaCalculation.pvSystems[0]' },
      { kind: 'error', code: 'missing_g_value', path: 'zones[0].surfaces[0].windows[0].gValue' },
    ], { step: 'installations', sub: 'heating' }, ['a', 'b', 'a']);
    expect(summary).toMatchObject({ errors: 2, warnings: 1, omitted: ['a', 'b'] });
    expect(summary.here.map((issue) => issue.code)).toEqual(['missing_generator']);
    expect(summary.elsewhere.map((issue) => issue.route.sub)).toEqual(['generation', 'envelope']);
  });
});

function DraftProbe() {
  const shared = useNtaDraft();
  return <output data-testid="draft-probe">{shared ? 'yes' : 'no'}</output>;
}

describe('NtaDraftProvider', () => {
  it('is part of the document providers', () => {
    renderWithProviders(<DraftProbe />);
    expect(screen.getByTestId('draft-probe')).toHaveTextContent('yes');
  });
});
