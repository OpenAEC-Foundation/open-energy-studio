import { useState } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { BuildingFunction } from '../../../core/energy/types';
import { DialogShell } from '../DialogShell';

interface ProjectInfoDialogProps {
  editId?: string | null;
  onClose: () => void;
}

const buildingFunctions: BuildingFunction[] = [
  'residential', 'office', 'education', 'healthcare', 'retail', 'industrial', 'other',
];

/**
 * Project basics (name, description, function, address). The registration
 * data moved to the Registratie step in UI redesign F9 (`RegistrationForm`).
 */
export function ProjectInfoDialog({ onClose }: ProjectInfoDialogProps) {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();
  const { project } = state;

  const [name, setName] = useState(project.name);
  const [description, setDescription] = useState(project.description);
  const [buildingFunction, setBuildingFunction] = useState<BuildingFunction>(project.buildingFunction);
  const [address, setAddress] = useState(project.address);
  const [city, setCity] = useState(project.city);

  const handleSave = () => {
    dispatch({ type: 'UPDATE_PROJECT_INFO', payload: { name, description, buildingFunction, address, city } });
    onClose();
  };

  return (
    <DialogShell
      title={t('dialog.projectInfo.title')}
      onClose={onClose}
      onSubmit={handleSave}
      submitLabel={t('dialog.save')}
      cancelLabel={t('dialog.cancel')}
      className="project-info-dialog"
    >
      <section className="dialog-section" aria-labelledby="project-info-general">
        <h3 className="dialog-section-title" id="project-info-general">{t('dialog.projectInfo.general')}</h3>
        <div className="dialog-grid">
          <div className="dialog-field">
            <label htmlFor="project-name">{t('dialog.projectInfo.name')}</label>
            <input id="project-name" type="text" value={name} onChange={(e) => setName(e.target.value)} />
          </div>

          <div className="dialog-field">
            <label htmlFor="project-description">{t('dialog.projectInfo.description')}</label>
            <textarea id="project-description" rows={3} value={description}
              onChange={(e) => setDescription(e.target.value)} />
          </div>

          <div className="dialog-field">
            <label htmlFor="project-function">{t('dialog.projectInfo.function')}</label>
            <select id="project-function" value={buildingFunction}
              onChange={(e) => setBuildingFunction(e.target.value as BuildingFunction)}>
              {buildingFunctions.map((fn) => (
                <option key={fn} value={fn}>{t(`function.${fn}`)}</option>
              ))}
            </select>
          </div>

          <div className="dialog-field">
            <label htmlFor="project-address">{t('dialog.projectInfo.address')}</label>
            <input id="project-address" type="text" value={address} onChange={(e) => setAddress(e.target.value)} />
          </div>

          <div className="dialog-field">
            <label htmlFor="project-city">{t('dialog.projectInfo.city')}</label>
            <input id="project-city" type="text" value={city} onChange={(e) => setCity(e.target.value)} />
          </div>
        </div>
        <p className="dialog-hint">{t('dialog.projectInfo.registrationMoved')}</p>
      </section>
    </DialogShell>
  );
}
