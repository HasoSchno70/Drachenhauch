# Modul `particles`

Partikel-Systeme: Funken, Rauch, Explosionen, Trails. Konfigurierbar in Velocity, Lebensdauer, Schwerkraft, Farbe und Größe.

```basic
IMPORT "particles"
```

## Übersicht

| Funktion | Zweck |
|---|---|
| `PARTICLE_SYSTEM_NEW(x, y)` → PARTICLE_SYSTEM | neues System bei (x, y) |
| `PARTICLE_SET_POS(sys, x, y)` | Emitter umsetzen |
| `PARTICLE_SET_VELOCITY(sys, vx_min, vx_max, vy_min, vy_max)` | Streuung der Start-Geschwindigkeiten (Pixel/s) |
| `PARTICLE_SET_LIFETIME(sys, ms_min, ms_max)` | Lebensdauer-Streuung in ms |
| `PARTICLE_SET_GRAVITY(sys, gx, gy)` | konstante Beschleunigung (Pixel/s²) |
| `PARTICLE_SET_COLOR(sys, color)` | Farbe (24-Bit RGB) |
| `PARTICLE_SET_SIZE(sys, px_min, px_max)` | Pixelgröße |
| `PARTICLE_SET_FADE(sys, fade)` | TRUE: Helligkeit nimmt mit Alter ab |
| `PARTICLE_EMIT(sys, count)` | n Partikel sofort ausstoßen |
| `PARTICLE_UPDATE(sys, dt_ms)` | Physik fortschreiben |
| `PARTICLE_DRAW(sys)` | zeichnen (camera-aware) |
| `PARTICLE_COUNT(sys)` → INTEGER | Anzahl lebende Partikel |
| `PARTICLE_CLEAR(sys)` | alle löschen |

## Defaults

`PARTICLE_SYSTEM_NEW(x, y)` kommt mit sinnvollen Defaults:

- Velocity: `(-50..50, -100..-50)` — leichter Funkenregen nach oben
- Lifetime: `500..1000` ms
- Gravity: `(0, 200)` — fallen nach unten
- Color: weiß
- Size: `2..4` Pixel
- Fade: TRUE

Heißt: nur `PARTICLE_SYSTEM_NEW` + `PARTICLE_EMIT` reicht für sichtbare Funken — die Konfiguration kommt nur, wenn man was anderes will.

## Standard-Game-Loop

```basic
IMPORT "particles"

SCREEN(320, 240, "Funken-Demo", 2)

DIM funken AS PARTICLE_SYSTEM
funken = PARTICLE_SYSTEM_NEW(160.0, 120.0)
PARTICLE_SET_COLOR(funken, RGB(255, 200, 80))    ' goldgelb

DIM last_ms AS INTEGER
last_ms = MILLIS()

WHILE NOT QUITREQUESTED()
    DIM now_ms AS INTEGER
    DIM dt AS INTEGER
    now_ms = MILLIS()
    dt = now_ms - last_ms
    last_ms = now_ms

    ' Maus-Position als Emitter-Position
    PARTICLE_SET_POS(funken, MOUSEX() * 1.0, MOUSEY() * 1.0)

    ' Beim Halten der Maus: starker Strahl, sonst sanfter Trail
    IF MOUSEBUTTON(0) THEN
        PARTICLE_EMIT(funken, 5)
    ELSE
        PARTICLE_EMIT(funken, 1)
    END IF

    PARTICLE_UPDATE(funken, dt)

    CLS(RGB(0, 0, 30))
    PARTICLE_DRAW(funken)
    TEXT(8, 8, "Partikel: " + STR$(PARTICLE_COUNT(funken)), RGB(200, 200, 200))
    FLIP()
    SLEEP(16)
WEND
```

## Effekt-Rezepte

**Funken (Aufprall, Coin-Pickup):**

```basic
DIM s AS PARTICLE_SYSTEM
s = PARTICLE_SYSTEM_NEW(0.0, 0.0)
PARTICLE_SET_COLOR(s, RGB(255, 220, 80))
PARTICLE_SET_VELOCITY(s, -120.0, 120.0, -180.0, -40.0)   ' rundum-Streuung
PARTICLE_SET_LIFETIME(s, 300, 600)
PARTICLE_SET_GRAVITY(s, 0.0, 400.0)
PARTICLE_SET_SIZE(s, 2, 4)

' Beim Trigger:
PARTICLE_SET_POS(s, treffer_x, treffer_y)
PARTICLE_EMIT(s, 25)
```

**Rauch (langsam, schwerelos):**

```basic
DIM rauch AS PARTICLE_SYSTEM
rauch = PARTICLE_SYSTEM_NEW(0.0, 0.0)
PARTICLE_SET_COLOR(rauch, RGB(180, 180, 180))
PARTICLE_SET_VELOCITY(rauch, -8.0, 8.0, -30.0, -10.0)
PARTICLE_SET_LIFETIME(rauch, 1500, 2500)
PARTICLE_SET_GRAVITY(rauch, 0.0, 0.0)                    ' kein Fallen
PARTICLE_SET_SIZE(rauch, 4, 8)
PARTICLE_SET_FADE(rauch, TRUE)
```

**Explosion (groß, kurz):**

```basic
DIM bumm AS PARTICLE_SYSTEM
bumm = PARTICLE_SYSTEM_NEW(0.0, 0.0)
PARTICLE_SET_COLOR(bumm, RGB(255, 100, 30))
PARTICLE_SET_VELOCITY(bumm, -200.0, 200.0, -200.0, 200.0)   ' radial
PARTICLE_SET_LIFETIME(bumm, 200, 500)
PARTICLE_SET_GRAVITY(bumm, 0.0, 0.0)
PARTICLE_SET_SIZE(bumm, 3, 6)

' Bei Explosion:
PARTICLE_EMIT(bumm, 80)
```

**Regen (lang, gleichmäßig):**

```basic
DIM regen AS PARTICLE_SYSTEM
regen = PARTICLE_SYSTEM_NEW(0.0, 0.0)
PARTICLE_SET_COLOR(regen, RGB(180, 200, 255))
PARTICLE_SET_VELOCITY(regen, -10.0, 10.0, 100.0, 200.0)
PARTICLE_SET_LIFETIME(regen, 1500, 2500)
PARTICLE_SET_GRAVITY(regen, 0.0, 50.0)                  ' leichte zusätzliche Beschleunigung
PARTICLE_SET_SIZE(regen, 1, 2)
PARTICLE_SET_FADE(regen, FALSE)

' Im Loop: pro Frame über die Fenster-Breite verteilt emittieren
PARTICLE_SET_POS(regen, RND(SCREEN_W) * 1.0, 0.0)
PARTICLE_EMIT(regen, 2)
```

## Velocity & Gravity

- **Velocity** ist Pixel pro Sekunde — `(-50, 50)` heißt: zufällige horizontale Geschwindigkeit zwischen -50 und 50 px/s.
- **Gravity** ist Pixel pro Sekunde² — `(0, 200)` heißt: nach unten beschleunigend, 200 px/s² (etwa 1/10 Erdbeschleunigung).
- Negative `vy` bedeutet "nach oben" (in Bildschirm-Koordinaten ist Y nach unten positiv).

## Update mit dt

`PARTICLE_UPDATE(sys, dt_ms)` muss jeden Frame aufgerufen werden — die Partikel altern und bewegen sich. `dt_ms` ist die Frame-Zeit (Differenz zur vorigen `MILLIS()`).

```basic
DIM now_ms AS INTEGER
DIM dt AS INTEGER
now_ms = MILLIS()
dt = now_ms - last_ms
last_ms = now_ms

PARTICLE_UPDATE(funken, dt)
```

Wer den Loop pausieren will (z.B. Pause-Menü), ruft `PARTICLE_UPDATE` einfach nicht auf — Partikel "frieren ein".

## Camera-aware

`PARTICLE_DRAW` zeichnet via `CIRCLE` und respektiert daher die Camera (siehe [Camera-Modul](module-camera.md)). World-Koordinaten in `PARTICLE_SET_POS` reichen — Camera kümmert sich um die Screen-Konvertierung.

## Performance (NumPy-vektorisiert)

Position, Velocity, Lifetime, Age, Size und Color werden intern als NumPy-Arrays gehalten. `PARTICLE_UPDATE` ist komplett vektorisiert: Aging, Lifetime-Filter, Gravity-Integration und Position-Update sind je ein einziger Bulk-NumPy-Call. Bei 5000 Partikeln ist `PARTICLE_UPDATE` ca. **70× schneller** als die alte Python-Loop-Version (~0,01 ms statt ~0,55 ms).

`PARTICLE_EMIT` nutzt weiterhin Python-`random` (per-Partikel), damit die Resultate über `RANDOMIZE(seed)` deterministisch reproduzierbar bleiben — beim normalen Use-Case (5–50 emittierte Partikel pro Frame) ist das schnell genug, der Hot-Path ist `PARTICLE_UPDATE` der gleichzeitig mit allen aktiven Partikeln läuft.

`PARTICLE_DRAW` berechnet beim Fade-Effekt die per-Partikel-Farben vorab in einer einzigen vektorisierten Operation und zeichnet dann pro Partikel einen Kreis (nativ in `dhrt`).

Voraussetzung: `numpy` (wird über `pip install numpy` geholt).

## Komplettes Beispiel

- [examples/140_particles.gb](../examples/140_particles.gb) — Konsolen-Logik-Test mit `RANDOMIZE(42)`
- [examples/28_particles_visual.gb](../examples/28_particles_visual.gb) — interaktiv, Funken folgen der Maus
