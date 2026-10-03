import { useEffect, useRef, useState } from 'react';
import { GitCompare } from 'lucide-react';
import { useEnergy } from '../../context/EnergyContext';
import { useI18n } from '../../i18n/i18n';
import { assessRelabelWithRust, type NtaRelabelComparison } from '../../core/nta/KernelClient';
import { deserializeProject } from '../../core/io/ProjectSerializer';
import { sha256Hex } from '../../core/nta/Evidence';
import { labelInputSha256 } from '../../core/nta/Registration';
import { relabelCluster, relabelElementName, relabelNote, relabelValue } from '../../core/nta/RelabelText';
import '../NtaPerformancePanel/NtaPerformancePanel.css';
import './MaatwerkadviesPanel.css';


/**
 * BRL 9500 Bijlage 6a/6b: compares the project with the original label's project file.
 * The comparison is kept with the registration, so the project dossier carries the
 * overview of later changes (Bijlage 3).
 */
export function RelabelPanel() {
  const { state, dispatch } = useEnergy();
  const { t, locale } = useI18n();
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [currentSha, setCurrentSha] = useState<string | null>(null);
  const requestId = useRef(0);
  const fileInput = useRef<HTMLInputElement>(null);
  const stored: NtaRelabelComparison | undefined = state.project.registration?.relabelComparison;
  const result = stored?.assessment;

  useEffect(() => {
    // A comparison still running belongs to the project as it was; drop it.
    requestId.current += 1;
    setBusy(false);
    let active = true;
    void labelInputSha256(state.project).then((sha) => { if (active) setCurrentSha(sha); }).catch(() => undefined);
    return () => { active = false; };
  }, [state.project]);

  const store = (relabelComparison: NtaRelabelComparison | undefined) => {
    const registration = { ...(state.project.registration ?? {}) };
    if (relabelComparison) registration.relabelComparison = relabelComparison;
    else delete registration.relabelComparison;
    dispatch({ type: 'UPDATE_PROJECT_INFO', payload: { registration } });
  };

  const compare = async (file: File) => {
    const current = ++requestId.current;
    setError(null);
    setBusy(true);
    try {
      const text = await file.text();
      const original = deserializeProject(text);
      const assessment = await assessRelabelWithRust(original, state.project);
      const [originalSha256, currentSha256] = await Promise.all([
        sha256Hex(new TextEncoder().encode(text)), labelInputSha256(state.project),
      ]);
      if (requestId.current === current) {
        store({ originalFileName: file.name, originalSha256, currentSha256, comparedAt: new Date().toISOString(),
          originalProjectText: text, assessment });
      }
    } catch (reason) {
      if (requestId.current === current) setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      if (requestId.current === current) setBusy(false);
      if (fileInput.current) fileInput.current.value = '';
    }
  };

  const outdated = Boolean(stored?.currentSha256 && currentSha && stored.currentSha256 !== currentSha);

  return (
    <section className="nta-performance relabel-panel" aria-label={t('relabel.title')}>
      <div className="nta-performance-title"><GitCompare size={18} /><div><h3>{t('relabel.title')}</h3><p>{t('relabel.intro')}</p></div></div>
      <label className="nta-performance-actions">
        {t('relabel.choose')}{' '}
        <input ref={fileInput} type="file" accept=".json,.oes,application/json" disabled={busy}
          onChange={(event) => { const file = event.target.files?.[0]; if (file) void compare(file); }} />
      </label>
      {error && <p role="alert">{error}</p>}
      {result && stored && (
        <>
          <p>
            <strong className={result.allowed ? 'relabel-allowed' : 'relabel-not_allowed'}>
              {result.allowed ? t('relabel.allowed') : t('relabel.notAllowed')}
            </strong>
            {result.needsReview && <> · <span className="relabel-review">{t('relabel.review')}</span></>}
            {' · '}{stored.originalFileName}
            {stored.comparedAt && <> · <time dateTime={stored.comparedAt}>{new Date(stored.comparedAt).toLocaleString(locale)}</time></>}
          </p>
          <p className="dialog-hint">{t('relabel.stored')}</p>
          {outdated && <p role="status" className="relabel-review" data-testid="relabel-outdated">{t('relabel.outdated')}</p>}
          {result.changes.length === 0 ? <p>{t('relabel.noChanges')}</p> : (
            <div className="nta-performance-table">
              <table>
                <thead><tr><th>{t('relabel.col.verdict')}</th><th>{t('relabel.col.cluster')}</th><th>{t('relabel.col.path')}</th>
                  <th>{t('relabel.col.before')}</th><th>{t('relabel.col.after')}</th><th>{t('relabel.col.note')}</th></tr></thead>
                <tbody>
                  {result.changes.map((change, index) => (
                    <tr key={index}>
                      <td className={`relabel-${change.verdict}`}>{t(`relabel.verdict.${change.verdict}`)}</td>
                      <td>{relabelCluster(t, change.cluster)}</td>
                      <td title={change.path}>{relabelElementName(state.project, change.path)}</td>
                      <td>{relabelValue(t, locale, change.before)}</td>
                      <td>{relabelValue(t, locale, change.after)}</td>
                      <td>{relabelNote(t, change.note)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
          <small>{result.scheme === 'u' ? 'BRL 9500-U' : 'BRL 9500-W'} · {result.source}</small>
          <div className="nta-performance-actions">
            <button type="button" className="btn" onClick={() => store(undefined)}>{t('relabel.clear')}</button>
          </div>
        </>
      )}
    </section>
  );
}
