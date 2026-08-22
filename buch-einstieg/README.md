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
zeigen möchte — ein Sternenhimmel, ein hüpfender Ball, Pong, Snake, ein
Instrument zum Draufspielen, ein Feuerwerk aus fünfhundert Funken.

Beide Bände beschreiben dieselbe Sprache und widersprechen sich nicht. Wer hier
durch ist, schlägt dort nach.

Das Gegenstück zum [Galaga-Buch](../buch-galaga/README.md) und zum
[Tippspiel-Buch](../buch-tippspiel/README.md): dort baut jeweils ein Buch ein
Programm. Hier bauen viele kleine Programme einen Menschen, der programmieren
kann.

## Das Abschlussprojekt

Ein **Vokabeltrainer** mit richtigem Fenster, Knöpfen und Eingabefeldern. Er
holt seine Listen **aus dem Internet** (die drei unter `vokabellisten/` liegen
im selben Repo), kann **mehrere Sprachen**, nimmt **eigene Listen** entgegen und
fragt **nicht stumpf zufällig** ab: Ein Karteikasten nach Leitner entscheidet,
was drankommt, und das Fach entscheidet, **wie** gefragt wird — vorstellen,
Multiple Choice in beiden Richtungen, tippen in beiden Richtungen. Getippte
Antworten dürfen einen Tippfehler haben; gemessen wird mit dem
Levenshtein-Abstand.

`dhrt --export` macht daraus eine einzelne `.exe`, die man weitergeben kann.

## Aufbau

| Teil | Worum es geht | Was am Ende steht |
|---|---|---|
| I | Bilder aus Zahlen — Fenster, Variablen, Schleifen, Zufall, Verzweigungen, Tastatur, Arrays | Sternenhimmel, hüpfender Ball, **Pong**, **Snake** |
| II | Klang | ein kleines **Instrument** |
| III | Ordnung — Funktionen, Maps, Klassen, Text, Dateien | 500 Partikel, ein Highscore, der bleibt |
| IV | Sprites — selbst malen, Animation, Kollision | ein eigenes **Arcade-Spiel** |
| V | Fenster mit Knöpfen — GUI, Eingabefelder, Layout, Speichern | eine echte Oberfläche |
| VI | Der Vokabeltrainer — Datenmodell, HTTP, Karteikasten, vier Fragearten, eigene Listen | der **Vokabeltrainer** samt `--export` |
| Anhang | Farben, alle Befehle des Buchs, wie es weitergeht | Nachschlagewerk |

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

```
node pruef_abdruck.js
```

Hält jedes abgedruckte **Gesamtprogramm** gegen die Datei unter `code/kapNN/`.
Der Blockprüfer garantiert nur, dass der Abdruck *läuft* — nicht, dass er
*dasselbe* ist wie die mitgelieferte Datei. Beim Schreiben von Kapitel 9 wichen
beide an 15 Stellen voneinander ab; beides lief, beides war richtig, aber wer
abtippt und danach in die Datei sieht, zweifelt dann an sich statt am Buch.

`--check` allein reicht ebenfalls nicht — es prüft den Text, nicht die Werte.
`RGB(x * 255 / 640, ...)` geht anstandslos durch und bricht erst beim Laufen ab
(„RGB erwartet INTEGER, erhalten FLOAT"). Deshalb wird jedes Grafikprogramm
zusätzlich wirklich ausgeführt, nämlich beim Aufnehmen der Bilder.

## Anhang B schreibt sich selbst

`buch/content/35_anhang_b_befehle.js` pflegt keine Befehlsliste, sondern liest
beim Bauen die Programme unter `code/kapNN/` und zieht Namen und frühestes
Kapitel daraus. Aus der Hand kommt nur der erklärende Halbsatz je Befehl.

Der Grund ist derselbe wie bei allen anderen Prüfungen hier: Eine von Hand
gepflegte Liste wäre beim ersten neuen Kapitel veraltet gewesen, und niemand
hätte es gemerkt. Steht ein Befehl im Code, aber nicht in der Erklärungsliste,
landet er sichtbar unter „Noch ohne Beschreibung“ — im gedruckten Buch, wo man
ihn nicht übersehen kann.

Stand jetzt: **113 Befehle aus 85 Programmen**, keiner ohne Beschreibung.

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
keine zweite Wahrheit. Der Ordner `code/anhang/` wird genauso behandelt: aus
`anhang_farben` wird `../code/anhang/farben.dh`.

Aufgenommen wird mit `DHRT_SCALE=3`, also 1920×1200 aus einem 640×400-Fenster.
Mehr geht nicht: Mit Skalierung 4 wäre das Fenster 2560×1600 und damit höher
als der Bildschirm; Windows schiebt es dann weg, und die Aufnahme bekommt oben
179 schwarze Zeilen. 1920 Breite ist zugleich das Maß des Referenzbuchs und
reicht für den 300-dpi-Druck.

## Die Vokabellisten

`vokabellisten/` enthält den Grundwortschatz für Englisch, Französisch und
Spanisch sowie einen `katalog.txt`, der sagt, welche Listen es gibt. Format:

```
# name: Englisch Grundwortschatz
# sprache: Englisch
Haus;house
```

Der Trainer lädt sie über ihre `raw.githubusercontent.com`-Adresse. Das ist
Absicht: Eine fremde Übersetzungs-Schnittstelle wäre morgen vielleicht
kostenpflichtig oder abgeschaltet, und dann stünde in einem gedruckten Buch
eine Adresse, die ins Leere führt. Die Listen liegen dort, wo das Buch liegt.

Geht das Netz nicht, greifen alle Programme auf die Datei neben sich zurück —
und **sagen es auch** im Fenster.
