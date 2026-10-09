/**
 * New project in one window (feedback 9 Oct 2026): BAG lookup through PDOK,
 * the setup answers in the project, and houseboat and caravan as their own
 * dwelling choice (as in Uniec3's new-calculation menu).
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen, within } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { createDefaultProject } from '../context/EnergyContext';
import { lookupAddresses, normalizePostcode } from '../core/io/bagLookup';
import { applySetup, firstOpenSurveyStep, rememberedAdviser, type NewProjectSetup } from '../core/io/newProjectSetup';
import { NewProjectDialog } from '../components/WelcomeScreen/NewProjectDialog';

afterEach(() => { localStorage.clear(); vi.unstubAllGlobals(); });

const json = (body: unknown) => ({ ok: true, status: 200, json: async () => body });
const LOCATIE = { response: { docs: [{
  weergavenaam: 'Goejanverwelledijk 85, 2807CB Gouda', straatnaam: 'Goejanverwelledijk', huisnummer: 85,
  postcode: '2807CB', woonplaatsnaam: 'Gouda', adresseerbaarobject_id: '0513010013073557',
}] } };
const VBO = { features: [{ properties: { gebruiksdoel: 'woonfunctie', oppervlakte: 145, 'pand.href': ['https://api.pdok.nl/kadaster/bag/ogc/v2/collections/pand/items/p1'] } }] };
const PAND = { properties: { bouwjaar: 1932, aantal_verblijfsobjecten: 1 } };
const bag = () => vi.fn(async (url: string) => json(url.includes('locatieserver') ? LOCATIE : url.includes('/pand/') ? PAND : VBO)) as unknown as typeof fetch;

describe('BAG lookup', () => {
  it('normalizes a Dutch postcode', () => {
    expect(normalizePostcode('2807 cb')).toBe('2807CB');
    expect(normalizePostcode('0807CB')).toBeNull();
  });

  it('finds the address with use, floor area, year of construction and units', async () => {
    const fetcher = bag();
    const [address] = await lookupAddresses('2807 CB', '85', '', fetcher);
    expect(address).toMatchObject({
      street: 'Goejanverwelledijk', houseNumber: '85', postcode: '2807CB', city: 'Gouda', bagObjectId: '0513010013073557',
      purpose: 'woonfunctie', floorAreaM2: 145, constructionYear: 1932, unitsInBuilding: 1,
    });
    expect(String((fetcher as unknown as ReturnType<typeof vi.fn>).mock.calls[0][0])).toContain('fq=postcode:2807CB&fq=huisnummer:85');
  });
});

describe('project setup', () => {
  const address = {
    display: 'Goejanverwelledijk 85, 2807CB Gouda', street: 'Goejanverwelledijk', houseNumber: '85', addition: '', postcode: '2807CB',
    city: 'Gouda', bagObjectId: '0513010013073557', floorAreaM2: 145, constructionYear: 1932,
  };
  const setup = (patch: Partial<NewProjectSetup>): NewProjectSetup => ({ kind: 'existing_residential', address, ...patch });
  const survey = (project: ReturnType<typeof applySetup>) => (project.basisopname as { survey: Record<string, unknown> }).survey;

  it('fills the project data, the BAG values and the answered questions', () => {
    const project = applySetup(createDefaultProject(), setup({
      dwelling: 'end_or_corner', roofType: 'flat', heating: 'heat_pump', ventilation: 'balanced', cooling: false, pv: true,
      adviser: { name: 'A. Adviseur', competenceNumber: 'V123', certificateNumber: 'C9' },
    }));
    expect(project).toMatchObject({ name: 'Goejanverwelledijk 85 Gouda', address: 'Goejanverwelledijk', city: 'Gouda' });
    expect(project.registration).toMatchObject({
      postcode: '2807CB', houseNumber: '85', bagObjectId: '0513010013073557', certificateNumber: 'C9',
      surveyingAdvisor: { name: 'A. Adviseur', competenceNumber: 'V123' },
    });
    expect(survey(project)).toMatchObject({
      constructionYear: 1932, usableFloorAreaM2: 145,
      dwelling: { kind: 'single_family', position: 'end_or_corner', roofType: 'flat' },
      heating: { generator: { kind: 'heat_pump' } }, ventilation: { principle: 'balanced' },
    });
    expect((project.basisopname as { progress: { done: string[] } }).progress.done.sort()).toEqual(
      ['koeling.koeling', 'ventilatie.systeem', 'verwarming.toestel', 'woning.adres', 'woning.soort'],
    );
    // The general question (year, area) stays open: the BAG values are checked at the survey.
    expect(firstOpenSurveyStep(project)).toBe('woning');
  });

  it('makes a houseboat by berth date and a caravan their own building kind', () => {
    expect(survey(applySetup(createDefaultProject(), setup({ dwelling: 'houseboat_2018' })))).toMatchObject({
      dwelling: { kind: 'single_family', position: 'detached' },
      envelope: { buildingKind: { kind: 'floating', newBerthSince2018: true } },
    });
    expect(survey(applySetup(createDefaultProject(), setup({ dwelling: 'houseboat' })))).toMatchObject({
      envelope: { buildingKind: { kind: 'floating', newBerthSince2018: false } },
    });
    expect(survey(applySetup(createDefaultProject(), setup({ dwelling: 'caravan' })))).toMatchObject({
      envelope: { buildingKind: { kind: 'caravan' } },
    });
  });

  it('leaves a new-build project without a survey', () => {
    const project = applySetup(createDefaultProject(), setup({ kind: 'residential', dwelling: 'terraced' }));
    expect(project.basisopname).toBeUndefined();
    expect(project.name).toBe('Goejanverwelledijk 85 Gouda');
  });
});

describe('new-project window', () => {
  it('looks up the address, offers houseboats as their own choice and remembers the adviser', async () => {
    vi.stubGlobal('fetch', bag());
    const user = userEvent.setup();
    const onCreate = vi.fn();
    renderWithProviders(<NewProjectDialog onCreate={onCreate} onClose={vi.fn()} />);
    await user.type(screen.getByRole('textbox', { name: 'Postcode' }), '2807CB');
    await user.type(screen.getByRole('textbox', { name: /^House number/ }), '85');
    await user.click(screen.getByRole('button', { name: 'Look up' }));
    expect(await screen.findByText('Goejanverwelledijk 85, 2807CB Gouda')).toBeInTheDocument();
    expect(screen.getByText(/built 1932/)).toBeInTheDocument();
    const types = within(screen.getByRole('group', { name: 'Dwelling type' }));
    expect(types.getByRole('button', { name: /Houseboat, berth before/ })).toBeInTheDocument();
    await user.click(types.getByRole('button', { name: /Houseboat, berth from/ }));
    await user.click(within(screen.getByRole('group', { name: 'Solar panels' })).getByRole('button', { name: 'No' }));
    await user.type(screen.getByRole('textbox', { name: 'Name' }), 'A. Adviseur');
    await user.click(screen.getByRole('button', { name: 'Create project' }));
    expect(onCreate).toHaveBeenCalledWith(expect.objectContaining({
      kind: 'existing_residential', dwelling: 'houseboat_2018', pv: false,
      address: expect.objectContaining({ bagObjectId: '0513010013073557', constructionYear: 1932 }),
    }));
    expect(rememberedAdviser()?.name).toBe('A. Adviseur');
  });
});
