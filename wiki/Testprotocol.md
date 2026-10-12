# Testprotocol

Het testprotocol beschrijft wat er getest wordt, hoe, en wanneer een test geslaagd is. Het is bedoeld voor de attesteringsinstelling (BRL 9501) en voor bijdragers.

Onderdelen (zie ook [[Verificatie en testen]]):
- **Gate:** `scripts/verify-nta.sh` moet met exitcode 0 eindigen.
- **Rust 1.77.2:** de tests van de rekenkern op de minimale Rust-versie.
- **Referentiesuites:** banden per geval. Een suite met alleen deeltests zonder verwachte uitkomst geeft los gedraaid het oordeel *geen vergelijking* (exitcode 3); in de gate vergelijken de openbare gevallen en RVO wel (exitcode 0).
- **Testset ISSO 54:** de voorbereiding op versie 5.0; zie [docs/nta8800-isso54-voorbereiding.md](https://github.com/Erika98-be/open-energy-studio/blob/nta8800-kernel/docs/nta8800-isso54-voorbereiding.md).
- **Releases:** per release worden de gate- en referentierapporten bewaard (`scripts/release-nta.sh`).

Het volledige protocol: [docs/nta8800-testprotocol.md](https://github.com/Erika98-be/open-energy-studio/blob/nta8800-kernel/docs/nta8800-testprotocol.md).
