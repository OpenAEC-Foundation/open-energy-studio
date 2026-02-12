import { createContext, useContext } from 'react';
import { en } from './en';
import { nl } from './nl';

export type Locale = 'en' | 'nl';
export type TranslationKeys = typeof en;

const translations: Record<Locale, TranslationKeys> = { en, nl };

export interface I18nContextType {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: string) => string;
}

export const I18nContext = createContext<I18nContextType>({
  locale: 'nl',
  setLocale: () => {},
  t: (key: string) => key,
});

export function useI18n(): I18nContextType {
  return useContext(I18nContext);
}

export function getTranslation(locale: Locale, key: string): string {
  const dict = translations[locale] as Record<string, string>;
  return dict[key] ?? key;
}

export function getStoredLocale(): Locale {
  const stored = localStorage.getItem('energy-locale');
  if (stored === 'en' || stored === 'nl') return stored;
  return 'nl';
}
