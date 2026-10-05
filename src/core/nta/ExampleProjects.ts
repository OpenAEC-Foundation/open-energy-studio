import terracedDwelling from '../../../training-data/nta8800-example-terraced-dwelling.json';
import smallOffice from '../../../training-data/nta8800-example-office.json';
import type { IProject } from '../energy/types';

// Complete fictitious projects that calculate with the NTA 8800 kernel out
// of the box; the kernel tests (project_performance.rs) load the same files.

export type ExampleKind = 'terraced_dwelling' | 'small_office';

export const EXAMPLE_KINDS: ExampleKind[] = ['terraced_dwelling', 'small_office'];

/** A fresh copy of an example project with its own id. */
export function exampleProject(kind: ExampleKind): IProject {
  const source = kind === 'terraced_dwelling' ? terracedDwelling : smallOffice;
  const project = structuredClone(source) as unknown as IProject;
  return { ...project, id: crypto.randomUUID() };
}
