// Types for scripts/nta-gate-summary.mjs, used by its Vitest test.
export interface RustCounts {
  binaries: number;
  passed: number;
  failed: number;
  ignored: number;
}
export interface VitestCounts {
  passed: number;
  failed: number;
  skipped: number;
  total: number;
}
export interface GateStep {
  name: string;
  seconds: number | null;
  rust: RustCounts | null;
  vitest: { files?: VitestCounts; tests?: VitestCounts } | null;
  failed?: boolean;
}
export interface GateLogSummary {
  steps: GateStep[];
  totalSeconds: number | null;
  passed: boolean;
  status: 'passed' | 'failed' | 'incomplete';
  missingSteps: string[];
}
export interface ReferenceSummary {
  verdict: string | null;
  comparedCases: number | null;
  pendingExpectationCases: number | null;
  totalCases: number;
  kernelVersion: string | null;
  suites: { suite: string; total: number; compared: number; passed: number; pending: number; other: number }[];
}
export interface GateSummary {
  gate: GateLogSummary;
  reference: ReferenceSummary | null;
}
export function summariseGateLog(logText: string): GateLogSummary;
export function summariseReferenceReport(report: unknown): ReferenceSummary;
export function renderGateSummaryMarkdown(summary: GateSummary | null): string;
export function renderGateSummaryEnglish(summary: GateSummary | null): string;
export const REQUIRED_STEPS: string[];
export function gateSummaryProblems(gate: GateLogSummary, options?: { requireAllSteps?: boolean }): string[];
