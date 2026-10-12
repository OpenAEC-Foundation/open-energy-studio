/**
 * Shared accessibility audit for rendered forms, pages and dialogs (axe-core is
 * not a dependency, so these are targeted DOM checks):
 *  - every control, button and widget role has an accessible name;
 *  - ids are unique and aria references resolve;
 *  - every dialog has role="dialog", aria-modal="true" and a name;
 *  - an invalid control says why (aria-describedby to text, or aria-description),
 *    and every field error (role="alert" in a field) is referenced by its control;
 *  - no positive tabindex and nothing focusable inside aria-hidden.
 */

// The accessible-name algorithm Testing Library uses for *ByRole (a transitive dependency; its
// package "exports" hide the typings from this tsconfig, hence the typed dynamic import).
const A11Y_API = 'dom-accessibility-api';
const { computeAccessibleName } = await import(/* @vite-ignore */ A11Y_API) as { computeAccessibleName: (element: Element) => string };

export const FOCUSABLE = 'a[href], button:not([disabled]), input:not([disabled]):not([type="hidden"]), '
  + 'select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

const NAMED = 'button, a[href], input:not([type="hidden"]), select, textarea, [role="button"], [role="tab"], '
  + '[role="checkbox"], [role="switch"], [role="combobox"], [role="slider"], [role="radio"], [role="menuitem"]';

export const accessibleName = (element: Element): string => computeAccessibleName(element).trim();

const describe = (element: Element) => element.outerHTML.slice(0, 140);

/** Problems in `root`, one line each; an empty list means the audit passed. */
export function auditAccessibility(root: ParentNode): string[] {
  const problems: string[] = [];
  const doc = (root as Node).ownerDocument ?? document;

  for (const element of Array.from(root.querySelectorAll(NAMED))) {
    if (element.closest('[aria-hidden="true"], [hidden]') || (element as HTMLElement).style?.display === 'none') continue;
    if (!accessibleName(element)) problems.push(`no accessible name: ${describe(element)}`);
  }

  const ids = new Map<string, number>();
  for (const element of Array.from(root.querySelectorAll('[id]'))) ids.set(element.id, (ids.get(element.id) ?? 0) + 1);
  for (const [id, count] of ids) if (count > 1) problems.push(`duplicate id: ${id} (${count}×)`);
  for (const attribute of ['aria-labelledby', 'aria-describedby', 'aria-controls']) {
    for (const element of Array.from(root.querySelectorAll(`[${attribute}]`))) {
      for (const ref of (element.getAttribute(attribute) ?? '').split(/\s+/).filter(Boolean)) {
        if (!doc.getElementById(ref)) problems.push(`${attribute} → missing #${ref}: ${describe(element)}`);
      }
    }
  }

  for (const dialog of Array.from(root.querySelectorAll('[role="dialog"], [role="alertdialog"], dialog'))) {
    if (dialog.tagName !== 'DIALOG' && dialog.getAttribute('aria-modal') !== 'true') problems.push(`dialog without aria-modal="true": ${describe(dialog)}`);
    if (!accessibleName(dialog)) problems.push(`dialog without a name: ${describe(dialog)}`);
  }

  for (const control of Array.from(root.querySelectorAll('[aria-invalid="true"]'))) {
    const described = (control.getAttribute('aria-describedby') ?? '').split(/\s+/).filter(Boolean)
      .map((ref) => doc.getElementById(ref)?.textContent?.trim() ?? '').join(' ').trim();
    const description = control.getAttribute('aria-description')?.trim() ?? '';
    if (!described && !description) problems.push(`invalid control without a description: ${describe(control)}`);
  }
  const referenced = new Set(Array.from(root.querySelectorAll('[aria-describedby]'))
    .flatMap((element) => (element.getAttribute('aria-describedby') ?? '').split(/\s+/).filter(Boolean)));
  for (const error of Array.from(root.querySelectorAll('.ui-field [role="alert"], .ui-field__error, .dialog-field .field-error-text'))) {
    if (!error.id || !referenced.has(error.id)) problems.push(`field error not linked to its control: ${describe(error)}`);
  }

  for (const element of Array.from(root.querySelectorAll('[tabindex]'))) {
    if (Number(element.getAttribute('tabindex')) > 0) problems.push(`positive tabindex: ${describe(element)}`);
  }
  for (const hidden of Array.from(root.querySelectorAll('[aria-hidden="true"]'))) {
    if (hidden.matches(FOCUSABLE) || hidden.querySelector(FOCUSABLE)) problems.push(`focusable inside aria-hidden: ${describe(hidden)}`);
  }
  return problems;
}
