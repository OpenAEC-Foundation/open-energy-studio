/**
 * OpenAEC Design System — CSS Token Tests
 *
 * Validates that the OpenAEC style book tokens are correctly defined
 * in the CSS custom properties for all themes.
 */
import { describe, it, expect, beforeEach } from 'vitest';

// ── Helper: parse CSS and extract custom properties for a selector ──

import { readFileSync } from 'fs';
import { resolve } from 'path';

function loadCSS(): string {
  return readFileSync(resolve(__dirname, '../index.css'), 'utf-8');
}

let css: string;

beforeEach(() => {
  css = loadCSS();
});

// ── OpenAEC Color Tokens ──

describe('OpenAEC Color Tokens', () => {
  describe('Dark theme (default :root)', () => {
    it('defines Construction Amber as accent', () => {
      expect(css).toMatch(/--accent:\s*#D97706/i);
    });

    it('defines Signal Orange as accent-hover', () => {
      expect(css).toMatch(/--accent-hover:\s*#EA580C/i);
    });

    it('defines Deep Forge as bg-primary', () => {
      expect(css).toMatch(/--bg-primary:\s*#36363E/i);
    });

    it('defines Night Build as bg-dark', () => {
      expect(css).toMatch(/--bg-dark:\s*#2A2A32/i);
    });

    it('defines Blueprint White as text-primary', () => {
      expect(css).toMatch(/--text-primary:\s*#FAFAF9/i);
    });

    it('defines Scaffold Gray as text-secondary', () => {
      expect(css).toMatch(/--text-secondary:\s*#A1A1AA/i);
    });

    it('defines Warm Gold as warning', () => {
      expect(css).toMatch(/--warning:\s*#F59E0B/i);
    });

    it('defines success green', () => {
      expect(css).toMatch(/--success:\s*#16A34A/i);
    });

    it('defines error red', () => {
      expect(css).toMatch(/--danger:\s*#DC2626/i);
    });

    it('defines energy-green', () => {
      expect(css).toMatch(/--energy-green:\s*#16A34A/i);
    });

    it('defines energy-orange', () => {
      expect(css).toMatch(/--energy-orange:\s*#F59E0B/i);
    });

    it('defines energy-red', () => {
      expect(css).toMatch(/--energy-red:\s*#DC2626/i);
    });
  });

  describe('Light theme', () => {
    it('uses Blueprint White (#FAFAF9) as bg-primary', () => {
      // Extract light theme block
      const lightBlock = css.match(/\[data-theme="light"\]\s*\{([^}]+)\}/s)?.[1] || '';
      expect(lightBlock).toMatch(/--bg-primary:\s*#FAFAF9/i);
    });

    it('uses Concrete (#F5F5F4) as bg-secondary', () => {
      const lightBlock = css.match(/\[data-theme="light"\]\s*\{([^}]+)\}/s)?.[1] || '';
      expect(lightBlock).toMatch(/--bg-secondary:\s*#F5F5F4/i);
    });

    it('uses Deep Forge (#36363E) as text-primary', () => {
      const lightBlock = css.match(/\[data-theme="light"\]\s*\{([^}]+)\}/s)?.[1] || '';
      expect(lightBlock).toMatch(/--text-primary:\s*#36363E/i);
    });

    it('uses Construction Amber as accent', () => {
      const lightBlock = css.match(/\[data-theme="light"\]\s*\{([^}]+)\}/s)?.[1] || '';
      expect(lightBlock).toMatch(/--accent:\s*#D97706/i);
    });
  });

  describe('Color rules', () => {
    it('never uses amber (#D97706) as a background fill in index.css', () => {
      // Amber should only be used for accents, never as bg-primary/bg-secondary/bg-dark
      const rootBlock = css.match(/:root[^{]*\{([^}]+)\}/s)?.[1] || '';
      expect(rootBlock).not.toMatch(/--bg-primary:\s*#D97706/i);
      expect(rootBlock).not.toMatch(/--bg-secondary:\s*#D97706/i);
      expect(rootBlock).not.toMatch(/--bg-dark:\s*#D97706/i);
    });
  });
});

// ── Shell tokens (F10: ribbon and title bar were replaced by the workflow shell) ──

function loadTokens(): string {
  return readFileSync(resolve(__dirname, '../styles/tokens.css'), 'utf-8');
}

describe('Shell tokens (tokens.css)', () => {
  it('keeps Construction Amber as the accent in dark and light', () => {
    const tokens = loadTokens();
    expect(tokens).toMatch(/--amber-600:\s*#D97706/i);
    expect(tokens).toMatch(/--accent:\s*var\(--amber-600\)/);
  });

  it('uses an amber-tinted selection and accent-subtle', () => {
    const tokens = loadTokens();
    expect(tokens).toMatch(/--surface-selected:\s*rgba\(217,\s*119,\s*6/i);
    expect(tokens).toMatch(/--accent-subtle:\s*rgba\(217,\s*119,\s*6/i);
  });

  it('draws the focus ring in amber', () => {
    expect(loadTokens()).toMatch(/--focus-ring:[^;]*var\(--amber-5\d\d\)/);
  });

  it('no longer defines ribbon or title-bar variables in index.css', () => {
    expect(css).not.toMatch(/--ribbon-/);
    expect(css).not.toMatch(/--titlebar-/);
  });
});


describe('OpenAEC Typography', () => {
  it('sets Inter as the body font', () => {
    expect(css).toMatch(/font-family:\s*'Inter'/);
  });

  it('defines Space Grotesk for headings', () => {
    expect(css).toMatch(/font-family:\s*'Space Grotesk'/);
  });

  it('defines JetBrains Mono for code/monospace', () => {
    expect(css).toMatch(/font-family:\s*'JetBrains Mono'/);
  });

  it('uses -0.02em letter-spacing for headings', () => {
    expect(css).toMatch(/letter-spacing:\s*-0\.02em/);
  });
});

// ── Buttons ──

describe('Button Styles', () => {
  it('defines border-radius: 8px for buttons', () => {
    expect(css).toMatch(/\.btn\s*\{[^}]*border-radius:\s*8px/s);
  });

  it('defines btn-primary with amber background', () => {
    expect(css).toMatch(/\.btn-primary\s*\{[^}]*background:\s*var\(--accent\)/s);
  });

  it('defines btn-primary with white text', () => {
    expect(css).toMatch(/\.btn-primary\s*\{[^}]*color:\s*white/s);
  });

  it('defines font-weight 600 for buttons (Inter Semi-Bold)', () => {
    expect(css).toMatch(/\.btn\s*\{[^}]*font-weight:\s*600/s);
  });
});

// ── Dialog Styles ──

describe('Dialog Styles', () => {
  it('defines border-radius: 8px for dialogs', () => {
    expect(css).toMatch(/\.dialog\s*\{[^}]*border-radius:\s*8px/s);
  });

  it('defines amber focus ring on inputs', () => {
    // F10: the faint 3px amber shadow became the shared focus ring (amber, checked under Shell tokens).
    expect(css).toMatch(/\.dialog-field input:focus[^{]*\{[^}]*border-color:\s*var\(--accent\);[^}]*box-shadow:\s*var\(--focus-ring\)/s);
  });

  it('defines Space Grotesk for dialog header title', () => {
    expect(css).toMatch(/\.dialog-header-title\s*\{[^}]*font-family:\s*'Space Grotesk'/s);
  });
});

// ── Theme Removal ──

describe('Theme cleanup', () => {
  it('does NOT define a blue theme', () => {
    expect(css).not.toMatch(/\[data-theme="blue"\]/);
  });
});
