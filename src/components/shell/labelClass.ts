/** CSS class of the energy label colour (`.label-badge.lbl-…` in shell.css). */
export function labelClassName(label: string | null | undefined): string {
  if (!label) return '';
  const plus = (label.match(/\+/g) ?? []).length;
  if (label.startsWith('A') && plus > 0) return plus === 1 ? 'lbl-ap' : `lbl-app${plus === 2 ? '2' : plus}`;
  return `lbl-${label.toLowerCase()}`;
}
