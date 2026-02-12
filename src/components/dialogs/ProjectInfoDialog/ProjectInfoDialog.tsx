import { useState } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { BuildingFunction } from '../../../core/energy/types';

interface ProjectInfoDialogProps {
  editId?: string | null;
  onClose: () => void;
}

const buildingFunctions: BuildingFunction[] = [
  'residential', 'office', 'education', 'healthcare', 'retail', 'industrial', 'other',
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

  const handleSave = () => {
    dispatch({
      type: 'UPDATE_PROJECT_INFO',
      payload: { name, description, buildingFunction, address, city },
    });
    onClose();
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-title">{t('dialog.projectInfo.title')}</div>

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

        <div className="dialog-actions">
          <button className="btn" onClick={onClose}>
            {t('dialog.cancel')}
          </button>
          <button className="btn btn-primary" onClick={handleSave}>
            {t('dialog.save')}
          </button>
        </div>
      </div>
    </div>
  );
}
