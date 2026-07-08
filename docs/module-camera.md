# Modul `camera`

Eine globale Kamera für die Spielwelt. Verschiebt und zoomt **alle** Drawing-Befehle, ohne dass du jeden einzelnen Aufruf umschreiben musst.

```basic
IMPORT "camera"
```

## Übersicht

| Funktion | Zweck |
|---|---|
| `CAMERA_SET(x, y[, zoom])` | Camera-Position (Welt-Punkt für Bildschirm-Top-Left) und Zoom |
| `CAMERA_RESET()` | wieder Identität (0, 0, 1.0) |
| `CAMERA_X()`, `CAMERA_Y()`, `CAMERA_ZOOM()` → FLOAT | aktuelle Werte lesen |
| `CAMERA_FOLLOW(target_x, target_y, screen_w, screen_h)` | zentriert Camera auf target |
| `CAMERA_S2W_X(sx)`, `CAMERA_S2W_Y(sy)` → FLOAT | Screen-Pixel zu Welt-Koordinate (z.B. für Maus-Klick) |
| `CAMERA_SHAKE(staerke[, dauer_ms])` | Screen-Shake: zufälliger Kamera-Ruckel (Welt-Pixel), klingt linear über `dauer_ms` ab (Default 300) — läuft selbstständig, kein Pro-Frame-Code. `staerke = 0` stoppt sofort |

## Konzept

Solange Camera Identität ist (`(0, 0, 1.0)`), zeichnet alles wie immer. Sobald `CAMERA_SET` einen Offset oder Zoom setzt, gelten alle Koordinaten in `BOX`, `CIRCLE`, `LINE`, `TEXT`, `DRAWIMAGE`, `DRAWTILEMAP`, `SPRITE_DRAW`, `PARTICLE_DRAW` als **World-Koordinaten** — die Camera projiziert auf den Screen.

```basic
IMPORT "camera"

SCREEN(320, 240, "Camera-Demo", 2)

' Welt-Mitte (240, 160) auf Bildschirm-Mitte
CAMERA_SET(240.0 - 160.0, 160.0 - 120.0)

CLS()
BOX(240, 160, 260, 180, RGB(255, 0, 0))   ' erscheint im Bildschirm-Zentrum
FLIP()
SLEEP(2000)
```

## Standard-Pattern: Camera folgt Held

```basic
IMPORT "camera"
IMPORT "sprite"

CONST W AS INTEGER = 320
CONST H AS INTEGER = 240
SCREEN(W, H, "Held-Folger", 2)

DIM held AS SPRITE
held = SPRITE_NEW(LOADIMAGE("assets/hero.png"), 16, 16)

WHILE NOT QUITREQUESTED()
    ' Eingabe + Update ...

    ' Camera zentriert auf Held
    CAMERA_FOLLOW(SPRITE_GET_X(held) + 8.0, SPRITE_GET_Y(held) + 8.0, W * 1.0, H * 1.0)

    CLS(RGB(20, 25, 40))

    ' Welt zeichnen (in World-Koordinaten)
    DIM r AS INTEGER
    DIM c AS INTEGER
    FOR r = 0 TO 9
        FOR c = 0 TO 14
            BOX(c * 32, r * 32, c * 32 + 31, r * 32 + 31, RGB(60, 50, 80))
        NEXT
    NEXT

    SPRITE_DRAW(held)

    ' HUD im Screen-Space (Camera tempordr aus)
    CAMERA_RESET()
    TEXT(4, 4, "HP: 100", RGB(255, 255, 255))

    FLIP()
    SLEEP(16)
WEND
```

`CAMERA_FOLLOW(target_x, target_y, screen_w, screen_h)` setzt die Camera so, dass `(target_x, target_y)` im Bildschirm-Zentrum landet. Hat den Zoom unverändert.

## Zoom

```basic
CAMERA_SET(0.0, 0.0, 2.0)        ' alles doppelt so groß
CAMERA_SET(0.0, 0.0, 0.5)        ' alles halb so groß (mehr Welt sichtbar)
```

- Bei `zoom = 2.0` wird **jedes** Drawing — Linien, Boxen, **und Bilder** — verdoppelt gezeichnet.
- Bei zoom != 1 werden Bilder jeden Frame neu skaliert gezeichnet; das kostet etwas Performance, ist aber meist OK.
- **Text wird NICHT gezoomt** (nur translatiert). Sonst würde Text bei Zoom > 1 verschwommen werden. Wer großen Text will, nutzt entweder eine größere Schrift oder rendert vorab als Bild.

## HUD im Screen-Space

Wenn die Camera aktiv ist und du HUD-Text im Bildschirm-Koordinaten zeichnen willst:

```basic
' während des Game-Frame:
CAMERA_FOLLOW(spieler_x, spieler_y, W, H)
CLS()
' ... Welt zeichnen (World-Coords) ...

CAMERA_RESET()
' ... HUD zeichnen (Screen-Coords) ...
TEXT(4, 4, "Score: " + STR$(score), RGB(255, 255, 255))

FLIP()
```

`CAMERA_RESET()` ist billig — keine Sorge wegen Performance bei mehrfachem Aufruf pro Frame.

## Maus-Klick → Welt-Position

Wenn der User irgendwo klickt und du wissen willst, **welche Welt-Koordinate** das ist:

```basic
IF MOUSEBUTTON(0) THEN
    DIM wx AS FLOAT
    DIM wy AS FLOAT
    wx = CAMERA_S2W_X(MOUSEX() * 1.0)
    wy = CAMERA_S2W_Y(MOUSEY() * 1.0)
    PRINT "Klick in Welt: ", wx, wy
END IF
```

Bei Identitäts-Camera gibt das einfach die Maus-Position zurück; bei verschobener oder gezoomter Camera die korrekte Welt-Koordinate.

## Camera-Verhalten in BOX/CIRCLE/LINE

Die Größen werden mit dem Zoom skaliert:

| Aufruf | bei Zoom 1.0 | bei Zoom 2.0 |
|---|---|---|
| `CIRCLE(100, 100, 10, …)` | Kreis Radius 10 bei (100, 100) | Kreis Radius 20 bei (200, 200) |
| `BOX(0, 0, 100, 100, …)` | 100×100 Pixel | 200×200 Pixel |
| `LINE(0, 0, 100, 0, …)` | 100 Pixel lang | 200 Pixel lang (Linien-Dicke bleibt 1) |

Die Camera versteht "Welt-Koordinaten" als **Top-Left**: `CAMERA_SET(x, y, ...)` heißt: der Welt-Punkt `(x, y)` landet bei Bildschirm-`(0, 0)`. Das ist konsistent mit `DRAWTILEMAP(sx, sy)`-Konventionen.

## Beispiel: Scrollender Welt-Editor

```basic
IMPORT "camera"

SCREEN(320, 240, "Scroller", 2)

DIM cam_x AS FLOAT
DIM cam_y AS FLOAT
DIM zoom AS FLOAT
cam_x = 0.0
cam_y = 0.0
zoom = 1.0

WHILE NOT QUITREQUESTED()
    IF KEYPRESSED(KEY_LEFT) THEN
        cam_x = cam_x - 4.0
    END IF
    IF KEYPRESSED(KEY_RIGHT) THEN
        cam_x = cam_x + 4.0
    END IF
    IF KEYPRESSED(KEY_UP) THEN
        cam_y = cam_y - 4.0
    END IF
    IF KEYPRESSED(KEY_DOWN) THEN
        cam_y = cam_y + 4.0
    END IF
    IF KEYPRESSED(43) THEN     ' +
        zoom = zoom * 1.05
    END IF
    IF KEYPRESSED(45) THEN     ' -
        zoom = zoom / 1.05
        IF zoom < 0.2 THEN
            zoom = 0.2
        END IF
    END IF

    CAMERA_SET(cam_x, cam_y, zoom)

    CLS(RGB(20, 20, 35))

    ' grosse Welt
    DIM r AS INTEGER
    DIM c AS INTEGER
    FOR r = 0 TO 19
        FOR c = 0 TO 29
            DIM color AS INTEGER
            IF (r + c) MOD 2 = 0 THEN
                color = RGB(60, 80, 120)
            ELSE
                color = RGB(80, 120, 60)
            END IF
            BOX(c * 32, r * 32, c * 32 + 31, r * 32 + 31, color)
        NEXT
    NEXT

    ' HUD
    CAMERA_RESET()
    TEXT(4, 4, "cam=(" + STR$(ROUND(cam_x)) + "," + STR$(ROUND(cam_y)) + ") zoom=" + STR$(zoom), RGB(255, 255, 255))

    FLIP()
    SLEEP(16)
WEND
```

## Komplettes Beispiel

- [examples/29_camera.gb](../examples/29_camera.gb) — Logik-Test ohne Grafik-Fenster (S2W-Konvertierung, FOLLOW)
- [examples/141_camera_visual.gb](../examples/141_camera_visual.gb) — interaktiv mit Pfeilen + Zoom

## Tipp: Camera-Push/Pop fehlt

Es gibt **keinen** Camera-Stack (kein `CAMERA_PUSH/POP`). Wenn du HUD über bewegtem Hintergrund zeichnen willst, ist das Pattern:

```basic
CAMERA_SET(welt_x, welt_y, zoom)
' Welt zeichnen ...

CAMERA_RESET()
' HUD zeichnen ...
```

Wer wirklich verschachtelte Cameras braucht (z.B. Mini-Map): die aktuellen Werte mit `CAMERA_X/Y/ZOOM()` lesen, `CAMERA_SET` für die andere Camera, dann mit `CAMERA_SET(alt_x, alt_y, alt_zoom)` zurückwechseln.
