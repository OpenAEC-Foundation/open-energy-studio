#!/usr/bin/env bash
# Local mirror of the CI gates for the NTA 8800 kernel and UI.
# Stops at the first failing gate. Set NTA_SKIP_MSRV=1 to skip the Rust 1.77.2 run.
set -euo pipefail
cd "$(dirname "$0")/.."

step() { printf '\n==> %s\n' "$1"; }

for crate in crates/nta8800-core crates/nta8800-service; do
  step "rustfmt $crate"
  cargo fmt --manifest-path "$crate/Cargo.toml" --check
  step "tests $crate"
  cargo test --manifest-path "$crate/Cargo.toml" --quiet
  step "clippy $crate"
  cargo clippy --manifest-path "$crate/Cargo.toml" --all-targets --quiet -- -D warnings
done

if [ "${NTA_SKIP_MSRV:-0}" != "1" ]; then
  step "core on Rust 1.77.2"
  CARGO_TARGET_DIR=target/msrv-core cargo +1.77.2 test --locked --quiet \
    --manifest-path crates/nta8800-core/Cargo.toml
fi

step "Tauri check"
cargo check --manifest-path src-tauri/Cargo.toml --quiet

step "TypeScript"
npx tsc --noEmit

step "UI tests"
npx vitest run --testTimeout=20000

printf '\nAll NTA gates passed.\n'
