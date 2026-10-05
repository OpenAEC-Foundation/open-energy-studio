# NTA 8800 devbuild en technische verificatie — 3 oktober 2026

Broncommit: `c4d6fc6` (`Update the verification status after closing the audit 2 gaps`). De werkboom had bij de bouw geen gewijzigde bronbestanden. De debugbuild is gemaakt op Linux amd64 met `npm run tauri:build -- --debug --bundles deb`.

| Controle | Uitkomst |
| --- | --- |
| Rust-kern `cargo test --manifest-path crates/nta8800-core/Cargo.toml` | 635 geslaagd, 0 mislukt |
| HTTP/MCP-service `cargo test --manifest-path crates/nta8800-service/Cargo.toml --quiet` | 52 geslaagd, 0 mislukt |
| Frontend `npm test` | 265 geslaagd in 41 testbestanden, 0 mislukt |
| Tauri debug-`.deb` | Bouw geslaagd; pakket `open-energy-studio`, versie `0.1.6-alpha`, architectuur `amd64` |
| Pakketafhankelijkheden | `libwebkit2gtk-4.1-0`, `libgtk-3-0` volgens `dpkg-deb -f` |

Pakket: `Open Energy Studio_0.1.6-alpha_amd64.deb` in `src-tauri/target/debug/bundle/deb/`. SHA-256: `6f0fa2e3b18df1fd0f4f424c3583137ba202436d7e7c147fb2ea1198bb76f250`. Een kopie voor overdracht heet `open-energy-studio_0.1.6-alpha_c4d6fc6_debug_amd64.deb`.

Dit zijn technische regressie- en bouwcontroles. Een visuele desktopacceptatietest, vergelijking met actuele onafhankelijke NTA 8800-referentie-uitkomsten en formele attestering zijn niet uitgevoerd. De berekende energielabelklasse blijft indicatief en ongeattesteerd.
