import { useState } from 'react';
import { describe, expect, it } from 'vitest';
import { cleanup, fireEvent, screen } from '@testing-library/react';
import { FieldPathPrefixProvider, Section, WIDE_SELECT_CHARS, write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { NTA_SECTIONS, type NtaSectionDef } from '../components/NtaPerformancePanel/NtaSections';
import { BasisopnamePanel } from '../components/BasisopnamePanel/BasisopnamePanel';
import type { IProject } from '../core/energy/types';
import i18next from '../i18n/i18n';
import office from '../../training-data/nta8800-example-office.json';
import terraced from '../../training-data/nta8800-example-terraced-dwelling.json';
import { renderWithProviders } from './test-utils';

// A select whose longest option text does not fit one grid column (minimum
// 200 px) takes two, so the chosen text is not cut off. The edition select
// showed "NTA 8800:2025+C1:2026 (aangew…" in the Algemeen block. Selects in
// table cells keep their column.

const ADD = /^\+|\badd\b|toevoeg/i;

/** Like NtaStepPage: each section inside a Section. */
function Harness({ def, project }: { def: NtaSectionDef; project: IProject }) {
  const [draft, setDraft] = useState<Draft>(project.ntaCalculation as unknown as Draft);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  const update = (next: (current: Draft) => Draft) => setDraft((current) => next(current));
  const Component = def.Component;
  return <FieldPathPrefixProvider value="ntaCalculation">
    <form aria-label="widths" className="nta-form">
      <Section title={def.id}><Component draft={draft} change={change} update={update} project={project} /></Section>
    </form>
  </FieldPathPrefixProvider>;
}

function openLists(root: HTMLElement) {
  for (const button of Array.from(root.querySelectorAll('button')).filter((item) => ADD.test(item.textContent ?? ''))) {
    try { fireEvent.click(button); } catch { /* needs more context */ }
  }
}

/** Long selects outside a table whose label is not marked wide. */
function narrowLongSelects(root: HTMLElement): string[] {
  const problems: string[] = [];
  for (const select of Array.from(root.querySelectorAll('select'))) {
    if (select.closest('td, th')) continue;
    const longest = Math.max(0, ...Array.from(select.options).map((option) => (option.textContent ?? '').trim().length));
    if (longest <= WIDE_SELECT_CHARS) continue;
    const label = select.closest('label');
    const wide = label?.hasAttribute('data-wide-select') || label?.classList.contains('nta-form-wide');
    if (!wide) problems.push(`${label?.childNodes[0]?.textContent ?? select.getAttribute('aria-label') ?? '?'} (${longest})`);
  }
  return problems;
}

describe('selects with long option texts', () => {
  it('marks the edition select wide', async () => {
    await i18next.changeLanguage('nl');
    try {
      const general = NTA_SECTIONS.find((def) => def.id === 'general')!;
      renderWithProviders(<Harness def={general} project={office as unknown as IProject} />);
      const select = screen.getByLabelText('Uitgave NTA 8800') as HTMLSelectElement;
      expect(select.closest('label')!.hasAttribute('data-wide-select')).toBe(true);
      cleanup();
    } finally {
      await i18next.changeLanguage('en');
    }
  });

  it('leaves short selects in one column', async () => {
    const general = NTA_SECTIONS.find((def) => def.id === 'general')!;
    renderWithProviders(<Harness def={general} project={office as unknown as IProject} />);
    const short = Array.from(document.querySelectorAll('select')).filter((select) =>
      Math.max(...Array.from(select.options).map((option) => (option.textContent ?? '').trim().length)) <= WIDE_SELECT_CHARS);
    expect(short.length).toBeGreaterThan(0);
    for (const select of short) expect(select.closest('label')?.hasAttribute('data-wide-select')).toBe(false);
    cleanup();
  });

  for (const lng of ['nl', 'en']) {
    it(`gives every long select in the NTA sections two columns (${lng})`, async () => {
      await i18next.changeLanguage(lng);
      const problems: string[] = [];
      try {
        for (const [name, project] of [['office', office], ['terraced', terraced]] as const) {
          for (const def of NTA_SECTIONS) {
            if (def.id === 'bcrg') continue;
            renderWithProviders(<Harness def={def} project={project as unknown as IProject} />);
            const form = screen.getByRole('form', { name: 'widths' });
            openLists(form);
            problems.push(...narrowLongSelects(form).map((line) => `${name} / ${def.id}: ${line}`));
            cleanup();
          }
        }
      } finally {
        await i18next.changeLanguage('en');
      }
      expect([...new Set(problems)]).toEqual([]);
    }, 300000);
  }

  it('gives every long select in the survey two columns', () => {
    for (const start of ['Start dwelling survey', 'Start utility survey']) {
      renderWithProviders(<BasisopnamePanel />);
      fireEvent.click(screen.getByRole('button', { name: start }));
      openLists(document.body);
      expect(narrowLongSelects(document.body)).toEqual([]);
      cleanup();
    }
  }, 120000);
});
