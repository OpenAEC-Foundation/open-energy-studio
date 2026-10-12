import { useEffect, useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { act, render, screen, waitFor } from '@testing-library/react';
import { Section } from '../components/NtaPerformancePanel/NtaFormFields';
import { RegistrationForm } from '../components/shell/pages/RegistrationForm';
import { useEnergy } from '../context/EnergyContext';
import { softwareIdentity } from '../core/nta/Registration';
import i18next from '../i18n/i18n';
import { renderWithProviders } from './test-utils';
import { SLOW_UI_TEST_MS } from './timeouts';

vi.mock('../core/io/KernelStampClient', () => ({
  stampProject: vi.fn(async () => ({ kernelVersion: '0.4.0', targetNormVersion: 'nta8800_2025_c1_2026', inputFingerprint: 'x' })),
}));

// Review 30 (10 October 2026): findings on the UI merges since 9fa2683.

describe('review 30', () => {
  it('marks a select wide when only its option text changes, without a render of the section', async () => {
    let setLong: (value: boolean) => void = () => undefined;
    function Options() {
      const [long, setLongState] = useState(false);
      setLong = setLongState;
      return <label>Keuze<select><option>{long ? 'Een heel lange optietekst die niet in één kolom past' : 'Kort'}</option></select></label>;
    }
    render(<Section title="Test"><Options /></Section>);
    const label = screen.getByText('Keuze').closest('label')!;
    expect(label).not.toHaveAttribute('data-wide-select');
    act(() => setLong(true));
    await waitFor(() => expect(label).toHaveAttribute('data-wide-select'));
    act(() => setLong(false));
    await waitFor(() => expect(label).not.toHaveAttribute('data-wide-select'));
  });

  it('uses plural forms for the edition-switch summary and the heat-pump count', async () => {
    await i18next.changeLanguage('nl');
    expect(i18next.t('nta.editionSwitch.summary', { count: 1, automatic: 0, open: 1 })).toBe('1 punt; 0 automatisch aan te passen, daarna 1 voor u.');
    expect(i18next.t('nta.editionSwitch.summary', { count: 3, automatic: 1, open: 2 })).toBe('3 punten; 1 automatisch aan te passen, daarna 2 voor u.');
    expect(i18next.t('installations.heatPumpCount', { count: 1 })).toBe('1 warmtepomp');
    expect(i18next.t('installations.heatPumpCount', { count: 2 })).toBe('2 warmtepompen');
    await i18next.changeLanguage('en');
    expect(i18next.t('installations.heatPumpCount', { count: 1 })).toBe('1 heat pump');
    expect(i18next.t('nta.editionSwitch.summary', { count: 2, automatic: 0, open: 2 })).toBe('2 items; 0 can be adjusted automatically, 2 remain for you.');
  });

  it('lets Save restamp a registration saved with another kernel version', async () => {
    function WithStoredRegistration() {
      const { dispatch } = useEnergy();
      useEffect(() => {
        dispatch({ type: 'UPDATE_PROJECT_INFO', payload: { registration: { bagObjectId: '0363010000000001', software: softwareIdentity('0.3.0') } } });
      }, [dispatch]);
      return <RegistrationForm />;
    }
    renderWithProviders(<WithStoredRegistration />);
    const save = screen.getByRole('button', { name: 'Save' });
    await waitFor(() => expect(save).toBeEnabled());
    expect(screen.getByText('Saved with another kernel version; saving records the current one')).toBeInTheDocument();
    // Discarding has nothing to discard: the fields are unchanged.
    expect(screen.getByRole('button', { name: 'Discard' })).toBeDisabled();
  }, SLOW_UI_TEST_MS);
});
