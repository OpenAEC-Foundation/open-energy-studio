import { useEffect } from "react";

import { useProjectStore } from "../store/projectStore";
import { PageShell, EmptyCard } from "./PageShell";

export default function Results() {
  const project = useProjectStore((s) => s.project);
  const result = useProjectStore((s) => s.result);
  const isCalculating = useProjectStore((s) => s.isCalculating);
  const error = useProjectStore((s) => s.error);
  const calculate = useProjectStore((s) => s.calculate);

  useEffect(() => {
    if (project && !result && !isCalculating && !error) {
      void calculate();
    }
  }, [project, result, isCalculating, error, calculate]);

  return (
    <PageShell
      title="Resultaten"
      subtitle="BENG 1/2/3, energielabel, maandverdeling, primair-energiebreakdown."
      actions={
        <button
          type="button"
          onClick={() => calculate()}
          disabled={!project || isCalculating}
          className="rounded bg-primary px-4 py-2 text-sm font-medium text-on-accent transition-colors hover:bg-accent-hover disabled:opacity-60"
        >
          {isCalculating ? "Rekenen…" : "Herbereken"}
        </button>
      }
    >
      {!project ? (
        <EmptyCard
          icon={<ChartIcon />}
          title="Nog geen project"
          description="Maak eerst een project aan via het Bestand-tabblad."
        />
      ) : !result ? (
        <EmptyCard
          icon={<ChartIcon />}
          title={isCalculating ? "Rekenen…" : "Nog geen resultaat"}
          description="Klik op Herbereken om de berekening uit te voeren."
        />
      ) : (
        <div className="space-y-6">
          <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
            <div className="metric-card">
              <p className="metric-card-label">BENG 1</p>
              <p className="metric-card-value">—</p>
              <p className="metric-card-label">kWh/m²·jr</p>
            </div>
            <div className="metric-card">
              <p className="metric-card-label">BENG 2</p>
              <p className="metric-card-value">—</p>
              <p className="metric-card-label">kWh/m²·jr</p>
            </div>
            <div className="metric-card">
              <p className="metric-card-label">BENG 3</p>
              <p className="metric-card-value">—</p>
              <p className="metric-card-label">% hernieuwbaar</p>
            </div>
          </div>
          <section className="rounded-lg border border-[var(--theme-border-subtle)] bg-[var(--theme-bg-lighter)] p-5">
            <h3 className="mb-3 font-heading text-sm font-semibold text-on-surface">
              Ruwe isso51_core uitvoer
            </h3>
            <pre className="overflow-auto rounded bg-[var(--theme-bg)] p-4 font-mono text-xs leading-relaxed text-on-surface-secondary">
              {JSON.stringify(result, null, 2)}
            </pre>
            <p className="mt-3 text-xs text-on-surface-muted">
              BENG-aggregatie via de nta8800-* crates komt in Phase B-6.
            </p>
          </section>
        </div>
      )}
    </PageShell>
  );
}

function ChartIcon() {
  return (
    <svg
      width="48"
      height="48"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <line x1="18" y1="20" x2="18" y2="10" />
      <line x1="12" y1="20" x2="12" y2="4" />
      <line x1="6" y1="20" x2="6" y2="14" />
    </svg>
  );
}
