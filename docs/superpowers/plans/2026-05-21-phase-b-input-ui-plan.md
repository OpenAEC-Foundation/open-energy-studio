# Phase B Implementation Plan — NTA 8800 Input UI

**Date:** 2026-05-21
**Spec:** `docs/superpowers/specs/2026-05-21-phase-b-nta8800-input-ui-design.md`

## Approach

Six page-level workstreams plus shared components. Each page is a separate
commit; pages can be built in parallel because they read independent slices
of `ProjectV2`. After each page lands, integration test: load a fixture
project, open the page, mutate a field, recalc, see the result update.

## Steps

### B.1 — Project file dialogs + ProjectV2 bootstrap
Wire `loadProject` / `saveProject` to native file dialogs. Wire the
"New project" action to bootstrap an empty `ProjectV2`. AppShell gains a
ribbon-lite header (Open / Save / Save As / New).
Verification: `npm run tauri:dev` boots, "New" creates a project, Results
auto-calcs (best-effort), Save round-trips the envelope.

### B.2 — ProjectSetup form (`/project`)
Real form on `project.shared`: identification, location (postcode +
auto-lat/lon stubbed at NL center for now), building usage function,
build year, status. Field-level validation via Zod schemas mirroring Rust.

### B.3 — Zones master/detail (`/zones`)
Sidebar list of zones, central editor pane. Per zone: name, floor area,
height, usage function. CRUD via `patchProject`.

### B.4 — Constructions catalogue (`/constructions`)
Two-pane catalogue editor. Construction types (Wall/Roof/Floor/Window/Door/
ThermalBridge) with U-value or layer-based input. Assignment matrix to
zones with boundary type.

### B.5 — Systems tabbed page (`/systems/*`)
Tab strip routing under `/systems/`: Heating, Cooling, DHW, Ventilation,
Lighting, PV, Humidity, Automation. Each tab is its own subcomponent that
reads/writes `project.calcs.*`. Shared `FormSection` and `SelectField`.

### B.6 — Results page
Replace JSON dump with BENG indicator cards, energy label, monthly stacked
bar chart, primary-energy breakdown table. Re-uses the heat loss result;
BENG aggregation lives in a new `bengAggregator` Rust module that calls
the per-domain `nta8800-*` crates and produces an `OesProjectResult`.

### B.7 — Migration mapper from legacy `.oes`
Replace the placeholder `from_legacy_v1` route with a proper mapper that
understands our current TS project shape (rooms, surfaces, systems,
solar) and produces a populated `ProjectV2`. Verification: load every file
in `verification-files/Warmteverlies/*.vp` and three legacy `.oes` files
from the user's archive; all must produce a `ProjectV2` whose BENG results
are within ±0.5% of the legacy calculation.

### B.8 — Delete legacy (was Step A.7)
Remove `src/core/`, `src/context/EnergyContext.tsx`, `AppContent`, the
`/legacy` route. tsc clean + cargo workspace test green.

## Risks

- BENG orchestration is non-trivial; budget time for B.6.
- Legacy schema is partially documented; B.7 needs sample files.
- OpenAEC ribbon design may require coordination with the style-book repo.

## Out of scope
- IFC import / 3D viewer (deferred)
- Mobile / web build
- PDF report layout (Phase B.6 ships HTML; PDF in a later patch)
