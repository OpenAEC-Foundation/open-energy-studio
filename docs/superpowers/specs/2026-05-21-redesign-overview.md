# Open Energy Studio — Redesign Overview

**Date:** 2026-05-21
**Status:** Approved (user override of brainstorm gate; execute sequentially)
**Author:** Claude
**Branch:** `claude/admiring-poincare-5ec4c0`

## Goal

Rebuild `open-energy-studio` as a credible, reference-grade NTA 8800 / BENG calculator,
modelled after the architecture of `open-heatloss-studio`. The TypeScript calculation
engine in `src/core/` is retired; all calculations move to the Rust crates in the sibling
`crates-warehouse` repository.

Beyond core BENG calculation, the application must:

1. **Verify** its own correctness by importing reference cases (Vabi `.vp` projects from
   `verification-files/Warmteverlies/`) and comparing computed results against expected.
2. **Export** to Vabi (`.vp`) and Uniec project formats so users can roundtrip work into
   the two dominant Dutch commercial calculation tools.

## Reference Projects

| Project | Role |
|---|---|
| `open-heatloss-studio` | Architecture template — Tauri 2 + React 19 + Zustand + React Router + crate-per-domain |
| `crates-warehouse` | Source of truth for NTA 8800 calculation crates (`isso51-core`, 11× `nta8800-*`, `openaec-project-shared`) |
| `OpenAEC-style-book` | Design system; consumed via `@openaec/ui` package + CSS tokens |
| `verification-files/Warmteverlies/` | Vabi `.vp` regression test corpus (~12 projects with reference PDFs) |

## Phase Decomposition

The work is split into five sub-projects. Each has its own design spec and implementation
plan; they are executed sequentially in this order.

### Phase A — Architecture Migration
Replace TypeScript calc engine with crate-backed Tauri commands. Adopt OpenAEC shell,
ribbon, Zustand store, React Router, project-file format from `openaec-project-shared`.
Fix WebView2 bundling per user CLAUDE.md guidance. Add Tauri 2 capabilities.
Result: same user-facing functionality as today, but on the new substrate, ready to grow.

### Phase B — NTA 8800 Input UI
Build the BENG-focused input workflow on the new shell:
ProjectSetup → Zones → Constructions → Systems (tabbed) → Results.
Replace the dialog-heavy modal model with a page-flow plus side-by-side property editing.

### Phase C — Verification Module
New "Verificatie" page. Loads a Vabi `.vp` reference case, runs our calculation against
its inputs, and presents a side-by-side diff against the reference results with
configurable tolerances and pass/fail status per indicator (BENG 1/2/3, energy label,
monthly Q_H/Q_C, primary energy).

### Phase D — Vabi Export
New crate `openaec-vabi-exporter` (and `-importer` for Phase C). Maps
`ProjectV2 + ProjectResult` to Vabi's zipped-XML format. Tauri command
`export_to_vabi(project, path)`. Validated by roundtrip: import verification corpus,
re-export, diff against original.

### Phase E — Uniec Export
New crate `openaec-uniec-exporter`. Maps `ProjectV2` to Uniec's project format. Tauri
command `export_to_uniec(project, path)`. Validated by opening exported files in Uniec
and confirming clean import.

## Cross-Cutting Decisions

- **Crates-warehouse integration**: git submodule at `vendor/crates-warehouse/`. Path
  deps in `src-tauri/Cargo.toml` point there. CI uses `git submodule update --init`.
- **Project file format**: `.oes.json` JSON envelope wrapping `ProjectV2` from
  `openaec-project-shared`. Versioned with `schema_version` field. Auto-migration of
  legacy `.oes` files from current TypeScript engine.
- **State management**: Zustand, not React Context. One central `projectStore` plus
  smaller stores for UI-only state (toasts, modeller tools).
- **Routing**: React Router v7.
- **i18n**: keep all 14 existing languages (Dutch primary).
- **Shell**: `@openaec/ui` for tokens + primitives; custom ribbon (matches heatloss
  studio's pattern).
- **CI**: existing `release.yml` and `live.yml` continue to work; only crate
  checkout step needs adjustment.

## Success Criteria (full redesign)

1. `npm run tauri:dev` boots, displays new shell, lets user create and edit a project.
2. `cargo test` passes in `src-tauri/` and across vendored crates.
3. All 14 i18n strings render; no broken translation keys.
4. WebView2 bootstrap embedded; clean Windows VM install succeeds.
5. Verification page shows green status against ≥80% of `.vp` reference cases
   (within ±2% tolerance on BENG indicators).
6. Vabi roundtrip preserves all input fields; exported `.vp` opens in Vabi without
   errors.
7. Uniec export imports cleanly into Uniec.
8. Release build produces signed Windows installer ≤ 25 MB.

## Out of Scope

- Multi-user / cloud sync.
- IFC import wizard rewrite (initial migration keeps current behaviour; deep IFC work
  postponed).
- Mobile / web-only build (desktop-first).
- Replacing `verification-files` corpus with synthetic cases.
