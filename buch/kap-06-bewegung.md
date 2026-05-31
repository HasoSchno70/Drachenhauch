# Kapitel 6 — Der Spieler bewegt sich

Am Ende von Kapitel 5 stand ein gelber Block am unteren Rand des Bildschirms. Er sah ganz hübsch aus, aber: er hat nichts getan. Ein Spiel, in dem nichts passiert, ist kein Spiel.

In diesem Kapitel ändert sich das. Wir lassen den Player auf Pfeiltasten reagieren — links, rechts, hin, her. Am Ende des Kapitels hast du eine kleine Box, die dir auf Eingabe folgt. Das ist der Moment, in dem aus „Programm mit Bild" ein „interaktives Programm" wird. Das ist der Moment, in dem aus „Programm" ein **Spiel** wird.

## Lernziele

Nach diesem Kapitel:

- weißt du, was ein **Frame** ist und warum der Game-Loop pro Frame dieselben Schritte macht
- liest du Tastatureingabe mit `KEYPRESSED` ohne das Programm zu blockieren
- kennst du das Prinzip **„Update vor Draw"**
- begrenzt du die Player-Position auf den Bildschirm-Bereich
- nutzt du Konstanten für Spiel-Tuning (Geschwindigkeit, Player-Größe)

## Was ein Frame ist

Stell dir einen Daumenkino-Comic vor: viele leicht unterschiedliche Bilder, schnell hintereinander geblättert ergeben Bewegung. Computerspiele machen genau das — nur dass die Bilder pro Frame **neu berechnet** werden statt schon gezeichnet im Block zu liegen.

Ein **Frame** ist ein Bildschirm-Update. Pro Sekunde laufen typischerweise 60 davon ab. In jedem Frame passiert dasselbe:

1. **Eingabe lesen** — was hat der Spieler gerade getan?
2. **Welt aktualisieren** — Player bewegen, Gegner schießen, Score erhöhen.
3. **Bild zeichnen** — alles auf den Bildschirm bringen.
4. **Warten** — bis die nächste sechzigstel Sekunde vorbei ist.

Punkt 1 und 2 zusammen heißen oft **Update**, Punkt 3 heißt **Draw**. Das Mantra: **Update vor Draw**. Wir denken erst nach, malen erst dann. Das macht den Code viel klarer als wenn beides gemischt ist.

## KEYPRESSED — die nicht-blockierende Eingabe

In Kap 1 hast du `INPUT` schon gesehen (kurz erwähnt) — der wartet, bis der Spieler Enter drückt. Das ist für Konsolen-Programme okay, aber katastrophal für Spiele: das ganze Spiel würde stehenbleiben, bis der Spieler eine Taste drückt. Niemand will das.

Stattdessen gibt's `KEYPRESSED(taste)`. Es liefert `TRUE` oder `FALSE`, abhängig davon, ob die Taste **gerade jetzt** gedrückt ist:

```basic
IF KEYPRESSED(KEY_LEFT) THEN
    PRINT "Pfeil nach links wird gedrueckt"
END IF
```

Ist die Taste gedrückt, läuft der Block. Ist sie nicht gedrückt, läuft er nicht — *aber das Programm läuft trotzdem weiter*. Kein Warten, kein Blockieren. Genau was wir brauchen.

Tasten haben in GameBasic vordefinierte Konstanten:

| Konstante | Taste |
|---|---|
| `KEY_LEFT`, `KEY_RIGHT`, `KEY_UP`, `KEY_DOWN` | Pfeiltasten |
| `KEY_W`, `KEY_A`, `KEY_S`, `KEY_D` | WASD |
| `KEY_SPACE` | Leertaste |
| `KEY_RETURN` (oder `KEY_ENTER`) | Enter |
| `KEY_ESCAPE` | Esc |
| `KEY_0` bis `KEY_9` | Zahlentasten |

Es gibt mehr — alle Buchstaben, F1–F12, Backspace usw. Wenn du eine bestimmte Taste brauchst, schau in der [Sprachreferenz](../docs/sprache.md) nach.

## Schritt 1: Erste Bewegung

Bauen wir auf dem Kap-5-Stand auf. Zwei Änderungen:

1. Die Player-Position wird zu einer **Variablen** — sie soll sich ja ändern können.
2. Wir lesen die Pfeiltasten und ändern die Position entsprechend.

```basic
CONST WIDTH    AS INTEGER = 320
CONST HEIGHT   AS INTEGER = 240
CONST BG_COLOR AS INTEGER = &H141E3C
CONST PLAYER_C AS INTEGER = &HFFDC00
CONST PLAYER_W AS INTEGER = 40
CONST PLAYER_H AS INTEGER = 24

DIM player_x AS INTEGER
DIM player_y AS INTEGER
player_x = 140
player_y = 200

SCREEN(WIDTH, HEIGHT, "Star Pilot", 2)

WHILE NOT QUITREQUESTED()
    ' --- Update ---
    IF KEYPRESSED(KEY_LEFT) THEN
        player_x -= 2
    END IF
    IF KEYPRESSED(KEY_RIGHT) THEN
        player_x += 2
    END IF

    ' --- Draw ---
    CLS(BG_COLOR)
    BOX(player_x, player_y, player_x + PLAYER_W, player_y + PLAYER_H, PLAYER_C)
    FLIP()
    SLEEP(16)
WEND
```

Speicher das als `01_player_keys.gb`, drück Run. **Drücke und halte** die Pfeil-Links- oder Pfeil-Rechts-Taste — der gelbe Block bewegt sich!

> **Tipp**: schau, dass das Spielfenster den **Fokus** hat (du musst ggf. einmal reinklicken). Wenn der Editor noch den Fokus hat, gehen die Tastendrücke an ihn statt ans Spiel.

### Was passiert hier eigentlich?

Pro Frame (60 Mal pro Sekunde):

1. `KEYPRESSED(KEY_LEFT)` liefert TRUE oder FALSE.
2. Wenn TRUE: `player_x -= 2`. Aus `140` wird `138`. Im nächsten Frame `136`. Im nächsten `134`...
3. `BOX(player_x, ...)` zeichnet den Player an seiner aktualisierten Position.

Bei 60 FPS und 2 Pixel pro Frame bewegt sich der Player **120 Pixel pro Sekunde**. Bei einem 320-Pixel-Bildschirm ist er also in unter drei Sekunden quer durch.

### Das Problem mit der Bewegung

Probier was: drücke und halte **lange** Pfeil-Links. Der Player bewegt sich aus dem Bild raus. Du siehst nichts mehr — er ist weg. Drück Pfeil-Rechts lang genug, er kommt wieder. Aber das ist Müll.

Das Problem: `player_x` darf weiter unter `0` und über `WIDTH` gehen. Das müssen wir verhindern.

## Schritt 2: Bildschirmrand begrenzen

Nach dem Update der Position (aber **vor** dem Draw) prüfen wir, ob die Position außerhalb des sichtbaren Bereichs liegt. Wenn ja, korrigieren wir sie zurück:

```basic
WHILE NOT QUITREQUESTED()
    IF KEYPRESSED(KEY_LEFT) THEN
        player_x -= 2
    END IF
    IF KEYPRESSED(KEY_RIGHT) THEN
        player_x += 2
    END IF

    ' --- Begrenzen: 0 <= player_x <= WIDTH - PLAYER_W ---
    IF player_x < 0 THEN
        player_x = 0
    END IF
    IF player_x > WIDTH - PLAYER_W THEN
        player_x = WIDTH - PLAYER_W
    END IF

    CLS(BG_COLOR)
    BOX(player_x, player_y, player_x + PLAYER_W, player_y + PLAYER_H, PLAYER_C)
    FLIP()
    SLEEP(16)
WEND
```

Speicher als `02_player_clamped.gb`, Run. Jetzt bleibt der Player im Bild — am linken Rand klebt er bei `x = 0`, am rechten bei `x = WIDTH - PLAYER_W`.

> **Aha-Moment**: Warum `WIDTH - PLAYER_W` als rechte Grenze, nicht `WIDTH`? Weil `player_x` die **linke Kante** des Players ist. Damit der Player komplett im Bild bleibt, muss seine rechte Kante (`player_x + PLAYER_W`) bei `WIDTH` liegen. Auflösen: `player_x = WIDTH - PLAYER_W`. Hätten wir nur `player_x <= WIDTH` geprüft, könnte der Player zur Hälfte aus dem Bild ragen.

> **Begriff**: das nennt man **Clamp** — einen Wert „klemmen". Ein klassisches Pattern. Später, wenn wir Funktionen lernen (Kap 7), schreiben wir uns dazu eine eigene `clamp()`-Funktion und sparen Tipparbeit.

## Schritt 3: Geschwindigkeit als Konstante

In den Schritten 1 und 2 haben wir die `2` zweimal getippt — einmal beim Links-Bewegen, einmal beim Rechts-Bewegen. Wenn du die Geschwindigkeit ändern willst, musst du **beide** Stellen anfassen. Bei wenig Code geht das, aber es summiert sich. Profi-Tipp: **immer wenn dieselbe magische Zahl mehrfach im Code steht, mach eine Konstante draus**.

```basic
CONST PLAYER_SPEED AS INTEGER = 3

' ...

IF KEYPRESSED(KEY_LEFT) THEN
    player_x -= PLAYER_SPEED
END IF
IF KEYPRESSED(KEY_RIGHT) THEN
    player_x += PLAYER_SPEED
END IF
```

Probier verschiedene Werte: `2` ist langsam, `3` ist normal, `5` schon sehr flott, `8` fühlt sich „rutschig" an. Welcher Wert dir am besten gefällt, ist Geschmackssache — *Game-Feel* nennt man das. Profis verbringen Tage damit, an solchen Werten zu drehen.

> **Tipp**: ändere `PLAYER_SPEED` während dein Spiel läuft *nicht* — das wäre der erste Schritt Richtung Power-Ups. Aber das ist Stoff für Kap 12.

## Der vollständige Endstand

Hier nochmal alles zusammen, mit allen Konstanten oben gruppiert. Das ist der Stand, mit dem wir in Kapitel 7 weiterarbeiten:

```basic
CONST WIDTH        AS INTEGER = 320
CONST HEIGHT       AS INTEGER = 240
CONST BG_COLOR     AS INTEGER = &H141E3C
CONST PLAYER_C     AS INTEGER = &HFFDC00
CONST PLAYER_W     AS INTEGER = 40
CONST PLAYER_H     AS INTEGER = 24
CONST PLAYER_SPEED AS INTEGER = 3

DIM player_x AS INTEGER
DIM player_y AS INTEGER
player_x = 140
player_y = 200

SCREEN(WIDTH, HEIGHT, "Star Pilot", 2)

WHILE NOT QUITREQUESTED()
    IF KEYPRESSED(KEY_LEFT) THEN
        player_x -= PLAYER_SPEED
    END IF
    IF KEYPRESSED(KEY_RIGHT) THEN
        player_x += PLAYER_SPEED
    END IF

    IF player_x < 0 THEN
        player_x = 0
    END IF
    IF player_x > WIDTH - PLAYER_W THEN
        player_x = WIDTH - PLAYER_W
    END IF

    CLS(BG_COLOR)
    BOX(player_x, player_y, player_x + PLAYER_W, player_y + PLAYER_H, PLAYER_C)
    FLIP()
    SLEEP(16)
WEND
```

Schau dir die Struktur an: oben **Konstanten**, dann **Variablen**, dann `SCREEN`, dann der **Game-Loop** mit zwei klar getrennten Phasen — Update und Draw.

Das wird das Grundgerüst für die nächsten Kapitel. Wir packen mehr Funktionalität hinein, aber das **Skelett** bleibt: Konstanten, Variablen, `SCREEN`, `WHILE NOT QUITREQUESTED() ... WEND`. Wenn du das verinnerlichst, hast du den ersten Schritt zum Spiele-Programmieren geschafft.

## Frame-Timing: warum genau 16 ms?

Eine Sekunde hat 1000 Millisekunden. 60 Frames pro Sekunde heißt: pro Frame stehen `1000 / 60 ≈ 16.67` Millisekunden zur Verfügung. Mit `SLEEP(16)` warten wir nach jedem Frame ungefähr eine Sechzigstelsekunde — fertig sind 60 FPS.

Das ist nur eine Annäherung — `SLEEP(16)` blockiert genau 16 ms, der Rest des Frames (Eingabe lesen, zeichnen) braucht aber auch Zeit. In der Praxis kommen wir damit auf vielleicht 55–58 FPS. Für unser Spiel ist das fein. Profis nutzen Frame-Pacing-Mechanismen, die das exakter machen — die brauchen wir hier nicht.

> **Was wenn `SLEEP` weg ist?** Dann läuft die Schleife so schnell wie deine CPU sie schafft — auf modernen Rechnern Tausende Frames pro Sekunde. Effekt: der Player schießt mit Lichtgeschwindigkeit über den Bildschirm, der Lüfter geht an, dein Akku ist in einer Stunde leer. Niemals ohne `SLEEP`.

## Übungen

**1. Geschwindigkeit fühlen.** Setze `PLAYER_SPEED` nacheinander auf 1, 2, 3, 5, 8, 12. Probier jede aus — bei welchem fühlt sich die Bewegung am besten an? Notiere dir den Wert und eine kurze Begründung.

**2. Vier Richtungen.** Erweitere die Steuerung um `KEY_UP` und `KEY_DOWN`. Begrenze die vertikale Bewegung so, dass der Player nur in der **unteren Hälfte** des Bildschirms bleibt: `player_y >= HEIGHT / 2` und `player_y <= HEIGHT - PLAYER_H`.

**3. WASD zusätzlich.** Mach das Spiel mit beiden Steuerungen bedienbar — Pfeiltasten *und* WASD sollen funktionieren. Tipp: zwei `IF`s pro Richtung, oder `OR` in einer Bedingung.

**4. Stretch — Präzisions-Modus.** Wenn `KEY_SHIFT` (oder `KEY_LSHIFT`, in der Sprachreferenz nachschauen) gedrückt ist, bewegt sich der Player **halb so schnell**. Praktisch zum genauen Zielen. Implementiere das mit einer lokalen `speed`-Variable, die je nach Shift-Status entweder `PLAYER_SPEED` oder `PLAYER_SPEED / 2` ist.

## Zusammenfassung

Du hast in diesem Kapitel:

- den ersten echten **Game-Loop** mit Update- und Draw-Phase aufgebaut,
- Tastatureingabe mit `KEYPRESSED` ohne Blockieren gelesen,
- die Player-Position auf den Bildschirm-Bereich begrenzt (Clamp),
- die Geschwindigkeit als Konstante extrahiert,
- ein Gefühl für Frame-Timing und 60 FPS bekommen.

Im **nächsten Kapitel** wird das Programm langsam unübersichtlich. Wir teilen den Code in **Funktionen** auf — `Setup()`, `UpdatePlayer()`, `DrawPlayer()` — und lernen dabei `SUB`, `FUNCTION` und Default-Parameter kennen.

## Code-Stand am Ende des Kapitels

- [`code/kap-06/01_player_keys.gb`](code/kap-06/01_player_keys.gb) — erste Bewegung, ohne Begrenzung
- [`code/kap-06/02_player_clamped.gb`](code/kap-06/02_player_clamped.gb) — mit Bildschirm-Clamp
- [`code/kap-06/03_player_speed.gb`](code/kap-06/03_player_speed.gb) — Geschwindigkeit als Konstante (Stand fürs nächste Kapitel)
