import { IProject } from '../energy/types';
import { withoutBom } from './importIssues';

/** File format version written by this build. */
export const FILE_VERSION = '1.0';

/** Highest major file version this build reads. A newer major version may
 * mean something else for the same fields, so it is refused instead of
 * being read as 1.x and silently written back as 1.0. */
export const SUPPORTED_MAJOR = 1;

const ENVELOPE_FIELDS = new Set(['version', 'type', 'project', 'kernel']);

/** A file written by a newer build with an incompatible format. */
export class ProjectFileTooNewError extends Error {
  readonly fileVersion: string;
  constructor(fileVersion: string) {
    super(`project_file_too_new: file version ${fileVersion}, this build reads ${SUPPORTED_MAJOR}.x`);
    this.name = 'ProjectFileTooNewError';
    this.fileVersion = fileVersion;
  }
}

/** Major and minor of a version string; a missing version is the original 1.0. */
export function parseFileVersion(version: unknown): { major: number; minor: number; text: string } {
  if (version === undefined || version === null) return { major: 1, minor: 0, text: '1.0' };
  const text = String(version);
  const match = /^(\d+)(?:\.(\d+))?/.exec(text);
  if (!match) throw new Error(`Invalid file format: unknown version "${text}"`);
  return { major: Number(match[1]), minor: Number(match[2] ?? 0), text };
}

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
  /** Version in the file (1.0 when it had none). */
  fileVersion?: string;
  /** Envelope fields this build does not know: a newer 1.x file. They are
   * not kept on the next save, so the app warns before that happens. */
  unknownFields?: string[];
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
  // A UTF-8 byte-order mark (a file re-saved by a Windows editor) is not
  // part of the JSON. The browser's File.text() drops it already; the
  // desktop read (Rust read_to_string) keeps it.
  const parsed: unknown = JSON.parse(withoutBom(json));
  if (parsed == null || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new Error('Invalid file format: expected a JSON object');
  }
  const data = parsed as IProjectFile;

  if (data.type !== FILE_TYPE) {
    throw new Error(`Invalid file format: expected type "${FILE_TYPE}", got "${data.type}"`);
  }

  const version = parseFileVersion((data as { version?: unknown }).version);
  if (version.major > SUPPORTED_MAJOR) throw new ProjectFileTooNewError(version.text);

  if (!data.project) {
    throw new Error('Invalid file format: missing project data');
  }

  const unknownFields = Object.keys(data).filter((field) => !ENVELOPE_FIELDS.has(field));
  const extras = {
    fileVersion: version.text,
    ...(unknownFields.length > 0 ? { unknownFields } : {}),
  };

  const kernel = data.kernel;
  const validStamp = kernel != null && typeof kernel.kernelVersion === 'string'
    && typeof kernel.targetNormVersion === 'string' && typeof kernel.inputFingerprint === 'string';
  return validStamp ? { project: data.project, kernel, ...extras } : { project: data.project, ...extras };
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
