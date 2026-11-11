import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { render } from '@testing-library/react';
import { initSync, run as runKernel } from '../kernel-wasm/nta8800.js';
import { EnergyProvider, createDefaultProject, documentManagerReducer, type DocumentManagerAction, type DocumentManagerState } from '../context/EnergyContext';
import { getDocumentStore, type DocumentStore } from '../context/documentStore';
import { createSiteAppTools, installSiteAppTools } from '../core/site/siteAppTools';
import type { IProject } from '../core/energy/types';

// The energy_* tools of the site assistant on the reducer of the app, and once inside EnergyProvider. The kernel is the
// committed WebAssembly module, answering the HTTP route the client uses under vitest.
const example = JSON.parse(readFileSync(join(__dirname, '..', '..', 'training-data', 'nta8800-example-terraced-dwelling.json'), 'utf8'));

function storeWith(project: IProject): DocumentStore & { actions: DocumentManagerAction[] } {
  let state: DocumentManagerState = documentManagerReducer({ documents: [], activeDocumentId: null },
    { type: 'DOC_OPEN', payload: { id: 'doc', project, filePath: null } });
  const actions: DocumentManagerAction[] = [];
  return { actions, getState: () => state, dispatch(action) { actions.push(action); state = documentManagerReducer(state, action); return state; } };
}

const project = () => storeWith(createDefaultProject());
const doc = (store: DocumentStore) => store.getState().documents[0].state;

beforeAll(() => {
  initSync({ module: readFileSync(join(__dirname, '..', 'kernel-wasm', 'nta8800_bg.wasm')) });
});
afterEach(() => { vi.unstubAllGlobals(); });

function kernelOverFetch() {
  vi.stubGlobal('fetch', vi.fn(async (url: string, init?: RequestInit) => {
    const outcome = JSON.parse(runKernel(url, String(init?.body ?? '{}'))) as { status: number; body: unknown };
    return new Response(JSON.stringify(outcome.body), { status: outcome.status, headers: { 'content-type': 'application/json' } });
  }));
}

describe('site app tools', () => {
  it('installs with prefixed definitions and refuses unknown tools and arguments', async () => {
    const target: Parameters<typeof installSiteAppTools>[0] = {};
    installSiteAppTools(target);
    const names = await target.__OPDS_APP__!.tools();
    expect(names.length).toBeGreaterThanOrEqual(6);
    expect(names.length).toBeLessThanOrEqual(12);
    expect(names.every((name) => name.startsWith('energy_'))).toBe(true);
    const store = project();
    const tools = createSiteAppTools({ store: () => store });
    expect((await tools.definitions()).every((def) => def.description && def.inputSchema)).toBe(true);
    await expect(tools.run('energy_nope')).rejects.toThrow('onbekende_tool');
    await expect(tools.run('energy_zone_wijzigen', { floorArea: 10 })).rejects.toThrow('zoneId ontbreekt');
    await expect(tools.run('energy_zone_wijzigen', { zoneId: 'zone-main', floorArea: -1 })).rejects.toThrow('kleiner dan 0');
    await expect(tools.run('energy_stand', { x: 1 })).rejects.toThrow('onbekend');
    await expect(createSiteAppTools({ store: () => null }).run('energy_stand')).rejects.toThrow('app_niet_klaar');
    const empty = storeWith(createDefaultProject());
    empty.dispatch({ type: 'DOC_CLOSE', payload: 'doc' });
    await expect(createSiteAppTools({ store: () => empty }).run('energy_zones')).rejects.toThrow('geen_document');
  });

  it('reads the project, zones, constructions and installations', async () => {
    const store = project();
    const tools = createSiteAppTools({ store: () => store });
    const stand = await tools.run('energy_stand') as { project: { name: string }; building: { floorArea: number; constructions: number; installations: Record<string, number> } };
    expect(stand.project.name).toContain('Goejanverwelledijk');
    expect(stand.building.floorArea).toBeCloseTo(133.06);
    expect(stand.building.installations).toMatchObject({ verwarming: 1, pv: 2, zonthermisch: 0 });
    const zones = await tools.run('energy_zones', { zoneId: 'zone-main' }) as { zones: Array<{ qv10: number; surfaces: Array<{ id: string; construction: string; windows: unknown[] }> }> };
    expect(zones.zones[0].qv10).toBe(0.98);
    expect(zones.zones[0].surfaces.find((s) => s.id === 'surf-wall-s')).toMatchObject({ construction: 'Gevel Rc=6.76' });
    await expect(tools.run('energy_zones', { zoneId: 'nope' })).rejects.toThrow('zone_onbekend');
    const constructions = await tools.run('energy_constructies') as { constructions: Array<{ id: string; surfaces: number }> };
    expect(constructions.constructions.find((c) => c.id === 'con-wall')!.surfaces).toBe(4);
    const installations = await tools.run('energy_installaties') as Record<string, Array<{ id: string }>>;
    expect(installations.pv.map((p) => p.id)).toEqual(['pv-west', 'pv-east']);
  });

  it('changes the input through the actions of the app', async () => {
    const store = project();
    const tools = createSiteAppTools({ store: () => store });
    await tools.run('energy_project_bijwerken', { name: 'Aanbouw', buildingFunction: 'office' });
    expect(doc(store).project).toMatchObject({ name: 'Aanbouw', buildingFunction: 'office', city: 'Gouda' });
    const zone = await tools.run('energy_zone_wijzigen', { zoneId: 'zone-main', floorArea: 140, qv10: 0.6 }) as { floorArea: number; qv10: number };
    expect(zone).toMatchObject({ floorArea: 140, qv10: 0.6 });
    await tools.run('energy_vlak_wijzigen', { zoneId: 'zone-main', surfaceId: 'surf-wall-n', area: 30, constructionId: 'con-roof' });
    expect(doc(store).project.zones[0].surfaces[0]).toMatchObject({ area: 30, constructionId: 'con-roof' });
    await expect(tools.run('energy_vlak_wijzigen', { zoneId: 'zone-main', surfaceId: 'surf-wall-n', constructionId: 'x' })).rejects.toThrow('constructie_onbekend');
    await tools.run('energy_raam_wijzigen', { zoneId: 'zone-main', surfaceId: 'surf-wall-s', windowId: 'win-s1', uValue: 0.8, gValue: 0.5 });
    expect(doc(store).project.zones[0].surfaces[2].windows[0]).toMatchObject({ uValue: 0.8, gValue: 0.5 });
    // con-roof now lies first in a wall (surf-wall-n above): U = 1 / (0.13 + 8 + 0.04).
    const roof = await tools.run('energy_constructie_wijzigen', { constructionId: 'con-roof', rcValue: 8 }) as { uValue: number; surfaceType: string };
    expect(roof).toMatchObject({ surfaceType: 'wall', uValue: 0.122 });
    const floor = await tools.run('energy_constructie_wijzigen', { constructionId: 'con-floor', uValue: 0.2 }) as { rcValue: number };
    expect(floor.rcValue).toBeCloseTo(1 / 0.2 - 0.17 - 0.04, 2);
    await expect(tools.run('energy_constructie_wijzigen', { constructionId: 'con-floor', uValue: 9 })).rejects.toThrow('u_te_hoog');
    const changed = await tools.run('energy_installatie_wijzigen', { kind: 'verwarming', id: 'heat-1', fields: { cop: 4.2 } }) as { cop: number };
    expect(changed.cop).toBe(4.2);
    const added = await tools.run('energy_installatie_wijzigen', { kind: 'zonthermisch', fields: { collectorArea: 4 } }) as { added: boolean; tilt: number };
    expect(added).toMatchObject({ added: true, collectorArea: 4, tilt: 45, type: 'flat_plate' });
    expect(doc(store).project.solarThermal).toHaveLength(1);
    await expect(tools.run('energy_installatie_wijzigen', { kind: 'pv', fields: { cop: 3 } })).rejects.toThrow('hoort niet bij pv');
    await expect(tools.run('energy_installatie_wijzigen', { kind: 'ventilatie', id: 'vent-1', fields: { heatRecoveryEfficiency: 90 } })).rejects.toThrow('0 tot 1');
    expect(doc(store).isDirty).toBe(true);
    expect(store.actions.every((action) => action.type === 'DOC_DISPATCH')).toBe(true);
  });

  it('calculates with the NTA 8800 kernel and the indicative engine', async () => {
    kernelOverFetch();
    const store = storeWith((example.project ?? example) as IProject);
    const tools = createSiteAppTools({ store: () => store });
    const outcome = await tools.run('energy_bereken') as {
      nta8800: Record<string, unknown>;
      indicative: { beng1: number } | null;
    };
    expect(outcome.nta8800).toMatchObject({ status: 'calculated_unverified', energyLabel: 'A+', gaps: 0,
      beng1: { value: 51.37 }, beng2: { value: 80.56 }, beng3: { value: 22.3 }, toJuli: { max: 0.85 } });
    expect(outcome.indicative!.beng1).toBeGreaterThan(0);
    expect(doc(store).result).not.toBeNull();
    expect(doc(store).route.step).toBe('results');
    const again = await tools.run('energy_uitkomst') as { nta8800: { status: string } };
    expect(again.nta8800.status).toBe(outcome.nta8800.status);
  });

  it('reports a kernel that cannot run instead of failing', async () => {
    const store = project();
    const tools = createSiteAppTools({ store: () => store, kernel: () => Promise.reject(new Error('weg')) });
    const outcome = await tools.run('energy_uitkomst') as { nta8800: { status: string }; indicative: null };
    expect(outcome).toEqual({ nta8800: { status: 'kernel_niet_beschikbaar', error: 'weg' }, indicative: null });
  });

  it('works on the live document state inside EnergyProvider', async () => {
    render(<EnergyProvider><div /></EnergyProvider>);
    const store = getDocumentStore();
    expect(store).not.toBeNull();
    const tools = createSiteAppTools({ kernel: () => Promise.reject(new Error('niet nodig')) });
    const before = (await tools.run('energy_stand')) as { activeDocumentId: string };
    expect(before.activeDocumentId).toBeTruthy();
    const zone = (store!.getState().documents.find((d) => d.id === before.activeDocumentId)!.state.project.zones[0]);
    if (zone) {
      const out = await tools.run('energy_zone_wijzigen', { zoneId: zone.id, height: 3.1 }) as { height: number };
      expect(out.height).toBe(3.1);
    }
  });
});
