/**
 * A new utility survey starts empty and names its required answers; a flat
 * roof asks no orientation (feedback 8 Oct 2026).
 */
import { useEffect } from 'react';
import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { useEnergy } from '../context/EnergyContext';
import { BasisopnamePanel } from '../components/BasisopnamePanel/BasisopnamePanel';
import { missingSurveyAnswers, surveyTemplate } from '../core/nta/SurveyTemplates';
import { questionForPath } from '../core/survey/surveyFlow';

describe('utility survey template', () => {
  it('starts empty and names the answers the kernel needs', () => {
    const stored = surveyTemplate('utility');
    expect((stored.survey.envelope as { surfaces: unknown[] }).surfaces).toEqual([]);
    expect(missingSurveyAnswers(stored)).toEqual(expect.arrayContaining(['buildingType', 'functions', 'constructionYear', 'heating.generator']));
    expect(questionForPath('heatingInstallation.capacityKw', stored)).toEqual({ step: 'verwarming', question: 'afgifte' });
  });

  it('asks the building type and hides the orientation of a flat roof', async () => {
    const user = userEvent.setup();
    function Harness() {
      const { state, dispatch } = useEnergy();
      useEffect(() => {
        const stored = surveyTemplate('utility');
        (stored.survey.envelope as { surfaces: unknown[] }).surfaces = [
          { id: 'dak', element: 'roof', boundary: { kind: 'outdoor' }, grossAreaM2: 100, tiltDeg: 0, cavity: false, insulation: { kind: 'none_or_unknown' }, sourceReference: '' },
        ];
        dispatch({ type: 'SET_BASISOPNAME', payload: stored });
      }, [dispatch]);
      return <>{state.project.basisopname && <BasisopnamePanel />}
        <output data-testid="survey">{JSON.stringify(state.project.basisopname?.survey ?? null)}</output></>;
    }
    renderWithProviders(<Harness />);
    await user.selectOptions(await screen.findByRole('combobox', { name: 'Building type' }), 'single_layer');
    expect(JSON.parse(screen.getByTestId('survey').textContent ?? '{}').buildingType)
      .toEqual({ kind: 'single_layer', position: 'detached', roof: 'flat' });
    expect(screen.getByText('Flat roof (tilt up to 5°): no orientation needed.')).toBeInTheDocument();
  }, 60000);
});
