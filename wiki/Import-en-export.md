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
- **Vloeren op grond:** de export schrijft de grens grond/kruipruimte/kelder, de perimeter P en een verwarmde kelder (z, wandoppervlak) in geen van beide formaten; hij noemt `ntaCalculation.groundFloors` apart. De import van een echte Uniec-export neemt de perimeter over; de rest vul je na een import opnieuw in.
- **Overige exports:** IFC, het BENG-rapport, het NTA-rekenrapport, het invoerdossier en het projectdossier.

- **Uniec 3, echte export:** de import van een export van Uniec 3.4 is getest op één geanonimiseerde woning. Dakhellingen, de omtrek van de vloer op de grond en belemmering en zonwering per raam komen mee; bij meer gebouwen in één bestand leest de import het eerste en meldt de rest. De registratie vraagt per import een bevestiging dat de gegevens zijn gecontroleerd.
- **Uniec 3, export:** een nagebouwd invoerconcept (`.input-draft.uniec3`); niet vastgesteld is dat Uniec het accepteert.
- **Uniec 3, geometrie (Excel, concept):** de geometrie van een grondgebonden woning volgens de openbare Excel-import van Uniec (specificatie 18-09-2026). De app weigert met de reden als verplichte gegevens ontbreken; nog niet in Uniec getest. Zie [de vertaling](https://github.com/OpenAEC-Foundation/open-energy-studio/blob/main/docs/nta8800-uniec-geometrie-export.md).

> Brede compatibiliteit met Uniec- en VABI-bestanden is niet aangetoond; de VABI-veldcodes zijn afgeleid. Zie [[Bekende beperkingen]].

Details: [hoofdstuk 9 van de handleiding](https://github.com/OpenAEC-Foundation/open-energy-studio/blob/main/docs/handleiding-nta8800/09-bestanden-en-uitwisseling.md).
