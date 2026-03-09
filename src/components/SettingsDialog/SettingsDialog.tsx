import { useState, useRef } from 'react';
import { useI18n } from '../../i18n/i18n';
import type { Locale } from '../../i18n/i18n';
import { DialogShell } from '../dialogs/DialogShell';
import './SettingsDialog.css';

type SettingsTab = 'general' | 'language';

type Theme = 'system' | 'light' | 'dark' | 'blue' | 'highContrast';

const THEMES: { value: Theme; labelKey: string; swatches: string[] }[] = [
  { value: 'system', labelKey: 'theme.system', swatches: ['#161b22', '#1c2333', '#3b82f6', '#e6edf3'] },
  { value: 'dark', labelKey: 'theme.dark', swatches: ['#0d1117', '#161b22', '#3b82f6', '#e6edf3'] },
  { value: 'light', labelKey: 'theme.light', swatches: ['#f5f5f7', '#ffffff', '#3b82f6', '#1a1a2e'] },
  { value: 'blue', labelKey: 'theme.blue', swatches: ['#0d1b2a', '#1b263b', '#00b4d8', '#e0e1dd'] },
  { value: 'highContrast', labelKey: 'theme.highContrast', swatches: ['#000000', '#0a0a0a', '#ffff00', '#ffffff'] },
];

const LANGUAGES: { code: Locale; region: string }[] = [
  { code: 'ar', region: 'SA' },
  { code: 'de', region: 'DE' },
  { code: 'en', region: 'GB' },
  { code: 'es', region: 'ES' },
  { code: 'fa', region: 'IR' },
  { code: 'fr', region: 'FR' },
  { code: 'it', region: 'IT' },
  { code: 'ja', region: 'JP' },
  { code: 'ko', region: 'KR' },
  { code: 'nl', region: 'NL' },
  { code: 'pl', region: 'PL' },
  { code: 'pt', region: 'PT' },
  { code: 'tr', region: 'TR' },
  { code: 'zh', region: 'CN' },
];

interface SettingsDialogProps {
  onClose: () => void;
}

export function SettingsDialog({ onClose }: SettingsDialogProps) {
  const { t, locale, setLocale } = useI18n();
  const [activeTab, setActiveTab] = useState<SettingsTab>('general');

  // Draft state — only committed on OK
  const storedTheme = (localStorage.getItem('energy-theme') || 'dark') as Theme;
  const originalTheme = useRef(document.documentElement.dataset.theme || 'dark');
  const [draftLocale, setDraftLocale] = useState<Locale>(locale);
  const [draftTheme, setDraftTheme] = useState<Theme>(storedTheme);

  const resolveTheme = (theme: Theme) =>
    theme === 'system'
      ? window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
      : theme;

  const handleThemeChange = (theme: Theme) => {
    setDraftTheme(theme);
    document.documentElement.dataset.theme = resolveTheme(theme); // preview live
  };

  const handleOk = () => {
    // Commit: apply language and persist both
    setLocale(draftLocale);
    localStorage.setItem('energy-theme', draftTheme);
    localStorage.setItem('energy-locale', draftLocale);
    onClose();
  };

  const handleCancel = () => {
    // Revert theme preview (language was never changed)
    document.documentElement.dataset.theme = originalTheme.current;
    onClose();
  };

  const tabs: { id: SettingsTab; label: string; icon: React.ReactNode }[] = [
    {
      id: 'general',
      label: t('settings.general'),
      icon: (
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
        </svg>
      ),
    },
    {
      id: 'language',
      label: t('settings.language'),
      icon: (
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <circle cx="12" cy="12" r="10" />
          <line x1="2" y1="12" x2="22" y2="12" />
          <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
        </svg>
      ),
    },
  ];

  const settingsFooter = (
    <div className="dialog-footer">
      <button className="btn" onClick={handleCancel}>
        {t('dialog.cancel')}
      </button>
      <button className="btn btn-primary" onClick={handleOk}>
        OK
      </button>
    </div>
  );

  return (
    <DialogShell
      title={t('settings.title')}
      onClose={handleCancel}
      className="settings-dialog"
      bodyClassName="settings-body"
      footer={settingsFooter}
    >
      <div className="settings-sidebar">
        {tabs.map(tab => (
          <button
            key={tab.id}
            className={`settings-tab${activeTab === tab.id ? ' active' : ''}`}
            onClick={() => setActiveTab(tab.id)}
          >
            {tab.icon}
            <span>{tab.label}</span>
          </button>
        ))}
      </div>

      <div className="settings-content">
        {activeTab === 'general' && (
          <div className="settings-section">
            <h3 className="settings-section-title">{t('settings.general.theme')}</h3>
            <div className="theme-table">
              {THEMES.map(theme => (
                <button
                  key={theme.value}
                  className={`theme-row${draftTheme === theme.value ? ' active' : ''}`}
                  onClick={() => handleThemeChange(theme.value)}
                >
                  <span className="theme-row-swatches">
                    {theme.swatches.map((color, i) => (
                      <span key={i} className="theme-row-swatch" style={{ background: color }} />
                    ))}
                  </span>
                  <span className="theme-row-name">{t(theme.labelKey)}</span>
                </button>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'language' && (
          <div className="settings-section">
            <h3 className="settings-section-title">{t('settings.language.select')}</h3>
            <div className="language-table">
              <div className="language-table-header">
                <span className="language-col-code">{t('properties.type')}</span>
                <span className="language-col-name">{t('settings.language')}</span>
              </div>
              {LANGUAGES.map(lang => (
                <button
                  key={lang.code}
                  className={`language-row${draftLocale === lang.code ? ' active' : ''}`}
                  onClick={() => setDraftLocale(lang.code)}
                >
                  <span className="language-col-code">{lang.code.toUpperCase()}</span>
                  <span className="language-col-name">{t(`language.${lang.code}`)}</span>
                </button>
              ))}
            </div>
          </div>
        )}
      </div>
    </DialogShell>
  );
}
