# Diagnostische transmissie via onverwarmde ruimtes

De Rust-kern kan voor **expliciet benoemde** onverwarmde ruimtes een aangeleverde transmissiesom per grensvlak met een eveneens aangeleverde reductiefactor vermenigvuldigen. Voor elke ruimte is de rekenkundige diagnose `b × [Σ(A·U) + Σ(L·Ψ) + Σχ]` in W/K. Verschillende ruimtes worden daarna opgeteld.

Dit is **geen geverifieerde NTA 8800-uitkomst**. Open Heatloss Studio beschrijft een verwante route voor §8.4 met een door de gebruiker opgegeven factor, maar de primaire tekst van NTA 8800:2025+C1:2026, de afleiding van `b`, de toepasselijkheid van koudebruggen, alle correcties en onafhankelijke verwachte waarden zijn hier nog niet geverifieerd. De functie is niet aan warmtebehoefte, BENG of label gekoppeld. `referenceVerified=false` en `bengCalculationAvailable=false` blijven vast.

## Contract

`POST /v1/nta8800/transmission/unheated/diagnose` accepteert:

```json
{
  "input": {
    "spaces": [{
      "id": "garage",
      "reductionFactor": 0.5,
      "factorSourceReference": "supplied-assumption-1",
      "boundary": {
        "elements": [{
          "id": "wall", "areaM2": 10, "uValueWPerM2k": 0.4,
          "sourceReference": "drawing-1"
        }],
        "linearBridges": [{
          "id": "edge", "lengthM": 2, "psiWPerMk": 0.1,
          "sourceReference": "detail-1"
        }]
      }
    }]
  }
}
```

Het voorbeeld geeft vóór reductie `4,0 + 0,2 = 4,2 W/K` en na de **opgegeven** factor `2,1 W/K`. De response bevat beide getallen per ruimte, de som, issuepaden, kernel-/doelnormversie en SHA-256 van de getypeerde invoer. Deze hash maakt invoer herkenbaar en verifieert geen bewijsstukken.

IDs moeten per ruimte uniek zijn; elke ruimte vereist een factor binnen 0–1 met een herkomstverwijzing en een geldige expliciete grensvlaksom. Ongeldige factoren, dubbele IDs, vormfouten, numerieke overloop of ongeldige vlak-/brugtermen leveren **geen** gedeeltelijke getallen. HTTP geeft dan 422 voor een inhoudsfout. De MCP-tool `diagnose_unheated_transmission` gebruikt dezelfde kernfunctie en geeft bij ongeldige invoer een tool error.

De projecteditor bewaart nu `unheatedSpaces` met naam, stabiele ID, aangeleverde `reductionFactor` en `factorSourceReference`. Vlakken, lineaire en puntkoudebruggen met `thermalBoundary=unheated_space` krijgen een expliciete `unheatedSpaceId`. `/validate` controleert factorbereik, bron, unieke ruimte-IDs, bestaande koppelingen en ten minste één gekoppeld vlak per ruimte. Bij een structureel geldig project, volledig geclassificeerde grenzen en een bevestigde puntkoudebruginventaris verschijnt de diagnostische som in `summary.unheatedTransmissionDiagnostic` en de project-UI. Ontbrekende koppelingen of factoren leveren geen getal. Ruimte-ID en factor kunnen via de UI worden ingevoerd; de factorbron wordt opgeslagen maar niet inhoudelijk geverifieerd.

## Voor normatieve vrijgave

1. Controleer de exacte §8.4-route en de methode voor `b` tegen de volledige doeluitgave, inclusief ventilatie, koudebruggen, eenheden, grenzen en afronding.
2. Controleer per project de werkelijke ruimte-/grenskoppelingen en verifieer de herkomst van elke factor buiten de huidige invoercontrole.
3. Vergelijk de deelposten en grensgevallen met onafhankelijke actuele ISSO/EDR-waarden. Pas daarna mag dit blok aan de maandvraag en BENG-route worden gekoppeld.
