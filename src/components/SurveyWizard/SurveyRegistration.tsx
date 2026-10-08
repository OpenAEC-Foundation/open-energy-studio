/**
 * Registratie of a basisopname project (feedback 8 Oct 2026): a checklist
 * "Klaar voor registratie?" with a way to each open point, only the fields the
 * survey did not ask yet, and what the survey already answered shown as taken
 * over (`surveyRegistration`). Every change is saved at once.
 */
import { useEffect, useRef, useState } from 'react';
import { CheckCircle2, CircleX, Clock } from 'lucide-react';
import './SurveyWizard.css';
import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import { assessSurvey, currentResult, useSurveyAssessment } from '../../core/survey/surveyAssessment';
import {
  registrationChecklist, surveyRegistration, takeoversPending, validUntil, type ChecklistItem,
} from '../../core/survey/surveyRegistration';
import { softwareIdentity } from '../../core/nta/Registration';
import type { NtaRegistration } from '../../core/nta/KernelClient';
import type { StoredSurvey } from '../../core/nta/SurveyTemplates';
import type { Route } from '../../core/navigation/routes';
import { EvidenceRegister } from '../dialogs/ProjectInfoDialog/EvidenceRegister';
import type { ShellActions } from '../shell/ShellActions';

type FieldKey = 'registrationDate' | 'replacedEpOnlineNumber';

const ICON = { done: CheckCircle2, open: CircleX, waiting: Clock };

export function SurveyRegistration({ actions }: { actions: Pick<ShellActions, 'navigate'> }) {
  const { t, locale } = useI18n();
  const { state, dispatch } = useEnergy();
  const project = state.project;
  const stored = project.basisopname as (StoredSurvey & { surveyDate?: string }) | undefined;
  const assessment = useSurveyAssessment();
  const surveyJson = stored ? JSON.stringify(stored.survey) : null;
  const evidenceRef = useRef<HTMLDetailsElement>(null);
  const [evidenceOpen, setEvidenceOpen] = useState(false);

  useEffect(() => {
    if (stored) void assessSurvey(stored);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [surveyJson]);
  // The survey's answers are part of the registration (EP-Online export, dossier).
  const pending = takeoversPending(project);
  useEffect(() => {
    if (pending) dispatch({ type: 'UPDATE_PROJECT_INFO', payload: { registration: surveyRegistration(project) } });
  }, [pending, project, dispatch]);

  if (!stored) return null;
  const result = currentResult(assessment, stored);
  const registration = surveyRegistration(project);
  const set = (patch: Partial<NtaRegistration>) =>
    dispatch({ type: 'UPDATE_PROJECT_INFO', payload: { registration: { ...registration, ...patch } } });
  const checklist = registrationChecklist(project, result);
  const open = checklist.filter((item) => item.state === 'open').length;
  const software = softwareIdentity();
  const date = (value: string | undefined | null) => (value ? new Date(value).toLocaleDateString(locale) : '—');
  const surveyStep = stored.kind === 'residential' ? 'woning' : 'gebouw';
  const focus = (key: FieldKey) => {
    const input = document.getElementById(`sreg-${key}`);
    input?.scrollIntoView({ block: 'center' });
    input?.focus();
  };
  const go: Partial<Record<ChecklistItem['id'], () => void>> = {
    label: () => actions.navigate({ step: 'survey', sub: 'label' } as Route),
    address: () => actions.navigate({ step: 'survey', sub: surveyStep, question: 'adres' } as Route),
    adviser: () => actions.navigate({ step: 'survey', sub: surveyStep, question: 'adres' } as Route),
    certificate: () => actions.navigate({ step: 'survey', sub: surveyStep, question: 'adres' } as Route),
    surveyDate: () => actions.navigate({ step: 'survey', sub: surveyStep, question: 'adres' } as Route),
    bag: () => actions.navigate({ step: 'survey', sub: surveyStep, question: 'adres' } as Route),
    client: () => actions.navigate({ step: 'survey', sub: surveyStep, question: 'adres' } as Route),
    registrationDate: () => focus('registrationDate'),
    reasons: () => actions.navigate({ step: 'survey', sub: 'controle' } as Route),
    evidence: () => {
      setEvidenceOpen(true);
      requestAnimationFrame(() => evidenceRef.current?.scrollIntoView({ block: 'start' }));
    },
  };
  const detail = (item: ChecklistItem): string | null => {
    switch (item.id) {
      case 'label': return item.values ? t('surveyReg.detail.label', { label: String(item.values.label) }) : null;
      case 'address': return [project.address, [registration.postcode, project.city].filter(Boolean).join(' ')].filter(Boolean).join(', ') || null;
      case 'adviser': return [registration.surveyingAdvisor?.name, registration.surveyingAdvisor?.competenceNumber].filter(Boolean).join(' · ') || null;
      case 'certificate': return registration.certificateNumber ?? null;
      case 'surveyDate': return registration.surveyDate ? date(registration.surveyDate) : null;
      case 'registrationDate': return registration.registrationDate ? date(registration.registrationDate) : null;
      case 'client': return registration.client ?? null;
      case 'bag': return registration.bagObjectId ?? null;
      case 'reasons': return item.state === 'done'
        ? t('surveyReg.detail.reasonsDone', { total: Number(item.values?.total ?? 0) })
        : t('surveyReg.detail.reasons', { count: Number(item.values?.count ?? 0), total: Number(item.values?.total ?? 0) });
      case 'evidence': return Number(item.values?.count) > 0 ? t('surveyReg.detail.evidence', { count: Number(item.values?.count) }) : t('surveyReg.detail.evidenceNone');
      case 'attest': return software.attestNumber ?? t('surveyReg.detail.attestNone');
      default: return null;
    }
  };
  const field = (key: FieldKey, type: 'text' | 'date' = 'text') => <label key={key}>{t(`surveyReg.field.${key}`)}
    <input id={`sreg-${key}`} type={type} value={registration[key] ?? ''}
      onChange={(event) => set({ [key]: event.target.value || undefined })} />
  </label>;
  const messageType = registration.messageType ?? 'regular';
  const until = validUntil(registration.surveyDate);
  const advisorText = (advisor: NtaRegistration['surveyingAdvisor']) =>
    [advisor?.name, advisor?.competenceNumber].filter(Boolean).join(' · ') || '—';

  return <div className="survey-wizard survey-registration">
    <div className="survey-main">
      <h1 id="page-title" tabIndex={-1}>{t('surveyReg.title')}</h1>
      <p className="survey-lead">{t('surveyReg.lead')}</p>

      <section className="survey-card" aria-labelledby="sreg-check">
        <h2 id="sreg-check" className="survey-overline">{t('surveyReg.checklist')}
          <span className="survey-reg-count">{open === 0 ? t('surveyReg.allDone') : t('surveyReg.open', { count: open })}</span></h2>
        <ul className="survey-reg-list">
          {checklist.map((item) => {
            const Icon = ICON[item.state];
            const text = detail(item);
            const action = go[item.id];
            return <li key={item.id} data-state={item.state}>
              <Icon aria-hidden="true" />
              <span className="survey-reg-text">{t(`surveyReg.item.${item.id}`)}{text && <small>{text}</small>}</span>
              <span className="sr-only">{t(`surveyReg.state.${item.state}`)}</span>
              {item.state !== 'done' && action && <button type="button" className="btn" onClick={action}>
                {t(item.id === 'registrationDate' ? 'surveyReg.fill' : 'surveyReg.goTo')}</button>}
            </li>;
          })}
        </ul>
      </section>

      <section className="survey-card" aria-labelledby="sreg-fields">
        <h2 id="sreg-fields" className="survey-overline">{t('surveyReg.fields')}</h2>
        <div className="nta-form"><div className="nta-form-grid">
          <label>{t('surveyReg.field.representation')}
            <select value={registration.representation ?? ''}
              onChange={(event) => set({ representation: (event.target.value || undefined) as NtaRegistration['representation'] })}>
              <option value="">—</option>
              <option value="unique">{t('reg.representation.unique')}</option>
              <option value="reference">{t('reg.representation.reference')}</option>
              <option value="similar">{t('reg.representation.similar')}</option>
            </select>
          </label>
          {registration.representation === 'similar' && <label>{t('reg.referenceObjectId')}
            <input type="text" value={registration.referenceObjectId ?? ''}
              onChange={(event) => set({ referenceObjectId: event.target.value || undefined })} /></label>}
          <label>{t('surveyReg.field.messageType')}
            <select value={messageType}
              onChange={(event) => set({ messageType: event.target.value as NonNullable<NtaRegistration['messageType']>, relabel: undefined })}>
              <option value="regular">{t('reg.messageType.regular')}</option>
              <option value="relabel">{t('reg.messageType.relabel')}</option>
              <option value="replacement">{t('reg.messageType.replacement')}</option>
            </select>
          </label>
          {messageType === 'relabel' && <p className="nta-form-note">{t('surveyReg.relabelNote')}{' '}
            <button type="button" className="btn" onClick={() => actions.navigate({ step: 'relabel' } as Route)}>{t('surveyReg.toRelabel')}</button></p>}
          {messageType === 'replacement' && field('replacedEpOnlineNumber')}
          {field('registrationDate', 'date')}
        </div></div>
      </section>

      <section className="survey-card" aria-labelledby="sreg-taken">
        <h2 id="sreg-taken" className="survey-overline">{t('surveyReg.takenOver')}</h2>
        <dl className="survey-reg-kv">
          <dt>{t('reg.purpose')}</dt><dd>{t('reg.purpose.existing_building')}</dd>
          <dt>{t('reg.surveyType')}</dt><dd>{t('surveyReg.surveyType', { method: stored.kind === 'residential' ? 'ISSO 82.1' : 'ISSO 75.1' })}</dd>
          <dt>{t('reg.constructionYear')}</dt><dd>{registration.constructionYear ?? '—'}</dd>
          <dt>{t('survey.surveyDate')}</dt><dd>{date(registration.surveyDate)}</dd>
          <dt>{t('reg.surveyingAdvisor')}</dt><dd>{advisorText(registration.surveyingAdvisor)}</dd>
          <dt>{t('reg.registeringAdvisor')}</dt><dd>{advisorText(registration.registeringAdvisor)}</dd>
        </dl>
        <p className="survey-muted">{t('surveyReg.takenOverHint')}</p>
      </section>

      <details className="survey-card survey-reg-evidence" ref={evidenceRef} open={evidenceOpen}
        onToggle={(event) => setEvidenceOpen((event.target as HTMLDetailsElement).open)}>
        <summary>{t('surveyReg.evidence', { count: registration.evidence?.length ?? 0 })}</summary>
        <EvidenceRegister evidence={registration.evidence ?? []} onChange={(evidence) => set({ evidence })} />
      </details>
    </div>

    <aside className="survey-aside" aria-label={t('surveyReg.aside')}>
      <div className="survey-card">
        <span className="survey-overline">EP-Online</span>
        <p className="survey-final-facts">{t('surveyReg.export')}</p>
        <button type="button" className="btn btn-primary" disabled={!software.attestNumber || open > 0}
          onClick={() => actions.navigate({ step: 'report', sub: 'exports' } as Route)}>{t('surveyReg.exportButton')}</button>
        <p className="survey-muted">{!software.attestNumber ? t('surveyReg.exportNoAttest')
          : open > 0 ? t('surveyReg.exportOpen', { count: open }) : t('surveyReg.exportReady')}</p>
      </div>
      <div className="survey-card">
        <span className="survey-overline">{t('surveyReg.validity')}</span>
        <p className="survey-final-facts">{until ? t('surveyReg.validUntil', { date: date(until) }) : t('surveyReg.validNoDate')}</p>
      </div>
      <div className="survey-card">
        <span className="survey-overline">{t('registration.page.software')}</span>
        <p className="survey-final-facts">{software.name} {software.version}<br />
          {software.attestNumber ? `${t('reg.software.attest')} ${software.attestNumber}` : t('reg.software.unattested')}</p>
      </div>
    </aside>
  </div>;
}
