# Phase C — Verification & Comparison Module

**Date:** 2026-05-21
**Status:** Ready for implementation plan
**Dependencies:** Phase A (commands surface), Phase B (Project UI present so users
can inspect imported cases)

## Purpose

Establish confidence that the NTA 8800 crates compute correctly by importing
reference cases (Vabi `.vp` projects from `verification-files/Warmteverlies/`),
running our calculation against the imported inputs, and diffing the result against
the reference output. Surface pass/fail per indicator with configurable tolerances.

The verification module is both an end-user feature (architects can re-verify their
own .vp projects on our engine) and a development tool (regression suite for the
crates).

## Conceptual Model

```
Vabi .vp file (ZIP of XML + PDF report)
   │
   │  parse via openaec-vabi-importer (new crate, Phase C deliverable)
   ▼
VabiReference {
    project_inputs:   ProjectV2,           // what we'd compute from
    reference_results: ReferenceResult,    // what Vabi computed (parsed from XML)
}
   │
   ├─▶ ProjectV2 → invoke('calculate') → OurResult: ProjectResult
   │
   ▼
VerificationReport {
    case_name: String,
    indicators: Vec<IndicatorComparison>,  // BENG1, BENG2, BENG3, label, ...
    monthly: MonthlyComparison,            // 12 × {ours, reference, Δ%}
    overall_status: Pass | Warning | Fail,
    tolerance: f64,                         // configurable, default 2.0%
}
```

## User Flow

1. User opens `/verify` page.
2. Page lists reference cases discovered in
   `verification-files/Warmteverlies/*.vp`. Each row shows case name, last verified
   date, last status (green/yellow/red), and a "Run" button.
3. Clicking "Run" loads the .vp, runs the calculation, displays a side-by-side
   diff in a slide-over panel.
4. Bulk "Run all" button regenerates the full report; finishes within ~30 s for
   the 12-case corpus.
5. "Export report" produces a PDF summarising the entire run for audit.

## Components

### Crate: `openaec-vabi-importer` (new)

Lives in `vendor/crates-warehouse/openaec-vabi-importer/` so other OpenAEC apps
can reuse it. (We commit upstream first.)

```rust
pub fn import_vp(path: &Path) -> Result<VabiReference, ImportError>;

pub struct VabiReference {
    pub project: ProjectV2,
    pub reference_results: ReferenceResult,
    pub metadata: VabiMetadata,
}

pub struct ReferenceResult {
    pub beng_1: f64,
    pub beng_2: f64,
    pub beng_3_pct: f64,
    pub energy_label: String,
    pub monthly_q_h_use: [f64; 12],
    pub monthly_q_c_use: [f64; 12],
    pub monthly_q_w_use: [f64; 12],
    pub primary_energy_breakdown: HashMap<EnergyCarrier, f64>,
}
```

**.vp format reverse-engineering**: Vabi .vp is a ZIP containing one or more XML
documents plus a PDF. We extract via `zip` crate, parse XML via `quick-xml`,
inspect against the 12 known samples to derive the schema. We document the
mapping in `openaec-vabi-importer/FORMAT.md` so future Vabi version bumps are
traceable.

If a field cannot be mapped 1:1 (Vabi uses some Dutch-only enum values), the
mapper records a `MappingNote` in `metadata.notes` rather than failing — partial
imports are useful for diagnosing differences.

### Crate: `openaec-verification` (new)

Pure-Rust comparison logic, no UI.

```rust
pub fn verify(
    ours: &ProjectResult,
    theirs: &ReferenceResult,
    tolerance: f64,
) -> VerificationReport;
```

The comparator handles unit normalisation (Vabi may report kWh/m²·yr; our crates
return MJ; convert before diffing) and produces structured results.

### Tauri commands (added to `src-tauri/src/commands/verify.rs`)

```rust
#[tauri::command] fn list_reference_cases() -> Result<Vec<ReferenceCase>, AppError>;
#[tauri::command] fn run_verification(case_id: String, tolerance: f64)
    -> Result<VerificationReport, AppError>;
#[tauri::command] fn run_all_verifications(tolerance: f64)
    -> Result<Vec<VerificationReport>, AppError>;
#[tauri::command] fn export_verification_pdf(reports: Vec<VerificationReport>,
    path: String) -> Result<(), AppError>;
```

Reference case path is resolved from a configurable setting; default
`%USERPROFILE%/Documents/GitHub/verification-files/Warmteverlies/`. Configurable
via Settings dialog.

### Frontend page: `/verify`

```
┌─────────────────────────────────────────────────────────────┐
│  Verification              Tolerance: [ 2.0 ] %    [Run all]│
├─────────────────────────────────────────────────────────────┤
│  ☑ Bekend.vp                  ✓  Last run: 2026-05-21       │
│  ☑ Tussenwoning_voor.vp       ✓  Last run: 2026-05-21       │
│  ☑ Tussenwoning_na.vp         △  3 indicators outside tol.  │
│  ☐ HoekwoningGroot.vp         —  not yet run                │
│  ...                                                         │
└─────────────────────────────────────────────────────────────┘
           Click row → slide-over with full diff
```

Slide-over panel:

```
Case: Tussenwoning_na.vp
                            Ours      Vabi      Δ        Status
BENG 1 (kWh/m²·yr)          61.3      62.0      -1.1%    ✓
BENG 2 (kWh/m²·yr)          39.8      40.5      -1.7%    ✓
BENG 3 (% renewable)        43.1      41.2      +4.6%    △ outside 2%
Energy label                A         A         —        ✓

Monthly heating demand (MJ)
   Jan  Feb  Mar  Apr  May  Jun  Jul  Aug  Sep  Oct  Nov  Dec
Ours:  ...
Vabi:  ...
Δ%:    ...

Notes from import:
  • Construction "Vloer_BG_v1" mapped to nearest match (Vabi uses code 14B)
  • Ventilation system reported as Klasse C; we use system C with WTW η=0
```

### Verification Report PDF

Generated server-side via `printpdf` (already used in heatloss-studio). One page
per case, plus a summary cover page. Used for archiving regression status.

## Tolerance Model

- Default tolerance: **±2.0%** on numeric indicators
- Energy label: must match exactly (label is a discrete class)
- Monthly arrays: each month within tolerance; overall status fails if ≥1 month
  fails by more than tolerance
- Configurable per-indicator via `Settings → Verification → Tolerances`

Verification statuses:
- **Pass**: every indicator within tolerance
- **Warning**: indicators within 2× tolerance — investigate but not a regression
- **Fail**: any indicator beyond 2× tolerance

## CLI Bonus (developer-facing)

A small Rust binary `openaec-verify-cli` runs verification headless from CI:

```sh
openaec-verify-cli --corpus ./verification-files/Warmteverlies --tolerance 2.0 \
    --output verify-report.json
```

CI gate: PRs that regress any case beyond tolerance fail. Manageable because the
corpus is small (~12).

## Testing

- Unit tests: `openaec-verification::verify()` with synthetic ProjectResult and
  ReferenceResult fixtures.
- Importer tests: pre-canned .vp files (commit minimal samples into the crate's
  `tests/fixtures/`) verify XML parsing.
- Integration: end-to-end run against the live verification-files corpus, gated
  in CI.

## Risks

| Risk | Mitigation |
|---|---|
| .vp format isn't pure XML (binary blobs?) | If so, parse what we can; document non-importable sections; degrade gracefully |
| Vabi reports rounded values; tight tolerances always fail | Default tolerance is 2%; this is reasonable for engineering work |
| Reference results in PDF only, not in XML | Fall back to PDF text extraction via `lopdf`; document quality is poor but workable |
| Mapping ambiguity for system types | Document mapping table in importer README; surface "best-effort match" notes in UI |

## Success Criteria

1. ≥10 of the 12 cases parse and run end-to-end without import errors.
2. ≥8 cases pass at 2% tolerance on BENG 1/2/3.
3. Verification report PDF generates cleanly and is human-readable.
4. CLI runs in CI and gates on regressions.
