import { useState, useEffect, useRef } from 'react';

export type Theme = 'system' | 'light' | 'dark' | 'highContrast';

interface ThemeOption {
  value: Theme;
  label: string;
  swatches: string[];
}

const THEME_OPTIONS: ThemeOption[] = [
  { value: 'system', label: 'System', swatches: ['#36363E', '#2A2A32', '#D97706', '#FAFAF9'] },
  { value: 'light', label: 'Light', swatches: ['#FAFAF9', '#F5F5F4', '#D97706', '#36363E'] },
  { value: 'dark', label: 'Dark', swatches: ['#2A2A32', '#36363E', '#D97706', '#FAFAF9'] },
  { value: 'highContrast', label: 'High Contrast', swatches: ['#000000', '#0a0a0a', '#ffff00', '#ffffff'] },
];

function resolveTheme(theme: Theme): string {
  if (theme === 'system') {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }
  return theme;
}

interface ThemePickerProps {
  theme: Theme;
  onThemeChange: (theme: Theme) => void;
}

export function ThemePicker({ theme, onThemeChange }: ThemePickerProps) {
  const [open, setOpen] = useState(false);
  const pickerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (pickerRef.current && !pickerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('click', handler);
    return () => document.removeEventListener('click', handler);
  }, []);

  // Apply theme to document
  useEffect(() => {
    const effective = resolveTheme(theme);
    document.documentElement.dataset.theme = effective;
    localStorage.setItem('energy-theme', theme);
  }, [theme]);

  // Listen for OS theme changes when "system" is selected
  useEffect(() => {
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = () => {
      if (theme === 'system') {
        document.documentElement.dataset.theme = mq.matches ? 'dark' : 'light';
      }
    };
    mq.addEventListener('change', handler);
    return () => mq.removeEventListener('change', handler);
  }, [theme]);

  const current = THEME_OPTIONS.find(t => t.value === theme) || THEME_OPTIONS[2];

  return (
    <div className="ribbon-input-group" style={{ justifyContent: 'center' }}>
      <label className="ribbon-input-label">Theme</label>
      <div className="theme-picker" ref={pickerRef}>
        <button
          className="theme-picker-toggle"
          type="button"
          onClick={(e) => { e.stopPropagation(); setOpen(!open); }}
        >
          <span className="theme-picker-swatches">
            {current.swatches.map((color, i) => (
              <span key={i} className="theme-swatch" style={{ background: color }} />
            ))}
          </span>
          <span className="theme-picker-label">{current.label}</span>
          <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" width="10" height="10">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="3" d="M6 9l6 6 6-6"/>
          </svg>
        </button>
        <div
          className={`theme-picker-dropdown${open ? ' open' : ''}`}
          onClick={(e) => e.stopPropagation()}
        >
          {THEME_OPTIONS.map(option => (
            <div
              key={option.value}
              className={`theme-picker-option${option.value === theme ? ' selected' : ''}`}
              onClick={() => { onThemeChange(option.value); setOpen(false); }}
            >
              <span className="theme-picker-option-swatches">
                {option.swatches.map((color, i) => (
                  <span key={i} className="theme-swatch" style={{ background: color }} />
                ))}
              </span>
              <span className="theme-picker-option-label">{option.label}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
