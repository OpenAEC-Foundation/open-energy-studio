import { forwardRef, type ButtonHTMLAttributes, type ReactNode } from 'react';

export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger';
export type ButtonSize = 'sm' | 'md' | 'lg';

function classes(...parts: Array<string | false | null | undefined>): string {
  return parts.filter(Boolean).join(' ');
}

export { classes as cx };

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  /** Icon shown left of the label; replaced by a spinner while `loading`. */
  icon?: ReactNode;
  /** Keyboard hint shown after the label, e.g. "Ctrl S". */
  kbd?: string;
  loading?: boolean;
}

/**
 * Button of the design system (ontwerp.md §5.2). One `primary` per area;
 * `danger` keeps a neutral surface with red text. While loading the spinner
 * replaces the icon so the width stays the same.
 */
export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  { variant = 'secondary', size = 'md', icon, kbd, loading = false, className, children, disabled, type = 'button', ...rest },
  ref,
) {
  return (
    <button
      ref={ref}
      type={type}
      className={classes('ui-btn', `ui-btn--${variant}`, size !== 'md' && `ui-btn--${size}`, className)}
      disabled={disabled || loading}
      aria-busy={loading || undefined}
      {...rest}
    >
      {loading ? <span className="ui-spinner" aria-hidden="true" /> : icon}
      {children}
      {kbd && <Kbd>{kbd}</Kbd>}
    </button>
  );
});

export interface IconButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  /** Required: an icon alone carries no meaning for screen readers. */
  'aria-label': string;
  icon: ReactNode;
  size?: 'sm' | 'md';
}

/** Square 32 px button with only an icon; the label doubles as tooltip. */
export const IconButton = forwardRef<HTMLButtonElement, IconButtonProps>(function IconButton(
  { icon, className, size = 'md', type = 'button', title, ...rest },
  ref,
) {
  return (
    <button
      ref={ref}
      type={type}
      className={classes('ui-iconbtn', size === 'sm' && 'ui-iconbtn--sm', className)}
      title={title ?? rest['aria-label']}
      {...rest}
    >
      {icon}
    </button>
  );
});

/** Keyboard shortcut hint. */
export function Kbd({ children }: { children: ReactNode }) {
  return <kbd className="ui-kbd">{children}</kbd>;
}
