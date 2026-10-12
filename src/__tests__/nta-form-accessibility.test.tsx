import { useState } from 'react';
import { describe, expect, it } from 'vitest';
import { cleanup, fireEvent, screen } from '@testing-library/react';
// @ts-expect-error the package ships no type declarations for this resolution mode
import { computeAccessibleName } from 'dom-accessibility-api';
import { NumberField, write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { NTA_SECTIONS, type NtaSectionDef } from '../components/NtaPerformancePanel/NtaSections';
import { BasisopnamePanel } from '../components/BasisopnamePanel/BasisopnamePanel';
import { applyIssueMarks } from '../components/shell/NtaStepPage';
import type { IProject } from '../core/energy/types';
import { en } from '../i18n/en';
import { nl } from '../i18n/nl';
import office from '../../training-data/nta8800-example-office.json';
import terraced from '../../training-data/nta8800-example-terraced-dwelling.json';
import { renderWithProviders } from './test-utils';

// Accessibility of the NTA forms and the survey (UI quality pass, 9 October 2026). Every NTA
// section is rendered on the example office and terraced dwelling, and both surveys are started;
// every "add" button is clicked once so the list items render. Then every control must have an
// accessible name, every fieldset a legend, every button a name, and no two controls in one group
// may share a name: a list item is a labelled group, so "Window area" of two surfaces is told apart.

const accessibleName = (element: Element): string => (computeAccessibleName(element) as string).trim();
const ADD = /^\+|\badd\b|toevoeg/i;

function Harness({ def, project }: { def: NtaSectionDef; project: IProject }) {
  const [draft, setDraft] = useState<Draft>(project.ntaCalculation as unknown as Draft);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  const update = (next: (current: Draft) => Draft) => setDraft((current) => next(current));
  const Component = def.Component;
  return <form aria-label="audit"><Component draft={draft} change={change} update={update} project={project} /></form>;
}

/** Problems in `root`, as `kind: detail` lines. */
function audit(root: HTMLElement): string[] {
  const out: string[] = [];
  const groups = Array.from(root.querySelectorAll('fieldset, [role=group]'));
  const seen = new Map<string, number>();
  for (const control of Array.from(root.querySelectorAll('input:not([type=hidden]), select, textarea'))) {
    const name = accessibleName(control);
    if (!name) { out.push(`no name: ${control.outerHTML.slice(0, 120)}`); continue; }
    const group = control.closest('fieldset, [role=group]');
    const key = `${group ? groups.indexOf(group) : -1}\u0000${name}`;
    seen.set(key, (seen.get(key) ?? 0) + 1);
  }
  for (const [key, count] of seen) if (count > 1) out.push(`duplicate name in one group: ${count}× ${key.split('\u0000')[1]}`);
  for (const fieldset of Array.from(root.querySelectorAll('fieldset'))) {
    if (!fieldset.querySelector(':scope > legend')) out.push(`fieldset without legend: ${fieldset.outerHTML.slice(0, 120)}`);
  }
  for (const button of Array.from(root.querySelectorAll('button'))) {
    if (!accessibleName(button)) out.push(`button without name: ${button.outerHTML.slice(0, 120)}`);
  }
  return out;
}

function openLists(root: HTMLElement) {
  for (const button of Array.from(root.querySelectorAll('button')).filter((item) => ADD.test(item.textContent ?? ''))) {
    try { fireEvent.click(button); } catch { /* a button that needs more context; the audit still runs */ }
  }
}

describe('NTA forms: accessible names, legends and groups', () => {
  for (const fixture of [{ name: 'office', project: office }, { name: 'terraced dwelling', project: terraced }]) {
    it(`every section on the ${fixture.name}`, () => {
      const problems: string[] = [];
      for (const def of NTA_SECTIONS) {
        if (def.id === 'bcrg') continue; // a calculation tool, not an input section
        renderWithProviders(<Harness def={def} project={fixture.project as unknown as IProject} />);
        const form = screen.getByRole('form', { name: 'audit' });
        openLists(form);
        problems.push(...audit(form).map((line) => `${def.id}: ${line}`));
        cleanup();
      }
      expect(problems).toEqual([]);
    }, 120000);
  }

  for (const start of ['Start dwelling survey', 'Start utility survey']) {
    it(`the survey after "${start}"`, () => {
      renderWithProviders(<BasisopnamePanel />);
      fireEvent.click(screen.getByRole('button', { name: start }));
      openLists(document.body);
      expect(audit(document.body)).toEqual([]);
    }, 120000);
  }
});

describe('units in the accessible name', () => {
  it('reads the unit after a separator, not glued to the label', () => {
    renderWithProviders(<NumberField draft={{ areaM2: 12.5 }} path={['areaM2']} label="Door area" unit="m²" onChange={() => {}} />);
    expect(screen.getByRole('spinbutton', { name: 'Door area, m²' })).toBeTruthy();
  });

  it('accepts a decimal comma', () => {
    let written: unknown;
    renderWithProviders(<NumberField draft={{}} path={['areaM2']} label="Area" unit="m²" onChange={(_, value) => { written = value; }} />);
    fireEvent.change(screen.getByRole('spinbutton', { name: 'Area, m²' }), { target: { value: '12,5' } });
    expect(written).toBe(12.5);
  });
});

describe('kernel findings on a field are announced, not only coloured', () => {
  const t = (key: string) => (en as Record<string, string>)[key] ?? key;
  function page() {
    const root = document.createElement('div');
    root.innerHTML = '<label data-path="ntaCalculation.setpoints.heatingC">Heating setpoint<input></label>'
      + '<label data-path="ntaCalculation.setpoints.coolingC">Cooling setpoint<input></label>';
    document.body.append(root);
    return root;
  }

  it('sets aria-invalid and an aria-description on the control of an error', () => {
    const root = page();
    applyIssueMarks(root, [{ kind: 'error', code: 'nta_value_missing', focus: 'ntaCalculation.setpoints.heatingC' }], t);
    const [heating, cooling] = Array.from(root.querySelectorAll('input'));
    expect(heating.getAttribute('aria-invalid')).toBe('true');
    expect(heating.getAttribute('aria-description')).toMatch(/^Error: /);
    expect(heating.closest('label')?.dataset.issue).toBe('error');
    expect(cooling.hasAttribute('aria-invalid')).toBe(false);
    root.remove();
  });

  it('describes a warning without marking the field invalid, and clears marks that no longer apply', () => {
    const root = page();
    applyIssueMarks(root, [{ kind: 'warning', code: 'nta_value_missing', focus: 'ntaCalculation.setpoints.coolingC' }], t);
    const cooling = root.querySelectorAll('input')[1];
    expect(cooling.hasAttribute('aria-invalid')).toBe(false);
    expect(cooling.getAttribute('aria-description')).toMatch(/^Warning: /);
    applyIssueMarks(root, [], t);
    expect(cooling.hasAttribute('aria-description')).toBe(false);
    expect(cooling.closest('label')?.hasAttribute('data-issue')).toBe(false);
    root.remove();
  });
});

describe('language of the label texts', () => {
  it('has no Dutch norm terms left in English values', () => {
    const dutch = /\bafb\.|\bBijlage \d|\btabel \d|\bformule \d/;
    expect(Object.entries(en).filter(([, value]) => dutch.test(value))).toEqual([]);
  });

  it('has no English norm terms left in Dutch values', () => {
    const english = /\bfig\. \d|\bannex \d|\btable \d|\bformula \d/i;
    expect(Object.entries(nl).filter(([, value]) => english.test(value))).toEqual([]);
  });
});
