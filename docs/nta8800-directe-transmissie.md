# Diagnostische directe transmissie

De Rust-kern heeft een zelfstandige functie voor de rekenkundige som `Σ(A·U) + Σ(L·Ψ) + Σχ` in W/K. Alleen **expliciet als rechtstreeks aan buiten grenzend** aangeleverde elementen horen in de invoer. De functie bepaalt de thermische begrenzing niet zelf. Grond, onverwarmde en aangrenzende ruimtes, reductiefactoren, maandverloop, warmtebehoefte en BENG vallen buiten deze functie.

Dit is een eerste bouwsteen, **geen geverifieerde NTA 8800-berekening**. De analyse `docs/2026-07-13-c1-norm-analyse-transmissie.md` in Open Heatloss Studio is gebruikt om de beoogde deelposten te inventariseren; de primaire normtekst en een onafhankelijk verwacht deelresultaat zijn in deze checkout nog niet geverifieerd. Er is geen Heatloss-code overgenomen. Het handvoorbeeld hieronder controleert rekenkunde en eenheden, geen normconformiteit. De returnwaarden `referenceVerified=false` en `bengCalculationAvailable=false` zijn verplicht zolang die bewijsstappen ontbreken.

Een afzonderlijke [maandelijkse diagnose](nta8800-maandtransmissie.md) kan deze expliciete W/K vermenigvuldigen met twaalf aangeleverde temperatuurverschillen en uren. Ook die uitkomst blijft buiten de NTA-vraag- en BENG-route.

## Invoercontract

`POST /v1/nta8800/transmission/direct/diagnose` accepteert:

```json
{
  "input": {
    "elements": [
      {"id":"wall", "areaM2":10, "uValueWPerM2k":0.2, "sourceReference":"drawing-1"},
      {"id":"window", "areaM2":2, "uValueWPerM2k":1.1, "sourceReference":"declaration-1"}
    ],
    "linearBridges": [
      {"id":"edge", "lengthM":3, "psiWPerMk":0.05, "sourceReference":"detail-1"}
    ],
    "pointBridges": [
      {"id":"junction", "chiWPerK":0.02, "sourceReference":"detail-2"}
    ]
  }
}
```

De som is `4,20 + 0,15 + 0,02 = 4,37 W/K`. De response scheidt de drie termen en bevat doelversie, kernelversie en SHA-256 van de getypeerde invoer. De referenties zijn door de gebruiker opgegeven herkomstlabels; de service controleert niet of documenten bestaan of of hun waarden volgens de norm toepasbaar zijn. De vingerafdruk bewijst uitsluitend welke invoer is verwerkt.

Op fysiek ongeldige invoer retourneert HTTP 422 met `status=invalid`, issuepaden en **geen** getal. Een vormfout wordt door de JSON-extractor geweigerd. MCP biedt `diagnose_direct_transmission` met dezelfde `input`; het retourneert een tool error bij vorm- of invoerfouten. De projectvalidatie kan nu een beperkte buiten-diagnose afleiden; de BENG-route gebruikt deze deelberekening niet.

## Projectadapter en editor

Schilvlakken en lineaire koudebruggen hebben nu een optioneel `thermalBoundary`: `outdoor`, `ground`, `unheated_space`, `adjacent_conditioned` of `internal`. De editors laten de gebruiker de werkelijke grens vastleggen. Bestaande `.oes`-bestanden blijven zonder deze waarde leesbaar; de kernel vult geen grens in op basis van `wall`, `roof` of `floor`.

`POST /v1/nta8800/validate` rapporteert hoeveel vlakken en koudebruggen expliciet zijn geclassificeerd. Alleen als alle vlakken en bruggen zijn geclassificeerd, de koudebruglijsten expliciet aanwezig zijn, de projectinvoer structureel geldig is en alle gebruikte buiten-U-waarden bruikbaar zijn, verschijnt `summary.directOutdoorDiagnostic`. De adapter splitst een buitenvlak in een dicht deel (`bruto − ramen`) en de afzonderlijke ramen; buiten-koudebruggen tellen als `L·Ψ`. Grond- en aangrenzende grenzen worden niet in deze directe som opgenomen. Bestaande projecten met onbekende grenzen leveren dus geen projectgebonden transmissiegetal.

Zones kunnen nu optioneel `pointThermalBridges` vastleggen met ID, naam, χ in W/K, thermische grens, zone en `sourceReference`. De puntbruginventaris wordt pas voor de projectdiagnose gebruikt wanneer `pointBridgeInventoryComplete=true` expliciet is bevestigd. Een oude zone zonder deze lijst of bevestiging krijgt **geen** afgeleide directe buiten-som, ook niet wanneer alle vlak- en lineaire grenzen zijn ingevuld. De editor toont de inventarisstatus en zet de bevestiging terug bij toevoegen of verwijderen. Alleen als buiten geclassificeerde punten dragen bij aan `Σχ`; grond- en aangrenzende punten blijven buiten deze diagnostische som.

De adapter gebruikt voor vlak- en lineaire eigenschappen interne projectveldverwijzingen als `sourceReference`: die zijn **geen extern gecontroleerd product- of detailbewijs**. Puntbijdragen bewaren de door de gebruiker ingevoerde detailreferentie, die evenmin door de software wordt geverifieerd. De weergave in het auditpaneel noemt het daarom consequent een diagnose.

## Voorwaarden voordat dit een NTA-deelresultaat wordt

1. De primaire tekst van de doeluitgave en toepasselijke verwijzingsnormen worden formule voor formule gecontroleerd, inclusief correcties, uitsluitingen en afronding.
2. Het canonieke projectmodel legt buiten-/grond-/aangrenzende grenzen en alle relevante vlak- en koudebrugcomponenten expliciet vast, met herleidbare eigenschappen.
3. Minstens één onafhankelijk verwacht transmissie-deelresultaat, plus grensgevallen voor ramen, koudebruggen en meerdere zones, wordt tegen deze Rust-functie en de projectadapter getoetst.
4. Pas daarna wordt de deelpost aan de vraag- en BENG-keten gekoppeld. Attestering blijft een aparte externe toets.
