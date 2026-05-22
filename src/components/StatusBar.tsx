import { useProjectStore } from "../store/projectStore";
import "./StatusBar.css";

/**
 * Bottom status bar. Phase A scope: project name + dirty/saved state +
 * calculation status. Phase B adds zone/construction counts and progress
 * spinners.
 */
export default function StatusBar() {
  const project = useProjectStore((s) => s.project);
  const filePath = useProjectStore((s) => s.filePath);
  const isDirty = useProjectStore((s) => s.isDirty);
  const isCalculating = useProjectStore((s) => s.isCalculating);
  const isSaving = useProjectStore((s) => s.isSaving);

  const projectName = projectDisplayName(project);

  return (
    <div className="statusbar">
      <div className="statusbar-section">
        <span className="statusbar-label">Project:</span>
        <span>{projectName ?? "Geen project geladen"}</span>
      </div>
      {filePath && (
        <div className="statusbar-section">
          <span className="statusbar-label">Bestand:</span>
          <span title={filePath}>{shortenPath(filePath)}</span>
        </div>
      )}
      <div className="statusbar-section">
        {isCalculating ? (
          <span style={{ color: "var(--theme-accent)" }}>● Rekenen…</span>
        ) : isSaving ? (
          <span style={{ color: "var(--theme-accent)" }}>● Opslaan…</span>
        ) : isDirty ? (
          <span style={{ color: "#e9c46a" }}>● Niet opgeslagen</span>
        ) : projectName ? (
          <span>● Opgeslagen</span>
        ) : null}
      </div>
      <div className="statusbar-spacer" />
      <div className="statusbar-section">NTA 8800 · BENG</div>
    </div>
  );
}

function projectDisplayName(project: unknown): string | null {
  if (
    project &&
    typeof project === "object" &&
    "shared" in project &&
    (project as { shared?: { name?: unknown } }).shared
  ) {
    const name = (project as { shared: { name?: unknown } }).shared.name;
    if (typeof name === "string" && name.length > 0) return name;
  }
  return null;
}

function shortenPath(path: string): string {
  const parts = path.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] ?? path;
}
