# Rekenkern in de browser (WebAssembly)

De webversie van Open Energy Studio rekent sinds 5 oktober 2026 met dezelfde Rust-kern als de desktop-app en de HTTP-API. De kern wordt naar WebAssembly vertaald en met de webpagina meegeleverd; de berekening draait op de computer van de gebruiker. Er is geen server met de kernservice nodig, en projectgegevens verlaten de browser niet.

## Opbouw

| Onderdeel | Rol |
|---|---|
| `crates/nta8800-core` | de rekenkern, ongewijzigd |
| `crates/nta8800-operations` | het register van alle operaties (naam, HTTP-route, invoer, statusregel). Voorheen `operations.rs` in de servicecrate; nu een eigen crate zonder webserver-afhankelijkheden, zodat hij ook voor `wasm32-unknown-unknown` compileert. De service exporteert hem opnieuw als `nta8800_service::operations`. |
| `crates/nta8800-wasm` | de WebAssembly-adapter: `run(route, body)` zoekt de operatie op de HTTP-route (met of zonder `/api`) of de MCP-naam en geeft `{"status", "body"}` terug, precies zoals de HTTP-API zou antwoorden. Verder `version()` en `list_operations()`. |
| `src/kernel-wasm/` | het gebouwde pakket (`nta8800.js`, `nta8800_bg.wasm`, typedefinities), gegenereerd met `npm run build:wasm` en meegecommit zodat de webbuild geen Rust-toolchain nodig heeft. |
| `src/core/nta/kernelTransport.ts` | één aanroeplaag voor de UI: `kernelCall(command, route, payload)` en `kernelGet`. Volgorde: Tauri-commando in de desktop-app, anders de wasm-module, anders de ontwikkelserver met `/api`-proxy, anders een foutmelding. |

Alle functies in `src/core/nta/KernelClient.ts` gaan door `kernelCall`; de losse `isTauri()`/`fetch`-takken per functie zijn weg.

## Statusregels

De wasm-adapter past dezelfde statusregels toe als de HTTP-API (`StatusRule` in het register). Een geweigerde berekening (422) draagt nog steeds de beoordeling met haar `status`, en de UI toont die. Een transport- of vormfout (400, 404, 500) bevat alleen het foutomhulsel met `code` en `message`; `kernelCall` maakt daar een exception van, zoals de HTTP-client dat deed bij een mislukte respons.

## Bouwen en testen

```bash
npm run build:wasm      # wasm-pack, release, opt-level s; schrijft src/kernel-wasm/
npm test                # src/__tests__/kernel-wasm.test.ts laadt het pakket en rekent een opname en een project door
```

`cargo test` in `crates/nta8800-wasm` draait dezelfde dispatchtests op de eigen machine zonder browser. CI controleert bovendien dat de crate voor het wasm-doel compileert.

Na een wijziging in de kern of het register moet het pakket opnieuw worden gebouwd en meegecommit; de test op de kernelversie in `kernel-wasm.test.ts` en de `buildFingerprint` in `version()` maken een verouderd pakket zichtbaar.

## Omvang en laden

Het wasm-bestand is ongeveer 8,5 MB (2,3 MB gecomprimeerd) en wordt pas geladen bij de eerste kernaanroep, niet bij het openen van de pagina. De startbundel van de app verandert niet.

## Niet in de browser

De MCP-server en de referentiegate-CLI blijven losse programma's; die hebben bestanden en standaardinvoer nodig. De OpenAPI-specificatie blijft bij de HTTP-service.
