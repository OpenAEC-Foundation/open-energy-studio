/**
 * Settings (UI redesign F9): Algemeen (theme, language), Berekening (the
 * live preview run of the open document and the edition new calculations
 * start in) and Over (program identity and attest status). Nothing is
 * committed before OK; the theme previews live and Cancel reverts it.
 */
import { useId, useRef, useState, type KeyboardEvent, type ReactNode } from 'react';
import { Calculator, Info, SlidersHorizontal } from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import type { Locale } from '../../i18n/i18n';
import { DEFAULT_NORM_VERSION, IMPLEMENTED_NORM_VERSIONS, type NormVersion } from '../../core/nta/KernelClient';
import { readDefaultEdition, writeDefaultEdition } from '../../core/nta/defaultEdition';
import { softwareIdentity } from '../../core/nta/Registration';
import { DialogShell } from '../dialogs/DialogShell';
import { Pill, Select, Switch } from '../ui';
import './SettingsDialog.css';

type SettingsTab = 'general' | 'calculation' | 'about';

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

/** Radio group of option rows (theme, language); arrow keys move the choice. */
function OptionRows<T extends string>({ label, className, rowClassName, options, value, onChange }: {
  label: string;
  className: string;
  rowClassName: string;
  options: { value: T; content: ReactNode }[];
  value: T;
  onChange: (value: T) => void;
}) {
  const refs = useRef<Record<string, HTMLButtonElement | null>>({});
  const onKeyDown = (event: KeyboardEvent<HTMLButtonElement>, index: number) => {
    const step = event.key === 'ArrowDown' || event.key === 'ArrowRight' ? 1
      : event.key === 'ArrowUp' || event.key === 'ArrowLeft' ? -1 : 0;
    if (!step) return;
    event.preventDefault();
    const next = options[(index + step + options.length) % options.length];
    onChange(next.value);
    refs.current[next.value]?.focus();
  };
  return (
    <div className={className} role="radiogroup" aria-label={label}>
      {options.map((option, index) => {
        const checked = option.value === value;
        return (
          <button key={option.value} type="button" role="radio" aria-checked={checked} tabIndex={checked ? 0 : -1}
            ref={(element) => { refs.current[option.value] = element; }}
            className={`${rowClassName}${checked ? ' active' : ''}`}
            onClick={() => onChange(option.value)} onKeyDown={(event) => onKeyDown(event, index)}>
            {option.content}
          </button>
        );
      })}
    </div>
  );
}

export function SettingsDialog({ onClose, previewSetting }: SettingsDialogProps) {
  const { t, locale, setLocale } = useI18n();
  const [activeTab, setActiveTab] = useState<SettingsTab>('general');
  const prefix = useId();
  const tabRefs = useRef<Record<string, HTMLButtonElement | null>>({});

  // Draft state — only committed on OK
  const storedTheme = (localStorage.getItem('energy-theme') || 'dark') as Theme;
  const originalTheme = useRef(document.documentElement.dataset.theme || 'dark');
  const [draftLocale, setDraftLocale] = useState<Locale>(locale);
  const [draftTheme, setDraftTheme] = useState<Theme>(storedTheme);
  const [draftPreview, setDraftPreview] = useState(previewSetting?.enabled ?? true);
  const [draftEdition, setDraftEdition] = useState<NormVersion>(readDefaultEdition);

  const resolveTheme = (theme: Theme) =>
    theme === 'system'
      ? window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
      : theme;

  const handleThemeChange = (theme: Theme) => {
    setDraftTheme(theme);
    document.documentElement.dataset.theme = resolveTheme(theme); // preview live
  };

  const handleOk = () => {
    // Commit: apply language and persist all
    setLocale(draftLocale);
    localStorage.setItem('energy-theme', draftTheme);
    localStorage.setItem('energy-locale', draftLocale);
    writeDefaultEdition(draftEdition);
    if (previewSetting && draftPreview !== previewSetting.enabled) previewSetting.onChange(draftPreview);
    onClose();
  };

  const handleCancel = () => {
    // Revert theme preview (language was never changed)
    document.documentElement.dataset.theme = originalTheme.current;
    onClose();
  };

  const tabs: { id: SettingsTab; label: string; icon: ReactNode }[] = [
    { id: 'general', label: t('settings.general'), icon: <SlidersHorizontal aria-hidden="true" /> },
    { id: 'calculation', label: t('settings.calculation'), icon: <Calculator aria-hidden="true" /> },
    { id: 'about', label: t('settings.about'), icon: <Info aria-hidden="true" /> },
  ];

  const onTabKey = (event: KeyboardEvent<HTMLButtonElement>, index: number) => {
    const step = event.key === 'ArrowDown' ? 1 : event.key === 'ArrowUp' ? -1 : 0;
    if (!step) return;
    event.preventDefault();
    const next = tabs[(index + step + tabs.length) % tabs.length];
    setActiveTab(next.id);
    tabRefs.current[next.id]?.focus();
  };

  const software = softwareIdentity();

  const settingsFooter = (
    <div className="dialog-footer">
      <button type="button" className="btn" onClick={handleCancel}>
        {t('dialog.cancel')}
      </button>
      <button type="button" className="btn btn-primary" onClick={handleOk}>
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
      <div className="settings-sidebar" role="tablist" aria-orientation="vertical" aria-label={t('settings.title')}>
        {tabs.map((tab, index) => (
          <button
            key={tab.id}
            ref={(element) => { tabRefs.current[tab.id] = element; }}
            type="button"
            role="tab"
            id={`${prefix}-tab-${tab.id}`}
            aria-selected={activeTab === tab.id}
            aria-controls={`${prefix}-panel`}
            tabIndex={activeTab === tab.id ? 0 : -1}
            className={`settings-tab${activeTab === tab.id ? ' active' : ''}`}
            onClick={() => setActiveTab(tab.id)}
            onKeyDown={(event) => onTabKey(event, index)}
          >
            {tab.icon}
            <span>{tab.label}</span>
          </button>
        ))}
      </div>

      <div className="settings-content" role="tabpanel" id={`${prefix}-panel`} aria-labelledby={`${prefix}-tab-${activeTab}`}>
        {activeTab === 'general' && (<>
          <div className="settings-section">
            <h3 className="settings-section-title">{t('settings.general.theme')}</h3>
            <OptionRows label={t('settings.general.theme')} className="theme-table" rowClassName="theme-row"
              value={draftTheme} onChange={handleThemeChange}
              options={THEMES.map((theme) => ({
                value: theme.value,
                content: <>
                  <span className="theme-row-swatches" aria-hidden="true">
                    {theme.swatches.map((color, i) => (
                      <span key={i} className="theme-row-swatch" style={{ background: color }} />
                    ))}
                  </span>
                  <span className="theme-row-name">{t(theme.labelKey)}</span>
                </>,
              }))} />
          </div>
          <div className="settings-section">
            <h3 className="settings-section-title">{t('settings.language.select')}</h3>
            <OptionRows label={t('settings.language.select')} className="language-table" rowClassName="language-row"
              value={draftLocale} onChange={setDraftLocale}
              options={LANGUAGES.map((lang) => ({
                value: lang.code,
                content: <>
                  <span className="language-col-code">{lang.code.toUpperCase()}</span>
                  <span className="language-col-name">{t(`language.${lang.code}`)}</span>
                </>,
              }))} />
            <p className="settings-hint">{t('settings.language.hint')}</p>
          </div>
        </>)}

        {activeTab === 'calculation' && (<>
          <div className="settings-section">
            <h3 className="settings-section-title">{t('settings.inspector.title')}</h3>
            <Switch label={t('settings.inspector.preview')} checked={draftPreview}
              disabled={!previewSetting} onChange={setDraftPreview} />
            <p className="settings-hint">{previewSetting ? t('settings.inspector.previewHint') : t('settings.inspector.previewNoDocument')}</p>
          </div>
          <div className="settings-section">
            <h3 className="settings-section-title" id={`${prefix}-edition`}>{t('settings.edition.title')}</h3>
            <Select aria-labelledby={`${prefix}-edition`} value={draftEdition}
              onChange={(value) => setDraftEdition(value as NormVersion)}
              options={IMPLEMENTED_NORM_VERSIONS.map((edition) => ({ value: edition, label: t(`nta.edition.${edition}`) }))} />
            <p className="settings-hint">{t('settings.edition.hint')}</p>
            {draftEdition !== DEFAULT_NORM_VERSION && (
              <p className="settings-warning" role="note">{t('nta.form.normVersionLegacy')}</p>
            )}
          </div>
        </>)}

        {activeTab === 'about' && (
          <div className="settings-section">
            <h3 className="settings-section-title">{t('settings.about')}</h3>
            <table className="kv-table settings-about">
              <tbody>
                <tr><td>{t('registration.page.program')}</td><td>{software.name} {software.version}</td></tr>
                <tr><td>{t('registration.page.attestNumber')}</td>
                  <td>{software.attestNumber ?? <Pill tone="unv">{t('status.unattested')}</Pill>}</td></tr>
                <tr><td>{t('settings.edition.current')}</td><td>{t(`nta.edition.${DEFAULT_NORM_VERSION}`)}</td></tr>
              </tbody>
            </table>
            <p className="settings-hint">{t('settings.about.hint')}</p>
          </div>
        )}
      </div>
    </DialogShell>
  );
}
