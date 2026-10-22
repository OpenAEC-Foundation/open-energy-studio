# Tauri codegen build-order patch

`tauri-codegen` 2.5.5 is copied from the crates.io package locked in
`src-tauri/Cargo.lock` (original checksum
`d4a24476afd977c5d5d169f72425868613d82747916dd29e0a357c84c4bd6d29`).
The upstream Apache-2.0 and MIT license files are retained in the package.

Our sole source change is in `src/embedded_assets.rs`: sort embedded asset keys,
global CSP hashes, HTML paths, and per-HTML hashes when generating Rust tokens.
The same hashes and asset bytes are retained. This addresses a cross-host desktop
debug-binary difference caused by hash ordering. Keep this patch small and remove
it if an upstream version gives deterministic asset codegen. The cross-host
evidence and limitations are in
`docs/nta8800-crosshost-build-2026-10-16-0d4e239.md`.
