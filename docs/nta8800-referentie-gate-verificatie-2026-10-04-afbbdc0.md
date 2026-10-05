# Referentiebatch-CLI — technische controle op `afbbdc0`

Datum: 4 oktober 2026. Broncommit: `afbbdc0` (`nta8800-kernel`). Doeluitgave: NTA 8800:2025+C1:2026.

De service is met de lokale Rust-toolchain gecontroleerd: `cargo fmt --check`, `cargo test` (56 servicetests en vier CLI-tests), `cargo clippy --all-targets -- -D warnings` en `cargo build --bin reference_gate`. Een runtimeproef met het interne synthetische project en `beng2=8,17` gaf exitcode 0 voor een passend plan. Wanneer hetzelfde plan ook `beng1` vereiste, bleef `numericComparisonPassed=true`, werd `plannedCoveragePassed=false` en volgde exitcode 1. `referenceVerified=false` en `attestStatus=unattested` bleven in beide rapporten staan.

Gebouwde Linux x86-64 debugbinary: `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-reference-gate_afbbdc0_linux-amd64`.

SHA-256: `fecbd79f4131b25dba27c89d6c802b779cb79a49fe033261d6c5fafe4818bdfd`.

Dit is een technische controle van de vergelijkings- en dekkingsmechaniek. De testinvoer is synthetisch; officiële actuele EDR-uitkomsten, een onafhankelijk vastgesteld W/U-dekkingsplan en een BRL 9501-attest ontbreken. De bestaande desktop-devbuild van `37ccce1` is hierdoor niet veranderd.
