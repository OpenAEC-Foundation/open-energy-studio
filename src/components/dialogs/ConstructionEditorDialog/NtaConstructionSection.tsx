import { useState } from 'react';
import { useI18n } from '../../../i18n/i18n';
import type { IConstructionLayer } from '../../../core/energy/types';
import { calculateConstructionsWithRust, type EnvelopeAssessment } from '../../../core/nta/KernelClient';
import {
  constructionDraft, envelopeFromForfait, envelopeFromLayers,
  type NtaConstructionDraft, type NtaForfaitDraft, type NtaLayerDraft,
} from '../../../core/nta/NtaFormModels';

export interface NtaConstructionResult {
  rc: number | null;
  u: number;
  route: string;
}

const cell = { padding: '2px 4px' } as const;

/**
 * NTA 8800 U/R_c of a construction through the Rust kernel (§8.2.2, annexes
 * C/E/F, or the annex I defaults for existing buildings). The project keeps
 * its simple Σd/λ value until the user applies the kernel result.
 */
export function NtaConstructionSection({ layers, onApply }: {
  layers: IConstructionLayer[];
  onApply: (result: NtaConstructionResult) => void;
}) {
  const { t } = useI18n();
  const [mode, setMode] = useState<'layers' | 'forfait'>('layers');
  const [draft, setDraft] = useState<NtaConstructionDraft>(() => constructionDraft({ layers }));
  const [forfait, setForfait] = useState<NtaForfaitDraft>({
    element: 'facade', constructionYear: 1980, insulation: 'absent_or_unknown', thicknessMm: 0, cavity: true,
  });
  const [result, setResult] = useState<EnvelopeAssessment | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const updateLayer = (index: number, layer: NtaLayerDraft) =>
    setDraft((current) => ({ ...current, layers: current.layers.map((item, i) => (i === index ? layer : item)) }));

  const calculate = async () => {
    setBusy(true);
    setError(null);
    try {
      const input = mode === 'layers' ? envelopeFromLayers('construction', draft) : envelopeFromForfait('construction', forfait);
      setResult(await calculateConstructionsWithRust(input));
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
      setResult(null);
    } finally {
      setBusy(false);
    }
  };

  const element = result?.status === 'calculated_unverified' ? result.elements[0] : null;

  return <fieldset className="dialog-field" aria-label={t('nta.construction.title')}>
    <legend>{t('nta.construction.title')}</legend>
    <p style={{ fontSize: 12, color: 'var(--text-secondary)' }}>{t('nta.construction.help')}</p>
    <label>{t('nta.construction.mode')}
      <select value={mode} onChange={(event) => { setMode(event.target.value as 'layers' | 'forfait'); setResult(null); }}>
        <option value="layers">{t('nta.construction.mode.layers')}</option>
        <option value="forfait">{t('nta.construction.mode.forfait')}</option>
      </select>
    </label>
    {mode === 'layers' ? <>
      <div style={{ display: 'flex', gap: 12, flexWrap: 'wrap' }}>
        <label>{t('nta.construction.heatFlow')}
          <select value={draft.heatFlow} onChange={(event) => setDraft({ ...draft, heatFlow: event.target.value as NtaConstructionDraft['heatFlow'] })}>
            <option value="horizontal">{t('nta.construction.heatFlow.horizontal')}</option>
            <option value="upward">{t('nta.construction.heatFlow.upward')}</option>
            <option value="downward">{t('nta.construction.heatFlow.downward')}</option>
          </select>
        </label>
        <label><input type="checkbox" checked={draft.exteriorAir}
          onChange={(event) => setDraft({ ...draft, exteriorAir: event.target.checked })} /> {t('nta.construction.exterior')}</label>
      </div>
      <table style={{ width: '100%', borderCollapse: 'collapse' }}>
        <thead><tr>
          <th>{t('nta.construction.layerKind')}</th><th>{t('dialog.construction.material')}</th>
          <th>{t('nta.construction.thickness')}</th><th>{t('nta.construction.lambdaOrVentilation')}</th><th>{t('nta.form.source')}</th><th />
        </tr></thead>
        <tbody>{draft.layers.map((layer, index) => <tr key={index}>
          <td style={cell}>
            <select aria-label={t('nta.construction.layerKind')} value={layer.kind} onChange={(event) => updateLayer(index, event.target.value === 'air_cavity'
              ? { kind: 'air_cavity', name: layer.name, thicknessMm: 40, ventilation: 'unventilated' }
              : { kind: 'material', name: layer.name, thicknessM: 0.1, lambda: 0.04, sourceReference: '' })}>
              <option value="material">{t('nta.construction.kind.material')}</option>
              <option value="air_cavity">{t('nta.construction.kind.cavity')}</option>
            </select>
          </td>
          <td style={cell}><input value={layer.name} onChange={(event) => updateLayer(index, { ...layer, name: event.target.value })} /></td>
          {layer.kind === 'material' ? <>
            <td style={cell}><input type="number" step="0.001" aria-label={`${t('nta.construction.thickness')} ${index + 1}`} value={layer.thicknessM}
              onChange={(event) => updateLayer(index, { ...layer, thicknessM: Number(event.target.value) })} /></td>
            <td style={cell}><input type="number" step="0.001" aria-label={`λ ${index + 1}`} value={layer.lambda}
              onChange={(event) => updateLayer(index, { ...layer, lambda: Number(event.target.value) })} /></td>
            <td style={cell}><input value={layer.sourceReference}
              onChange={(event) => updateLayer(index, { ...layer, sourceReference: event.target.value })} /></td>
          </> : <>
            <td style={cell}><input type="number" step="1" aria-label={`mm ${index + 1}`} value={layer.thicknessMm}
              onChange={(event) => updateLayer(index, { ...layer, thicknessMm: Number(event.target.value) })} /> mm</td>
            <td style={cell}><select value={layer.ventilation}
              onChange={(event) => updateLayer(index, { ...layer, ventilation: event.target.value as 'unventilated' | 'weakly' | 'strongly' })}>
              <option value="unventilated">{t('nta.construction.cavity.unventilated')}</option>
              <option value="weakly">{t('nta.construction.cavity.weakly')}</option>
              <option value="strongly">{t('nta.construction.cavity.strongly')}</option>
            </select></td>
            <td style={cell} />
          </>}
          <td style={cell}><button type="button" className="btn btn-sm"
            onClick={() => setDraft({ ...draft, layers: draft.layers.filter((_, i) => i !== index) })}>x</button></td>
        </tr>)}</tbody>
      </table>
      <button type="button" className="btn btn-sm" onClick={() => setDraft({ ...draft, layers: [...draft.layers,
        { kind: 'material', name: '', thicknessM: 0.1, lambda: 0.04, sourceReference: '' }] })}>{t('dialog.construction.addLayer')}</button>
      <button type="button" className="btn btn-sm" onClick={() => setDraft(constructionDraft({ layers }, draft.heatFlow))}>
        {t('nta.construction.reload')}</button>
    </> : <div style={{ display: 'flex', gap: 12, flexWrap: 'wrap' }}>
      <label>{t('nta.construction.element')}
        <select value={forfait.element} onChange={(event) => setForfait({ ...forfait, element: event.target.value as NtaForfaitDraft['element'] })}>
          <option value="facade">{t('nta.construction.element.facade')}</option>
          <option value="roof">{t('nta.construction.element.roof')}</option>
          <option value="floor">{t('nta.construction.element.floor')}</option>
        </select>
      </label>
      <label>{t('nta.vent.constructionYear')}
        <input type="number" step="1" value={forfait.constructionYear}
          onChange={(event) => setForfait({ ...forfait, constructionYear: Number(event.target.value) })} />
      </label>
      <label>{t('nta.construction.insulation')}
        <select value={forfait.insulation} onChange={(event) => setForfait({ ...forfait, insulation: event.target.value as NtaForfaitDraft['insulation'] })}>
          <option value="absent_or_unknown">{t('nta.construction.insulation.absent')}</option>
          <option value="present_unknown_thickness">{t('nta.construction.insulation.unknownThickness')}</option>
          <option value="known_thickness">{t('nta.construction.insulation.known')}</option>
        </select>
      </label>
      {forfait.insulation === 'known_thickness' && <label>{t('nta.construction.insulationMm')}
        <input type="number" step="10" value={forfait.thicknessMm}
          onChange={(event) => setForfait({ ...forfait, thicknessMm: Number(event.target.value) })} />
      </label>}
      <label><input type="checkbox" checked={forfait.cavity}
        onChange={(event) => setForfait({ ...forfait, cavity: event.target.checked })} /> {t('nta.construction.cavityPresent')}</label>
    </div>}
    <div style={{ marginTop: 8, display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap' }}>
      <button type="button" className="btn btn-sm" disabled={busy} onClick={calculate}>{t('nta.construction.calculate')}</button>
      {element && <>
        <span role="status">{t('nta.construction.result')}: U = {element.uRounded.toFixed(3)} W/m²K
          {element.rCRounded != null && <> · R<sub>c</sub> = {element.rCRounded.toFixed(2)} m²K/W</>}</span>
        <button type="button" className="btn btn-sm" onClick={() => onApply({ rc: element.rCRounded, u: element.uRounded, route: element.route })}>
          {t('nta.construction.apply')}</button>
      </>}
    </div>
    {error && <p role="alert">{t('kernel.unavailable')} <small>{error}</small></p>}
    {result?.status === 'invalid' && <ul role="alert">
      {result.issues.map((item, index) => <li key={index}><strong>{item.code}</strong> <code>{item.path}</code></li>)}
    </ul>}
  </fieldset>;
}
