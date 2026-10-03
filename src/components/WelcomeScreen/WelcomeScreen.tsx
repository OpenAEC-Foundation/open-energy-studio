import { useI18n } from '../../i18n/i18n';
import { EXAMPLE_KINDS, type ExampleKind } from '../../core/nta/ExampleProjects';
import './WelcomeScreen.css';

interface WelcomeScreenProps {
  onNewProject: () => void;
  onOpenProject: () => void;
  onOpenExample: (kind: ExampleKind) => void;
}

export function WelcomeScreen({ onNewProject, onOpenProject, onOpenExample }: WelcomeScreenProps) {
  const { t, locale } = useI18n();

  return (
    <div className="welcome-screen">
      <div className="welcome-content">
        <img src="/icon.png" alt="" className="welcome-icon" />
        <h1 className="welcome-title">{t('app.title')}</h1>
        <p className="welcome-subtitle">{t('welcome.subtitle')}</p>
        <div className="welcome-actions">
          <button className="btn btn-primary welcome-btn" onClick={onNewProject}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
              <line x1="12" y1="18" x2="12" y2="12" />
              <line x1="9" y1="15" x2="15" y2="15" />
            </svg>
            {t('app.newProject')}
          </button>
          <button className="btn welcome-btn" onClick={onOpenProject}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
            </svg>
            {t('ribbon.open')}
          </button>
        </div>
        <div className="welcome-examples">
          <p className="welcome-examples-title">{t('welcome.examples')}</p>
          {EXAMPLE_KINDS.map((kind) => (
            <button key={kind} className="btn welcome-btn" onClick={() => onOpenExample(kind)}>
              {t(`welcome.example.${kind}`)}
            </button>
          ))}
          <p className="welcome-examples-note">{locale === 'nl'
            ? 'Fictieve oefenprojecten. BENG en labelklasse zijn indicatief; geen geregistreerd energielabel.'
            : 'Fictional practice projects. BENG and label class are indicative; no registered energy label.'}</p>
        </div>
      </div>
    </div>
  );
}
