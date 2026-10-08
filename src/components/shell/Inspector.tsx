/**
 * The one contextual panel on the right (ontwerp §3.3). It holds the two
 * former side panels, Eigenschappen (the selected item, with edit and delete)
 * and Voorbeeld (the live kernel preview; the old ribbon "Preview" toggle,
 * `TOGGLE_PREVIEW`), and since F6 Controle: the kernel findings and omitted
 * corrections, this page first, each with "Ga naar". Ctrl . hides the panel.
 * A selected building element gets the inline editor (F5); on Installaties
 * without a selection the panel shows the energy per service.
 */
import { projectCalculated } from '../../core/nta/KernelClient';
import { useState } from 'react';
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
import { InspectorCheckPanel } from './InspectorCheckPanel';

type InspectorTab = 'properties' | 'preview' | 'check';

export function Inspector({ onClose }: { onClose: () => void }) {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();
  const preview = state.previewVisible;
  const element = findBuildingElement(state.project, state.selectedItemType, state.selectedItemId);
  // On Installaties a building selection is left over from Gebouw: show the energy per service instead.
  const installationsContext = state.route.step === 'installations' && (!state.selectedItemId || element != null);
  const [checkOpen, setCheckOpen] = useState(false);
  // A new selection shows its properties ("Klik selecteert, eigenschappen rechts"), also while the
  // preview is on; choosing Voorbeeld again keeps the preview until the next selection.
  const [previewChosenFor, setPreviewChosenFor] = useState<string | null>(null);
  const showSelection = state.selectedItemId != null && state.selectedItemId !== previewChosenFor;
  const tab: InspectorTab = checkOpen ? 'check' : preview && !showSelection ? 'preview' : 'properties';
  const select = (next: InspectorTab) => {
    setCheckOpen(next === 'check');
    if (next === 'check') return;
    if (next === 'preview') {
      setPreviewChosenFor(state.selectedItemId);
      if (!preview) dispatch({ type: 'TOGGLE_PREVIEW' });
    } else if (preview && !showSelection) {
      dispatch({ type: 'TOGGLE_PREVIEW' });
    }
  };
  const tabs: Array<{ id: InspectorTab; label: string }> = [
    { id: 'properties', label: t('shell.inspector.properties') },
    { id: 'preview', label: t('shell.inspector.preview') },
    { id: 'check', label: t('shell.inspector.check') },
  ];

  const onTabKey = (event: React.KeyboardEvent) => {
    if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return;
    event.preventDefault();
    const index = tabs.findIndex((item) => item.id === tab);
    const next = (index + (event.key === 'ArrowRight' ? 1 : -1) + tabs.length) % tabs.length;
    select(tabs[next].id);
    event.currentTarget.querySelectorAll<HTMLButtonElement>('[role="tab"]')[next]?.focus();
  };

  return (
    <aside className="inspector" aria-label={t('shell.inspector.title')}>
      <div className="inspector-head">
        <div className="inspector-tabs" role="tablist" aria-label={t('shell.inspector.title')} onKeyDown={onTabKey}>
          {tabs.map((item) => (
            <button key={item.id} type="button" role="tab" id={`inspector-tab-${item.id}`} className="inspector-tab"
              aria-selected={tab === item.id} aria-controls="inspector-panel" tabIndex={tab === item.id ? 0 : -1}
              onClick={() => select(item.id)}>
              {item.label}
            </button>
          ))}
        </div>
        <IconButton size="sm" icon={<X aria-hidden="true" />} aria-label={t('shell.inspector.close')} onClick={onClose} />
      </div>
      <div className="inspector-body" id="inspector-panel" role="tabpanel" aria-labelledby={`inspector-tab-${tab}`}>
        <ErrorBoundary resetKey={state.project}>
          {tab === 'check' ? <InspectorCheckPanel /> : tab === 'preview' ? <PreviewPanel embedded /> : installationsContext
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
