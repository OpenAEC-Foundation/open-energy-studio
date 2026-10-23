# Architectuur (ontwikkelaars)

| Laag | Map | Rol |
|---|---|---|
| Rekenkern | `crates/nta8800-core` | NTA 8800-berekening, uitgaven (`norm_versions/`), opname (`opname/`), maatwerkadvies, herlabelen, registratie, labelgegevens. Geen I/O, deterministisch. |
| Service | `crates/nta8800-service` | Bewerkingenregister, HTTP-API (`api`), MCP-server (`mcp`), `reference_gate`. |
| Desktop | `src-tauri` | Tauri 2: asynchrone kerncommando's die een panic opvangen, atomisch opslaan, bestandsscope, CSP. |
| UI | `src` | React + TypeScript: werkstroom, NTA-formulieren, opname, rapporten, dossier, handleiding. |

- Desktop, API en MCP roepen dezelfde kernfuncties aan, met dezelfde uitgavebehandeling.
- Elk resultaat draagt `KERNEL_VERSION`, de normversie en een invoer-fingerprint.

Zie ook de [programmabeschrijving](https://github.com/OpenAEC-Foundation/open-energy-studio/blob/main/docs/nta8800-programmabeschrijving.md).
