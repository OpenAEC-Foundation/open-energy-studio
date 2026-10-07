import { SOFTWARE_ATTEST_NUMBER } from './Registration';

/**
 * The BRL 9501 attest number of this program, or null until the program is
 * attested. Every screen and report reads the number through this accessor.
 */
export function softwareAttestNumber(): string | null {
  return SOFTWARE_ATTEST_NUMBER.trim() || null;
}

/**
 * What the app and the report show about the attest. An attested program
 * carries the NL-EPBD mark and the attest number (BRL 9501 §8.2 and §8.4
 * opmerking, p. 15). Without an attest no mark is shown, only the plain
 * statement that the program is not attested.
 *
 * The official NL-EPBD artwork is a protected mark and is not in the
 * repository. Once the licence is granted it goes in `src/assets/nl-epbd/`;
 * until then `markText` is the placeholder that fills the mark's slot.
 */
export interface AttestMark {
  attested: boolean;
  attestNumber: string | null;
  /** Text in the mark's slot; null without an attest. */
  markText: string | null;
}

export function attestMark(attestNumber: string | null = softwareAttestNumber()): AttestMark {
  const number = attestNumber?.trim() || null;
  return {
    attested: number !== null,
    attestNumber: number,
    markText: number ? `NL-EPBD · BRL 9501-attest ${number}` : null,
  };
}
