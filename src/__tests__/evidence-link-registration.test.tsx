import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { useEffect } from 'react';
import { beforeEach, describe, expect, it } from 'vitest';
import { act, fireEvent, screen, waitFor } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import type { IProject } from '../core/energy/types';
import { normalizeProject } from '../core/energy/normalizeProject';
import { deserializeProject, serializeProject } from '../core/io/ProjectSerializer';
import { clearSessionEvidence } from '../core/nta/Evidence';
import { resolvePointer } from '../core/nta/EvidenceLinks';
import { exampleProject } from '../core/nta/ExampleProjects';
import type { NtaEvidenceItem } from '../core/nta/KernelClient';
import { projectRouteInput } from '../core/nta/KernelInput';
import { EvidenceAttach } from '../components/EvidenceLink/EvidenceLink';
import { renderWithProviders } from './test-utils';

// A link made in the app, saved and opened again, is sent to the project
// route as `projectRouteInput`. That request is kept as a fixture, and
// crates/nta8800-core/tests/evidence_link_registration.rs runs it through the
// kernel's registration check (`evidence_link_path_unknown` must not appear).
// Regenerate with UPDATE_EVIDENCE_LINK_FIXTURE=1.
const FIXTURE = resolve(__dirname, '../../training-data/nta8800-evidence-link-project.json');

function Seed({ value }: { value: IProject }) {
  const { dispatch } = useEnergy();
  useEffect(() => { dispatch({ type: 'SET_PROJECT', payload: structuredClone(value) }); }, [dispatch, value]);
  return null;
}

let current: IProject | null = null;
function Capture() {
  const { state } = useEnergy();
  current = state.project;
  return null;
}

const photo: NtaEvidenceItem = {
  id: 'foto-gevel', kind: 'photo_detail', fileName: 'gevel.jpg', sha256: 'c'.repeat(64),
  date: '2026-10-01', checkedBy: 'EP-adviseur',
};

describe('evidence link made in the app and checked at registration', () => {
  beforeEach(() => { clearSessionEvidence(); current = null; });

  it('keeps an id link through save and reload, and the kernel can resolve it', async () => {
    const base = exampleProject('terraced_dwelling');
    // A fixed project id: the example gets a fresh one each time.
    const seeded = { ...base, id: 'voorbeeld-bewijskoppeling', registration: { ...(base.registration ?? {}), evidence: [photo] } } as IProject;
    const { container } = renderWithProviders(<>
      <Seed value={seeded} />
      <EvidenceAttach pointer="/zones/0/surfaces/0" />
      <Capture />
    </>);
    await waitFor(() => expect(container.querySelector('select')).not.toBeNull());
    await act(async () => {
      fireEvent.change(container.querySelector('select') as HTMLSelectElement, { target: { value: photo.id } });
    });
    await waitFor(() => expect(current?.registration?.evidence?.[0].linkedPaths?.length).toBe(1));

    const reopened = normalizeProject(deserializeProject(serializeProject(current as IProject)));
    const links = reopened.registration?.evidence?.[0].linkedPaths ?? [];
    expect(links).toHaveLength(1);
    expect(links[0]).toMatch(/^\/zones\/@[^/]+\/surfaces\/@/);
    expect(resolvePointer(reopened, links[0])).toBe('/zones/0/surfaces/0');

    const request = projectRouteInput(reopened);
    const text = `${JSON.stringify(request, null, 2)}\n`;
    if (process.env.UPDATE_EVIDENCE_LINK_FIXTURE === '1' || !existsSync(FIXTURE)) writeFileSync(FIXTURE, text);
    expect(JSON.parse(readFileSync(FIXTURE, 'utf8'))).toEqual(JSON.parse(text));
    expect(screen.queryByText(/evidence_link_path_unknown/)).toBeNull();
  });
});
