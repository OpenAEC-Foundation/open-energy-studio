/**
 * The one contextual panel on the right (ontwerp §3.3). For now it holds the
 * two former side panels: Eigenschappen (the selected item, with edit and
 * delete) and Voorbeeld (the live kernel preview). The switch between them is
 * the old ribbon "Preview" toggle (`TOGGLE_PREVIEW`); Ctrl . hides the panel.
 */
import { X } from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import { IconButton } from '../ui';
import { PropertiesPanel } from '../PropertiesPanel/PropertiesPanel';
import { PreviewPanel } from '../PreviewPanel/PreviewPanel';
import { ErrorBoundary } from '../ErrorBoundary/ErrorBoundary';

export function Inspector({ onClose }: { onClose: () => void }) {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();
  const preview = state.previewVisible;
  const show = (wantPreview: boolean) => { if (wantPreview !== preview) dispatch({ type: 'TOGGLE_PREVIEW' }); };

  const onTabKey = (event: React.KeyboardEvent) => {
    if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return;
    event.preventDefault();
    show(!preview);
    const tabs = event.currentTarget.querySelectorAll<HTMLButtonElement>('[role="tab"]');
    tabs[preview ? 0 : 1]?.focus();
  };

  return (
    <aside className="inspector" aria-label={t('shell.inspector.title')}>
      <div className="inspector-head">
        <div className="inspector-tabs" role="tablist" aria-label={t('shell.inspector.title')} onKeyDown={onTabKey}>
          <button type="button" role="tab" id="inspector-tab-properties" className="inspector-tab" aria-selected={!preview}
            aria-controls="inspector-panel" tabIndex={preview ? -1 : 0} onClick={() => show(false)}>
            {t('shell.inspector.properties')}
          </button>
          <button type="button" role="tab" id="inspector-tab-preview" className="inspector-tab" aria-selected={preview}
            aria-controls="inspector-panel" tabIndex={preview ? 0 : -1} onClick={() => show(true)}>
            {t('shell.inspector.preview')}
          </button>
        </div>
        <IconButton size="sm" icon={<X aria-hidden="true" />} aria-label={t('shell.inspector.close')} onClick={onClose} />
      </div>
      <div className="inspector-body" id="inspector-panel" role="tabpanel"
        aria-labelledby={preview ? 'inspector-tab-preview' : 'inspector-tab-properties'}>
        <ErrorBoundary resetKey={state.project}>
          {preview ? <PreviewPanel embedded /> : <PropertiesPanel embedded />}
        </ErrorBoundary>
      </div>
    </aside>
  );
}
