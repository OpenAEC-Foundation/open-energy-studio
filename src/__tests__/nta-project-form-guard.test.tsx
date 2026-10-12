import { useState } from 'react';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { afterAll, describe, expect, it, vi } from 'vitest';
import { act, cleanup, fireEvent, screen } from '@testing-library/react';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { NTA_SECTIONS, type NtaSectionDef } from '../components/NtaPerformancePanel/NtaSections';
import { syncVentilation } from '../core/nta/NtaFormModels';
import { projectRouteInput } from '../core/nta/KernelInput';
import type { IProject } from '../core/energy/types';
import office from '../../training-data/nta8800-example-office.json';
import terraced from '../../training-data/nta8800-example-terraced-dwelling.json';
import { renderWithProviders } from './test-utils';

// Structural guard for the project forms (reviews 7 and 8 found JSON the kernel refuses, stale
// fields after a switch, stale dependent fields and data lost on Toepassen). Every NTA section is
// rendered on a realistic project and exercised: every choice is switched through all its options,
// every checkbox is toggled, every "add" button is clicked once, and finally every number is
// emptied and every empty choice picked. After each action the draft goes through what
// "Toepassen" and the project-route call do (syncVentilation, projectRouteInput) and is collected.
// crates/nta8800-core/tests/project_form_guard.rs runs every collected project through the kernel
// (the gate passes the file with NTA_FORM_GUARD_DOCS) and refuses shape errors and leftover state.

interface GuardDoc { fixture: string; section: string; action: string; project: unknown }

const OUT = resolve(process.env.NTA_FORM_GUARD_OUT ?? 'nta-evidence/form-guard/project-forms.jsonl');
/** Snapshots per section and fixture; keeps the run within a few seconds per section. */
const MAX_SNAPSHOTS = 120;
const ADD_BUTTON = /^\+|\badd\b|toevoeg|own data for|eigen gegevens/i;
const SKIPPED_SECTIONS = new Set(['bcrg']); // a calculation tool that calls the kernel itself

/** The office with a second calculation zone: zone 1 again, every nested `id` given a `-2` suffix. */
function twoZoneOffice(): IProject {
  const project = structuredClone(office) as unknown as IProject & Record<string, unknown>;
  const suffix = (value: unknown): unknown => {
    if (Array.isArray(value)) return value.map(suffix);
    if (value && typeof value === 'object') {
      return Object.fromEntries(Object.entries(value).map(([key, item]) =>
        [key, key === 'id' && typeof item === 'string' ? `${item}-2` : suffix(item)]));
    }
    return value;
  };
  const second = suffix(structuredClone(project.zones[0])) as IProject['zones'][number];
  second.name = 'Verdieping';
  project.zones = [project.zones[0], second];
  return project;
}

/** The terraced dwelling on a heated basement with the depth per wall part (8.42/D.12). */
function terracedHeatedBasement(): IProject {
  const project = structuredClone(terraced) as unknown as IProject & { ntaCalculation: Record<string, unknown> };
  const floors = project.ntaCalculation.groundFloors as Array<Record<string, unknown>>;
  floors[0] = { ...floors[0], heatedBasement: {
    wallDepths: [{ lengthM: 10, depthM: 1.5 }, { lengthM: 5, depthM: 0.5 }], wallResistanceM2kPerW: 2.5,
  } };
  return project;
}

/** `sections`: only these sections run on the fixture (a variant for one form). */
const FIXTURES: Array<{ name: string; project: IProject; sections?: string[] }> = [
  { name: 'office', project: office as unknown as IProject },
  { name: 'terraced-dwelling', project: terraced as unknown as IProject },
  { name: 'office-two-zones', project: twoZoneOffice() },
  { name: 'terraced-heated-basement', project: terracedHeatedBasement(), sections: ['groundFloors'] },
];

const collected = new Map<string, GuardDoc>();
const counts: Record<string, number> = {};

function Harness({ def, project }: { def: NtaSectionDef; project: IProject }) {
  const [draft, setDraft] = useState<Draft>(project.ntaCalculation as unknown as Draft);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  const update = (next: (current: Draft) => Draft) => setDraft((current) => next(current));
  const Component = def.Component;
  return <form aria-label="guard">
    <Component draft={draft} change={change} update={update} project={project} />
    <output data-testid="draft">{JSON.stringify(draft)}</output>
  </form>;
}

function explore(def: NtaSectionDef, fixture: { name: string; project: IProject }) {
  renderWithProviders(<Harness def={def} project={fixture.project} />);
  const form = screen.getByRole('form', { name: 'guard' });
  let snapshots = 0;
  const snap = (action: string) => {
    if (snapshots >= MAX_SNAPSHOTS) return;
    snapshots += 1;
    const draft = JSON.parse(screen.getByTestId('draft').textContent ?? '{}') as Draft;
    // What "Toepassen" writes and what the project route receives (projectRouteInput: blanks kept as null).
    const applied = syncVentilation(draft, fixture.project) as Draft;
    const project = projectRouteInput({ ...fixture.project, ntaCalculation: applied as never });
    const key = `${fixture.name}\u0000${JSON.stringify(project.ntaCalculation)}`;
    if (!collected.has(key)) collected.set(key, { fixture: fixture.name, section: def.id, action, project });
  };
  const selects = () => Array.from(form.querySelectorAll('select'));
  const switchAll = (label: string) => {
    for (let index = 0; index < selects().length && snapshots < MAX_SNAPSHOTS; index += 1) {
      const element = selects()[index];
      const original = element.value;
      for (const value of Array.from(element.options).map((option) => option.value)) {
        const current = selects()[index];
        if (!current || current.value === value) continue;
        act(() => { fireEvent.change(current, { target: { value } }); });
        snap(`${label} select ${index} = ${value}`);
      }
      const back = selects()[index];
      if (back && back.value !== original) act(() => { fireEvent.change(back, { target: { value: original } }); });
    }
  };
  const toggleAll = (label: string) => {
    const boxes = Array.from(form.querySelectorAll<HTMLInputElement>('input[type="checkbox"]'));
    boxes.forEach((_, index) => {
      const box = form.querySelectorAll<HTMLInputElement>('input[type="checkbox"]')[index];
      if (!box) return;
      act(() => { fireEvent.click(box); });
      snap(`${label} toggle ${index}`);
      const again = form.querySelectorAll<HTMLInputElement>('input[type="checkbox"]')[index];
      if (again) act(() => { fireEvent.click(again); });
    });
  };
  const addAll = () => {
    const buttons = Array.from(form.querySelectorAll('button')).filter((button) => ADD_BUTTON.test(button.textContent ?? ''));
    buttons.forEach((button, index) => {
      if (!button.isConnected) return;
      act(() => { fireEvent.click(button); });
      snap(`add ${index} (${button.textContent})`);
    });
  };
  const emptyAll = () => {
    for (const input of Array.from(form.querySelectorAll<HTMLInputElement>('[role="spinbutton"]'))) {
      act(() => { fireEvent.change(input, { target: { value: '' } }); fireEvent.blur(input); });
    }
    for (const element of selects()) {
      if (Array.from(element.options).some((option) => option.value === '')) {
        act(() => { fireEvent.change(element, { target: { value: '' } }); });
      }
    }
    snap('emptied');
  };

  snap('initial');
  switchAll('initial');
  toggleAll('initial');
  addAll();
  switchAll('after add');
  toggleAll('after add');
  emptyAll();
  cleanup();
  counts[`${fixture.name}/${def.id}`] = snapshots;
}

describe('project form guard', () => {
  const confirm = vi.spyOn(window, 'confirm').mockReturnValue(true);
  for (const fixture of FIXTURES) {
    // The sections the step pages show for this project (`when`), as the adviser sees them.
    const shown = NTA_SECTIONS.filter((section) => !SKIPPED_SECTIONS.has(section.id)
      && (!fixture.sections || fixture.sections.includes(section.id))
      && (!section.when || section.when(fixture.project, fixture.project.ntaCalculation as unknown as Draft)));
    for (const def of shown) {
      it(`${fixture.name}: ${def.id}`, () => {
        explore(def, fixture);
        expect(counts[`${fixture.name}/${def.id}`]).toBeGreaterThan(0);
      });
    }
  }

  afterAll(() => {
    confirm.mockRestore();
    mkdirSync(dirname(OUT), { recursive: true });
    writeFileSync(OUT, [...collected.values()].map((doc) => JSON.stringify(doc)).join('\n') + '\n');
  });
});
