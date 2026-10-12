import { useState } from 'react';
import { describe, expect, it } from 'vitest';
import { cleanup, fireEvent, screen } from '@testing-library/react';
import { FieldPathPrefixProvider, write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { NTA_SECTIONS, type NtaSectionDef } from '../components/NtaPerformancePanel/NtaSections';
import { BasisopnamePanel } from '../components/BasisopnamePanel/BasisopnamePanel';
import type { IProject } from '../core/energy/types';
import i18next from '../i18n/i18n';
import office from '../../training-data/nta8800-example-office.json';
import terraced from '../../training-data/nta8800-example-terraced-dwelling.json';
import { renderWithProviders } from './test-utils';

// Every option of every select in the NTA sections and the survey shows a
// translated label, never its raw value. The manual screenshot of "Zonwering
// per raam" showed `dark`: keys such as `device.external_screen:dark` were
// read by i18next as namespace `…external_screen` and key `dark`.

const publicCases = import.meta.glob('../../training-data/nta8800-public-comparison-*.json', { eager: true, import: 'default' });
const ADD = /^\+|\badd\b|toevoeg/i;

function Harness({ def, project }: { def: NtaSectionDef; project: IProject }) {
  const [draft, setDraft] = useState<Draft>(project.ntaCalculation as unknown as Draft);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  const update = (next: (current: Draft) => Draft) => setDraft((current) => next(current));
  const Component = def.Component;
  return <FieldPathPrefixProvider value="ntaCalculation">
    <form aria-label="options"><Component draft={draft} change={change} update={update} project={project} /></form>
  </FieldPathPrefixProvider>;
}

function openLists(root: HTMLElement) {
  for (const button of Array.from(root.querySelectorAll('button')).filter((item) => ADD.test(item.textContent ?? ''))) {
    try { fireEvent.click(button); } catch { /* needs more context */ }
  }
}

/**
 * Options that show their raw value: an enum-like value (snake_case, a colon)
 * whose text is an untranslated key, empty, or the part after a colon. In
 * Dutch (`strict`) the text must also differ from the value itself; several
 * English labels are the value word itself ("office", "central").
 */
function rawOptions(root: HTMLElement, strict: boolean): string[] {
  const problems: string[] = [];
  for (const select of Array.from(root.querySelectorAll('select'))) {
    for (const option of Array.from(select.options)) {
      const value = option.value;
      const text = (option.textContent ?? '').trim();
      if (!value || !/^[a-z][a-z0-9_]*(:[a-z0-9_]+)?$/.test(value)) continue;
      const colonPart = value.includes(':') && text === value.split(':').pop();
      const untranslated = text === '' || /^(nta|ntaStep|opname|kernel)\./.test(text);
      if (colonPart || untranslated || (strict && text === value)) {
        problems.push(`${select.getAttribute('aria-label') ?? select.closest('label')?.textContent?.slice(0, 40) ?? '?'}: ${value} → "${text}"`);
      }
    }
  }
  return [...new Set(problems)];
}

const projects: Array<{ name: string; project: IProject }> = [
  { name: 'office', project: office as unknown as IProject },
  { name: 'terraced dwelling', project: terraced as unknown as IProject },
  ...Object.entries(publicCases).map(([path, value]) => ({ name: path.replace(/.*comparison-/, 'case '), project: value as IProject })),
];

describe('select options show translated labels', () => {
  it('reads a colon key as one key, not a namespace', () => {
    expect(i18next.t('nta.form.windowShadings.device.external_screen:dark')).not.toBe('dark');
    expect(i18next.t('nta.form.windowShadings.device.external_screen:dark')).toMatch(/screen|screen/i);
  });

  for (const lng of ['en', 'nl']) {
    it(`every NTA section on the examples and public cases (${lng})`, async () => {
      await i18next.changeLanguage(lng);
      const problems: string[] = [];
      try {
        for (const fixture of projects) {
          for (const def of NTA_SECTIONS) {
            if (def.id === 'bcrg') continue;
            renderWithProviders(<Harness def={def} project={fixture.project} />);
            const form = screen.getByRole('form', { name: 'options' });
            openLists(form);
            problems.push(...rawOptions(form, lng === 'nl').map((line) => `${fixture.name} / ${def.id}: ${line}`));
            cleanup();
          }
        }
      } finally {
        await i18next.changeLanguage('en');
      }
      expect([...new Set(problems)]).toEqual([]);
    }, 300000);
  }

  it('the survey forms', () => {
    for (const start of ['Start dwelling survey', 'Start utility survey']) {
      renderWithProviders(<BasisopnamePanel />);
      fireEvent.click(screen.getByRole('button', { name: start }));
      openLists(document.body);
      expect(rawOptions(document.body, false)).toEqual([]);
      cleanup();
    }
  }, 120000);
});
