#!/usr/bin/env bash
# Local mirror of the CI gates for the NTA 8800 kernel and UI.
# Stops at the first failing gate. Set NTA_SKIP_MSRV=1 to skip the Rust 1.77.2 run.
# The reference-suite report is written to $NTA_EVIDENCE_DIR (default
# nta-evidence/gate, ignored by git); scripts/release-nta.sh keeps it per release.
set -euo pipefail
cd "$(dirname "$0")/.."

step() { printf '\n==> %s\n' "$1"; }
nta_evidence_dir="${NTA_EVIDENCE_DIR:-nta-evidence/gate}"
nta_commit="$(git rev-parse HEAD 2>/dev/null || printf 'onbekend')"
if [[ -n "$(git status --porcelain 2>/dev/null)" ]]; then
  nta_commit="$nta_commit (met lokale wijzigingen)"
fi

step "kernel version and release notes"
node scripts/nta-kernel-version.mjs check

step "manual stamped for this kernel version"
node scripts/nta-manual.mjs check

if [[ -n "${NTA_REFERENCE_PLAN:-}" || -n "${NTA_REFERENCE_CASE_DIR:-}" ]]; then
  if [[ -z "${NTA_REFERENCE_PLAN:-}" || -z "${NTA_REFERENCE_CASE_DIR:-}" ]]; then
    printf 'Set both NTA_REFERENCE_PLAN and NTA_REFERENCE_CASE_DIR for the reference gate.\n' >&2
    exit 2
  fi
  if [[ ! -f "$NTA_REFERENCE_PLAN" || ! -d "$NTA_REFERENCE_CASE_DIR" ]]; then
    printf 'Reference plan file or case directory is missing.\n' >&2
    exit 2
  fi
fi

for crate in crates/nta8800-core crates/nta8800-service; do
  step "rustfmt $crate"
  cargo fmt --manifest-path "$crate/Cargo.toml" --check
  step "tests $crate"
  cargo test --locked --manifest-path "$crate/Cargo.toml" --quiet
  step "clippy $crate"
  cargo clippy --locked --manifest-path "$crate/Cargo.toml" --all-targets --quiet -- -D warnings
done

mapfile -d '' -t nta_reference_suites < <(find training-data/reference-suites -maxdepth 1 -type f -name '*.json' -print0 | sort -z)
nta_suite_args=()
for suite in "${nta_reference_suites[@]}"; do
  nta_suite_args+=(--suite "$suite")
done
step "reference suites (report in $nta_evidence_dir/reference)"
cargo run --locked --quiet --manifest-path crates/nta8800-service/Cargo.toml \
  --bin reference_gate -- --commit "$nta_commit" --report-dir "$nta_evidence_dir/reference" \
  "${nta_suite_args[@]}"

if [[ -n "${NTA_REFERENCE_PLAN:-}" || -n "${NTA_REFERENCE_CASE_DIR:-}" ]]; then
  mapfile -d '' -t nta_reference_cases < <(find "$NTA_REFERENCE_CASE_DIR" -maxdepth 1 -type f -name '*.json' -print0 | sort -z)
  step "planned reference comparisons"
  cargo run --locked --quiet --manifest-path crates/nta8800-service/Cargo.toml \
    --bin reference_gate -- --plan "$NTA_REFERENCE_PLAN" "${nta_reference_cases[@]}"
else
  step "planned reference comparisons skipped (no plan or case directory configured)"
fi

if [ "${NTA_SKIP_MSRV:-0}" != "1" ]; then
  step "core on Rust 1.77.2"
  CARGO_TARGET_DIR=target/msrv-core cargo +1.77.2 test --locked --quiet \
    --manifest-path crates/nta8800-core/Cargo.toml
fi

step "Tauri check"
cargo check --locked --manifest-path src-tauri/Cargo.toml --quiet

step "Tauri command tests"
cargo test --locked --manifest-path src-tauri/Cargo.toml --lib --quiet

step "TypeScript"
npx tsc --noEmit

step "UI tests"
npx vitest run --testTimeout=20000 --maxWorkers="${NTA_VITEST_MAX_WORKERS:-4}"

step "frontend bundle"
npm run build

printf '\nAll configured technical NTA gates passed; reference verification and attestation require separate evidence.\n'
