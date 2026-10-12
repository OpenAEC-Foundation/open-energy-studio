# API en MCP (ontwikkelaars)

Elke bewerking van de rekenkern is beschikbaar op twee manieren:
- als **lokale HTTP-API**, met OpenAPI 3.1 op `/v1/openapi.json` en de versie op `/v1/version`;
- als **MCP-server** (stdio), met dezelfde bewerkingen als tools.

Het register telt 46 bewerkingen; elke bewerking is een HTTP-route en een MCP-tool. Nieuw in rekenkern 0.4.0 is `compare_project_editions` (`POST /v1/nta8800/project/editions/compare`): wat een andere uitgave weigert, eist, negeert of anders leest, met schakelpunt.

```bash
cargo run --release --manifest-path crates/nta8800-service/Cargo.toml --bin api      # standaard 127.0.0.1:3007
cargo build --release --manifest-path crates/nta8800-service/Cargo.toml --bin mcp
```

- **Beveiliging:**
  - standaard alleen bereikbaar op `127.0.0.1`;
  - Host-controle tegen DNS-rebinding;
  - CORS alleen voor opgegeven origins.
- **Limieten:**
  - een begrensd aantal gelijktijdige berekeningen en wachtende verzoeken;
  - een aparte wachtrij- en rekentimeout, met 503 en `Retry-After`;
  - een maximale omvang en een maximaal aantal JSON-waarden per verzoek.
- **Onbekende velden** op topniveau worden geweigerd (`invalid_request_shape`).
- **Uitgave:** `normVersion` per verzoek. Oudere uitgaven krijgen de status `calculated_legacy_edition`.
- **Rekentijd:** een groot utiliteitsgebouw (50 zones, 2 250 ramen) rekent in minder dan een halve seconde (release); zie de API-documentatie.

Details: [docs/nta8800-api.md](https://github.com/Erika98-be/open-energy-studio/blob/nta8800-kernel/docs/nta8800-api.md) en [docs/nta8800-mcp.md](https://github.com/Erika98-be/open-energy-studio/blob/nta8800-kernel/docs/nta8800-mcp.md).
