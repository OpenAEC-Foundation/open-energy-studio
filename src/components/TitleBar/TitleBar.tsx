import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import { Zap } from 'lucide-react';
import './TitleBar.css';

export function TitleBar() {
  const { t } = useI18n();
  const { state } = useEnergy();
  const projectName = state.project.name || t('app.untitledProject');

  return (
    <div className="title-bar">
      <div className="title-bar-left">
        <Zap size={16} className="title-bar-icon" />
        <span className="title-bar-app">{t('app.title')}</span>
        <span className="title-bar-separator">—</span>
        <span className="title-bar-project">{projectName}</span>
        {state.isDirty && <span className="title-bar-dirty">*</span>}
      </div>
    </div>
  );
}
