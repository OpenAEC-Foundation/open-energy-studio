# Projectberekening

De projectberekening is de volledige NTA 8800-invoer:

| Stap | Belangrijkste invoer |
|---|---|
| Gebouw | rekenzones en gebruiksfuncties, schil (vlakken, ramen, koudebruggen), belemmering, zonwering en glas per raam, kruipruimte en kelder, onverwarmde ruimten |
| Installaties | verwarming (opwekking, distributie, afgifte, regeling), tapwater, ventilatie, koeling, PV, verlichting (utiliteit), GBS/BACS |

- **Opgegeven productwaarden:** kwaliteitsverklaringen vervangen de forfaitaire waarden. Voorbeelden zijn een gedeclareerde f<sub>ctrl</sub>, ketelconstanten A/B/C, WKK-rendementen of warmtepomp-COP's. Bewaar het bewijs; zie [[Bron en bewijs]].
- **Gemeenschappelijke ruimten (6.6.4):** geef per rekenzone de gemeenschappelijke ruimte op met de functies die ze bedient. De kern verdeelt haar binnen de zone (setpoint, interne warmte, ventilatie) en over het gebouw (Bbl- en labelfuncties). Op verzoek vult de kern ook de tapwater- en verlichtingslijsten met de verdeling binnen de zone. Het rapport toont de verdeling.
- **Kelder en grond:** de wand van een verwarmde kelder telt met de opgegeven wandoppervlakte (bijlage K) in A<sub>ls</sub> en in 8.38. Alleen vloeren kunnen aan de grond grenzen; een wand of dak tegen de grond wordt geweigerd.
- **Daglicht:** per sector kan een opgegeven F<sub>D;dayl</sub> volgens bijlage F van NEN-EN 15193-1, met bron.
- **Lege velden:**
  - een leeg veld met een standaardwaarde in de kern krijgt die standaardwaarde;
  - een leeg verplicht veld krijgt een melding met **Ga naar**.
- **Formulieren:** vrijwel elke invoeroptie van de kern heeft een formulier. Een klein aantal expertroutes staat alleen onder *Geavanceerd (JSON)*; zie [formulierdekking](https://github.com/Erika98-be/open-energy-studio/blob/nta8800-kernel/docs/ui-redesign/formulierdekking.md).

Details: [hoofdstuk 3 van de handleiding](https://github.com/Erika98-be/open-energy-studio/blob/nta8800-kernel/docs/handleiding-nta8800/03-projectberekening.md).
