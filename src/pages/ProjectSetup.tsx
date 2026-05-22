import { open as openDialog } from "@tauri-apps/plugin-dialog";

import { BrandLogo, BrandSymbol } from "../components/BrandSymbol";
import { isTauri } from "../lib/backend";
import { useProjectStore } from "../store/projectStore";
import { PageShell } from "./PageShell";

const FILTERS = [
  { name: "Open Energy Studio project", extensions: ["oes.json", "oes", "json"] },
];

export default function ProjectSetup() {
  const project = useProjectStore((s) => s.project);
  const newProject = useProjectStore((s) => s.newProject);
  const loadProject = useProjectStore((s) => s.loadProject);

  const handleOpen = async () => {
    if (!isTauri()) return;
    const picked = await openDialog({ multiple: false, directory: false, filters: FILTERS });
    if (typeof picked === "string") await loadProject(picked);
  };

  return (
    <PageShell
      title="Projectgegevens"
      subtitle="Project, opdrachtgever, locatie en bouwfysisch kader (NTA 8800)."
    >
      {!project ? <WelcomeHero onNew={() => newProject("Nieuw project")} onOpen={handleOpen} /> : <PlaceholderForm />}
    </PageShell>
  );
}

function WelcomeHero({ onNew, onOpen }: { onNew: () => void; onOpen: () => void }) {
  return (
    <div className="relative overflow-hidden rounded-xl border border-[var(--theme-border-subtle)] bg-gradient-to-br from-[var(--theme-bg-lighter)] to-[var(--theme-bg)] p-10">
      {/* Decorative background symbol */}
      <div className="pointer-events-none absolute -right-12 -top-12 opacity-[0.07]">
        <BrandSymbol size={320} color="#D97706" accent="#F59E0B" />
      </div>

      <div className="relative grid grid-cols-1 gap-10 md:grid-cols-[1fr_320px]">
        <div className="flex flex-col justify-center">
          <BrandLogo height={64} />
          <h2 className="mt-6 font-heading text-3xl font-semibold text-on-surface">
            Open Energy Studio
          </h2>
          <p className="mt-2 text-sm text-on-surface-secondary">
            BENG-berekeningen volgens NTA 8800. Reken-engine in Rust (isso51-core
            + nta8800-* crates), verificatie tegen Vabi-referentiecases, export
            naar Vabi en Uniec.
          </p>

          <div className="mt-8 flex flex-wrap items-center gap-3">
            <button
              type="button"
              onClick={onNew}
              className="rounded-md bg-primary px-5 py-2.5 text-sm font-medium text-on-accent shadow-sm transition-colors hover:bg-accent-hover"
            >
              Nieuw project
            </button>
            <button
              type="button"
              onClick={onOpen}
              className="rounded-md border border-[var(--theme-border)] bg-transparent px-5 py-2.5 text-sm font-medium text-on-surface transition-colors hover:bg-[var(--theme-hover)] hover:text-accent"
            >
              Project openen…
            </button>
          </div>

          <p className="mt-6 font-mono text-2xs uppercase tracking-widest text-on-surface-muted">
            NTA 8800 · BENG · ISSO 51 · OpenAEC Foundation
          </p>
        </div>

        <aside className="hidden flex-col gap-3 self-center md:flex">
          <FeatureBadge title="Volledige NTA 8800" body="Transmissie, ventilatie, verwarming, koeling, tapwater, verlichting, PV, bevochtiging, automatisering." />
          <FeatureBadge title="Verificatie" body="Side-by-side diff tegen Vabi referentie‑cases met instelbare tolerantie." />
          <FeatureBadge title="Export" body="Vabi (.vp) en Uniec 3 (.unx) — round-trip getest in CI." />
        </aside>
      </div>
    </div>
  );
}

function FeatureBadge({ title, body }: { title: string; body: string }) {
  return (
    <div className="rounded-lg border border-[var(--theme-border-subtle)] bg-[var(--theme-bg)] p-4">
      <p className="font-heading text-sm font-semibold text-accent">{title}</p>
      <p className="mt-1 text-xs leading-relaxed text-on-surface-secondary">{body}</p>
    </div>
  );
}

function PlaceholderForm() {
  return (
    <div className="space-y-6">
      <FormSection title="Identificatie">
        <Field label="Projectnaam" placeholder="bv. Woning Lange Akker 12" />
        <Field label="Projectnummer" placeholder="bv. 2026-014" />
        <Field label="Opdrachtgever" placeholder="bv. Familie De Vries" />
      </FormSection>

      <FormSection title="Locatie">
        <Field label="Adres" placeholder="Straat + huisnummer" />
        <Field label="Postcode" placeholder="0000 AA" />
        <Field label="Plaats" placeholder="bv. Dordrecht" />
      </FormSection>

      <FormSection title="Gebouw">
        <Field label="Gebruiksfunctie" placeholder="Woonfunctie" />
        <Field label="Bouwjaar" placeholder="bv. 1985" />
        <Field label="Status" placeholder="Nieuwbouw / Bestaand / Renovatie" />
      </FormSection>

      <p className="text-xs text-on-surface-muted">
        Form wiring naar <code className="font-mono">project.shared</code> volgt in
        Phase B-2. Voor nu: structuur is in plaats, ribbon en navigatie werken end-to-end.
      </p>
    </div>
  );
}

function FormSection({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="rounded-lg border border-[var(--theme-border-subtle)] bg-[var(--theme-bg-lighter)] p-6">
      <h2 className="mb-4 font-heading text-base font-semibold text-on-surface">{title}</h2>
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">{children}</div>
    </section>
  );
}

function Field({ label, placeholder }: { label: string; placeholder: string }) {
  return (
    <label className="flex flex-col gap-1.5">
      <span className="text-xs font-medium uppercase tracking-wider text-on-surface-muted">
        {label}
      </span>
      <input
        type="text"
        placeholder={placeholder}
        className="rounded border border-[var(--theme-border)] bg-[var(--theme-bg)] px-3 py-2 text-sm text-on-surface placeholder:text-on-surface-muted focus:border-accent focus:outline-none"
      />
    </label>
  );
}

