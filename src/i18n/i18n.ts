import i18next from 'i18next';
import { initReactI18next, useTranslation } from 'react-i18next';
import { en } from './en';
import { nl } from './nl';

export type Locale = 'en' | 'nl' | 'fr' | 'es' | 'zh' | 'it' | 'de' | 'pt' | 'pl' | 'tr' | 'ja' | 'ko' | 'ar' | 'fa';

const VALID_LOCALES: Locale[] = ['en', 'nl', 'fr', 'es', 'zh', 'it', 'de', 'pt', 'pl', 'tr', 'ja', 'ko', 'ar', 'fa'];
const RTL_LOCALES: Locale[] = ['ar', 'fa'];

/**
 * Dutch and English are always bundled (English is the fallback); the other
 * languages are loaded on first use so they stay out of the start-up bundle (F10).
 */
const LOADERS: Partial<Record<Locale, () => Promise<Record<string, string>>>> = {
  fr: () => import('./fr').then((m) => m.fr),
  es: () => import('./es').then((m) => m.es),
  zh: () => import('./zh').then((m) => m.zh),
  it: () => import('./it').then((m) => m.it),
  de: () => import('./de').then((m) => m.de),
  pt: () => import('./pt').then((m) => m.pt),
  pl: () => import('./pl').then((m) => m.pl),
  tr: () => import('./tr').then((m) => m.tr),
  ja: () => import('./ja').then((m) => m.ja),
  ko: () => import('./ko').then((m) => m.ko),
  ar: () => import('./ar').then((m) => m.ar),
  fa: () => import('./fa').then((m) => m.fa),
};

/** Make sure the translations of `locale` are registered before switching to it. */
async function ensureLocale(locale: Locale): Promise<void> {
  const load = LOADERS[locale];
  if (!load || i18next.hasResourceBundle(locale, 'translation')) return;
  i18next.addResourceBundle(locale, 'translation', await load());
}

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
// A stored locale other than nl/en: load it and re-render once it arrives (English meanwhile).
if (LOADERS[getStoredLocale()]) {
  void ensureLocale(getStoredLocale()).then(() => i18next.changeLanguage(getStoredLocale()));
}

/** Drop-in replacement hook — same API as before */
export function useI18n() {
  const { t, i18n } = useTranslation();

  const setLocale = (newLocale: Locale) => {
    localStorage.setItem('energy-locale', newLocale);
    applyDirection(newLocale);
    void ensureLocale(newLocale).then(() => i18n.changeLanguage(newLocale));
  };

  return {
    t: t as (key: string, options?: Record<string, unknown>) => string,
    locale: i18n.language as Locale,
    setLocale,
  };
}

export default i18next;
