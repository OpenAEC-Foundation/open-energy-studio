# Phase C Implementation Plan — Verification Module

**Date:** 2026-05-21
**Spec:** `docs/superpowers/specs/2026-05-21-phase-c-verification-comparison-design.md`
**Dependencies:** Phase B (`OesProjectResult` defined)

## Steps

### C.1 — `openaec-vabi-importer` crate
New workspace member under `vendor/crates-warehouse/openaec-vabi-importer`.
Coordinated PR upstream first; bumps submodule.

- `import_vp(&Path) -> Result<VabiReference>` using `zip` + `quick-xml`.
- Map XML to `ProjectV2` + extract `ReferenceResult` from results XML.
- `tests/fixtures/` mini sample for unit tests.
- `FORMAT.md` documenting Vabi's XML schema (best-effort RE from 12 corpus
  samples).

### C.2 — `openaec-verification` crate
`verify(ours: &OesProjectResult, theirs: &ReferenceResult, tolerance: f64)`.
Pure logic, no I/O. Unit-tested with synthetic fixtures.

### C.3 — Tauri commands `list_reference_cases / run_verification /
        run_all_verifications / export_verification_pdf`
File under `src-tauri/src/commands/verify.rs`. Reference path comes from
settings store (default `~/Documents/GitHub/verification-files/Warmteverlies/`).

### C.4 — Verify page UI
Replace the placeholder `/verify` route with the real list + slide-over
diff panel described in the spec. Use `MonthlyStackChart` and the
existing diff colors. Add tolerance slider.

### C.5 — `openaec-verify-cli` binary
Stand-alone runner for CI gating. Emits JSON summary; non-zero exit on
failure.

### C.6 — CI integration
Add a GitHub Actions step that downloads the verification-files corpus
(as a separate submodule under `vendor/verification-files/`) and runs the
CLI. Failures block merges to main.

## Risks
- .vp schema RE may surface non-XML blobs; mitigate with passthrough and
  document limits.
- Reference PDFs may carry richer data than XML; fall back to `lopdf` text
  extraction where needed.
