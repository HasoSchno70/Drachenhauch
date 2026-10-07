# Module `zeit`

Calculating with dates and times. `DATE$()` and `TIME$()` return text — good for
display, useless for calculation. As soon as a program wants to know "15 minutes
before kick-off", "2:15 h to go" or "which weekday is that", it needs
numbers. That is exactly what this module does: text in, number out, calculate,
text back.

```basic
IMPORT "zeit"
```

## The model: a point in time is a number

A point in time is an `INTEGER` — **seconds since 1 January 1970**, in
**local time**. There is nothing more to it, and that is the whole trick:

- **Later** means *greater*. `IF anstoss > jetzt THEN` is the whole comparison.
- **Calculating** means adding. 15 minutes are `15 * 60`, a day is `86400`.
- **No special case.** Month ends, year changes and leap years live in
  the conversion, not in your program.

Local time means: `ZEIT_TEXT$(ZEIT_JETZT())` shows the same time as your
wristwatch, and `DATE$() + " " + TIME$()` returns the same text. When calculating
across a daylight-saving change, the module counts real seconds, not
clock-face hours.

## Text and point in time

| Function | Effect |
|---|---|
| `ZEIT_JETZT()` → INTEGER | now, as a point in time |
| `ZEIT_PARSE(text$)` → INTEGER | text to point in time; aborts on anything unreadable |
| `ZEIT_LESBAR(text$)` → BOOLEAN | could the text be read? (asks, does not abort) |
| `ZEIT_TEXT$(zeit)` → STRING | point in time as `JJJJ-MM-TT hh:mm:ss` (YYYY-MM-DD hh:mm:ss) |
| `ZEIT_AUS_TEILEN(jahr, monat, tag [, stunde, minute, sekunde])` → INTEGER | point in time from individual values |

`ZEIT_PARSE` understands the notations you come across in databases and web
APIs:

```basic
IMPORT "zeit"

PRINT ZEIT_TEXT$(ZEIT_PARSE("2026-08-28 20:30:00"))   ' normal form
PRINT ZEIT_TEXT$(ZEIT_PARSE("2026-08-28T20:30:00"))   ' ISO with T
PRINT ZEIT_TEXT$(ZEIT_PARSE("2026-08-28T20:30:00Z"))  ' with time-zone suffix
PRINT ZEIT_TEXT$(ZEIT_PARSE("2026-08-28 20:30"))      ' without seconds
PRINT ZEIT_TEXT$(ZEIT_PARSE("2026-08-28"))            ' date only = 00:00:00
```

Anything unreadable aborts with a plain message instead of silently returning `-1` — otherwise a
broken date only shows up much later as a nonsensical result. Where input
is uncertain, ask first:

```basic
IMPORT "zeit"

DIM eingabe AS STRING
eingabe = "naechsten Dienstag"
IF ZEIT_LESBAR(eingabe) THEN
    PRINT ZEIT_TEXT$(ZEIT_PARSE(eingabe))
ELSE
    PRINT "Bitte als JJJJ-MM-TT hh:mm eingeben."
END IF
```

`ZEIT_TEXT$` and `ZEIT_PARSE` are inverses of each other. The normal form is
the same one that SQLite sorts correctly as a text column — so points in time
can be written to the database this way and calculated back out again.

## Calculating

| Function | Effect |
|---|---|
| `ZEIT_PLUS(zeit, sekunden)` → INTEGER | add seconds (negative = back) |
| `ZEIT_DIFF(zeit, frueher)` → INTEGER | distance in seconds (negative if `zeit` is earlier) |
| `ZEIT_DAUER$(sekunden)` → STRING | seconds as a readable span |

```basic
IMPORT "zeit"

DIM anstoss AS INTEGER
anstoss = ZEIT_PARSE("2026-08-28 20:30:00")

' betting closes: a quarter of an hour before kick-off
DIM schluss AS INTEGER
schluss = ZEIT_PLUS(anstoss, -15 * 60)
PRINT "Tippschluss: "; ZEIT_TEXT$(schluss)

' Can bets still be placed?
IF ZEIT_JETZT() < schluss THEN
    PRINT "Noch "; ZEIT_DAUER$(ZEIT_DIFF(schluss, ZEIT_JETZT())); " Zeit."
ELSE
    PRINT "Tippschluss ist vorbei."
END IF
```

`ZEIT_DAUER$` chooses the unit by size, so that the display does not drown in
numbers of seconds:

| Seconds | Output |
|---|---|
| `45` | `45 s` |
| `720` | `12 min` |
| `8100` | `2:15 h` |
| `86400` | `1 Tag` |
| `259200` | `3 Tage` |
| `-3600` | `vor 1:00 h` |

Negative values get a `vor` (ago) — so the same function answers both "how
long to go" and "how long ago".

## Reading and displaying parts

| Function | Effect |
|---|---|
| `ZEIT_TEIL(zeit, feld$)` → INTEGER | one field: `jahr`, `monat`, `tag`, `stunde`, `minute`, `sekunde` (year, month, day, hour, minute, second) |
| `ZEIT_WOCHENTAG(zeit)` → INTEGER | 1 = Monday … 7 = Sunday |
| `ZEIT_FORMAT$(zeit, muster$)` → STRING | point in time according to a pattern |

The patterns are written in the same language as the display itself:

| Pattern | Meaning | Example |
|---|---|---|
| `JJJJ` | year, four digits | `2026` |
| `MM` | month, two digits | `08` |
| `TT` | day, two digits | `28` |
| `hh` | hour, two digits | `20` |
| `mm` | minute, two digits | `30` |
| `ss` | second, two digits | `00` |
| `WT` | weekday, short | `Fr` |
| `WTAG` | weekday, written out | `Freitag` |

```basic
IMPORT "zeit"

DIM a AS INTEGER
a = ZEIT_PARSE("2026-08-28 20:30:00")

PRINT ZEIT_FORMAT$(a, "TT.MM.JJJJ")             ' 28.08.2026
PRINT ZEIT_FORMAT$(a, "WT TT.MM. hh:mm")        ' Fr 28.08. 20:30
PRINT ZEIT_FORMAT$(a, "WTAG, TT.MM.JJJJ hh:mm") ' Freitag, 28.08.2026 20:30
PRINT ZEIT_WOCHENTAG(a)                         ' 5
```

Without a pattern (`""`) you get the normal form, exactly as with `ZEIT_TEXT$`.

`ZEIT_WOCHENTAG` counts from Monday, because otherwise a match day at the weekend
would run across two numbers: Monday 1 … Sunday 7.

## Example: countdown in a window

A point in time as a number, redisplayed every second — that is all a
countdown needs.

```basic
IMPORT "zeit"

DIM anstoss AS INTEGER
anstoss = ZEIT_PARSE("2026-08-28 20:30:00")

SCREEN(400, 120, "Anpfiff")
WHILE NOT QUITREQUESTED()
    CLS(RGB(20, 24, 32))
    DIM rest AS INTEGER
    rest = ZEIT_DIFF(anstoss, ZEIT_JETZT())
    IF rest > 0 THEN
        TEXT(20, 30, "Anpfiff in " + ZEIT_DAUER$(rest), RGB(240, 240, 240))
    ELSE
        TEXT(20, 30, "Laeuft seit " + ZEIT_DAUER$(-rest), RGB(120, 220, 140))
    END IF
    TEXT(20, 60, ZEIT_FORMAT$(anstoss, "WTAG, TT.MM.JJJJ hh:mm"), RGB(160, 170, 190))
    FLIP()
WEND
```

## Pitfalls

- **Points in time are local time.** A point in time created on a machine in Berlin
  gives the same *number* on a machine in Tokyo only if both
  read the same text — and the same number shows a different time there.
  If you compare across time zones, store the text, not the number.
- **`MILLIS()` is something else and cannot be converted.** It is the
  stopwatch for frame times and timers. If you compare `MILLIS() / 1000` with
  `ZEIT_JETZT()`, you are off by the time-zone offset —
  measured on 16.08.2026 in Central Europe: `MILLIS()/1000 = 1786883970`,
  `ZEIT_JETZT() = 1786891170`, i.e. **two hours of difference**. For date
  and time always use `ZEIT_JETZT()`.
- **A day is not always 86400 seconds.** On the changeover days it is
  23 or 25 hours. `ZEIT_PLUS(t, 86400)` adds exactly 86400 seconds — on those
  two days of the year that is not the same time on the following day. If the
  time of day matters, go via `ZEIT_TEIL`/`ZEIT_AUS_TEILEN`.
- **Before 1970** points in time are negative. The calculations are still correct.

## In the native runtime (dhrt)

`zeit` is implemented entirely in Rust (`rust/drachenhauch_runtime/src/zeit.rs`),
without any additional dependency and without a Cargo feature — the module is in every
build, including console-only ones. The date ↔ days conversion uses
the civil calendar (leap-year rule including the 100/400 exception).
