# Referentiebatch-CLI — vastgezette manifests op `cd2e81a`

Datum: 4 oktober 2026. Broncommit: `cd2e81a` (`nta8800-kernel`). Doeluitgave: NTA 8800:2025+C1:2026.

`cargo fmt --check`, `cargo test` (56 service- en vier CLI-tests), `cargo clippy --all-targets -- -D warnings` en `cargo build --bin reference_gate` slaagden. De runtimeproef gebruikte één synthetisch project met BENG 2 = 8,17. De geplande case slaagde ongewijzigd met exitcode 0. Na een wijziging van alleen de bron-document-ID bleef `numericComparisonPassed=true`, maar werd `plannedCoveragePassed=false` en volgde exitcode 1 wegens een afwijkende manifestvingerafdruk.

Linux x86-64 debugbinary: `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-reference-gate_cd2e81a_linux-amd64`.

SHA-256: `edcea77ed0eacc25d2dde81864a1069cef2498db47e703ce0fca6e1725951520`.

De desktopcode is niet gewijzigd; de devbuild van `58bf6c7` blijft de actuele desktopbuild. Deze controle bewijst alleen de werking van de technische integriteits- en numerieke gate op een interne fixture. Officiële actuele W/U-verwachtingen, onafhankelijke bronvalidatie en een BRL 9501-attest ontbreken.
