import { PageShell, EmptyCard } from "./PageShell";

export default function Constructies() {
  return (
    <PageShell
      title="Constructies"
      subtitle="Wanden, daken, vloeren, ramen, deuren en koudebruggen — incl. U-/Rc-waarden."
    >
      <EmptyCard
        icon={<LayersIcon />}
        title="Constructiebibliotheek leeg"
        description="Voeg constructies toe en koppel ze aan rekenzones met begrenzingstype (buiten/onverwarmd/aangrenzend/grond)."
        action={
          <button
            type="button"
            disabled
            className="rounded bg-primary px-4 py-2 text-sm font-medium text-on-accent opacity-60"
          >
            Constructie toevoegen (Phase B-4)
          </button>
        }
      />
    </PageShell>
  );
}

function LayersIcon() {
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
      <polygon points="12 2 2 7 12 12 22 7 12 2" />
      <polyline points="2 17 12 22 22 17" />
      <polyline points="2 12 12 17 22 12" />
    </svg>
  );
}
