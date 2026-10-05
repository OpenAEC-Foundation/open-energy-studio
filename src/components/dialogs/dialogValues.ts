/**
 * Helpers that keep an edit dialog's save a no-op when the user changed
 * nothing: a percentage field is shown as stored × 100, and dividing it back
 * can drift or turn an absent value into 0.
 */
export function fractionFromPercent<T extends number | undefined>(percent: number, stored: T): number | T {
  if (stored === undefined) return percent === 0 ? stored : percent / 100;
  return Math.abs(stored * 100 - percent) < 1e-9 ? stored : percent / 100;
}

/** The heat-pump record without its id, with every saved field kept. */
export function heatPumpDraftOf<T extends { id: string }>(record: T): Omit<T, 'id'> {
  const { id: _id, ...rest } = record;
  return rest;
}
