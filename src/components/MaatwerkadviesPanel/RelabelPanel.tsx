import { useEffect, useRef, useState } from 'react';
import { GitCompare } from 'lucide-react';
import { useEnergy } from '../../context/EnergyContext';
import { useI18n } from '../../i18n/i18n';
import { assessRelabelWithRust, type RelabelAssessment } from '../../core/nta/KernelClient';
import { deserializeProject } from '../../core/io/ProjectSerializer';
import { relabelCluster, relabelElementName, relabelNote, relabelValue } from '../../core/nta/RelabelText';
import '../NtaPerformancePanel/NtaPerformancePanel.css';
import './MaatwerkadviesPanel.css';


/** BRL 9500 Bijlage 6a/6b: compares the project with the original label's project file. */
export function RelabelPanel() {
  const { state } = useEnergy();
  const { t, locale } = useI18n();
  const [result, setResult] = useState<RelabelAssessment | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string | null>(null);
  const requestId = useRef(0);
  const fileInput = useRef<HTMLInputElement>(null);

  useEffect(() => {
    requestId.current += 1;
    setResult(null);
    setError(null);
    setFileName(null);
    if (fileInput.current) fileInput.current.value = '';
    return () => { requestId.current += 1; };
  }, [state.project]);

  const compare = async (file: File) => {
    const current = ++requestId.current;
    setError(null);
    setResult(null);
    setFileName(file.name);
    try {
      const original = deserializeProject(await file.text());
      const assessment = await assessRelabelWithRust(original, state.project);
      if (requestId.current === current) setResult(assessment);
    } catch (reason) {
      if (requestId.current === current) setError(reason instanceof Error ? reason.message : String(reason));
    }
  };

  return (
    <section className="nta-performance relabel-panel" aria-label={t('relabel.title')}>
      <div className="nta-performance-title"><GitCompare size={18} /><div><h3>{t('relabel.title')}</h3><p>{t('relabel.intro')}</p></div></div>
      <label className="nta-performance-actions">
        {t('relabel.choose')}{' '}
        <input ref={fileInput} type="file" accept=".json,.oes,application/json"
          onChange={(event) => { const file = event.target.files?.[0]; if (file) void compare(file); }} />
      </label>
      {error && <p role="alert">{error}</p>}
      {result && (
        <>
          <p>
            <strong className={result.allowed ? 'relabel-allowed' : 'relabel-not_allowed'}>
              {result.allowed ? t('relabel.allowed') : t('relabel.notAllowed')}
            </strong>
            {result.needsReview && <> · <span className="relabel-review">{t('relabel.review')}</span></>}
            {fileName && <> · {fileName}</>}
          </p>
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
        </>
      )}
    </section>
  );
}
