/**
 * Shared waits for the UI tests. They are wall-clock limits for waiting, not performance
 * budgets: the gate runs Vitest workers next to Rust builds, and tests that pass alone failed
 * on a 1 s findBy or a 60 s test limit under that load. Tests that time the app keep their own
 * budgets (large-project-ui.test.tsx scales its budget with a baseline measured in the same run).
 */

/**
 * How long findBy* and waitFor wait by default (Testing Library's own default is 1 s). A
 * kernel answer alone waits KERNEL_RUN_DEBOUNCE_MS (400 ms) before it starts.
 */
export const UI_WAIT_MS = 5000;

/**
 * Limit per test unless the test sets its own (vitest.config.ts). Heavy single-page tests such as
 * the form coverage sweep take seconds alone and overran 20 s under heavy load.
 */
export const TEST_TIMEOUT_MS = 60_000;

/** Limit for a whole workflow test that clicks and types through several pages. */
export const SLOW_UI_TEST_MS = 180_000;
