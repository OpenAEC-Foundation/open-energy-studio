import { useEffect, useState } from 'react';
import { AlertCircle, CheckCircle2, CircleHelp, ShieldCheck } from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import { formatNumber } from '../../i18n/format';
import type { IProject } from '../../core/energy/types';
import { assessProjectWithRust, type KernelAssessment } from '../../core/nta/KernelClient';
import { KernelDetail } from '../KernelCode/KernelCode';
import './KernelAuditPanel.css';

type AuditQuery =
  | { project: IProject; kind: 'loading' }
  | { project: IProject; kind: 'done'; assessment: KernelAssessment }
  | { project: IProject; kind: 'error'; error: string };

export function KernelAuditPanel({ project }: { project: IProject }) {
  const { t, locale } = useI18n();
  const [query, setQuery] = useState<AuditQuery | null>(null);

  useEffect(() => {
    let cancelled = false;
    setQuery({ project, kind: 'loading' });
    assessProjectWithRust(project).then(
      (value) => { if (!cancelled) setQuery({ project, kind: 'done', assessment: value }); },
      (reason: unknown) => {
        if (!cancelled) {
          setQuery({ project, kind: 'error', error: reason instanceof Error ? reason.message : String(reason) });
        }
      },
    );
    return () => { cancelled = true; };
  }, [project]);

  const currentQuery = query?.project === project ? query : null;
  const loading = currentQuery == null || currentQuery.kind === 'loading';
  const assessment = currentQuery?.kind === 'done' ? currentQuery.assessment : null;
  const error = currentQuery?.kind === 'error' ? currentQuery.error : null;

  const errorCount = assessment?.issues.filter((item) => item.severity === 'error').length ?? 0;
  const warningCount = assessment?.issues.filter((item) => item.severity === 'warning').length ?? 0;

  return (
    <section className="kernel-audit" aria-label={t('kernel.title')}>
      <div className="kernel-audit-header">
        <div className="kernel-audit-title"><ShieldCheck size={19} /><div>
          <h3>{t('kernel.title')}</h3>
          <p>{t('kernel.subtitle')}</p>
        </div></div>
        <span className="kernel-audit-badge">Rust · NTA 8800</span>
      </div>

      {loading && <p className="kernel-audit-state" role="status">{t('kernel.loading')}</p>}
      {!loading && error && (
        <div className="kernel-audit-unavailable" role="alert">
          <CircleHelp size={18} />
          <span>{t('kernel.unavailable')} <small>{error}</small></span>
        </div>
      )}
      {!loading && assessment && <>
        <div className={`kernel-audit-status ${errorCount ? 'has-errors' : 'is-checked'}`} role="status">
          {errorCount ? <AlertCircle size={18} /> : <CheckCircle2 size={18} />}
          <span>{errorCount ? t('kernel.errorsFound') : t('kernel.structureChecked')}</span>
          <strong>{errorCount} {t('kernel.errors')} · {warningCount} {t('kernel.warnings')}</strong>
        </div>
        <div className="kernel-audit-metrics">
          <span>{t('kernel.heatPumps')}: <strong>{assessment.summary.heatPumpCount}</strong></span>
          <span>{t('kernel.classified')}: <strong>{assessment.summary.classifiedHeatPumpCount}</strong></span>
          {assessment.summary.auxiliaryComponentCount != null && <span>{t('kernel.auxiliary.count')}:
            <strong> {assessment.summary.auxiliaryComponentCount}</strong></span>}
          {assessment.summary.systemLinkCount != null && <span>{t('kernel.links.count')}:
            <strong> {assessment.summary.systemLinkCount}</strong></span>}
          {assessment.summary.thermalBoundaries && <span>{t('kernel.boundary.progress')}:
            <strong> {assessment.summary.thermalBoundaries.classifiedSurfaceCount}/{assessment.summary.thermalBoundaries.surfaceCount} {t('kernel.boundary.surfaces')}
              {' · '}{assessment.summary.thermalBoundaries.classifiedBridgeCount}/{assessment.summary.thermalBoundaries.bridgeCount} {t('kernel.boundary.bridges')}</strong>
          </span>}
          {assessment.summary.thermalBoundaries && <span>{t('kernel.pointBridge.title')}:
            <strong> {assessment.summary.thermalBoundaries.classifiedPointBridgeCount ?? 0}/{assessment.summary.thermalBoundaries.pointBridgeCount ?? 0}
              {' · '}{assessment.summary.thermalBoundaries.pointInventoryComplete
                ? t('kernel.pointBridge.inventoryComplete') : t('kernel.pointBridge.inventoryUnknown')}</strong>
          </span>}
        </div>
        {assessment.summary.envelopeGeometry && <details className="kernel-audit-geometry">
          <summary>{t('kernel.geometry.title')}</summary>
          <p>{t('kernel.geometry.scope')}</p>
          <table>
            <thead><tr>
              <th scope="col">{t('kernel.geometry.zone')}</th>
              <th scope="col">{t('kernel.geometry.gross')}</th>
              <th scope="col">{t('kernel.geometry.window')}</th>
              <th scope="col">{t('kernel.geometry.opaque')}</th>
            </tr></thead>
            <tbody>
              {assessment.summary.envelopeGeometry.zones.map((zone) => <tr key={zone.zoneId}>
                <th scope="row">{zone.zoneId}</th>
                <td>{formatNumber(zone.grossSurfaceAreaM2, locale, 2)}</td>
                <td>{formatNumber(zone.windowAreaM2, locale, 2)}</td>
                <td>{formatNumber(zone.remainingOpaqueAreaM2, locale, 2)}</td>
              </tr>)}
              <tr className="kernel-audit-geometry-total"><th scope="row">{t('kernel.geometry.total')}</th>
                <td>{formatNumber(assessment.summary.envelopeGeometry.grossSurfaceAreaM2, locale, 2)}</td>
                <td>{formatNumber(assessment.summary.envelopeGeometry.windowAreaM2, locale, 2)}</td>
                <td>{formatNumber(assessment.summary.envelopeGeometry.remainingOpaqueAreaM2, locale, 2)}</td>
              </tr>
            </tbody>
          </table>
        </details>}
        {assessment.summary.directOutdoorDiagnostic?.totalDirectConductanceWPerK != null &&
          <details className="kernel-audit-geometry">
            <summary>{t('kernel.direct.title')}</summary>
            <p>{t('kernel.direct.scope')}</p>
            <table><tbody>
              <tr><th scope="row">{t('kernel.direct.elements')}</th><td>{formatNumber(assessment.summary.directOutdoorDiagnostic.elementConductanceWPerK, locale, 2)} W/K</td></tr>
              <tr><th scope="row">{t('kernel.direct.linear')}</th><td>{formatNumber(assessment.summary.directOutdoorDiagnostic.linearBridgeConductanceWPerK, locale, 2)} W/K</td></tr>
              <tr><th scope="row">{t('kernel.direct.point')}</th><td>{formatNumber(assessment.summary.directOutdoorDiagnostic.pointBridgeConductanceWPerK, locale, 2)} W/K</td></tr>
              <tr className="kernel-audit-geometry-total"><th scope="row">{t('kernel.direct.total')}</th><td>{formatNumber(assessment.summary.directOutdoorDiagnostic.totalDirectConductanceWPerK, locale, 2)} W/K</td></tr>
            </tbody></table>
          </details>}
        {assessment.summary.unheatedTransmissionDiagnostic?.totalReducedConductanceWPerK != null &&
          <details className="kernel-audit-geometry">
            <summary>{t('kernel.unheated.diagnosticTitle')}</summary>
            <p>{t('kernel.unheated.diagnosticScope')}</p>
            <table><thead><tr><th scope="col">{t('kernel.unheated.space')}</th>
              <th scope="col">H (W/K)</th><th scope="col">b</th><th scope="col">b·H (W/K)</th></tr></thead>
              <tbody>{assessment.summary.unheatedTransmissionDiagnostic.spaces.map((space) => <tr key={space.id}>
                <th scope="row">{project.unheatedSpaces?.find((item) => item.id === space.id)?.name ?? space.id}</th><td>{formatNumber(space.unreducedConductanceWPerK, locale, 2)}</td>
                <td>{formatNumber(space.reductionFactor, locale, 3)}</td><td>{formatNumber(space.reducedConductanceWPerK, locale, 2)}</td>
              </tr>)}<tr className="kernel-audit-geometry-total"><th scope="row">{t('kernel.direct.total')}</th>
                <td></td><td></td><td>{formatNumber(assessment.summary.unheatedTransmissionDiagnostic.totalReducedConductanceWPerK, locale, 2)}</td>
              </tr></tbody></table>
          </details>}
        {(assessment.summary.performancePointDiagnostics?.length ?? 0) > 0 &&
          <details className="kernel-audit-geometry">
            <summary>{t('kernel.pointRatio.title')}</summary>
            <p>{t('kernel.pointRatio.scope')}</p>
            <table>
              <thead><tr>
                <th scope="col">{t('kernel.pointRatio.pump')}</th>
                <th scope="col">{t('kernel.pointRatio.point')}</th>
                <th scope="col">{t('kernel.points.service')}</th>
                <th scope="col">{t('kernel.points.carrier')}</th>
                <th scope="col">{t('kernel.pointRatio.ratio')}</th>
              </tr></thead>
              <tbody>{assessment.summary.performancePointDiagnostics?.map((point) =>
                <tr key={`${point.heatPumpPath}-${point.pointId}`}>
                  <th scope="row">{point.heatPumpId}</th>
                  <td>{point.pointId}</td>
                  <td>{t(`kernel.points.service.${point.service}`)}</td>
                  <td>{t(`kernel.points.carrier.${point.inputEnergyCarrier}`)}</td>
                  <td>{formatNumber(point.instantaneousUsefulToInputRatio, locale, 2)}</td>
                </tr>)}</tbody>
            </table>
          </details>}
        {(assessment.summary.dhwTestDiagnostics?.length ?? 0) > 0 &&
          <details className="kernel-audit-geometry">
            <summary>{t('kernel.dhwTest.diagnosticTitle')}</summary>
            <p>{t('kernel.dhwTest.diagnosticScope')}</p>
            <table><thead><tr><th scope="col">{t('kernel.pointRatio.pump')}</th>
              <th scope="col">{t('kernel.dhwTest.tapProfile')}</th><th scope="col">{t('kernel.dhwTest.normVersion')}</th>
              <th scope="col">{t('kernel.dhwTest.ratio')}</th></tr></thead><tbody>
              {assessment.summary.dhwTestDiagnostics?.map((point) => <tr key={`${point.heatPumpId}-${point.pointId}`}>
                <th scope="row">{point.heatPumpId}</th><td>{point.tapProfile}</td>
                <td>{point.declarationNormVersion}</td><td>{formatNumber(point.declaredUsefulToInputRatio, locale, 3)}</td>
              </tr>)}
            </tbody></table>
          </details>}
        {assessment.issues.length > 0 && <ul className="kernel-audit-issues">
          {assessment.issues.map((item, index) => {
            const translated = t(`kernel.issue.${item.code}`);
            return <li key={`${item.path}-${item.code}-${index}`} className={`kernel-audit-issue ${item.severity}`}>
              <span className="kernel-audit-path">{item.path}</span>
              <span>{translated === `kernel.issue.${item.code}` ? item.message : translated}</span>
              <KernelDetail detail={item.detail} />
            </li>;
          })}
        </ul>}
        <p className="kernel-audit-scope">{t('kernel.scope')}</p>
        <details className="kernel-audit-provenance">
          <summary>{t('kernel.provenance')}</summary>
          <dl>
            <dt>{t('kernel.targetNorm')}</dt><dd>{assessment.targetNormVersion}</dd>
            <dt>{t('kernel.version')}</dt><dd>{assessment.kernelVersion}</dd>
            <dt>{t('kernel.inputFingerprint')}</dt><dd><code>{assessment.inputFingerprint}</code></dd>
          </dl>
        </details>
      </>}
    </section>
  );
}
