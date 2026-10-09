/** New-version check of the web build (feedback 9 Oct 2026). */
import { describe, expect, it, vi } from 'vitest';
import { bundleOf, newerBuildAvailable } from '../core/io/newVersion';

const page = (bundle: string) => `<html><head><script type="module" crossorigin src="/assets/${bundle}"></script></head></html>`;
const docRunning = (bundle: string) => {
  const doc = document.implementation.createHTMLDocument('x');
  const script = doc.createElement('script');
  script.src = `https://example.test/assets/${bundle}`;
  doc.head.append(script);
  return doc;
};
const serving = (html: string) => vi.fn().mockResolvedValue({ ok: true, text: async () => html }) as unknown as typeof fetch;

describe('new-version check', () => {
  it('reads the bundle of a start page', () => {
    expect(bundleOf(page('index-C5FiGs1g.js'))).toBe('assets/index-C5FiGs1g.js');
    expect(bundleOf('<html></html>')).toBeNull();
  });

  it('reports a newer build only when the server has another bundle, and refreshes the cached page', async () => {
    const fetcher = serving(page('index-NEW.js'));
    expect(await newerBuildAvailable(fetcher, docRunning('index-OLD.js'))).toBe(true);
    expect(fetcher).toHaveBeenCalledWith('/', { cache: 'reload' });
    expect(await newerBuildAvailable(serving(page('index-OLD.js')), docRunning('index-OLD.js'))).toBe(false);
    expect(await newerBuildAvailable(vi.fn().mockRejectedValue(new Error('offline')) as unknown as typeof fetch, docRunning('index-OLD.js'))).toBe(false);
  });
});
