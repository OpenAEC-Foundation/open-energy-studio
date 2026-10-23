# 4. Uitvoer

Na het toepassen van de NTA-invoer rekent de kern zelf. Dat gebeurt een korte tijd na de laatste wijziging; **Herberekenen** (Ctrl+Enter) start meteen een nieuwe run. Staan er onder *Resultaten* nog geen uitkomsten, dan staat daar ook de knop **Herberekenen**, en een verwijzing naar *Controle › Controleoverzicht* als de kern de invoer nog niet compleet vindt. De uitkomsten verschijnen op vier plekken:
- de statusbalk en het tabblad *Voorbeeld* van het contextpaneel;
- de stap Resultaten (dashboard met de subpagina's Per dienst, Per zone, Maandwaarden en Herkomst);
- het paneel *NTA 8800-berekening (rekenkern)* onder Controle › NTA-invoer;
- de stap Rapport & dossier.

Rekent het project in een oudere uitgave van NTA 8800, dan melden de projectstatus, het dashboard van Resultaten, het rekenpaneel, de statusbalk en het rapport "Oudere uitgave — niet voor registratie", met de uitgave waarin is gerekend (zie [hoofdstuk 10](10-normversies.md)).

Alle getoonde BENG-waarden komen uit de kern. De oude, vereenvoudigde berekening verschijnt alleen als er geen kernresultaat is. Dan draagt ze het label "Indicatief, niet volgens NTA 8800".

## Indicatoren

| Indicator | Betekenis | Afronding |
|---|---|---|
| BENG 1 (EP1) | energiebehoefte in kWh/m²·jr, uit een aparte run met het vaste C1-ventilatiesysteem (§5.4) | omhoog op 0,01 |
| BENG 2 (EP2) | primair fossiel energiegebruik in kWh/m²·jr | omhoog op 0,01 |
| BENG 3 (RER) | aandeel hernieuwbare energie in % | omlaag op 0,1 |
| TOjuli | temperatuuroverschrijding per oriëntatie, alleen bij woningbouw (§5.7) | omhoog op 0,01 |
| CO₂ | emissie per drager (tabel 5.3) | — |
| Finaal energiegebruik | per drager (§5.9) | — |
| ZEB-indicator | informatief (bijlage AB) | — |

**Bijzondere gevallen**
- Als EPTot en de hernieuwbare energie samen nul zijn, blijft EP2 bestaan en is het aandeel hernieuwbaar leeg (`null`).
- Een negatieve EPTot kan een aandeel boven 100 % geven. Dan volgt een waarschuwing.

**TOjuli narekenen.** De resultatenpagina (tab Rekenzones) toont per oriëntatie alle termen van formule 5.40: de oppervlakte, H_C;juli, het warmtetransport Q_ht, de warmtewinst Q_gn, de terugwinbare systeemverliezen Q_rbl, de koudebehoefte Q_C;nd en de onttrekking van een boosterwarmtepomp Q_C;HP. Per rekenzone staan daaronder de vaste waarden en de zonetermen: t_juli en θ_e;juli uit tabel 17.1, de setpoint uit tabel 7.13, a_C;red, de interne warmtelast, H_C;ve en H_gr;an in juli, de warmtecapaciteit, de gebruiksoppervlakte, de minimale oppervlakte per oriëntatie en de grenswaarde. Daarmee is TOjuli = (Q_C;nd − Q_C;HP) · 1000 / (H_C;juli · t_juli) met de hand na te rekenen. De API geeft dezelfde termen (`tojuli[].orientations[]` en `tojuli[].july`).

## Label en labelgegevens

- **Klasse:** de labelklasse volgt uit de afgeronde EP2 en de klassegrenzen van de Omgevingsregeling (bijlagen IX en X). Bij gemengde functies zijn de grenzen naar oppervlakte gewogen.
- **Woningen:** het label gebruikt de forfaitaire EMG-waarden (Omgevingsregeling art. 5.11 lid 4). BENG 2 en 3 en de Bbl-toets blijven op de kwaliteitsverklaring. De uitvoer heeft daarom aparte velden: `labelPrimaryFossilIndicatorKwhPerM2Year` en `labelRenewableSharePercent`.
- **Indicatief:** de klasse is altijd indicatief, omdat het programma niet geattesteerd is.
- **Labelgegevens (Omgevingsregeling art. 5.13):**
  - algemene gegevens: functie, bouwjaar, A_g op twee decimalen, woningtype;
  - isolatie per element;
  - installaties;
  - EP2, het aandeel hernieuwbaar en TOjuli.

  Het bouwjaar komt eerst uit de registratie en anders uit de NTA-invoer.

## Bbl-toets

- **Woningen:** de toets vergelijkt BENG 1, 2 en 3 met de grenswaarden. BENG 1 hangt af van A_ls/A_g.
- **Utiliteit:** de grenswaarden zijn per functie naar oppervlakte gewogen (art. 4.149 lid 2). Woonfuncties en utiliteitsfuncties mogen niet samen worden gewogen.
- **TOjuli:** de grens van 1,20 geldt alleen voor woningbouw.
- **TOjuli bij een serre:** het geveldeel naar een aangrenzende onverwarmde serre, zijn transmissie en de zonwinst via de serre blijven buiten de verdeling per oriëntatie. De rekenzone krijgt dan een melding; hoe dit geveldeel moet meetellen, is als vraag aan NEN voorgelegd.
- **Rapport:** het rapport toont elke grens met hetzelfde aantal decimalen als de waarde.

De grenswaarden komen uit de Bbl-tekst, niet uit de NTA. Zie het bronnenregister in [hoofdstuk 8](08-versies-en-verwijzingen.md).

## Energie per energiefunctie

Per functie, drager en maand toont de uitvoer (`energyByService`) het gebruik, de levering, het primair fossiele en het hernieuwbare deel (§5.5.3, 5.20/5.21). De functies zijn:
- verwarming;
- warm tapwater;
- koeling;
- bevochtiging;
- ventilatie;
- verlichting;
- hulpenergie;
- collectieve bron.

Over de verdeling:
- **Eigen opwekking:** eigen PV-stroom die zelf wordt gebruikt, is naar rato van het elektriciteitsgebruik over de functies verdeeld. Dat is een interpretatie.
- **Combi-tapwater:** het tapwaterdeel van een combitoestel telt als tapwater, zonder f_BACS (13.184/13.185).
- **Totalen:** de som van de functies sluit aan op de totalen per drager.

Het rapport toont daarnaast:
- warm tapwater in detail;
- elke koelinstallatie apart;
- PV per systeem.

## Meldingen bij de uitkomst

- **Invoergaten:** wat ontbreekt; de berekening stopt.
- **Waarschuwingen:** wat ongebruikelijk is; de berekening gaat door. Voorbeelden:
  - opgegeven gebruik met een rendement boven 1;
  - f_BACS 1,0 zonder BACS-gegevens;
  - een zeer laag distributierendement door standaardwaarden voor de circulatieleiding;
  - een emissieverlies dat bijna oneindig wordt (10.15).
- **Codes en paden:** elke melding heeft een vertaalde tekst, met de code en het pad klein erbij.

Zie [hoofdstuk 5](05-validatie.md) voor de statussen.

## Bijlage Interpretaties

Het rekenrapport heeft een bijlage met alle interpretaties van de kern. Dat zijn de plaatsen waar de norm onduidelijk of strijdig is en de kern een lezing kiest. De lijst komt uit één functie (`kernel_interpretations()`) en is ook op te vragen:
- API: `GET /v1/nta8800/interpretations`;
- desktop-app: commando `kernel_interpretations`.

De lijst is in het Engels; het rapport vermeldt dat het de eigen tekst van de kern is.

## Rapportage Energieprestatie (NTA 8800)

De subpagina **Rapport & dossier › Rekenrapport** bevat het onderdeel **Rapportage Energieprestatie (NTA 8800)**. Daarmee stel je het hoofdrapport samen.

**Rapportniveau**
- **Samenvatting:** projectgegevens, uitgangspunten (rekenprogramma met programmaversie, rekenkernversie, normversie, BRL 9501-attestnummer en invoervingerafdruk), eisen en resultaten (BENG 1/2/3 en TOjuli tegen de Bbl-grenzen, met oordeel), indicatieve labelklasse met de labelgegevens van Omgevingsregeling art. 5.11–5.13a, voor een woning de nettowarmtebehoefte tegen de Standaard voor Woningisolatie, A0, meldingen en registratie. Zolang het programma geen attest heeft staat bij het attestnummer "nog niet toegekend". Het registratieblok staat er altijd; zonder registratiegegevens noemt het wat nog ontbreekt. Welke uitvoereis van BRL 9500 en 9501 waar staat, zie [uitvoereisen](../nta8800-uitvoereisen.md).
- **Standaard:** daarbij gebouw en rekenzones (A<sub>g</sub>, A<sub>ls</sub>, compactheid, D<sub>m</sub>), bouwkundige uitgangspunten per element (oppervlakte, U, g, A·U, grondvloeren, thermische bruggen, verticale leidingen), installatietechnische uitgangspunten per functie, energie per functie en drager en het volledige invoeroverzicht met het invoerpad per gegeven.
- **Gedetailleerd:** daarbij de berekeningen die je aanvinkt, tot op maandniveau:
  - transmissie (H<sub>D</sub> per element, H<sub>g</sub>, H<sub>U</sub>, H<sub>tr</sub>);
  - ventilatie en infiltratie per maand;
  - interne en zonnewinst per maand, ook per raam;
  - warmte- en koudebalans per rekenzone (θ, Q<sub>ht</sub>, Q<sub>gn</sub>, γ, τ, a, η, Q<sub>nd</sub>), met een uitgewerkt rekenvoorbeeld voor januari en juli;
  - verwarmingsketen, warm tapwater, koeling, verlichting en zonnestroom per maand;
  - primaire energie per drager met BENG 2 en BENG 3 uitgewerkt;
  - TOjuli per oriëntatie;
  - BENG 1 met het vaste ventilatiesysteem C1.

Elke berekeningsstap staat als "formule → waarden → uitkomst" met het formulenummer van NTA 8800. Alle getallen komen uit de uitvoer van de rekenkern; het rapport rekent alleen presentatiewaarden uit, zoals A·U per element of een jaarrendement als controle.

**Voorbeeld, export en afdrukken**
- Het voorbeeld onder de keuzes toont het rapport zoals het wordt geëxporteerd.
- **Rapportage exporteren (HTML)** slaat het rapport op; in de desktop-app via het opslagvenster.
- Zolang de rekenkern geen uitkomst heeft, staan exporteren en afdrukken uit; de pagina zegt dat het rapport nog niet kan worden samengesteld. Het NTA-invoerdossier kun je dan wel al exporteren.
- **Afdrukken of opslaan als PDF** opent het rapport in een venster en start het afdrukken. Het rapport is opgemaakt voor A4: inhoudsopgave, genummerde tabellen, elk hoofdstuk op een nieuwe pagina en een kopregel met project, datum en kernversie.
- De app onthoudt je laatste keuze van niveau en hoofdstukken.

Weigert de rekenkern de invoer (status ongeldig of onvolledig), dan bevat het rapport geen BENG-waarden of labelklasse maar de lijst met invoergaten.

## Rapporten

| Rapport | Inhoud | Taal |
|---|---|---|
| Rapportage Energieprestatie (Rapport & dossier › Rekenrapport) | samenvatting, standaard of gedetailleerd, zie hierboven | Nederlands (BRL 9500-document) |
| BENG-rapport (knop **Exporteer rapport** op Rekenrapport en Exports, en in het palet) | indicatoren, grenzen, maandoverzicht, energiebalans | volgt de taal van de interface |
| NTA-rekenrapport (knop **NTA-rekenrapport exporteren** onder Exports) | volledige uitkomst, labelgegevens, registratie, interpretaties | Nederlands (BRL 9500-document) |
| NTA-invoerdossier (knop **NTA-invoerdossier exporteren** onder Invoerdossier en Exports) | de ingevoerde toestellen en het bewijs, zonder BENG-uitkomst of label | Nederlands |
| Adviesrapport maatwerkadvies | zie [hoofdstuk 6](06-maatwerkadvies.md) | Nederlands |

**Opmaak in de Nederlandse documenten**
- getallen met een decimale komma;
- tijden als `<time datetime="…">`, gelijk aan het manifest van het dossier.
