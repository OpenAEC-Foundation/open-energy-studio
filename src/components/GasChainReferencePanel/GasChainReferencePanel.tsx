import { useEffect, useRef, useState } from 'react';
import syntheticCase from '../../../training-data/nta8800-gas-chain-diagnostic-synthetic.json';
import { compareGasHeatPumpChainDiagnosticWithRust, type GasChainDiagnosticCase, type GasChainDiagnosticComparison } from '../../core/nta/KernelClient';
import { useI18n } from '../../i18n/i18n';
import './GasChainReferencePanel.css';

export function GasChainReferencePanel() {
  const { t, locale } = useI18n();
  const [text, setText] = useState('');
  const [result, setResult] = useState<GasChainDiagnosticComparison | null>(null);
  const [comparedSource, setComparedSource] = useState<GasChainDiagnosticCase['source'] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const sequence = useRef(0);
  useEffect(() => () => { sequence.current += 1; }, []);

  const setCaseText = (value: string) => {
    sequence.current += 1;
    setText(value);
    setResult(null);
    setComparedSource(null);
    setError(null);
    setLoading(false);
  };

  const loadFile = async (file: File | undefined) => {
    if (!file) return;
    const current = ++sequence.current;
    setResult(null);
    setComparedSource(null);
    setError(null);
    setLoading(true);
    try {
      const content = await file.text();
      if (sequence.current === current) {
        setText(content);
        setLoading(false);
      }
    } catch (reason) {
      if (sequence.current === current) {
        setError(String(reason));
        setLoading(false);
      }
    }
  };

  const compare = async () => {
    let caseInput: GasChainDiagnosticCase;
    try {
      const parsed: unknown = JSON.parse(text);
      if (parsed === null || typeof parsed !== 'object' || Array.isArray(parsed)) throw new Error(t('kernel.gasReference.jsonObject'));
      caseInput = parsed as GasChainDiagnosticCase;
    } catch (reason) {
      setError(String(reason));
      setResult(null);
      setComparedSource(null);
      return;
    }
    const current = ++sequence.current;
    setLoading(true);
    setError(null);
    setResult(null);
    setComparedSource(null);
    try {
      const assessment = await compareGasHeatPumpChainDiagnosticWithRust(caseInput);
      if (sequence.current === current) {
        setResult(assessment);
        const source = caseInput.source;
        setComparedSource(assessment.status !== 'invalid_case' && source
          && typeof source.publisher === 'string' && typeof source.documentId === 'string'
          && typeof source.edition === 'string' && typeof source.independentReviewer === 'string'
          ? source : null);
        setLoading(false);
      }
    } catch (reason) {
      if (sequence.current === current) {
        setError(String(reason));
        setLoading(false);
      }
    }
  };

  return <details className="gas-chain-reference" aria-label={t('kernel.gasReference.title')}>
    <summary>{t('kernel.gasReference.title')}</summary>
    <div className="gas-chain-reference-content">
      <p>{t('kernel.gasReference.scope')}</p>
      <div className="gas-chain-reference-actions">
        <button type="button" onClick={() => setCaseText(JSON.stringify(syntheticCase, null, 2))}>
          {t('kernel.gasReference.example')}
        </button>
        <label>{t('kernel.gasReference.file')}
          <input type="file" accept=".json,application/json" onChange={(event) => void loadFile(event.target.files?.[0])} />
        </label>
      </div>
      <label>{t('kernel.gasReference.json')}
        <textarea value={text} rows={9} spellCheck={false} onChange={(event) => setCaseText(event.target.value)} />
      </label>
      <button type="button" disabled={loading || !text.trim()} onClick={() => void compare()}>
        {loading ? t('kernel.gasReference.loading') : t('kernel.gasReference.compare')}
      </button>
      {error && <p role="alert" className="gas-chain-reference-error">{error}</p>}
      {result && <div role="status" className="gas-chain-reference-result" data-status={result.status}>
        <p><strong>{t(`kernel.gasReference.status.${result.status}`)}</strong> · {result.caseId}</p>
        <p>{t('kernel.gasReference.unverified')}</p>
        {comparedSource && <dl className="gas-chain-reference-source" aria-label={locale === 'nl' ? 'Herkomst referentiecase' : 'Reference case provenance'}>
          <div><dt>{locale === 'nl' ? 'Uitgever' : 'Publisher'}</dt><dd>{comparedSource.publisher}</dd></div>
          <div><dt>{locale === 'nl' ? 'Document' : 'Document'}</dt><dd>{comparedSource.documentId}</dd></div>
          <div><dt>{locale === 'nl' ? 'Editie' : 'Edition'}</dt><dd>{comparedSource.edition}</dd></div>
          <div><dt>{locale === 'nl' ? 'Onafhankelijke controleur' : 'Independent reviewer'}</dt><dd>{comparedSource.independentReviewer}</dd></div>
        </dl>}
        {result.issues.length > 0 && <ul>{result.issues.map((item, index) =>
          <li key={`${item.path}-${item.code}-${index}`}><code>{item.path}</code>: {item.code}</li>)}</ul>}
        {result.metrics.length > 0 && <div className="gas-chain-reference-table-wrap"><table>
          <thead><tr><th>{t('kernel.gasReference.metric')}</th><th>{t('kernel.gasReference.month')}</th>
            <th>{t('kernel.gasReference.expected')}</th><th>{t('kernel.gasReference.actual')}</th>
            <th>{t('kernel.gasReference.difference')}</th><th>{t('kernel.gasReference.tolerance')}</th>
            <th>{locale === 'nl' ? 'Beoordeling' : 'Assessment'}</th></tr></thead>
          <tbody>{result.metrics.map((item, index) => <tr key={`${item.metric}-${item.month ?? 'year'}-${index}`}
            className={item.withinTolerance ? '' : 'outside-tolerance'}>
            <th scope="row">{t(`kernel.gasReference.metric.${item.metric}`)}</th>
            <td>{item.month ?? t('kernel.gasReference.year')}</td>
            <td>{item.expectedKwh.toFixed(3)}</td><td>{item.actualKwh.toFixed(3)}</td>
            <td>{item.absoluteDifferenceKwh.toFixed(3)}</td><td>{item.absoluteToleranceKwh.toExponential(1)}</td>
            <td>{item.withinTolerance ? (locale === 'nl' ? 'Binnen tolerantie' : 'Within tolerance')
              : (locale === 'nl' ? 'Buiten tolerantie' : 'Outside tolerance')}</td>
          </tr>)}</tbody></table></div>}
        <small>{t('kernel.inputFingerprint')}: <code>{result.inputFingerprint}</code><br />
          {t('kernel.gasReference.caseFingerprint')}: <code>{result.caseFingerprint}</code></small>
      </div>}
    </div>
  </details>;
}
