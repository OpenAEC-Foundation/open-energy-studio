import { describe, expect, it, vi, beforeEach } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from './test-utils';
import type { IProject } from '../core/energy/types';
import type { ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import { dutchNumber } from '../core/report/DutchReportText';

vi.mock('../core/nta/KernelClient', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../core/nta/KernelClient')>()),
  fetchKernelInterpretations: vi.fn().mockResolvedValue([]),
}));

const { ReportBuilder, loadReportChoice } = await import('../components/ReportView/ReportBuilder');

const root = resolve(__dirname, '../..');
const project = JSON.parse(readFileSync(resolve(root, 'training-data/nta8800-example-terraced-dwelling.json'), 'utf8')) as IProject;
const assessment = JSON.parse(readFileSync(resolve(root, 'training-data/nta8800-example-terraced-dwelling.kernel-output.json'), 'utf8')) as ProjectPerformanceAssessment;
const preview = () => (screen.getByTestId('report-builder-preview') as HTMLIFrameElement).getAttribute('srcdoc') ?? '';

describe('report builder', () => {
  beforeEach(() => localStorage.clear());

  it('previews the chosen level, enables the detail chapters only for the detailed level and remembers the choice', async () => {
    const user = userEvent.setup();
    renderWithProviders(<ReportBuilder project={project} assessment={assessment} pending={false} />);
    // Default: the detailed level with the calculation chapters (feedback 9 Oct 2026).
    const balance = screen.getByLabelText(/Heat and cold balance|Warmte- en koudebalans/);
    expect(screen.getByRole('radio', { name: /Detailed|Gedetailleerd/ })).toBeChecked();
    expect(balance).toBeEnabled();
    // Standard: building data, no calculation chapters.
    await user.click(screen.getByRole('radio', { name: /Standard|Standaard/ }));
    expect(preview()).toContain('Bouwkundige uitgangspunten');
    expect(preview()).not.toContain('Berekening: Warmte- en koudebalans');
    expect(balance).toBeDisabled();

    await user.click(screen.getByRole('radio', { name: /Detailed|Gedetailleerd/ }));
    expect(balance).toBeEnabled();
    expect(preview()).toContain('Berekening: Warmte- en koudebalans');
    expect(preview()).toContain(dutchNumber(assessment.performance!.spaceHeating.demand.monthly[0].heating.needKwh));

    await user.click(balance);
    expect(preview()).not.toContain('Berekening: Warmte- en koudebalans');
    expect(loadReportChoice()).toMatchObject({ level: 'detailed', details: { balance: false, transmission: true } });

    await user.click(screen.getByRole('radio', { name: /Summary|Samenvatting/ }));
    expect(preview()).not.toContain('Bouwkundige uitgangspunten');
    expect(preview()).toContain(dutchNumber(assessment.performance!.primaryFossilIndicatorKwhPerM2Year, 2));
  }, 60000);

  it('says so while the kernel is still running', () => {
    renderWithProviders(<ReportBuilder project={project} assessment={null} pending />);
    expect(screen.getByRole('status')).toHaveTextContent(/kernel|rekenkern/i);
  }, 60000);
});
