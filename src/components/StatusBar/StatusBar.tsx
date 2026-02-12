import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import './StatusBar.css';

export function StatusBar() {
  const { t } = useI18n();
  const { state } = useEnergy();
  const { project, result } = state;

  const zoneCount = project.zones.length;
  const surfaceCount = project.zones.reduce((sum, z) => sum + z.surfaces.length, 0);

  return (
    <div className="status-bar">
      <div className="status-section">
        <span className="status-hint">
          {result ? t('status.calculated') : t('status.ready')}
        </span>
      </div>
      <div className="status-section status-stats">
        <span>
          <strong>{t('status.zones')}:</strong> {zoneCount}
        </span>
        <span>
          <strong>{t('status.surfaces')}:</strong> {surfaceCount}
        </span>
        {result && (
          <>
            <span className={result.beng1Pass && result.beng2Pass && result.beng3Pass ? 'status-pass' : 'status-fail'}>
              BENG: {result.beng1Pass && result.beng2Pass && result.beng3Pass ? t('results.pass') : t('results.fail')}
            </span>
          </>
        )}
      </div>
    </div>
  );
}
