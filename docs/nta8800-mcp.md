# NTA 8800 MCP-server

De MCP-server maakt de Rust-rekenkern bruikbaar voor AI-assistenten zoals Claude Code en Claude Desktop, via het Model Context Protocol over stdio.
- **Tools:** elke tool is een bewerking uit hetzelfde register als de HTTP-API (`crates/nta8800-service/src/operations.rs`). Tool `x` en route `x` geven dus dezelfde uitkomst; de tabel staat in [nta8800-api.md](nta8800-api.md#bewerkingen).
- **Resources:** de voorbeeldprojecten, de opnamefixtures, de interpretatielijst, de handleiding en het OpenAPI-document.

Uitkomsten zijn **onverifieerd**: het programma heeft nog geen BRL 9501-attest.

## Bouwen

```
cargo build --release --manifest-path crates/nta8800-service/Cargo.toml --bin mcp
```

Het binary staat dan in `crates/nta8800-service/target/release/mcp`. Het heeft geen andere bestanden nodig: de resources zijn bij het bouwen ingebakken. `mcp --version` drukt de versie-informatie af.

## Registreren

**Claude Code** (vanuit de map van de repository):

```
claude mcp add oes-nta8800 -- "$PWD/crates/nta8800-service/target/release/mcp"
```

Of in `.mcp.json` van een project:

```json
{
  "mcpServers": {
    "oes-nta8800": {
      "command": "/pad/naar/open-energy-studio/crates/nta8800-service/target/release/mcp"
    }
  }
}
```

**Claude Desktop**: voeg hetzelfde blok toe aan `claude_desktop_config.json`, onder `mcpServers`:
- macOS: `~/Library/Application Support/Claude/`;
- Windows: `%APPDATA%\Claude\`;
- Linux: `~/.config/Claude/`.

Herstart daarna Claude Desktop.

## Protocol

- **Transport:** stdio, JSON-RPC 2.0, één bericht per regel.
- **Protocolversie:** onderhandeld bij `initialize`. De server ondersteunt onder meer `2025-06-18`; de test gebruikt die versie.
- **Capabilities:** `tools` en `resources`. De `instructions` bij `initialize` leggen in een paar zinnen uit hoe je begint.
- **`tools/list`:**
  - per tool een naam;
  - een beschrijving in het Engels met Nederlandse vaktermen, waaronder de HTTP-route;
  - een JSON Schema van de invoer: een object met het vereiste invoerlid (`project`, `survey`, `input`, `case`, of `original` en `current`).
- **`tools/call`:**
  - **Inhoud:** het resultaat is precies de JSON-body die de HTTP-route teruggeeft, als `structuredContent` en als tekst in `content`.
  - **`isError`:** dit staat op `true` als de route met 4xx of 5xx zou antwoorden.
    - Bij een ontbrekend of verkeerd invoerlid krijg je de foutenvelop met `code` en `path`, bijvoorbeeld `missing_request_member` met pad `project`.
    - Bij een weigering van de kern krijg je de beoordeling, met `status`, `gaps` en `issues`.
    - Een onbekende tool geeft `unknown_tool`.
- **`resources/list` en `resources/read`:** zie hieronder. Een onbekende URI geeft de JSON-RPC-fout *resource not found*.

Een tool die lang rekent, draait buiten de protocol-lus, zodat de server bereikbaar blijft.

## Resources

| URI | Inhoud |
|---|---|
| `oes://examples/terraced-dwelling` | fictief voorbeeldproject tussenwoning (JSON), als `project` voor `calculate_project_performance` |
| `oes://examples/office` | fictief voorbeeldproject kantoor (JSON) |
| `oes://surveys/residential-1930-terraced`, `…-1975-apartment`, `…-2015-detached` | basisopnames woning (ISSO 82.1), als `survey` voor `assess_residential_survey` |
| `oes://surveys/utility-1985-office`, `…-2005-school`, `…-1970-retail` | basisopnames utiliteit (ISSO 75.1), als `survey` voor `assess_utility_survey` |
| `oes://kernel/interpretations` | de interpretatielijst van de kern (JSON) |
| `oes://api/openapi.json` | het OpenAPI 3.1-document van de HTTP-API |
| `oes://manual/index` en `oes://manual/01-…` t/m `09-…` | de Nederlandse gebruikershandleiding (Markdown) |

## Een sessie

1. Lees `oes://examples/terraced-dwelling`.
2. Roep `calculate_project_performance` aan met `{"project": <inhoud>}`. Laat `null`-leden weg, zoals de app doet.
3. Bij status `incomplete` staan de ontbrekende velden met hun pad in `gaps`. Vul ze aan en reken opnieuw.
4. Voor een bestaande woning zonder volledige invoer: `assess_residential_survey` met een basisopname.
5. Voor maatregelen: `assess_maatwerkadvies` met het project en maatregelen als JSON-patch.
6. Voor een herlabeling: `assess_relabel` met `original` en `current`.

## Tests

- **Eenheidstests** in `crates/nta8800-service/src/bin/mcp.rs`:
  - elke bewerking is een tool met een objectschema;
  - elke resource is leesbaar, en JSON-resources zijn geldige JSON;
  - fouten worden `isError`.
- **`crates/nta8800-service/tests/mcp_stdio.rs`:** start het binary en doorloopt het protocol: `initialize` met protocol 2025-06-18, `tools/list`, drie `tools/call` (versie, berekening van het voorbeeldproject, ontbrekend lid), `resources/list` en `resources/read`.

## Build

De SHA-256 van de release-binaries staat in [nta8800-api.md](nta8800-api.md#build).
