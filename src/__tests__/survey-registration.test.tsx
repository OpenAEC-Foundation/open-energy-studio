/**
 * Registratie of a basisopname project (feedback 8 Oct 2026): the survey's
 * answers are taken over, the checklist names what is still open, and only
 * the fields the survey did not ask are on the page.
 */
import { useEffect } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { screen, waitFor, within } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { createDefaultProject, useEnergy } from '../context/EnergyContext';
import { RegistrationPage } from '../components/shell/pages/RegistrationPage';
import { surveyTemplate } from '../core/nta/SurveyTemplates';
import {
  parseHouseNumber, registrationChecklist, surveyRegistration, takeoversPending, validUntil,
} from '../core/survey/surveyRegistration';
import type { IProject } from '../core/energy/types';
import { ProjectDataFields } from '../components/shared/ProjectDataFields';
import { projectAddress } from '../components/SurveyWizard/SurveyWizard';

function surveyProject(): IProject {
  const basisopname = { ...surveyTemplate('residential'), progress: {}, surveyDate: '2021-04-24' };
  (basisopname.survey as Record<string, unknown>).constructionYear = 2006;
  return {
    ...createDefaultProject(),
    address: 'Goejanverwelledijk 85a', city: 'Gouda', basisopname,
    registration: { postcode: '2800 AA', surveyingAdvisor: { name: 'A. Adviseur', competenceNumber: '123' } },
  } as IProject;
}

describe('project data on one place', () => {
  it('stores the house number, BAG id and client with the address, and the address text includes the number', async () => {
    const user = userEvent.setup();
    function Harness() {
      const { state } = useEnergy();
      return <><ProjectDataFields /><output data-testid="registration">{JSON.stringify(state.project.registration ?? null)}</output></>;
    }
    renderWithProviders(<Harness />);
    await user.type(screen.getByRole('textbox', { name: 'House number' }), '12');
    await user.type(screen.getByRole('textbox', { name: 'Addition' }), 'a');
    await user.type(screen.getByRole('textbox', { name: 'BAG id of the object' }), '0513010000000001');
    await user.type(screen.getByRole('textbox', { name: 'Client' }), 'Eigenaar');
    expect(JSON.parse(screen.getByTestId('registration').textContent ?? 'null'))
      .toMatchObject({ houseNumber: '12', houseNumberAddition: 'a', bagObjectId: '0513010000000001', client: 'Eigenaar' });
    expect(projectAddress({ address: 'Straat', city: 'Delft', registration: { postcode: '1234 AB', houseNumber: '12', houseNumberAddition: 'a' } })).toBe('Straat 12a, 1234 AB Delft');
    expect(projectAddress({ address: 'Straat 12', city: 'Delft', registration: { houseNumber: '12' } })).toBe('Straat 12, Delft');
  }, 60000);
});

describe('survey registration', () => {
  it('takes the survey answers over and parses the house number', () => {
    const project = surveyProject();
    expect(parseHouseNumber('Goejanverwelledijk 85a')).toEqual({ houseNumber: '85', houseNumberAddition: 'a' });
    expect(parseHouseNumber('Dorpsstraat 12-2')).toEqual({ houseNumber: '12', houseNumberAddition: '2' });
    expect(parseHouseNumber('Zonder nummer')).toEqual({});
    expect(takeoversPending(project)).toBe(true);
    expect(surveyRegistration(project)).toMatchObject({
      purpose: 'existing_building', surveyType: 'basic', constructionYear: 2006, surveyDate: '2021-04-24',
      houseNumber: '85', houseNumberAddition: 'a', postcode: '2800 AA',
      registeringAdvisor: { name: 'A. Adviseur', competenceNumber: '123' },
    });
    expect(takeoversPending({ ...project, registration: surveyRegistration(project) })).toBe(false);
    expect(validUntil('2021-04-24')).toBe('2031-04-24');
  });

  it('names what is still open', () => {
    const states = Object.fromEntries(registrationChecklist(surveyProject(), null).map((item) => [item.id, item.state]));
    expect(states).toMatchObject({
      address: 'done', adviser: 'done', surveyDate: 'done',
      bag: 'open', certificate: 'open', client: 'open', registrationDate: 'open', evidence: 'open', label: 'open',
    });
    expect(['done', 'waiting']).toContain(states.attest);
  });

  it('shows the checklist and saves the remaining fields at once', async () => {
    const user = userEvent.setup();
    const navigate = vi.fn();
    function Harness() {
      const { state, dispatch } = useEnergy();
      const loaded = state.project.basisopname != null;
      useEffect(() => {
        const project = surveyProject();
        dispatch({ type: 'SET_BASISOPNAME', payload: project.basisopname! });
        dispatch({ type: 'UPDATE_PROJECT_INFO', payload: { address: project.address, city: project.city, registration: project.registration } });
      }, [dispatch]);
      return <>
        {loaded && <RegistrationPage project={state.project} actions={{ navigate, openDialog: vi.fn() }} />}
        <output data-testid="registration">{JSON.stringify(state.project.registration ?? null)}</output>
      </>;
    }
    renderWithProviders(<Harness />);
    expect(await screen.findByRole('heading', { name: 'Ready for registration?' })).toBeInTheDocument();
    await waitFor(() => expect(JSON.parse(screen.getByTestId('registration').textContent ?? 'null'))
      .toMatchObject({ purpose: 'existing_building', surveyType: 'basic', surveyDate: '2021-04-24' }));
    expect(screen.queryByRole('textbox', { name: /WLC-GWP/ })).not.toBeInTheDocument();
    // The BAG id is project data, entered on the survey's first question; the checklist leads there.
    expect(screen.queryByRole('textbox', { name: 'BAG id of the object' })).not.toBeInTheDocument();
    const bagRow = screen.getByText('BAG identification of the object').closest('li')!;
    await user.click(within(bagRow).getByRole('button', { name: 'Go to' }));
    expect(navigate).toHaveBeenCalledWith(expect.objectContaining({ step: 'survey', question: 'adres' }));
    expect(screen.getByRole('button', { name: 'Export' })).toBeDisabled();
  }, 60000);
});
