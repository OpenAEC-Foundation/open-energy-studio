import { useEffect, useState } from 'react';
import { describe, expect, it } from 'vitest';
import { cleanup, fireEvent, screen, waitFor, within } from '@testing-library/react';
import { FieldPathPrefixProvider, write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { NTA_SECTIONS, type NtaSectionDef } from '../components/NtaPerformancePanel/NtaSections';
import { BasisopnamePanel } from '../components/BasisopnamePanel/BasisopnamePanel';
import { SurveyTakeoverAction } from '../components/BasisopnamePanel/SurveyTakeoverAction';
import { SettingsDialog } from '../components/SettingsDialog/SettingsDialog';
import { ProjectInfoDialog } from '../components/dialogs/ProjectInfoDialog/ProjectInfoDialog';
import { AirTightnessDialog } from '../components/dialogs/AirTightnessDialog/AirTightnessDialog';
import { ConstructionEditorDialog } from '../components/dialogs/ConstructionEditorDialog/ConstructionEditorDialog';
import { CoolingSystemDialog } from '../components/dialogs/CoolingSystemDialog/CoolingSystemDialog';
import { FeedbackDialog } from '../components/dialogs/FeedbackDialog/FeedbackDialog';
import { HeatingSystemDialog } from '../components/dialogs/HeatingSystemDialog/HeatingSystemDialog';
import { HotWaterSystemDialog } from '../components/dialogs/HotWaterSystemDialog/HotWaterSystemDialog';
import { PointBridgeDialog } from '../components/dialogs/PointBridgeDialog/PointBridgeDialog';
import { SolarPVDialog } from '../components/dialogs/SolarPVDialog/SolarPVDialog';
import { SolarThermalDialog } from '../components/dialogs/SolarThermalDialog/SolarThermalDialog';
import { SurfaceEditorDialog } from '../components/dialogs/SurfaceEditorDialog/SurfaceEditorDialog';
import { ThermalBridgeDialog } from '../components/dialogs/ThermalBridgeDialog/ThermalBridgeDialog';
import { VentilationSystemDialog } from '../components/dialogs/VentilationSystemDialog/VentilationSystemDialog';
import { WindowEditorDialog } from '../components/dialogs/WindowEditorDialog/WindowEditorDialog';
import { ZoneEditorDialog } from '../components/dialogs/ZoneEditorDialog/ZoneEditorDialog';
import { CommandPalette } from '../components/shell/CommandPalette';
import { focusPathIn } from '../components/shell/focusPath';
import { applyIssueMarks } from '../components/shell/NtaStepPage';
import { Field, TextInput } from '../components/ui';
import { en } from '../i18n/en';
import { WorkflowNav } from '../components/shell/WorkflowNav';
import { StepRouter } from '../components/shell/StepRouter';
import type { ShellActions } from '../components/shell/ShellActions';
import { useEnergy } from '../context/EnergyContext';
import type { IProject } from '../core/energy/types';
import type { BuildingPerformanceInput, OpnameAssessment } from '../core/nta/KernelClient';
import { stepStatuses } from '../core/nta/stepStatus';
import { exampleProject, type ExampleKind } from '../core/nta/ExampleProjects';
import office from '../../training-data/nta8800-example-office.json';
import terraced from '../../training-data/nta8800-example-terraced-dwelling.json';
import takeoverFixture from '../../training-data/nta8800-survey-takeover-ground.json';
import { auditAccessibility } from './a11yAudit';
import { renderWithProviders } from './test-utils';
import { SLOW_UI_TEST_MS } from './timeouts';

// Accessibility and keyboard audit of the NTA forms and dialogs (10 October 2026). The shared
// audit (a11yAudit.ts) runs over every NTA section on both examples with their lists opened, both
// surveys, the survey takeover dialog, the registration page and the shell dialogs. New forms are
// covered automatically through NTA_SECTIONS, so this file is the regression guard.

const ADD = /^\+|\badd\b|toevoeg/i;

function Harness({ def, project }: { def: NtaSectionDef; project: IProject }) {
  const [draft, setDraft] = useState<Draft>(project.ntaCalculation as unknown as Draft);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  const update = (next: (current: Draft) => Draft) => setDraft((current) => next(current));
  const Component = def.Component;
  // The prefix gives every field its kernel `data-path`, as on the step page.
  return <FieldPathPrefixProvider value="ntaCalculation">
    <form aria-label="audit"><Component draft={draft} change={change} update={update} project={project} /></form>
  </FieldPathPrefixProvider>;
}

function openLists(root: HTMLElement) {
  for (const button of Array.from(root.querySelectorAll('button')).filter((item) => ADD.test(item.textContent ?? ''))) {
    try { fireEvent.click(button); } catch { /* a button that needs more context; the audit still runs */ }
  }
}

function Seed({ value }: { value: IProject }) {
  const { dispatch } = useEnergy();
  useEffect(() => { dispatch({ type: 'SET_PROJECT', payload: structuredClone(value) }); }, [dispatch, value]);
  return null;
}

const noop = () => undefined;
const actions: ShellActions = {
  newProject: noop, openProject: noop, saveProject: noop, saveAsProject: noop, calculate: noop,
  openDialog: noop, navigate: noop, exportReport: noop, printReport: noop, exportIFC: noop,
  exportModelIFC: noop, exportUNIEC3: noop, importUNIEC3: noop, exportVABI: noop, importVABI: noop,
  openSettings: noop, openFeedback: noop, openPalette: noop, toggleInspector: noop, togglePreview: noop,
};

function Registration({ example }: { example: ExampleKind }) {
  const { state, dispatch } = useEnergy();
  const [loaded, setLoaded] = useState(false);
  useEffect(() => { dispatch({ type: 'SET_PROJECT', payload: exampleProject(example) }); setLoaded(true); }, [dispatch, example]);
  if (!loaded) return null;
  const route = { step: 'registration' as const };
  const statuses = stepStatuses(state.project, null);
  return <>
    <WorkflowNav project={state.project} route={route} statuses={statuses} actions={actions} />
    <main><StepRouter project={state.project} route={route} statuses={statuses} actions={actions} /></main>
  </>;
}

const takeoverProject = (): IProject => {
  const value = structuredClone(terraced) as unknown as IProject;
  for (const zone of value.zones) for (const surface of zone.surfaces) if (surface.id === 'floor') surface.area = takeoverFixture.surveyFloor.areaM2;
  return value;
};

const takeoverAssessment = {
  status: 'calculated_unverified', scope: 'residential', source: 'ISSO 82.1', appliedDefaults: [], warnings: [], issues: [],
  derivedInput: {
    spaceHeating: { demand: { transmission: { groundFloors: [takeoverFixture.surveyFloor] } } },
    hotWater: { generator: { kind: 'gas_combi' } },
  } as unknown as BuildingPerformanceInput,
  performance: null, referenceVerified: false,
} as unknown as OpnameAssessment;

describe('accessibility audit: NTA sections', () => {
  for (const fixture of [{ name: 'office', project: office }, { name: 'terraced dwelling', project: terraced }]) {
    it(`every section on the ${fixture.name}`, () => {
      const problems: string[] = [];
      for (const def of NTA_SECTIONS) {
        if (def.id === 'bcrg') continue; // a calculation tool, not an input section
        renderWithProviders(<Harness def={def} project={fixture.project as unknown as IProject} />);
        const form = screen.getByRole('form', { name: 'audit' });
        openLists(form);
        problems.push(...auditAccessibility(document.body).map((line) => `${def.id}: ${line}`));
        cleanup();
      }
      expect(problems).toEqual([]);
    }, 180000);
  }
});

describe('accessibility audit: survey, takeover, registration and shell dialogs', () => {
  for (const start of ['Start dwelling survey', 'Start utility survey']) {
    it(`the survey after "${start}"`, () => {
      renderWithProviders(<BasisopnamePanel />);
      fireEvent.click(screen.getByRole('button', { name: start }));
      openLists(document.body);
      expect(auditAccessibility(document.body)).toEqual([]);
    }, 120000);
  }

  it('the survey takeover dialog', async () => {
    renderWithProviders(<><Seed value={takeoverProject()} /><SurveyTakeoverAction result={takeoverAssessment} /></>);
    fireEvent.click(await screen.findByRole('button', { name: /projectmodel|project model/i }));
    await screen.findByRole('dialog');
    expect(auditAccessibility(document.body)).toEqual([]);
  });

  for (const example of ['terraced_dwelling', 'small_office'] as const) {
    it(`the registration page (${example})`, async () => {
      renderWithProviders(<Registration example={example} />);
      await screen.findByRole('heading', { level: 1 });
      expect(auditAccessibility(document.body)).toEqual([]);
    }, SLOW_UI_TEST_MS);
  }

  it('the settings, project and command-palette dialogs', () => {
    const problems: string[] = [];
    renderWithProviders(<SettingsDialog onClose={noop} />);
    problems.push(...auditAccessibility(document.body).map((line) => `settings: ${line}`));
    cleanup();
    renderWithProviders(<ProjectInfoDialog onClose={noop} />);
    problems.push(...auditAccessibility(document.body).map((line) => `project info: ${line}`));
    cleanup();
    renderWithProviders(<CommandPalette entries={[]} onClose={noop} />);
    problems.push(...auditAccessibility(document.body).map((line) => `palette: ${line}`));
    expect(problems).toEqual([]);
  });
});

describe('errors are linked to their field', () => {
  it('a kernel error on a section field marks it invalid with a description, and the audit passes', () => {
    const t = (key: string) => (en as Record<string, string>)[key] ?? key;
    const def = NTA_SECTIONS.find((section) => section.id === 'general')!;
    renderWithProviders(<Harness def={def} project={office as unknown as IProject} />);
    const form = screen.getByRole('form', { name: 'audit' });
    const paths = Array.from(form.querySelectorAll<HTMLElement>('label[data-path]')).slice(0, 3).map((label) => label.dataset.path!);
    expect(paths.length).toBeGreaterThan(0);
    applyIssueMarks(form, paths.map((focus) => ({ kind: 'error' as const, code: 'nta_value_missing', focus })), t);
    expect(form.querySelectorAll('[aria-invalid="true"]').length).toBe(paths.length);
    expect(auditAccessibility(document.body)).toEqual([]);
  });

  it('a form field error is announced and linked with aria-describedby', () => {
    renderWithProviders(<Field label="Area" error="Must be positive"><TextInput value="-1" onChange={noop} /></Field>);
    const input = screen.getByRole('textbox', { name: /Area/ });
    expect(input.getAttribute('aria-invalid')).toBe('true');
    expect(screen.getByRole('alert').id).toBe(input.getAttribute('aria-describedby'));
    expect(auditAccessibility(document.body)).toEqual([]);
  });
});

describe('keyboard: "Ga naar" lands on the field', () => {
  for (const fixture of [{ name: 'office', project: office }, { name: 'terraced dwelling', project: terraced }]) {
    it(`every labelled field of every section on the ${fixture.name} takes the focus`, () => {
      const problems: string[] = [];
      let checked = 0;
      for (const def of NTA_SECTIONS) {
        if (def.id === 'bcrg') continue;
        renderWithProviders(<Harness def={def} project={fixture.project as unknown as IProject} />);
        const form = screen.getByRole('form', { name: 'audit' });
        openLists(form);
        for (const label of Array.from(form.querySelectorAll<HTMLElement>('label[data-path], .ui-field[data-path]'))) {
          const path = label.dataset.path!;
          // Only fields that own their path: an outer label shares it with an inner, more specific field.
          if (Array.from(form.querySelectorAll<HTMLElement>('[data-path]')).some((other) => other !== label && other.dataset.path === path && label.compareDocumentPosition(other) & Node.DOCUMENT_POSITION_FOLLOWING)) continue;
          const control = label.querySelector('input, select, textarea, button');
          if (!control || control.matches(':disabled')) continue; // a disabled control cannot take the focus by design
          checked += 1;
          const target = focusPathIn(form, path);
          if (document.activeElement !== target || !target || !/^(INPUT|SELECT|TEXTAREA|BUTTON)$/.test(target.tagName)) {
            problems.push(`${def.id}: ${path} → ${document.activeElement?.tagName ?? 'nothing'}`);
          }
        }
        cleanup();
      }
      expect(problems).toEqual([]);
      expect(checked).toBeGreaterThan(100);
    }, 180000);
  }
});

const EDITOR_DIALOGS = {
  AirTightnessDialog, ConstructionEditorDialog, CoolingSystemDialog, FeedbackDialog, HeatingSystemDialog, HotWaterSystemDialog,
  PointBridgeDialog, SolarPVDialog, SolarThermalDialog, SurfaceEditorDialog, ThermalBridgeDialog, VentilationSystemDialog,
  WindowEditorDialog, ZoneEditorDialog,
} as const;

describe('accessibility audit: editor dialogs', () => {
  for (const [name, Component] of Object.entries(EDITOR_DIALOGS)) {
    it(name, async () => {
      renderWithProviders(<><Seed value={office as unknown as IProject} /><Component onClose={noop} /></>);
      await screen.findByRole('dialog');
      expect(auditAccessibility(document.body)).toEqual([]);
    });
  }
});

describe('dialog field errors', () => {
  it('a refused save links each error to its field and clears the link once fixed', async () => {
    renderWithProviders(<><Seed value={office as unknown as IProject} /><ThermalBridgeDialog onClose={noop} /></>);
    const dialog = await screen.findByRole('dialog');
    fireEvent.click(within(dialog).getByRole('button', { name: /save|opslaan/i }));
    const invalid = await waitFor(() => {
      const found = Array.from(dialog.querySelectorAll('[aria-invalid="true"]'));
      expect(found.length).toBeGreaterThan(0);
      return found;
    });
    for (const control of invalid) {
      const error = document.getElementById((control.getAttribute('aria-describedby') ?? '').split(' ').pop()!);
      expect(error?.classList.contains('field-error-text')).toBe(true);
    }
    expect(auditAccessibility(document.body)).toEqual([]);
    const name = within(dialog).getByRole('textbox', { name: /name|naam/i });
    fireEvent.change(name, { target: { value: 'Balkonplaat' } });
    await waitFor(() => expect(name.hasAttribute('aria-invalid')).toBe(false));
    expect(name.hasAttribute('aria-describedby')).toBe(false);
  });
});

describe('keyboard: dialogs', () => {
  it('the takeover dialog takes focus, traps Tab, closes on Escape and returns focus to its trigger', async () => {
    renderWithProviders(<><Seed value={takeoverProject()} /><SurveyTakeoverAction result={takeoverAssessment} /></>);
    const trigger = await screen.findByRole('button', { name: /projectmodel|project model/i });
    trigger.focus();
    fireEvent.click(trigger);
    const dialog = await screen.findByRole('dialog');
    await waitFor(() => expect(dialog.contains(document.activeElement)).toBe(true));

    const focusable = Array.from(dialog.querySelectorAll<HTMLElement>('button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), a[href]'));
    const last = focusable[focusable.length - 1];
    last.focus();
    fireEvent.keyDown(document.activeElement ?? document.body, { key: 'Tab' });
    expect(document.activeElement).toBe(focusable[0]);
    focusable[0].focus();
    fireEvent.keyDown(document.activeElement ?? document.body, { key: 'Tab', shiftKey: true });
    expect(document.activeElement).toBe(last);

    fireEvent.keyDown(document.activeElement ?? document.body, { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(document.activeElement).toBe(trigger);
  });

  it('the settings dialog closes on Escape', () => {
    let closed = false;
    renderWithProviders(<SettingsDialog onClose={() => { closed = true; }} />);
    const dialog = screen.getByRole('dialog');
    expect(dialog.contains(document.activeElement)).toBe(true);
    fireEvent.keyDown(document.activeElement ?? document.body, { key: 'Escape' });
    expect(closed).toBe(true);
  });

  it('the command palette closes on Escape', () => {
    let closed = false;
    renderWithProviders(<CommandPalette entries={[]} onClose={() => { closed = true; }} />);
    const dialog = screen.getByRole('dialog');
    fireEvent.keyDown(within(dialog).getByRole('combobox'), { key: 'Escape' });
    expect(closed).toBe(true);
  });
});
