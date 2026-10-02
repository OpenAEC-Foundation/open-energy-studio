import { IProject } from '../energy/types';

/** File format version */
const FILE_VERSION = '1.0';

/** File type identifier */
const FILE_TYPE = 'open-energy-studio';

/**
 * Kernel and norm version plus the input fingerprint at save time, so a
 * reopened project shows when it would be calculated with a different
 * kernel (BRL 9500 §4.2.4/§4.2.7: reproducible, relabel with the original
 * software version).
 */
export interface KernelStamp {
  kernelVersion: string;
  targetNormVersion: string;
  inputFingerprint: string;
}

/** Envelope for the serialized project file */
interface IProjectFile {
  version: string;
  type: string;
  project: IProject;
  kernel?: KernelStamp;
}

export interface LoadedProjectFile {
  project: IProject;
  kernel?: KernelStamp;
}

/**
 * Serialize an IProject to a JSON string suitable for saving to disk.
 */
export function serializeProject(project: IProject, kernel?: KernelStamp | null): string {
  const file: IProjectFile = {
    version: FILE_VERSION,
    type: FILE_TYPE,
    project,
    ...(kernel ? { kernel } : {}),
  };
  return JSON.stringify(file, null, 2);
}

/**
 * Deserialize a saved file, including the kernel stamp when present.
 * Throws if the file format is invalid.
 */
export function deserializeProjectFile(json: string): LoadedProjectFile {
  const data: IProjectFile = JSON.parse(json);

  if (data.type !== FILE_TYPE) {
    throw new Error(`Invalid file format: expected type "${FILE_TYPE}", got "${data.type}"`);
  }

  if (!data.project) {
    throw new Error('Invalid file format: missing project data');
  }

  const kernel = data.kernel;
  const validStamp = kernel != null && typeof kernel.kernelVersion === 'string'
    && typeof kernel.targetNormVersion === 'string' && typeof kernel.inputFingerprint === 'string';
  return validStamp ? { project: data.project, kernel } : { project: data.project };
}

/**
 * Deserialize a JSON string (from a saved file) back into an IProject.
 * Throws if the file format is invalid.
 */
export function deserializeProject(json: string): IProject {
  return deserializeProjectFile(json).project;
}

export type StampDifference = 'version' | 'input';

/**
 * Compares the saved stamp with the current kernel's stamp of the same
 * project. A missing current stamp (kernel unavailable) reports nothing.
 */
export function compareKernelStamp(saved: KernelStamp | undefined, current: KernelStamp | null): StampDifference[] {
  if (!saved || !current) return [];
  const differences: StampDifference[] = [];
  if (saved.kernelVersion !== current.kernelVersion || saved.targetNormVersion !== current.targetNormVersion) {
    differences.push('version');
  } else if (saved.inputFingerprint !== current.inputFingerprint) {
    differences.push('input');
  }
  return differences;
}

export function describeStamp(stamp: KernelStamp): string {
  return `${stamp.kernelVersion} / ${stamp.targetNormVersion}`;
}
