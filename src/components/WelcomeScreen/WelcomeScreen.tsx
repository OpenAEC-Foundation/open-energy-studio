/**
 * Start screen without an open document (UI redesign F9): new dwelling or
 * utility project, open and import, the recent project files and the two
 * fictional examples.
 */
import { Building2, Clock, FileInput, FolderOpen, Home, X } from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import { EXAMPLE_KINDS, type ExampleKind } from '../../core/nta/ExampleProjects';
import type { RecentProject } from '../../core/io/recentProjects';
import { IconButton } from '../ui';
import { labelClassName } from '../shell/labelClass';
import './WelcomeScreen.css';

/** New-build calculation, or a basisopname of an existing dwelling or utility building. */
export type NewProjectKind = 'residential' | 'utility' | 'existing_residential' | 'existing_utility';

interface WelcomeScreenProps {
  onNewProject: (kind: NewProjectKind) => void;
  onOpenProject: () => void;
  onOpenExample: (kind: ExampleKind) => void;
  onImportUNIEC3?: () => void;
  onImportVABI?: () => void;
  recent?: RecentProject[];
  onOpenRecent?: (path: string) => void;
  onForgetRecent?: (path: string) => void;
}

/** The file name of a path, for the recent list (the full path is the title). */
function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

export function WelcomeScreen({
  onNewProject, onOpenProject, onOpenExample, onImportUNIEC3, onImportVABI, recent = [], onOpenRecent, onForgetRecent,
}: WelcomeScreenProps) {
  const { t, locale } = useI18n();
  const date = (iso: string) => {
    const value = new Date(iso);
    return Number.isNaN(value.getTime()) ? '' : value.toLocaleDateString(locale, { day: 'numeric', month: 'short', year: 'numeric' });
  };

  return (
    <div className="welcome-screen">
      <div className="welcome-content">
        <header className="welcome-hero">
          <img src="/icon.png" alt="" className="welcome-icon" />
          <div>
            <h1 className="welcome-title">{t('app.title')}</h1>
            <p className="welcome-subtitle">{t('welcome.subtitle')}</p>
          </div>
        </header>

        <div className="welcome-grid">
          <section className="welcome-panel" aria-labelledby="welcome-start">
            <h2 className="welcome-heading" id="welcome-start">{t('welcome.start')}</h2>
            <div className="welcome-new">
              <button type="button" className="welcome-card welcome-card--primary" onClick={() => onNewProject('existing_residential')}>
                <Home aria-hidden="true" />
                <span className="welcome-card__text">
                  <strong>{t('welcome.existingResidential')}</strong>
                  <span>{t('welcome.existingResidentialHint')}</span>
                </span>
              </button>
              <button type="button" className="welcome-card" onClick={() => onNewProject('existing_utility')}>
                <Building2 aria-hidden="true" />
                <span className="welcome-card__text">
                  <strong>{t('welcome.existingUtility')}</strong>
                  <span>{t('welcome.existingUtilityHint')}</span>
                </span>
              </button>
              <button type="button" className="welcome-card" onClick={() => onNewProject('residential')}>
                <Home aria-hidden="true" />
                <span className="welcome-card__text">
                  <strong>{t('welcome.newResidential')}</strong>
                  <span>{t('welcome.newResidentialHint')}</span>
                </span>
              </button>
              <button type="button" className="welcome-card" onClick={() => onNewProject('utility')}>
                <Building2 aria-hidden="true" />
                <span className="welcome-card__text">
                  <strong>{t('welcome.newUtility')}</strong>
                  <span>{t('welcome.newUtilityHint')}</span>
                </span>
              </button>
            </div>
            <div className="welcome-actions">
              <button type="button" className="btn welcome-btn" onClick={onOpenProject}>
                <FolderOpen aria-hidden="true" />{t('ribbon.open')}
              </button>
              {onImportUNIEC3 && (
                <button type="button" className="btn welcome-btn" onClick={onImportUNIEC3}>
                  <FileInput aria-hidden="true" />{t('ribbon.importUNIEC3')}
                </button>
              )}
              {onImportVABI && (
                <button type="button" className="btn welcome-btn" onClick={onImportVABI}>
                  <FileInput aria-hidden="true" />{t('ribbon.importVABI')}
                </button>
              )}
            </div>

            <h2 className="welcome-heading">{t('welcome.examples')}</h2>
            <div className="welcome-examples">
              {EXAMPLE_KINDS.map((kind) => (
                <button key={kind} type="button" className="btn welcome-btn welcome-example" onClick={() => onOpenExample(kind)}>
                  {t(`welcome.example.${kind}`)}
                </button>
              ))}
            </div>
            <p className="welcome-examples-note">{t('welcome.examplesNote')}</p>
          </section>

          <section className="welcome-panel" aria-labelledby="welcome-recent">
            <h2 className="welcome-heading" id="welcome-recent">{t('welcome.recent')}</h2>
            {recent.length === 0 ? (
              <p className="welcome-empty"><Clock aria-hidden="true" />{t('welcome.recentEmpty')}</p>
            ) : (
              <ul className="welcome-recent">
                {recent.map((entry) => (
                  <li key={entry.path} className="welcome-recent__item">
                    <button type="button" className="welcome-recent__open" title={entry.path}
                      onClick={() => onOpenRecent?.(entry.path)}>
                      <span className="welcome-recent__name">
                        {entry.labelClass && <span className={`label-badge label-badge--sm ${labelClassName(entry.labelClass)}`}
                          title={t('overview.labelClass')}>{entry.labelClass}</span>}
                        {entry.name || fileName(entry.path)}
                      </span>
                      <span className="welcome-recent__meta">
                        {entry.buildingFunction && <span>{t(`function.${entry.buildingFunction}`)}</span>}
                        <span>{date(entry.at)}</span>
                      </span>
                      <span className="welcome-recent__path">{entry.path}</span>
                    </button>
                    {onForgetRecent && (
                      <IconButton size="sm" icon={<X aria-hidden="true" />}
                        aria-label={t('welcome.recentForget', { name: entry.name || fileName(entry.path) })}
                        onClick={() => onForgetRecent(entry.path)} />
                    )}
                  </li>
                ))}
              </ul>
            )}
          </section>
        </div>
      </div>
    </div>
  );
}
