/**
 * OpenAEC Design System — CSS Token Tests
 *
 * Validates that the OpenAEC style book tokens are correctly defined
 * in the CSS custom properties for all themes.
 */
import { describe, it, expect, beforeEach } from 'vitest';

// ── Helper: parse CSS and extract custom properties for a selector ──

function loadCSS(): string {
  // We read the raw CSS source so we can verify token values without rendering
  // This is a static analysis approach
  const fs = require('fs');
  const path = require('path');
  return fs.readFileSync(path.resolve(__dirname, '../index.css'), 'utf-8');
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

// ── Ribbon Tokens ──

describe('Ribbon Tokens', () => {
  it('defines amber file tab background', () => {
    expect(css).toMatch(/--ribbon-file-tab-bg:\s*#D97706/i);
  });

  it('defines amber group border with opacity', () => {
    expect(css).toMatch(/--ribbon-group-border:\s*rgba\(217,\s*119,\s*6/i);
  });

  it('defines amber tab active border', () => {
    expect(css).toMatch(/--ribbon-tab-active-border:\s*#D97706/i);
  });

  it('defines amber-tinted button hover', () => {
    expect(css).toMatch(/--ribbon-btn-hover:\s*rgba\(217,\s*119,\s*6/i);
  });
});

// ── Titlebar Tokens ──

describe('Titlebar Tokens', () => {
  it('defines amber-tinted border', () => {
    expect(css).toMatch(/--titlebar-border:\s*rgba\(217,\s*119,\s*6/i);
  });

  it('defines amber-tinted button hover', () => {
    expect(css).toMatch(/--titlebar-btn-hover:\s*rgba\(217,\s*119,\s*6/i);
  });
});

// ── Typography ──

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
    expect(css).toMatch(/box-shadow:\s*0 0 0 3px rgba\(217,\s*119,\s*6,\s*0\.15\)/);
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
