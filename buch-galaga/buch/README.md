# Das Buch (Word, farbig, druckbar)

Das fertige Dokument ist **`GameBasic-Buch.docx`** — öffne es in **Word** oder
**LibreOffice**, bearbeite es nach Belieben und drucke es aus. Es enthält bisher
die **Einleitung** (Was ist GameBasic, was kann es, Vorstellung des Galaga-
Projekts) mit Farb-Screenshots.

## Selbst bearbeiten
Einfach `GameBasic-Buch.docx` in Word öffnen und ändern — Text, Farben, Bilder,
alles frei editierbar. Zum Drucken: Datei → Drucken.

## Bilder
Die Screenshots liegen in [`images/`](images) (PNG, farbig). Du kannst eigene
einfügen oder die vorhandenen ersetzen.

## Neu erzeugen (optional, für Entwickler)
Das Dokument wird aus `build_book.js` (Node + docx-js) generiert:

```
cd buch-galaga/buch
npm install      # einmalig (lädt das docx-Paket)
node build_book.js
```

> Hinweis: Erzeugst du das Dokument neu, werden eigene Word-Änderungen
> überschrieben. Bearbeite also entweder die `.docx` direkt **oder** das Skript —
> nicht beides parallel.

## Screenshots neu aufnehmen
Die Bilder werden headless aus dem Spiel/den Beispielen erzeugt, z. B.:

```
set DHRT_FRAMES=40
set DHRT_SCREENSHOT=buch/images/galaga_titel.png
dhrt run buch-galaga/code/galaga.dh
```
