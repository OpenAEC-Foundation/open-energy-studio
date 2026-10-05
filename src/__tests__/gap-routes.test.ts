/**
 * Kernel paths → workflow step (UI redesign F3, `GAP_ROUTES`, ontwerp §3.4)
 * and the item a path points at ("Ga naar" selects it).
 */
import { describe, expect, it } from 'vitest';
import { FALLBACK_ROUTE, GAP_ROUTES, hasExplicitRoute, routeForPath } from '../core/nta/gapRoutes';
import { normalizeRoute, routeFromHash, routeToHash, stepDefinition, WORKFLOW_STEPS, TOOL_STEP } from '../core/navigation/routes';
import { projectItems, selectionForPath } from '../core/navigation/projectPaths';
import { createDefaultProject } from '../context/EnergyContext';

describe('routeForPath', () => {
  it.each([
    ['zones[0].surfaces[1].windows[0].gValue', 'building', 'envelope'],
    ['/constructions/0/uValue', 'building', 'envelope'],
    ['unheatedSpaces[2].kind', 'building', 'unheated'],
    ['heatingSystems[0].efficiency', 'installations', 'systems'],
    ['ntaHeatPumps[1].declaration', 'installations', 'heatPumps'],
    ['ntaCalculation.dynamicWindows[0]', 'check', 'input'],
    ['ntaCalculation.someFutureBlock', 'check', 'input'],
    ['registration.client', 'registration', undefined],
    ['registration.relabelComparison.original', 'relabel', undefined],
    ['basisopname.zones[0].area', 'survey', undefined],
    ['maatwerkadvies.measures[3]', 'advice', undefined],
    ['derivedInput.zones[0].floorAreaM2', 'building', 'envelope'],
  ])('routes %s to %s › %s', (path, step, sub) => {
    const route = routeForPath(path);
    expect(route.step).toBe(step);
    expect(route.sub).toBe(sub);
    expect(route.focusPath).toBe(path);
  });

  it('lets the longest prefix win', () => {
    expect(routeForPath('registration.relabelComparison').step).toBe('relabel');
    expect(routeForPath('registration').step).toBe('registration');
  });

  it('falls back to the check overview, keeping the raw path', () => {
    expect(routeForPath('somethingElse.deep[0]')).toEqual({ ...FALLBACK_ROUTE, focusPath: 'somethingElse.deep[0]' });
    expect(routeForPath('')).toEqual(FALLBACK_ROUTE);
    expect(hasExplicitRoute('somethingElse')).toBe(false);
    expect(hasExplicitRoute('zones[0]')).toBe(true);
  });

  it('only routes to existing steps and sub pages', () => {
    for (const entry of GAP_ROUTES) {
      const definition = stepDefinition(entry.step);
      expect(definition.id).toBe(entry.step);
      if (entry.sub) expect(definition.subs.map((sub) => sub.id)).toContain(entry.sub);
    }
  });
});

describe('route hash', () => {
  it('round-trips every step and sub page', () => {
    for (const step of [...WORKFLOW_STEPS, TOOL_STEP]) {
      const subs = step.subs.length ? step.subs.map((sub) => sub.id) : [undefined];
      for (const sub of subs) {
        const route = normalizeRoute({ step: step.id, sub });
        expect(routeFromHash(routeToHash(route))).toEqual(route);
      }
    }
    expect(routeToHash({ step: 'installations', sub: 'heatPumps' })).toBe('#/installaties/heatPumps');
    expect(routeFromHash('#/onbekend')).toBeNull();
    expect(routeFromHash('')).toBeNull();
  });
});

describe('selectionForPath', () => {
  const project = createDefaultProject();

  it('selects the deepest item on the path', () => {
    const window = projectItems(project).find((item) => item.itemType === 'window')!;
    expect(selectionForPath(project, `${window.path}.gValue`)).toEqual({ id: window.id, itemType: 'window' });
    expect(selectionForPath(project, 'zones[0].floorArea')).toEqual({ id: project.zones[0].id, itemType: 'zone' });
    expect(selectionForPath(project, '/heatingSystems/0/efficiency')).toEqual({ id: project.heatingSystems[0].id, itemType: 'heatingSystem' });
  });

  it('selects nothing for paths outside the project items', () => {
    expect(selectionForPath(project, 'ntaCalculation.generator')).toBeNull();
    expect(selectionForPath(project, 'zones[99]')).toBeNull();
  });
});

describe('"Ga naar" focus', () => {
  it('focuses and flashes the deepest element that contains the path', async () => {
    const { focusPathIn } = await import('../components/shell/StepRouter');
    const container = document.createElement('div');
    container.innerHTML = '<section data-path="zones[0]"><table><tbody><tr data-path="zones[0].surfaces[1]"><td>Gevel</td></tr>'
      + '<tr data-path="zones[0].surfaces[1].windows[0]"><td>Raam</td></tr></tbody></table></section>';
    document.body.appendChild(container);
    const target = focusPathIn(container, 'zones[0].surfaces[1].windows[0].gValue');
    expect(target?.dataset.path).toBe('zones[0].surfaces[1].windows[0]');
    expect(document.activeElement).toBe(target);
    expect(target).toHaveClass('focus-flash');
    expect(focusPathIn(container, 'heatingSystems[0]')).toBeNull();
    container.remove();
  });
});
