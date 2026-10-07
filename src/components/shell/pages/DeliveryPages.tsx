/**
 * Step 9 Rapport & dossier (UI redesign F9, mockup 06): the frame around the
 * report builder, the input summary, the BRL 9500 appendix 3 dossier check
 * with the evidence register and the EP-Online data overview, and every
 * export. The report content itself stays with `ReportView`/`ReportBuilder`.
 */
import { useState, type ReactNode } from 'react';
import { Box, Download, FileArchive, FileDown, FileText, Pencil, Printer } from 'lucide-react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { useKernelQuery } from '../../../context/KernelProvider';
import { downloadNtaCalculationReportHTML, downloadNtaInputDossierHTML } from '../../../core/report/ReportGenerator';
import type { DossierItem, DossierStatus } from '../../../core/report/ProjectDossier';
import { buildEpOnlineOverview, EP_ONLINE_FIELDS, type EpOnlineValue } from '../../../core/report/EpOnlineOverview';
import { LazyPage, ReportView } from '../lazyPages';
import { useDossier } from '../../ReportView/useDossier';
import { Banner, Button, Card, EmptyState, Pill, StatusPill } from '../../ui';
import type { ShellActions } from '../ShellActions';
import {
  danglingEvidenceReferences, evidenceUsage, pointerToPath, resolvePointer, unresolvedEvidenceLinks,
} from '../../../core/nta/EvidenceLinks';
import { routeForPath } from '../../../core/nta/gapRoutes';
import './delivery.css';

const GROUPS: DossierItem['group'][] = [
  'general', 'building', 'installations', 'elaboration', 'representativity', 'result', 'evidence', 'relabel',
];

/** Status pill of a dossier item; `pending` waits for the kernel run. */
export function DossierStatusPill({ status }: { status: DossierStatus }) {
  const { t } = useI18n();
  const label = t(`report.dossierStatus.${status}`);
  if (status === 'not_applicable') return <Pill>{label}</Pill>;
  const tone = status === 'ok' ? 'ok' : status === 'missing' ? 'err' : status === 'check' ? 'warn' : 'info';
  return <StatusPill tone={tone}>{label}</StatusPill>;
}

function useDeliveryContext() {
  const { state } = useEnergy();
  const project = state.project;
  const query = useKernelQuery(project);
  const pending = query == null || query.kind === 'loading';
  const assessment = query?.kind === 'done' ? query.assessment : null;
  return { project, pending, assessment };
}

/** Rekenrapport: verification notice and the report builder (level, language, chapters, preview). */
export function ReportPage() {
  return <LazyPage><ReportView section="report" /></LazyPage>;
}

/** Invoerdossier: export of the NTA input dossier plus the input summary. */
export function InputDossierPage() {
  const { t } = useI18n();
  const { state } = useEnergy();
  return <>
    <Card title={t('delivery.input.title')} subtitle={t('report.inputDossierScope')} level={2}
      actions={<Button icon={<FileDown aria-hidden="true" />} onClick={() => downloadNtaInputDossierHTML(state.project)}>
        {t('report.exportInputDossier')}</Button>} />
    <LazyPage><ReportView section="input" /></LazyPage>
  </>;
}

function formatValue(value: EpOnlineValue | undefined, yes: string, no: string): string {
  if (value == null) return '—';
  if (typeof value === 'boolean') return value ? yes : no;
  if (Array.isArray(value)) return value.join(', ');
  return String(value);
}

/** Checklist BRL 9500: dossier items per group, the ZIP export, evidence and the EP-Online overview. */
export function DossierPage({ actions }: { actions: Pick<ShellActions, 'navigate'> }) {
  const { t } = useI18n();
  const { project, pending, assessment } = useDeliveryContext();
  const dossier = useDossier(project, assessment, pending);
  const { checklist, open } = dossier;
  const count = (status: DossierStatus) => checklist.filter((item) => item.status === status).length;
  const done = count('ok') + count('not_applicable');
  const evidence = project.registration?.evidence ?? [];
  const usage = evidenceUsage(project);
  const dangling = danglingEvidenceReferences(project);
  const unresolved = unresolvedEvidenceLinks(project);
  const overview = buildEpOnlineOverview(project, assessment);
  const yes = t('registration.page.yes');
  const no = t('registration.page.no');

  return <>
    <Card title={t('delivery.dossier.title')} subtitle={t('delivery.dossier.subtitle')} level={2}
      className="delivery-dossier-summary"
      actions={<Button variant="primary" icon={<FileArchive aria-hidden="true" />} onClick={dossier.exportDossier}
        disabled={dossier.busy}>{t('report.exportProjectDossier')}</Button>}>
      <div className="delivery-meter" role="group" aria-label={t('delivery.dossier.progress', { done, total: checklist.length })}>
        <div className="delivery-meter__value"><strong>{done}</strong> / {checklist.length}</div>
        <div className="delivery-meter__bar" aria-hidden="true">
          <span style={{ width: `${checklist.length ? (100 * done) / checklist.length : 0}%` }} />
        </div>
        <ul className="delivery-meter__counts">
          <li><DossierStatusPill status="missing" /> {count('missing')}</li>
          <li><DossierStatusPill status="check" /> {count('check')}</li>
          <li><DossierStatusPill status="pending" /> {count('pending')}</li>
          <li><DossierStatusPill status="ok" /> {count('ok')}</li>
        </ul>
      </div>
      <p className="delivery-note">{t('report.projectDossierScope')}</p>
      {open.length === 0 && checklist.length > 0 && <Banner tone="info" role="status">{t('delivery.dossier.allDone')}</Banner>}
      {dossier.error && <Banner tone="err">{dossier.error}</Banner>}
      {(dossier.missingEvidence ?? 0) > 0 && (
        <Banner tone="warn" role="status">{t('report.dossierMissingEvidence', { count: dossier.missingEvidence ?? 0 })}</Banner>
      )}
    </Card>

    {GROUPS.map((group) => {
      const items = checklist.filter((item) => item.group === group);
      if (items.length === 0) return null;
      const groupOpen = items.filter((item) => open.includes(item)).length;
      return (
        <Card key={group} level={3} flush title={t(`delivery.group.${group}`)}
          actions={groupOpen > 0 ? <Pill tone="warn">{t('delivery.dossier.groupOpen', { count: groupOpen })}</Pill> : <Pill tone="ok">{t('report.dossierStatus.ok')}</Pill>}>
          <table className="delivery-checklist">
            <caption className="visually-hidden">{t(`delivery.group.${group}`)}</caption>
            <thead><tr>
              <th scope="col">{t('report.status')}</th>
              <th scope="col">{t('delivery.dossier.item')}</th>
              <th scope="col">{t('delivery.dossier.detail')}</th>
            </tr></thead>
            <tbody>
              {items.map((item) => (
                <tr key={item.id} data-status={item.status}>
                  <td><DossierStatusPill status={item.status} /></td>
                  <td>{item.label}</td>
                  <td className="delivery-checklist__detail">{item.detail ?? ''}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </Card>
      );
    })}

    <Card title={t('evidence.title')} subtitle={t('delivery.evidence.subtitle')} level={2} flush
      actions={<Button size="sm" variant="ghost" icon={<Pencil aria-hidden="true" />}
        onClick={() => actions.navigate({ step: 'registration', focusPath: 'registration.evidence' })}>
        {t('delivery.evidence.edit')}</Button>}>
      {dangling.length > 0 && <Banner tone="warn">{t('evidenceLink.dangling', { count: dangling.length })}</Banner>}
      {unresolved.length > 0 && <Banner tone="warn">{t('evidenceLink.unresolved', { count: unresolved.length })}</Banner>}
      {evidence.length === 0
        ? <EmptyState title={t('delivery.evidence.empty')} />
        : (
          <table className="delivery-checklist">
            <thead><tr>
              <th scope="col">{t('evidence.kind')}</th>
              <th scope="col">{t('delivery.evidence.file')}</th>
              <th scope="col">{t('evidence.date')}</th>
              <th scope="col">SHA-256</th>
              <th scope="col">{t('evidenceLink.supports')}</th>
            </tr></thead>
            <tbody>
              {evidence.map((item) => (
                <tr key={item.id}>
                  <td>{t(`evidence.kind.${item.kind}`)}</td>
                  <td>{item.fileName}</td>
                  <td>{item.date ?? '—'}</td>
                  <td className="mono" title={item.sha256}>{item.sha256.slice(0, 12)}…</td>
                  <td>
                    {(usage.get(item.id) ?? []).length === 0
                      ? <Pill tone="warn">{t('evidenceLink.unused')}</Pill>
                      : <ul className="delivery-evidence-links">
                        {(usage.get(item.id) ?? []).map((pointer) => <li key={pointer}>
                          {resolvePointer(project, pointer) == null
                            ? <><code>{pointerToPath(pointer)}</code> <Pill tone="warn">{t('evidenceLink.unresolvedItem')}</Pill></>
                            : <button type="button" className="ui-btn ui-btn--ghost ui-btn--sm"
                              onClick={() => actions.navigate(routeForPath(pointerToPath(pointer, project)))}>
                              <code>{pointerToPath(pointer)}</code>
                            </button>}
                        </li>)}
                      </ul>}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
    </Card>

    <Card title={t('delivery.epOnline.title')} subtitle={t('delivery.epOnline.subtitle')} level={2} flush
      className="delivery-ep-online">
      {overview.missingRequired.length > 0 && (
        <div className="delivery-ep-online__missing">
          <span>{t('delivery.epOnline.missing')}</span>
          {overview.missingRequired.map((field) => <Pill key={field} tone="warn">{field}</Pill>)}
        </div>
      )}
      <table className="delivery-checklist" data-testid="ep-online-overview">
        <thead><tr>
          <th scope="col">{t('delivery.epOnline.field')}</th>
          <th scope="col">{t('report.value')}</th>
        </tr></thead>
        <tbody>
          {EP_ONLINE_FIELDS.filter(([field]) => field in overview.pandcertificaat).map(([field]) => (
            <tr key={field}>
              <td className="mono">{field}</td>
              <td>{formatValue(overview.pandcertificaat[field], yes, no)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </Card>
  </>;
}

function ExportCard({ icon, title, hint, children }: { icon: ReactNode; title: string; hint: string; children: ReactNode }) {
  return (
    <Card level={2} className="delivery-export" title={<span className="delivery-export__title">{icon}{title}</span>}>
      <p className="delivery-note">{hint}</p>
      <div className="delivery-export__actions">{children}</div>
    </Card>
  );
}

/** Exports: the reports, the dossier and the exchange formats in one place. */
export function ExportsPage({ actions }: {
  actions: Pick<ShellActions, 'exportReport' | 'printReport' | 'exportIFC' | 'exportModelIFC' | 'exportUNIEC3' | 'exportVABI'>;
}) {
  const { t } = useI18n();
  const { project, pending, assessment } = useDeliveryContext();
  const dossier = useDossier(project, assessment, pending);
  const [error, setError] = useState<string | null>(null);
  const exportCalculation = () => {
    setError(null);
    downloadNtaCalculationReportHTML(project).catch((reason: unknown) =>
      setError(reason instanceof Error ? reason.message : String(reason)));
  };
  return <>
    {(error ?? dossier.error) && <Banner tone="err">{error ?? dossier.error}</Banner>}
    <div className="delivery-exports">
      <ExportCard icon={<FileText aria-hidden="true" />} title={t('delivery.exports.report')} hint={t('delivery.exports.reportHint')}>
        <Button icon={<FileDown aria-hidden="true" />} onClick={actions.exportReport}>{t('report.export')}</Button>
        <Button icon={<Printer aria-hidden="true" />} onClick={actions.printReport}>{t('report.print')}</Button>
      </ExportCard>
      <ExportCard icon={<FileText aria-hidden="true" />} title={t('delivery.exports.calculation')} hint={t('report.ntaCalculationScope')}>
        <Button icon={<FileDown aria-hidden="true" />} onClick={exportCalculation}>{t('report.exportNtaCalculation')}</Button>
      </ExportCard>
      <ExportCard icon={<FileText aria-hidden="true" />} title={t('delivery.exports.input')} hint={t('report.inputDossierScope')}>
        <Button icon={<FileDown aria-hidden="true" />} onClick={() => downloadNtaInputDossierHTML(project)}>{t('report.exportInputDossier')}</Button>
      </ExportCard>
      <ExportCard icon={<FileArchive aria-hidden="true" />} title={t('delivery.exports.dossier')} hint={t('report.projectDossierScope')}>
        <Button icon={<FileArchive aria-hidden="true" />} onClick={dossier.exportDossier} disabled={dossier.busy}>
          {t('report.exportProjectDossier')}</Button>
      </ExportCard>
      <ExportCard icon={<Download aria-hidden="true" />} title={t('delivery.exports.exchange')} hint={t('delivery.exports.exchangeHint')}>
        <Button icon={<Download aria-hidden="true" />} onClick={actions.exportUNIEC3}>{t('ribbon.exportUNIEC3Draft')}</Button>
        <Button icon={<Download aria-hidden="true" />} onClick={actions.exportVABI}>{t('ribbon.exportVABI')}</Button>
      </ExportCard>
      <ExportCard icon={<Box aria-hidden="true" />} title={t('delivery.exports.ifc')} hint={t('delivery.exports.ifcHint')}>
        <Button icon={<Download aria-hidden="true" />} onClick={actions.exportIFC}>{t('report.page.ifc')}</Button>
        <Button icon={<Box aria-hidden="true" />} onClick={actions.exportModelIFC}>{t('ribbon.exportModelIFC')}</Button>
      </ExportCard>
    </div>
  </>;
}
