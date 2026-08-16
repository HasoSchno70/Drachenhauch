# Tippspiel — eine richtige Anwendung, Schritt für Schritt mit Drachenhauch

Ein schlankes Lehrbuch, das eine **Bundesliga-Tippanwendung von Grund auf baut**:
Spiele in einer Datenbank, Tipps eingeben, Ergebnisse eintragen, Punkte rechnen,
Rangliste führen, Spielplan aus dem Netz holen. Am Ende steht ein Programm, das
man einer Tipprunde wirklich in die Hand geben kann.

Das Gegenstück zum [Galaga-Buch](../buch-galaga/README.md): dort ein Spiel, hier
**die erste ernsthafte Anwendung**. Ein Spiel darf beim Beenden alles vergessen —
eine Anwendung nicht. Deshalb steht hier von Anfang an die Frage im Mittelpunkt,
die jede Anwendung trägt: *wo leben die Daten, und wer darf sie ändern?*

## Das Buch

Das fertige Buch liegt als Word-Dokument bereit — 31 Seiten, farbig, druckbar:
**[`buch/Drachenhauch-Tippspiel.docx`](buch/Drachenhauch-Tippspiel.docx)**.
Vorwort, Einleitung und alle 13 Kapitel mit Quelltext, Screenshots und
Erklärkästen. Wie man es neu erzeugt, steht in [`buch/README.md`](buch/README.md).

## Zwei Wege hinein

**Der Schnitt.** [`code/tippspiel.dh`](code/tippspiel.dh) ist der senkrechte
Schnitt durch die Anwendung: von der Datenbank bis zum Fenster jede Schicht
einmal, klein gehalten. Er ist in einem Stück lesbar und zeigt in einer
Viertelstunde, worum es geht:

```
dhrun.py buch-tippspiel/code/tippspiel.dh
```

**Der Zielstand.** Das fertige Programm ist der Stand von Kapitel 12,
[`code/kap12/politur.dh`](code/kap12) — dasselbe Programm, aber mit allem, was
die Kapitel unterwegs dazugeben: Anstoßzeiten und Tippschluss, Spielplan aus
dem Netz, Sicherung und Einspielen, drei Reiter mit Punktediagramm:

```
dhrun.py buch-tippspiel/code/kap12/politur.dh
```

Beide legen ihre Datenbank beim ersten Start selbst an (drei Spieler, fünf
Spiele des 1. Spieltags). **[ESC]** beendet.

## Aufbau (13 Kapitel)

| # | Kapitel | Was dazukommt | Thema |
|---|---|---|---|
| 1 | [Das erste Fenster](code/kap01) | Fenster, Schleife, Titel, ein Knopf, der etwas tut | Programmaufbau, `SCREEN`, `WHILE`, `GUI_*` |
| 2 | [Wo die Daten wohnen](code/kap02) | SQLite: Tabellen anlegen, Spiele eintragen, auslesen — in der Konsole | `db`-Modul, `CREATE TABLE`, `INSERT`, `SELECT` |
| 3 | [Die Regel](code/kap03) | `punkte()`: 3 / 2 / 1 / 0 — und ein Programm, das sich selbst prüft | `FUNCTION`, `IF`, Selbstprüfung |
| 4 | [Der Spielplan im Fenster](code/kap04) | Tabelle mit den Spielen; jede Zeile merkt sich ihre `id` | `GUI_TABLE`, Arrays, Ansicht ≠ Wahrheit |
| 5 | [Tipps eingeben](code/kap05) | Spinner, Speichern-Knopf, ein Tipp je Spieler und Spiel | `GUI_SPINNER`, `UNIQUE` + `UPSERT` |
| 6 | [Ergebnisse und Punkte](code/kap06) | Ergebnis eintragen, alle Punkte neu rechnen, Ausbeute einfärben | Transaktionen, `GUI_TABLE_CELL_COLOR` |
| 7 | [Die Rangliste](code/kap07) | Zweiter Reiter: Punkte je Spieler, sortiert | `GUI_TABS`, `GROUP BY`, `CASE WHEN` |
| 8 | [Zeit](code/kap08) | Anstoßzeiten, Tippschluss, Countdown | `zeit`-Modul, Zeitpunkt als Zahl |
| 9 | [Daten aus dem Netz](code/kap09) | Spielplan von OpenLigaDB — ohne dass das Fenster steht | `HTTP_GET_START`, `json`-Modul |
| 10 | [Wenn etwas schiefgeht](code/kap10) | Fehler abfangen, Eingaben prüfen, Rückfragen stellen | `TRY`/`CATCH`, `GUI_CONFIRM` |
| 11 | [Sicherung und Umbau](code/kap11) | Datenbank sichern, einspielen, eine Spalte nachrüsten | Sicherungskopie, `ALTER TABLE`, `PRAGMA` |
| 12 | [Politur](code/kap12) | Wappen in der Tabelle, Farben, Diagramm des Punkteverlaufs | Zellbilder, `chart`-Modul |
| 13 | [Weitergeben](code/kap13) | Das Programm auf einen fremden Rechner bringen | Standalone-`.exe`, Datenpfade |

Jedes Kapitel ist ein **vollständig lauffähiges Programm**. Man kann jederzeit
einsteigen, den Stand starten und von dort weiterbauen.

## Was dieses Buch anders macht als ein Spielbuch

**Die Daten kommen zuerst.** Kapitel 2 und 3 haben noch kein Fenster: erst die
Datenbank, dann die Regel, dann die Oberfläche. Wer umgekehrt anfängt, baut die
Regel dreimal — einmal in jedem Knopf.

**Eine Regel, eine Stelle.** `punkte()` steht einmal da und wird überall gefragt.
Ein Programm, in dem dieselbe Entscheidung an zwei Orten getroffen wird, hat
irgendwann zwei verschiedene Antworten.

**Die Datenbank ist die Wahrheit, die Tabelle ihre Ansicht.** Jede Tabellenzeile
merkt sich die `id` ihres Spiels. Sortieren und Filtern stellen die Ansicht um,
nicht die Daten.

**Gewertet wird nur, was gespielt ist.** Ein Tipp auf ein Spiel ohne Ergebnis ist
keine Null — er zählt einfach noch nicht.

**Warten heißt nachsehen, nicht stehenbleiben.** Der Abruf aus dem Netz läuft im
Hintergrund; das Fenster bleibt bedienbar.

## Voraussetzungen

- Eine Drachenhauch-Installation (native Runtime `dhrt` gebaut — siehe
  [Haupt-README](../README.md)).
- Der Qt-Editor `dhedit` genügt; für dieses Buch braucht es keinen Sprite-Editor.
- Keine Vorkenntnisse. Wer das Galaga-Buch gelesen hat, kennt Fenster und
  Schleife schon — dann sind die Kapitel 1 und 4 schnell gelesen.

## Prüfprogramme

Neben den Kapiteln liegen Programme, die nichts zeigen, sondern nachrechnen. Sie
gehören zum Buch: einer Anwendung, die man nicht prüfen kann, kann man nicht
trauen.

| Datei | Was es prüft |
|---|---|
| [`code/tippspiel_pruefung.dh`](code/tippspiel_pruefung.dh) | Punkteregel und Rangliste — 13 Prüfungen, ohne Fenster |
| [`code/zeit_pruefung.dh`](code/zeit_pruefung.dh) | Die Datumsrechnungen: Tippschluss, Countdown, Schaltjahre |
| [`code/abruf_pruefung.dh`](code/abruf_pruefung.dh) | Blockierender gegen nebenläufigen Abruf, mit gemessenen Zahlen |
| [`code/stolpersteine.dh`](code/stolpersteine.dh) | Messprotokoll: wo Drachenhauch für so eine Anwendung an Grenzen stößt |

```
dhrun.py buch-tippspiel/code/tippspiel_pruefung.dh
dhrun.py buch-tippspiel/code/zeit_pruefung.dh
dhrun.py buch-tippspiel/code/abruf_pruefung.dh
dhrun.py buch-tippspiel/code/stolpersteine.dh
```

## Gefundene Stolpersteine

Gemessen, nicht vermutet — Zahlen aus `stolpersteine.dh` vom 16.08.2026.

### Geschlossen

1. ~~**HTTP blockiert das Fenster.**~~ **Erledigt.** Ein `HTTP_GET` auf
   OpenLigaDB dauerte **381 ms** (246 KB) — 22 ausgefallene Bilder, in denen
   weder Maus noch Tastatur reagierten. Es gibt jetzt
   `HTTP_GET_START`/`HTTP_READY`/`HTTP_RESULT`: der Abruf läuft im
   Hintergrund, das Programm sieht einmal pro Bild nach.
   Gemessen mit [`code/abruf_pruefung.dh`](code/abruf_pruefung.dh): derselbe
   Abruf, **126 Schleifendurchläufe während der Wartezeit** statt Stillstand;
   zwei Abrufe gleichzeitig sind zusammen so schnell wie einer.
   → [docs/module-html.md](../docs/module-html.md#abrufe-im-hintergrund)

2. ~~**Keine Datumsrechnung.**~~ **Erledigt.** Neues Modul `zeit`: ein
   Zeitpunkt ist eine Zahl (Sekunden seit 1970), `ZEIT_PARSE`/`ZEIT_TEXT$`
   wandeln hin und zurück, `ZEIT_PLUS`/`ZEIT_DIFF` rechnen,
   `ZEIT_FORMAT$`/`ZEIT_DAUER$` zeigen an. Damit kann die Anwendung
   Tippschluss („Anstoß minus 15 Minuten"), Countdown („noch 2:15 h") und
   Wochentag. → [docs/module-zeit.md](../docs/module-zeit.md)

3. ~~**Die Textausgabe im Fenster kann keine Umlaute.**~~ **Erledigt.**
   `TEXT(x, y, "Köln")` zeichnete `K?ln` — zwei Ursachen, beide behoben:
   `LOADFONT` backte nur die 95 ASCII-Glyphen (jetzt zusätzlich der ganze
   Latin-1-Bereich und gängige Typografie), und die eingebaute Schrift kennt
   überhaupt nur ASCII (jetzt springt bei Nicht-ASCII eine Systemschrift ein).
   Reiner ASCII-Text geht unverändert durch die eingebaute Schrift, damit
   bestehende Programme gleich aussehen. Deshalb stehen die Vereine jetzt
   richtig da: **FC Bayern München**, **Bor. Mönchengladbach**, **1. FC Köln**.
   → [docs/builtins-grafik.md](../docs/builtins-grafik.md#umlaute-und-akzente)

4. ~~**`MILLIS()` liefert etwas anderes als dokumentiert.**~~ **Erledigt.**
   Die Doku sagte „ms seit Programmstart", die Runtime gab Millisekunden seit
   1970 zurück (gemessen: `1786881140256`). Jetzt ist es eine **monotone
   Stoppuhr ab Programmstart** — `TIMER()` teilt sich dieselbe Uhr, und die
   Tweens rechnen ebenfalls damit. Das ist nicht nur Kosmetik: die Systemuhr
   kann mitten in einer Messung springen (Zeitumstellung, NTP-Korrektur), eine
   monotone Uhr nicht. Datum und Uhrzeit liefert `ZEIT_JETZT()`.

5. ~~**Leeres Array-Literal hat keinen Typ.**~~ **Erledigt.** `[]` ist erlaubt;
   den Elementtyp gibt der Ort vor, an dem es landet — Variable, Parameter,
   Rückgabewert. `DIM a AS ARRAY OF STRING : a = []` bleibt also ein
   `ARRAY OF STRING`, und `ARRAY_PUSH(a, 42)` scheitert weiterhin. Ohne
   typisiertes Ziel bleibt es `ARRAY OF ANY`. Builtins, die `ARRAY OF STRING`
   erwarten, nehmen ein leeres Array an — es kann keinen falschen Wert
   enthalten. Der Platzhalter-Eintrag im Tippspiel ist weg:
   `GUI_DROPDOWN(win, 90, 46, 200, 26, [])`.

6. ~~**Kein Ja/Nein-Dialog.**~~ **Das war ein Irrtum von mir:** `GUI_CONFIRM`
   gab es die ganze Zeit, dokumentiert und registriert — ich hatte nur
   `GUI_MESSAGE` gesehen. Was fehlte, war die Beschriftung: der Dialog zeigte
   immer OK/Abbrechen. `GUI_CONFIRM(titel$, text$, "janein")` zeigt jetzt
   **Ja/Nein**. Im Tippspiel fragt „Ergebnis eintragen" nach, wenn schon ein
   Ergebnis dasteht — dabei würden alle Punkte des Spiels neu gerechnet.

7. ~~**Arrays prüfen ihren Elementtyp bei der Zuweisung nicht.**~~
   **Erledigt.** `DIM a AS ARRAY OF STRING : a = b` ging durch, auch wenn `b`
   ein `ARRAY OF INTEGER` war — bei einfachen Werten meldete die Runtime das
   („Erwartet STRING, erhalten INTEGER"), Arrays reichte sie ungeprüft
   weiter. Der Fehler fiel dadurch erst weit entfernt auf, beim Lesen eines
   Elements mit dem falschen Typ. Geprüft wird jetzt bei **Variablen,
   Parametern und Rückgabewerten**. Erlaubt bleibt, was nicht schiefgehen
   kann: das leere Literal `[]` erbt weiterhin den Zieltyp, ein
   `ARRAY OF ANY` darf man bewusst einengen, und ein frisches `[1, 2, 3]` an
   einem `ARRAY OF FLOAT` wird umgebaut statt abgelehnt — ein *vorhandenes*
   `ARRAY OF INTEGER` dagegen nicht, sein bisheriger Name zeigt ja weiter auf
   dieselben Zellen. → [docs/sprache.md](../docs/sprache.md#arrays)

### Offen

Nichts. Alle sieben gefundenen Punkte sind geschlossen — was beim
Weiterschreiben auffällt, kommt hier dazu.

## Was gut passt

Reiter (`GUI_TABS`), Tabellen mit Sortieren/Filtern/Zellfarben/Zellbildern,
`GUI_SPINNER` für Torzahlen, `db` mit `?`-Bindung und Transaktionen,
Diagramme über das `chart`-Modul, Wappen über `GUI_TABLE_CELL_IMAGE`.
Für eine Anwendung dieser Art ist das Fundament da.
