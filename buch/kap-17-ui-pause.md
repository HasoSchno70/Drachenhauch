# Kapitel 17 — UI und Pause: das Spiel wird komfortabel

In Kapitel 15 haben wir den Scene-Stack eingeführt und versprochen, dass Pause „fast trivial" wird. Jetzt lösen wir das Versprechen ein. Plus: wir bauen ein **Optionen-Menü** mit echten UI-Elementen — Slider für die Geschwindigkeit, Checkbox für den Triebwerks-Trail.

Dafür gibt's das `ui`-Modul. Es bietet **Immediate-Mode-UI**: pro Frame zeichnen wir die Widgets neu, jedes hat eine String-ID, und der Modul-State (Mauseingaben, Klick-Erkennung, Slider-Werte) lebt im Modul. Klingt komplexer als es ist — du wirst sehen.

## Lernziele

Nach diesem Kapitel:

- aktivierst du das `ui`-Modul mit `IMPORT "ui"`
- nutzt du `UI_BUTTON`, `UI_CHECKBOX`, `UI_SLIDER` mit String-IDs
- weißt, warum `UI_END_FRAME()` vor jedem `FLIP()` Pflicht ist
- baust du ein Pause-Overlay als `SCENE_PUSH`
- hast Star Pilot um Pause + Optionen-Menü erweitert
- nutzt einen Tween für Pop-In-Animation des Pause-Panels

## Was Immediate-Mode-UI ist

In klassischen GUI-Toolkits (Qt, Tk, etc.) **legst du Widget-Objekte an**, hängst Event-Handler dran, und das Toolkit kümmert sich um Lebenszyklus, Layout, Events. Das ist mächtig, aber für Spiele Overkill.

**Immediate-Mode-UI** macht das anders: pro Frame deklarierst du, was du sehen willst, und kriegst Eingaben als Rückgabewert:

```basic
IF UI_BUTTON("btn_start", 100, 50, 80, 24, "Start") THEN
    PRINT "Klick!"
END IF
```

Diese Zeile macht *alles*: zeichnet den Button, prüft Mausposition, erkennt Klick, gibt `TRUE` im Frame des Klicks zurück. Du musst nichts allokieren, nichts disposen, kein Callback.

Der „State" zwischen Frames (z.B. „User hat Maus gedrückt aber noch nicht losgelassen", „Slider ist auf 0.7") lebt **im Modul** — über die **ID-Strings**. Pro Widget eine eindeutige ID; das Modul merkt sich pro ID den State.

> **Wichtig**: zwei Widgets mit **derselben ID** auf einem Bildschirm sind ein Bug. Der State würde sich überschneiden. Gib jedem Widget einen eigenen Namen.

## Schritt 1: Drei Widgets

Eine kleine Demo, alle drei UI-Bausteine isoliert:

```basic
IMPORT "ui"

DIM clicks AS INTEGER
clicks = 0

SCREEN(320, 240, "UI Demo", 2)

WHILE NOT QUITREQUESTED()
    CLS(&H141E3C)

    IF UI_BUTTON("btn_count", 20, 30, 100, 24, "Klick mich") THEN
        clicks = clicks + 1
    END IF
    UI_LABEL(140, 36, f"Klicks: {clicks}", &HFFFFFF)

    DIM trail_on AS BOOLEAN
    trail_on = UI_CHECKBOX("cb_trail", 20, 80, "Trail an", TRUE)
    UI_LABEL(140, 80, f"Trail: {trail_on}", &HFFFFFF)

    DIM speed AS FLOAT
    speed = UI_SLIDER("sl_speed", 20, 130, 100, 1.0, 10.0, 5.0)
    UI_LABEL(140, 132, f"Speed: {INT(speed)}", &HFFFFFF)

    UI_END_FRAME()
    FLIP()
    SLEEP(16)
WEND
```

Run drücken. Klicke den Button mehrfach — der Counter rechts steigt. Klicke die Checkbox — `trail_on` wechselt zwischen TRUE und FALSE. Ziehe den Slider — Speed-Wert ändert sich live.

Das war's. Drei Widget-Aufrufe, drei direkte Ergebnisse. Kein Setup, kein Teardown.

### Die Signaturen

| Widget | Signatur | Rückgabe |
|---|---|---|
| `UI_LABEL(x, y, text$[, color])` | nur Anzeige | — |
| `UI_BUTTON(id$, x, y, w, h, text$[, bg, fg])` | Button | BOOLEAN — `TRUE` im Frame des Klick-Loslassens |
| `UI_CHECKBOX(id$, x, y, label$[, default])` | Toggle | BOOLEAN — aktueller Status |
| `UI_SLIDER(id$, x, y, w, min, max[, default])` | horizontaler Slider | FLOAT — aktueller Wert |

Defaults werden **nur beim ersten Aufruf** der ID angewandt. Später bleiben die User-getoggelten Werte erhalten — auch wenn der `UI_CHECKBOX("cb_trail", ..., FALSE)`-Aufruf in jedem Frame `FALSE` als Default mitgibt.

### `UI_END_FRAME()` — die Pflicht-Zeile

Vor jedem `FLIP()` muss `UI_END_FRAME()` aufgerufen werden. Das Modul nutzt das, um Maus-Edge-Detection zu erkennen („wurde gerade losgelassen?") — vergisst du es, klappt Klick-Erkennung nicht mehr richtig.

Faustregel: in jeder Datei mit `IMPORT "ui"` steht am Ende der Schleife:

```basic
UI_END_FRAME()
FLIP()
SLEEP(16)
```

Vergiss es nicht. Wenn deine Buttons plötzlich nicht reagieren — das ist die erste Stelle zum Schauen.

## Schritt 2: Pause als Scene-Push

In Kapitel 15 hatten wir die Scene-Stack-Mechanik etabliert. Pause ist die Lehrbuch-Anwendung:

```basic
' Im UpdatePlaying:
IF p_pressed_now THEN SCENE_PUSH("pause")

' In UpdatePause:
IF p_pressed_now THEN SCENE_POP()       ' zurueck zu "playing"
```

Der Scene-Stack regelt automatisch:
- Beim Push: aktueller Spielzustand (Player, Bullets, Enemies) bleibt erhalten — er liegt unter dem Push.
- Beim Pop: wir sind genau dort, wo wir waren. Player-Position, Score, Welle, alles wie vorher.

Im **Update-Loop** wird im Pause-Modus *nur* `UpdatePause()` aufgerufen — Player und Enemies werden nicht aktualisiert, das Spiel friert ein. Aber im **Draw-Loop** zeichnen wir die Spielwelt weiterhin im Hintergrund — das gibt dem Pause-Overlay Atmosphäre statt eines schwarzen Bildschirms.

### Pause-Mechanik isoliert

Bevor wir das Spiel anpacken, eine Mini-Demo:

```basic
IMPORT "scene"
IMPORT "ui"

DIM box_x AS INTEGER : DIM p_was_pressed AS BOOLEAN
box_x = 20
SCENE_SWITCH("playing")

SCREEN(320, 240, "Pause-Demo", 2)
WHILE NOT QUITREQUESTED()
    DIM p_now AS BOOLEAN
    p_now = KEYPRESSED(KEY_P)
    DIM p_pressed_now AS BOOLEAN
    p_pressed_now = p_now AND NOT p_was_pressed
    p_was_pressed = p_now

    SELECT CASE SCENE_CURRENT()
        CASE "playing"
            box_x = box_x + 1
            IF box_x > 300 THEN box_x = 20

            CLS(&H141E3C)
            BOX(box_x, 100, box_x + 20, 140, &HFFDC00)
            TEXT(8, 220, "P = Pause", &HCCCCCC)

            IF p_pressed_now THEN SCENE_PUSH("pause")

        CASE "pause"
            ' Spielwelt eingefroren weiter zeichnen
            CLS(&H141E3C)
            BOX(box_x, 100, box_x + 20, 140, &HFFDC00)

            ' Pause-Overlay
            BOX(60, 70, 260, 170, &H1E2845)
            RECT(60, 70, 260, 170, &HFFDC00)
            TEXT(120, 80, "PAUSE", &HFFDC00)

            IF UI_BUTTON("btn_resume", 80, 110, 160, 22, "Weiter (oder P)") THEN
                SCENE_POP()
            END IF

            IF p_pressed_now THEN SCENE_POP()
    END SELECT

    UI_END_FRAME()
    FLIP()
    SLEEP(16)
WEND
```

Run drücken. Box bewegt sich, P drücken — Box steht still, Pause-Panel mit Resume-Button erscheint. Resume klicken oder P drücken — Box bewegt sich weiter, von wo sie war.

Das ist die Mechanik in 50 Zeilen. Der Rest des Kapitels ist Anwendung im Spiel.

## Schritt 3: Star Pilot mit Pause + Optionen

Vier neue Scenes (auf dem Stack über `"playing"`):

| Scene | Was passiert |
|---|---|
| `"pause"` | Drei Buttons: Weiter, Optionen, Zurück zum Menue |
| `"options"` | Slider (Geschwindigkeit), Checkbox (Trail), Zurück-Button |

Wenn du im `"options"`-Menu „Zurück" klickst, popst du auf `"pause"`. Wenn du dort „Weiter" klickst, popst du auf `"playing"`. Stack-Magic.

### Globale Settings

Damit der Slider etwas zu ändern hat, machen wir aus `PLAYER_SPEED` (`CONST`) eine normale Variable, plus `trail_on` als Setting:

```basic
DIM player_speed AS INTEGER       ' Slider aendert das
DIM trail_on AS BOOLEAN           ' Checkbox aendert das

' Im Setup:
player_speed = 3
trail_on = TRUE
```

Im `Player.Update()`-Block bleibt `PLAYER_SPEED` stehen — GameBasic ist case-insensitive, `PLAYER_SPEED` und `player_speed` sind dieselbe Variable. Gemeinsame Konvention im Buch ist Kleinschrift für DIMs, Großschrift für CONSTs — der Compiler ist's egal, der Leser nicht.

In `UpdatePlaying`:

```basic
IF trail_on THEN
    PARTICLE_SET_POS(trail_fx, player.x + player.w / 2, player.y + player.h)
    PARTICLE_EMIT(trail_fx, 2)
END IF
```

Trail wird nur emittiert, wenn die Option an ist.

### P toggelt Pause

In `UpdatePlaying`, ganz oben:

```basic
DIM p_now AS BOOLEAN
p_now = KEYPRESSED(KEY_P)
IF p_now AND NOT p_was_pressed THEN
    p_was_pressed = p_now
    SCENE_PUSH("pause")
    pause_slide = TWEEN_NEW(-110.0, 0.0, 250, "out_back")
    RETURN
END IF
p_was_pressed = p_now
```

Edge-Detection wie bei der Leertaste in Kap 8. Beim Push starten wir gleich einen Tween für die Pop-In-Animation.

`UpdatePause()` macht's analog: P drücken popt zurück.

### DrawPause mit Pop-In

```basic
SUB DrawPause()
    CLS(BG_COLOR)
    DrawGameWorld()       ' Spielwelt eingefroren weiter zeichnen
    DrawHUD()

    ' Panel-Y per Tween, Pop-In von oben
    DIM panel_y AS INTEGER
    panel_y = 60 + INT(TWEEN_VALUE(pause_slide))

    BOX(60, panel_y, 260, panel_y + 110, &H1E2845)
    RECT(60, panel_y, 260, panel_y + 110, PLAYER_C)
    TEXT(WIDTH / 2 - 24, panel_y + 8, "PAUSE", PLAYER_C)

    IF UI_BUTTON("p_resume",  80, panel_y + 30, 160, 22, "Weiter (P)") THEN
        SCENE_POP()
    END IF
    IF UI_BUTTON("p_options", 80, panel_y + 56, 160, 22, "Optionen") THEN
        SCENE_PUSH("options")
    END IF
    IF UI_BUTTON("p_quit",    80, panel_y + 82, 160, 22, "Zurueck zum Menue") THEN
        SCENE_SWITCH("menu")
    END IF
END SUB
```

Drei Beobachtungen:

1. **`panel_y` aus dem Tween**: in der ersten Viertelsekunde gleitet das ganze Panel von oben (offscreen) zur Zielposition, mit `out_back`-Easing — leicht überschießend. Buttons positionieren sich relativ zu `panel_y`, also gleiten sie mit.
2. **Drei Buttons mit klar unterschiedlichen IDs**: `p_resume`, `p_options`, `p_quit`. Wenn du zwei Buttons mit derselben ID hättest, würde das Modul den State verwechseln.
3. **Drei verschiedene Aktionen pro Button**: `SCENE_POP()` → zurück zu Playing, `SCENE_PUSH("options")` → Sub-Menü, `SCENE_SWITCH("menu")` → komplett raus.

### DrawOptions mit Slider und Checkbox

```basic
SUB DrawOptions()
    CLS(BG_COLOR)
    DrawGameWorld()
    DrawHUD()

    BOX(40, 50, 280, 200, &H1E2845)
    RECT(40, 50, 280, 200, PLAYER_C)
    TEXT(WIDTH / 2 - 36, 60, "OPTIONEN", PLAYER_C)

    UI_LABEL(56, 90, "Geschwindigkeit:", &HFFFFFF)
    DIM speed_value AS FLOAT
    speed_value = UI_SLIDER("opt_speed", 56, 108, 200, 1.0, 6.0, player_speed + 0.0)
    player_speed = INT(speed_value)
    UI_LABEL(260, 108, f"{player_speed}", PLAYER_C)

    trail_on = UI_CHECKBOX("opt_trail", 56, 138, "Triebwerks-Trail an", trail_on)

    IF UI_BUTTON("opt_back", 56, 168, 200, 22, "Zurueck") THEN
        SCENE_POP()
    END IF
END SUB
```

> **Aha-Moment**: warum `player_speed + 0.0` als Default? Weil `UI_SLIDER` einen FLOAT-Default erwartet, `player_speed` aber INTEGER ist. `+ 0.0` ist ein häufiger Trick zur impliziten Konversion — `INTEGER + FLOAT` → `FLOAT`. Sauberer wäre `FLOAT(player_speed)` als Cast-Funktion, aber `+ 0.0` reicht.

> **Slider gibt FLOAT zurück, wir brauchen INTEGER**: `player_speed = INT(speed_value)` rundet ab. Bei `speed_value = 4.7` → `player_speed = 4`. Da der Slider stetig ist, springt der Wert bei der Mitte zwischen 4 und 5 um — der User merkt das nicht, weil die Anzeige rechts daneben den ganzzahligen Wert zeigt.

### Hauptschleife mit den neuen Scenes

```basic
WHILE NOT QUITREQUESTED()
    SELECT CASE SCENE_CURRENT()
        CASE "menu"        : UpdateMenu()
        CASE "playing"     : UpdatePlaying()
        CASE "wave_intro"  : UpdateWaveIntro()
        CASE "gameover"    : UpdateGameOver()
        CASE "pause"       : UpdatePause()
        CASE "options"     : UpdateOptions()
    END SELECT

    PARTICLE_UPDATE(explosion_fx, 16)
    PARTICLE_UPDATE(trail_fx, 16)

    SELECT CASE SCENE_CURRENT()
        CASE "menu"        : DrawMenu()
        CASE "playing"     : DrawPlaying()
        CASE "wave_intro"  : DrawWaveIntro()
        CASE "gameover"    : DrawGameOver()
        CASE "pause"       : DrawPause()
        CASE "options"     : DrawOptions()
    END SELECT
    UI_END_FRAME()
    FLIP()
    SLEEP(16)
WEND
```

Das Hinzufügen einer neuen Scene ist genau **eine** Zeile in jedem `SELECT CASE`. So skaliert das Pattern.

## Schritt 4: Was wir gerade gebaut haben

Star Pilot ist jetzt ein vollständiges Spiel mit:

- **Hauptmenü** (Kap 15)
- **Spiel** mit Wellen-Mechanik (Kap 12) und Tween-Choreographie (Kap 14)
- **Pause** mit Sub-Menüs (jetzt)
- **Optionen** mit Slider und Checkbox (jetzt)
- **Game-Over** mit persistentem Highscore (Kap 16)

Wenn du jetzt einem Freund das Spiel zeigst, würde er nicht sofort merken, dass es **Lehrbuch-Code** ist. Es fühlt sich wie ein normales kleines Spiel an. Das ist der Punkt, an dem du auf vier Monate „Lernen" zurückblickst und siehst, was rausgekommen ist.

## Übungen

**1. Settings persistent.** `player_speed` und `trail_on` werden aktuell nur in der laufenden Sitzung gemerkt — nach Neustart sind sie wieder Defaults. Speichere sie zusammen mit dem Highscore in `starpilot.save`. Hinweis: in `Setup` mit `SAVE_GET_INT_OR(...)` / `SAVE_GET_BOOL_OR(...)` laden, in `DrawOptions` (am Ende) bei Änderung speichern.

**2. Lautstärke-Slider.** Wenn du Sound einbaust (siehe Modul-Doku zu `LOADSOUND` / `PLAYSOUND`), addiere einen zweiten Slider „Lautstärke" (0.0–1.0). Speichere ihn ebenfalls persistent.

**3. Schwierigkeits-Buttons.** Statt einem Slider drei Buttons im Optionen-Menu: „Easy", „Normal", „Hard". Jeder setzt Variablen wie `enemy_speed_multiplier`, `wave_size_multiplier`. In `Wave.StartCurrent()` werden die angewandt.

**4. Stretch — Pause während Wave-Intro.** P kann aktuell nur im `"playing"`-Modus gedrückt werden. Mache es robust: drückbar auch in `"wave_intro"` — dann wird das Wave-Intro pausiert (intro_frames_left wird nicht runtergezählt, Tween wird mit `TWEEN_PAUSE` eingefroren, beim POP wieder mit `TWEEN_RESUME` aufgeweckt).

## Zusammenfassung

Du hast in diesem Kapitel:

- das `ui`-Modul mit Buttons, Checkboxen und Slidern kennengelernt,
- Immediate-Mode-UI als Konzept verstanden — pro Frame zeichnen, der State lebt im Modul,
- die Pflicht von `UI_END_FRAME()` vor `FLIP()` verinnerlicht,
- ein Pause-Overlay mit Pop-In-Animation gebaut (Push + Tween),
- ein Sub-Menü „Optionen" gebaut, das aus dem Pause-Menü gepusht wird,
- `player_speed` und `trail_on` als Live-veränderbare Settings integriert.

Im **letzten Hauptkapitel** (Kapitel 18) bauen wir den **Boss-Fight** mit mehreren Phasen, laden **Wellen aus einer JSON-Datei** und nutzen Star Pilots erste **Named Arguments** beim `NEW Enemy(...)`-Aufruf. Das ist die letzte Etappe.

## Code-Stand am Ende des Kapitels

- [`code/kap-17/01_ui_basics.gb`](code/kap-17/01_ui_basics.gb) — Button, Checkbox, Slider isoliert
- [`code/kap-17/02_pause_overlay.gb`](code/kap-17/02_pause_overlay.gb) — Pause-Mechanik isoliert mit Mock-Spielwelt
- [`code/kap-17/main.gb`](code/kap-17/main.gb) — Star Pilot mit Pause-Menu + Optionen-Sub-Menu + Pop-In-Animation
