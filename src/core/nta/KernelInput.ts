import type { IProject } from '../energy/types';

// The NTA form and the system templates keep unknown values as null so the
// adviser sees what is still open. The kernel's typed input has no null: an
// optional field is absent, and a required one that is absent is reported as
// a gap naming its path. So nulls are left out before the kernel call.

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

/** The project as the kernel receives it: the NTA block without nulls. */
export function kernelProject(project: IProject): IProject {
  return project.ntaCalculation
    ? { ...project, ntaCalculation: withoutNulls(project.ntaCalculation) }
    : project;
}
