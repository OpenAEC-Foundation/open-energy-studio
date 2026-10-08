/**
 * "Opname doorrekenen" from a section of the survey wizard gives feedback in
 * that section: the status, this section's errors with "Ga naar" and the way
 * to the full outcome. Before, a calculation from General showed nothing in
 * the page itself.
 */
import { useState } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen, within } from '@testing-library/react';
import { BasisopnamePanel, type SurveySection } from '../components/BasisopnamePanel/BasisopnamePanel';
import { renderWithProviders, userEvent } from './test-utils';

afterEach(() => vi.unstubAllGlobals());

const assessment = {
  status: 'invalid', scope: 'x', source: 'ISSO 75.1',
  appliedDefaults: [],
  warnings: [],
  issues: [
    { code: 'value_required', path: 'constructionYear' },
    { code: 'value_required', path: 'envelope.surfaces[0].areaM2' },
  ],
  derivedInput: null,
  performance: { indicativeLabelClass: 'D', needIndicatorKwhPerM2Year: 150, primaryFossilIndicatorKwhPerM2Year: 260.4, renewableSharePercent: 2 },
  referenceVerified: false,
};

function Wizard({ calls }: { calls: Array<[SurveySection, string | undefined]> }) {
  const [section, setSection] = useState<SurveySection>('general');
  return <BasisopnamePanel section={section} onSection={(next, focus) => { calls.push([next, focus]); setSection(next); }} />;
}

describe('survey outcome in the current section', () => {
  it('shows the status and this section\'s errors after calculating, with a way to the field and to the full outcome', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({ json: async () => assessment })));
    const calls: Array<[SurveySection, string | undefined]> = [];
    const user = userEvent.setup();
    renderWithProviders(<Wizard calls={calls} />);
    await user.click(screen.getByRole('button', { name: 'Start utility survey' }));
    expect(screen.queryByRole('status', { name: 'Outcome for this part' })).toBeNull();

    const card = screen.getByRole('complementary', { name: 'Survey outcome' });
    await user.click(within(card).getByRole('button', { name: 'Calculate survey' }));

    const outcome = await screen.findByRole('status', { name: 'Outcome for this part' });
    expect(outcome).toHaveTextContent('Survey calculated:');
    expect(outcome).toHaveTextContent('D');
    expect(outcome).toHaveTextContent('1 more error(s) in other parts');
    // Only General's own error, with a way to its field.
    await user.click(within(outcome).getByRole('button', { name: 'Go to the field' }));
    expect(calls[calls.length - 1]).toEqual(['general', 'basisopname.constructionYear']);

    await user.click(within(outcome).getByRole('button', { name: 'Full outcome' }));
    expect(calls[calls.length - 1]).toEqual(['result', undefined]);
    // On the outcome page itself the section block is not repeated.
    expect(screen.queryByRole('status', { name: 'Outcome for this part' })).toBeNull();
  }, 60000);

  it('says so when the section has no errors', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({ json: async () => ({ ...assessment, issues: [], status: 'calculated_unverified' }) })));
    const user = userEvent.setup();
    renderWithProviders(<Wizard calls={[]} />);
    await user.click(screen.getByRole('button', { name: 'Start dwelling survey' }));
    const card = screen.getByRole('complementary', { name: 'Survey outcome' });
    await user.click(within(card).getByRole('button', { name: 'Calculate survey' }));
    const outcome = await screen.findByRole('status', { name: 'Outcome for this part' });
    expect(outcome).toHaveTextContent('No errors in General.');
    expect(within(outcome).queryByRole('button', { name: 'Go to the field' })).toBeNull();
  }, 60000);
});
