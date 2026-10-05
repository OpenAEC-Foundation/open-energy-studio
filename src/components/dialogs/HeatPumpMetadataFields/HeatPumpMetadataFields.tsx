import { useState } from 'react';
import type { INtaHeatPumpAuxiliaryComponent, INtaHeatPumpDhwTestPoint, INtaHeatPumpInput, INtaHeatPumpOperatingLimits, INtaHeatPumpPerformancePoint, INtaHeatPumpSystemLink } from '../../../core/energy/types';
import { useEnergy } from '../../../context/EnergyContext';
import { useI18n } from '../../../i18n/i18n';
import './HeatPumpMetadataFields.css';

export type HeatPumpDraft = Omit<INtaHeatPumpInput, 'id'>;

export function hasElectricCarrierMismatch(value: HeatPumpDraft): boolean {
  return value.drive === 'electric_compression'
    && (value.performancePoints ?? []).some((point) => point.inputEnergyCarrier !== 'electricity');
}

export function hasIncompleteRegistryRecord(value: HeatPumpDraft): boolean {
  const record = value.performanceEvidence.registryRecord;
  return Boolean(record && (value.performanceEvidence.kind !== 'controlled_quality_declaration'
    || !record.registrationNumber.trim() || !record.productName.trim()
    || !record.manufacturer.trim() || !/^https:\/\/[^/\s]+/.test(record.sourceUrl)
    || /\s/.test(record.sourceUrl)));
}

export function hasDhwDeclarationMismatch(value: HeatPumpDraft): boolean {
  return Boolean(value.dhwTestPoints?.length)
    && (value.performanceEvidence.kind !== 'controlled_quality_declaration'
      || !['domestic_hot_water', 'combined_hydronic_and_hot_water'].includes(value.sink));
}

export function hasOperatingLimitEvidenceMismatch(value: HeatPumpDraft): boolean {
  return Boolean(value.declaredOperatingLimits)
    && value.performanceEvidence.kind !== 'controlled_quality_declaration';
}

export function defaultHeatPumpDraft(
  source: INtaHeatPumpInput['source'],
  sink: INtaHeatPumpInput['sink'],
): HeatPumpDraft {
  return {
    source, sink, drive: 'electric_compression',
    reversible: false, hybrid: false, booster: false,
    performanceEvidence: { kind: 'normative_default', reference: null },
  };
}

const sources: INtaHeatPumpInput['source'][] = [
  'outdoor_air', 'exhaust_air', 'ground', 'groundwater', 'surface_water',
  'district_water', 'waste_heat', 'other',
];
const sinks: INtaHeatPumpInput['sink'][] = [
  'indoor_air', 'hydronic', 'domestic_hot_water', 'combined_hydronic_and_hot_water',
];
const drives: INtaHeatPumpInput['drive'][] = ['electric_compression', 'gas_engine', 'absorption'];
const services: INtaHeatPumpPerformancePoint['service'][] = ['space_heating', 'domestic_hot_water', 'space_cooling'];
const carriers: INtaHeatPumpPerformancePoint['inputEnergyCarrier'][] = ['electricity', 'gas', 'district_heat', 'other'];
const auxiliaryKinds: INtaHeatPumpAuxiliaryComponent['kind'][] = [
  'source_pump', 'source_fan', 'indoor_fan', 'distribution_pump',
  'controls_standby', 'defrost', 'backup_heater', 'other',
];
const measurementBoundaries: INtaHeatPumpAuxiliaryComponent['measurementBoundary'][] = [
  'unknown', 'included_in_declared_performance', 'additional',
];
const linkRoles: INtaHeatPumpSystemLink['role'][] = [
  'backup_generator', 'upstream_heat_pump', 'shared_source', 'source_ventilation',
];
const linkTargetKinds: INtaHeatPumpSystemLink['targetKind'][] = [
  'heat_pump', 'heating_system', 'hot_water_system', 'ventilation_system',
];

type PointDraft = Omit<INtaHeatPumpPerformancePoint,
  'id' | 'sourceTemperatureC' | 'sinkTemperatureC' | 'usefulCapacityKw' | 'inputPowerKw'> & {
  sourceTemperatureC: string;
  sinkTemperatureC: string;
  usefulCapacityKw: string;
  inputPowerKw: string;
};

type AuxiliaryDraft = Omit<INtaHeatPumpAuxiliaryComponent, 'id' | 'nominalPowerW'> & {
  nominalPowerW: string;
};

type DhwDraft = Omit<INtaHeatPumpDhwTestPoint, 'id' | 'usefulEnergyKwhPerDay' | 'inputEnergyKwhPerDay' | 'nominalCapacityKw' | 'practiceFactor' | 'testSetpointC' | 'designSetpointC' | 'sourceAirFlowM3PerHour' | 'sourceAirDryBulbC' | 'sourceAirWetBulbC'> & {
  usefulEnergyKwhPerDay: string;
  inputEnergyKwhPerDay: string;
  nominalCapacityKw: string;
  practiceFactor: string;
  testSetpointC: string;
  designSetpointC: string;
  sourceAirFlowM3PerHour: string;
  sourceAirDryBulbC: string;
  sourceAirWetBulbC: string;
};

type OperatingLimitDraft = Omit<INtaHeatPumpOperatingLimits, 'minimumOperatingCop' | 'maximumSupplyTemperatureC'> & {
  minimumOperatingCop: string;
  maximumSupplyTemperatureC: string;
};

export function HeatPumpMetadataFields({
  value, onChange, hotWaterOnly = false, selfId,
}: {
  value: HeatPumpDraft;
  onChange: (next: HeatPumpDraft) => void;
  hotWaterOnly?: boolean;
  selfId?: string | null;
}) {
  const { t } = useI18n();
  const { state } = useEnergy();
  const [pointDraft, setPointDraft] = useState<PointDraft | null>(null);
  const [editingPointId, setEditingPointId] = useState<string | null>(null);
  const [pointError, setPointError] = useState<string | null>(null);
  const [dhwDraft, setDhwDraft] = useState<DhwDraft | null>(null);
  const [editingDhwId, setEditingDhwId] = useState<string | null>(null);
  const [dhwError, setDhwError] = useState<string | null>(null);
  const [operatingLimitDraft, setOperatingLimitDraft] = useState<OperatingLimitDraft | null>(null);
  const [operatingLimitError, setOperatingLimitError] = useState<string | null>(null);
  const [auxiliaryDraft, setAuxiliaryDraft] = useState<AuxiliaryDraft | null>(null);
  const [editingAuxiliaryId, setEditingAuxiliaryId] = useState<string | null>(null);
  const [auxiliaryError, setAuxiliaryError] = useState<string | null>(null);
  const [linkDraft, setLinkDraft] = useState<Omit<INtaHeatPumpSystemLink, 'id'> | null>(null);
  const [editingLinkId, setEditingLinkId] = useState<string | null>(null);
  const [linkError, setLinkError] = useState<string | null>(null);
  const change = <K extends keyof HeatPumpDraft>(key: K, next: HeatPumpDraft[K]) =>
    onChange({ ...value, [key]: next });
  const electricCarrierMismatch = hasElectricCarrierMismatch(value);
  const registryRecord = value.performanceEvidence.registryRecord;
  const updateRegistryField = (field: keyof NonNullable<HeatPumpDraft['performanceEvidence']['registryRecord']>, text: string) =>
    change('performanceEvidence', { ...value.performanceEvidence, registryRecord: {
      registrationNumber: '', productName: '', manufacturer: '', sourceUrl: '',
      ...registryRecord, [field]: text,
    } });
  const openOperatingLimits = () => {
    const existing = value.declaredOperatingLimits;
    setOperatingLimitDraft({
      minimumOperatingCop: existing?.minimumOperatingCop == null ? '' : String(existing.minimumOperatingCop),
      maximumSupplyTemperatureC: existing?.maximumSupplyTemperatureC == null ? '' : String(existing.maximumSupplyTemperatureC),
      declarationNormVersion: existing?.declarationNormVersion ?? '',
      sourceReference: existing?.sourceReference ?? '',
    });
    setOperatingLimitError(null);
  };
  const saveOperatingLimits = () => {
    if (!operatingLimitDraft) return;
    const minimumOperatingCop = operatingLimitDraft.minimumOperatingCop.trim() === ''
      ? undefined : Number(operatingLimitDraft.minimumOperatingCop);
    const maximumSupplyTemperatureC = operatingLimitDraft.maximumSupplyTemperatureC.trim() === ''
      ? undefined : Number(operatingLimitDraft.maximumSupplyTemperatureC);
    if (value.performanceEvidence.kind !== 'controlled_quality_declaration'
      || (minimumOperatingCop == null && maximumSupplyTemperatureC == null)
      || (minimumOperatingCop != null && (!Number.isFinite(minimumOperatingCop) || minimumOperatingCop <= 0))
      || (maximumSupplyTemperatureC != null && (!Number.isFinite(maximumSupplyTemperatureC) || maximumSupplyTemperatureC <= 0))
      || !operatingLimitDraft.declarationNormVersion.trim() || !operatingLimitDraft.sourceReference.trim()) {
      setOperatingLimitError(t('kernel.operatingLimits.invalid'));
      return;
    }
    change('declaredOperatingLimits', {
      minimumOperatingCop, maximumSupplyTemperatureC,
      declarationNormVersion: operatingLimitDraft.declarationNormVersion.trim(),
      sourceReference: operatingLimitDraft.sourceReference.trim(),
    });
    setOperatingLimitDraft(null);
    setOperatingLimitError(null);
  };
  const openPoint = (point?: INtaHeatPumpPerformancePoint) => {
    setEditingPointId(point?.id ?? null);
    setPointDraft({
      service: point?.service ?? (hotWaterOnly ? 'domestic_hot_water' : 'space_heating'),
      sourceTemperatureC: point ? String(point.sourceTemperatureC) : '',
      sinkTemperatureC: point ? String(point.sinkTemperatureC) : '',
      usefulCapacityKw: point ? String(point.usefulCapacityKw) : '',
      inputPowerKw: point ? String(point.inputPowerKw) : '',
      inputEnergyCarrier: point?.inputEnergyCarrier ?? (value.drive === 'electric_compression' ? 'electricity' : 'gas'),
      testReference: point?.testReference ?? '',
    });
    setPointError(null);
  };
  const savePoint = () => {
    if (!pointDraft) return;
    const fields = [pointDraft.sourceTemperatureC, pointDraft.sinkTemperatureC,
      pointDraft.usefulCapacityKw, pointDraft.inputPowerKw];
    const numbers = fields.map((field) => field.trim() === '' ? NaN : Number(field));
    if (numbers.some((number) => !Number.isFinite(number))
      || numbers[2] <= 0 || numbers[3] <= 0 || !pointDraft.testReference.trim()) {
      setPointError(t('kernel.points.invalid'));
      return;
    }
    if (!Number.isFinite(numbers[2] / numbers[3])) {
      setPointError(t('kernel.points.ratioOverflow'));
      return;
    }
    if ((value.performancePoints ?? []).some((existing) => existing.id !== editingPointId
      && existing.service === pointDraft.service
      && existing.sourceTemperatureC === numbers[0]
      && existing.sinkTemperatureC === numbers[1]
      && existing.usefulCapacityKw === numbers[2]
      && existing.inputEnergyCarrier === pointDraft.inputEnergyCarrier)) {
      setPointError(t('kernel.points.conditionDuplicate'));
      return;
    }
    if (value.drive === 'electric_compression' && pointDraft.inputEnergyCarrier !== 'electricity') {
      setPointError(t('kernel.points.carrierMismatch'));
      return;
    }
    const point: INtaHeatPumpPerformancePoint = {
      id: editingPointId ?? crypto.randomUUID(),
      service: pointDraft.service,
      sourceTemperatureC: numbers[0], sinkTemperatureC: numbers[1],
      usefulCapacityKw: numbers[2], inputPowerKw: numbers[3],
      inputEnergyCarrier: pointDraft.inputEnergyCarrier,
      testReference: pointDraft.testReference.trim(),
    };
    const points = value.performancePoints ?? [];
    change('performancePoints', editingPointId
      ? points.map((existing) => existing.id === editingPointId ? point : existing)
      : [...points, point]);
    setPointDraft(null);
    setEditingPointId(null);
    setPointError(null);
  };
  const openDhw = (point?: INtaHeatPumpDhwTestPoint) => {
    setEditingDhwId(point?.id ?? null);
    setDhwDraft({
      tapProfile: point?.tapProfile ?? '',
      usefulEnergyKwhPerDay: point ? String(point.usefulEnergyKwhPerDay) : '',
      inputEnergyKwhPerDay: point ? String(point.inputEnergyKwhPerDay) : '',
      nominalCapacityKw: point ? String(point.nominalCapacityKw) : '',
      practiceFactor: point ? String(point.practiceFactor) : '',
      testSetpointC: point ? String(point.testSetpointC) : '',
      designSetpointC: point ? String(point.designSetpointC) : '',
      sourceAirFlowM3PerHour: point?.sourceAirFlowM3PerHour == null ? '' : String(point.sourceAirFlowM3PerHour),
      sourceAirDryBulbC: point?.sourceAirDryBulbC == null ? '' : String(point.sourceAirDryBulbC),
      sourceAirWetBulbC: point?.sourceAirWetBulbC == null ? '' : String(point.sourceAirWetBulbC),
      declarationNormVersion: point?.declarationNormVersion ?? '',
      sourceReference: point?.sourceReference ?? '',
    });
    setDhwError(null);
  };
  const saveDhw = () => {
    if (!dhwDraft) return;
    const numericFields = ['usefulEnergyKwhPerDay', 'inputEnergyKwhPerDay', 'nominalCapacityKw',
      'practiceFactor', 'testSetpointC', 'designSetpointC'] as const;
    const numeric = numericFields.map((field) => dhwDraft[field].trim() === '' ? NaN : Number(dhwDraft[field]));
    const optionalAir = (['sourceAirFlowM3PerHour', 'sourceAirDryBulbC', 'sourceAirWetBulbC'] as const)
      .map((field) => dhwDraft[field].trim() === '' ? undefined : Number(dhwDraft[field]));
    if (!dhwDraft.tapProfile.trim() || !dhwDraft.declarationNormVersion.trim() || !dhwDraft.sourceReference.trim()
      || numeric.some((item) => !Number.isFinite(item)) || numeric.slice(0, 4).some((item) => item <= 0)
      || !Number.isFinite(numeric[0] / numeric[1])
      || optionalAir.some((item) => item !== undefined && !Number.isFinite(item))
      || (optionalAir[0] !== undefined && optionalAir[0] <= 0)
      || (value.dhwTestPoints ?? []).some((item) => item.id !== editingDhwId && item.tapProfile === dhwDraft.tapProfile.trim())
      || !['domestic_hot_water', 'combined_hydronic_and_hot_water'].includes(value.sink)) {
      setDhwError(t('kernel.dhwTest.invalid'));
      return;
    }
    const point: INtaHeatPumpDhwTestPoint = {
      id: editingDhwId ?? crypto.randomUUID(), tapProfile: dhwDraft.tapProfile.trim(),
      usefulEnergyKwhPerDay: numeric[0], inputEnergyKwhPerDay: numeric[1], nominalCapacityKw: numeric[2],
      practiceFactor: numeric[3], testSetpointC: numeric[4], designSetpointC: numeric[5],
      sourceAirFlowM3PerHour: optionalAir[0], sourceAirDryBulbC: optionalAir[1], sourceAirWetBulbC: optionalAir[2],
      declarationNormVersion: dhwDraft.declarationNormVersion.trim(), sourceReference: dhwDraft.sourceReference.trim(),
    };
    const points = value.dhwTestPoints ?? [];
    change('dhwTestPoints', editingDhwId ? points.map((item) => item.id === editingDhwId ? point : item) : [...points, point]);
    setDhwDraft(null);
  };
  const openAuxiliary = (component?: INtaHeatPumpAuxiliaryComponent) => {
    setEditingAuxiliaryId(component?.id ?? null);
    setAuxiliaryDraft({
      kind: component?.kind ?? 'source_pump',
      service: component?.service ?? (hotWaterOnly ? 'domestic_hot_water' : 'space_heating'),
      nominalPowerW: component ? String(component.nominalPowerW) : '',
      energyCarrier: component?.energyCarrier ?? 'electricity',
      measurementBoundary: component?.measurementBoundary ?? 'unknown',
      evidenceReference: component?.evidenceReference ?? '',
    });
    setAuxiliaryError(null);
  };
  const saveAuxiliary = () => {
    if (!auxiliaryDraft) return;
    const nominalPowerW = auxiliaryDraft.nominalPowerW.trim() === ''
      ? NaN : Number(auxiliaryDraft.nominalPowerW);
    if (!Number.isFinite(nominalPowerW) || nominalPowerW <= 0
      || !auxiliaryDraft.evidenceReference.trim()) {
      setAuxiliaryError(t('kernel.auxiliary.invalid'));
      return;
    }
    const component: INtaHeatPumpAuxiliaryComponent = {
      ...auxiliaryDraft,
      id: editingAuxiliaryId ?? crypto.randomUUID(),
      nominalPowerW,
      evidenceReference: auxiliaryDraft.evidenceReference.trim(),
    };
    const components = value.auxiliaryComponents ?? [];
    change('auxiliaryComponents', editingAuxiliaryId
      ? components.map((existing) => existing.id === editingAuxiliaryId ? component : existing)
      : [...components, component]);
    setAuxiliaryDraft(null);
    setEditingAuxiliaryId(null);
    setAuxiliaryError(null);
  };
  const linkTargets = (kind: INtaHeatPumpSystemLink['targetKind']) => {
    const project = state.project;
    if (kind === 'heating_system') return project.heatingSystems
      .filter((system) => system.id !== selfId).map((system) => ({ id: system.id, label: system.name || system.id }));
    if (kind === 'hot_water_system') return project.hotWaterSystems
      .filter((system) => system.id !== selfId).map((system) => ({ id: system.id, label: system.name || system.id }));
    if (kind === 'ventilation_system') return project.ventilationSystems
      .map((system) => ({ id: system.id, label: system.name || system.id }));
    return [
      ...(project.ntaHeatPumps ?? []).map((pump) => ({ id: pump.id, label: `${pump.source} → ${pump.sink}` })),
      ...project.heatingSystems.filter((system) => system.ntaHeatPump)
        .map((system) => ({ id: system.id, label: system.name || system.id })),
      ...project.hotWaterSystems.filter((system) => system.ntaHeatPump)
        .map((system) => ({ id: system.id, label: system.name || system.id })),
    ].filter((target) => target.id !== selfId);
  };
  const openLink = (link?: INtaHeatPumpSystemLink) => {
    setEditingLinkId(link?.id ?? null);
    const targetKind = link?.targetKind ?? 'heating_system';
    setLinkDraft({
      role: link?.role ?? 'backup_generator', targetKind,
      targetId: link?.targetId ?? linkTargets(targetKind)[0]?.id ?? '',
      evidenceReference: link?.evidenceReference ?? '',
    });
    setLinkError(null);
  };
  const saveLink = () => {
    if (!linkDraft || !linkDraft.targetId || !linkDraft.evidenceReference.trim()) {
      setLinkError(t('kernel.links.invalid'));
      return;
    }
    const link: INtaHeatPumpSystemLink = {
      ...linkDraft, id: editingLinkId ?? crypto.randomUUID(),
      evidenceReference: linkDraft.evidenceReference.trim(),
    };
    const links = value.systemLinks ?? [];
    change('systemLinks', editingLinkId
      ? links.map((existing) => existing.id === editingLinkId ? link : existing)
      : [...links, link]);
    setLinkDraft(null);
    setEditingLinkId(null);
    setLinkError(null);
  };
  const hotWaterSinks = sinks.slice(2);
  const availableSinks = hotWaterOnly
    ? hotWaterSinks.includes(value.sink) ? hotWaterSinks : [value.sink, ...hotWaterSinks]
    : sinks;

  return <fieldset className="heat-pump-metadata">
    <legend>{t('kernel.metadata.title')}</legend>
    <p>{t('kernel.metadata.scope')}</p>
    <div className="heat-pump-metadata-grid">
      <div className="dialog-field">
        <label htmlFor="heat-pump-source">{t('kernel.metadata.source')}</label>
        <select id="heat-pump-source" value={value.source} onChange={(event) => change('source', event.target.value as HeatPumpDraft['source'])}>
          {sources.map((source) => <option key={source} value={source}>{t(`kernel.metadata.source.${source}`)}</option>)}
        </select>
      </div>
      <div className="dialog-field">
        <label htmlFor="heat-pump-sink">{t('kernel.metadata.sink')}</label>
        <select id="heat-pump-sink" value={value.sink} onChange={(event) => change('sink', event.target.value as HeatPumpDraft['sink'])}>
          {availableSinks.map((sink) => <option key={sink} value={sink}>{t(`kernel.metadata.sink.${sink}`)}</option>)}
        </select>
      </div>
      <div className="dialog-field">
        <label htmlFor="heat-pump-drive">{t('kernel.metadata.drive')}</label>
        <select id="heat-pump-drive" value={value.drive} onChange={(event) => change('drive', event.target.value as HeatPumpDraft['drive'])}>
          {drives.map((drive) => <option key={drive} value={drive}>{t(`kernel.metadata.drive.${drive}`)}</option>)}
        </select>
      </div>
      <div className="dialog-field">
        <label htmlFor="heat-pump-evidence">{t('kernel.metadata.evidence')}</label>
        <select id="heat-pump-evidence" value={value.performanceEvidence.kind} onChange={(event) => change('performanceEvidence', {
          kind: event.target.value as HeatPumpDraft['performanceEvidence']['kind'],
          reference: value.performanceEvidence.reference,
          registryRecord: event.target.value === 'controlled_quality_declaration' ? registryRecord : undefined,
        })}>
          <option value="normative_default">{t('kernel.metadata.evidence.normative_default')}</option>
          <option value="controlled_quality_declaration">{t('kernel.metadata.evidence.controlled_quality_declaration')}</option>
        </select>
      </div>
    </div>
    {value.performanceEvidence.kind === 'controlled_quality_declaration' && <>
      <div className="dialog-field">
        <label htmlFor="heat-pump-evidence-reference">{t('kernel.metadata.reference')}</label>
        <input id="heat-pump-evidence-reference" type="text" value={value.performanceEvidence.reference ?? ''}
          onChange={(event) => change('performanceEvidence', { ...value.performanceEvidence, reference: event.target.value })} />
      </div>
      <div className="heat-pump-registry">
        <label><input type="checkbox" checked={Boolean(registryRecord)} onChange={(event) => change('performanceEvidence', {
          ...value.performanceEvidence,
          registryRecord: event.target.checked ? { registrationNumber: '', productName: '', manufacturer: '', sourceUrl: '' } : undefined,
        })} />{t('kernel.metadata.registryRecord')}</label>
        <p>{t('kernel.metadata.registryScope')}</p>
        {registryRecord && <div className="heat-pump-metadata-grid">
          {(['registrationNumber', 'productName', 'manufacturer', 'sourceUrl'] as const).map((field) =>
            <div className="dialog-field" key={field}>
              <label htmlFor={`hp-registry-${field}`}>{t(`kernel.metadata.registry.${field}`)}</label>
              <input id={`hp-registry-${field}`} type={field === 'sourceUrl' ? 'url' : 'text'}
                value={registryRecord[field]} onChange={(event) => updateRegistryField(field, event.target.value)} />
            </div>)}
        </div>}
      </div>
    </>}
    <div className="heat-pump-metadata-options">
      {(['reversible', 'hybrid', 'booster'] as const).map((key) => <label key={key}>
        <input type="checkbox" checked={value[key]} onChange={(event) => change(key, event.target.checked)} />
        {t(`kernel.metadata.${key}`)}
      </label>)}
    </div>
    <section className="heat-pump-points" aria-label={t('kernel.operatingLimits.title')}>
      <div className="heat-pump-points-heading">
        <div><strong>{t('kernel.operatingLimits.title')}</strong><p>{t('kernel.operatingLimits.scope')}</p></div>
        <button type="button" onClick={openOperatingLimits}
          disabled={value.performanceEvidence.kind !== 'controlled_quality_declaration'}>
          {value.declaredOperatingLimits ? t('kernel.inventory.edit') : t('kernel.operatingLimits.add')}
        </button>
      </div>
      {value.declaredOperatingLimits && <div className="heat-pump-point">
        <span>{value.declaredOperatingLimits.minimumOperatingCop == null ? ''
          : `COP ≥ ${value.declaredOperatingLimits.minimumOperatingCop} · `}
          {value.declaredOperatingLimits.maximumSupplyTemperatureC == null ? ''
            : `θsup ≤ ${value.declaredOperatingLimits.maximumSupplyTemperatureC} °C · `}
          {value.declaredOperatingLimits.declarationNormVersion}</span>
        <button type="button" onClick={() => change('declaredOperatingLimits', undefined)}>{t('kernel.operatingLimits.remove')}</button>
      </div>}
      {operatingLimitDraft && <div className="heat-pump-point-editor">
        <div className="heat-pump-metadata-grid">
          {(['minimumOperatingCop', 'maximumSupplyTemperatureC'] as const).map((field) =>
            <div className="dialog-field" key={field}>
              <label htmlFor={`hp-limit-${field}`}>{t(`kernel.operatingLimits.${field}`)}</label>
              <input id={`hp-limit-${field}`} type="number" step="any" value={operatingLimitDraft[field]}
                onChange={(event) => setOperatingLimitDraft({ ...operatingLimitDraft, [field]: event.target.value })} />
            </div>)}
          <div className="dialog-field"><label htmlFor="hp-limit-norm">{t('kernel.operatingLimits.normVersion')}</label>
            <input id="hp-limit-norm" value={operatingLimitDraft.declarationNormVersion}
              onChange={(event) => setOperatingLimitDraft({ ...operatingLimitDraft, declarationNormVersion: event.target.value })} /></div>
          <div className="dialog-field"><label htmlFor="hp-limit-source">{t('kernel.operatingLimits.source')}</label>
            <input id="hp-limit-source" value={operatingLimitDraft.sourceReference}
              onChange={(event) => setOperatingLimitDraft({ ...operatingLimitDraft, sourceReference: event.target.value })} /></div>
        </div>
        {operatingLimitError && <p role="alert" className="heat-pump-point-error">{operatingLimitError}</p>}
        <div className="heat-pump-point-actions"><button type="button" onClick={() => setOperatingLimitDraft(null)}>{t('dialog.cancel')}</button>
          <button type="button" onClick={saveOperatingLimits}>{t('kernel.operatingLimits.save')}</button></div>
      </div>}
    </section>
    <section className="heat-pump-points" aria-label={t('kernel.points.title')}>
      <div className="heat-pump-points-heading">
        <div><strong>{t('kernel.points.title')}</strong><p>{t('kernel.points.scope')}</p></div>
        <button type="button" onClick={() => openPoint()}>{t('kernel.points.add')}</button>
      </div>
      {electricCarrierMismatch && <p role="status" className="heat-pump-point-error">{t('kernel.points.carrierMismatch')}</p>}
      {(value.performancePoints ?? []).map((point) => <div className="heat-pump-point" key={point.id}>
        <span>{t(`kernel.points.service.${point.service}`)} · {point.usefulCapacityKw} kW / {point.inputPowerKw} kW · {point.testReference}</span>
        <div>
          <button type="button" onClick={() => openPoint(point)}>{t('kernel.inventory.edit')}</button>
          <button type="button" onClick={() => change('performancePoints',
            (value.performancePoints ?? []).filter((item) => item.id !== point.id))}
            aria-label={`${t('kernel.points.remove')}: ${point.id}`}>{t('kernel.points.remove')}</button>
        </div>
      </div>)}
      {pointDraft && <div className="heat-pump-point-editor">
        <div className="heat-pump-metadata-grid">
          <div className="dialog-field"><label htmlFor="hp-point-service">{t('kernel.points.service')}</label>
            <select id="hp-point-service" value={pointDraft.service}
              onChange={(event) => setPointDraft({ ...pointDraft, service: event.target.value as PointDraft['service'] })}>
              {services.map((service) => <option key={service} value={service}>{t(`kernel.points.service.${service}`)}</option>)}
            </select></div>
          {(['sourceTemperatureC', 'sinkTemperatureC', 'usefulCapacityKw', 'inputPowerKw'] as const).map((field) =>
            <div className="dialog-field" key={field}><label htmlFor={`hp-point-${field}`}>{t(`kernel.points.${field}`)}</label>
              <input id={`hp-point-${field}`} type="number" step="any" value={pointDraft[field]}
                onChange={(event) => setPointDraft({ ...pointDraft, [field]: event.target.value })} />
            </div>)}
          <div className="dialog-field"><label htmlFor="hp-point-carrier">{t('kernel.points.carrier')}</label>
            <select id="hp-point-carrier" value={pointDraft.inputEnergyCarrier}
              onChange={(event) => setPointDraft({ ...pointDraft, inputEnergyCarrier: event.target.value as PointDraft['inputEnergyCarrier'] })}>
              {carriers.map((carrier) => <option key={carrier} value={carrier}>{t(`kernel.points.carrier.${carrier}`)}</option>)}
            </select></div>
          <div className="dialog-field"><label htmlFor="hp-point-reference">{t('kernel.points.reference')}</label>
            <input id="hp-point-reference" type="text" value={pointDraft.testReference}
              onChange={(event) => setPointDraft({ ...pointDraft, testReference: event.target.value })} />
          </div>
        </div>
        {pointError && <p role="alert" className="heat-pump-point-error">{pointError}</p>}
        <div className="heat-pump-point-actions">
          <button type="button" onClick={() => setPointDraft(null)}>{t('dialog.cancel')}</button>
          <button type="button" onClick={savePoint}>{t('kernel.points.save')}</button>
        </div>
      </div>}
    </section>
    <section className="heat-pump-points" aria-label={t('kernel.dhwTest.title')}>
      <div className="heat-pump-points-heading">
        <div><strong>{t('kernel.dhwTest.title')}</strong><p>{t('kernel.dhwTest.scope')}</p></div>
        <button type="button" onClick={() => openDhw()}
          disabled={!['domestic_hot_water', 'combined_hydronic_and_hot_water'].includes(value.sink)
            || value.performanceEvidence.kind !== 'controlled_quality_declaration'}>{t('kernel.dhwTest.add')}</button>
      </div>
      {(value.dhwTestPoints ?? []).map((point) => <div className="heat-pump-point" key={point.id}>
        <span>{point.tapProfile} · {point.usefulEnergyKwhPerDay}/{point.inputEnergyKwhPerDay} kWh/d · {point.declarationNormVersion}</span>
        <div><button type="button" onClick={() => openDhw(point)}>{t('kernel.inventory.edit')}</button>
          <button type="button" onClick={() => change('dhwTestPoints',
            (value.dhwTestPoints ?? []).filter((item) => item.id !== point.id))}
            aria-label={`${t('kernel.dhwTest.remove')}: ${point.id}`}>{t('kernel.dhwTest.remove')}</button></div>
      </div>)}
      {dhwDraft && <div className="heat-pump-point-editor">
        <div className="heat-pump-metadata-grid">
          <div className="dialog-field"><label htmlFor="hp-dhw-tapProfile">{t('kernel.dhwTest.tapProfile')}</label>
            <input id="hp-dhw-tapProfile" value={dhwDraft.tapProfile}
              onChange={(event) => setDhwDraft({ ...dhwDraft, tapProfile: event.target.value })} /></div>
          {(['usefulEnergyKwhPerDay', 'inputEnergyKwhPerDay', 'nominalCapacityKw', 'practiceFactor',
            'testSetpointC', 'designSetpointC'] as const).map((field) => <div className="dialog-field" key={field}>
              <label htmlFor={`hp-dhw-${field}`}>{t(`kernel.dhwTest.${field}`)}</label>
              <input id={`hp-dhw-${field}`} type="number" step="any" value={dhwDraft[field]}
                onChange={(event) => setDhwDraft({ ...dhwDraft, [field]: event.target.value })} />
            </div>)}
          {(['sourceAirFlowM3PerHour', 'sourceAirDryBulbC', 'sourceAirWetBulbC'] as const).map((field) =>
            <div className="dialog-field" key={field}>
              <label htmlFor={`hp-dhw-${field}`}>{t(`kernel.dhwTest.${field}`)}</label>
              <input id={`hp-dhw-${field}`} type="number" step="any" value={dhwDraft[field]}
                onChange={(event) => setDhwDraft({ ...dhwDraft, [field]: event.target.value })} />
            </div>)}
          <div className="dialog-field"><label htmlFor="hp-dhw-norm">{t('kernel.dhwTest.normVersion')}</label>
            <input id="hp-dhw-norm" value={dhwDraft.declarationNormVersion}
              onChange={(event) => setDhwDraft({ ...dhwDraft, declarationNormVersion: event.target.value })} /></div>
          <div className="dialog-field"><label htmlFor="hp-dhw-source">{t('kernel.dhwTest.source')}</label>
            <input id="hp-dhw-source" value={dhwDraft.sourceReference}
              onChange={(event) => setDhwDraft({ ...dhwDraft, sourceReference: event.target.value })} /></div>
        </div>
        {dhwError && <p role="alert" className="heat-pump-point-error">{dhwError}</p>}
        <div className="heat-pump-point-actions"><button type="button" onClick={() => setDhwDraft(null)}>{t('dialog.cancel')}</button>
          <button type="button" onClick={saveDhw}>{t('kernel.dhwTest.save')}</button></div>
      </div>}
    </section>
    <section className="heat-pump-points" aria-label={t('kernel.auxiliary.title')}>
      <div className="heat-pump-points-heading">
        <div><strong>{t('kernel.auxiliary.title')}</strong><p>{t('kernel.auxiliary.scope')}</p></div>
        <button type="button" onClick={() => openAuxiliary()}>{t('kernel.auxiliary.add')}</button>
      </div>
      {(value.auxiliaryComponents ?? []).map((component) => <div className="heat-pump-point" key={component.id}>
        <span>{t(`kernel.auxiliary.kind.${component.kind}`)} · {component.nominalPowerW} W · {component.evidenceReference}</span>
        <div>
          <button type="button" onClick={() => openAuxiliary(component)}>{t('kernel.inventory.edit')}</button>
          <button type="button" onClick={() => change('auxiliaryComponents',
            (value.auxiliaryComponents ?? []).filter((item) => item.id !== component.id))}
            aria-label={`${t('kernel.auxiliary.remove')}: ${component.id}`}>{t('kernel.auxiliary.remove')}</button>
        </div>
      </div>)}
      {auxiliaryDraft && <div className="heat-pump-point-editor">
        <div className="heat-pump-metadata-grid">
          <div className="dialog-field"><label htmlFor="hp-aux-kind">{t('kernel.auxiliary.kind')}</label>
            <select id="hp-aux-kind" value={auxiliaryDraft.kind}
              onChange={(event) => setAuxiliaryDraft({ ...auxiliaryDraft, kind: event.target.value as AuxiliaryDraft['kind'] })}>
              {auxiliaryKinds.map((kind) => <option key={kind} value={kind}>{t(`kernel.auxiliary.kind.${kind}`)}</option>)}
            </select></div>
          <div className="dialog-field"><label htmlFor="hp-aux-service">{t('kernel.points.service')}</label>
            <select id="hp-aux-service" value={auxiliaryDraft.service}
              onChange={(event) => setAuxiliaryDraft({ ...auxiliaryDraft, service: event.target.value as AuxiliaryDraft['service'] })}>
              {services.map((service) => <option key={service} value={service}>{t(`kernel.points.service.${service}`)}</option>)}
            </select></div>
          <div className="dialog-field"><label htmlFor="hp-aux-power">{t('kernel.auxiliary.power')}</label>
            <input id="hp-aux-power" type="number" step="any" value={auxiliaryDraft.nominalPowerW}
              onChange={(event) => setAuxiliaryDraft({ ...auxiliaryDraft, nominalPowerW: event.target.value })} />
          </div>
          <div className="dialog-field"><label htmlFor="hp-aux-carrier">{t('kernel.points.carrier')}</label>
            <select id="hp-aux-carrier" value={auxiliaryDraft.energyCarrier}
              onChange={(event) => setAuxiliaryDraft({ ...auxiliaryDraft, energyCarrier: event.target.value as AuxiliaryDraft['energyCarrier'] })}>
              {carriers.map((carrier) => <option key={carrier} value={carrier}>{t(`kernel.points.carrier.${carrier}`)}</option>)}
            </select></div>
          <div className="dialog-field"><label htmlFor="hp-aux-boundary">{t('kernel.auxiliary.measurementBoundary')}</label>
            <select id="hp-aux-boundary" value={auxiliaryDraft.measurementBoundary}
              onChange={(event) => setAuxiliaryDraft({ ...auxiliaryDraft, measurementBoundary: event.target.value as AuxiliaryDraft['measurementBoundary'] })}>
              {measurementBoundaries.map((boundary) => <option key={boundary} value={boundary}>
                {t(`kernel.auxiliary.measurementBoundary.${boundary}`)}
              </option>)}
            </select></div>
          <div className="dialog-field"><label htmlFor="hp-aux-reference">{t('kernel.auxiliary.reference')}</label>
            <input id="hp-aux-reference" type="text" value={auxiliaryDraft.evidenceReference}
              onChange={(event) => setAuxiliaryDraft({ ...auxiliaryDraft, evidenceReference: event.target.value })} />
          </div>
        </div>
        {auxiliaryError && <p role="alert" className="heat-pump-point-error">{auxiliaryError}</p>}
        <div className="heat-pump-point-actions">
          <button type="button" onClick={() => setAuxiliaryDraft(null)}>{t('dialog.cancel')}</button>
          <button type="button" onClick={saveAuxiliary}>{t('kernel.auxiliary.save')}</button>
        </div>
      </div>}
    </section>
    <section className="heat-pump-points" aria-label={t('kernel.links.title')}>
      <div className="heat-pump-points-heading">
        <div><strong>{t('kernel.links.title')}</strong><p>{t('kernel.links.scope')}</p></div>
        <button type="button" onClick={() => openLink()}>{t('kernel.links.add')}</button>
      </div>
      {(value.systemLinks ?? []).map((link) => <div className="heat-pump-point" key={link.id}>
        <span>{t(`kernel.links.role.${link.role}`)} · {link.targetId} · {link.evidenceReference}</span>
        <div>
          <button type="button" onClick={() => openLink(link)}>{t('kernel.inventory.edit')}</button>
          <button type="button" onClick={() => change('systemLinks',
            (value.systemLinks ?? []).filter((item) => item.id !== link.id))}
            aria-label={`${t('kernel.links.remove')}: ${link.id}`}>{t('kernel.links.remove')}</button>
        </div>
      </div>)}
      {linkDraft && <div className="heat-pump-point-editor">
        <div className="heat-pump-metadata-grid">
          <div className="dialog-field"><label htmlFor="hp-link-role">{t('kernel.links.role')}</label>
            <select id="hp-link-role" value={linkDraft.role}
              onChange={(event) => {
                const role = event.target.value as INtaHeatPumpSystemLink['role'];
                const targetKind = role === 'source_ventilation' ? 'ventilation_system'
                  : role === 'backup_generator' && linkDraft.targetKind !== 'ventilation_system'
                    ? linkDraft.targetKind : 'heat_pump';
                setLinkDraft({ ...linkDraft, role, targetKind,
                  targetId: linkTargets(targetKind)[0]?.id ?? '' });
              }}>
              {linkRoles.filter((role) => role !== 'source_ventilation' || value.source === 'exhaust_air')
                .map((role) => <option key={role} value={role}>{t(`kernel.links.role.${role}`)}</option>)}
            </select></div>
          <div className="dialog-field"><label htmlFor="hp-link-kind">{t('kernel.links.targetKind')}</label>
            <select id="hp-link-kind" value={linkDraft.targetKind}
              onChange={(event) => {
                const targetKind = event.target.value as INtaHeatPumpSystemLink['targetKind'];
                setLinkDraft({ ...linkDraft, targetKind,
                  targetId: linkTargets(targetKind)[0]?.id ?? '' });
              }}>
              {linkTargetKinds.filter((kind) => linkDraft.role === 'source_ventilation'
                ? kind === 'ventilation_system'
                : linkDraft.role === 'backup_generator'
                  ? kind !== 'ventilation_system' : kind === 'heat_pump')
                .map((kind) => <option key={kind} value={kind}>{t(`kernel.links.targetKind.${kind}`)}</option>)}
            </select></div>
          <div className="dialog-field"><label htmlFor="hp-link-target">{t('kernel.links.target')}</label>
            <select id="hp-link-target" value={linkDraft.targetId}
              onChange={(event) => setLinkDraft({ ...linkDraft, targetId: event.target.value })}>
              {linkTargets(linkDraft.targetKind).length === 0 && <option value="">{t('kernel.links.noTargets')}</option>}
              {linkTargets(linkDraft.targetKind).map((target) => <option key={target.id} value={target.id}>
                {target.label} ({target.id})</option>)}
            </select></div>
          <div className="dialog-field"><label htmlFor="hp-link-reference">{t('kernel.links.reference')}</label>
            <input id="hp-link-reference" type="text" value={linkDraft.evidenceReference}
              onChange={(event) => setLinkDraft({ ...linkDraft, evidenceReference: event.target.value })} />
          </div>
        </div>
        {linkError && <p role="alert" className="heat-pump-point-error">{linkError}</p>}
        <div className="heat-pump-point-actions">
          <button type="button" onClick={() => setLinkDraft(null)}>{t('dialog.cancel')}</button>
          <button type="button" onClick={saveLink}>{t('kernel.links.save')}</button>
        </div>
      </div>}
    </section>
  </fieldset>;
}
