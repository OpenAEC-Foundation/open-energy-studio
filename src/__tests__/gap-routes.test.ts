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
    ['/constructions/0/uValue', 'building', 'constructions'],
    ['zones[0].thermalBridges[2].psiValue', 'building', 'thermalBridges'],
    ['zones[0].pointThermalBridges[0]', 'building', 'thermalBridges'],
    ['zones[1].airTightness.qv10', 'building', 'airTightness'],
    ['zones[0].floorArea', 'building', 'zones'],
    ['unheatedSpaces[2].kind', 'building', 'unheated'],
    ['heatingSystems[0].efficiency', 'installations', 'heating'],
    ['hotWaterSystems[0]', 'installations', 'hotWater'],
    ['ventilationSystems[0].sfp', 'installations', 'ventilation'],
    ['coolingSystems[0]', 'installations', 'cooling'],
    ['solarPV[0].peakPower', 'installations', 'generation'],
    ['solarThermal[0]', 'installations', 'generation'],
    ['ntaHeatPumps[1].declaration', 'installations', 'heatPumps'],
    ['ntaCalculation.dynamicWindows[0]', 'building', 'envelope'],
    ['ntaCalculation.groundFloors[0].exposedPerimeterM', 'building', 'envelope'],
    ['ntaCalculation.setpoints.heatingC', 'building', 'zones'],
    ['ntaCalculation.zoneData[1].setpoints', 'building', 'zones'],
    ['ntaCalculation.zoneData[1].verticalPipes', 'installations', 'heating'],
    ['ntaCalculation.sunrooms[0]', 'building', 'unheated'],
    ['ntaCalculation.generator.kind', 'installations', 'heating'],
    ['ntaCalculation.emission.fans.testedPowerW', 'installations', 'heating'],
    ['ntaCalculation.ventilation.infiltration', 'installations', 'ventilation'],
    ['ntaCalculation.hotWater.need.dwellingCount', 'installations', 'hotWater'],
    ['ntaCalculation.activeCooling.capacity.calculation.effectiveMassKgPerM2', 'installations', 'cooling'],
    ['ntaCalculation.humidifiers[0]', 'installations', 'humidification'],
    ['ntaCalculation.lighting.zones[0]', 'installations', 'lighting'],
    ['ntaCalculation.pvSystems[0].peakPower', 'installations', 'generation'],
    ['ntaCalculation.externalSupply.collectiveHeatPumpSource.realisedFrom2013', 'installations', 'generation'],
    ['ntaCalculation.bacsFactor', 'installations', 'bacs'],
    ['ntaCalculation.normVersion', 'project', undefined],
    ['ntaCalculation.useInventoryComplete', 'check', 'input'],
    ['ntaCalculation.someFutureBlock', 'check', 'input'],
    ['registration.client', 'registration', undefined],
    ['registration.relabelComparison.original', 'relabel', undefined],
    ['basisopname.zones[0].area', 'survey', 'zones'],
    ['basisopname.envelope.surfaces[1].insulation', 'survey', 'envelope'],
    ['basisopname.additionalHotWaterSystems[0]', 'survey', 'hotWater'],
    ['basisopname.id', 'survey', 'general'],
    ['maatwerkadvies.measures[3]', 'advice', 'measures'],
    ['maatwerkadvies.tariffs.gasEurPerM3', 'advice', 'use'],
    ['maatwerkadvies.renovationPassport.demandPackageId', 'advice', 'passport'],
    ['derivedInput.zones[0].floorAreaM2', 'building', 'zones'],
  ])('routes %s to %s › %s', (path, step, sub) => {
    const route = routeForPath(path);
    expect(route.step).toBe(step);
    expect(route.sub).toBe(sub);
    expect(route.focusPath).toBe(path);
  });

  it('routes NTA paths of the building assessment (without prefix) as NTA input paths', () => {
    expect(routeForPath('externalSupply.collectiveHeatPumpSource.realisedFrom2013')).toEqual({
      step: 'installations', sub: 'generation', focusPath: 'ntaCalculation.externalSupply.collectiveHeatPumpSource.realisedFrom2013',
    });
    expect(routeForPath('activeCooling.capacity.calculation.rooms[0].roofAreaM2').focusPath)
      .toBe('ntaCalculation.activeCooling.capacity.calculation.rooms[0].roofAreaM2');
    // Project-model lists keep their own route.
    expect(routeForPath('heatingSystems[0]').focusPath).toBe('heatingSystems[0]');
    expect(hasExplicitRoute('pvSystems[0]')).toBe(true);
  });

  it('sends every section of the NTA register to the page that renders it', async () => {
    const { NTA_SECTIONS } = await import('../components/NtaPerformancePanel/NtaSections');
    for (const section of NTA_SECTIONS) {
      for (const path of section.paths) {
        const route = routeForPath(`ntaCalculation.${path}`);
        expect([section.id, route.step, route.sub]).toEqual([section.id, section.step, section.sub]);
      }
    }
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

  it('opens a collapsed "Geavanceerd" block and matches the extra paths of a section', async () => {
    const { focusPathIn } = await import('../components/shell/StepRouter');
    const container = document.createElement('div');
    container.innerHTML = '<details><summary>Geavanceerd</summary><fieldset data-path="ntaCalculation.declaredUses" '
      + 'data-paths="ntaCalculation.onSiteProduction"><label data-path="ntaCalculation.declaredUses[0].monthlyKwh">x<input /></label>'
      + '</fieldset></details>';
    document.body.appendChild(container);
    const details = container.querySelector('details')!;
    expect(details.open).toBe(false);
    expect(focusPathIn(container, 'ntaCalculation.declaredUses[0].monthlyKwh')?.tagName).toBe('INPUT');
    expect(details.open).toBe(true);
    details.open = false;
    expect(focusPathIn(container, 'ntaCalculation.onSiteProduction.pv')?.tagName).toBe('FIELDSET');
    expect(details.open).toBe(true);
    container.remove();
  });
});
