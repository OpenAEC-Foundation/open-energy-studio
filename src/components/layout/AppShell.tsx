import { type ReactNode, useEffect } from "react";

import Ribbon from "../ribbon/Ribbon";
import StatusBar from "../StatusBar";
import TitleBar from "../TitleBar";
import { Sidebar } from "./Sidebar";
import { useProjectStore } from "../../store/projectStore";
import "../../themes.css";

interface AppShellProps {
  children: ReactNode;
}

/**
 * Top-level app shell — TitleBar + Ribbon + Sidebar + main + StatusBar.
 *
 * Patterned after `open-heatloss-studio`'s AppShell but stripped of auth,
 * auto-save, modeller store, backstage modal, settings dialog, feedback
 * dialog, and the conflict dialog. These are added back as Phase B builds
 * each feature on the new substrate.
 */
export function AppShell({ children }: AppShellProps) {
  const error = useProjectStore((s) => s.error);
  const setError = useProjectStore((s) => s.setError);

  // Default theme from previous build; user-configurable in a future
  // SettingsDialog. `data-theme` drives every CSS variable in themes.css.
  useEffect(() => {
    if (!document.documentElement.dataset.theme) {
      document.documentElement.dataset.theme = "openaec";
    }
    // Show the Tauri window once content has rendered — avoids the white
    // flash that otherwise shows up on cold start.
    import("@tauri-apps/api/window")
      .then(({ getCurrentWindow }) => getCurrentWindow().show())
      .catch(() => {});
  }, []);

  return (
    <div className="flex h-screen flex-col overflow-hidden">
      <TitleBar />
      <Ribbon />

      <div className="flex min-h-0 flex-1">
        <Sidebar />
        <main className="flex-1 overflow-auto bg-surface text-on-surface">
          {error && (
            <div className="flex items-center gap-2 border-b border-red-600/30 bg-red-600/15 px-4 py-2.5 text-sm text-red-400">
              <span className="flex-1">{error.message}</span>
              <button
                onClick={() => setError(null)}
                className="shrink-0 rounded p-0.5 hover:bg-red-600/20"
                aria-label="Sluiten"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                  className="h-4 w-4"
                >
                  <path d="M6.28 5.22a.75.75 0 00-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 101.06 1.06L10 11.06l3.72 3.72a.75.75 0 101.06-1.06L11.06 10l3.72-3.72a.75.75 0 00-1.06-1.06L10 8.94 6.28 5.22z" />
                </svg>
              </button>
            </div>
          )}
          {children}
        </main>
      </div>

      <StatusBar />
    </div>
  );
}
