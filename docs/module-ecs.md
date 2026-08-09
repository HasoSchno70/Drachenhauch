# Modul `ecs`

Entity-Component-System mit Sparse-Set-Storage. Pragmatisch fuer Spiele: Entities sind INTEGER-IDs, Components sind benannte typed Werte. Architektur ist auf Iteration ueber Component-Halter optimiert (Cache-freundlich), nicht auf Reflection oder Type-Hierarchien.

Native Implementation in [`rust/drachenhauch_runtime/src/ecs.rs`](../rust/drachenhauch_runtime/src/ecs.rs) (Sparse-Set + Bulk-System-Ops).

```basic
IMPORT "ecs"
```

## Konzept

- **World** — Container fuer Entities + Components. Du kannst mehrere haben (z.B. eine pro Scene).
- **Entity** — eine INTEGER-ID. Kein Wrapper-Objekt, kein Type.
- **Component** — ein benannter typed Wert (INT, FLOAT, STRING, BOOL, oder beliebig via OBJ), an eine Entity gebunden.

Eine Entity kann beliebig viele Components haben. Pattern fuer Spiele:

```basic
DIM player AS INTEGER
player = ECS_NEW_ENTITY(world)
ECS_ADD_FLOAT(world, player, "px", 100.0)
ECS_ADD_FLOAT(world, player, "py", 100.0)
ECS_ADD_FLOAT(world, player, "vx", 0.0)
ECS_ADD_FLOAT(world, player, "vy", 0.0)
ECS_ADD_INT(world, player, "hp", 100)
ECS_ADD_STRING(world, player, "name", "Hero")
```

Iteration ueber alle "Mover" (Entities mit `px` UND `vx`):

```basic
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

**Aber:** dieser pro-Entity-Loop ist **langsam** (300.000 Builtin-Calls pro Frame bei 500 Entities). Fuer Hot-Path-Systeme nutze die [Bulk-System-Ops](#bulk-system-ops) — typisch **40× schneller**:

```basic
ECS_INTEGRATE_FLOAT(world, "px", "vx")   ' alle Entities mit px+vx
ECS_INTEGRATE_FLOAT(world, "py", "vy")
```

## World-Lifecycle

| Funktion | Rueckgabe / Wirkung |
|---|---|
| `ECS_NEW_WORLD()` | ECS_WORLD — leere Welt |
| `ECS_NEW_ENTITY(w)` | INTEGER — neue Entity-ID (>= 1) |
| `ECS_DESTROY(w, ent)` | BOOLEAN — TRUE wenn entfernt, FALSE wenn nicht existierte |
| `ECS_ALIVE(w, ent)` | BOOLEAN |
| `ECS_COUNT(w)` | INTEGER — Anzahl lebender Entities |

`ECS_DESTROY` entfernt die Entity aus allen Component-Stores (Sparse-Set-Cleanup).

## Component-Add

Pro Typ ein Builtin — der Type-Check passiert beim Add (nicht beim Get).

| Funktion | value-Typ |
|---|---|
| `ECS_ADD_INT(w, ent, name$, value)` | INTEGER |
| `ECS_ADD_FLOAT(w, ent, name$, value)` | FLOAT (int wird akzeptiert + konvertiert) |
| `ECS_ADD_STRING(w, ent, name$, value)` | STRING |
| `ECS_ADD_BOOL(w, ent, name$, value)` | BOOLEAN |
| `ECS_ADD_OBJ(w, ent, name$, value)` | beliebig (User-Klasse, MAP, ARRAY, ...) |

Ist die Entity bereits Halter, wird der Wert ueberschrieben. Component-Name muss als Argument STRING sein (kein Identifier — Components werden zur Laufzeit angelegt).

## Component-Has / Remove

| Funktion | Rueckgabe |
|---|---|
| `ECS_HAS(w, ent, name$)` | BOOLEAN |
| `ECS_REMOVE(w, ent, name$)` | BOOLEAN — TRUE wenn entfernt |

## Component-Get

`ECS_GET_*` wirft, wenn die Entity den Component nicht hat (oder die Entity tot ist). Wer "default wenn nicht da" will: `ECS_GET_OR_*`.

| Funktion | Rueckgabe |
|---|---|
| `ECS_GET_INT(w, ent, name$)` | INTEGER |
| `ECS_GET_FLOAT(w, ent, name$)` | FLOAT |
| `ECS_GET_STRING(w, ent, name$)` | STRING |
| `ECS_GET_BOOL(w, ent, name$)` | BOOLEAN |
| `ECS_GET(w, ent, name$)` | beliebig (ungetypt) |
| `ECS_GET_OR_INT(w, ent, name$, default)` | INTEGER |
| `ECS_GET_OR_FLOAT(w, ent, name$, default)` | FLOAT |
| `ECS_GET_OR_STRING(w, ent, name$, default)` | STRING |
| `ECS_GET_OR_BOOL(w, ent, name$, default)` | BOOLEAN |

`GET_OR_*` liefert `default` auch wenn der Component existiert, aber vom falschen Typ ist (z.B. STRING wo INT erwartet). Das ist defensiv — wer Type-Konflikte als Bug sehen will, nutzt die strikte `GET_*`-Variante.

## Query

Liefert ein `ARRAY OF INTEGER` mit den Entity-IDs, die ALLE genannten Components haben. Ergebnis ist eine Snapshot-Liste — waehrend du darueber iterierst, kannst du Entities zerstoeren / erzeugen, ohne dass die Iteration crasht.

| Funktion | Rueckgabe |
|---|---|
| `ECS_QUERY(w, name$)` | ARRAY OF INTEGER |
| `ECS_QUERY2(w, n1$, n2$)` | ARRAY OF INTEGER — Entities mit n1 UND n2 |
| `ECS_QUERY3(w, n1$, n2$, n3$)` | ARRAY OF INTEGER — Entities mit n1 UND n2 UND n3 |

**Strategie:** der Component mit den wenigsten Haltern wird als Basis gewaehlt (kleinste Iterations-Schleife), dann wird gegen die anderen Sparse-Sets getestet. Bei sparsem Component (z.B. `boss = 1 Entity`) ist Query O(kleinster-Component-Count), nicht O(world-Size).

## Bulk-System-Ops

Klassische ECS-Performance-Falle: ein BASIC-Loop, der pro Entity 6 Builtin-Calls absetzt (`ECS_GET_FLOAT` × 2 + Arithmetik + `ECS_ADD_FLOAT` × 1). Bei 500 Entities × 100 Frames sind das **300.000 Builtin-Calls** — der Hot-Path liegt im Builtin-Dispatch-Overhead, nicht in der eigentlichen Rechnung.

Die Bulk-Ops verarbeiten eine ganze Component-Schicht in EINEM Builtin-Call, intern als cdef-Loop. Drastisch schneller bei vielen Entities (40× ist realistisch).

### Movement-Patterns

| Funktion | Wirkung |
|---|---|
| `ECS_INTEGRATE_FLOAT(w, target$, delta$)` | `target += delta` fuer alle Entities mit beiden Components |
| `ECS_INTEGRATE_INT(w, target$, delta$)` | INT-Variante |

Klassische Anwendung: Position += Velocity.

```basic
ECS_INTEGRATE_FLOAT(world, "px", "vx")
ECS_INTEGRATE_FLOAT(world, "py", "vy")
```

Liefert die Anzahl bewegter Entities (= Schnittmenge).

### Scaling / Reset / Bounds

| Funktion | Wirkung |
|---|---|
| `ECS_SCALE_FLOAT(w, target$, factor)` | `target *= factor` fuer alle Halter |
| `ECS_FILL_FLOAT(w, target$, value)` | alle Werte = value |
| `ECS_FILL_INT(w, target$, value)` | INT-Variante |
| `ECS_CLAMP_FLOAT(w, target$, lo, hi)` | clamp auf `[lo, hi]` |

```basic
ECS_SCALE_FLOAT(world, "vx", 0.95)            ' Friction
ECS_SCALE_FLOAT(world, "vy", 0.95)
ECS_CLAMP_FLOAT(world, "px", 0.0, 640.0)      ' Bounds auf Screen
ECS_CLAMP_FLOAT(world, "py", 0.0, 480.0)
```

### Lifecycle

| Funktion | Wirkung |
|---|---|
| `ECS_REMOVE_DEAD(w, name$, threshold)` | Entities mit `value <= threshold` zerstoeren |
| `ECS_COUNT_WITH(w, name$)` | O(1) — Anzahl Halter eines Components |

```basic
ECS_REMOVE_DEAD(world, "hp", 0)               ' Tote zerstoeren
PRINT "Lebende Bullets:", ECS_COUNT_WITH(world, "px")
```

## Bullet-Hell-Game-Loop Vollbeispiel

```basic
IMPORT "ecs"

DIM world AS ECS_WORLD
world = ECS_NEW_WORLD()

' 1000 Bullets erzeugen
DIM i AS INTEGER
FOR i = 1 TO 1000
    DIM e AS INTEGER
    e = ECS_NEW_ENTITY(world)
    ECS_ADD_FLOAT(world, e, "px", RND(640) + 0.0)
    ECS_ADD_FLOAT(world, e, "py", RND(480) + 0.0)
    ECS_ADD_FLOAT(world, e, "vx", (RND(40) - 20) * 0.5)
    ECS_ADD_FLOAT(world, e, "vy", (RND(40) - 20) * 0.5)
    ECS_ADD_INT(world, e, "hp", 100 + RND(100))
    ECS_ADD_INT(world, e, "regen", -1)
NEXT

' Pro Frame: 8 Bulk-System-Calls statt 6000 pro-Entity-Calls
DIM frame AS INTEGER
FOR frame = 1 TO 200
    ECS_INTEGRATE_FLOAT(world, "px", "vx")
    ECS_INTEGRATE_FLOAT(world, "py", "vy")
    ECS_SCALE_FLOAT(world, "vx", 0.99)
    ECS_SCALE_FLOAT(world, "vy", 0.99)
    ECS_CLAMP_FLOAT(world, "px", 0.0, 640.0)
    ECS_CLAMP_FLOAT(world, "py", 0.0, 480.0)
    ECS_INTEGRATE_INT(world, "hp", "regen")
    ECS_REMOVE_DEAD(world, "hp", 0)
NEXT
```

Auf der Native-VM laeuft das in ~20 ms — 1000 Bullets × 200 Frames × 8 Systeme = **1.6 Mio Entity-Updates**.

Volles Beispiel: [examples/bench_ecs_systems.dh](../examples/bench_ecs_systems.dh).

## Storage-Architektur (Sparse-Set)

Jeder Component hat drei parallele Strukturen:

```
dense:  list[entity_id]      -- kompakte Liste, eine Eintrag je Halter
values: list[any]            -- parallel zu dense, gleicher Index
sparse: dict[entity_id, dense_index]   -- O(1) Lookup
```

- **Iteration** ueber `dense` ist eine flache Liste (cache-freundlicher als Dict-View ueber tausende Entries).
- **GET/HAS** sind ein einzelner Dict-Lookup.
- **ADD/REMOVE** laufen swap-with-last in O(1) — kein Listen-Verschieben.

Die Bulk-Ops nutzen die Sparse-Set-Struktur direkt: sie iterieren ueber den kleineren der beiden Components (bei Integrate) und fragen den anderen Sparse-Set ab.

## Eigene Bulk-Ops hinzufuegen

ECS ist nativ in `dhrt` ([`rust/drachenhauch_runtime/src/ecs.rs`](../rust/drachenhauch_runtime/src/ecs.rs)).
Eine neue Bulk-Op fuegt man so hinzu:

1. **Methode auf `World`** in `ecs.rs` — iteriert in einer Rust-Loop ueber die
   Sparse-Set-Storage (iteriere ueber den kleineren der beiden Components, frage
   den anderen ab).
2. **Builtin-Arm** im ECS-Dispatch (`vm.rs` `try_ecs` bzw. `builtins.rs`) — Arity-
   und Typ-Checks, dann Delegate an die `World`-Methode.
3. **Golden-Test** in `tests/` + Eintrag in `editor_qt/builtin_index.json`.

Beispiel-Skizze fuer ein hypothetisches `ECS_ADD_TO(w, target, source, scale)`:

```rust
pub fn add_to_scaled(&mut self, target: &str, source: &str, scale: f64) {
    let (Some(t), Some(s)) = (self.components.get(target), self.components.get(source))
        else { return; };
    // ... Loop ueber die Schnittmenge: t[e] += s[e] * scale ...
}
```

## Externer Typ

| Typ | Wirkung |
|---|---|
| `ECS_WORLD` | World-Container. `DIM w AS ECS_WORLD` |

Entity-IDs sind INTEGER (kein eigener Typ).
