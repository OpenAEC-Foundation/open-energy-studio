/**
 * Shared "Coming in Phase B" placeholder. Used by every input page in
 * Phase A; the Results page has its own rendering.
 */
import { useNavigate } from 'react-router-dom';

export interface PagePlaceholderProps {
  title: string;
  subtitle: string;
  nextHref?: string;
  nextLabel?: string;
}

export function PagePlaceholder({
  title,
  subtitle,
  nextHref,
  nextLabel,
}: PagePlaceholderProps) {
  const navigate = useNavigate();
  return (
    <article style={containerStyle}>
      <header>
        <h2 style={titleStyle}>{title}</h2>
        <p style={subtitleStyle}>{subtitle}</p>
      </header>
      <div style={badgeStyle}>
        <strong>Coming in Phase B</strong>
        <p style={{ margin: '4px 0 0', fontSize: 13 }}>
          The full input form lives in the next phase. The Phase A migration
          delivers the substrate (Tauri commands, Zustand store, project file
          envelope); this page exists so routing is in place.
        </p>
      </div>
      {nextHref ? (
        <button type="button" style={buttonStyle} onClick={() => navigate(nextHref)}>
          {nextLabel ?? 'Next'} →
        </button>
      ) : null}
    </article>
  );
}

const containerStyle: React.CSSProperties = {
  maxWidth: 720,
  display: 'flex',
  flexDirection: 'column',
  gap: 24,
};

const titleStyle: React.CSSProperties = {
  margin: 0,
  fontSize: 22,
  fontWeight: 600,
};

const subtitleStyle: React.CSSProperties = {
  margin: '6px 0 0',
  fontSize: 14,
  color: 'var(--oes-fg-muted, #8a93a3)',
};

const badgeStyle: React.CSSProperties = {
  padding: '16px 20px',
  background: 'var(--oes-bg-elev, #14171d)',
  border: '1px solid var(--oes-border, #232830)',
  borderRadius: 8,
};

const buttonStyle: React.CSSProperties = {
  alignSelf: 'flex-start',
  padding: '8px 16px',
  background: 'var(--oes-accent, #2c6dff)',
  color: '#fff',
  border: 'none',
  borderRadius: 6,
  cursor: 'pointer',
  fontSize: 14,
};
