# Phase A Implementation Plan — Architecture Migration

**Date:** 2026-05-21
**Spec:** `docs/superpowers/specs/2026-05-21-phase-a-architecture-migration-design.md`
**Branch:** `claude/admiring-poincare-5ec4c0`

## Approach

Seven atomic steps. Each is its own commit. After each step the build must
remain green (`npm run build` + `cargo build --workspace`). If a step would
break the build, split it.

## Steps

### Step 1 — Workspace + crates-warehouse submodule

Deliverables:
- `vendor/crates-warehouse/` git submodule pinned at
  `https://github.com/OpenAEC-Foundation/crates-warehouse.git` commit `e6a9edf`
- Root `Cargo.toml` declaring a workspace with `src-tauri` and selected
  `vendor/crates-warehouse/*` paths as members
- `src-tauri/Cargo.toml` updated to use path-deps to needed crates
- `rust-toolchain.toml` pinning a recent stable

Verification:
- `git submodule status` shows clean submodule
- `cargo build --workspace` builds successfully (may be empty/no-op for now)

### Step 2 — WebView2 bundling + Tauri 2 capabilities

Deliverables:
- `src-tauri/WebView2Loader.dll` committed (160 KB)
- `src-tauri/tauri.conf.json` updated: `webviewInstallMode.embedBootstrapper`,
  `bundle.resources` array, `bundle.windows`
- `src-tauri/capabilities/default.json` created with minimal needed permissions
- `src-tauri/Cargo.toml` build deps refreshed

Verification:
- `cargo build -p open-energy-studio` builds; capability file validated by
  `tauri-build`
- `npm run tauri:dev` boots without WebView2 errors

### Step 3 — Tauri commands (calculate, project I/O, schema)

Deliverables:
- `src-tauri/src/commands/mod.rs`, `calculate.rs`, `project.rs`, `schema.rs`,
  `printers.rs` (existing printer code moved here), `error.rs`
- `src-tauri/src/lib.rs` registers all commands via `invoke_handler!`
- Initial `calculate` command implementation chains the crate calls
- `AppError` enum + serde

Verification:
- `cargo test -p open-energy-studio` runs (may have stub tests)
- Manual invoke test: from devtools console
  `await window.__TAURI__.core.invoke('project_schema')` returns a JSON Schema
  string

### Step 4 — Zustand stores + React Router

Deliverables:
- `package.json`: add `zustand`, `react-router-dom`, `immer`,
  `@openaec/ui` (github dep)
- `src/store/projectStore.ts`, `settingsStore.ts`, `toastStore.ts`
- `src/lib/backend.ts` (thin invoke wrapper)
- `src/App.tsx` refactored to use `<BrowserRouter>` + `<Routes>`

Verification:
- `npm run build` passes
- App boots and routes between placeholder pages

### Step 5 — Backend wrapper + type generation

Deliverables:
- Schema-export build step in `src-tauri/` writing JSON Schema files
- `scripts/generate-types.mjs` running schema export then `json2ts`
- `src/types/project.ts`, `result.ts` generated
- `package.json` script `generate-types`

Verification:
- `npm run generate-types` produces non-empty TS files
- TypeScript references compile

### Step 6 — Page stubs + legacy migration

Deliverables:
- `src/pages/ProjectSetup.tsx`, `Zones.tsx`, `Constructions.tsx`,
  `Systems.tsx`, `Results.tsx` — initial stubs reading from store, "coming in
  Phase B" placeholder on input pages, real result rendering on Results
- `src-tauri/src/commands/project.rs::migrate_legacy()` mapping current
  `src/core/io/ProjectSerializer.ts` shape to `ProjectV2`
- `src/lib/backend.ts::loadProject` calls `migrate_legacy` if `schema_version`
  missing

Verification:
- Loading an existing legacy `.oes` from `verification-files/` (or a synthetic
  fixture) populates `projectStore` and Results renders BENG
- `tsc --noEmit` clean

### Step 7 — Delete `src/core/` and `EnergyContext`; verify build

Deliverables:
- All `src/core/energy/*` deleted
- `src/core/io/ProjectSerializer.ts` deleted (replaced by Rust commands)
- `src/core/ifc/*` retained for now (IFC migration is out of scope this phase;
  these are imported only from Phase B pages we haven't built; keep until B)
- `src/context/EnergyContext.tsx` deleted
- All imports updated

Verification:
- `tsc --noEmit` clean
- `cargo build --workspace` clean
- `cargo test --workspace` green
- `npm run tauri:dev` boots; manual sanity check on Results page
- WebView2 fix smoke-tested (optional: clean VM)

## Order of Commits

```
A.1  chore(workspace): add Cargo workspace + crates-warehouse submodule
A.2  feat(tauri): WebView2 bundling + Tauri 2 capabilities
A.3  feat(commands): add calculate/project/schema Tauri commands
A.4  feat(frontend): Zustand stores + React Router shell
A.5  feat(build): schema-driven TS type generation
A.6  feat(pages): page stubs + legacy .oes migration
A.7  chore(cleanup): remove TypeScript calculation engine and EnergyContext
```

## Risks

- Submodule + git worktree interactions: if `git submodule add` misbehaves in
  the worktree, switch to a relative path `[patch]` strategy and revisit later.
- Crate API mismatch: if `openaec-project-shared::ProjectV2` shape doesn't
  cover all NTA 8800 inputs we need, file a coordinated PR upstream first.
- Type generation needs `json2ts` (`json-schema-to-typescript` npm package).

## Out of Scope (deferred)

- IFC import wiring (preserved at TS level until Phase B touches it)
- Dialog cleanup — dialogs remain functional via old EnergyContext-less
  rerouting (slide-overs come in Phase B)
- E2E tests
