# Ondertekende installers — issue #18

De releaseworkflow bouwt zes formaten: NSIS EXE, MSI, universele macOS DMG,
DEB, RPM en AppImage. Publicatie blijft handmatig. Een geslaagde workflowrun
moet bij de release worden vastgelegd vóór publicatie. Gewone CI-builds zijn
ontwikkelartefacten en vormen geen ondertekende releaseacceptatie.

## Bestaande organisatiecredentials koppelen

Op 29 oktober 2026 waren de namen van alle zes onderstaande Azure-secrets
zichtbaar in deze repository. De waarden zijn niet uitgelezen. Apple- en
OpenPGP-secrets en de uitgevervariabele waren niet zichtbaar. Organisatiebrede
secretinventarisatie gaf HTTP 403; dit bewijst niet dat de organisatie die
credentials niet heeft. Deel bestaande credentials met deze repository via
GitHub Actions secrets; vraag geen nieuwe licentie aan op basis van deze audit.
De controle `signing-config` stopt bij ontbrekende namen, vóór installeruploads.

| Platform | GitHub Actions-configuratie |
| --- | --- |
| Windows | Secrets `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, `AZURE_TENANT_ID`, `AZURE_ENDPOINT`, `AZURE_TRUSTED_SIGNING_ACCOUNT_NAME`, `AZURE_CERTIFICATE_PROFILE_NAME` |
| Windows-uitgever | Repositoryvariabele `WINDOWS_SIGNING_SUBJECT`: de volledige verwachte certificaat-Subject, exact zoals `Get-AuthenticodeSignature` deze levert. Bevestigen met de organisatiebeheerder; nooit afleiden uit een willekeurige nieuwe download. Korte Azure-certificaatgeldigheid maakt een vast leaf-thumbprint ongeschikt. |
| macOS | Secrets `APPLE_CERTIFICATE` (base64 P12), `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY` (`Developer ID Application: …`), `KEYCHAIN_PASSWORD`, `APPLE_ID`, `APPLE_PASSWORD` (app-specifiek), `APPLE_TEAM_ID` |
| OpenPGP | Secret `RELEASE_GPG_PRIVATE_KEY` met de bestaande releasekey; optioneel `RELEASE_GPG_PASSPHRASE` als die key beveiligd is. Geen sleutel genereren ter vervanging van de organisatiekey. |

Vertrouwde OpenPGP-primaryfingerprint uit de audit:
`13F6D414ADEE4ED4CD09AC60601D86C93C504D29`.
De workflow exporteert alleen die publieke key als `OpenAEC-release-signing.asc`.
Een ander primaryfingerprint moet eerst onafhankelijk worden gekwalificeerd;
een nieuw sleutelbestand naast een release creëert op zichzelf geen vertrouwen.

## Werking en controles

Windows gebruikt de bestaande Azure-voorziening via Microsofts
`ArtifactSigning` PowerShell-module (0.1.20). De release-overlay
`src-tauri/tauri.release.windows.conf.json` stelt Tauri's `signCommand` in.
Tauri roept de hook tijdens bundelen aan voor eigen binaries, de gegenereerde
NSIS-uninstaller en de eindinstaller. Elke hookaanroep moet eindigen met een
geldige Authenticode-handtekening, de verwachte uitgever en een tijdstempel.
Daarna downloadt de Windows-runner de aangeboden EXE en MSI opnieuw,
controleert ze, installeert beide afzonderlijk, controleert alle geïnstalleerde
EXE/DLL-bestanden en voert de uninstall uit. Nieuwe externe binaries vragen een
expliciet beoordeeld uitgeverbeleid; de controle slaat ze niet stilzwijgend over.
De echte Windows-run moet ook bevestigen dat de NSIS-uninstallerhook in de
vastgelegde Tauri-versie werkt; lokale beleidsmocks bewijzen dit niet.

macOS importeert het bestaande Developer ID-certificaat in een tijdelijke
keychain. Tauri ondertekent en notariseert de app met de bestaande instellingen;
er worden geen entitlements vervangen. Daarna wordt ook de DMG ondertekend,
genotariseerd en gestapled. De opnieuw gedownloade DMG wordt readonly gemount.
De daarin aanwezige app moet `codesign --verify --deep --strict`, een Developer
ID Application-authority met de verwachte Team ID, Gatekeeper en
`stapler validate` doorstaan. De DMG zelf wordt ook gecontroleerd.
De tijdelijke keychain en het tijdelijke P12-bestand worden verwijderd.

Voor Linux is gekozen voor een OpenPGP-ondertekende checksumlijst:
`SHA256SUMS-installers.txt` met `.asc`. Die lijst bevat exact één bestand per
formaat, inclusief de drie Linux-formaten. Dit levert losse cryptografische
verificatie; het claimt geen ingebedde DEB/RPM/AppImage-handtekening.
De lijst wordt pas gemaakt over de werkelijk geüploade bestanden, nadat alle
platformcontroles zijn geslaagd. Alle bestanden, de key en de handtekening
worden opnieuw gedownload. Een afzonderlijke lege keyring, gepind op het
bovenstaande primaryfingerprint, controleert de handtekening en alle hashes.
Een ontbrekend formaat, extra installer, dubbel record of gewijzigde byte stopt
de workflow. Checksums worden nooit als vervanging voor signing beschouwd.

Deze repository configureert momenteel geen Tauri updater-plugin of
`createUpdaterArtifacts`. Er wordt dus geen `latest.json` of minisign-handtekening
verzonnen. De workflow weigert een nieuwe updaterconfiguratie tot de bijbehorende
signing-/verificatieketen expliciet is toegevoegd. Bestaande historische updater-
of releasebestanden worden niet veranderd.

## Vrijgave en historische links

Gebruik een nieuwe versie/tag die de huidige sources en deze workflow bevat.
De workflow weigert gepubliceerde releases te wijzigen en weigert bestaande
installerassets te behouden. Upload daarom het lokaal gemaakte, ongetekende
DEB-pakket van `release-nta.sh` niet vooraf als release-installer; dat pakket
blijft met zijn eigen manifest en leveringsdocument in het lokale bewijsarchief.
De zes voor download aangeboden installers worden door de workflow gemaakt.
Een mislukte run kan een gedeeltelijk gevuld draft achterlaten. Voor een retry
moeten de gedeeltelijke installerassets bewust worden verwijderd of moet een
nieuwe draft/tag worden gebruikt. Bestaande gepubliceerde bytes worden niet
stilzwijgend vervangen.

Sluit #18 pas na een echte ondertekende release met geslaagde controles, een
link naar de workflowrun en controle van de werkelijk aangeboden downloads.
Controleer ook de vaste verwijzingen op `https://open-aec.com/#ecosysteem`:
oude unsigned Linux- of ad-hoc macOS-downloadlinks mogen niet als opgelost
worden aangemerkt zolang de website nog naar die versies verwijst. De workflow
wijzigt die externe website niet. Snap- en winkelassertions vallen buiten de
hier aanwezige zes bundelformaten.

Een gebruiker kan Linux-downloads controleren met GnuPG en Python:

```sh
gpg --import OpenAEC-release-signing.asc
gpg --fingerprint 13F6D414ADEE4ED4CD09AC60601D86C93C504D29
# Vergelijk het fingerprint met de hierboven onafhankelijk vertrouwde waarde.
gpg --export 13F6D414ADEE4ED4CD09AC60601D86C93C504D29 > trusted-key.gpg
gpgv --keyring "$PWD/trusted-key.gpg" SHA256SUMS-installers.txt.asc SHA256SUMS-installers.txt
# Download alle zes installers voor de strikte manifestcontrole:
python3 scripts/signing/manifest.py verify .
# Of controleer de hashes van gedownloade individuele bestanden uit de getekende lijst:
sha256sum --check --ignore-missing SHA256SUMS-installers.txt
```

## Lokale validatie en grenzen

```sh
python3 -m unittest discover -s scripts/signing -p 'test_*.py' -v
pwsh -NoProfile -File scripts/signing/test-windows-verify.ps1
for script in scripts/signing/*.sh; do bash -n "$script"; done
actionlint .github/workflows/release.yml .github/workflows/installer-signing-checks.yml
```

De beleidsmocks bevestigen afwijzing van ongetekend/gewijzigd bewijs, een
verkeerde uitgever, een ontbrekende tijdstempel en een ontbrekende uitgeverpin.
Zij zijn geen daadwerkelijke Authenticode-, installatie-, Gatekeeper- of
notarizationproef. Voor merge blijven de volledige project-CI en voor publicatie
de echte release-signingrun nodig. Ontbrekende credentials zijn geen bewezen
ontbrekende licenties.

## Primaire documentatie

- [Tauri Windows signing en custom signCommand](https://v2.tauri.app/distribute/sign/windows/)
- [Microsoft Artifact Signing Action en gebruikte PowerShell-module](https://github.com/azure/trusted-signing-action)
- [Tauri macOS signing en notarization](https://v2.tauri.app/distribute/sign/macos/)
- [Tauri updater: eigen verplichte handtekeningen](https://v2.tauri.app/plugin/updater/)
