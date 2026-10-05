import type { ReactNode } from 'react';
import {
  AlertCircle, AlertTriangle, CheckCircle2, History, Info, ShieldQuestion, XCircle,
} from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import { cx } from './Button';

export type Tone = 'ok' | 'warn' | 'err' | 'info' | 'unv' | 'accent' | 'neutral';

const TONE_ICON: Record<Exclude<Tone, 'accent' | 'neutral'>, ReactNode> = {
  ok: <CheckCircle2 aria-hidden="true" />,
  warn: <AlertTriangle aria-hidden="true" />,
  err: <XCircle aria-hidden="true" />,
  info: <Info aria-hidden="true" />,
  unv: <ShieldQuestion aria-hidden="true" />,
};

// ── Card ─────────────────────────────────────────────────────────────

export interface CardProps {
  title?: ReactNode;
  subtitle?: ReactNode;
  actions?: ReactNode;
  children?: ReactNode;
  className?: string;
  /** Heading level for the title (default h3). */
  level?: 2 | 3 | 4;
  /** Render the body without padding (tables that run edge to edge). */
  flush?: boolean;
  'aria-label'?: string;
}

/** Card with an optional head (title, subtitle, actions on the right). Never nest cards. */
export function Card({ title, subtitle, actions, children, className, level = 3, flush = false, ...aria }: CardProps) {
  const Heading = `h${level}` as 'h2' | 'h3' | 'h4';
  return (
    <section className={cx('ui-card', className)} aria-label={aria['aria-label']}>
      {(title || actions) && (
        <header className="ui-card__head">
          <div className="ui-card__titles">
            {title && <Heading className="ui-card__title">{title}</Heading>}
            {subtitle && <span className="ui-card__sub">{subtitle}</span>}
          </div>
          {actions && <div className="ui-card__actions">{actions}</div>}
        </header>
      )}
      <div className={cx('ui-card__body', flush && 'ui-card__body--flush')}>{children}</div>
    </section>
  );
}

/** A divided section inside a card body. */
export function CardSection({ title, reference, children }: { title?: ReactNode; reference?: ReactNode; children: ReactNode }) {
  return (
    <div className="ui-card__section">
      {title && (
        <div className="ui-card__section-head">
          <h4>{title}</h4>
          {reference}
        </div>
      )}
      {children}
    </div>
  );
}

// ── Pills, tags ──────────────────────────────────────────────────────

export interface PillProps {
  tone?: Tone;
  icon?: ReactNode;
  children: ReactNode;
  className?: string;
  title?: string;
}

export function Pill({ tone = 'neutral', icon, children, className, title }: PillProps) {
  return (
    <span className={cx('ui-pill', `ui-pill--${tone}`, className)} title={title}>
      {icon}
      {children}
    </span>
  );
}

/** Status pill: icon + text are both required so the state never depends on colour alone. */
export function StatusPill({ tone, children, className, title }: { tone: Exclude<Tone, 'accent' | 'neutral'>; children: ReactNode; className?: string; title?: string }) {
  return <Pill tone={tone} icon={TONE_ICON[tone]} className={className} title={title}>{children}</Pill>;
}

/** Small caps badge for a kind, e.g. WONING, BESTAAND, ISSO 75.1. */
export function Tag({ children, className }: { children: ReactNode; className?: string }) {
  return <span className={cx('ui-tag', className)}>{children}</span>;
}

// ── Banner ───────────────────────────────────────────────────────────

export interface BannerProps {
  tone: 'unv' | 'info' | 'warn' | 'err';
  title?: ReactNode;
  children?: ReactNode;
  action?: ReactNode;
  className?: string;
  /** Override the ARIA role; errors and warnings default to `alert`, others to `status`. */
  role?: 'alert' | 'status' | 'note';
}

/** Full-width inline message. Errors and warnings keep `role="alert"` (tests rely on it). */
export function Banner({ tone, title, children, action, className, role }: BannerProps) {
  const ariaRole = role ?? (tone === 'err' || tone === 'warn' ? 'alert' : 'status');
  return (
    <div className={cx('ui-banner', `ui-banner--${tone}`, className)} role={ariaRole}>
      <span className="ui-banner__icon">{TONE_ICON[tone]}</span>
      <div className="ui-banner__text">
        {title && <strong>{title}</strong>}
        {title && children ? ' ' : null}
        {children}
      </div>
      {action && <div className="ui-banner__action">{action}</div>}
    </div>
  );
}

// ── States ───────────────────────────────────────────────────────────

export interface StateProps {
  title: ReactNode;
  children?: ReactNode;
  icon?: ReactNode;
  action?: ReactNode;
  className?: string;
}

/** Empty state: what is missing and the one action that fills it. */
export function EmptyState({ title, children, icon, action, className }: StateProps) {
  return (
    <div className={cx('ui-state', className)} role="status">
      {icon && <span className="ui-state__icon" aria-hidden="true">{icon}</span>}
      <p className="ui-state__title">{title}</p>
      {children && <p className="ui-state__text">{children}</p>}
      {action && <div className="ui-state__action">{action}</div>}
    </div>
  );
}

/** Error state: keeps `role="alert"` so the failure is announced. */
export function ErrorState({ title, children, action, className }: StateProps) {
  return (
    <div className={cx('ui-state', 'ui-state--error', className)} role="alert">
      <span className="ui-state__icon" aria-hidden="true"><AlertCircle /></span>
      <p className="ui-state__title">{title}</p>
      {children && <p className="ui-state__text">{children}</p>}
      {action && <div className="ui-state__action">{action}</div>}
    </div>
  );
}

/** Placeholder block while content loads. */
export function Skeleton({ width = '100%', height = 14, className }: { width?: number | string; height?: number | string; className?: string }) {
  return <span className={cx('ui-skeleton', className)} style={{ width, height }} aria-hidden="true" />;
}

/** Shown above results that belong to an earlier input. */
export function StaleBanner({ action, children }: { action?: ReactNode; children?: ReactNode }) {
  const { t } = useI18n();
  return (
    <div className="ui-banner ui-banner--warn ui-banner--stale" role="status">
      <span className="ui-banner__icon"><History aria-hidden="true" /></span>
      <div className="ui-banner__text">
        <strong>{t('ui.stale.title')}</strong> {children ?? t('ui.stale.text')}
      </div>
      {action && <div className="ui-banner__action">{action}</div>}
    </div>
  );
}
