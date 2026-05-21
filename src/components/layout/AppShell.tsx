/**
 * AppShell — Phase A scaffold.
 *
 * Sidebar with navigation links, a slim header, and an `<Outlet />` for the
 * active page. Phase B replaces this with the OpenAEC ribbon shell + the
 * real page-flow chrome.
 */
import { NavLink, Outlet } from 'react-router-dom';
import { useTranslation } from 'react-i18next';

import { HeaderBar } from './HeaderBar';
import './AppShell.css';

const NAV_ITEMS: Array<{ to: string; key: string; label: string }> = [
  { to: '/project', key: 'project', label: 'Project' },
  { to: '/zones', key: 'zones', label: 'Rekenzones' },
  { to: '/constructions', key: 'constructions', label: 'Constructies' },
  { to: '/systems', key: 'systems', label: 'Installaties' },
  { to: '/results', key: 'results', label: 'Resultaten' },
  { to: '/verify', key: 'verify', label: 'Verificatie' },
];

export function AppShell() {
  const { t } = useTranslation();

  return (
    <div className="oes-shell">
      <aside className="oes-shell__sidebar">
        <header className="oes-shell__brand">
          <h1>Open Energy Studio</h1>
          <p className="oes-shell__brand-sub">NTA 8800 / BENG</p>
        </header>
        <nav className="oes-shell__nav">
          {NAV_ITEMS.map((item) => (
            <NavLink
              key={item.key}
              to={item.to}
              className={({ isActive }) =>
                'oes-shell__nav-link' +
                (isActive ? ' oes-shell__nav-link--active' : '')
              }
            >
              {t(`nav.${item.key}`, item.label)}
            </NavLink>
          ))}
        </nav>
        <footer className="oes-shell__footer">
          <NavLink to="/legacy" className="oes-shell__legacy-link">
            ↩ Legacy view
          </NavLink>
        </footer>
      </aside>
      <div className="oes-shell__content">
        <HeaderBar />
        <main className="oes-shell__main">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
