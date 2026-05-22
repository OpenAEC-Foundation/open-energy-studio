import type { ReactNode } from "react";

interface PageShellProps {
  title: string;
  subtitle?: string;
  actions?: ReactNode;
  children: ReactNode;
}

/**
 * Page chrome shared by every NTA 8800 page. Header + optional toolbar +
 * scrollable content area. Tailwind classes only; theme tokens via
 * `themes.css`.
 */
export function PageShell({ title, subtitle, actions, children }: PageShellProps) {
  return (
    <div className="flex h-full flex-col">
      <header className="flex items-end justify-between border-b border-[var(--theme-border-subtle)] bg-[var(--theme-bg-lighter)] px-8 py-5">
        <div>
          <h1 className="font-heading text-2xl font-semibold text-on-surface">{title}</h1>
          {subtitle ? (
            <p className="mt-1 text-sm text-on-surface-secondary">{subtitle}</p>
          ) : null}
        </div>
        {actions ? <div className="flex items-center gap-2">{actions}</div> : null}
      </header>
      <div className="flex-1 overflow-auto px-8 py-6">{children}</div>
    </div>
  );
}

interface EmptyCardProps {
  icon?: ReactNode;
  title: string;
  description: string;
  action?: ReactNode;
}

/** Reusable empty state — large icon, title, description, optional CTA. */
export function EmptyCard({ icon, title, description, action }: EmptyCardProps) {
  return (
    <div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-[var(--theme-border)] bg-[var(--theme-bg-lighter)] px-8 py-16 text-center">
      {icon ? <div className="mb-4 text-accent opacity-80">{icon}</div> : null}
      <h2 className="font-heading text-lg font-medium text-on-surface">{title}</h2>
      <p className="mt-2 max-w-md text-sm text-on-surface-secondary">{description}</p>
      {action ? <div className="mt-6">{action}</div> : null}
    </div>
  );
}
