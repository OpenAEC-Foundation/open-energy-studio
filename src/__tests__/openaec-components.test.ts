/**
 * OpenAEC Component CSS Tests
 *
 * Validates that individual component stylesheets follow the OpenAEC design system:
 * - Space Grotesk for headings
 * - JetBrains Mono for section labels and code
 * - Amber accent color
 * - Correct border-radius (8px cards, 9999px badges)
 * - Gradient accent strip
 */
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'fs';
import { resolve } from 'path';

function readCSS(componentPath: string): string {
  return readFileSync(resolve(__dirname, '..', componentPath), 'utf-8');
}

// ── TopBar (UI redesign F4; replaces TitleBar) ──

describe('TopBar', () => {
  const css = readCSS('components/shell/shell.css');
  const tokens = readCSS('styles/tokens.css');

  it('uses the display font (Space Grotesk) for the app name', () => {
    expect(css).toMatch(/\.top-bar-app-name[^}]*font-family:\s*var\(--font-display\)/s);
    expect(tokens).toMatch(/--font-display:\s*'Space Grotesk'/);
  });

  it('uses font-weight 700 for app name', () => {
    expect(css).toMatch(/\.top-bar-app-name[^}]*font-weight:\s*700/s);
  });
});

// ── StatusBar ──

describe('StatusBar', () => {
  const css = readCSS('components/StatusBar/StatusBar.css');

  // UI redesign F1: the status bar is neutral chrome, not an amber call to action.
  it('uses the neutral chrome surface as background', () => {
    expect(css).toMatch(/\.status-bar[^}]*background:\s*var\(--surface-chrome\)/s);
  });

  it('uses theme text colour on the chrome surface', () => {
    expect(css).toMatch(/\.status-bar[^}]*color:\s*var\(--fg-2\)/s);
  });

});

// ── WelcomeScreen ──

describe('WelcomeScreen', () => {
  const css = readCSS('components/WelcomeScreen/WelcomeScreen.css');

  it('uses Space Grotesk for title', () => {
    expect(css).toMatch(/\.welcome-title[^}]*font-family:\s*'Space Grotesk'/s);
  });

  it('uses font-weight 700 for title', () => {
    expect(css).toMatch(/\.welcome-title[^}]*font-weight:\s*700/s);
  });

  it('uses -0.02em letter-spacing for title', () => {
    expect(css).toMatch(/\.welcome-title[^}]*letter-spacing:\s*-0\.02em/s);
  });
});

// ── WorkflowNav (UI redesign F4; replaces ProjectBrowser) ──

describe('WorkflowNav', () => {
  const css = readCSS('components/shell/shell.css');

  it('uses the section label style for step groups (uppercase, spaced, muted)', () => {
    expect(css).toMatch(/\.nav-group[^}]*text-transform:\s*uppercase/s);
    expect(css).toMatch(/\.nav-group[^}]*letter-spacing:\s*\.08em/s);
    expect(css).toMatch(/\.nav-group[^}]*color:\s*var\(--fg-3\)/s);
  });

  it('marks the active step with the amber accent bar', () => {
    expect(css).toMatch(/\.nav-step\[aria-current="page"\]::before[^}]*background:\s*var\(--accent\)/s);
  });

  it('uses the neutral chrome surface', () => {
    expect(css).toMatch(/\.workflow-nav[^}]*background:\s*var\(--surface-chrome\)/s);
  });
});

// ── PropertiesPanel ──

describe('PropertiesPanel', () => {
  const css = readCSS('components/PropertiesPanel/PropertiesPanel.css');

  it('uses JetBrains Mono for header', () => {
    expect(css).toMatch(/\.properties-panel-header[^}]*font-family:\s*'JetBrains Mono'/s);
  });

  it('uses amber color for header', () => {
    expect(css).toMatch(/\.properties-panel-header[^}]*color:\s*var\(--accent\)/s);
  });
});

// ── PreviewPanel ──

describe('PreviewPanel', () => {
  const css = readCSS('components/PreviewPanel/PreviewPanel.css');

  it('uses JetBrains Mono for header', () => {
    expect(css).toMatch(/\.preview-panel-header[^}]*font-family:\s*'JetBrains Mono'/s);
  });

  it('uses amber color for header', () => {
    expect(css).toMatch(/\.preview-panel-header[^}]*color:\s*var\(--accent\)/s);
  });

  it('uses pill radius for BENG badges', () => {
    expect(css).toMatch(/\.preview-beng-badge[^}]*border-radius:\s*9999px/s);
  });

  it('uses JetBrains Mono for section titles', () => {
    expect(css).toMatch(/\.preview-section-title[^}]*font-family:\s*'JetBrains Mono'/s);
  });

  it('uses amber for section titles', () => {
    expect(css).toMatch(/\.preview-section-title[^}]*color:\s*var\(--accent\)/s);
  });
});

// ── BENGIndicator ──

describe('BENGIndicator', () => {
  const css = readCSS('components/BENGIndicator/BENGIndicator.css');

  it('uses Space Grotesk for title', () => {
    expect(css).toMatch(/\.beng-indicator-title[^}]*font-family:\s*'Space Grotesk'/s);
  });

  it('uses pill radius for badges', () => {
    expect(css).toMatch(/\.beng-indicator-badge[^}]*border-radius:\s*9999px/s);
  });

  it('uses uppercase for badges', () => {
    expect(css).toMatch(/\.beng-indicator-badge[^}]*text-transform:\s*uppercase/s);
  });
});

// ── ResultsView ──

describe('ResultsView', () => {
  const css = readCSS('components/ResultsView/ResultsView.css');

  it('uses Space Grotesk for h2', () => {
    expect(css).toMatch(/\.results-view h2[^}]*font-family:\s*'Space Grotesk'/s);
  });

  it('uses pill radius for TO-juli badges', () => {
    expect(css).toMatch(/\.to-juli-badge[^}]*border-radius:\s*9999px/s);
  });
});

// ── ReportView ──

describe('ReportView', () => {
  const css = readCSS('components/ReportView/ReportView.css');

  it('uses Space Grotesk for h1', () => {
    expect(css).toMatch(/\.report-header h1[^}]*font-family:\s*'Space Grotesk'/s);
  });

  it('uses Space Grotesk for h2', () => {
    expect(css).toMatch(/\.report-section h2[^}]*font-family:\s*'Space Grotesk'/s);
  });

  it('uses gradient accent strip on header border', () => {
    expect(css).toMatch(/border-image:\s*linear-gradient\(90deg,\s*#D97706.*#F59E0B.*#EA580C/i);
  });

  it('has white background for report content', () => {
    expect(css).toMatch(/\.report-content[^}]*background:\s*#ffffff/s);
  });
});

// ── EnvelopeView ──

describe('EnvelopeView', () => {
  const css = readCSS('components/EnvelopeView/EnvelopeView.css');

  it('uses Space Grotesk for h2', () => {
    expect(css).toMatch(/\.envelope-view h2[^}]*font-family:\s*'Space Grotesk'/s);
  });

  it('uses JetBrains Mono for section h4', () => {
    expect(css).toMatch(/\.envelope-section h4[^}]*font-family:\s*'JetBrains Mono'/s);
  });

  it('uses amber for section h4', () => {
    expect(css).toMatch(/\.envelope-section h4[^}]*color:\s*var\(--accent\)/s);
  });
});

// ── Page head (UI redesign F4; replaces the ProjectView header) ──

describe('PageHeader', () => {
  const css = readCSS('components/shell/shell.css');

  it('uses Space Grotesk for the page title', () => {
    expect(css).toMatch(/\.page-title[^}]*font-family:\s*'Space Grotesk'/s);
  });

  it('uses font-weight 700 for the page title', () => {
    expect(css).toMatch(/\.page-title[^}]*font-weight:\s*700/s);
  });
});

// ── Shell menus (UI redesign F4; replace the AppMenu backstage) ──

describe('Shell menus', () => {
  const css = readCSS('components/shell/shell.css');

  it('float on the raised surface with the strongest shadow', () => {
    expect(css).toMatch(/\.shell-menu \{[^}]*background:\s*var\(--surface-raised\)/s);
    expect(css).toMatch(/\.shell-menu \{[^}]*box-shadow:\s*var\(--shadow-3\)/s);
  });
});

// ── SettingsDialog ──

describe('SettingsDialog', () => {
  const css = readCSS('components/SettingsDialog/SettingsDialog.css');

  it('uses JetBrains Mono for section titles', () => {
    expect(css).toMatch(/\.settings-section-title[^}]*font-family:\s*'JetBrains Mono'/s);
  });

  it('uses amber for section titles', () => {
    expect(css).toMatch(/\.settings-section-title[^}]*color:\s*var\(--accent\)/s);
  });

  it('uses border-radius 4px for tables', () => {
    expect(css).toMatch(/\.language-table[^}]*border-radius:\s*4px/s);
    expect(css).toMatch(/\.theme-table[^}]*border-radius:\s*4px/s);
  });
});

// ── No ribbon (UI redesign F4: "Beperk aantal tabbladen op ribbon") ──

describe('App shell without ribbon', () => {
  const app = readCSS('App.tsx');

  it('renders no ribbon or tool tab strip', () => {
    expect(app).not.toMatch(/Ribbon/);
    expect(app).toMatch(/<WorkflowNav/);
  });
});
