/** "Ga naar": focus the field of a kernel path inside a page (ontwerp §6.4). */
import { isPathWithin } from '../../core/nta/pathUtil';

const FLASH_MS = 1200;

/**
 * Scroll to and focus the element of a kernel path ("Ga naar"); the deepest `[data-path]` (or one of
 * the extra `data-paths` of a section) that contains it wins. A collapsed block around it opens
 * first; a field label hands the focus to its control.
 */
export function focusPathIn(container: HTMLElement, path: string): HTMLElement | null {
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
