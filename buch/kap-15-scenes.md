# Kapitel 15 — Scenes: Menu, Playing, GameOver

In Kapitel 12 hatten wir den Spielzustand mit einem `ENUM GameState` modelliert — `PLAYING`, `WAVE_INTRO`, `GAMEOVER`. Das hat funktioniert, war aber nicht ganz ideal: jeder neue Zustand brauchte einen Enum-Eintrag, jede Daten-Verbindung zwischen Zuständen lief über globale Variablen.

In diesem Kapitel ersetzen wir das durch das **`scene`-Modul**: einen Stack-basierten Scene-Manager. Statt eines flachen Enum-Werts haben wir einen Stack von Scene-Namen — und können temporäre Overlays (Wave-Intro, später Pause) einfach „obendrauf legen", ohne den darunter liegenden Zustand zu verlieren. Plus: das Spiel bekommt endlich ein **Hauptmenü** und einen anständigen **Game-Over-Screen**.

## Lernziele

Nach diesem Kapitel:

- aktivierst du das `scene`-Modul mit `IMPORT "scene"`
- unterscheidest du `SCENE_PUSH`, `SCENE_POP` und `SCENE_SWITCH` und weißt, wann was
- nutzt du `SCENE_CURRENT()` mit `SELECT CASE` für den Update-/Draw-Dispatch
- speicherst pro Scene Daten mit `SCENE_SET_INT(...)` / `SCENE_GET_INT_OR(...)`
- hast in Star Pilot vier Scenes implementiert: Menu, Playing, WaveIntro, GameOver

## Schritt 1: Stack statt Flag

Ein einzelnes Boolean reicht für zwei Zustände (`game_over: TRUE/FALSE`). Ein ENUM reicht für eine Handvoll. Aber sobald **temporäre Overlays** dazukommen — Pause, Wave-Intro, Inventar, Optionen — wird's umständlich:

```basic
' Variante 1: viele Booleans, schnell unübersichtlich
DIM is_paused        AS BOOLEAN
DIM is_in_wave_intro AS BOOLEAN
DIM is_inventory_open AS BOOLEAN
IF NOT is_paused AND NOT is_in_wave_intro AND NOT is_inventory_open THEN
    UpdateGame()
END IF
```

Mit einem **Stack** wird das natürlich. Die oberste Scene bestimmt, was passiert. Wenn ein Overlay gepusht wird, läuft das. Wenn es gepoppt wird, kehren wir automatisch zum vorherigen Zustand zurück:

```basic
SCENE_PUSH("pause")        ' jetzt ist Pause oben
' ... User klickt Resume ...
SCENE_POP()                ' Pause weg, "playing" wieder oben
```

Das `scene`-Modul macht genau das.

## Schritt 2: Drei Stack-Operationen

Drei Funktionen, klar abgegrenzt:

| Funktion | Wann benutzen |
|---|---|
| `SCENE_SWITCH(name$)` | Stack komplett leeren und durch eine neue Scene ersetzen — **kein Zurück**. Beispiele: Menu → Playing, Playing → GameOver. |
| `SCENE_PUSH(name$)` | Eine Scene **on top** legen, vorherige bleibt erhalten. Klassisch für Overlays. Beispiele: Playing → WaveIntro, Playing → Pause. |
| `SCENE_POP()` | Oberste Scene entfernen, automatisch zurück zur darunterliegenden. Pendant zu PUSH. |

Plus zwei Lese-Hilfen:

- `SCENE_CURRENT() -> STRING` — Name der obersten Scene (oder `""` wenn Stack leer)
- `SCENE_DEPTH() -> INTEGER` — Anzahl Scenes im Stack

## Schritt 3: Eine Mini-Demo

Bevor wir das Spiel umbauen, ein Mini-Beispiel mit drei Scenes ohne Spielmechanik:

```basic
IMPORT "scene"

SCENE_SWITCH("title")
SCREEN(320, 240, "Scene Demo", 2)

WHILE NOT QUITREQUESTED()
    SELECT CASE SCENE_CURRENT()
        CASE "title"
            CLS(&H141E3C)
            TEXT(120, 80, "TITLE-SCREEN", &HFFDC00)
            TEXT(100, 120, "ENTER -> Spiel", &HFFFFFF)
            IF KEYPRESSED(KEY_RETURN) THEN
                SCENE_SWITCH("playing")
            END IF
        CASE "playing"
            CLS(&H101A30)
            TEXT(132, 80, "SPIELE...", &HFFDC00)
            TEXT(100, 120, "ESC -> Game Over", &HFFFFFF)
            IF KEYPRESSED(KEY_ESCAPE) THEN
                SCENE_SWITCH("gameover")
            END IF
        CASE "gameover"
            CLS(&H300010)
            TEXT(128, 80, "GAME OVER", &HFF4444)
            TEXT(76, 120, "ENTER -> zurueck zum Titel", &HFFFFFF)
            IF KEYPRESSED(KEY_RETURN) THEN
                SCENE_SWITCH("title")
            END IF
    END SELECT
    FLIP()
    SLEEP(16)
WEND
```

Run drücken. Du siehst den Title-Screen, ENTER bringt dich zu „Playing", ESC zu „Game Over", ENTER zurück zum Title. Das ist das Schema, das wir gleich auf Star Pilot anwenden.

> **Beobachte**: jeder Scene-Block hat eigene `CLS()`-Farbe, eigene Texte, eigene Tastenabfragen. Saubere Trennung — kein gemischter Code mehr.

## Schritt 4: Star Pilot mit vier Scenes

Vier Scenes für unser Spiel:

| Scene | Was passiert |
|---|---|
| `"menu"` | Titel, „ENTER zum Starten" |
| `"playing"` | Das eigentliche Spiel: Player bewegt sich, Bullets fliegen, Welle läuft |
| `"wave_intro"` | Overlay (PUSH): Schriftzug „WELLE X", für ~1.5 Sekunden über dem laufenden Spielbild |
| `"gameover"` | Endstand, „ENTER → zurück zum Menü" |

Wechsel-Logik:

- **Programm-Start**: `SCENE_SWITCH("menu")` (in Setup)
- **ENTER im Menu**: `ResetGame()`, dann `SCENE_SWITCH("playing")`, dann `StartWaveIntro()` für Welle 1
- **Welle abgeschlossen**: `wave.NextWave()`, dann `StartWaveIntro()` (= `SCENE_PUSH("wave_intro")`)
- **WaveIntro nach 90 Frames**: `SCENE_POP()` → automatisch zurück zu `"playing"`
- **Player-Tod**: `SCENE_SET_INT("final_score", score)`, dann `SCENE_SWITCH("gameover")`
- **ENTER im GameOver**: `SCENE_SWITCH("menu")`

### Pro-Scene-Daten für den Endstand

Beim Player-Tod merken wir uns den Score in der GameOver-Scene:

```basic
IF NOT player.alive THEN
    SCENE_SET_INT("final_score", score)
    SCENE_SWITCH("gameover")
    RETURN
END IF
```

`SCENE_SET_INT(key$, value)` speichert den Wert in der **aktuell obersten** Scene — also gleich nach dem `SWITCH` im neuen `"gameover"`-Bucket. Wer sich wundert: bei `SCENE_SWITCH` wird der Stack komplett geleert *bevor* die neue Scene gepusht wird, also gibt's einen frischen Daten-Bucket.

In der `DrawGameOver`-Sub lesen wir den Wert wieder:

```basic
DIM final_score AS INTEGER
final_score = SCENE_GET_INT_OR("final_score", 0)
TEXT(..., f"Endstand: {final_score}", &HFFFFFF)
```

`_OR(0)` ist die tolerante Variante — wenn der Key fehlt (sollte nicht, ist aber sicher), kommt `0` zurück.

### Setup vs. ResetGame

Eine wichtige Unterscheidung: **Setup() läuft genau einmal** beim Programmstart — Player-Klasse instanziieren, Pools allokieren, Particles konfigurieren, Window öffnen. Das passiert nicht jedes Mal, wenn der User ein neues Spiel startet.

**ResetGame() läuft jedes Mal**, wenn der User vom Menü aus „neu starten" wählt: Player-Position auf den Anfangswert, Score auf 0, Lives auf 3, alle Bullets/Enemies auf tot, neue Wave-Instanz.

```basic
SUB Setup()
    ' Allokationen die EINMAL passieren
    player = NEW Player(...)
    ' ... Pools, Particles, SCREEN ...
    SCENE_SWITCH("menu")
END SUB

SUB ResetGame()
    ' Variablen die bei JEDEM neuen Spiel zuruecksetzen
    player.x = WIDTH / 2 - 20
    player.y = HEIGHT - 40
    player.alive = TRUE
    player.lives = 3
    score = 0
    ' Bullets/Enemies auf tot, frische Welle
    ...
END SUB
```

Diese Trennung ist Standard in Spielen mit mehreren Runden. Vorher hatten wir nur Setup — das hat funktioniert, weil das Spiel nach Game-Over einfach hängen blieb. Jetzt mit Menu → mehrere Runden ist die Trennung notwendig.

### StartWaveIntro: PUSH + Animation

Das Wave-Intro ist ein klassisches Push-Overlay. Wir schreiben einen Helper, der Push, Counter und Tween in einem Aufruf zusammenfasst:

```basic
SUB StartWaveIntro()
    SCENE_PUSH("wave_intro")
    intro_frames_left = INTRO_FRAMES
    wave_intro_slide = TWEEN_NEW(-20.0, HEIGHT / 2 - 4.0, 600, "out_back")
END SUB
```

Aufrufer (Menu-Start und Welle-Cleared) machen einfach `StartWaveIntro()`, der Rest läuft automatisch. In `UpdateWaveIntro`:

```basic
SUB UpdateWaveIntro()
    intro_frames_left = intro_frames_left - 1
    IF intro_frames_left <= 0 THEN
        SCENE_POP()        ' zurueck zur darunterliegenden "playing"-Scene
    END IF
END SUB
```

Sobald die 90 Frames um sind, popen wir und sind automatisch wieder im Playing-Modus.

### Update- und Draw-Dispatch

Der Hauptloop wird zur klaren Inhaltsangabe:

```basic
WHILE NOT QUITREQUESTED()
    SELECT CASE SCENE_CURRENT()
        CASE "menu"        : UpdateMenu()
        CASE "playing"     : UpdatePlaying()
        CASE "wave_intro"  : UpdateWaveIntro()
        CASE "gameover"    : UpdateGameOver()
    END SELECT

    PARTICLE_UPDATE(explosion_fx, 16)
    PARTICLE_UPDATE(trail_fx, 16)

    SELECT CASE SCENE_CURRENT()
        CASE "menu"        : DrawMenu()
        CASE "playing"     : DrawPlaying()
        CASE "wave_intro"  : DrawWaveIntro()
        CASE "gameover"    : DrawGameOver()
    END SELECT
    FLIP()
    SLEEP(16)
WEND
```

Pro Scene eigene Update- und eigene Draw-Sub. Die Spielwelt-Drawing (Trail + Bullets + Enemies + Player + Explosionen + HUD) ziehen wir in eine Helper-Sub `DrawGameWorld()` zusammen, weil sie sowohl in `DrawPlaying`, `DrawWaveIntro` als auch `DrawGameOver` gleich aussieht — der Unterschied ist nur das Overlay obendrauf.

> **Aha-Moment**: warum zeigt `DrawGameOver` das letzte Spielbild im Hintergrund? Weil's *atmosphärisch* ist — der Spieler sieht, wo er gestorben ist. Wenn du den schwarzen Game-Over-Screen lieber hättest, ersetze `DrawGameWorld()` darin durch `CLS(&H300000)`.

## Was nicht mehr gebraucht wird

- **`ENUM GameState`** — komplett raus, ersetzt durch Scene-Strings.
- **Globale `state` Variable** — `SCENE_CURRENT()` ist die Quelle der Wahrheit.

`ENUM EnemyType` bleibt — das ist eine Daten-Klassifikation, nicht ein Spielzustand.

## Vorgriff auf Pause

In Kapitel 17 fügen wir einen Pause-Modus hinzu. Mit dem Scene-Stack ist das fast trivial:

```basic
' Im Playing: P drueckt
IF KEYPRESSED(KEY_P) THEN
    SCENE_PUSH("pause")        ' Spiel weiterzeichnen, aber nichts updaten
END IF

' In UpdatePause: P drueckt erneut
IF KEYPRESSED(KEY_P) THEN
    SCENE_POP()                ' zurueck zu "playing"
END IF
```

Mit dem ENUM-Ansatz aus Kap 12 wäre das hässlicher gewesen — man müsste sich „aus welchem State komme ich?" merken, weil PUSH/POP auf einem Flag nicht nativ ist. Der Stack löst das implizit.

## Übungen

**1. Highscore im Menu.** Lege eine globale Variable `highscore AS INTEGER`. Wenn der Player stirbt: `IF score > highscore THEN highscore = score`. Im Menu: zusätzliche `TEXT(...)`-Zeile mit „Highscore: X". (In Kap 16 machen wir den Highscore persistent über Programm-Neustarts.)

**2. Credits-Scene.** Im Menu, neben „ENTER zum Starten", eine Zeile „C → Credits". Drückt der User C: `SCENE_PUSH("credits")` (nicht switch — er soll zurück können). In `UpdateCredits`: ESC pop't zurück zum Menu. Inhalt der Credits-Scene: dein Name, Buch-Verweis, Tools.

**3. Intro-Scene.** Vor dem Menu eine `"intro"`-Scene: zeigt 3 Sekunden lang „GameBasic Press" oder „Studio Star" mit Tween-Pop-In. Nach 3 Sekunden automatisch `SCENE_SWITCH("menu")`. Hinweis: Frame-Counter wie bei `wave_intro`.

**4. Stretch — Endstand-Animation.** Statt das `final_score` als statischen Text zu zeigen, animiere ihn von 0 hochzählend bis zum Endwert. Hinweis: `TWEEN_NEW(0.0, final_score + 0.0, 1500, "out_quad")`, in DrawGameOver lies den Tween-Wert und zeige ihn als gerundeten Integer.

## Zusammenfassung

Du hast in diesem Kapitel:

- den Scene-Stack als Alternative zu Boolean-/ENUM-Flags verstanden,
- `SCENE_PUSH` (Overlay), `SCENE_POP` (zurück), `SCENE_SWITCH` (Stack-Reset) unterschieden,
- vier Scenes für Star Pilot implementiert: Menu, Playing, WaveIntro, GameOver,
- Pro-Scene-Daten genutzt (Endstand-Score in der GameOver-Scene),
- die Trennung Setup() / ResetGame() eingeführt,
- den Pause-Modus aus Kapitel 17 vorbereitet (PUSH/POP statt Flag).

Im **nächsten Kapitel** machen wir den Highscore aus Übung 1 **persistent** mit dem `save`-Modul. Nach Programm-Ende bleibt er auf der Platte und wird beim nächsten Start wieder geladen.

## Code-Stand am Ende des Kapitels

- [`code/kap-15/01_scene_demo.gb`](code/kap-15/01_scene_demo.gb) — drei Scenes mit `SCENE_SWITCH` (ohne Spielmechanik)
- [`code/kap-15/main.gb`](code/kap-15/main.gb) — Star Pilot mit vier Scenes (Menu, Playing, WaveIntro, GameOver)
