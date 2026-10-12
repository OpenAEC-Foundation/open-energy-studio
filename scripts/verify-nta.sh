#!/usr/bin/env bash
# Local mirror of the CI gates for the NTA 8800 kernel and UI.
# Stops at the first failing gate. Set NTA_SKIP_MSRV=1 to skip the Rust 1.77.2 run,
# NTA_SKIP_KERNEL_E2E=1 to skip the app run against the real kernel (kernel_once).
# The reference-suite report is written to $NTA_EVIDENCE_DIR (default
# nta-evidence/gate, ignored by git); scripts/release-nta.sh keeps it per release.
set -euo pipefail
cd "$(dirname "$0")/.."

# Each step's wall-clock time is printed when the next one starts, and a summary at the end
# (also when a step fails), so slow steps are visible in every gate log.
nta_step_name=""
nta_step_start=0
nta_timings=()
step_done() {
  # $1: a note for the step the gate stopped at, e.g. " (failed, exit 101)".
  if [[ -n "$nta_step_name" ]]; then
    nta_timings+=("$(printf '%5ds  %s%s' "$((SECONDS - nta_step_start))" "$nta_step_name" "${1:-}")")
    printf '    (%ds)\n' "$((SECONDS - nta_step_start))"
  fi
  nta_step_name=""
}
step() {
  step_done
  nta_step_name="$1"
  nta_step_start=$SECONDS
  printf '\n==> %s\n' "$1"
}
timing_summary() {
  local status="$1"
  if [[ "$status" != 0 ]]; then
    printf '\nGate FAILED (exit %s) at step: %s\n' "$status" "${nta_step_name:-before the first step}" >&2
    step_done " (failed, exit $status)"
  else
    step_done
  fi
  printf '\nGate step times:\n'
  printf '%s\n' "${nta_timings[@]}"
  printf '%5ds  total\n' "$SECONDS"
}
nta_evidence_dir="${NTA_EVIDENCE_DIR:-nta-evidence/gate}"

# With NTA_GATE_PARALLEL=1 (see start_bg), independent pipelines run in the background while
# the kernel steps run in the foreground:
# the Rust 1.77.2 core run (its own toolchain and target dir) and the frontend bundle with the
# Tauri steps (their own target dir). The kernel's slowest test binaries hold only two or three
# tests each, so most cores are idle during them. Each background pipeline writes its own log,
# printed in full when it is collected; a failing pipeline fails the gate. All of them are
# collected before the UI tests, so vitest never runs next to a cargo build.
nta_bg_dir="$nta_evidence_dir/parallel"
nta_bg_names=()
nta_bg_pids=()
bg_file() { printf '%s/%s' "$nta_bg_dir" "$(printf '%s' "$1" | tr -c 'A-Za-z0-9.' '_')"; }
# Opt-in (NTA_GATE_PARALLEL=1): on 12 October 2026 three parallel cargo builds on an 8-core,
# 13 GB machine ran out of memory. By default the pipelines are only queued here and run one
# after another where they are collected (after the kernel steps and TypeScript).
nta_bg_queue=()
start_bg() {
  local name="$1"
  shift
  mkdir -p "$nta_bg_dir"
  if [ "${NTA_GATE_PARALLEL:-0}" != "1" ]; then
    nta_bg_queue+=("$name" "$1")
    return 0
  fi
  printf '\n==> %s (started in the background, log %s.log)\n' "$name" "$(bg_file "$name")"
  ( nta_bg_timing="$(bg_file "$name").timing"; "$@" ) >"$(bg_file "$name").log" 2>&1 &
  nta_bg_names+=("$name")
  nta_bg_pids+=("$!")
}
# Inside a background pipeline: a sub-step header, and its time in the pipeline's timing file.
nta_bg_sub=""
nta_bg_sub_start=0
bg_step() {
  if [[ -n "$nta_bg_sub" ]]; then
    printf '%5ds  %s (background)\n' "$((SECONDS - nta_bg_sub_start))" "$nta_bg_sub" >>"$nta_bg_timing"
  fi
  nta_bg_sub="$1"
  nta_bg_sub_start=$SECONDS
  if [[ -n "$1" ]]; then printf '\n==> %s\n' "$1"; fi
}
finish_bg() {
  local i status line name
  for ((i = 0; i < ${#nta_bg_queue[@]}; i += 2)); do
    name="${nta_bg_queue[$i]}"
    step "$name"
    status=0
    ( nta_bg_timing="$(bg_file "$name").timing"; "${nta_bg_queue[$((i + 1))]}" ) \
      >"$(bg_file "$name").log" 2>&1 || status=$?
    cat "$(bg_file "$name").log"
    if [[ -f "$(bg_file "$name").timing" ]]; then
      while IFS= read -r line; do nta_timings+=("$line"); done <"$(bg_file "$name").timing"
    fi
    if [[ "$status" != 0 ]]; then
      printf '\nPipeline failed: %s (exit %s)\n' "$name" "$status" >&2
      exit "$status"
    fi
  done
  nta_bg_queue=()
  for i in "${!nta_bg_pids[@]}"; do
    step "collect: ${nta_bg_names[$i]}"
    status=0
    wait "${nta_bg_pids[$i]}" || status=$?
    cat "$(bg_file "${nta_bg_names[$i]}").log"
    if [[ -f "$(bg_file "${nta_bg_names[$i]}").timing" ]]; then
      while IFS= read -r line; do nta_timings+=("$line"); done <"$(bg_file "${nta_bg_names[$i]}").timing"
    fi
    if [[ "$status" != 0 ]]; then
      printf '\nBackground pipeline failed: %s (exit %s)\n' "${nta_bg_names[$i]}" "$status" >&2
      exit "$status"
    fi
  done
  nta_bg_pids=()
  nta_bg_names=()
}
# A background pipeline runs cargo or npm, which start their own children (rustc, vitest
# workers): stop the whole tree, deepest first, so nothing keeps building after a failure.
kill_tree() {
  local child
  for child in $(pgrep -P "$1" 2>/dev/null); do kill_tree "$child"; done
  kill -TERM "$1" 2>/dev/null || true
}
stop_bg() {
  local pid
  for pid in "${nta_bg_pids[@]}"; do kill_tree "$pid"; done
  if [[ ${#nta_bg_pids[@]} -gt 0 ]]; then wait "${nta_bg_pids[@]}" 2>/dev/null || true; fi
}
# The gate's own exit status is kept: the summary never turns a failure into a pass.
trap 'nta_exit=$?; stop_bg; timing_summary "$nta_exit"; exit "$nta_exit"' EXIT

msrv_core() {
  bg_step "core on Rust 1.77.2"
  CARGO_TARGET_DIR=target/msrv-core cargo +1.77.2 test --locked --quiet \
    --manifest-path crates/nta8800-core/Cargo.toml
  bg_step ""
}
frontend_and_tauri() {
  bg_step "frontend bundle"
  # Tauri's generate_context! needs ../dist, so a fresh checkout (or a release
  # from one) must build the frontend before the Tauri steps.
  npm run build
  bg_step "Tauri check"
  cargo check --locked --manifest-path src-tauri/Cargo.toml --quiet
  bg_step "Tauri command tests"
  cargo test --locked --manifest-path src-tauri/Cargo.toml --lib --quiet
  bg_step ""
}
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

rm -rf "$nta_bg_dir"
if [ "${NTA_SKIP_MSRV:-0}" != "1" ]; then
  start_bg "core on Rust 1.77.2" msrv_core
fi
start_bg "frontend bundle and Tauri" frontend_and_tauri

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

step "TypeScript"
npx tsc --noEmit

finish_bg

step "UI tests"
# The project-form guard (src/__tests__/nta-project-form-guard.test.tsx) writes every form state
# it produced; the next step runs them through the kernel.
# The clear-field sweep (src/__tests__/nta-clear-field-sweep.test.tsx, review 58) writes every
# field cleared on its own; the same step checks each gives one gap, at that field.
nta_form_guard_docs="$nta_evidence_dir/form-guard/project-forms.jsonl"
nta_clear_sweep_docs="$nta_evidence_dir/form-guard/clear-fields.jsonl"
rm -f "$nta_form_guard_docs" "$nta_clear_sweep_docs"
NTA_FORM_GUARD_OUT="$nta_form_guard_docs" NTA_CLEAR_SWEEP_OUT="$nta_clear_sweep_docs" \
  npx vitest run --maxWorkers="${NTA_VITEST_MAX_WORKERS:-4}"

if [ "${NTA_SKIP_KERNEL_E2E:-0}" != "1" ]; then
  # The full app against the real kernel, without the API server: kernel_once answers
  # each /api call in-process. Reuses the service crate's target dir (built above).
  step "app against the real kernel (kernel_once)"
  cargo build --locked --quiet --manifest-path crates/nta8800-service/Cargo.toml --example kernel_once
  # Where cargo put it: CARGO_TARGET_DIR when set (relative to the repository root), else the crate's own target.
  nta_kernel_once="${CARGO_TARGET_DIR:-crates/nta8800-service/target}/debug/examples/kernel_once"
  case "$nta_kernel_once" in /*) ;; *) nta_kernel_once="$(pwd)/$nta_kernel_once" ;; esac
  OES_KERNEL_ONCE="$nta_kernel_once" \
    npx vitest run src/__tests__/real-kernel-e2e.test.tsx
fi

step "project-form states through the kernel"
NTA_FORM_GUARD_DOCS="$(realpath "$nta_form_guard_docs")" \
  NTA_CLEAR_SWEEP_DOCS="$(realpath "$nta_clear_sweep_docs")" \
  cargo test --locked --manifest-path crates/nta8800-core/Cargo.toml --quiet --test project_form_guard

printf '\nAll configured technical NTA gates passed; reference verification and attestation require separate evidence.\n'
