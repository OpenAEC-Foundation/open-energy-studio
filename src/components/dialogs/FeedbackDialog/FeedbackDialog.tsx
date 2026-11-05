/**
 * Feedback dialog, the same flow as the one on open-aec.com
 * (website/shared/feedback.js): a report goes to Open Feedback Studio, whose
 * /api/feedback opens a GitHub issue in the repository mapped to the app id.
 * Here the app is always Open Energy Studio, so every report becomes an issue
 * in OpenAEC-Foundation/open-energy-studio. The report is public; the email
 * address is optional and does not go into the issue.
 */
import { useId, useState } from 'react';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { DialogShell } from '../DialogShell';
import { useI18n } from '../../../i18n/i18n';
import { useKernel } from '../../../context/KernelProvider';
import { MANUAL_KERNEL_VERSION } from '../../../core/manual/manual-version';
import { version } from '../../../../package.json';
import './FeedbackDialog.css';

export const FEEDBACK_API = 'https://open-feedback-studio.pages.dev/api/feedback';
export const FEEDBACK_APP_ID = 'open-energy-studio';
export const FEEDBACK_REPO = 'OpenAEC-Foundation/open-energy-studio';
const MAX_SUBJECT = 120;
const MAX_TEXT = 4500;
const MIN_TOTAL = 10;

type Category = 'bug' | 'feature' | 'general';
type Status = 'idle' | 'submitting' | 'success';

/** The issue number of a GitHub issue URL of this repository, or null. */
export function issueNumber(url: string | null | undefined): number | null {
  const match = /^https:\/\/github\.com\/OpenAEC-Foundation\/open-energy-studio\/issues\/(\d+)$/.exec(url ?? '');
  return match ? Number(match[1]) : null;
}

interface FeedbackDialogProps {
  onClose: () => void;
}

export function FeedbackDialog({ onClose }: FeedbackDialogProps) {
  const fieldId = useId();
  const { t, locale } = useI18n();
  const kernelVersion = useKernel()?.settled?.kernelVersion ?? MANUAL_KERNEL_VERSION;
  const desktop = isTauri();

  const [category, setCategory] = useState<Category>('bug');
  const [subject, setSubject] = useState('');
  const [text, setText] = useState('');
  const [email, setEmail] = useState(() => {
    try { return localStorage.getItem('feedback-email') ?? ''; } catch { return ''; }
  });
  const [status, setStatus] = useState<Status>('idle');
  const [error, setError] = useState('');
  const [issueUrl, setIssueUrl] = useState<string | null>(null);

  // What goes with the report, so the makers can reproduce it.
  const context = [
    `Open Energy Studio ${version}`,
    kernelVersion ? `${t('feedback.kernel')} ${kernelVersion}` : '',
    desktop ? 'desktop' : 'web',
    locale,
  ].filter(Boolean).join(' · ');

  const problem = (() => {
    const s = subject.trim();
    const d = text.trim();
    if (!s) return t('feedback.needTitle');
    if ((s + d).length < MIN_TOTAL) return t('feedback.needText');
    if (email.trim() && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email.trim())) return t('feedback.badMail');
    return '';
  })();
  const touched = Boolean(subject || text);

  async function submit() {
    if (problem || status === 'submitting') return;
    setStatus('submitting');
    setError('');
    const fields: Record<string, string> = {
      app: FEEDBACK_APP_ID,
      category,
      message: `${subject.trim()}\n\n${text.trim()}\n\n_${t('feedback.via')} ${context}_`,
      appVersion: version,
    };
    if (email.trim()) fields.email = email.trim();
    try {
      const response = await fetch(FEEDBACK_API, {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(fields),
      });
      const body = await response.json().catch(() => ({})) as { success?: boolean; githubIssueUrl?: string };
      if (response.ok && body.success) {
        try { localStorage.setItem('feedback-email', email.trim()); } catch { /* not remembered */ }
        setIssueUrl(issueNumber(body.githubIssueUrl) ? body.githubIssueUrl! : null);
        setStatus('success');
        return;
      }
      setError(t(response.status === 429 ? 'feedback.errorRate' : response.status === 409 ? 'feedback.errorDuplicate'
        : response.status === 400 ? 'feedback.errorRejected' : 'feedback.errorGeneric'));
    } catch {
      setError(t('feedback.errorGeneric'));
    }
    setStatus('idle');
  }

  function openIssue(event: React.MouseEvent) {
    const number = issueNumber(issueUrl);
    // The desktop app opens only pages of this repository in the browser (Tauri command).
    if (desktop && number) {
      event.preventDefault();
      void invoke('open_issue_page', { number });
    }
  }

  function again() {
    setSubject('');
    setText('');
    setCategory('bug');
    setIssueUrl(null);
    setError('');
    setStatus('idle');
  }

  const categories: Array<{ key: Category; label: string }> = [
    { key: 'bug', label: t('feedback.categoryBug') },
    { key: 'feature', label: t('feedback.categoryFeature') },
    { key: 'general', label: t('feedback.categoryGeneral') },
  ];

  return (
    <DialogShell title={t('feedback.title')} onClose={onClose} className="feedback-dialog" footer={null}>
      {status === 'success' ? (
        <div className="feedback-success" role="status">
          <h3>{t('feedback.successTitle')}</h3>
          <p>{issueUrl ? t('feedback.madeIssue', { repo: FEEDBACK_REPO }) : t('feedback.stored')}</p>
          <div className="feedback-actions">
            {issueUrl && (
              <a className="btn btn-primary" href={issueUrl} target="_blank" rel="noopener noreferrer" onClick={openIssue}>
                {t('feedback.onGithub')}
              </a>
            )}
            <button type="button" className="btn" onClick={again}>{t('feedback.sendAnother')}</button>
            <button type="button" className="btn" onClick={onClose}>{t('dialog.close')}</button>
          </div>
        </div>
      ) : (
        <form className="feedback-form" noValidate onSubmit={(event) => { event.preventDefault(); void submit(); }}>
          <div className="feedback-section">
            <span className="feedback-legend" id={`${fieldId}-kind`}>{t('feedback.kind')}</span>
            <div className="feedback-categories" role="group" aria-labelledby={`${fieldId}-kind`}>
              {categories.map((item) => (
                <button key={item.key} type="button" aria-pressed={category === item.key}
                  className={`feedback-category-btn${category === item.key ? ' active' : ''}`}
                  onClick={() => setCategory(item.key)}>{item.label}</button>
              ))}
            </div>
          </div>
          <div className="dialog-field">
            <label htmlFor={`${fieldId}-subject`}>{t('feedback.subject')}</label>
            <input id={`${fieldId}-subject`} type="text" maxLength={MAX_SUBJECT} value={subject}
              placeholder={t('feedback.subjectPlaceholder')} onChange={(event) => setSubject(event.target.value)} />
          </div>
          <div className="dialog-field">
            <label htmlFor={`${fieldId}-text`}>{t('feedback.message')}</label>
            <textarea id={`${fieldId}-text`} className="feedback-message" maxLength={MAX_TEXT} value={text}
              placeholder={t('feedback.messagePlaceholder')} onChange={(event) => setText(event.target.value)} />
          </div>
          <div className="dialog-field">
            <label htmlFor={`${fieldId}-mail`}>{t('feedback.email')} <span className="feedback-optional">{t('feedback.emailOptional')}</span></label>
            <input id={`${fieldId}-mail`} type="email" autoComplete="email" value={email} onChange={(event) => setEmail(event.target.value)} />
          </div>
          <p className="feedback-note">{t('feedback.publicNote', { repo: FEEDBACK_REPO })}</p>
          <p className="feedback-meta">{t('feedback.included', { context })}</p>
          <p className="feedback-hint" aria-live="polite">{error || (touched ? problem : '')}</p>
          <div className="feedback-actions">
            <button type="button" className="btn" onClick={onClose}>{t('dialog.cancel')}</button>
            <button type="submit" className="btn btn-primary feedback-submit-btn" disabled={Boolean(problem) || status === 'submitting'}>
              {status === 'submitting' ? t('feedback.submitting') : t('feedback.submit')}
            </button>
          </div>
        </form>
      )}
    </DialogShell>
  );
}
