import i18next from 'i18next';
import { initReactI18next, useTranslation } from 'react-i18next';
import { en } from './en';
import { nl } from './nl';
import { fr } from './fr';
import { es } from './es';
import { zh } from './zh';
import { it } from './it';
import { de } from './de';
import { pt } from './pt';
import { pl } from './pl';
import { tr } from './tr';
import { ja } from './ja';
import { ko } from './ko';
import { ar } from './ar';
import { fa } from './fa';

export type Locale = 'en' | 'nl' | 'fr' | 'es' | 'zh' | 'it' | 'de' | 'pt' | 'pl' | 'tr' | 'ja' | 'ko' | 'ar' | 'fa';

const VALID_LOCALES: Locale[] = ['en', 'nl', 'fr', 'es', 'zh', 'it', 'de', 'pt', 'pl', 'tr', 'ja', 'ko', 'ar', 'fa'];
const RTL_LOCALES: Locale[] = ['ar', 'fa'];

function applyDirection(locale: Locale) {
  document.documentElement.dir = RTL_LOCALES.includes(locale) ? 'rtl' : 'ltr';
}

function detectOSLocale(): Locale {
  // navigator.languages gives preferred languages in order, navigator.language is the primary
  const candidates = [...(navigator.languages || []), navigator.language].filter(Boolean);
  for (const tag of candidates) {
    const code = tag.split('-')[0].toLowerCase();
    if (VALID_LOCALES.includes(code as Locale)) return code as Locale;
  }
  return 'en';
}

function getStoredLocale(): Locale {
  const stored = localStorage.getItem('energy-locale');
  if (stored && VALID_LOCALES.includes(stored as Locale)) return stored as Locale;
  return detectOSLocale();
}

i18next.use(initReactI18next).init({
  resources: {
    en: { translation: en },
    nl: { translation: nl },
    fr: { translation: fr },
    es: { translation: es },
    zh: { translation: zh },
    it: { translation: it },
    de: { translation: de },
    pt: { translation: pt },
    pl: { translation: pl },
    tr: { translation: tr },
    ja: { translation: ja },
    ko: { translation: ko },
    ar: { translation: ar },
    fa: { translation: fa },
  },
  lng: getStoredLocale(),
  fallbackLng: 'en',
  interpolation: {
    escapeValue: false,
  },
  react: {
    useSuspense: false,
  },
});

// Set initial text direction based on stored locale
applyDirection(getStoredLocale());

/** Drop-in replacement hook — same API as before */
export function useI18n() {
  const { t, i18n } = useTranslation();

  const setLocale = (newLocale: Locale) => {
    i18n.changeLanguage(newLocale);
    localStorage.setItem('energy-locale', newLocale);
    applyDirection(newLocale);
  };

  return {
    t: t as (key: string) => string,
    locale: i18n.language as Locale,
    setLocale,
  };
}

export default i18next;
