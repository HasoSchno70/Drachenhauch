# Kapitel 20 — Eigene Bulk-Builtins schreiben

In Kapitel 19 haben wir gesehen, wie eingebaute Bulk-Ops wie `ECS_INTEGRATE_FLOAT` einen pro-Entity-Loop in BASIC um Faktor 40 schlagen. Das wirft die Frage auf: **was, wenn ich ein Spiel-Pattern habe, das nicht in der eingebauten Liste vorkommt?**

Beispiele aus echten Spielen:

- **Kollisions-Resolver:** "alle Bullets gegen alle Enemies pruefen, bei Treffer: Bullet zerstoeren, Enemy HP abziehen". Das ist O(N×M), in BASIC mit ECS_GET/ADD pro Paar ist es bei 500 Bullets × 50 Enemies unbenutzbar (25.000 Vergleiche/Frame × 4 Builtin-Calls = 100.000 Calls).
- **Steering-Behaviors:** "alle Enemies bewegen sich auf den Spieler zu". Pro Enemy: Distanz lesen, Vec2 normalisieren, Velocity setzen.
- **Lifecycle-Cleanup:** "alle Particles mit lifetime <= 0 zerstoeren, sonst lifetime -= dt".

Die Antwort: **schreib das System als eigenes Built-in.** Es ist nicht so viel Arbeit wie es klingt — die Engine hat dafuer ein klares Muster, das wir hier durchlaufen.

## Lernziele

Nach diesem Kapitel:

- erkennst du, wann ein eigenes Bulk-Builtin sich lohnt (Faustregel: > 1000 Builtin-Calls pro Frame)
- weisst du, dass das Pattern aus drei Teilen besteht: cdef-Method in `ecs_native.pyx`, Python-Fallback in `ecs.py`, `@builtin`-Wrapper
- hast du selbst ein neues Bulk-Builtin geschrieben: `ECS_BULLET_HIT_ENEMY` — den klassischen Kollisions-Resolver
- benchst du das gegen die naive BASIC-Variante

## Schritt 1: Wann lohnt sich ein eigenes Builtin?

Faustregel: wenn du dasselbe Operation-Pattern **pro Frame** auf vielen Entities ausfuehrst und in BASIC die Anzahl Builtin-Calls > ~1000 ist.

Beispiele wo es sich lohnt:

- Kollisions-Resolver (paar tausend Vergleiche/Frame)
- Steering (jeder Enemy berechnet seine Velocity neu)
- Wave-Spawner mit komplexer Verteilung
- Tile-basierte Pixel-Effekte (Lava-Animation, Feuer)

Beispiele wo es sich **nicht** lohnt:

- Player-Input (1 Player, 1× pro Frame)
- Cutscenes (ein Tween, ein Tween-Update)
- Menue-Logik (Klicks sind selten)

Wenn du dir unsicher bist: profiler-aufruf zaehlen oder einfach den BASIC-Code mit `gbrun.py --bench` messen. Wenn der Frame > 5 ms braucht und der Bottleneck ein Loop ist — Kandidat fuer Bulk.

## Schritt 2: Das Dreiteiler-Pattern

Jedes Bulk-Builtin im `ecs`-Modul folgt demselben Muster:

1. **cdef-Method auf `_World`** in [`gamebasic/modules/ecs_native.pyx`](../gamebasic/modules/ecs_native.pyx) — der schnelle C-Pfad
2. **Python-Fallback-Method** in [`gamebasic/modules/ecs.py`](../gamebasic/modules/ecs.py) — funktional identisch, fuer Entwickler ohne kompilierte `.pyd`
3. **`@builtin`-Wrapper** in `ecs.py` — Type-Checks + Delegate auf eine der beiden Methods

Der Wrapper ruft die Method automatisch auf der richtigen Implementation auf — die cdef-Version wenn da, sonst Python-Fallback. Das funktioniert, weil beide unter demselben Namen registriert sind: `register_type("ecs_world", _World)` im jeweiligen Pfad.

## Schritt 3: Beispiel — Kollisions-Resolver

Ziel: ein Builtin `ECS_BULLET_HIT_ENEMY(world)`, das alle Bullets gegen alle Enemies prueft. Bei Kollision: Bullet zerstoeren, Enemy "damage"-Component um 1 hochzaehlen (oder Enemy zerstoeren, je nach Design).

### 3.1 Was die Funktion machen soll

```basic
' Vorher (BASIC, langsam):
DIM bullets AS ARRAY OF INTEGER
DIM enemies AS ARRAY OF INTEGER
bullets = ECS_QUERY3(world, "px", "py", "bullet_tag")
enemies = ECS_QUERY3(world, "px", "py", "enemy_tag")
FOR i = 0 TO LEN(bullets) - 1
    FOR j = 0 TO LEN(enemies) - 1
        DIM dx AS FLOAT
        DIM dy AS FLOAT
        dx = ECS_GET_FLOAT(world, bullets[i], "px") - ECS_GET_FLOAT(world, enemies[j], "px")
        dy = ECS_GET_FLOAT(world, bullets[i], "py") - ECS_GET_FLOAT(world, enemies[j], "py")
        IF dx*dx + dy*dy < 100.0 THEN     ' radius=10
            ECS_DESTROY(world, bullets[i])
            ECS_INTEGRATE_INT(world, "damage", "one") ' uhhh
        END IF
    NEXT
NEXT
```

Bei 500 Bullets × 50 Enemies sind das 25.000 Distanz-Checks × 4 ECS_GET_FLOAT-Calls = **100.000 Calls pro Frame**. Inakzeptabel.

### 3.2 Cdef-Method (Performance-Pfad)

In `gamebasic/modules/ecs_native.pyx`, im `_World` cdef-class:

```cython
cpdef Py_ssize_t bullet_hit_enemy(self, double radius_sq):
    """Fuer alle Entities mit "bullet_tag" und "enemy_tag" pruefe Kollision
    via Distanz-Quadrat. Bei Treffer: Bullet zerstoeren, Enemy bleibt.
    Liefert Anzahl Kollisionen."""
    cdef _Component bullet_pos_x = self.components.get("px")
    cdef _Component bullet_tag = self.components.get("bullet_tag")
    cdef _Component enemy_tag = self.components.get("enemy_tag")
    if bullet_pos_x is None or bullet_tag is None or enemy_tag is None:
        return 0

    cdef _Component pos_y = self.components.get("py")
    if pos_y is None:
        return 0

    # Liste der Bullet-IDs und der Enemy-IDs vorab erstellen (Snapshot,
    # damit destroy() unten den Sparse-Set nicht waehrend Iteration mutiert)
    cdef list bullet_ids = []
    cdef list enemy_ids = []
    cdef Py_ssize_t i, n
    cdef object ent
    n = len(bullet_tag.dense)
    for i in range(n):
        ent = bullet_tag.dense[i]
        if ent in bullet_pos_x.sparse and ent in pos_y.sparse:
            bullet_ids.append(ent)
    n = len(enemy_tag.dense)
    for i in range(n):
        ent = enemy_tag.dense[i]
        if ent in bullet_pos_x.sparse and ent in pos_y.sparse:
            enemy_ids.append(ent)

    # O(N*M) Distanz-Check, alles in C
    cdef Py_ssize_t hits = 0
    cdef Py_ssize_t bi, ei
    cdef double bx, by, ex, ey, dx, dy
    cdef list killed = []
    cdef dict px_sparse = bullet_pos_x.sparse
    cdef dict py_sparse = pos_y.sparse
    cdef list px_vals = bullet_pos_x.values
    cdef list py_vals = pos_y.values
    for bi in range(len(bullet_ids)):
        b_ent = bullet_ids[bi]
        bx = <double>px_vals[<Py_ssize_t>px_sparse[b_ent]]
        by = <double>py_vals[<Py_ssize_t>py_sparse[b_ent]]
        for ei in range(len(enemy_ids)):
            e_ent = enemy_ids[ei]
            ex = <double>px_vals[<Py_ssize_t>px_sparse[e_ent]]
            ey = <double>py_vals[<Py_ssize_t>py_sparse[e_ent]]
            dx = bx - ex
            dy = by - ey
            if dx * dx + dy * dy < radius_sq:
                killed.append(b_ent)
                hits += 1
                break  # Bullet trifft maximal einmal pro Frame
    for ent in killed:
        self.destroy(ent)
    return hits
```

**Key-Insights:**

1. **Snapshot der IDs vorab** — `bullet_ids` und `enemy_ids` werden vor der Iteration gesammelt. Sonst wuerde `self.destroy(b_ent)` den Sparse-Set waehrend der Iteration mutieren.
2. **Lokalisierung der Dict/Listen** als cdef-Variablen (`px_sparse`, `px_vals`) — die Cython-internen Zugriffe werden zu C-Code statt Python-Attribut-Lookup.
3. **`<double>`-Cast** beim Lesen aus der Python-Liste — unboxt einmal in eine C-double, dann sind die Arithmetik-Operationen reine C-Befehle.
4. **`break` nach Hit** — Bullet trifft maximal einmal pro Frame. Bei Mass-Damage-Spielen weglassen.

### 3.3 Python-Fallback

Funktional identisch, aber in Python — fuer Entwickler ohne Cython-Build. In `ecs.py` im Python-`_World`:

```python
def bullet_hit_enemy(self, radius_sq):
    bullet_pos_x = self.components.get("px")
    bullet_tag = self.components.get("bullet_tag")
    enemy_tag = self.components.get("enemy_tag")
    if bullet_pos_x is None or bullet_tag is None or enemy_tag is None:
        return 0
    pos_y = self.components.get("py")
    if pos_y is None:
        return 0
    px_sparse = bullet_pos_x.sparse
    py_sparse = pos_y.sparse
    px_vals = bullet_pos_x.values
    py_vals = pos_y.values

    bullet_ids = [e for e in bullet_tag.dense
                  if e in px_sparse and e in py_sparse]
    enemy_ids = [e for e in enemy_tag.dense
                 if e in px_sparse and e in py_sparse]

    hits = 0
    killed = []
    for b in bullet_ids:
        bx = float(px_vals[px_sparse[b]])
        by = float(py_vals[py_sparse[b]])
        for e in enemy_ids:
            ex = float(px_vals[px_sparse[e]])
            ey = float(py_vals[py_sparse[e]])
            dx = bx - ex
            dy = by - ey
            if dx * dx + dy * dy < radius_sq:
                killed.append(b)
                hits += 1
                break
    for b in killed:
        self.destroy(b)
    return hits
```

### 3.4 @builtin-Wrapper

In `ecs.py`, ans Ende der Datei:

```python
@builtin("ECS_BULLET_HIT_ENEMY", arity=2, types=("any", "num"))
def _b_bullet_hit_enemy(world, radius):
    """Resolver fuer Bullet-vs-Enemy-Kollision via Distanz-Quadrat.
    Zerstoert Bullets die Enemies treffen. Liefert Anzahl Treffer.

    Tipp: radius_sq = radius * radius wird einmal ausgerechnet -- die
    Funktion erwartet das Quadrat, damit du sqrt sparst.
    """
    return _check_world(world, "ECS_BULLET_HIT_ENEMY").bullet_hit_enemy(
        float(radius) * float(radius)
    )
```

`_check_world` ist eine private Helper-Funktion oben in der Datei — sie pruefe, dass das erste Argument wirklich ein ECS_WORLD ist, und liefert es typed zurueck. Die Method-Aufruf-Magie funktioniert: das `_check_world(...)` liefert entweder das cdef-`_World` oder den Pure-Python-Fallback. Beide haben `bullet_hit_enemy`, beide funktionieren identisch.

## Schritt 4: Bauen und benchen

Nach den drei Edits:

```
.venv\Scripts\python.exe setup.py build_ext --inplace
```

Das rekompiliert `ecs_native.pyx`. Wenn dabei Fehler kommen (Type-Mismatch in cdef, fehlende Casts), gibt's eine klare C-Compile-Error-Message — fixen und nochmal.

Bench-File anlegen (`examples/bench_bullet_hit.gb`):

```basic
IMPORT "ecs"

DIM world AS ECS_WORLD
world = ECS_NEW_WORLD()

' 500 Bullets bei zufaelligen Positionen
DIM i AS INTEGER
FOR i = 1 TO 500
    DIM e AS INTEGER
    e = ECS_NEW_ENTITY(world)
    ECS_ADD_FLOAT(world, e, "px", RND(640) + 0.0)
    ECS_ADD_FLOAT(world, e, "py", RND(480) + 0.0)
    ECS_ADD_INT(world, e, "bullet_tag", 1)
NEXT

' 50 Enemies
FOR i = 1 TO 50
    DIM e AS INTEGER
    e = ECS_NEW_ENTITY(world)
    ECS_ADD_FLOAT(world, e, "px", RND(640) + 0.0)
    ECS_ADD_FLOAT(world, e, "py", RND(480) + 0.0)
    ECS_ADD_INT(world, e, "enemy_tag", 1)
NEXT

' 100 Frames Kollision pruefen
DIM frame AS INTEGER
DIM total_hits AS INTEGER
total_hits = 0
FOR frame = 1 TO 100
    total_hits = total_hits + ECS_BULLET_HIT_ENEMY(world, 15.0)
NEXT
PRINT "Hits:", total_hits, "verbleibende Bullets:", ECS_COUNT_WITH(world, "bullet_tag")
```

Run:

```
.venv\Scripts\python.exe gbrun.py --bench examples/bench_bullet_hit.gb
```

Erwartung: **3–8 ms** auf der Native-VM. Die BASIC-Variante (zwei verschachtelte Loops mit ECS_GET) wuerde mehrere **Sekunden** brauchen.

## Schritt 5: Wann *nicht* selbst schreiben

Schreib kein eigenes Bulk-Builtin, wenn:

- **die Operation < 1000-mal pro Frame** ausgefuehrt wird. BASIC-Loop reicht — verstaendlicher, debugbarer.
- **die Operation pro-Aufruf komplex ist** (z.B. ein Tween mit verschachteltem State). Bulk lohnt sich nur, wenn pro Iteration wenig pro Iteration passiert.
- **das Pattern voraussichtlich nochmal aendert.** Built-ins sind harder zu iterieren als BASIC — wer dein Spielprototyp noch nicht stabil ist, lass es in BASIC.

Klassische gute Kandidaten:

- Movement (`INTEGRATE_FLOAT/INT`)
- Kollisions-Resolver (wie hier)
- Lifecycle (`REMOVE_DEAD`, decay-counter)
- Steering (alle Entities zu einem Ziel bewegen)
- Spatial-Hash-Rebuild (jeden Frame Hash neu aufbauen)

## Schritt 6: Vorsicht bei Mutationen waehrend Iteration

Klassischer Bug: in der cdef-Loop wird `destroy` aufgerufen, das den Sparse-Set umbaut, was zu **Out-of-Bounds** in den parallelen `dense`/`values`-Listen fuehrt.

**Falsch:**

```cython
for i in range(len(bullet_tag.dense)):
    ent = bullet_tag.dense[i]
    if hit_condition:
        self.destroy(ent)        # Sparse-Set wird modifiziert!
    # Naechste Iteration kann auf einen verschobenen Eintrag lesen
```

**Richtig:**

```cython
killed = []
for i in range(len(bullet_tag.dense)):
    ent = bullet_tag.dense[i]
    if hit_condition:
        killed.append(ent)
for ent in killed:
    self.destroy(ent)             # erst NACH der Iteration
```

Das war der Grund fuer die "Snapshot der IDs"-Phase oben.

## Schritt 7: Testen

Tests fuer das neue Builtin in `tests/test_modules_ecs.py`:

```python
def test_bullet_hit_enemy_basic(call_builtin, world):
    # Bullet bei (10, 10), Enemy bei (10, 10): direkter Treffer
    b = call_builtin("ecs_new_entity", [world])
    call_builtin("ecs_add_float", [world, b, "px", 10.0])
    call_builtin("ecs_add_float", [world, b, "py", 10.0])
    call_builtin("ecs_add_int", [world, b, "bullet_tag", 1])
    e = call_builtin("ecs_new_entity", [world])
    call_builtin("ecs_add_float", [world, e, "px", 10.0])
    call_builtin("ecs_add_float", [world, e, "py", 10.0])
    call_builtin("ecs_add_int", [world, e, "enemy_tag", 1])
    hits = call_builtin("ecs_bullet_hit_enemy", [world, 5.0])
    assert hits == 1
    # Bullet sollte zerstoert sein
    assert call_builtin("ecs_alive", [world, b]) is False
    # Enemy lebt noch
    assert call_builtin("ecs_alive", [world, e]) is True
```

Plus ein paar Edge-Cases: leere Welt, Bullet ohne Enemy, Enemy ohne Bullet, mehrere Bullets hit denselben Enemy.

## Was du jetzt kannst

Du hast deine eigene Game-Engine-Erweiterung gebaut:

- **Cdef-Method** mit Snapshot-vor-Mutation-Pattern + lokalisiertem List-Access fuer C-Speed
- **Python-Fallback** mit gleicher Signatur fuer Dev-Workflow
- **@builtin-Wrapper** mit klarem Type-Check
- **Bench-File** und **Test** um die Erweiterung zu validieren

Damit kannst du jedes spielspezifische Pattern, das du oft genug brauchst, auf C-Geschwindigkeit bringen. Die Engine bleibt sauber: Standard-Patterns sind im Core (`ECS_INTEGRATE_FLOAT`, `BATCH_DRAW`), spielspezifische Patterns leben in deinem eigenen Modul-Fork oder als zusaetzliche Datei in `gamebasic/modules/`.

## Wo es weiter geht

Damit endet das Buch. Was du noch lernen kannst, lebt nicht mehr im Buch-Format, sondern in:

- **[CLAUDE.md](../CLAUDE.md)** — Architektur-Details der drei VM-Pfade, alle Sprach-Konstrukte
- **[docs/](../docs/README.md)** — vollstaendige Referenz aller Built-ins und Module
- **[docs/PERFORMANCE.md](../docs/PERFORMANCE.md)** — Bench-Zahlen und alle Optimierungen, die in der Engine drin sind
- **Code lesen** — `gamebasic/modules/ecs_native.pyx`, `array_native.pyx`, `vm_native.pyx` sind ueberschaubar (jeweils <500 Zeilen) und der Ort, wo die wirklich heissen Pfade implementiert sind

Viel Erfolg mit deinem naechsten Spiel.
