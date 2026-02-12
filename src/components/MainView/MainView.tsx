import { useEnergy } from '../../context/EnergyContext';
import { ProjectView } from '../ProjectView/ProjectView';
import { EnvelopeView } from '../EnvelopeView/EnvelopeView';
import { ResultsView } from '../ResultsView/ResultsView';
import { ReportView } from '../ReportView/ReportView';
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
    </div>
  );
}
