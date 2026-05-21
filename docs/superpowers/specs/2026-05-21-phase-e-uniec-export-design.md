# Phase E — Uniec Export

**Date:** 2026-05-21
**Status:** Ready for implementation plan
**Dependencies:** Phase A; Phases C/D for parser/format infrastructure patterns

## Purpose

Export a `ProjectV2` to Uniec's project format so users can open Open Energy Studio
projects in Uniec 3 (BCRG-attested NTA 8800 implementation) for officially
submittable BENG calculations.

Where Vabi is the commercial/architectural mainstay (Phase D), Uniec is the
*authoritative* reference implementation: many Dutch BENG submissions are
calculated and signed off in Uniec. Producing Uniec-compatible output gives Open
Energy Studio maximum credibility.

## Scope

In:
- Uniec project file writer (format details TBD via sample acquisition)
- ProjectV2 → Uniec mapping
- Tauri command `export_to_uniec(project, path)`
- "Export to Uniec" button in Results page and File menu

Out:
- Uniec project import (one-way only in this phase; bidirectional may be a later
  phase)
- Uniec result re-import for verification (would require parser; defer)

## Open Question: File Format

Uniec uses a closed file format with two main variants depending on version:
- Uniec 3.x: `.unx` (XML-ish container, similar to Vabi)
- Uniec 4.x (planned): JSON-based

We will target the **current Uniec 3.x `.unx` format** in Phase E v1. The format
is not publicly documented, so the first implementation milestone is **format
reverse-engineering**:

1. Acquire ≥5 `.unx` samples covering the range of NTA 8800 inputs (woning,
   utiliteit, multi-zone, PV, HP, district heating). Coordinate with the user
   to drop samples into `verification-files/Uniec/`.
2. Document the schema in `openaec-uniec-exporter/FORMAT.md`.
3. Build the mapping in `openaec-uniec-exporter/MAPPING.md`.

Until samples are available, the spec lists the expected commands and
infrastructure; the mapping detail is filled in during implementation.

## Crate: `openaec-uniec-exporter`

```rust
pub fn export_unx(
    project: &ProjectV2,
    out_path: &Path,
) -> Result<Vec<ExportNote>, ExportError>;
```

Internal model `UniecProject` mirrors Uniec's XML structure. Serialization via
`quick-xml`.

## Tauri Command

```rust
// src-tauri/src/commands/export.rs (same file as Phase D)
#[tauri::command]
fn export_to_uniec(
    project: ProjectV2,
    path: String,
) -> Result<Vec<ExportNote>, AppError>;
```

Note: unlike Vabi (Phase D), we do **not** embed calculated results in the
exported file. Uniec is expected to recompute on open (this is the whole point —
to verify against the BCRG-attested engine).

## Frontend UI

- **Results page** gets an "Export to Uniec" button.
- **File menu** gets `File → Export → Uniec (.unx)…`.
- Export-time export notes shown identically to Phase D.

## Field Mapping

Will be documented in `openaec-uniec-exporter/MAPPING.md`. Initial sketch:

| ProjectV2 path | Uniec .unx element (provisional) |
|---|---|
| `shared.identification.*` | `/Project/Header/*` |
| `shared.building.usage_function` | `/Project/Building/UsageFunction` |
| `geometry.spaces[]` | `/Project/Zones/Zone[]` |
| `geometry.constructions[]` | `/Project/Constructions/*` |
| `calcs.systems.*` | `/Project/Systems/*` |

(Exact paths fixed during sample analysis.)

## Validation Strategy

Without an importer, we validate by:
1. Opening exported files in Uniec on a real install (manual QA per release)
2. Diffing exported XML against the original Uniec-saved sample for the same
   logical project (XML normalization first)
3. Comparing Uniec-computed BENG indicators (after re-opening) against our own —
   should match within tolerance

## Testing

- Unit tests on `UniecProject → XML` round-trip
- Schema-conformance tests once we have a clear schema definition
- Manual QA gate per release

## Risks

| Risk | Mitigation |
|---|---|
| No samples available | Block Phase E start until ≥5 samples acquired; do not waste effort speculating |
| Format changes in Uniec 3.x point releases | Test against multiple Uniec versions; document required version in app |
| Uniec rejects export with cryptic errors | Test driven by Uniec-provided sample diffs; build a "minimal valid .unx" first then add fields |
| Some ProjectV2 inputs have no Uniec equivalent (or vice versa) | Document gaps in MAPPING.md; lossy collapse with ExportNote warning |
| Uniec format covered by commercial restrictions | We don't redistribute Uniec content; we write compatible output. Legal review if uncertain. |

## Success Criteria

1. Exported `.unx` opens cleanly in Uniec 3 on Windows.
2. Uniec-recomputed BENG indicators match ours within ±2% on all corpus cases.
3. Mapping documentation complete and committed.
4. Manual QA checklist defined in `docs/qa/uniec-export.md` and executed for at
   least one release.
