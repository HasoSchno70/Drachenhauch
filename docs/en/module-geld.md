# Module `geld`

An amount of money as a value of its own (`geld` = money).

```basic
IMPORT "geld"
```

## Why, when `CENT` already exists

[`CENT`, `EURO$` and `ROUND_HALF_UP`](builtins-core.md#calculating-with-money)
are a **way of calculating**: you calculate in whole cents and have to remember to.
The `geld` module turns that into a **type**, and the type remembers for you:

```basic
DIM preis AS GELD
preis = GELD_NEU("19,99")
PRINT preis * 3          ' 59,97 €
PRINT preis + 1.0        ' error -- GELD and decimal numbers do not mix
```

That error message is precisely the difference. Calculated in cents, an
amount is an `INTEGER` like any other; nothing stops you from accidentally
adding a euro value to it. A GELD cannot be mixed with an
ordinary number — for that you have to write `GELD_NEU` or `GELD_ZAHL`,
and then you know what you are doing.

## Overview

| Function | Purpose |
|---|---|
| `GELD_NEU(betrag)` → GELD | from text (`"19,99"`, `"1.234,56"`) or a number |
| `GELD_AUS_CENT(cent)` → GELD | from whole cents |
| `GELD_CENT(g)` → INTEGER | to whole cents, rounded commercially (half up) |
| `GELD_ZAHL(g)` → FLOAT | step out deliberately (display in charts or the like) |
| `GELD_TEXT$(g [, symbol$])` → STRING | `19,99 €`, symbol selectable |
| `GELD_RUNDEN(g [, stellen])` → GELD | commercial rounding, 2 places by default |
| `GELD_TEILEN(g, anzahl)` → ARRAY OF GELD | split up **without losing a cent** |
| `GELD_ABS(g)` → GELD | amount without sign |

In addition, the operators: `+` `-` between two amounts, `*` `/` with a number,
`/` between two amounts (gives a ratio as FLOAT), unary `-`, and
the comparisons `=` `<>` `<` `>` `<=` `>=` — **exact**, not approximate.

## Four decimal places, not two

Inside, a GELD is an `INTEGER` in hundredths of a cent. Four places, because
tax rates and discounts produce intermediate results with more places:

```basic
PRINT GELD_NEU("0,29") * 0.19          ' 0,0551 €
PRINT GELD_RUNDEN(GELD_NEU("0,29") * 0.19)   ' 0,06 €
```

`PRINT` shows the places that are really there. Printing a not yet rounded
intermediate result straight away as "0,06 €" would be convenient and misleading —
you could no longer tell that it has not been rounded yet. **So rounding is
done deliberately**, with `GELD_RUNDEN`, at the point where an amount is
stated.

The value range goes up to around ±922 trillion euros; beyond that it is an
error and not a silently wrapping sum.

## Calculations are integer — even with a factor

`betrag * 0.19` does **not** go via floating point. The factor is split into its
decimal digits, and `wert * 19 / 100` is calculated in whole
numbers. The detour via `FLOAT` would be exactly the inaccuracy this
type is up against.

The same rule as with `CENT`: what is rounded is **the number as written** —
`GELD_NEU(19.99)` gives 19,99 €, not 19,9899…

## Splitting without shrinkage

10,00 € divided by three is not three times 3,33 € — one cent would be left over.

```basic
DIM t AS ARRAY OF GELD
t = GELD_TEILEN(GELD_NEU("10,00"), 3)
PRINT t[0]      ' 3,34 €
PRINT t[1]      ' 3,33 €
PRINT t[2]      ' 3,33 €
```

The first parts get the remainder. Splitting is done in **whole cents** —
nobody can transfer fractions of a cent. By hand you almost always forget this
remainder; that is one of the reasons why a type of its own pays off.

## Limits

* **No currency in the value.** A GELD is an amount, not "19,99 EUR". If you
  calculate with several currencies, keep them apart — the module does not
  help with that.
* **No literal.** `preis = 19.99` does not assign 19,99 € to a GELD variable,
  but is an error; you need `GELD_NEU`. A literal (`19.99g`)
  would have required an intervention in the language core — the trade-off is described in
  [Design: money (German)](../entwurf-geldtyp.md).
* **There is no `GELD * GELD`.** Euros times euros would be square euros.
* **To the outside it becomes a number again.** `xlsx`, `json` and `chart` do not know
  GELD; it goes there as `GELD_CENT` (whole number, lossless) or
  `GELD_ZAHL` (FLOAT, inaccurate again from then on).

## In the native runtime (dhrt)

`rust/drachenhauch_runtime/src/geld.rs`, ungated — a till program should
not need the network built in. The value is a `Value` variant of its own;
operators go through `module_op` in `vm.rs` (as with
`vec2`), and the module shares the splitting of numbers and texts into digits
with `CENT`/`ROUND_HALF_UP` in `builtins.rs` — there is only **one**
rounding rule in the whole house.

Unlike the other module types, GELD is **strict** on assignment:
`DIM x AS GELD : x = 5` is an error. Without that, the separation the type
exists for would be gone straight away.

Example: [examples/180_geld.dh](../../examples/180_geld.dh).
