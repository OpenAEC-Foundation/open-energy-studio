/**
 * All building pages on one sheet (UI redesign F5). The shell shows each part
 * on its own Gebouw sub page; this composite keeps the old single view for the
 * editor tests and for embedding outside the shell.
 */
import {
  AirTightnessPage, ConstructionsPage, EnvelopePage, ThermalBridgesPage, ZonesPage,
} from '../shell/pages/BuildingPages';
import './EnvelopeView.css';

export function EnvelopeView() {
  return (
    <div className="envelope-view">
      <ZonesPage />
      <EnvelopePage />
      <ThermalBridgesPage />
      <AirTightnessPage />
      <ConstructionsPage />
    </div>
  );
}
