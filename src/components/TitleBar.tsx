import { useCallback, useEffect, useRef, useState } from "react";

import { isTauri } from "../lib/backend";
import "./TitleBar.css";

interface TitleBarProps {
  onSettingsClick?: () => void;
}

/**
 * Custom Tauri title bar — drag region, app title, window controls.
 *
 * Stripped variant of the heatloss-studio title bar: no auth widget, no
 * feedback button. Phase B/C add back the quick-actions wiring (save / undo /
 * redo / print / settings) once those dialogs exist.
 */
function TitleBar({ onSettingsClick }: TitleBarProps) {
  const [isMaximized, setIsMaximized] = useState(false);
  const [appVersion, setAppVersion] = useState("");
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const appWindowRef = useRef<any>(null);

  const getWindow = useCallback(async () => {
    if (!appWindowRef.current) {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      appWindowRef.current = getCurrentWindow();
    }
    return appWindowRef.current;
  }, []);

  useEffect(() => {
    import("@tauri-apps/api/app")
      .then(({ getVersion }) => getVersion())
      .then(setAppVersion)
      .catch(() => setAppVersion(""));
  }, []);

  const updateMaximizedState = useCallback(async () => {
    try {
      const win = await getWindow();
      setIsMaximized(await win.isMaximized());
    } catch {
      /* not in Tauri */
    }
  }, [getWindow]);

  useEffect(() => {
    updateMaximizedState();
    let cleanup: (() => void) | undefined;
    getWindow()
      .then((win) => win.onResized(() => updateMaximizedState()))
      .then((unlisten: () => void) => {
        cleanup = unlisten;
      })
      .catch(() => {});
    return () => {
      cleanup?.();
    };
  }, [updateMaximizedState, getWindow]);

  const handleMinimize = async () => {
    try {
      (await getWindow()).minimize();
    } catch {
      /* web */
    }
  };
  const handleMaximize = async () => {
    try {
      (await getWindow()).toggleMaximize();
    } catch {
      /* web */
    }
  };
  const handleClose = async () => {
    try {
      (await getWindow()).close();
    } catch {
      /* web */
    }
  };

  const handleDoubleClick = async (e: React.MouseEvent) => {
    if ((e.target as HTMLElement).closest(".titlebar-button")) return;
    try {
      (await getWindow()).toggleMaximize();
    } catch {
      /* web */
    }
  };

  return (
    <div className="titlebar" onDoubleClick={handleDoubleClick}>
      <div className="titlebar-drag" data-tauri-drag-region />

      <div className="titlebar-left">
        <div className="titlebar-icon">
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="var(--theme-accent)"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path d="M3 21h18M5 21V7l8-4v18M19 21V11l-6-4" />
          </svg>
        </div>

        <div className="titlebar-quick-access">
          <button
            className="titlebar-quick-btn"
            title="Voorkeuren"
            aria-label="Voorkeuren"
            tabIndex={-1}
            onClick={onSettingsClick}
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <circle cx="12" cy="12" r="3" />
              <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z" />
            </svg>
          </button>
        </div>
      </div>

      <span className="titlebar-title" data-tauri-drag-region>
        Open Energy Studio
        {appVersion && <span className="titlebar-version">v{appVersion}</span>}
      </span>

      <div className="titlebar-controls">
        {isTauri() && (
          <>
            <button
              className="titlebar-button titlebar-minimize"
              onClick={handleMinimize}
              aria-label="Minimaliseren"
              tabIndex={-1}
            >
              <svg width="10" height="1" viewBox="0 0 10 1">
                <rect width="10" height="1" fill="currentColor" />
              </svg>
            </button>
            <button
              className="titlebar-button titlebar-maximize"
              onClick={handleMaximize}
              aria-label={isMaximized ? "Herstellen" : "Maximaliseren"}
              tabIndex={-1}
            >
              {isMaximized ? (
                <svg width="10" height="10" viewBox="0 0 10 10">
                  <rect
                    x="0.5"
                    y="2.5"
                    width="7"
                    height="7"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="1.2"
                  />
                  <polyline
                    points="2.5 2.5 2.5 0.5 9.5 0.5 9.5 7.5 7.5 7.5"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="1.2"
                  />
                </svg>
              ) : (
                <svg width="10" height="10" viewBox="0 0 10 10">
                  <rect
                    x="0.5"
                    y="0.5"
                    width="9"
                    height="9"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="1.2"
                  />
                </svg>
              )}
            </button>
            <button
              className="titlebar-button titlebar-close"
              onClick={handleClose}
              aria-label="Sluiten"
              tabIndex={-1}
            >
              <svg width="10" height="10" viewBox="0 0 10 10">
                <line x1="0" y1="0" x2="10" y2="10" stroke="currentColor" strokeWidth="1.2" />
                <line x1="10" y1="0" x2="0" y2="10" stroke="currentColor" strokeWidth="1.2" />
              </svg>
            </button>
          </>
        )}
      </div>
    </div>
  );
}

export default TitleBar;
