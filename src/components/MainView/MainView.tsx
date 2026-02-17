import { useEnergy } from '../../context/EnergyContext';
import { ProjectView } from '../ProjectView/ProjectView';
import { EnvelopeView } from '../EnvelopeView/EnvelopeView';
import { ResultsView } from '../ResultsView/ResultsView';
import { ReportView } from '../ReportView/ReportView';
import { Building3DView } from '../Building3DView/Building3DView';
import { UValueCalculator } from '../UValueCalculator/UValueCalculator';
import { ThermalBridgeCalculator } from '../ThermalBridgeCalculator/ThermalBridgeCalculator';
import './MainView.css';

export function MainView() {
  const { state } = useEnergy();

  return (
    <div className="main-view">
      {state.viewMode === 'project' && <ProjectView />}
      {state.viewMode === 'envelope' && <EnvelopeView />}
      {(state.viewMode === 'installations' || state.viewMode === 'renewables') && <ProjectView />}
      {state.viewMode === 'results' && <ResultsView />}
      {state.viewMode === 'report' && <ReportView />}
      {state.viewMode === 'model3d' && <Building3DView />}
      {state.viewMode === 'uvalue-calc' && <UValueCalculator />}
      {state.viewMode === 'thermal-bridge-calc' && <ThermalBridgeCalculator />}
    </div>
  );
}
