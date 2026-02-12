import { IProject } from '../energy/types';

/** File format version */
const FILE_VERSION = '1.0';

/** File type identifier */
const FILE_TYPE = 'open-energy-studio';

/** Envelope for the serialized project file */
interface IProjectFile {
  version: string;
  type: string;
  project: IProject;
}

/**
 * Serialize an IProject to a JSON string suitable for saving to disk.
 */
export function serializeProject(project: IProject): string {
  const file: IProjectFile = {
    version: FILE_VERSION,
    type: FILE_TYPE,
    project,
  };
  return JSON.stringify(file, null, 2);
}

/**
 * Deserialize a JSON string (from a saved file) back into an IProject.
 * Throws if the file format is invalid.
 */
export function deserializeProject(json: string): IProject {
  const data: IProjectFile = JSON.parse(json);

  if (data.type !== FILE_TYPE) {
    throw new Error(`Invalid file format: expected type "${FILE_TYPE}", got "${data.type}"`);
  }

  if (!data.project) {
    throw new Error('Invalid file format: missing project data');
  }

  return data.project;
}
