import { describe, expect, it, vi } from 'vitest';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { useState } from 'react';
import { fireEvent, screen } from '@testing-library/react';
import { DecimalInput } from '../components/ui';
import { useEnergy } from '../context/EnergyContext';
import { ZoneEditorDialog } from '../components/dialogs/ZoneEditorDialog/ZoneEditorDialog';
import { ConstructionEditorDialog } from '../components/dialogs/ConstructionEditorDialog/ConstructionEditorDialog';
import { HeatingSystemDialog } from '../components/dialogs/HeatingSystemDialog/HeatingSystemDialog';
import { MaatwerkadviesPanel } from '../components/MaatwerkadviesPanel/MaatwerkadviesPanel';
import { renderWithProviders, userEvent } from './test-utils';

// A raw <input type="number"> rejects a decimal comma ("0,35") without a word in most
// browsers, so every number field goes through NumberInput or DecimalInput.
const RAW_NUMBER_ALLOWED: Record<string, string> = {};

function sourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return sourceFiles(path);
    return /\.tsx?$/.test(name) ? [path] : [];
  });
}

function withoutComments(source: string): string {
  return source.replace(/\/\*[\s\S]*?\*\//g, '').replace(/(^|[^:])\/\/.*$/gm, '$1');
}

describe('no raw number inputs', () => {
  it('finds no type="number" outside the allow list in src/components', () => {
    const offenders = sourceFiles('src/components')
      .filter((path) => /type=["']number["']/.test(withoutComments(readFileSync(path, 'utf8'))))
      .filter((path) => !(path in RAW_NUMBER_ALLOWED));
    expect(offenders).toEqual([]);
  });
});

function Controlled({ initial, accept = () => true, spy }: {
  initial: number | string; accept?: (value: number) => boolean; spy: (value: string) => void;
}) {
  const [value, setValue] = useState<number | string>(initial);
  return <><label>Amount
    <DecimalInput value={value} onChange={(event) => {
      spy(event.target.value);
      if (event.target.value === '') { setValue(''); return; }
      const next = Number(event.target.value);
      if (accept(next)) setValue(next);
    }} /></label>
    <output data-testid="stored">{String(value)}</output>
  </>;
}

describe('DecimalInput', () => {
  it('passes a typed comma on as a point decimal and keeps the spinbutton role', () => {
    const spy = vi.fn();
    renderWithProviders(<Controlled initial={1} spy={spy} />);
    const input = screen.getByRole('spinbutton', { name: 'Amount' });
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: '0,35' } });
    expect(spy).toHaveBeenLastCalledWith('0.35');
    expect(screen.getByTestId('stored')).toHaveTextContent('0.35');
    expect(input).toHaveValue('0,35');
    // On leaving the field it shows the value in the UI locale (English here).
    fireEvent.blur(input);
    expect(input).toHaveValue('0.35');
  });

  it('passes an empty field on as "" and holds back text that is no number', () => {
    const spy = vi.fn();
    renderWithProviders(<Controlled initial={2} spy={spy} />);
    const input = screen.getByRole('spinbutton', { name: 'Amount' });
    fireEvent.change(input, { target: { value: '' } });
    expect(spy).toHaveBeenLastCalledWith('');
    expect(screen.getByTestId('stored')).toHaveTextContent('');
    spy.mockClear();
    fireEvent.change(input, { target: { value: '1,2,3' } });
    expect(spy).not.toHaveBeenCalled();
    expect(input).toHaveAttribute('aria-invalid', 'true');
  });

  it('shows the parent value again when the parent does not take the number', () => {
    const spy = vi.fn();
    renderWithProviders(<Controlled initial={5} accept={(value) => value >= 0} spy={spy} />);
    const input = screen.getByRole('spinbutton', { name: 'Amount' });
    fireEvent.change(input, { target: { value: '-3' } });
    expect(spy).toHaveBeenLastCalledWith('-3');
    expect(input).toHaveValue('5');
  });
});

function ProjectProbe() {
  const { state } = useEnergy();
  const project = state.project;
  return <output data-testid="project">{JSON.stringify({
    zones: project.zones.map((zone) => ({ name: zone.name, floorArea: zone.floorArea })),
    construction: project.constructions[project.constructions.length - 1]?.layers,
    heating: project.heatingSystems.map((system) => system.cop),
    mwa: project.maatwerkadvies?.tariffs,
  })}</output>;
}

function stored() {
  return JSON.parse(screen.getByTestId('project').textContent ?? '{}');
}

describe('dialogs take a decimal comma', () => {
  it('stores a zone area typed with a comma, and 0 for an emptied field', () => {
    renderWithProviders(<><ZoneEditorDialog onClose={() => {}} /><ProjectProbe /></>);
    const [area, volume] = screen.getAllByRole('spinbutton') as HTMLInputElement[];
    fireEvent.change(area, { target: { value: '12,5' } });
    fireEvent.change(volume, { target: { value: '' } });
    expect(volume).toHaveValue('0');
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    const zones = stored().zones;
    expect(zones[zones.length - 1].floorArea).toBe(12.5);
  });

  it('stores a layer thickness typed with a comma in the construction editor', () => {
    renderWithProviders(<><ConstructionEditorDialog onClose={() => {}} /><ProjectProbe /></>);
    fireEvent.change(screen.getByLabelText('Thickness (m) 1'), { target: { value: '0,12' } });
    fireEvent.change(screen.getByLabelText('Lambda (W/mK) 1'), { target: { value: '0,035' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(stored().construction[0]).toMatchObject({ thickness: 0.12, lambda: 0.035 });
  });

  it('stores a COP typed with a comma in the heating-system dialog', () => {
    renderWithProviders(<><HeatingSystemDialog onClose={() => {}} /><ProjectProbe /></>);
    fireEvent.change(screen.getByLabelText('COP / Efficiency (-)'), { target: { value: '3,85' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    const heating = stored().heating;
    expect(heating[heating.length - 1]).toBe(3.85);
  });

  it('stores a gas tariff typed with a comma in the tailored advice', async () => {
    const user = userEvent.setup();
    renderWithProviders(<><MaatwerkadviesPanel /><ProjectProbe /></>);
    await user.click(screen.getByRole('button', { name: 'Start tailored advice' }));
    fireEvent.change(screen.getByLabelText('Gas [€/m³]'), { target: { value: '1,25' } });
    expect(stored().mwa.gasEurPerM3).toBe(1.25);
  });
});
