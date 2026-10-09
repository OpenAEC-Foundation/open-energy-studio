/**
 * Says when a newer build of the web app is online, with a button to load it
 * (feedback 9 Oct 2026: a fix was live, the open tab still ran the old build).
 */
import { useEffect, useState } from 'react';
import { isTauri } from '@tauri-apps/api/core';
import { useI18n } from '../../i18n/i18n';
import { newerBuildAvailable } from '../../core/io/newVersion';

const CHECK_EVERY_MS = 5 * 60 * 1000;
const AUTO_RELOAD_KEY = 'oes.autoReloaded.v1';

export function NewVersionBanner() {
  const { t } = useI18n();
  const [newer, setNewer] = useState(false);
  useEffect(() => {
    if (import.meta.env.DEV || import.meta.env.MODE === 'test' || isTauri()) return undefined;
    let stopped = false;
    const check = () => { void newerBuildAvailable().then((found) => { if (found && !stopped) setNewer(true); }); };
    // At start the browser may have served a cached start page of an older build (the server sends no
    // cache instruction; a new tab showed yesterday's build, 9 Oct 2026). The check has refreshed the
    // cached page, so one reload starts the current build; once per tab, so a failed refresh cannot loop.
    void newerBuildAvailable().then((found) => {
      if (!found || stopped) return;
      let reloaded = false;
      try { reloaded = sessionStorage.getItem(AUTO_RELOAD_KEY) === '1'; sessionStorage.setItem(AUTO_RELOAD_KEY, '1'); } catch { reloaded = true; }
      if (reloaded) setNewer(true); else window.location.reload();
    });
    const timer = window.setInterval(check, CHECK_EVERY_MS);
    const onVisible = () => { if (document.visibilityState === 'visible') check(); };
    document.addEventListener('visibilitychange', onVisible);
    return () => { stopped = true; window.clearInterval(timer); document.removeEventListener('visibilitychange', onVisible); };
  }, []);
  if (!newer) return null;
  return <div className="new-version-banner" role="status">
    <span>{t('shell.newVersion')}</span>
    <button type="button" className="btn btn-sm btn-primary" onClick={() => window.location.reload()}>{t('shell.newVersionLoad')}</button>
  </div>;
}
