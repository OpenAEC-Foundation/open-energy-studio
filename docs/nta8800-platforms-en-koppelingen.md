# Platforms, versies en koppelingen

Stand bij programma 2026.44.0 en rekenkern 0.5.0 (26 oktober 2026). Dit is het overzicht dat issue #9 vraagt: wat er geleverd wordt, op welk platform, in welke vorm, en wat (nog) niet, met de reden.

> **Niet geattesteerd.** Geen enkele vorm hieronder heeft een BRL 9501-attest. Elke uitkomst heeft de status `calculated_unverified`.

## Vormen en platforms

| Vorm | Platform | Levering | Ondertekening | Stand |
|---|---|---|---|---|
| Webapp | elke browser met WebAssembly | https://open-energy-studio.open-aec.com/, gebouwd uit `main` | HTTPS | in gebruik; getest in Chromium. De rekenkern draait in de browser, er gaat geen projectdata naar een server |
| Desktop-app | Windows 10/11 x64 | `x64-setup.exe` (NSIS) en `.msi` bij elke release | Authenticode, Impertio Studio B.V., met tijdstempel van Microsoft | pre-release |
| Desktop-app | macOS (Intel en Apple Silicon) | `universal.dmg` | niet ondertekend en niet genotariseerd | pre-release |
| Desktop-app | Linux x86_64 | `.deb`, `.rpm`, `.AppImage` | geen losse handtekening; SHA-256 in `SHA256SUMS-installers.txt` | pre-release |
| HTTP-API | Windows, macOS, Linux | binary `api`, zelf te bouwen uit de broncode (Rust 1.88+) | — | getest, zie [smoketest](nta8800-api-mcp-smoketest-2026-10-26.md) |
| MCP-server (stdio) | Windows, macOS, Linux | binary `mcp`, zelf te bouwen (Rust 1.88+) | — | getest; registratie in Claude Code en Claude Desktop: [nta8800-mcp.md](nta8800-mcp.md) |
| Rust-bibliotheek | elk platform met Rust | crates `nta8800-core` (Rust 1.77.2+) en `nta8800-operations` (Rust 1.88+), als git-afhankelijkheid op een releasetag | — | voorbeeld: `crates/nta8800-operations/examples/external_consumer.rs` |
| WebAssembly-pakket | browser of Node.js | `src/kernel-wasm` (gebouwd met `npm run build:wasm`) | — | gebruikt door de webapp; niet apart gepubliceerd |

De desktoppakketten worden gebouwd door `.github/workflows/release.yml` op `windows-latest`, `macos-15-intel` en `ubuntu-22.04`. Er zijn geen pakketten voor Windows op ARM of Linux op ARM.

**Ondertekening nagaan.**
- Windows (PowerShell): `Get-AuthenticodeSignature .\Open.Energy.Studio_2026.44.0_x64-setup.exe` geeft `Valid`, ondertekenaar *Impertio Studio B.V.* (vastgesteld op 2026.43.0, 22 oktober 2026).
- Alle pakketten: `sha256sum -c --ignore-missing SHA256SUMS-installers.txt` in de map met de downloads.

## Versies

| Wat | Schema | Waar |
|---|---|---|
| Programma | `JJJJ.W.N` (jaar, ISO-week, volgnummer), Windows-MSI `JJ.W.N` | `package.json`; `npm run bump` zet het ([versiebeheer](nta8800-versiebeheer.md)) |
| Rekenkern | `MAJOR.MINOR.PATCH` (`KERNEL_VERSION`), gekoppeld aan de NTA 8800-uitgave (BRL 9501 §5.2) | `crates/nta8800-core/Cargo.toml`; in elke uitkomst |
| HTTP-API | voorvoegsel `/v1`; een brekende wijziging krijgt een nieuw voorvoegsel | [nta8800-api.md](nta8800-api.md) |
| Bibliotheek | volgt de rekenkernversie; gebruik een releasetag (`tag = "v2026.44.0"`), niet `main` | — |

**Stabiel of pre-release.** Elke release is tot nu toe een pre-release: het programma is een testversie zonder attest. Een release als "stabiel" volgt pas met het attest.

**Nieuwe versies.** De webapp laadt een nieuwe versie zelf. De desktop-app meldt sinds 2026.44.0 een nieuwere release (bij het starten en dagelijks, uit te zetten); hij installeert niet zelf.

## Rekenkern als bibliotheek

De grens van de bibliotheek is het bewerkingenregister in `crates/nta8800-operations`: elke bewerking heeft een naam, een invoerlid en een statusregel, en is precies dezelfde als de HTTP-route en de MCP-tool met die naam. Ingangen:
- `operations()` en `operation(naam)`: de bewerkingen; `(op.run)(json!({ "project": … }))` geeft een `Outcome` met `status` (de HTTP-code) en `body` (de JSON van de route);
- `version_value()`: rekenkernversie, normversie en de ondersteunde uitgaven;
- de invoerschema's staan in het OpenAPI-document (`GET /v1/openapi.json`, of `nta8800_service::openapi_document()`).

Directe aanroepen van functies in `nta8800-core` (bijvoorbeeld `assess_project_performance`) werken ook, maar vallen niet onder de versiebelofte van de API: hun Rust-signatuur kan per rekenkernversie veranderen. In een ander Rust-project:

```toml
[dependencies]
nta8800-operations = { git = "https://github.com/OpenAEC-Foundation/open-energy-studio", tag = "v2026.44.0" }
serde_json = "1"
```

**Licentie.** Alle crates, de app en de documentatie vallen onder LGPL-3.0-or-later. Normteksten (NEN, ISSO, BRL) zijn niet opgenomen; de code verwijst alleen naar paginanummers.

## Nog niet geleverd

| Onderdeel | Reden |
|---|---|
| Native bibliotheek met C-ABI (Windows-DLL, Linux `.so`) of .NET-assembly | Niet gebouwd. De kern is nu bruikbaar als Rust-crate, WebAssembly, HTTP-API en MCP. Een C-ABI vraagt eerst een keuze voor welke afnemer (welke taal, welke bewerkingen); een JSON-in/JSON-uit-ingang per bewerking zou dan de natuurlijke vorm zijn. |
| Uitrol in AWS | Niet ingericht: er is geen container, uitroldefinitie of authenticatie. De API is bedoeld voor lokaal gebruik; achter een proxy met authenticatie kan hij in een container draaien, maar dat is niet beproefd. |
| Installeerbare MCP-bundel (`.mcpb`) of Claude-connector | Niet gemaakt; de MCP-server wordt nu zelf gebouwd en geregistreerd. |
| Kant-en-klare binaries van API en MCP bij de release | Niet bij de release; zelf bouwen uit de broncode. |
| Ondertekening voor macOS en Linux | Er is geen Apple Developer ID; voor Linux alleen controlesommen. |
| Automatisch bijwerken van de desktop-app | Niet ingebouwd; de app meldt een nieuwe versie. Automatisch bijwerken vraagt een eigen updatesleutel. |
| Aansluiting op de huidige OpenAEC Style Repo | De UI gebruikt eigen componenten en tokens (`src/components/ui`); de verhouding tot de Style Repo-pakketten is nog niet vastgelegd. |
