# Open Energy Studio — UI-herontwerp

*Status: ontwerpvoorstel, 5 oktober 2026 · branch `nta8800-kernel` · geen code gewijzigd*

Dit document is een volledig herontwerp van de gebruikersinterface. Het gaat niet om een opfrisbeurt. Het is geschreven voor een bouwer die het fase voor fase kan uitvoeren.

**Mockups.** Alle mockups staan als HTML en PNG in [`mockups/`](mockups/) (1600 × 1000). Ze gebruiken één tokenbestand, [`mockups/tokens.css`](mockups/tokens.css). Dat bestand is één-op-één bedoeld als `src/styles/tokens.css`.

**Audit-screenshots.** De screenshots van de huidige app staan in `~/oes-shots/ui-audit/`. Ze zijn gemaakt in NL, op 1600 × 1000 en 1280 × 800, met beide voorbeelden, plus de lichte variant.

| # | Mockup | Bestand |
|---|---|---|
| 0 | Designsysteem (tokens, typografie, componenten) | `mockups/00-designsystem.png` |
| 1 | App-shell + projectoverzicht | `mockups/01-projectoverzicht.png` |
| 2 | Gebouw › Schil & ramen (editor met inspector) | `mockups/02-gebouw-schil.png` |
| 3 | Installaties (systeemketens) | `mockups/03-installaties.png` |
| 4 | NTA-invoerstap Verwarming met controlepaneel | `mockups/04-nta-stap-verwarming.png` |
| 5 | Resultatendashboard (donker / licht) | `mockups/05-resultaten.png`, `05b-resultaten-licht.png` |
| 6 | Rapport & dossier | `mockups/06-rapport-dossier.png` |
| 7 | Basisopname (utiliteit) | `mockups/07-basisopname.png` |
| 8 | Maatwerkadvies | `mockups/08-maatwerkadvies.png` |
| 9 | Toestanden: leeg, laden, fout, verouderd, onvolledig, melding | `mockups/09-toestanden.png` |

---

## 0. Kernbeslissingen in het kort

1. **Een werkstroom in plaats van een ribbon.** De Office-ribbon (8 tabbladen, 1–6 knoppen per tab) verdwijnt. Er komt een vaste linkernavigatie met genummerde stappen:
   - Invoer: Project → Gebouw → Installaties;
   - Berekening: Controle → Resultaten;
   - Bestaande bouw: Basisopname → Maatwerkadvies → Herlabelen;
   - Oplevering: Rapport & dossier → Registratie.

   Elke stap toont zijn status uit de kerncontrole: klaar, aantal aandachtspunten of aantal fouten.
2. **Het NTA-formulier gaat op in de werkstroom.** Het is nu één formulier van 7.700–11.500 px hoog. De 31 secties verhuizen naar de stappen waar ze inhoudelijk horen: zones en gebruik, schil, verwarming, tapwater, ventilatie enzovoort. Per stap verdelen subtabs de secties ("Opwekking · Distributie · Afgifte …"). Eén concept (*draft*) blijft over alle stappen heen bestaan. Er is één plek om toe te passen.
3. **Kernmeldingen worden navigatie.** Elke `gap`/`warning` van de Rust-kern (met `path`) wordt vertaald naar een stap, subtab en veld. Overal staat "Ga naar", en het veld licht op.
4. **Eén contextpaneel rechts (inspector)** vervangt Eigenschappen én Live preview. Het paneel toont wat bij het scherm hoort:
   - de geselecteerde constructie in de schil-editor;
   - het controleoverzicht in een NTA-stap;
   - het dossierlijstje bij het rapport.

   De BENG-indicatoren staan altijd in de statusbalk.
5. **Eén bron van kernresultaten per document.** `useProjectPerformance` wordt één `KernelProvider` op documentniveau. Nav, statusbalk, resultaten en rapport lezen dezelfde query. "Verouderd" en "laden" zijn daardoor overal gelijk.
6. **Designsysteem met eigen tokennamen.** Er komen drie lagen tokens: primitieven → rollen → componenten. De rollen heten `--surface-*`, `--fg-*`, `--line-*`, `--ok/warn/error/info`, `--viz-*`, `--fs-*` en `--space-*`. Daardoor kunnen ze naast de oude `--bg-*`/`--text-*` bestaan zonder ze te overschrijven. De OpenAEC-merkwaarden (#D97706, #36363E, #2A2A32, #FAFAF9 …) blijven, en dus blijven de tests `openaec-*.test.ts` groen.
7. **Grafieken zelf in SVG, geen nieuwe dependency.** Er zijn vijf grafiekvormen nodig: bullet, gestapelde maandkolommen, horizontale balken, labelschaal en labelpad. Dat is ±600 regels. Een chartbibliotheek is niet nodig; `node_modules` heeft er ook geen. Iconen blijven `lucide-react` (al aanwezig).
8. **Onverifieerd ≠ waarschuwing.** De status "onverifieerd / geen attest" krijgt een eigen, rustige kleur (violet), los van geel (aandacht) en rood (fout). Nu is alles amber, waardoor het systeem altijd lijkt te waarschuwen.

---

## 1. Audit van de huidige UI

### 1.1 Inventaris

| Gebied | Component(en) | Waar nu |
|---|---|---|
| Venster | `TitleBar` (eigen titelbalk, Tauri `decorations:false`), `DocumentTabs`, `StatusBar` | boven / onder |
| Commando's | `Ribbon` met tabs Bestand · Start · Gebouwschil · Installaties · Hernieuwbaar · Resultaten · Rapport · 3D Model · Gereedschap; `AppMenu` (Bestand) | boven, 95 px hoog |
| Navigatie | `ProjectBrowser` (boom: Zones › Oppervlakken/Koudebruggen/Puntkoudebruggen, Constructies, Installaties, Hernieuwbaar) | links 260 px |
| Werkvlak | `MainView` schakelt op `viewMode`: `ProjectView`, `EnvelopeView`, `ResultsView`, `ReportView`, `Building3DView`, `UValueCalculator`, `ThermalBridgeCalculator`, `HeatPumpSizingCalculator` | midden |
| Start (`ProjectView`) | `KernelAuditPanel`, `NtaPerformancePanel` (+ `NtaCalculationForm` en JSON-editor), `BasisopnamePanel`, `UnheatedSpacesPanel`, `HeatPumpInventoryPanel`, `GasChainReferencePanel`, samenvattingskaarten | één lange kolom |
| Resultaten (`ResultsView`) | `CalculationNotice`, opnieuw `KernelAuditPanel` en `NtaPerformancePanel`, `MaatwerkadviesPanel` (+ `MwaTemplateEditor`), `RelabelPanel`, `BENGIndicator` × 3, `EnergyBreakdownChart`, `MonthlyBreakdownChart` | één lange kolom |
| Rechts | `PreviewPanel` (live preview: label, BENG, maandbalkjes, kengetallen) óf `PropertiesPanel` | 320 px, standaard aan |
| Dialogen | Project-info (incl. registratie, adres, opname, WLC, evidence), zone, constructie, vlak, raam, lineaire en puntkoudebrug, luchtdichtheid, verwarming, ventilatie, koeling, tapwater, PV, zonneboiler, afdrukvoorbeeld, feedback, instellingen | modaal |
| Diagnosepanelen | `HeatPumpForfait*`, `HeatPumpAux*`, `GasHeatPump*`, `HybridHeatPumpMonthlyPanel` | in de NTA-panelen |
| Overig | `WelcomeScreen`, `SettingsDialog` (thema: systeem/donker/licht/hoog contrast, taal), `ErrorBoundary` | — |

**Flows.** Er zijn zes hoofdflows:
- nieuw project of voorbeeld openen;
- schil en installaties invoeren via ribbonknoppen en dialogen;
- NTA-invoer bewerken (formulier of JSON);
- basisopname starten;
- berekenen, waarna de resultatentab volgt;
- maatwerkadvies, herlabelen, rapport exporteren en dossier-ZIP.

Daarnaast zijn er import en export (UNIEC3, VABI, IFC).

### 1.2 Gemeten feiten

| Meting (beide voorbeelden, 1600 × 1000) | Tussenwoning | Klein kantoor |
|---|---|---|
| Hoogte Start-scherm met NTA-formulier open | 7.697 px | 8.938 px |
| … idem op 1280 × 800 | 9.666 px | 11.474 px |
| Invoervelden in het NTA-formulier | 121 | 161 |
| `fieldset`-secties in het NTA-formulier | 31 | 33 |
| Hoogte met basisopname gestart (1280 breed) | 13.041 px | 14.849 px |
| Velden met opname erbij | 198 | 238 |
| Hoogte Resultaten-scherm | 2.915 px | 2.854 px |

CSS:
- 238 losse hex-kleuren;
- 15 verschillende fontgroottes (8–32 px);
- 10 verschillende radii;
- 81 inline `style={{…}}` in componenten;
- 3 `:focus-visible`-regels en 8 keer `outline: none`;
- geen `prefers-reduced-motion`;
- 8 keer `alert()`/`confirm()` in de UI.

### 1.3 Problemen

**Informatiearchitectuur — waar gebruikers verdwalen**

1. *Het NTA-hart zit op het Start-scherm.* De belangrijkste invoer, de NTA-berekening, staat in een kaart op "Start". Onder de knop "NTA-invoer bewerken" opent een formulier van 31 secties *in* die kaart. Gebouwschil- en Installaties-tabs bewerken een ander datamodel: het vereenvoudigde `zones/heatingSystems`. Verwarming bestaat zo op twee plekken, met twee verschillende invoerdiepten.
2. *Dubbele panelen.* `KernelAuditPanel` en `NtaPerformancePanel` staan zowel op Start als op Resultaten. Op Resultaten volgt daarna nóg een BENG-blok (`BENGIndicator` × 3) en de live preview toont ze een derde keer. Op één scherm (`05-resultaten.png`) staan BENG 1 = 51,4 en 51,37 en "BENG 1" vier keer.
3. *Verkeerde plek voor processtappen.*
   - Maatwerkadvies en herlabelen staan onder "Resultaten", onder het label-overzicht.
   - Registratie (BRL 9500), adres/BAG, opname-triggers, WLC en bewijs zitten in de modale dialoog "Projectgegevens" (`12-projectinfo.png`).
   - Rapport en dossier-export staan zowel in de ribbon als in het rapportscherm.
4. *De ribbon koppelt tab aan weergave.* Een ribbontab kiezen verandert ook het werkvlak (`SET_RIBBON_TAB` → `SET_VIEW_MODE`). "Installaties" en "Hernieuwbaar" tonen echter gewoon Start (`MainView`: `installations|renewables → ProjectView`). De gebruiker klikt "Installaties" en ziet het projectoverzicht.
5. *Geen volgorde of voortgang.* Niets vertelt wat de volgende stap is of wat nog ontbreekt. Een EP-adviseur werkt in vaste volgorde:
   - projectgegevens;
   - schematisering en rekenzones;
   - bouwkundig;
   - installaties;
   - kwaliteitsverklaringen;
   - resultaten;
   - registratie.

   De publieke BENG-rapporten in `scratchpad/publicbeng` volgen precies die indeling (§2.3.1 Algemene gebouwgegevens · 2.3.2 Schematisering en bouwwijze · 2.3.3 Bouwkundige uitgangspunten · 2.3.4 Installatietechnische uitgangspunten · 2.3.5 Kwaliteitsverklaringen). Gangbare EP-pakketten werken met een linkerboom in die volgorde.
6. *Projectboom zonder status.* De boom toont structuur (Zones › Woning › Oppervlakken …). Hij laat niet zien wat compleet is en bevat niets van NTA-invoer, opname of advies.

**Visuele hiërarchie, dichtheid, consistentie**

7. *Alles is amber.* Accent, waarschuwing, "onverifieerd", groepslabels, de statusbalk (volledig oranje) en de actieve tab zijn allemaal oranje. Een echte waarschuwing valt daardoor niet op. Op Resultaten zijn alle drie de BENG-kaarten oranje omrand, ook als BENG 2 niet voldoet (`small_office-1600-f-results-03.png`). De preview gebruikt juist rood/oranje randen. Kleur draagt geen betekenis.
8. *Monospace voor cijfers.* KPI's en kengetallen staan in JetBrains Mono, met spaties in de eenheden: `kWh/m²·jaar`. Dat leest als code, niet als meetwaarde.
9. *Lange proza in kaarten.* Uitleg als "NTA 8800-berekening door de Rust-kern: behoefte (H7), transmissie (H8) …" staat bovenaan elke kaart. Dat is waardevol, maar het hoort in een tooltip of in "Herkomst".
10. *Rommelige maatvoering.* De ribbon is 95 px hoog voor 2–6 knoppen. De lege preview-kolom neemt altijd 320 px. Op 1280 px blijft er 690 px werkvlak over (`*-1280-*`).
11. *Regenboogbalken.* "Energiebalans" kleurt elke rij anders (rood, oranje, geel, groen, paars, cyaan) zonder vaste betekenis.
12. *Een witte strook* tussen werkvlak en preview is een niet-gethemede scrollbalk (`color-scheme`/`scrollbar-color` ontbreekt op `.main-view`).

**Formulierergonomie (NTA-formulier en basisopname)**

13. *Eén scrollende muur.* 31 secties staan onder elkaar, zonder inhoudsopgave, zonder plaats in het geheel en zonder validatie per sectie. "Annuleren/Opslaan" zweven als sticky balk over de inhoud.
14. *Een broncontrol naast elke waarde.* Bij bijna elk veld staat een vrij tekstveld "Bron" ("synthetic survey", "synthetic type plate"). Dat verdubbelt het aantal velden en de bron leest als een gewone waarde.
15. *Het decimaalteken wisselt.* `input type="number"` toont `0.25`, `3.87` en `7.5` met een punt, terwijl de rest van de UI `51,37` met een komma toont. Eenheden staan in het label ("Isolatiedikte, mm") in plaats van in het veld. Draaiknopjes (spinners) staan op velden waar ze geen zin hebben (Rc, oppervlakte).
16. *Herhaalgroepen zonder overzicht.* Ramen, opwekkers, tapwatersystemen en PV worden uitgevouwen fieldsets, zonder lijst of samenvatting.
17. *Opname en model liggen los.* De basisopname (ISSO 82.1/75.1) staat als derde blok op Start. Je ziet niet hoe opname, NTA-invoer en model samenhangen. In het kantoorvoorbeeld telt de opname 1.300 m² aan functies op 400 m² aan zones; dat staat er alleen als tekst.

**Resultaatpresentatie**

18. *Eisen zijn alleen tekst.* "Eis: ≤ 40 kWh/m²·jaar" staat onder een balk zonder schaal. Voldoet of niet blijkt alleen uit kleur, en die kleur is fout (zie 7).
19. *Geen maandgrafiek per dienst of drager in het hoofdscherm.* Alleen de preview heeft 12 piepkleine balkjes (rood = verwarming, blauw = koeling).
20. *Het indicatieve TS-resultaat* ("Bereken BENG") staat gelijkwaardig naast het kernresultaat. Dat verwart, al is het gelabeld.

**Toestanden**

21. *Lege toestanden zijn kale tekst* ("Nog geen NTA 8800-berekening. Kies in het NTA-paneel …").
22. *Laden is onzichtbaar.* De kern rekent met 400 ms debounce, maar er is geen indicator, alleen het verschuiven van cijfers.
23. *Fouten* komen als `alert()` of als een rode balk boven het werkvlak (`calculation-input-error`). Gesloten tabbladen met wijzigingen vragen in het Engels ("Unsaved Changes … Discard").
24. *Een leeg 3D-canvas* zonder melding als WebGL niet beschikbaar is (`*-07-3d.png`).
25. *"Verouderd" bestaat alleen impliciet* (`stale-result-on-project-edit.test.tsx`). Het is niet zichtbaar als toestand.

**Thema's**

26. In licht thema:
    - is `--text-muted` #A1A1AA op #FAFAF9 = **2,45 : 1** (ruim onder AA);
    - is amber als tekst **3,05 : 1**;
    - zijn de statusbalkcijfers amber op amber nagenoeg onleesbaar.
27. In donker thema:
    - is `--text-muted` #71717A op #36363E = **2,48 : 1**;
    - is donkere tekst op amberen knoppen **3,76 : 1**, en wit op amber 3,19 : 1.

**Toegankelijkheid en taal**

28. Focus is nauwelijks zichtbaar (3 regels `:focus-visible`, 8 keer `outline: none`). De boom en de ribbon hebben geen landmarks of rollen. Grafieken hebben geen tekstalternatief of tabel.
29. Er staat Engels in de NL-UI:
    - "Send Feedback";
    - "Browse… No file selected." (native file input bij herlabelen);
    - "Ready" en "Untitled 1";
    - onvertaalde sleutels `model3d.resetView`, `model3d.exportIFC` en `model3d.dragToRotate`;
    - ruwe paden in meldingen (`heatingSystems[0]`, `hotWaterSystems[0]`).
30. Lettertypen komen van fonts.googleapis.com. In de Tauri-build zonder internet valt alles terug op systeemfonts.

---

## 2. Ontwerpprincipes

1. **Betrouwbaar boven indrukwekkend.** Elk getal toont zijn herkomst: eenheid, norm-§, kernversie en vingerafdruk zijn één klik weg. Een cijfer dat bij oude invoer hoort, ziet er ook zo uit. Uitkomsten worden nooit stil vervangen.
2. **Dicht maar rustig.** We gaan uit van een 13,5 px basis, 32 px controls en 36 px tabelrijen, met veel informatie per scherm. Rust komt uit:
   - één accentkleur;
   - neutrale vlakken;
   - uitlijning op een 4 px-raster;
   - één kaartstijl.

   Proza gaat naar tooltip, inspector of "Herkomst".
3. **Status is altijd zichtbaar en eenduidig.** Er zijn vier toestanden met vaste betekenis:

   | Toestand | Kleur |
   |---|---|
   | voldoet / klaar | groen |
   | aandacht | geel |
   | fout / voldoet niet | rood |
   | onverifieerd | violet |

   Kleur gaat altijd samen met een icoon en een woord.
4. **De werkstroom is de navigatie.** De volgorde van het scherm volgt de volgorde van het werk en van het rapport. Wat niet van toepassing is (koeling, bevochtiging, bestaande bouw bij nieuwbouw), is ingeklapt of verborgen, maar wel vindbaar.
5. **De kern is de scheidsrechter.** De UI rekent niet zelf en valideert niet anders dan de kern. De UI vertaalt kernmeldingen naar plekken en woorden.
6. **Toetsenbord eerst.** Alles is bereikbaar via Tab, pijltjes en sneltoetsen, met een commandopalet (Ctrl K) om naar elk veld, elke stap en elke opdracht te springen.

---

## 3. Nieuwe informatiearchitectuur en navigatie

### 3.1 App-shell (`01-projectoverzicht.png`)

```
┌ Topbalk 48px: logo · documenttabs · zoek/commando (Ctrl K) · Opslaan · Herbereken · paneel · vensterknoppen ┐
├ Nav 264px ─────────┬ Werkvlak (paginakop + inhoud) ────────────────────────┬ Inspector 340px (contextueel) ┤
│ projectkaart       │ kruimelpad · titel · lead · acties                      │ selectie / controle / dossier │
│ stappen + status   │ [subtabs of stepper]                                    │ inklapbaar (Ctrl .)           │
│ gereedschap        │ kaarten, tabellen, formulieren                          │                               │
├ Statusbalk 30px: resultaat actueel/verouderd · kernversie · BENG 1/2/3 · TOjuli · label · onverifieerd · opgeslagen ┤
```

**Topbalk.** De topbalk vervangt `TitleBar` + `Ribbon` + `DocumentTabs` (data-tauri-drag-region op de lege delen). Er komen alleen globale acties in: opslaan, herberekenen, contextpaneel en instellingen. Alle andere commando's staan op de pagina waar ze horen of in het commandopalet.

**Statusbalk.** De statusbalk is neutraal (niet oranje) en toont de kernuitkomst compact. Een klik op een chip opent Resultaten.

**Inspector.** De inspector is per pagina gedefinieerd (§3.3). Onder 1360 px breedte wordt hij een zijlade die over het werkvlak schuift; de nav kan dan inklappen tot 56 px met iconen.

### 3.2 Stappen

| Groep | Stap | Subpagina's | Oude bron |
|---|---|---|---|
| Invoer | **1 Project** | Overzicht · Gegevens & adres (BAG) · Adviseur & opdracht | `ProjectView`, `ProjectInfoDialog` (algemeen, adres/object) |
| | **2 Gebouw** | Rekenzones & gebruik · Constructies · Schil & ramen · Koudebruggen · Luchtdichtheid · Onverwarmde ruimten · 3D-model | `EnvelopeView`, zone/constructie/vlak/raam/koudebrug/luchtdichtheid-dialogen, `UnheatedSpacesPanel`, `Building3DView`; NTA-secties algemeen, functies, setpoints, massa, interne winst, ramen, dynamische ramen, serres, dakhellingen, vloeren op grond |
| | **3 Installaties** | Verwarming · Warm tapwater · Ventilatie · Koeling · Bevochtiging · Verlichting (utiliteit) · Opwekking (PV, extern) · Warmtepompen · Gebouwautomatisering | installatiedialogen, `HeatPumpInventoryPanel`, `GasChainReferencePanel`, diagnosepanelen; NTA-secties afgifte/distributie, opwekker, BCRG, verticale leidingen, ventilatie (+ H11), tapwater, zonneboilers, koeling, bevochtiging, verlichting, PV, externe levering, BACS |
| Berekening | **4 Controle** | Controleoverzicht · Weggelaten correcties · Invoer (JSON, geavanceerd) | `KernelAuditPanel`, gaps/warnings uit `NtaPerformancePanel`, JSON-editor |
| | **5 Resultaten** | Overzicht · Per dienst · Per zone · Maandwaarden · Herkomst | `ResultsView`, `PreviewPanel`, `BENGIndicator`, charts, `CalculationNotice`, indicatief TS-resultaat (onder Herkomst) |
| Bestaande bouw *(zichtbaar bij registratietype bestaand, of aan te zetten)* | **6 Basisopname** | per onderdeel (Algemeen … Foto's & bewijs) | `BasisopnamePanel` |
| | **7 Maatwerkadvies** | Maatregelen & pakketten · Gemeten verbruik · Woningpas · Advies & rapport | `MaatwerkadviesPanel`, `MwaTemplateEditor` |
| | **8 Herlabelen** | Vergelijking · Toegestane / niet-toegestane wijzigingen | `RelabelPanel` |
| Oplevering | **9 Rapport & dossier** | Rekenrapport · Invoerdossier · Checklist BRL 9500 · Exports (UNIEC3/VABI/IFC) | `ReportView`/`NtaCalculationReport` (andere agent), `PrintPreviewDialog`, exportknoppen, dossier-ZIP |
| | **10 Registratie** | Registratiegegevens · EP-Online-overzicht · Termijncontrole | `ProjectInfoDialog` (registratie, opname, WLC, triggers, bewijs), `ep-online-overview` |
| Voet | Gereedschap | U-waarde · Koudebrug · Warmtepompdimensionering | Ribbon › Gereedschap |
| Voet | Instellingen | thema, taal, kernadres | `SettingsDialog` |

Elk oud scherm, paneel en elke dialoog heeft dus een vaste plek; er gaat geen functie verloren. Import (UNIEC3/VABI) staat op het projectoverzicht en in het palet. Export staat bij Rapport & dossier, Nieuw/Openen/Opslaan in de topbalk, het welkomstscherm en het palet.

### 3.3 Inspector per pagina

| Pagina | Inspector |
|---|---|
| Projectoverzicht | geen (brede layout) |
| Gebouw-lijsten | het geselecteerde element bewerken (vervangt modale dialogen) |
| Installaties-overzicht | energie per dienst (klein) + snelle toevoegacties |
| NTA-invoerstap | **Controle**: fouten / aandachtspunten / weggelaten, per stap gegroepeerd, met "Ga naar" en het effect op de uitkomst |
| Resultaten | geen, of een detail bij een geselecteerde grafiekreeks |
| Basisopname | uitkomst van de opname + "Overnemen in projectmodel" |
| Maatwerkadvies | het geselecteerde pakket |
| Rapport | dossiercheck BRL 9500 bijlage 3 |

### 3.4 Status per stap (kernaangedreven)

`stepStatus(assessment, project)` levert per stap en subpagina `{ errors, warnings, complete, applicable }`:

- **Fouten**: `assessment.gaps` en `assessment.status === 'invalid'`-issues, toegewezen via het pad.
- **Aandachtspunten**: `assessment.warnings`, plus bewijsgaten (lege `sourceReference`/evidence) en dossierpunten.
- **Compleet**: een niet-lege invoer van die stap zonder fouten.
- **Toepasbaar**: bepaald door gebouwfunctie en registratietype. Bestaande bouw verschijnt alleen bij `registration.type` = bestaand of nadat het handmatig is aangezet.

**Paden toewijzen.** Kernpaden komen voor als `zones[0].surfaces[1].windows[0].gValue`, `ntaCalculation.dynamicWindows[0]`, `/constructions/0/uValue` en `registration.client`. Ze worden genormaliseerd naar `Path = (string|number)[]`; JSON-Pointer en punt/haak-notatie zijn allebei toegestaan. Een prefix-tabel `GAP_ROUTES` in `src/core/nta/gapRoutes.ts` wijst ze toe; de langste prefix wint.

| Pad-prefix | Stap › subpagina |
|---|---|
| `zones[i]` (zonder `surfaces`) · `ntaCalculation.functions/setpoints/thermalMass/internalGains` | Gebouw › Rekenzones & gebruik |
| `constructions` | Gebouw › Constructies |
| `zones[i].surfaces` · `ntaCalculation.windows/dynamicWindows/sunrooms/roofTilts/groundFloors` | Gebouw › Schil & ramen |
| `zones[i].thermalBridges/pointBridges` | Gebouw › Koudebruggen |
| `unheatedSpaces` | Gebouw › Onverwarmde ruimten |
| `heatingSystems` · `ntaCalculation.emission/distribution/generator/bcrg/heatingSystems/verticalPipes/spaceHeatingSolar` | Installaties › Verwarming |
| `hotWaterSystems` · `ntaCalculation.hotWater/dhwSystems/solar` | Installaties › Warm tapwater |
| `ventilationSystems` · `ntaCalculation.ventilation/chapter11` | Installaties › Ventilatie |
| `coolingSystems` · `ntaCalculation.cooling/coolingSystems` | Installaties › Koeling |
| `ntaCalculation.humidifiers` | Installaties › Bevochtiging |
| `ntaCalculation.lighting` | Installaties › Verlichting |
| `solarPV` · `solarThermal` · `ntaCalculation.pvSystems/onSiteProduction/externalSupply/declaredUses` | Installaties › Opwekking |
| `ntaHeatPumps` | Installaties › Warmtepompen |
| `ntaCalculation.bacs` | Installaties › Gebouwautomatisering |
| `basisopname` | Basisopname |
| `maatwerkadvies` | Maatwerkadvies |
| `relabelComparison` | Herlabelen |
| `registration` | Registratie |
| *(geen match)* | Controle › Controleoverzicht (met ruwe code) |

De exacte sleutelnamen worden bij implementatie gecontroleerd tegen `NtaCalculationForm` en de kernfixtures. Een test (`gap-routes.test.ts`) loopt alle `nta.gap.*`-codes uit `kernelCodeLabels.ts` en `nl.ts` langs en eist dat elke code een route of de expliciete fallback heeft.

### 3.5 Navigatiegedrag en sneltoetsen

**Routering.** `viewMode` en `activeRibbonTab` worden één `route: { step, sub?, focusPath? }` in `EnergyContext`. De oude acties `SET_VIEW_MODE` en `SET_RIBBON_TAB` blijven als compatibele aliassen, zodat code en tests die ze dispatchen werken. De URL-hash spiegelt de route (`#/installaties/verwarming`) voor de browserbuild en voor links vanuit rapport of meldingen.

**Ga naar.** De route wordt gezet met `focusPath`. De stappagina scrollt naar `[data-path="…"]`, zet de focus op het veld en laat 1,2 s een accent-ring zien.

**Sneltoetsen.**
- Ctrl K: commandopalet (stappen, velden op label, opdrachten, recente projecten).
- Ctrl ↵: herberekenen.
- Ctrl .: contextpaneel aan of uit.
- Alt ↑/↓: vorige of volgende stap.
- Alt ←/→: vorige of volgende subtab.
- Bestaande sneltoetsen blijven: Ctrl N/O/S/Shift S/W.

**Toetsenbord in de nav.** De nav is een `nav` met `aria-label="Werkstappen"`. De stappen zijn een lijst met `aria-current="page"`; pijltjestoetsen werken binnen de lijst (roving tabindex).

---

## 4. Visuele identiteit

**Logo en naam.** Het logo staat linksboven: een amberen tegel van 26 px met een huis-met-bliksem-glyph, plus "Open Energy Studio" in Space Grotesk 700 met de versie in Inter 10,5 px. Het logo verschijnt verder alleen op het welkomstscherm (64 px) en in de rapportkop.

**Iconen.** Alleen `lucide-react`, met een vaste vertaling per begrip:

| Begrip | Icoon |
|---|---|
| Gebouw / zone | `building-2` / `layers` |
| Verwarming | `flame` |
| Tapwater | `droplet` |
| Ventilatie | `wind` |
| Koeling | `snowflake` |
| PV | `zap` |
| Verlichting | `lightbulb` |
| Controle | `shield-check` |
| Rapport | `file-text` |
| Registratie | `send` |
| Opname | `clipboard-check` |
| Maatwerkadvies | `leaf` |
| Gereedschap | `wrench` |
| Bewijs | `paperclip` |
| Verouderd | `history` |

Maten zijn 14, 16 en 18 px, met stroke 1.75 (2.2 in pills). Iconen nooit los als enige betekenisdrager (altijd met label of `aria-label`).

**Datavisualisatiepalet.** De kleur hoort vast bij de dienst, niet bij de rang, en de volgorde is gevalideerd met de CVD-validator van de dataviz-skill:
- licht: aangrenzend ΔE ≥ 16,3 (CVD) en ≥ 19,6 (normaal zicht);
- donker: ≥ 13,0 en ≥ 19,3.

| Dienst | Licht | Donker |
|---|---|---|
| Verwarming | #EB6834 | #D95926 |
| Koeling | #2A78D6 | #3987E5 |
| Warm tapwater | #1BAF7A | #199E70 |
| Ventilatoren | #4A3AA7 | #9085E9 |
| Verlichting · PV | #EDA100 | #C98500 |
| Bevochtiging | #E87BA4 | #D55181 |
| Hulpenergie | #008300 | #2E9B2E |

Enkele kleuren halen geen 3 : 1 op hun vlak: tapwater, verlichting en bevochtiging in licht, en hulpenergie in donker. Daarom hebben grafieken altijd een legenda met waarden, directe labels bij ≤ 4 reeksen en een tabelweergave. Energiedragers volgen dezelfde vaste volgorde: aardgas (verwarmingskleur), elektriciteit (koelingskleur), warmte extern (tapwaterkleur). Energielabels gebruiken de officiële labelkleuren A++++ → G, en alleen daarvoor.

---

## 5. Designsysteem

### 5.1 Tokens

**Bestand.** Alles staat in `src/styles/tokens.css` (nu als `docs/ui-redesign/mockups/tokens.css`).

**Laag 1 — primitieven.** `--forge-975…600`, `--stone-500…50`, `--amber-700/600/500`, `--orange-600`, `--green-600`, `--red-600`.

**Laag 2 — rollen, donker standaard en `[data-theme="light"]`.**

| Rol | Donker | Licht | Gebruik |
|---|---|---|---|
| `--surface-chrome` | #2A2A32 | #F5F5F4 | topbalk, nav, statusbalk, inspector |
| `--surface-canvas` | #303038 | #F1F0EE | werkvlak |
| `--surface-card` | #36363E | #FFFFFF | kaarten |
| `--surface-raised` | #3E3E46 | #FFFFFF | popover, tooltip, lade |
| `--surface-sunken` | #25252C | #FAFAF9 | invoervelden |
| `--fg-1 / -2 / -3` | #FAFAF9 / #C4C4CB / #A1A1AA | #36363E / #57534E / #6B6560 | tekst; `-3` haalt nog 4,7 : 1 resp. 5,0 : 1 |
| `--fg-on-accent` | #1F1F25 | #1F1F25 | tekst op amber (5,2 : 1; wit haalt maar 3,2 : 1) |
| `--line-subtle / --line / --line-strong` | wit 6 / 10 / 18 % | #ECEAE7 / #E7E5E4 / #D6D3D1 | lijnen |
| `--accent` / `--accent-text` | #D97706 / #F0A43A | #D97706 / #B45309 | primaire actie, actieve stap, focus |
| `--ok`, `--warn`, `--error`, `--info`, `--unverified` (+ `-text`, `-subtle`) | zie bestand | zie bestand | status |
| `--viz-*` | §4 | §4 | grafieken |

**Hoog contrast.** `[data-theme="highContrast"]` blijft bestaan. Het krijgt dezelfde rollen met zwart, wit, geel en dikke lijnen.

**Typografie.** De schaal (`--fs-*`) is 10,5 · 11,5 · 12,5 · **13,5 (basis)** · 15 · 18 · 22 · 30 · 40 px.

| Font | Gebruik |
|---|---|
| Inter | UI; cijfers met `tabular-nums` in tabellen en kolommen |
| Space Grotesk 700 | paginatitels, KPI-cijfers en het labelteken |
| JetBrains Mono | alleen codes, hashes en paden |

Lettertypen worden lokaal meegeleverd (OFL, `src/assets/fonts/*.woff2`, `@font-face` met `font-display: swap`). De Google-link in `index.html` blijft als fallback staan; die test blijft dus geldig.

**Ruimte, vorm, beweging.**
- Ruimte: 4 px-raster (`--space-1…12` = 4 … 48).
- Radii: 3 / 5 / 7 / 10 / pill.
- Elevatie: `--shadow-1` (kaart), `-2` (zwevende balk, segment-aan), `-3` (dialoog, toast).
- Beweging: 120 / 180 ms `cubic-bezier(.2,.7,.2,1)`, en alles uit bij `prefers-reduced-motion`.

**Maatvoering.**
- Controls 26 / 32 / 38 px.
- Tabelrij 36 px.
- Nav 264 px.
- Inspector 340 px.
- Topbalk 48 px.
- Statusbalk 30 px.

### 5.2 Componenten (`src/components/ui/`)

| Component | Kern van de specificatie |
|---|---|
| `Button` | Varianten `primary` (één per gebied), `secondary`, `ghost` en `danger` (tekst rood, vlak neutraal); maten `sm/md/lg`; altijd optioneel een icoon links en een `kbd`-hint. Loading-toestand: de spinner vervangt het icoon en de breedte blijft gelijk. |
| `IconButton` | 32 × 32, verplicht `aria-label`, tooltip na 500 ms. |
| `Field` | label (+ optionele `§`-verwijzing rechts), control, hint of fout. Fout = rode rand + 3 px halo + icoon + tekst, gekoppeld via `aria-describedby`. Heeft `data-path` voor ga-naar. |
| `NumberInput` | Tekstveld met `inputmode="decimal"`; accepteert komma én punt en toont altijd in de locale (nl: komma). De eenheid staat ín het veld, rechts in `--fg-3`. Getallen rechts uitgelijnd. Geen spinners; ↑/↓ = ± stap, Shift = ×10. Leeg → `null` of `undefined` volgens het bestaande `optional`-gedrag, zodat het kernsemantisch gelijk blijft aan `NumberField`. Varianten `derived` (stippelrand, alleen-lezen, "afgeleid") en `forfait` (lege waarde toont de forfaitaire kernwaarde als placeholder). |
| `Select` | Native `<select>` in eigen huls (toetsenbord en Tauri-webview veilig); lange opties krijgen 2 kolommen + `title`, zoals nu. |
| `Segmented` | Voor 2–4 exclusieve keuzes (Wand/Dak/Vloer, Binnen/Buiten, Basis/Alle velden). `role="radiogroup"`. |
| `Check`, `Switch`, `TriState` | TriState = segmented Ja/Nee/Onbekend in plaats van een select. |
| `Card` | Kop (titel, sub, acties rechts), body, optionele secties met `fsec`-scheiding; geen geneste kaarten. |
| `Pill` / `StatusPill` | `ok · warn · err · info · unv · accent · neutral`; de icoon-tekst-combinatie is verplicht voor status. |
| `Tag` | Kleine caps-badge voor soort (WONING, BESTAAND, ISSO 75.1). |
| `Ref` | `§ 9.3.2` in `--fg-3`; klikbaar naar bronnenregister. |
| `Banner` | Inline melding over de breedte: `unv · info · warn · err` (+ actie rechts). |
| `Toast` + `ToastProvider` | Rechtsonder, max. 3. Succes verdwijnt na 6 s; fout blijft. Vervangt alle `alert()`. |
| `ConfirmDialog` | Vervangt `confirm()` en de Engelstalige Tauri-`ask`. Knoppen "Opslaan · Niet opslaan · Annuleren". |
| `Dialog` | Native `<dialog>` (focus-trap, Esc), max 720 px, kop, inhoud en vaste voet. |
| `SideSheet` | Inspector-variant van een dialoog. De bestaande editors (`SurfaceEditorDialog` …) renderen hierin via `DialogShell variant="sheet"`. |
| `Tabs` / `Stepper` | Tabs voor gelijkwaardige weergaven; Stepper voor geordende subtappen met status per stap. |
| `DataTable` | Groepering, selectie (rij-accent links), sticky kop, numerieke kolommen rechts met eenheid in de kop, toetsenbordnavigatie (↑/↓, Enter = inspector). |
| `IssueList` | Ernst-icoon, titel (vertaald), plaats (kruimelpad), technische code klein in mono, en "Ga naar". |
| `EmptyState`, `Skeleton`, `ErrorState`, `StaleBanner` | zie §9. |
| `KpiTile`, `BulletMeter`, `LabelBadge`, `LabelScale` | zie §7. |
| `CommandPalette` | Combobox (`role="combobox"` + listbox) over stappen, `data-path`-velden (label uit i18n), opdrachten en recente bestanden. |

### 5.3 Getallen en eenheden

Er komt één functie, `formatQuantity(value, kind, locale)`, in `src/i18n/format.ts`, naast `formatNumber`. Er komt ook één tabel `QUANTITIES`:

| Grootheid | Eenheid (nl) | Decimalen | Voorbeeld |
|---|---|---|---|
| BENG 1, BENG 2, EP-waarden | kWh/m²·jr | 2 (KPI), 1 (statusbalk) | 51,37 · 51,4 |
| BENG 3, aandelen | % | 1 | 22,3 % |
| TOjuli | K | 2 | 0,85 K |
| Energie per jaar/maand | kWh | 0, duizendtallen met punt | 10.232 kWh |
| Gas | m³ | 0 | 1.015 m³ |
| Oppervlakte vlak / raam | m² | 2 | 30,00 m² |
| Gebruiksoppervlakte A_g | m² | 1 | 100,0 m² |
| U-waarde | W/m²K | 3 | 0,210 |
| Rc | m²K/W | 2 | 4,70 |
| ψ | W/mK | 3 | 0,045 |
| q_v;10 | dm³/(s·m²) | 2 | 0,40 |
| Temperatuur | °C | 0 (ontwerp), 1 (gemeten) | 45 °C |
| Geld | € | 0, € vóór, duizendtallen | € 26.400 |
| Terugverdientijd | jr | 1 | 16,1 jr |

**Regels.**
- Tussen getal en eenheid staat een harde spatie (U+00A0). `%` staat in NL met een spatie ervoor.
- Negatief schrijf je met U+2212 (−).
- Eenheden zijn opgemaakt met super- en subscript: `m²`, `A<sub>g</sub>`, `EP<sub>2</sub>`.
- In tabellen staat de eenheid in de kolomkop en niet in elke cel.
- In het rapport (andere agent) gelden dezelfde decimalen. De tabel is de gedeelde bron.

---

## 6. Herontwerp van het NTA-formulier

Zie `04-nta-stap-verwarming.png`.

### 6.1 Van één formulier naar stappen

`NtaCalculationForm.tsx` (535 regels) en de secties eromheen worden een **sectieregister** `src/components/nta/sections.ts`:

```ts
interface NtaSectionDef {
  id: string;                       // 'heating.generator'
  step: StepId; sub: SubId;         // 'install' / 'heating'
  tab: string;                      // subtab in de stepper: 'Opwekking'
  titleKey: string; refKey?: string;
  paths: Path[];                    // prefixen in ntaCalculation (voor gaps en 'compleet')
  when?: (p: IProject, d: Draft) => boolean;  // residential, zones>1, hotWater != null …
  basic: FC<SectionProps>;          // velden in 'Basis'
  advanced?: FC<SectionProps>;      // velden achter 'Geavanceerd'
}
```

**Toewijzing van de 31 huidige secties** (de namen zoals in de screenshots):

| Stap › subpagina › subtab | Secties |
|---|---|
| Gebouw › Rekenzones & gebruik | Algemeen · Gebruiksfuncties met oppervlakte · Setpoints (7.13) · Thermische massa (7.10) · Interne warmtewinst |
| Gebouw › Schil & ramen | Ramen (zonwinst) · Dynamische ramen (bijl. A) · Aangrenzende onverwarmde serres (7.30b) · Dakhellingen · Vloeren op grond (§8.3) |
| Installaties › Verwarming › Opwekking / Distributie / Afgifte / Regeling & BCRG / Hulpenergie / Zonneverwarming | Opwekker · Afgifte en distributie (§9.3/9.4) · Verticale leidingen (7.3.3) · BCRG-verklaringstabel · Verwarmingssystemen per zone · Zonneverwarming zonder tapwatersysteem |
| Installaties › Warm tapwater | Warm tapwater (§13) · Extra tapwatersystemen (§13.2.4) · Zonneboilers (§13.7) |
| Installaties › Ventilatie | Ventilatie · H11 gebouw · Ventilatiesysteem (11.5) · Infiltratie (11.2.5) · Open verbrandingstoestellen (11.2.4) · Ventilatieve koeling (11.2.3.3) · Voorverwarming toevoerroosters (11.3.2.9) · Ventilatoren (11.4) |
| Installaties › Koeling / Bevochtiging / Verlichting | Koeling (§10.5) · Bevochtiging (H12) · Verlichting (utiliteit) |
| Installaties › Opwekking | PV (§16) · Externe levering · Productgeneratoren · Bijlage P |
| Controle | Bevestigingen (inventaris compleet) · JSON-editor ("Geavanceerd (JSON)") |

### 6.2 Concept, toepassen, ongedaan maken

`NtaDraftProvider` (per document) houdt het `Draft` uit `NtaFormFields.ts` vast; `read`/`write` blijven gelijk. Een stap bewerkt het gedeelde concept, en wisselen van stap verliest niets. Onderaan het werkvlak staat één zwevende balk: "*n* wijzigingen … · Ongedaan maken · Vorige stap · **Toepassen en verder**". Toepassen schrijft het concept in `project.ntaCalculation`, net als de huidige Opslaan-knop. Daarna rekent de kern (debounce 400 ms).

Bij het verlaten van een document met een niet-toegepast concept vraagt een `ConfirmDialog` "Toepassen · Verwerpen · Annuleren".

*Waarom geen autosave per veld?* Het kernblok wordt als geheel gevalideerd, en halve invoer geeft een golf van gaps tijdens het typen. Het expliciete toepassen blijft. Het "effect op uitkomst" in de inspector rekent wél al op het concept (een preview-run met `calculateProjectPerformanceShared({...project, ntaCalculation: draft})`). Het resultaat is daar duidelijk gemarkeerd als *concept*.

### 6.3 Progressieve onthulling

- **Basis / Alle velden** (segmented in de paginakop). Basis toont de velden die het voorbeeld en een gewone woning of utiliteit nodig hebben. De rest staat achter per-sectie "Geavanceerd · *n* velden — opsomming · alle op standaard". Een geavanceerd veld met een afwijkende waarde of een kernmelding klapt automatisch open.
- **Niet van toepassing = niet zichtbaar.** Voorbeelden: verlichting alleen bij utiliteit; tweede opwekker achter "Opwekker toevoegen"; serres achter "Serre toevoegen". De bestaande `when`-condities uit het formulier gaan mee naar het register.
- **Herhaalgroepen worden lijst + detail.** Ramen, opwekkers, tapwatersystemen, PV, maatregelen en opname-elementen worden een compacte lijst met per regel de samenvatting en status. De geselecteerde regel klapt uit of opent in de inspector.
- **Bron & bewijs per sectie.** Alle `sourceReference`-velden van een sectie gaan in één blok "Bron & bewijs" onderaan die sectie. Elk blok is één regel per onderwerp ("Toestel", "Opstelling") met documentkeuze (paperclip) of vrije tekst. Het datamodel verandert niet; alleen de presentatie. Ontbrekend bewijs is een aandachtspunt (geel), geen fout.

### 6.4 Validatie en "ga naar"

- **Inline per veld.** Een kernmelding met een pad naar dit veld geeft een rode of gele rand en de vertaalde melding onder het veld (`KernelCode`/`KernelDetail` blijven de tekstbron).
- **Per subtab.** De stepper toont ✓, een geel getal of een rood getal.
- **Per stap.** De nav toont dezelfde tellingen (§3.4).
- **Overzicht.** De inspector "Controle" telt fouten, aandachtspunten en weggelaten correcties en toont de lijst gegroepeerd per stap. Klik = `navigate({step, sub, focusPath})`. De pagina *Controle* toont dezelfde lijst schermbreed, met filter op ernst en stap en met de technische code en het kernpad uitklapbaar (voor support).
- **Mechanisme.** Elk `Field` krijgt `data-path={JSON.stringify(path)}`. `GAP_ROUTES` (§3.4) bepaalt stap en tab; daarna zoekt de pagina het veld. Valt het binnen een ingeklapte "Geavanceerd" of een lijstregel, dan klapt die eerst open.

### 6.5 Eenheden en invoer

Alle `NumberField`s worden `NumberInput` met eenheid (§5.2), met de eenheid uit de labeltekst gehaald. Voorbeeld: "Isolatiedikte, mm" → label "Isolatiedikte", eenheid "mm". Dat kan zonder dat de i18n-sleutels veranderen: er komen nieuwe sleutels `*.unit` en de oude labels blijven als fallback.

**Afgeleide waarden** staan als `derived` naast de invoer, bijvoorbeeld netto oppervlakte en Rc → U.

**Forfaitaire waarden** staan als placeholder, met "forfaitair (tabel 9.27)" als hint.

---

## 7. Resultatendashboard

Zie `05-resultaten.png` en `05b-resultaten-licht.png`.

**Kop.** De kop toont berekeningstijd, kernversie, normversie en de invoer-vingerafdruk. De weergaven zijn Overzicht · Per dienst · Per zone · Maandwaarden · Herkomst. Een smalle `unv`-banner meldt "Onverifieerde berekening — geen attest, geen geregistreerd label".

**Rij 1 — label + vier toetsen.**
- `LabelBadge`: officiële kleur, EP₂ met de klassengrenzen eronder.
- `LabelScale`: A++++ → G, met de actieve klasse uitgelicht.
- `KpiTile` met `BulletMeter` voor BENG 1, BENG 2, BENG 3 en TOjuli. Elke tegel toont:
  - de waarde (Space Grotesk 30 px, 2 decimalen) en de eenheid;
  - een pill "Voldoet ✓" of "Voldoet niet ✕";
  - de bullet: een balk tot de waarde, met een harde streep op de eis (≤ of ≥). De balk is groen als hij voldoet en rood als niet;
  - de regel "Eis ≤ 30,0 · +50,6 boven eis".

  De schaal loopt van 0 tot max(1,5 × eis, 1,1 × waarde). Bij BENG 3 (≥) blijft de vulrichting gelijk; alleen de tekst "tekort/overschot" draait om.
- Bronvelden:

  | Tegel | Velden |
  |---|---|
  | BENG 1 | `performance.needIndicatorKwhPerM2Year`, eis `bblCheck.limits.energyNeedMaxKwhPerM2` |
  | BENG 2 | `primaryFossilIndicatorKwhPerM2Year`, eis `primaryFossilMaxKwhPerM2` |
  | BENG 3 | `renewableSharePercent`, eis `renewableShareMinPercent` |
  | TOjuli | `tojuliMaxK`, `tojuliMeetsBblLimit` |
  | Label | `indicativeLabelClass`, `labelPrimaryFossilIndicatorKwhPerM2Year` |

**Rij 2 — grafieken.**
- *Energiegebruik per maand.* Gestapelde kolommen per dienst uit `energyByService.months` (`usedKwh` per `service`), met PV-opwek uit `pvSystems[].monthlyKwh` als kolommen *onder* de nullijn. Dat is één as en één eenheid, dus geen dubbele as. Een segmented control wisselt de meetgrootheid tussen Gebruik, Primair fossiel (`primaryFossilKwh`) en Per drager. Hover geeft een maandtooltip. Er is een tabelknop.
- *Netto behoefte.* Warmte (`spaceHeating.demand.monthly[].heating.needKwh`) en koude (`…cooling.needKwh`), gestapeld per maand.
- *Aandachtspunten.* Eisen die niet gehaald worden, met een korte, uit de data afgeleide duiding ("Gasketel + gas-tapwater bepalen 99 % van EP_fossiel", berekend uit het aandeel per dienst in `primaryFossilKwh`), plus kernwaarschuwingen.

**Rij 3 — kengetallen.** Primair fossiel, hernieuwbaar, finale energie, CO₂ en ZEB, elk in een kleine tegel.

**Per dienst.** Horizontale balken per dienst × drager (`energyByService.annual`), plus de tabel uit het huidige "Energie per dienst".

**Per zone.** Behoefte en TOjuli per zone (`performance.tojuli[]`).

**Maandwaarden.** De bestaande maandtabellen.

**Herkomst.** Vingerafdrukken, kern- en normversie, weggelaten correcties, het BBL-bronartikel en het indicatieve TS-resultaat ("vereenvoudigde schatting, niet NTA") met de bestaande badge en tests.

**Een Sankey** (gevraagd als optie) staat op de "Per dienst"-weergave als fase-8-optie: verliezen/winsten → behoefte → geleverd per drager. Het is gebouwd als eenvoudige SVG met hooguit 3 kolommen. Hij is niet nodig voor de kern van het dashboard.

**Toegankelijkheid.**
- Elke grafiek is een `<figure>` met `role="img"` en een `aria-label` dat de som noemt, plus een tabelknop.
- Waarden staan in de legenda.
- Kleur staat nooit alleen (§4).

---

## 8. Overige schermen

**Projectoverzicht** (`01`). Voortgang per stap (balk, telling, klik = ga naar), open punten met ga-naar, uitkomst (label, schaal, mini-bullets) en projectgegevens. Dit vervangt het huidige Start-scherm.

**Gebouw** (`02`). Per subpagina een `DataTable` met groeperen op oriëntatie, type of constructie, en filters. Het kompasglyph geeft de oriëntatie. Ramen staan als kindregels onder hun vlak. De inspector bewerkt de selectie; dat zijn de bestaande dialogen als `SideSheet`.

Onder de tabel staan twee kaarten:
- het aandeel H_T per elementtype;
- de controles voor de schil (kernaudit: thermische grenzen, koudebruggen forfaitair).

"Vlak toevoegen" opent de inspector met een leeg vlak. De boom van `ProjectBrowser` verdwijnt: lijst plus kruimelpad vervangen hem.

**Installaties** (`03`). Per dienst een kaart met de *systeemketen*: Opwekking → Opslag → Distributie → Afgifte (→ Regeling). Elk schakel-tegeltje is klikbaar naar de bijbehorende subtab van de NTA-stap. Status en ⋯-menu (dupliceren, verwijderen) staan rechtsboven. Diensten die niet aanwezig zijn, krijgen een gestippelde lege kaart met uitleg en een toevoegknop. De diagnosepanelen voor warmtepomp en gaswarmtepomp gaan naar Installaties › Warmtepompen als uitklapbare "Diagnose"-kaarten.

**Basisopname** (`07`). Een eigen pagina met links de onderdelen en hun voortgang, en midden de elementkaarten. Het geselecteerde element is open; de rest is samengevat op één regel.

- Isolatie is een segmented control: *Bekende dikte · Rc bekend · Onbekend*.
- De afgeleide Rc/U staat als info-pill met de ISSO-tabel erbij.
- Er zijn fotovakken. Foto's worden in het dossier opgenomen; het opslagmodel komt uit de bestaande evidence-structuur. Komt er later een nieuw veld bij, dan is dat een aparte beslissing.
- "Kopieer vorige" en "Locatie-modus": grotere doelen (44 px) en één element per scherm, voor tablet op locatie.

De inspector toont de indicatieve uitkomst van de opname, het aantal forfaitaire waarden en consistentiefouten (A_g functies ≠ zones). De primaire actie is **Overnemen in projectmodel**; het bestaande gedrag van `BasisopnamePanel` blijft de bron.

**Maatwerkadvies** (`08`). Tabs: Maatregelen & pakketten · Gemeten verbruik · Woningpas · Advies & rapport.
- *Labelpad*: kolommen EP₂ per variant met labelbadge. Het geadviseerde pakket is amber; de rest is neutraal.
- *Pakketvergelijking*: een tabel met sorteer-segmented NCW / TVT / invoervolgorde (`mwa.order.*`).
- *Maatregelentabel*: per maatregel de deelname aan pakket 1, 2 en 3.
- De inspector toont het gekozen pakket: label van → naar, zes kerncijfers, fasering en motivatie, plus waarschuwingen uit de woningpas.
- De sjablooneditor (`MwaTemplateEditor`) opent in een `SideSheet`.

**Herlabelen.** Bovenaan staat het bronbestand kiezen. Dat wordt een eigen knop "Oorspronkelijk projectbestand kiezen…" met een vertaalde bestandsnaam, in plaats van de native `<input type=file>`. Daaronder drie kolommen:
- toegestaan (6a);
- niet toegestaan (6b);
- te beoordelen.

Elk item heeft een veldpad dat met "Ga naar" opent, plus de migratiemelding (nu een `alert`) als `Banner`.

**Rapport & dossier** (`06`). Het scherm is een *frame* rond het rapport dat de andere agent bouwt (`NtaCalculationReport` met niveaus samenvatting, standaard en gedetailleerd):
- links: instellingen (niveau, taal, hoofdstukken);
- midden: de paginavoorbeelden met miniaturen;
- rechts: de dossiercheck (BRL 9500 bijlage 3);
- kop: exportknoppen HTML, Afdrukken/PDF en Projectdossier (ZIP).

Tabs: Rekenrapport · Invoerdossier · Checklist · Exports (UNIEC3/VABI/IFC).

**Afspraak met de rapport-agent.** Het frame geeft `level` en `locale` door en toont wat het rapport rendert. Het herdefinieert geen inhoud, geen secties en geen opmaak in het rapport. De hoofdstuk-schakelaars zijn alleen actief als het rapport een `sections`-prop aanbiedt; zo niet, dan vervallen ze. `PrintPreviewDialog` wordt deze pagina.

**Registratie.** De registratiesecties uit `ProjectInfoDialog` worden een pagina:
- registratie (doel, opname, representativiteit, berichttype);
- object/BAG;
- opnametriggers;
- WLC/GWP;
- bewijs;
- het EP-Online-overzicht.

Bovenaan staat een blokkerende banner zolang er geen attest is: "Registratie in EP-Online kan pas met een attest volgens BRL 9501". De velden blijven invulbaar.

**Gereedschap.** De U-waardecalculator, de koudebrugcalculator en de warmtepompdimensionering houden hun eigen werkvlak (ze zijn al het best vormgegeven deel). Ze krijgen de nieuwe tokens. Vanuit Constructies en de schil-inspector komt een link "Open in U-waardecalculator", met overnemen terug.

**3D-model.** Een toolbar met vertaalde knoppen (de ontbrekende `model3d.*`-sleutels toevoegen) en een `ErrorState` als WebGL niet beschikbaar is.

**Welkomstscherm.** Recente projecten (bestandspad, datum, label), Nieuw, Openen, Voorbeeld tussenwoning, Voorbeeld klein kantoor en Importeren (UNIEC3/VABI). Het scherm verschijnt alleen als er geen document open is.

**Instellingen.** Een `Dialog` met de tabs Algemeen (thema, taal), Kern (adres/poort met verbindingstest) en Over.

---

## 9. Toestanden (`09-toestanden.png`)

| Toestand | Wanneer | Ontwerp |
|---|---|---|
| **Leeg** | geen zones, geen systemen, geen pakketten … | `EmptyState`: icoon in accent-vlak, titel, één zin uitleg, primaire en secundaire actie (vaak "Importeren"). Nooit een lege tabel. |
| **Laden** | kernrun bezig (`query.kind === 'loading'`) | De vorige uitkomst blijft staan met een kleine spinner "Rust-kern berekent…" in de statusbalk en de kaartkop. Een skeleton verschijnt alleen als er nog geen vorige uitkomst is. |
| **Verouderd** | project gewijzigd na de laatste geslaagde run, of een niet-toegepast concept | De statusbalk-pill wordt geel "Resultaat verouderd". KPI's worden gedimd (60 %, desaturatie). `StaleBanner` met "Herbereken". Uitkomsten worden nooit stil vervangen. |
| **Onvolledig / ongeldig** | `assessment.status` `incomplete`/`invalid` | `Banner err` "Uitkomst achtergehouden" + `IssueList` met ga-naar. BENG, label en grafieken worden niet getoond (bestaand KernelVerdict-gedrag). |
| **Fout** | kern of HTTP niet bereikbaar, uitzondering | `ErrorState`: wat er mis is, wat bewaard blijft, "Opnieuw proberen" en "Details kopiëren". De `ErrorBoundary` gebruikt dezelfde component. |
| **Melding** | actie gelukt of mislukt (export, import, opslaan) | `Toast`; vervangt `alert()`. |
| **Niet van toepassing** | dienst ontbreekt, bestaande bouw bij nieuwbouw | gestippelde kaart met uitleg en een "toevoegen"-actie (`03`). |

---

## 10. Toegankelijkheid, i18n, platform

**Contrast.** Alle tekstrollen halen WCAG AA (≥ 4,5 : 1) op hun vlak. De berekende waarden staan in het tokenbestand. Grote KPI-cijfers en iconen halen ≥ 3 : 1. Hoog contrast blijft een thema.

**Focus.** Een globale `:focus-visible { box-shadow: var(--focus-ring) }`. Er komt geen `outline: none` zonder vervanging.

**Landmarks.** `header`, `nav[aria-label]`, `main`, `aside[aria-label="Contextpaneel"]` en `footer[role=status]`. Paginatitels zijn `h1`, kaarttitels `h2`/`h3`.

**Live regions.** De statusbalk-pill gebruikt `aria-live="polite"` voor "Resultaat actueel/verouderd". Toasts met een fout gebruiken `role="alert"`.

**Toetsenbord.** Zie §3.5. Alle dialogen gebruiken native `<dialog>`.

**Beweging.** Alles uit bij `prefers-reduced-motion`.

**i18n.**
- Alle nieuwe teksten krijgen sleutels in `nl.ts` en `en.ts`; de 12 andere talen vallen terug op `en` (`fallbackLng: 'en'`, zoals nu).
- Er komt een test die alle `t('…')`-sleutels uit `src/components` in nl en en eist. Die vangt fouten als `model3d.*`.
- Hardgecodeerd Engels verdwijnt: Send Feedback, Ready, Untitled, de unsaved-dialoog, alerts en "IFC Export".
- Getallen lopen via `formatQuantity`.

**Platform.**
- Tauri: eigen titelbalk blijft (`decorations:false`), met `data-tauri-drag-region` op de lege delen van de topbalk.
- Browserbuild: werkt ook (hash-routing, file-input-fallback zoals nu).
- Lettertypen lokaal.
- Geen nieuwe runtime-dependency.

---

## 11. Implementatieplan

### Uitgangspunten

**Wat blijft.** Al het gedrag blijft:
- reducers en acties in `EnergyContext`;
- `KernelClient`;
- serialisatie en kernstempel;
- import en export;
- `KernelVerdict`-regels (achterhouden bij incompleet);
- indicatief TS-resultaat met badge;
- herlabelmigratie;
- alle tests in `src/__tests__` (74 bestanden, inclusief helpers).

Testbestanden die de ribbon of de projectboom als UI testen (`ribbon.test.tsx`, `project-browser.test.tsx`, delen van `walkthrough-*`, `ui-walkthrough-2`, `titlebar.test.tsx`) worden in de fase waarin die component verdwijnt *herschreven naar de nieuwe navigatie, met dezelfde beweringen*: elke ribbonactie roept nog dezelfde handler of opent nog dezelfde editor. Geen test wordt geschrapt zonder vervanger met dezelfde dekking.

**Wat blijft staan in de stijlen.** De OpenAEC-tests pinnen waarden in `index.css` en `SettingsDialog.tsx`. Die blijven staan.

**Afhankelijkheden.** Er komt geen nieuwe runtime-dependency.

| Behoefte | Oplossing |
|---|---|
| Iconen | `lucide-react` (aanwezig) |
| Zip | `fflate` (aanwezig) |
| Grafieken | eigen SVG |
| Dialoog | native `<dialog>` |
| Fonts | woff2-bestanden in `src/assets/fonts` (OFL), geen npm-pakket |

**Waarom geen chartbibliotheek.** Recharts of visx zouden ~100 kB+ gz en d3-afhankelijkheden toevoegen voor vijf eenvoudige vormen.

**Per fase.**
- Groen `npm run test` en `npm run build`.
- Screenshots van beide voorbeelden (NL, 1600 en 1280, donker en licht) met de bestaande puppeteer-scripts. Die vergelijk je met de mockups.

**Volgorde.** F1 → F2 → F3 → F4 is een strikte reeks. Daarna kunnen F5 t/m F9 parallel, elk in een eigen worktree, want ze raken disjuncte pagina's. F10 sluit af.

### F1 — Tokens, fonts, basis-CSS (klein, geen gedragswijziging)

- **Nieuw**:
  - `src/styles/tokens.css` (uit `mockups/tokens.css`);
  - `src/styles/base.css` (reset, `:focus-visible`, scrollbars `scrollbar-color` + `color-scheme`, reduced motion, `tabular-nums`-utility);
  - `src/assets/fonts/*.woff2` + `@font-face`.
- **Wijzig**:
  - `src/main.tsx` (imports);
  - `src/index.css`: oude variabelen blijven. `--text-muted` gaat naar de contrastwaarden (#A1A1AA donker, #6B6560 licht); die is niet gepind. De statusbalk wordt neutraal (`--surface-chrome`).
  - `src/components/SettingsDialog/SettingsDialog.tsx`: ongewijzigd.
- **Acceptatie**:
  - witte scrollstrook weg;
  - contrastwaarden gehaald;
  - fonts laden offline (Tauri);
  - `openaec-*.test.ts` groen.

### F2 — UI-primitieven en meldingen

- **Nieuw**:
  - `src/components/ui/`: `Button`, `IconButton`, `Field`, `NumberInput`, `Select`, `Segmented`, `Check`, `Switch`, `TriState`, `Card`, `Pill`, `StatusPill`, `Tag`, `Ref`, `Banner`, `Toast`, `ToastProvider`, `ConfirmDialog`, `Dialog`, `SideSheet`, `Tabs`, `Stepper`, `DataTable`, `IssueList`, `EmptyState`, `Skeleton`, `ErrorState`, `StaleBanner`, `Kbd` (+ `ui.css`, `index.ts`);
  - `src/i18n/format.ts`: `formatQuantity` + `QUANTITIES`;
  - `src/core/nta/pathUtil.ts`: `parseKernelPath`.
- **Wijzig**:
  - `App.tsx`: alle `alert()`/`confirm()`/`ask` → toasts en `ConfirmDialog`;
  - `DialogShell.tsx`: prop `variant: 'modal' | 'sheet'`.
- **Tests (nieuw)**:
  - `NumberInput` (komma/punt, leeg → null/undefined, ↑/↓);
  - `formatQuantity` per grootheid;
  - `ConfirmDialog` bij sluiten met wijzigingen;
  - de i18n-sleuteltest.
- **Bestaande tests.** Alle 8 `alert()`/`confirm()`-aanroepen zitten in `App.tsx`. Tests die dat gedrag raken (openen, sluiten met wijzigingen, import) gaan naar toasts of dialoog, met dezelfde inhoudelijke beweringen.
- **Let op `role="alert"`.** Ongeveer twintig testbestanden zoeken meldingen via `getByRole('alert')`, bijvoorbeeld `invalid-floor-area`, `kernel-audit-ui`, `nta-performance-panel` en `registration-dialog`. `Banner err/warn`, `ErrorState` en inline veldfouten houden daarom `role="alert"` (of `role="status"` waar dat nu zo is). Neem die rol niet weg.

### F3 — Eén kernquery per document + stapstatus

- **Nieuw**:
  - `src/context/KernelProvider.tsx`: per actief document één `useProjectPerformance`, met de vorige geslaagde uitkomst en de vlag `stale`;
  - `src/core/nta/gapRoutes.ts` (`GAP_ROUTES`, `routeForPath`);
  - `src/core/nta/stepStatus.ts`;
  - `useKernel()`.
- **Wijzig**: `ResultsView`, `ReportView`, `PrintPreviewDialog`, `PreviewPanel` en `NtaPerformancePanel` lezen `useKernel()`. Het `suppliedQuery`-patroon blijft compatibel.
- **Tests**:
  - `gap-routes.test.ts` (alle `nta.gap.*`-codes routeerbaar);
  - `step-status.test.ts` met beide voorbeelden;
  - `preview-kernel`, `results-view-kernel` en `stale-result-on-project-edit` blijven groen.

### F4 — App-shell en navigatie

- **Nieuw**:
  - `src/components/shell/AppShell.tsx`;
  - `TopBar.tsx` (vervangt `TitleBar` + `DocumentTabs`; drag-region en vensterknoppen worden uit `TitleBar` overgenomen);
  - `WorkflowNav.tsx`;
  - `Inspector.tsx`;
  - `StatusBar.tsx` (nieuw ontwerp);
  - `CommandPalette.tsx`;
  - `src/core/navigation/routes.ts` (`StepId`, `SubId`, hash-sync).
- **Wijzig**:
  - `EnergyContext.tsx`: `route` + aliassen `SET_VIEW_MODE`/`SET_RIBBON_TAB` → `navigate`;
  - `MainView.tsx` → `StepRouter`;
  - `App.tsx` (shell, sneltoetsen Ctrl K/↵/.).
- **Verwijder**:
  - `Ribbon`, `ProjectBrowser`, `PropertiesPanel` (de laatste gaat naar de inspector);
  - `AppMenu`: de inhoud gaat naar het palet en het welkomstscherm; Bestand-acties staan in de topbalk-overloop.

  Er blijven tijdelijk re-exports totdat de tests zijn omgezet.
- **Tussenstand.** Tijdens deze fase tonen de stappen nog de bestaande views (Start → Projectoverzicht met de bestaande panelen, Gebouw → `EnvelopeView`, Resultaten → `ResultsView` …). Elke stap is dus meteen bruikbaar.
- **Tests**:
  - `ribbon.test.tsx` → `workflow-nav.test.tsx` (zelfde acties: zone/vlak/raam/koudebrug/luchtdichtheid/verwarming/ventilatie/koeling/tapwater/PV/zonneboiler-editor openen; berekenen; rapport exporteren/afdrukken; UNIEC3/VABI);
  - `project-browser.test.tsx` → lijsten in Gebouw/Installaties;
  - `titlebar.test.tsx` → `top-bar.test.tsx`;
  - nieuw: palet en toetsenbord-nav.

### F5 — Gebouw en Installaties (parallel)

- **Nieuw**:
  - `src/pages/building/{ZonesPage, ConstructionsPage, EnvelopePage, ThermalBridgesPage, AirTightnessPage, UnheatedPage, ModelPage}.tsx`;
  - `src/pages/installations/{InstallationsOverview, SystemChainCard}.tsx`.
- **Wijzig**:
  - `EnvelopeView` gaat op in `EnvelopePage` (`DataTable` + groepering);
  - editor-dialogen renderen als `SideSheet` in de inspector;
  - `UnheatedSpacesPanel`, `HeatPumpInventoryPanel`, `GasChainReferencePanel` en de diagnosepanelen verhuizen naar hun subpagina;
  - `Building3DView`: `ErrorState` + i18n.
- **Tests**: `envelope-edit`, `thermal-boundary-editors`, `point-thermal-bridges`, `heat-pump-inventory`, `unheated-spaces-ui`, `heat-pump-*-ui`, `gas-*-ui` en `exterior-surface-resistance` blijven inhoudelijk gelijk. Alleen de weg naar het scherm (render via route) verandert.

### F6 — NTA-invoer in stappen (parallel, grootste fase)

- **Nieuw**:
  - `src/components/nta/sections.ts` (register, §6.1);
  - `NtaDraftProvider.tsx`;
  - `NtaStepPage.tsx` (stepper + secties + zwevende toepasbalk);
  - `ValidationPanel.tsx` (inspector);
  - `ControlPage.tsx` (stap 4, met de bestaande JSON-editor);
  - `SourceEvidenceBlock.tsx`.
- **Wijzig**:
  - `NtaFormFields.tsx`: velden gebruiken `ui/Field` en `NumberInput` en krijgen `data-path`; de API (`draft`, `path`, `onChange`) blijft gelijk;
  - `NtaCalculationForm.tsx` wordt een dunne wrapper die alle secties onder elkaar rendert. Die blijft bestaan voor de tests en voor "Alle secties"-afdruk;
  - `NtaSystemSections`, `NtaVentilationSection`, `NtaExtraSections`, `NtaAdvancedSections`, `NtaProjectExtras`, `NtaExternalSupply`, `NtaProductGenerators`, `NtaDynamicWindows`, `NtaAnnexPDetails`, `NtaLightingDetails` en `NtaPvFields` krijgen een opsplitsing in `basic`/`advanced`. De velden en paden blijven gelijk.
  - `NtaPerformancePanel`: de uitkomsten gaan naar Resultaten, de gaps naar Controle, en de knoppen "NTA-invoer bewerken/Geavanceerd" worden routes.
- **Tests**:
  - alle `nta-*.test.tsx` blijven groen; ze renderen het formulier via de wrapper of via de stappagina en gebruiken dezelfde labels;
  - nieuw: ga-naar (gap → stap → veld gefocust en uitgeklapt), het concept blijft bewaard over stappen heen, en Toepassen = huidige Opslaan.

### F7 — Resultatendashboard (parallel)

- **Nieuw**:
  - `src/components/charts/{BulletMeter, KpiTile, LabelBadge, LabelScale, MonthlyStackChart, HBarChart, ChartTable, Legend, Tooltip}.tsx` (pure SVG; props = getallen + tokens);
  - `src/pages/results/{ResultsOverview, ByService, ByZone, Monthly, Provenance}.tsx`;
  - `src/core/nta/resultSeries.ts`: dataselectie uit `ProjectPerformanceAssessment`, los testbaar.
- **Vervang**: `BENGIndicator`, `EnergyBreakdownChart`, `MonthlyBreakdownChart`, `PreviewPanel` (en `BENGIndicatorCompact` en `MonthlyBarChart`) en `CalculationNotice`.
- **Tests**:
  - `resultSeries.test.ts` met de fixture `terraced-dwelling-out.json` (maandsommen = jaartotalen);
  - `results-view-kernel`, `indicative-results-ui`, `preview-label-scope` en `indicative-export-status` omzetten naar de nieuwe tegels met dezelfde beweringen. Voorbeelden: het label komt alleen uit de kern; bij incompleet wordt het achtergehouden; de indicatieve badge staat onder Herkomst.

### F8 — Bestaande bouw: opname, maatwerkadvies, herlabelen (parallel)

- **Nieuw**: `src/pages/existing/{SurveyPage, SurveySectionNav, SurveyElementCard, MwaPage, LabelPathChart, PackageInspector, RelabelPage}.tsx`.
- **Wijzig**:
  - `BasisopnamePanel` wordt opgesplitst per onderdeel; de logica en het overnemen blijven;
  - `MaatwerkadviesPanel` wordt opgesplitst in tabs;
  - `MwaTemplateEditor` in een `SideSheet`;
  - `RelabelPanel` krijgt een eigen bestandsknop.
- **Tests**: `basisopname-ui`, `maatwerkadvies-ui`, `mwa-*` en `relabel-panel` blijven groen.

### F9 — Oplevering: rapportframe, registratie, welkom, instellingen (parallel; afstemmen met de rapport-agent)

- **Nieuw**: `src/pages/delivery/{ReportPage, DossierChecklist, ExportsTab, RegistrationPage}.tsx`.
- **Wijzig**:
  - `ReportView`/`PrintPreviewDialog`: het frame rond het `NtaCalculationReport` van de andere agent. Het niveau komt van zijn component;
  - `ProjectInfoDialog`: de algemene gegevens blijven als Project › Gegevens; registratie, opname, WLC, triggers en bewijs gaan naar `RegistrationPage`;
  - `WelcomeScreen` en `SettingsDialog`: nieuwe vormgeving.
- **Tests**: `registration-dialog` → `registration-page` (dezelfde velden en beweringen), `project-dossier`, `nta-calculation-report`, `nta-input-dossier` en `welcome-screen`.

### F10 — Afwerking

- Licht en hoog contrast nalopen op alle pagina's.
- Toetsenbordronde.
- 1280 × 800 met nav ingeklapt.
- Screenshots van alle pagina's in `docs/ui-redesign/na/` naast de mockups.
- Documentatie in `docs/handleiding-nta8800` bijwerken (schermnamen).
- Oude CSS opruimen: `--ribbon-*` en `--titlebar-*` alleen weghalen als de bijbehorende tests worden aangepast. Die testen ze nu (`openaec-design-system.test.ts`). Voorstel: deze variabelen laten staan als ongebruikte merkvariabelen, of de test bewust aanpassen in overleg.

### Overzicht

| Fase | Omvang | Afhankelijk van | Mergebaar los? | Zichtbaar resultaat |
|---|---|---|---|---|
| F1 Tokens & basis | S | — | ja | contrast, fonts, scrollbalk, neutrale statusbalk |
| F2 Primitieven & meldingen | M | F1 | ja | geen `alert()` meer, nieuwe invoervelden waar al gebruikt |
| F3 Kernprovider & stapstatus | S–M | F2 | ja | één consistente "actueel/verouderd" |
| F4 Shell & navigatie | L | F3 | ja (oude views in nieuwe shell) | nieuwe app-shell, nav met status, palet |
| F5 Gebouw & installaties | M | F4 | ja | `02`, `03` |
| F6 NTA in stappen | L | F4 (F5 handig) | ja | `04`, Controle |
| F7 Resultaten | M | F4 | ja | `05` |
| F8 Bestaande bouw | M | F4 | ja | `07`, `08`, herlabelen |
| F9 Oplevering | M | F4 + rapport-agent | ja | `06`, registratie |
| F10 Afwerking | S | alles | ja | thema's, a11y, documentatie |

---

## 12. Open punten voor de opdrachtgever

1. **Bestaande bouw verbergen bij nieuwbouw.** Standaard verbergen, of altijd tonen (ingeklapt)? Voorstel: verbergen, zichtbaar maken via Project › Registratietype of het palet.
2. **De vereenvoudigde invoer** (`heatingSystems`, `solarPV`, … via dialogen) naast de NTA-invoer. Het ontwerp toont beide in dezelfde stap: de systeemketen-kaart vat samen, de NTA-stap is de diepte. Op termijn kan de vereenvoudigde invoer volledig uit de NTA-invoer worden afgeleid. Dat is buiten scope van deze UI-fasen.
3. **Foto's in de basisopname.** Er is opslag nodig in het projectbestand of als losse bestanden naast het project in het dossier-ZIP. Dat raakt `ProjectSerializer` en is een aparte beslissing.
4. **Concept-preview in de controle-inspector.** Dat is een extra kernrun per wijziging. Prima lokaal, maar uitschakelbaar in Instellingen.

### Besluiten (5 oktober 2026, bij fase F1/F2)

1. **Bestaande bouw bij nieuwbouw:** de stappen Basisopname, Maatwerkadvies en Herlabelen blijven zichtbaar, maar gedimd (lagere opaciteit, label "niet van toepassing bij nieuwbouw"). Ze worden niet verborgen.
2. **Vereenvoudigd model:** blijft de bron voor de geometrie (zones, vlakken, ramen, constructies). De NTA-invoer blijft de bron voor de berekening. Er komt nu geen samenvoeging.
3. **Foto's in de basisopname:** worden later opgeslagen via het bewijsregister. Buiten scope van de UI-fasen.
4. **Concept-preview:** staat standaard uit en is een instelling.

### Uitgevoerd in F1/F2

- `src/styles/tokens.css`, `src/styles/base.css` en `src/styles/fonts.css`; lettertypen lokaal in `src/assets/fonts` (latin en latin-ext, OFL). De Google-link in `index.html` blijft als browserfallback.
- Contrast: `--text-muted` is #A1A1AA (donker, 4,7 : 1) en #6B6560 (licht, 5,0 : 1). De statusbalk is neutraal. De scrollbalk volgt het thema. "Onverifieerd" is violet.
- `src/components/ui/`: alle bouwstenen uit §5.2 behalve `CommandPalette`, `KpiTile`, `BulletMeter`, `LabelBadge` en `LabelScale` (F3/F7). Extra: `FileButton` voor de vertaalde bestandskeuze.
- `formatQuantity`, `QUANTITIES` en `parseDecimal` in `src/i18n/format.ts`; `parseKernelPath` in `src/core/nta/pathUtil.ts`.
- Geen `alert()`/`confirm()`/Tauri-`ask` meer in `App.tsx`: meldingen zijn toasts, sluiten met wijzigingen gebruikt `ConfirmDialog` (Opslaan · Niet opslaan · Annuleren).

### Uitgevoerd in F3/F4

- `src/context/KernelProvider.tsx`: één `useProjectPerformance` per actief document. `useKernel()` geeft de vorige geslaagde uitkomst (`settled`), de fase (`loading`/`current`/`stale`/`error`) en `refresh()`. `useKernelQuery()` deelt die run met `ResultsView`, `ReportView`, `PrintPreviewDialog` en `NtaPerformancePanel`; buiten de provider draaien ze een eigen query. `PreviewPanel` gebruikt nog `calculateProjectPerformanceShared`; dat is dezelfde run, want die wordt per projectobject gedeeld.
- `src/core/nta/gapRoutes.ts` (`GAP_ROUTES`, `routeForPath`) en `src/core/nta/stepStatus.ts` (`stepStatuses`).
  - Er zijn nog geen NTA-substappen. NTA-paden gaan daarom naar Controle › NTA-invoer; F6 verfijnt dat.
  - `src/core/navigation/projectPaths.ts` selecteert bij "Ga naar" het element van het pad.
- `src/core/navigation/routes.ts`: `route` in `EnergyContext` met de actie `NAVIGATE`. `SET_VIEW_MODE` blijft een alias. De hash wordt gesynchroniseerd.
- `src/components/shell/`:
  - `TopBar`, `WorkflowNav` (met het menu Gereedschap), `Inspector`, `CommandPalette` en `StepRouter` (vervangt `MainView`);
  - `commands.ts`: de inventaris van de ribbonacties, elk met een `ribbonSource` en een nieuwe plek;
  - de pagina's `ProjectOverview`, `InstallationsPage` en `RegistrationPage`.
- Verwijderd:
  - `Ribbon` (`ThemePicker.tsx` blijft voor de themaconfiguratie), `ProjectBrowser`, `TitleBar`, `MainView`, `ProjectView` en `AppMenu`;
  - `PropertiesPanel` en `PreviewPanel` blijven, als inhoud van de inspector (`embedded`).
- Tests:
  - `ribbon.test.tsx` is nu `workflow-nav.test.tsx`;
  - `titlebar.test.tsx` is nu `top-bar.test.tsx`;
  - `project-browser.test.tsx` is opgegaan in de lijsttests van `workflow-nav.test.tsx` en `envelope-edit.test.tsx`;
  - nieuw: `gap-routes`, `step-status`, `command-palette` en `report-titles`.
- Nog open voor F5–F9:
  - subpagina's per onderdeel (Verwarming, Ventilatie …);
  - editors als zijlade in de inspector;
  - NTA-secties per stap;
  - het concept-voorbeeld.

### Uitgevoerd in F5 (5 oktober 2026)

- Routes (`src/core/navigation/routes.ts`):
  - Gebouw: `envelope` (blijft standaard), `zones`, `constructions`, `thermalBridges`, `airTightness`, `unheated` en `model3d`;
  - Installaties: `systems` (overzicht), `heating`, `hotWater`, `ventilation`, `cooling`, `humidification`, `generation`, `heatPumps` en `bacs`;
  - `reference` is vervallen: de gaswarmtepomp-referentie is een inklapbare diagnose onder `heatPumps`.
- `GAP_ROUTES` en `projectItems` verwijzen naar die subpagina's, bijvoorbeeld:
  - `zones.*.thermalBridges` → Koudebruggen;
  - `constructions` → Constructies;
  - `solarPV` → Opwekking.
  - NTA-paden blijven naar Controle › NTA-invoer gaan; F6 verfijnt dat.
- `src/components/shell/pages/BuildingPages.tsx`:
  - pagina's: `EnvelopePage` (`DataTable` met groepering, filter, ramen als kindregels en `data-path` per rij), `ZonesPage`, `ConstructionsPage`, `ThermalBridgesPage` en `AirTightnessPage` (inline q<sub>v;10</sub>);
  - kaarten: H<sub>T</sub>-aandeel en controles schil.
  - Afwijking van §F5: de pagina's staan in één bestand onder `components/shell/pages/`, net als de F4-pagina's, en niet in `src/pages/building/`. `UnheatedPage` en `ModelPage` zijn niet als aparte bestanden gemaakt: `UnheatedSpacesPanel` en `Building3DView` (nu met `EmptyState`/`ErrorState`) renderen direct.
- `EnvelopeView` is een samenstelling van die pagina's, voor de editortests en voor gebruik buiten de shell.
- `src/components/shell/ElementInspector.tsx`:
  - de inline-editor van het geselecteerde vlak, raam, de zone of de koudebrug, met `NumberInput`/`Select`;
  - `UPDATE_*` voegt samen, dus ids blijven stabiel;
  - een leeg verplicht getal wordt niet weggeschreven.
- `Inspector`: het element krijgt de inline-editor. Op Installaties, zonder systeemselectie, toont het paneel `ServiceEnergyPanel` (energie per dienst).
- `src/components/shell/pages/InstallationsPage.tsx`:
  - `SERVICES` (lijsten, NTA-blokken en padprefixen per dienst);
  - `InstallationsOverview` met ketenkaarten en status uit de kernmeldingen;
  - `ServicePage` per dienst, met de systeemtabel, open punten en NTA-blokken met link naar Controle › NTA-invoer.
- De 13 editordialogen renderen als zijlade (`DialogShell variant="sheet"`). `DataTable` kreeg `rowProps` (`data-path`, klasse) en `ItemActions` kreeg `compact` (icoonknoppen, dezelfde toegankelijke namen).
- Labels staan in `src/i18n/buildingInstallLabels.ts` (NL/EN).
- Tests:
  - nieuw: `building-installations-pages.test.tsx`;
  - aangepast: `workflow-nav` (toevoegen per subpagina, dienstpagina, overzicht) en `gap-routes`;
  - `envelope-edit` en `point-thermal-bridges` zijn ongewijzigd groen.
- Schermafdrukken: `~/oes-shots/f5/`.
- Nog open:
  - de ketenfasen (opwekking › distributie › afgifte › regeling) per dienst uit de NTA-invoer; nu toont de kaart de systemen en de ingevulde NTA-blokken;
  - "Bron & bewijs" per vlak;
  - een type-wissel (wand/dak/vloer) in het inline-paneel.

### Uitgevoerd in F8 (5 oktober 2026)

- Routes: Basisopname heeft per onderdeel een subpagina (`general`, `zones`, `envelope`, `heating`, `hotWater`, `ventilation`, `cooling`, `pv`, `result`). Maatwerkadvies heeft de subpagina's `measures`, `use`, `passport` en `advice`. `GAP_ROUTES` stuurt `basisopname.<onderdeel>…` en `maatwerkadvies.<blok>…` naar die subpagina's.
- `BasisopnamePanel` heeft de props `section` en `onSection`:
  - zonder `section` blijft het de bestaande lange kolom, zodat de bestaande tests en het gedrag ongewijzigd blijven;
  - met `section` is het de wizard uit `07`: voortgang en onderdelen links (fouten per onderdeel uit de laatste doorrekening), het onderdeel in het midden met Vorige/Volgende, en rechts de kaart *Uitkomst opname*. De kaart toont `Indicatief`, label, EP₂, fouten, forfaitaire waarden, Doorrekenen en Verwijderen;
  - de velden krijgen `data-path` `basisopname.…`. Ga naar bij een fout (`surveySectionForPath`) opent het onderdeel en focust het veld;
  - de kernuitkomst blijft bewaard bij het wisselen van onderdeel;
  - een antwoord zonder `issues` (serde-weigering) wordt een foutmelding;
  - de woning heeft geen Rekenzones: die subpagina valt terug op Algemeen.
- `MaatwerkadviesPanel` heeft de prop `tab`. Zonder `tab` blijft de oude kolom staan (de tests `maatwerkadvies-ui` en `mwa-*` zijn ongewijzigd). Met `tab`:
  - de blokken worden `Card`'s;
  - nieuw in `src/components/shell/pages/existing/MwaViews.tsx`: `LabelPathChart` (SVG, `labelPathBars`), `PackageComparison` (`Segmented` NCW/TVT/invoer met `orderResults`), `MeasureCard` (sjabloonkeuze, compleet/onvolledig uit `buildTemplatePatch`, pakketchips 1/2/3) en `PackageInspector`;
  - `MeasureEditor` (met `MwaTemplateEditor`) opent in een `SideSheet`.
  - Bij het doorrekenen worden de sjabloonpatches nog steeds opnieuw opgebouwd. Verwijderen haalt de maatregel ook uit de pakketten.
- `RelabelPanel` is herschreven als stepper (`Stepper` met status per stap):
  - eigen bestandsknop (`FileButton`), bestandsnaam, datum en SHA-256;
  - het oordeel als `StatusPill` en de verouderd-melding als `Banner`;
  - de wijzigingentabel met filter 6a/6b/te beoordelen, cluster, leesbare namen (`relabelElementName`), pad en Ga naar;
  - de bewijsrollen geteld uit `registration.evidence[].relabelProof`, met een melding voor `review`-facturen;
  - de gereedheid uit `useKernel().settled.registration`: de hercontrole `relabelAssessment`, gereed voor registratie, en de `relabel_*`-punten.
  - Opslag, hashes, het weggooien van een lopende vergelijking na een wijziging, en Verwijderen werken zoals voorheen.
- Afwijking van §11 F8: de onderdelen staan in de bestaande componenten (props) en in `shell/pages/existing/`, niet in `src/pages/existing/`. Locatie-modus, fotovakken en "Overnemen in projectmodel" uit `07` zijn niet gebouwd; het bestaande gedrag kende ze niet.
- Labels: `src/i18n/existingLabels.ts` (NL/EN). Stijl: `shell/pages/existing/existing.css` (alleen tokens en de labelkleuren uit `resultsData`).
- Tests:
  - nieuw: `existing-building-pages.test.tsx` (wizard, Ga naar, uitkomstkaart, labelpad, maatregelkaarten, zijpaneel, pakketchips, sorteren, inspector, relabelstepper, filter, bewijsrollen);
  - aangepast: `gap-routes` (subpagina's voor basisopname en maatwerkadvies);
  - ongewijzigd groen: `basisopname-ui`, `maatwerkadvies-ui`, `mwa-*`, `relabel-panel` en `relabel-migration`.
- Schermafdrukken: `~/oes-shots/f8/` (donker en licht, NL; EN licht; utiliteit), met een volledige herlabelronde tegen een gewijzigd origineel.

### Uitgevoerd in F7 (5 oktober 2026)

- Routes: Resultaten heeft de subpagina's `overview` (standaard), `services`, `zones`, `monthly` en `provenance`.
- `src/components/shell/pages/results/`:
  - `resultsData.ts`: zuivere mapping van de kernuitkomst naar maandreeksen per dienst, per drager of primair fossiel. PV en de exportcredit staan als negatieve, gearceerde reeks onder de nullijn. De reeksen sommeren per constructie tot de kerntotalen (EP<sub>tot</sub> volgens §5.5.3, geleverd per drager, PV-opwek, netto behoefte). Verder: meters tegen de Bbl-eisen uit `bblCheck`, en TO<sub>juli</sub> alleen voor woonfuncties;
  - `MonthlyChart.tsx`: handgeschreven SVG, gestapeld of gegroepeerd, zonder bibliotheek. Kleuren komen uit de `--viz-*`-tokens; de dienstvolgorde is gevalideerd op kleurenblindheid. Elke maand is een focusbare groep (←/→) met een tooltip bij hover en focus. De legenda toont jaartotalen. De tabelweergave is een `<table>` met caption en jaartotaal;
  - `ResultsDashboard.tsx`: labelkaart met klasseschaal, BENG 1/2/3 en TO<sub>juli</sub> met oordeel en marge tot de eis, de energiegrafiek (gebruik / primair fossiel / per drager), de netto behoefte (warmte en koude), aandachtspunten met Ga naar (Bbl-tekorten en kernmeldingen) en de kerngetallen (primair fossiel, hernieuwbaar, finaal, CO₂, ZEB). De tabbladen Per dienst, Per zone, Maandwaarden en Herkomst tonen kernversie, normversie, status, vingerafdruk, bronnen en interpretaties.
- Alleen de verdict `calculated` toont het dashboard. Andere verdicts vallen terug op `ResultsView`, dus ingehouden en indicatief gedrag blijft ongewijzigd. Een verouderde uitkomst krijgt de `StaleBanner`, een gedimde inhoud en `aria-busy`.
- De lay-out reageert op de breedte van de inhoud (container query) en niet op het venster, zodat een open contextpaneel de kaarten niet samenperst.
- `PageHeader`: de titel toont de subpagina, behalve op de eerste subpagina van een stap (F5-restpunt).
- Labels staan in `src/i18n/resultsLabels.ts`.
- Tests:
  - nieuw: `results-dashboard.test.tsx` (sommen tegen de kerntotalen voor beide voorbeelden, meters, tabbladen, verouderd, ingehouden);
  - aangepast: `workflow-nav` (paginatitel Constructies).
- Schermafdrukken: `~/oes-shots/f7/`.

### Uitgevoerd in F9 (5 oktober 2026)

- Routes: Rapport & dossier heeft de subpagina's `report` (standaard, Rekenrapport), `input` (Invoerdossier), `checklist` (Checklist BRL 9500) en `exports`.
- `src/components/shell/pages/DeliveryPages.tsx`:
  - `ReportPage` is het frame rond `ReportBuilder` via `ReportView section="report"`. Niveau, taal, hoofdstukken, voorbeeld en export blijven van de rapportcomponent;
  - `InputDossierPage` bevat de export van het invoerdossier en `ReportView section="input"`;
  - `DossierPage` toont de voortgangsmeter, de dossierpunten per groep met `DossierStatusPill` (inclusief `pending`), het bewijsregister (alleen-lezen, met "Bewerken in Registratie") en het EP-Online-overzicht uit `buildEpOnlineOverview` met de ontbrekende verplichte velden;
  - `ExportsPage` bundelt alle exports in kaarten.
  - De kop houdt Exporteer rapport en Afdrukken. Onder Rekenrapport, Invoerdossier en Checklist staat daar ook een knop naar Exports.
- `ReportView/useDossier.ts`: de live dossiercheck en de ZIP-export, gedeeld door `ReportView` en de pagina's. `ReportView` zonder `section` toont alles zoals voorheen.
- Registratie:
  - `RegistrationForm` bevat alle registratiesecties uit `ProjectInfoDialog`, met dezelfde veld-id's en labels en een `data-path` per veld. Er is een eigen concept met Verwerpen en Opslaan in een vaste balk;
  - `RegistrationPage` toont de attestbanner, de gereedheid met de redenen (`readinessReasons`), het rekenprogramma en de openstaande punten en plausibiliteit met Ga naar (focus via `data-path`);
  - `ProjectInfoDialog` houdt alleen de projectbasis. `CommandPalette` stuurt registratievelden naar de stap.
- `WelcomeScreen`: kaarten Nieuwe woning en Nieuw utiliteitsgebouw (`office`), Openen, UNIEC3/VABI-import, recente projecten (`core/io/recentProjects.ts`, localStorage, alleen desktoppaden) en de voorbeelden.
- `SettingsDialog`: tabbladen Algemeen, Berekening en Over als verticale tablist; thema en taal zijn radiogroepen. De editie voor nieuwe berekeningen staat in `core/nta/defaultEdition.ts`, en `buildNtaCalculationTemplate` neemt die over.
  - Afwijking van §8: er is geen tabblad Kern (adres/poort), want de app heeft geen instelbare kernverbinding.
- Labels staan in `src/i18n/deliveryLabels.ts` (NL/EN). Stijlen staan in `shell/pages/delivery.css`, `WelcomeScreen.css` en `SettingsDialog.css`, alleen met tokens.
- Tests:
  - `registration-dialog` → `registration-page` (dezelfde velden en beweringen; nu ook `readinessReasons`, Verwerpen, attestbanner en de projectbasisdialoog);
  - `welcome-screen` (nieuwe knoppen, import, recente projecten en de opslag);
  - nieuw: `settings-dialog` (tabbladen, vastleggen bij OK, standaardeditie in het sjabloon);
  - aangepast: `workflow-nav` (rapportsubpagina's, exports, checklist) en `ui-walkthrough-2` (relabelvelden op het formulier).
- Schermafdrukken: `~/oes-shots/f9/` (NL donker, EN licht).
- Nog open:
  - paginaminiaturen naast het rapportvoorbeeld;
  - de dossiercheck als zijkolom naast het rekenrapport (§8). Nu staat die op een eigen subpagina;
  - het labeltype in de recente projecten.

### Uitgevoerd in F6 (5 oktober 2026)

- Sectieregister `src/components/NtaPerformancePanel/NtaSections.tsx` (`NTA_SECTIONS`): elke sectie van het oude formulier heeft een stap, een subpagina, voor Verwarming een stepperdeel, `paths`, `advanced`, `when` en een component. Velden en paden zijn ongewijzigd.
  - Afwijking van §6.1: het register staat naast de veldcomponenten in `NtaPerformancePanel/` en niet in `src/components/nta/`. Een sectie heeft één component; "Geavanceerd" werkt per sectie (`advanced`), niet per veld.
  - Gesplitst: Afgifte en distributie in Afgifte, Hulpenergie (ventilatoren), Regeling en Distributie; actieve koeling uit Algemeen naar Koeling; Bevestigingen in GBS, Externe levering, Opgegeven stromen en Opslag; C1 bij Ventilatie.
  - Toewijzing: Project ← Algemeen (uitgave, rekenomvang); Gebouw › Rekenzones ← gebruiksfuncties, setpoints, massa, interne warmte; Schil & ramen ← ramen, dynamische ramen, dakhellingen, vloeren op grond; Onverwarmde ruimten ← serres; Installaties › Verwarming ← Opwekking / Distributie (met verticale leidingen) / Afgifte / Regeling & BCRG / Hulpenergie / Zonneverwarming; Warm tapwater, Ventilatie (met H11), Koeling, Bevochtiging, nieuwe subpagina Verlichting (utiliteit), Opwekking (PV, bijlage P, opgegeven stromen, opslag), Gebouwautomatisering; Controle › NTA-invoer ← Bevestigingen.
- `NtaCalculationForm` rendert alle secties uit het register in de oude volgorde en blijft het volledige formulier onder Controle › NTA-invoer (met de JSON-editor). Exports voor de tests (`table713Setpoints`, `setpointChecks`, `setpointWriteBack`, `GroundEdgeBridgesFields`) blijven.
- `src/context/NtaDraftProvider.tsx`: één concept per document (in `App` binnen `KernelProvider`, ook in `renderWithProviders`). Gewijzigde bladpaden, ongedaan maken per bewerking, en Toepassen = het oude Opslaan (`SET_NTA_CALCULATION` met `syncVentilation`). Een blok dat van buiten verandert (JSON, volledig formulier, import) vervangt het concept. Het volledige formulier begint met het concept als dat wijzigingen heeft.
- `src/components/shell/NtaStepPage.tsx`:
  - `NtaStepSections` op Project, Gebouw, Installaties en Controle › NTA-invoer, met Basis/Alle velden (`Segmented`, per kijker in localStorage), "Geavanceerd · n secties — … · alle op standaard" (klapt open bij invoer, een kernmelding of de Ga naar-plek), de stepper voor Verwarming en per sectie "Bron & bewijs" (alle `*Reference`-velden, ingevuld of aandachtspunt, met Ga naar);
  - velden met een kernmelding krijgen een rode of gele rand;
  - `NtaApplyBar`: "n wijzigingen … · Ongedaan maken · Vorige stap · Toepassen en verder"; verder en terug lopen over de NTA-pagina's en de verwarmingsdelen.
- `NtaFormFields`: `NumberField` gebruikt `ui/NumberInput` (komma of punt, eenheid, ↑/↓) en houdt de rol spinbutton. Elk veld krijgt `data-path` (`ntaCalculation.…`) via `FieldPathPrefixProvider`; de basisopname en het maatwerkadvies gebruiken de velden zonder prefix.
- `GAP_ROUTES`: `NTA_INPUT_ROUTES` stuurt elk lid van `ntaCalculation` naar de pagina van zijn sectie; alleen `ntaCalculation` zelf en de bevestigingen gaan naar Controle › NTA-invoer. Paden van de gebouwbeoordeling zonder prefix (bijv. `externalSupply.collectiveHeatPumpSource.realisedFrom2013`) worden als NTA-pad gerouteerd en gefocust. `nta_calculation_block_invalid` krijgt het pad uit de serde-melding (`setpoints: missing field \`heatingC\`` → `ntaCalculation.setpoints.heatingC`).
- `focusPathIn` (nu `shell/focusPath.ts`): kijkt ook naar `data-paths` van een sectie, opent een ingeklapte `<details>`, geeft de focus aan het invoerveld in een label en kiest bij gelijke paden het binnenste element.
- Inspector: derde tab Controle (`InspectorCheckPanel`) met fouten, aandachtspunten en weggelaten correcties, de punten van deze pagina eerst en de rest per stap, elk met Ga naar.
- Alleen in de editie 2024: velden voor bijlage AA `effectiveMassKgPerM2` en `rooms[].roofAreaM2` (als `capacity.calculation` bestaat) en `collectiveHeatPumpSource.realisedFrom2013`. Onder een andere editie staan achtergebleven waarden als melding met Verwijderen.
- Labels: `src/i18n/ntaStepLabels.ts` (NL/EN).
- Tests: nieuw `nta-step-pages.test.tsx`; aangepast `gap-routes` (nieuwe doelen, register-consistentie, details openen), `step-status` (generator-gap telt bij Installaties, pad uit de serde-melding) en `nta-performance-panel` (tekstveld geeft `'20'`). De overige `nta-*`-tests zijn ongewijzigd groen.
- Schermafdrukken: `~/oes-shots/f6/`.
- Nog open:
  - Bron & bewijs toont de bronvelden samen met Ga naar; de velden zelf staan nog in de sectie (geen documentkeuze/paperclip-upload);
  - "Geavanceerd" per veld binnen een sectie, en eenheden uit de labeltekst halen (§6.5);
  - de ConfirmDialog bij het sluiten van een document met een niet-toegepast concept;
  - een preview-run op het concept ("effect op uitkomst").

