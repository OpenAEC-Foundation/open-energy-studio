# NTA 8800 batch-referentiegate — broncommit de45278

De nieuwe `reference_gate`-CLI leest losse manifest-JSON-bestanden voor de bestaande Rust-projectvergelijking. Dit is een numerieke regressiegate; de bron, rechten, normtoepassing en gekozen toleranties blijven werk voor de onafhankelijke toets.

| Controle | Resultaat |
| --- | --- |
| Rust-formatcontrole en Clippy van de service, alle targets, waarschuwingen als fout | geslaagd |
| Volledige servicetests | 56 HTTP/MCP-tests en 3 CLI-tests geslaagd |
| `cargo build --manifest-path crates/nta8800-service/Cargo.toml --bin reference_gate` | geslaagd |
| Gebouwde CLI met een synthetisch BENG 2-manifest | passende 8,17 met exitcode 0; afwijkende 9,17 en lege batch met exitcode 1 |
| JSON-rapport | `numericComparisonPassed` volgt de drie gevallen; `referenceVerified=false`, `attestStatus=unattested` blijven staan |

Overdrachtskopie: `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-reference-gate_de45278_linux-amd64`; SHA-256 `c3291d7051b0a1d79db73e0beac3cff0411b03691e3be4f65f0ca5693786bdd5`.

De kern en frontend zijn op deze commit niet gewijzigd. De laatste volledige kerntest en desktop-devbuild staan bij [broncommit `37ccce1`](nta8800-build-verificatie-2026-10-04-37ccce1.md); de laatste volledige frontendtest bij `7ac7e52`. De CLI-test gebruikt een interne synthetische verwachting die uit dezelfde Rust-fixture komt. Er zijn nog geen actuele officiële EDR-resultaten of externe BRL 9501-attestering.
