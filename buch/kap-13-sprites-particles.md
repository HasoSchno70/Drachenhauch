# Kapitel 13 — Sprites und Particles

In Kapitel 12 hatten wir alles, was ein Spiel mechanisch braucht — Player, Bullets, Wellen, Score, Game Over. Aber visuell ist Star Pilot noch weit von „spektakulär" entfernt. Drei Boxen-Farben gegen einen dunkelblauen Hintergrund. Wenn ein Gegner stirbt, *poppt* er einfach weg. Wenn der Player fliegt, ist da kein Triebwerks-Feuer.

Dieses Kapitel ändert das. Wir bringen das **particles**-Modul ins Spiel — kleine Funken-Wolken, die bei Treffern explodieren und unter dem Player wie ein Triebwerks-Strahl pulsieren. Das ist der Moment, in dem aus einem „funktionalen Spiel" ein **lebendiges** Spiel wird.

Zum Thema **Sprites** — den schönen Pixel-Art-Schiffen statt langweiligen Kästchen — schreibe ich eine kompakte Erklärung am Ende. Sprites brauchen externe Pixel-Art (PNG-Dateien), die ich hier nicht beilegen kann. Wer Pixel-Art zeichnen kann oder mag, findet den Übergang in Übung 4 als klar abgesteckten Stretch.

## Lernziele

Nach diesem Kapitel:

- aktivierst du das `particles`-Modul mit `IMPORT "particles"`
- konfigurierst du einen Emitter mit Velocity, Lifetime, Color, Size, Fade
- triggerst du Bursts mit `PARTICLE_EMIT` an einer Position
- kombinierst du **mehrere Emitter** im Spiel (Explosion + Trail)
- weißt du, wie das `sprite`-Modul funktioniert und wo es im Spiel hinkommt

## Das Konzept Particle

Ein **Particle** ist ein winziger Punkt mit eigener Position, Geschwindigkeit und Lebensdauer. Allein ist das langweilig — wirken tut es erst in der Masse. Dreißig Particles, die in eine Richtung losfliegen, langsamer werden und ausblenden — das ist eine Explosion. Zwei Particles pro Frame nach unten — das ist ein Triebwerks-Trail.

Ein **Emitter** (das `PARTICLE_SYSTEM`-Objekt) ist die Quelle. Er hält die Konfiguration (welche Farbe? wie schnell? wie lang?) und die Liste aller noch lebenden Partikel. Pro Frame:

1. Eventuell neue Partikel emittieren (`PARTICLE_EMIT`)
2. Alle vorhandenen altern lassen (`PARTICLE_UPDATE`)
3. Sie zeichnen (`PARTICLE_DRAW`)

Tote Partikel verschwinden automatisch. Du musst sie nicht selbst aufräumen.

## Schritt 1: Erste Explosion

Bringen wir Particles ins Bild. Eine Datei, ein Emitter, alle 60 Frames eine Explosion in der Bildschirmmitte:

```basic
IMPORT "particles"

CONST WIDTH    AS INTEGER = 320
CONST HEIGHT   AS INTEGER = 240
CONST BG_COLOR AS INTEGER = &H141E3C

DIM emitter AS PARTICLE_SYSTEM
emitter = PARTICLE_SYSTEM_NEW(WIDTH / 2, HEIGHT / 2)

PARTICLE_SET_VELOCITY(emitter, -80, 80, -80, 80)
PARTICLE_SET_LIFETIME(emitter, 400, 800)
PARTICLE_SET_COLOR(emitter, &HFFAA00)
PARTICLE_SET_SIZE(emitter, 1, 3)
PARTICLE_SET_FADE(emitter, TRUE)

DIM tick AS INTEGER
tick = 0

SCREEN(WIDTH, HEIGHT, "Particle Demo", 2)

WHILE NOT QUITREQUESTED()
    tick = tick + 1
    IF tick MOD 60 = 0 THEN
        PARTICLE_EMIT(emitter, 30)
    END IF
    PARTICLE_UPDATE(emitter, 16)

    CLS(BG_COLOR)
    PARTICLE_DRAW(emitter)
    FLIP()
    SLEEP(16)
WEND
```

Run drücken. Du solltest jede Sekunde eine kleine orange Wolke in der Mitte aufpoppen sehen — sie expandiert kurz, blendet aus, ist weg.

### Die Konfiguration im Detail

| Aufruf | Bedeutung |
|---|---|
| `PARTICLE_SYSTEM_NEW(x, y)` | Erstellt einen Emitter an Position `(x, y)`. Die Position kannst du jederzeit mit `PARTICLE_SET_POS` ändern. |
| `PARTICLE_SET_VELOCITY(s, vx_min, vx_max, vy_min, vy_max)` | Geschwindigkeits-Range pro neuem Partikel. Pro Achse zwischen `min` und `max` zufällig. Werte sind in **Pixel pro Sekunde**. `(-80, 80, -80, 80)` heißt: rundum, bis 80 px/s in jede Richtung. |
| `PARTICLE_SET_LIFETIME(s, ms_min, ms_max)` | Lebensdauer in Millisekunden. `(400, 800)` heißt: jeder Partikel lebt 0.4–0.8 Sekunden. |
| `PARTICLE_SET_COLOR(s, color)` | Farbe als Hex (`&HFFAA00` = warmes Orange). |
| `PARTICLE_SET_SIZE(s, smin, smax)` | Pixel-Größe pro Partikel. `(1, 3)` heißt: kleine bis mittlere Punkte. |
| `PARTICLE_SET_FADE(s, TRUE)` | Wenn `TRUE`, blendet jeder Partikel über seine Lebensdauer aus. Sieht meistens besser aus als hartes Verschwinden. |
| `PARTICLE_EMIT(s, count)` | Spawnt `count` neue Partikel an der aktuellen Emitter-Position. |
| `PARTICLE_UPDATE(s, dt_ms)` | Lässt alle Partikel um `dt_ms` Millisekunden altern. Bei 60 FPS: `16`. |
| `PARTICLE_DRAW(s)` | Zeichnet alle lebenden Partikel. |

> **Tipp**: spiel mit den Werten. Höhere Velocity → spritzig-dramatische Explosion. Längere Lifetime → trägere, „rauchigere" Wolke. `SET_GRAVITY(s, gx, gy)` zieht Partikel nach unten — gut für „abfallende Funken".

## Schritt 2: Player-Trail

Eine zweite Form von Emitter: kontinuierlich, statt in Bursts. Pro Frame ein paar Partikel unter dem Player, leichte Streuung nach unten — das wirkt wie ein Triebwerks-Strahl.

```basic
DIM trail AS PARTICLE_SYSTEM
trail = PARTICLE_SYSTEM_NEW(player_x, player_y)
PARTICLE_SET_VELOCITY(trail, -10, 10, 30, 60)    ' nach unten, leichte Streuung
PARTICLE_SET_LIFETIME(trail, 200, 400)
PARTICLE_SET_COLOR(trail, &HFF8800)
PARTICLE_SET_SIZE(trail, 1, 2)
PARTICLE_SET_FADE(trail, TRUE)
```

Im Game-Loop:

```basic
PARTICLE_SET_POS(trail, player_x + 20, player_y + 24)    ' unter dem Schiff
PARTICLE_EMIT(trail, 2)                                   ' jeden Frame 2 Partikel
PARTICLE_UPDATE(trail, 16)
```

Mit `PARTICLE_SET_POS` verschieben wir die Quelle pro Frame — die Partikel selbst behalten ihre eigene Bewegung, aber neue spawnen jetzt an der aktuellen Player-Position. So entsteht eine Spur, die dem Player folgt.

### Velocity nach unten verstehen

Die `(-10, 10, 30, 60)`-Werte sind ein hübsches Beispiel:

- **X-Velocity**: `-10` bis `+10` — fast keine seitliche Bewegung, leichte Streuung
- **Y-Velocity**: `30` bis `60` — alle nach unten (Y wächst nach unten in der Bildschirm-Koordinaten-Welt)

Der Player fliegt nach **oben** durchs Bild — sein Trail soll nach **hinten** fliegen, also relativ zum Schiff nach unten. Das ergibt das natürliche Triebwerks-Gefühl.

## Schritt 3: Im Spiel — zwei Emitter parallel

Jetzt kombinieren wir beides im echten Star Pilot. Wir brauchen **zwei** Emitter:

- `explosion_fx` — wird bei Enemy-Tod und Player-Hit getriggert
- `trail_fx` — kontinuierlich unter dem Player

Beide werden in `Setup()` einmal konfiguriert. Pro Frame: `trail_fx` emittiert, `explosion_fx` wird nur bei Treffern getriggert.

Die Setup-Routine:

```basic
explosion_fx = PARTICLE_SYSTEM_NEW(0, 0)
PARTICLE_SET_VELOCITY(explosion_fx, -120, 120, -120, 120)
PARTICLE_SET_LIFETIME(explosion_fx, 300, 600)
PARTICLE_SET_COLOR(explosion_fx, &HFFAA00)
PARTICLE_SET_SIZE(explosion_fx, 1, 3)
PARTICLE_SET_FADE(explosion_fx, TRUE)

trail_fx = PARTICLE_SYSTEM_NEW(0, 0)
PARTICLE_SET_VELOCITY(trail_fx, -10, 10, 30, 60)
PARTICLE_SET_LIFETIME(trail_fx, 200, 400)
PARTICLE_SET_COLOR(trail_fx, &HFF8800)
PARTICLE_SET_SIZE(trail_fx, 1, 2)
PARTICLE_SET_FADE(trail_fx, TRUE)
```

Position `(0, 0)` ist Platzhalter — wir setzen sie pro Emit/pro Frame neu.

### Helper für Explosionen

Die Geste „setze Position, dann emittiere" ist so häufig, dass wir sie in eine Sub packen:

```basic
SUB Explode(cx AS INTEGER, cy AS INTEGER, count AS INTEGER)
    PARTICLE_SET_POS(explosion_fx, cx, cy)
    PARTICLE_EMIT(explosion_fx, count)
END SUB
```

Im Treffer-Code:

```basic
IF ... PHYSICS_BOX_BOX(...) ... THEN
    bullets[i].alive = FALSE
    grunts[j].alive  = FALSE
    score = score + GRUNT_PTS
    Explode(grunts[j].x + grunts[j].w / 2, grunts[j].y + grunts[j].h / 2, 25)
END IF
```

Wir explodieren in der **Mitte** des Gegners, nicht an seiner Ecke. `25` Partikel für Grunts, `40` für Bombers (die sind größer und sollen "fetter" sterben), `50` für den Player-Tod.

### Trail im UpdatePlaying

Im `UpdatePlaying` einfach diese zwei Zeilen vor allem anderen:

```basic
PARTICLE_SET_POS(trail_fx, player.x + player.w / 2, player.y + player.h)
PARTICLE_EMIT(trail_fx, 2)
```

Position auf die Mitte unter dem Player, zwei Partikel emittieren. So einfach.

## Schritt 4: Update und Draw

Particles sind globaler State und müssen **immer** weiterlaufen, auch wenn das Spiel im Wave-Intro oder Game-Over-Modus ist. Sonst frieren noch fliegende Funken ein:

```basic
WHILE NOT QUITREQUESTED()
    SELECT CASE state
        CASE GameState.PLAYING
            UpdatePlaying()
        CASE GameState.WAVE_INTRO
            UpdateWaveIntro()
        CASE GameState.GAMEOVER
            ' Eingefroren
    END SELECT

    ' Particles laufen IMMER weiter
    PARTICLE_UPDATE(explosion_fx, 16)
    PARTICLE_UPDATE(trail_fx, 16)

    DrawAll()
    SLEEP(16)
WEND
```

In `DrawAll`: Trail **unter** dem Player zeichnen (sonst überdeckt das Schiff seinen eigenen Trail nicht), Explosionen **über** allem (sollen im Vordergrund sein):

```basic
SUB DrawAll()
    CLS(BG_COLOR)

    PARTICLE_DRAW(trail_fx)         ' zuerst, im Hintergrund

    ' ... Bullets, Enemies, Player ...

    PARTICLE_DRAW(explosion_fx)     ' zuletzt, vorne
    ' ... HUD-Text ...
END SUB
```

> **Wichtig**: in 2D-Spielen bestimmt die **Reihenfolge der Draw-Aufrufe**, was vorne ist. Was du zuletzt zeichnest, liegt obenauf. Anders als in 3D, wo's einen Z-Buffer gäbe.

## Star Pilot mit Effekten

Der vollständige Code in [`code/kap-13/main.gb`](code/kap-13/main.gb). Run drücken — und du solltest sehen:

- **Trail** unter dem Player: orangene Funken-Wolke, die mit dir fliegt
- **Explosionen** wenn du einen Grunt triffst (kleine Wolke), einen Bomber (größere Wolke), oder einen Treffer abkriegst (große Wolke)
- Alles **lebendig**, nicht mehr statisch

Vergleich vorher/nachher: schalt die `PARTICLE_DRAW`-Aufrufe versuchsweise mal aus (ein `IF FALSE THEN ...` drumherum). Du wirst spüren, wie viel die Particles ausmachen — sie sind keine Deko, sie sind **das Spielgefühl**.

## Sprites — das nächste Level

Das `sprite`-Modul (`IMPORT "sprite"`) ersetzt die langweiligen `BOX(...)`-Aufrufe durch echte Pixel-Art mit Animation. Der typische Workflow:

```basic
IMPORT "sprite"

' Pixel-Art-Sheet laden (PNG mit 4 nebeneinander liegenden 16x16-Frames)
DIM sheet AS IMAGE
sheet = LOADIMAGE("assets/sprites/grunt.png")

' Sprite-Objekt aus dem Sheet, Frame-Größe 16x16
DIM grunt_sprite AS SPRITE
grunt_sprite = SPRITE_NEW(sheet, 16, 16)

' Eine Animation namens "fly" definieren: Frames 0..3, mit 8 FPS
SPRITE_ADD_ANIMATION(grunt_sprite, "fly", 0, 3, 8)
SPRITE_PLAY(grunt_sprite, "fly")

' Im Game-Loop pro Frame:
SPRITE_SET_POS(grunt_sprite, grunt.x, grunt.y)
SPRITE_UPDATE(grunt_sprite, 16)
SPRITE_DRAW(grunt_sprite)
```

Das ersetzt im Wesentlichen `BOX(x, y, ...)` durch ein animiertes Pixel-Schiff.

**Was du brauchst**: ein Sprite-Sheet pro Enemy-Typ — eine PNG-Datei mit 2–4 Frames nebeneinander, gleicher Größe. Die kannst du selbst zeichnen mit:

- **Aseprite** — kostenpflichtig, aber Standard für Pixel-Art (~ 20€)
- **Piskel** — kostenlos, läuft im Browser ([piskelapp.com](https://www.piskelapp.com))
- **GIMP** oder **Photoshop** mit Pixel-Modus

Für unser Star Pilot würden wir typischerweise drei PNGs erzeugen:

| Datei | Frames | Größe | Inhalt |
|---|---|---|---|
| `player.png` | 1 (statisch) | 40×24 | das Spielerschiff |
| `grunt.png` | 4 (Animation) | 16×14 (× 4 = 64×14) | rote Sub-Drohne mit Flackerlicht |
| `bomber.png` | 4 (Animation) | 22×18 (× 4 = 88×18) | breitere orange Drohne |

Wenn du das selber machen willst: in der [Modul-Doku zu sprite](../docs/module-sprite.md) findest du alle Funktions-Signaturen. In Übung 4 unten skizziere ich die Stellen, die du im Hauptcode ändern musst.

> **Warum nicht im Buch beigelegt**: Pixel-Art ist Handarbeit, und wenn ich generische Test-Sprites beilege, werden sie hässlich, lieblos und stilbruchig wirken. Das Spiel **ohne** Sprites ist ehrlich-funktional. Mit deinen eigenen Sprites wird es deins.

## Übungen

**1. Farbige Explosionen.** Erweitere `Explode` um einen `farbe`-Parameter. Bei Grunt-Tod: orange (`&HFFAA00`); bei Bomber-Tod: rot (`&HFF4422`); bei Player-Tod: weiß. Hinweis: du brauchst dafür **drei separate Emitter** (oder du `SET_COLOR`-st den Emitter vor jedem Emit um — auch ok).

**2. Gravitation in der Explosion.** Probier `PARTICLE_SET_GRAVITY(explosion_fx, 0, 200)` aus. Was ändert sich? Die Funken fallen nach unten statt geradlinig zu schweben — fühlt sich „realistischer" an, ist aber auch eine stilistische Frage.

**3. Bullet-Trails.** Gib jedem fliegenden Bullet eine kleine Spur — pro Frame ein einzelner Partikel weiß-leuchtend hinter dem Bullet. Hinweis: einen dritten Emitter `bullet_trail_fx` einrichten (kurze Lifetime, fast keine Velocity, weiße Farbe, fade aktiviert). Im `UpdateBullets`-Loop: für jeden lebenden Bullet `SET_POS` + `EMIT(1)`.

**4. Stretch — Sprites einbauen.** Beschaffe oder zeichne dir drei Sprite-Sheets (Player, Grunt, Bomber). Lege sie unter `assets/sprites/` ab. In den Klassen `Player`, `Grunt`, `Bomber`: füge ein Feld `sprite AS SPRITE` hinzu, lade es in `Init` bzw. `Spawn`, und ersetze in `Draw` den `BOX(...)`-Aufruf durch `SPRITE_DRAW(sprite)`. Das ist viel Tipparbeit, aber kein Konzept-Sprung — alles, was wir gelernt haben, reicht aus.

## Zusammenfassung

Du hast in diesem Kapitel:

- das `particles`-Modul mit `IMPORT "particles"` aktiviert,
- einen Emitter konfiguriert (Velocity, Lifetime, Color, Size, Fade),
- zwei Emitter parallel benutzt — Burst (Explosion) und kontinuierlich (Trail),
- die Reihenfolge im `DrawAll` strukturiert (Trail unten, Player drauf, Explosionen oben),
- `PARTICLE_UPDATE` außerhalb von `UpdatePlaying` aufgerufen, damit Partikel auch im Wave-Intro nicht einfrieren,
- konzeptionell verstanden, wie das `sprite`-Modul funktioniert (mit konkretem Pfad zum Einsatz in Übung 4).

Im **nächsten Kapitel** wird die Bewegung der Gegner natürlicher: statt geradlinig oder einfach-zickzackig kommen sie mit dem **`tween`-Modul** in Formation reingeflogen, schwingen sanft und bekommen Pop-In-Effekte. Das ist der eigentliche Galaga-Charme.

## Code-Stand am Ende des Kapitels

- [`code/kap-13/01_particles_intro.gb`](code/kap-13/01_particles_intro.gb) — Auto-Explosion alle 60 Frames
- [`code/kap-13/02_player_trail.gb`](code/kap-13/02_player_trail.gb) — Player mit kontinuierlichem Triebwerks-Trail
- [`code/kap-13/main.gb`](code/kap-13/main.gb) — Star Pilot mit beidem: Explosionen bei Treffern + Player-Trail
