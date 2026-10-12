/**
 * Registration data of the Registratie step (UI redesign F9): message type,
 * object and BAG, survey and advisers, WLC-GWP, detail-survey triggers and the
 * evidence register. Formerly part of the project-info dialog. It edits the
 * document's registration draft (`RegistrationDraftProvider`, review 35), saved
 * with `cleanRegistration` (software identity kept on a relabel) and the BAG
 * ledger update.
 */
import { NumberInput } from '../../ui/Field';
import { useEffect, useId, useState } from 'react';
import { Save, Undo2 } from 'lucide-react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { registrationChanged, useRegistrationDraft } from '../../../context/RegistrationDraftProvider';
import type { NtaDetailSurveyTriggers, NtaRegistration } from '../../../core/nta/KernelClient';
import {
  bagConflicts, cleanRegistration, readBagLedger, recordBagRegistration, registrationSoftware,
} from '../../../core/nta/Registration';
import { stampProject } from '../../../core/io/KernelStampClient';
import { EvidenceRegister } from '../../dialogs/ProjectInfoDialog/EvidenceRegister';
import { Banner, Button, Card, Pill } from '../../ui';

type TextKey = 'referenceObjectId' | 'bagObjectId' | 'postcode' | 'houseNumber' | 'houseNumberAddition'
  | 'buildingType' | 'client' | 'certificateNumber' | 'surveyDate' | 'registrationDate'
  | 'originalKernelVersion' | 'epOnlineNumber' | 'completionDate' | 'improvementDate'
  | 'replacedEpOnlineNumber' | 'previousLabelClass'
  | 'originalCertificateNumber' | 'originalEpOnlineNumber' | 'originalDossierReference';

type TextField = { key: TextKey; type?: 'date' };

/** Address and object (Omgevingsregeling art. 5.14 lid 1 onder a). */
const objectFields: TextField[] = [
  { key: 'bagObjectId' }, { key: 'postcode' }, { key: 'houseNumber' }, { key: 'houseNumberAddition' },
  { key: 'buildingType' }, { key: 'referenceObjectId' },
];
/** Survey and registration (BRL 9500 §4.2.5/§4.2.6). */
const surveyFields: TextField[] = [
  { key: 'client' }, { key: 'certificateNumber' },
  { key: 'surveyDate', type: 'date' }, { key: 'registrationDate', type: 'date' }, { key: 'completionDate', type: 'date' },
  { key: 'epOnlineNumber' }, { key: 'previousLabelClass' },
];
/** Relabel only (BRL 9500-W §4.2.3/4.2.4 p. 23–24, Bijlage 6a/6b; the relabel is an addendum to the original dossier). */
const relabelFields: TextField[] = [
  { key: 'originalKernelVersion' }, { key: 'improvementDate', type: 'date' },
  { key: 'originalCertificateNumber' }, { key: 'originalEpOnlineNumber' }, { key: 'originalDossierReference' },
];
/** Relabel answers the kernel checks; empty means not yet established. */
type RelabelAnswer = 'productionPhysicallyConnected' | 'noExcludedChangesConfirmed';
const relabelAnswers: RelabelAnswer[] = ['productionPhysicallyConnected', 'noExcludedChangesConfirmed'];
/** Replacement of an incorrect label only. */
const replacementFields: TextField[] = [{ key: 'replacedEpOnlineNumber' }];

/** BRL 9500 §3.1 situations that require a detailed survey. */
const detailTriggers: Array<keyof NtaDetailSurveyTriggers> = [
  'rebuiltAfterDemolition', 'fullRenovationWithNewBuildRequirements', 'energyPerformanceFee',
  'bengRequirementProof', 'previousDetailedRegistration', 'addedAfter2021',
];

export function RegistrationForm() {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();
  const { project } = state;
  const stored = project.registration;
  // The document's draft, so leaving the step loses nothing (review 35); a
  // local one outside a document.
  const shared = useRegistrationDraft();
  const [local, setLocal] = useState<NtaRegistration>(stored ?? {});
  // A new stored registration (save, undo, another document) restarts the local draft.
  useEffect(() => { if (!shared) setLocal(stored ?? {}); }, [stored, shared]);
  const registration = shared?.draft ?? local;
  // Calculation core of the kernel that calculates this project, stored with
  // the program identity (BRL 9500-W §4.2.4, p. 24).
  const [kernelVersion, setKernelVersion] = useState<string | undefined>();
  useEffect(() => {
    let active = true;
    void stampProject(project).then((stamp) => { if (active) setKernelVersion(stamp?.kernelVersion); });
    return () => { active = false; };
  }, [project]);
  const update = (patch: Partial<NtaRegistration>) => {
    if (shared) shared.update(patch);
    else setLocal((current) => ({ ...current, ...patch }));
  };
  const discard = () => {
    if (shared) shared.discard();
    else setLocal(stored ?? {});
  };
  const advisor = (key: 'surveyingAdvisor' | 'registeringAdvisor') =>
    registration[key] ?? { name: '', competenceNumber: '' };

  const messageType = registration.messageType ?? (registration.relabel ? 'relabel' : 'regular');
  const ledgerEntry = {
    bagObjectId: registration.bagObjectId ?? '',
    projectId: project.id,
    projectName: project.name,
    residential: project.buildingFunction === 'residential',
    messageType,
    epOnlineNumber: registration.epOnlineNumber,
    registrationDate: registration.registrationDate,
    surveyDate: registration.surveyDate,
  };
  const conflicts = bagConflicts(ledgerEntry, readBagLedger());
  const software = registrationSoftware(registration, kernelVersion);
  // Saving also restamps the program identity (Omgevingsregeling art. 5.14 lid 1 onder b), so a
  // stored stamp from another kernel version is a change to save even with the fields unchanged.
  const stampOutdated = stored?.software != null && kernelVersion != null
    && JSON.stringify(stored.software) !== JSON.stringify(software);
  // A field set and cleared again is no change: compared as it would be stored.
  const dirty = registrationChanged(registration, stored);
  const canSave = dirty || stampOutdated;
  const formId = useId();

  const textInput = ({ key, type }: TextField) => (
    <div className="dialog-field" key={key} data-path={`registration.${key}`}>
      <label htmlFor={`reg-${key}`}>{t(`reg.${key}`)}</label>
      <input id={`reg-${key}`} type={type ?? 'text'} value={registration[key] ?? ''}
        onChange={(e) => update({ [key]: e.target.value || undefined })} />
    </div>
  );

  const handleSave = () => {
    // Keeps registered labels only and drops this project's entry when the
    // BAG id or the EP-Online number was cleared.
    recordBagRegistration(ledgerEntry);
    dispatch({ type: 'UPDATE_PROJECT_INFO', payload: { registration: cleanRegistration(registration, kernelVersion) } });
  };

  return (
    <form className="registration-form" aria-label={t('registration.form.title')}
      onSubmit={(event) => { event.preventDefault(); handleSave(); }}>
      <Card title={t('dialog.projectInfo.registration')} subtitle={t('dialog.projectInfo.registrationHint')} level={2}>
        <div className="dialog-grid">
          <div className="dialog-field" data-path="registration.purpose">
            <label htmlFor="reg-purpose">{t('reg.purpose')}</label>
            <select id="reg-purpose" value={registration.purpose ?? ''}
              onChange={(e) => update({ purpose: (e.target.value || undefined) as NtaRegistration['purpose'] })}>
              <option value="">—</option>
              <option value="existing_building">{t('reg.purpose.existing_building')}</option>
              <option value="delivery">{t('reg.purpose.delivery')}</option>
              <option value="bbl_check">{t('reg.purpose.bbl_check')}</option>
            </select>
          </div>
          <div className="dialog-field" data-path="registration.surveyType">
            <label htmlFor="reg-survey-type">{t('reg.surveyType')}</label>
            <select id="reg-survey-type" value={registration.surveyType ?? ''}
              onChange={(e) => update({ surveyType: (e.target.value || undefined) as NtaRegistration['surveyType'] })}>
              <option value="">—</option>
              <option value="basic">{t('reg.surveyType.basic')}</option>
              <option value="detailed">{t('reg.surveyType.detailed')}</option>
            </select>
          </div>
          <div className="dialog-field" data-path="registration.representation">
            <label htmlFor="reg-representation">{t('reg.representation')}</label>
            <select id="reg-representation" value={registration.representation ?? ''}
              onChange={(e) => update({ representation: (e.target.value || undefined) as NtaRegistration['representation'] })}>
              <option value="">—</option>
              <option value="unique">{t('reg.representation.unique')}</option>
              <option value="reference">{t('reg.representation.reference')}</option>
              <option value="similar">{t('reg.representation.similar')}</option>
            </select>
          </div>
          <div className="dialog-field" data-path="registration.constructionYear">
            <label htmlFor="reg-construction-year">{t('reg.constructionYear')}</label>
            <NumberInput id="reg-construction-year" value={registration.constructionYear ?? null} optional
              onChange={(next) => update({ constructionYear: next ?? undefined })} />
          </div>
          <div className="dialog-field" data-path="registration.messageType">
            <label htmlFor="reg-message-type">{t('reg.messageType')}</label>
            <select id="reg-message-type" value={messageType}
              onChange={(e) => update({ messageType: e.target.value as NonNullable<NtaRegistration['messageType']>, relabel: undefined })}>
              <option value="regular">{t('reg.messageType.regular')}</option>
              <option value="relabel">{t('reg.messageType.relabel')}</option>
              <option value="replacement">{t('reg.messageType.replacement')}</option>
            </select>
          </div>
          {messageType === 'relabel' && relabelFields.map(textInput)}
          {messageType === 'relabel' && relabelAnswers.map((key) => (
            <div className="dialog-field" key={key} data-path={`registration.${key}`}>
              <label htmlFor={`reg-${key}`}>{t(`reg.${key}`)}</label>
              <select id={`reg-${key}`} value={registration[key] == null ? '' : String(registration[key])}
                onChange={(e) => update({ [key]: e.target.value === '' ? undefined : e.target.value === 'true' })}>
                <option value="">—</option>
                <option value="true">{t('common.yes')}</option>
                <option value="false">{t('common.no')}</option>
              </select>
            </div>
          ))}
          {messageType === 'replacement' && replacementFields.map(textInput)}
        </div>
      </Card>

      <Card title={t('dialog.projectInfo.object')} subtitle={t('registration.form.objectHint')} level={2}>
        <div className="dialog-grid">{objectFields.map(textInput)}</div>
        {conflicts.length > 0 && (
          <Banner tone="warn" role="alert">
            {t('reg.bagConflict')} {conflicts.map((item) => item.projectName || item.projectId).join(', ')}
          </Banner>
        )}
      </Card>

      <Card title={t('dialog.projectInfo.survey')} level={2}>
        <div className="dialog-grid">
          {surveyFields.map(textInput)}
          {(['surveyingAdvisor', 'registeringAdvisor'] as const).map((key) => (
            <div className="dialog-field" key={key} data-path={`registration.${key}`}>
              <label htmlFor={`reg-${key}-name`}>{t(`reg.${key}`)}</label>
              <input id={`reg-${key}-name`} type="text" placeholder={t('reg.advisorName')} value={advisor(key).name}
                onChange={(e) => update({ [key]: { ...advisor(key), name: e.target.value } })} />
              <input aria-label={`${t(`reg.${key}`)} — ${t('reg.competenceNumber')}`} type="text"
                placeholder={t('reg.competenceNumber')} value={advisor(key).competenceNumber}
                onChange={(e) => update({ [key]: { ...advisor(key), competenceNumber: e.target.value } })} />
            </div>
          ))}
          <label className="dialog-check" data-path="registration.serialProject">
            <input type="checkbox" checked={registration.serialProject ?? false}
              onChange={(e) => update({ serialProject: e.target.checked })} />
            {t('reg.serialProject')}
          </label>
        </div>
      </Card>

      <Card title={t('reg.wlcGwp')} subtitle={t('reg.wlcGwpHint')} level={2}>
        <div className="dialog-grid">
          <div className="dialog-field" data-path="registration.wlcGwp.valueKgCo2EqPerM2Year">
            <label htmlFor="reg-wlc-value">{t('reg.wlcGwp.value')}</label>
            <NumberInput id="reg-wlc-value" value={registration.wlcGwp?.valueKgCo2EqPerM2Year ?? null} optional
              onChange={(next) => update({ wlcGwp: { ...registration.wlcGwp, valueKgCo2EqPerM2Year: next ?? undefined } })} />
          </div>
          <div className="dialog-field" data-path="registration.wlcGwp.reportReference">
            <label htmlFor="reg-wlc-reference">{t('reg.wlcGwp.reference')}</label>
            <input id="reg-wlc-reference" type="text" value={registration.wlcGwp?.reportReference ?? ''}
              onChange={(e) => update({ wlcGwp: { ...registration.wlcGwp, reportReference: e.target.value || undefined } })} />
          </div>
          <div className="dialog-field" data-path="registration.bblCheckDate">
            <label htmlFor="reg-bbl-check-date">{t('reg.bblCheckDate')}</label>
            <input id="reg-bbl-check-date" type="date" value={registration.bblCheckDate ?? ''}
              onChange={(e) => update({ bblCheckDate: e.target.value || undefined })} />
          </div>
          <div className="dialog-field" data-path="registration.buildingUsableFloorAreaM2">
            <label htmlFor="reg-building-area">{t('reg.buildingUsableFloorArea')}</label>
            <NumberInput id="reg-building-area" value={registration.buildingUsableFloorAreaM2 ?? null} optional
              onChange={(next) => update({ buildingUsableFloorAreaM2: next ?? undefined })} />
          </div>
        </div>
      </Card>

      <Card title={t('reg.labelStatements')} subtitle={t('reg.labelStatementsHint')} level={2}>
        <div className="dialog-grid">
          {(['respondsToExternalSignals', 'lowTemperatureHeating'] as const).map((key) => {
            const answer = registration.labelStatements?.[key];
            return (
              <div key={key} className="dialog-field" data-path={`registration.labelStatements.${key}`}>
                <label htmlFor={`reg-statement-${key}`}>{t(`reg.labelStatements.${key}`)}</label>
                <select id={`reg-statement-${key}`} value={answer === undefined ? '' : answer ? 'yes' : 'no'}
                  onChange={(e) => update({
                    labelStatements: {
                      ...registration.labelStatements,
                      [key]: e.target.value === '' ? undefined : e.target.value === 'yes',
                    },
                  })}>
                  <option value="">{t('reg.labelStatements.unanswered')}</option>
                  <option value="yes">{t('common.yes')}</option>
                  <option value="no">{t('common.no')}</option>
                </select>
              </div>
            );
          })}
        </div>
      </Card>

      <Card title={t('registration.form.software')} level={2}>
        <p className="registration-software" data-testid="reg-software" data-path="registration.software">
          {t('reg.software')}: {software.name} {software.version} — {software.attestNumber
            ? `${t('reg.software.attest')} ${software.attestNumber}` : t('reg.software.unattested')}
          {' '}{software.attestNumber ? <Pill tone="ok">{t('registration.page.attested')}</Pill>
            : <Pill tone="unv">{t('status.unattested')}</Pill>}
        </p>
      </Card>

      <Card title={t('reg.detailSurveyTriggers')} subtitle={t('reg.detailSurveyTriggersHint')} level={2}>
        <div className="registration-checks" data-path="registration.detailSurveyTriggers">
          {detailTriggers.map((key) => (
            <label className="dialog-check" key={key}>
              <input type="checkbox" checked={registration.detailSurveyTriggers?.[key] ?? false}
                onChange={(e) => update({ detailSurveyTriggers: { ...registration.detailSurveyTriggers, [key]: e.target.checked } })} />
              {t(`reg.trigger.${key}`)}
            </label>
          ))}
        </div>
      </Card>

      <Card title={t('evidence.title')} level={2}>
        {(shared?.evidenceConflicts.length ?? 0) > 0 && (
          <Banner tone="warn" role="status">
            {t('registration.form.evidenceConflict', { names: shared!.evidenceConflicts.join(', ') })}
          </Banner>
        )}
        <div data-path="registration.evidence">
          <EvidenceRegister evidence={registration.evidence ?? []} onChange={(evidence) => update({ evidence })} />
        </div>
      </Card>

      <div className="registration-form-actions" role="group" aria-label={t('registration.form.actions')}>
        {dirty
          ? <span className="registration-form-dirty">{t('registration.form.unsaved')}</span>
          : stampOutdated
            ? <span className="registration-form-dirty">{t('registration.form.stampOutdated')}</span>
            : <span id={`${formId}-clean`} className="registration-form-clean">{t('registration.form.nothingToSave')}</span>}
        <Button type="button" variant="ghost" icon={<Undo2 aria-hidden="true" />} disabled={!dirty}
          onClick={discard}>{t('registration.form.discard')}</Button>
        {/* Nothing changed, nothing to save (UX finding 8). */}
        <Button type="submit" variant="primary" icon={<Save aria-hidden="true" />} disabled={!canSave}
          aria-describedby={canSave ? undefined : `${formId}-clean`}>{t('registration.form.save')}</Button>
      </div>
    </form>
  );
}
