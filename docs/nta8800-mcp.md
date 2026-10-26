# NTA 8800 MCP-server

De MCP-server maakt de Rust-rekenkern bruikbaar voor AI-assistenten zoals Claude Code en Claude Desktop, via het Model Context Protocol over stdio.
- **Tools:** elke tool is een bewerking uit hetzelfde register als de HTTP-API (`crates/nta8800-operations/src/lib.rs`). Tool `x` en route `x` geven dus dezelfde uitkomst; de tabel staat in [nta8800-api.md](nta8800-api.md#bewerkingen).
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
  - een JSON Schema van de invoer: een object met het vereiste invoerlid (`project`, `survey`, `input`, `case`, of `original` en `current`);
  - annotaties: elke tool is een berekening zonder bijwerkingen (`readOnlyHint: true`, `destructiveHint: false`, `idempotentHint: true`, `openWorldHint: false`). Een tool leest alleen zijn argumenten en geeft een uitkomst; hij schrijft geen bestanden, gebruikt geen netwerk en bewaart niets tussen aanroepen. Er is dus ook niets terug te draaien. Een client kan de tools daarom zonder schrijfrechten toestaan.
- **`tools/call`:**
  - **Inhoud:** het resultaat is precies de JSON-body die de HTTP-route teruggeeft, als `structuredContent` en als tekst in `content`. Eén uitzondering: MCP vraagt een object als `structuredContent`, dus de lijst van `list_interpretations` komt verpakt als `{"result": [...]}`.
  - **`isError`:** dit staat op `true` als de route met 4xx of 5xx zou antwoorden.
    - Bij een ontbrekend of verkeerd invoerlid krijg je de foutenvelop met `code` en `path`, bijvoorbeeld `missing_request_member` met pad `project`.
    - Bij een weigering van de kern krijg je de beoordeling, met `status`, `gaps` en `issues`.
    - Een onbekende tool geeft `unknown_tool`.
    - Een onbekend lid naast het invoerlid en `normVersion` geeft `invalid_request_shape` met het pad van dat lid, zoals bij de HTTP-API.
    - Een tool zonder invoer (de GET-operaties, zoals `get_version` en `list_interpretations`) weigert elk argument met `invalid_request_shape`, ook `normVersion`; het schema staat geen leden toe (`additionalProperties: false`). Een leeg `arguments`-object of geen `arguments` is goed.
- **`compare_project_editions`:** hier is `normVersion` de doeluitgave; het project behoudt zijn eigen uitgave. De tool toont wat die uitgave weigert, extra vraagt, niet gebruikt of anders leest, met omschakelpunt, pad en aanpassing, plus het aangepaste project (zie [API, Uitgave wijzigen](nta8800-api.md#uitgave-wijzigen-compare_project_editions)).
- **`resources/list` en `resources/read`:** zie hieronder. Een onbekende URI geeft de JSON-RPC-fout *resource not found*.

Een tool die lang rekent, draait buiten de protocol-lus, zodat de server bereikbaar blijft. Er rekenen hoogstens zoveel tools tegelijk als de HTTP-API standaard rekenplaatsen heeft (het aantal processorkernen); verdere aanroepen wachten op een vrije plaats. `OES_MCP_MAX_CALCULATIONS` (1–1024) stelt een ander aantal in.

### Grenzen en foutregels

Elke regel op stdin gaat eerst door een poortwachter. Die beantwoordt de regels waar de MCP-bibliotheek anders geen antwoord op geeft, zodat een client nooit blijft wachten:

| Regel | Antwoord |
|---|---|
| Geen geldige JSON, of afgebroken | JSON-RPC-fout `-32700` (*parse error*), met het `id` als dat nog te lezen is, anders `null` |
| Dieper genest dan 128 niveaus | `-32600` (*invalid request*), met het `id` |
| Langer dan de regelgrens | `-32600`, met `id` `null`; de rest van de regel wordt gelezen en weggegooid, niet bewaard |
| Meer JSON-waarden dan de waardengrens | `-32600`, met het `id` als dat te lezen is en `data` `{"code": "payload_too_large", "maxJsonValues": …}`; de regel wordt niet ingelezen |
| `tools/call` met `params` of `arguments` die geen object zijn | `-32602` (*invalid params*), met het `id` |

- **Regelgrens:** 64 MiB per bericht. `OES_MCP_MAX_LINE_BYTES` (in bytes, minimaal 1024) stelt een andere grens in.
- **Waardengrens:** 500 000 JSON-waarden per bericht, dezelfde standaard als `--max-json-values` van de HTTP-API. Een ingelezen bericht neemt geheugen per waarde, niet per byte; zonder deze grens kon een regel binnen 64 MiB een veelvoud daarvan aan geheugen vragen. `OES_MCP_MAX_JSON_VALUES` (minimaal 1000) stelt een andere grens in.
- Een melding (een bericht zonder `id`) krijgt nooit een antwoord, ook niet als hij ongeldig is.
- Na zo'n fout blijft de server gewoon antwoorden.

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
  - fouten worden `isError`;
  - de poortwachter: ongeldige, afgebroken en te diepe JSON, verkeerde `arguments`, de regelgrens en het lezen van het `id`.
- **`crates/nta8800-service/tests/mcp_stdio.rs`:**
  - start het binary en doorloopt het protocol: `initialize` met protocol 2025-06-18, `tools/list`, drie `tools/call` (versie, berekening van het voorbeeldproject, ontbrekend lid), `resources/list` en `resources/read`;
  - stuurt ongeldige, afgebroken, te diepe en te lange regels en verkeerde `arguments`, controleert elk foutantwoord en dat de server daarna gewoon antwoordt.

## Build

De SHA-256 van de release-binaries staat in [nta8800-api.md](nta8800-api.md#build).
