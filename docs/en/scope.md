# Variable scope

Drachenhauch is Pascal-strict: variables must be declared with `DIM`, keep their type, and visibility follows a clear three-level model without block scoping.

## Three scope levels

```
Globals (Top-Level)
    │
    ├─ Klassen-Felder (nur in Methoden)
    │       │
    │       └─ SUB / FUNCTION lokal
    │
    └─ SUB / FUNCTION lokal
```

Implementation reference: scope resolution in `dhrt` — [compiler.rs](../../rust/drachenhauch_runtime/src/compiler.rs) (slot allocation) + [vm.rs](../../rust/drachenhauch_runtime/src/vm.rs) (locals/globals).

| Level | When | Lifetime |
|---|---|---|
| **Global** | top level outside SUB/FUNCTION/CLASS | the whole program |
| **Class fields** | visible automatically inside a method | as long as the object lives |
| **Local** | inside SUB/FUNCTION/method | until the function ends |

## Block statements do *not* create a new scope

`IF` / `WHILE` / `FOR` / `SELECT` / `TRY` do not create a scope of their own. Whatever you `DIM` there lives on in the enclosing scope.

```basic
IF x > 0 THEN
    DIM hilf AS INTEGER
    hilf = 42
END IF
PRINT hilf                ' 42 - hilf still exists
```

## Lookup rule: local → fields → global

`get_slot()` walks up the parent chain — when reading *and* when writing. That is the decisive difference from Python.

```basic
DIM zaehler AS INTEGER
zaehler = 0

SUB inkrement()
    zaehler = zaehler + 1     ' writes the GLOBAL zaehler
END SUB

inkrement()
PRINT zaehler                 ' 1
```

In Python, the assignment `zaehler = zaehler + 1` would create a *new local* variable (or run into `UnboundLocalError`). In Drachenhauch, a write resolves through the parent chain just like a read — without a `global` keyword.

## Local shadowing

As soon as there is a `DIM x` inside the SUB/FUNCTION, `x` becomes local — reads and writes access the local copy:

```basic
DIM x AS INTEGER
x = 100

SUB demo()
    DIM x AS INTEGER          ' local x - shadows the global one
    x = 5
    PRINT x                   ' 5
END SUB

demo()
PRINT x                       ' 100 - global untouched
```

## Class fields without a prefix

In methods, the instance fields form an intermediate scope between locals and globals. No `self.` / `this.` needed.

```basic
CLASS Spieler
    DIM hp AS INTEGER
    DIM name AS STRING

    SUB Init(n AS STRING, start_hp AS INTEGER)
        name = n               ' directly - no prefix
        hp = start_hp
    END SUB

    SUB heile(menge AS INTEGER)
        hp = hp + menge        ' reads and writes the field
        IF hp > 100 THEN hp = 100
    END SUB
END CLASS
```

If a local parameter had the same name as a field, the parameter would shadow it — the usual solution is different names (above `n` vs `name`).

## Parameters are local, arguments are passed by value/by reference

Every SUB/FUNCTION call creates its own `local_env`. Parameters are declared there and initialized with the argument values.

| Parameter type | Passing |
|---|---|
| `INTEGER`, `FLOAT`, `STRING`, `BOOLEAN` | by value (copy) |
| Arrays | by reference (mutations visible to the caller) |
| Object instances (CLASS) | by reference |

```basic
SUB modify_int(n AS INTEGER)
    n = n + 1                 ' changes only the local copy
END SUB

SUB modify_arr(a AS ARRAY OF INTEGER)
    a[0] = 99                 ' changes the original
END SUB
```

## The FOR variable outlives the loop

`FOR i` declares `i` in the *enclosing* scope (if it does not exist yet). After `NEXT`, `i` keeps the value that ended the loop.

```basic
DIM i AS INTEGER
FOR i = 1 TO 5
    PRINT i
NEXT
PRINT "Nach Loop: ", i        ' 6
```

## The CATCH variable is *not* block-local

Unlike in many languages, the `CATCH e` variable is not limited to the CATCH block — it is created in the enclosing scope and remains reachable after `END TRY`.

```basic
TRY
    PRINT 1 / 0
CATCH msg
    PRINT "Fehler: ", msg
END TRY
PRINT "Letzte Fehler-Message: ", msg     ' works
```

The variable must be of type `STRING` (or not declared yet — then it is created as a STRING).

## DIM is idempotent for the same type

Within the same scope, `DIM x AS INTEGER` is allowed several times as long as the type stays the same. On a type conflict: error.

```basic
DIM x AS INTEGER
DIM x AS INTEGER         ' OK - idempotent, value stays
DIM x AS STRING          ' error: type conflict
```

That is why `DIM` in loop bodies is safe — it is declared again on every iteration, but without losing the value.

## CONST locks the slot

```basic
CONST PI AS FLOAT = 3.14159
PI = 3.0                 ' error: PI is CONST
DIM PI AS FLOAT          ' error: cannot be declared again
```

`FOR i = 1 TO 10` throws as well if `i` was declared as CONST.

## No closures, no nested SUBs

Drachenhauch does not allow a SUB/FUNCTION inside a SUB/FUNCTION. Every routine sees its own local scope + (for methods) fields + globals — that's it. There is no "outer" local scope that could be closed over.

## Quick Reference

| Situation | What happens |
|---|---|
| `x = 5` in a SUB without a local `DIM x` | writes the global `x` (or the field in a method) |
| `DIM x AS INT` in a SUB | creates a local `x`, shadows |
| `DIM x` in IF/WHILE/FOR | ends up in the enclosing SUB/top-level scope |
| FOR `i` without a previous DIM | is declared automatically (INTEGER, or FLOAT for FLOAT bounds) |
| reading FOR `i` after the loop | allowed, holds the end value |
| `CATCH e` | `e` stays reachable in the enclosing scope |
| parameter `p` | by value for scalars, by reference for arrays/objects |
| `DIM x AS INTEGER` twice with the same type | idempotent (no error, no value reset) |
| `DIM x AS INTEGER` then `DIM x AS STRING` | type conflict error |
