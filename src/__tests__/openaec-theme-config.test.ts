/**
 * OpenAEC Theme Configuration Tests
 *
 * Validates that ThemePicker, SettingsDialog, and main.tsx
 * are correctly configured for the OpenAEC theme system.
 */
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'fs';
import { resolve } from 'path';

function readSource(relativePath: string): string {
  return readFileSync(resolve(__dirname, '..', relativePath), 'utf-8');
}

// ── ThemePicker ──

describe('Theme options (SettingsDialog; ThemePicker removed with the ribbon)', () => {
  const src = readSource('components/SettingsDialog/SettingsDialog.tsx');

  it('exports Theme type without "blue"', () => {
    expect(src).toMatch(/type Theme\s*=\s*'system'\s*\|\s*'light'\s*\|\s*'dark'\s*\|\s*'highContrast'/);
    expect(src).not.toMatch(/type Theme.*'blue'/);
  });

  it('defines System theme with OpenAEC swatches', () => {
    expect(src).toContain('#36363E');
    expect(src).toContain('#D97706');
    expect(src).toContain('#FAFAF9');
  });

  it('defines Dark theme with Deep Forge / Night Build swatches', () => {
    expect(src).toContain('#2A2A32');
    expect(src).toContain('#36363E');
  });

  it('defines Light theme with Blueprint White swatches', () => {
    expect(src).toContain('#FAFAF9');
    expect(src).toContain('#F5F5F4');
  });

  it('does NOT include a Blue theme option', () => {
    expect(src).not.toMatch(/value:\s*'blue'/);
    expect(src).not.toContain('#00b4d8'); // old blue accent
    expect(src).not.toContain('#0d1b2a'); // old blue bg
  });

  it('does NOT use old dark theme colors', () => {
    expect(src).not.toContain('#0d1117'); // old GitHub-style dark
    expect(src).not.toContain('#161b22'); // old GitHub-style primary
    expect(src).not.toContain('#3b82f6'); // old blue accent
  });
});

// ── SettingsDialog ──

describe('SettingsDialog theme options', () => {
  const src = readSource('components/SettingsDialog/SettingsDialog.tsx');

  it('defines Theme type without "blue"', () => {
    expect(src).toMatch(/type Theme\s*=\s*'system'\s*\|\s*'light'\s*\|\s*'dark'\s*\|\s*'highContrast'/);
    expect(src).not.toMatch(/type Theme.*'blue'/);
  });

  it('uses OpenAEC amber swatches', () => {
    expect(src).toContain('#D97706');
  });

  it('uses Deep Forge / Night Build dark swatches', () => {
    expect(src).toContain('#36363E');
    expect(src).toContain('#2A2A32');
  });

  it('does NOT reference old blue theme colors', () => {
    expect(src).not.toContain('#00b4d8');
    expect(src).not.toContain('#0d1b2a');
    expect(src).not.toContain('#1b263b');
  });

  it('does NOT include blue theme entry', () => {
    expect(src).not.toMatch(/value:\s*'blue'/);
    expect(src).not.toMatch(/labelKey:\s*'theme\.blue'/);
  });
});

// ── main.tsx theme migration ──

describe('main.tsx theme migration', () => {
  const src = readSource('main.tsx');

  it('migrates "blue" theme to "dark"', () => {
    expect(src).toContain("stored === 'blue'");
  });

  it('sets localStorage to "dark" after migration', () => {
    expect(src).toContain("localStorage.setItem('energy-theme', 'dark')");
  });

  it('defaults to dark theme', () => {
    expect(src).toContain("|| 'dark'");
  });
});

// ── index.html font loading ──

describe('index.html Google Fonts', () => {
  const html = readFileSync(resolve(__dirname, '../../index.html'), 'utf-8');

  it('preconnects to Google Fonts', () => {
    expect(html).toContain('fonts.googleapis.com');
    expect(html).toContain('fonts.gstatic.com');
  });

  it('loads Space Grotesk', () => {
    expect(html).toContain('Space+Grotesk');
  });

  it('loads Inter', () => {
    expect(html).toContain('Inter');
  });

  it('loads JetBrains Mono', () => {
    expect(html).toContain('JetBrains+Mono');
  });

  it('loads correct font weights (500,700 for Space Grotesk; 400,500,600,700 for Inter; 400,500 for JetBrains Mono)', () => {
    expect(html).toMatch(/Space\+Grotesk:wght@500;700/);
    expect(html).toMatch(/Inter:wght@400;500;600;700/);
    expect(html).toMatch(/JetBrains\+Mono:wght@400;500/);
  });
});
