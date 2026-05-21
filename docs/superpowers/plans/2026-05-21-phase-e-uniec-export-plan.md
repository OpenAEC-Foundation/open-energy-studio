# Phase E Implementation Plan — Uniec (.unx) Export

**Date:** 2026-05-21
**Spec:** `docs/superpowers/specs/2026-05-21-phase-e-uniec-export-design.md`
**Dependencies:** Phase A; reuses infrastructure patterns from Phases C/D.

## Pre-step — sample acquisition

**Blocker:** Phase E execution waits until ≥5 representative Uniec `.unx`
samples land in `verification-files/Uniec/`. Coordinate with the user to
gather: 1 simple woning, 1 multi-zone woning, 1 utiliteit, 1 with PV, 1
with heat pump.

## Steps

### E.1 — Format reverse-engineering
Decompress samples, classify XML elements, document the schema in
`vendor/crates-warehouse/openaec-uniec-exporter/FORMAT.md`. Coordinate with
the user where ambiguity arises.

### E.2 — `openaec-uniec-exporter` crate
`export_unx(&ProjectV2, &Path) -> Result<Vec<ExportNote>>`. Internal
`UniecProject` model derived from the schema doc. Serialization via
`quick-xml`.

### E.3 — Tauri command + UI
`export_to_uniec` command, "Export → Uniec (.unx)…" entry. Same
ExportNote dialog pattern as Vabi.

### E.4 — Manual QA
`docs/qa/uniec-export.md`. Open every exported sample in Uniec, confirm
clean import, compare Uniec-recalculated BENG against ours (≤2% diff).

## Risks
- No samples → Phase E does not start. Surface this immediately.
- Uniec format may change between point releases; lock testing version.
- Legal/commercial restrictions: confirm we may emit compatible files. If
  not, ship as a documented but optional feature.
