# Diagnostische maandelijkse directe warmteflow

De Rust-kern heeft naast de directe buitentransmissiesom een getypeerde maandfunctie. Met expliciet aangeleverde buitencomponenten bepaalt zij eerst `H_direct = Σ(A·U) + Σ(L·Ψ) + Σχ` in W/K. Voor elk van twaalf aangeleverde maanden berekent zij vervolgens de **getekende** energiestroom `Q = H_direct × (T_binnen − T_buiten) × uren / 1000` in kWh. Een negatieve waarde betekent dat buiten warmer is dan binnen. De jaarsom is de som van deze getekende waarden; zij is geen verwarmingsbehoefte.

De functie kiest geen NTA-klimaatdata, binnentemperatuur, maanduren, thermische grens, correctie, grondroute, ventilatie, benuttingsfactor of warmtepompbedrijf. Zij berekent geen BENG, TO-juli of energielabel. [De directe-transmissiediagnose](nta8800-directe-transmissie.md) heeft dezelfde bewijsgrens: de primaire normtekst en een onafhankelijke verwachte deeluitkomst zijn nog niet geverifieerd. De velden `referenceVerified=false` en `bengCalculationAvailable=false` blijven vast. De overeenkomst met een handvoorbeeld controleert eenheden en implementatie, geen NTA-conformiteit.

## HTTP en MCP

`POST /v1/nta8800/transmission/direct/monthly-diagnose` en MCP `diagnose_monthly_direct` accepteren dezelfde `input`:

```json
{
  "input": {
    "direct": {
      "elements": [
        {"id":"wall", "areaM2":10, "uValueWPerM2k":0.2, "sourceReference":"drawing-1"}
      ]
    },
    "months": [
      {"month":1, "indoorTemperatureC":20, "outdoorTemperatureC":10, "hours":100}
    ]
  }
}
```

Dit fragment toont de vorm van één maand, maar is opzettelijk **onvolledig**. Een geldige aanvraag vereist precies twaalf unieke maandnummers 1–12, eindige temperaturen en positieve eindige uren. Ontbrekende/dubbele maanden, ongeldige directe invoer of numerieke overloop leveren HTTP 422 of een MCP-toolerror op, zonder maand- of jaarenergie. De volledige testinvoer met twaalf maal dezelfde conditie geeft `H_direct=2 W/K`, `2 kWh` per maand en `24 kWh` getekende jaarflow. De response bevat ook doelversie, kernelversie en SHA-256-invoervingerafdruk.

## Vervolg naar een NTA-rekenroute

De normatieve maandketen vergt gecontroleerde normparagrafen, aangewezen klimaat- en binnentemperatuurdata, grond en aangrenzende begrenzingen, ventilatie, zon- en interne winsten, benutting/thermische massa, installaties en onafhankelijke verwachte deelresultaten. Pas na die review kan deze rekensom als een gecontroleerde deelpost in een volledige vraag- en BENG-route worden gebruikt. Het [dekkingsregister](nta8800-dekkingsregister.md) houdt de huidige status bij.
