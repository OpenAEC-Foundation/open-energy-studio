import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { defaultValueLabel, BuildingFunctionFields } from '../components/BasisopnamePanel/BasisopnamePanel';
import { dutchDefaultValue } from '../core/nta/OpnameValueText';
import { relabelValue } from '../core/nta/RelabelText';
import { kernelNote } from '../core/nta/KernelNoteText';
import { buildTemplatePatch, insulationOptions } from '../core/nta/MwaTemplates';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import type { IProject } from '../core/energy/types';
import { renderWithProviders, userEvent } from './test-utils';

const t = (key: string, options?: Record<string, unknown>) =>
  options ? `${key}${JSON.stringify(options)}` : key;

describe('walkthrough 3 fixes', () => {
  it('shows recorded years without a thousands separator and translates number-built texts', () => {
    render(<span data-testid="year">{defaultValueLabel(t, '1985', 'nl-NL')}</span>);
    expect(screen.getByTestId('year').textContent).toBe('1985');
    render(<span data-testid="dec">{defaultValueLabel(t, '1.47', 'nl-NL')}</span>);
    expect(screen.getByTestId('dec').textContent).toBe('1,47');
    expect(dutchDefaultValue('R_c 1.47 m²K/W (construction year)')).toBe('R_c 1,47 m²K/W (bouwjaar)');
    expect(dutchDefaultValue('outside (A_g 1300 m² > 500 m²)')).toBe('buiten de thermische zone (A_g 1300 m² > 500 m²)');
    expect(dutchDefaultValue('200 m² of other functions (≤ 25 %) counted as education'))
      .toBe('200 m² van andere functies (≤ 25 %) geteld als onderwijs');
    expect(dutchDefaultValue('free text 2.5 m')).toBe('free text 2,5 m');
    expect(dutchDefaultValue('free text')).toBeNull();
  });

  it('summarises an added relabel element instead of printing JSON', () => {
    const text = relabelValue(t, 'nl-NL', { id: 'w9', name: 'Dakraam nieuw', area: 2, uValue: 1.1 });
    expect(text).not.toContain('{');
    expect(text).toContain('Dakraam nieuw');
    expect(text).toContain('relabel.value.area 2 m²');
    expect(text).toContain('1,1');
    expect(relabelValue(t, 'nl-NL', [1, 2, 3])).toBe('relabel.value.items{"count":3}');
  });

  it('translates the kernel notes in Dutch and leaves English as written', () => {
    const note = '8.5: H_A = 0 for adjacent heated spaces is prescribed by the norm';
    expect(kernelNote(note, 'nl')).toContain('aangrenzende verwarmde ruimten');
    expect(kernelNote(note, 'en')).toBe(note);
    expect(kernelNote('unknown note', 'nl')).toBe('unknown note');
  });

  it('names the new construction by its Rc and warns when it is not better', () => {
    const project = JSON.parse(readFileSync(resolve(process.cwd(), 'training-data/nta8800-example-terraced-dwelling.json'), 'utf8')) as IProject;
    const surfaces = insulationOptions(project, 'facade').map((option) => option.key);
    const better = buildTemplatePatch(project, { kind: 'insulation', part: 'facade', surfaces, rcValue: 8, uValue: null }, 'm1');
    const operation = better.patch.find((item) => item.path === '/constructions/-');
    const added = (operation && 'value' in operation ? operation.value : null) as { name: string };
    expect(added.name).toContain('→ Rc 8');
    expect(better.warnings).toEqual([]);
    const worse = buildTemplatePatch(project, { kind: 'insulation', part: 'facade', surfaces, rcValue: 1, uValue: null }, 'm1');
    expect(worse.warnings).toEqual(['notImproved']);
    expect(worse.problems).toEqual([]);
  });

  it('edits the use functions of a utility survey', async () => {
    const user = userEvent.setup();
    let draft: Record<string, unknown> = { functions: [{ function: 'education', areaM2: 1400 }] };
    const change = (path: Array<string | number>, value: unknown) => {
      const next = structuredClone(draft) as Record<string, unknown>;
      let target: Record<string | number, unknown> = next;
      for (const key of path.slice(0, -1)) target = target[key] as Record<string | number, unknown>;
      target[path[path.length - 1]] = value;
      draft = next;
    };
    const { rerender } = renderWithProviders(<BuildingFunctionFields draft={draft} change={change} t={(key) => key} />);
    await user.click(screen.getByRole('button', { name: 'opname.zones.addFunction' }));
    rerender(<BuildingFunctionFields draft={draft} change={change} t={(key) => key} />);
    expect((draft.functions as unknown[]).length).toBe(2);
  }, 60000);
});
