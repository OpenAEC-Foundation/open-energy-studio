# Verificatie en testen

Elke wijziging aan de berekening gaat door de gate:

```bash
scripts/verify-nta.sh
```

De gate controleert:
- **Versies:** de kernversie tegen de releasenotes, en de stempel van de handleiding.
- **Rekenkern:**
  - eenheidstests en tests per uitgave;
  - optiedekking: elke variant en elk optioneel veld van de kerninvoer, in elke uitgave;
  - numerieke fuzz: grenswaarden, willekeurige waarden en monotonie;
  - uitgaven: bronlabels per uitgave over de hele uitvoer, en het overzicht bij een wissel van uitgave;
  - grote projecten: een dicht utiliteitsgebouw (50 zones, 2 250 ramen) met tijdsbudget, en de zoektocht naar lege velden tegen de oude zoektocht als orakel.
- **Referentiesuites:**
  - de openbare rapporten A–O, als regressie op de vastgelegde uitkomsten van deze kern; de verschillen met de gepubliceerde waarden zijn per post verklaard;
  - de RVO-voorbeeldwoningen;
  - 260 van de 266 ISSO 54 v2.0-deeltests, als *pending* tot de verwachte uitkomsten beschikbaar zijn.
- **Service, API, MCP en desktopcommando's.**
- **Frontend (Vitest, 139 bestanden):** de end-to-end werkstroom, formulierdekking, formulierbewaking (formulierstanden door de kern), toegankelijkheid (namen, foutkoppeling, *Ga naar* en toetsenbord in alle formulieren en dialogen) en vertalingen.

De rekenkern wordt ook getest op Rust 1.77.2 (MSRV). Zie [[Testprotocol]].

Details: [hoofdstuk 5 van de handleiding](https://github.com/Erika98-be/open-energy-studio/blob/nta8800-kernel/docs/handleiding-nta8800/05-validatie.md) en [openbare rapporten](https://github.com/Erika98-be/open-energy-studio/blob/nta8800-kernel/docs/nta8800-vergelijking-openbare-rapporten.md).
