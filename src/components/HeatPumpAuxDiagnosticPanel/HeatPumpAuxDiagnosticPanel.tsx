import { useEffect, useRef, useState } from 'react';
import { useProjectEdition } from '../../context/EnergyContext';
import type { INtaHeatPumpInput } from '../../core/energy/types';
import {
  diagnoseHeatingAuxMeasuredDraftWithRust,
  type HeatingAuxMeasuredDraftAssessment,
  type HeatingAuxMeasuredDraftInput,
} from '../../core/nta/KernelClient';
import { useI18n } from '../../i18n/i18n';
import './HeatPumpAuxDiagnosticPanel.css';

type NumericField =
  | 'standbyElectronicsW' | 'deliveryPumpDuringCompressorW' | 'deliveryPumpPrePostW'
  | 'pumpPreRunSeconds' | 'pumpPostRunSeconds' | 'averageCompressorOnSeconds'
  | 'meanCompressorModulation' | 'nominalElectricDriveKw';

const numericFields: NumericField[] = [
  'standbyElectronicsW', 'deliveryPumpDuringCompressorW', 'deliveryPumpPrePostW',
  'pumpPreRunSeconds', 'pumpPostRunSeconds', 'averageCompressorOnSeconds',
  'meanCompressorModulation', 'nominalElectricDriveKw',
];

const emptyNumbers = (): Record<NumericField, string> => ({
  standbyElectronicsW: '', deliveryPumpDuringCompressorW: '', deliveryPumpPrePostW: '',
  pumpPreRunSeconds: '', pumpPostRunSeconds: '', averageCompressorOnSeconds: '',
  meanCompressorModulation: '', nominalElectricDriveKw: '',
});

const savedMonths = (input?: HeatingAuxMeasuredDraftInput): string[] => Array.from({ length: 12 }, (_, index) => {
  const value = input?.months.find((item) => item.month === index + 1)?.generatorInputElectricityKwh;
  return value === undefined ? '' : String(value);
});

export function HeatPumpAuxDiagnosticPanel({ pump, onSave }: { pump: INtaHeatPumpInput; onSave?: (input: HeatingAuxMeasuredDraftInput) => void }) {
  const { t } = useI18n();
  const saved = pump.heatingAuxMeasuredDraft;
  const [numbers, setNumbers] = useState(() => saved
    ? Object.fromEntries(numericFields.map((field) => [field, String(saved.measurements[field])])) as Record<NumericField, string>
    : emptyNumbers());
  const [generatorSource, setGeneratorSource] = useState(saved?.generatorSourceReference ?? pump.performanceEvidence.reference ?? '');
  const [measurementSource, setMeasurementSource] = useState(saved?.measurements.measurementSourceReference ?? '');
  const [timingSource, setTimingSource] = useState(saved?.measurements.timingSourceReference ?? '');
  const [inputEnergySource, setInputEnergySource] = useState(saved?.inputEnergySourceReference ?? '');
  const [months, setMonths] = useState<string[]>(() => savedMonths(saved));
  const [result, setResult] = useState<HeatingAuxMeasuredDraftAssessment | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const requestSequence = useRef(0);

  const edition = useProjectEdition();
  useEffect(() => {
    requestSequence.current += 1;
    setNumbers(pump.heatingAuxMeasuredDraft
      ? Object.fromEntries(numericFields.map((field) => [field, String(pump.heatingAuxMeasuredDraft!.measurements[field])])) as Record<NumericField, string>
      : emptyNumbers());
    setGeneratorSource(pump.heatingAuxMeasuredDraft?.generatorSourceReference ?? pump.performanceEvidence.reference ?? '');
    setMeasurementSource(pump.heatingAuxMeasuredDraft?.measurements.measurementSourceReference ?? '');
    setTimingSource(pump.heatingAuxMeasuredDraft?.measurements.timingSourceReference ?? '');
    setInputEnergySource(pump.heatingAuxMeasuredDraft?.inputEnergySourceReference ?? '');
    setMonths(savedMonths(pump.heatingAuxMeasuredDraft));
    setResult(null);
    setError(null);
    setLoading(false);
    return () => { requestSequence.current += 1; };
  }, [pump.id, pump.performanceEvidence.reference]);

  const invalidate = () => {
    requestSequence.current += 1;
    setResult(null);
    setError(null);
    setLoading(false);
  };

  const buildInput = (): HeatingAuxMeasuredDraftInput | null => {
    const values = Object.fromEntries(numericFields.map((field) => [field,
      numbers[field].trim() === '' ? Number.NaN : Number(numbers[field])])) as Record<NumericField, number>;
    const monthValues = months.map((value) => value.trim() === '' ? Number.NaN : Number(value));
    if (Object.values(values).some((value) => !Number.isFinite(value) || value < 0)
      || values.averageCompressorOnSeconds <= 0 || values.meanCompressorModulation <= 0
      || values.meanCompressorModulation > 1 || values.nominalElectricDriveKw <= 0
      || monthValues.some((value) => !Number.isFinite(value) || value < 0)
      || !generatorSource.trim() || !measurementSource.trim() || !timingSource.trim() || !inputEnergySource.trim()) {
      setError(t('kernel.auxDraft.invalid'));
      setResult(null);
      return null;
    }
    const input: HeatingAuxMeasuredDraftInput = {
      generatorId: pump.id,
      generatorSourceReference: generatorSource.trim(),
      measurements: {
        ...values,
        measurementSourceReference: measurementSource.trim(),
        timingSourceReference: timingSource.trim(),
      },
      inputEnergySourceReference: inputEnergySource.trim(),
      months: monthValues.map((generatorInputElectricityKwh, index) => ({
        month: index + 1, generatorInputElectricityKwh,
      })),
    };
    return input;
  };

  const calculate = async () => {
    const input = buildInput();
    if (!input) return;
    const current = ++requestSequence.current;
    setLoading(true);
    setError(null);
    setResult(null);
    try {
      const assessment = await diagnoseHeatingAuxMeasuredDraftWithRust(input, edition);
      if (requestSequence.current === current) {
        setResult(assessment);
        setLoading(false);
      }
    } catch (reason: unknown) {
      if (requestSequence.current === current) {
        setError(reason instanceof Error ? reason.message : String(reason));
        setLoading(false);
      }
    }
  };

  return <section className="heat-pump-aux-draft" aria-label={t('kernel.auxDraft.title')}>
    <h4>{t('kernel.auxDraft.title')}</h4>
    <p>{t('kernel.auxDraft.scope')}</p>
    <div className="heat-pump-aux-draft-grid">
      <label><span>{t('kernel.auxDraft.generatorSource')}</span>
        <input type="text" value={generatorSource} onChange={(event) => { invalidate(); setGeneratorSource(event.target.value); }} /></label>
      {numericFields.map((field) => <label key={field}>
        <span>{t(`kernel.auxDraft.${field}`)}</span>
        <input type="number" min="0" step="any" value={numbers[field]}
          onChange={(event) => { invalidate(); setNumbers((prior) => ({ ...prior, [field]: event.target.value })); }} />
      </label>)}
      <label><span>{t('kernel.auxDraft.measurementSource')}</span>
        <input type="text" value={measurementSource} onChange={(event) => { invalidate(); setMeasurementSource(event.target.value); }} /></label>
      <label><span>{t('kernel.auxDraft.timingSource')}</span>
        <input type="text" value={timingSource} onChange={(event) => { invalidate(); setTimingSource(event.target.value); }} /></label>
      <label><span>{t('kernel.auxDraft.inputEnergySource')}</span>
        <input type="text" value={inputEnergySource} onChange={(event) => { invalidate(); setInputEnergySource(event.target.value); }} /></label>
    </div>
    <fieldset>
      <legend>{t('kernel.auxDraft.months')}</legend>
      <div className="heat-pump-aux-draft-months">{months.map((value, index) => <label key={index}>
        <span>{t('kernel.auxDraft.month')} {index + 1}</span>
        <input type="number" min="0" step="any" value={value}
          onChange={(event) => { invalidate(); setMonths((prior) => prior.map((item, at) => at === index ? event.target.value : item)); }} />
      </label>)}</div>
    </fieldset>
    <div className="heat-pump-aux-draft-actions">
      <button type="button" onClick={calculate} disabled={loading}>{loading ? t('kernel.loading') : t('kernel.auxDraft.calculate')}</button>
      {onSave && <button type="button" onClick={() => { const input = buildInput(); if (input) onSave(input); }}>{t('kernel.auxDraft.save')}</button>}
    </div>
    {saved && <p>{t('kernel.auxDraft.saved')}</p>}
    {error && <p className="heat-pump-aux-draft-error" role="alert">{error}</p>}
    {result?.status === 'invalid' && <div className="heat-pump-aux-draft-error" role="alert">
      {t('kernel.auxDraft.invalid')} <ul>{result.issues.map((item, index) => <li key={`${item.path}-${index}`}>{item.path}: {item.code}</li>)}</ul>
    </div>}
    {result?.status === 'input_valid' && result.derivedCoefficients && result.auxiliary && <div className="heat-pump-aux-draft-result" role="status">
      <strong>{t('kernel.auxDraft.result')}</strong>
      <dl><dt>A</dt><dd>{result.derivedCoefficients.aAnnualKwh.toFixed(2)} {t('kernel.auxDraft.kwhPerYear')}</dd>
        <dt>B</dt><dd>{result.derivedCoefficients.bKw.toFixed(3)} kW</dd>
        <dt>C</dt><dd>{result.derivedCoefficients.cDimensionless.toFixed(3)}</dd>
        <dt>{t('kernel.auxDraft.annual')}</dt><dd>{result.auxiliary.annualAuxiliaryElectricityKwh?.toFixed(2)} {t('kernel.auxDraft.kwhPerYear')}</dd></dl>
      <details><summary>{t('kernel.auxDraft.monthlyResult')}</summary>
        <ol>{result.auxiliary.monthlyAuxiliaryElectricityKwh.map((month) =>
          <li key={month.month}>{t('kernel.auxDraft.month')} {month.month}: {month.electricityKwh.toFixed(2)} kWh</li>)}</ol>
      </details>
      <p>{t('kernel.auxDraft.resultScope')}</p>
      <small>{t('kernel.inputFingerprint')}: <code>{result.inputFingerprint}</code></small>
    </div>}
  </section>;
}
