import { useI18n } from '../../i18n/i18n';
import { formatNumber } from '../../i18n/format';
import type { IEnergyBreakdown } from '../../core/energy/types';
import './EnergyBreakdownChart.css';

interface EnergyBreakdownChartProps {
  breakdown: IEnergyBreakdown;
  /** The balance comes from the NTA kernel; infiltration is then part of the ventilation term. */
  source?: 'kernel' | 'indicative';
}

export function EnergyBreakdownChart({ breakdown, source = 'indicative' }: EnergyBreakdownChartProps) {
  const { t, locale } = useI18n();

  const losses = [
    { key: 'transmissionLoss', label: t('results.transmissionLoss'), value: breakdown.transmissionLoss, color: 'var(--viz-heating)' },
    { key: 'ventilationLoss', label: source === 'kernel' ? t('results.ventilationInfiltrationLoss') : t('results.ventilationLoss'),
      value: breakdown.ventilationLoss, color: 'var(--viz-fans)' },
    ...(source === 'kernel' ? [] : [
      { key: 'infiltrationLoss', label: t('results.infiltrationLoss'), value: breakdown.infiltrationLoss, color: 'var(--viz-humid)' },
    ]),
  ];

  const gains = [
    { key: 'solarGain', label: t('results.solarGain'), value: breakdown.solarGain, color: 'var(--viz-pv)' },
    { key: 'internalGain', label: t('results.internalGain'), value: breakdown.internalGain, color: 'var(--viz-aux)' },
    ...(breakdown.otherGain ? [
      { key: 'otherGain', label: t('results.otherGain'), value: breakdown.otherGain, color: 'var(--fg-disabled)' },
    ] : []),
  ];

  const delivered = [
    { key: 'heatingEnergy', label: t('results.heatingEnergy'), value: breakdown.heatingEnergy, color: 'var(--viz-heating)' },
    { key: 'coolingEnergy', label: t('results.coolingEnergy'), value: breakdown.coolingEnergy, color: 'var(--viz-cooling)' },
    { key: 'ventilationEnergy', label: t('results.ventilationEnergy'), value: breakdown.ventilationEnergy, color: 'var(--viz-fans)' },
    { key: 'hotWaterEnergy', label: t('results.hotWaterEnergy'), value: breakdown.hotWaterEnergy, color: 'var(--viz-dhw)' },
    { key: 'lightingEnergy', label: t('results.lightingEnergy'), value: breakdown.lightingEnergy, color: 'var(--viz-lighting)' },
    { key: 'auxiliaryEnergy', label: t('results.auxiliaryEnergy'), value: breakdown.auxiliaryEnergy, color: 'var(--viz-aux)' },
  ];

  const production = [
    { key: 'pvProduction', label: t('results.pvProduction'), value: breakdown.pvProduction, color: 'var(--viz-pv)' },
    { key: 'solarThermal', label: t('results.solarThermalProduction'), value: breakdown.solarThermalProduction, color: 'var(--viz-dhw)' },
  ];

  const allValues = [...losses, ...gains, ...delivered, ...production].map(d => d.value);
  const maxVal = Math.max(...allValues, 1);

  const renderBar = (item: { key: string; label: string; value: number; color: string }) => (
    <div key={item.key} className="breakdown-bar-row">
      <span className="breakdown-bar-label">{item.label}</span>
      <div className="breakdown-bar-track">
        <div
          className="breakdown-bar-fill"
          style={{
            width: `${(item.value / maxVal) * 100}%`,
            backgroundColor: item.color,
          }}
        />
      </div>
      <span className="breakdown-bar-value">{formatNumber(item.value, locale)} kWh</span>
    </div>
  );

  return (
    <div className="energy-breakdown-chart">
      <h3>{t('results.breakdown')}</h3>
      <p className="breakdown-source" data-testid="breakdown-source">
        {source === 'kernel' ? t('results.breakdownSourceKernel') : t('results.breakdownSourceIndicative')}
      </p>

      <div className="breakdown-section">
        <h4>{t('results.breakdownLosses')}</h4>
        {losses.map(renderBar)}
      </div>

      <div className="breakdown-section">
        <h4>{t('results.breakdownGains')}</h4>
        {gains.map(renderBar)}
      </div>

      <div className="breakdown-section">
        <h4>{t('results.breakdownDelivered')}</h4>
        {breakdown.deliveredUnavailable
          ? <p className="breakdown-source" role="status" data-testid="breakdown-delivered-unavailable">{t('results.breakdownDeliveredUnavailable')}</p>
          : delivered.map(renderBar)}
      </div>

      <div className="breakdown-section">
        <h4>{t('results.breakdownProduction')}</h4>
        {production.map(renderBar)}
      </div>
    </div>
  );
}
