import type { IProject } from '../energy/types';

// The NTA form and the system templates keep unknown values as null so the
// adviser sees what is still open. The project route keeps them: its blank
// search (crates/nta8800-core/src/project_performance.rs, `blank_paths`)
// reports every null the kernel needs as `nta_value_missing` at its own path
// and accepts the nulls of optional members. Without the nulls a required
// member is simply absent, and the kernel can then only refuse the whole NTA
// block (`nta_calculation_block_invalid`), which the project-form guard
// (src/__tests__/nta-project-form-guard.test.tsx) found for most empty
// fields. The validate route does not read the NTA block and keeps getting
// it without nulls; the kernel's input fingerprint ignores null members, so
// both give the same fingerprint.

/** Copy of `value` without null object members, at any depth. */
export function withoutNulls<T>(value: T): T {
  if (Array.isArray(value)) {
    return value.map((item) => withoutNulls(item)) as T;
  }
  if (value !== null && typeof value === 'object') {
    const result: Record<string, unknown> = {};
    for (const [key, item] of Object.entries(value)) {
      if (item !== null && item !== undefined) result[key] = withoutNulls(item);
    }
    return result as T;
  }
  return value;
}

/** Paths of the nulls in `value`, e.g. `generator.boiler.kind`. */
export function nullPaths(value: unknown, prefix = ''): string[] {
  if (Array.isArray(value)) {
    return value.flatMap((item, index) => item === null ? [`${prefix}[${index}]`] : nullPaths(item, `${prefix}[${index}]`));
  }
  if (value !== null && typeof value === 'object') {
    return Object.entries(value).flatMap(([key, item]) => {
      const path = prefix ? `${prefix}.${key}` : key;
      return item === null ? [path] : nullPaths(item, path);
    });
  }
  return [];
}

/** The project as the project route receives it: the NTA block with its blanks (null) kept. */
export function projectRouteInput(input: IProject): IProject {
  // The flow progress is app state: it must not change the input fingerprint.
  const { workflowProgress: _progress, ...project } = input;
  void _progress;
  return project;
}

/** The project as the validate route receives it: the NTA block without nulls. */
export function kernelProject(input: IProject): IProject {
  const project = projectRouteInput(input);
  return project.ntaCalculation
    ? { ...project, ntaCalculation: withoutNulls(project.ntaCalculation) }
    : project;
}
