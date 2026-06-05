# Modul `input`

Action-basiertes Input-Mapping mit Edge-Detection. Statt im Spielcode ueberall Keycodes (`KEYPRESSED(1073741904)`) zu streuen, gruppiert das `input`-Modul Tasten zu benannten **Actions**. Eine Action kann an mehrere Tasten gebunden sein (WASD + Pfeiltasten gleichzeitig).

```basic
IMPORT "input"
```

## Übersicht

### Action-Mapping + Edge-Detection (Tastatur **und** Gamepad)

| Funktion | Rueckgabe | Wirkung |
|---|---|---|
| `INPUT_BIND(action, key1, [key2], ...)` | — | Action → Key-Liste (mischt Tastatur + Gamepad-Codes) |
| `INPUT_UNBIND(action)` | — | Action loeschen |
| `INPUT_RESET()` | — | Alle Bindings + State leeren |
| `INPUT_UPDATE()` | — | Frame-Snapshot (Pflicht jeden Frame, pollt auch Gamepads) |
| `INPUT_HELD(action)` | BOOLEAN | Aktuell gedrueckt |
| `INPUT_PRESSED(action)` | BOOLEAN | Edge: gerade JETZT gedrueckt |
| `INPUT_RELEASED(action)` | BOOLEAN | Edge: gerade losgelassen |
| `INPUT_AXIS(neg, pos)` | INTEGER | `-1 / 0 / 1` als virtuelle Achse |
| `INPUT_BOUND(action)` | BOOLEAN | Ist die Action registriert? |

### Gamepad / Joystick

| Funktion | Rueckgabe | Wirkung |
|---|---|---|
| `INPUT_JOY_COUNT()` | INTEGER | Anzahl angeschlossener Pads |
| `INPUT_JOY_NAME(slot)` | STRING | Pad-Name (z.B. "Xbox Wireless Controller") |
| `INPUT_JOY_AXIS(slot, axis_name$)` | FLOAT | -1.0..+1.0 mit Deadzone |

**Gamepad-Konstanten** (case-insensitive, wie `KEY_*`):

| Konstante | Mapping (Xbox) |
|---|---|
| `JOY_BUTTON_A` | A |
| `JOY_BUTTON_B` | B |
| `JOY_BUTTON_X` | X |
| `JOY_BUTTON_Y` | Y |
| `JOY_BUTTON_LB` / `JOY_BUTTON_RB` | LB / RB (Bumper) |
| `JOY_BUTTON_BACK` / `JOY_BUTTON_START` | Back/Select, Start/Menu |
| `JOY_BUTTON_LSTICK` / `JOY_BUTTON_RSTICK` | Stick-Click (L3/R3) |
| `JOY_DPAD_UP` / `_DOWN` / `_LEFT` / `_RIGHT` | DPad-Richtungen |

**Axis-Namen** fuer `INPUT_JOY_AXIS`:

| Name | Wirkung |
|---|---|
| `"left_x"` / `"left_y"` | Linker Stick (Y: oben = -1, unten = +1) |
| `"right_x"` / `"right_y"` | Rechter Stick |
| `"lt"` / `"rt"` | Trigger (Ruhelage 0 oder -1, je nach Pad) |

## Konzept

Drei Zustands-Konzepte pro Action:

- **HELD** — die Action ist gerade gedrueckt (irgendeine ihrer Tasten). Pro Frame `TRUE` solange gedrueckt.
- **PRESSED** — Edge: die Action war im VORHERIGEN Frame `FALSE`, jetzt `TRUE`. Genau einen Frame lang `TRUE`. Klassisch fuer Sprung/Schuss (eine Aktion pro Tastendruck).
- **RELEASED** — Edge: war `TRUE`, ist jetzt `FALSE`. Genau einen Frame.

Edge-Detection braucht den `INPUT_UPDATE()`-Call am Frame-Anfang — das Modul vergleicht aktuellen Snapshot mit dem vorigen. Ohne UPDATE bleiben PRESSED/RELEASED dauerhaft `FALSE`.

## Klassischer Game-Loop

```basic
IMPORT "input"

' Setup einmal vor der Loop -- Tastatur UND Gamepad in einer Action.
INPUT_BIND("move_left",  KEY_LEFT,  KEY_A, JOY_DPAD_LEFT)
INPUT_BIND("move_right", KEY_RIGHT, KEY_D, JOY_DPAD_RIGHT)
INPUT_BIND("jump",       KEY_SPACE, KEY_W, JOY_BUTTON_A, JOY_DPAD_UP)
INPUT_BIND("shoot",      KEY_X,     KEY_RETURN, JOY_BUTTON_X)
INPUT_BIND("quit",       KEY_ESCAPE, JOY_BUTTON_BACK)

DIM x AS INTEGER
x = 100

WHILE NOT QUITREQUESTED()
    INPUT_UPDATE()              ' Snapshot fuer diesen Frame

    IF INPUT_HELD("move_left")  THEN x = x - 2
    IF INPUT_HELD("move_right") THEN x = x + 2

    IF INPUT_PRESSED("jump")  THEN PRINT "JUMP!"
    IF INPUT_PRESSED("shoot") THEN PRINT "BANG!"
    IF INPUT_PRESSED("quit")  THEN EXIT WHILE

    CLS()
    BOX(x, 100, x + 20, 120, WHITE)
    FLIP()
WEND
```

## `INPUT_AXIS` — virtuelle Achse

Klassisches Pattern fuer Movement: `axis = -1` wenn links gedrueckt, `+1` wenn rechts, `0` wenn nichts oder beides.

```basic
INPUT_BIND("move_left",  KEY_LEFT,  KEY_A)
INPUT_BIND("move_right", KEY_RIGHT, KEY_D)

DIM ax AS INTEGER
ax = INPUT_AXIS("move_left", "move_right")
x = x + ax * 2     ' negativ wenn links, positiv wenn rechts
```

Analog fuer Y-Achse:

```basic
DIM ay AS INTEGER
ay = INPUT_AXIS("move_up", "move_down")
```

## Re-Binding (z.B. Settings-Menue)

`INPUT_BIND` mit gleichem Action-Namen **ueberschreibt** die alte Key-Liste. Ideal fuer ein "Tasten umkonfigurieren"-Menue:

```basic
' User klickt "Jump aendern", drueckt neue Taste...
INPUT_BIND("jump", new_key_code)   ' alte Bindings verloren
```

## Multi-Key-Bindings

Eine Action triggert, sobald **irgendeine** ihrer Tasten gedrueckt ist. Damit kannst du WASD und Pfeiltasten parallel binden, ohne dass der User wechseln muss:

```basic
INPUT_BIND("move_left",  KEY_LEFT, KEY_A, KEY_KP4, KEY_H)   ' 4 Bindings
' INPUT_HELD("move_left") -> TRUE, sobald eine davon gedrueckt ist
```

## Action-Namen

Sind **case-insensitive** (intern lower-case-vergleichen). `INPUT_BIND("Jump", ...)` und `INPUT_HELD("jump")` referenzieren dieselbe Action.

## Test-Reset

`INPUT_RESET()` loescht alle Bindings + den prev/cur-State. Tests rufen das zwischen Cases auf, damit Bindings nicht persistieren.

## Analoger Stick

`INPUT_JOY_AXIS(slot, axis$)` liefert -1.0..+1.0 mit Deadzone (Default 0.15). Klassisches Pattern fuer fluessige Bewegung mit Pad:

```basic
DIM stick_x AS FLOAT
stick_x = INPUT_JOY_AXIS(0, "left_x")

' Stick + DPad zusammen: nimm den staerkeren Input
DIM axis AS INTEGER
axis = INPUT_AXIS("move_left", "move_right")    ' aus Tastatur/DPad
IF stick_x < -0.3 THEN axis = -1
IF stick_x >  0.3 THEN axis = 1
```

**Trigger** (`lt` / `rt`) haben keine Deadzone -- je nach Pad liegen sie als 0..1 oder -1..1 an. Wer Schiess-on-RT-Halten will: einfach `IF INPUT_JOY_AXIS(0, "rt") > 0.5 THEN Shoot() END IF`.

## Multi-Player

V1: alle angeschlossenen Pads schiessen in dieselbe Action. Wer Spieler-getrennt will, baut sich auf der Anwendungs-Ebene: pollt die Pads einzeln per `INPUT_JOY_AXIS` und liest Buttons direkt (oder per zusaetzliche `JOY_BUTTON_PAD_N`-Codes — ist eine Erweiterung fuer spaeter).

## Beispiel

- [examples/59_input.gb](../examples/59_input.gb) — Tastatur-Pattern mit Multi-Bind, Axis, Edge-Detection.
- [examples/77_tiled_platformer.gb](../examples/77_tiled_platformer.gb) — Platformer mit Tastatur + Gamepad (Stick + DPad + A-Sprung).
