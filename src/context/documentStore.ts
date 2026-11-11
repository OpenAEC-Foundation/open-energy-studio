/**
 * The open documents for code outside React: the assistant tools of the site (core/site/siteAppTools.ts).
 *
 * EnergyProvider registers its document state and dispatch here. `dispatch` returns after React committed the
 * action, with the state the user now sees; there is no second copy of the reducer.
 */
import type { DocumentManagerAction, DocumentManagerState } from './EnergyContext';

export interface DocumentStore {
  getState(): DocumentManagerState;
  dispatch(action: DocumentManagerAction): DocumentManagerState;
}

let current: DocumentStore | null = null;

export function setDocumentStore(store: DocumentStore | null): void {
  current = store;
}

/** The registered store, or null before the provider mounted. */
export function getDocumentStore(): DocumentStore | null {
  return current;
}
