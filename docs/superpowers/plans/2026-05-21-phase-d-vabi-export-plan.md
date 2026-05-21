# Phase D Implementation Plan — Vabi (.vp) Export

**Date:** 2026-05-21
**Spec:** `docs/superpowers/specs/2026-05-21-phase-d-vabi-export-design.md`
**Dependencies:** Phase C (the importer's schema knowledge is reused).

## Steps

### D.1 — `openaec-vabi-exporter` crate
Shares `VabiProject` model with the importer (move it to a `vabi-common`
module if needed). `export_vp(&ProjectV2, Option<&OesProjectResult>,
&Path) -> Result<Vec<ExportNote>>`.

### D.2 — Mapping table (`MAPPING.md`)
ProjectV2 ↔ Vabi XML mapping documented field-by-field. Generated where
possible from JSON Schema annotations.

### D.3 — Tauri command + UI
`export_to_vabi` Tauri command. "Export → Vabi (.vp)…" entry in File menu
and a button on the Results page. ExportNote summary dialog after success.

### D.4 — Round-trip CI gate
Add `openaec-verify-cli --roundtrip-vabi` mode. Each corpus case:
import → calc → export → re-import → assert equality. Failures block CI.

### D.5 — Manual Vabi QA
Document a manual checklist (`docs/qa/vabi-export.md`) — open three
exported files in Vabi, confirm parity. Run per release.

## Risks
- Lossy fields: documented in `MAPPING.md` + reported as ExportNote.
- Vabi version drift: pin to a tested Vabi build in docs; bump per release.
