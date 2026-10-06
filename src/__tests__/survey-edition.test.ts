import { describe, expect, it } from 'vitest';
import { asResidential, asUtility, surveyTemplate } from '../core/nta/SurveyTemplates';

// The survey is calculated in the project's edition (`ntaCalculation.normVersion`).
describe('survey edition', () => {
  it('carries an older edition and leaves the default out', () => {
    const residential = surveyTemplate('residential');
    expect(asResidential(residential, '2024')).toMatchObject({ normVersion: '2024' });
    expect(asResidential(residential)).not.toHaveProperty('normVersion');
    expect(asResidential(residential, '2025+C1')).not.toHaveProperty('normVersion');
    // A stored edition never overrides the project's.
    const stale = { ...residential, survey: { ...residential.survey, normVersion: '2024' } };
    expect(asResidential(stale, null)).not.toHaveProperty('normVersion');
    const utility = surveyTemplate('utility');
    expect(asUtility(utility, '2024')).toMatchObject({ normVersion: '2024' });
  });
});
