import { describe, expect, it } from 'vitest';
import { fireEvent, screen } from '@testing-library/react';
import { normalizeProject } from '../core/energy/normalizeProject';
import { createDefaultProject } from '../context/EnergyContext';
import type { IProject } from '../core/energy/types';
import { ZoneEditorDialog } from '../components/dialogs/ZoneEditorDialog/ZoneEditorDialog';
import { renderWithProviders } from './test-utils';

// The numeric fuzz found that the kernel ignores these project-model numbers, so a negative
// value never reached a refusal. They still feed the editor, the simplified engine and the
// exports: a file with them is refused on opening, and the editors ignore a typed minus sign.
describe('project-model numbers that the NTA calculation does not read', () => {
  const project = createDefaultProject();

  it('refuses negative or non-finite zone, construction and heating-system numbers on opening', () => {
    const zone = project.zones[0];
    const cases: Array<[IProject, string]> = [
      [{ ...project, zones: [{ ...zone, volume: -1 }] }, 'zones[0].volume'],
      [{ ...project, zones: [{ ...zone, height: -2.6 }] }, 'zones[0].height'],
      [{ ...project, zones: [{ ...zone, floorArea: Number.NaN }] }, 'zones[0].floorArea'],
      [{ ...project, constructions: [{ id: 'c', name: 'c', layers: [], rcValue: -1, uValue: 0.2 }] } as unknown as IProject,
        'constructions[0].rcValue'],
      [{ ...project, constructions: [{ id: 'c', name: 'c', layers: [], rcValue: 4, uValue: -0.2 }] } as unknown as IProject,
        'constructions[0].uValue'],
      [{ ...project, heatingSystems: [{ id: 'h', name: 'h', type: 'hr107', cop: -1, coverageFraction: 1 }] } as unknown as IProject,
        'heatingSystems[0].cop'],
      [{ ...project, heatingSystems: [{ id: 'h', name: 'h', type: 'hr107', cop: 0.9, coverageFraction: 1.5 }] } as unknown as IProject,
        'heatingSystems[0].coverageFraction'],
    ];
    for (const [broken, path] of cases) expect(() => normalizeProject(broken), path).toThrow(path);
    // Valid values, including 0, still open, and an untouched project is returned as is.
    expect(normalizeProject(project)).toBe(project);
    const zero = { ...project, zones: [{ ...zone, volume: 0 }] };
    expect(normalizeProject(zero).zones[0].volume).toBe(0);
  });

  it('ignores a negative area, volume or height typed in the zone editor', () => {
    renderWithProviders(<ZoneEditorDialog onClose={() => {}} />);
    const fields = screen.getAllByRole('spinbutton') as HTMLInputElement[];
    const [area, volume, height] = fields;
    fireEvent.change(volume, { target: { value: '250' } });
    expect(volume.value).toBe('250');
    fireEvent.change(volume, { target: { value: '-5' } });
    expect(volume.value).toBe('250');
    fireEvent.change(height, { target: { value: '-2.6' } });
    expect(height.value).toBe('2.6');
    fireEvent.change(area, { target: { value: '-1' } });
    expect(area.value).toBe('0');
  });
});
