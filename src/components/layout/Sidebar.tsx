import { NavLink } from "react-router-dom";

import { useProjectStore } from "../../store/projectStore";

/* ─── SVG icon set (inline, no dependency) ─── */

function IconHome({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      width="20"
      height="20"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M3 9l9-7 9 7v11a2 2 0 01-2 2H5a2 2 0 01-2-2z" />
      <polyline points="9 22 9 12 15 12 15 22" />
    </svg>
  );
}

function IconGrid({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      width="20"
      height="20"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <rect x="3" y="3" width="7" height="7" />
      <rect x="14" y="3" width="7" height="7" />
      <rect x="3" y="14" width="7" height="7" />
      <rect x="14" y="14" width="7" height="7" />
    </svg>
  );
}

function IconLayers({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      width="20"
      height="20"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <polygon points="12 2 2 7 12 12 22 7 12 2" />
      <polyline points="2 17 12 22 22 17" />
      <polyline points="2 12 12 17 22 12" />
    </svg>
  );
}

function IconCpu({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      width="20"
      height="20"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <rect x="4" y="4" width="16" height="16" rx="2" ry="2" />
      <rect x="9" y="9" width="6" height="6" />
      <line x1="9" y1="1" x2="9" y2="4" />
      <line x1="15" y1="1" x2="15" y2="4" />
      <line x1="9" y1="20" x2="9" y2="23" />
      <line x1="15" y1="20" x2="15" y2="23" />
      <line x1="20" y1="9" x2="23" y2="9" />
      <line x1="20" y1="14" x2="23" y2="14" />
      <line x1="1" y1="9" x2="4" y2="9" />
      <line x1="1" y1="14" x2="4" y2="14" />
    </svg>
  );
}

function IconBarChart({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      width="20"
      height="20"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <line x1="18" y1="20" x2="18" y2="10" />
      <line x1="12" y1="20" x2="12" y2="4" />
      <line x1="6" y1="20" x2="6" y2="14" />
    </svg>
  );
}

function IconCheck({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      width="20"
      height="20"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M22 11.08V12a10 10 0 11-5.93-9.14" />
      <polyline points="22 4 12 14.01 9 11.01" />
    </svg>
  );
}

const NAV_MAIN = [
  { to: "/project", label: "Project", Icon: IconHome },
  { to: "/zones", label: "Rekenzones", Icon: IconGrid },
  { to: "/constructies", label: "Constructies", Icon: IconLayers },
  { to: "/systems", label: "Installaties", Icon: IconCpu },
  { to: "/results", label: "Resultaten", Icon: IconBarChart },
] as const;

const NAV_TOOLS = [
  { to: "/verify", label: "Verificatie", Icon: IconCheck },
] as const;

interface NavItemProps {
  to: string;
  label: string;
  Icon: React.ComponentType<{ className?: string }>;
}

function NavItem({ to, label, Icon }: NavItemProps) {
  return (
    <li>
      <NavLink
        to={to}
        className={({ isActive }) =>
          `flex items-center gap-3 rounded px-3 py-2 text-sm transition-colors
          ${
            isActive
              ? "bg-primary font-medium text-on-accent"
              : "text-on-surface-muted hover:bg-[var(--theme-hover)] hover:text-on-surface"
          }`
        }
      >
        {({ isActive }) => (
          <>
            <Icon className={isActive ? "text-white" : "text-scaffold-gray"} />
            {label}
          </>
        )}
      </NavLink>
    </li>
  );
}

function SaveStatus() {
  const isDirty = useProjectStore((s) => s.isDirty);
  const project = useProjectStore((s) => s.project);
  if (!project) return null;
  return (
    <div className="flex items-center gap-2 px-3 py-2 text-xs text-scaffold-gray">
      <span
        className={`inline-block h-2 w-2 rounded-full ${
          isDirty ? "bg-amber-500" : "bg-green-500"
        }`}
      />
      <span>{isDirty ? "Niet opgeslagen" : "Opgeslagen"}</span>
    </div>
  );
}

export function Sidebar() {
  return (
    <aside className="flex w-sidebar shrink-0 flex-col border-r border-[var(--theme-border-subtle)] bg-surface-alt text-on-surface-secondary overflow-hidden">
      <nav className="flex-1 overflow-y-auto px-3 py-4">
        <p className="px-3 pb-1.5 pt-3 font-mono text-2xs font-medium uppercase tracking-wider text-scaffold-gray">
          Berekening
        </p>
        <ul className="space-y-0.5">
          {NAV_MAIN.map((item) => (
            <NavItem key={item.to} {...item} />
          ))}
        </ul>

        <div className="mx-3 my-3 border-t border-[var(--theme-border-subtle)]" />

        <p className="px-3 pb-1.5 pt-3 font-mono text-2xs font-medium uppercase tracking-wider text-scaffold-gray">
          Tools
        </p>
        <ul className="space-y-0.5">
          {NAV_TOOLS.map((item) => (
            <NavItem key={item.to} {...item} />
          ))}
        </ul>
      </nav>

      <SaveStatus />

      <div className="border-t border-[var(--theme-border-subtle)] px-4 py-3">
        <p className="text-2xs text-scaffold-gray">Open Energy Studio</p>
      </div>
    </aside>
  );
}
