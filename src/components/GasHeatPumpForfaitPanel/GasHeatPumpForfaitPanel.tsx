import { useEffect, useRef, useState } from 'react';
import { useProjectEdition } from '../../context/EnergyContext';
import type { BuildingFunction, INtaHeatPumpInput } from '../../core/energy/types';
import { diagnoseGasHeatPumpForfaitDraftWithRust, type GasHeatPumpForfaitDraftAssessment, type GasHeatPumpForfaitDraftInput } from '../../core/nta/KernelClient';
import { useI18n } from '../../i18n/i18n';
import '../HeatPumpForfaitDiagnosticPanel/HeatPumpForfaitDiagnosticPanel.css';

const sourceMap: Record<string, GasHeatPumpForfaitDraftInput['source'] | undefined> = {
  ground: 'ground', outdoor_air: 'outdoor_air', exhaust_air: 'exhaust_air',
  groundwater: 'groundwater_aquifer', surface_water: 'surface_water',
};

export function GasHeatPumpForfaitPanel({ pump, buildingFunction, onSave }: {
  pump: INtaHeatPumpInput; buildingFunction: BuildingFunction;
  onSave: (input: GasHeatPumpForfaitDraftInput) => void;
}) {
  const { t } = useI18n();
  const saved = pump.gasHeatPumpForfaitDraft;
  const [application, setApplication] = useState<GasHeatPumpForfaitDraftInput['application']>(
    saved?.application ?? (buildingFunction === 'residential' ? 'residential_collective_at_most25_kw' : 'utility'));
  const [applicationReference, setApplicationReference] = useState(saved?.applicationReference ?? '');
  const [collective, setCollective] = useState(saved?.collectiveBuildingInstallation ?? false);
  const [externalHeat, setExternalHeat] = useState(saved?.externalHeatSupply ?? false);
  const [capacity, setCapacity] = useState(saved ? String(saved.thermalCapacityKw) : '');
  const [capacityReference, setCapacityReference] = useState(saved?.capacityReference ?? '');
  const [sourceReference, setSourceReference] = useState(saved?.sourceReference ?? pump.performanceEvidence.reference ?? '');
  const [temperature, setTemperature] = useState(saved ? String(saved.designSupplyTemperatureC) : '');
  const [temperatureReference, setTemperatureReference] = useState(saved?.designSupplyReference ?? '');
  const [correction, setCorrection] = useState(saved?.sourceCorrectionFactor == null ? '' : String(saved.sourceCorrectionFactor));
  const [correctionReference, setCorrectionReference] = useState(saved?.sourceCorrectionReference ?? '');
  const [result, setResult] = useState<GasHeatPumpForfaitDraftAssessment | null>(null);
  const [error, setError] = useState<string | null>(null);
  const sequence = useRef(0);
  const edition = useProjectEdition();
  useEffect(() => () => { sequence.current += 1; }, []);
  const invalidate = () => { sequence.current += 1; setResult(null); setError(null); };
  const source = sourceMap[pump.source];
  const residential = application === 'residential_collective_at_most25_kw';
  const correctionRequired = residential && (source === 'ground' || source === 'groundwater_aquifer');

  const diagnose = async (saveInput: boolean) => {
    const thermalCapacityKw = Number(capacity);
    const designSupplyTemperatureC = Number(temperature);
    const sourceCorrectionFactor = correctionRequired && correction.trim() !== '' ? Number(correction) : null;
    if (!source || !capacity.trim() || !temperature.trim() || !Number.isFinite(thermalCapacityKw)
      || thermalCapacityKw <= 0 || !Number.isFinite(designSupplyTemperatureC)
      || designSupplyTemperatureC <= 0 || designSupplyTemperatureC > 55
      || !applicationReference.trim() || !capacityReference.trim()
      || !sourceReference.trim() || !temperatureReference.trim()
      || (application === 'collective_building' && !collective)
      || (residential && (!collective || thermalCapacityKw > 25
        || source === 'exhaust_air' || source === 'surface_water'
        || buildingFunction !== 'residential'))
      || (application === 'collective_building' && thermalCapacityKw <= 25)
      || (application === 'over25_kw' && thermalCapacityKw <= 25)
      || (application === 'utility' && buildingFunction === 'residential')
      || (correctionRequired && (sourceCorrectionFactor === null || !Number.isFinite(sourceCorrectionFactor)
        || sourceCorrectionFactor <= 0 || !correctionReference.trim()))) {
      setError(t('kernel.gasForfait.invalid')); setResult(null); return;
    }
    const input: GasHeatPumpForfaitDraftInput = {
      generatorId: pump.id, drive: pump.drive as 'gas_engine' | 'absorption', application,
      applicationReference: applicationReference.trim(), collectiveBuildingInstallation: collective,
      externalHeatSupply: externalHeat, thermalCapacityKw, capacityReference: capacityReference.trim(),
      source, sourceReference: sourceReference.trim(), designSupplyTemperatureC,
      designSupplyReference: temperatureReference.trim(),
      sourceCorrectionFactor,
      sourceCorrectionReference: correctionRequired ? correctionReference.trim() : null,
    };
    const current = ++sequence.current;
    setError(null);
    try {
      const assessment = await diagnoseGasHeatPumpForfaitDraftWithRust(input, edition);
      if (current !== sequence.current) return;
      setResult(assessment);
      if (assessment.status === 'invalid') setError(assessment.issues.map((issue) => issue.code).join(', '));
      else if (saveInput) onSave(input);
    } catch (cause) {
      if (current === sequence.current) setError(String(cause));
    }
  };

  return <div className="heat-pump-forfait-panel">
    <h4>{t('kernel.gasForfait.title')}</h4>
    <p>{t('kernel.gasForfait.scope')}</p>
    {!source && <p role="alert">{t('kernel.gasForfait.sourceUnavailable')}</p>}
    <label>{t('kernel.gasForfait.application')}
      <select value={application} onChange={(event) => { setApplication(event.target.value as typeof application); invalidate(); }}>
        {buildingFunction === 'residential' && <option value="residential_collective_at_most25_kw">{t('kernel.gasForfait.residential')}</option>}
        {buildingFunction !== 'residential' && <option value="utility">{t('kernel.gasForfait.utility')}</option>}
        <option value="collective_building">{t('kernel.gasForfait.collective')}</option>
        <option value="over25_kw">{t('kernel.gasForfait.over25')}</option>
      </select>
    </label>
    <label>{t('kernel.gasForfait.applicationReference')}<input value={applicationReference} onChange={(event) => { setApplicationReference(event.target.value); invalidate(); }} /></label>
    <label>{t('kernel.gasForfait.collectiveFlag')}<input type="checkbox" checked={collective} onChange={(event) => { setCollective(event.target.checked); invalidate(); }} /></label>
    <label>{t('kernel.gasForfait.externalHeat')}<input type="checkbox" checked={externalHeat} onChange={(event) => { setExternalHeat(event.target.checked); invalidate(); }} /></label>
    <label>{t('kernel.gasForfait.capacity')}<input type="number" min="0" step="any" value={capacity} onChange={(event) => { setCapacity(event.target.value); invalidate(); }} /></label>
    <label>{t('kernel.gasForfait.capacityReference')}<input value={capacityReference} onChange={(event) => { setCapacityReference(event.target.value); invalidate(); }} /></label>
    <label>{t('kernel.gasForfait.sourceReference')}<input value={sourceReference} onChange={(event) => { setSourceReference(event.target.value); invalidate(); }} /></label>
    <label>{t('kernel.gasForfait.temperature')}<input type="number" min="0" max="55" step="any" value={temperature} onChange={(event) => { setTemperature(event.target.value); invalidate(); }} /></label>
    <label>{t('kernel.gasForfait.temperatureReference')}<input value={temperatureReference} onChange={(event) => { setTemperatureReference(event.target.value); invalidate(); }} /></label>
    {correctionRequired && <>
      <label>{t('kernel.gasForfait.correction')}<input type="number" min="0" step="any" value={correction} onChange={(event) => { setCorrection(event.target.value); invalidate(); }} /></label>
      <label>{t('kernel.gasForfait.correctionReference')}<input value={correctionReference} onChange={(event) => { setCorrectionReference(event.target.value); invalidate(); }} /></label>
    </>}
    <button type="button" onClick={() => diagnose(false)} disabled={!source}>{t('kernel.gasForfait.lookup')}</button>
    <button type="button" onClick={() => diagnose(true)} disabled={!source}>{t('kernel.gasForfait.save')}</button>
    {error && <p role="alert">{error}</p>}
    {result?.forfaitCop != null && <p>{t('kernel.gasForfait.result')} {result.table}: {result.forfaitCop.toFixed(2)} ({t('kernel.gasForfait.band')} ≤{result.temperatureBandUpperC} °C)
      {correctionRequired && result.correctedCop != null && `; ${t('kernel.gasForfait.corrected')}: ${result.correctedCop.toFixed(2)}`}</p>}
    {result && <p>{t('kernel.gasForfait.limit')}</p>}
    {saved && <p>{t('kernel.gasForfait.saved')}</p>}
  </div>;
}
