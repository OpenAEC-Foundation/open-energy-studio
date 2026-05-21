# Phase D — Vabi Export

**Date:** 2026-05-21
**Status:** Ready for implementation plan
**Dependencies:** Phase A; Phase C completed (we reuse the .vp parser learnings)

## Purpose

Export a `ProjectV2` + `ProjectResult` to Vabi's `.vp` project format so users can
open Open Energy Studio projects in Vabi Elements for further work, validation, or
official submission.

Round-trip is the success criterion: a verification corpus `.vp` imported (Phase C),
re-exported (Phase D), then re-imported into Vabi cleanly with results within
verification tolerance.

## Scope

In:
- `.vp` writer producing files Vabi can open
- Field-by-field mapping ProjectV2 ↔ Vabi XML
- Tauri command `export_to_vabi(project, result, path)`
- "Export to Vabi" button in Results page and File menu

Out:
- Vabi-specific UI niceties (we don't replicate Vabi's UX choices)
- Vabi cloud sync
- Two-way live linking

## Crate: `openaec-vabi-exporter`

Sibling to `openaec-vabi-importer` (Phase C). Both share the same internal
representation:

```rust
// Internal model — mirrors Vabi XML structure 1:1
pub(crate) struct VabiProject {
    pub identification: VabiIdentification,
    pub building: VabiBuilding,
    pub zones: Vec<VabiZone>,
    pub constructions: Vec<VabiConstruction>,
    pub systems: VabiSystems,
    pub results: Option<VabiResults>,
}

pub fn export_vp(
    project: &ProjectV2,
    result: Option<&ProjectResult>,
    out_path: &Path,
) -> Result<(), ExportError>;
```

`export_vp` does:
1. Map `ProjectV2 → VabiProject`
2. Serialize `VabiProject → XML` (one file per logical section, mirroring Vabi's
   layout)
3. Embed the optional `ProjectResult` as a Vabi-compatible results XML so Vabi
   shows our calculated values until the user recomputes.
4. ZIP the XML files into `.vp`.

## Field Mapping

Documented in `openaec-vabi-exporter/MAPPING.md`. The mapping is derived from the
.vp samples already in `verification-files/Warmteverlies/` and refined iteratively.

High-level mapping:

| ProjectV2 path | Vabi XML element |
|---|---|
| `shared.identification.project_name` | `/Project/Naam` |
| `shared.identification.client` | `/Project/Opdrachtgever` |
| `shared.location.postcode` | `/Project/Locatie/Postcode` |
| `shared.building.usage_function` | `/Project/Gebouw/Gebruiksfunctie` |
| `geometry.spaces[].name` | `/Project/Rekenzones/Rekenzone/@naam` |
| `geometry.spaces[].floor_area` | `/Project/Rekenzones/Rekenzone/Ag` |
| `geometry.constructions[]` (walls) | `/Project/Constructies/Wand[]` |
| `geometry.constructions[]` (roofs) | `/Project/Constructies/Dak[]` |
| `geometry.constructions[]` (floors) | `/Project/Constructies/Vloer[]` |
| `geometry.openings[]` (windows) | `/Project/Constructies/Raam[]` |
| `calcs.systems.heating.generator` | `/Project/Installaties/Verwarming/Opwekker` |
| ... | ... |

Where a Vabi field has no analogue in ProjectV2, the exporter emits a sensible
default (logged as an `ExportNote` for the UI to surface). Where ProjectV2 has
richer detail than Vabi accepts, we lossy-collapse (logged similarly).

## Tauri Command

```rust
// src-tauri/src/commands/export.rs
#[tauri::command]
fn export_to_vabi(
    project: ProjectV2,
    result: Option<ProjectResult>,
    path: String,
) -> Result<Vec<ExportNote>, AppError>;
```

`ExportNote` is `{ severity: Info|Warning, field: String, message: String }`.
The UI shows a toast on success with a "View details" affordance.

## Frontend UI

- **Results page** gets an "Export to Vabi" button next to "Export PDF".
- **File menu** gets `File → Export → Vabi (.vp)…`.
- After export, a non-blocking dialog summarises any `ExportNote` items.

## Round-trip Validation

Built into CI:

```sh
openaec-verify-cli --corpus ./verification-files/Warmteverlies \
    --roundtrip-vabi \
    --tolerance 2.0
```

For each case in the corpus:
1. Import .vp → ProjectV2 + ReferenceResult
2. Calculate → OurResult
3. Export ProjectV2 + OurResult → .vp (different filename)
4. Re-import the new .vp → ProjectV2' + ReferenceResult'
5. Diff ProjectV2 vs ProjectV2' (field-by-field equality) — fail if any
   data-bearing field differs
6. Diff OurResult vs ReferenceResult' — fail if outside tolerance

This catches lossy round-trips. Test corpus must pass 100% before Phase D is
declared done.

## Testing

- Unit tests on `VabiProject ↔ XML` round-trip via `quick-xml::DeError`/Serializer
- Property tests on `ProjectV2 → VabiProject → ProjectV2` for randomly generated
  ProjectV2 (proptest crate)
- Integration tests against the corpus

## Risks

| Risk | Mitigation |
|---|---|
| Vabi rejects exported .vp due to schema validation differences | Diff against the in-corpus .vp files at the XML level; fix elements one at a time |
| Vabi adds fields we don't yet model | `MAPPING.md` tracks "Vabi-only" fields; we preserve them by carrying through unknown XML on import and re-emitting on export ("passthrough" mode) |
| .vp is partly binary (PDF embedded) | We keep the imported PDF blob and re-embed; we don't try to regenerate it |
| Licensing concern around emitting Vabi-compatible files | Vabi format is closed but used in legal energy-label submissions; our output is provided as-is with a disclaimer |

## Success Criteria

1. All 12 corpus .vp files round-trip with ProjectV2 equality.
2. All 12 corpus .vp files re-imported into Vabi (manual smoke test on a Vabi
   install) without error dialogs.
3. CI gates on round-trip success.
4. Mapping documentation complete and committed.
