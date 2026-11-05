/**
 * Feedback in the app (5 Nov 2026): the same flow as open-aec.com. A report
 * goes to Open Feedback Studio with app id `open-energy-studio`, which opens
 * a GitHub issue in OpenAEC-Foundation/open-energy-studio. The tests stub
 * the API; nothing is sent.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { FEEDBACK_API, FeedbackDialog, issueNumber } from '../components/dialogs/FeedbackDialog/FeedbackDialog';
import { renderWithProviders, userEvent } from './test-utils';
import { SLOW_UI_TEST_MS } from './timeouts';
import { version } from '../../package.json';

afterEach(() => {
  vi.unstubAllGlobals();
  localStorage.removeItem('feedback-email');
});

function stubApi(status: number, body: unknown) {
  const fetchMock = vi.fn(async () => new Response(JSON.stringify(body), { status }));
  vi.stubGlobal('fetch', fetchMock);
  return fetchMock;
}

async function fillReport(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByRole('button', { name: 'Idea' }));
  await user.type(screen.getByLabelText('Title'), 'TOjuli per zone exporteren');
  await user.type(screen.getByLabelText('Description'), 'Graag een CSV met alle termen.');
}

describe('feedback dialog', () => {
  it('sends a report for Open Energy Studio and links the GitHub issue', async () => {
    const fetchMock = stubApi(200, { success: true, githubIssueUrl: 'https://github.com/OpenAEC-Foundation/open-energy-studio/issues/42' });
    const user = userEvent.setup();
    renderWithProviders(<FeedbackDialog onClose={() => {}} />);
    // The report is public and says so.
    expect(screen.getByText(/public issue on GitHub in OpenAEC-Foundation\/open-energy-studio/)).toBeInTheDocument();
    const send = screen.getByRole('button', { name: 'Send' });
    expect(send).toBeDisabled();
    await fillReport(user);
    expect(send).toBeEnabled();
    await user.click(send);
    expect(fetchMock).toHaveBeenCalledTimes(1);
    const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe(FEEDBACK_API);
    const body = JSON.parse(String(init.body)) as Record<string, string>;
    expect(body).toMatchObject({ app: 'open-energy-studio', category: 'feature', appVersion: version });
    expect(body.message).toMatch(/^TOjuli per zone exporteren\n\nGraag een CSV met alle termen\.\n\n_Via Open Energy Studio /);
    // No email given: none sent.
    expect(body.email).toBeUndefined();
    expect(await screen.findByText(/An issue has been created in OpenAEC-Foundation\/open-energy-studio/)).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'View on GitHub' })).toHaveAttribute('href', 'https://github.com/OpenAEC-Foundation/open-energy-studio/issues/42');
  }, SLOW_UI_TEST_MS);

  it('sends the optional email address and refuses an invalid one', async () => {
    const fetchMock = stubApi(200, { success: true });
    const user = userEvent.setup();
    renderWithProviders(<FeedbackDialog onClose={() => {}} />);
    await fillReport(user);
    await user.type(screen.getByLabelText(/Email address/), 'geen-adres');
    expect(screen.getByText('This email address is not valid.')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Send' })).toBeDisabled();
    await user.clear(screen.getByLabelText(/Email address/));
    await user.type(screen.getByLabelText(/Email address/), 'tester@example.com');
    await user.click(screen.getByRole('button', { name: 'Send' }));
    const body = JSON.parse(String((fetchMock.mock.calls[0] as unknown as [string, RequestInit])[1].body)) as Record<string, string>;
    expect(body.email).toBe('tester@example.com');
    // Without an issue URL the report is still received.
    expect(await screen.findByText('The makers will see your report in their overview.')).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'View on GitHub' })).toBeNull();
  }, SLOW_UI_TEST_MS);

  it('names the reason when the service refuses', async () => {
    stubApi(429, { success: false });
    const user = userEvent.setup();
    renderWithProviders(<FeedbackDialog onClose={() => {}} />);
    await fillReport(user);
    await user.click(screen.getByRole('button', { name: 'Send' }));
    expect(await screen.findByText(/sent many reports in a short time/)).toBeInTheDocument();
    // The form stays filled, so the report is not lost.
    expect(screen.getByLabelText('Title')).toHaveValue('TOjuli per zone exporteren');
  }, SLOW_UI_TEST_MS);

  it('links only issues of this repository', () => {
    expect(issueNumber('https://github.com/OpenAEC-Foundation/open-energy-studio/issues/7')).toBe(7);
    expect(issueNumber('https://github.com/OpenAEC-Foundation/website/issues/7')).toBeNull();
    expect(issueNumber('javascript:alert(1)')).toBeNull();
    expect(issueNumber(undefined)).toBeNull();
  });
});
