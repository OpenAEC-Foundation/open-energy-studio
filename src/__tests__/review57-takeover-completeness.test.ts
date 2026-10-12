import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import type { BuildingPerformanceInput } from '../core/nta/KernelClient';
import { syncVentilation } from '../core/nta/NtaFormModels';
import { NOT_TAKEN_OVER, surveyTakeover } from '../core/nta/SurveyTakeover';
import type { IProject } from '../core/energy/types';
import officeJson from '../../training-data/nta8800-example-office.json';
import dwellingJson from '../../training-data/nta8800-example-terraced-dwelling.json';

const office = officeJson as unknown as IProject;
const dwelling = dwellingJson as unknown as IProject;

// Review 57: "Overnemen in projectmodel" is complete for every example survey.
// The kernel's derived input of each training-data/nta8800-opname-*.json survey
// comes from training-data/nta8800-survey-takeover-derived.json (written by
// crates/nta8800-core/tests/survey_takeover_completeness.rs). Every member a
// survey derives must be taken over or listed in NOT_TAKEN_OVER, and what the
// takeover writes is kept in training-data/nta8800-survey-takeover-written.json,
// which the Rust test runs through the project route against the survey route.
// OES_WRITE_TAKEOVER_FIXTURE=1 rewrites that file after a deliberate change.

type Loose = Record<string, unknown>;
const data = (name: string) => resolve(process.cwd(), 'training-data', name);
const derived = (JSON.parse(readFileSync(data('nta8800-survey-takeover-derived.json'), 'utf8')) as { derived: Record<string, Loose> }).derived;
const WRITTEN = data('nta8800-survey-takeover-written.json');

/** Where each derived member lands in the project draft (`ntaCalculation`). */
function target(path: string): string {
  if (path.startsWith('spaceHeating.demand.')) return path.slice('spaceHeating.demand.'.length);
  if (path.startsWith('spaceHeating.')) return path.slice('spaceHeating.'.length);
  return path;
}

/** The derived members at the level the takeover works on. */
function members(input: Loose): string[] {
  const heating = (input.spaceHeating ?? {}) as Loose;
  const demand = (heating.demand ?? {}) as Loose;
  return [
    ...Object.keys(input).filter((key) => key !== 'spaceHeating'),
    ...Object.keys(heating).filter((key) => key !== 'demand').map((key) => `spaceHeating.${key}`),
    ...Object.keys(demand).map((key) => `spaceHeating.demand.${key}`),
  ];
}

const read = (input: Loose, path: string): unknown =>
  path.split('.').reduce<unknown>((value, key) => (value as Loose | undefined)?.[key], input);

describe('survey takeover completeness (review 57)', () => {
  const names = Object.keys(derived).sort();

  it('covers every example survey', () => {
    expect(names.length).toBeGreaterThanOrEqual(6);
    expect(names.some((name) => name.includes('utility'))).toBe(true);
  });

  it.each(names)('%s: every derived member is taken over or listed as left to the project', (name) => {
    const input = derived[name];
    const { next, keys } = surveyTakeover(input as unknown as BuildingPerformanceInput, {});
    for (const path of members(input)) {
      const value = read(input, path);
      if (path in NOT_TAKEN_OVER) continue;
      // Building facts without a value (a dwelling has no label function) leave the project's alone.
      if (value === null || (Array.isArray(value) && value.length === 0 && !keys.includes(target(path)))) continue;
      expect(keys, `${name}: ${path} is neither taken over nor listed in NOT_TAKEN_OVER`).toContain(target(path));
      expect(next[target(path)], `${name}: ${path}`).toEqual(value);
    }
  });

  it.each(names)('%s: the takeover carries the steam humidifier, the construction year and the label function', (name) => {
    const input = derived[name];
    const { next } = surveyTakeover(input as unknown as BuildingPerformanceInput, { labelFunction: 'office' });
    expect(next.constructionYear).toBe(input.constructionYear);
    const humidifiers = read(input, 'spaceHeating.humidifiers');
    if (humidifiers !== undefined) expect(next.humidifiers).toEqual(humidifiers);
    // A survey without a label function keeps the project's.
    expect(next.labelFunction).toBe(input.labelFunction ?? 'office');
  });

  it.each(names.filter((name) => name.includes('utility')))('%s: lighting and humidifiers follow the project zone and its area', (name) => {
    const input = derived[name];
    const demand = read(input, 'spaceHeating.demand') as Loose;
    const { next } = surveyTakeover(input as unknown as BuildingPerformanceInput, {}, { id: 'z1', floorArea: 400 });
    const lighting = next.lighting as Loose[];
    expect(lighting.length).toBeGreaterThan(0);
    for (const item of lighting) {
      expect(item.zoneId).toBe('z1');
      const sum = (list: unknown) => (list as Loose[]).reduce((total, entry) => total + (entry.areaM2 as number), 0);
      expect(sum(item.functions)).toBeCloseTo(400, 6);
      expect(sum(item.lightingZones)).toBeCloseTo(400, 6);
    }
    for (const humidifier of (next.humidifiers ?? []) as Loose[]) expect(humidifier.zoneId).toBe('z1');
    // Without a project zone the survey's own zone and areas stay.
    const plain = surveyTakeover(input as unknown as BuildingPerformanceInput, {});
    expect((plain.next.lighting as Loose[])[0].zoneId).toBe(demand.zoneId);
  });

  it('matches the committed takeover fixture the kernel test runs', () => {
    // The drafts as "Toepassen" hands them to the kernel: NtaDraftProvider.apply runs syncVentilation
    // against the project, which gives the chapter 11 ventilation the project zone's id and area.
    const written = Object.fromEntries(names.map((name) => {
      const project = name.includes('utility') ? office : dwelling;
      const input = derived[name] as unknown as BuildingPerformanceInput;
      const zone = { id: project.zones[0].id, floorArea: project.zones[0].floorArea ?? null };
      const empty = surveyTakeover(input, {}, zone);
      const prefilled = surveyTakeover(input, structuredClone(project.ntaCalculation) as unknown as Loose, zone);
      return [name, {
        keys: empty.keys,
        skipped: empty.skipped,
        empty: syncVentilation(empty.next, project),
        prefilled: syncVentilation(prefilled.next, project),
      }];
    }));
    const fixture = {
      description: 'What surveyTakeover writes from each derived survey input (training-data/nta8800-survey-takeover-derived.json) into an empty draft and into the example project\'s draft (terraced dwelling or office), as Toepassen applies it (syncVentilation). Generated by src/__tests__/review57-takeover-completeness.test.ts (OES_WRITE_TAKEOVER_FIXTURE=1); crates/nta8800-core/tests/survey_takeover_completeness.rs runs it through the project route.',
      written,
    };
    if (process.env.OES_WRITE_TAKEOVER_FIXTURE) writeFileSync(WRITTEN, `${JSON.stringify(fixture, null, 2)}\n`);
    expect(existsSync(WRITTEN)).toBe(true);
    expect(JSON.parse(readFileSync(WRITTEN, 'utf8'))).toEqual(fixture);
  });
});
