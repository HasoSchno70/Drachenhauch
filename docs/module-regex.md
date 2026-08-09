# Modul `regex`

Python-kompatibles Regex-Matching mit Pattern-Cache. Praktisch fuer Text-Verarbeitung, Score-Parser, Save-File-Migration, Chat-Filter, Highscore-Listen, alles wo `INSTR` zu schwach wird.

```basic
IMPORT "regex"
```

## Übersicht

| Funktion | Rueckgabe | Wirkung |
|---|---|---|
| `REGEX_MATCH(text, pattern)` | BOOLEAN | Voller Match (Pattern muss ganzen Text abdecken) |
| `REGEX_TEST(text, pattern)` | BOOLEAN | Sucht Vorkommen irgendwo im Text |
| `REGEX_FIND(text, pattern)` | STRING | Erster Treffer (leerer String wenn nicht gefunden) |
| `REGEX_FIND_ALL(text, pattern)` | ARRAY OF STRING | Alle nicht-ueberlappenden Treffer |
| `REGEX_REPLACE(text, pattern, repl)` | STRING | Ersetzt alle Treffer |
| `REGEX_REPLACE_ONCE(text, pattern, repl)` | STRING | Ersetzt nur den ersten Treffer |
| `REGEX_SPLIT(text, pattern)` | ARRAY OF STRING | Splittet an jedem Treffer |

## Pattern-Syntax

Die Patterns sind **Python-Regex** (`re`-Modul). Die wichtigsten Bausteine:

| Pattern | Bedeutung |
|---|---|
| `.` | beliebiges einzelnes Zeichen (ausser Newline) |
| `\d` `\D` | Ziffer / Nicht-Ziffer |
| `\w` `\W` | Wort-Zeichen (a-z A-Z 0-9 _) / Nicht-Wort |
| `\s` `\S` | Whitespace / Nicht-Whitespace |
| `\b` | Wort-Grenze |
| `[abc]` | Eins von a, b, c |
| `[^abc]` | KEINS von a, b, c |
| `[a-z]` | Bereich |
| `*` `+` `?` | 0+, 1+, 0-oder-1 Wiederholungen (greedy) |
| `*?` `+?` | Wiederholung, lazy/non-greedy |
| `{n}` `{n,m}` | Exakt n / zwischen n und m Wiederholungen |
| `\|` | Alternative (a\|b = a oder b) |
| `(...)` | Capture-Gruppe |
| `^` `$` | Anfang / Ende |

**Achtung:** Backslashes in BASIC-Strings sind keine Escape-Sequenzen. `"\d+"` ist im Pattern korrekt; man braucht nicht `"\\d+"`.

## Match vs. Test vs. Find

```basic
PRINT REGEX_MATCH("123",   "\d+")          ' TRUE  -- ganzer Text matcht
PRINT REGEX_MATCH("123 hi", "\d+")         ' FALSE -- "hi" matcht nicht mit
PRINT REGEX_TEST("123 hi", "\d+")          ' TRUE  -- "123" ist drin
PRINT REGEX_FIND("123 hi 456", "\d+")      ' "123" -- erster Treffer
```

`REGEX_FIND` liefert leeren String wenn nichts gefunden — kein NIL, daher kein NIL-Check noetig:

```basic
DIM number AS STRING
number = REGEX_FIND(input_line, "\d+")
IF number <> "" THEN
    PRINT "Gefunden:", number
END IF
```

## Alle Treffer als Array

`REGEX_FIND_ALL` liefert ein `ARRAY OF STRING` mit allen Treffern:

```basic
DIM nums AS ARRAY OF STRING
nums = REGEX_FIND_ALL("Hp: 80, Mp: 30, Lv: 5", "\d+")
DIM i AS INTEGER
FOR i = 0 TO LEN(nums) - 1
    PRINT nums[i]              ' "80", "30", "5"
NEXT
```

**Capture-Gruppen:** Hat das Pattern Klammern, wird die ERSTE Gruppe extrahiert (nicht das ganze Match). Beispiel:

```basic
DIM ips AS ARRAY OF STRING
ips = REGEX_FIND_ALL("ip 10.0.0.1, fw 192.168.1.1", "ip (\d+\.\d+\.\d+\.\d+)")
PRINT ips[0]                   ' "10.0.0.1" (Capture-Gruppe, nicht "ip 10.0.0.1")
```

## Replace

`REGEX_REPLACE` ersetzt alle Treffer durch `repl`. Backslash-References fuer Capture-Gruppen:

```basic
PRINT REGEX_REPLACE("Hello WORLD", "WORLD", "GameBasic")
' "Hello GameBasic"

' Swap mit Capture-Gruppen
PRINT REGEX_REPLACE("Anna 30, Bob 25", "(\w+) (\d+)", "\2 (\1)")
' "30 (Anna), 25 (Bob)"
```

`REGEX_REPLACE_ONCE` ersetzt nur das erste Vorkommen — praktisch fuer "fixiere den ersten Bug, lass den Rest":

```basic
PRINT REGEX_REPLACE_ONCE("ha ha ha", "ha", "OK")
' "OK ha ha"
```

## Split

`REGEX_SPLIT` ist `SPLIT$` auf Steroiden — das Trennmuster ist eine Regex:

```basic
' Splitting auf beliebigem Whitespace (Leerzeichen, Tab, Newline gemischt):
DIM parts AS ARRAY OF STRING
parts = REGEX_SPLIT("foo   bar\tbaz", "\s+")
PRINT LEN(parts)               ' 3
PRINT parts[0]; "|"; parts[1]; "|"; parts[2]
' "foo|bar|baz"

' CSV mit optionalem Whitespace ums Komma:
parts = REGEX_SPLIT("a, b , c,d", "\s*,\s*")
' ["a", "b", "c", "d"]
```

## Performance: Pattern-Cache

`regex` kompiliert jedes Pattern einmal und cacht das Ergebnis. Wenn du dasselbe Pattern in einer Schleife verwendest, wird es nicht jedes Mal neu kompiliert.

```basic
' Diese 1000 Iterationen kompilieren das Pattern EINMAL:
DIM i AS INTEGER
FOR i = 0 TO 999
    IF REGEX_TEST(lines[i], "^\d+:") THEN ...
NEXT
```

## Praktische Patterns

**Score aus Highscore-Zeile parsen:**

```basic
DIM line AS STRING
line = "  3. Bob ........... 12500 pts"
DIM score AS STRING
score = REGEX_FIND(line, "\d+(?= pts)")   ' positive lookahead
PRINT score                                ' "12500"
```

**Chat-Wort-Filter:**

```basic
DIM clean AS STRING
clean = REGEX_REPLACE(user_message, "(damn|hell)", "***")
```

**Datums-Format pruefen:**

```basic
IF REGEX_MATCH(save_date, "\d{4}-\d{2}-\d{2}") THEN
    ' YYYY-MM-DD korrekt
END IF
```

## Edge-Cases

- **Pattern ungueltig:** Wenn das Pattern Regex-Sicht ungueltig ist (z.B. unbalanced Klammern), wirft `regex` einen `GBRuntimeError` mit der Python-Fehlermeldung.
- **Leerer Text:** `REGEX_FIND("", ".*")` matched (das `.*` matcht den leeren String). `REGEX_FIND("", "x")` liefert `""`.
- **REGEX_SPLIT mit Pattern, das den Anfang matcht:** liefert leeren ersten String. Standard-Python-Verhalten.

## In der nativen Runtime (dhrt)

`regex` laeuft auch nativ (`gbrun.py --native`, Standalone-`.exe`) — immer dabei (kein Feature-Flag, nutzt die Rust-`regex`-Crate). Bit-identisch zu den Python-Pfaden fuer die ueblichen Patterns (Zeichenklassen, Anker, Quantoren, Gruppen, Alternation). **Nicht unterstuetzt** (Rust-`regex`-Limit): Backreferences (`\1`) *im Pattern* sowie Lookahead/Lookbehind. In `REGEX_REPLACE` werden Python-Backrefs (`\1`, `\g<name>`) automatisch in die Rust-Syntax uebersetzt.
