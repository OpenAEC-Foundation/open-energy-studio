# Wiki publiceren

Deze map is opgezet als GitHub-wiki. Publiceren, vanuit de hoofdmap van de repository:

```bash
git clone https://github.com/Erika98-be/open-energy-studio.wiki.git /tmp/oes-wiki
cp wiki/*.md /tmp/oes-wiki/
rm /tmp/oes-wiki/README-publiceren.md
cd /tmp/oes-wiki
git add -A
git commit -m "Wiki bijwerken"
git push
```

Voorwaarden:
- **Wiki aan:** de wiki moet aanstaan (*Settings › Features › Wikis*).
- **Eerste pagina:** er moet minstens één pagina via de webinterface zijn aangemaakt. Anders bestaat de `.wiki.git`-repository nog niet.

Pagina's die uit `wiki/` zijn verwijderd, verwijder je met `git rm` in `/tmp/oes-wiki`; kopiëren alleen haalt ze daar niet weg.

Paginanamen volgen de bestandsnamen: `Bron-en-bewijs.md` wordt de pagina "Bron en bewijs". Links in de vorm `[[Bron en bewijs]]` verwijzen naar die pagina's.
