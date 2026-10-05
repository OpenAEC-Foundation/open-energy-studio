/**
 * Global commands of the app shell. The handlers live in App (they need the
 * document manager, dialogs and file access); every place that offers them —
 * top bar, workflow navigation, tools menu, command palette and the step pages —
 * reads them from this context, so each ribbon action of old stays reachable
 * through exactly one implementation.
 */
import { createContext, useContext, type ReactNode } from 'react';
import type { Route } from '../../core/navigation/routes';
import type { DialogType } from '../../core/energy/types';

export interface ShellActions {
  newProject: () => void;
  openProject: () => void;
  saveProject: () => void;
  saveAsProject: () => void;
  /** "Herberekenen": the indicative engine and a fresh kernel run, then the results. */
  calculate: () => void;
  openDialog: (type: DialogType) => void;
  navigate: (route: Route) => void;
  exportReport: () => void;
  printReport: () => void;
  exportIFC: () => void;
  exportModelIFC: () => void;
  exportUNIEC3: () => void;
  importUNIEC3: () => void;
  exportVABI: () => void;
  importVABI: () => void;
  openSettings: () => void;
  openFeedback: () => void;
  openPalette: () => void;
  toggleInspector: () => void;
  /** The former ribbon "Preview" toggle: Voorbeeld or Eigenschappen in the inspector. */
  togglePreview: () => void;
}

const ShellActionsContext = createContext<ShellActions | null>(null);

export function ShellActionsProvider({ value, children }: { value: ShellActions; children: ReactNode }) {
  return <ShellActionsContext.Provider value={value}>{children}</ShellActionsContext.Provider>;
}

/** The shell commands; null when a component renders outside the app shell (isolated tests). */
export function useShellActions(): ShellActions | null {
  return useContext(ShellActionsContext);
}
