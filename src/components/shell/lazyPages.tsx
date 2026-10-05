/**
 * Heavy pages loaded on first use (UI redesign F10): the 3D model, the report
 * builder, the print preview and the tools. They live in their own chunks so
 * the start-up bundle stays smaller; a skeleton shows while a chunk loads.
 */
import { lazy, Suspense, type ReactNode } from 'react';
import { useI18n } from '../../i18n/i18n';
import { Skeleton } from '../ui';

export const Building3DView = lazy(() => import('../Building3DView/Building3DView').then((m) => ({ default: m.Building3DView })));
export const ReportView = lazy(() => import('../ReportView/ReportView').then((m) => ({ default: m.ReportView })));
export const PrintPreviewDialog = lazy(() => import('../dialogs/PrintPreviewDialog/PrintPreviewDialog').then((m) => ({ default: m.PrintPreviewDialog })));
export const UValueCalculator = lazy(() => import('../UValueCalculator/UValueCalculator').then((m) => ({ default: m.UValueCalculator })));
export const ThermalBridgeCalculator = lazy(() => import('../ThermalBridgeCalculator/ThermalBridgeCalculator').then((m) => ({ default: m.ThermalBridgeCalculator })));
export const HeatPumpSizingCalculator = lazy(() => import('../HeatPumpSizingCalculator/HeatPumpSizingCalculator').then((m) => ({ default: m.HeatPumpSizingCalculator })));

function Loading() {
  const { t } = useI18n();
  return (
    <div className="lazy-page-loading" role="status">
      <span className="visually-hidden">{t('shell.loadingPage')}</span>
      <Skeleton height={20} width="40%" />
      <Skeleton height={14} />
      <Skeleton height={14} width="80%" />
    </div>
  );
}

/** Suspense boundary for a lazily loaded page. */
export function LazyPage({ children }: { children: ReactNode }) {
  return <Suspense fallback={<Loading />}>{children}</Suspense>;
}
