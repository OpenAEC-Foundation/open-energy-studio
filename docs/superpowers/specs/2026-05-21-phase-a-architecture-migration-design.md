# Phase A — Architecture Migration

**Date:** 2026-05-21
**Status:** Ready for implementation plan
**Dependencies:** none (this is the foundation)
**Estimated scope:** large — repository-wide changes

## Purpose

Replace the in-tree TypeScript calculation engine (`src/core/energy/*`) with Tauri
commands that call into the Rust crates in `crates-warehouse`. Adopt the architectural
patterns of `open-heatloss-studio` so future phases can build on a known-good substrate.

This phase is value-neutral for the end user: when complete, the app must behave the
same as before (same BENG calculation, same project files openable). What changes is
the substrate underneath.

## Current State (baseline)

- React 19 + Vite + custom Context (`EnergyContext.tsx`) state, no routing
- All calculation in `src/core/energy/*.ts` (26 files, ~6000 lines TypeScript)
- `src-tauri/src/lib.rs` only contains Windows printer commands; no calc
- `tauri.conf.json` missing `webviewInstallMode` and `bundle.resources`
- No Tauri 2 capabilities directory
- No workspace; flat Cargo.toml
- 23 component folders organised by widget type, dialog-heavy modal UI
- Project file: ad-hoc `.oes` JSON via `src/core/io/ProjectSerializer.ts`

## Target State

- React 19 + Vite + **Zustand** + **React Router v7**, page-based navigation
- All calculation via `invoke()` to Rust commands; `src/core/energy/*` deleted
- `src-tauri/src/lib.rs` registers calculation commands + I/O commands
- `tauri.conf.json` includes `webviewInstallMode: "embedBootstrapper"` and
  `bundle.resources: ["WebView2Loader.dll"]`; DLL committed to `src-tauri/`
- `src-tauri/capabilities/default.json` declares required permissions
- Cargo workspace at repo root with `src-tauri/` as member; crates pulled from
  `vendor/crates-warehouse/` git submodule
- Component folders reorganised: `components/layout/`, `components/ribbon/`,
  `components/ui/`, plus page-scoped folders
- Project file: `.oes.json` envelope wrapping `ProjectV2` (from
  `openaec-project-shared`); auto-migrate legacy

## Architecture

### Repo layout (after migration)

```
open-energy-studio/
├── vendor/
│   └── crates-warehouse/         # git submodule
├── src-tauri/
│   ├── Cargo.toml                # workspace member; path-deps into vendor/
│   ├── tauri.conf.json           # WebView2 bundling, capabilities, bundle resources
│   ├── capabilities/
│   │   └── default.json
│   ├── WebView2Loader.dll        # committed (~160 KB)
│   └── src/
│       ├── main.rs
│       ├── lib.rs                # tauri::Builder + invoke_handler
│       ├── commands/
│       │   ├── mod.rs
│       │   ├── calculate.rs      # full + per-domain calc commands
│       │   ├── project.rs        # load/save/migrate .oes.json
│       │   ├── schema.rs         # project_schema, result_schema
│       │   └── printers.rs       # existing Windows printer code, preserved
│       └── error.rs              # AppError, IntoResponse
├── Cargo.toml                    # workspace root
├── src/
│   ├── main.tsx
│   ├── App.tsx                   # <BrowserRouter><Routes>...</Routes></BrowserRouter>
│   ├── components/
│   │   ├── layout/AppShell.tsx
│   │   ├── ribbon/Ribbon.tsx
│   │   ├── ui/                   # buttons, fields, toasts (move from current places)
│   │   └── ...
│   ├── pages/                    # NEW — one .tsx per route
│   ├── store/                    # NEW — Zustand stores
│   │   ├── projectStore.ts       # ProjectV2, undo/redo, dirty flag, result
│   │   ├── settingsStore.ts      # theme, language, recent files
│   │   └── toastStore.ts
│   ├── lib/
│   │   └── backend.ts            # thin wrapper around @tauri-apps/api invoke()
│   ├── types/
│   │   ├── project.ts            # generated from ProjectV2 JSON Schema
│   │   └── result.ts             # generated from result JSON Schema
│   ├── i18n/                     # unchanged
│   └── core/                     # DELETED
└── scripts/
    └── generate-types.mjs        # runs schemars-export then json2ts
```

### Tauri command surface (Phase A only — domain-specific commands come in Phase B)

```rust
// commands/calculate.rs
#[tauri::command] fn calculate(project: ProjectV2) -> Result<ProjectResult, AppError>;

// commands/project.rs
#[tauri::command] fn load_project(path: String) -> Result<ProjectV2, AppError>;
#[tauri::command] fn save_project(path: String, project: ProjectV2) -> Result<(), AppError>;
#[tauri::command] fn migrate_legacy(path: String) -> Result<ProjectV2, AppError>;

// commands/schema.rs
#[tauri::command] fn project_schema() -> Result<String, AppError>;
#[tauri::command] fn result_schema() -> Result<String, AppError>;

// commands/printers.rs (preserved)
#[tauri::command] fn list_printers() -> Vec<PrinterInfo>;
#[tauri::command] fn print_pdf(...) -> Result<(), AppError>;
```

`calculate` orchestrates the chain documented in
`crates-warehouse/openaec-project-shared`:
```
ProjectV2
  → project_to_nta8800() → Gebouw + Rekenzone
  → nta8800-transmission::calculate_transmission
  → nta8800-ventilation::calculate_ventilation
  → nta8800-demand::calculate_demand           ← uses transmission + ventilation
  → nta8800-heating::calculate_heating         ← uses demand
  → nta8800-cooling::calculate_cooling         ← uses demand
  → nta8800-dhw::calculate_dhw
  → nta8800-lighting::calculate_lighting
  → nta8800-pv::calculate_pv_yield
  → (humidity, automation as configured)
  → aggregate into ProjectResult
```

### State management

`projectStore` (Zustand) mirrors the heatloss pattern:

```ts
interface ProjectStore {
  // data
  project: ProjectV2 | null;
  result: ProjectResult | null;
  filePath: string | null;
  isDirty: boolean;

  // history
  past: ProjectV2[];      // snapshot stack, 50-item cap
  future: ProjectV2[];

  // status
  isCalculating: boolean;
  error: string | null;

  // actions
  setProject(p: ProjectV2): void;
  patchProject(fn: (draft: ProjectV2) => void): void;   // immer-style
  undo(): void;
  redo(): void;
  load(path: string): Promise<void>;
  save(path?: string): Promise<void>;
  calculate(): Promise<void>;
  reset(): void;
}
```

### Project file format

`.oes.json` envelope:

```json
{
  "schema_version": "2.0",
  "exported_at": "2026-05-21T10:00:00Z",
  "app_version": "0.2.0-alpha",
  "project": { /* ProjectV2 */ },
  "result": { /* ProjectResult or null */ }
}
```

Legacy `.oes` files (current TS format) are detected by absence of `schema_version`
or presence of `"version": "1.x"`. `migrate_legacy()` converts via a mapping module
`commands/project.rs::migrate_v1_to_v2()`.

### Routing

```
/                        → redirect to /project
/project                 → ProjectSetup (placeholder; real UI in Phase B)
/zones                   → Zones (placeholder)
/constructions           → Constructions (placeholder)
/systems                 → Systems (placeholder)
/results                 → Results
/verify                  → Verification (Phase C lives here)
```

In Phase A all non-`/results` pages are stubs that read/write the store; Phase B
fills them with real input UI.

### WebView2 bundling (per user CLAUDE.md)

`tauri.conf.json` patch:

```json
{
  "bundle": {
    "resources": ["WebView2Loader.dll"],
    "windows": {
      "webviewInstallMode": { "type": "embedBootstrapper", "silent": true }
    }
  }
}
```

`src-tauri/WebView2Loader.dll` committed to git (160 KB binary).

### Capabilities (Tauri 2)

`src-tauri/capabilities/default.json`:

```json
{
  "identifier": "default",
  "description": "Capabilities for Open Energy Studio main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:default",
    "fs:allow-read-text-file",
    "fs:allow-write-text-file",
    "fs:scope-app",
    "log:default"
  ]
}
```

### Type generation pipeline

1. `cargo run -p schema-export` (small Rust binary in `src-tauri/`) writes:
   - `schemas/project.schema.json` from `ProjectV2`
   - `schemas/result.schema.json` from `ProjectResult`
2. `npx json2ts -i schemas/ -o src/types/` produces TS interfaces.
3. `npm run generate-types` runs both steps. CI runs it before `npm run build`.

## Migration Strategy

Migration is **destructive** within `src/core/` but additive elsewhere. Sequence:

1. **Add vendor submodule** and workspace Cargo.toml — does not break anything.
2. **Add capabilities + WebView2 fixes** — improves Windows install, no regression.
3. **Add Tauri command stubs** that call into crates — does not yet wire to UI.
4. **Add Zustand stores + Router** alongside existing `EnergyContext`. Both coexist.
5. **Create `pages/` placeholder shells** that read from store but render
   "Coming in Phase B" placeholder content for input pages; Results page uses
   real calc.
6. **Wire `App.tsx`** to BrowserRouter; replace the old single-view shell.
7. **Port project load/save** to use Tauri commands + migrate_legacy.
8. **Delete `src/core/energy/`** once all references are dead. Use TypeScript compiler
   to verify (`tsc --noEmit`) — every removal must keep the build green.
9. **Delete `EnergyContext.tsx`** once no component imports it.

Each step is its own commit; CI must stay green between commits.

## Error Handling

`AppError` is the single error type returned to TypeScript. It wraps:

```rust
pub enum AppError {
    Io(std::io::Error),
    Serde(serde_json::Error),
    Calc(isso51_core::Error),
    Validation(String),
    Migration { from: String, reason: String },
    Other(String),
}
```

`impl serde::Serialize for AppError` flattens to `{ kind: "calc", message: "..." }`
so TypeScript can branch on `error.kind`.

## Testing

- **Rust**: `cargo test -p open-energy-studio` runs unit tests for command wiring
  (calc with fixture ProjectV2 → expected ProjectResult; migration of a v1 sample).
- **TypeScript**: keep Vitest. Tests that exercised `src/core/energy/*` are deleted
  alongside the code. New tests exercise the store (`projectStore`) with mocked
  `invoke()`.
- **Integration**: a `tests/` directory with a "smoke" test that boots Tauri in
  headless mode and runs the `calculate` command on a fixture. Out of scope if too
  flaky; Phase C verification suite is the real integration coverage.

## Risks & Mitigations

| Risk | Mitigation |
|---|---|
| Crates-warehouse API drift breaks build | Pin submodule to a tagged commit; bump deliberately. |
| Crate's `ProjectV2` lacks fields the current TS model has | Add fields to crates-warehouse in a coordinated PR before depending on them. |
| Migration from v1 `.oes` loses data | Migration is best-effort; missing fields default; ProjectInfoDialog shows a "migrated from v1, please review" toast. |
| Workspace inflates compile time | Use `[profile.dev] codegen-units = 256` + cache `target/` in CI. |
| `tsc --noEmit` keeps passing while UI breaks silently | Phase A acceptance requires running the app and opening 3 dialog screens; checklist in the plan. |

## Success Criteria (Phase A done)

1. `npm run tauri:dev` boots; old behaviour preserved (load project → calc → see BENG).
2. `tsc --noEmit` clean.
3. `cargo build --workspace` clean.
4. `cargo test --workspace` green.
5. `src/core/energy/` deleted; no references remain.
6. `EnergyContext.tsx` deleted; only `projectStore` in use.
7. Loading a legacy `.oes` file produces a populated `ProjectV2` and recomputes BENG
   within ±0.5% of legacy output.
8. Clean Windows VM install succeeds (no WebView2 error).
