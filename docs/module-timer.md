# Modul `timer`

Geplante Aktionen ohne MILLIS-Buchfuehrung (nativ in `dhrt`): "in 2 Sekunden
mach X", "alle 500 ms mach Y" und Cooldowns in einer Zeile. Schliesst die
Luecke zwischen den rohen Zeit-Builtins (`MILLIS`/`TIMER`/`DELTA`) und dem,
was Spiele wirklich brauchen -- Spawner, verzoegerte Effekte, Schuss-Raten.

```basic
IMPORT "timer"
```

## Timer: AFTER / EVERY

| Funktion | Wirkung |
|---|---|
| `TIMER_AFTER(ms, fn)` → INTEGER | FUNCREF einmalig nach `ms` aufrufen; liefert Timer-ID |
| `TIMER_EVERY(ms, fn)` → INTEGER | FUNCREF alle `ms` aufrufen (`ms > 0`); liefert Timer-ID |
| `TIMER_CANCEL(id)` | Timer abbrechen (gefeuerte/unbekannte ID = No-Op) |
| `TIMER_ACTIVE(id)` → BOOLEAN | laeuft der Timer noch? |
| `TIMER_COUNT()` → INTEGER | Anzahl aktiver Timer |
| `TIMER_CLEAR()` | alle Timer + Cooldowns verwerfen (z.B. beim Scene-Wechsel) |
| `TIMER_UPDATE()` | **pro Frame aufrufen** -- feuert die faelligen Callbacks |

`TIMER_UPDATE()` folgt demselben Muster wie `INPUT_UPDATE()`/`GUI_UPDATE()`:
ohne den Aufruf am Frame-Anfang passiert nichts. Die Callbacks sind
parameterlose `SUB`s/`FUNCTION`s als FUNCREF (bare Name).

```basic
IMPORT "timer"

SUB explodiere()
    PRINT "BOOM"
END SUB

SUB spawne()
    PRINT "neuer Gegner"
END SUB

TIMER_AFTER(2000, explodiere)        ' einmalig in 2s
DIM spawner AS INTEGER
spawner = TIMER_EVERY(500, spawne)   ' alle 500ms

WHILE NOT QUITREQUESTED()
    TIMER_UPDATE()                   ' faellige Callbacks feuern
    ' ... Spiel-Logik ...
    FLIP()
WEND

TIMER_CANCEL(spawner)
```

**Semantik-Details:**

- Ein `TIMER_EVERY` feuert pro `TIMER_UPDATE` hoechstens **einmal** -- nach
  einem Lag/`SLEEP` gibt es keinen Aufhol-Burst, der naechste Termin ist
  immer `jetzt + ms`.
- Timer-IDs bleiben stabil (Tombstones): `TIMER_CANCEL` einer ID veraendert
  keine anderen IDs.
- Callbacks duerfen selbst Timer registrieren oder canceln; neu registrierte
  Timer werden fruehestens beim **naechsten** `TIMER_UPDATE` faellig.
- `TIMER_AFTER(0, fn)` feuert beim naechsten `TIMER_UPDATE` -- praktisch
  fuer "am Frame-Ende / entkoppelt ausfuehren".

## COOLDOWN -- Ratenbegrenzer in einer Zeile

`COOLDOWN(id$, ms)` ist String-ID-basiert (wie das Immediate-Mode-`ui`-Modul,
kein Handle noetig): liefert `TRUE`, wenn die ID frei ist -- **und startet
dann sofort die Sperre**. Solange die Sperre laeuft, kommt `FALSE`.

```basic
' Schussrate: maximal alle 250ms
IF INPUT_HELD("fire") AND COOLDOWN("schuss", 250) THEN
    Schiesse()
END IF

' Sound-Spam-Schutz (ersetzt das MILLIS-Pattern aus docs/module-audio.md)
IF COOLDOWN("hit_sfx", 100) THEN AUDIO_PLAY(hit_sound)
```

`COOLDOWN` braucht kein `TIMER_UPDATE` -- die Pruefung passiert beim Aufruf.

## Abgrenzung

- **Werte ueber Zeit animieren** → Modul `tween` (Easings) bzw. `curves`.
- **Komplexe zeitliche Ablaeufe** (Cutscenes, Boss-Phasen) → Coroutines
  (`YIELD` + `CORO_RESUME`).
- **Frame-Delta fuer Bewegung** → `DELTA()`; **Stoppuhr** → `TIMER()`;
  **Zeitstempel** → `MILLIS()`.

## Beispiel

[examples/113_timer.gb](../examples/113_timer.gb) — Spawner via
`TIMER_EVERY`, verzoegerte Explosion via `TIMER_AFTER`, Schuss-Cooldown.
