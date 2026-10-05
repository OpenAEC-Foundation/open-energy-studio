/**
 * UI redesign F5: Gebouw sub pages (envelope table with filter and grouping,
 * thermal bridges, air tightness), the inline element editor in the inspector
 * and the Installaties service pages.
 */
import { describe, expect, it } from 'vitest';
import { fireEvent, screen, within } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { AirTightnessPage, EnvelopePage, transmissionShares } from '../components/shell/pages/BuildingPages';
import { ElementInspector } from '../components/shell/ElementInspector';
import { ServicePage, SERVICES, servicePresent } from '../components/shell/pages/InstallationsPage';
import { createDefaultProject } from '../context/EnergyContext';
import { renderWithProviders, userEvent } from './test-utils';

function Probe() {
  const { state } = useEnergy();
  const zone = state.project.zones[0];
  return <output data-testid="probe">{JSON.stringify({
    surfaceIds: zone.surfaces.map((surface) => surface.id),
    east: zone.surfaces.find((surface) => surface.id === 'surf-wall-e'),
    windowIds: zone.surfaces.flatMap((surface) => surface.windows.map((win) => win.id)),
    qv10: zone.airTightness.qv10,
  })}</output>;
}
const probe = () => JSON.parse(screen.getByTestId('probe').textContent ?? '{}');

describe('Gebouw › Schil & ramen', () => {
  it('lists surfaces with their windows as child rows, grouped by orientation', () => {
    renderWithProviders(<EnvelopePage />);
    const table = screen.getByRole('table', { name: 'Envelope & windows' });
    const south = within(table).getByRole('rowheader', { name: /^South · 1 surface/ });
    expect(south).toBeInTheDocument();
    const window = within(table).getByRole('row', { name: /^Raam Zuid groot/ });
    expect(window).toHaveClass('ui-table__row--child');
    expect(window.getAttribute('data-path')).toMatch(/^zones\[0\]\.surfaces\[\d+\]\.windows\[0\]$/);
    expect(within(table).getByRole('row', { name: /^Gevel Zuid/ })).toHaveTextContent('90°');
  });

  it('filters by type and by name, and regroups by type', async () => {
    const user = userEvent.setup();
    renderWithProviders(<EnvelopePage />);
    await user.click(screen.getByRole('radio', { name: /^Roofs/ }));
    expect(screen.queryByRole('row', { name: /^Gevel Zuid/ })).toBeNull();
    expect(screen.getByRole('row', { name: /^Dak Oost/ })).toBeInTheDocument();
    await user.click(screen.getByRole('radio', { name: /^All/ }));
    fireEvent.change(screen.getByLabelText('Filter'), { target: { value: 'oost' } });
    expect(screen.getByRole('row', { name: /^Gevel Oost/ })).toBeInTheDocument();
    expect(screen.queryByRole('row', { name: /^Gevel West/ })).toBeNull();
    fireEvent.change(screen.getByLabelText('Filter'), { target: { value: '' } });
    await user.click(screen.getByRole('radio', { name: 'Type' }));
    expect(screen.getByRole('rowheader', { name: /^Wall · 4 surfaces/ })).toBeInTheDocument();
  });

  it('splits the indicative H_T over element types', () => {
    const shares = Object.fromEntries(transmissionShares(createDefaultProject()).map((share) => [share.key, share.value]));
    // Linear bridges of the default project: 0,05·34 + 0,05·34 + 0,03·65.
    expect(shares.bridges).toBeCloseTo(5.35, 6);
    expect(shares.windows).toBeGreaterThan(0);
    expect(shares.wall).toBeGreaterThan(0);
  });
});

describe('inspector: inline element editor', () => {
  function Harness() {
    return <><EnvelopePage /><ElementInspector /><Probe /></>;
  }

  it('edits area, orientation and construction of the selected surface in place, keeping ids and order', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    const before = probe();
    await user.click(within(screen.getByRole('row', { name: /^Gevel Oost/ })).getByText('Gevel Oost'));
    const inspector = screen.getByRole('region', { name: 'Selected element' });
    expect(within(inspector).getByRole('heading', { name: 'Gevel Oost' })).toBeInTheDocument();
    fireEvent.change(within(inspector).getByLabelText('Gross A'), { target: { value: '41,5' } });
    await user.selectOptions(within(inspector).getByLabelText('Orientation'), 'SE');
    const constructions = within(inspector).getByLabelText('Assigned construction') as HTMLSelectElement;
    const other = Array.from(constructions.options).find((option) => option.value && option.value !== 'con-wall')!;
    await user.selectOptions(constructions, other.value);
    const after = probe();
    expect(after.surfaceIds).toEqual(before.surfaceIds);
    expect(after.windowIds).toEqual(before.windowIds);
    expect(after.east).toMatchObject({ id: 'surf-wall-e', area: 41.5, orientation: 'SE', constructionId: other.value });
    expect(after.east.windows).toEqual(before.east.windows);
  });

  it('does not write an empty required number', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    await user.click(within(screen.getByRole('row', { name: /^Gevel Oost/ })).getByText('Gevel Oost'));
    const area = within(screen.getByRole('region', { name: 'Selected element' })).getByLabelText('Gross A');
    const before = probe().east.area;
    fireEvent.change(area, { target: { value: '' } });
    expect(probe().east.area).toBe(before);
  });

  it('selects a window of the surface from the inspector', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    await user.click(within(screen.getByRole('row', { name: /^Gevel Zuid/ })).getByText('Gevel Zuid'));
    await user.click(screen.getByRole('button', { name: /^Deur Zuid/ }));
    expect(within(screen.getByRole('region', { name: 'Selected element' })).getByRole('heading', { name: 'Deur Zuid' })).toBeInTheDocument();
  });
});

describe('Gebouw › Luchtdichtheid', () => {
  it('edits q_v;10 inline', () => {
    renderWithProviders(<><AirTightnessPage /><Probe /></>);
    fireEvent.change(screen.getByLabelText(/^q_v;10 — /), { target: { value: '0,6' } });
    expect(probe().qv10).toBe(0.6);
  });
});

describe('Installaties sub pages', () => {
  it('knows which services the default project has', () => {
    const project = createDefaultProject();
    const present = SERVICES.filter((service) => servicePresent(project, service)).map((service) => service.id);
    expect(present).toContain('heating');
    expect(present).not.toContain('humidification');
  });

  it('shows the NTA blocks of a service with the way to Controle › NTA-invoer', () => {
    renderWithProviders(<ServicePage sub="humidification" />);
    expect(screen.getByRole('heading', { name: 'NTA 8800 input' })).toBeInTheDocument();
  });

  it('lists the systems of a service with key figures', () => {
    renderWithProviders(<ServicePage sub="ventilation" />);
    const table = screen.getByRole('table', { name: 'Ventilation' });
    expect(within(table).getByRole('columnheader', { name: 'SFP' })).toBeInTheDocument();
    expect(within(table).getAllByRole('row').length).toBeGreaterThan(1);
  });
});
