import { describe, expect, it } from 'vitest';
import { MemoryStorage, installStorageFallback } from '../core/io/storageFallback';

// A sandboxed frame without allow-same-origin throws on reading localStorage and sessionStorage.
function sandboxedWindow(): Window {
  const target = {};
  for (const name of ['localStorage', 'sessionStorage']) {
    Object.defineProperty(target, name, { configurable: true, get() { throw new DOMException('sandboxed', 'SecurityError'); } });
  }
  return target as Window;
}

describe('storage fallback', () => {
  it('replaces blocked storage with an in-memory Storage', () => {
    const target = sandboxedWindow();
    expect(installStorageFallback(target)).toEqual(['localStorage', 'sessionStorage']);
    target.localStorage.setItem('energy-theme', 'light');
    expect(target.localStorage.getItem('energy-theme')).toBe('light');
    expect(target.localStorage.length).toBe(1);
    expect(target.localStorage.key(0)).toBe('energy-theme');
    expect(target.sessionStorage.getItem('energy-theme')).toBeNull();
    target.localStorage.clear();
    expect(target.localStorage.length).toBe(0);
  });

  it('leaves working storage alone', () => {
    const real = new MemoryStorage();
    const target = { localStorage: real, sessionStorage: real } as unknown as Window;
    expect(installStorageFallback(target)).toEqual([]);
    expect(target.localStorage).toBe(real);
  });
});
