import { PageShell, EmptyCard } from "./PageShell";

export default function Verificatie() {
  return (
    <PageShell
      title="Verificatie"
      subtitle="Vergelijk berekeningen tegen Vabi referentiecases uit verification-files."
    >
      <EmptyCard
        icon={<CheckIcon />}
        title="Referentiecorpus nog niet ingeladen"
        description="Phase C voegt de .vp-importer en side-by-side diff toe. Pad: verification-files/Warmteverlies/*.vp"
      />
    </PageShell>
  );
}

function CheckIcon() {
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
      <path d="M22 11.08V12a10 10 0 11-5.93-9.14" />
      <polyline points="22 4 12 14.01 9 11.01" />
    </svg>
  );
}
