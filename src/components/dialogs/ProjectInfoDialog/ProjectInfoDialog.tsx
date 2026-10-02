import { useState } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { BuildingFunction } from '../../../core/energy/types';
import type { NtaRegistration } from '../../../core/nta/KernelClient';
import { cleanRegistration } from '../../../core/nta/Registration';
import { DialogShell } from '../DialogShell';

interface ProjectInfoDialogProps {
  editId?: string | null;
  onClose: () => void;
}

const buildingFunctions: BuildingFunction[] = [
  'residential', 'office', 'education', 'healthcare', 'retail', 'industrial', 'other',
];

type TextKey = 'referenceObjectId' | 'bagObjectId' | 'postcode' | 'houseNumber' | 'houseNumberAddition'
  | 'buildingType' | 'client' | 'certificateNumber' | 'surveyDate' | 'registrationDate'
  | 'originalKernelVersion' | 'epOnlineNumber';

const textFields: Array<{ key: TextKey; type?: 'date' }> = [
  { key: 'bagObjectId' }, { key: 'postcode' }, { key: 'houseNumber' }, { key: 'houseNumberAddition' },
  { key: 'buildingType' }, { key: 'client' }, { key: 'certificateNumber' },
  { key: 'surveyDate', type: 'date' }, { key: 'registrationDate', type: 'date' },
  { key: 'referenceObjectId' }, { key: 'originalKernelVersion' }, { key: 'epOnlineNumber' },
];

export function ProjectInfoDialog({ onClose }: ProjectInfoDialogProps) {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();
  const { project } = state;

  const [name, setName] = useState(project.name);
  const [description, setDescription] = useState(project.description);
  const [buildingFunction, setBuildingFunction] = useState<BuildingFunction>(project.buildingFunction);
  const [address, setAddress] = useState(project.address);
  const [city, setCity] = useState(project.city);
  const [registration, setRegistration] = useState<NtaRegistration>(project.registration ?? {});
  const update = (patch: Partial<NtaRegistration>) => setRegistration((current) => ({ ...current, ...patch }));
  const advisor = (key: 'surveyingAdvisor' | 'registeringAdvisor') =>
    registration[key] ?? { name: '', competenceNumber: '' };

  const handleSave = () => {
    dispatch({
      type: 'UPDATE_PROJECT_INFO',
      payload: { name, description, buildingFunction, address, city, registration: cleanRegistration(registration) },
    });
    onClose();
  };

  return (
    <DialogShell
      title={t('dialog.projectInfo.title')}
      onClose={onClose}
      onSubmit={handleSave}
      submitLabel={t('dialog.save')}
      cancelLabel={t('dialog.cancel')}
    >
        <div className="dialog-field">
          <label>{t('dialog.projectInfo.name')}</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.projectInfo.description')}</label>
          <textarea
            rows={3}
            value={description}
            onChange={(e) => setDescription(e.target.value)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.projectInfo.function')}</label>
          <select
            value={buildingFunction}
            onChange={(e) => setBuildingFunction(e.target.value as BuildingFunction)}
          >
            {buildingFunctions.map((fn) => (
              <option key={fn} value={fn}>
                {t(`function.${fn}`)}
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-field">
          <label>{t('dialog.projectInfo.address')}</label>
          <input
            type="text"
            value={address}
            onChange={(e) => setAddress(e.target.value)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.projectInfo.city')}</label>
          <input
            type="text"
            value={city}
            onChange={(e) => setCity(e.target.value)}
          />
        </div>

        <h3 className="dialog-section-title">{t('dialog.projectInfo.registration')}</h3>
        <p className="dialog-hint">{t('dialog.projectInfo.registrationHint')}</p>
        <div className="dialog-field">
          <label htmlFor="reg-purpose">{t('reg.purpose')}</label>
          <select id="reg-purpose" value={registration.purpose ?? ''}
            onChange={(e) => update({ purpose: (e.target.value || undefined) as NtaRegistration['purpose'] })}>
            <option value="">—</option>
            <option value="existing_building">{t('reg.purpose.existing_building')}</option>
            <option value="delivery">{t('reg.purpose.delivery')}</option>
            <option value="bbl_check">{t('reg.purpose.bbl_check')}</option>
          </select>
        </div>
        <div className="dialog-field">
          <label htmlFor="reg-survey-type">{t('reg.surveyType')}</label>
          <select id="reg-survey-type" value={registration.surveyType ?? ''}
            onChange={(e) => update({ surveyType: (e.target.value || undefined) as NtaRegistration['surveyType'] })}>
            <option value="">—</option>
            <option value="basic">{t('reg.surveyType.basic')}</option>
            <option value="detailed">{t('reg.surveyType.detailed')}</option>
          </select>
        </div>
        <div className="dialog-field">
          <label htmlFor="reg-representation">{t('reg.representation')}</label>
          <select id="reg-representation" value={registration.representation ?? ''}
            onChange={(e) => update({ representation: (e.target.value || undefined) as NtaRegistration['representation'] })}>
            <option value="">—</option>
            <option value="unique">{t('reg.representation.unique')}</option>
            <option value="reference">{t('reg.representation.reference')}</option>
            <option value="similar">{t('reg.representation.similar')}</option>
          </select>
        </div>
        <div className="dialog-field">
          <label htmlFor="reg-construction-year">{t('reg.constructionYear')}</label>
          <input id="reg-construction-year" type="number" value={registration.constructionYear ?? ''}
            onChange={(e) => update({ constructionYear: e.target.value ? Number(e.target.value) : undefined })} />
        </div>
        {textFields.map(({ key, type }) => (
          <div className="dialog-field" key={key}>
            <label htmlFor={`reg-${key}`}>{t(`reg.${key}`)}</label>
            <input id={`reg-${key}`} type={type ?? 'text'} value={registration[key] ?? ''}
              onChange={(e) => update({ [key]: e.target.value || undefined })} />
          </div>
        ))}
        {(['surveyingAdvisor', 'registeringAdvisor'] as const).map((key) => (
          <div className="dialog-field" key={key}>
            <label htmlFor={`reg-${key}-name`}>{t(`reg.${key}`)}</label>
            <input id={`reg-${key}-name`} type="text" placeholder={t('reg.advisorName')} value={advisor(key).name}
              onChange={(e) => update({ [key]: { ...advisor(key), name: e.target.value } })} />
            <input aria-label={`${t(`reg.${key}`)} — ${t('reg.competenceNumber')}`} type="text"
              placeholder={t('reg.competenceNumber')} value={advisor(key).competenceNumber}
              onChange={(e) => update({ [key]: { ...advisor(key), competenceNumber: e.target.value } })} />
          </div>
        ))}
        <label className="dialog-check">
          <input type="checkbox" checked={registration.serialProject ?? false}
            onChange={(e) => update({ serialProject: e.target.checked })} />
          {t('reg.serialProject')}
        </label>
        <label className="dialog-check">
          <input type="checkbox" checked={registration.relabel ?? false}
            onChange={(e) => update({ relabel: e.target.checked })} />
          {t('reg.relabel')}
        </label>
    </DialogShell>
  );
}
