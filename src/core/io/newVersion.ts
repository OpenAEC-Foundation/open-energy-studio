/**
 * New-version check of the web build (feedback 9 Oct 2026): the server sends the
 * start page without a cache instruction, so an open tab or a plain navigation
 * keeps the previous build. The app compares its own bundle with the one the
 * server has now.
 */

/** The bundle (assets/index-<hash>.js) a start page loads, or null. */
export function bundleOf(html: string): string | null {
  return html.match(/assets\/index-[\w-]+\.js/)?.[0] ?? null;
}

/** The bundle this page runs. */
export function runningBundle(doc: Document = document): string | null {
  for (const script of Array.from(doc.scripts)) {
    const found = bundleOf(script.src);
    if (found) return found;
  }
  return null;
}

/**
 * Whether the server has another build than this page runs. Fetches with
 * `cache: 'reload'`, which also refreshes the browser's copy, so a reload then
 * gets the new build.
 */
export async function newerBuildAvailable(fetcher: typeof fetch = fetch, doc: Document = document): Promise<boolean> {
  const running = runningBundle(doc);
  if (!running) return false;
  try {
    const response = await fetcher('/', { cache: 'reload' });
    if (!response.ok) return false;
    const live = bundleOf(await response.text());
    return live != null && live !== running;
  } catch {
    return false;
  }
}
