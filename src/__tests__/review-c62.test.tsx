import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { useState } from 'react';
import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { IProject } from '../core/energy/types';
import {
  deleteSurfaceFromProject, deleteWindowFromProject, deleteZoneFromProject, manualMeasuresShiftedBy,
} from '../core/energy/projectDelete';
import { deleteTarget } from '../core/energy/projectItems';
import { buildTemplatePatch } from '../core/nta/MwaTemplates';
import { checkDossierCompleteness } from '../core/report/ProjectDossier';
import { DialogShell } from '../components/dialogs/DialogShell';

type Block = Record<string, unknown>;

function load(name: string): IProject {
  return JSON.parse(readFileSync(resolve(process.cwd(), `training-data/${name}`), 'utf8')) as IProject;
}

/** The terraced example with a second zone and NTA input that refers to zones, surfaces and windows by id. */
function referencedProject(): IProject {
  const project = load('nta8800-example-terraced-dwelling.json');
  const zone = project.zones[0];
  project.zones.push({ ...zone, id: 'z2', name: 'Aanbouw', surfaces: [] });
  const nta = project.ntaCalculation as unknown as Block;
  nta.dynamicWindows = [{ windowId: 'win-S', dynamic: { method: 'single_state' } }];
  nta.zoneData = [{ zoneId: 'z1' }, { zoneId: 'z2' }];
  nta.additionalHeatingSystems = [{ zoneIds: ['z2'], generator: { kind: 'x' } }, { zoneIds: ['z1', 'z2'], generator: { kind: 'y' } }];
  nta.lighting = [{ zoneId: 'z2', functions: [] }];
  return project;
}

/** Every `zoneId`/`surfaceId`/`windowId`/`zoneIds` value in the NTA block. */
function references(value: unknown, found: string[] = []): string[] {
  if (Array.isArray(value)) value.forEach((item) => references(item, found));
  else if (value && typeof value === 'object') {
    for (const [key, child] of Object.entries(value)) {
      if ((key === 'zoneId' || key === 'surfaceId' || key === 'windowId') && typeof child === 'string') found.push(child);
      else if (key === 'zoneIds' && Array.isArray(child)) found.push(...child.map(String));
      else references(child, found);
    }
  }
  return found;
}

describe('deleting geometry cascades into the NTA input', () => {
  it('removes the dynamic window of a deleted window', () => {
    const { project, cascade } = deleteWindowFromProject(referencedProject(), 'z1', 'wall-S', 'win-S');
    expect(references(project.ntaCalculation)).not.toContain('win-S');
    expect(cascade).toEqual([{ path: 'ntaCalculation.dynamicWindows[0]', id: 'win-S' }]);
  });

  it('removes the ground-floor data, tilt and windows of a deleted surface', () => {
    const before = referencedProject();
    const nta = before.ntaCalculation as unknown as Block;
    expect((nta.groundFloors as Block[]).some((item) => item.surfaceId === 'floor')).toBe(true);
    const floor = deleteSurfaceFromProject(before, 'z1', 'floor');
    expect(references(floor.project.ntaCalculation)).not.toContain('floor');
    const wall = deleteSurfaceFromProject(before, 'z1', 'wall-S');
    expect(references(wall.project.ntaCalculation)).not.toContain('win-S');
  });

  it('drops zone data and systems that only served a deleted zone, and keeps shared ones', () => {
    const { project, cascade } = deleteZoneFromProject(referencedProject(), 'z2');
    const nta = project.ntaCalculation as unknown as Block;
    expect(references(nta)).not.toContain('z2');
    expect(nta.zoneData).toEqual([{ zoneId: 'z1' }]);
    expect(nta.additionalHeatingSystems).toEqual([{ zoneIds: ['z1'], generator: { kind: 'y' } }]);
    expect(nta.lighting).toEqual([]);
    expect(cascade.map((entry) => entry.path).sort()).toEqual([
      'ntaCalculation.additionalHeatingSystems[0]',
      'ntaCalculation.additionalHeatingSystems[1].zoneIds',
      'ntaCalculation.lighting[0]',
      'ntaCalculation.zoneData[1]',
    ]);
    expect(cascade).toHaveLength(4);
  });

  it('lists the cascade in the delete target', () => {
    const target = deleteTarget(referencedProject(), 'window', 'win-S');
    expect(target).toMatchObject({ kind: 'action', cascade: [{ path: 'ntaCalculation.dynamicWindows[0]' }] });
  });

  it('leaves an NTA block without references untouched', () => {
    const project = load('nta8800-example-terraced-dwelling.json');
    const { project: after } = deleteWindowFromProject(project, 'z1', 'wall-N', 'win-N');
    expect(after.ntaCalculation).toBe(project.ntaCalculation);
  });
});

describe('manual maatwerkadvies measures and deletes', () => {
  const withManual = (path: string): IProject => {
    const project = load('nta8800-example-terraced-dwelling.json');
    project.maatwerkadvies = {
      measures: [{ id: 'm1', name: 'Dak handmatig', category: 'insulation', target: 'project', investmentEur: 1,
        costSource: 'x', lifetimeYears: 30, patch: [{ op: 'replace', path, value: 'c2' }] }],
      packages: [], tariffs: {} as never,
    };
    return project;
  };

  it('blocks a delete that renumbers a manual measure path', () => {
    // Deleting surface 1 (wall-S) moves `roof` from index 2 to 1.
    const project = withManual('/zones/0/surfaces/2/constructionId');
    expect(deleteTarget(project, 'surface', 'wall-S')).toEqual({ kind: 'blocked', reason: 'manualMeasures', usedBy: ['Dak handmatig'] });
    // A delete after the addressed index changes nothing for it.
    expect(deleteTarget(project, 'surface', 'floor')).toMatchObject({ kind: 'action' });
  });

  it('blocks deleting the element the measure addresses', () => {
    const project = withManual('/zones/0/surfaces/1/windows/0/uValue');
    expect(deleteTarget(project, 'window', 'win-S')).toMatchObject({ kind: 'blocked', reason: 'manualMeasures' });
  });

  it('ignores template measures and building-target patches', () => {
    const project = withManual('/zones/0/surfaces/2/constructionId');
    const after = deleteSurfaceFromProject(project, 'z1', 'wall-S').project;
    const measures = project.maatwerkadvies!.measures;
    expect(manualMeasuresShiftedBy(project, after, [{ ...measures[0], target: 'building' }])).toEqual([]);
    expect(manualMeasuresShiftedBy(project, after, [{ ...measures[0], template: { kind: 'airtightness', qv10DmPerSM2: 0.4, sourceReference: 'x' } }])).toEqual([]);
  });
});

describe('hot-water template storage over all generators (§13.6.2, kernel rule)', () => {
  const tested = { kind: 'heat_pump_en16147', profile: 'l', deliveredKwhPerDay: 11.655, inputKwhPerDay: 3.2,
    exhaustAirSource: false, sourceReference: 'x' };
  const office = (storage: Block[], additional: Block[] = []): IProject => {
    const project = load('nta8800-example-office.json');
    const hotWater = (project.ntaCalculation as unknown as Block).hotWater as Block;
    hotWater.storage = storage;
    if (additional.length) hotWater.additionalGenerators = additional;
    return project;
  };
  const storagePatch = (project: IProject, generator: Block) =>
    buildTemplatePatch(project, { kind: 'hot_water', generator }, 'm1').patch.find((op) => op.path === '/ntaCalculation/hotWater/storage');

  it('keeps only the vessels outside the appliance test when they are mixed', () => {
    const outside = { id: 'a', notInApplianceTest: true };
    const inside = { id: 'b' };
    expect(storagePatch(office([outside, inside]), tested)).toEqual({ op: 'replace', path: '/ntaCalculation/hotWater/storage', value: [outside] });
    expect(storagePatch(office([outside]), tested)).toBeUndefined();
  });

  it('decides on the additional generators too', () => {
    const forfait = { kind: 'heat_pump', exhaustAirSource: false, measuredClass: 'class4' };
    const vessel = { id: 'a', notInApplianceTest: true };
    // An external-heat additional generator allows the vessel.
    expect(storagePatch(office([{ id: 'v' }], [{ generator: { kind: 'external_heat' } }]), forfait)).toBeUndefined();
    // A tested additional generator keeps vessels outside its test.
    expect(storagePatch(office([vessel, { id: 'v' }], [{ generator: tested }]), forfait))
      .toEqual({ op: 'replace', path: '/ntaCalculation/hotWater/storage', value: [vessel] });
    // An electric boiler as additional generator keeps everything.
    expect(storagePatch(office([{ id: 'v' }], [{ generator: { kind: 'electric_boiler' } }]), forfait)).toBeUndefined();
    // Without such a generator the vessel is in the efficiency.
    expect(storagePatch(office([{ id: 'v' }]), forfait)).toEqual({ op: 'replace', path: '/ntaCalculation/hotWater/storage', value: [] });
  });
});

describe('dialog stack keyboard handling', () => {
  function Nested() {
    const [outer, setOuter] = useState(true);
    const [inner, setInner] = useState(false);
    const [field, setField] = useState(true);
    return <>
      <button type="button">Outside</button>
      {outer && <DialogShell title="Outer" onClose={() => setOuter(false)}>
        {field && <input aria-label="Disappearing" onChange={() => setField(false)} />}
        <button type="button" onClick={() => setInner(true)}>Open inner</button>
      </DialogShell>}
      {inner && <DialogShell title="Inner" onClose={() => setInner(false)}>
        <input aria-label="Inner field" />
      </DialogShell>}
    </>;
  }

  it('keeps Tab inside after the focused field unmounts, and Escape closes only the topmost dialog', async () => {
    const user = userEvent.setup();
    render(<Nested />);
    const outer = screen.getByRole('dialog', { name: 'Outer' });
    expect(screen.getByLabelText('Disappearing')).toHaveFocus();
    await user.keyboard('x');
    // The focused input unmounted: focus is on <body>, Tab still lands in the dialog.
    expect(outer.contains(document.activeElement)).toBe(false);
    await user.tab();
    expect(outer.contains(document.activeElement)).toBe(true);
    await user.click(screen.getByRole('button', { name: 'Open inner' }));
    const inner = screen.getByRole('dialog', { name: 'Inner' });
    expect(screen.getByLabelText('Inner field')).toHaveFocus();
    for (let i = 0; i < 6; i += 1) {
      await user.tab();
      expect(inner.contains(document.activeElement)).toBe(true);
    }
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('dialog', { name: 'Inner' })).toBeNull();
    expect(screen.getByRole('dialog', { name: 'Outer' })).toBeInTheDocument();
    expect(outer.contains(document.activeElement)).toBe(true);
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('dialog')).toBeNull();
  }, 60000);
});

describe('dossier checklist while the kernel runs', () => {
  it('shows kernel-dependent items as pending instead of missing', () => {
    const project = load('nta8800-example-terraced-dwelling.json');
    const pending = checkDossierCompleteness({ project, assessment: null, pending: true });
    const status = (id: string, items = pending) => items.find((item) => item.id === id)?.status;
    expect(status('delivered_report')).toBe('pending');
    expect(status('output_file')).toBe('pending');
    const done = checkDossierCompleteness({ project, assessment: null });
    expect(status('delivered_report', done)).toBe('missing');
    // Items that do not depend on the kernel keep their own status.
    expect(status('floor_plan')).toBe(status('floor_plan', done));
  });

  it('evaluates the collapse reasons from the survey assessment', () => {
    const project = load('nta8800-example-terraced-dwelling.json');
    project.registration = { surveyType: 'basic' };
    const opname = { appliedDefaults: [{ rule: 'r', path: 'p', value: 'v', source: 's' }] } as never;
    const items = checkDossierCompleteness({ project, assessment: null, opname });
    expect(items.find((item) => item.id === 'collapse_reasons')).toMatchObject({ status: 'missing' });
  });
});
