# Kapitel 14 — Wellen-Choreographie mit Tween

In Kapitel 13 haben wir Star Pilot **visuell** aufgewertet — Particles für Explosionen und Trail. Aber das **Bewegungs-Gefühl** ist noch immer mechanisch: Bomber fliegen Pixel-für-Pixel mit harter Reflexion am Rand, der Wave-Intro-Schriftzug erscheint *poppt* einfach in der Mitte. In Galaga-Klassikern fühlt sich Bewegung anders an — sie *atmet*, schwingt, federt.

Das ist die Aufgabe von **Tweens**: vorprogrammierte Bewegungs-Kurven, die einen Wert über Zeit von A nach B führen — mit einer **Beschleunigungs-Form** (Easing), die natürlicher wirkt als linear. In diesem Kapitel ersetzen wir zwei Stellen im Code: Bombers Zickzack-Bewegung und den Wave-Intro-Schriftzug.

## Lernziele

Nach diesem Kapitel:

- erstellst du Tweens mit `TWEEN_NEW(start, end, dauer_ms, easing)`
- liest den aktuellen Wert mit `TWEEN_VALUE(t)`
- nutzt **Easing-Kurven** (`linear`, `out_quad`, `out_bounce`, `inout_sine`, …) bewusst
- kennst den Unterschied zwischen **One-shot**, **Loop** und **Pingpong**
- hast in Star Pilot zwei Tween-Anwendungen integriert

## Was ein Tween macht

Ein Tween ist nicht mehr und nicht weniger als ein Wert, der sich automatisch über die Zeit von einem Start- zu einem End-Wert bewegt. Du erstellst ihn einmal, fragst ihn jeden Frame mit `TWEEN_VALUE(...)` ab, kriegst eine `FLOAT` zurück.

```basic
DIM t AS TWEEN
t = TWEEN_NEW(0.0, 100.0, 2000, "linear")    ' 0 -> 100 in 2 Sek

' im Game-Loop:
DIM v AS FLOAT
v = TWEEN_VALUE(t)
PRINT v       ' nach 1 Sek: 50.0; nach 2 Sek: 100.0; danach bleibt 100.0
```

Das Schöne: du musst nicht selbst zählen, nicht selbst Pixel-pro-Frame ausrechnen. Du sagst: „von 0 bis 100 in 2 Sekunden". Das Modul macht den Rest.

## Easing — der entscheidende Unterschied

Eine **lineare** Bewegung von 0 zu 100 fühlt sich roboter-artig an: die Geschwindigkeit ist konstant, der Stop ist abrupt. **Easing-Kurven** ändern das. Sie bestimmen, *wie* der Wert zwischen Start und Ende verteilt wird.

| Easing | Verhalten | Wofür |
|---|---|---|
| `linear` | konstante Geschwindigkeit | seltener Use-Case, oft zu „mechanisch" |
| `out_quad` | startet schnell, bremst sanft | UI-Slide-Ins, Pop-Ups |
| `inout_sine` | sanft an, sanft aus | Idle-Animationen, atmen |
| `out_bounce` | springt am Ende mehrmals | Coins „fallen", Drop-Animationen |
| `out_back` | überschießt das Ziel und schnellt zurück | Pop-In-Effekte mit Charakter |
| `in_elastic` | wackelt am Anfang, schießt los | Federsprünge, Startseile |

Das volle Sortiment liefert `TWEEN_EASINGS()` als komma-getrennten String. Im Praxis-Alltag reichen 4–5 Easings — die anderen sind situativ.

### Sehen statt erklären

Hier ein Programm, das vier Boxen parallel mit verschiedenen Easings ping-pong-bewegt:

```basic
IMPORT "tween"

DIM t1 AS TWEEN : DIM t2 AS TWEEN : DIM t3 AS TWEEN : DIM t4 AS TWEEN
t1 = TWEEN_NEW_PINGPONG(40.0, 280.0, 2000, "linear")
t2 = TWEEN_NEW_PINGPONG(40.0, 280.0, 2000, "out_quad")
t3 = TWEEN_NEW_PINGPONG(40.0, 280.0, 2000, "out_bounce")
t4 = TWEEN_NEW_PINGPONG(40.0, 280.0, 2000, "inout_sine")

SCREEN(320, 240, "Easing Vergleich", 2)

WHILE NOT QUITREQUESTED()
    CLS(&H141E3C)
    BOX(INT(TWEEN_VALUE(t1)) - 8, 30, INT(TWEEN_VALUE(t1)) + 8, 50, &HFFFF00)
    TEXT(8, 35, "linear", &HFFFFFF)
    ' ... t2, t3, t4 analog ...
    FLIP()
    SLEEP(16)
WEND
```

Run drücken, eine Minute zuschauen. Der Unterschied zwischen `linear` und `inout_sine` ist offensichtlich, sobald du's gesehen hast.

## Drei Tween-Modi

`TWEEN_NEW` ist nicht alles. GameBasic hat drei Varianten, je nachdem was du brauchst:

| Builtin | Verhalten | Use Case |
|---|---|---|
| `TWEEN_NEW(...)` | **Once** — einmal von start zu end, klemmt am Ende | Pop-In, Slide-Out, Übergänge |
| `TWEEN_NEW_LOOP(...)` | wiederholt sich (start → end → start → end …) | Conveyor-Streifen, Spinner, Dauer-Pulse |
| `TWEEN_NEW_PINGPONG(...)` | hin und zurück (start → end → start →…) | Idle-Bobs, Schaukel-Bewegungen |

**One-shot** ist der häufigste Fall — eine einmalige Animation, die am Ende stehenbleibt. **Loop** und **Pingpong** liefern nie `TWEEN_DONE = TRUE`, sie laufen forever bis du sie pausierst.

> **Performance**: Tweens sind sehr leichtgewichtig — pro Tween ein paar Werte und eine Funktions-Indirektion. Du kannst hunderte parallel laufen lassen, kein Problem.

## Schritt 1: Bomber-Zickzack mit Pingpong

In Kapitel 10 hatten wir den Bomber so:

```basic
CLASS Bomber EXTENDS Entity
    DIM dx AS INTEGER

    SUB Spawn(at_x AS INTEGER)
        x = at_x
        ...
        dx = 1
    END SUB

    SUB Update()
        IF NOT alive THEN RETURN
        y = y + 1
        x = x + dx
        IF x < 0 OR x > WIDTH - w THEN dx = -dx     ' harte Reflexion
        IF y > HEIGHT THEN alive = FALSE
    END SUB
END CLASS
```

Funktional, aber roboter-artig: linearer Anflug, harter Richtungswechsel am Rand. Mit Pingpong-Tween:

```basic
CLASS Bomber EXTENDS Entity
    DIM spawn_x AS INTEGER
    DIM swing AS TWEEN

    SUB Spawn(at_x AS INTEGER)
        spawn_x = at_x
        x = at_x
        y = -20
        w = 22
        h = 18
        alive = TRUE
        ' Pingpong: -50 .. +50 .. -50 .. mit sanftem Sinus-Easing
        swing = TWEEN_NEW_PINGPONG(-50.0, 50.0, 2000, "inout_sine")
    END SUB

    SUB Update()
        IF NOT alive THEN RETURN
        y = y + 1
        x = spawn_x + INT(TWEEN_VALUE(swing))
        IF y > HEIGHT THEN alive = FALSE
    END SUB
END CLASS
```

Drei Punkte:

1. **`spawn_x`** merkt sich, wo der Bomber gespawnt wurde — er schwingt um diese Mittellinie, nicht um eine fixe Bildschirm-Mitte.
2. **Der Tween liefert -50 bis +50** — wir addieren ihn als **Offset** auf `spawn_x`. Effekt: der Bomber bewegt sich seitlich 50 Pixel nach links, zur Mitte zurück, 50 nach rechts, zurück …
3. **`inout_sine`** macht die Bewegung organisch — am Wendepunkt langsamer als in der Mitte, wie ein Pendel.

Das Resultat sieht völlig anders aus. Der Bomber fühlt sich wie eine echte fliegende Drohne an, die kurvt, statt gegen Wände zu prallen.

## Schritt 2: Wave-Intro-Schriftzug

In Kapitel 12 hatte der Wave-Intro-Schriftzug einfach existiert für 90 Frames:

```basic
TEXT(WIDTH / 2 - 36, HEIGHT / 2 - 4, f"WELLE {wave.number}", &HFFDC00)
```

Mit einem Tween können wir ihn **einfliegen** lassen — von oben her, mit `out_back`-Easing für einen kleinen Pop-Effekt:

```basic
DIM wave_intro_slide AS TWEEN

' Beim Wechsel zu WAVE_INTRO (in Setup und nach NextWave):
wave_intro_slide = TWEEN_NEW(-20.0, HEIGHT / 2 - 4.0, 600, "out_back")

' In DrawAll, statt der festen Mitte:
TEXT(WIDTH / 2 - 36, INT(TWEEN_VALUE(wave_intro_slide)), _
     f"WELLE {wave.number}", &HFFDC00)
```

Die Animation läuft 600 ms — der Schriftzug startet bei `y = -20` (über dem Bildschirm), gleitet zur Mitte und schießt dabei leicht über die Mitte hinweg, federt zurück. Das ist `out_back` in Aktion. Klein, aber spürbar besser als „springt einfach in die Mitte".

> **Aha-Moment**: warum dauert die `INTRO_FRAMES = 90` Pause länger als die 600ms-Animation? Weil der Tween bei 600 ms am Ziel angekommen ist und dort stehen bleibt — die restlichen ~900 ms liegt der Schriftzug einfach in der Mitte. Genug Zeit für den Spieler zum Lesen.

## Was wir nicht gemacht haben

- **Grunts** mit Tween: könnten wir machen (sanfter Sinus-Anflug), aber Grunts sind absichtlich „dumm und zahlreich" — Linearität passt zu ihrem Charakter. Stilistische Entscheidung.
- **Player-Bewegung** mit Tween: macht keinen Sinn, weil Player auf Tasten reagiert, nicht auf einer vordefinierten Kurve fliegt.
- **Bullet-Bahn** mit Tween: linear ist physikalisch korrekt für Schüsse — kein Easing nötig.

Die Faustregel: **Tween nutzen wo Bewegung „autonom" ist** (das System bewegt etwas vorgegebenes), **nicht für Spieler-gesteuerte Bewegung**.

## Pause und Resume

Falls du im Spiel eine Pause-Funktion einbauen willst (kommt in Kap 17): `TWEEN_PAUSE(t)` und `TWEEN_RESUME(t)` machen das richtige. Der pausierte Tween hält seine Position; nach Resume läuft er weiter, als wäre nichts passiert.

```basic
TWEEN_PAUSE(swing)        ' Bomber friert horizontal ein
TWEEN_RESUME(swing)       ' setzt fort, wo er war
```

Du musst das nicht für jeden Tween manuell machen — wenn der ganze Spielzustand pausiert (UpdatePlaying läuft nicht mehr), zeichnen wir nur das letzte Bild. Aber falls du den `wave_intro_slide` z.B. fortsetzen willst, sind die Funktionen da.

## Übungen

**1. Player-Triebwerks-Pulse.** Erweitere den Triebwerks-Trail aus Kap 13 um eine pulsierende **Helligkeit**. Lege einen `TWEEN_NEW_LOOP(0.5, 1.0, 500, "inout_sine")` an. Pro Frame: lies den Wert, multipliziere mit der Anzahl emittierter Particles (statt fester `2`, jetzt `INT(2 + value)`). Effekt: der Trail pulsiert dezent.

**2. Score-Pop.** Wenn der Spieler einen Bomber tötet (250 Pkt), zeige für 800 ms „+250" als Pop-Up an der Treffer-Position. Die Y-Position wird per `TWEEN_NEW(...)` von der Treffer-Position nach oben getweent (negative Y-Veränderung), die Anzeige verschwindet, wenn der Tween done ist.

**3. Bomber-Sweep.** Statt Pingpong-Schaukel: lass den Bomber einmal vollständig von links nach rechts gleiten und dann zurück, mit `TWEEN_NEW_PINGPONG`-Bereich `(0.0, WIDTH - w)`. So macht er eine durchgehende Sweep-Bewegung über das gesamte Spielfeld.

**4. Stretch — Synchronisierte Welle.** Spawn-Logik so ändern, dass alle Gegner einer Welle zur gleichen Zeit reinfliegen — aber jeder mit leichter Verzögerung. Hinweis: pro Bomber einen `intro`-Tween mit verschiedener Dauer, in `Update()` erst nach `TWEEN_DONE(intro)` die normale Bewegung starten.

## Zusammenfassung

Du hast in diesem Kapitel:

- das `tween`-Modul mit `IMPORT "tween"` aktiviert,
- den Unterschied zwischen `TWEEN_NEW`, `TWEEN_NEW_LOOP` und `TWEEN_NEW_PINGPONG` verstanden,
- Easing-Kurven kennengelernt und vier davon in Aktion gesehen,
- Bombers Zickzack-Bewegung von linear auf `inout_sine`-Pingpong umgestellt,
- den Wave-Intro-Schriftzug mit `out_back`-Easing einfliegen lassen,
- gelernt, **wo** Tween hilft (autonome Bewegung) und **wo** nicht (Spieler-gesteuert).

Im **nächsten Kapitel** wechseln wir aus dem Spielfluss heraus und um ihn herum: wir bauen ein **Hauptmenü**, einen **Game-Over-Screen** und einen **Pause-Modus** mit dem `scene`-Modul. Star Pilot wird endlich ein „richtiges" Spiel mit Anfang und Ende.

## Code-Stand am Ende des Kapitels

- [`code/kap-14/01_tween_demo.gb`](code/kap-14/01_tween_demo.gb) — vier Easings im direkten Vergleich
- [`code/kap-14/02_formation.gb`](code/kap-14/02_formation.gb) — fünf Enemies in V-Formation, gestaffelter Einflug mit `out_back`
- [`code/kap-14/main.gb`](code/kap-14/main.gb) — Star Pilot mit Bomber-Pingpong und Wave-Intro-Slide
