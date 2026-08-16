# Das Buch (Word, farbig, druckbar)

Das fertige Dokument ist **`Drachenhauch-Tippspiel.docx`** — öffne es in **Word**
oder **LibreOffice**, bearbeite es nach Belieben und drucke es aus. Es enthält
das vollständige Buch: Vorwort, Einleitung und alle 13 Kapitel mit Quelltext,
Screenshots und Erklärkästen.

Umfang: 31 Seiten, Inhaltsverzeichnis mit Seitenzahlen.

## Selbst bearbeiten

Einfach `Drachenhauch-Tippspiel.docx` in Word öffnen und ändern — Text, Farben,
Bilder, alles frei editierbar. Zum Drucken: Datei → Drucken.

## Neu erzeugen (für Entwickler)

```
cd buch-tippspiel/buch
npm install      # einmalig (lädt das docx-Paket)
node build_book.js
```

> **Hinweis:** Erzeugst du das Dokument neu, werden eigene Word-Änderungen
> überschrieben. Bearbeite also entweder die `.docx` direkt **oder** das
> Skript — nicht beides parallel.

### Mit Seitenzahlen im Inhaltsverzeichnis

`build_book.js` allein baut das Verzeichnis ohne Seitenzahlen (die kennt es zu
diesem Zeitpunkt noch nicht). Der Zwei-Pass-Build misst sie:

```
python make_book.py
```

Pass 1 baut das Dokument, lässt LibreOffice ein PDF daraus rendern und sucht
darin jede Überschrift; Pass 2 baut mit den gemessenen Zahlen neu. Das Layout
bleibt zwischen beiden Pässen stabil, weil das Verzeichnis gleich viele Zeilen
belegt — nur die Zahlen kommen hinzu.

Braucht LibreOffice (`C:\Program Files\LibreOffice\program\soffice.exe`) und
PyMuPDF (`pip install pymupdf`). Fehlt eines von beiden, sagt das Skript es und
das Dokument entsteht trotzdem — nur ohne Seitenzahlen.

## Bilder

Die Screenshots liegen in [`images/`](images) und werden **aus den
Kapitelständen selbst** aufgenommen, nicht von Hand geschossen. So zeigt das
Buch immer das, was der Leser wirklich sieht, wenn er den Stand startet:

```
DHRT_FRAMES=60 DHRT_SCREENSHOT=buch/images/kap06_punkte.png \
    dhrt run buch-tippspiel/code/kap06/punkte.dh
```

Kapitel mit mehreren Reitern brauchen einen Handgriff, weil headless niemand
klicken kann: vor der Hauptschleife ein `GUI_SET_ACTIVE_TAB(win, 1)` einfügen,
Bild aufnehmen, Zeile wieder entfernen.

## Aufbau des Skripts

`build_book.js` ist bewusst so gebaut wie das Gegenstück im Galaga-Band
(`buch-galaga/buch/build_book.js`): dieselben Bausteine, dieselbe Farbwelt —
damit beide Bände zusammen im Regal stehen können.

| Baustein | Wofür |
|---|---|
| `p()`, `pmix()` | Fließtext, wahlweise mit Inline-Code |
| `codeBlock()` | Quelltext (grauer Kasten, blaue Leiste) |
| `konsole()` | Programmausgabe (dunkler Kasten) |
| `figure()` | Bild mit Unterschrift, automatisch skaliert |
| `tip()` | „In diesem Kapitel" / „Zum Ausprobieren" (blau) |
| `why()` | „Warum so?" — die Begründung einer Entscheidung (amber) |
| `warn()` | „Vorsicht" — Fallen, in die man einmal tritt (rot) |
| `chapter()`, `h1()`, `h2()` | Überschriften; `chapter()` beginnt eine neue Seite |

Fehlt ein Bild, warnt der Build und lässt die Stelle aus, statt abzubrechen.
