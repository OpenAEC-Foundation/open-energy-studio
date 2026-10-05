/**
 * Step 3 Installaties › Systemen: the add actions of the former Installations
 * and Renewables ribbon tabs, and the systems of the project model as lists
 * (formerly the project tree). Click selects (inspector), double-click edits.
 */
import { Plus } from 'lucide-react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { editAction } from '../../../core/energy/projectItems';
import { formatKernelPath } from '../../../core/nta/pathUtil';
import { SYSTEM_LISTS } from '../../../core/navigation/projectPaths';
import type { DialogType, IProject } from '../../../core/energy/types';
import { formatNumber } from '../../../i18n/format';
import { Banner, Button, Card } from '../../ui';
import { ItemActions } from '../../ItemActions/ItemActions';

/**
 * Installed PV peak power: from the NTA input when it has PV systems (16.4a/16.4b),
 * otherwise from the simplified model. A table 16.1 system has no peak power of
 * its own in the input; the total is then marked as partial.
 */
export function projectPvPeak(project: IProject): { kwp: number; partial: boolean } {
  const systems = project.ntaCalculation?.pvSystems ?? [];
  if (systems.length === 0) return { kwp: project.solarPV.reduce((sum, pv) => sum + pv.peakPower, 0), partial: false };
  let watts = 0;
  let partial = false;
  for (const system of systems) {
    const peak = system.peakPower;
    if (peak.method === 'panels') watts += (peak.panelPeakPowerW ?? 0) * (peak.panelCount ?? 0);
    else if (peak.method === 'declared_specific') watts += (peak.peakPowerWPerM2 ?? 0) * (peak.panelAreaM2 ?? 0);
    else partial = true;
  }
  return { kwp: watts / 1000, partial };
}

export const INSTALLATION_ADD: Array<{ dialog: DialogType; labelKey: string }> = [
  { dialog: 'heating-system', labelKey: 'ribbon.addHeating' },
  { dialog: 'ventilation-system', labelKey: 'ribbon.addVentilation' },
  { dialog: 'cooling-system', labelKey: 'ribbon.addCooling' },
  { dialog: 'hot-water-system', labelKey: 'ribbon.addHotWater' },
  { dialog: 'solar-pv', labelKey: 'ribbon.addSolarPV' },
  { dialog: 'solar-thermal', labelKey: 'ribbon.addSolarThermal' },
];

/** The add buttons; their accessible name and title are the former ribbon labels. */
export function InstallationAddBar({ onOpenDialog }: { onOpenDialog: (type: DialogType) => void }) {
  const { t } = useI18n();
  return (
    <div className="add-bar" role="group" aria-label={t('installations.add')}>
      {INSTALLATION_ADD.map((entry) => (
        <Button key={entry.dialog} size="sm" icon={<Plus aria-hidden="true" />} title={t(entry.labelKey)}
          onClick={() => onOpenDialog(entry.dialog)}>{t(entry.labelKey)}</Button>
      ))}
    </div>
  );
}

export function InstallationsPage() {
  const { t, locale } = useI18n();
  const { state, dispatch } = useEnergy();
  const { project } = state;
  const pvPeak = projectPvPeak(project);
  const lists = SYSTEM_LISTS.map((list) => ({ ...list, items: project[list.key] as Array<{ id: string; name: string }> }));
  const total = lists.reduce((sum, list) => sum + list.items.length, 0);

  return <>
    {total === 0 && <p className="page-lead">{t('installations.empty')}</p>}
    {lists.filter((list) => list.items.length > 0).map((list) => (
      <Card key={list.key} title={`${t(list.kindKey)} (${list.items.length})`} level={2} flush
        subtitle={list.key === 'solarPV' ? `${formatNumber(pvPeak.kwp, locale, 1)}${pvPeak.partial ? '+' : ''} kWp` : undefined}>
        <table className="shell-list">
          <tbody>
            {list.items.map((item, index) => (
              <tr key={item.id} data-path={formatKernelPath([list.key, index])}
                className={state.selectedItemId === item.id ? 'selected' : undefined}
                onClick={() => dispatch({ type: 'SELECT_ITEM', payload: { id: item.id, itemType: list.itemType } })}
                onDoubleClick={() => { const action = editAction(list.itemType, item.id); if (action) dispatch(action); }}>
                <td>{item.name}</td>
                <td className="actions"><ItemActions itemType={list.itemType} id={item.id} name={item.name} /></td>
              </tr>
            ))}
          </tbody>
        </table>
      </Card>
    ))}
    <Banner tone="info">{t('installations.ntaNote')}</Banner>
  </>;
}
