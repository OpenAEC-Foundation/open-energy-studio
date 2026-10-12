import { useEffect, useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { FieldPathPrefixProvider, write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { NTA_SECTIONS, type NtaSectionDef } from '../components/NtaPerformancePanel/NtaSections';
import { StepRouter } from '../components/shell/StepRouter';
import { WorkflowNav } from '../components/shell/WorkflowNav';
import type { ShellActions } from '../components/shell/ShellActions';
import { ManualView } from '../components/ManualView/ManualView';
import { useEnergy } from '../context/EnergyContext';
import type { IProject } from '../core/energy/types';
import type { ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import { exampleProject, type ExampleKind } from '../core/nta/ExampleProjects';
import { stepStatuses } from '../core/nta/stepStatus';
import { MANUAL_CHAPTERS } from '../core/manual/manual';
import { WORKFLOW_STEPS, TOOL_STEP, type StepId } from '../core/navigation/routes';
import { generateEnergyPerformanceReportHTML } from '../core/report/EnergyPerformanceReport';
import { generateNtaCalculationReportHTML } from '../core/report/NtaCalculationReport';
import { generateNtaInputDossierHTML } from '../core/report/NtaInputDossier';
import i18next from '../i18n/i18n';
import office from '../../training-data/nta8800-example-office.json';
import terraced from '../../training-data/nta8800-example-terraced-dwelling.json';
import { renderWithProviders } from './test-utils';
import { SLOW_UI_TEST_MS } from './timeouts';

// No visible text shows an unfilled placeholder: "{{name}}" left by t() without the
// variable, or "{name}" left by a single-brace label given to t(). Covers the NTA
// sections, every workflow page, the Handleiding window and the reports, in Dutch
// and English. The static side is in i18n-placeholders.test.ts.

const PLACEHOLDER = /\{\{[^}]*\}\}|\{[A-Za-z_][\w.]*\}/;
const ADD = /^\+|\badd\b|toevoeg/i;

/** Text and the readable attributes of every element under `root` that hold a placeholder. */
function unfilled(root: ParentNode): string[] {
  const found: string[] = [];
  const walker = document.createTreeWalker(root as Node, NodeFilter.SHOW_TEXT);
  for (let node = walker.nextNode(); node; node = walker.nextNode()) {
    const parent = node.parentElement;
    if (parent && (parent.closest('code, pre, script, style'))) continue;
    const text = node.textContent ?? '';
    if (PLACEHOLDER.test(text)) found.push(text.trim().slice(0, 120));
  }
  for (const element of Array.from((root as Element).querySelectorAll?.('[aria-label], [title], [placeholder], [alt]') ?? [])) {
    for (const attribute of ['aria-label', 'title', 'placeholder', 'alt']) {
      const value = element.getAttribute(attribute);
      if (value && PLACEHOLDER.test(value)) found.push(`${attribute}="${value.slice(0, 120)}"`);
    }
  }
  return [...new Set(found)];
}

function Section({ def, project }: { def: NtaSectionDef; project: IProject }) {
  const [draft, setDraft] = useState<Draft>(project.ntaCalculation as unknown as Draft);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  const update = (next: (current: Draft) => Draft) => setDraft((current) => next(current));
  const Component = def.Component;
  return <FieldPathPrefixProvider value="ntaCalculation">
    <form aria-label="section"><Component draft={draft} change={change} update={update} project={project} /></form>
  </FieldPathPrefixProvider>;
}

const noop = () => undefined;
const actions: ShellActions = {
  newProject: noop, openProject: noop, saveProject: noop, saveAsProject: noop, calculate: noop,
  openDialog: noop, navigate: noop, exportReport: noop, printReport: noop, exportIFC: noop,
  exportModelIFC: noop, exportUNIEC3: noop, importUNIEC3: noop, exportVABI: noop, importVABI: noop,
  openSettings: noop, openFeedback: noop, openPalette: noop, toggleInspector: noop, togglePreview: noop,
};

function Page({ example, step, sub }: { example: ExampleKind; step: StepId; sub?: string }) {
  const { state, dispatch } = useEnergy();
  const [loaded, setLoaded] = useState(false);
  useEffect(() => { dispatch({ type: 'SET_PROJECT', payload: exampleProject(example) }); setLoaded(true); }, [dispatch, example]);
  if (!loaded) return null;
  const route = { step, ...(sub ? { sub } : {}) };
  const statuses = stepStatuses(state.project, null);
  return <>
    <WorkflowNav project={state.project} route={route} statuses={statuses} actions={actions} />
    <main><StepRouter project={state.project} route={route} statuses={statuses} actions={actions} /></main>
  </>;
}

/** Workflow pages; the 3D model needs WebGL and the manual is covered on its own. */
const PAGES: Array<{ step: StepId; sub?: string }> = [...WORKFLOW_STEPS, TOOL_STEP]
  .flatMap((step): Array<{ step: StepId; sub?: string }> => (step.subs.length ? step.subs.map((sub) => ({ step: step.id, sub: sub.id })) : [{ step: step.id }]))
  .filter((page) => page.sub !== 'model3d' && page.sub !== 'manual');

const root = resolve(__dirname, '../..');
const reportInput = (name: string) => ({
  project: JSON.parse(readFileSync(resolve(root, `training-data/nta8800-example-${name}.json`), 'utf8')) as IProject,
  assessment: JSON.parse(readFileSync(resolve(root, `training-data/nta8800-example-${name}.kernel-output.json`), 'utf8')) as ProjectPerformanceAssessment,
});

function htmlProblems(html: string): string[] {
  const doc = new DOMParser().parseFromString(html, 'text/html');
  return unfilled(doc.body);
}

describe('no unfilled placeholder is shown', () => {
  for (const lng of ['nl', 'en']) {
    it(`NTA sections on both examples (${lng})`, async () => {
      await i18next.changeLanguage(lng);
      const problems: string[] = [];
      try {
        for (const [name, project] of [['office', office], ['terraced dwelling', terraced]] as const) {
          for (const def of NTA_SECTIONS) {
            if (def.id === 'bcrg') continue;
            renderWithProviders(<Section def={def} project={project as unknown as IProject} />);
            const form = screen.getByRole('form', { name: 'section' });
            for (const button of Array.from(form.querySelectorAll('button')).filter((item) => ADD.test(item.textContent ?? ''))) {
              try { fireEvent.click(button); } catch { /* needs more context */ }
            }
            problems.push(...unfilled(document.body).map((line) => `${name} ${def.id}: ${line}`));
            cleanup();
          }
        }
      } finally {
        await i18next.changeLanguage('en');
      }
      expect(problems).toEqual([]);
    }, 240000);

    it(`every workflow page on both examples (${lng})`, async () => {
      await i18next.changeLanguage(lng);
      const problems: string[] = [];
      try {
        for (const example of ['terraced_dwelling', 'small_office'] as const) {
          for (const page of PAGES) {
            renderWithProviders(<Page example={example} step={page.step} sub={page.sub} />);
            await screen.findAllByRole('navigation');
            problems.push(...unfilled(document.body).map((line) => `${example} ${page.step}/${page.sub ?? ''}: ${line}`));
            cleanup();
          }
        }
      } finally {
        await i18next.changeLanguage('en');
      }
      expect(problems).toEqual([]);
    }, 240000);

    it(`the Handleiding window, every chapter (${lng})`, async () => {
      await i18next.changeLanguage(lng);
      const problems: string[] = [];
      try {
        for (const chapter of MANUAL_CHAPTERS) {
          renderWithProviders(<ManualView chapterRef={chapter.id} onOpen={vi.fn()} />);
          problems.push(...unfilled(document.body).map((line) => `${chapter.id}: ${line}`));
          cleanup();
        }
      } finally {
        await i18next.changeLanguage('en');
      }
      expect(problems).toEqual([]);
    }, SLOW_UI_TEST_MS);
  }

  it('the reports on both examples', () => {
    const at = new Date('2026-10-10T10:00:00Z');
    const problems: string[] = [];
    for (const name of ['terraced-dwelling', 'office']) {
      const { project, assessment } = reportInput(name);
      for (const level of ['summary', 'standard', 'detailed'] as const) {
        problems.push(...htmlProblems(generateEnergyPerformanceReportHTML(project, assessment, { level, generatedAt: at, interpretations: [] })).map((line) => `${name} ${level}: ${line}`));
      }
      problems.push(...htmlProblems(generateNtaCalculationReportHTML(project, assessment, [], at)).map((line) => `${name} calculation: ${line}`));
      problems.push(...htmlProblems(generateNtaInputDossierHTML(project, at)).map((line) => `${name} input dossier: ${line}`));
    }
    expect(problems).toEqual([]);
  });
});
