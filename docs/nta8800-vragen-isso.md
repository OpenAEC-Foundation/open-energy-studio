# Vragen aan ISSO over de basisopname (ISSO 82.1 en 75.1)

Concepten voor vragen aan ISSO over punten waar de basisopname (ISSO 82.1 voor woningen, ISSO 75.1 voor utiliteitsgebouwen) een andere standaardwaarde gebruikt dan NTA 8800:2025+C1:2026 voor dezelfde onbekende invoer. Ze komen uit review 76 (`crates/nta8800-core/tests/opname_defaults_independent.rs`), waarin circa 45 standaardwaarden van de opname tegen NTA 8800 zijn gelegd. De kern volgt in deze gevallen de opnamemethodiek van ISSO; de vragen gaan over de vraag of dat de bedoeling is. Alleen paginanummers en waarden, geen tekst uit de documenten.

## 1. Bouwjaar in plaats van een onbekend toestel- of isolatiejaar

**Situatie.** Als het jaar van een toestel, een leidingisolatie of een voorraadvat niet bekend is, gebruikt de opname het bouwjaar van het gebouw (ISSO 82.1 p. 28 en de tabellen 9.12 en 11.15, p. 174). De tabellen van NTA 8800 hebben voor die gevallen een eigen rij "onbekend", die meestal de ongunstigste waarde geeft. Het verschil treedt alleen op bij gebouwen die nieuwer zijn dan de tabelgrens.

| Grootheid | NTA 8800 | Kern (bouwjaar) | Voorbeeld | Plaats in de kern |
|---|---|---|---|---|
| Specifiek ventilatorvermogen f_SFP, gelijkstroom | tabel 11.23: onbekend 2,20/3,6 | 0,45/3,6 (bouwjaar na 2006) | woning 2015 | `opname/ventilation.rs`, `opname/utility.rs` |
| Ψ van de leidingisolatie verwarming | tabel 9.16: onbekend | jaarklasse van het bouwjaar (0,3 tegen 0,4) | woning 2015 | `opname/heating.rs` |
| Ψ van de leidingisolatie koeling | tabel 10.9: onbekend | jaarklasse van het bouwjaar (0,3 tegen 0,4) | kantoor 2015 | `opname/utility.rs` |
| Hulpenergie ketel A (9.6.8.1.1.2) | onbekend jaar | jaarklasse van het bouwjaar (43,8 tegen 87,6 kWh) | woning 2015 | `opname/general.rs` (`device_year`) |
| Label van een elektrische boiler (13.6.3, p. 569) | onbekend jaar | woning: bouwjaar (label C tegen G); utiliteit: tot en met 2017 volgens de NTA | woning 2020 | `opname/hot_water.rs`, `opname/utility.rs` |
| PV-paneel en gasopslag | onbekend jaar | bouwjaar | — | `opname/production.rs` |

**Vraag.** Is het de bedoeling dat de basisopname bij een onbekend toestel-, isolatie- of vatjaar het bouwjaar aanhoudt, ook waar NTA 8800 een rij "onbekend" heeft? Zo ja: geldt dat ook voor de elektrische boiler bij woningen, waar ISSO 75.1 voor utiliteitsgebouwen de NTA-grens van 2017 aanhoudt?

## 2. Ontwerpklasse van de afgifte bij een onbekende temperatuur

**Situatie.** ISSO 82.1 tabel 9.9 leidt de ontwerpklasse af uit het afgiftesysteem (vloerverwarming 45/40, lagetemperatuurradiatoren en warmtepompen 55/47 of 70/50). NTA 8800 houdt bij een onbekende ontwerptemperatuur 90/70 aan (p. 307). De kern volgt ISSO (`opname/heating.rs`).

**Vraag.** Mag de opname de ontwerpklasse uit het afgiftesysteem afleiden, of geldt bij een onbekende ontwerptemperatuur altijd 90/70?

## 3. Bypass van een warmteterugwinunit met onbekend fabricagejaar (§11.3.2.2)

**Situatie.** §11.3.2.2 geeft 100 % bypass bij een bouw- of fabricagejaar vanaf 2010 en 0 % bij een onbekend fabricagejaar. Bij een woning vanaf 2010 met een unit van onbekend jaar spreken die regels elkaar tegen. De kern geeft dan 100 %.

**Vraag.** Welke regel gaat voor bij een woning vanaf 2010 met een WTW-unit van onbekend fabricagejaar?

## 4. Zontoetredingsfactor van drievoudig HR-glas

**Situatie.** ISSO 82.1 tabel 8.14 geeft voor drievoudig HR-glas g 0,5. Tabel 7.4 van NTA 8800 geeft 0,40 voor glas met twee coatings, wat drievoudig HR-glas meestal heeft. De kern volgt ISSO.

**Vraag.** Geldt in de opname voor drievoudig HR-glas g 0,5 (ISSO tabel 8.14) of de NTA-waarde voor glas met twee coatings?
