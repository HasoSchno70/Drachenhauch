# Umstieg aus QBasic, VB, Blitz und Python

Drachenhauch sieht aus wie das BASIC, das man kennt — und ist an einigen
Stellen bewusst anders: jede Variable braucht ein `DIM` mit Typ, es gibt kein
`GOTO`, Felder beginnen bei 0, und Befehle bekommen ihre Werte in Klammern.
Diese Seite sammelt, was man aus anderen Sprachen mitbringt, und wie es hier
heißt. Das meiste davon sagt dir auch der Übersetzer selbst, sobald du es so
schreibst, wie du es gewohnt bist.

Alle Fälle unten sind in `tests/pruef/umstieg.dhtest` geprüft.

## Was einfach geht

Diese Formen aus anderen BASICs versteht Drachenhauch so, wie man sie kennt:

| Du schreibst | Das passiert |
|---|---|
| `DIM hp AS INTEGER = 100` | anlegen und ersten Wert setzen in einer Zeile |
| `EXIT FOR`, `EXIT DO`, `EXIT WHILE` | verlässt die Schleife — aber nur, wenn es die **innerste** ist; sonst ein Fehler statt still die falsche |
| `EXIT SUB` | verlässt das Unterprogramm (in einer FUNCTION: `RETURN wert`) |
| `END` allein auf der Zeile | beendet das Programm, wie `EXIT(0)` |
| `WHILE ... END WHILE` | wie `WHILE ... WEND` |
| `SWAP a, b` | tauscht zwei Variablen (auch Feld-Elemente und Felder von Objekten) |
| `LET x = 5` | das `LET` wird übergangen |
| `? "Hallo"` | Kurzform für `PRINT` |
| `INPUT "Name"; n` | wie in QBasic mit `? ` hinter der Frage; mit `,` ohne |
| `CLS`, `meineSub`, `spieler.springen` | ein Name allein ruft die SUB, den Befehl oder die Methode auf |
| `m["name"] = 5`, `PRINT m["name"]` | MAPs mit eckigen Klammern, gleichwertig zu `MAPPUT`/`MAPGET` |
| `LEN(m)` | Zahl der Einträge einer MAP |
| `a = a + [x]` | zwei Felder zu einem neuen verbinden (gleicher Elementtyp) |
| `DIM a AS ARRAY OF INTEGER` + `ARRAY_PUSH(a, 7)` | ein Feld ohne Größe ist leer und wächst sofort |
| `VAL("3 Äpfel")`, `VAL("&HFF")`, `VAL("1e3")` | die Zahl am **Anfang** des Textes; `&H`/`&O`/`&B` und `0x`/`0b` gehen |
| `RANDOMIZE(TIMER())` | Startwert aus der Uhr |
| `CALL meineSub(1)`, `SUB s(BYVAL x AS INTEGER)` | `CALL` und `BYVAL` werden übergangen (übergeben wird ohnehin als Wert) |
| `WHILE NOT EOF(f)` | `EOF(f)` sagt, ob die Datei zu Ende ist — `READLINE` liefert dort `""` wie bei einer leeren Zeile |
| `READALL$("datei.txt")` | die ganze Datei auf einmal, ohne sie vorher zu öffnen |

## Was anders heißt

| Aus anderen BASICs | In Drachenhauch |
|---|---|
| `a$ = "x"` ohne DIM | `DIM a$ AS STRING` — jede Variable braucht DIM und Typ |
| `DIM a(10)` | `DIM a[10] AS INTEGER` — eckige Klammern, Index 0 bis 9 |
| `DIM SHARED g` | nicht nötig: oben mit `DIM` Angelegtes sehen alle SUBs |
| `x%`, `wert!`, `d#` | kein Typzeichen — `DIM x AS INTEGER` |
| `DOUBLE`, `SINGLE` | `FLOAT` (immer 64 Bit) |
| `LONG`, `SHORT`, `BYTE` | `INTEGER` (immer 64 Bit) |
| `BOOL` | `BOOLEAN` |
| `STRING * 10` | Texte haben keine feste Länge — `LEFT$`/`PADR$` bringen auf Breite |
| `TYPE ... END TYPE` | `STRUCT ... END STRUCT`, Felder mit `DIM` |
| `REDIM a(20)` | Feld ohne Größe + `ARRAY_PUSH`, oder `DIM a[20] AS INTEGER` neu |
| `LBOUND` / `UBOUND` | `0` / `LEN(a) - 1` |
| `GOTO`, `GOSUB`, Zeilennummern | gibt es nicht — Schleifen (`WHILE`, `DO`, `FOR`) und `SUB` |
| `ON ERROR GOTO` | `TRY ... CATCH meldung ... END TRY` |
| `DEF FN q(x) = x * x` | `FUNCTION q(x AS INTEGER) AS INTEGER` … `RETURN x * x` |
| `f = ergebnis` in `FUNCTION f` | `RETURN ergebnis` |
| `ELSE IF` (zwei Wörter), `ENDIF` | `ELSEIF` (ein Wort), `END IF` (zwei) |
| `LOCATE 1, 1`, `COLOR 14`, `SLEEP 100` | Werte in Klammern: `SLEEP(100)` |
| `r = RND`, `t = TIMER` | mit Klammern: `RND()`, `TIMER()` |
| `UCASE$`, `LCASE$`, `STRING$` | `UPPER$`, `LOWER$`, `REPEAT$` |
| `FIX` | `INT` rundet **ab**; zur Null hin `IIF(x < 0, -INT(-x), INT(x))` |
| `CINT`, `CDBL`, `CSTR` | `INT`/`ROUND`, `FLT`, `STR$` |
| `INSTR(start, text, suche)` | `INSTR(text, suche, start)` — Startstelle hinten |
| `MID$(s, 1, 1) = "x"` | Text neu zusammensetzen: `LEFT$(s, i) + neu + MID$(s, i + 1)` |
| `PRINT USING "##.##"; x` | `PRINT FORMAT$(x, "%5.2f")` oder `f"{x:.2f}"` |
| `LINE INPUT s$` | `INPUT s$` liest schon die ganze Zeile |
| `OPEN "f" FOR OUTPUT AS #1` | `f = OPENFILE("f", "w")`, `WRITELINE(f, text)`, `CLOSEFILE(f)` |
| `"a" & "b"` | `"a" + "b"` |
| `a XOR b` | `a BXOR b` (Bits) oder `a <> b` (Wahrheitswerte) |
| `6 AND 3` als Bits | `6 BAND 3` — `AND`/`OR` sind hier logisch (Warnung beim Übersetzen) |
| `IF s = 5` mit `s` als Text, `IF b = 1` mit `b` als BOOLEAN | verschiedene Arten sind nie gleich: `VAL(s) = 5`, `IF b THEN` (Warnung beim Übersetzen) |
| `FOR i = 0 TO 1 STEP 0.5` mit INTEGER `i` | `DIM i AS FLOAT` — eine Kommazahl als Schritt braucht eine FLOAT-Laufvariable; `STEP 0` ist ein Fehler |
| `x = zwei + 1` mit `FUNCTION zwei()` | `zwei() + 1` — ohne Klammern ist eine FUNCTION nur ihr Name |
| `Me.x`, `this.x` | `Self.x` |
| `SUB New()` als Konstruktor | `SUB Init()` |
| `OPTIONAL x AS INTEGER` | `x AS INTEGER = 0` (Vorgabewert) |
| `CATCH e AS STRING` | `CATCH e` — die Meldung ist immer ein STRING |
| `s[0] = "x"` | Texte ändert man nicht stellenweise: `s = "x" + MID$(s, 1)` |
| `SCREEN 12` | `SCREEN(640, 480, "Titel")` — Bildschirmmodi gibt es nicht |
| `LINE (x1, y1)-(x2, y2), farbe` | `LINE(x1, y1, x2, y2, farbe)`; `CIRCLE(x, y, r, farbe)` |
| `PSET`, `COLOR 14` | `PLOT(x, y, farbe)`; die Farbe ist ein Argument jedes Zeichenbefehls |
| `KEYDOWN` | `KEYPRESSED(KEY_A)` (gehalten) bzw. `KEYHIT(KEY_A)` (gerade gedrückt) |
| `IMPORT math` | `IMPORT "json"` in Anführungszeichen, eigene Dateien mit Endung: `IMPORT "helfer.dh"` |
| `CLASS B INHERITS A` | `CLASS B EXTENDS A` |
| `PUBLIC x AS INTEGER` in einer Klasse | `DIM x AS INTEGER` — alles in einer Klasse ist von außen erreichbar |
| ein Ausdruck über zwei Zeilen | die Zeile mit ` _` beenden |
| `TAB`, `SPC` | `PADR$(text, n)`, `SPACE$(3)` |
| `x = meineSub()` | eine SUB liefert keinen Wert — `FUNCTION ... AS <Typ>` mit `RETURN` |
| `TRUNC` | `INT` rundet ab; zur Null hin `IIF(x < 0, -INT(-x), INT(x))` |
| `ZEIT_JAHR` | `ZEIT_TEIL(t, "jahr")` |
| `GUI_WINDOW_NEW` | `GUI_WINDOW(titel$, x, y, breite, höhe)` |
| `ARRAY_REMOVE` | `ARRAY_REMOVE_AT(a, i)` |
| `MAKEDIR` | `MKDIR(pfad$)` — bei jedem unbekannten Befehl schlägt die Meldung den echten Namen vor |
| `MouseDown(1)` für links | `MOUSEBUTTON(0)` — 0 links, 1 rechts, 2 Mitte |
| `KEYHIT("a")` | `KEYHIT(ASC("a"))` oder `KEYHIT(KEY_A)` — eine Taste ist eine Zahl |
| `RGB("FF0000")` | `COLOR_FROM_HEX("#FF0000")`, im Quelltext `&HFF0000` |
| `PLAYSOUND("sprung.wav")` | erst laden: `s = LOADSOUND("sprung.wav")`, dann `PLAYSOUND(s)` (ebenso `DRAWIMAGE`, `SPRITE_NEW` mit `LOADIMAGE`) |

## Das erste Programm mit Grafik

Vier Zeilen, die in QBasic oder Blitz fehlen dürfen und hier nicht:

```basic
SCREEN(320, 200, "Mein Spiel")        ' 1. zuerst das Fenster
WHILE NOT QUITREQUESTED()
    CLS(BLACK)
    CIRCLE(160, 100, 20, YELLOW)
    FLIP()                            ' 2. zeigt, was gezeichnet wurde
WEND
```

- **`SCREEN` kommt vor dem ersten Zeichenbefehl.** Ohne Fenster ist `BOX` ein
  Fehler mit genau diesem Satz — vorher zeichnete es unsichtbar weiter.
- **`FLIP()` zeigt das Bild.** Drachenhauch sammelt die Zeichenbefehle und
  spielt sie beim `FLIP` ab; ohne `FLIP` bleibt das Fenster leer (das Programm
  sagt es am Ende).
- **`WHILE NOT QUITREQUESTED()`** — ohne die Schleife ist das Programm nach dem
  ersten Bild zu Ende und das Fenster schließt sich sofort.
- **Bilder und Klänge werden geladen, nicht benannt:** `b = LOADIMAGE("held.png")`
  und dann `DRAWIMAGE(b, x, y)`.

## Aus Python, C und JavaScript

| Gewohnt | In Drachenhauch |
|---|---|
| `# Kommentar`, `// Kommentar` | `' Kommentar` oder `REM` |
| `a == b`, `a != b` | `a = b`, `a <> b` |
| `7 % 3` | `7 MOD 3` |
| `None`, `null` | `NIL` |
| `double`, `long`, `bool`, `str` | `FLOAT`, `INTEGER`, `BOOLEAN`, `STRING` |
| `list`, `dict` | `ARRAY OF T`, `MAP OF T` |
| `"a\nb"`, `"{\"a\": 1}"` | `!"a\nb"`, `!"{\"a\": 1}"` — Escape-Folgen und Anführungszeichen im Text nur mit `!` davor |
| `x++`, `x += 1` | geht beides |

## Drei Unterschiede, die keine Meldung bekommen

Sie sind Absicht und laufen darum ohne Warnung — man muss sie kennen:

- **`MID$` und `INSTR` zählen ab 0.** `MID$("Hallo", 1, 2)` ist `"al"`, und
  `INSTR` liefert `-1`, wenn nichts gefunden wird (nicht `0`).
- **Felder beginnen bei 0.** `DIM a[3] AS INTEGER` hat `a[0]` bis `a[2]`.
- **`/` kann eine Kommazahl liefern.** `7 / 2` ist `3.5`; für die
  ganzzahlige Division gibt es `\` (`7 \ 2` ist `3`).
- **`=` auf Feldern und MAPs fragt nach DEMSELBEN Behälter**, nicht nach
  gleichem Inhalt: zwei Felder mit denselben Zahlen sind `<>`. Inhalte
  vergleicht man Platz für Platz.
- **`b = a` kopiert ein Feld nicht**, es gibt ihm einen zweiten Namen — wer
  danach `b[0]` schreibt, ändert `a[0]` mit. Dasselbe gilt für MAPs und
  Objekte. Eine echte Kopie liefert `b = a[:]`.
