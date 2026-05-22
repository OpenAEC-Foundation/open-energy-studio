import { PageShell } from "./PageShell";

const SYSTEMS = [
  { key: "heating", label: "Verwarming", desc: "Opwekker, distributie, afgifte, regeling" },
  { key: "cooling", label: "Koeling", desc: "Compressie / absorptie / vrije koeling" },
  { key: "dhw", label: "Tapwater", desc: "Opwekker + douche-wtw" },
  { key: "ventilation", label: "Ventilatie", desc: "Systeem A–E, WTW" },
  { key: "lighting", label: "Verlichting", desc: "Pn (W/m²), regelfactoren" },
  { key: "pv", label: "PV", desc: "Vermogen, hellingshoek, oriëntatie" },
  { key: "humidity", label: "Bevochtiging", desc: "Bevochtiging / ontvochtiging" },
  { key: "automation", label: "Automatisering", desc: "BACS-klasse per dienst" },
] as const;

export default function Systems() {
  return (
    <PageShell
      title="Installaties"
      subtitle="Acht subsystemen van NTA 8800: verwarming · koeling · tapwater · ventilatie · verlichting · PV · bevochtiging · automatisering."
    >
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
        {SYSTEMS.map((s) => (
          <div
            key={s.key}
            className="rounded-lg border border-[var(--theme-border-subtle)] bg-[var(--theme-bg-lighter)] p-5 transition-colors hover:border-accent"
          >
            <h3 className="font-heading text-base font-semibold text-on-surface">{s.label}</h3>
            <p className="mt-1 text-sm text-on-surface-secondary">{s.desc}</p>
            <p className="mt-3 text-xs text-on-surface-muted">Phase B-5 ↳ formulieren</p>
          </div>
        ))}
      </div>
    </PageShell>
  );
}
