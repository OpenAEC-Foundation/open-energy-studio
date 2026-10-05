/**
 * F10 accessibility audit (axe-core is not a dependency, so these are the
 * targeted checks): every workflow step page and sub page, in the dark and
 * the light theme, renders with
 *  - an accessible name on every button, link and form control,
 *  - unique ids and aria references that resolve,
 *  - a single h1 and no skipped heading levels below it,
 *  - no positive tabindex and no focusable element inside aria-hidden.
 * The dark run uses the terraced dwelling example, the light run the small office.
 */
import { useEffect, useState } from 'react';
import { describe, expect, it } from 'vitest';
import { waitFor } from '@testing-library/react';
import { renderWithProviders } from './test-utils';
import { useEnergy } from '../context/EnergyContext';
import { WorkflowNav } from '../components/shell/WorkflowNav';
import { StepRouter } from '../components/shell/StepRouter';
import type { ShellActions } from '../components/shell/ShellActions';
import { stepStatuses } from '../core/nta/stepStatus';
import { TOOL_STEP, WORKFLOW_STEPS, type Route } from '../core/navigation/routes';
import { exampleProject, type ExampleKind } from '../core/nta/ExampleProjects';

// The accessible-name algorithm Testing Library uses for *ByRole (a transitive dependency; its
// package "exports" hide the typings from this tsconfig, hence the typed dynamic import).
const A11Y_API = 'dom-accessibility-api';
const { computeAccessibleName } = await import(/* @vite-ignore */ A11Y_API) as { computeAccessibleName: (element: Element) => string };

const noop = () => undefined;
const actions: ShellActions = {
  newProject: noop, openProject: noop, saveProject: noop, saveAsProject: noop, calculate: noop,
  openDialog: noop, navigate: noop, exportReport: noop, printReport: noop, exportIFC: noop,
  exportModelIFC: noop, exportUNIEC3: noop, importUNIEC3: noop, exportVABI: noop, importVABI: noop,
  openSettings: noop, openFeedback: noop, openPalette: noop, toggleInspector: noop, togglePreview: noop,
};

/** The shell on an example project, so lists, tables and item pages have content. */
function Shell({ route, example }: { route: Route; example: ExampleKind }) {
  const { state, dispatch } = useEnergy();
  const [loaded, setLoaded] = useState(false);
  useEffect(() => { dispatch({ type: 'SET_PROJECT', payload: exampleProject(example) }); setLoaded(true); }, [dispatch, example]);
  const statuses = stepStatuses(state.project, null);
  if (!loaded) return null;
  return <>
    <WorkflowNav project={state.project} route={route} statuses={statuses} actions={actions} />
    <main><StepRouter project={state.project} route={route} statuses={statuses} actions={actions} /></main>
  </>;
}

const ROUTES: Route[] = [...WORKFLOW_STEPS, TOOL_STEP].flatMap((step) =>
  step.subs.length === 0 ? [{ step: step.id }] : step.subs.map((sub) => ({ step: step.id, sub: sub.id })));

const FOCUSABLE = 'a[href], button:not([disabled]), input:not([disabled]):not([type="hidden"]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

function audit(root: HTMLElement): string[] {
  const problems: string[] = [];
  const describe = (el: Element) => `<${el.tagName.toLowerCase()} class="${el.getAttribute('class') ?? ''}">`;

  for (const el of root.querySelectorAll('button, a[href], input:not([type="hidden"]), select, textarea, [role="button"], [role="tab"], [role="checkbox"], [role="switch"], [role="combobox"], [role="slider"]')) {
    if (el.closest('[aria-hidden="true"]')) continue;
    if (!computeAccessibleName(el).trim()) problems.push(`no accessible name: ${describe(el)}`);
  }

  const ids = new Map<string, number>();
  for (const el of root.querySelectorAll('[id]')) ids.set(el.id, (ids.get(el.id) ?? 0) + 1);
  for (const [id, count] of ids) if (count > 1) problems.push(`duplicate id: ${id}`);
  for (const attr of ['aria-labelledby', 'aria-describedby', 'aria-controls']) {
    for (const el of root.querySelectorAll(`[${attr}]`)) {
      for (const ref of (el.getAttribute(attr) ?? '').split(/\s+/).filter(Boolean)) {
        if (!document.getElementById(ref)) problems.push(`${attr} → missing #${ref} on ${describe(el)}`);
      }
    }
  }

  const headings = Array.from(root.querySelectorAll('main h1, main h2, main h3, main h4, main h5, main h6'))
    .filter((h) => !h.closest('[aria-hidden="true"]'));
  const h1 = headings.filter((h) => h.tagName === 'H1');
  if (h1.length !== 1) problems.push(`expected one h1 in main, found ${h1.length}`);
  let previous = 0;
  for (const h of headings) {
    const level = Number(h.tagName[1]);
    if (previous && level > previous + 1) problems.push(`heading jumps h${previous} → h${level}: "${h.textContent?.trim().slice(0, 40)}"`);
    previous = level;
  }

  for (const el of root.querySelectorAll('[tabindex]')) {
    if (Number(el.getAttribute('tabindex')) > 0) problems.push(`positive tabindex: ${describe(el)}`);
  }
  for (const hidden of root.querySelectorAll('[aria-hidden="true"]')) {
    if (hidden.matches(FOCUSABLE) || hidden.querySelector(FOCUSABLE)) problems.push(`focusable inside aria-hidden: ${describe(hidden)}`);
  }
  return problems;
}

describe.each([['dark', 'terraced_dwelling'], ['light', 'small_office']] as const)('accessibility of the step pages (%s theme, %s)', (theme, example) => {
  it.each(ROUTES.map((route) => [route.sub ? `${route.step}/${route.sub}` : route.step, route] as const))('%s', async (_name, route) => {
    document.documentElement.dataset.theme = theme;
    const { container, findByRole, unmount } = renderWithProviders(<Shell route={route} example={example} />);
    await findByRole('heading', { level: 1 });
    // Lazily loaded pages (3D model, report builder, tools) replace their skeleton.
    await waitFor(() => expect(container.querySelector('.lazy-page-loading')).toBeNull(), { timeout: 10000 });
    expect(audit(container)).toEqual([]);
    unmount();
  }, 30000);
});
