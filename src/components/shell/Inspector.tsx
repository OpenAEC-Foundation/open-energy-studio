/**
 * The one contextual panel on the right (ontwerp §3.3). For now it holds the
 * two former side panels: Eigenschappen (the selected item, with edit and
 * delete) and Voorbeeld (the live kernel preview). The switch between them is
 * the old ribbon "Preview" toggle (`TOGGLE_PREVIEW`); Ctrl . hides the panel.
 * A selected building element gets the inline editor (F5); on Installaties
 * without a selection the panel shows the energy per service.
 */
import { projectCalculated } from '../../core/nta/KernelClient';
import { X } from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import { useKernel } from '../../context/KernelProvider';
import { summarizeServiceEnergy } from '../../core/nta/ServiceEnergy';
import { formatNumber } from '../../i18n/format';
import { IconButton } from '../ui';
import { ElementInspector, findBuildingElement } from './ElementInspector';
import { PropertiesPanel } from '../PropertiesPanel/PropertiesPanel';
import { PreviewPanel } from '../PreviewPanel/PreviewPanel';
import { ErrorBoundary } from '../ErrorBoundary/ErrorBoundary';

export function Inspector({ onClose }: { onClose: () => void }) {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();
  const preview = state.previewVisible;
  const element = findBuildingElement(state.project, state.selectedItemType, state.selectedItemId);
  // On Installaties a building selection is left over from Gebouw: show the energy per service instead.
  const installationsContext = state.route.step === 'installations' && (!state.selectedItemId || element != null);
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
          {preview ? <PreviewPanel embedded /> : installationsContext
            ? <ServiceEnergyPanel />
            : element ? <ElementInspector /> : <PropertiesPanel embedded />}
        </ErrorBoundary>
      </div>
    </aside>
  );
}

/** Annual use per service from the last kernel run (mockup 03, "Energie per dienst"). */
export function ServiceEnergyPanel() {
  const { t, locale } = useI18n();
  const kernel = useKernel();
  const performance = projectCalculated(kernel?.settled?.status) ? kernel?.settled?.performance ?? null : null;
  const summary = performance ? summarizeServiceEnergy(performance.energyByService) : null;
  const kwh = (value: number) => `${formatNumber(value, locale, 0)} kWh`;
  const carrier = (code: string) => { const key = `inspector.carrier.${code}`; const text = t(key); return text === key ? code : text; };
  const largest = Math.max(1, ...(summary?.rows ?? []).map((row) => Object.values(row.usedKwh).reduce((sum, value) => sum + Math.abs(value), 0)));
  const totals = new Map<string, number>();
  for (const row of summary?.rows ?? []) {
    for (const [code, value] of Object.entries(row.usedKwh)) totals.set(code, (totals.get(code) ?? 0) + value);
  }
  return (
    <section className="service-energy" aria-labelledby="service-energy-title">
      <h3 id="service-energy-title" className="service-energy__title">{t('inspector.serviceEnergy')}</h3>
      {!summary ? <p className="service-energy__hint">{t('inspector.serviceEnergyNone')}</p> : <>
        <p className="service-energy__hint">{t('inspector.serviceEnergyHint')}</p>
        <dl className="service-energy__rows">
          {summary.rows.map((row) => {
            const used = Object.entries(row.usedKwh).filter(([, value]) => value !== 0);
            const total = used.reduce((sum, [, value]) => sum + Math.abs(value), 0);
            return (
              <div key={row.service} className="service-energy__row">
                <dt>{t(`nta.performance.service.${row.service}`)}</dt>
                <dd>
                  <span className={`service-energy__bar service-energy__bar--${row.service}`} aria-hidden="true"
                    style={{ width: `${Math.max(2, (total / largest) * 100)}%` }} />
                  {used.map(([code, value]) => <span key={code}><b className="ui-num">{kwh(value)}</b> <small>{carrier(code)}</small></span>)}
                </dd>
              </div>
            );
          })}
        </dl>
        <dl className="service-energy__total">
          {[...totals.entries()].filter(([, value]) => value !== 0).map(([code, value]) => (
            <div key={code}><dt>{t('inspector.serviceEnergyTotal')} · {carrier(code)}</dt><dd className="ui-num">{kwh(value)}</dd></div>
          ))}
        </dl>
      </>}
    </section>
  );
}
