import { useEffect, useRef, useState } from 'react';
import { useProjectEdition } from '../../context/EnergyContext';
import type { INtaHeatPumpInput } from '../../core/energy/types';
import { diagnoseForfaitHeatPumpMonthlyDraftWithRust, type ForfaitHeatPumpMonthlyDraftAssessment, type ForfaitHeatPumpMonthlyDraftInput } from '../../core/nta/KernelClient';
import { useI18n } from '../../i18n/i18n';
import './HeatPumpForfaitMonthlyPanel.css';

type SourceSystem = ForfaitHeatPumpMonthlyDraftInput['sourceSystem'];
const months = Array.from({ length: 12 }, (_, index) => index + 1);
const blankMonths = () => months.map(() => '');

function systemsFor(source: NonNullable<INtaHeatPumpInput['forfaitHeatPumpDraft']>['source']): SourceSystem[] {
  if (source === 'ground') return ['individual', 'collective_ground'];
  if (source === 'groundwater_below15_c' || source === 'surface_water')
    return ['individual', 'collective_groundwater_surface_or_at_least15_c'];
  if (source.startsWith('collective')) return ['collective_groundwater_surface_or_at_least15_c'];
  return ['individual'];
}

export function HeatPumpForfaitMonthlyPanel({ pump }: { pump: INtaHeatPumpInput }) {
  const { t } = useI18n();
  const forfait = pump.forfaitHeatPumpDraft;
  const available = forfait ? systemsFor(forfait.source) : ['individual'] as SourceSystem[];
  const [sourceSystem, setSourceSystem] = useState<SourceSystem>(available[0]);
  const [outputValues, setOutputValues] = useState<string[]>(blankMonths);
  const [outputReference, setOutputReference] = useState('');
  const [systemReference, setSystemReference] = useState('');
  const [result, setResult] = useState<ForfaitHeatPumpMonthlyDraftAssessment | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const sequence = useRef(0);
  const edition = useProjectEdition();
  useEffect(() => () => { sequence.current += 1; }, []);
  const invalidate = () => { sequence.current += 1; setResult(null); setError(null); setLoading(false); };
  const updateMonth = (values: string[], setValues: (value: string[]) => void, index: number, value: string) => {
    invalidate();
    setValues(values.map((current, offset) => offset === index ? value : current));
  };
  const diagnose = async () => {
    if (!forfait) return;
    const valid = (values: string[]) => values.every((value) => value.trim() !== ''
      && Number.isFinite(Number(value)) && Number(value) >= 0);
    if (!outputReference.trim() || !systemReference.trim() || !valid(outputValues)) {
      setError(t('kernel.forfaitMonthly.invalid'));
      setResult(null);
      return;
    }
    const input: ForfaitHeatPumpMonthlyDraftInput = {
      forfait,
      generatorOutputKwh: months.map((month, index) => ({ month, energyKwh: Number(outputValues[index]) })),
      generatorOutputReference: outputReference.trim(),
      sourceSystem,
      sourceSystemReference: systemReference.trim(),
    };
    const current = ++sequence.current;
    setLoading(true); setError(null); setResult(null);
    try {
      const assessment = await diagnoseForfaitHeatPumpMonthlyDraftWithRust(input, edition);
      if (sequence.current === current) { setResult(assessment); setLoading(false); }
    } catch (reason: unknown) {
      if (sequence.current === current) {
        setError(reason instanceof Error ? reason.message : String(reason)); setLoading(false);
      }
    }
  };
  return <section className="heat-pump-forfait-monthly" aria-label={t('kernel.forfaitMonthly.title')}>
    <h4>{t('kernel.forfaitMonthly.title')}</h4>
    <p>{t('kernel.forfaitMonthly.scope')}</p>
    <label>{t('kernel.forfaitMonthly.sourceSystem')}
      <select value={sourceSystem} onChange={(event) => {
        invalidate(); setSourceSystem(event.target.value as SourceSystem);
      }}>{available.map((value) => <option key={value} value={value}>{t(`kernel.forfaitMonthly.sourceSystem.${value}`)}</option>)}</select>
    </label>
    <div className="heat-pump-forfait-monthly-references">
      <label>{t('kernel.forfaitMonthly.outputReference')}
        <input type="text" value={outputReference} onChange={(event) => { invalidate(); setOutputReference(event.target.value); }} /></label>
      <label>{t('kernel.forfaitMonthly.systemReference')}
        <input type="text" value={systemReference} onChange={(event) => { invalidate(); setSystemReference(event.target.value); }} /></label>
    </div>
    <div className="heat-pump-forfait-monthly-table-wrap"><table>
      <thead><tr><th>{t('kernel.forfaitMonthly.month')}</th><th>{t('kernel.forfaitMonthly.generatorOutput')}</th></tr></thead>
      <tbody>{months.map((month, index) => <tr key={month}>
        <th scope="row">{month}</th>
        <td><input type="number" min="0" step="any" aria-label={`${t('kernel.forfaitMonthly.generatorOutput')} ${month}`}
          value={outputValues[index]} onChange={(event) => updateMonth(outputValues, setOutputValues, index, event.target.value)} /></td>
      </tr>)}</tbody>
    </table></div>
    <button type="button" onClick={() => void diagnose()} disabled={loading}>{loading ? t('kernel.loading') : t('kernel.forfaitMonthly.calculate')}</button>
    {error && <p role="alert" className="heat-pump-forfait-monthly-error">{error}</p>}
    {result?.status === 'invalid' && <p role="alert" className="heat-pump-forfait-monthly-error">
      {t('kernel.forfaitMonthly.invalid')} {result.issues.map((item) => item.code).join(', ')}
    </p>}
    {result?.status === 'diagnostic_valid' && <div role="status" className="heat-pump-forfait-monthly-result">
      <p>{t('kernel.forfaitMonthly.cop')}: {result.correctedCop?.toFixed(2)} · {t('kernel.forfaitMonthly.correction')}: {result.collectiveSourceCorrectionFactor}</p>
      <div className="heat-pump-forfait-monthly-table-wrap"><table><thead><tr>
        <th>{t('kernel.forfaitMonthly.month')}</th>
        {result.collectiveSourceHeatDerived && <th>{t('kernel.forfaitMonthly.sourceHeatDerived')}</th>}
        <th>{t('kernel.forfaitMonthly.generatorElectricity')}</th>
      </tr></thead><tbody>{result.monthly.map((item) => <tr key={item.month}>
        <th scope="row">{item.month}</th>
        {result.collectiveSourceHeatDerived && <td>{item.collectiveSourceHeatKwh.toFixed(2)}</td>}
        <td>{item.generatorInputElectricityKwh.toFixed(2)}</td>
      </tr>)}</tbody></table></div>
      <p>{t('kernel.forfaitMonthly.resultWarning')}</p>
      <small>{t('kernel.inputFingerprint')}: <code>{result.inputFingerprint}</code></small>
    </div>}
  </section>;
}
