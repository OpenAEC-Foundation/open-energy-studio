import { PageShell, EmptyCard } from "./PageShell";

export default function Zones() {
  return (
    <PageShell
      title="Rekenzones"
      subtitle="Verdeel het gebouw in rekenzones en energiefunctieruimtes (NTA 8800 §6)."
    >
      <EmptyCard
        icon={<GridIcon />}
        title="Nog geen rekenzones"
        description="Voor woningen volstaat vaak één zone voor het hele gebouw. Voor utiliteit verdeel je per gebruiksfunctie."
        action={
          <button
            type="button"
            disabled
            className="rounded bg-primary px-4 py-2 text-sm font-medium text-on-accent opacity-60"
          >
            Zone toevoegen (Phase B-3)
          </button>
        }
      />
    </PageShell>
  );
}

function GridIcon() {
  return (
    <svg
      width="48"
      height="48"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <rect x="3" y="3" width="7" height="7" />
      <rect x="14" y="3" width="7" height="7" />
      <rect x="3" y="14" width="7" height="7" />
      <rect x="14" y="14" width="7" height="7" />
    </svg>
  );
}
