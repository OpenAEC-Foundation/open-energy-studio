# Import en export

- **Uniec 3 en VABI:** elke import legt de bestandsnaam, een SHA-256 en een lijst *importpunten* vast.
  - Importpunten zijn standaardwaarden, weggelaten gegevens, begrensde waarden, onbekende of dubbele verwijzingen en meerdere gebouwen.
  - Je ziet ze na de import in een melding, en onder *Projectinformatie › Importverslag*.
  - Geïmporteerde projecten doorlopen dezelfde controle als geopende projecten.
- **VABI:**
  - koudebruggen worden eenmaal per rekenzone geschreven;
  - de U-waarde blijft behouden;
  - helling en horizontale vlakken komen correct over;
  - de bestanden mogen UTF-8, UTF-16 of windows-1252 zijn;
  - bestanden met een `<!DOCTYPE` worden geweigerd.
- **Export:** de app meldt welke projectgegevens het formaat niet kan bevatten, zoals de NTA-specifieke invoer.
- **Vloeren op grond:** de grens grond/kruipruimte/kelder, de perimeter P en een verwarmde kelder (z, wandoppervlak) gaan in geen van beide formaten mee. De export noemt `ntaCalculation.groundFloors` apart; na een import vul je ze opnieuw in.
- **Overige exports:** IFC, het BENG-rapport, het NTA-rekenrapport, het invoerdossier en het projectdossier.

> De veldcodes van Uniec 3 en VABI zijn afgeleid; er is nog geen echt voorbeeldbestand getest. Zie [[Bekende beperkingen]].

Details: [hoofdstuk 9 van de handleiding](https://github.com/Erika98-be/open-energy-studio/blob/nta8800-kernel/docs/handleiding-nta8800/09-bestanden-en-uitwisseling.md).
