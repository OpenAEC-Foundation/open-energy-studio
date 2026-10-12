import type { KernelState } from '../../context/KernelProvider';

/**
 * Text for an empty "Open punten" list. "Geen open punten" only holds once the
 * kernel has answered; before that, or when it cannot be reached, the list is
 * empty for another reason and says so.
 */
export function openPointsEmptyText(t: (key: string) => string, kernel: KernelState | null): string {
  if (kernel?.settled) return t('overview.noIssues');
  if (kernel?.phase === 'error') return t('status.kernel.error');
  if (kernel?.phase === 'loading') return t('status.kernel.loading');
  return t('overview.kernelStatus.none');
}
