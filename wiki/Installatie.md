# Installatie

## Desktop-app

De desktop-app is gebouwd met Tauri 2. Een release levert nu een Linux-pakket (`.deb`). Pakketten voor Windows (NSIS), andere Linux-formaten (`.rpm`, AppImage) en macOS (`.dmg`) bouw je zelf met `npm run tauri:build` op dat platform.

Bij elke release horen een leveringsdocument, `SHA256SUMS` en de handleiding als HTML. Controleer een pakket in de map van de release met `sha256sum -c --ignore-missing SHA256SUMS`.

## Zelf bouwen

Vereist:
- Node.js 20 of nieuwer (CI en de releasebuilds gebruiken 22);
- Rust 1.77.2 of nieuwer voor de rekenkern; de desktop-app en de API/MCP-service vragen Rust 1.88;
- Tauri 2;
- op Linux ook `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`.

```bash
git clone https://github.com/Erika98-be/open-energy-studio.git
cd open-energy-studio
npm install
npm run tauri:build        # desktop-installers voor het eigen besturingssysteem
npm run build              # webversie (map dist/)
```

## Webversie

De webversie rekent via de lokale HTTP-API (standaard poort 3007). `npm run dev` stuurt `/api/*` door naar die poort; een statische `dist/`-build heeft dezelfde doorverwijzing nodig (`/api` → `http://127.0.0.1:3007`, zonder `/api`). Start de API met:

```bash
cargo run --release --manifest-path crates/nta8800-service/Cargo.toml --bin api
```

Zie [[API en MCP]] en de [README](https://github.com/Erika98-be/open-energy-studio/blob/nta8800-kernel/README.md).
