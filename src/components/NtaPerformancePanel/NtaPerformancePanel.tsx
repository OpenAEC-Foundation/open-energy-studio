import { useEffect, useState } from 'react';
import { AlertCircle, Calculator, CircleHelp } from 'lucide-react';
import { useEnergy } from '../../context/EnergyContext';
import { useI18n } from '../../i18n/i18n';
import {
  calculateProjectPerformanceWithRust,
  type NtaCalculationInput,
  type ProjectPerformanceAssessment,
} from '../../core/nta/KernelClient';
import { buildNtaCalculationTemplate } from '../../core/nta/NtaCalculationTemplate';
import './NtaPerformancePanel.css';

const MONTHS = ['jan', 'feb', 'mrt', 'apr', 'mei', 'jun', 'jul', 'aug', 'sep', 'okt', 'nov', 'dec'];

function kwh(value: number | null | undefined): string {
  return value == null ? '–' : Math.round(value).toLocaleString('nl-NL');
}

export function NtaPerformancePanel() {
  const { state, dispatch } = useEnergy();
  const { t } = useI18n();
  const project = state.project;
  const [assessment, setAssessment] = useState<ProjectPerformanceAssessment | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState('');
  const [parseError, setParseError] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    setAssessment(null);
    calculateProjectPerformanceWithRust(project).then(
      (value) => { if (!cancelled) { setAssessment(value); setLoading(false); } },
      (reason: unknown) => {
        if (!cancelled) {
          setError(reason instanceof Error ? reason.message : String(reason));
          setLoading(false);
        }
      },
    );
    return () => { cancelled = true; };
  }, [project]);

  const openEditor = () => {
    const block = project.ntaCalculation ?? buildNtaCalculationTemplate(project);
    setDraft(JSON.stringify(block, null, 2));
    setParseError(false);
    setEditing(true);
  };

  const save = () => {
    let parsed: unknown;
    try {
      parsed = JSON.parse(draft);
    } catch {
      setParseError(true);
      return;
    }
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
      setParseError(true);
      return;
    }
    // Shape and content are validated by the Rust kernel on the next run.
    dispatch({ type: 'SET_NTA_CALCULATION', payload: parsed as NtaCalculationInput });
    setEditing(false);
  };

  const gapLabel = (code: string) => {
    const key = `nta.gap.${code}`;
    const text = t(key);
    return text === key ? code : text;
  };

  const performance = assessment?.performance ?? null;
  const calculated = assessment?.status === 'calculated_unverified' && performance;
  const heating = performance?.spaceHeating;
  const issues = performance?.issues ?? [];

  return (
    <section className="nta-performance" aria-label={t('nta.performance.title')}>
      <div className="nta-performance-head">
        <div className="nta-performance-title"><Calculator size={19} /><div>
          <h3>{t('nta.performance.title')}</h3>
          <p>{t('nta.performance.scope')}</p>
        </div></div>
        <span className="nta-performance-badge">{t('nta.performance.unverified')}</span>
      </div>

      {loading && <p role="status">{t('kernel.loading')}</p>}
      {!loading && error && (
        <div className="nta-performance-unavailable" role="alert">
          <CircleHelp size={18} />
          <span>{t('kernel.unavailable')} <small>{error}</small></span>
        </div>
      )}

      {!loading && assessment && assessment.status === 'incomplete' && (
        <div className="nta-performance-gaps">
          <p role="status"><AlertCircle size={16} /> {t('nta.performance.incomplete')}</p>
          <ul>{assessment.gaps.map((gap, index) => (
            <li key={`${gap.code}-${index}`}>
              <strong>{gapLabel(gap.code)}</strong>
              <code>{gap.path}</code>
              {gap.detail && <small>{gap.detail}</small>}
            </li>
          ))}</ul>
        </div>
      )}

      {!loading && assessment && assessment.status === 'invalid' && (
        <div className="nta-performance-gaps">
          <p role="status"><AlertCircle size={16} /> {t('nta.performance.invalid')}</p>
          <ul>{issues.map((item, index) => (
            <li key={`${item.code}-${index}`}><strong>{item.code}</strong><code>{item.path}</code></li>
          ))}</ul>
        </div>
      )}

      {!loading && calculated && performance && heating && <>
        <div className="nta-performance-indicators" role="group" aria-label={t('nta.performance.indicators')}>
          <div>
            <span>BENG 1</span>
            <strong>{performance.needIndicatorKwhPerM2Year?.toFixed(2) ?? '–'}</strong>
            <small>{performance.needIndicatorKwhPerM2Year == null ? t('nta.performance.beng1RequiresC1') : 'kWh/m²·jr'}</small>
          </div>
          <div>
            <span>BENG 2</span>
            <strong>{performance.primaryFossilIndicatorKwhPerM2Year?.toFixed(2) ?? '–'}</strong>
            <small>kWh/m²·jr</small>
          </div>
          <div>
            <span>{t('nta.performance.labelClass')}</span>
            <strong>{performance.indicativeLabelClass ?? '–'}</strong>
            <small>{t('nta.performance.labelIndicative')}</small>
          </div>
          <div>
            <span>BENG 3</span>
            <strong>{performance.renewableSharePercent?.toFixed(1) ?? '–'}</strong>
            <small>%</small>
          </div>
        </div>
        <dl className="nta-performance-totals">
          <div><dt>{t('nta.performance.heatingNeed')}</dt><dd>{kwh(heating.demand.annualHeatingNeedKwh)} kWh</dd></div>
          <div><dt>{t('nta.performance.coolingNeed')}</dt><dd>{kwh(heating.demand.annualCoolingNeedKwh)} kWh</dd></div>
          <div><dt>{t('nta.performance.primaryFossil')}</dt><dd>{kwh(performance.annualPrimaryFossilKwh)} kWh</dd></div>
          <div><dt>{t('nta.performance.renewable')}</dt><dd>{kwh(performance.annualRenewablePrimaryKwh)} kWh</dd></div>
        </dl>
        {performance.bblCheck && <div className="nta-performance-bbl" role="group" aria-label={t('nta.performance.bbl')}>
          <strong>{t('nta.performance.bbl')}</strong>
          <small>A<sub>ls</sub>/A<sub>g</sub> = {performance.bblCheck.lossAreaRatio.toFixed(2)}</small>
          <ul>
            {([
              ['BENG 1', performance.bblCheck.energyNeedMeets, `≤ ${performance.bblCheck.limits.energyNeedMaxKwhPerM2.toFixed(1)}`],
              ['BENG 2', performance.bblCheck.primaryFossilMeets, `≤ ${performance.bblCheck.limits.primaryFossilMaxKwhPerM2.toFixed(1)}`],
              ['BENG 3', performance.bblCheck.renewableShareMeets, `≥ ${performance.bblCheck.limits.renewableShareMinPercent.toFixed(0)}%`],
            ] as const).map(([name, meets, limit]) => (
              <li key={name}>
                <span>{name} {limit}</span>
                <em>{meets == null ? t('nta.performance.bblUnknown') : meets ? t('nta.performance.bblMeets') : t('nta.performance.bblFails')}</em>
              </li>
            ))}
          </ul>
        </div>}
        <details className="nta-performance-monthly">
          <summary>{t('nta.performance.monthly')}</summary>
          <div className="nta-performance-table"><table>
            <thead><tr>
              <th scope="col">{t('nta.performance.month')}</th>
              <th scope="col">Q<sub>H;nd</sub></th>
              <th scope="col">Q<sub>C;nd</sub></th>
              <th scope="col">{t('nta.performance.emissionLoss')}</th>
              <th scope="col">{t('nta.performance.gas')}</th>
              <th scope="col">{t('nta.performance.electricity')}</th>
            </tr></thead>
            <tbody>{heating.monthly.map((row, index) => (
              <tr key={row.month}>
                <th scope="row">{MONTHS[row.month - 1]}</th>
                <td>{kwh(row.heatingNeedKwh)}</td>
                <td>{kwh(heating.demand.monthly[index]?.cooling.needKwh)}</td>
                <td>{kwh(row.emissionLossKwh)}</td>
                <td>{kwh(row.naturalGasKwh)}</td>
                <td>{kwh(row.generatorElectricityKwh + (row.auxiliaryElectricityKwh ?? 0))}</td>
              </tr>
            ))}</tbody>
          </table></div>
        </details>
        <details className="nta-performance-limits">
          <summary>{t('nta.performance.limits')}</summary>
          <ul>
            {[...heating.demand.omittedCorrections, ...heating.omittedTerms].map((item) => <li key={item}>{item}</li>)}
          </ul>
          <p>{t('nta.performance.fingerprint')}: <code>{assessment.inputFingerprint}</code></p>
        </details>
      </>}

      <div className="nta-performance-actions">
        {!editing && <button type="button" onClick={openEditor}>
          {project.ntaCalculation ? t('nta.performance.edit') : t('nta.performance.start')}
        </button>}
      </div>
      {editing && <div className="nta-performance-editor">
        <label>{t('nta.performance.blockLabel')}
          <textarea value={draft} spellCheck={false} rows={18} onChange={(event) => setDraft(event.target.value)} />
        </label>
        <p>{t('nta.performance.blockHelp')}</p>
        {parseError && <p role="alert">{t('nta.performance.parseError')}</p>}
        <div>
          <button type="button" onClick={() => setEditing(false)}>{t('dialog.cancel')}</button>
          <button type="button" onClick={save}>{t('dialog.save')}</button>
        </div>
      </div>}
    </section>
  );
}
