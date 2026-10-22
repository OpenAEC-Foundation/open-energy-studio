import { useEffect } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { act, fireEvent, screen, waitFor } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { BasisopnamePanel } from '../components/BasisopnamePanel/BasisopnamePanel';
import type { NtaEvidenceItem } from '../core/nta/KernelClient';
import { isImageFile, linksAfterEdit, linksOutside } from '../core/nta/EvidenceLinks';
import { rememberEvidenceBytes, sha256Hex } from '../core/nta/Evidence';
import { EvidenceAttach } from '../components/EvidenceLink/EvidenceLink';
import { renderWithProviders, userEvent } from './test-utils';
import { SLOW_UI_TEST_MS } from './timeouts';

// Review 36: the basisopname as an adviser uses it on site.

const photo = (id: string, linkedPaths?: string[]): NtaEvidenceItem => ({
  id, kind: 'photo_detail', fileName: `${id}.jpg`, sha256: 'c'.repeat(64), ...(linkedPaths ? { linkedPaths } : {}),
});

function Editor({ section, photos }: { section?: 'envelope' | 'zones' | 'result'; photos?: NtaEvidenceItem[] }) {
  const { state, dispatch } = useEnergy();
  return <>
    <BasisopnamePanel section={section} />
    <button type="button" onClick={() => dispatch({ type: 'UPDATE_PROJECT_INFO', payload: { registration: {
      ...state.project.registration,
      evidence: photos ?? [photo('ev-1', ['/basisopname/survey/envelope/surfaces/@voorgevel']), photo('ev-2', ['/zones/0'])],
    } } })}>seed photos</button>
    <output data-testid="survey">{JSON.stringify(state.project.basisopname ?? null)}</output>
    <output data-testid="evidence">{JSON.stringify(state.project.registration?.evidence ?? [])}</output>
  </>;
}

const survey = () => (JSON.parse(screen.getByTestId('survey').textContent ?? 'null') as { survey: Record<string, any> } | null)?.survey;
const evidence = () => JSON.parse(screen.getByTestId('evidence').textContent ?? '[]') as NtaEvidenceItem[];

describe('survey review 36', () => {
  it('renames a surface in its card; openings, parts and photo links follow the new name', async () => {
    const user = userEvent.setup();
    const surfaces = '/basisopname/survey/envelope/surfaces';
    renderWithProviders(<Editor section="envelope" photos={[photo('ev-1', [`${surfaces}/@gevel-1`])]} />);
    await user.click(screen.getByRole('button', { name: 'Start dwelling survey' }));
    await user.click(screen.getByRole('button', { name: 'Add facade' }));
    await user.click(screen.getByRole('button', { name: 'Window in this facade' }));
    await user.click(screen.getByRole('button', { name: 'Add opaque part' }));
    await user.click(screen.getByRole('button', { name: 'seed photos' }));
    const part = survey()!.envelope.surfaces[1].id as string;

    const name = screen.getByLabelText('Surface name');
    await user.clear(name);
    // An empty name leaves every reference on the old one.
    expect(survey()!.envelope.windows[0].surfaceId).toBe('gevel-1');
    await user.type(name, 'Voorgevel');
    const after = JSON.parse(screen.getByTestId('survey').textContent ?? 'null');
    expect(after.survey.envelope.surfaces[0].id).toBe('Voorgevel');
    expect(after.survey.envelope.windows[0].surfaceId).toBe('Voorgevel');
    expect(after.surfaceParts[part]).toBe('Voorgevel');
    expect(evidence()[0].linkedPaths).toEqual([`${surfaces}/@Voorgevel`]);
    // The card shows the adviser's own name instead of "Facade 1".
    expect(screen.getAllByText('Voorgevel').length).toBeGreaterThan(0);
  }, SLOW_UI_TEST_MS);

  // Ported to the surface cards (12 Oct 2026 merge). The cards add a facade, roof or floor with a fresh id
  // and name it by position ("Facade 1") until the adviser gives it a name of its own (test above). What changes a
  // surface's place is removing one (with its parts and openings) and adding one, which reuses the first
  // free id: the photo links must stay on their own surface and never pass to the new one.
  it('adds and removes envelope surfaces; openings and photo links stay with their own surface', async () => {
    const user = userEvent.setup();
    const surfaces = '/basisopname/survey/envelope/surfaces';
    renderWithProviders(<Editor section="envelope" photos={[
      photo('ev-1', [`${surfaces}/@gevel-1`]), photo('ev-2', [`${surfaces}/@gevel-2`]),
      photo('ev-3', ['/basisopname/survey/envelope/windows/@raam1']), photo('ev-4', ['/zones/0']),
    ]} />);
    await user.click(screen.getByRole('button', { name: 'Start dwelling survey' }));
    // A new dwelling survey starts without surfaces.
    expect(survey()!.envelope.surfaces).toEqual([]);

    // Two new façades, each to outside air until the adviser answers otherwise.
    await user.click(screen.getByRole('button', { name: 'Add facade' }));
    await user.click(screen.getByRole('button', { name: 'Add facade' }));
    expect(survey()!.envelope.surfaces).toHaveLength(2);
    expect(survey()!.envelope.surfaces[0]).toMatchObject({ id: 'gevel-1', element: 'facade', boundary: { kind: 'outdoor' } });
    expect(survey()!.envelope.surfaces[1]).toMatchObject({ id: 'gevel-2', element: 'facade', boundary: { kind: 'outdoor' } });

    // A window in the first façade, then photos of both façades and of the window.
    await user.click(screen.getAllByRole('button', { name: 'Window in this facade' })[0]);
    expect(survey()!.envelope.windows).toEqual([expect.objectContaining({ id: 'raam1', surfaceId: 'gevel-1' })]);
    await user.click(screen.getByRole('button', { name: 'seed photos' }));

    // Removing the first façade takes its window along (the kernel refuses an opening without its surface);
    // its photos and the window's stay in the register, unlinked. The other façade keeps its photo.
    await user.click(screen.getAllByRole('button', { name: 'Remove surface' })[0]);
    expect(survey()!.envelope.surfaces.map((surface: { id: string }) => surface.id)).toEqual(['gevel-2']);
    expect(survey()!.envelope.windows).toEqual([]);
    expect(evidence()[0]).not.toHaveProperty('linkedPaths');
    expect(evidence()[1].linkedPaths).toEqual([`${surfaces}/@gevel-2`]);
    expect(evidence()[2]).not.toHaveProperty('linkedPaths');
    expect(evidence()[3].linkedPaths).toEqual(['/zones/0']);

    // A façade added now takes the free id "gevel-1" and a window in it "raam1" again: the photos of the
    // removed ones do not reappear on them.
    await user.click(screen.getByRole('button', { name: 'Add facade' }));
    await user.click(screen.getAllByRole('button', { name: 'Window in this facade' })[1]);
    expect(survey()!.envelope.surfaces.map((surface: { id: string }) => surface.id)).toEqual(['gevel-2', 'gevel-1']);
    expect(survey()!.envelope.windows).toEqual([expect.objectContaining({ id: 'raam1', surfaceId: 'gevel-1' })]);
    expect(evidence()[0]).not.toHaveProperty('linkedPaths');
    expect(evidence()[2]).not.toHaveProperty('linkedPaths');

    // An opaque part of the remaining façade with a window moved into it: removing the façade removes the
    // part and that window too, so no opening is left pointing at a surface that is gone.
    await user.click(screen.getAllByRole('button', { name: 'Add opaque part' })[0]);
    expect(survey()!.envelope.surfaces.map((surface: { id: string }) => surface.id)).toEqual(['gevel-2', 'gevel-1', 'gevel-2-deel-2']);
    await user.click(screen.getAllByRole('button', { name: 'Window in this facade' })[0]);
    expect(survey()!.envelope.windows[1]).toMatchObject({ id: 'raam2', surfaceId: 'gevel-2' });
    await user.selectOptions(screen.getAllByLabelText('In surface')[0], 'gevel-2-deel-2');
    expect(survey()!.envelope.windows[1].surfaceId).toBe('gevel-2-deel-2');
    await user.click(screen.getAllByRole('button', { name: 'Remove surface' })[0]);
    expect(survey()!.envelope.surfaces.map((surface: { id: string }) => surface.id)).toEqual(['gevel-1']);
    expect(survey()!.envelope.windows).toEqual([expect.objectContaining({ id: 'raam1', surfaceId: 'gevel-1' })]);
    expect(evidence()[1]).not.toHaveProperty('linkedPaths');
    expect(evidence()[3].linkedPaths).toEqual(['/zones/0']);
  }, SLOW_UI_TEST_MS);

  it('removes an opaque part: its window moves to the surface and its photo link goes', async () => {
    const user = userEvent.setup();
    const part = '/basisopname/survey/envelope/surfaces/@gevel-1-deel-2';
    renderWithProviders(<Editor section="envelope" photos={[photo('ev-1', [part]), photo('ev-2', ['/basisopname/survey/envelope/windows/@raam1'])]} />);
    await user.click(screen.getByRole('button', { name: 'Start dwelling survey' }));
    await user.click(screen.getByRole('button', { name: 'Add facade' }));
    await user.click(screen.getByRole('button', { name: 'Add opaque part' }));
    await user.click(screen.getByRole('button', { name: 'Window in this facade' }));
    await user.selectOptions(screen.getByLabelText('In surface'), 'gevel-1-deel-2');
    await user.click(screen.getByRole('button', { name: 'seed photos' }));

    await user.click(screen.getByRole('button', { name: 'Remove opaque part' }));
    expect(survey()!.envelope.surfaces.map((surface: { id: string }) => surface.id)).toEqual(['gevel-1']);
    // The window stays in the façade (an opening needs a surface the kernel knows) with its photo.
    expect(survey()!.envelope.windows).toEqual([expect.objectContaining({ id: 'raam1', surfaceId: 'gevel-1' })]);
    expect(evidence()[0]).not.toHaveProperty('linkedPaths');
    expect(evidence()[1].linkedPaths).toEqual(['/basisopname/survey/envelope/windows/@raam1']);
    // A new part takes the same id; the removed part's photo does not come back on it.
    await user.click(screen.getByRole('button', { name: 'Add opaque part' }));
    expect(survey()!.envelope.surfaces.map((surface: { id: string }) => surface.id)).toEqual(['gevel-1', 'gevel-1-deel-2']);
    expect(evidence()[0]).not.toHaveProperty('linkedPaths');
  }, SLOW_UI_TEST_MS);

  // Ported (12 Oct 2026 merge): the utility survey starts without zones and lighting zones, so the adviser
  // adds the lighting zone first ("verlichting-1") before giving it a calculation zone.
  it('keeps zone references when a zone name is cleared and retyped past another zone name', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Editor section="zones" />);
    await user.click(screen.getByRole('button', { name: 'Start utility survey' }));
    expect(survey()!.lighting).toEqual([]);
    await user.click(screen.getByRole('button', { name: 'Add lighting zone' }));
    await user.click(screen.getByRole('button', { name: 'Add calculation zone' }));
    await user.click(screen.getByRole('button', { name: 'Add calculation zone' }));
    expect(survey()!.zones.map((zone: { id: string }) => zone.id)).toEqual(['zone1', 'zone2']);
    await user.selectOptions(screen.getByLabelText('Calculation zone of lighting zone verlichting-1'), 'zone2');
    expect(survey()!.lighting[0].zoneId).toBe('zone2');
    const name = screen.getAllByLabelText('Zone name')[1];
    // Cleared, then typed through "zone1" (the other zone's name) to "zone12".
    await user.clear(name);
    expect(screen.getAllByText(/Empty or duplicate name/).length).toBeGreaterThan(0);
    await user.type(name, 'zone12');
    expect(survey()!.zones[1].id).toBe('zone12');
    // Before review 36 the reference stayed on "zone" or "" and pointed at no zone.
    expect(survey()!.lighting[0].zoneId).toBe('zone12');
    expect(screen.queryByText(/Empty or duplicate name/)).toBeNull();
  }, SLOW_UI_TEST_MS);

  it('asks before discarding the survey and unlinks its photos', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Editor section="envelope" />);
    await user.click(screen.getByRole('button', { name: 'Start dwelling survey' }));
    await user.click(screen.getByRole('button', { name: 'seed photos' }));

    await user.click(screen.getByRole('button', { name: 'Discard survey' }));
    expect(screen.getByText(/1 linked file stays in the evidence register/)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Cancel' }));
    expect(survey()).toBeDefined();

    await user.click(screen.getByRole('button', { name: 'Discard survey' }));
    const dialog = screen.getByRole('dialog');
    await user.click(Array.from(dialog.querySelectorAll('button')).find((button) => button.textContent === 'Discard survey')!);
    expect(survey()).toBeUndefined();
    // The photo stays in the register, unlinked; the link into the project stays.
    expect(evidence()[0]).not.toHaveProperty('linkedPaths');
    expect(evidence()[1].linkedPaths).toEqual(['/zones/0']);
  }, SLOW_UI_TEST_MS);

  // Ported (12 Oct 2026 merge): the field change elsewhere is "Add facade" on the surface cards; the new
  // dwelling survey has no surfaces, so the façade is "gevel-1".
  it('refuses JSON that is not an object and warns when the survey changed since the JSON edit', async () => {
    const user = userEvent.setup();
    const { rerender } = renderWithProviders(<Editor section="result" />);
    await user.click(screen.getByRole('button', { name: 'Start dwelling survey' }));
    const box = screen.getByLabelText('Full survey (JSON)') as HTMLTextAreaElement;
    fireEvent.change(box, { target: { value: 'null' } });
    await user.click(screen.getByRole('button', { name: 'Apply JSON' }));
    expect(screen.getByRole('alert')).toHaveTextContent('one JSON object');
    expect(survey()).toBeTruthy();

    // A JSON edit, then a field change elsewhere: the JSON view says Apply would overwrite it.
    fireEvent.change(box, { target: { value: JSON.stringify({ ...survey(), constructionYear: 1950 }) } });
    expect(screen.queryByText(/changed since you started editing/)).toBeNull();
    rerender(<Editor section="envelope" />);
    await user.click(screen.getByRole('button', { name: 'Add facade' }));
    rerender(<Editor section="result" />);
    expect(screen.getByText(/changed since you started editing/)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Reload JSON' }));
    expect(screen.queryByText(/changed since you started editing/)).toBeNull();
    const reloaded = (screen.getByLabelText('Full survey (JSON)') as HTMLTextAreaElement).value;
    expect(reloaded).toContain('gevel-1');
    // The reload drops the unapplied JSON edit.
    expect(JSON.parse(reloaded).constructionYear).not.toBe(1950);
  }, SLOW_UI_TEST_MS);
});

describe('survey photos and links (review 36)', () => {
  it('counts HEIF, TIFF and BMP photos as images', () => {
    for (const name of ['IMG_0001.HEIC', 'IMG_0002.heif', 'scan.tif', 'scan.TIFF', 'plaat.bmp', 'a.jpeg']) {
      expect(isImageFile(name)).toBe(true);
    }
    expect(isImageFile('factuur.pdf')).toBe(false);
  });

  it('moves links to a renamed item and drops links into a removed block', () => {
    const before = { basisopname: { survey: { pv: [{ id: 'pv1' }, { id: 'pv2' }] } } };
    const after = { basisopname: { survey: { pv: [{ id: 'pv-dak' }, { id: 'pv2' }] } } };
    const items = [photo('ev-1', ['/basisopname/survey/pv/@pv1']), photo('ev-2', ['/basisopname/survey/pv/@pv2'])];
    const moved = linksAfterEdit(items, before, after);
    expect(moved[0].linkedPaths).toEqual(['/basisopname/survey/pv/@pv-dak']);
    expect(moved[1]).toBe(items[1]);

    const kept = linksOutside([photo('ev-1', ['/basisopname/survey/pv/@pv1', '/zones/0']), photo('ev-2', ['/basisopname'])], '/basisopname');
    expect(kept[0].linkedPaths).toEqual(['/zones/0']);
    expect(kept[1]).not.toHaveProperty('linkedPaths');
  });

  it('shows the icon instead of a broken thumbnail for a photo the browser cannot draw', async () => {
    vi.stubGlobal('URL', Object.assign(URL, { createObjectURL: () => 'blob:heic', revokeObjectURL: () => undefined }));
    const bytes = new Uint8Array([1, 2, 3]);
    // The register checks the file against its hash before showing it.
    const item = { ...photo('ev-9', ['/basisopname/survey/pv/0']), fileName: 'IMG_0001.HEIC', sha256: await sha256Hex(bytes) };
    function Seeded() {
      const { dispatch } = useEnergy();
      useEffect(() => {
        dispatch({ type: 'UPDATE_PROJECT_INFO', payload: { registration: { evidence: [item] } } });
        // After the register holds the item, so the session store does not prune its bytes.
        rememberEvidenceBytes(item.sha256, bytes);
      }, [dispatch]);
      return <EvidenceAttach photo pointer="/basisopname/survey/pv/0" />;
    }
    const { container } = renderWithProviders(<Seeded />);
    await waitFor(() => expect(container.querySelector('img.evidence-link__thumb')).not.toBeNull());
    await act(async () => { fireEvent.error(container.querySelector('img.evidence-link__thumb')!); });
    expect(container.querySelector('img.evidence-link__thumb')).toBeNull();
    vi.unstubAllGlobals();
  });
});
