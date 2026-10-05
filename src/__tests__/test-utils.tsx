/**
 * Test utilities — shared wrapper with all required providers.
 */
import React from 'react';
import { render, type RenderOptions } from '@testing-library/react';
import { EnergyProvider } from '../context/EnergyContext';
import { NtaDraftProvider } from '../context/NtaDraftProvider';
import { I18nProvider } from '../i18n/I18nProvider';

function AllProviders({ children }: { children: React.ReactNode }) {
  return (
    <I18nProvider>
      <EnergyProvider>
        <NtaDraftProvider>{children}</NtaDraftProvider>
      </EnergyProvider>
    </I18nProvider>
  );
}

export function renderWithProviders(ui: React.ReactElement, options?: Omit<RenderOptions, 'wrapper'>) {
  return render(ui, { wrapper: AllProviders, ...options });
}

export { default as userEvent } from '@testing-library/user-event';
