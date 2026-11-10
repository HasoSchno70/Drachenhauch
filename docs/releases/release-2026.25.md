# Drachenhauch 2026.25

*Die Notizen zu dieser Fassung. Einen Tag und drei Pull Requests nach
2026.24, alle zu einem Thema: C-Structs können jetzt alles, was eine
C-Bibliothek von ihnen erwartet -- viele davon in einem Feld, als Feld einer
Klasse, in mehreren Dimensionen, und Bitfelder auch gepackt.*

## Neu in dieser Fassung

### Felder von Structs

```basic
STRUCT POINT LAYOUT C
    x AS LONG
    y AS LONG
END STRUCT
DECLARE FUNCTION CreatePolygonRgn LIB "gdi32" (pts AS POINT, n AS LONG, modus AS LONG) AS ZEIGER

DIM dreieck[3] AS POINT
dreieck[1].x = 100
dreieck[2].y = 100
DIM r AS INTEGER
r = CreatePolygonRgn(dreieck, LEN(dreieck), 1)
```

* `DIM pts[n] AS Punkt` legt die Structs **hintereinander in einem Puffer**
  an, wie ein C-Feld -- so geht es als `Punkt*` an eine Bibliothek, und
  `qsort` sortiert an Ort und Stelle. `LEN` zählt die Structs, ein Index
  außerhalb ist ein Fehler, die Größe darf erst zur Laufzeit feststehen.
  `ARRAY OF Punkt` als Parameter oder Rückgabe meint dasselbe.
* **Mehrere Dimensionen** wie in C: `DIM g[n, 4] AS Punkt` liegt
  zeilenweise; die hinteren Größen stehen beim Übersetzen fest, jeder Index
  wird gegen seine eigene geprüft.
* **Ein Element als Ganzes ist eine Kopie** (`p = pts[2]`, `pts[2] = p`,
  auch ein Struct im Struct). An eine Bibliothek geht ein Element dagegen
  als **Zeiger an seine Stelle** -- `GetCursorPos(pts[2])` schreibt ins Feld,
  nicht in eine Kopie.
* **`FOR EACH p IN pts`** liefert je Runde eine Kopie, auch über ein Feld im
  Struct; ein Feld der Kopie zu schreiben ist eine Meldung, weil es ins
  Leere ginge.

### Structs als Feld einer Klasse

```basic
STRUCT Punkt LAYOUT C
    x AS LONG
    y AS LONG
END STRUCT
CLASS Figur
    DIM ort AS Punkt
    DIM ecken[3] AS Punkt
    SUB setze(x AS INTEGER, y AS INTEGER)
        Self.ort.x = x
        ort.y = y
    END SUB
END CLASS
```

Jedes Objekt bekommt bei `NEW` seinen eigenen Puffer; `f.ort` als Ganzes
geht so an eine Bibliothek. Auch ein exportiertes Programm (`.dhc`) kennt
die Größe.

### Bitfelder mit PACK, ohne Namen und mit 0 Bits

```basic
STRUCT Kopf LAYOUT C PACK 1
    art AS UBYTE : 3
    laenge AS LONG : 30
    AS ULONG : 0               ' beendet die Einheit
    AS ULONG : 4               ' Füllbits ohne Namen
    flag AS UBYTE : 1
END STRUCT
```

* Bitfelder gehen jetzt zusammen mit `PACK`, nach den Regeln des Systems:
  MSVC behält unter Windows seine Einheiten, GCC und Clang legen die Bits
  unter Linux und macOS lückenlos, auch über die Grenze ihres Typs hinweg.
* Wie in C: `AS ULONG : 3` ohne Namen sind Füllbits, `AS ULONG : 0` beendet
  die laufende Einheit.
* Linux auf ARM64 hat dafür eine eigene, dritte Regel (Füllbits und Felder
  mit 0 Bits zählen dort zur Ausrichtung des Structs).

## Unter der Haube

**Zahlen.** Es bleibt bei **2109** Befehlen in 48 Modulen und **224**
Beispielen. **4896 Fälle in 373 Prüfsammlungen** (vorher 4885) und
**533 Rust-Testfunktionen** (vorher 530).

* Die Lagen der Bitfelder sind gemessen, nicht erinnert: Clang hat für fünf
  Ziele (Windows, Linux x86-64 und ARM64, macOS ARM und Intel) die Lage von
  32 Structs ausgegeben, elf davon sind mit dem echten gcc und `cl.exe`
  nachgemessen. Der Rust-Test rechnet alle 32 unter allen drei Regeln gegen
  diese Zahlen.
* Ein Bitfeld kann unter `PACK` bei GCC bis zu neun Bytes berühren; Lesen und
  Schreiben gehen dafür über 128 Bit.
* `ARRAY OF Punkt` kommt vom Parser als gewöhnliches Feld an und wird an
  jeder Stelle, die einen Typ an die VM gibt, zum Puffer (`typ_norm`) --
  sonst lehnte die VM den Puffer als „kein ARRAY“ ab.

## Was offen bleibt

* Ein Struct als Wert hinter `...`, ein `va_list` im Rückruf, ein Index
  direkt auf dem Ergebnis eines Aufrufs (`reihe(5)[0].x`, eine Meldung);
  C++ und COM. `VALIST` und die Bitfelder auf Linux-ARM sind nach den Regeln
  gebaut, aber nicht ausprobiert.
* Die Formular-Fälle gegen die Datenbank-Server brauchen ein Fenster und
  sind nur lokal belegt.
* Ein Tray-Symbol unter macOS und Linux.
* Auf einem echten Mac ist das Paket weiter nicht ausprobiert und nicht
  beglaubigt; der Installer ist nicht signiert.
