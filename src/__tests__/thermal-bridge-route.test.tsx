import { useState } from 'react';
import { describe, expect, it } from 'vitest';
import { fireEvent, screen } from '@testing-library/react';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { NTA_SECTIONS } from '../components/NtaPerformancePanel/NtaSections';
import { routeForPath } from '../core/nta/gapRoutes';
import type { IProject } from '../core/energy/types';
import terraced from '../../training-data/nta8800-example-terraced-dwelling.json';
import { renderWithProviders } from './test-utils';

// Review 44: the forfait ΔU_for of 8.2.1 is a choice for the whole building,
// also for one without a floor on the ground (an upper-floor dwelling).

function Section({ project }: { project: IProject }) {
  const def = NTA_SECTIONS.find((section) => section.id === 'thermalBridgeRoute')!;
  const [draft, setDraft] = useState<Draft>(project.ntaCalculation as unknown as Draft);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  const update = (next: (current: Draft) => Draft) => setDraft((current) => next(current));
  const Component = def.Component;
  return <form aria-label="section">
    <Component draft={draft} change={change} update={update} project={project} />
    <output data-testid="draft">{JSON.stringify(draft)}</output>
  </form>;
}

const draft = () => JSON.parse(screen.getByTestId('draft').textContent ?? '{}') as Record<string, unknown>;

describe('thermal-bridge route of the building', () => {
  it('writes the route and clears it again', () => {
    renderWithProviders(<Section project={terraced as unknown as IProject} />);
    const select = screen.getByLabelText(/Linear thermal bridges of the building/);
    fireEvent.change(select, { target: { value: 'forfait' } });
    expect(draft().thermalBridgeRoute).toBe('forfait');
    fireEvent.change(select, { target: { value: '' } });
    expect(draft().thermalBridgeRoute ?? null).toBeNull();
  });

  it('sends a gap on the route to the envelope page', () => {
    expect(routeForPath('ntaCalculation.thermalBridgeRoute')).toMatchObject({ step: 'building', sub: 'envelope' });
  });
});
