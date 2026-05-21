/**
 * Results page — Phase A wiring.
 *
 * Reads `result` and status from the project store and renders a minimal
 * summary. Phase B replaces this with BENG indicator cards, monthly stack
 * charts, and PDF export. For Phase A we show whatever isso51_core returns
 * plus a Recalculate button that proves the round-trip works.
 */
import { useEffect } from 'react';

import { useProjectStore } from '../store/projectStore';

export default function Results() {
  const project = useProjectStore((s) => s.project);
  const result = useProjectStore((s) => s.result);
  const isCalculating = useProjectStore((s) => s.isCalculating);
  const error = useProjectStore((s) => s.error);
  const calculate = useProjectStore((s) => s.calculate);

  // Auto-calc when project is present and we don't have a result yet.
  useEffect(() => {
    if (project && !result && !isCalculating && !error) {
      void calculate();
    }
  }, [project, result, isCalculating, error, calculate]);

  if (!project) {
    return (
      <div>
        <h2 style={titleStyle}>Resultaten</h2>
        <p style={mutedStyle}>
          Nog geen project geladen. Ga naar <strong>Project</strong> om te
          beginnen of laad een bestaand .oes-bestand.
        </p>
      </div>
    );
  }

  return (
    <div style={containerStyle}>
      <header>
        <h2 style={titleStyle}>Resultaten</h2>
        <p style={mutedStyle}>
          Phase A toont de raw warmteverlies-uitvoer van isso51_core. De volledige
          BENG-aggregatie volgt in Phase B.
        </p>
      </header>

      <button
        type="button"
        style={buttonStyle}
        onClick={() => calculate()}
        disabled={isCalculating}
      >
        {isCalculating ? 'Rekenen…' : 'Herbereken'}
      </button>

      {error ? (
        <div style={errorStyle}>
          <strong>Fout ({error.kind}):</strong> {error.message}
        </div>
      ) : null}

      {result ? (
        <pre style={resultStyle}>{JSON.stringify(result, null, 2)}</pre>
      ) : (
        <p style={mutedStyle}>{isCalculating ? 'Bezig met rekenen…' : 'Geen resultaat.'}</p>
      )}
    </div>
  );
}

const containerStyle: React.CSSProperties = {
  display: 'flex',
  flexDirection: 'column',
  gap: 16,
  maxWidth: 960,
};
const titleStyle: React.CSSProperties = { margin: 0, fontSize: 22, fontWeight: 600 };
const mutedStyle: React.CSSProperties = {
  margin: 0,
  fontSize: 13,
  color: 'var(--oes-fg-muted, #8a93a3)',
};
const buttonStyle: React.CSSProperties = {
  alignSelf: 'flex-start',
  padding: '8px 16px',
  background: 'var(--oes-accent, #2c6dff)',
  color: '#fff',
  border: 'none',
  borderRadius: 6,
  cursor: 'pointer',
  fontSize: 14,
};
const errorStyle: React.CSSProperties = {
  padding: '12px 16px',
  background: '#3b1f24',
  border: '1px solid #6e2b35',
  color: '#fbb',
  borderRadius: 6,
  fontSize: 13,
};
const resultStyle: React.CSSProperties = {
  padding: '12px 16px',
  background: 'var(--oes-bg-elev, #14171d)',
  border: '1px solid var(--oes-border, #232830)',
  borderRadius: 6,
  overflow: 'auto',
  fontSize: 12,
  lineHeight: 1.5,
};
