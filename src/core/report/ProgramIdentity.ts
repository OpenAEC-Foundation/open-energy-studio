import { version } from '../../../package.json';
import { softwareAttestNumber } from '../nta/Attest';
import { SOFTWARE_NAME } from '../nta/Registration';
import { escapeHtml } from './HtmlEscaping';

/** What an output shows while the program has no BRL 9501 attest (instead of leaving the item out). */
export const ATTEST_NOT_YET = 'nog niet toegekend';

/**
 * The program behind an output, kept apart from the calculation core:
 * Omgevingsregeling art. 5.14 lid 1 onder b names the program, BRL 9501 §5.2
 * (p. 9) the version of the rekenkern and §6.1 (p. 10) the version in the
 * attest. A missing attest number is stated, not omitted.
 */
export interface ProgramIdentity {
  name: string;
  programVersion: string;
  kernelVersion: string | null;
  normEdition: string | null;
  attestNumber: string | null;
}

export function programIdentity(
  kernelVersion?: string | null,
  normEdition?: string | null,
  attestNumber: string | null = softwareAttestNumber(),
): ProgramIdentity {
  return {
    name: SOFTWARE_NAME,
    programVersion: version,
    kernelVersion: kernelVersion ?? null,
    normEdition: normEdition ?? null,
    attestNumber: attestNumber?.trim() || null,
  };
}

/** "Open Energy Studio 0.2.0". */
export function programText(identity: ProgramIdentity): string {
  return `${identity.name} ${identity.programVersion}`;
}

/** The attest number, or the statement that none has been granted yet. */
export function attestText(identity: ProgramIdentity): string {
  return identity.attestNumber ?? ATTEST_NOT_YET;
}

/** Two table rows (four cells each): program, kernel, edition and attest number. */
export function programIdentityRows(identity: ProgramIdentity): string {
  const td = (value: string | null) => `<td>${escapeHtml(value ?? '—')}</td>`;
  return `<tr><th>Rekenprogramma</th>${td(programText(identity))}<th>Rekenkernversie</th>${td(identity.kernelVersion)}</tr>
      <tr><th>Bepalingsmethode</th>${td(identity.normEdition)}<th>BRL 9501-attestnummer</th>${td(attestText(identity))}</tr>`;
}
