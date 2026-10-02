import { useState } from 'react';
import { ClipboardList } from 'lucide-react';
import { useEnergy } from '../../context/EnergyContext';
import { useI18n } from '../../i18n/i18n';
import {
  assessMaatwerkadviesWithRust,
  type MaatwerkadviesAssessment,
  type MwaMeasure,
  type MwaMeasureCategory,
  type MwaPackage,
  type MwaUserProfile,
  type MwaVariantResult,
  type NtaMaatwerkadvies,
} from '../../core/nta/KernelClient';
import { downloadMaatwerkadviesReportHTML } from '../../core/report/ReportGenerator';
import '../NtaPerformancePanel/NtaPerformancePanel.css';
import './MaatwerkadviesPanel.css';

const CATEGORIES: MwaMeasureCategory[] = [
  'insulation', 'glazing', 'airtightness', 'ventilation', 'heat_recovery', 'heating', 'heat_pump',
  'hot_water', 'cooling', 'pv', 'solar_thermal', 'lighting', 'control', 'other',
];
const PROFILES: MwaUserProfile[] = ['nta', 'energy_conscious', 'average', 'not_energy_conscious'];

export function emptyMaatwerkadvies(): NtaMaatwerkadvies {
  return {
    currentUse: { profile: 'nta', sourceReference: '' },
    measures: [],
    packages: [],
    tariffs: { gasEurPerM3: 1.4, electricityEurPerKwh: 0.3, electricityExportEurPerKwh: 0.05, sourceReference: '' },
    economics: { discountRate: 0.03, energyPriceChange: 0, sourceReference: '' },
  };
}

function format(value: number | null | undefined, digits = 0): string {
  return value == null || !Number.isFinite(value)
    ? '–'
    : value.toLocaleString('nl-NL', { minimumFractionDigits: digits, maximumFractionDigits: digits });
}

function nextId(prefix: string, taken: string[]): string {
  let index = taken.length + 1;
  while (taken.includes(`${prefix}${index}`)) index += 1;
  return `${prefix}${index}`;
}

function MeasureEditor({ measure, onChange, onRemove }: {
  measure: MwaMeasure;
  onChange: (measure: MwaMeasure) => void;
  onRemove: () => void;
}) {
  const { t } = useI18n();
  const [patchText, setPatchText] = useState(JSON.stringify(measure.patch, null, 2));
  const [patchError, setPatchError] = useState(false);
  const num = (value: string) => (value.trim() === '' ? undefined : Number(value));
  return (
    <fieldset className="mwa-measure">
      <legend>{measure.name || measure.id}</legend>
      <label>{t('mwa.measure.name')}
        <input value={measure.name} onChange={(e) => onChange({ ...measure, name: e.target.value })} />
      </label>
      <label>{t('mwa.measure.category')}
        <select value={measure.category} onChange={(e) => onChange({ ...measure, category: e.target.value as MwaMeasureCategory })}>
          {CATEGORIES.map((item) => <option key={item} value={item}>{t(`mwa.category.${item}`)}</option>)}
        </select>
      </label>
      <label>{t('mwa.measure.target')}
        <select value={measure.target} onChange={(e) => onChange({ ...measure, target: e.target.value as MwaMeasure['target'] })}>
          <option value="project">{t('mwa.target.project')}</option>
          <option value="building">{t('mwa.target.building')}</option>
        </select>
      </label>
      <label>{t('mwa.measure.investment')}
        <input type="number" value={measure.investmentEur} onChange={(e) => onChange({ ...measure, investmentEur: Number(e.target.value) })} />
      </label>
      <label>{t('mwa.measure.costSource')}
        <input value={measure.costSource} onChange={(e) => onChange({ ...measure, costSource: e.target.value })} />
      </label>
      <label>{t('mwa.measure.lifetime')}
        <input type="number" value={measure.lifetimeYears} onChange={(e) => onChange({ ...measure, lifetimeYears: Number(e.target.value) })} />
      </label>
      <label>{t('mwa.measure.maintenance')}
        <input type="number" value={measure.maintenanceEurPerYear ?? ''} onChange={(e) => onChange({ ...measure, maintenanceEurPerYear: num(e.target.value) })} />
      </label>
      <label>{t('mwa.measure.phaseYear')}
        <input type="number" value={measure.phaseYear ?? ''} onChange={(e) => onChange({ ...measure, phaseYear: num(e.target.value) })} />
      </label>
      <label className="mwa-wide">{t('mwa.measure.specialist')}
        <input value={measure.specialistNote ?? ''} onChange={(e) => onChange({ ...measure, specialistNote: e.target.value || undefined })} />
      </label>
      <label className="mwa-wide">{t('mwa.measure.patch')}
        <textarea rows={4} value={patchText} spellCheck={false}
          onChange={(e) => {
            setPatchText(e.target.value);
            try {
              const parsed = JSON.parse(e.target.value) as unknown;
              if (!Array.isArray(parsed)) throw new Error('array');
              setPatchError(false);
              onChange({ ...measure, patch: parsed as MwaMeasure['patch'] });
            } catch {
              setPatchError(true);
            }
          }} />
        {patchError && <span role="alert">{t('mwa.measure.patchInvalid')}</span>}
      </label>
      <button type="button" onClick={onRemove}>{t('mwa.remove')}</button>
    </fieldset>
  );
}

function ResultRow({ result }: { result: MwaVariantResult }) {
  const use = result.actualUse;
  return (
    <tr>
      <th scope="row">{result.name}{!result.valid && ' ⚠'}</th>
      <td>{result.label.labelClass ?? '–'}</td>
      <td>{format(result.label.primaryFossilIndicatorKwhPerM2, 1)}</td>
      <td>{format(use?.gasM3)}</td>
      <td>{format(use ? use.electricityImportKwh - use.electricityExportKwh : null)}</td>
      <td>{format(use?.co2Kg)}</td>
      <td>{format(use?.energyCostEur)}</td>
      <td>{format(result.savings?.energyCostEur)}</td>
      <td>{format(result.investmentEur)}</td>
      <td>{format(result.simplePaybackYears, 1)}</td>
      <td>{format(result.netPresentValueEur)}</td>
    </tr>
  );
}

/** Maatwerkadvies (BRL 9500-MWA, ISSO 82.2/75.2): measures, packages, results and advice. */
export function MaatwerkadviesPanel() {
  const { state, dispatch } = useEnergy();
  const { t } = useI18n();
  const project = state.project;
  const definition = project.maatwerkadvies;
  const [assessment, setAssessment] = useState<MaatwerkadviesAssessment | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const update = (next: NtaMaatwerkadvies) => {
    dispatch({ type: 'SET_MAATWERKADVIES', payload: next });
    setAssessment(null);
  };

  if (!definition) {
    return (
      <section className="nta-performance mwa-panel" aria-label={t('mwa.title')}>
        <div className="nta-performance-title"><ClipboardList size={18} /><div><h3>{t('mwa.title')}</h3><p>{t('mwa.intro')}</p></div></div>
        <div className="nta-performance-actions">
          <button type="button" onClick={() => update(emptyMaatwerkadvies())}>{t('mwa.start')}</button>
        </div>
      </section>
    );
  }

  const use = definition.currentUse ?? { profile: 'nta' as MwaUserProfile, sourceReference: '' };
  const setMeasure = (index: number, measure: MwaMeasure) =>
    update({ ...definition, measures: definition.measures.map((item, i) => (i === index ? measure : item)) });
  const removeMeasure = (index: number) => {
    const id = definition.measures[index].id;
    update({
      ...definition,
      measures: definition.measures.filter((_, i) => i !== index),
      packages: definition.packages.map((item) => ({ ...item, measureIds: item.measureIds.filter((m) => m !== id) })),
    });
  };
  const addMeasure = () => update({
    ...definition,
    measures: [...definition.measures, {
      id: nextId('m', definition.measures.map((item) => item.id)),
      name: '', category: 'insulation', target: 'project', patch: [],
      investmentEur: 0, costSource: '', lifetimeYears: 30,
    }],
  });
  const setPackage = (index: number, value: MwaPackage) =>
    update({ ...definition, packages: definition.packages.map((item, i) => (i === index ? value : item)) });
  const addPackage = () => update({
    ...definition,
    packages: [...definition.packages, { id: nextId('p', definition.packages.map((item) => item.id)), name: '', measureIds: [] }],
  });

  const calculate = async () => {
    setBusy(true);
    setError(null);
    try {
      setAssessment(await assessMaatwerkadviesWithRust(project, definition));
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  };

  const advice = assessment?.advice;
  const chosen = assessment?.packages.find((item) => item.id === advice?.packageId);

  return (
    <section className="nta-performance mwa-panel" aria-label={t('mwa.title')}>
      <div className="nta-performance-head">
        <div className="nta-performance-title"><ClipboardList size={18} /><div><h3>{t('mwa.title')}</h3><p>{t('mwa.intro')}</p></div></div>
        <span className="nta-performance-badge">{t('mwa.unverified')}</span>
      </div>

      <details open>
        <summary>{t('mwa.use')}</summary>
        <div className="mwa-grid">
          <label>{t('mwa.profile')}
            <select value={use.profile} onChange={(e) => update({ ...definition, currentUse: { ...use, profile: e.target.value as MwaUserProfile } })}>
              {PROFILES.map((item) => <option key={item} value={item}>{t(`mwa.profile.${item}`)}</option>)}
            </select>
          </label>
          <label>{t('mwa.occupants')}
            <input type="number" value={use.occupants ?? ''}
              onChange={(e) => update({ ...definition, currentUse: { ...use, occupants: e.target.value === '' ? undefined : Number(e.target.value) } })} />
          </label>
          <label>{t('mwa.heatingSetpoint')}
            <input type="number" value={use.heatingSetpointC ?? ''}
              onChange={(e) => update({ ...definition, currentUse: { ...use, heatingSetpointC: e.target.value === '' ? undefined : Number(e.target.value) } })} />
          </label>
          <label className="mwa-wide">{t('mwa.source')}
            <input value={use.sourceReference} onChange={(e) => update({ ...definition, currentUse: { ...use, sourceReference: e.target.value } })} />
          </label>
        </div>
      </details>

      <details open>
        <summary>{t('mwa.tariffs')}</summary>
        <div className="mwa-grid">
          <label>{t('mwa.tariff.gas')}
            <input type="number" step="0.01" value={definition.tariffs.gasEurPerM3}
              onChange={(e) => update({ ...definition, tariffs: { ...definition.tariffs, gasEurPerM3: Number(e.target.value) } })} />
          </label>
          <label>{t('mwa.tariff.electricity')}
            <input type="number" step="0.01" value={definition.tariffs.electricityEurPerKwh}
              onChange={(e) => update({ ...definition, tariffs: { ...definition.tariffs, electricityEurPerKwh: Number(e.target.value) } })} />
          </label>
          <label>{t('mwa.tariff.export')}
            <input type="number" step="0.01" value={definition.tariffs.electricityExportEurPerKwh ?? 0}
              onChange={(e) => update({ ...definition, tariffs: { ...definition.tariffs, electricityExportEurPerKwh: Number(e.target.value) } })} />
          </label>
          <label>{t('mwa.tariff.heat')}
            <input type="number" step="0.01" value={definition.tariffs.districtHeatEurPerKwh ?? 0}
              onChange={(e) => update({ ...definition, tariffs: { ...definition.tariffs, districtHeatEurPerKwh: Number(e.target.value) } })} />
          </label>
          <label>{t('mwa.discount')}
            <input type="number" step="0.005" value={definition.economics?.discountRate ?? 0.03}
              onChange={(e) => update({ ...definition, economics: { ...definition.economics, discountRate: Number(e.target.value) } })} />
          </label>
          <label className="mwa-wide">{t('mwa.source')}
            <input value={definition.tariffs.sourceReference}
              onChange={(e) => update({ ...definition, tariffs: { ...definition.tariffs, sourceReference: e.target.value } })} />
          </label>
        </div>
      </details>

      <details open>
        <summary>{t('mwa.measures')} ({definition.measures.length})</summary>
        <p>{t('mwa.patchHelp')}</p>
        {definition.measures.map((measure, index) => (
          <MeasureEditor key={measure.id} measure={measure}
            onChange={(value) => setMeasure(index, value)} onRemove={() => removeMeasure(index)} />
        ))}
        <button type="button" onClick={addMeasure}>{t('mwa.addMeasure')}</button>
      </details>

      <details open>
        <summary>{t('mwa.packages')} ({definition.packages.length})</summary>
        {definition.packages.map((item, index) => (
          <fieldset key={item.id} className="mwa-measure">
            <legend>{item.name || item.id}</legend>
            <label>{t('mwa.measure.name')}
              <input value={item.name} onChange={(e) => setPackage(index, { ...item, name: e.target.value })} />
            </label>
            <div className="mwa-wide mwa-checks">
              {definition.measures.map((measure) => (
                <label key={measure.id} className="mwa-check">
                  <input type="checkbox" checked={item.measureIds.includes(measure.id)}
                    onChange={(e) => setPackage(index, {
                      ...item,
                      measureIds: e.target.checked
                        ? [...item.measureIds, measure.id]
                        : item.measureIds.filter((id) => id !== measure.id),
                    })} />
                  {measure.name || measure.id}
                </label>
              ))}
            </div>
            <label className="mwa-wide">{t('mwa.partialWarning')}
              <input value={item.partialExecutionWarning ?? ''}
                onChange={(e) => setPackage(index, { ...item, partialExecutionWarning: e.target.value || undefined })} />
            </label>
            <button type="button" onClick={() => update({ ...definition, packages: definition.packages.filter((_, i) => i !== index) })}>{t('mwa.remove')}</button>
          </fieldset>
        ))}
        <button type="button" onClick={addPackage}>{t('mwa.addPackage')}</button>
        <label className="mwa-advised">{t('mwa.advised')}
          <select value={definition.advisedPackageId ?? ''}
            onChange={(e) => update({ ...definition, advisedPackageId: e.target.value || undefined })}>
            <option value="">{t('mwa.advised.automatic')}</option>
            {definition.packages.map((item) => <option key={item.id} value={item.id}>{item.name || item.id}</option>)}
          </select>
        </label>
      </details>

      <div className="nta-performance-actions mwa-actions">
        <button type="button" onClick={calculate} disabled={busy}>{busy ? t('mwa.calculating') : t('mwa.calculate')}</button>
        <button type="button" onClick={() => { void downloadMaatwerkadviesReportHTML(project).catch((reason: unknown) => setError(String(reason))); }}>
          {t('mwa.report')}
        </button>
      </div>
      {error && <p role="alert">{error}</p>}

      {assessment && (
        <div className="mwa-results">
          {assessment.issues.length > 0 && (
            <ul className="nta-performance-gaps">
              {assessment.issues.map((item, index) => <li key={index}><strong>{item.code}</strong><code>{item.path}</code>{item.detail}</li>)}
            </ul>
          )}
          {assessment.current && (
            <div className="nta-performance-table">
              <table>
                <thead><tr>
                  <th>{t('mwa.col.variant')}</th><th>{t('mwa.col.label')}</th><th>EP2</th><th>{t('mwa.col.gas')}</th>
                  <th>{t('mwa.col.electricity')}</th><th>CO₂ [kg]</th><th>{t('mwa.col.cost')}</th><th>{t('mwa.col.saving')}</th>
                  <th>{t('mwa.col.investment')}</th><th>{t('mwa.col.payback')}</th><th>{t('mwa.col.npv')}</th>
                </tr></thead>
                <tbody>
                  <ResultRow result={assessment.current} />
                  {assessment.measures.map((item) => <ResultRow key={`m-${item.id}`} result={item} />)}
                  {assessment.packages.map((item) => <ResultRow key={`p-${item.id}`} result={item} />)}
                </tbody>
              </table>
            </div>
          )}
          {advice && (
            <div className="nta-performance-bbl">
              <strong>{t('mwa.advice')}: {chosen?.name ?? '–'} ({advice.chosenBy === 'adviser' ? t('mwa.advised.adviser') : t('mwa.advised.automatic')})</strong>
              {advice.warnings.length > 0 && <><em>{t('mwa.warnings')}</em><ul>{advice.warnings.map((item) => <li key={item}>{item}</li>)}</ul></>}
              {advice.specialistNotes.length > 0 && <><em>{t('mwa.specialist')}</em><ul>{advice.specialistNotes.map((item) => <li key={item}>{item}</li>)}</ul></>}
            </div>
          )}
        </div>
      )}
    </section>
  );
}
