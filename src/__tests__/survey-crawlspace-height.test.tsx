import { useState, type ReactElement } from 'react';
import { describe, expect, it } from 'vitest';
import { fireEvent, screen } from '@testing-library/react';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { SurfaceExtrasFields } from '../components/BasisopnamePanel/SurveyBasics';
import { en } from '../i18n/en';
import { renderWithProviders } from './test-utils';

// H2 of ISSO 82.1 4e druk afb. 8.23 (p. 73): NTA 8800:2022 and 2020+A1 take the
// measured height h in (8.47) (2022 p. 236); from 2023 h is the fixed 0,125 m.
// The kernel side is crates/nta8800-core/tests/survey_editions.rs.

function Harness({ initial, children }: {
  initial: Draft;
  children: (draft: Draft, change: (path: Path, value: unknown) => void) => ReactElement;
}) {
  const [draft, setDraft] = useState<Draft>(initial);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  return <form aria-label="test">
    {children(draft, change)}
    <output data-testid="draft">{JSON.stringify(draft)}</output>
  </form>;
}

const draft = () => JSON.parse(screen.getByTestId('draft').textContent ?? '{}');
const t = (key: string) => (en as Record<string, string>)[key] ?? key;
const base = ['envelope', 'surfaces', 0];
const crawlFloor = (kind: string) => ({ envelope: { surfaces: [{ id: 'vloer', element: 'floor', boundary: { kind } }] } });
const label = /Height H2/;

function render(kind: string, edition: string | null) {
  return renderWithProviders(<Harness initial={crawlFloor(kind)}>
    {(d, change) => <SurfaceExtrasFields draft={d} change={change} t={t} base={base} edition={edition} />}</Harness>);
}

describe('crawlspace wall height H2 in the survey', () => {
  it('asks H2 above a crawlspace or unheated cellar in 2022 and 2020+A1 and writes it', () => {
    for (const edition of ['2022', '2020+A1']) {
      for (const kind of ['crawlspace', 'unheated_cellar']) {
        const { unmount } = render(kind, edition);
        fireEvent.change(screen.getByLabelText(label), { target: { value: '0,45' } });
        expect(draft().envelope.surfaces[0].floorHeightAboveGroundM).toBe(0.45);
        unmount();
      }
    }
  });

  it('does not ask H2 from 2023 on, or for a floor on ground', () => {
    for (const edition of ['2025+C1', '2024', '2023', null]) {
      const { unmount } = render('crawlspace', edition);
      expect(screen.queryByLabelText(label)).toBeNull();
      unmount();
    }
    const { unmount } = render('ground', '2022');
    expect(screen.queryByLabelText(label)).toBeNull();
    unmount();
  });
});
