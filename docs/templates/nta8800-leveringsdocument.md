# Leveringsdocument {{softwareName}} {{programVersion}}

Dit leveringsdocument hoort bij de levering van het NTA 8800-rekenprogramma (BRL 9501 §6.1). Het wordt bij elke vrijgave gegenereerd door `scripts/release-nta.sh` uit `src/core/nta/attest.json`, de versienummers in de broncode en de gebouwde pakketten.

| Gegeven | Waarde |
| --- | --- |
| Programma | {{softwareName}} |
| Programmaversie | {{programVersion}} |
| Rekenkernversie (`KERNEL_VERSION`) | {{kernelVersion}} |
| Normversie (`TARGET_NORM_VERSION`) | {{targetNormVersion}} |
| Attestnummer (BRL 9501) | {{attestNumber}} |
| Identificatiecode | {{identificationCode}} |
| Attesteringsinstelling | {{attestingBody}} |
| Broncode (commit) | {{commit}} |
| Git-tag | {{tag}} |
| Datum van vrijgave | {{releaseDate}} |

## Pakketten

| Bestand | SHA-256 |
| --- | --- |
{{packages}}

## Bijgeleverde verificatie

- De testrapporten van deze vrijgave (gate en referentieberekeningen) staan naast dit document in het vrijgavearchief, met hun eigen SHA-256 in `SHA256SUMS`.
- Wijzigingen ten opzichte van de vorige versie staan in de releasenotes onder "Rekenkern {{kernelVersion}}" (`docs/nta8800-releasenotes.md`).

{{attestStatement}}
