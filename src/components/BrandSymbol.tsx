/**
 * OpenAEC brand symbol — isometric open building cube.
 *
 * Inline SVG so it inherits color via CSS vars and never blocks first paint
 * on a 404. Source: OpenAEC-style-book/brandbook/assets/logo/svg/
 * openaec-symbol-v2-transparent.svg.
 *
 * Use `size` to override the default 24x24 box. Pass a `className` to control
 * placement (e.g. Tailwind utilities).
 */
interface BrandSymbolProps {
  size?: number;
  className?: string;
  /** Override the amber stroke color. Defaults to `var(--theme-accent)`. */
  color?: string;
  /** Override the lighter gold accent color. Defaults to #F59E0B. */
  accent?: string;
}

export function BrandSymbol({
  size = 24,
  className,
  color = "var(--theme-accent)",
  accent = "#F59E0B",
}: BrandSymbolProps) {
  const id = `oaec-amber-${Math.random().toString(36).slice(2, 8)}`;
  return (
    <svg
      className={className}
      width={size}
      height={size * (160 / 140)}
      viewBox="0 0 140 160"
      xmlns="http://www.w3.org/2000/svg"
      role="img"
      aria-label="OpenAEC"
    >
      <defs>
        <linearGradient id={id} x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor={accent} />
          <stop offset="100%" stopColor={color} />
        </linearGradient>
      </defs>
      <g transform="translate(70,78)">
        {/* Left face */}
        <polygon
          points="-60,-20 0,15 0,80 -60,45"
          fill={color}
          fillOpacity="0.22"
          stroke={color}
          strokeWidth="2.8"
          strokeLinejoin="round"
        />
        {/* Top face */}
        <polygon
          points="0,-55 60,-20 0,15 -60,-20"
          fill={color}
          fillOpacity="0.12"
          stroke={color}
          strokeWidth="2.8"
          strokeLinejoin="round"
        />
        {/* Right face — open edges only */}
        <line x1="60" y1="-20" x2="60" y2="45" stroke={color} strokeWidth="2.8" strokeLinecap="round" />
        <line x1="0" y1="80" x2="60" y2="45" stroke={color} strokeWidth="2.8" strokeLinecap="round" />
        {/* Inner shelf — the "open" detail */}
        <polygon
          points="60,-20 0,15 0,32 60,0"
          fill={`url(#${id})`}
          fillOpacity="0.18"
        />
        <line x1="0" y1="15" x2="0" y2="32" stroke={color} strokeWidth="2.5" strokeLinecap="round" />
        <line x1="0" y1="32" x2="60" y2="0" stroke={accent} strokeWidth="2.2" strokeLinecap="round" />
        {/* Subtle gold accents */}
        <line x1="0" y1="32" x2="-28" y2="48" stroke={accent} strokeWidth="1.2" opacity="0.3" />
        <line x1="60" y1="0" x2="32" y2="16" stroke={accent} strokeWidth="1.2" opacity="0.3" />
        <line x1="-28" y1="48" x2="32" y2="16" stroke={accent} strokeWidth="1" opacity="0.18" />
        {/* Window grid on left face */}
        <g stroke={color} strokeWidth="1.2" fill="none" opacity="0.55">
          <rect x="-44" y="-6" width="10" height="7" rx="1" transform="skewY(-30)" />
          <rect x="-30" y="-6" width="10" height="7" rx="1" transform="skewY(-30)" />
          <rect x="-44" y="6" width="10" height="7" rx="1" transform="skewY(-30)" />
          <rect x="-30" y="6" width="10" height="7" rx="1" transform="skewY(-30)" />
          <rect x="-44" y="18" width="10" height="7" rx="1" transform="skewY(-30)" />
          <rect x="-30" y="18" width="10" height="7" rx="1" transform="skewY(-30)" />
        </g>
      </g>
    </svg>
  );
}

/**
 * Full wordmark logo: symbol + "OpenAEC" + tagline. Used on welcome / empty
 * project hero. Source: openaec-logo-v2-amber-on-dark.svg, with the
 * background panel removed so it floats over our own surface.
 */
export function BrandLogo({
  height = 56,
  tagline = true,
}: {
  height?: number;
  tagline?: boolean;
}) {
  const w = height * (520 / 140);
  return (
    <svg
      width={w}
      height={height}
      viewBox="0 0 520 140"
      xmlns="http://www.w3.org/2000/svg"
      role="img"
      aria-label="OpenAEC Foundation"
    >
      <defs>
        <linearGradient id="oaec-logo-grad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#F59E0B" />
          <stop offset="100%" stopColor="#D97706" />
        </linearGradient>
      </defs>
      <g transform="translate(70,68) scale(0.52)">
        <polygon
          points="-60,-20 0,15 0,80 -60,45"
          fill="#D97706"
          fillOpacity="0.22"
          stroke="#D97706"
          strokeWidth="2.8"
          strokeLinejoin="round"
        />
        <polygon
          points="0,-55 60,-20 0,15 -60,-20"
          fill="#D97706"
          fillOpacity="0.12"
          stroke="#D97706"
          strokeWidth="2.8"
          strokeLinejoin="round"
        />
        <line x1="60" y1="-20" x2="60" y2="45" stroke="#D97706" strokeWidth="2.8" strokeLinecap="round" />
        <line x1="0" y1="80" x2="60" y2="45" stroke="#D97706" strokeWidth="2.8" strokeLinecap="round" />
        <polygon
          points="60,-20 0,15 0,32 60,0"
          fill="url(#oaec-logo-grad)"
          fillOpacity="0.18"
        />
        <line x1="0" y1="15" x2="0" y2="32" stroke="#D97706" strokeWidth="2.5" strokeLinecap="round" />
        <line x1="0" y1="32" x2="60" y2="0" stroke="#F59E0B" strokeWidth="2.2" strokeLinecap="round" />
        <line x1="0" y1="32" x2="-28" y2="48" stroke="#F59E0B" strokeWidth="1.2" opacity="0.3" />
        <line x1="60" y1="0" x2="32" y2="16" stroke="#F59E0B" strokeWidth="1.2" opacity="0.3" />
        <g stroke="#D97706" strokeWidth="1.2" fill="none" opacity="0.55">
          <rect x="-44" y="-6" width="10" height="7" rx="1" transform="skewY(-30)" />
          <rect x="-30" y="-6" width="10" height="7" rx="1" transform="skewY(-30)" />
          <rect x="-44" y="6" width="10" height="7" rx="1" transform="skewY(-30)" />
          <rect x="-30" y="6" width="10" height="7" rx="1" transform="skewY(-30)" />
          <rect x="-44" y="18" width="10" height="7" rx="1" transform="skewY(-30)" />
          <rect x="-30" y="18" width="10" height="7" rx="1" transform="skewY(-30)" />
        </g>
      </g>
      <text
        x="120"
        y="62"
        fontFamily="'Space Grotesk', system-ui, sans-serif"
        fontWeight="700"
        fontSize="40"
        letterSpacing="-0.025em"
      >
        <tspan fill="var(--theme-text)">Open</tspan>
        <tspan fill="var(--theme-accent)">AEC</tspan>
      </text>
      {tagline ? (
        <>
          <text
            x="122"
            y="88"
            fontFamily="'Inter', system-ui, sans-serif"
            fontWeight="500"
            fontSize="13"
            fill="var(--theme-text-secondary)"
            letterSpacing="0.06em"
          >
            Build free. Build together.
          </text>
          <text
            x="122"
            y="108"
            fontFamily="'Inter', system-ui, sans-serif"
            fontWeight="400"
            fontSize="10"
            fill="var(--theme-text-muted)"
            letterSpacing="0.15em"
          >
            FOUNDATION
          </text>
        </>
      ) : null}
    </svg>
  );
}
