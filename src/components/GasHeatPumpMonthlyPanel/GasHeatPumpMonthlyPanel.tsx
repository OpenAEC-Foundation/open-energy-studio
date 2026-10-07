import { useEffect, useRef, useState } from 'react';
import { useProjectEdition } from '../../context/EnergyContext';
import type { INtaHeatPumpInput } from '../../core/energy/types';
import { diagnoseGasCollectiveSourceDraftWithRust, diagnoseGasHeatPumpChainDraftWithRust, diagnoseGasHeatPumpMonthlyDraftWithRust, type GasCollectiveSourceDraftAssessment, type GasCollectiveSourceTemperatureClass, type GasHeatPumpChainDraftAssessment, type GasHeatPumpMonthlyDraftAssessment, type GasHeatPumpMonthlyDraftInput } from '../../core/nta/KernelClient';
import { useI18n } from '../../i18n/i18n';
import '../HeatPumpForfaitMonthlyPanel/HeatPumpForfaitMonthlyPanel.css';

const months = Array.from({ length: 12 }, (_, index) => index + 1);
type SourceSystem = GasHeatPumpMonthlyDraftInput['sourceSystem'];

function sourceSystems(source: NonNullable<INtaHeatPumpInput['gasHeatPumpForfaitDraft']>['source']): SourceSystem[] {
  if (source === 'ground') return ['individual', 'collective_ground'];
  if (source === 'groundwater_aquifer' || source === 'surface_water')
    return ['individual', 'collective_groundwater_surface_or_at_least15_c'];
  return ['individual'];
}

export function GasHeatPumpMonthlyPanel({ pump }: { pump: INtaHeatPumpInput }) {
  const { t } = useI18n();
  const forfait = pump.gasHeatPumpForfaitDraft;
  const available = forfait ? sourceSystems(forfait.source) : ['individual'] as SourceSystem[];
  const [sourceSystem, setSourceSystem] = useState<SourceSystem>(available[0]);
  const [sourceReference, setSourceReference] = useState('');
  const [outputReference, setOutputReference] = useState(pump.gasHeatPumpAuxDraft?.generatorOutputReference ?? '');
  const [output, setOutput] = useState(months.map((month) => {
    const value = pump.gasHeatPumpAuxDraft?.months.find((item) => item.month === month)?.generatorOutputKwh;
    return value == null ? '' : String(value);
  }));
  const [result, setResult] = useState<GasHeatPumpMonthlyDraftAssessment | null>(null);
  const [chainResult, setChainResult] = useState<GasHeatPumpChainDraftAssessment | null>(null);
  const [sourceResult, setSourceResult] = useState<GasCollectiveSourceDraftAssessment | null>(null);
  const [temperatureClass, setTemperatureClass] = useState<GasCollectiveSourceTemperatureClass>('unknown');
  const [temperatureReference, setTemperatureReference] = useState('');
  const [noDeclaration, setNoDeclaration] = useState(false);
  const [declarationReference, setDeclarationReference] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const sequence = useRef(0);
  const edition = useProjectEdition();
  useEffect(() => () => { sequence.current += 1; }, []);
  const invalidate = () => { sequence.current += 1; setResult(null); setChainResult(null); setSourceResult(null); setError(null); setLoading(false); };
  const diagnose = async (kind: 'monthly' | 'linked' | 'source' = 'monthly') => {
    if (!forfait) return;
    if (!sourceReference.trim() || !outputReference.trim() || output.some((value) => value.trim() === ''
      || !Number.isFinite(Number(value)) || Number(value) < 0)) {
      setError(t('kernel.gasMonthly.invalid')); setResult(null); setChainResult(null); setSourceResult(null); return;
    }
    const input: GasHeatPumpMonthlyDraftInput = {
      forfait, generatorOutputKwh: months.map((month, index) => ({ month, energyKwh: Number(output[index]) })),
      generatorOutputReference: outputReference.trim(), sourceSystem,
      sourceSystemReference: sourceReference.trim(),
    };
    const current = ++sequence.current;
    setLoading(true); setError(null); setResult(null); setChainResult(null); setSourceResult(null);
    try {
      const chain = pump.gasHeatPumpAuxDraft ? { monthly: input, auxiliary: pump.gasHeatPumpAuxDraft } : null;
      const assessment = kind === 'source' && chain
        ? await diagnoseGasCollectiveSourceDraftWithRust({ chain, sourceTemperatureClass: temperatureClass,
          sourceTemperatureReference: temperatureReference.trim(), noQualityDeclarationConfirmed: noDeclaration,
          noQualityDeclarationReference: declarationReference.trim() }, edition)
        : kind === 'linked' && chain ? await diagnoseGasHeatPumpChainDraftWithRust(chain, edition)
          : await diagnoseGasHeatPumpMonthlyDraftWithRust(input, edition);
      if (sequence.current !== current) return;
      setLoading(false);
      if (kind === 'source') setSourceResult(assessment as GasCollectiveSourceDraftAssessment);
      else if (kind === 'linked') setChainResult(assessment as GasHeatPumpChainDraftAssessment);
      else setResult(assessment as GasHeatPumpMonthlyDraftAssessment);
      if (assessment.status === 'invalid') setError(assessment.issues.map((item) => item.code).join(', '));
    } catch (reason) {
      if (sequence.current === current) { setLoading(false); setError(String(reason)); }
    }
  };
  if (!forfait) return null;
  return <section className="heat-pump-forfait-monthly" aria-label={t('kernel.gasMonthly.title')}>
    <h4>{t('kernel.gasMonthly.title')}</h4>
    <p>{t('kernel.gasMonthly.scope')}</p>
    <label>{t('kernel.gasMonthly.sourceSystem')}
      <select value={sourceSystem} onChange={(event) => { invalidate(); setSourceSystem(event.target.value as SourceSystem); }}>
        {available.map((value) => <option key={value} value={value}>{t(`kernel.gasMonthly.sourceSystem.${value}`)}</option>)}
      </select>
    </label>
    <div className="heat-pump-forfait-monthly-references">
      <label>{t('kernel.gasMonthly.sourceReference')}<input value={sourceReference} onChange={(event) => { invalidate(); setSourceReference(event.target.value); }} /></label>
      <label>{t('kernel.gasMonthly.outputReference')}<input value={outputReference} onChange={(event) => { invalidate(); setOutputReference(event.target.value); }} /></label>
    </div>
    <div className="heat-pump-forfait-monthly-table-wrap"><table><thead><tr>
      <th>{t('kernel.gasMonthly.month')}</th><th>{t('kernel.gasMonthly.output')}</th>
    </tr></thead><tbody>{months.map((month, index) => <tr key={month}>
      <th scope="row">{month}</th>
      <td><input type="number" min="0" step="any" aria-label={`${t('kernel.gasMonthly.output')} ${month}`}
        value={output[index]} onChange={(event) => { invalidate(); setOutput(output.map((value, offset) => offset === index ? event.target.value : value)); }} /></td>
    </tr>)}</tbody></table></div>
    <button type="button" disabled={loading} onClick={() => void diagnose()}>{t('kernel.gasMonthly.calculate')}</button>
    {pump.gasHeatPumpAuxDraft && <button type="button" disabled={loading} onClick={() => void diagnose('linked')}>{t('kernel.gasMonthly.calculateLinked')}</button>}
    {pump.gasHeatPumpAuxDraft && sourceSystem !== 'individual' && <fieldset>
      <legend>{t('kernel.gasMonthly.collectiveSource')}</legend>
      <p>{t('kernel.gasMonthly.collectiveScope')}</p>
      <label>{t('kernel.gasMonthly.temperatureClass')}<select value={temperatureClass} onChange={(event) => { invalidate(); setTemperatureClass(event.target.value as GasCollectiveSourceTemperatureClass); }}>
        {(['unknown', 'below20_c', 'at_least20_c'] as const).map((value) => <option key={value} value={value}>{t(`kernel.gasMonthly.temperatureClass.${value}`)}</option>)}
      </select></label>
      <label>{t('kernel.gasMonthly.temperatureReference')}<input value={temperatureReference} onChange={(event) => { invalidate(); setTemperatureReference(event.target.value); }} /></label>
      <label><input type="checkbox" checked={noDeclaration} onChange={(event) => { invalidate(); setNoDeclaration(event.target.checked); }} />{t('kernel.gasMonthly.noDeclaration')}</label>
      <label>{t('kernel.gasMonthly.declarationReference')}<input value={declarationReference} onChange={(event) => { invalidate(); setDeclarationReference(event.target.value); }} /></label>
      <button type="button" disabled={loading} onClick={() => void diagnose('source')}>{t('kernel.gasMonthly.calculateSource')}</button>
    </fieldset>}
    {error && <p role="alert" className="heat-pump-forfait-monthly-error">{error}</p>}
    {result?.status === 'diagnostic_valid' && <div role="status" className="heat-pump-forfait-monthly-result">
      <p>{t('kernel.gasMonthly.cop')}: {result.correctedCop?.toFixed(2)} · {t('kernel.gasMonthly.factor')}: {result.collectiveSourceCorrectionFactor}</p>
      <div className="heat-pump-forfait-monthly-table-wrap"><table><thead><tr>
        <th>{t('kernel.gasMonthly.month')}</th>
        {result.collectiveSourceHeatDerived && <th>{t('kernel.gasMonthly.sourceHeat')}</th>}
        <th>{t('kernel.gasMonthly.inputTerm')}</th>
      </tr></thead><tbody>{result.monthly.map((month) => <tr key={month.month}>
        <th scope="row">{month.month}</th>
        {result.collectiveSourceHeatDerived && <td>{month.collectiveSourceHeatKwh.toFixed(2)}</td>}
        <td>{month.equation962InputTermKwh.toFixed(2)}</td>
      </tr>)}</tbody></table></div>
      <p>{t('kernel.gasMonthly.limit')}</p>
      <small>{t('kernel.inputFingerprint')}: <code>{result.inputFingerprint}</code></small>
    </div>}
    {chainResult?.status === 'diagnostic_valid' && <div role="status" className="heat-pump-forfait-monthly-result">
      <p>{t('kernel.gasMonthly.linkedAnnualAux')}: {chainResult.annualEquipmentAuxiliaryElectricityKwh?.toFixed(2)} kWh</p>
      <div className="heat-pump-forfait-monthly-table-wrap"><table><thead><tr>
        <th>{t('kernel.gasMonthly.month')}</th>
        <th>{t('kernel.gasMonthly.inputTerm')}</th>
        <th>{t('kernel.gasMonthly.linkedAux')}</th>
      </tr></thead><tbody>{chainResult.monthly.map((month) => <tr key={month.month}>
        <th scope="row">{month.month}</th>
        <td>{month.equation962UnallocatedInputTermKwh.toFixed(2)}</td>
        <td>{month.equipmentAuxiliaryElectricityKwh.toFixed(2)}</td>
      </tr>)}</tbody></table></div>
      <p>{t('kernel.gasMonthly.linkedLimit')}</p>
      <small>{t('kernel.inputFingerprint')}: <code>{chainResult.inputFingerprint}</code></small>
    </div>}
    {sourceResult?.status === 'diagnostic_valid' && <div role="status" className="heat-pump-forfait-monthly-result">
      <p>{t('kernel.gasMonthly.annualSourceHeat')}: {sourceResult.annualDeliveredSourceHeatKwh?.toFixed(2)} kWh (dh)</p>
      <p>{t('kernel.gasMonthly.fossilFactor')}: {sourceResult.primaryFossilFactor?.toFixed(5)} · {t('kernel.gasMonthly.renewableFactor')}: {sourceResult.primaryRenewableFactor?.toFixed(2)}</p>
      <p>{t('kernel.gasMonthly.annualFossil')}: {sourceResult.annualDraftPrimaryFossilKwh?.toFixed(2)} kWh · {t('kernel.gasMonthly.annualRenewable')}: {sourceResult.annualDraftPrimaryRenewableKwh?.toFixed(2)} kWh</p>
      <p>{t('kernel.gasMonthly.sourceLimit')}</p>
      <small>{t('kernel.inputFingerprint')}: <code>{sourceResult.inputFingerprint}</code></small>
    </div>}
  </section>;
}
