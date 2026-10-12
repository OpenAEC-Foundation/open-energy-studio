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

function Editor({ section }: { section?: 'envelope' | 'zones' | 'result' }) {
  const { state, dispatch } = useEnergy();
  return <>
    <BasisopnamePanel section={section} />
    <button type="button" onClick={() => dispatch({ type: 'UPDATE_PROJECT_INFO', payload: { registration: {
      ...state.project.registration,
      evidence: [photo('ev-1', ['/basisopname/survey/envelope/surfaces/@voorgevel']), photo('ev-2', ['/zones/0'])],
    } } })}>seed photos</button>
    <output data-testid="survey">{JSON.stringify(state.project.basisopname ?? null)}</output>
    <output data-testid="evidence">{JSON.stringify(state.project.registration?.evidence ?? [])}</output>
  </>;
}

const survey = () => (JSON.parse(screen.getByTestId('survey').textContent ?? 'null') as { survey: Record<string, any> } | null)?.survey;
const evidence = () => JSON.parse(screen.getByTestId('evidence').textContent ?? '[]') as NtaEvidenceItem[];

describe('survey review 36', () => {
  // Kernel-branch UI test (12 Oct 2026 merge): main has surface cards (Add facade/roof/floor, no rename of the
  // surface id) and empty survey templates (no default lighting zone); to be ported to the cards.
  it.skip('adds, renames and removes envelope surfaces; references and photo links follow the name', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Editor section="envelope" />);
    await user.click(screen.getByRole('button', { name: 'Start dwelling survey' }));
    await user.click(screen.getByRole('button', { name: 'seed photos' }));
    const before = survey()!.envelope.surfaces.length;

    // A new surface (the template house has six; a detached house needs others).
    await user.click(screen.getByRole('button', { name: 'Add surface' }));
    expect(survey()!.envelope.surfaces).toHaveLength(before + 1);
    expect(survey()!.envelope.surfaces[before]).toMatchObject({ id: `vlak-${before + 1}`, element: 'facade', boundary: { kind: 'outdoor' } });

    // Renaming the front façade by clearing and retyping: its window and photo follow.
    await user.click(screen.getByRole('button', { name: 'Add window' }));
    const windowSurface = survey()!.envelope.windows.at(-1).surfaceId;
    expect(windowSurface).toBe('voorgevel');
    const name = screen.getAllByLabelText('Surface name')[0];
    await user.clear(name);
    expect(screen.getAllByText(/Empty or duplicate name/).length).toBeGreaterThan(0);
    await user.type(name, 'gevel-zuid');
    expect(survey()!.envelope.surfaces[0].id).toBe('gevel-zuid');
    expect(survey()!.envelope.windows.at(-1).surfaceId).toBe('gevel-zuid');
    expect(evidence()[0].linkedPaths).toEqual(['/basisopname/survey/envelope/surfaces/@gevel-zuid']);

    // A surface with a window in it cannot go; an empty one can.
    const removes = screen.getAllByRole('button', { name: 'Remove surface' });
    expect(removes[0]).toBeDisabled();
    expect(screen.getByText(/still lies in this surface/)).toBeInTheDocument();
    await user.click(removes[before]);
    expect(survey()!.envelope.surfaces).toHaveLength(before);
  }, SLOW_UI_TEST_MS);

  // Kernel-branch UI test (12 Oct 2026 merge): main has surface cards (Add facade/roof/floor, no rename of the
  // surface id) and empty survey templates (no default lighting zone); to be ported to the cards.
  it.skip('keeps zone references when a zone name is cleared and retyped past another zone name', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Editor section="zones" />);
    await user.click(screen.getByRole('button', { name: 'Start utility survey' }));
    await user.click(screen.getByRole('button', { name: 'Add calculation zone' }));
    await user.click(screen.getByRole('button', { name: 'Add calculation zone' }));
    await user.selectOptions(screen.getByLabelText('Calculation zone of lighting zone kantoren'), 'zone2');
    const name = screen.getAllByLabelText('Zone name')[1];
    // Cleared, then typed through "zone1" (the other zone's name) to "zone12".
    await user.clear(name);
    await user.type(name, 'zone12');
    expect(survey()!.zones[1].id).toBe('zone12');
    // Before review 36 the reference stayed on "zone" or "" and pointed at no zone.
    expect(survey()!.lighting[0].zoneId).toBe('zone12');
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

  // Kernel-branch UI test (12 Oct 2026 merge): main has surface cards (Add facade/roof/floor, no rename of the
  // surface id) and empty survey templates (no default lighting zone); to be ported to the cards.
  it.skip('refuses JSON that is not an object and warns when the survey changed since the JSON edit', async () => {
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
    await user.click(screen.getByRole('button', { name: 'Add surface' }));
    rerender(<Editor section="result" />);
    expect(screen.getByText(/changed since you started editing/)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Reload JSON' }));
    expect((screen.getByLabelText('Full survey (JSON)') as HTMLTextAreaElement).value).toContain('vlak-7');
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
