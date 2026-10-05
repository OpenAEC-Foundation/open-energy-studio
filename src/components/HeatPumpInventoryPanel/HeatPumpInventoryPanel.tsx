import { useState } from 'react';
import { Plus, Trash2 } from 'lucide-react';
import { useEnergy } from '../../context/EnergyContext';
import { useI18n } from '../../i18n/i18n';
import { defaultHeatPumpDraft, hasDhwDeclarationMismatch, hasElectricCarrierMismatch, hasIncompleteRegistryRecord, hasOperatingLimitEvidenceMismatch, HeatPumpMetadataFields, type HeatPumpDraft } from '../dialogs/HeatPumpMetadataFields/HeatPumpMetadataFields';
import { HeatPumpAuxDiagnosticPanel } from '../HeatPumpAuxDiagnosticPanel/HeatPumpAuxDiagnosticPanel';
import { HeatPumpForfaitDiagnosticPanel } from '../HeatPumpForfaitDiagnosticPanel/HeatPumpForfaitDiagnosticPanel';
import { HeatPumpForfaitMonthlyPanel } from '../HeatPumpForfaitMonthlyPanel/HeatPumpForfaitMonthlyPanel';
import { HybridHeatPumpMonthlyPanel } from '../HeatPumpForfaitMonthlyPanel/HybridHeatPumpMonthlyPanel';
import { GasHeatPumpForfaitPanel } from '../GasHeatPumpForfaitPanel/GasHeatPumpForfaitPanel';
import { GasHeatPumpAuxPanel } from '../GasHeatPumpAuxPanel/GasHeatPumpAuxPanel';
import { GasHeatPumpMonthlyPanel } from '../GasHeatPumpMonthlyPanel/GasHeatPumpMonthlyPanel';
import './HeatPumpInventoryPanel.css';

export function HeatPumpInventoryPanel() {
  const { state, dispatch } = useEnergy();
  const { t } = useI18n();
  const { project } = state;
  const pumps = project.ntaHeatPumps ?? [];
  const [editingId, setEditingId] = useState<string | null>(null);
  const [draft, setDraft] = useState<HeatPumpDraft | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [diagnosticPumpId, setDiagnosticPumpId] = useState<string | null>(null);
  const [forfaitPumpId, setForfaitPumpId] = useState<string | null>(null);
  const [monthlyPumpId, setMonthlyPumpId] = useState<string | null>(null);
  const [hybridPumpId, setHybridPumpId] = useState<string | null>(null);
  const [gasPumpId, setGasPumpId] = useState<string | null>(null);
  const [gasAuxPumpId, setGasAuxPumpId] = useState<string | null>(null);
  const [gasMonthlyPumpId, setGasMonthlyPumpId] = useState<string | null>(null);
  const diagnosticPump = pumps.find((pump) => pump.id === diagnosticPumpId && pump.drive === 'electric_compression'
    && pump.sink === 'hydronic' && !pump.hybrid && !pump.booster);
  const forfaitPump = pumps.find((pump) => pump.id === forfaitPumpId && pump.drive === 'electric_compression'
    && (pump.sink === 'hydronic' || (pump.sink === 'indoor_air' && pump.source === 'outdoor_air'))
    && !pump.booster);
  const monthlyPump = pumps.find((pump) => pump.id === monthlyPumpId && pump.forfaitHeatPumpDraft);
  const hybridPump = pumps.find((pump) => pump.id === hybridPumpId && pump.hybrid && pump.forfaitHeatPumpDraft
    && pump.drive === 'electric_compression' && pump.sink === 'hydronic' && !pump.booster);
  const gasPump = pumps.find((pump) => pump.id === gasPumpId && ['gas_engine', 'absorption'].includes(pump.drive)
    && pump.sink === 'hydronic' && !pump.booster);
  const gasAuxPump = pumps.find((pump) => pump.id === gasAuxPumpId && ['gas_engine', 'absorption'].includes(pump.drive)
    && pump.sink === 'hydronic' && !pump.booster && pump.gasHeatPumpForfaitDraft);
  const gasMonthlyPump = pumps.find((pump) => pump.id === gasMonthlyPumpId && ['gas_engine', 'absorption'].includes(pump.drive)
    && pump.sink === 'hydronic' && !pump.booster && pump.gasHeatPumpForfaitDraft);

  const startNew = () => {
    setEditingId(null);
    setDraft({ ...defaultHeatPumpDraft('outdoor_air', 'hydronic'), servedZoneIds: [] });
    setError(null);
  };
  const startEdit = (id: string) => {
    const pump = pumps.find((item) => item.id === id);
    if (!pump) return;
    const { id: _id, ...details } = pump;
    setEditingId(id);
    setDraft({ ...details, servedZoneIds: details.servedZoneIds ?? [] });
    setError(null);
  };
  const save = () => {
    if (!draft) return;
    if (hasOperatingLimitEvidenceMismatch(draft)) {
      setError(t('kernel.operatingLimits.invalid'));
      return;
    }
    if (draft.performanceEvidence.kind === 'controlled_quality_declaration'
      && !draft.performanceEvidence.reference?.trim()) {
      setError(t('kernel.inventory.referenceRequired'));
      return;
    }
    if (hasElectricCarrierMismatch(draft)) {
      setError(t('kernel.points.carrierMismatch'));
      return;
    }
    if (hasIncompleteRegistryRecord(draft)) {
      setError(t('kernel.metadata.registryIncomplete'));
      return;
    }
    if (hasDhwDeclarationMismatch(draft)) {
      setError(t('kernel.dhwTest.invalid'));
      return;
    }
    if (draft.heatingAuxMeasuredDraft && (draft.drive !== 'electric_compression'
      || draft.sink !== 'hydronic' || draft.hybrid || draft.booster)) {
      setError(t('kernel.auxDraft.removeFirst'));
      return;
    }
    const prior = pumps.find((item) => item.id === editingId);
    if (draft.forfaitHeatPumpDraft && (draft.drive !== 'electric_compression'
      || !['hydronic', 'indoor_air'].includes(draft.sink) || (draft.hybrid && draft.sink !== 'hydronic') || draft.booster
      || (prior && (draft.source !== prior.source || draft.sink !== prior.sink)))) {
      setError(t('kernel.forfait.removeFirst'));
      return;
    }
    if (draft.gasHeatPumpForfaitDraft && (!['gas_engine', 'absorption'].includes(draft.drive)
      || draft.drive !== draft.gasHeatPumpForfaitDraft.drive || draft.sink !== 'hydronic' || draft.booster
      || (prior && (draft.source !== prior.source || draft.sink !== prior.sink)))) {
      setError(t('kernel.gasForfait.removeFirst'));
      return;
    }
    if (draft.gasHeatPumpAuxDraft && (!draft.gasHeatPumpForfaitDraft
      || draft.drive !== draft.gasHeatPumpAuxDraft.drive || draft.sink !== 'hydronic' || draft.booster
      || draft.gasHeatPumpAuxDraft.nominalThermalCapacityKw !== draft.gasHeatPumpForfaitDraft.thermalCapacityKw)) {
      setError(t('kernel.gasAux.removeFirst'));
      return;
    }
    const pump = { id: editingId ?? crypto.randomUUID(), ...draft };
    dispatch({ type: editingId ? 'UPDATE_NTA_HEAT_PUMP' : 'ADD_NTA_HEAT_PUMP', payload: pump });
    setDraft(null);
    setEditingId(null);
    setError(null);
  };

  return <section className="heat-pump-inventory" aria-label={t('kernel.inventory.title')}>
    <div className="heat-pump-inventory-header">
      <div>
        <h2>{t('kernel.inventory.title')}</h2>
        <p>{t('kernel.inventory.scope')}</p>
      </div>
      <button type="button" onClick={startNew}><Plus size={15} />{t('kernel.inventory.add')}</button>
    </div>
    {pumps.length === 0 && <p className="heat-pump-inventory-empty">{t('kernel.inventory.empty')}</p>}
    {pumps.length > 0 && <ul className="heat-pump-inventory-list">
      {pumps.map((pump) => <li key={pump.id}>
        <span>{t(`kernel.metadata.source.${pump.source}`)} → {t(`kernel.metadata.sink.${pump.sink}`)}</span>
        <small>{pump.servedZoneIds?.map((zoneId) => project.zones.find((zone) => zone.id === zoneId)?.name ?? zoneId).join(', ') || t('kernel.inventory.noZones')}</small>
        <div>
          <button type="button" onClick={() => startEdit(pump.id)}>{t('kernel.inventory.edit')}</button>
          {pump.drive === 'electric_compression' && pump.sink === 'hydronic' && !pump.hybrid && !pump.booster &&
            <button type="button" onClick={() => setDiagnosticPumpId((current) => current === pump.id ? null : pump.id)}>{t('kernel.auxDraft.open')}</button>}
          {pump.drive === 'electric_compression' && (pump.sink === 'hydronic' || (pump.sink === 'indoor_air' && pump.source === 'outdoor_air')) && !pump.booster &&
            <button type="button" onClick={() => setForfaitPumpId((current) => current === pump.id ? null : pump.id)}>{t('kernel.forfait.open')}</button>}
          {pump.forfaitHeatPumpDraft && <button type="button" onClick={() => setMonthlyPumpId((current) => current === pump.id ? null : pump.id)}>{t('kernel.forfaitMonthly.open')}</button>}
          {pump.hybrid && pump.forfaitHeatPumpDraft && pump.sink === 'hydronic' && <button type="button"
            onClick={() => setHybridPumpId((current) => current === pump.id ? null : pump.id)}>{t('kernel.hybridDraft.open')}</button>}
          {['gas_engine', 'absorption'].includes(pump.drive) && pump.sink === 'hydronic' && !pump.booster && <button type="button"
            onClick={() => setGasPumpId((current) => current === pump.id ? null : pump.id)}>{t('kernel.gasForfait.open')}</button>}
          {pump.gasHeatPumpForfaitDraft && <button type="button"
            onClick={() => setGasAuxPumpId((current) => current === pump.id ? null : pump.id)}>{t('kernel.gasAux.open')}</button>}
          {pump.gasHeatPumpForfaitDraft && <button type="button"
            onClick={() => setGasMonthlyPumpId((current) => current === pump.id ? null : pump.id)}>{t('kernel.gasMonthly.open')}</button>}
          {pump.heatingAuxMeasuredDraft && <button type="button" onClick={() => {
            const { heatingAuxMeasuredDraft: _measured, ...withoutMeasured } = pump;
            dispatch({ type: 'UPDATE_NTA_HEAT_PUMP', payload: withoutMeasured });
            if (editingId === pump.id) setDraft((current) => current ? { ...current, heatingAuxMeasuredDraft: undefined } : current);
            setDiagnosticPumpId(null);
          }}>{t('kernel.auxDraft.remove')}</button>}
          {pump.forfaitHeatPumpDraft && <button type="button" onClick={() => {
            const { forfaitHeatPumpDraft: _forfait, ...withoutForfait } = pump;
            dispatch({ type: 'UPDATE_NTA_HEAT_PUMP', payload: withoutForfait });
            if (editingId === pump.id) setDraft((current) => current ? { ...current, forfaitHeatPumpDraft: undefined } : current);
            setForfaitPumpId(null);
            setMonthlyPumpId(null);
            setHybridPumpId(null);
          }}>{t('kernel.forfait.remove')}</button>}
          {pump.gasHeatPumpForfaitDraft && <button type="button" onClick={() => {
            const { gasHeatPumpForfaitDraft: _gasDraft, gasHeatPumpAuxDraft: _gasAux, ...withoutGasDraft } = pump;
            dispatch({ type: 'UPDATE_NTA_HEAT_PUMP', payload: withoutGasDraft });
            if (editingId === pump.id) setDraft((current) => current ? { ...current, gasHeatPumpForfaitDraft: undefined, gasHeatPumpAuxDraft: undefined } : current);
            setGasPumpId(null);
            setGasAuxPumpId(null);
            setGasMonthlyPumpId(null);
          }}>{t('kernel.gasForfait.remove')}</button>}
          {pump.gasHeatPumpAuxDraft && <button type="button" onClick={() => {
            const { gasHeatPumpAuxDraft: _gasAux, ...withoutGasAux } = pump;
            dispatch({ type: 'UPDATE_NTA_HEAT_PUMP', payload: withoutGasAux });
            if (editingId === pump.id) setDraft((current) => current ? { ...current, gasHeatPumpAuxDraft: undefined } : current);
            setGasAuxPumpId(null);
          }}>{t('kernel.gasAux.remove')}</button>}
          <button type="button" onClick={() => dispatch({ type: 'DELETE_NTA_HEAT_PUMP', payload: pump.id })} aria-label={`${t('kernel.inventory.remove')}: ${pump.id}`}><Trash2 size={15} /></button>
        </div>
      </li>)}
    </ul>}
    {diagnosticPump && <HeatPumpAuxDiagnosticPanel key={diagnosticPump.id} pump={diagnosticPump}
      onSave={(input) => dispatch({ type: 'UPDATE_NTA_HEAT_PUMP', payload: { ...diagnosticPump, heatingAuxMeasuredDraft: input } })} />}
    {forfaitPump && <HeatPumpForfaitDiagnosticPanel key={forfaitPump.id} pump={forfaitPump}
      buildingFunction={project.buildingFunction}
      onSave={(input) => dispatch({ type: 'UPDATE_NTA_HEAT_PUMP', payload: { ...forfaitPump, forfaitHeatPumpDraft: input } })} />}
    {monthlyPump && <HeatPumpForfaitMonthlyPanel key={`${monthlyPump.id}:${JSON.stringify(monthlyPump.forfaitHeatPumpDraft)}`} pump={monthlyPump} />}
    {hybridPump && <HybridHeatPumpMonthlyPanel key={`${hybridPump.id}:${JSON.stringify(hybridPump.forfaitHeatPumpDraft)}`} pump={hybridPump} />}
    {gasPump && <GasHeatPumpForfaitPanel key={gasPump.id} pump={gasPump} buildingFunction={project.buildingFunction}
      onSave={(input) => dispatch({ type: 'UPDATE_NTA_HEAT_PUMP', payload: { ...gasPump, gasHeatPumpForfaitDraft: input,
        gasHeatPumpAuxDraft: gasPump.gasHeatPumpAuxDraft?.nominalThermalCapacityKw === input.thermalCapacityKw
          ? gasPump.gasHeatPumpAuxDraft : undefined } })} />}
    {gasAuxPump && <GasHeatPumpAuxPanel key={gasAuxPump.id} pump={gasAuxPump}
      onSave={(input) => dispatch({ type: 'UPDATE_NTA_HEAT_PUMP', payload: { ...gasAuxPump, gasHeatPumpAuxDraft: input } })} />}
    {gasMonthlyPump && <GasHeatPumpMonthlyPanel key={`${gasMonthlyPump.id}:${JSON.stringify(gasMonthlyPump.gasHeatPumpForfaitDraft)}:${JSON.stringify(gasMonthlyPump.gasHeatPumpAuxDraft)}`} pump={gasMonthlyPump} />}
    {draft && <div className="heat-pump-inventory-editor">
      <HeatPumpMetadataFields value={draft} onChange={setDraft} selfId={editingId} />
      <fieldset>
        <legend>{t('kernel.inventory.zones')}</legend>
        {project.zones.map((zone) => <label key={zone.id}>
          <input type="checkbox" checked={draft.servedZoneIds?.includes(zone.id) ?? false}
            onChange={(event) => setDraft({ ...draft, servedZoneIds: event.target.checked
              ? [...(draft.servedZoneIds ?? []), zone.id]
              : (draft.servedZoneIds ?? []).filter((id) => id !== zone.id) })} />
          {zone.name || zone.id}
        </label>)}
      </fieldset>
      {error && <p className="heat-pump-inventory-error" role="alert">{error}</p>}
      <div className="heat-pump-inventory-actions">
        <button type="button" onClick={() => setDraft(null)}>{t('dialog.cancel')}</button>
        <button type="button" onClick={save}>{t('dialog.save')}</button>
      </div>
    </div>}
  </section>;
}
