# Installatie

## Desktop-app

De desktop-app is gebouwd met Tauri 2. Een release ([GitHub-releases](https://github.com/OpenAEC-Foundation/open-energy-studio/releases)) levert pakketten voor Windows (NSIS-setup en MSI, ondertekend), Linux (`.deb`, `.rpm`, AppImage) en macOS (universele `.dmg`, niet ondertekend), met `SHA256SUMS-installers.txt`. Zelf bouwen kan met `npm run tauri:build` op het eigen platform.

Bij elke release horen een leveringsdocument, `SHA256SUMS` en de handleiding als HTML. Controleer een pakket in de map van de release met `sha256sum -c --ignore-missing SHA256SUMS`.

## Zelf bouwen

Vereist:
- Node.js 20 of nieuwer (CI en de releasebuilds gebruiken 22);
- Rust 1.77.2 of nieuwer voor de rekenkern; de desktop-app en de API/MCP-service vragen Rust 1.88;
- Tauri 2;
- op Linux ook `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`.

```bash
git clone https://github.com/OpenAEC-Foundation/open-energy-studio.git
cd open-energy-studio
npm install
npm run tauri:build        # desktop-installers voor het eigen besturingssysteem
npm run build              # webversie (map dist/)
```

## Webversie

De webversie rekent met de rekenkern als WebAssembly (`src/kernel-wasm`), in de browser zelf; een statische `dist/`-build heeft geen server nodig. Alleen de ontwikkelserver (`npm run dev`) valt terug op de lokale HTTP-API (poort 3007, via `/api/*`) als de WebAssembly-module niet laadt. De desktop-app gebruikt de ingebouwde kern. Zie [nta8800-wasm.md](https://github.com/OpenAEC-Foundation/open-energy-studio/blob/main/docs/nta8800-wasm.md).

De HTTP-API en de MCP-server zijn voor andere programma's; zie [[API en MCP]] en de [README](https://github.com/OpenAEC-Foundation/open-energy-studio/blob/main/README.md).
