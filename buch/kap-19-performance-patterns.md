# Kapitel 19 — Performance-Patterns: Bulk-Ops, Atlas, Layers

Star Pilot ist fertig. Du kannst es spielen, du hast Wellen, Boss, Highscore, Menüs. Wenn du aber jetzt ein **größeres** Spiel angehen willst — etwa einen Bullet-Hell-Shooter mit 500 gleichzeitigen Schüssen, oder ein Plattform-Spiel mit gescrolltem Tile-Level — dann reichen die Pattern aus den vorherigen Kapiteln nicht mehr. Naive Loops, die bei 20 Entities flüssig laufen, ruckeln bei 500.

Dieses Kapitel zeigt **drei Patterns**, mit denen du von „Pong-tauglich" zu „Mario/Candy-Crush-tauglich" kommst — alle drei sind bereits in der GameBasic-Engine eingebaut, du musst sie nur nutzen:

1. **Bulk-System-Ops** statt pro-Entity-Loops in BASIC.
2. **Sprite-Atlas + Batch-Draw** statt vieler `DRAWIMAGE`-Aufrufe.
3. **Z-Layer-Rendering** für sauberes Bg/Sprites/UI-Trennen.
4. (Bonus) **`LOAD_ASSETS`** als Workflow-Hygiene.

## Lernziele

Nach diesem Kapitel:

- erkennst du, warum pro-Entity-`ECS_GET_FLOAT`-Loops bei vielen Entities einbrechen
- ersetzt du sie durch `ECS_INTEGRATE_FLOAT`, `ECS_SCALE_FLOAT`, `ECS_CLAMP_FLOAT` und Co.
- lädst du einen Sprite-Atlas mit `ATLAS_LOAD` und renderst hunderte Tiles mit `BATCH_DRAW` + `BATCH_FLUSH`
- strukturierst du dein Rendering mit `LAYER_DEFINE` / `LAYER` in saubere Z-Schichten
- nutzt du `LOAD_ASSETS` für einen zentralen Asset-Preload

## Schritt 1: Warum naive Loops langsam werden

Stell dir den ECS-Movement-Loop aus dem ECS-Kapitel vor — 500 Bullets, die sich pro Frame bewegen:

```basic
' Naive Variante -- N Entities, M Builtin-Calls pro Entity
DIM movers AS ARRAY OF INTEGER
movers = ECS_QUERY2(world, "px", "vx")
DIM i AS INTEGER
FOR i = 0 TO LEN(movers) - 1
    DIM e AS INTEGER
    e = movers[i]
    ECS_ADD_FLOAT(world, e, "px",
                  ECS_GET_FLOAT(world, e, "px")
                  + ECS_GET_FLOAT(world, e, "vx"))
NEXT
```

Sieht harmlos aus. Aber pro Entity sind das **drei Builtin-Calls** (`ECS_GET_FLOAT × 2` + `ECS_ADD_FLOAT × 1`). Bei 500 Entities × 60 fps × 2 Achsen (x und y) sind das

> 500 × 60 × 2 × 3 = **180.000 Builtin-Calls pro Sekunde**.

Jeder Builtin-Call hat einen Fixkostenanteil: Stack-Operationen, Type-Checks, Dict-Lookup. Bei 180k/s summiert sich das zu mehreren Millisekunden pro Frame — und Frame-Time von 16 ms ist das Limit für 60 fps.

Die schlechte Nachricht: optimieren der einzelnen Builtins hilft kaum noch — sie sind so dünn wie möglich. Die gute Nachricht: man muss sie gar nicht so oft aufrufen.

## Schritt 2: Bulk-System-Ops

Statt N Builtin-Calls pro Entity ein Builtin-Call, der **die ganze Schicht** auf einmal macht:

```basic
' Bulk-Variante -- 1 Builtin-Call fuer alle Movers
ECS_INTEGRATE_FLOAT(world, "px", "vx")    ' px += vx fuer alle
ECS_INTEGRATE_FLOAT(world, "py", "vy")    ' py += vy fuer alle
```

Das ist nicht nur **kürzer im Code** — es ist auch **40× schneller**. Der Grund: das Bulk-Builtin läuft intern in einer C-Loop über die Component-Storage, ohne jeden Schleifendurchgang über Python-Dispatch zu gehen.

Verfügbare Bulk-Ops:

| Funktion | Pattern |
|---|---|
| `ECS_INTEGRATE_FLOAT(w, a, b)` | `a += b` (Movement) |
| `ECS_INTEGRATE_INT(w, a, b)` | INT-Variante |
| `ECS_SCALE_FLOAT(w, a, factor)` | `a *= factor` (Friction) |
| `ECS_FILL_FLOAT/INT(w, a, value)` | Reset alle Werte |
| `ECS_CLAMP_FLOAT(w, a, lo, hi)` | Bounds (z.B. Position auf Screen) |
| `ECS_REMOVE_DEAD(w, name, threshold)` | Entities mit `value <= threshold` zerstören |
| `ECS_COUNT_WITH(w, name)` | O(1) — Halter zählen |

### Star Pilot: Bullet-Movement auf Bulk

Im Spiel haben wir bisher Bullets als Klasse mit `update()`-Methode pro Bullet. Wenn wir auf ECS umstellen, sieht der ganze Bullet-Move-System so aus:

```basic
' Pro Frame, EIN Aufruf pro System:
ECS_INTEGRATE_FLOAT(world, "px", "vx")
ECS_INTEGRATE_FLOAT(world, "py", "vy")

' Bullets ausserhalb des Screens loeschen:
ECS_REMOVE_DEAD(world, "px", -10.0)            ' links raus
ECS_REMOVE_DEAD(world, "py", -10.0)            ' oben raus
' (oder mit CLAMP, wenn sie an der Wand abprallen sollen)
```

Das war's. Egal ob 20 Bullets oder 5000 — dieselben 4 Calls. Skaliert.

### Was Bulk-Ops nicht können

Sie sind auf das geschriebene Muster fixiert. Wenn jedes Bullet eine **andere** Velocity-Formel hat (z.B. homing missiles), greift das `ECS_INTEGRATE_FLOAT` nicht — du brauchst pro-Entity-Code. Pragmatischer Rat: **80% der Bullets folgen Standard-Patterns**, schreib die als Bulk. Die spezial-15-Bullets können den Per-Entity-Loop ruhig haben.

## Schritt 3: Sprite-Atlas + Batch-Draw

Bisher haben wir für jedes Sprite (Player, Enemy, Bullet, Explosion-Frame) ein eigenes PNG geladen. Das funktioniert bis ungefähr 20 verschiedene Sprites. Bei 100+ wird's hässlich: 100 LOADIMAGE-Calls verstreut im Code, 100 Dateien im Asset-Ordner.

**Sprite-Atlas** löst das: ein einziges großes PNG enthält **alle** Sprites, ein JSON-Manifest beschreibt die Sub-Rects.

### Manifest

```json
{
  "image": "atlas.png",
  "sprites": {
    "player":      [0,   0, 32, 32],
    "enemy_red":   [32,  0, 32, 32],
    "bullet":      [64,  0,  8,  8],
    "explosion_0": [0,  32, 32, 32],
    "explosion_1": [32, 32, 32, 32]
  }
}
```

Rects sind `[x, y, w, h]` in Pixeln. Das Bild ist relativ zum Manifest.

### Laden und einzeln zeichnen

```basic
DIM atlas AS SPRITE_ATLAS
atlas = ATLAS_LOAD("assets/star_atlas.json")

ATLAS_DRAW(atlas, "player", player_x, player_y)
ATLAS_DRAW(atlas, "bullet", bx, by)
```

Das funktioniert wie `DRAWIMAGEPART` — du gibst aber den **Namen** statt Koordinaten an. Der Atlas erinnert sich an die Rect-Tabelle.

### Der eigentliche Speed: Batch-Draw

Pygame hat einen Mechanismus, **viele Blits auf einmal** zu machen: `Surface.blits([(img, dest, src_rect), ...])`. Das spart den Python-Call-Overhead von N einzelnen Aufrufen. GameBasic exponiert das als `BATCH_DRAW` + `BATCH_FLUSH`:

```basic
' Statt 100 Einzelcalls:
FOR i = 0 TO 99
    ATLAS_DRAW(atlas, "bullet", bx[i], by[i])    ' 100 Builtin-Calls
NEXT

' Mache es einmal:
FOR i = 0 TO 99
    BATCH_DRAW(atlas, "bullet", bx[i], by[i])    ' nur queuen
NEXT
BATCH_FLUSH()    ' EIN pygame-Call fuer alle 100
```

Bei Tilemaps mit 600 Tiles pro Frame ist der Unterschied dramatisch — der Batch-Code rendert das in unter 1 ms, der einzelne Pfad braucht 5–10× so lang.

### Star Pilot: Bullets gebatcht

```basic
DIM bullets AS ARRAY OF INTEGER
bullets = ECS_QUERY(world, "bullet")
DIM i AS INTEGER
FOR i = 0 TO LEN(bullets) - 1
    DIM e AS INTEGER
    e = bullets[i]
    BATCH_DRAW(atlas, "bullet",
               ECS_GET_FLOAT(world, e, "px"),
               ECS_GET_FLOAT(world, e, "py"))
NEXT
BATCH_FLUSH()
```

Wichtig: in dieser Variante haben wir wieder `ECS_GET_FLOAT`-Calls pro Entity. Die sind aber für **Read** nötig (Position lesen für den Draw); für **Update** wäre `INTEGRATE` schneller. Für die letzten 10% holt man später NumPy oder eine Bulk-Draw-API ins Spiel — aber dieser hybrid-Code ist schon dramatisch schneller als der Original-Loop.

### Auto-Flush

`BATCH_FLUSH` muss man nicht immer manuell rufen. Die Engine flusht automatisch bei:
- `FLIP()` (sonst geht der Batch verloren)
- `LAYER(...)`-Wechsel (sonst landet er auf dem falschen Layer)
- jedem nicht-Batch-Draw (`DRAWIMAGE`, `ATLAS_DRAW`, `TEXT`, …)

Du brauchst `BATCH_FLUSH` also nur, wenn du **innerhalb** einer Frame-Phase den Stapel jetzt rendern willst.

## Schritt 4: Z-Layer-Rendering

In Star Pilot zeichnen wir bisher in einer festen Reihenfolge: erst CLS, dann Background-Sterne, dann Player, dann Bullets, dann Enemies, dann UI-Text. Diese Reihenfolge ist **implizit im Code** — wer das Spiel erweitert, muss die Reihenfolge im Kopf behalten oder es gibt Z-Bugs (Bullets vor dem UI-Text, oder Background über Sprites).

**Z-Layer** machen die Reihenfolge **explizit**:

```basic
LAYER_DEFINE("bg",      0)
LAYER_DEFINE("sprites", 10)
LAYER_DEFINE("ui",      100)
```

Jede Zahl ist ein z-Wert. Niedrig = hinten, hoch = vorne. Im Game-Loop:

```basic
LAYER("bg")
CLS(NIGHT_BLUE)
DrawStars()

LAYER("sprites")
DrawPlayer()
DrawEnemies()
DrawBullets()

LAYER("ui")
DRAWTEXT(...)

FLIP()   ' composit + cleart Layer fuer naechsten Frame
```

`FLIP` macht jetzt etwas Neues: es **kombiniert** die Layer in z-Reihenfolge auf den Screen. Die Reihenfolge im Code spielt **keine Rolle** mehr — der bg-Layer ist immer hinten, weil sein z-Wert 0 ist.

### Bonus: Layer haben Alpha

Jeder Layer ist eine `SRCALPHA`-Surface. Wo du nichts zeichnest, ist er **transparent**. So kannst du z.B. einen Effekt-Layer haben, der nur Partikel enthält, und der lässt die Sprites darunter durchscheinen — ohne Setup.

### Camera + Layer

Die Camera ist global, gilt für alle Layer. Wenn du **kein** Camera-Scrolling beim UI-Layer willst (HUD bleibt fix):

```basic
LAYER("ui")
CAMERA_RESET()           ' fuer den UI-Block
DRAWTEXT(10, 10, "Score: " + STR$(score))
' nach LAYER-Wechsel ist Camera automatisch wieder aktiv
```

## Schritt 5: `LOAD_ASSETS` als Workflow

Bei wachsenden Spielen haben wir gerne Asset-Loading-Code an 20 Stellen verstreut:

```basic
hero = LOADIMAGE("assets/hero.png")
' ... 200 Zeilen weiter ...
enemy = LOADIMAGE("assets/enemy_red.png")
' ... wieder 200 Zeilen ...
laser = LOADSOUND("assets/laser.wav")
```

Das ist **brüchig**: wer ein File umbenennt, muss 20 Stellen finden. Wer Loading-Zeit messen will, muss alles zu Beginn machen.

`LOAD_ASSETS(manifest)` lädt alle Assets aus einer JSON-Datei vorab:

```json
{
  "images": {
    "hero":      "sprites/hero.png",
    "enemy_red": "sprites/enemy_red.png",
    "atlas":     "atlas.png"
  },
  "sounds": [
    "audio/laser.wav",
    "audio/boom.wav"
  ]
}
```

Im Spiel:

```basic
SCREEN(640, 480, "Star Pilot")
LOAD_ASSETS("assets/manifest.json")    ' alles vorab im RAM

DIM hero AS IMAGE
hero = LOADIMAGE("hero")               ' Alias-Hit, kein Disk-IO
```

`LOADIMAGE` cacht intern. Sobald `LOAD_ASSETS` durch ist, sind weitere `LOADIMAGE`-Aufrufe **kostenlos**. Plus: wenn du eine Asset-Datei umbenennst, ist eine Stelle im Manifest zu ändern, nicht 20 im Code.

## Schritt 6: Alles zusammen — Bullet-Hell-Skelett

So sähe das Skelett eines Bullet-Hell-Game-Loops mit allen vier Patterns aus:

```basic
IMPORT "ecs"
IMPORT "audio"

SCREEN(640, 480, "Star Pilot Bullets")
LOAD_ASSETS("assets/manifest.json")

DIM atlas AS SPRITE_ATLAS
atlas = ATLAS_LOAD("assets/atlas.json")

LAYER_DEFINE("bg",       0)
LAYER_DEFINE("bullets",  5)
LAYER_DEFINE("sprites",  10)
LAYER_DEFINE("ui",       100)

DIM world AS ECS_WORLD
world = ECS_NEW_WORLD()

' Player + 500 Enemy-Bullets vorbereiten ...

WHILE NOT QUITREQUESTED()
    ' --- UPDATE -----------------------------------------------------
    ' Movement: alle Bullets in einem Call
    ECS_INTEGRATE_FLOAT(world, "px", "vx")
    ECS_INTEGRATE_FLOAT(world, "py", "vy")
    ' Drag/Friction:
    ECS_SCALE_FLOAT(world, "vx", 0.99)
    ECS_SCALE_FLOAT(world, "vy", 0.99)
    ' Bounds:
    ECS_REMOVE_DEAD(world, "py", -20.0)      ' oben raus -> weg
    ECS_REMOVE_DEAD(world, "py",  500.0)     ' unten raus (negativ = unter -500)
    ' HP-Decay (jeder Bullet verliert HP):
    ECS_INTEGRATE_INT(world, "hp", "regen")  ' regen = -1
    ECS_REMOVE_DEAD(world, "hp", 0)

    ' --- DRAW -------------------------------------------------------
    LAYER("bg")
    CLS(NIGHT_BLUE)
    DrawStars()

    LAYER("bullets")
    DIM blist AS ARRAY OF INTEGER
    blist = ECS_QUERY(world, "px")
    DIM i AS INTEGER
    FOR i = 0 TO LEN(blist) - 1
        DIM e AS INTEGER
        e = blist[i]
        BATCH_DRAW(atlas, "bullet",
                   ECS_GET_FLOAT(world, e, "px"),
                   ECS_GET_FLOAT(world, e, "py"))
    NEXT
    ' BATCH_FLUSH wird beim LAYER-Wechsel automatisch aufgerufen

    LAYER("sprites")
    ATLAS_DRAW(atlas, "player", player_x, player_y)

    LAYER("ui")
    DRAWTEXT(10, 10, "Bullets: " + STR$(LEN(blist)))

    FLIP()
WEND
```

Das ist die Skelett-Form eines Bullet-Hell-Loops, der **mehrere tausend Bullets** auf der Native-VM bei stabilen 60 fps schafft. Versuche nicht, das ohne die Patterns nachzubauen — du wirst bei ein paar hundert Bullets schon ruckeln.

## Schritt 7: Anti-Patterns vermeiden

Häufige Fallen:

**1. Pro-Entity-Update-Loop in BASIC mit ECS_GET/ADD.** Wenn du das tippst, frag dich immer: *kann ich das als Bulk-Op ausdrücken?* In 80 % der Fälle ja.

**2. `DRAWIMAGE`-Loop für hunderte gleicher Sprites.** Wenn du dieselbe Surface (oder dasselbe Atlas-Sprite) oft zeichnest, nimm `BATCH_DRAW`. Bei 50+ Sprites pro Frame lohnt es sich.

**3. Z-Bugs durch implizite Render-Reihenfolge.** Wer eine neue Effekt-Schicht (z.B. „Schaden-Flash-Overlay") einbaut, vergisst leicht, ob das vor oder nach dem UI gezeichnet wird. Mit `LAYER`-Trennung gibt es diese Frage gar nicht.

**4. `LOADIMAGE` in der Update-Schleife.** Klassiker. `LOADIMAGE` cacht zwar — aber selbst der Cache-Hit ist nicht kostenlos. Lade Assets **einmal** beim Start, nutze die Referenz.

## Übung

Nimm dir den `playing`-Code aus Kapitel 18 (Star Pilot fertig) und mach folgende Umbauten:

1. Schreibe ein `assets/manifest.json` mit allen Bildern + Sounds, die du bisher in `LOADIMAGE`/`LOADSOUND`-Aufrufen hast. Ersetze die Aufrufe durch `LOAD_ASSETS("assets/manifest.json")` am Anfang + Alias-Reads im Spiel.

2. Lege drei Layer an (`bg`, `sprites`, `ui`) und sortiere alle bisherigen Draw-Calls in einen davon ein. Sieh dir das Spiel an — sieht es genauso aus wie vorher? Soll es.

3. Probiere experimentell: ersetze den Bullet-Update-Loop durch `ECS_INTEGRATE_FLOAT`. Du musst dafür die Bullets in einem `ECS_WORLD` speichern statt als Klasse — bau eine Version, in der Bullets als Entities mit `px/py/vx/vy` existieren und vergleiche die fps mit deiner Klassen-Variante.

Wer Punkt 3 zu groß findet: bau einen kleinen separaten Bullet-Stress-Test (1000 Bullets, sonst nichts) und messe die FPS mit beiden Varianten. Der Unterschied wird drastisch.

## Was du jetzt kannst

Du hast die drei kritischen Patterns für „echtes" 2D-Spiel:

- **ECS Bulk-Ops** statt pro-Entity-BASIC-Loops für alle Standard-Systeme (Movement, Lifecycle, Bounds, Decay).
- **Sprite-Atlas + Batch** für viele gleichartige Sprites (Tiles, Bullets, Particles).
- **Z-Layer** für explizite Render-Reihenfolge ohne Z-Bugs.
- **LOAD_ASSETS** für sauberen Asset-Workflow.

Damit sind die typischen Performance-Klippen zwischen „Pong tut" und „Mario tut" überwunden. Das nächste Kapitel (geplant) zeigt, wie man darüber hinaus mit eigenen Spiel-spezifischen Bulk-Builtins arbeitet — wann lohnt es sich, ein eigenes `ECS_RESOLVE_COLLISIONS(world, ...)` als Builtin in das `ecs_native.pyx` einzubauen, statt es in BASIC zu lassen.
