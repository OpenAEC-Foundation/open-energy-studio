import { useState, useRef } from 'react';
import { useI18n } from '../../i18n/i18n';
import type { Locale } from '../../i18n/i18n';
import { DialogShell } from '../dialogs/DialogShell';
import './SettingsDialog.css';

type SettingsTab = 'general' | 'language';

type Theme = 'system' | 'light' | 'dark' | 'highContrast';

const THEMES: { value: Theme; labelKey: string; swatches: string[] }[] = [
  { value: 'system', labelKey: 'theme.system', swatches: ['#36363E', '#2A2A32', '#D97706', '#FAFAF9'] },
  { value: 'dark', labelKey: 'theme.dark', swatches: ['#2A2A32', '#36363E', '#D97706', '#FAFAF9'] },
  { value: 'light', labelKey: 'theme.light', swatches: ['#FAFAF9', '#F5F5F4', '#D97706', '#36363E'] },
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
  /**
   * The live preview run of the active document (the inspector's Voorbeeld,
   * formerly the ribbon Preview toggle); absent without an open document.
   */
  previewSetting?: { enabled: boolean; onChange: (enabled: boolean) => void };
}

export function SettingsDialog({ onClose, previewSetting }: SettingsDialogProps) {
  const { t, locale, setLocale } = useI18n();
  const [activeTab, setActiveTab] = useState<SettingsTab>('general');

  // Draft state — only committed on OK
  const storedTheme = (localStorage.getItem('energy-theme') || 'dark') as Theme;
  const originalTheme = useRef(document.documentElement.dataset.theme || 'dark');
  const [draftLocale, setDraftLocale] = useState<Locale>(locale);
  const [draftTheme, setDraftTheme] = useState<Theme>(storedTheme);
  const [draftPreview, setDraftPreview] = useState(previewSetting?.enabled ?? true);

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
    if (previewSetting && draftPreview !== previewSetting.enabled) previewSetting.onChange(draftPreview);
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
        {activeTab === 'general' && (<>
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
          {previewSetting && (
            <div className="settings-section">
              <h3 className="settings-section-title">{t('settings.inspector.title')}</h3>
              <label className="settings-check">
                <input type="checkbox" checked={draftPreview} onChange={(event) => setDraftPreview(event.target.checked)} />
                <span>{t('settings.inspector.preview')}</span>
              </label>
              <p className="settings-hint">{t('settings.inspector.previewHint')}</p>
            </div>
          )}
        </>)}

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
