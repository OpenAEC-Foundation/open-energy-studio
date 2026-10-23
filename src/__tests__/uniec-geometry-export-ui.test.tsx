/**
 * Issue #14: the Exports page offers the Uniec geometry workbook. A project
 * that lacks what the contract needs is refused with the reasons; nothing is
 * downloaded then.
 */
import { describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { ExportsPage } from '../components/shell/pages/DeliveryPages';
import * as importFile from '../core/io/importFile';
import { renderWithProviders, userEvent } from './test-utils';
import { SLOW_UI_TEST_MS } from './timeouts';

const noop = () => {};
const actions = { exportReport: noop, printReport: noop, exportIFC: noop, exportModelIFC: noop, exportUNIEC3: noop, exportVABI: noop };

describe('Uniec geometry on the Exports page', () => {
  it('refuses a new project with the missing data and downloads nothing', async () => {
    const download = vi.spyOn(importFile, 'downloadBytes').mockImplementation(() => {});
    const user = userEvent.setup();
    renderWithProviders(<ExportsPage actions={actions} />);
    await user.click(screen.getByRole('button', { name: 'Uniec geometry (Excel, draft)' }));
    const banner = screen.getByText(/Uniec geometry not made/).closest('[role="alert"]') as HTMLElement;
    expect(banner).not.toBeNull();
    expect(banner.textContent).toContain('Missing or not supported:');
    expect(banner.textContent).toContain('Plaats');
    expect(banner.textContent).toContain('Bouwjaar');
    expect(download).not.toHaveBeenCalled();
    download.mockRestore();
  }, SLOW_UI_TEST_MS);
});
