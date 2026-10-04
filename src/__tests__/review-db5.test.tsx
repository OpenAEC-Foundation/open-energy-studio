import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { strFromU8 } from 'fflate';
import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { INtaHeatPumpInput, IProject } from '../core/energy/types';
import { deleteZoneFromProject } from '../core/energy/projectDelete';
import { deleteTarget } from '../core/energy/projectItems';
import { buildProjectDossier, checkDossierCompleteness, openDossierItems, type DossierItem } from '../core/report/ProjectDossier';
import { DialogShell } from '../components/dialogs/DialogShell';

function load(name: string): IProject {
  return JSON.parse(readFileSync(resolve(process.cwd(), `training-data/${name}`), 'utf8')) as IProject;
}

function pump(id: string, servedZoneIds: string[]): INtaHeatPumpInput {
  return { id, servedZoneIds } as INtaHeatPumpInput;
}

function withManualMeasure(project: IProject, path: string, target: 'project' | 'building' = 'project'): IProject {
  project.maatwerkadvies = {
    measures: [{ id: 'm1', name: 'Handmatig', category: 'insulation', target, investmentEur: 1,
      costSource: 'x', lifetimeYears: 30, patch: [{ op: 'replace', path, value: 1 }] }],
    packages: [], tariffs: {} as never,
  };
  return project;
}

describe('a zone delete also filters servedZoneIds outside the NTA block', () => {
  it('filters heating, hot-water and NTA heat pumps and lists them in the cascade', () => {
    const project = load('nta8800-example-terraced-dwelling.json');
    project.zones.push({ ...project.zones[0], id: 'z2', name: 'Aanbouw', surfaces: [] });
    project.heatingSystems[0].ntaHeatPump = pump('hp1', ['z1', 'z2']);
    project.hotWaterSystems = [{ id: 'w1', name: 'Boiler', type: 'heat_pump', efficiency: 2, hasSolarBoiler: false,
      solarBoilerFraction: 0, ntaHeatPump: pump('hp2', ['z2']) } as IProject['hotWaterSystems'][number]];
    project.ntaHeatPumps = [pump('hp3', ['z2', 'z1'])];

    const { project: after, cascade } = deleteZoneFromProject(project, 'z2');
    expect(after.heatingSystems[0].ntaHeatPump?.servedZoneIds).toEqual(['z1']);
    // An emptied list is kept: the kernel then only warns (`heat_pump_zones_missing`).
    expect(after.hotWaterSystems[0].ntaHeatPump?.servedZoneIds).toEqual([]);
    expect(after.ntaHeatPumps?.[0].servedZoneIds).toEqual(['z1']);
    expect(cascade.map((entry) => entry.path)).toEqual(expect.arrayContaining([
      'heatingSystems[0].ntaHeatPump.servedZoneIds[1]',
      'hotWaterSystems[0].ntaHeatPump.servedZoneIds[0]',
      'ntaHeatPumps[0].servedZoneIds[0]',
    ]));
    expect(deleteTarget(project, 'zone', 'z2')).toMatchObject({
      kind: 'action', cascade: expect.arrayContaining([{ path: 'ntaHeatPumps[0].servedZoneIds[0]', id: 'z2' }]),
    });
  });

  it('leaves the systems untouched when no heat pump served the zone', () => {
    const project = load('nta8800-example-terraced-dwelling.json');
    project.zones.push({ ...project.zones[0], id: 'z2', name: 'Aanbouw', surfaces: [] });
    const { project: after } = deleteZoneFromProject(project, 'z2');
    expect(after.heatingSystems).toBe(project.heatingSystems);
  });
});

describe('the manual-measure guard covers every delete', () => {
  it('blocks a thermal-bridge delete that renumbers a manual measure path', () => {
    const project = load('nta8800-example-terraced-dwelling.json');
    project.zones[0].thermalBridges = [
      { id: 'tb1', name: 'A', psiValue: 0.1, length: 5, zoneId: 'z1' },
      { id: 'tb2', name: 'B', psiValue: 0.2, length: 5, zoneId: 'z1' },
    ];
    withManualMeasure(project, '/zones/0/thermalBridges/1/psiValue');
    expect(deleteTarget(project, 'thermalBridge', 'tb1')).toEqual({ kind: 'blocked', reason: 'manualMeasures', usedBy: ['Handmatig'] });
    expect(deleteTarget(project, 'thermalBridge', 'tb2')).toMatchObject({ kind: 'blocked' });
  });

  it('blocks PV, construction and system deletes that renumber a path, and allows the others', () => {
    const pv = load('nta8800-example-terraced-dwelling.json');
    pv.solarPV = [
      { id: 'pv1', name: 'Oost', peakPower: 2, orientation: 'E', tilt: 30, area: 10 },
      { id: 'pv2', name: 'West', peakPower: 2, orientation: 'W', tilt: 30, area: 10 },
    ] as IProject['solarPV'];
    withManualMeasure(pv, '/solarPV/1/peakPower');
    expect(deleteTarget(pv, 'solarPV', 'pv1')).toMatchObject({ kind: 'blocked', reason: 'manualMeasures' });

    const construction = load('nta8800-example-terraced-dwelling.json');
    construction.constructions.push({ ...construction.constructions[0], id: 'c-spare', name: 'Reserve' });
    withManualMeasure(construction, '/constructions/2/name');
    // c-spare is unused and last: deleting it does not renumber index 2's predecessor path.
    expect(deleteTarget(construction, 'construction', 'c-spare')).toMatchObject({ kind: 'action' });

    const system = load('nta8800-example-terraced-dwelling.json');
    system.heatingSystems.push({ ...system.heatingSystems[0], id: 'h2', name: 'Tweede' });
    withManualMeasure(system, '/heatingSystems/1/cop');
    expect(deleteTarget(system, 'heatingSystem', 'h1')).toMatchObject({ kind: 'blocked', reason: 'manualMeasures' });
    expect(deleteTarget(system, 'heatingSystem', 'h2')).toMatchObject({ kind: 'blocked', reason: 'manualMeasures' });
  });
});

describe('manual building-target measures are named for review', () => {
  it('lists them in the delete target without blocking', () => {
    const project = load('nta8800-example-terraced-dwelling.json');
    project.heatingSystems.push({ ...project.heatingSystems[0], id: 'h2', name: 'Tweede' });
    withManualMeasure(project, '/spaceHeating/generator/efficiency', 'building');
    expect(deleteTarget(project, 'heatingSystem', 'h2')).toMatchObject({ kind: 'action', buildingMeasures: ['Handmatig'] });
    const noMeasures = load('nta8800-example-terraced-dwelling.json');
    noMeasures.heatingSystems.push({ ...noMeasures.heatingSystems[0], id: 'h2', name: 'Tweede' });
    expect(deleteTarget(noMeasures, 'heatingSystem', 'h2')).toMatchObject({ kind: 'action', buildingMeasures: [] });
  });
});

describe('dialog stack order follows DOM nesting', () => {
  it('gives focus and Escape to a nested dialog mounted with its parent', async () => {
    const user = userEvent.setup();
    let outerClosed = false;
    let innerClosed = false;
    render(<DialogShell title="Outer" onClose={() => { outerClosed = true; }}>
      <input aria-label="Outer field" />
      <DialogShell title="Inner" onClose={() => { innerClosed = true; }}>
        <input aria-label="Inner field" />
      </DialogShell>
    </DialogShell>);
    expect(screen.getByLabelText('Inner field')).toHaveFocus();
    const inner = screen.getByRole('dialog', { name: 'Inner' });
    await user.tab();
    expect(inner.contains(document.activeElement)).toBe(true);
    await user.keyboard('{Escape}');
    expect(innerClosed).toBe(true);
    expect(outerClosed).toBe(false);
  }, 60000);
});

describe('pending never reaches an exported dossier', () => {
  it('builds the checklist without pending even when the caller passes it', async () => {
    const project = load('nta8800-example-terraced-dwelling.json');
    const bundle = await buildProjectDossier({ project, assessment: null, pending: true, labelInputSha256: 'x' });
    const checklist = JSON.parse(strFromU8(bundle.files['dossier-checklist.json'])) as DossierItem[];
    expect(checklist.some((item) => item.status === 'pending')).toBe(false);
  });

  it('counts pending items as open in the summary', () => {
    const project = load('nta8800-example-terraced-dwelling.json');
    const pending = checkDossierCompleteness({ project, assessment: null, pending: true });
    const open = openDossierItems(pending);
    expect(open.some((item) => item.status === 'pending')).toBe(true);
    expect(open.every((item) => ['missing', 'check', 'pending'].includes(item.status))).toBe(true);
  });
});
