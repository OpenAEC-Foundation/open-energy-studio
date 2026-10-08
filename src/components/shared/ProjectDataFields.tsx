/**
 * The project data on one place (feedback 8 Oct 2026: "alle projectinfo qua
 * adres enzo bij elkaar in plaats van los"): name, address with house number,
 * postcode and city, the BAG id and the client, the surveying adviser with
 * competence number and certificate, and for a survey project the survey
 * date. The survey's first question and the new-build project question both
 * show this block; Registratie only takes the values over.
 */
import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import type { NtaRegistration } from '../../core/nta/KernelClient';

export function ProjectDataFields() {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();
  const project = state.project;
  const registration = (project.registration ?? {}) as NtaRegistration;
  const stored = project.basisopname as (NonNullable<typeof project.basisopname> & { surveyDate?: string }) | undefined;
  const set = (payload: Record<string, unknown>) => dispatch({ type: 'UPDATE_PROJECT_INFO', payload });
  const setRegistration = (patch: Partial<NtaRegistration>) => set({ registration: { ...registration, ...patch } });
  // The registering adviser taken over from the surveying one follows its changes (Registratie).
  const setAdvisor = (surveyingAdvisor: { name: string; competenceNumber: string }) => {
    const follows = JSON.stringify(registration.registeringAdvisor) === JSON.stringify(registration.surveyingAdvisor);
    setRegistration({ surveyingAdvisor, ...(follows ? { registeringAdvisor: { ...surveyingAdvisor } } : {}) });
  };
  const text = (label: string, value: string | undefined, onChange: (value: string) => void, options: { wide?: boolean; path?: string } = {}) =>
    <label className={options.wide ? 'survey-address-wide' : undefined} data-path={options.path}>{label}
      <input type="text" value={value ?? ''} onChange={(event) => onChange(event.target.value)} />
    </label>;
  const registrationText = (key: 'postcode' | 'houseNumber' | 'houseNumberAddition' | 'bagObjectId' | 'client' | 'certificateNumber', label: string) =>
    text(label, registration[key], (value) => setRegistration({ [key]: value || undefined }), { path: `registration.${key}` });

  return <div className="nta-form survey-address"><div className="nta-form-grid">
    {text(t('survey.address.name'), project.name, (name) => set({ name }), { wide: true, path: 'name' })}
    {text(t('survey.address.street'), project.address, (address) => set({ address }), { path: 'address' })}
    {registrationText('houseNumber', t('surveyReg.field.houseNumber'))}
    {registrationText('houseNumberAddition', t('surveyReg.field.houseNumberAddition'))}
    {registrationText('postcode', t('survey.address.postcode'))}
    {text(t('survey.address.city'), project.city, (city) => set({ city }), { path: 'city' })}
    {registrationText('bagObjectId', t('surveyReg.field.bagObjectId'))}
    {registrationText('client', t('surveyReg.field.client'))}
    <p className="nta-form-subhead">{t('survey.address.adviser')}</p>
    {text(t('survey.address.adviserName'), registration.surveyingAdvisor?.name,
      (name) => setAdvisor({ name, competenceNumber: registration.surveyingAdvisor?.competenceNumber ?? '' }), { path: 'registration.surveyingAdvisor.name' })}
    {text(t('survey.address.adviserNumber'), registration.surveyingAdvisor?.competenceNumber,
      (competenceNumber) => setAdvisor({ name: registration.surveyingAdvisor?.name ?? '', competenceNumber }), { path: 'registration.surveyingAdvisor.competenceNumber' })}
    {registrationText('certificateNumber', t('survey.address.certificate'))}
    {stored && <label data-path="basisopname.surveyDate">{t('survey.surveyDate')}
      <input type="date" value={stored.surveyDate ?? ''}
        onChange={(event) => dispatch({ type: 'SET_BASISOPNAME', payload: { ...stored, surveyDate: event.target.value || undefined } })} />
    </label>}
  </div></div>;
}
