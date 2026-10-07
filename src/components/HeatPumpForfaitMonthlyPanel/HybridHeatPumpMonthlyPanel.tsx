import { useEffect, useRef, useState } from 'react';
import { useProjectEdition } from '../../context/EnergyContext';
import type { INtaHeatPumpInput } from '../../core/energy/types';
import {
  diagnoseBoilerForfaitDraftWithRust, diagnoseForfaitHeatPumpDraftWithRust, diagnoseHybridHeatPumpMonthlyDraftWithRust,
  type BoilerForfaitDraftInput, type ForfaitHeatPumpMonthlyDraftInput, type HybridHeatPumpMonthlyDraftAssessment,
} from '../../core/nta/KernelClient';
import { useI18n } from '../../i18n/i18n';
import './HeatPumpForfaitMonthlyPanel.css';

type SourceSystem = ForfaitHeatPumpMonthlyDraftInput['sourceSystem'];
const months = Array.from({ length: 12 }, (_, index) => index + 1);

function systemsFor(source: NonNullable<INtaHeatPumpInput['forfaitHeatPumpDraft']>['source']): SourceSystem[] {
  if (source === 'ground') return ['individual', 'collective_ground'];
  if (source === 'groundwater_below15_c' || source === 'surface_water')
    return ['individual', 'collective_groundwater_surface_or_at_least15_c'];
  if (source.startsWith('collective')) return ['collective_groundwater_surface_or_at_least15_c'];
  return ['individual'];
}

export function HybridHeatPumpMonthlyPanel({ pump }: { pump: INtaHeatPumpInput }) {
  const { t } = useI18n();
  const forfait = pump.forfaitHeatPumpDraft;
  const available = forfait ? systemsFor(forfait.source) : ['individual'] as SourceSystem[];
  const [sourceSystem, setSourceSystem] = useState<SourceSystem>(available[0]);
  const [newBuildConfirmed, setNewBuildConfirmed] = useState(false);
  const [nodeValues, setNodeValues] = useState<string[]>(months.map(() => ''));
  const [nodeReference, setNodeReference] = useState('');
  const [sourceReference, setSourceReference] = useState('');
  const [backupId, setBackupId] = useState('');
  const [backupClassReference, setBackupClassReference] = useState('');
  const [backupPower, setBackupPower] = useState('');
  const [backupPowerReference, setBackupPowerReference] = useState('');
  const [boilerKind, setBoilerKind] = useState<BoilerForfaitDraftInput['kind']>('hr107');
  const [boilerLocation, setBoilerLocation] = useState<BoilerForfaitDraftInput['location']>('inside_thermal_boundary');
  const [emissionTemperature, setEmissionTemperature] = useState('');
  const [emissionCircuit, setEmissionCircuit] = useState<BoilerForfaitDraftInput['emissionCircuit']>('direct');
  const [locationReference, setLocationReference] = useState('');
  const [emissionReference, setEmissionReference] = useState('');
  const [boilerYear, setBoilerYear] = useState('');
  const [boilerYearReference, setBoilerYearReference] = useState('');
  const [pilotFlamePresent, setPilotFlamePresent] = useState(false);
  const [result, setResult] = useState<HybridHeatPumpMonthlyDraftAssessment | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const sequence = useRef(0);
  const edition = useProjectEdition();
  useEffect(() => () => { sequence.current += 1; }, []);
  const invalidate = () => { sequence.current += 1; setResult(null); setError(null); setLoading(false); };
  const updateMonth = (index: number, value: string) => {
    invalidate(); setNodeValues((current) => current.map((item, offset) => offset === index ? value : item));
  };
  const diagnose = async () => {
    if (!forfait) return;
    if (pump.declaredOperatingLimits) {
      setError(t('kernel.hybridDraft.operatingLimitsUnsupported')); setResult(null); return;
    }
    const finitePositive = (value: string) => value.trim() !== '' && Number.isFinite(Number(value)) && Number(value) > 0;
    const validMonths = nodeValues.every((value) => value.trim() !== '' && Number.isFinite(Number(value)) && Number(value) >= 0);
    if (!newBuildConfirmed || !forfait.thermalCapacityKw || !forfait.capacitySourceReference?.trim() || !validMonths
      || !nodeReference.trim() || !sourceReference.trim() || !backupId.trim()
      || backupId.trim() === forfait.generatorId || !backupClassReference.trim() || !finitePositive(backupPower)
      || !backupPowerReference.trim() || !locationReference.trim() || !emissionReference.trim()
      || (boilerYear.trim() === '' && boilerYearReference.trim() !== '')
      || (boilerYear.trim() !== '' && (!Number.isInteger(Number(boilerYear)) || Number(boilerYear) < 1900 || Number(boilerYear) > 2026 || !boilerYearReference.trim()))
      || emissionTemperature.trim() === '' || !Number.isFinite(Number(emissionTemperature))) {
      setError(t('kernel.hybridDraft.invalid')); setResult(null); return;
    }
    const current = ++sequence.current;
    setLoading(true); setError(null); setResult(null);
    try {
      const lookup = await diagnoseForfaitHeatPumpDraftWithRust(forfait, edition);
      if (lookup.status === 'invalid' || !lookup.correctedCop) {
        throw new Error(t('kernel.hybridDraft.lookupInvalid'));
      }
      const boiler: BoilerForfaitDraftInput = {
        generatorId: backupId.trim(), role: 'individual_supplementary', location: boilerLocation,
        kind: boilerKind, fuel: 'natural_gas', averageDesignEmissionTemperatureC: Number(emissionTemperature),
        emissionCircuit, equipmentReference: backupClassReference.trim(), locationReference: locationReference.trim(),
        temperatureAndCircuitReference: emissionReference.trim(), pilotFlamePresent,
        installationYear: boilerYear.trim() === '' ? null : Number(boilerYear),
        installationYearReference: boilerYear.trim() === '' ? null : boilerYearReference.trim(),
      };
      const boilerLookup = await diagnoseBoilerForfaitDraftWithRust(boiler, edition);
      if (boilerLookup.status === 'invalid' || !boilerLookup.generationEfficiency) {
        throw new Error(t('kernel.hybridDraft.boilerLookupInvalid'));
      }
      const assessment = await diagnoseHybridHeatPumpMonthlyDraftWithRust({
        forfait, boiler, sourceSystem, sourceSystemReference: sourceReference.trim(),
        declaredOperatingLimitsPresent: false,
        heatPumpAuxiliaryMeasurements: pump.heatingAuxMeasuredDraft ? {
          generatorId: pump.heatingAuxMeasuredDraft.generatorId,
          generatorSourceReference: pump.heatingAuxMeasuredDraft.generatorSourceReference,
          measurements: pump.heatingAuxMeasuredDraft.measurements,
        } : null,
        dispatch: {
          nodeInputKwh: months.map((month, index) => ({ month, energyKwh: Number(nodeValues[index]) })),
          nodeInputReference: nodeReference.trim(), designContext: 'new_build',
          generators: [
            { id: forfait.generatorId, class: 'heat_pump', classificationReference: forfait.classificationSourceReference,
              nominalThermalPowerKw: forfait.thermalCapacityKw,
              powerReference: forfait.capacitySourceReference.trim(), priorityEfficiency: lookup.correctedCop,
              efficiencyReference: `forfait_sha256:${lookup.inputFingerprint}` },
            { id: backupId.trim(), class: 'other_boiler', classificationReference: backupClassReference.trim(),
              nominalThermalPowerKw: Number(backupPower),
              powerReference: backupPowerReference.trim(), priorityEfficiency: boilerLookup.generationEfficiency,
              efficiencyReference: `boiler_forfait_sha256:${boilerLookup.inputFingerprint}` },
          ],
        },
      }, edition);
      if (sequence.current === current) { setResult(assessment); setLoading(false); }
    } catch (reason: unknown) {
      if (sequence.current === current) {
        setError(reason instanceof Error ? reason.message : String(reason)); setLoading(false);
      }
    }
  };
  return <section className="heat-pump-forfait-monthly" aria-label={t('kernel.hybridDraft.title')}>
    <h4>{t('kernel.hybridDraft.title')}</h4>
    <p>{t('kernel.hybridDraft.scope')}</p>
    <label><input type="checkbox" checked={newBuildConfirmed} onChange={(event) => { invalidate(); setNewBuildConfirmed(event.target.checked); }} />{t('kernel.hybridDraft.newBuildConfirm')}</label>
    <p>{t('kernel.hybridDraft.pumpPower')}: {forfait?.thermalCapacityKw ?? '—'} kW · {forfait?.capacitySourceReference || t('kernel.hybridDraft.powerMissing')}</p>
    <p>{pump.heatingAuxMeasuredDraft ? t('kernel.hybridDraft.measuredAuxLinked') : t('kernel.hybridDraft.measuredAuxMissing')}</p>
    <div className="heat-pump-forfait-monthly-references">
      <label>{t('kernel.hybridDraft.nodeReference')}<input value={nodeReference} onChange={(event) => { invalidate(); setNodeReference(event.target.value); }} /></label>
      <label>{t('kernel.forfaitMonthly.sourceSystem')}<select value={sourceSystem} onChange={(event) => { invalidate(); setSourceSystem(event.target.value as SourceSystem); }}>
        {available.map((value) => <option key={value} value={value}>{t(`kernel.forfaitMonthly.sourceSystem.${value}`)}</option>)}
      </select></label>
      <label>{t('kernel.forfaitMonthly.systemReference')}<input value={sourceReference} onChange={(event) => { invalidate(); setSourceReference(event.target.value); }} /></label>
      <label>{t('kernel.hybridDraft.backupId')}<input value={backupId} onChange={(event) => { invalidate(); setBackupId(event.target.value); }} /></label>
      <label>{t('kernel.hybridDraft.backupClassReference')}<input value={backupClassReference} onChange={(event) => { invalidate(); setBackupClassReference(event.target.value); }} /></label>
      <label>{t('kernel.hybridDraft.backupPower')}<input type="number" min="0" step="any" value={backupPower} onChange={(event) => { invalidate(); setBackupPower(event.target.value); }} /></label>
      <label>{t('kernel.hybridDraft.backupPowerReference')}<input value={backupPowerReference} onChange={(event) => { invalidate(); setBackupPowerReference(event.target.value); }} /></label>
      <label>{t('kernel.hybridDraft.boilerKind')}<select value={boilerKind} onChange={(event) => { invalidate(); setBoilerKind(event.target.value as BoilerForfaitDraftInput['kind']); }}>
        {(['conventional', 'vr', 'hr100', 'hr104', 'hr107'] as const).map((kind) => <option key={kind} value={kind}>{t(`kernel.hybridDraft.boilerKind.${kind}`)}</option>)}
      </select></label>
      <label>{t('kernel.hybridDraft.boilerLocation')}<select value={boilerLocation} onChange={(event) => { invalidate(); setBoilerLocation(event.target.value as BoilerForfaitDraftInput['location']); }}>
        {(['inside_thermal_boundary', 'outside_thermal_boundary'] as const).map((location) => <option key={location} value={location}>{t(`kernel.hybridDraft.boilerLocation.${location}`)}</option>)}
      </select></label>
      <label>{t('kernel.hybridDraft.locationReference')}<input value={locationReference} onChange={(event) => { invalidate(); setLocationReference(event.target.value); }} /></label>
      <label>{t('kernel.hybridDraft.emissionTemperature')}<input type="number" step="any" value={emissionTemperature} onChange={(event) => { invalidate(); setEmissionTemperature(event.target.value); }} /></label>
      <label>{t('kernel.hybridDraft.emissionCircuit')}<select value={emissionCircuit} onChange={(event) => { invalidate(); setEmissionCircuit(event.target.value as BoilerForfaitDraftInput['emissionCircuit']); }}>
        {(['direct', 'mixing_with_return_limit', 'mixing_without_return_limit'] as const).map((circuit) => <option key={circuit} value={circuit}>{t(`kernel.hybridDraft.emissionCircuit.${circuit}`)}</option>)}
      </select></label>
      <label>{t('kernel.hybridDraft.emissionReference')}<input value={emissionReference} onChange={(event) => { invalidate(); setEmissionReference(event.target.value); }} /></label>
      <label>{t('kernel.hybridDraft.boilerYear')}<input type="number" min="1900" max="2026" step="1" value={boilerYear} onChange={(event) => { invalidate(); setBoilerYear(event.target.value); }} /></label>
      <label>{t('kernel.hybridDraft.boilerYearReference')}<input value={boilerYearReference} onChange={(event) => { invalidate(); setBoilerYearReference(event.target.value); }} /></label>
      <label><input type="checkbox" checked={pilotFlamePresent} onChange={(event) => { invalidate(); setPilotFlamePresent(event.target.checked); }} />{t('kernel.hybridDraft.pilotFlame')}</label>
    </div>
    <div className="heat-pump-forfait-monthly-table-wrap"><table><thead><tr><th>{t('kernel.forfaitMonthly.month')}</th><th>{t('kernel.hybridDraft.nodeInput')}</th></tr></thead>
      <tbody>{months.map((month, index) => <tr key={month}><th scope="row">{month}</th><td><input type="number" min="0" step="any"
        aria-label={`${t('kernel.hybridDraft.nodeInput')} ${month}`} value={nodeValues[index]}
        onChange={(event) => updateMonth(index, event.target.value)} /></td></tr>)}</tbody></table></div>
    <button type="button" disabled={loading} onClick={() => void diagnose()}>{loading ? t('kernel.loading') : t('kernel.hybridDraft.calculate')}</button>
    {error && <p role="alert" className="heat-pump-forfait-monthly-error">{error}</p>}
    {result?.status === 'invalid' && <p role="alert" className="heat-pump-forfait-monthly-error">{result.issues.map((item) => item.code).join(', ')}</p>}
    {result?.status === 'diagnostic_valid' && result.dispatch && result.heatPump && result.boiler && <div role="status" className="heat-pump-forfait-monthly-result">
      <div className="heat-pump-forfait-monthly-table-wrap"><table><thead><tr>
        <th>{t('kernel.forfaitMonthly.month')}</th><th>{t('kernel.hybridDraft.heatPumpOutput')}</th>
        <th>{t('kernel.hybridDraft.backupOutput')}</th><th>{t('kernel.forfaitMonthly.generatorElectricity')}</th>
        <th>{t('kernel.hybridDraft.boilerGas')}</th>
        <th>{t('kernel.hybridDraft.boilerAux')}</th>
        <th>{t('kernel.hybridDraft.pumpAux')}</th>
      </tr></thead><tbody>{result.dispatch.monthly.map((month, index) => <tr key={month.month}>
        <th scope="row">{month.month}</th>
        <td>{result.heatPump!.monthly[index].generatorOutputKwh.toFixed(2)}</td>
        <td>{month.generators.find((item) => item.generatorId === backupId.trim())?.deliveredHeatKwh.toFixed(2)}</td>
        <td>{result.heatPump!.monthly[index].generatorInputElectricityKwh.toFixed(2)}</td>
        <td>{result.boiler!.monthly[index].inputNaturalGasKwh.toFixed(2)}</td>
        <td>{result.boiler!.monthly[index].auxiliaryElectricityKwh?.toFixed(2) ?? '—'}</td>
        <td>{result.heatPumpAuxiliary?.auxiliary?.monthlyAuxiliaryElectricityKwh[index]?.electricityKwh.toFixed(2) ?? '—'}</td>
      </tr>)}</tbody></table></div>
      <p>{t('kernel.hybridDraft.warning')}</p>
      <small>{t('kernel.inputFingerprint')}: <code>{result.inputFingerprint}</code></small>
    </div>}
  </section>;
}
