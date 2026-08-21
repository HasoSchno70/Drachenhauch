# Drachenhauch 2026.8

*Die Notizen zu dieser Fassung. Was dahintersteckt und warum es so und nicht
anders gebaut wurde, steht in der [Allzweck-Roadmap](allzweck-roadmap.md) und
den vier Entwürfen daneben.*

## Neu in dieser Fassung

Drachenhauch fing als BASIC zum Schreiben von Spielen an. Diese Fassung macht
daraus eine Sprache, mit der man **alles** schreiben kann — Werkzeuge,
Auswertungen, kleine Dienste. Zehn Arbeitspakete, entstanden aus einer
einzigen Frage: was fehlt eigentlich noch?

**Ein Programm ist jetzt ein Programm, kein Fenster.** Es liest
Kommandozeilen-Argumente, kennt Umgebungsvariablen, legt Dateien und Ordner
an, startet fremde Programme (`SHELL`, `SHELL_OUT$`) und beendet sich mit
einem Exit-Code, den ein Skript auswerten kann. Nichts davon braucht `SCREEN`.
Ein `.dh`-Skript kann ein Cron-Job sein.

**Namensräume — der größte Brocken.** Bisher teilten sich alle per `IMPORT`
eingebundenen Dateien einen flachen Namensraum mit über 1400 eingebauten
Befehlen; zwei Bibliotheken mit je einer Funktion `Init` ließen sich nicht
zusammen benutzen. Jetzt:

```basic
IMPORT "mathe.dh" AS mathe

DIM p AS mathe.Punkt
p = NEW mathe.Punkt()
PRINT mathe.Quadrat(5)
PRINT mathe.Farbe.ROT
```

Klassen, Structs und ENUMs inbegriffen. `PRIVATE` versteckt, was niemanden
angeht. Und eine mit `AS` eingebundene Datei sieht die Globals des
Hauptprogramms **nicht** mehr — sie hängt damit nicht mehr davon ab, was
zufällig oben im Programm steht.

**Eigene Funktionen im Hintergrund.** `TASK_START(BerechneKarte, 4242)` lässt
deine Funktion nebenher rechnen, während die Hauptschleife weiterläuft —
gemessen dreht sie sich dabei fast zwei Millionen Mal. Dazu Datenbankabfragen
und fremde Programme, die das Bild nicht mehr anhalten.

**Daten, die von woanders kommen.** CSV nach RFC 4180 (der `SPLIT$`-Behelf
zerlegte jedes Feld falsch, das ein Trennzeichen enthielt), ZIP zum Lesen und
Schreiben, `BUFFER` für Binärdateien, dazu `SHA256$`, `HMAC_SHA256$` und
`UUID4$` für angemeldete Web-Dienste.

**Fehler, mit denen man arbeiten kann.** `TRY` hat jetzt `FINALLY`, ein
geworfener Fehler trägt einen Code (`THROW "netz", "..."`), und `ERROR_LINE()`
sagt, wo er herkam. `ASSERT` und `ASSERT_EQ` mit Sammel-Modus machen aus einem
Programm eine Prüfung — das mitgelieferte Tippspiel prüft sich damit selbst,
136 Zeilen kürzer als vorher.

**Vererbung rund.** `SUPER.Methode(...)` und `ABSTRACT`-Methoden, die eine
Klasse ankündigt und die Erben ausfüllen müssen.

**Mengen und schnellere Maps.** `SET_ADD`/`SET_HAS` statt `MAPPUT(m, STR$(x), 1)`.
Und MAP selbst hat einen Hash-Index bekommen: eine Map mit 20 000 Einträgen
brauchte 224 ms zum Füllen, jetzt sind es 8 ms — **28-mal schneller**, und das
Wachstum ist linear statt quadratisch.

**Das mitgelieferte Tippspiel ist ein richtiges Programm geworden.** Es diente
als Prüfstein für alles oben: was beim Bauen fehlte, wurde zur Sprache
hinzugefügt. Aus dem ersten Spieltag sind alle 34 geworden, die 2. Bundesliga
ist dazugekommen, und **Ergebnisse holt es sich aus dem Netz**, statt sie
eingetippt zu bekommen. Neu sind **Saisons** — der nächste August löscht nicht
mehr den letzten —, **Tippgemeinschaften**, damit das Büro gegen das Büro
tippen kann, eine **Ligatabelle**, die gerechnet und nicht gespeichert wird,
und eine Statistik, die zeigt, *welcher Art* die Treffer waren statt nur ihrer
Punktzahl. Die Punkteregel gehört jetzt der Saison und nicht mehr dem
Quelltext.

**Und man kann es weitergeben.** `dhrt --export` macht daraus eine
eigenständige `.exe` von knapp 15 MB, die ihre Datenbank neben sich anlegt —
egal, aus welchem Ordner sie gestartet wird. Ließ sich die Datenbank nicht
anlegen, ging die Meldung bisher auf die Konsole; beim Doppelklick gibt es
keine, also passierte scheinbar gar nichts. Der häufigste Grund dafür: Windows
öffnet ZIP-Ordner nur lesbar, und wer einen Ordner geschickt bekommt, klickt
gern hinein, statt ihn auszupacken. Jetzt erscheint ein Fenster mit einem Satz,
den man versteht, und der Anweisung, die hilft.

Geprüft wird es von sich selbst: die eigene Ligatabelle gegen die amtliche, die
Prüfung über `ASSERT_EQ` und dabei 136 Zeilen kürzer als vorher.

### Unter der Haube

**Der Python-Parser ist weg.** 2450 Zeilen, die niemand mehr ausführte: `dhrt`
bringt sein Frontend seit langem selbst mit. Eine Sprachänderung fasst ab jetzt
einen Parser an statt zwei.

**Getestet wird jetzt auf drei Systemen.** Bisher lief die Testsuite nur unter
Windows; Linux und macOS wurde bloß kompiliert. Beide neuen Läufe fanden im
ersten Anlauf je einen echten Fehler, den man auf einem Windows-Rechner nicht
sehen kann — eine Lücke in der Zip-Entpackprüfung und eine Wettlaufbedingung
im Netz-Modul.

Die Testsuite ist von 3224 auf über 3350 Prüfungen gewachsen. Die Dokumentation
wurde einmal vollständig gegen die Wirklichkeit gehalten; elf falsche Aussagen
sind korrigiert, und vier Sorten davon prüft jetzt ein Werkzeug bei jedem
Commit mit.
