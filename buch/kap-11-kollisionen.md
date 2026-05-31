# Kapitel 11 — Kollisionen mit dem physics-Modul

Am Ende von Kapitel 10 hatten wir alles, was ein Spiel ausmacht — Player, Bullets, Gegner — *außer* dem entscheidenden Detail: die Bullets fliegen einfach durch die Gegner durch. Die Gegner ignorieren den Player. Es passiert *nichts*.

In diesem Kapitel schließen wir den Kreis. Wir bringen den **physics-Modul** ins Spiel und damit die Kollisions-Erkennung. Bullet trifft Enemy → Enemy stirbt, Score steigt. Enemy trifft Player → Lives sinken. Lives auf 0 → Game Over.

## Lernziele

Nach diesem Kapitel:

- aktivierst du ein eingebautes Modul mit `IMPORT "physics"`
- testest du Rechteck-Überlappungen mit `PHYSICS_BOX_BOX`
- baust du eine Kollisions-Erkennung im Spiel-Loop ein
- verwaltest du Score und Leben sauber
- zeigst du Status (Score, Lives) als Text im Spielfenster mit `TEXT(...)`
- beendest das Spiel beim Game Over

## Schritt 1: Was ist eine Kollision?

In unserem Spiel ist alles ein Rechteck — Player, Enemies, Bullets. Eine Kollision heißt: **zwei Rechtecke überlappen sich**. Im Fachjargon: **AABB-Kollision** (Axis-Aligned Bounding Box). „Axis-aligned" bedeutet, die Boxen sind nicht gedreht — Kanten liegen parallel zu X- und Y-Achse. Das ist der einfachste und schnellste Test.

Mathematisch: zwei Rechtecke A und B überlappen sich genau dann, wenn auf **beiden** Achsen ihre Bereiche sich überschneiden:

- Auf der X-Achse: `A_links < B_rechts` UND `A_rechts > B_links`
- Auf der Y-Achse: `A_oben < B_unten` UND `A_unten > B_oben`

Beides zusammen → Treffer. Wir könnten das selbst schreiben — vier Vergleiche, ein `AND`. Aber GameBasic hat ein Modul dafür, das wir gleich nutzen.

## Schritt 2: Module einbinden

Bislang waren alle Built-ins (Math, Strings, Grafik) automatisch verfügbar. Module sind anders — sie müssen explizit aktiviert werden:

```basic
IMPORT "physics"
```

Diese Zeile gehört ganz an den Anfang deiner Datei, vor allen `CLASS`- und `DIM`-Deklarationen. Ab da kannst du die Funktionen aus `physics` benutzen — `PHYSICS_BOX_BOX`, `PHYSICS_DISTANCE`, `PHYSICS_RAY_BOX` und mehr.

Andere Module funktionieren genauso: `IMPORT "tween"` für Animationen (kommt in Kap 14), `IMPORT "scene"` für Scene-Management (Kap 15), `IMPORT "save"` für Highscores (Kap 16). Der Mechanismus ist immer identisch.

## Schritt 3: PHYSICS_BOX_BOX

Die Funktion sieht so aus:

```
PHYSICS_BOX_BOX(x1, y1, w1, h1, x2, y2, w2, h2) -> BOOLEAN
```

Acht Argumente — Position und Größe von Box 1, dann von Box 2. Liefert `TRUE` bei Überlappung, sonst `FALSE`. Ein kleines Beispiel zum Vertrautmachen, ganz ohne Spiel:

```basic
IMPORT "physics"

DIM bx  AS INTEGER
DIM hit AS BOOLEAN

FOR bx = 0 TO 100 STEP 10
    hit = PHYSICS_BOX_BOX(50, 50, 20, 20, bx, 50, 20, 20)
    PRINT f"  bx={bx}: hit = {hit}"
NEXT bx
```

Box A liegt fest bei `(50, 50)` mit Größe `20×20`. Box B verschieben wir nach rechts. Output:

```
bx=0: hit = FALSE
bx=10: hit = FALSE
bx=20: hit = FALSE
bx=30: hit = FALSE
bx=40: hit = TRUE
bx=50: hit = TRUE
bx=60: hit = TRUE
bx=70: hit = FALSE
...
```

A reicht von `x=50` bis `x=70`. Bei `bx=30` hört B bei `x=50` auf — Kanten berühren sich, aber keine echte Überlappung → kein Treffer. Erst ab `bx=31` ragt B in A hinein. Bei `bx=70` beginnt B genau wo A endet → wieder kein Treffer.

> **Merksatz**: AABB ist *strikt* — nur echte Überlappung zählt, sich-berühren reicht nicht. Bei vielen Spielen ist das exakt was du willst.

## Schritt 4: Bullet × Enemy — die doppelte Schleife

Im Spiel haben wir bis zu 20 Bullets und bis zu 15 Grunts (plus 15 Bomber). Pro Frame müssen wir **jeden lebenden Bullet gegen jeden lebenden Enemy** testen. Das geht mit zwei verschachtelten Schleifen:

```basic
FOR i = 0 TO BULLET_POOL - 1
    IF NOT bullets[i].alive THEN CONTINUE

    FOR j = 0 TO ENEMY_POOL - 1
        IF grunts[j].alive THEN
            IF PHYSICS_BOX_BOX(bullets[i].x, bullets[i].y, bullets[i].w, bullets[i].h, _
                               grunts[j].x,  grunts[j].y,  grunts[j].w,  grunts[j].h) THEN
                bullets[i].alive = FALSE
                grunts[j].alive  = FALSE
                score = score + GRUNT_PTS
            END IF
        END IF
    NEXT j
NEXT i
```

Das Schema:

1. **Bullet überspringen** wenn er tot ist (`CONTINUE`).
2. **Über alle Enemies iterieren**, jeweils prüfen ob lebt und ob `PHYSICS_BOX_BOX` einen Treffer meldet.
3. **Bei Treffer**: beide auf `alive = FALSE`, Score erhöhen.

> **Aha-Moment**: warum `bullets[i].alive = FALSE` *nach* dem Treffer? Damit ein Bullet nur **einen** Enemy trifft — sonst würde derselbe Bullet im selben Frame gleich mehrere Gegner aushebeln. Realistisch ist „ein Schuss, ein Kill" — und das Pattern hier setzt genau das um.

> **Tipp**: das `_` am Zeilenende ist Zeilenfortsetzung — der lange `PHYSICS_BOX_BOX`-Aufruf wird sonst zu unleserlich.

### Warum zwei mal: gegen Grunts und gegen Bombers

Wir haben zwei separate Enemy-Pools. Also zwei Schleifen pro Bullet — einmal gegen Grunts, einmal gegen Bombers. Im Code prüfen wir nach der Grunt-Schleife noch mal, ob der Bullet noch lebt (er könnte einen Grunt getroffen haben):

```basic
IF NOT bullets[i].alive THEN CONTINUE       ' weiter mit naechstem Bullet

' Gegen Bombers pruefen
FOR j = 0 TO ENEMY_POOL - 1
    IF bombers[j].alive THEN
        ' ... gleicher Test mit Bomber-Werten ...
    END IF
NEXT j
```

Eleganter wäre **ein einziger** Pool aller Enemies — dafür müssten wir aber Polymorphie (gemischte Enemy-Typen in einer Liste) so verwenden, dass Update- und Draw-Aufrufe richtig dispatchen. GB unterstützt das beim Tree-Walker im Prinzip (wir sehen das ein bisschen in Kap 12), aber die zwei separaten Pools sind hier konkreter und leichter nachvollziehbar.

## Schritt 5: Player × Enemy

Genauso, nur ohne äußere Schleife — es gibt nur einen Player:

```basic
FOR j = 0 TO ENEMY_POOL - 1
    IF grunts[j].alive THEN
        IF PHYSICS_BOX_BOX(player.x, player.y, player.w, player.h, _
                           grunts[j].x, grunts[j].y, grunts[j].w, grunts[j].h) THEN
            grunts[j].alive = FALSE
            player.TakeHit()
        END IF
    END IF
    ' ... gleiche Logik fuer bombers[j] ...
NEXT j
```

Bei jedem Player-Treffer:

- Enemy stirbt (er hat sich am Player „zerschmettert")
- Player verliert ein Leben

Die `TakeHit()`-Methode (neu in `Player`) reduziert `lives` und setzt bei 0 das `alive`-Flag auf FALSE:

```basic
SUB TakeHit()
    lives = lives - 1
    IF lives <= 0 THEN alive = FALSE
END SUB
```

## Schritt 6: Game Over

Wenn der Player nicht mehr lebt, wollen wir das Spiel anhalten — aber das Fenster offen lassen, damit der Spieler die Game-Over-Anzeige sieht. Dafür eine globale `game_over`-Flag:

```basic
IF NOT player.alive THEN
    game_over = TRUE
END IF
```

Und in der Hauptschleife: bei `game_over = TRUE` rufen wir `UpdateAll()` nicht mehr auf — die Welt friert ein. `DrawAll()` läuft weiter, damit das letzte Bild + die Game-Over-Anzeige gerendert werden:

```basic
WHILE NOT QUITREQUESTED()
    IF NOT game_over THEN
        UpdateAll()
    END IF

    CLS(BG_COLOR)
    DrawAll()
    FLIP()
    SLEEP(16)
WEND
```

In Kap 15 lernen wir mit dem `scene`-Modul eine sauberere Lösung kennen — Game-Over wird dann seine eigene Scene mit eigener Logik. Für jetzt reicht uns das Flag.

## Schritt 7: HUD — Score und Lives anzeigen

`TEXT(x, y, text$, farbe)` zeichnet Text auf den Bildschirm. Im `DrawAll()` ergänzen wir oben links:

```basic
TEXT(8, 8, f"Score: {score}", PLAYER_C)
TEXT(8, 22, f"Lives: {player.lives}", &HFFFFFF)

IF game_over THEN
    TEXT(WIDTH / 2 - 36, HEIGHT / 2 - 4, "GAME OVER", &HFF4444)
END IF
```

Das ist unser erstes echtes **HUD** (Heads-Up Display). Mit f-Strings können wir den Score-Wert dynamisch in den Text einbauen.

## Schritt 8: Punkte je nach Enemy-Typ

Verschiedene Gegner sollen verschiedene Punkte geben — Bombers sind träger und gefährlicher → mehr Punkte:

```basic
CONST GRUNT_PTS  AS INTEGER = 100
CONST BOMBER_PTS AS INTEGER = 250
```

Im Treffer-Code: `score = score + GRUNT_PTS` bzw. `BOMBER_PTS`. Klein, aber gibt dem Spiel sofort eine Strategie-Note: lieber den Bomber abschießen als drei Grunts.

## Der vollständige Spielcode

Der `main.gb` ist mittlerweile zu lang für vollständigen Abdruck (rund 220 Zeilen) — du findest ihn in [`code/kap-11/main.gb`](code/kap-11/main.gb). Die wesentlichen Änderungen gegenüber Kap 10:

- `IMPORT "physics"` ganz oben
- `Bullet` hat jetzt `w` und `h` als Felder (statt 3/8 hardcodiert in Draw)
- `Player` hat `lives AS INTEGER` und eine `TakeHit()`-Methode
- Globale Variablen `score`, `game_over`
- Zwei neue Funktionen: `CheckBulletEnemyCollisions()` und `CheckPlayerEnemyCollisions()`
- Im `UpdateAll`: nach Update der Objekte werden die Kollisionen geprüft
- HUD-Anzeige in `DrawAll`
- Hauptschleife frieren bei `game_over`

Run drücken. Du solltest jetzt ein **echtes Spiel** vor dir haben:

- Schießt du auf einen Grunt → 100 Punkte
- Schießt du auf einen Bomber → 250 Punkte
- Berührt dich ein Enemy → Lives -1, Enemy verschwindet
- Lives auf 0 → „GAME OVER"-Schriftzug, Spiel friert ein

Das ist der erste **vollständige Mini-Spiel**-Stand. Kapitel 5 bis 11 zusammen ergeben ein Spiel, das tatsächlich spielbar ist. Alles andere ab hier ist Polish und Tiefe.

## Andere physics-Funktionen

Das `physics`-Modul hat mehr als nur `BOX_BOX`. Eine Auswahl:

| Funktion | Was sie liefert |
|---|---|
| `PHYSICS_BOX_BOX(...)` | Box-Box-Überlappung |
| `PHYSICS_CIRCLE_CIRCLE(...)` | Kreis-Kreis-Überlappung |
| `PHYSICS_BOX_CIRCLE(...)` | Box-Kreis-Überlappung |
| `PHYSICS_DISTANCE(x1, y1, x2, y2)` | Euklidische Distanz |
| `PHYSICS_REFLECT_X / _Y(...)` | Reflexion eines Vektors an einer Normalen |
| `PHYSICS_RAY_BOX(...)` | Strahl-Box-Schnitt (für Schüsse mit perfekter Bahn) |

Für Star Pilot reicht uns `BOX_BOX`. Wenn du ein Spiel mit runden Objekten oder physikalischer Reflexion baust — Pong, Breakout, Bumper-Cars — schau in die [physics-Doku](../docs/module-physics.md).

## Vorgriff: Particles

Beim Treffer eines Enemies passiert aktuell nur `alive = FALSE` — der Gegner verschwindet einfach. In Kapitel 13 ergänzen wir hier **Partikel-Effekte**: an der Position des Treffers entstehen 30 kleine Funken, die schnell verfliegen. Das fühlt sich um Welten besser an.

Du kannst dir die Position schon merken (z.B. in einer Variable `last_hit_x`, `last_hit_y`) — aber das brauchen wir erst in Kap 13.

## Übungen

**1. Punkte-Pop-Up.** Bei einem Treffer: zeichne kurz (für ein paar Frames) die Punktzahl als kleinen Text an der Treffer-Position. Hinweis: brauchst eine kleine Klasse `PopUp` mit `x, y, text, frames_left`. Pro Frame: `frames_left -= 1`, beim Erreichen von 0 verschwinden lassen.

**2. Bomber braucht zwei Treffer.** Erweitere `Bomber` um ein Feld `hp AS INTEGER` (Default 2 in `Spawn`). Beim Treffer: `bombers[j].hp -= 1`, erst bei `hp <= 0` auf `alive = FALSE` und Punkte vergeben. Effekt: Bomber sind zäher, der Spieler muss sich konzentrieren.

**3. Unverwundbarkeit nach Treffer.** Direkt nach einem Player-Treffer wäre realistisch: kurze Unverwundbarkeit (~1 Sekunde), in der der Player blinkt. Implementiere ein Feld `invuln_frames AS INTEGER`. Während es > 0 ist: in `CheckPlayerEnemyCollisions` skippen, in `Draw` jedes zweite Frame nicht zeichnen (Blink-Effekt).

**4. Stretch — Nahkollisions-Warnung.** Wenn ein Enemy weniger als 30 Pixel vom Player entfernt ist (`PHYSICS_DISTANCE`), mach einen visuellen Effekt: roter Rand um den Player, oder die HUD-Lives-Anzeige blinkt. Subtil, aber erhöht den Adrenalin-Pegel.

## Zusammenfassung

Du hast in diesem Kapitel:

- ein eingebautes Modul mit `IMPORT "physics"` aktiviert,
- `PHYSICS_BOX_BOX` für AABB-Kollisionen kennengelernt,
- die doppelte Schleife für Bullet × Enemy verstanden,
- Score, Lives und Game-Over implementiert,
- mit `TEXT(...)` ein erstes HUD gebaut,
- den ersten **vollständig spielbaren** Stand von Star Pilot fertig.

Im **nächsten Kapitel** kommt eine ordentliche **Wave-Mechanik**: ENUM für den Game-State, ein Wave-Manager, der Wellen-Anzahl und Schwierigkeit hochzählt. Außerdem wird's Zeit für das `scene`-Modul, das Menu/Playing/GameOver sauber trennt — aber das ist Kap 15.

## Code-Stand am Ende des Kapitels

- [`code/kap-11/01_aabb_demo.gb`](code/kap-11/01_aabb_demo.gb) — `PHYSICS_BOX_BOX` als Konsolen-Demo
- [`code/kap-11/02_collision_loop.gb`](code/kap-11/02_collision_loop.gb) — doppelte Schleife isoliert (Bullets vs. Enemies)
- [`code/kap-11/main.gb`](code/kap-11/main.gb) — Star Pilot mit Kollisionen, Score, Lives, Game Over
