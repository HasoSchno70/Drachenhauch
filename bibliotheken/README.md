# Libraries / Bibliotheken

Small libraries for Drachenhauch programs, fetched as packages with
`dhrt paket` -- each folder is one package.

Kleine Bibliotheken für Drachenhauch-Programme, geholt als Pakete mit
`dhrt paket` -- jeder Ordner ist ein Paket.

```bash
dhrt paket hole github:HasoSchno70/Drachenhauch@main#bibliotheken/highscores
```

| Package | What it does / Was es tut |
|---|---|
| [highscores](highscores/) | a high-score table in a file / eine Bestenliste in einer Datei |
| [menu](menu/) | a title or pause menu / ein Titel- oder Pausenmenü |
| [dialogbox](dialogbox/) | a text box that types letter by letter / ein Textkasten, der Buchstabe für Buchstabe tippt |

Use a version tag instead of `@main` to pin the state (`@v2026.53` and
later). More in the [package manual](../docs/en/pakete.md) /
im [Paket-Handbuch](../docs/pakete.md). Each package is checked in
`tests/pruef/werkzeugkette/bibliotheken.dhtest`.
