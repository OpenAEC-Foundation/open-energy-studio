import type { IProject } from '../energy/types';
import { assessProjectWithRust } from '../nta/KernelClient';
import type { KernelStamp } from './ProjectSerializer';

/** Kernel stamp of a project, or null when the Rust kernel is unavailable. */
export async function stampProject(project: IProject): Promise<KernelStamp | null> {
  try {
    const assessment = await assessProjectWithRust(project);
    return {
      kernelVersion: assessment.kernelVersion,
      targetNormVersion: assessment.targetNormVersion,
      inputFingerprint: assessment.inputFingerprint,
    };
  } catch {
    return null;
  }
}
