import { useState } from 'react';
import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { NtaVentilationSection } from '../components/NtaPerformancePanel/NtaVentilationSection';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { renderWithProviders, userEvent } from './test-utils';

function Harness({ initial }: { initial: Draft }) {
  const [draft, setDraft] = useState<Draft>(initial);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  return <form aria-label="test">
    <NtaVentilationSection draft={draft} change={change} />
    <output data-testid="draft">{JSON.stringify(draft)}</output>
  </form>;
}

const current = () => JSON.parse(screen.getByTestId('draft').textContent ?? '{}');

describe('NTA ventilation section', () => {
  it('records evidence for a favourable flow reduction (11.60/11.61)', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness initial={{ calculationScope: 'utility', ventilation: {} }} />);
    await user.type(screen.getByLabelText('Flow control down to x %, % (multiple of 10)'), '50');
    await user.type(
      screen.getByLabelText('Evidence for recirculation above 20 % or flow control below 80 % (11.60/11.61)'),
      'design note V-01',
    );
    expect(current().ventilation.flowReduction).toMatchObject({
      flowControlPercent: 50, evidenceReference: 'design note V-01',
    });
  }, 60_000);
});
