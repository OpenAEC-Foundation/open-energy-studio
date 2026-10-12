/** "Ga naar": focus the field of a kernel path inside a page (ontwerp §6.4). */
import { isPathWithin } from '../../core/nta/pathUtil';

const FLASH_MS = 1200;

/**
 * Scroll to and focus the element of a kernel path ("Ga naar"); the deepest `[data-path]` (or one of
 * the extra `data-paths` of a section) that contains it wins. A collapsed block around it opens
 * first; a field label hands the focus to its control.
 */
export function focusPathIn(container: HTMLElement, path: string): HTMLElement | null {
  const best = elementForPath(container, path);
  if (!best) return null;
  for (let parent = best.parentElement; parent && parent !== container; parent = parent.parentElement) {
    if (parent instanceof HTMLDetailsElement) parent.open = true;
  }
  if (best instanceof HTMLDetailsElement) best.open = true;
  const flash = best;
  const target = best.tagName === 'LABEL'
    ? best.querySelector<HTMLElement>('input, select, textarea, button') ?? best
    : best;
  target.scrollIntoView?.({ block: 'center' });
  if (!target.hasAttribute('tabindex') && !/^(INPUT|SELECT|TEXTAREA|BUTTON|A)$/.test(target.tagName)) target.tabIndex = -1;
  target.focus({ preventScroll: true });
  flash.classList.add('focus-flash');
  window.setTimeout(() => flash.classList.remove('focus-flash'), FLASH_MS);
  return target;
}

/** The deepest `[data-path]` (or extra `data-paths`) element inside `container` that contains `path`. */
export function elementForPath(container: HTMLElement, path: string): HTMLElement | null {
  let best: HTMLElement | null = null;
  let bestLength = -1;
  for (const element of Array.from(container.querySelectorAll<HTMLElement>('[data-path], [data-paths]'))) {
    const candidates = [element.dataset.path ?? '', ...(element.dataset.paths ?? '').split(' ')];
    for (const candidate of candidates) {
      // On equal paths the later element wins: it is the inner one (a field inside its section).
      if (candidate && isPathWithin(path, candidate) && candidate.length >= bestLength) {
        best = element;
        bestLength = candidate.length;
      }
    }
  }
  return best;
}

/** Visible text of a label without the text of the control inside it (a select lists its options). */
function labelText(label: Element): string {
  const copy = label.cloneNode(true) as Element;
  copy.querySelectorAll('input, select, textarea, button, [aria-hidden="true"]').forEach((node) => node.remove());
  return (copy.textContent ?? '').replace(/\s+/g, ' ').trim();
}

/**
 * The form label of the field that "Ga naar" would focus for `path`, or null when the field has no
 * label of its own (Bron & bewijs shows it instead of the kernel path, UX finding 4). Only an exact
 * field counts: a section that merely contains the path has no label for it.
 */
export function fieldLabelIn(container: HTMLElement, path: string): string | null {
  const element = elementForPath(container, path);
  if (!element) return null;
  const own = [element.dataset.path ?? '', ...(element.dataset.paths ?? '').split(' ')];
  if (!own.includes(path)) return null;
  if (element.tagName === 'LABEL') return labelText(element) || null;
  const control = /^(INPUT|SELECT|TEXTAREA)$/.test(element.tagName)
    ? element
    : element.querySelector<HTMLElement>('input, select, textarea');
  if (!control) return null;
  const aria = control.getAttribute('aria-label');
  if (aria) return aria.trim();
  const wrapping = control.closest('label');
  if (wrapping && container.contains(wrapping)) return labelText(wrapping) || null;
  if (control.id) {
    const forLabel = container.ownerDocument.querySelector(`label[for="${CSS.escape(control.id)}"]`);
    if (forLabel) return labelText(forLabel) || null;
  }
  return null;
}
