# Normversies

| `normVersion` | Uitgave | Status |
|---|---|---|
| `2025+C1` (standaard) | NTA 8800:2025+C1:2026 | registreerbaar |
| `2024` | NTA 8800:2024 met INT-V1:2024 | oudere uitgave, niet registreerbaar |
| `2023` | NTA 8800:2023 | oudere uitgave, niet registreerbaar |
| `2022` | NTA 8800:2022 | oudere uitgave, niet registreerbaar |
| `2020+A1` | NTA 8800:2020+A1:2020 | oudere uitgave, niet registreerbaar |

- **Kiezen:** je kiest de uitgave op de stap *Project*. Nieuwe projecten krijgen de uitgave uit *Instellingen › Berekening*.
- **Oudere uitgave:**
  - bedoeld voor vergelijking;
  - resultaten krijgen de status `calculated_legacy_edition` en zijn niet registreerbaar;
  - het rapport, de statusbalk en het EP-Online-overzicht melden dit.
- **Uitgavegebonden invoer:** invoer die alleen in een bepaalde uitgave bestaat, verschijnt alleen onder die uitgave. Een route die een uitgave niet kent, wordt geweigerd met `route_not_in_edition`.
- **Schakelpunten:** elke uitgave is een reeks schakelpunten bovenop de nieuwere uitgave. Elk schakelpunt is vastgelegd met een paginaverwijzing.
- **Uitgave wisselen:** bij een andere keuze toont de app eerst wat de nieuwe uitgave weigert, nieuw eist, negeert of anders leest, met schakelpunt en *Ga naar*. Het overzicht komt van de kern, die het project in beide uitgaven doorrekent (`compare_project_editions`, ook via de HTTP-API, MCP en de desktopapp).
  - *Automatisch aanpassen en wijzigen* voert alleen veilige wijzigingen uit, zoals een veld weglaten dat de nieuwe uitgave weigert. Vereiste waarden vult de app nooit zelf in, en de testnorm van tabel 9.28 blijft bewijs van de adviseur.
  - Eén keer ongedaan maken herstelt de uitgave en alle aanpassingen.
- **Bronlabels:** elke bronvermelding in uitvoer, rapport en dossier noemt de uitgave die werkelijk is berekend.
- **Verschillen tussen uitgaven:** op de openbare gevallen is elk verschil van meer dan 0,5 % tussen opeenvolgende uitgaven herleid tot een schakelpunt (tabel in docs/nta8800-normversies.md).

Details: [Normversies in de handleiding](https://github.com/Erika98-be/open-energy-studio/blob/nta8800-kernel/docs/handleiding-nta8800/10-normversies.md) en [docs/nta8800-normversies.md](https://github.com/Erika98-be/open-energy-studio/blob/nta8800-kernel/docs/nta8800-normversies.md).
