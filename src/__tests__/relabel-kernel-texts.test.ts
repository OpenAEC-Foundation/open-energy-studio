import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { isProductionPath, relabelCluster, relabelNote } from '../core/nta/RelabelText';
import { en } from '../i18n/en';
import { nl } from '../i18n/nl';

// The relabel classification in the kernel names clusters and notes in fixed English
// sentences; every one of them must have an NL/EN text in the app.
const source = readFileSync(resolve(__dirname, '../../crates/nta8800-core/src/relabel.rs'), 'utf8');
const production = source.slice(0, source.indexOf('#[cfg(test)]'));

function literals(patterns: RegExp[]): string[] {
  const found = new Set<string>();
  for (const pattern of patterns) {
    for (const match of production.matchAll(pattern)) found.add(match[1]);
  }
  return [...found];
}

const clusters = literals([
  /change\(\s*RelabelVerdict::\w+,\s*"([^"]+)"/g,
  /cluster:\s*"([^"]+)"/g,
  /change\.cluster\s*=\s*"([^"]+)"/g,
  /const (?:CLUSTER|HOT_WATER_SWAP_CLUSTER): &str =\s*"([^"]+)"/g,
]);
// Notes are sentences; `Some("ntaCalculation")` in a path match is not one.
const notes = literals([/Some\("([^"]+)"\)/g]).filter((text) => /\s/.test(text));

const keyOf = (key: string) => key;

describe('relabel texts from the kernel', () => {
  it('finds the cluster and note texts in relabel.rs', () => {
    expect(clusters.length).toBeGreaterThan(15);
    expect(notes.length).toBeGreaterThan(5);
  });

  it.each(clusters)('cluster "%s" has an NL and EN text', (cluster) => {
    const key = relabelCluster(keyOf, cluster);
    expect(key).toMatch(/^relabel\.cluster\./);
    expect(nl[key]).toBeTruthy();
    expect(en[key]).toBeTruthy();
  });

  it.each(notes)('note "%s" has an NL and EN text', (note) => {
    const key = relabelNote(keyOf, note);
    expect(key).toMatch(/^relabel\.note\./);
    expect(nl[key]).toBeTruthy();
    expect(en[key]).toBeTruthy();
  });

  it('does not take window solar properties for production', () => {
    expect(isProductionPath('/ntaCalculation/windowSolar/frameFraction')).toBe(false);
    expect(isProductionPath('/ntaCalculation/pvSystems/0/tiltDeg')).toBe(true);
    expect(isProductionPath('/ntaCalculation/spaceHeatingSolar/0')).toBe(true);
  });
});
