import { useState } from 'react';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { afterAll, describe, expect, it, vi } from 'vitest';
import { act, cleanup, fireEvent, screen } from '@testing-library/react';
import { FieldPathPrefixProvider, write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { NTA_SECTIONS, type NtaSectionDef } from '../components/NtaPerformancePanel/NtaSections';
import { NTA_PATH_PREFIX } from '../components/NtaPerformancePanel/NtaCalculationForm';
import { syncVentilation } from '../core/nta/NtaFormModels';
import { projectRouteInput } from '../core/nta/KernelInput';
import { routeForPath } from '../core/nta/gapRoutes';
import { normalizeRoute } from '../core/navigation/routes';
import type { IProject } from '../core/energy/types';
import office from '../../training-data/nta8800-example-office.json';
import terraced from '../../training-data/nta8800-example-terraced-dwelling.json';
import caseN from '../../training-data/nta8800-public-comparison-n.json';
import { renderWithProviders } from './test-utils';

// Review 58: one user action, one gap at the field the user changed. Every field an adviser can
// clear (a number, a choice with an empty option, a text) is cleared on its own, through the
// form's own change handler, on a fresh copy of the example project. The draft goes through what
// "Toepassen" and the project route do (syncVentilation, projectRouteInput) and is written out;
// crates/nta8800-core/tests/project_form_guard.rs (`every_cleared_field_is_one_gap_at_that_field`)
// runs each through the kernel and allows only gaps at that field (or below it). Here: the field's
// own path must route "Ga naar" to the page of its section.

interface ClearDoc { fixture: string; section: string; field: string; kind: string; project: unknown }

const OUT = resolve(process.env.NTA_CLEAR_SWEEP_OUT ?? 'nta-evidence/form-guard/clear-fields.jsonl');
const SKIPPED_SECTIONS = new Set(['bcrg']); // a calculation tool that calls the kernel itself

const FIXTURES: Array<{ name: string; project: IProject }> = [
  { name: 'office', project: office as unknown as IProject },
  { name: 'terraced-dwelling', project: terraced as unknown as IProject },
  { name: 'public-case-n', project: caseN as unknown as IProject },
];

const collected: ClearDoc[] = [];
const misrouted: string[] = [];
const notBlank: string[] = [];

function Harness({ def, project }: { def: NtaSectionDef; project: IProject }) {
  const [draft, setDraft] = useState<Draft>(project.ntaCalculation as unknown as Draft);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  const update = (next: (current: Draft) => Draft) => setDraft((current) => next(current));
  const Component = def.Component;
  return <form aria-label="sweep">
    <FieldPathPrefixProvider value={NTA_PATH_PREFIX}>
      <Component draft={draft} change={change} update={update} project={project} />
    </FieldPathPrefixProvider>
    <output data-testid="draft">{JSON.stringify(draft)}</output>
  </form>;
}

/** The clearable control of a labelled field, and how it is cleared. */
function clearable(label: Element): { kind: string; clear: () => void } | null {
  const control = label.querySelector<HTMLInputElement | HTMLSelectElement>(':scope > input, :scope > select, :scope input, :scope select');
  if (!control || control.disabled) return null;
  if (control instanceof HTMLSelectElement) {
    // Only "—" clears; an empty value with its own text (a declared F_c) is a choice like any other.
    if (!Array.from(control.options).some((option) => option.value === '' && option.text === '—') || control.value === '') return null;
    return { kind: 'select', clear: () => fireEvent.change(control, { target: { value: '' } }) };
  }
  if (control.type === 'checkbox') return null;
  if (control.getAttribute('role') === 'spinbutton') {
    if (control.value === '') return null;
    return { kind: 'number', clear: () => { fireEvent.change(control, { target: { value: '' } }); fireEvent.blur(control); } };
  }
  if (control.value === '') return null;
  return { kind: 'text', clear: () => fireEvent.change(control, { target: { value: '' } }) };
}

function fieldsOf(def: NtaSectionDef, project: IProject): string[] {
  renderWithProviders(<Harness def={def} project={project} />);
  const form = screen.getByRole('form', { name: 'sweep' });
  const paths = Array.from(form.querySelectorAll('label[data-path]'))
    .filter((label) => clearable(label) != null)
    .map((label) => label.getAttribute('data-path') as string);
  cleanup();
  return [...new Set(paths)];
}

function sweep(def: NtaSectionDef, fixture: { name: string; project: IProject }): number {
  const fields = fieldsOf(def, fixture.project);
  const expected = normalizeRoute({ step: def.step, sub: def.sub });
  for (const field of fields) {
    renderWithProviders(<Harness def={def} project={fixture.project} />);
    const form = screen.getByRole('form', { name: 'sweep' });
    const label = form.querySelector(`label[data-path="${CSS.escape(field)}"]`);
    const control = label && clearable(label);
    if (control) {
      act(() => { control.clear(); });
      // "—" must leave the choice blank; a "—" that quietly picks another option (a template of a
      // tagged choice) is an option the adviser cannot mean.
      const after = form.querySelector(`label[data-path="${CSS.escape(field)}"] select`) as HTMLSelectElement | null;
      if (control.kind === 'select' && after && after.value !== '') {
        notBlank.push(`${fixture.name}/${def.id}: ${field} shows "${after.value}" after "—"`);
      }
      const draft = JSON.parse(screen.getByTestId('draft').textContent ?? '{}') as Draft;
      const applied = syncVentilation(draft, fixture.project) as Draft;
      const project = projectRouteInput({ ...fixture.project, ntaCalculation: applied as never });
      collected.push({ fixture: fixture.name, section: def.id, field, kind: control.kind, project });
      const route = normalizeRoute(routeForPath(field));
      if (route.step !== expected.step || (expected.sub && route.sub !== expected.sub)) {
        misrouted.push(`${fixture.name}/${def.id}: ${field} → ${route.step}/${route.sub ?? ''}, section on ${expected.step}/${expected.sub ?? ''}`);
      }
    }
    cleanup();
  }
  return fields.length;
}

describe('clear-field sweep (review 58)', () => {
  const confirm = vi.spyOn(window, 'confirm').mockReturnValue(true);
  for (const fixture of FIXTURES) {
    // The untouched project: the gaps it already has are not the cleared field's.
    const applied = syncVentilation(fixture.project.ntaCalculation as unknown as Draft, fixture.project) as Draft;
    collected.push({ fixture: fixture.name, section: '', field: '', kind: 'baseline',
      project: projectRouteInput({ ...fixture.project, ntaCalculation: applied as never }) });
    const shown = NTA_SECTIONS.filter((section) => !SKIPPED_SECTIONS.has(section.id)
      && (!section.when || section.when(fixture.project, fixture.project.ntaCalculation as unknown as Draft)));
    for (const def of shown) {
      it(`${fixture.name}: ${def.id}`, () => {
        sweep(def, fixture);
      });
    }
  }

  it('routes every cleared field to the page of its section', () => {
    expect(misrouted).toEqual([]);
  });

  it('leaves a choice blank after "—"', () => {
    expect(notBlank).toEqual([]);
  });

  afterAll(() => {
    confirm.mockRestore();
    mkdirSync(dirname(OUT), { recursive: true });
    writeFileSync(OUT, collected.map((doc) => JSON.stringify(doc)).join('\n') + '\n');
  });
});
