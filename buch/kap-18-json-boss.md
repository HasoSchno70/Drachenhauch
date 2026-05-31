# Kapitel 18 — JSON-Wellen und der Boss-Fight

Der letzte Schritt. Bisher waren alle Spiel-Werte fest im Code: Wellen-Größe als `4 + number * 2`, Spawn-Intervall als `60 - number * 5`. Das funktioniert, aber es bedeutet: jede Änderung am Schwierigkeitsgrad braucht einen Editor-Lauf, einen Recompile, einen Test.

In diesem Kapitel **externalisieren** wir die Wellen-Daten. Sie liegen in einer JSON-Datei neben dem Programm, und ein Spielentwickler (du oder ein Tester) kann sie ändern, ohne den Code anzufassen. Plus: wir bauen den **Boss-Fight** als großen Showdown — eine dicke Drohne mit drei Phasen, die HP-basiert eskaliert.

Und ganz nebenbei: dieses Kapitel zeigt zum ersten Mal **Named Arguments** im Star-Pilot-Code. Wenn der Boss mit `NEW Boss(start_hp: 30)` instanziiert wird, ist das nicht nur kürzer — es ist auch dem Leser klarer, was die Zahl bedeutet.

## Lernziele

Nach diesem Kapitel:

- liest du JSON-Daten mit `IMPORT "json"` aus einer Datei
- nutzt du Pfad-Notation (`"waves.0.size"`) für verschachtelte Werte
- robusten Code mit `TRY / CATCH` für fehlende oder kaputte Save-Dateien
- baust einen mehrphasigen Boss als eigene Klasse mit HP-basierten Phasen
- kennst Star Pilot Ende-zu-Ende: Menu → Spielen → Wellen → Boss → Win

## Schritt 1: Die JSON-Datei

Wellen-Daten liegen in `waves.json` neben dem Spiel:

```json
{
  "_version": 1,
  "waves": [
    { "size":  6, "interval": 55, "bomber_chance": 25 },
    { "size":  8, "interval": 50, "bomber_chance": 30 },
    { "size": 10, "interval": 45, "bomber_chance": 35 },
    { "size": 12, "interval": 40, "bomber_chance": 40 },
    { "size": 14, "interval": 35, "bomber_chance": 45 }
  ],
  "boss": {
    "wave": 6,
    "hp": 30
  }
}
```

Drei Werte pro Welle:

- **`size`**: wie viele Gegner spawnen insgesamt
- **`interval`**: Frames zwischen zwei Spawns (kleiner = schneller)
- **`bomber_chance`**: Wahrscheinlichkeit (0–100) für einen Bomber statt Grunt

Plus: ab Welle 6 gibt's einen Boss mit 30 HP.

> **Vorteil**: ein Tester kann Welle 3 leichter machen (`"interval": 60` statt `45`), eine schwere Boss-Welle ausprobieren (`"hp": 50`), eine ganze neue Welle anhängen — alles ohne den Code anzufassen.

## Schritt 2: JSON laden und auslesen

Drei Funktionen reichen für 95% der Use-Cases:

| Funktion | Was sie tut |
|---|---|
| `JSON_LOAD(path$)` | Lädt eine JSON-Datei, liefert `JSON_HANDLE` |
| `JSON_GET_INT(handle, pfad$)` | Liest INTEGER an `pfad$` (Pfad mit Punkt-Notation) |
| `JSON_LEN(handle, pfad$)` | Länge eines Arrays oder Strings |

Plus `JSON_GET_STRING`, `JSON_GET_FLOAT`, `JSON_GET_BOOL`, `JSON_HAS` (existiert?), und Stringify-Funktionen.

### Pfad-Notation

```basic
JSON_GET_INT(j, "boss.hp")           ' = 30
JSON_GET_INT(j, "waves.0.size")      ' = 6 (erste Welle, Index 0)
JSON_GET_INT(j, "waves.4.interval")  ' = 35 (fünfte Welle)
JSON_LEN(j, "waves")                 ' = 5
```

Mit Punkten getrennt — Objekt-Keys, Array-Indizes (numerisch), tiefer beliebig verschachtelt.

### Demo in der Konsole

```basic
IMPORT "json"

DIM s AS STRING
s = "{ ""name"": ""Anna"", ""score"": 4200, ""levels"": [10, 20, 30] }"

DIM j AS JSON_HANDLE
j = JSON_PARSE(s)

PRINT "  Name:  " + JSON_GET_STRING(j, "name")
PRINT f"  Score: {JSON_GET_INT(j, "score")}"

DIM i AS INTEGER
FOR i = 0 TO JSON_LEN(j, "levels") - 1
    DIM path AS STRING
    DIM val  AS INTEGER
    path = "levels." + STR$(i)
    val  = JSON_GET_INT(j, path)
    PRINT f"    levels.{i} = {val}"
NEXT i
```

Output:

```
  Name:  Anna
  Score: 4200
    levels.0 = 10
    levels.1 = 20
    levels.2 = 30
```

> **Stolperfalle**: f-Strings können **nicht verschachtelt** sein. `f"...{JSON_GET_INT(j, f"levels.{i}")}"` (innerer f-String im äußeren) wirft einen Lexer-Fehler. Daher der Pfad-String separat zusammenbauen — `path = "levels." + STR$(i)` — und dann lesen.
>
> **Stolperfalle 2**: innerhalb eines f-String-Ausdrucks `{...}` brauchst du **keine** Anführungszeichen-Verdoppelung. `f"{JSON_GET_INT(j, "score")}"` funktioniert — der Lexer parst nur bis zum schließenden `}`.

## Schritt 3: TRY / CATCH

Was passiert, wenn `waves.json` fehlt? `JSON_LOAD` wirft einen Fehler — und dein Spiel stürzt beim Start ab. Das wollen wir nicht: ein fehlende JSON sollte das Programm nicht beenden, sondern auf Defaults zurückfallen.

`TRY ... CATCH ... END TRY` ist GameBasics Antwort:

```basic
TRY
    wave_data = JSON_LOAD(WAVES_PATH)
    boss_at_wave = JSON_GET_INT(wave_data, "boss.wave")
    boss_hp_init = JSON_GET_INT(wave_data, "boss.hp")
CATCH err
    ' Datei fehlt oder kaputt -> mit Defaults weiterleben
    PRINT "Warnung: " + err
    wave_data = JSON_PARSE("{ ""waves"": [], ""boss"": {} }")
    boss_at_wave = 6
    boss_hp_init = 30
END TRY
```

Das `CATCH err` fängt jeden Fehler im TRY-Block ab. `err` ist ein STRING mit der Fehlermeldung. Wir geben eine Warnung aus (statt zu crashen) und setzen Default-Werte.

> **Konvention**: TRY/CATCH ist nicht für jede „könnte schiefgehen"-Stelle. Wenn ein Fehler ein **echter Bug** wäre (z.B. NULL-Pointer-Zugriff, Index out of range), willst du den Crash — er hilft dir, den Bug zu finden. TRY ist nur für **erwartete** Fehler-Quellen: fehlende Dateien, kaputte User-Daten, Netzwerk-Aussetzer.

> **Stolperfalle: Reihenfolge im Setup**. `wave_data` muss **vor** `wave = NEW Wave()` initialisiert sein — `Wave.Init` ruft `StartCurrent`, das gleich `JSON_LEN(wave_data, ...)` braucht. Wenn `wave_data` zu dem Zeitpunkt noch NIL ist, wirft das Modul einen Type-Mismatch-Error („JSON_LEN erwartet JSON-Handle"). Praktische Regel: Daten-Loading **immer ganz oben** in `Setup()`, danach erst die Klassen-Instanzen.

## Schritt 4: Wave-Klasse aus JSON

Die Wave-Klasse liest jetzt ihre Werte aus `wave_data`:

```basic
CLASS Wave
    DIM number          AS INTEGER
    DIM enemies_left    AS INTEGER
    DIM spawn_timer     AS INTEGER
    DIM spawn_interval  AS INTEGER
    DIM bomber_chance   AS INTEGER

    SUB Init()
        number = 1
        StartCurrent()
    END SUB

    SUB StartCurrent()
        DIM idx     AS INTEGER
        DIM n_waves AS INTEGER
        idx = number - 1
        n_waves = JSON_LEN(wave_data, "waves")
        IF idx < n_waves THEN
            DIM prefix AS STRING
            prefix = "waves." + STR$(idx) + "."
            enemies_left   = JSON_GET_INT(wave_data, prefix + "size")
            spawn_interval = JSON_GET_INT(wave_data, prefix + "interval")
            bomber_chance  = JSON_GET_INT(wave_data, prefix + "bomber_chance")
        ELSE
            ' Endlos-Modus jenseits der definierten Wellen
            enemies_left   = 14 + (number - n_waves) * 2
            spawn_interval = 30 - (number - n_waves)
            IF spawn_interval < 15 THEN spawn_interval = 15
            bomber_chance  = 50
        END IF
        spawn_timer = spawn_interval
    END SUB

    ' ... ShouldSpawn, Cleared, NextWave wie vorher ...
END CLASS
```

Zwei Punkte:

1. **`prefix = "waves." + STR$(idx) + "."`** baut den Pfad-Präfix einmal, dann `prefix + "size"` etc. Lesbarer als drei separate `f"..."`-Aufrufe.
2. **Fallback bei `idx >= n_waves`**: das Spiel geht über die definierten Wellen hinaus. Endlos-Modus mit linear steigender Schwierigkeit.

In `UpdatePlaying` nutzen wir `bomber_chance` für die Spawn-Wahl:

```basic
IF INT(RND() * 100) < wave.bomber_chance THEN
    typ = EnemyType.BOMBER
ELSE
    typ = EnemyType.GRUNT
END IF
```

## Schritt 5: Die Boss-Klasse

Der Boss ist groß, hat HP, drei Phasen, schwingt seitlich:

```basic
CLASS Boss
    DIM x       AS INTEGER
    DIM y       AS INTEGER
    DIM w       AS INTEGER
    DIM h       AS INTEGER
    DIM hp      AS INTEGER
    DIM hp_max  AS INTEGER
    DIM phase   AS INTEGER
    DIM alive   AS BOOLEAN
    DIM swing   AS TWEEN

    SUB Init(start_hp AS INTEGER)
        w = 60 : h = 32
        hp = start_hp : hp_max = start_hp
        phase = 1 : alive = FALSE
        x = WIDTH / 2 - w / 2 : y = 30
        swing = TWEEN_NEW_PINGPONG(-100.0, 100.0, 4000, "inout_sine")
    END SUB

    SUB Spawn()
        hp = hp_max : phase = 1 : alive = TRUE
        x = WIDTH / 2 - w / 2 : y = 30
        swing = TWEEN_NEW_PINGPONG(-100.0, 100.0, 4000, "inout_sine")
    END SUB

    SUB TakeHit()
        IF NOT alive THEN RETURN
        hp = hp - 1
        IF hp <= 0 THEN
            alive = FALSE
            RETURN
        END IF
        UpdatePhase()
    END SUB

    SUB UpdatePhase()
        DIM ratio AS FLOAT
        ratio = (hp + 0.0) / hp_max
        IF ratio < 0.2 AND phase < 3 THEN
            phase = 3
            swing = TWEEN_NEW_PINGPONG(-120.0, 120.0, 1500, "inout_sine")
        ELSEIF ratio < 0.5 AND phase < 2 THEN
            phase = 2
            swing = TWEEN_NEW_PINGPONG(-110.0, 110.0, 2500, "inout_sine")
        END IF
    END SUB

    SUB Update()
        IF NOT alive THEN RETURN
        x = WIDTH / 2 - w / 2 + INT(TWEEN_VALUE(swing))
    END SUB

    SUB Draw()
        IF NOT alive THEN RETURN
        BOX(x, y, x + w, y + h, BOSS_C)
        ' HP-Balken
        BOX(WIDTH - 90, 24, WIDTH - 8, 30, &H400000)
        DIM hp_w AS INTEGER
        hp_w = INT((hp + 0.0) / hp_max * 82)
        BOX(WIDTH - 90, 24, WIDTH - 90 + hp_w, 30, BOSS_C)
        TEXT(WIDTH - 88, 14, f"BOSS: {hp}/{hp_max}", BOSS_C)
    END SUB
END CLASS
```

Vier Schlüsselpunkte:

1. **Init vs. Spawn**: gleicher Trick wie bei Bullets in Kap 9 — `Init` wird einmal beim `NEW` aufgerufen (allokiert das Objekt), `Spawn` aktiviert es bei Bedarf. Der Boss-Slot lebt also vom Programmstart, ist aber bis zur Boss-Welle inaktiv.
2. **Phase wechselt in `UpdatePhase`**, nicht direkt in `TakeHit`. Dank impliziter Methoden-Aufrufe (Kap 12) können wir das ohne `Self.`-Präfix machen.
3. **HP-Schwellen**: `< 50%` triggert Phase 2 (Schwung 2500 ms statt 4000 — schneller), `< 20%` triggert Phase 3 (1500 ms — hektisch). Pro Phase ein **neuer** Tween mit anderen Parametern.
4. **HP-Balken oben rechts**: visuelles Feedback. Der Spieler sieht in Echtzeit, wie viel der Boss noch aushält.

## Schritt 6: Boss-Kollisionen

Zwei neue Helper:

```basic
SUB CheckBulletBossCollision()
    IF NOT boss.alive THEN RETURN
    DIM i AS INTEGER
    FOR i = 0 TO BULLET_POOL - 1
        IF NOT bullets[i].alive THEN CONTINUE
        IF PHYSICS_BOX_BOX(bullets[i].x, bullets[i].y, bullets[i].w, bullets[i].h, _
                           boss.x, boss.y, boss.w, boss.h) THEN
            bullets[i].alive = FALSE
            boss.TakeHit()
            score = score + 50
            Explode(bullets[i].x, bullets[i].y, 8)
            ' Wenn Boss gerade gestorben: dicke finale Explosion
            IF NOT boss.alive THEN
                Explode(boss.x + boss.w / 2, boss.y + boss.h / 2, 80)
            END IF
        END IF
    NEXT i
END SUB

SUB CheckPlayerBossCollision()
    IF NOT boss.alive THEN RETURN
    IF PHYSICS_BOX_BOX(player.x, player.y, player.w, player.h, _
                       boss.x, boss.y, boss.w, boss.h) THEN
        player.TakeHit()
        Explode(player.x + player.w / 2, player.y + player.h / 2, 50)
    END IF
END SUB
```

Das Pattern ist identisch zu den vorherigen Kollisions-Checks (Kap 11). Pro Bullet ein `PHYSICS_BOX_BOX` gegen den Boss; pro Player-Frame einer gegen den Boss.

## Schritt 7: Boss-Welle erkennen

In `UpdatePlaying`, vor der normalen Spawn-Logik:

```basic
' Boss-Welle: keine normale Spawn-Logik, der Boss laeuft schon
IF wave.number = boss_at_wave THEN
    RETURN
END IF
```

Bei der Boss-Welle spawnen wir keine normalen Gegner. Der Boss wurde direkt beim Wave-Start aktiviert (siehe Wechsel-Logik unten).

Bei `wave.NextWave()` prüfen wir, ob die nächste Welle die Boss-Welle ist:

```basic
IF wave.Cleared() AND EnemiesAlive() = 0 THEN
    wave.NextWave()
    StartWaveIntro()
    IF wave.number = boss_at_wave THEN
        boss.Spawn()
    END IF
END IF
```

## Schritt 8: Win-Scene

Wenn der Boss tot ist (und keine anderen Gegner mehr da):

```basic
IF wave.number = boss_at_wave AND NOT boss.alive AND EnemiesAlive() = 0 THEN
    IF score > highscore THEN
        highscore = score
        SaveHighscore()
    END IF
    DIM win_score AS INTEGER
    win_score = score
    SCENE_SWITCH("win")
    SCENE_SET_INT("final_score", win_score)
    RETURN
END IF
```

Das Switch zur `"win"`-Scene löst den Sieg aus. `UpdateWin` und `DrawWin` analog zu Game-Over:

```basic
SUB UpdateWin()
    IF KEYPRESSED(KEY_RETURN) THEN
        SCENE_SWITCH("menu")
    END IF
END SUB

SUB DrawWin()
    CLS(BG_COLOR)
    DrawHUD()
    TEXT(WIDTH / 2 - 40, 60, "DU HAST GEWONNEN!", PLAYER_C)
    DIM final_score AS INTEGER
    final_score = SCENE_GET_INT_OR("final_score", 0)
    TEXT(WIDTH / 2 - 70, 100, f"Endstand: {final_score}", &HFFFFFF)
    TEXT(WIDTH / 2 - 60, 130, f"Highscore: {highscore}", &HCCCCCC)
    TEXT(WIDTH / 2 - 84, 170, "ENTER -> zurueck zum Menue", &HCCCCCC)
END SUB
```

## Schritt 9: Named Arguments

Eine subtile aber lesbarkeits-relevante Verbesserung: beim `NEW Boss(...)`-Aufruf nutzen wir Named Args, falls der Constructor mehrere Parameter hat. Aktuell ist's nur `start_hp`, daher nicht zwingend — aber wenn du den Boss erweiterst (`hp`, `start_y`, `swing_speed`, `color`), wird's sehr nützlich:

```basic
boss = NEW Boss(start_hp: boss_hp_init, start_y: 30, color: &HFF44AA)
```

Statt:

```basic
boss = NEW Boss(boss_hp_init, 30, &HFF44AA)    ' was bedeutet die 30?
```

Das ist Übung 4 unten.

## Star Pilot — komplett

Der `main.gb` ist mit ~700 Zeilen das längste File des Buchs. Kein Problem: es ist klar strukturiert in Sektionen — Klassen oben, globale Variablen, Setup, Helpers, Update- und Draw-Subs pro Scene, Hauptloop.

Wenn du das Spiel jetzt von vorne durchspielst:

- **Menü**: Highscore aus persistentem Save geladen, ENTER startet
- **Welle 1–5**: Schwierigkeit aus `waves.json`
- **Welle 6**: Boss erscheint, drei Phasen je nach HP, dicker Showdown
- **Win**: „DU HAST GEWONNEN!" — und der Highscore wurde gespeichert

Das ist ein vollständiges kleines Arcade-Spiel. **Dein Spiel.**

## Übungen

**1. Eigene Welle 6.** Erweitere `waves.json` um eine sechste Welle (vor dem Boss): kleinere `size`, kleinster `interval`, hoher `bomber_chance` — eine echte Hürde, bevor's gegen den Boss geht. Setze gleichzeitig `boss.wave: 7`.

**2. Boss-Bullets.** Aktuell ist der Boss „nur" eine bewegliche Wand — er schießt nicht. Erweitere ihn um einen kleinen Bullet-Pool (z.B. 5 boss-Bullets), die in regelmäßigen Abständen nach unten fliegen. In Phase 3: Doppelschuss (zwei nebeneinander).

**3. Schema-Versionierung.** Wenn du `waves.json` änderst (neue Felder, andere Struktur), erhöhe `_version` von 1 auf 2. Im Code: prüfe `JSON_GET_INT(wave_data, "_version")` — wenn nicht 2, gib eine Warnung aus und nutze Defaults.

**4. Stretch — Boss mit Named Args.** Erweitere die Boss-Klasse: `Init(start_hp, target_y AS INTEGER = 30, color AS INTEGER = &HFF44AA)`. Erzeuge den Boss mit Named Args:
```basic
boss = NEW Boss(start_hp: boss_hp_init, target_y: 30, color: &HFF44AA)
```
Schau, wie viel klarer das ist als drei positionale Argumente.

## Zusammenfassung

Du hast in diesem Kapitel:

- das `json`-Modul mit `JSON_LOAD`, `JSON_GET_*` und Pfad-Notation kennengelernt,
- Wellen-Daten aus einer externen Datei gelesen,
- mit `TRY / CATCH` robust auf fehlende Dateien reagiert,
- einen Boss als Klasse mit drei HP-basierten Phasen gebaut,
- Boss-Kollisionen und Win-Scene integriert,
- Named Arguments als Lesbarkeits-Booster für komplexe Constructors gesehen.

## Was kommt jetzt?

**Du hast ein fertiges Spiel.** Das war der Anspruch des Buchs. Was als „PRINT 'Hallo, Pilot!'" angefangen hat (Kapitel 1), ist jetzt Star Pilot mit Wellen, Boss, Highscore, Pause-Menü, animierten Effekten — knapp 700 Zeilen Code, die du Schritt für Schritt selbst getippt hast.

**Was tun mit dem Wissen?**

- **Modifizieren.** Mehr Wellen in `waves.json`. Eigene Sprite-Pixel-Art (siehe Kap 13). Andere Easings probieren. Eine neue Klasse — Drifter, Power-Up, Schild-Drohne. Was du dir vorstellst, kannst du jetzt auch bauen.
- **Eigenes Spiel.** Pong, Breakout, Tetris, Snake. Die Bausteine aus diesem Buch — Klassen, Scene-Stack, Particles, Tween, Save — passen für jedes 2D-Arcade-Spiel.
- **Die Anhänge lesen.** [Anhang A](anhang-a-troubleshooting.md) sammelt häufige Fehler. [Anhang B](anhang-b-cython.md) zeigt, wie du Star Pilot mit der Cython-VM beschleunigst und benchmarkst.

Aber das Wichtigste: **du kannst jetzt Spiele schreiben.** Das war das Ziel. Den Rest entscheidet deine Fantasie.

Viel Spaß.

## Code-Stand am Ende des Kapitels

- [`code/kap-18/01_json_demo.gb`](code/kap-18/01_json_demo.gb) — JSON parsen + lesen, mit Datei-Beispiel
- [`code/kap-18/02_boss.gb`](code/kap-18/02_boss.gb) — Boss-Klasse + Phasen-Wechsel-Test in der Konsole
- [`code/kap-18/main.gb`](code/kap-18/main.gb) — vollständiges Star Pilot mit JSON-Wellen, Boss-Fight, Win-Scene
- [`code/kap-18/waves.json`](code/kap-18/waves.json) — die externe Wellen-Konfiguration
