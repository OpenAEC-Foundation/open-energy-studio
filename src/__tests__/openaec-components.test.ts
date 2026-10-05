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

// ── TitleBar ──

describe('TitleBar', () => {
  const css = readCSS('components/TitleBar/TitleBar.css');

  it('uses Space Grotesk for app name', () => {
    expect(css).toMatch(/\.title-bar-app-name[^}]*font-family:\s*'Space Grotesk'/s);
  });

  it('uses font-weight 700 for app name', () => {
    expect(css).toMatch(/\.title-bar-app-name[^}]*font-weight:\s*700/s);
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

// ── ProjectBrowser ──

describe('ProjectBrowser', () => {
  const css = readCSS('components/ProjectBrowser/ProjectBrowser.css');

  it('uses JetBrains Mono for header (section label style)', () => {
    expect(css).toMatch(/\.project-browser-header[^}]*font-family:\s*'JetBrains Mono'/s);
  });

  it('uses amber color for header', () => {
    expect(css).toMatch(/\.project-browser-header[^}]*color:\s*var\(--accent\)/s);
  });

  it('uses uppercase text-transform', () => {
    expect(css).toMatch(/\.project-browser-header[^}]*text-transform:\s*uppercase/s);
  });

  it('uses 0.1em letter-spacing', () => {
    expect(css).toMatch(/\.project-browser-header[^}]*letter-spacing:\s*0\.1em/s);
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

// ── ProjectView ──

describe('ProjectView', () => {
  const css = readCSS('components/ProjectView/ProjectView.css');

  it('uses Space Grotesk for h2', () => {
    expect(css).toMatch(/\.project-view-header h2[^}]*font-family:\s*'Space Grotesk'/s);
  });
});

// ── AppMenu ──

describe('AppMenu', () => {
  const css = readCSS('components/AppMenu/AppMenu.css');

  it('uses Space Grotesk for panel title', () => {
    expect(css).toMatch(/\.app-menu-panel-title[^}]*font-family:\s*'Space Grotesk'/s);
  });

  it('uses font-weight 700 for panel title', () => {
    expect(css).toMatch(/\.app-menu-panel-title[^}]*font-weight:\s*700/s);
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

// ── Ribbon ──

describe('Ribbon', () => {
  const css = readCSS('components/Ribbon/Ribbon.css');

  it('uses gradient background', () => {
    expect(css).toMatch(/\.ribbon-container[^}]*background:\s*linear-gradient/s);
  });

  it('defines group label style', () => {
    expect(css).toMatch(/\.ribbon-group-label[^}]*text-transform:\s*uppercase/s);
  });
});
