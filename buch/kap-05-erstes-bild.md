# Kapitel 5 — Das erste Bild

In den ersten vier Kapiteln haben wir Programme an der Konsole geschrieben. Das war wichtig — du kannst jetzt Variablen, Bedingungen, Schleifen. Aber jetzt machen wir den Sprung, den dieses Buch verspricht: wir öffnen ein Fenster, zeichnen etwas hinein, und legen den Grundstein für **Star Pilot**.

Am Ende dieses Kapitels hast du ein eigenes kleines Programm, das ein Fenster aufmacht, auf einem dunkelblauen Hintergrund einen gelben Block zeichnet — den Player — und so lange läuft, bis du das Fenster schließt.

## Lernziele

Nach diesem Kapitel:

- weißt du, wie man mit `SCREEN` ein Fenster öffnet
- kennst du den Aufbau eines Game-Loops (`WHILE NOT QUITREQUESTED() ... WEND`)
- kannst du den Bildschirm mit `CLS` löschen, mit `BOX` Rechtecke zeichnen, mit `FLIP` das Ergebnis sichtbar machen
- verstehst du, wie Farben mit `RGB(rot, grün, blau)` zusammengesetzt werden
- hast du den ersten lauffähigen Stand des Spiels: einen Player am unteren Bildschirmrand

## Vom Konsolenprogramm zum Spiel

Bisher hat dein Programm so ausgesehen:

```basic
DIM name AS STRING
name = "Pilot"
PRINT "Hallo, " + name
```

Du startest das Programm, in der Konsole erscheint `Hallo, Pilot`, das Programm endet. Linear, kurz, klar.

Ein Spiel funktioniert anders. Ein Spiel **läuft weiter** — Frame für Frame, viele Male pro Sekunde, bis der Spieler entscheidet aufzuhören. Jeder Frame zeichnet das aktuelle Bild komplett neu. Klingt nach viel Arbeit? Computer machen das mit links, sechzig Mal pro Sekunde sind kein Problem.

Was wir brauchen, sind drei Dinge:

1. **Ein Fenster**, in das wir zeichnen können.
2. **Eine Schleife**, die so lange läuft, bis der Spieler Schluss macht.
3. **Drei Befehle**: löschen, malen, zeigen.

Genau das bauen wir jetzt — Schritt für Schritt.

## Schritt 1: Das Fenster

Erste Datei. Lege im GameBasic-Editor eine neue Datei an, speichere sie unter `01_screen.gb` (im Buch-Code-Ordner liegt sie unter `code/kap-05/01_screen.gb`).

```basic
SCREEN(320, 240, "Star Pilot", 2)
```

Mehr nicht. Eine Zeile. Drück **Run** (oder F5).

Du solltest ein kleines Fenster sehen — schwarzer Hintergrund, der Titel "Star Pilot" oben. Das Fenster schließt sich nach einer Sekunde von selbst, weil das Programm endet.

> **Warum die Werte?** `320, 240` ist die Größe in Pixeln — winzig nach heutigen Maßstäben, aber genau richtig für ein klassisches Pixel-Spiel. Das vierte Argument `2` ist der **Skalierungsfaktor**: jeder logische Pixel wird auf ein 2×2-Block hochgezogen. So ist das Fenster auf dem Bildschirm 640×480 groß, aber wir programmieren in 320×240. Das ist die geheime Zutat für authentisches Retro-Gefühl: kleine logische Auflösung, große Pixel.

> **Stolperfalle**: wenn du `2` weglässt, funktioniert das Fenster auch — ist nur winzig. Spiel ein bisschen mit dem Wert: `1`, `3`, `4` — was dir am besten gefällt.

## Schritt 2: Die Schleife

Ein Fenster, das nach einer Sekunde verschwindet, ist kein Spiel. Wir brauchen eine **Schleife**, die so lange läuft, bis der Spieler das Fenster schließt.

GameBasic hat dafür eine Built-in-Funktion: `QUITREQUESTED()`. Sie liefert `TRUE`, sobald der Spieler das Fenster zumacht oder ESC drückt. Solange das nicht passiert, soll unsere Schleife laufen.

```basic
SCREEN(320, 240, "Star Pilot", 2)

WHILE NOT QUITREQUESTED()
    ' Hier passiert pro Frame etwas - aktuell: nichts.
    SLEEP(16)
WEND
```

Speichern, Run drücken. Diesmal bleibt das Fenster offen — bis du es zumachst.

`SLEEP(16)` lässt das Programm 16 Millisekunden schlafen pro Schleifendurchlauf. Warum genau 16? Weil `1000 / 60 ≈ 16.7` — wir wollen etwa 60 Frames pro Sekunde, das ist die Standard-Bildwiederholrate für flüssige Spiele. Ohne `SLEEP` würde unsere Schleife so schnell laufen, wie der Computer kann (Tausende Frames pro Sekunde), und die CPU vollständig auslasten. Wir brauchen das nicht.

Was wir gerade gebaut haben, ist der **Game-Loop**. Er ist der Herzschlag jedes Spiels. Pro Frame:

1. Eingabe abfragen (kommt in Kap 6).
2. Spielzustand aktualisieren (kommt in Kap 6).
3. Alles zeichnen (kommt jetzt).
4. Auf den nächsten Frame warten.

Punkte 1 und 2 lassen wir noch leer — wir konzentrieren uns auf Punkt 3.

## Schritt 3: Löschen, malen, zeigen

Hier kommen die drei wichtigsten Drawing-Befehle. Du wirst sie in jedem Frame jedes Spiels wiedersehen.

| Befehl | Was er tut |
|---|---|
| `CLS()` | Bildschirm löschen (überall schwarz) |
| `BOX(x1, y1, x2, y2, farbe)` | gefülltes Rechteck zeichnen |
| `FLIP()` | das, was wir gemalt haben, sichtbar machen |

Dass `FLIP` einen eigenen Schritt braucht, ist kein Zufall. Wir malen alles **unsichtbar** in einen Hintergrund-Puffer und zeigen das fertige Bild dann **in einem Stück**. Würden wir live auf den Bildschirm malen, würde der Spieler Halbfertiges sehen (Geflacker, Tearing). Mit dieser Technik — sie heißt **Double-Buffering** — bleibt das Bild ruhig.

Probieren wir's:

```basic
SCREEN(320, 240, "Star Pilot", 2)

WHILE NOT QUITREQUESTED()
    CLS(RGB(20, 30, 60))                    ' dunkelblauer Hintergrund
    BOX(140, 200, 180, 224, RGB(255, 220, 0))  ' gelber Block, Player
    FLIP()
    SLEEP(16)
WEND
```

Run. Du solltest jetzt sehen: dunkelblaues Spielfeld, gelber Block am unteren Rand. Das ist dein **Star Pilot**. Sieht noch nicht spektakulär aus — aber alles, was wir in den nächsten 13 Kapiteln draufpacken, geht von hier aus.

## Farben verstehen: RGB

Was bedeutet `RGB(20, 30, 60)`? Drei Zahlen für die Anteile von Rot, Grün und Blau. Jeder Wert geht von **0** (gar nicht) bis **255** (voll).

| Aufruf | Farbe |
|---|---|
| `RGB(255, 0, 0)` | knalliges Rot |
| `RGB(0, 255, 0)` | knalliges Grün |
| `RGB(0, 0, 255)` | knalliges Blau |
| `RGB(255, 255, 255)` | Weiß |
| `RGB(0, 0, 0)` | Schwarz |
| `RGB(128, 128, 128)` | mittleres Grau |
| `RGB(255, 220, 0)` | warmes Gelb |

GameBasic hat einige Farben auch als **Konstanten** vordefiniert: `WHITE`, `BLACK`, `RED`, `GREEN`, `BLUE`, `YELLOW`, `CYAN`, `MAGENTA`, `GRAY`, `DARKGRAY`, `LIGHTGRAY`, `ORANGE`, `PINK`, `DARKRED`, `DARKGREEN`, `DARKBLUE`. Du kannst sie direkt benutzen:

```basic
BOX(140, 200, 180, 224, YELLOW)
```

Das ist kürzer als `RGB(255, 255, 0)` und liest sich besser. Für eigene Farbtöne (das spezifische dunkle Blau unseres Hintergrunds) brauchst du `RGB`.

> **Tipp**: lege dir am Anfang des Programms ein paar Farb-Konstanten als `CONST` an — dann hast du sie zentral:
>
> ```basic
> CONST BG       AS INTEGER = &H141E3C   ' = RGB(20, 30, 60)
> CONST PLAYER_C AS INTEGER = &HFFDC00   ' = RGB(255, 220, 0)
> ```
>
> Die Hex-Schreibweise `&H...` ist die Profi-Notation für Farben. Wer mit CSS oder Photoshop gearbeitet hat, kennt sie. Die letzten zwei Hex-Stellen sind das Blau, davor Grün, davor Rot.

## Die Koordinaten verstehen

In `BOX(140, 200, 180, 224, YELLOW)` sind das vier Zahlen: `(x1, y1)` ist die linke obere Ecke, `(x2, y2)` die rechte untere. Beim 320×240-Fenster sieht das so aus:

```
(0,0) ----------------------------- (320,0)
   |                                       |
   |          Spielfeld                    |
   |                                       |
   |            +-------+                  |
   |            | Box   |                  |
   |            +-------+                  |
   |        (140,200)  (180,224)           |
(0,240) -------------------------- (320,240)
```

**X** steigt nach rechts, **Y** steigt nach **unten** (anders als im Mathe-Unterricht!). Das ist eine Konvention, die fast alle Computer-Grafik-Systeme so machen — wir gewöhnen uns dran.

Die Box ist also `40` Pixel breit (`180 − 140`) und `24` Pixel hoch (`224 − 200`). Sie sitzt nicht ganz am Rand, sondern hat `16` Pixel Luft nach unten (`240 − 224`). Das wirkt visuell ruhiger als ein Block, der direkt an der Kante klebt.

## Der vollständige Code

Hier nochmal der ganze Stand, mit allen guten Praktiken:

```basic
' Star Pilot - Kapitel 5: das erste Bild.
'
' Oeffnet ein Fenster, zeichnet einen dunklen Hintergrund und den Player
' als gelben Block am unteren Rand. Laeuft bis das Fenster zugemacht oder
' ESC gedrueckt wird.

CONST WIDTH     AS INTEGER = 320
CONST HEIGHT    AS INTEGER = 240
CONST BG_COLOR  AS INTEGER = &H141E3C
CONST PLAYER_C  AS INTEGER = &HFFDC00

SCREEN(WIDTH, HEIGHT, "Star Pilot", 2)

WHILE NOT QUITREQUESTED()
    CLS(BG_COLOR)
    BOX(140, 200, 180, 224, PLAYER_C)
    FLIP()
    SLEEP(16)
WEND
```

Schau dir den Unterschied zur ersten Variante an: statt magischer Zahlen `320, 240` haben wir `CONST WIDTH, HEIGHT`. Statt `RGB(...)` mitten im Code haben wir benannte Farb-Konstanten. Wenn du das Fenster später vergrößern willst, musst du das nur an einer einzigen Stelle ändern.

Diese Disziplin lohnt sich. Spiel-Code wächst schnell, und in Kap 9 wirst du froh sein, wenn du nicht 27 verstreute `320`er Zahlen aufspüren musst.

## Übungen

Nicht überspringen — die Übungen sind das, was den Stoff in den Kopf bringt.

**1. Dein eigener Hintergrund.** Ändere `BG_COLOR` zu einem Farbton, den du magst. Dunkles Lila, tiefes Grün, Nachtblau. Probier mehrere aus.

**2. Mehrere Boxen.** Zeichne zwei zusätzliche Boxen ins Bild — sagen wir, einen "Score-Hintergrund" oben links und ein paar "Sterne" als kleine weiße Punkte irgendwo im Bild. (Tipp: für einen Stern reicht `BOX(x, y, x+1, y+1, WHITE)` — eine 1×1-Box.)

**3. Player-Position justieren.** Ändere die Player-Box so, dass sie genau in der Mitte des Bildschirms (horizontal) sitzt. Hinweis: nutze `WIDTH / 2` und überlege, wie breit dein Player ist.

**4. Stretch.** Zeichne unter dem Player einen "Triebwerks-Strahl": eine zweite Box, ein paar Pixel hoch, in Orange (`RGB(255, 100, 0)`) oder so. Das ist die statische Version dessen, was wir in Kap 13 mit Particles animieren werden.

## Zusammenfassung

Du hast in einem Kapitel:

- ein Spielfenster geöffnet,
- einen Game-Loop geschrieben, der bis zum Beenden läuft,
- gelernt wie `CLS`, `BOX` und `FLIP` zusammenspielen,
- Farben mit `RGB` und Hex-Konstanten gebaut,
- den Player erstmals als sichtbares Element platziert.

Im nächsten Kapitel macht der Player das, wozu Player da sind: er **bewegt sich**.

## Code-Stand am Ende des Kapitels

[`code/kap-05/04_player.gb`](code/kap-05/04_player.gb) — vollständiger Stand wie oben gezeigt. Dazwischen liegen die Zwischenstände `01_screen.gb`, `02_cls_color.gb`, `03_box.gb` für die Schritt-für-Schritt-Nachvollziehbarkeit.
