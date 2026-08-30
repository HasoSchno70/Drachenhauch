# Modul `scene`

Stack-basierter Scene-/State-Manager für Spiele. Statt globale Flags zu jonglieren (`isInMenu`, `isPaused`, `gameOver`) wird der jeweils aktive Bildschirm als String-Name auf einen Stack gepusht. Pro Scene gibt es einen eigenen Daten-Bucket für kurzzeitigen State.

```basic
IMPORT "scene"
```

## Übersicht

| Funktion | Rückgabe / Wirkung |
|---|---|
| `SCENE_PUSH(name$)` | Scene oben auf den Stack |
| `SCENE_POP()` | Oberste entfernen, vorherige aktiv |
| `SCENE_SWITCH(name$)` | Stack komplett ersetzen |
| `SCENE_CURRENT()` | STRING — aktiver Name (`""` wenn leer) |
| `SCENE_DEPTH()` | INTEGER — wie viele Szenen liegen auf dem Stapel? |
| `SCENE_HAS(name$)` | BOOLEAN — irgendwo im Stack? |
| `SCENE_RESET()` | Stack komplett leeren |
| `SCENE_SET_INT/FLOAT/STRING/BOOL(key$, value)` | Daten in der obersten Scene setzen |
| `SCENE_GET_INT/FLOAT/STRING/BOOL(key$)` | strikt — wirft bei fehlend / falschem Typ |
| `SCENE_GET_INT_OR/FLOAT_OR/STRING_OR/BOOL_OR(key$, default)` | mit Fallback — liefert `default`, wenn der Schluessel fehlt oder den falschen Typ hat |
| `SCENE_HAS_KEY(key$)` | BOOLEAN — ist der Schluessel in der obersten Szene belegt? |
| `SCENE_DELETE(key$)` | idempotent |

## Konzept

Drei Operationen reichen für die meisten Spiele:

- **PUSH/POP**: Pause-Overlay über das laufende Spiel — `SCENE_PUSH("pause")` → Pause aktiv, `SCENE_POP()` → zurück zum Spiel.
- **SWITCH**: kompletter Wechsel ohne Rückweg — Menü → Spiel → Game-Over. `SCENE_SWITCH("playing")` schmeißt den Stack weg und startet bei der neuen Scene.
- **CURRENT**: deine Game-Loop fragt jeden Frame ab, wer dran ist, und dispatcht via `SELECT CASE`.

```basic
IMPORT "scene"

SCENE_SWITCH("menu")

WHILE NOT QUITREQUESTED()
    SELECT CASE SCENE_CURRENT()
        CASE "menu"
            UpdateMenu()
            DrawMenu()
        CASE "playing"
            UpdatePlaying()
            DrawPlaying()
        CASE "gameover"
            UpdateGameOver()
            DrawGameOver()
    END SELECT
    FLIP()
    SLEEP(16)
WEND
```

## Pro-Scene-Daten

Jede Scene hat einen eigenen Daten-Bucket. Du kannst per Scene-Name nicht direkt zugreifen — Operationen wirken immer auf die **oberste** Scene.

```basic
SCENE_PUSH("playing")
SCENE_SET_INT("score", 0)
SCENE_SET_INT("lives", 3)

' ... irgendwann später, im Update-Loop:
DIM s AS INTEGER
s = SCENE_GET_INT("score")
SCENE_SET_INT("score", s + 100)
```

Strikte Getter werfen bei fehlendem Key oder falschem Typ — gut um Tippfehler früh zu fangen. Tolerante Getter mit `_OR(default)` werfen nie:

```basic
DIM hi AS INTEGER
hi = SCENE_GET_INT_OR("highscore", 0)   ' kein Fehler wenn noch nie gesetzt
```

### Daten-Lebenszyklus

- `SCENE_PUSH` erzeugt einen frischen, **leeren** Daten-Bucket. Der Bucket der darunter liegenden Scene bleibt unangetastet und ist nach `SCENE_POP` wieder erreichbar.
- `SCENE_SWITCH` wirft *alle* alten Buckets weg.
- `SCENE_POP` löscht den oberen Bucket — auch wenn du gleich `SCENE_PUSH("gleichname")` hinterherschiebst, ist der neue leer.

## Beispiel: Pause-Overlay

```basic
IMPORT "scene"

SCENE_SWITCH("playing")
SCENE_SET_INT("score", 0)

WHILE NOT QUITREQUESTED()
    IF KEYPRESSED(KEY_P) THEN
        IF SCENE_CURRENT() = "playing" THEN
            SCENE_PUSH("pause")
        ELSEIF SCENE_CURRENT() = "pause" THEN
            SCENE_POP()
        END IF
    END IF

    SELECT CASE SCENE_CURRENT()
        CASE "playing"
            ' Spielmechanik
            ...
        CASE "pause"
            ' Spiel ist eingefroren - nur das Overlay zeichnen
            DrawPlayingFrozen()
            DrawPauseOverlay()
    END SELECT
    FLIP()
WEND
```

## Externer Typ

Das Modul registriert keinen GB-sichtbaren Handle-Typ — der Stack lebt im Modul-State. Ein Test sollte `SCENE_RESET()` vor jedem Test-Run aufrufen.

## Siehe auch

- [`save`](module-save.md) — High-Level Save/Load mit JSON-Backend, ergänzt sich gut mit Scene (Highscore-Persistierung, Run-Settings)
- Vollständiges Beispiel: [`examples/49_pong_scene.dh`](../examples/49_pong_scene.dh) — Pong mit Menu/Playing/GameOver-Scenes
