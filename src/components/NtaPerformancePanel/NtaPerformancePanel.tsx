import { useEffect, useRef, useState } from 'react';
import { AlertCircle, Calculator, CircleHelp } from 'lucide-react';
import { useEnergy } from '../../context/EnergyContext';
import type { IProject } from '../../core/energy/types';
import { useI18n } from '../../i18n/i18n';
import type { NtaCalculationInput } from '../../core/nta/KernelClient';
import { useProjectPerformance, type ProjectPerformanceQuery } from '../../core/nta/useProjectPerformance';
import { formatNumber } from '../../i18n/format';
import { KernelCode, KernelDetail } from '../KernelCode/KernelCode';
import { buildNtaCalculationTemplate } from '../../core/nta/NtaCalculationTemplate';
import { NtaCalculationForm } from './NtaCalculationForm';
import { summarizeExtras } from '../../core/nta/NtaResultSummary';
import { summarizeServiceEnergy } from '../../core/nta/ServiceEnergy';
import { kernelNote } from '../../core/nta/KernelNoteText';
import './NtaPerformancePanel.css';

/** Short month name in the UI language. */
export function shortMonth(month: number, locale: string): string {
  return new Date(2021, month - 1, 1).toLocaleString(locale, { month: 'short' });
}

interface NtaPerformancePanelProps {
  /** Kernel query run by the parent; when absent the panel runs its own. */
  query?: ProjectPerformanceQuery | null;
}

export function NtaPerformancePanel({ query: suppliedQuery }: NtaPerformancePanelProps = {}) {
  const { state, dispatch } = useEnergy();
  const { t, locale } = useI18n();
  const project: IProject = state.project;
  const ownQuery = useProjectPerformance(project, suppliedQuery === undefined);
  const kwh = (value: number | null | undefined) => formatNumber(value, locale);
  const [editing, setEditing] = useState(false);
  const [formOpen, setFormOpen] = useState(false);
  const [draft, setDraft] = useState('');
  const [parseError, setParseError] = useState(false);
  const editorProjectId = useRef(project.id);

  useEffect(() => {
    if (editorProjectId.current !== project.id) {
      editorProjectId.current = project.id;
      setEditing(false);
      setFormOpen(false);
      setDraft('');
      setParseError(false);
    }
  }, [project.id]);

  const query = suppliedQuery === undefined ? ownQuery : suppliedQuery;
  const currentQuery = query?.project === project ? query : null;
  const loading = currentQuery == null || currentQuery.kind === 'loading';
  const assessment = currentQuery?.kind === 'done' ? currentQuery.assessment : null;
  const error = currentQuery?.kind === 'error' ? currentQuery.error : null;

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


  const performance = assessment?.performance ?? null;
  const calculated = assessment?.status === 'calculated_unverified' && performance;
  const heating = performance?.spaceHeating;
  const issues = performance?.issues ?? [];
  const extras = calculated && performance ? summarizeExtras(performance) : null;
  const services = calculated && performance ? summarizeServiceEnergy(performance.energyByService) : null;

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

      {!loading && assessment?.geometry && (
        <p className="nta-performance-geometry">
          A<sub>g</sub> = {formatNumber(assessment.geometry.usableFloorAreaM2, locale, 1)} m² · A<sub>ls</sub> = {formatNumber(assessment.geometry.lossAreaM2, locale, 1)} m²
          {assessment.geometry.lossAreaRatio != null && <> · A<sub>ls</sub>/A<sub>g</sub> = {formatNumber(assessment.geometry.lossAreaRatio, locale, 3)}</>}
          {assessment.geometry.unclassifiedSurfaceCount > 0 && <> · {assessment.geometry.unclassifiedSurfaceCount} {t('nta.performance.unclassified')}</>}
        </p>
      )}
      {!loading && assessment && assessment.status === 'incomplete' && (
        <div className="nta-performance-gaps">
          <p role="status"><AlertCircle size={16} /> {t('nta.performance.incomplete')}</p>
          <ul>{assessment.gaps.map((gap, index) => (
            <li key={`${gap.code}-${index}`}>
              <KernelCode code={gap.code} />
              <code>{gap.path}</code>
              <KernelDetail detail={gap.detail} />
            </li>
          ))}</ul>
        </div>
      )}

      {!loading && assessment && (assessment.warnings?.length ?? 0) > 0 && (
        <div className="nta-performance-gaps" data-testid="nta-plausibility-warnings">
          <p><AlertCircle size={16} /> {t('nta.performance.warnings')}</p>
          <ul>{assessment.warnings!.map((warning, index) => (
            <li key={`${warning.code}-${index}`}>
              <KernelCode code={warning.code} prefixes={['nta.warning.', 'nta.gap.', 'kernel.issue.']} />
              <code>{warning.path}</code>
              <KernelDetail detail={warning.detail} />
            </li>
          ))}</ul>
        </div>
      )}

      {!loading && assessment && assessment.status === 'invalid' && (
        <div className="nta-performance-gaps">
          <p role="status"><AlertCircle size={16} /> {t('nta.performance.invalid')}</p>
          <ul data-testid="nta-invalid-reasons">{assessment.gaps.map((gap, index) => (
            <li key={`gap-${gap.code}-${index}`}>
              <KernelCode code={gap.code} />
              <code>{gap.path}</code>
              {gap.detail && <small>{gap.detail}</small>}
            </li>
          ))}{issues.map((item, index) => (
            <li key={`${item.code}-${index}`}><KernelCode code={item.code} /><code>{item.path}</code></li>
          ))}</ul>
        </div>
      )}

      {!loading && calculated && (performance?.warnings?.length ?? 0) > 0 && (
        <div className="nta-performance-gaps" data-testid="nta-performance-warnings">
          <p role="status"><AlertCircle size={16} /> {t('nta.performance.buildingWarnings')}</p>
          <ul>{(performance?.warnings ?? []).map((item, index) => (
            <li key={`${item.code}-${index}`}><KernelCode code={item.code} /><code>{item.path}</code></li>
          ))}</ul>
        </div>
      )}

      {!loading && calculated && performance && heating && <>
        <div className="nta-performance-indicators" role="group" aria-label={t('nta.performance.indicators')}>
          <div>
            <span>BENG 1</span>
            <strong>{formatNumber(performance.needIndicatorKwhPerM2Year, locale, 2)}</strong>
            <small>{performance.needIndicatorKwhPerM2Year == null ? t('nta.performance.beng1RequiresC1') : t('unit.kwhPerM2Year')}</small>
          </div>
          <div>
            <span>BENG 2</span>
            <strong>{formatNumber(performance.primaryFossilIndicatorKwhPerM2Year, locale, 2)}</strong>
            <small>{t('unit.kwhPerM2Year')}</small>
          </div>
          <div>
            <span>{t('nta.performance.labelClass')}</span>
            <strong>{performance.indicativeLabelClass ?? '–'}</strong>
            <small>{t('nta.performance.labelIndicative')}</small>
            {performance.labelPrimaryFossilIndicatorKwhPerM2Year != null
              && performance.labelPrimaryFossilIndicatorKwhPerM2Year !== performance.primaryFossilIndicatorKwhPerM2Year
              && <small data-testid="nta-label-ep2">{t('nta.performance.labelEp2', { value: formatNumber(performance.labelPrimaryFossilIndicatorKwhPerM2Year, locale, 2) })}</small>}
          </div>
          <div>
            <span>BENG 3</span>
            <strong>{formatNumber(performance.renewableSharePercent, locale, 1)}</strong>
            <small>%</small>
          </div>
        </div>
        <dl className="nta-performance-totals">
          <div><dt>{t('nta.performance.heatingNeed')}</dt><dd>{kwh(heating.monthly.reduce((sum, row) => sum + row.heatingNeedKwh, 0))} kWh</dd></div>
          <div><dt>{t('nta.performance.coolingNeed')}</dt><dd>{kwh([heating.demand, ...(heating.additionalZoneDemands ?? [])].reduce((sum, zone) => sum + (zone.annualCoolingNeedKwh ?? 0), 0))} kWh</dd></div>
          <div><dt>{t('nta.performance.primaryFossil')}</dt><dd>{kwh(performance.annualPrimaryFossilKwh)} kWh</dd></div>
          <div><dt>{t('nta.performance.renewable')}</dt><dd>{kwh(performance.annualRenewablePrimaryKwh)} kWh</dd></div>
        </dl>
        {(performance.annualZebPrimaryTotalKwh != null || performance.zebPrimaryTotalIndicatorKwhPerM2 != null
          || performance.annualZebCo2Kg != null) && <section className="nta-performance-bbl" aria-label="ZEB (Annex AB)">
          <strong>ZEB · {locale === 'nl' ? 'bijlage AB' : 'Annex AB'}</strong>
          <small>{locale === 'nl'
            ? 'Informatieve uitkomsten, niet geattesteerd en geen geregistreerd energielabel.'
            : 'Informative results, unattested and not a registered energy label.'}</small>
          <dl className="nta-performance-totals">
            <div><dt>E<sub>P,ZEB;Tot;an</sub></dt><dd>{formatNumber(performance.annualZebPrimaryTotalKwh, locale, 2)} kWh</dd></div>
            <div><dt>E<sub>weP,ZEB;Tot</sub></dt><dd>{formatNumber(performance.zebPrimaryTotalIndicatorKwhPerM2, locale, 2)} kWh/m²</dd></div>
            <div><dt>CO₂eq {locale === 'nl' ? 'operationeel' : 'operational'}</dt><dd>{formatNumber(performance.annualZebCo2Kg, locale, 2)} {t('unit.kgPerYear')}</dd></div>
          </dl>
        </section>}
        {(performance.annualFinalEnergyKwh != null || (performance.finalEnergyByCarrier?.length ?? 0) > 0) &&
          <details className="nta-performance-monthly" aria-label={locale === 'nl' ? 'Finaal energiegebruik' : 'Final energy use'}>
            <summary>{locale === 'nl' ? 'Finaal energiegebruik (§5.9)' : 'Final energy use (§5.9)'}</summary>
            <p>{locale === 'nl'
              ? 'Berekening zonder aftrek van eigen opwekking; uitkomsten zijn niet geattesteerd.'
              : 'Calculation without netting on-site generation; results are unattested.'}</p>
            <dl className="nta-performance-totals">
              <div><dt>E<sub>Final</sub></dt><dd>{formatNumber(performance.annualFinalEnergyKwh, locale, 2)} {t('unit.kwhPerYear')}</dd></div>
              <div><dt>E<sub>Final;EED</sub></dt><dd>{formatNumber(performance.annualFinalEnergyEedKwh, locale, 2)} {t('unit.kwhPerYear')}</dd></div>
            </dl>
            {(performance.finalEnergyByCarrier?.length ?? 0) > 0 && <div className="nta-performance-table"><table>
              <thead><tr><th scope="col">{locale === 'nl' ? 'Energiedrager' : 'Energy carrier'}</th>
                <th scope="col">{locale === 'nl' ? 'Finaal (kWh/jaar)' : 'Final (kWh/year)'}</th></tr></thead>
              <tbody>{performance.finalEnergyByCarrier.map((item) => <tr key={item.carrier}>
                <th scope="row">{item.carrier}</th><td>{formatNumber(item.annualKwh, locale, 2)}</td>
              </tr>)}</tbody>
            </table></div>}
          </details>}
        {services && <details className="nta-performance-monthly" data-testid="nta-energy-by-service">
          <summary>{t('nta.performance.byService')}</summary>
          <div className="nta-performance-table"><table>
            <thead><tr><th scope="col">{t('nta.performance.byService.function')}</th>
              {services.carriers.map((carrier) => <th scope="col" key={carrier}>{carrier}</th>)}
              <th scope="col">{t('nta.performance.byService.primaryFossil')}</th>
              <th scope="col">{t('nta.performance.byService.renewable')}</th></tr></thead>
            <tbody>
              {services.rows.map((row) => <tr key={row.service}>
                <th scope="row">{t(`nta.performance.service.${row.service}`)}</th>
                {services.carriers.map((carrier) => <td key={carrier}>{kwh(row.usedKwh[carrier] ?? 0)}</td>)}
                <td>{kwh(row.primaryFossilKwh)}</td><td>{kwh(row.renewablePrimaryKwh)}</td>
              </tr>)}
              <tr><th scope="row">{t('nta.performance.byService.export')}</th>
                {services.carriers.map((carrier) => <td key={carrier} />)}
                <td>−{kwh(services.exportedElectricityCreditKwh + services.storageCorrectionKwh)}</td>
                <td>{kwh(services.renewableElectricityKwh)}</td></tr>
              <tr><th scope="row">{t('nta.performance.byService.total')}</th>
                {services.carriers.map((carrier) => <td key={carrier} />)}
                <td>{kwh(services.primaryFossilTotalKwh)}</td><td>{kwh(services.renewableTotalKwh)}</td></tr>
            </tbody>
          </table></div>
          <p>{t('nta.performance.byService.note')}</p>
        </details>}
        {performance.tojuli.length > 0 && <div className="nta-performance-bbl" role="group" aria-label="TOjuli">
          <strong>TO<sub>juli</sub> (§5.7)</strong>
          <small>{t('nta.performance.tojuliScope')}</small>
          <ul>
            {performance.tojuli.flatMap((zone) => zone.activeCooling
              ? [<li key={zone.zoneId}><span>{zone.zoneId}: {t('nta.performance.tojuliCooled')}</span><em>{formatNumber(0, locale, 2)} K</em></li>]
              : zone.status !== 'calculated_unverified'
                ? [<li key={zone.zoneId}><span>{zone.zoneId}</span><em>{zone.issues[0] ? <KernelCode code={zone.issues[0].code} /> : null}</em></li>]
                : zone.orientations.filter((item) => item.assessed).map((item) => (
                  <li key={`${zone.zoneId}-${item.orientation}`}>
                    <span>{performance.tojuli.length > 1 ? `${zone.zoneId} · ` : ''}{t(`nta.orientation.${item.orientation}`)}</span>
                    <em>{formatNumber(item.tojuliK, locale, 2)} K</em>
                  </li>
                )))}
            <li>
              <span>{t('nta.performance.tojuliMax')} ≤ {formatNumber(1.2, locale, 2)}</span>
              <em>{performance.tojuliMeetsBblLimit == null
                ? t(assessment?.derivedInput?.calculationScope === 'utility' ? 'nta.performance.bblNotApplicable' : 'nta.performance.bblUnknown')
                : performance.tojuliMeetsBblLimit ? t('nta.performance.bblMeets') : t('nta.performance.bblFails')}</em>
            </li>
          </ul>
        </div>}
        {performance.a0Check && <div className="nta-performance-bbl" role="group" aria-label="A0">
          <strong>{t('nta.performance.a0')}</strong>
          <ul>
            <li><span>BENG 2 ≤ {formatNumber(performance.a0Check.primaryFossilMaxKwhPerM2, locale, 0)} (IXa/Xa)</span>
              <em>{performance.a0Check.primaryFossilMeets == null ? t('nta.performance.bblUnknown') : performance.a0Check.primaryFossilMeets ? t('nta.performance.bblMeets') : t('nta.performance.bblFails')}</em></li>
            <li><span>{t('nta.performance.a0Fossil')}</span>
              <em>{performance.a0Check.noOnSiteFossilCombustion ? t('nta.performance.bblMeets') : t('nta.performance.bblFails')}</em></li>
            <li><span>A0</span>
              <em>{performance.a0Check.eligible == null ? t('nta.performance.bblUnknown') : performance.a0Check.eligible ? t('nta.performance.a0Yes') : t('nta.performance.a0No')}</em></li>
          </ul>
        </div>}
        {performance.bblCheck && <div className="nta-performance-bbl" role="group" aria-label={t('nta.performance.bbl')}>
          <strong>{t('nta.performance.bbl')}</strong>
          <small>A<sub>ls</sub>/A<sub>g</sub> = {formatNumber(performance.bblCheck.lossAreaRatio, locale, 2)}</small>
          <ul>
            {([
              ['BENG 1', performance.bblCheck.energyNeedMeets, `≤ ${formatNumber(performance.bblCheck.limits.energyNeedMaxKwhPerM2, locale, 1)}`],
              ['BENG 2', performance.bblCheck.primaryFossilMeets, `≤ ${formatNumber(performance.bblCheck.limits.primaryFossilMaxKwhPerM2, locale, 1)}`],
              ['BENG 3', performance.bblCheck.renewableShareMeets, `≥ ${formatNumber(performance.bblCheck.limits.renewableShareMinPercent, locale, 0)}%`],
            ] as const).map(([name, meets, limit]) => (
              <li key={name}>
                <span>{name} {limit}</span>
                <em>{meets == null ? t('nta.performance.bblUnknown') : meets ? t('nta.performance.bblMeets') : t('nta.performance.bblFails')}</em>
              </li>
            ))}
          </ul>
        </div>}
        {extras && <details className="nta-performance-monthly" open={extras.ventilation.length > 0}>
          <summary>{t('nta.performance.details')}</summary>
          <dl className="nta-performance-totals">
            {extras.beng1Basis && <div className="nta-performance-wide"><dt>{t('nta.performance.beng1Basis')}</dt>
              <dd>{extras.beng1Basis === 'fixed_c1'
                ? `${t('nta.performance.beng1FixedC1')} · ${kwh(extras.fixedC1NeedKwh)} kWh`
                : t('nta.performance.beng1Confirmed')}</dd></div>}
            {extras.ventilation.map((zone) => <div key={zone.zoneId} className="nta-performance-wide">
              <dt>{t('nta.performance.ventilation')}{extras.ventilation.length > 1 ? ` · ${zone.zoneId}` : ''}</dt>
              <dd>{t('nta.performance.ventilationFlow')}: {kwh(zone.requiredJanuaryM3PerH)} m³/h · {t('nta.performance.infiltration')}: {kwh(zone.infiltrationJanuaryM3PerH)} m³/h
                · {t('nta.performance.ventilationConductance')}: {formatNumber(zone.conductanceJanuaryWPerK, locale, 1)} W/K
                · {t('nta.performance.fans')}: {kwh(zone.fanKwh)} kWh
                {zone.frostProtectionKwh > 0 && <> · {t('nta.performance.frost')}: {kwh(zone.frostProtectionKwh)} kWh</>}</dd>
            </div>)}
            {extras.recoverableLossesApplied && <div className="nta-performance-wide"><dt>{t('nta.performance.recoverable')}</dt>
              <dd>{t('nta.performance.recoverableKwh')}: {kwh(extras.recoverableLossKwh)} kWh</dd></div>}
            {extras.lightingKwh != null && <div><dt>{t('nta.performance.lighting')}</dt><dd>{kwh(extras.lightingKwh)} kWh</dd></div>}
            {extras.co2Kg != null && <div><dt>{t('nta.performance.co2')}</dt>
              <dd>{kwh(extras.co2Kg)} kg · {formatNumber(extras.co2KgPerM2, locale, 1)} kg/m²</dd></div>}
            {extras.storageCorrectionKwh != null && extras.storageCorrectionKwh !== 0 && <div><dt>{t('nta.performance.storage')}</dt>
              <dd>{kwh(extras.storageCorrectionKwh)} kWh</dd></div>}
          </dl>
        </details>}
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
                <th scope="row">{shortMonth(row.month, locale)}</th>
                <td>{kwh(row.heatingNeedKwh)}</td>
                <td>{kwh([heating.demand, ...(heating.additionalZoneDemands ?? [])].reduce((sum, zone) => sum + (zone.monthly[index]?.cooling.needKwh ?? 0), 0))}</td>
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
            {[...heating.demand.omittedCorrections, ...heating.omittedTerms].map((item) => <li key={item}>{kernelNote(item, locale)}</li>)}
          </ul>
        </details>
      </>}

      {!loading && assessment && <details className="nta-performance-limits" aria-label={t('kernel.provenance')}>
        <summary>{t('kernel.provenance')}</summary>
        <dl className="nta-performance-totals">
          <div><dt>{t('kernel.targetNorm')}</dt><dd>{assessment.targetNormVersion ?? '–'}</dd></div>
          <div><dt>{t('kernel.version')}</dt><dd>{assessment.kernelVersion ?? '–'}</dd></div>
          <div><dt>{t('kernel.inputFingerprint')}</dt><dd><code className="nta-performance-fingerprint" title={assessment.inputFingerprint}>{assessment.inputFingerprint}</code></dd></div>
          <div><dt>{locale === 'nl' ? 'Atteststatus' : 'Attestation status'}</dt>
            <dd>{assessment.attestStatus === 'unattested'
              ? (locale === 'nl' ? 'Niet geattesteerd' : 'Unattested') : '–'}</dd></div>
        </dl>
      </details>}

      <div className="nta-performance-actions">
        {!editing && !formOpen && <>
          <button type="button" className="btn btn-primary" onClick={() => { setFormOpen(true); }}>
            {project.ntaCalculation ? t('nta.performance.edit') : t('nta.performance.start')}
          </button>
          <button type="button" className="btn" onClick={openEditor}>{t('nta.performance.advanced')}</button>
        </>}
      </div>
      {formOpen && editorProjectId.current === project.id && <NtaCalculationForm project={project}
        initial={(project.ntaCalculation as unknown as Record<string, unknown> | undefined) ?? buildNtaCalculationTemplate(project)}
        onCancel={() => setFormOpen(false)}
        onSave={(block) => {
          dispatch({ type: 'SET_NTA_CALCULATION', payload: block as unknown as NtaCalculationInput });
          setFormOpen(false);
        }} />}
      {editing && editorProjectId.current === project.id && <div className="nta-performance-editor">
        <label>{t('nta.performance.blockLabel')}
          <textarea value={draft} spellCheck={false} rows={18} onChange={(event) => setDraft(event.target.value)} />
        </label>
        <p>{t('nta.performance.blockHelp')}</p>
        {parseError && <p role="alert">{t('nta.performance.parseError')}</p>}
        <div>
          <button type="button" className="btn" onClick={() => setEditing(false)}>{t('dialog.cancel')}</button>
          <button type="button" className="btn btn-primary" onClick={save}>{t('dialog.save')}</button>
        </div>
      </div>}
    </section>
  );
}
