# NTA 8800 §8.2 en bijlagen C, E–I, L: constructies in de Rust-kern

Modules in `crates/nta8800-core/src/`:

| Module | Inhoud |
|---|---|
| `materials.rs` | Rekenwaarden λ_calc en R_calc (bijlage E) en de materialen voor ramen en kozijnen (bijlage H) |
| `constructions.rs` | Dichte constructies (§8.2.2.2, bijlage C, bijlage F), afschotdaken, ΔU_for |
| `window_u.rs` | Ramen, deuren en luiken (§8.2.2.3), ψ en χ (§8.2.3, §8.2.4), bijlage G en bijlage L |
| `forfait_envelope.rs` | Forfaitaire waarden voor bestaande bouw (bijlage I) |
| `envelope_elements.rs` | Eén invoer voor alle elementen, met de beoordeling |

Ontsluiting:

- route: `POST /v1/nta8800/constructions/calculate`
- MCP-tool en Tauri-command: `calculate_constructions`
- TS-client: `calculateConstructionsWithRust`
- fixture: `training-data/nta8800-constructions-synthetic.json`

Bron: NTA 8800:2025+C1:2026, pagina 223–247 en 773–875. De normtekst staat niet in de repository; in de code staan alleen getranscribeerde waarden en formules, met paginaverwijzing.

Status: **ongeverifieerd**. Er zijn nog geen referentiegevallen vergeleken.

## Wat er is uitgewerkt

| Onderdeel | Normdeel | Uitvoering |
|---|---|---|
| Rekenwaarde λ_calc en R_calc | E.3–E.10, tabel E.1–E.5 | Gedeclareerde isolatie met F_T, F_M, F_A en F_conv. Forfaitaire λ (tabel E.10, kolom bestaande bouw; tabel E.11 en E.12). Metselwerk via E.5 of via tabel E.14–E.17. Overige materialen met F_MA. Afronding volgens NEN-EN-ISO 10456. |
| Reflecterende folies | tabel E.13 | Foliepakket d/0,03, cachering 0, en systemen met twee of drie folies. |
| Materialen voor ramen en kozijnen | tabel H.1 | λ_for per materiaal. |
| Overgangsweerstanden | tabel C.2 | R_si naar warmtestroomrichting, R_se = 0,04. |
| Luchtlagen | C.3, tabel C.3 en C.4, C.12 | Niet, zwak en sterk geventileerd, met en zonder reflecterend oppervlak. Bij een sterk geventileerde laag vervallen de buitenste lagen en geldt de stilstaande-luchtweerstand. |
| Smalle en buisvormige holten | tabel F.1–F.3 | Bilineair geïnterpoleerd. Factor 1,2 of 0,8 bij verticale warmtestroom. |
| Kapruimte en onverwarmde ruimte | tabel C.5, C.15 | Als buitenste laag. Standaardwaarden U_e = 2 en n = 0,3. |
| R_T | C.3–C.7, tabel C.1 | Homogene opbouw. Samengestelde opbouw met boven- en ondergrens en a′ volgens de Nederlandse formule C.4. |
| Toeslagen ΔU | 8.8–8.13, tabel 8.2 | ΔU_a, ΔU_fa (via χ of via 8.11/8.12) en ΔU_r. Alleen toegepast boven 3 % van U_T. |
| U_C, R_C, R_eq | 8.4, 8.6, C.1, C.2, C.9, C.10 | Afronding op 2 decimalen. Bij een numerieke berekening: U_T = L_C/A_con (8.5). |
| Afschotdak | C.16–C.22 | Typen 1 t/m 4, tot een helling van 5 %. |
| ΔU_for | 8.3 | Mag niet worden gecombineerd met ψ-waarden in hetzelfde gebouw. |
| Ramen en deuren | 8.14–8.21, tabel 8.3 | Gedetailleerd, vereenvoudigd (max(U1; U2)), via de kozijntabel 8.3 (lineair geïnterpoleerd) en standaarddeuren. |
| Luiken en zonwering | 8.22, 8.23, tabel 8.4 | Met verplichte onderbouwing van de toepassingsvoorwaarden. |
| U_gl en ψ_gl | 8.24, tabel G.1, tabel L.1 en L.2, L.1 | Inclusief het criterium voor een thermisch verbeterde afstandhouder. |
| Uitkomsten van numerieke berekeningen | 8.25–8.29, tabel 8.5 | U_fr, ψ, ψ_gl, ψ_p en χ, plus de toets of een puntvormige thermische brug meetelt. |
| Forfaitaire waarden bestaande bouw | bijlage I | Beslisboom figuur I.4. Tabel I.4–I.7 (regulier, woonwagen, drijvend). Formule I.2 met luchtspouw, riet en thermokussens. Ramen, deuren en panelen (tabel I.8–I.12, I.15, I.16). ψ_for (tabel I.1 en I.2, kolom A en B, standaard 0,5). H_ue = 5·A (I.8). |
| Afronding transparante delen | 8.2.2.1 | Boven 1,0 op 1 decimaal, daaronder op 2 decimalen. |

## Niet ondersteund (expliciete foutcode)

- Luchtlagen dunner dan 20 mm, behalve via bijlage F (`air_cavity_below_20_mm_unsupported`). Luchtlagen dikker dan 300 mm (`air_cavity_above_300_mm_requires_heat_balance`).
- Afschotdaken met een helling boven 5 % (`tapered_roof_above_5_percent_requires_numerical_method`).
- Numerieke 2D- en 3D-berekeningen (§8.6). De kern verwerkt alleen de resulterende L_2D/L_3D (8.25–8.29) of L_C (8.5). Vliesgevels (NEN-EN-ISO 12631), bedrijfsdeuren (NEN-EN 12428) en dakkoepels (NEN-EN 1873) worden als opgegeven productwaarde ingevoerd.
- Bijlage J (statistische bepaling van gedeclareerde waarden) en de grafische schematiseringsregels van bijlage K. A_T, A_con, ℓ_gl en A_fr worden als invoer gevraagd.
- Paneeldiktes buiten 10–300 mm in tabel I.15/I.16 (`panel_thickness_outside_table`).

## Interpretatiekeuzes

Deze keuzes staan ook in de uitvoer (`interpretations`):

1. Bij C.12 met een bekende openingsoppervlakte A_V wordt geïnterpoleerd tussen R_cav;nv en R_cav;sv = 2·R_zv − R_nv. Dat is de waarde die in de tabel ligt opgesloten bij A_V = 1 000 mm².
2. Tabel C.4 bij warmtestroom omlaag: tussenliggende diktes krijgen de waarde van de eerstvolgende kleinere tabeldikte.
3. Tabel F.1–F.3: bilineair geïnterpoleerd. Buiten de tabelassen wordt de randwaarde gebruikt.
4. De 3 %-regel van 8.2.2.2.2 vergelijkt de totale ΔU met U_T.
5. Tabel E.1 neemt de dichtstbijzijnde dichtheid (bij gelijke afstand de hoogste factor). Tabel E.14–E.17 nemen de eerstvolgende hogere dichtheid (conservatief).
6. Formule 8.25 rekent U_p van het vervangende paneel met R_T = R_p = d_p/λ_p, zonder overgangsweerstanden, zoals de formule is geschreven.
7. Tabel I.6 (woonwagens, 1965–1983): voor gevels geldt 0,19. De paneelwaarde 0,04 wordt niet automatisch toegepast.

## Open punten

- De app gebruikt in `src/core/energy/ConstructionCalculator.ts` nog Σd/λ zonder correcties. Vervangen vraagt een andere invoer (materiaalroute per laag, warmtestroomrichting, luchtlagen). Daarom is dat hier niet gedaan; de kernroute is beschikbaar via `calculateConstructionsWithRust`.
- De uitkomsten zijn nog niet gekoppeld aan `direct_transmission` en `monthly_demand`. U-waarden en ψ-waarden worden daar nog los ingevoerd.
