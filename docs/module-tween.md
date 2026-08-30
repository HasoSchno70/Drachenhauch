# Modul `tween`

Werteinterpolation über Zeit — für Animationen, sanfte Bewegungen, Fade-Effekte. 13 verschiedene Easing-Funktionen.

```basic
IMPORT "tween"
```

## Übersicht

| Funktion | Rückgabe | Bedeutung |
|---|---|---|
| `TWEEN_NEW(start, end, dauer_ms[, easing$])` | TWEEN (one-shot) | Wert einmal von `start` nach `end` fahren |
| `TWEEN_NEW_LOOP(start, end, dauer_ms[, easing$])` | TWEEN (forever, springt zurück) | wie TWEEN_NEW, springt am Ende zurueck an den Anfang |
| `TWEEN_NEW_PINGPONG(start, end, dauer_ms[, easing$])` | TWEEN (forever, hin & zurück) | wie TWEEN_NEW, laeuft am Ende rueckwaerts zurueck |
| `TWEEN_VALUE(t)` | FLOAT | aktueller Wert -- das, was man zeichnet |
| `TWEEN_PROGRESS(t)` | FLOAT (0.0 .. 1.0) | wie weit ist er? (0 = Anfang, 1 = Ende) |
| `TWEEN_DONE(t)` | BOOLEAN (immer FALSE bei loop/pingpong) | ist er durch? |
| `TWEEN_RESTART(t)` | — | wieder von vorn |
| `TWEEN_PAUSE(t)`, `TWEEN_RESUME(t)` | — | anhalten und weiterlaufen lassen |
| `TWEEN_REVERSE(t)` | — | Richtung umkehren |
| `TWEEN_EASINGS()` | STRING (komma-getrennte Liste) | welche Verlaufskurven gibt es? |

## Konzept

Ein Tween ist ein Wert, der sich automatisch über eine bestimmte Zeit zwischen zwei Endpunkten bewegt. Du erstellst ihn einmal mit `TWEEN_NEW`, fragst ihn jeden Frame mit `TWEEN_VALUE` ab.

```basic
DIM t AS TWEEN
t = TWEEN_NEW(0.0, 100.0, 2000, "out_quad")    ' 0 -> 100 in 2000ms

' Im Game-Loop:
DIM x AS FLOAT
x = TWEEN_VALUE(t)
BOX(INT(x), 100, INT(x) + 20, 120, RGB(255, 200, 0))
```

Nach 2000ms steht `TWEEN_VALUE(t)` fest auf `100.0`, `TWEEN_DONE(t)` ist TRUE.

## Anim-Modi: One-Shot / Loop / Pingpong

`TWEEN_NEW` ist **one-shot** — läuft einmal von `start` zu `end` und bleibt dann am Endwert hängen.

Für **kontinuierliche Hintergrund-Animationen** gibt es zwei Endlos-Varianten:

| Builtin | Verhalten | Use Case |
|---|---|---|
| `TWEEN_NEW(start, end, ms)` | once, klemmt am Ende | Pop-In, Fade-Out, Übergänge |
| `TWEEN_NEW_LOOP(start, end, ms)` | wiederholt sich (start → end → start → end …) | Conveyor-Streifen, rotierende Spinner, BPM-pulsierende UI |
| `TWEEN_NEW_PINGPONG(start, end, ms)` | hin & zurück (start → end → start → end → start …) | Idle-Bobs, atmende Skala, Schaukel-Effekte |

Loop- und Pingpong-Tweens sind **niemals** "done" — `TWEEN_DONE(t)` liefert für sie immer `FALSE`. Sie laufen forever, bis du sie pausierst.

```basic
' Coin schwebt 4 Pixel auf und ab, in 800 ms pro Halbwelle
DIM bob AS TWEEN
bob = TWEEN_NEW_PINGPONG(-4.0, 4.0, 800, "inout_sine")

WHILE NOT QUITREQUESTED()
    DIM offset AS FLOAT
    offset = TWEEN_VALUE(bob)
    SPRITE_SET_POS(coin, 100, 60 + offset)
    SPRITE_DRAW(coin)
    FLIP()
    SLEEP(16)
WEND
```

```basic
' Lade-Spinner: 0..360 Grad in 1 Sekunde, dann zurück auf 0 und neu
DIM angle AS TWEEN
angle = TWEEN_NEW_LOOP(0.0, 360.0, 1000, "linear")
```

Pop-In **plus** kontinuierliche Idle-Animation kombinieren? Zwei Tweens parallel: ein one-shot für den Spawn, ein pingpong für den Bob danach. Das visuelle Ergebnis im Frame ist `pop_in_value × bob_offset` (Skala) bzw. `base + bob_offset` (Position).

## Easing-Funktionen

Easing bestimmt die "Beschleunigungs-Kurve" zwischen den Endpunkten.

| Name | Wirkung |
|---|---|
| `linear` | konstante Geschwindigkeit (Default) |
| `in_quad`, `out_quad`, `inout_quad` | quadratisch (sanft an/aus) |
| `in_cubic`, `out_cubic`, `inout_cubic` | kubisch (stärker) |
| `in_sine`, `out_sine`, `inout_sine` | sinus (sanft) |
| `in_bounce`, `out_bounce`, `inout_bounce` | Springen wie ein Ball |
| `in_elastic`, `out_elastic`, `inout_elastic` | überschwingen wie eine Feder |
| `in_back`, `out_back`, `inout_back` | leicht überschießen (Pop-Effekt) |

Komplette Liste programmatisch via `TWEEN_EASINGS()`.

`in_*`: langsam starten, schnell enden. `out_*`: schnell starten, sanft enden. `inout_*`: beides.

```basic
' Coin-Banner fliegt von links rein
DIM banner AS TWEEN
banner = TWEEN_NEW(-200.0, 80.0, 600, "out_quad")

' Power-Up "ploppt" rein
DIM popup AS TWEEN
popup = TWEEN_NEW(0.0, 1.0, 400, "out_bounce")

' Knopf "wackelt" beim Klick
DIM wackel AS TWEEN
wackel = TWEEN_NEW(0.0, 1.0, 600, "out_elastic")
```

## Pause / Resume / Restart

```basic
' Pause während Spiel-Pause
TWEEN_PAUSE(t)
' ... Pause-Menü ...
TWEEN_RESUME(t)

' Animation neu starten (z.B. wenn der User das gleiche Power-Up nochmal nimmt)
TWEEN_RESTART(popup)

' Endpunkte tauschen + restart (z.B. Banner wieder nach links wegfliegen)
TWEEN_REVERSE(banner)
```

## Beispiel: Banner-Slide-In

```basic
IMPORT "tween"

SCREEN(320, 240, "Tween-Demo", 2)

DIM banner AS TWEEN
banner = TWEEN_NEW(-200.0, 80.0, 700, "out_quad")

WHILE NOT QUITREQUESTED()
    CLS(RGB(20, 20, 30))

    DIM y AS INTEGER
    y = ROUND(TWEEN_VALUE(banner))
    BOX(20, y, 300, y + 50, RGB(0, 0, 0))
    RECT(20, y, 300, y + 50, RGB(255, 220, 80))
    TEXT(40, y + 18, "GAME OVER", RGB(255, 255, 255))

    FLIP()
    SLEEP(16)
WEND
```

## Beispiel: HUD-Score-Pop

Bei jedem Pickup soll der Score-Text kurz "ploppen":

```basic
IMPORT "tween"

DIM score_pop AS TWEEN
score_pop = TWEEN_NEW(1.0, 1.0, 0, "linear")        ' Initial: nichts zu animieren

' Bei Pickup:
SUB on_pickup()
    score_pop = TWEEN_NEW(1.6, 1.0, 350, "out_quad")    ' 1.6 -> 1.0 in 350ms
END SUB

' Im Draw-Loop:
DIM scale AS FLOAT
scale = TWEEN_VALUE(score_pop)
IF scale > 1.05 THEN
    TEXT(80, 8, "+100", RGB(255, 220, 80))             ' "Pop"-Indikator
END IF
```

## Beispiel: Coin-Spawn-Pulsar

Damit Coins beim Erscheinen "wachsen":

```basic
IMPORT "tween"

DIM coin_pop[10] AS TWEEN
DIM i AS INTEGER
FOR i = 0 TO 9
    coin_pop[i] = TWEEN_NEW(0.0, 1.0, 300 + RND(200), "out_bounce")
NEXT

' Beim Zeichnen: nur sichtbar, wenn der Pulsar "fertig" ist
FOR i = 0 TO 9
    IF TWEEN_VALUE(coin_pop[i]) > 0.5 THEN
        DRAWIMAGE(coin_img, coin_x[i], coin_y[i])
    END IF
NEXT
```

## Komplettes Beispiel

Siehe [examples/26_tween.dh](../examples/26_tween.dh) — zeigt linear, out_bounce, Pause/Resume und Reverse anhand von Konsolen-Output.

Im Spiel ([examples/32_coinquest.dh](../examples/32_coinquest.dh)) werden Tweens für Banner-Slide, Coin-Spawn und Pickup-Pop kombiniert.

## Es gibt kein `TWEEN_UPDATE`

Das ist Absicht und keine Lücke. `timer`, `input` und `gui` verlangen alle
einen `..._UPDATE()`-Aufruf pro Bild — `tween` nicht:

```basic
DIM t AS TWEEN
t = TWEEN_NEW(0, 100, 500)          ' von 0 nach 100 in 500 ms

WHILE NOT QUITREQUESTED()
    x = TWEEN_VALUE(t)              ' fertig -- kein Update noetig
    ...
WEND
```

Ein Tween rechnet seinen Wert bei **jedem Abruf** aus der Uhr aus
(`MILLIS()` seit Programmstart). Er läuft also weiter, ob du ihn abfragst
oder nicht — und läuft in **echter Zeit**, unabhängig von der Bildrate.

Zwei Folgen, die man kennen sollte:

- **Bricht die Bildrate ein, springt der Tween** statt langsamer zu werden.
  Für Oberflächen-Animation ist das genau richtig; wer eine Spielmechanik an
  einen Tween hängt, bekommt bei einem Ruckler einen Sprung.
- **Ein Tween ist nicht reproduzierbar.** Bei einer aufgezeichneten Eingabe
  (`AUTOMATION_PLAY`) läuft er nach der Wanduhr weiter und nicht nach den
  Bildern — zwei Durchläufe sehen also nicht exakt gleich aus. `timer` ist
  bildgetrieben und damit reproduzierbar; das ist der Unterschied zwischen
  den beiden Modulen.

Pausieren kannst du trotzdem: `TWEEN_PAUSE` / `TWEEN_RESUME` frieren die
verstrichene Zeit ein.

## Tipps

- **Wähle das richtige Easing**: `out_quad` für "sanftes Ankommen", `out_bounce` für "auffälliges Pop", `linear` wenn Konstanz wichtig ist.
- **Tween-Dauer in ms**: 100-300ms wirkt schnell, 500-1000ms ruhig, > 2000ms kann sich träge anfühlen.
- **Mehrere Tweens parallel**: einfach mehrere Variablen, jede ihren eigenen Tween. Sind unabhängig voneinander.
- **Kein `dt`-Argument**: Tweens nutzen intern `MILLIS()`, du musst keine Frame-Zeit übergeben — vereinfacht den Loop-Code.
- **Reset bei Re-Trigger**: wenn ein Effekt mehrfach getriggert werden kann (z.B. Score-Pop), erstelle den Tween jedes Mal neu mit `TWEEN_NEW(...)`.
