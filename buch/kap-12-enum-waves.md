# Kapitel 12 — ENUM und der Wave-Manager

In Kapitel 11 haben wir den Game-Loop geschlossen — Bullets treffen Enemies, Player verliert Leben, Game Over. Aber das Spiel hat noch keinen **Rhythmus**. Gegner spawnen einfach in Endlos-Folge, alle gleich gefährlich. Wir brauchen Wellen — Welle 1 leicht zum Aufwärmen, Welle 5 schon flott, Welle 10 hektisch.

Außerdem leben wir noch mit einer **`game_over AS BOOLEAN`**-Variable. In Kap 15 wird daraus ein voller Scene-Manager, aber schon jetzt können wir den Spiel-Zustand sauberer machen — mit einem `ENUM`. Spätestens wenn drei oder vier Zustände dazukommen (Wave-Intro, Pause, ...) wird ein einziges Boolean unhaltbar.

## Lernziele

Nach diesem Kapitel:

- definierst du ein `ENUM` in Compact- und Block-Form
- nutzt du `ENUM`-Werte mit `SELECT CASE` im Game-Loop
- typsicher Funktions-Parameter mit ENUM (`SpawnEnemy(typ AS EnemyType, ...)`)
- baust du einen `Wave`-Manager, der Wellen-Nummer, Spawn-Rate und Schwierigkeit verwaltet
- hast du einen Wave-Intro-Screen ("WELLE 3"), der zwischen Wellen für Atempause sorgt

## Schritt 1: Das Problem mit Magic Numbers

Stell dir vor, in Kap 11 hätten wir den `game_over`-Zustand über eine Integer-Variable verwaltet:

```basic
DIM state AS INTEGER
state = 1                       ' was bedeutet 1?

' ... irgendwo im Code:
IF state = 2 THEN
    ' was passiert hier? Wer weiss.
END IF
```

`state = 1` und `state = 2` sind **magische Zahlen**. Du als Autor weißt vielleicht, was sie bedeuten — der Leser nicht. Tipfehler bleiben unbemerkt: `state = 3` ist ein gültiger Wert, auch wenn du ihn nie als Zustand vorgesehen hast.

`ENUM` löst das mit benannten Konstanten:

```basic
ENUM GameState = MENU, PLAYING, PAUSED, GAMEOVER

DIM state AS GameState
state = GameState.PLAYING
```

`GameState.PLAYING` ist klar — niemand muss raten. Tippst du `GameState.PLOYING`, wirft GameBasic einen Fehler („kein Member dieses Namens"). Tippfehler werden zur Compile-Zeit gefangen, nicht zur Laufzeit.

> **Was ENUM intern ist**: hinter den Kulissen sind ENUM-Werte ganz normale Integer (0, 1, 2, ...). `DIM state AS GameState` ist äquivalent zu `DIM state AS INTEGER` — der Parser löst Enum-Typen zu INTEGER auf. Der Vorteil ist nicht *technische Sicherheit*, sondern **Lesbarkeit und Tippfehler-Schutz**. Beides reicht zum Profitieren.

## Schritt 2: ENUM-Formen

GameBasic hat zwei Schreibweisen.

**Compact-Form** (auto-nummeriert ab 0):

```basic
ENUM State = MENU, PLAYING, PAUSED, GAMEOVER
```

Werte: `MENU=0`, `PLAYING=1`, `PAUSED=2`, `GAMEOVER=3`. Schnell zu tippen, gut wenn die Werte selbst egal sind — sie sind nur Marker.

**Block-Form** (mit eigenen Werten):

```basic
ENUM Permission
    NONE  = 0
    READ  = 1
    WRITE = 2
    EXEC  = 4
END ENUM
```

Brauchst du z.B. wenn die Werte **Bit-Flags** sind (1, 2, 4, 8, ...) und du sie kombinieren willst (`READ + WRITE = 3`). Oder bei HTTP-Codes (200, 404, 500), wo die Zahlen selbst eine Bedeutung haben.

> **Mixed**: du kannst beides mischen — explizite Werte und Auto-Nummerierung im Block. Nach einem expliziten Wert zählt's automatisch weiter:
>
> ```basic
> ENUM Http
>     OK = 200
>     CREATED        ' 201
>     ACCEPTED       ' 202
>     NOT_FOUND = 404
> END ENUM
> ```

## Schritt 3: ENUM in einem Funktions-Parameter

Eine Funktion, die einen Typ-Parameter braucht, soll nicht `INTEGER` nehmen — sie soll das ENUM nehmen:

```basic
ENUM EnemyType = GRUNT, BOMBER

SUB SpawnEnemy(typ AS EnemyType, at_x AS INTEGER)
    SELECT CASE typ
        CASE EnemyType.GRUNT
            ' ... grunt spawnen ...
        CASE EnemyType.BOMBER
            ' ... bomber spawnen ...
    END SELECT
END SUB
```

Aufruf:

```basic
SpawnEnemy(EnemyType.GRUNT, 100)
```

Statt `SpawnEnemy(0, 100)` — was für jeden Leser kryptisch wäre — sagst du klar, *welcher* Typ.

> **Vorgriff auf Kap 18 (Named Args)**: in Kapitel 18 lernen wir benannte Argumente. Dann wird daraus `SpawnEnemy(typ: EnemyType.GRUNT, at_x: 100)` — noch lesbarer.

## Schritt 4: Game-State-Machine

Der naheliegendste Einsatz für ENUMs in Star Pilot: der **Spielzustand**. Bisher war's `game_over AS BOOLEAN`. Wir erweitern auf drei Zustände:

```basic
ENUM GameState = PLAYING, WAVE_INTRO, GAMEOVER
```

- **PLAYING**: das normale Spiel — du fliegst, schießt, weichst aus.
- **WAVE_INTRO**: 1.5 Sekunden Pause zwischen Wellen, mit „WELLE 3"-Anzeige in der Mitte.
- **GAMEOVER**: das Bild friert ein, „GAME OVER" wird angezeigt.

Im Game-Loop dispatchen wir per `SELECT CASE`:

```basic
WHILE NOT QUITREQUESTED()
    SELECT CASE state
        CASE GameState.PLAYING
            UpdatePlaying()
        CASE GameState.WAVE_INTRO
            UpdateWaveIntro()
        CASE GameState.GAMEOVER
            ' Eingefroren - Bild bleibt stehen
    END SELECT

    DrawAll()
    SLEEP(16)
WEND
```

Klar erkennbar, was wann passiert. Wenn in Kap 17 ein Pause-Modus dazukommt, fügst du nur `CASE GameState.PAUSED` ein — der Rest bleibt gleich.

## Schritt 5: Der Wave-Manager als Klasse

Statt eine `spawn_timer`-Variable und vier verstreute Hilfs-Variablen pflegen wir alles in einer **Wave-Klasse**:

```basic
CLASS Wave
    DIM number          AS INTEGER
    DIM enemies_left    AS INTEGER
    DIM spawn_timer     AS INTEGER
    DIM spawn_interval  AS INTEGER

    SUB Init()
        number = 1
        StartCurrent()
    END SUB

    SUB StartCurrent()
        enemies_left   = 4 + number * 2
        spawn_interval = 60 - number * 5
        IF spawn_interval < 20 THEN spawn_interval = 20
        spawn_timer    = spawn_interval
    END SUB

    FUNCTION ShouldSpawn() AS BOOLEAN
        IF enemies_left <= 0 THEN RETURN FALSE
        spawn_timer = spawn_timer - 1
        IF spawn_timer <= 0 THEN
            spawn_timer  = spawn_interval
            enemies_left = enemies_left - 1
            RETURN TRUE
        END IF
        RETURN FALSE
    END FUNCTION

    FUNCTION Cleared() AS BOOLEAN
        RETURN enemies_left <= 0
    END FUNCTION

    SUB NextWave()
        number = number + 1
        StartCurrent()
    END SUB
END CLASS
```

Vier wichtige Punkte:

1. **`ShouldSpawn`** wird **pro Frame** aufgerufen. Der Spawn-Timer läuft intern runter; wenn er bei 0 ankommt, liefert die Funktion einmal `TRUE` und resettet den Timer.
2. **`Cleared`** sagt, ob alle geplanten Spawns für diese Welle durch sind. Achtung: das heißt nicht „alle Gegner sind tot" — die laufen oft noch weiter den Bildschirm runter. Erst wenn *zusätzlich* alle vom Bild verschwunden sind, ist die Welle wirklich beendet.
3. **Schwierigkeitskurve**: `enemies_left = 4 + number * 2` (Welle 1: 6, Welle 2: 8, ...), `spawn_interval = 60 - number * 5` (Welle 1: 55 Frames, Welle 2: 50, ...). Mit `IF spawn_interval < 20 THEN spawn_interval = 20` zähmen wir die obere Schwierigkeit — sonst würde es ab Welle 12 unspielbar.
4. **`StartCurrent` wird aus `Init` und `NextWave` aufgerufen** — kein doppelter Code. GameBasic erlaubt seit kurzem den **impliziten Methoden-Aufruf** innerhalb einer Klasse: `StartCurrent()` (ohne `Self.`) findet die Methode der eigenen Klasse, der Compiler dispatcht das automatisch. Genauso könnten wir auch `Self.StartCurrent()` schreiben — beides ist erlaubt, der implizite Aufruf ist nur kürzer.

### Test in der Konsole

Bevor wir die Wave-Klasse ins Spiel integrieren, simulieren wir sie ohne Bild:

```basic
DIM w AS Wave
w = NEW Wave()
PRINT f"Welle {w.number}: {w.enemies_left} Gegner geplant, alle {w.spawn_interval} Frames"

DIM frame AS INTEGER
DIM spawns AS INTEGER
spawns = 0
FOR frame = 1 TO 1000
    IF w.ShouldSpawn() THEN
        spawns = spawns + 1
    END IF
    IF w.Cleared() THEN BREAK
NEXT frame

PRINT f"  -> {spawns} Spawns, abgeschlossen nach Frame {frame}"
```

Output:

```
Welle 1: 6 Gegner geplant, alle 55 Frames
  -> 6 Spawns, abgeschlossen nach Frame 330
```

Sechs Spawns nach 330 Frames — das ist `6 × 55 = 330`. Passt mathematisch.

## Schritt 6: Der Wave-Intro-Screen

Zwischen den Wellen wollen wir einen Atempause-Moment: 1.5 Sekunden lang steht „WELLE 3" in der Bildschirmmitte, dann geht's weiter.

Im `UpdatePlaying`, sobald die Welle clear *und* alle Gegner weg sind:

```basic
IF wave.Cleared() AND EnemiesAlive() = 0 THEN
    wave.NextWave()
    state = GameState.WAVE_INTRO
    intro_frames_left = INTRO_FRAMES        ' 90 = 1.5 Sek
END IF
```

Und der `UpdateWaveIntro` ist trivial:

```basic
SUB UpdateWaveIntro()
    intro_frames_left = intro_frames_left - 1
    IF intro_frames_left <= 0 THEN
        state = GameState.PLAYING
    END IF
END SUB
```

In `DrawAll` zeigen wir den Schriftzug:

```basic
SELECT CASE state
    CASE GameState.WAVE_INTRO
        TEXT(WIDTH / 2 - 36, HEIGHT / 2 - 4, f"WELLE {wave.number}", &HFFDC00)
    CASE GameState.GAMEOVER
        TEXT(WIDTH / 2 - 36, HEIGHT / 2 - 4, "GAME OVER", &HFF4444)
END SELECT
```

Während `WAVE_INTRO` läuft `UpdatePlaying` *nicht* — keine neuen Spawns, keine Tasten, kein Schießen. Player und Bullets stehen still. Spieler bekommt eine Sekunde Luft.

## Schritt 7: SpawnEnemy mit ENUM-Parameter

Bisher hatten wir `SpawnGrunt(at_x)` und `SpawnBomber(at_x)` als zwei separate Funktionen. Jetzt kombinieren wir sie:

```basic
SUB SpawnEnemy(typ AS EnemyType, at_x AS INTEGER)
    DIM i AS INTEGER
    SELECT CASE typ
        CASE EnemyType.GRUNT
            FOR i = 0 TO ENEMY_POOL - 1
                IF NOT grunts[i].alive THEN
                    grunts[i].Spawn(at_x)
                    RETURN
                END IF
            NEXT i
        CASE EnemyType.BOMBER
            FOR i = 0 TO ENEMY_POOL - 1
                IF NOT bombers[i].alive THEN
                    bombers[i].Spawn(at_x)
                    RETURN
                END IF
            NEXT i
    END SELECT
END SUB
```

Im Wave-Spawn:

```basic
DIM typ AS EnemyType
IF INT(RND() * 2) = 0 THEN
    typ = EnemyType.GRUNT
    SpawnEnemy(typ, INT(RND() * (WIDTH - 16)))
ELSE
    typ = EnemyType.BOMBER
    SpawnEnemy(typ, INT(RND() * (WIDTH - 22)))
END IF
```

Lesbar — der Aufrufer sagt klar, *welchen Typ* er spawnen will.

## Der vollständige Spielcode

Der `main.gb` ist mit gut 280 Zeilen wieder zu lang für vollständigen Abdruck — du findest ihn in [`code/kap-12/main.gb`](code/kap-12/main.gb). Die Änderungen gegenüber Kap 11:

- Zwei ENUMs ganz oben: `GameState`, `EnemyType`
- `Wave`-Klasse für die Wellen-Logik
- Globaler Zustand: `state AS GameState` statt `game_over AS BOOLEAN`, plus `intro_frames_left`
- `SpawnEnemy(typ, at_x)` ersetzt `SpawnGrunt` und `SpawnBomber`
- `UpdatePlaying` ruft `wave.ShouldSpawn()`, prüft `wave.Cleared()` und triggert WAVE_INTRO
- Hauptschleife dispatcht via `SELECT CASE state`

Run drücken. Du siehst:

- **WELLE 1** in der Mitte (1.5 Sek)
- Dann **6 Gegner** (gemischt Grunts + Bombers), die nach und nach kommen
- Wenn alle weg sind: **WELLE 2** → 8 Gegner, etwas schneller
- Welle 3 → 10 Gegner, noch schneller
- ...

Der Schwierigkeitsgrad steigt — bei Welle 8 kommt's hektisch zu, ab Welle 9 hast du dauerhaft Druck.

## Vorgriff: Tween-Choreographie (Kap 14)

Aktuell spawnen Gegner an **zufälligen X-Positionen**. Authentisches Galaga-Feeling sieht anders aus — die Gegner kommen in **Formation** rein, schwingen synchron, gehen dann zur Attacke über.

In Kap 14 lernen wir das `tween`-Modul und ersetzen die `Update`-Methode der Enemies um Tween-basierte Bewegung. Erst rein-fliegen, dann oszillieren, dann attackieren. Das gibt dem Spiel den eigentlichen *arcade-Charme*.

## Übungen

**1. Schwierigkeit als ENUM.** Definiere `ENUM Difficulty = EASY, NORMAL, HARD`. In `Wave.Init` lass die Werte für `enemies_left` und `spawn_interval` von der Schwierigkeit abhängen (HARD: doppelt so viele Gegner, halb so lange Intervalle). Setze die Schwierigkeit über eine globale Variable, die du am Programm-Anfang änderst. (In Kap 17 wird daraus ein UI-Slider.)

**2. Score-basierte Boss-Welle.** Alle 5 Wellen (Welle 5, 10, 15...) soll eine "Boss-Welle" kommen — keine normalen Gegner, sondern *ein* Bomber mit 5x Lebenspunkten, dafür 1000 Punkte wert. Skelett: erweitere `Wave` um eine Methode `IsBossWave()` und einen entsprechenden Spawn-Mechanismus.

**3. ENUM für Powerup-Typen.** Definiere `ENUM PowerupType = SHIELD, RAPID_FIRE, MULTI_SHOT, EXTRA_LIFE`. Schreibe noch keine Logik — überlege dir nur, welche Werte und welche Typen du brauchst, falls du die Powerups in Kap 18 implementieren möchtest. (Stretch-Übung Schritt 1: das **Design** vor der Implementierung.)

**4. Stretch — Wellen-Übersicht.** Schreibe eine SUB `PrintWaveStats(start AS INTEGER, ende AS INTEGER)`, die in der Konsole eine Tabelle ausgibt: für jede Welle von `start` bis `ende` die geplante Gegner-Anzahl und das Spawn-Interval. Gut um deine Schwierigkeitskurve zu evaluieren — wo wird's frustrierend, wo zu leicht?

## Zusammenfassung

Du hast in diesem Kapitel:

- ENUMs in Compact- und Block-Form definiert,
- typsicher Funktions-Parameter mit ENUM gemacht,
- den Game-Loop mit `SELECT CASE state` strukturiert (statt mit Boolean-Flags),
- einen `Wave`-Manager als Klasse gebaut, der Schwierigkeit und Spawn-Rate pro Welle kontrolliert,
- einen Wave-Intro-Screen als ersten zusätzlichen Spielzustand eingebaut.

Im **nächsten Kapitel** wird's visuell: wir holen das `sprite`-Modul ins Boot und ersetzen die langweiligen Boxen durch echte animierte Pixelsprites. Dazu kommen **Particles** für Explosionen — das ist der Moment, wo Star Pilot endlich aussieht wie ein Spiel.

## Code-Stand am Ende des Kapitels

- [`code/kap-12/01_enum.gb`](code/kap-12/01_enum.gb) — ENUM-Grundlagen, Compact und Block
- [`code/kap-12/02_state_machine.gb`](code/kap-12/02_state_machine.gb) — `GameState` mit `SELECT CASE` dispatchen
- [`code/kap-12/03_wave_manager.gb`](code/kap-12/03_wave_manager.gb) — `Wave`-Klasse isoliert, mit Schwierigkeitskurve
- [`code/kap-12/main.gb`](code/kap-12/main.gb) — Star Pilot mit Wellen, Wave-Intro und ENUM-State-Machine
