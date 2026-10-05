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
| Rekenwaarde λ_calc en R_calc | E.3–E.10, tabel E.1–E.5 | Gedeclareerde isolatie met F_T, F_M, F_A en F_conv; gedeclareerde R_D via E.4. Forfaitaire λ (tabel E.10, kolom bestaande bouw; tabel E.11 en E.12). Metselwerk via E.5 of via tabel E.14–E.17. Overige materialen met F_MA. Afronding volgens NEN-EN-ISO 10456. |
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

- Luchtlagen dunner dan 20 mm, behalve via bijlage F (`air_cavity_below_20_mm_unsupported`). Tabel C.3/C.4 voetnoot c (p. 784) geeft zelf geen waarden. De voetnoot verwijst naar tabel 8 en bijlage D van NEN-EN-ISO 6946:2017, die niet in de NTA staan. Zo'n spouw voer je in als `resistance`-laag met een R_cav die volgens NEN-EN-ISO 6946 is berekend, met bron. Luchtlagen dikker dan 300 mm (`air_cavity_above_300_mm_requires_heat_balance`).
- Afschotdaken met een helling boven 5 % (`tapered_roof_above_5_percent_requires_numerical_method`).
- Numerieke 2D- en 3D-berekeningen (§8.6). De kern verwerkt alleen de resulterende L_2D/L_3D (8.25–8.29) of L_C (8.5). Vliesgevels (NEN-EN-ISO 12631), bedrijfsdeuren (NEN-EN 12428) en dakkoepels (NEN-EN 1873) worden als opgegeven productwaarde ingevoerd.
- Bijlage J (statistische bepaling van gedeclareerde waarden) en de grafische schematiseringsregels van bijlage K. A_T, A_con, ℓ_gl en A_fr worden als invoer gevraagd.
- Paneeldiktes onder 10 mm, en boven 300 mm bij panelen die niet aan buitenlucht grenzen (tabel I.16) (`panel_thickness_outside_table`). Boven 300 mm aan buitenlucht rekent de kern met de formule achter tabel I.15 (I.4, I.1, 25 % kozijn met U_fr;for, ψ = 0, p. 837–838); die formule reproduceert de tabelrij van 300 mm.

## Validatieregels (foutcodes)

| Regel | Normdeel | Foutcode |
|---|---|---|
| Producten die van nature in situ worden aangebracht (vlokken, parels, gespoten PUR, UF, cellulose, gespoten vlas) moeten F_A uit tabel E.5 gebruiken, met de passende productgroep of "overig" | E.2.1.4.1, tabel E.5 (p. 802–803) | `in_situ_material_requires_in_situ_ageing`, `in_situ_product_mismatch` |
| De indringingsdiepte d_fa ligt binnen de isolatielaag | 8.12 (p. 234) | `fastener_penetration_exceeds_insulation` |
| Tabel F.1 alleen bij U ≤ 1,0; tabel F.2/F.3 alleen bij U > 1,0 | bijlage F | `narrow_cavity_table_f1_requires_u_at_most_1`, `tubular_cavity_tables_f2_f3_require_u_above_1` |
| Opgegeven isolatiesectie bestaat | 8.9/8.11/8.13 | `insulation_section_unknown` |
| Thermisch verbeterde afstandhouder alleen met wegen die aan L.1 voldoen of met een productverklaring | L.3 (p. 868) | `spacer_evidence_required`, `spacer_not_thermally_improved` |
| Eén keuze tussen 8.14 en 8.15 voor alle ramen | 8.2.2.3.1 | `window_formula_8_14_and_8_15_mixed` |
| ΔU_for alleen voor dichte delen van categorie a); geen ramen, deuren, panelen of daklichten | 8.3, 8.2.2.1 | `transparent_element_not_in_supplement`, `door_or_panel_not_in_supplement` |
| Thermokussens: R_ad + 1,8, zonder isolatiedikte | I.2.1.4 (p. 833) | `thermal_cushions_exclude_insulation_thickness` |

## Interpretatiekeuzes

Deze keuzes staan ook in de uitvoer (`interpretations`):

1. Bij C.12 met een bekende openingsoppervlakte A_V wordt geïnterpoleerd tussen R_cav;nv en R_cav;sv = 2·R_zv − R_nv. Dat is de waarde die in de tabel ligt opgesloten bij A_V = 1 000 mm².
2. Tabel C.4 bij warmtestroom omlaag: tussenliggende diktes krijgen de waarde van de eerstvolgende kleinere tabeldikte.
3. Tabel F.1–F.3: bilineair geïnterpoleerd. Buiten de tabelassen wordt de randwaarde gebruikt.
4. De 3 %-regel van 8.2.2.2.2 vergelijkt de totale ΔU met U_T.
5. Tabel E.1 neemt de dichtstbijzijnde dichtheid (bij gelijke afstand de hoogste factor). Tabel E.14–E.17 nemen de eerstvolgende hogere dichtheid (conservatief).
6. Formule 8.25 rekent U_p van het vervangende paneel met R_T = R_p = d_p/λ_p, zonder overgangsweerstanden, zoals de formule is geschreven.
7. Tabel I.6 (woonwagens, 1965–1983): voor gevels geldt 0,19. De paneelwaarde 0,04 wordt niet automatisch toegepast.
8. ΔU_a, ΔU_fa en ΔU_r bij een samengestelde constructie: R_1 en R_T (C.3, zonder thermische bruggen, p. 231–234) komen uit de isolatiesectie. Standaard is dat de sectie met de hoogste C.3-R_T; met `insulationSection` kies je een sectie expliciet. Daarmee hangt de uitkomst niet meer af van de volgorde van de secties.
9. Tabel C.4 voetnoot b: een reflecterende laag die omhoog is gericht (`reflectiveFacingUp`) krijgt geen waarde tussen haakjes, tenzij de spouw hermetisch is afgesloten (`hermeticallySealed`). De kern past dit toe zodra de invoer de laag als omhooggericht markeert, ook bij verticale spouwen.
10. Tabel C.4: bij warmtestroom omhoog met een werkzame reflecterende laag geldt R_se = 0,05 (waarde tussen haakjes). R_C trekt dezelfde R_se af.
11. Tabel F.1 en F.2/F.3 worden getoetst aan de U_C van de hele constructie.
12. §8.2.2.1: de U in H_D (8.1) en in ΔU_for (8.3) is de afgeronde waarde (`uValue`). De onafgeronde waarde staat in `uUnrounded`. Forfaitaire deuren en panelen volgen de afronding van transparante delen.
13. ΔU_for: ventilatieroosters (8.2.2.2.1) vallen onder §8.2.2.2 en mogen meetellen. Ramen, deuren, panelen en daklichten niet.
14. R_calc wordt naar beneden afgerond (E.2.1.1): opgegeven R_calc, E.4 met R_D (`declared_resistance`) en foliepakketten d/0,03.
15. Zoldervloeren (`attic_floor`) gebruiken de vloerrijen van tabel I.4/I.5 met R_si = 0,10 (warmtestroom omhoog). Vloeren boven kruipruimte of op de grond blijven op 0,17.
16. Forfaitaire constructies naar een onverwarmde ruimte (`towardsUnheatedSpace`): in I.1 wordt R_se vervangen door de R_si van die ruimte bij dezelfde warmtestroomrichting (8.4.2.1).
17. `renovation` bij "aanwezig, dikte onbekend" volgt ISSO 82.1/75.1 §8.7.2.1 (afb. 8.14) over de jaarklassen van tabel I.5/I.6. Drijvende gebouwen worden afgewezen (`renovation_year_classes_unavailable`). Het renovatiejaar ligt niet vóór het bouwjaar (`renovation_year_invalid`).
18. C.12 (p. 780): R_cav;sv van een sterk geventileerde luchtlaag staat niet in de tabellen C.3/C.4. De kern leidt hem af uit de kolom "zwak geventileerd" (de tabelwaarde bij A_V = 1 000): R_sv = 2·R_zv − R_nv. Dat is de enige waarde waarmee C.12 bij A_V = 1 000 de tabelwaarde teruggeeft.
19. Tabel C.4 (warmtestroom omlaag): voor een spouwdikte tussen twee rijen neemt de kern de lagere rij. De norm geeft voor deze tabel geen interpolatieregel; de lagere rij is de veilige kant.
20. Samengestelde constructie met een sterk geventileerde spouw (C.3.3, C.5–C.7): elke sectie moet de spouw op dezelfde laagpositie hebben (`composite_strong_cavity_mismatch`). In de bovengrens (C.5) kapt elke sectie af bij de spouw. In de ondergrens (C.6) tellen alleen de lagen binnen de spouw mee, met de stilstaande-lucht-R_se van C.3.3 in plaats van R_se.

## Open punten

- De constructiedialoog heeft een sectie *NTA 8800 U/R_c* die de kern aanroept met de lagen (λ_calc met bron, luchtspouwen, warmtestroomrichting) of met de forfaitaire bijlage I-gegevens. **Toepassen** zet R_c en U van de kern op de projectconstructie; zonder toepassen blijft de eenvoudige Σd/λ uit `ConstructionCalculator.ts` staan. Samengestelde constructies, correcties (ΔU) en ramen gaan nog via de API/JSON.
- De uitkomsten zijn nog niet gekoppeld aan `direct_transmission` en `monthly_demand`. U-waarden en ψ-waarden worden daar nog los ingevoerd.
