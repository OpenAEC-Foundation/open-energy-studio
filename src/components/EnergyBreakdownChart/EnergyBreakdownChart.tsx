import { useI18n } from '../../i18n/i18n';
import type { IEnergyBreakdown } from '../../core/energy/types';
import './EnergyBreakdownChart.css';

interface EnergyBreakdownChartProps {
  breakdown: IEnergyBreakdown;
}

export function EnergyBreakdownChart({ breakdown }: EnergyBreakdownChartProps) {
  const { t } = useI18n();

  const losses = [
    { key: 'transmissionLoss', label: t('results.transmissionLoss'), value: breakdown.transmissionLoss, color: '#ef4444' },
    { key: 'ventilationLoss', label: t('results.ventilationLoss'), value: breakdown.ventilationLoss, color: '#f97316' },
    { key: 'infiltrationLoss', label: t('results.infiltrationLoss'), value: breakdown.infiltrationLoss, color: '#f59e0b' },
  ];

  const gains = [
    { key: 'solarGain', label: t('results.solarGain'), value: breakdown.solarGain, color: '#eab308' },
    { key: 'internalGain', label: t('results.internalGain'), value: breakdown.internalGain, color: '#84cc16' },
  ];

  const delivered = [
    { key: 'heatingEnergy', label: t('results.heatingEnergy'), value: breakdown.heatingEnergy, color: '#ef4444' },
    { key: 'coolingEnergy', label: t('results.coolingEnergy'), value: breakdown.coolingEnergy, color: '#3b82f6' },
    { key: 'ventilationEnergy', label: t('results.ventilationEnergy'), value: breakdown.ventilationEnergy, color: '#8b5cf6' },
    { key: 'hotWaterEnergy', label: t('results.hotWaterEnergy'), value: breakdown.hotWaterEnergy, color: '#06b6d4' },
    { key: 'lightingEnergy', label: t('results.lightingEnergy'), value: breakdown.lightingEnergy, color: '#f59e0b' },
  ];

  const production = [
    { key: 'pvProduction', label: t('results.pvProduction'), value: breakdown.pvProduction, color: '#22c55e' },
    { key: 'solarThermal', label: t('results.solarThermalProduction'), value: breakdown.solarThermalProduction, color: '#10b981' },
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
      <span className="breakdown-bar-value">{item.value.toFixed(0)} kWh</span>
    </div>
  );

  return (
    <div className="energy-breakdown-chart">
      <h3>{t('results.breakdown')}</h3>

      <div className="breakdown-section">
        <h4>Verliezen</h4>
        {losses.map(renderBar)}
      </div>

      <div className="breakdown-section">
        <h4>Winsten</h4>
        {gains.map(renderBar)}
      </div>

      <div className="breakdown-section">
        <h4>Geleverde energie</h4>
        {delivered.map(renderBar)}
      </div>

      <div className="breakdown-section">
        <h4>Opwek</h4>
        {production.map(renderBar)}
      </div>
    </div>
  );
}
