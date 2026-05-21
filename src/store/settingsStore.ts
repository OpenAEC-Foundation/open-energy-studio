/**
 * User-level settings persisted in localStorage. Lightweight on purpose:
 * theme, language, recent files, and verification tolerance.
 */
import { create } from 'zustand';

const STORAGE_KEY = 'open-energy-studio.settings';

export type ThemeName = 'light' | 'dark' | 'system';

export interface Settings {
  theme: ThemeName;
  language: string; // i18next code, e.g. 'nl'
  recentFiles: string[];
  verificationTolerancePct: number;
}

interface SettingsState extends Settings {
  setTheme: (theme: ThemeName) => void;
  setLanguage: (lang: string) => void;
  pushRecentFile: (path: string) => void;
  setVerificationTolerance: (pct: number) => void;
}

const defaultSettings: Settings = {
  theme: 'dark',
  language: 'nl',
  recentFiles: [],
  verificationTolerancePct: 2.0,
};

function load(): Settings {
  if (typeof localStorage === 'undefined') return defaultSettings;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return defaultSettings;
    return { ...defaultSettings, ...(JSON.parse(raw) as Partial<Settings>) };
  } catch {
    return defaultSettings;
  }
}

function persist(s: Settings): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(s));
  } catch {
    // Ignore quota errors; settings are non-critical.
  }
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  ...load(),

  setTheme: (theme) =>
    set(() => {
      const next = { ...get(), theme };
      persist(extract(next));
      return { theme };
    }),

  setLanguage: (language) =>
    set(() => {
      const next = { ...get(), language };
      persist(extract(next));
      return { language };
    }),

  pushRecentFile: (path) =>
    set(() => {
      const filtered = get().recentFiles.filter((p) => p !== path);
      const recentFiles = [path, ...filtered].slice(0, 10);
      persist(extract({ ...get(), recentFiles }));
      return { recentFiles };
    }),

  setVerificationTolerance: (pct) =>
    set(() => {
      const verificationTolerancePct = Math.max(0, pct);
      persist(extract({ ...get(), verificationTolerancePct }));
      return { verificationTolerancePct };
    }),
}));

function extract(s: Settings & Record<string, unknown>): Settings {
  return {
    theme: s.theme,
    language: s.language,
    recentFiles: s.recentFiles,
    verificationTolerancePct: s.verificationTolerancePct,
  };
}
