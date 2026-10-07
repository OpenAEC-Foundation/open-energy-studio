import { useEffect, useRef, useState } from 'react';
import { useProjectEdition } from '../../context/EnergyContext';
import type { INtaHeatPumpInput } from '../../core/energy/types';
import { diagnoseGasHeatPumpAuxDraftWithRust, type GasHeatPumpAuxDraftAssessment, type GasHeatPumpAuxDraftInput } from '../../core/nta/KernelClient';
import { useI18n } from '../../i18n/i18n';
import '../HeatPumpForfaitMonthlyPanel/HeatPumpForfaitMonthlyPanel.css';

const monthNumbers = Array.from({ length: 12 }, (_, index) => index + 1);
const blank = () => monthNumbers.map(() => '');

export function GasHeatPumpAuxPanel({ pump, onSave }: {
  pump: INtaHeatPumpInput;
  onSave: (input: GasHeatPumpAuxDraftInput) => void;
}) {
  const { t } = useI18n();
  const saved = pump.gasHeatPumpAuxDraft;
  const forfait = pump.gasHeatPumpForfaitDraft;
  const [capacityReference, setCapacityReference] = useState(saved?.capacityReference ?? forfait?.capacityReference ?? '');
  const [share, setShare] = useState(saved ? String(saved.buildingShare) : '1');
  const [shareReference, setShareReference] = useState(saved?.buildingShareReference ?? '');
  const [hoursReference, setHoursReference] = useState(saved?.monthHoursReference ?? '');
  const [outputReference, setOutputReference] = useState(saved?.generatorOutputReference ?? '');
  const [hours, setHours] = useState(saved ? monthNumbers.map((month) => String(saved.months.find((item) => item.month === month)?.hours ?? '')) : blank);
  const [output, setOutput] = useState(saved ? monthNumbers.map((month) => String(saved.months.find((item) => item.month === month)?.generatorOutputKwh ?? '')) : blank);
  const [result, setResult] = useState<GasHeatPumpAuxDraftAssessment | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const sequence = useRef(0);
  const edition = useProjectEdition();
  useEffect(() => () => { sequence.current += 1; }, []);
  const invalidate = () => { sequence.current += 1; setResult(null); setError(null); setLoading(false); };
  const update = (values: string[], setter: (next: string[]) => void, index: number, value: string) => {
    invalidate(); setter(values.map((current, offset) => offset === index ? value : current));
  };
  const diagnose = async (save: boolean) => {
    if (!forfait) return;
    const fraction = Number(share);
    const validHours = hours.length === 12 && hours.every((value) => value.trim() !== ''
      && Number.isFinite(Number(value)) && Number(value) > 0 && Number(value) <= 744);
    const validOutput = output.length === 12 && output.every((value) => value.trim() !== ''
      && Number.isFinite(Number(value)) && Number(value) >= 0);
    if (!capacityReference.trim() || !shareReference.trim() || !hoursReference.trim()
      || !outputReference.trim() || !Number.isFinite(fraction) || fraction <= 0 || fraction > 1
      || !validHours || !validOutput) {
      setError(t('kernel.gasAux.invalid')); setResult(null); return;
    }
    const input: GasHeatPumpAuxDraftInput = {
      generatorId: pump.id, drive: pump.drive as 'gas_engine' | 'absorption',
      nominalThermalCapacityKw: forfait.thermalCapacityKw,
      capacityReference: capacityReference.trim(),
      standbyElectronicsW: 10, burnerAuxiliaryWPerKw: 1, solutionPumpWPerKw: 0,
      coefficientsReference: 'NTA 8800:2026 consultatieconcept §9.6.8.2.3',
      meanModulation: 1, modulationReference: 'NTA 8800:2026 consultatieconcept §9.6.8.2.3',
      buildingShare: fraction, buildingShareReference: shareReference.trim(),
      forfaitCopUsed: true,
      monthHoursReference: hoursReference.trim(), generatorOutputReference: outputReference.trim(),
      months: monthNumbers.map((month, index) => ({ month, hours: Number(hours[index]), generatorOutputKwh: Number(output[index]) })),
    };
    const current = ++sequence.current;
    setLoading(true); setError(null); setResult(null);
    try {
      const assessment = await diagnoseGasHeatPumpAuxDraftWithRust(input, edition);
      if (current !== sequence.current) return;
      setLoading(false); setResult(assessment);
      if (assessment.status === 'invalid') setError(assessment.issues.map((item) => item.code).join(', '));
      else if (save) onSave(input);
    } catch (reason) {
      if (current === sequence.current) { setLoading(false); setError(String(reason)); }
    }
  };
  if (!forfait) return null;
  return <section className="heat-pump-forfait-monthly" aria-label={t('kernel.gasAux.title')}>
    <h4>{t('kernel.gasAux.title')}</h4>
    <p>{t('kernel.gasAux.scope')}</p>
    <p>{t('kernel.gasAux.capacity')}: {forfait.thermalCapacityKw} kW · {t('kernel.gasAux.coefficients')}</p>
    <div className="heat-pump-forfait-monthly-references">
      <label>{t('kernel.gasAux.capacityReference')}<input value={capacityReference} onChange={(event) => { invalidate(); setCapacityReference(event.target.value); }} /></label>
      <label>{t('kernel.gasAux.share')}<input type="number" min="0" max="1" step="any" value={share} onChange={(event) => { invalidate(); setShare(event.target.value); }} /></label>
      <label>{t('kernel.gasAux.shareReference')}<input value={shareReference} onChange={(event) => { invalidate(); setShareReference(event.target.value); }} /></label>
      <label>{t('kernel.gasAux.hoursReference')}<input value={hoursReference} onChange={(event) => { invalidate(); setHoursReference(event.target.value); }} /></label>
      <label>{t('kernel.gasAux.outputReference')}<input value={outputReference} onChange={(event) => { invalidate(); setOutputReference(event.target.value); }} /></label>
    </div>
    <div className="heat-pump-forfait-monthly-table-wrap"><table><thead><tr>
      <th>{t('kernel.gasAux.month')}</th><th>{t('kernel.gasAux.hours')}</th><th>{t('kernel.gasAux.output')}</th>
    </tr></thead><tbody>{monthNumbers.map((month, index) => <tr key={month}>
      <th scope="row">{month}</th>
      <td><input type="number" min="0" max="744" step="any" aria-label={`${t('kernel.gasAux.hours')} ${month}`}
        value={hours[index]} onChange={(event) => update(hours, setHours, index, event.target.value)} /></td>
      <td><input type="number" min="0" step="any" aria-label={`${t('kernel.gasAux.output')} ${month}`}
        value={output[index]} onChange={(event) => update(output, setOutput, index, event.target.value)} /></td>
    </tr>)}</tbody></table></div>
    <button type="button" disabled={loading} onClick={() => void diagnose(false)}>{t('kernel.gasAux.calculate')}</button>
    <button type="button" disabled={loading} onClick={() => void diagnose(true)}>{t('kernel.gasAux.save')}</button>
    {error && <p role="alert" className="heat-pump-forfait-monthly-error">{error}</p>}
    {result?.status === 'diagnostic_valid' && <div role="status" className="heat-pump-forfait-monthly-result">
      <p>{t('kernel.gasAux.annual')}: {result.annualAuxiliaryElectricityKwh?.toFixed(2)} kWh</p>
      <div className="heat-pump-forfait-monthly-table-wrap"><table><thead><tr>
        <th>{t('kernel.gasAux.month')}</th><th>{t('kernel.gasAux.onHours')}</th><th>{t('kernel.gasAux.electricity')}</th>
      </tr></thead><tbody>{result.monthly.map((month) => <tr key={month.month}>
        <th scope="row">{month.month}</th><td>{month.cappedOnHours.toFixed(2)}</td><td>{month.auxiliaryElectricityKwh.toFixed(2)}</td>
      </tr>)}</tbody></table></div>
      <p>{t('kernel.gasAux.limit')}</p>
      <small>{t('kernel.inputFingerprint')}: <code>{result.inputFingerprint}</code></small>
    </div>}
    {saved && <p>{t('kernel.gasAux.saved')}</p>}
  </section>;
}
