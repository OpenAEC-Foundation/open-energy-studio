import { useEffect, useRef, useState } from 'react';
import type { BuildingFunction, INtaHeatPumpInput } from '../../core/energy/types';
import { diagnoseForfaitHeatPumpDraftWithRust, type ForfaitHeatPumpDraftAssessment, type ForfaitHeatPumpDraftInput } from '../../core/nta/KernelClient';
import { useI18n } from '../../i18n/i18n';
import './HeatPumpForfaitDiagnosticPanel.css';

type Source = ForfaitHeatPumpDraftInput['source'];
type TestCondition = NonNullable<ForfaitHeatPumpDraftInput['highEfficiencyEvidence']>['points'][number]['condition'];
const collectiveSources: Source[] = ['collective15_to20_c', 'collective20_to40_c', 'collective_at_least40_c'];

function highConditions(source: Source | ''): Array<{ condition: TestCondition; threshold: number; label: string }> {
  switch (source) {
    case 'ground': return [
      { condition: 'b0_w45', threshold: 3, label: 'B0/W45' },
      { condition: 'b0_w35', threshold: 3.5, label: 'B0/W35' },
    ];
    case 'groundwater_below15_c': return [
      { condition: 'w10_w45', threshold: 3.75, label: 'W10/W45' },
      { condition: 'w10_w35', threshold: 4.4, label: 'W10/W35' },
    ];
    case 'outdoor_air': return [
      { condition: 'a7_wet6_w45', threshold: 2.75, label: 'A7(6)/W45' },
      { condition: 'a7_wet6_w35', threshold: 2.85, label: 'A7(6)/W35' },
      { condition: 'a_minus7_wet_minus8_w45', threshold: 1.9, label: 'A−7(−8)/W45' },
    ];
    default: return [];
  }
}

function sourcesFor(pump: INtaHeatPumpInput): Source[] {
  if (pump.sink === 'indoor_air') return pump.source === 'outdoor_air' ? ['outdoor_air'] : [];
  switch (pump.source) {
    case 'outdoor_air': return ['outdoor_air'];
    case 'exhaust_air': return ['exhaust_air'];
    case 'ground': return ['ground', ...collectiveSources];
    case 'groundwater': return ['groundwater_below15_c', 'ground_or_groundwater_unknown', ...collectiveSources];
    case 'surface_water': return ['surface_water', ...collectiveSources];
    default: return ['ground_or_groundwater_unknown', ...collectiveSources];
  }
}

export function HeatPumpForfaitDiagnosticPanel({ pump, buildingFunction, onSave }: {
  pump: INtaHeatPumpInput;
  buildingFunction: BuildingFunction;
  onSave: (input: ForfaitHeatPumpDraftInput) => void;
}) {
  const { t } = useI18n();
  const saved = pump.forfaitHeatPumpDraft;
  const options = sourcesFor(pump);
  const [scope, setScope] = useState<ForfaitHeatPumpDraftInput['scope']>(
    saved?.scope ?? (buildingFunction === 'residential' ? 'residential_at_most25_kw' : 'utility_collective_or_over25_kw'));
  const [source, setSource] = useState<Source | ''>(saved?.source ?? (options.length === 1 ? options[0] : ''));
  const [temperature, setTemperature] = useState(saved?.designSupplyTemperatureC == null ? '' : String(saved.designSupplyTemperatureC));
  const [factor, setFactor] = useState(saved?.sourceCorrectionFactor == null ? '' : String(saved.sourceCorrectionFactor));
  const [factorReference, setFactorReference] = useState(saved?.sourceCorrectionReference ?? '');
  const [classificationReference, setClassificationReference] = useState(saved?.classificationSourceReference ?? pump.performanceEvidence.reference ?? '');
  const [capacity, setCapacity] = useState(saved?.thermalCapacityKw == null ? '' : String(saved.thermalCapacityKw));
  const [capacityReference, setCapacityReference] = useState(saved?.capacitySourceReference ?? '');
  const [collective, setCollective] = useState(saved?.collectiveBuildingInstallation == null ? '' : String(saved.collectiveBuildingInstallation));
  const [rowVariant, setRowVariant] = useState<NonNullable<ForfaitHeatPumpDraftInput['rowVariant']>>(saved?.rowVariant ?? 'base');
  const [productReference, setProductReference] = useState(saved?.highEfficiencyEvidence?.productReference ?? '');
  const [testReportReference, setTestReportReference] = useState(saved?.highEfficiencyEvidence?.testReportReference ?? '');
  const [testCops, setTestCops] = useState<Partial<Record<TestCondition, string>>>(() => Object.fromEntries(
    saved?.highEfficiencyEvidence?.points.map((point) => [point.condition, String(point.measuredCop)]) ?? [],
  ));
  const [sourceTemperature, setSourceTemperature] = useState(saved?.sourceTemperatureC == null ? '' : String(saved.sourceTemperatureC));
  const [sourceTemperatureReference, setSourceTemperatureReference] = useState(saved?.sourceTemperatureEvidenceReference ?? '');
  const [sourceQualityReference, setSourceQualityReference] = useState(saved?.sourceQualityDeclarationReference ?? '');
  const [result, setResult] = useState<ForfaitHeatPumpDraftAssessment | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const sequence = useRef(0);

  useEffect(() => () => { sequence.current += 1; }, []);
  const invalidate = () => { sequence.current += 1; setResult(null); setError(null); setLoading(false); };
  const sourceQualityRelevant = source === 'collective20_to40_c' || source === 'collective_at_least40_c';
  const fallbackToGroundwater = sourceQualityRelevant && !sourceQualityReference.trim();
  const fallbackToGround = source === 'ground_or_groundwater_unknown';
  const correctionRequired = scope === 'residential_at_most25_kw'
    && (source === 'ground' || source === 'groundwater_below15_c' || fallbackToGroundwater || fallbackToGround);
  const conditions = highConditions(source);
  const highRow = rowVariant === 'table_9_28_high_efficiency';
  const highRowAvailable = scope === 'residential_at_most25_kw' && pump.sink === 'hydronic' && conditions.length > 0;
  const sourceTemperatureBounds = (() => {
    switch (source) {
      case 'groundwater_below15_c': return [-Infinity, 15] as const;
      case 'collective15_to20_c': return [15, 20] as const;
      case 'collective20_to40_c': return [20, 40] as const;
      case 'collective_at_least40_c': return [40, Infinity] as const;
      default: return null;
    }
  })();

  const buildInput = (): ForfaitHeatPumpDraftInput | null => {
    const supply = pump.sink === 'hydronic' && temperature.trim() !== '' ? Number(temperature) : null;
    const correction = correctionRequired && factor.trim() !== '' ? Number(factor) : null;
    const thermalCapacity = capacity.trim() === '' ? null : Number(capacity);
    const sourceTemperatureValue = sourceTemperatureBounds && sourceTemperature.trim() !== '' ? Number(sourceTemperature) : null;
    if (!source || !options.includes(source) || !classificationReference.trim()
      || (sourceTemperatureBounds && (sourceTemperatureValue === null || !Number.isFinite(sourceTemperatureValue)
        || sourceTemperatureValue < sourceTemperatureBounds[0] || sourceTemperatureValue >= sourceTemperatureBounds[1]
        || !sourceTemperatureReference.trim()))
      || thermalCapacity === null || !Number.isFinite(thermalCapacity) || thermalCapacity <= 0
      || !capacityReference.trim() || collective === ''
      || (scope === 'residential_at_most25_kw' && (thermalCapacity > 25 || collective === 'true'))
      || (scope === 'utility_collective_or_over25_kw' && buildingFunction === 'residential'
        && collective === 'false' && thermalCapacity <= 25)
      || (highRow && (!highRowAvailable || !productReference.trim() || !testReportReference.trim()
        || conditions.some((item) => {
          const measured = Number(testCops[item.condition]);
          return !testCops[item.condition]?.trim() || !Number.isFinite(measured) || measured <= item.threshold;
        })))
      || (scope === 'residential_at_most25_kw' && buildingFunction !== 'residential')
      || (pump.sink === 'hydronic' && (supply === null || !Number.isFinite(supply) || supply <= 0 || supply > 70))
      || (scope === 'residential_at_most25_kw' && source === 'surface_water')
      || (source === 'collective_at_least40_c' && !fallbackToGroundwater && supply !== null && supply <= 40)
      || (correctionRequired && (correction === null || !Number.isFinite(correction) || correction <= 0 || !factorReference.trim()))) {
      setError(t('kernel.forfait.invalid'));
      setResult(null);
      return null;
    }
    return {
      generatorId: pump.id,
      classificationSourceReference: classificationReference.trim(),
      scope,
      source,
      sink: pump.sink === 'indoor_air' ? 'indoor_air' : 'hydronic',
      designSupplyTemperatureC: supply,
      sourceCorrectionFactor: correction,
      sourceCorrectionReference: correctionRequired ? factorReference.trim() : null,
      thermalCapacityKw: thermalCapacity,
      capacitySourceReference: capacityReference.trim(),
      collectiveBuildingInstallation: collective === 'true',
      rowVariant,
      highEfficiencyEvidence: highRow ? {
        productReference: productReference.trim(),
        testReportReference: testReportReference.trim(),
        testStandardEdition: 'NEN-EN 14511-2:2022',
        points: conditions.map((item) => ({ condition: item.condition, measuredCop: Number(testCops[item.condition]) })),
      } : null,
      sourceTemperatureC: sourceTemperatureValue,
      sourceTemperatureEvidenceReference: sourceTemperatureBounds ? sourceTemperatureReference.trim() : null,
      sourceQualityDeclarationReference: sourceQualityRelevant && sourceQualityReference.trim() ? sourceQualityReference.trim() : null,
    };
  };

  const diagnose = async (saveInput: boolean) => {
    const input = buildInput();
    if (!input) return;
    const current = ++sequence.current;
    setLoading(true); setError(null); setResult(null);
    try {
      const assessment = await diagnoseForfaitHeatPumpDraftWithRust(input);
      if (sequence.current === current) {
        setResult(assessment);
        setLoading(false);
        if (saveInput && assessment.status === 'input_valid') onSave(input);
      }
    } catch (reason: unknown) {
      if (sequence.current === current) {
        setError(reason instanceof Error ? reason.message : String(reason)); setLoading(false);
      }
    }
  };

  return <section className="heat-pump-forfait-draft" aria-label={t('kernel.forfait.title')}>
    <h4>{t('kernel.forfait.title')}</h4>
    <p>{t('kernel.forfait.scopeWarning')}</p>
    <div className="heat-pump-forfait-draft-grid">
      <label><span>{t('kernel.forfait.scope')}</span><select value={scope} onChange={(event) => {
        invalidate(); setScope(event.target.value as ForfaitHeatPumpDraftInput['scope']); setRowVariant('base');
      }}>
        {buildingFunction === 'residential' && <option value="residential_at_most25_kw">{t('kernel.forfait.residential')}</option>}
        <option value="utility_collective_or_over25_kw">{t('kernel.forfait.utility')}</option>
      </select></label>
      <label><span>{t('kernel.forfait.source')}</span><select value={source} onChange={(event) => {
        invalidate(); setSource(event.target.value as Source | ''); setRowVariant('base');
      }}><option value="">{t('kernel.forfait.choose')}</option>
        {options.map((item) => <option key={item} value={item}>{t(`kernel.forfait.source.${item}`)}</option>)}
      </select></label>
      {sourceTemperatureBounds && <>
        <label><span>{t('kernel.forfait.sourceTemperature')}</span><input type="number" step="any" value={sourceTemperature}
          onChange={(event) => { invalidate(); setSourceTemperature(event.target.value); }} /></label>
        <label><span>{t('kernel.forfait.sourceTemperatureReference')}</span><input type="text" value={sourceTemperatureReference}
          onChange={(event) => { invalidate(); setSourceTemperatureReference(event.target.value); }} /></label>
      </>}
      {sourceQualityRelevant && <label><span>{t('kernel.forfait.sourceQualityReference')}</span><input type="text" value={sourceQualityReference}
        onChange={(event) => { invalidate(); setSourceQualityReference(event.target.value); }} /></label>}
      {fallbackToGroundwater && <p className="heat-pump-forfait-draft-warning">{t('kernel.forfait.sourceFallback')}</p>}
      {fallbackToGround && <p className="heat-pump-forfait-draft-warning">{t('kernel.forfait.unknownGroundFallback')}</p>}
      <label><span>{t('kernel.forfait.rowVariant')}</span><select value={rowVariant} onChange={(event) => {
        invalidate(); setRowVariant(event.target.value as NonNullable<ForfaitHeatPumpDraftInput['rowVariant']>);
      }}><option value="base">{t('kernel.forfait.baseRow')}</option>
        {highRowAvailable && <option value="table_9_28_high_efficiency">{t('kernel.forfait.highRow')}</option>}
      </select></label>
      {pump.sink === 'hydronic' && <label><span>{t('kernel.forfait.temperature')}</span><input type="number" min="0" max="70" step="any"
        value={temperature} onChange={(event) => { invalidate(); setTemperature(event.target.value); }} /></label>}
      <label><span>{t('kernel.forfait.classificationReference')}</span><input type="text" value={classificationReference}
        onChange={(event) => { invalidate(); setClassificationReference(event.target.value); }} /></label>
      {highRow && <>
        <label><span>{t('kernel.forfait.productReference')}</span><input type="text" value={productReference}
          onChange={(event) => { invalidate(); setProductReference(event.target.value); }} /></label>
        <label><span>{t('kernel.forfait.testReportReference')}</span><input type="text" value={testReportReference}
          onChange={(event) => { invalidate(); setTestReportReference(event.target.value); }} /></label>
        {conditions.map((item) => <label key={item.condition}><span>{item.label} · COP &gt; {item.threshold}</span>
          <input type="number" min={item.threshold} step="any" value={testCops[item.condition] ?? ''}
            onChange={(event) => { invalidate(); setTestCops((current) => ({ ...current, [item.condition]: event.target.value })); }} /></label>)}
      </>}
      <label><span>{t('kernel.forfait.capacity')}</span><input type="number" min="0" step="any" value={capacity}
        onChange={(event) => { invalidate(); setCapacity(event.target.value); }} /></label>
      <label><span>{t('kernel.forfait.capacityReference')}</span><input type="text" value={capacityReference}
        onChange={(event) => { invalidate(); setCapacityReference(event.target.value); }} /></label>
      <label><span>{t('kernel.forfait.collective')}</span><select value={collective} onChange={(event) => {
        invalidate(); setCollective(event.target.value);
      }}><option value="">{t('kernel.forfait.chooseArrangement')}</option>
        <option value="false">{t('kernel.forfait.individual')}</option>
        <option value="true">{t('kernel.forfait.collectiveYes')}</option>
      </select></label>
      {correctionRequired && <>
        <label><span>{t('kernel.forfait.factor')}</span><input type="number" min="0" step="any" value={factor}
          onChange={(event) => { invalidate(); setFactor(event.target.value); }} /></label>
        <label><span>{t('kernel.forfait.factorReference')}</span><input type="text" value={factorReference}
          onChange={(event) => { invalidate(); setFactorReference(event.target.value); }} /></label>
      </>}
    </div>
    <div className="heat-pump-forfait-draft-actions">
      <button type="button" onClick={() => void diagnose(false)} disabled={loading}>{loading ? t('kernel.loading') : t('kernel.forfait.calculate')}</button>
      <button type="button" onClick={() => void diagnose(true)} disabled={loading}>{t('kernel.forfait.save')}</button>
    </div>
    {saved && <p>{t('kernel.forfait.saved')}</p>}
    {error && <p role="alert" className="heat-pump-forfait-draft-error">{error}</p>}
    {result?.status === 'invalid' && <p role="alert" className="heat-pump-forfait-draft-error">
      {t('kernel.forfait.invalid')} {result.issues.map((item) => item.code).join(', ')}
    </p>}
    {result?.status === 'input_valid' && <div className="heat-pump-forfait-draft-result" role="status">
      <strong>{t('kernel.forfait.result')}: {result.correctedCop?.toFixed(2)}</strong>
      <p>{t('kernel.forfait.table')} {result.table} · {t(result.rowVariant === 'table_9_28_high_efficiency'
        ? 'kernel.forfait.highRow' : 'kernel.forfait.baseRow')}{result.temperatureBand ? ` · ${result.temperatureBand}` : ''}</p>
      {result.sourceFallbackApplied && <p>{t('kernel.forfait.sourceFallbackResult')} {t(`kernel.forfait.source.${result.selectedSource}`)}</p>}
      <p>{t('kernel.forfait.resultWarning')}</p>
      <small>{t('kernel.inputFingerprint')}: <code>{result.inputFingerprint}</code></small>
    </div>}
  </section>;
}
