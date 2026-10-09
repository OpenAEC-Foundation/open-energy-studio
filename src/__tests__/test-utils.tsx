/**
 * Test utilities — shared wrapper with all required providers.
 */
import React from 'react';
import { render, screen, within, type RenderOptions } from '@testing-library/react';
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

/** Chooses "With declaration" at a declaration field and types its number (feedback 9 Oct 2026). */
export async function declare(user: { click: (element: Element) => Promise<void>; type: (element: Element, text: string) => Promise<void> }, label: string | RegExp, text: string) {
  const group = screen.getByRole('group', { name: label });
  await user.click(within(group).getByRole('button', { name: 'With declaration' }));
  const box = group.closest('.survey-declaration') as HTMLElement;
  await user.type(within(box).getByRole('textbox', { name: /Declaration number/ }), text);
}
