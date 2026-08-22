# Der Einstieg — Programmieren lernen von Null an

Ein Lehrbuch für Menschen, die **noch nie programmiert haben**. Kein „ein
bisschen HTML", keine Vorkenntnisse, keine Mathematik über das kleine
Einmaleins hinaus.

## Wofür es das gibt

Im Regal steht bereits [Das Lehrbuch](../buch-referenz/README.md) — vollständig,
systematisch, jeder Befehl mit Beispiel. Es lehrt die Sprache in der richtigen
Reihenfolge: erst die Konsole, dann die Grammatik, Grafik ab Teil IV.

Dieses Buch geht den umgekehrten Weg. **Das allererste Programm ist fünf Zeilen
lang und öffnet ein Fenster mit einer leuchtenden Sonne.** Verstanden wird
hinterher. Nach jedem Kapitel steht etwas auf dem Bildschirm, das man jemandem
zeigen möchte — ein Sternenhimmel, ein hüpfender Ball, Pong, ein Instrument zum
Draufspielen, ein Feuerwerk aus fünfhundert Funken.

Beide Bände beschreiben dieselbe Sprache und widersprechen sich nicht. Wer hier
durch ist, schlägt dort nach.

Das Gegenstück zum [Galaga-Buch](../buch-galaga/README.md) und zum
[Tippspiel-Buch](../buch-tippspiel/README.md): dort baut jeweils ein Buch ein
Programm. Hier bauen viele kleine Programme einen Menschen, der programmieren
kann.

## Das Abschlussprojekt

Ein **Vokabeltrainer** mit richtigem Fenster, Knöpfen und Eingabefeldern: legt
Vokabeln an, fragt ab, merkt sich, was schwerfällt, und zeigt den Fortschritt.
Etwas, das man weitergeben kann — und das genau die Bausteine braucht, die die
Kapitel davor eingeführt haben.

## Aufbau

| Teil | Worum es geht | Was am Ende steht |
|---|---|---|
| I | Bilder aus Zahlen — Fenster, Variablen, Schleifen, Zufall, Verzweigungen, Tastatur | Sternenhimmel, hüpfender Ball, **Pong** |
| II | Klang | ein kleines **Instrument** |
| III | Ordnung — Funktionen, Arrays, Maps, Klassen, Text, Dateien | 500 Partikel, ein Highscore, der bleibt |
| IV | Sprites — selbst malen, Animation, Kollision | ein eigenes **Arcade-Spiel** |
| V | Fenster mit Knöpfen — GUI, Eingabefelder, Layout, Speichern | eine echte Oberfläche |
| VI | Abschlussprojekt | der **Vokabeltrainer** |

## Das Buch bauen

```
cd buch-einstieg/buch
npm install                       # einmalig
node build_book.js                # -> Drachenhauch-Einstieg.docx
node build_epub.js                # -> Drachenhauch-Einstieg.epub
<venv>\python.exe make_book.py    # Zwei-Pass-Bau mit echten Seitenzahlen
```

Die Kapitel liegen in `buch/content/NN_*.js`, jede Datei exportiert
`(H) => [bloecke]` und weiß nicht, ob sie als `.docx` oder `.epub` gesetzt wird.
Ein neues Kapitel ist eine neue Datei — die Reihenfolge ergibt sich aus dem
Namen.

## Was geprüft wird

```
node pruef_codebloecke.js
```

Schickt **jeden abgedruckten Codeblock** durch `dhrt --check` und meldet
zusätzlich jede Zeile über **72 Zeichen**. Bei einem Anfängerbuch ist beides
keine Kür: Ein Tippfehler im Abdruck fällt sonst erst dem auf, der ihn
abtippt — und der weiß noch nicht, dass nicht er den Fehler gemacht hat.

Die 72 sind gemessen: Eine Zeile mit 81 Zeichen lief im gesetzten PDF aus dem
grauen Kasten heraus, und das folgende `NEXT` rutschte dadurch an den Rand.

`--check` allein reicht allerdings nicht — es prüft den Text, nicht die Werte.
`RGB(x * 255 / 640, ...)` geht anstandslos durch und bricht erst beim Laufen ab
(„RGB erwartet INTEGER, erhalten FLOAT"). Deshalb wird jedes Grafikprogramm
zusätzlich wirklich ausgeführt, nämlich beim Aufnehmen der Bilder.

## Die Bilder

Die Bildquellen liegen in `buch/figures/*.dh`, aufgenommen mit

```
<venv>\python.exe shoot.py
```

Das gilt aber nur für die **linearen** Programme der ersten Kapitel: Der
Screenshot fällt beim **letzten** Frame, und wer nur einmal zeichnet und dann
mit `SLEEP` wartet, bekommt ein schwarzes Bild. Deren Figurenquelle enthält
dieselben Zeichenbefehle in einer kurzen Schleife.

Ab Kapitel 5 haben die Programme eine eigene Spielschleife und laufen, bis man
sie beendet — die nimmt `shoot.py` **unverändert** aus `code/kapNN/` auf: Aus
`kap05_1_wanderer` wird `../code/kap05/1_wanderer.dh`. Keine zweite Quelle,
keine zweite Wahrheit.

Aufgenommen wird mit `DHRT_SCALE=3`, also 1920×1200 aus einem 640×400-Fenster.
Mehr geht nicht: Mit Skalierung 4 wäre das Fenster 2560×1600 und damit höher
als der Bildschirm; Windows schiebt es dann weg, und die Aufnahme bekommt oben
179 schwarze Zeilen. 1920 Breite ist zugleich das Maß des Referenzbuchs und
reicht für den 300-dpi-Druck.
