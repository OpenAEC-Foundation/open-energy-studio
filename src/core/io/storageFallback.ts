/**
 * Browser storage when the page may not have any: in a sandboxed frame without `allow-same-origin` (the workspace
 * of OpenAEC SuperCloud) reading `window.localStorage` or `window.sessionStorage` throws a SecurityError, and the app
 * would not even render. Then both become an in-memory Storage for this page: settings and autosave last until the
 * frame closes, and saving goes through the host. Imported first in main.tsx, before any module reads storage.
 */

export class MemoryStorage implements Storage {
  private items = new Map<string, string>();
  get length() { return this.items.size; }
  clear() { this.items.clear(); }
  getItem(key: string) { return this.items.get(String(key)) ?? null; }
  key(index: number) { return [...this.items.keys()][index] ?? null; }
  removeItem(key: string) { this.items.delete(String(key)); }
  setItem(key: string, value: string) { this.items.set(String(key), String(value)); }
}

/** Replaces every storage of `target` that cannot be read; returns the names it replaced. */
export function installStorageFallback(target: Window = window): string[] {
  const replaced: string[] = [];
  for (const name of ['localStorage', 'sessionStorage'] as const) {
    try {
      if (target[name]) continue;
    } catch {
      // Blocked: fall through.
    }
    try {
      Object.defineProperty(target, name, { value: new MemoryStorage(), configurable: true, enumerable: true });
      replaced.push(name);
    } catch {
      // Not replaceable here; the code that reads storage keeps its own guards.
    }
  }
  return replaced;
}

installStorageFallback();
