import { useProjectStore } from "../store/projectStore";
import { PageShell, EmptyCard } from "./PageShell";

export default function ProjectSetup() {
  const project = useProjectStore((s) => s.project);
  const newProject = useProjectStore((s) => s.newProject);

  return (
    <PageShell
      title="Projectgegevens"
      subtitle="Project, opdrachtgever, locatie en bouwfysisch kader (NTA 8800)."
    >
      {!project ? (
        <EmptyCard
          icon={<BuildingIcon />}
          title="Nog geen project"
          description="Maak een nieuw project aan of open een bestaand .oes-bestand via het Bestand-tabblad in het lint."
          action={
            <button
              type="button"
              onClick={() => newProject("Nieuw project")}
              className="rounded bg-primary px-4 py-2 text-sm font-medium text-on-accent transition-colors hover:bg-accent-hover"
            >
              Nieuw project
            </button>
          }
        />
      ) : (
        <PlaceholderForm />
      )}
    </PageShell>
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

function BuildingIcon() {
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
      <path d="M3 21h18M5 21V7l8-4v18M19 21V11l-6-4" />
    </svg>
  );
}
