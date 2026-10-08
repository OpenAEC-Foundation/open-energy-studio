/**
 * Herlabelen of a basisopname project (feedback 8 Oct 2026): the original
 * label's project file, the changes in the survey in words with their BRL 9500
 * Bijlage 6a/6b verdict, the evidence per change and a checklist. The original
 * label answers the relabel fields of the registration (`relabelTakeovers`).
 */
import { useEffect, useRef, useState } from 'react';
import { CheckCircle2, CircleX, Clock } from 'lucide-react';
import './SurveyWizard.css';
import { useI18n } from '../../i18n/i18n';
import { formatNumber } from '../../i18n/format';
import { useEnergy } from '../../context/EnergyContext';
import { assessSurvey, currentResult, useSurveyAssessment } from '../../core/survey/surveyAssessment';
import { questionForPath } from '../../core/survey/surveyFlow';
import {
  originalSurveyDate, relabelRows, relabelTakeovers, rowProofs, worst, type RelabelRow,
} from '../../core/survey/surveyRelabel';
import { surveyRegistration } from '../../core/survey/surveyRegistration';
import {
  assessRelabelWithRust, assessResidentialSurveyWithRust, assessUtilitySurveyWithRust,
  type NtaEvidenceItem, type NtaRegistration, type NtaRelabelProof, type OpnameAssessment, type RelabelChange,
} from '../../core/nta/KernelClient';
import { asResidential, asUtility, type StoredSurvey } from '../../core/nta/SurveyTemplates';
import { deserializeProject } from '../../core/io/ProjectSerializer';
import { createEvidenceItem, sha256Hex } from '../../core/nta/Evidence';
import { labelInputSha256, originalProjectTextForStorage, softwareIdentity } from '../../core/nta/Registration';
import { relabelNote } from '../../core/nta/RelabelText';
import type { Route } from '../../core/navigation/routes';
import type { IProject } from '../../core/energy/types';
import { FileButton } from '../ui';
import type { ShellActions } from '../shell/ShellActions';

type T = (key: string, values?: Record<string, string | number>) => string;
type Json = Record<string, unknown>;
const PROOFS: NtaRelabelProof[] = ['quote_with_order', 'specified_invoice', 'production_photo'];
const SECTION_STEP: Record<string, string> = {
  heating: 'survey.step.verwarming', hotWater: 'survey.step.warm-water', ventilation: 'survey.step.ventilatie',
  cooling: 'survey.step.koeling',
};

function read(value: unknown, path: string): unknown {
  return path.split(/\.|\[(\d+)\]/).filter(Boolean).reduce<unknown>(
    (node, key) => (node != null && typeof node === 'object' ? (node as Json)[key] : undefined), value);
}

/** A survey answer in words: "120 mm", "HR++-glas", "warmtepomp". */
function valueText(t: T, locale: string, value: unknown): string {
  if (value === undefined || value === null || value === '') return '—';
  if (typeof value === 'boolean') return t(value ? 'common.yes' : 'common.no');
  if (typeof value === 'number') return formatNumber(value, locale, Number.isInteger(value) ? 0 : 1);
  if (typeof value === 'string') {
    const key = `opname.value.${value}`;
    const text = t(key);
    return text === key ? value.replace(/_/g, ' ') : text;
  }
  if (Array.isArray(value)) return String(value.length);
  const object = value as Json;
  if (object.kind === 'thickness' && typeof object.thicknessMm === 'number') return `${formatNumber(object.thicknessMm, locale, 0)} mm`;
  if (typeof object.kind === 'string') return valueText(t, locale, object.kind);
  return t('surveyRelabel.changed');
}

/** Names of the surfaces as in the question flow ("Dak 1") by id. */
function surfaceNames(survey: Json | undefined, t: T): Map<string, string> {
  const counts: Record<string, number> = {};
  const names = new Map<string, string>();
  for (const surface of ((read(survey, 'envelope.surfaces') as Json[] | undefined) ?? [])) {
    const element = String(surface.element ?? 'facade');
    counts[element] = (counts[element] ?? 0) + 1;
    names.set(String(surface.id ?? ''), `${t(`survey.element.${element}`)} ${counts[element]}`);
  }
  return names;
}

function rowSubject(row: RelabelRow, before: Json | undefined, after: Json | undefined, t: T): string {
  const item = (read(after, row.path) ?? read(before, row.path)) as Json | undefined;
  const names = new Map([...surfaceNames(before, t), ...surfaceNames(after, t)]);
  const [, list, index] = /^envelope\.(\w+)\[(\d+)\]$/.exec(row.path) ?? [];
  if (list === 'surfaces') return names.get(String(item?.id ?? '')) ?? `${t('surveyRelabel.surface')} ${Number(index) + 1}`;
  if (list) {
    const host = names.get(String(item?.surfaceId ?? ''));
    return host ? t(`surveyRelabel.${list}In`, { surface: host }) : `${t(`surveyRelabel.${list}`)} ${Number(index) + 1}`;
  }
  const pv = /^pv\[(\d+)\]$/.exec(row.path);
  if (pv) return `${t('survey.step.zonnepanelen')} ${Number(pv[1]) + 1}`;
  const section = SECTION_STEP[row.path];
  if (section) return t(section);
  const key = `opname.section.${row.path}`;
  return t(key) === key ? row.path : t(key);
}

const ICON = { done: CheckCircle2, open: CircleX, waiting: Clock };

export function SurveyRelabel({ actions }: { actions: Pick<ShellActions, 'navigate'> }) {
  const { t, locale } = useI18n();
  const { state, dispatch } = useEnergy();
  const project = state.project;
  const stored = project.basisopname as (StoredSurvey & { surveyDate?: string }) | undefined;
  const registration = surveyRegistration(project);
  const comparison = registration.relabelComparison;
  const assessment = useSurveyAssessment();
  const surveyJson = stored ? JSON.stringify(stored.survey) : null;
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentSha, setCurrentSha] = useState<string | null>(null);
  const [originalResult, setOriginalResult] = useState<OpnameAssessment | null>(null);
  const [proofRole, setProofRole] = useState<NtaRelabelProof>('specified_invoice');
  const request = useRef(0);

  useEffect(() => {
    if (stored) void assessSurvey(stored);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [surveyJson]);
  useEffect(() => {
    let live = true;
    void labelInputSha256(project).then((sha) => { if (live) setCurrentSha(sha); }).catch(() => undefined);
    return () => { live = false; };
  }, [project]);

  let original: IProject | null = null;
  try { original = comparison?.originalProjectText ? deserializeProject(comparison.originalProjectText) : null; } catch { original = null; }
  const originalStored = original?.basisopname as StoredSurvey | undefined;
  const originalJson = originalStored ? JSON.stringify(originalStored.survey) : null;
  // The original label: its survey calculated again with this kernel.
  useEffect(() => {
    let live = true;
    setOriginalResult(null);
    if (originalStored) {
      const run = originalStored.kind === 'residential'
        ? assessResidentialSurveyWithRust(asResidential(originalStored)) : assessUtilitySurveyWithRust(asUtility(originalStored));
      void run.then((result) => { if (live) setOriginalResult(result); }).catch(() => undefined);
    }
    return () => { live = false; };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [originalJson]);

  if (!stored) return null;
  const set = (patch: Partial<NtaRegistration>) =>
    dispatch({ type: 'UPDATE_PROJECT_INFO', payload: { registration: { ...registration, ...patch } } });

  const compare = async (file: File) => {
    const id = ++request.current;
    setBusy(true);
    setError(null);
    try {
      const text = originalProjectTextForStorage(await file.text());
      const loaded = deserializeProject(text);
      if (!(loaded.basisopname as StoredSurvey | undefined)?.survey) throw new Error(t('surveyRelabel.noSurvey'));
      const result = await assessRelabelWithRust(loaded, project);
      const [originalSha256, currentSha256] = await Promise.all([
        sha256Hex(new TextEncoder().encode(text)), result.currentLabelInputHash ?? labelInputSha256(project),
      ]);
      if (id !== request.current) return;
      set({
        ...relabelTakeovers(loaded, file.name),
        relabelComparison: { originalFileName: file.name, originalSha256, currentSha256, comparedAt: new Date().toISOString(),
          originalProjectText: text, assessment: result },
      });
    } catch (reason) {
      if (id === request.current) setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      if (id === request.current) setBusy(false);
    }
  };
  const recompare = async () => {
    if (!original || !comparison) return;
    setBusy(true);
    try {
      const result = await assessRelabelWithRust(original, project);
      set({ relabelComparison: { ...comparison, assessment: result, currentSha256: result.currentLabelInputHash ?? comparison.currentSha256,
        comparedAt: new Date().toISOString() } });
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  };
  const addProof = async (row: RelabelRow, files: FileList | null) => {
    if (!files?.length) return;
    let evidence: NtaEvidenceItem[] = [...(registration.evidence ?? [])];
    for (const file of Array.from(files)) {
      const item = await createEvidenceItem({ name: file.name, bytes: new Uint8Array(await file.arrayBuffer()), lastModified: file.lastModified }, evidence);
      evidence = [...evidence, { ...item, relabelProof: proofRole, linkedPaths: [row.pointer] }];
    }
    set({ evidence });
  };

  const changes: RelabelChange[] = comparison?.assessment?.changes ?? [];
  const rows = relabelRows(changes);
  const other = changes.filter((change) => !change.path.startsWith('/basisopname/survey'));
  const outdated = Boolean(comparison?.currentSha256 && currentSha && comparison.currentSha256 !== currentSha);
  const before = originalStored?.survey as Json | undefined;
  const after = stored.survey as Json;
  const count = (verdict: RelabelChange['verdict']) => rows.filter((row) => row.verdict === verdict).length;
  const blocked = rows.some((row) => row.verdict === 'not_allowed') || other.some((change) => change.verdict === 'not_allowed');
  const production = rows.some((row) => row.production);
  const proofsMissing = rows.filter((row) => row.verdict !== 'not_allowed' && !rowProofs(row, registration.evidence).complete).length;
  const result = currentResult(assessment, stored);
  const go = (path: string) => {
    const target = questionForPath(path, stored);
    actions.navigate({ step: 'survey', sub: target.step, ...(target.question ? { question: target.question } : {}) } as Route);
  };
  const date = (value: string | undefined) => (value ? new Date(value).toLocaleDateString(locale) : '—');
  const originalDate = original ? originalSurveyDate(original) : undefined;
  const checklist: Array<{ id: string; state: 'done' | 'open' | 'waiting'; detail?: string }> = [
    { id: 'original', state: comparison ? 'done' : 'open', detail: comparison?.originalFileName },
    { id: 'current', state: comparison && !outdated ? 'done' : 'open' },
    { id: 'surveyDate', state: originalDate && registration.surveyDate === originalDate ? 'done' : 'open', detail: date(originalDate) },
    { id: 'noExcluded', state: comparison && !blocked ? 'done' : 'open' },
    { id: 'confirmed', state: registration.noExcludedChangesConfirmed ? 'done' : 'open' },
    { id: 'proofs', state: comparison && proofsMissing === 0 ? 'done' : 'open',
      detail: comparison ? t('surveyRelabel.proofsDetail', { count: proofsMissing }) : undefined },
    ...(production ? [{ id: 'connected', state: registration.productionPhysicallyConnected ? 'done' as const : 'open' as const }] : []),
    { id: 'attest', state: softwareIdentity().attestNumber ? 'done' : 'waiting' },
  ];
  const label = (value: OpnameAssessment | null) => value?.performance?.indicativeLabelClass ?? '—';
  const ep2 = (value: OpnameAssessment | null) => formatNumber(value?.performance?.primaryFossilIndicatorKwhPerM2Year ?? null, locale, 1);

  return <div className="survey-wizard survey-registration survey-relabel">
    <div className="survey-main">
      <h1 id="page-title" tabIndex={-1}>{t('surveyRelabel.title')}</h1>
      <p className="survey-lead">{t('surveyRelabel.lead')}</p>

      <section className="survey-card" aria-labelledby="srel-original">
        <h2 id="srel-original" className="survey-overline">{t('surveyRelabel.original')}</h2>
        {comparison ? <dl className="survey-reg-kv">
          <dt>{t('surveyRelabel.file')}</dt><dd>{comparison.originalFileName} · <span className="mono">{comparison.originalSha256?.slice(0, 8)}…</span></dd>
          <dt>{t('survey.surveyDate')}</dt><dd>{date(originalDate)} {t('surveyRelabel.dateKept')}</dd>
          <dt>{t('surveyRelabel.epOnline')}</dt><dd>{registration.originalEpOnlineNumber ?? '—'}</dd>
          <dt>{t('surveyRelabel.labelThen')}</dt><dd>{label(originalResult)} · EP2 {ep2(originalResult)}</dd>
        </dl> : <p className="survey-muted">{t('surveyRelabel.originalHint')}</p>}
        <div className="survey-relabel-file">
          <FileButton id="srel-file" accept=".json,.oes,application/json" disabled={busy}
            label={t(comparison ? 'surveyRelabel.otherFile' : 'surveyRelabel.chooseFile')}
            onChange={(event) => { const file = event.target.files?.[0]; event.target.value = ''; if (file) void compare(file); }} />
          {busy && <span role="status">{t('surveyRelabel.busy')}</span>}
        </div>
        {error && <p className="survey-issues-note" role="alert">{error}</p>}
      </section>

      {comparison && <section className="survey-card" aria-labelledby="srel-changes">
        <h2 id="srel-changes" className="survey-overline">{t('surveyRelabel.changes')}
          <span className="survey-reg-count">{t('surveyRelabel.counts', { allowed: count('allowed'), notAllowed: count('not_allowed'), review: count('review') })}</span></h2>
        {outdated && <p className="survey-issues-note" role="status">{t('surveyRelabel.outdated')}{' '}
          <button type="button" className="btn" onClick={() => void recompare()} disabled={busy}>{t('surveyRelabel.recompare')}</button></p>}
        {rows.length === 0 && other.length === 0 && <p className="survey-muted">{t('surveyRelabel.noChanges')}</p>}
        <ul className="survey-reg-list survey-relabel-rows">
          {rows.map((row) => {
            const notes = [...new Set(row.lines.flatMap((line) => line.changes.map((change) => relabelNote(t, change.note)).filter(Boolean)))];
            return <li key={row.path} data-verdict={row.verdict}>
              <span className={`survey-relabel-chip survey-relabel-chip--${row.verdict}`}>{t(`surveyRelabel.verdict.${row.verdict}`)}</span>
              <span className="survey-reg-text"><strong>{rowSubject(row, before, after, t)}</strong>
                {row.lines.map((line) => <small key={line.path} className="survey-relabel-line">
                  {line.field ? `${t(`surveyRelabel.field.${line.field}`) === `surveyRelabel.field.${line.field}` ? line.field : t(`surveyRelabel.field.${line.field}`)}: ` : ''}
                  {line.field ? `${valueText(t, locale, read(before, line.path))} → ${valueText(t, locale, read(after, line.path))}`
                    : t(read(after, line.path) == null ? 'surveyRelabel.removed' : 'surveyRelabel.added')}
                  {row.lines.length > 1 && line.verdict !== row.verdict && ` (${t(`surveyRelabel.verdict.${worst([line.verdict])}`)})`}
                </small>)}
                {notes.map((note) => <small key={note} className="survey-relabel-note">{note}</small>)}
              </span>
              <button type="button" className="btn" onClick={() => go(row.path)}>{t('surveyReg.goTo')}</button>
            </li>;
          })}
          {other.map((change) => <li key={change.path} data-verdict={change.verdict}>
            <span className={`survey-relabel-chip survey-relabel-chip--${change.verdict}`}>{t(`surveyRelabel.verdict.${change.verdict}`)}</span>
            <span className="survey-reg-text">{change.path === '/registration/surveyDate' ? t('surveyRelabel.surveyDateChanged') : change.path}</span>
          </li>)}
        </ul>
      </section>}

      {comparison && rows.some((row) => row.verdict !== 'not_allowed') && <section className="survey-card" aria-labelledby="srel-proofs">
        <h2 id="srel-proofs" className="survey-overline">{t('surveyRelabel.proofs')}</h2>
        <p className="survey-muted">{t('surveyRelabel.proofsHint')}</p>
        <label className="survey-relabel-role">{t('surveyRelabel.role')}
          <select value={proofRole} onChange={(event) => setProofRole(event.target.value as NtaRelabelProof)}>
            {PROOFS.map((role) => <option key={role} value={role}>{t(`evidence.relabelProof.${role}`)}</option>)}
          </select>
        </label>
        <ul className="survey-reg-list">
          {rows.filter((row) => row.verdict !== 'not_allowed').map((row) => {
            const proofs = rowProofs(row, registration.evidence);
            return <li key={row.path} data-state={proofs.complete ? 'done' : 'open'}>
              {proofs.complete ? <CheckCircle2 aria-hidden="true" /> : <CircleX aria-hidden="true" />}
              <span className="survey-reg-text">{rowSubject(row, before, after, t)}
                <small>{proofs.linked.length > 0 ? proofs.linked.map((item) => item.fileName).join(', ')
                  : t(row.production ? 'surveyRelabel.needsPhoto' : 'surveyRelabel.needsProof')}</small></span>
              <FileButton id={`srel-proof-${row.path}`} multiple label={t('surveyRelabel.addProof')}
                onChange={(event) => { const files = event.target.files; void addProof(row, files).finally(() => { event.target.value = ''; }); }} />
            </li>;
          })}
        </ul>
        <label className="dialog-check"><input type="checkbox" checked={registration.noExcludedChangesConfirmed ?? false}
          onChange={(event) => set({ noExcludedChangesConfirmed: event.target.checked })} />{t('surveyRelabel.confirm')}</label>
        {production && <label className="dialog-check"><input type="checkbox" checked={registration.productionPhysicallyConnected ?? false}
          onChange={(event) => set({ productionPhysicallyConnected: event.target.checked })} />{t('surveyRelabel.connected')}</label>}
      </section>}
    </div>

    <aside className="survey-aside" aria-label={t('surveyRelabel.aside')}>
      <div className="survey-card">
        <span className="survey-overline">{t('surveyRelabel.newLabel')}</span>
        {!comparison ? <p className="survey-muted">{t('surveyRelabel.newLabelPending')}</p>
          : blocked ? <p className="survey-final-facts">{t('surveyRelabel.blocked')}</p>
            : <p className="survey-final-facts"><span className="survey-relabel-label">{label(result)}</span><br />
              EP2 {ep2(originalResult)} → {ep2(result)}</p>}
      </div>
      <div className="survey-card">
        <span className="survey-overline">{t('surveyRelabel.ready')}</span>
        <ul className="survey-reg-list survey-relabel-check">
          {checklist.map((item) => {
            const Icon = ICON[item.state];
            return <li key={item.id} data-state={item.state}><Icon aria-hidden="true" />
              <span className="survey-reg-text">{t(`surveyRelabel.check.${item.id}`)}{item.detail && <small>{item.detail}</small>}</span>
              <span className="sr-only">{t(`surveyReg.state.${item.state}`)}</span></li>;
          })}
        </ul>
        <button type="button" className="btn" onClick={() => actions.navigate({ step: 'registration' } as Route)}>{t('surveyRelabel.toRegistration')}</button>
      </div>
    </aside>
  </div>;
}
